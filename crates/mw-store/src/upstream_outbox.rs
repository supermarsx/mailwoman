//! The durable per-account upstream outbox (t62-n6a; plan
//! `t62-n6-upstream-outbox.md` §2, §3.3, §3.4, §5.2).
//!
//! Used only with `MW_UPSTREAM_ACK=local`: a flag change, a move or a draft
//! copy is acknowledged after the LOCAL commit and applied to the user's mail
//! server later. Nothing in the default mode (`upstream`) calls this module,
//! so with the mode off no statement here runs and no row is written.
//!
//! # Model
//! `upstream_outbox` holds INTENTS, one row per (account, message, field):
//! `flag:<json flag>` (want `'1'`/`'0'`), `location` (want = destination
//! mailbox id; `base_*` = the server coordinates the message had before the
//! first unsent move), `append` (want = mailbox id of the draft's folder,
//! `NULL` once the draft was discarded while the APPEND was in flight) and
//! `expunge_copy` (`base_*` = the server copy to remove). A later change of the
//! same field overwrites the row; a change back to the server's value deletes
//! it ([`plan_value`], the table of §3.3).
//!
//! # Transactions and lock order
//! Every mutation here is ONE transaction holding the CAS on `messages`, the
//! mailbox counters, the Email/Mailbox change rows, the intent rows and the
//! head row. On PostgreSQL each begins with the account's advisory lock
//! (719304, the change-log and ingest lock), then `accounts FOR KEY SHARE`,
//! then the mailboxes `FOR UPDATE` in id order, then the messages `FOR
//! UPDATE`, and only then writes (the t62-n15 rule: nothing is locked after
//! the first write to a trigger-maintained row). Outbox rows are only written
//! under the advisory lock, so an account's `seq` order equals commit order.
//! On SQLite the write job (one savepoint of the group writer, or its own
//! transaction) serialises all of it.
//!
//! # Server coordinates
//! A message moved in local mode gets `uidvalidity = 0` and a pseudo-UID that
//! is unique in its new folder; its server coordinates live in the intent's
//! `base_*` columns. Zero already means "nothing upstream to address", so no
//! existing path sends a UID command for such a row.

use std::collections::{BTreeMap, BTreeSet, HashMap};

use crate::backend::{Row, Tx, q};
use crate::{Message, MessageUpsert, Store, StoreError};

/// Default backlog cap per account (plan §2.3; a setting between
/// [`MIN_BACKLOG_CAP`] and [`MAX_BACKLOG_CAP`]).
pub const DEFAULT_BACKLOG_CAP: i64 = 10_000;
/// Lowest accepted backlog cap.
pub const MIN_BACKLOG_CAP: i64 = 1_000;
/// Highest accepted backlog cap.
pub const MAX_BACKLOG_CAP: i64 = 100_000;

/// Field of a move intent.
pub const FIELD_LOCATION: &str = "location";
/// Field of a draft APPEND intent.
pub const FIELD_APPEND: &str = "append";
/// Field of the removal of a draft's server copy.
pub const FIELD_EXPUNGE_COPY: &str = "expunge_copy";
/// Shadow value recorded by sync for a message that vanished from the server.
pub const SHADOW_GONE: &str = "gone";

const SEAL_PURPOSE: &str = "mw.upstream_outbox.error_detail.v1";
const PSEUDO_UID_PROBES: u32 = 64;

/// The field name of one flag: `flag:` + the flag's JSON element as stored in
/// `flags_json` (`"Seen"`, `{"Keyword":"$Junk"}`).
pub fn flag_field(flag_json: &str) -> String {
    format!("flag:{flag_json}")
}

/// The JSON element of a `flag:` field, if it is one.
pub fn flag_of_field(field: &str) -> Option<&str> {
    field.strip_prefix("flag:")
}

/// Who asked for the change.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IntentOrigin {
    User,
    Rule,
    Sieve,
    Spam,
    System,
}

impl IntentOrigin {
    pub fn as_str(self) -> &'static str {
        match self {
            IntentOrigin::User => "user",
            IntentOrigin::Rule => "rule",
            IntentOrigin::Sieve => "sieve",
            IntentOrigin::Spam => "spam",
            IntentOrigin::System => "system",
        }
    }
    fn parse(value: &str) -> Result<Self, StoreError> {
        Ok(match value {
            "user" => IntentOrigin::User,
            "rule" => IntentOrigin::Rule,
            "sieve" => IntentOrigin::Sieve,
            "spam" => IntentOrigin::Spam,
            "system" => IntentOrigin::System,
            other => return Err(StoreError::Corrupt(format!("upstream_outbox origin {other:?}"))),
        })
    }
}

/// Lifecycle state of an intent row.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IntentState {
    /// Waiting to be sent.
    Pending,
    /// Claimed by the applier with a token; a command may be on the wire.
    Inflight,
    /// A move or APPEND whose outcome is not known; recovery decides (§3.4).
    Unresolved,
}

impl IntentState {
    pub fn as_str(self) -> &'static str {
        match self {
            IntentState::Pending => "pending",
            IntentState::Inflight => "inflight",
            IntentState::Unresolved => "unresolved",
        }
    }
    fn parse(value: &str) -> Result<Self, StoreError> {
        Ok(match value {
            "pending" => IntentState::Pending,
            "inflight" => IntentState::Inflight,
            "unresolved" => IntentState::Unresolved,
            other => return Err(StoreError::Corrupt(format!("upstream_outbox state {other:?}"))),
        })
    }
}

/// Server coordinates of one message.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ServerCoords {
    pub mailbox_id: String,
    pub uidvalidity: u32,
    pub uid: u32,
}

/// One intent row (`error_detail` stays sealed; see [`Store::upstream_error_detail`]).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OutboxIntent {
    pub account_id: String,
    pub stable_id: String,
    pub field: String,
    pub seq: i64,
    pub first_at: String,
    pub updated_at: String,
    pub want: Option<String>,
    pub sent: Option<String>,
    pub base: Option<String>,
    pub shadow: Option<String>,
    pub base_location: Option<ServerCoords>,
    pub state: IntentState,
    pub claim: Option<String>,
    pub attempts: i64,
    pub next_at: Option<String>,
    pub error_class: Option<String>,
    pub has_error_detail: bool,
    pub origin: IntentOrigin,
    pub upstream_gen: i64,
}

/// The account's head row.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OutboxHead {
    pub next_seq: i64,
    pub backlog: i64,
    pub upstream_gen: i64,
    /// `None`, `"auth"` or `"operator"`.
    pub paused: Option<String>,
    pub paused_at: Option<String>,
}

/// What a mutation did to the intent rows.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct IntentTally {
    /// New rows.
    pub created: u32,
    /// Rows whose wanted value changed.
    pub overwritten: u32,
    /// Rows deleted because the wanted value is the server's again (nothing is sent).
    pub cancelled: u32,
}

/// Result of an outbox mutation. `AtCap`: the account's backlog would pass
/// the cap; nothing was written (the caller falls back as plan §2.3 says).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum OutboxWrite<T> {
    Done(T),
    AtCap { backlog: i64, cap: i64 },
}

impl<T> OutboxWrite<T> {
    /// The result, or `None` at the cap.
    pub fn done(self) -> Option<T> {
        match self {
            OutboxWrite::Done(value) => Some(value),
            OutboxWrite::AtCap { .. } => None,
        }
    }
}

/// [`Store::set_flags_with_intents`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FlagsWritten {
    /// Per input pair: whether that id of this account was written.
    pub written: Vec<bool>,
    /// The new Email state, `None` when nothing was written.
    pub email_state: Option<u64>,
    pub tally: IntentTally,
}

/// [`Store::relocate_with_intent`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Relocated {
    /// `false`: the source changed (nothing written).
    pub moved: bool,
    /// The row's new coordinates (`uidvalidity` 0 unless a pending move was cancelled).
    pub uidvalidity: u32,
    pub uid: u32,
    pub email_state: Option<u64>,
    pub tally: IntentTally,
}

/// [`Store::ingest_draft_with_append_intent`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DraftIngested {
    pub stable_id: String,
    pub existed: bool,
    pub tally: IntentTally,
}

/// [`Store::delete_draft_with_copy_intent`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DraftDeleted {
    /// `false`: the source changed (nothing written).
    pub deleted: bool,
    pub email_state: Option<u64>,
    pub tally: IntentTally,
}

/// One row handed to the applier by [`Store::claim_upstream_intents`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClaimedIntent {
    pub intent: OutboxIntent,
    /// The state before the claim: `Unresolved` means recovery (§3.4) runs
    /// before anything is resent; `sent` then still holds the earlier value.
    pub was: IntentState,
}

/// [`Store::claim_upstream_intents`].
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ClaimOutcome {
    pub claimed: Vec<ClaimedIntent>,
    /// Pending rows whose wanted value equals the server's (`shadow`):
    /// deleted without sending (§3.3 row 4).
    pub cancelled: Vec<(String, String)>,
    /// Requested keys not claimed (changed, claimed elsewhere, or gone).
    pub skipped: Vec<(String, String)>,
    /// The account is paused: nothing was claimed.
    pub paused: Option<String>,
}

/// What the server answered for one claimed intent.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Confirmation {
    /// A flag STORE or a copy expunge was acknowledged.
    Applied,
    /// A move was acknowledged; the message is now at (`sent` mailbox, uidvalidity, uid).
    Moved { uidvalidity: u32, uid: u32 },
    /// An APPEND was acknowledged with APPENDUID (`sent` mailbox, uidvalidity, uid).
    Appended { uidvalidity: u32, uid: u32 },
}

/// The outcome of one confirmation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Confirmed {
    /// Applied; the intent is gone.
    Done,
    /// The value changed while in flight: the row is pending again with
    /// `base := sent` (or deleted when the new want equals that base).
    Requeued,
    /// The draft was discarded while its APPEND was in flight: the append
    /// intent became an `expunge_copy` of the new copy.
    ExpungeQueued,
    /// The new coordinates are already taken by another row or intent: the
    /// intent is `unresolved` (error class `collision`) for recovery.
    Collision,
    /// The row is not in flight under this token (nothing changed).
    Stale,
}

/// [`Store::reject_and_revert`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Reverted {
    /// The local value went back to shadow-else-base; a notice was written.
    Done { email_state: Option<u64>, notice_id: String },
    /// No such intent.
    Missing,
    /// A token was given and the row is not in flight under it.
    Stale,
}

/// One row of `upstream_notice`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UpstreamNotice {
    pub id: String,
    pub account_id: String,
    pub at: String,
    pub stable_id: Option<String>,
    pub op: String,
    pub reason: String,
    pub count: i64,
    pub seen_at: Option<String>,
}

/// What to do with one (message, field) given its row (§2.2 / §3.3).
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum ValuePlan {
    /// No row and the value differs from the server's: insert with this base.
    Insert,
    /// Pending row, new want: overwrite want (new seq).
    Overwrite,
    /// Pending row and the new want is the server's value again: delete.
    Cancel,
    /// In flight or unresolved: overwrite want only; resolved at acknowledgement.
    OverwriteInFlight,
    /// No row and nothing to do (the value is already the server's).
    Nothing,
}

/// The upsert rule for a scalar field (flags): `server_now` is the local value
/// before this change when there is no row (then it is the base).
pub(crate) fn plan_value(existing: Option<&OutboxIntent>, want: &str, server_now: &str) -> ValuePlan {
    match existing {
        None if want == server_now => ValuePlan::Nothing,
        None => ValuePlan::Insert,
        Some(row) => match row.state {
            IntentState::Pending => {
                let server = row.shadow.as_deref().or(row.base.as_deref());
                if server == Some(want) {
                    ValuePlan::Cancel
                } else if row.want.as_deref() == Some(want) {
                    ValuePlan::Nothing
                } else {
                    ValuePlan::Overwrite
                }
            }
            IntentState::Inflight | IntentState::Unresolved => {
                if row.want.as_deref() == Some(want) {
                    ValuePlan::Nothing
                } else {
                    ValuePlan::OverwriteInFlight
                }
            }
        },
    }
}

/// The intent columns every read selects (a literal, for `concat!`).
macro_rules! intent_columns {
    () => {
        "account_id, stable_id, field, seq, first_at, updated_at, want, sent, base, shadow, base_mailbox_id, base_uidvalidity, base_uid, state, claim, attempts, next_at, error_class, CAST(CASE WHEN error_detail IS NULL THEN 0 ELSE 1 END AS BIGINT) AS has_detail, origin, upstream_gen"
    };
}

fn intent_from_row(r: &Row) -> Result<OutboxIntent, StoreError> {
    let base_location = match (r.get_opt_string("base_mailbox_id"), r.get_opt_i64("base_uidvalidity"), r.get_opt_i64("base_uid")) {
        (Some(mailbox_id), Some(uidvalidity), Some(uid)) => Some(ServerCoords {
            mailbox_id,
            uidvalidity: u32::try_from(uidvalidity).map_err(|_| StoreError::Corrupt("upstream_outbox base_uidvalidity".into()))?,
            uid: u32::try_from(uid).map_err(|_| StoreError::Corrupt("upstream_outbox base_uid".into()))?,
        }),
        _ => None,
    };
    Ok(OutboxIntent {
        account_id: r.get_string("account_id"),
        stable_id: r.get_string("stable_id"),
        field: r.get_string("field"),
        seq: r.get_i64("seq"),
        first_at: r.get_string("first_at"),
        updated_at: r.get_string("updated_at"),
        want: r.get_opt_string("want"),
        sent: r.get_opt_string("sent"),
        base: r.get_opt_string("base"),
        shadow: r.get_opt_string("shadow"),
        base_location,
        state: IntentState::parse(&r.get_string("state"))?,
        claim: r.get_opt_string("claim"),
        attempts: r.get_i64("attempts"),
        next_at: r.get_opt_string("next_at"),
        error_class: r.get_opt_string("error_class"),
        has_error_detail: r.get_i64("has_detail") == 1,
        origin: IntentOrigin::parse(&r.get_string("origin"))?,
        upstream_gen: r.get_i64("upstream_gen"),
    })
}

fn is_pg(tx: &Tx) -> bool {
    matches!(tx, Tx::Postgres(_))
}

fn now() -> String {
    chrono::Utc::now().to_rfc3339()
}

fn ids_json(ids: &[String]) -> String {
    crate::cache::id_list_json(ids)
}

/// The canonical JSON of every element of a `flags_json` array (empty when it
/// does not parse, as `flags_json_is_unread` treats it).
fn flag_elements(flags_json: &str) -> Vec<serde_json::Value> {
    serde_json::from_str::<Vec<serde_json::Value>>(flags_json).unwrap_or_default()
}

fn flag_keys(flags_json: &str) -> BTreeSet<String> {
    flag_elements(flags_json)
        .iter()
        .filter_map(|v| serde_json::to_string(v).ok())
        .collect()
}

/// `flags_json` with one flag (by its JSON key) present or absent; order of
/// the other elements kept, an added flag goes last.
fn with_flag(flags_json: &str, key: &str, present: bool) -> Result<String, StoreError> {
    let mut elements = flag_elements(flags_json);
    let value: serde_json::Value =
        serde_json::from_str(key).map_err(|_| StoreError::Corrupt("upstream_outbox flag field".into()))?;
    let has = elements.iter().any(|v| v == &value);
    if present && !has {
        elements.push(value);
    } else if !present && has {
        elements.retain(|v| v != &value);
    }
    Ok(serde_json::to_string(&elements).expect("flags always serialize"))
}

fn bit(present: bool) -> &'static str {
    if present { "1" } else { "0" }
}

fn pseudo_uid(stable_id: &str, probe: u32) -> u32 {
    use sha2::Digest;
    let digest = sha2::Sha256::digest(stable_id.as_bytes());
    let seed = u32::from_be_bytes([digest[0], digest[1], digest[2], digest[3]]) | 1;
    seed.wrapping_add(probe.wrapping_mul(2)) | 1
}

/// Advisory account lock, then the account row (PostgreSQL; SQLite: no-op).
async fn lock_account(tx: &mut Tx, account_id: &str) -> Result<(), StoreError> {
    crate::background_maintenance::lock_body_account(tx, account_id).await?;
    crate::header_envelope::lock_parent(tx, account_id).await
}

async fn read_head(tx: &mut Tx, account_id: &str) -> Result<Option<OutboxHead>, StoreError> {
    let sql = if is_pg(tx) {
        "SELECT next_seq, backlog, upstream_gen, paused, paused_at FROM upstream_outbox_head WHERE account_id=?1 FOR UPDATE"
    } else {
        "SELECT next_seq, backlog, upstream_gen, paused, paused_at FROM upstream_outbox_head WHERE account_id=?1"
    };
    let rows = q(sql).bind(account_id).fetch_all_tx(tx).await?;
    Ok(rows.first().map(|r| OutboxHead {
        next_seq: r.get_i64("next_seq"),
        backlog: r.get_i64("backlog"),
        upstream_gen: r.get_i64("upstream_gen"),
        paused: r.get_opt_string("paused"),
        paused_at: r.get_opt_string("paused_at"),
    }))
}

fn default_head() -> OutboxHead {
    OutboxHead { next_seq: 1, backlog: 0, upstream_gen: 0, paused: None, paused_at: None }
}

/// Write `next_seq` and `backlog` back (one upsert statement).
async fn write_head(tx: &mut Tx, account_id: &str, head: &OutboxHead) -> Result<(), StoreError> {
    q("INSERT INTO upstream_outbox_head (account_id, next_seq, backlog, upstream_gen) VALUES (?1, ?2, ?3, ?4)
       ON CONFLICT (account_id) DO UPDATE SET next_seq=excluded.next_seq, backlog=excluded.backlog")
        .bind(account_id)
        .bind(head.next_seq)
        .bind(head.backlog)
        .bind(head.upstream_gen)
        .execute_tx(tx)
        .await?;
    Ok(())
}

async fn read_intents(tx: &mut Tx, account_id: &str, stable_ids: &[String]) -> Result<Vec<OutboxIntent>, StoreError> {
    if stable_ids.is_empty() {
        return Ok(Vec::new());
    }
    let sql = if is_pg(tx) {
        concat!("SELECT ", intent_columns!(), " FROM upstream_outbox WHERE account_id=?1 AND stable_id IN (SELECT value FROM json_array_elements_text(?2::json)) ORDER BY stable_id, field FOR UPDATE")
    } else {
        concat!("SELECT ", intent_columns!(), " FROM upstream_outbox WHERE account_id=?1 AND stable_id IN (SELECT value FROM json_each(?2)) ORDER BY stable_id, field")
    };
    let rows = q(sql).bind(account_id).bind(ids_json(stable_ids)).fetch_all_tx(tx).await?;
    rows.iter().map(intent_from_row).collect()
}

async fn read_intent(tx: &mut Tx, account_id: &str, stable_id: &str, field: &str) -> Result<Option<OutboxIntent>, StoreError> {
    let sql = if is_pg(tx) {
        concat!("SELECT ", intent_columns!(), " FROM upstream_outbox WHERE account_id=?1 AND stable_id=?2 AND field=?3 FOR UPDATE")
    } else {
        concat!("SELECT ", intent_columns!(), " FROM upstream_outbox WHERE account_id=?1 AND stable_id=?2 AND field=?3")
    };
    let rows = q(sql).bind(account_id).bind(stable_id).bind(field).fetch_all_tx(tx).await?;
    rows.first().map(intent_from_row).transpose()
}

/// The `record_changes` INSERT: N rows at ONE new state, inside the caller's
/// transaction (the caller holds the change-log serialisation: SQLite's single
/// writer, PostgreSQL's advisory lock 719304).
async fn record_changes_tx(tx: &mut Tx, account_id: &str, kind: &str, ids: &[String], op: &str) -> Result<Option<u64>, StoreError> {
    if ids.is_empty() {
        return Ok(None);
    }
    let sql = if is_pg(tx) {
        "INSERT INTO changes (account_id, type, state, stable_id, op, at)
         SELECT ?1, ?2, s.next, j.value, ?4, ?5
         FROM (SELECT COALESCE(MAX(state), 0) + 1 AS next FROM (
               SELECT MAX(state) AS state FROM changes WHERE account_id=?1 AND type=?2
               UNION ALL SELECT head_state AS state FROM change_log_checkpoint WHERE account_id=?1 AND type=?2
               ) heads) s,
              json_array_elements_text(?3::json) j
         RETURNING state"
    } else {
        "INSERT INTO changes (account_id, type, state, stable_id, op, at)
         SELECT ?1, ?2, s.next, j.value, ?4, ?5
         FROM (SELECT COALESCE(MAX(state), 0) + 1 AS next FROM (
               SELECT MAX(state) AS state FROM changes WHERE account_id=?1 AND type=?2
               UNION ALL SELECT head_state AS state FROM change_log_checkpoint WHERE account_id=?1 AND type=?2
               ) heads) s,
              json_each(?3) j
         RETURNING state"
    };
    let rows = q(sql).bind(account_id).bind(kind).bind(ids_json(ids)).bind(op).bind(now()).fetch_all_tx(tx).await?;
    Ok(rows.first().map(|r| r.get_i64_idx(0) as u64))
}

/// A pseudo-UID free in (account, mailbox, uidvalidity 0) for `stable_id`.
async fn free_pseudo_uid(tx: &mut Tx, account_id: &str, mailbox_id: &str, stable_id: &str) -> Result<u32, StoreError> {
    for probe in 0..PSEUDO_UID_PROBES {
        let uid = pseudo_uid(stable_id, probe);
        let taken = q("SELECT stable_id FROM messages WHERE account_id=?1 AND mailbox_id=?2 AND uidvalidity=0 AND uid=?3")
            .bind(account_id)
            .bind(mailbox_id)
            .bind(i64::from(uid))
            .fetch_all_tx(tx)
            .await?;
        match taken.first() {
            None => return Ok(uid),
            Some(r) if r.get_string("stable_id") == stable_id => return Ok(uid),
            Some(_) => continue,
        }
    }
    Err(StoreError::Corrupt("no free pseudo-UID in the destination".into()))
}

/// Another row of the account at these coordinates (not `stable_id`).
async fn coords_taken(tx: &mut Tx, account_id: &str, at: &ServerCoords, stable_id: &str) -> Result<bool, StoreError> {
    let rows = q("SELECT stable_id FROM messages WHERE account_id=?1 AND mailbox_id=?2 AND uidvalidity=?3 AND uid=?4")
        .bind(account_id)
        .bind(&at.mailbox_id)
        .bind(i64::from(at.uidvalidity))
        .bind(i64::from(at.uid))
        .fetch_all_tx(tx)
        .await?;
    Ok(rows.iter().any(|r| r.get_string("stable_id") != stable_id))
}

/// Another intent of the account already holds these base coordinates.
async fn base_taken(tx: &mut Tx, account_id: &str, at: &ServerCoords, key: (&str, &str)) -> Result<bool, StoreError> {
    let rows = q("SELECT stable_id, field FROM upstream_outbox WHERE account_id=?1 AND base_mailbox_id=?2 AND base_uidvalidity=?3 AND base_uid=?4")
        .bind(account_id)
        .bind(&at.mailbox_id)
        .bind(i64::from(at.uidvalidity))
        .bind(i64::from(at.uid))
        .fetch_all_tx(tx)
        .await?;
    Ok(rows.iter().any(|r| (r.get_string("stable_id").as_str(), r.get_string("field").as_str()) != key))
}

#[allow(clippy::too_many_arguments)]
async fn insert_intent(
    tx: &mut Tx,
    account_id: &str,
    stable_id: &str,
    field: &str,
    seq: i64,
    want: Option<&str>,
    base: Option<&str>,
    base_location: Option<&ServerCoords>,
    origin: IntentOrigin,
    upstream_gen: i64,
) -> Result<(), StoreError> {
    let at = now();
    q("INSERT INTO upstream_outbox (account_id, stable_id, field, seq, first_at, updated_at, want, base, base_mailbox_id, base_uidvalidity, base_uid, state, origin, upstream_gen)
       VALUES (?1, ?2, ?3, ?4, ?5, ?5, ?6, ?7, ?8, ?9, ?10, 'pending', ?11, ?12)")
        .bind(account_id)
        .bind(stable_id)
        .bind(field)
        .bind(seq)
        .bind(at)
        .bind(want)
        .bind(base)
        .bind(base_location.map(|c| c.mailbox_id.as_str()))
        .bind(base_location.map(|c| i64::from(c.uidvalidity)))
        .bind(base_location.map(|c| i64::from(c.uid)))
        .bind(origin.as_str())
        .bind(upstream_gen)
        .execute_tx(tx)
        .await?;
    Ok(())
}

async fn delete_intent(tx: &mut Tx, account_id: &str, stable_id: &str, field: &str) -> Result<u64, StoreError> {
    Ok(q("DELETE FROM upstream_outbox WHERE account_id=?1 AND stable_id=?2 AND field=?3")
        .bind(account_id)
        .bind(stable_id)
        .bind(field)
        .execute_tx(tx)
        .await?)
}

/// Overwrite `want` (and, for a pending row, `seq`).
async fn overwrite_want(tx: &mut Tx, account_id: &str, stable_id: &str, field: &str, want: Option<&str>, seq: Option<i64>) -> Result<(), StoreError> {
    q("UPDATE upstream_outbox SET want=?4, seq=COALESCE(?5, seq), updated_at=?6 WHERE account_id=?1 AND stable_id=?2 AND field=?3")
        .bind(account_id)
        .bind(stable_id)
        .bind(field)
        .bind(want)
        .bind(seq)
        .bind(now())
        .execute_tx(tx)
        .await?;
    Ok(())
}

async fn insert_notice(tx: &mut Tx, account_id: &str, stable_id: Option<&str>, op: &str, reason: &str) -> Result<String, StoreError> {
    let id = uuid::Uuid::new_v4().simple().to_string();
    q("INSERT INTO upstream_notice (id, account_id, at, stable_id, op, reason) VALUES (?1, ?2, ?3, ?4, ?5, ?6)")
        .bind(&id)
        .bind(account_id)
        .bind(now())
        .bind(stable_id)
        .bind(op)
        .bind(reason)
        .execute_tx(tx)
        .await?;
    Ok(id)
}

/// The notice `op` of a field.
fn notice_op(field: &str) -> &'static str {
    if flag_of_field(field).is_some() {
        "flag"
    } else if field == FIELD_LOCATION {
        "move"
    } else {
        "draft"
    }
}

struct MessageRow {
    stable_id: String,
    mailbox_id: String,
    uidvalidity: u32,
    flags_json: String,
}

fn message_row(r: &Row) -> MessageRow {
    MessageRow {
        stable_id: r.get_string("stable_id"),
        mailbox_id: r.get_string("mailbox_id"),
        uidvalidity: r.get_i64("uidvalidity") as u32,
        flags_json: r.get_string("flags_json"),
    }
}

/// Lock (PostgreSQL) and read the account's messages among `ids`: plan read,
/// mailboxes `FOR UPDATE` in id order, then the messages `FOR UPDATE`. A
/// message that moved between the plan and the lock fails with
/// `SourceChanged` before any write (as `set_flags_batch`).
async fn lock_messages(tx: &mut Tx, account_id: &str, ids: &[String]) -> Result<Vec<MessageRow>, StoreError> {
    if ids.is_empty() {
        return Ok(Vec::new());
    }
    if is_pg(tx) {
        let plan = q("SELECT stable_id, mailbox_id FROM messages WHERE account_id=?1 AND stable_id IN (SELECT value FROM json_array_elements_text(?2::json))")
            .bind(account_id)
            .bind(ids_json(ids))
            .fetch_all_tx(tx)
            .await?;
        let planned: BTreeMap<String, String> = plan.iter().map(|r| (r.get_string("stable_id"), r.get_string("mailbox_id"))).collect();
        let boxes: Vec<String> = planned.values().cloned().collect::<BTreeSet<_>>().into_iter().collect();
        q("SELECT id FROM mailboxes WHERE account_id=?1 AND id IN (SELECT value FROM json_array_elements_text(?2::json)) ORDER BY id FOR UPDATE")
            .bind(account_id)
            .bind(ids_json(&boxes))
            .fetch_all_tx(tx)
            .await?;
        let rows = q("SELECT stable_id, mailbox_id, uidvalidity, flags_json FROM messages WHERE account_id=?1 AND stable_id IN (SELECT value FROM json_array_elements_text(?2::json)) ORDER BY stable_id FOR UPDATE")
            .bind(account_id)
            .bind(ids_json(ids))
            .fetch_all_tx(tx)
            .await?;
        let rows: Vec<MessageRow> = rows.iter().map(message_row).collect();
        let current: BTreeMap<String, String> = rows.iter().map(|r| (r.stable_id.clone(), r.mailbox_id.clone())).collect();
        if current != planned {
            return Err(StoreError::SourceChanged);
        }
        Ok(rows)
    } else {
        let rows = q("SELECT stable_id, mailbox_id, uidvalidity, flags_json FROM messages WHERE account_id=?1 AND stable_id IN (SELECT value FROM json_each(?2))")
            .bind(account_id)
            .bind(ids_json(ids))
            .fetch_all_tx(tx)
            .await?;
        Ok(rows.iter().map(message_row).collect())
    }
}

/// Write `flags_json` of one locked row, its unread counter and nothing else.
async fn write_flags_tx(tx: &mut Tx, account_id: &str, row: &MessageRow, flags_json: &str) -> Result<(), StoreError> {
    q("UPDATE messages SET flags_json=?3 WHERE stable_id=?1 AND account_id=?2")
        .bind(&row.stable_id)
        .bind(account_id)
        .bind(flags_json)
        .execute_tx(tx)
        .await?;
    let before = crate::cache::flags_json_is_unread(&row.flags_json);
    let after = crate::cache::flags_json_is_unread(flags_json);
    if before != after {
        crate::cache::shift_counters_tx(tx, account_id, &row.mailbox_id, 0, if after { 1 } else { -1 }).await?;
    }
    Ok(())
}

/// Seal a server error text for `error_detail`, bound to its row.
fn seal_detail(store: &Store, account_id: &str, stable_id: &str, field: &str, detail: &str) -> Result<Vec<u8>, StoreError> {
    Ok(store.key.seal_bound(detail.as_bytes(), SEAL_PURPOSE, &[account_id, stable_id, field])?)
}

impl Store {
    // ── mutations (each ONE transaction: CAS + counters + change rows + intents) ──

    /// Write flags of messages of `account_id` (pairs of stable id and the new
    /// full `flags_json`, last pair of an id wins, as [`Store::set_flags_batch`])
    /// and, in the same transaction, the Email change rows of the written ids
    /// (one state) and one intent per changed flag of every message that has
    /// server coordinates (a real UID, or a pending move whose base holds
    /// them). Rows without server coordinates change locally only. Ids of
    /// another account are not written.
    pub async fn set_flags_with_intents(
        &self,
        account_id: &str,
        updates: &[(String, String)],
        origin: IntentOrigin,
        cap: i64,
    ) -> Result<OutboxWrite<FlagsWritten>, StoreError> {
        if updates.is_empty() {
            return Ok(OutboxWrite::Done(FlagsWritten { written: Vec::new(), email_state: None, tally: IntentTally::default() }));
        }
        let mut desired: HashMap<&str, &str> = HashMap::with_capacity(updates.len());
        for (id, flags) in updates {
            desired.insert(id.as_str(), flags.as_str());
        }
        let ids: Vec<String> = desired.keys().map(|id| (*id).to_string()).collect::<BTreeSet<_>>().into_iter().collect();
        crate::backend::write_job!(self.backend, crate::backend::WriteLane::Normal, |tx| async {
            lock_account(tx, account_id).await?;
            let rows = lock_messages(tx, account_id, &ids).await?;
            let mut head = read_head(tx, account_id).await?.unwrap_or_else(default_head);
            let present: Vec<String> = rows.iter().map(|r| r.stable_id.clone()).collect();
            let intents = read_intents(tx, account_id, &present).await?;
            let mut by_key: HashMap<(&str, &str), &OutboxIntent> = HashMap::new();
            for intent in &intents {
                by_key.insert((intent.stable_id.as_str(), intent.field.as_str()), intent);
            }

            // Plan every (message, flag) before writing anything (the cap check).
            let mut inserts: Vec<(String, String, String, String)> = Vec::new(); // id, field, want, base
            let mut overwrites: Vec<(String, String, String, bool)> = Vec::new(); // id, field, want, pending
            let mut cancels: Vec<(String, String)> = Vec::new();
            let mut writes: Vec<(&MessageRow, &str)> = Vec::new();
            for row in &rows {
                let Some(new_flags) = desired.get(row.stable_id.as_str()).copied() else { continue };
                writes.push((row, new_flags));
                let addressable = row.uidvalidity != 0
                    || by_key.get(&(row.stable_id.as_str(), FIELD_LOCATION)).is_some_and(|l| l.base_location.is_some());
                if !addressable {
                    continue;
                }
                let old = flag_keys(&row.flags_json);
                let new = flag_keys(new_flags);
                for key in old.symmetric_difference(&new) {
                    let field = flag_field(key);
                    let want = bit(new.contains(key));
                    let server_now = bit(old.contains(key));
                    let existing = by_key.get(&(row.stable_id.as_str(), field.as_str())).copied();
                    match plan_value(existing, want, server_now) {
                        ValuePlan::Insert => inserts.push((row.stable_id.clone(), field, want.into(), server_now.into())),
                        ValuePlan::Overwrite => overwrites.push((row.stable_id.clone(), field, want.into(), true)),
                        ValuePlan::OverwriteInFlight => overwrites.push((row.stable_id.clone(), field, want.into(), false)),
                        ValuePlan::Cancel => cancels.push((row.stable_id.clone(), field)),
                        ValuePlan::Nothing => {}
                    }
                }
            }
            let grows = inserts.len() as i64 - cancels.len() as i64;
            if !inserts.is_empty() && head.backlog + grows > cap {
                return Ok(OutboxWrite::AtCap { backlog: head.backlog, cap });
            }
            if writes.is_empty() {
                return Ok(OutboxWrite::Done(FlagsWritten { written: updates.iter().map(|_| false).collect(), email_state: None, tally: IntentTally::default() }));
            }

            // The flag UPDATE (one statement) and the unread counters.
            let pairs: Vec<[&str; 2]> = writes.iter().map(|(row, flags)| [row.stable_id.as_str(), *flags]).collect();
            let payload = serde_json::to_string(&pairs).expect("pairs always serialize");
            let sql = if is_pg(tx) {
                "UPDATE messages SET flags_json = j.f FROM (SELECT x ->> 0 AS id, x ->> 1 AS f FROM json_array_elements(?1::json) x) j WHERE messages.stable_id = j.id AND messages.account_id = ?2"
            } else {
                "UPDATE messages SET flags_json = j.f FROM (SELECT json_extract(value, '$[0]') AS id, json_extract(value, '$[1]') AS f FROM json_each(?1)) j WHERE messages.stable_id = j.id AND messages.account_id = ?2"
            };
            q(sql).bind(payload).bind(account_id).execute_tx(tx).await?;
            let mut delta: BTreeMap<&str, i64> = BTreeMap::new();
            for (row, flags) in &writes {
                match (crate::cache::flags_json_is_unread(&row.flags_json), crate::cache::flags_json_is_unread(flags)) {
                    (true, false) => *delta.entry(row.mailbox_id.as_str()).or_default() -= 1,
                    (false, true) => *delta.entry(row.mailbox_id.as_str()).or_default() += 1,
                    _ => {}
                }
            }
            for (mailbox_id, d) in delta {
                crate::cache::shift_counters_tx(tx, account_id, mailbox_id, 0, d).await?;
            }
            let written_ids: Vec<String> = writes.iter().map(|(row, _)| row.stable_id.clone()).collect();
            let email_state = record_changes_tx(tx, account_id, "Email", &written_ids, "updated").await?;

            // The intents (one statement per kind) and the head.
            let mut tally = IntentTally::default();
            if !inserts.is_empty() {
                let mut rows_json: Vec<serde_json::Value> = Vec::with_capacity(inserts.len());
                for (id, field, want, base) in &inserts {
                    rows_json.push(serde_json::json!([id, field, head.next_seq, want, base]));
                    head.next_seq += 1;
                }
                let sql = if is_pg(tx) {
                    "INSERT INTO upstream_outbox (account_id, stable_id, field, seq, first_at, updated_at, want, base, state, origin, upstream_gen)
                     SELECT ?1, x ->> 0, x ->> 1, CAST(x ->> 2 AS BIGINT), ?2, ?2, x ->> 3, x ->> 4, 'pending', ?3, ?4 FROM json_array_elements(?5::json) x"
                } else {
                    "INSERT INTO upstream_outbox (account_id, stable_id, field, seq, first_at, updated_at, want, base, state, origin, upstream_gen)
                     SELECT ?1, json_extract(value, '$[0]'), json_extract(value, '$[1]'), json_extract(value, '$[2]'), ?2, ?2, json_extract(value, '$[3]'), json_extract(value, '$[4]'), 'pending', ?3, ?4 FROM json_each(?5)"
                };
                q(sql)
                    .bind(account_id)
                    .bind(now())
                    .bind(origin.as_str())
                    .bind(head.upstream_gen)
                    .bind(serde_json::Value::Array(rows_json).to_string())
                    .execute_tx(tx)
                    .await?;
                tally.created = inserts.len() as u32;
            }
            if !overwrites.is_empty() {
                let mut rows_json: Vec<serde_json::Value> = Vec::with_capacity(overwrites.len());
                for (id, field, want, pending) in &overwrites {
                    // A pending row moves to the end of the order; one in flight keeps its seq.
                    let seq = if *pending {
                        head.next_seq += 1;
                        Some(head.next_seq - 1)
                    } else {
                        None
                    };
                    rows_json.push(serde_json::json!([id, field, seq, want]));
                }
                let sql = if is_pg(tx) {
                    "UPDATE upstream_outbox SET want = j.w, seq = COALESCE(j.s, upstream_outbox.seq), updated_at = ?2
                     FROM (SELECT x ->> 0 AS id, x ->> 1 AS f, CAST(x ->> 2 AS BIGINT) AS s, x ->> 3 AS w FROM json_array_elements(?3::json) x) j
                     WHERE upstream_outbox.account_id = ?1 AND upstream_outbox.stable_id = j.id AND upstream_outbox.field = j.f"
                } else {
                    "UPDATE upstream_outbox SET want = j.w, seq = COALESCE(j.s, upstream_outbox.seq), updated_at = ?2
                     FROM (SELECT json_extract(value, '$[0]') AS id, json_extract(value, '$[1]') AS f, json_extract(value, '$[2]') AS s, json_extract(value, '$[3]') AS w FROM json_each(?3)) j
                     WHERE upstream_outbox.account_id = ?1 AND upstream_outbox.stable_id = j.id AND upstream_outbox.field = j.f"
                };
                q(sql)
                    .bind(account_id)
                    .bind(now())
                    .bind(serde_json::Value::Array(rows_json).to_string())
                    .execute_tx(tx)
                    .await?;
                tally.overwritten = overwrites.len() as u32;
            }
            if !cancels.is_empty() {
                let keys: Vec<[&str; 2]> = cancels.iter().map(|(id, field)| [id.as_str(), field.as_str()]).collect();
                let sql = if is_pg(tx) {
                    "DELETE FROM upstream_outbox WHERE account_id = ?1 AND (stable_id, field) IN (SELECT x ->> 0, x ->> 1 FROM json_array_elements(?2::json) x)"
                } else {
                    "DELETE FROM upstream_outbox WHERE account_id = ?1 AND (stable_id, field) IN (SELECT json_extract(value, '$[0]'), json_extract(value, '$[1]') FROM json_each(?2))"
                };
                q(sql).bind(account_id).bind(serde_json::to_string(&keys).expect("keys serialize")).execute_tx(tx).await?;
                tally.cancelled = cancels.len() as u32;
            }
            if tally != IntentTally::default() {
                head.backlog += grows;
                write_head(tx, account_id, &head).await?;
            }
            let written: BTreeSet<&str> = written_ids.iter().map(String::as_str).collect();
            Ok::<_, StoreError>(OutboxWrite::Done(FlagsWritten {
                written: updates.iter().map(|(id, _)| written.contains(id.as_str())).collect(),
                email_state,
                tally,
            }))
        })
    }

    /// Move one message locally (source CAS as
    /// [`Store::relocate_message_if_current`]) and record the move as a
    /// `location` intent, in one transaction with both mailbox counters and the
    /// change rows (Email updated; Mailbox updated for source and destination).
    ///
    /// The row gets `uidvalidity` 0 and a pseudo-UID unique in the destination;
    /// the first unsent move keeps the server coordinates in the intent's base.
    /// Moving back to the base folder before anything was sent deletes the
    /// intent and restores the base coordinates (§3.3). A row without server
    /// coordinates moves locally only (a pending draft APPEND follows it).
    pub async fn relocate_with_intent(
        &self,
        expected: &Message,
        destination: &str,
        origin: IntentOrigin,
        cap: i64,
    ) -> Result<OutboxWrite<Relocated>, StoreError> {
        let account_id = expected.account_id.as_str();
        if expected.mailbox_id == destination {
            return Ok(OutboxWrite::Done(Relocated { moved: true, uidvalidity: expected.uidvalidity, uid: expected.uid, email_state: None, tally: IntentTally::default() }));
        }
        crate::backend::write_job!(self.backend, crate::backend::WriteLane::Normal, |tx| async {
            let unchanged = Relocated { moved: false, uidvalidity: expected.uidvalidity, uid: expected.uid, email_state: None, tally: IntentTally::default() };
            if crate::publication_mutations::lock_mailboxes(tx, account_id, &expected.mailbox_id, destination).await? != 2 {
                return Ok(OutboxWrite::Done(unchanged));
            }
            let mut head = read_head(tx, account_id).await?.unwrap_or_else(default_head);
            let intents = read_intents(tx, account_id, std::slice::from_ref(&expected.stable_id)).await?;
            let location = intents.iter().find(|i| i.field == FIELD_LOCATION);
            let append = intents.iter().find(|i| i.field == FIELD_APPEND);

            enum Step<'a> {
                Insert(ServerCoords),
                Overwrite(&'a OutboxIntent),
                OverwriteInFlight,
                Cancel(ServerCoords),
                FollowAppend,
                LocalOnly,
            }
            let step = match location {
                Some(l) if l.state == IntentState::Pending => {
                    match (&l.base_location, l.shadow.as_deref()) {
                        (Some(base), None) if base.mailbox_id == destination => Step::Cancel(base.clone()),
                        _ => Step::Overwrite(l),
                    }
                }
                Some(_) => Step::OverwriteInFlight,
                None if expected.uidvalidity != 0 => Step::Insert(ServerCoords {
                    mailbox_id: expected.mailbox_id.clone(),
                    uidvalidity: expected.uidvalidity,
                    uid: expected.uid,
                }),
                None if append.is_some_and(|a| a.state == IntentState::Pending) => Step::FollowAppend,
                None => Step::LocalOnly,
            };
            if matches!(step, Step::Insert(_)) && head.backlog + 1 > cap {
                return Ok(OutboxWrite::AtCap { backlog: head.backlog, cap });
            }
            let (uidvalidity, uid) = match &step {
                Step::Cancel(base) => {
                    if coords_taken(tx, account_id, base, &expected.stable_id).await? {
                        // Another row holds the server coordinates: sync took the
                        // message in as new mail, which the base lookup prevents.
                        return Err(StoreError::SourceChanged);
                    }
                    (base.uidvalidity, base.uid)
                }
                _ => (0, free_pseudo_uid(tx, account_id, destination, &expected.stable_id).await?),
            };
            let moved = q("UPDATE messages SET mailbox_id=?8, uid=?9, uidvalidity=?10 WHERE stable_id=?1 AND account_id=?2 AND mailbox_id=?3 AND uidvalidity=?4 AND uid=?5 AND cache_revision=?6 AND (blob_ref=?7 OR (blob_ref IS NULL AND ?7 IS NULL)) RETURNING flags_json")
                .bind(&expected.stable_id).bind(account_id).bind(&expected.mailbox_id)
                .bind(i64::from(expected.uidvalidity)).bind(i64::from(expected.uid)).bind(&expected.cache_revision)
                .bind(expected.blob_ref.as_deref()).bind(destination).bind(i64::from(uid)).bind(i64::from(uidvalidity))
                .fetch_all_tx(tx)
                .await?;
            let Some(row) = moved.first() else {
                return Ok(OutboxWrite::Done(unchanged));
            };
            let unread = i64::from(crate::cache::flags_json_is_unread(&row.get_string("flags_json")));
            crate::cache::shift_counters_tx(tx, account_id, &expected.mailbox_id, -1, -unread).await?;
            crate::cache::shift_counters_tx(tx, account_id, destination, 1, unread).await?;
            let email_state = record_changes_tx(tx, account_id, "Email", std::slice::from_ref(&expected.stable_id), "updated").await?;
            record_changes_tx(tx, account_id, "Mailbox", &[expected.mailbox_id.clone(), destination.to_string()], "updated").await?;

            let mut tally = IntentTally::default();
            match step {
                Step::Insert(base) => {
                    insert_intent(tx, account_id, &expected.stable_id, FIELD_LOCATION, head.next_seq, Some(destination), None, Some(&base), origin, head.upstream_gen).await?;
                    head.next_seq += 1;
                    head.backlog += 1;
                    tally.created = 1;
                }
                Step::Overwrite(l) => {
                    let _ = l;
                    overwrite_want(tx, account_id, &expected.stable_id, FIELD_LOCATION, Some(destination), Some(head.next_seq)).await?;
                    head.next_seq += 1;
                    tally.overwritten = 1;
                }
                Step::OverwriteInFlight => {
                    overwrite_want(tx, account_id, &expected.stable_id, FIELD_LOCATION, Some(destination), None).await?;
                    tally.overwritten = 1;
                }
                Step::Cancel(_) => {
                    delete_intent(tx, account_id, &expected.stable_id, FIELD_LOCATION).await?;
                    head.backlog -= 1;
                    tally.cancelled = 1;
                }
                Step::FollowAppend => {
                    overwrite_want(tx, account_id, &expected.stable_id, FIELD_APPEND, Some(destination), Some(head.next_seq)).await?;
                    head.next_seq += 1;
                    tally.overwritten = 1;
                }
                Step::LocalOnly => {}
            }
            if tally != IntentTally::default() {
                write_head(tx, account_id, &head).await?;
            }
            Ok::<_, StoreError>(OutboxWrite::Done(Relocated { moved: true, uidvalidity, uid, email_state, tally }))
        })
    }

    /// Ingest a locally composed draft (engine-local: `uidvalidity` 0) exactly
    /// as [`Store::ingest_message_with_body`] does, with its change rows, and an
    /// `append` intent (want = `m.mailbox_id`) in the same transaction.
    pub async fn ingest_draft_with_append_intent(
        &self,
        m: &MessageUpsert<'_>,
        raw: &[u8],
        origin: IntentOrigin,
        cap: i64,
    ) -> Result<OutboxWrite<DraftIngested>, StoreError> {
        if m.uidvalidity != 0 {
            return Err(StoreError::Invalid("a draft with an append intent is engine-local (uidvalidity 0)"));
        }
        let prepared = self.prepare_message_body(m, raw).await?;
        let mut tx = self.backend.begin().await?;
        let outcome = self.upsert_prepared_body_tx(m, raw, prepared, true, &mut tx).await?;
        let mut head = read_head(&mut tx, m.account_id).await?.unwrap_or_else(default_head);
        let existing = read_intent(&mut tx, m.account_id, &outcome.stable_id, FIELD_APPEND).await?;
        let mut tally = IntentTally::default();
        match existing {
            None => {
                if head.backlog + 1 > cap {
                    drop(tx);
                    return Ok(OutboxWrite::AtCap { backlog: head.backlog, cap });
                }
                insert_intent(&mut tx, m.account_id, &outcome.stable_id, FIELD_APPEND, head.next_seq, Some(m.mailbox_id), None, None, origin, head.upstream_gen).await?;
                head.next_seq += 1;
                head.backlog += 1;
                tally.created = 1;
            }
            Some(row) if row.want.as_deref() != Some(m.mailbox_id) => {
                let seq = (row.state == IntentState::Pending).then_some(head.next_seq);
                overwrite_want(&mut tx, m.account_id, &outcome.stable_id, FIELD_APPEND, Some(m.mailbox_id), seq).await?;
                if seq.is_some() {
                    head.next_seq += 1;
                }
                tally.overwritten = 1;
            }
            Some(_) => {}
        }
        if tally != IntentTally::default() {
            write_head(&mut tx, m.account_id, &head).await?;
        }
        tx.commit().await?;
        Ok(OutboxWrite::Done(DraftIngested { stable_id: outcome.stable_id, existed: outcome.existed, tally }))
    }

    /// Delete a draft locally (source CAS as
    /// [`Store::delete_message_if_current`]) with its counters and change rows
    /// (Email destroyed, Mailbox updated), and in the same transaction: a
    /// pending `append` intent is deleted (nothing is sent, §3.3 row 8), one in
    /// flight or unresolved keeps its row with `want = NULL` (its confirmation
    /// turns it into an `expunge_copy`), every other intent of the message is
    /// deleted, and `server_copy` (the APPENDUID coordinates the engine
    /// remembered) becomes an `expunge_copy` intent.
    pub async fn delete_draft_with_copy_intent(
        &self,
        expected: &Message,
        server_copy: Option<&ServerCoords>,
        origin: IntentOrigin,
        cap: i64,
    ) -> Result<OutboxWrite<DraftDeleted>, StoreError> {
        let account_id = expected.account_id.as_str();
        crate::backend::write_job!(self.backend, crate::backend::WriteLane::Normal, |tx| async {
            let unchanged = DraftDeleted { deleted: false, email_state: None, tally: IntentTally::default() };
            if crate::publication_mutations::lock_mailboxes(tx, account_id, &expected.mailbox_id, &expected.mailbox_id).await? != 1 {
                return Ok(OutboxWrite::Done(unchanged));
            }
            let mut head = read_head(tx, account_id).await?.unwrap_or_else(default_head);
            let intents = read_intents(tx, account_id, std::slice::from_ref(&expected.stable_id)).await?;
            let removed = intents.iter().filter(|i| i.field != FIELD_APPEND || i.state == IntentState::Pending).count() as i64;
            let created = i64::from(server_copy.is_some());
            if created > 0 && head.backlog - removed + created > cap {
                return Ok(OutboxWrite::AtCap { backlog: head.backlog, cap });
            }
            if let Some(copy) = server_copy {
                if base_taken(tx, account_id, copy, (&expected.stable_id, FIELD_EXPUNGE_COPY)).await? {
                    return Err(StoreError::Invalid("that server copy already has an intent"));
                }
            }
            let gone = q("DELETE FROM messages WHERE stable_id=?1 AND account_id=?2 AND mailbox_id=?3 AND uidvalidity=?4 AND uid=?5 AND cache_revision=?6 AND (blob_ref=?7 OR (blob_ref IS NULL AND ?7 IS NULL)) RETURNING flags_json")
                .bind(&expected.stable_id).bind(account_id).bind(&expected.mailbox_id)
                .bind(i64::from(expected.uidvalidity)).bind(i64::from(expected.uid)).bind(&expected.cache_revision)
                .bind(expected.blob_ref.as_deref())
                .fetch_all_tx(tx)
                .await?;
            let Some(row) = gone.first() else {
                return Ok(OutboxWrite::Done(unchanged));
            };
            q("DELETE FROM message_embeddings WHERE stable_id=?1 AND account_id=?2")
                .bind(&expected.stable_id)
                .bind(account_id)
                .execute_tx(tx)
                .await?;
            let unread = i64::from(crate::cache::flags_json_is_unread(&row.get_string("flags_json")));
            crate::cache::shift_counters_tx(tx, account_id, &expected.mailbox_id, -1, -unread).await?;
            let email_state = record_changes_tx(tx, account_id, "Email", std::slice::from_ref(&expected.stable_id), "destroyed").await?;
            record_changes_tx(tx, account_id, "Mailbox", std::slice::from_ref(&expected.mailbox_id), "updated").await?;

            let mut tally = IntentTally::default();
            for intent in &intents {
                if intent.field == FIELD_APPEND && intent.state != IntentState::Pending {
                    overwrite_want(tx, account_id, &expected.stable_id, FIELD_APPEND, None, None).await?;
                    tally.overwritten += 1;
                } else {
                    delete_intent(tx, account_id, &expected.stable_id, &intent.field).await?;
                    tally.cancelled += 1;
                }
            }
            head.backlog -= removed;
            if let Some(copy) = server_copy {
                insert_intent(tx, account_id, &expected.stable_id, FIELD_EXPUNGE_COPY, head.next_seq, Some("1"), None, Some(copy), origin, head.upstream_gen).await?;
                head.next_seq += 1;
                head.backlog += 1;
                tally.created = 1;
            }
            if tally != IntentTally::default() {
                write_head(tx, account_id, &head).await?;
            }
            Ok::<_, StoreError>(OutboxWrite::Done(DraftDeleted { deleted: true, email_state, tally }))
        })
    }

    // ── applier side ────────────────────────────────────────────────────────

    /// The account's head row, `None` before its first intent.
    pub async fn upstream_outbox_head(&self, account_id: &str) -> Result<Option<OutboxHead>, StoreError> {
        let rows = q("SELECT next_seq, backlog, upstream_gen, paused, paused_at FROM upstream_outbox_head WHERE account_id=?1")
            .bind(account_id)
            .fetch_all(&self.backend)
            .await?;
        Ok(rows.first().map(|r| OutboxHead {
            next_seq: r.get_i64("next_seq"),
            backlog: r.get_i64("backlog"),
            upstream_gen: r.get_i64("upstream_gen"),
            paused: r.get_opt_string("paused"),
            paused_at: r.get_opt_string("paused_at"),
        }))
    }

    /// Every intent of the account in seq order.
    pub async fn upstream_intents(&self, account_id: &str) -> Result<Vec<OutboxIntent>, StoreError> {
        let rows = q(concat!("SELECT ", intent_columns!(), " FROM upstream_outbox WHERE account_id=?1 ORDER BY seq"))
            .bind(account_id)
            .fetch_all(&self.backend)
            .await?;
        rows.iter().map(intent_from_row).collect()
    }

    /// Pending intents due at `now` (RFC 3339) of the current upstream
    /// generation, oldest seq first, at most `limit`.
    pub async fn due_upstream_intents(&self, account_id: &str, now: &str, limit: i64) -> Result<Vec<OutboxIntent>, StoreError> {
        let rows = q(concat!(
            "SELECT ", intent_columns!(), " FROM upstream_outbox WHERE account_id=?1 AND state='pending' AND (next_at IS NULL OR next_at<=?2)
             AND upstream_gen=(SELECT upstream_gen FROM upstream_outbox_head WHERE account_id=?1) ORDER BY seq LIMIT ?3"
        ))
        .bind(account_id)
        .bind(now)
        .bind(limit)
        .fetch_all(&self.backend)
        .await?;
        rows.iter().map(intent_from_row).collect()
    }

    /// Compare-and-set `pending`/`unresolved` → `inflight` with `token`, for
    /// the keys `(stable_id, field, seq)` the applier read (a row overwritten
    /// since then has a new seq and is skipped). Sets `sent := want` (an
    /// unresolved row keeps its earlier `sent` for recovery) and counts the
    /// attempt. A pending row whose want equals its shadow is deleted instead
    /// (§3.3: another client already did it). Nothing is claimed while the
    /// account is paused or for rows of an older upstream generation.
    pub async fn claim_upstream_intents(
        &self,
        account_id: &str,
        keys: &[(String, String, i64)],
        token: &str,
    ) -> Result<ClaimOutcome, StoreError> {
        crate::backend::write_job!(self.backend, crate::backend::WriteLane::Normal, |tx| async {
            lock_account(tx, account_id).await?;
            let Some(mut head) = read_head(tx, account_id).await? else {
                return Ok(ClaimOutcome { skipped: keys.iter().map(|(s, f, _)| (s.clone(), f.clone())).collect(), ..ClaimOutcome::default() });
            };
            if let Some(paused) = head.paused.clone() {
                return Ok(ClaimOutcome { paused: Some(paused), skipped: keys.iter().map(|(s, f, _)| (s.clone(), f.clone())).collect(), ..ClaimOutcome::default() });
            }
            let mut out = ClaimOutcome::default();
            for (stable_id, field, seq) in keys {
                let row = read_intent(tx, account_id, stable_id, field).await?;
                let Some(row) = row.filter(|r| r.seq == *seq && r.upstream_gen == head.upstream_gen && r.state != IntentState::Inflight) else {
                    out.skipped.push((stable_id.clone(), field.clone()));
                    continue;
                };
                if row.state == IntentState::Pending && row.shadow.is_some() && row.shadow == row.want {
                    delete_intent(tx, account_id, stable_id, field).await?;
                    head.backlog -= 1;
                    out.cancelled.push((stable_id.clone(), field.clone()));
                    continue;
                }
                let claimed = q(concat!(
                    "UPDATE upstream_outbox SET state='inflight', claim=?4, sent=CASE WHEN state='unresolved' THEN sent ELSE want END,
                       attempts=attempts+1, updated_at=?5
                     WHERE account_id=?1 AND stable_id=?2 AND field=?3 RETURNING ", intent_columns!()
                ))
                .bind(account_id)
                .bind(stable_id)
                .bind(field)
                .bind(token)
                .bind(now())
                .fetch_all_tx(tx)
                .await?;
                if let Some(r) = claimed.first() {
                    out.claimed.push(ClaimedIntent { intent: intent_from_row(r)?, was: row.state });
                }
            }
            if !out.cancelled.is_empty() {
                write_head(tx, account_id, &head).await?;
            }
            Ok::<_, StoreError>(out)
        })
    }

    /// Record the server's acknowledgement of claimed intents (one
    /// transaction for the batch; results aligned with `items`). See
    /// [`Confirmed`] for the outcomes and §3.3/§3.4 for the rules.
    pub async fn confirm_upstream_intents(
        &self,
        account_id: &str,
        token: &str,
        items: &[(String, String, Confirmation)],
    ) -> Result<Vec<Confirmed>, StoreError> {
        crate::backend::write_job!(self.backend, crate::backend::WriteLane::Normal, |tx| async {
            lock_account(tx, account_id).await?;
            // Lock every mailbox a move confirmation touches before the first write.
            if is_pg(tx) {
                let mut boxes: BTreeSet<String> = BTreeSet::new();
                for (stable_id, field, _) in items {
                    if field == FIELD_LOCATION {
                        if let Some(sent) = q("SELECT sent FROM upstream_outbox WHERE account_id=?1 AND stable_id=?2 AND field=?3")
                            .bind(account_id).bind(stable_id).bind(field).fetch_all_tx(tx).await?
                            .first().and_then(|r| r.get_opt_string("sent")) {
                            boxes.insert(sent);
                        }
                    }
                }
                if !boxes.is_empty() {
                    let boxes: Vec<String> = boxes.into_iter().collect();
                    q("SELECT id FROM mailboxes WHERE account_id=?1 AND id IN (SELECT value FROM json_array_elements_text(?2::json)) ORDER BY id FOR UPDATE")
                        .bind(account_id).bind(ids_json(&boxes)).fetch_all_tx(tx).await?;
                    let ids: Vec<String> = items.iter().filter(|(_, f, _)| f == FIELD_LOCATION).map(|(s, _, _)| s.clone()).collect::<BTreeSet<_>>().into_iter().collect();
                    q("SELECT stable_id FROM messages WHERE account_id=?1 AND stable_id IN (SELECT value FROM json_array_elements_text(?2::json)) ORDER BY stable_id FOR UPDATE")
                        .bind(account_id).bind(ids_json(&ids)).fetch_all_tx(tx).await?;
                }
            }
            let mut head = read_head(tx, account_id).await?.unwrap_or_else(default_head);
            let mut out = Vec::with_capacity(items.len());
            let mut dirty = false;
            for (stable_id, field, confirmation) in items {
                let row = read_intent(tx, account_id, stable_id, field).await?;
                let Some(row) = row.filter(|r| r.state == IntentState::Inflight && r.claim.as_deref() == Some(token)) else {
                    out.push(Confirmed::Stale);
                    continue;
                };
                dirty = true;
                let result = match (field.as_str(), confirmation) {
                    (FIELD_LOCATION, Confirmation::Moved { uidvalidity, uid }) => {
                        let sent = row.sent.clone().ok_or_else(|| StoreError::Corrupt("move in flight without sent".into()))?;
                        let at = ServerCoords { mailbox_id: sent.clone(), uidvalidity: *uidvalidity, uid: *uid };
                        if coords_taken(tx, account_id, &at, stable_id).await? || base_taken(tx, account_id, &at, (stable_id, field)).await? {
                            mark_collision(tx, account_id, stable_id, field, &at).await?;
                            Confirmed::Collision
                        } else if row.want.as_deref() == Some(sent.as_str()) {
                            q("UPDATE messages SET uidvalidity=?4, uid=?5 WHERE account_id=?1 AND stable_id=?2 AND mailbox_id=?3 AND uidvalidity=0")
                                .bind(account_id).bind(stable_id).bind(&sent)
                                .bind(i64::from(*uidvalidity)).bind(i64::from(*uid))
                                .execute_tx(tx).await?;
                            delete_intent(tx, account_id, stable_id, field).await?;
                            head.backlog -= 1;
                            Confirmed::Done
                        } else {
                            // Moved again while in flight: the new base is where it went.
                            q("UPDATE upstream_outbox SET base_mailbox_id=?4, base_uidvalidity=?5, base_uid=?6, state='pending', claim=NULL, sent=NULL, seq=?7, updated_at=?8, error_class=NULL, error_detail=NULL
                               WHERE account_id=?1 AND stable_id=?2 AND field=?3")
                                .bind(account_id).bind(stable_id).bind(field)
                                .bind(&sent).bind(i64::from(*uidvalidity)).bind(i64::from(*uid))
                                .bind(head.next_seq).bind(now())
                                .execute_tx(tx).await?;
                            head.next_seq += 1;
                            Confirmed::Requeued
                        }
                    }
                    (FIELD_APPEND, Confirmation::Appended { uidvalidity, uid }) => {
                        let sent = row.sent.clone().ok_or_else(|| StoreError::Corrupt("append in flight without sent".into()))?;
                        delete_intent(tx, account_id, stable_id, field).await?;
                        if row.want.is_none() {
                            let copy = ServerCoords { mailbox_id: sent, uidvalidity: *uidvalidity, uid: *uid };
                            if base_taken(tx, account_id, &copy, (stable_id, FIELD_EXPUNGE_COPY)).await? {
                                head.backlog -= 1;
                                Confirmed::Collision
                            } else {
                                insert_intent(tx, account_id, stable_id, FIELD_EXPUNGE_COPY, head.next_seq, Some("1"), None, Some(&copy), row.origin, head.upstream_gen).await?;
                                head.next_seq += 1;
                                Confirmed::ExpungeQueued
                            }
                        } else {
                            head.backlog -= 1;
                            Confirmed::Done
                        }
                    }
                    (FIELD_EXPUNGE_COPY, Confirmation::Applied) => {
                        delete_intent(tx, account_id, stable_id, field).await?;
                        head.backlog -= 1;
                        Confirmed::Done
                    }
                    (f, Confirmation::Applied) if flag_of_field(f).is_some() => {
                        if row.want == row.sent {
                            delete_intent(tx, account_id, stable_id, field).await?;
                            head.backlog -= 1;
                            Confirmed::Done
                        } else {
                            // Changed while in flight (§3.3 row 3): base := sent,
                            // shadow cleared, pending again (want differs from it).
                            q("UPDATE upstream_outbox SET base=?4, shadow=NULL, state='pending', claim=NULL, sent=NULL, seq=?5, updated_at=?6, error_class=NULL, error_detail=NULL
                               WHERE account_id=?1 AND stable_id=?2 AND field=?3")
                                .bind(account_id).bind(stable_id).bind(field)
                                .bind(row.sent.as_deref()).bind(head.next_seq).bind(now())
                                .execute_tx(tx).await?;
                            head.next_seq += 1;
                            Confirmed::Requeued
                        }
                    }
                    _ => return Err(StoreError::Invalid("confirmation does not match the intent's field")),
                };
                out.push(result);
            }
            if dirty {
                write_head(tx, account_id, &head).await?;
            }
            Ok::<_, StoreError>(out)
        })
    }

    /// Put claimed intents back to `pending` after a transient failure:
    /// `next_at` is when they may be sent again; the error class is kept and
    /// the server text sealed into `error_detail`.
    pub async fn retry_upstream_intents(
        &self,
        account_id: &str,
        token: &str,
        keys: &[(String, String)],
        error_class: &str,
        detail: Option<&str>,
        next_at: &str,
    ) -> Result<u64, StoreError> {
        self.release_claimed(account_id, token, keys, "pending", error_class, detail, Some(next_at)).await
    }

    /// Mark claimed moves or APPENDs whose outcome is unknown (connection lost
    /// after sending, no COPYUID/APPENDUID for them) `unresolved`: recovery
    /// (§3.4) runs before anything is resent. `sent` is kept.
    pub async fn mark_upstream_unresolved(
        &self,
        account_id: &str,
        token: &str,
        keys: &[(String, String)],
        error_class: &str,
        detail: Option<&str>,
    ) -> Result<u64, StoreError> {
        self.release_claimed(account_id, token, keys, "unresolved", error_class, detail, None).await
    }

    #[allow(clippy::too_many_arguments)]
    async fn release_claimed(
        &self,
        account_id: &str,
        token: &str,
        keys: &[(String, String)],
        state: &str,
        error_class: &str,
        detail: Option<&str>,
        next_at: Option<&str>,
    ) -> Result<u64, StoreError> {
        let mut sealed: Vec<Option<Vec<u8>>> = Vec::with_capacity(keys.len());
        for (stable_id, field) in keys {
            sealed.push(detail.map(|d| seal_detail(self, account_id, stable_id, field, d)).transpose()?);
        }
        crate::backend::write_job!(self.backend, crate::backend::WriteLane::Normal, |tx| async {
            lock_account(tx, account_id).await?;
            let mut n = 0;
            for ((stable_id, field), sealed) in keys.iter().zip(&sealed) {
                n += q("UPDATE upstream_outbox SET state=?5, claim=NULL, sent=CASE WHEN ?5='pending' THEN NULL ELSE sent END,
                          next_at=?6, error_class=?7, error_detail=?8, updated_at=?9
                        WHERE account_id=?1 AND stable_id=?2 AND field=?3 AND state='inflight' AND claim=?4")
                    .bind(account_id).bind(stable_id).bind(field).bind(token)
                    .bind(state).bind(next_at).bind(error_class).bind(sealed.clone()).bind(now())
                    .execute_tx(tx)
                    .await?;
            }
            Ok::<_, StoreError>(n)
        })
    }

    /// After a restart: `inflight` flag and copy-expunge rows go back to
    /// `pending` (both are idempotent), `inflight` moves and APPENDs become
    /// `unresolved` (§5.5). Returns (requeued, unresolved).
    pub async fn recover_upstream_after_restart(&self, account_id: &str) -> Result<(u64, u64), StoreError> {
        crate::backend::write_job!(self.backend, crate::backend::WriteLane::Normal, |tx| async {
            lock_account(tx, account_id).await?;
            let requeued = q("UPDATE upstream_outbox SET state='pending', claim=NULL, sent=NULL WHERE account_id=?1 AND state='inflight' AND field NOT IN ('location','append')")
                .bind(account_id).execute_tx(tx).await?;
            let unresolved = q("UPDATE upstream_outbox SET state='unresolved', claim=NULL WHERE account_id=?1 AND state='inflight' AND field IN ('location','append')")
                .bind(account_id).execute_tx(tx).await?;
            Ok::<_, StoreError>((requeued, unresolved))
        })
    }

    /// The server refused an intent for good (§5.2): one transaction sets the
    /// local value to `shadow`, else `base` (a move goes back to its base
    /// coordinates; with `shadow = 'gone'`, or its base folder gone, the row is
    /// deleted), writes the Email/Mailbox change rows at a new state, deletes
    /// the intent and inserts an `upstream_notice` row (op, reason, message
    /// id; no content). With `token`, only a row in flight under it.
    pub async fn reject_and_revert(
        &self,
        account_id: &str,
        stable_id: &str,
        field: &str,
        token: Option<&str>,
        reason: &str,
    ) -> Result<Reverted, StoreError> {
        crate::backend::write_job!(self.backend, crate::backend::WriteLane::Normal, |tx| async {
            lock_account(tx, account_id).await?;
            // The message's current mailbox (and a move's base mailbox) are
            // locked before anything is written.
            let plan = q("SELECT mailbox_id FROM messages WHERE account_id=?1 AND stable_id=?2")
                .bind(account_id).bind(stable_id).fetch_opt_scalar_string_tx(tx).await?;
            let base_box = q("SELECT base_mailbox_id FROM upstream_outbox WHERE account_id=?1 AND stable_id=?2 AND field=?3")
                .bind(account_id).bind(stable_id).bind(field).fetch_all_tx(tx).await?
                .first().and_then(|r| r.get_opt_string("base_mailbox_id"));
            if let Some(current) = &plan {
                let other = base_box.clone().unwrap_or_else(|| current.clone());
                crate::publication_mutations::lock_mailboxes(tx, account_id, current, &other).await?;
            }
            let rows = lock_messages(tx, account_id, &[stable_id.to_string()]).await?;
            let Some(row) = read_intent(tx, account_id, stable_id, field).await? else {
                return Ok(Reverted::Missing);
            };
            if let Some(token) = token {
                if row.state != IntentState::Inflight || row.claim.as_deref() != Some(token) {
                    return Ok(Reverted::Stale);
                }
            }
            let mut head = read_head(tx, account_id).await?.unwrap_or_else(default_head);
            let message = rows.into_iter().next();
            let mut email_state = None;
            if let (Some(key), Some(message)) = (flag_of_field(field), message.as_ref()) {
                let target = row.shadow.as_deref().or(row.base.as_deref()) == Some("1");
                let flags = with_flag(&message.flags_json, key, target)?;
                if flags != message.flags_json {
                    write_flags_tx(tx, account_id, message, &flags).await?;
                    email_state = record_changes_tx(tx, account_id, "Email", &[stable_id.to_string()], "updated").await?;
                }
            } else if let (true, Some(message)) = (field == FIELD_LOCATION, message.as_ref()) {
                let base = row.base_location.clone();
                let base_exists = match &base {
                    Some(b) => q("SELECT id FROM mailboxes WHERE account_id=?1 AND id=?2").bind(account_id).bind(&b.mailbox_id).fetch_opt_scalar_string_tx(tx).await?.is_some(),
                    None => false,
                };
                let restore = match base {
                    Some(b) if base_exists && row.shadow.as_deref() != Some(SHADOW_GONE) && !coords_taken(tx, account_id, &b, stable_id).await? => Some(b),
                    _ => None,
                };
                let unread = i64::from(crate::cache::flags_json_is_unread(&message.flags_json));
                match restore {
                    Some(b) => {
                        q("UPDATE messages SET mailbox_id=?3, uidvalidity=?4, uid=?5 WHERE account_id=?1 AND stable_id=?2")
                            .bind(account_id).bind(stable_id).bind(&b.mailbox_id)
                            .bind(i64::from(b.uidvalidity)).bind(i64::from(b.uid))
                            .execute_tx(tx).await?;
                        if b.mailbox_id != message.mailbox_id {
                            crate::cache::shift_counters_tx(tx, account_id, &message.mailbox_id, -1, -unread).await?;
                            crate::cache::shift_counters_tx(tx, account_id, &b.mailbox_id, 1, unread).await?;
                        }
                        email_state = record_changes_tx(tx, account_id, "Email", &[stable_id.to_string()], "updated").await?;
                        let mut boxes = vec![message.mailbox_id.clone()];
                        if b.mailbox_id != message.mailbox_id {
                            boxes.push(b.mailbox_id.clone());
                        }
                        record_changes_tx(tx, account_id, "Mailbox", &boxes, "updated").await?;
                    }
                    None => {
                        // Gone from the server (or its folder is): what sync would
                        // have shown is no message. Its other intents go with it.
                        q("DELETE FROM messages WHERE account_id=?1 AND stable_id=?2").bind(account_id).bind(stable_id).execute_tx(tx).await?;
                        q("DELETE FROM message_embeddings WHERE stable_id=?1 AND account_id=?2").bind(stable_id).bind(account_id).execute_tx(tx).await?;
                        crate::cache::shift_counters_tx(tx, account_id, &message.mailbox_id, -1, -unread).await?;
                        email_state = record_changes_tx(tx, account_id, "Email", &[stable_id.to_string()], "destroyed").await?;
                        record_changes_tx(tx, account_id, "Mailbox", std::slice::from_ref(&message.mailbox_id), "updated").await?;
                        let others = q("DELETE FROM upstream_outbox WHERE account_id=?1 AND stable_id=?2 AND field<>?3")
                            .bind(account_id).bind(stable_id).bind(field).execute_tx(tx).await?;
                        head.backlog -= others as i64;
                    }
                }
            }
            delete_intent(tx, account_id, stable_id, field).await?;
            head.backlog -= 1;
            let notice_id = insert_notice(tx, account_id, Some(stable_id), notice_op(field), reason).await?;
            write_head(tx, account_id, &head).await?;
            Ok::<_, StoreError>(Reverted::Done { email_state, notice_id })
        })
    }

    /// Record what sync saw on the server for masked fields (`shadow`), for
    /// existing rows only. `None` clears it.
    pub async fn set_upstream_shadow(&self, account_id: &str, items: &[(String, String, Option<String>)]) -> Result<u64, StoreError> {
        crate::backend::write_job!(self.backend, crate::backend::WriteLane::Normal, |tx| async {
            lock_account(tx, account_id).await?;
            let mut n = 0;
            for (stable_id, field, shadow) in items {
                n += q("UPDATE upstream_outbox SET shadow=?4, updated_at=?5 WHERE account_id=?1 AND stable_id=?2 AND field=?3")
                    .bind(account_id).bind(stable_id).bind(field).bind(shadow.as_deref()).bind(now())
                    .execute_tx(tx).await?;
            }
            Ok::<_, StoreError>(n)
        })
    }

    /// Delete every intent of these messages (an upstream expunge removed them,
    /// plan §4.2 row 7). Returns the rows deleted.
    pub async fn drop_upstream_intents(&self, account_id: &str, stable_ids: &[String]) -> Result<u64, StoreError> {
        crate::backend::write_job!(self.backend, crate::backend::WriteLane::Normal, |tx| async {
            lock_account(tx, account_id).await?;
            drop_intents_tx(tx, account_id, stable_ids).await
        })
    }

    /// Pause (`"auth"` or `"operator"`) or resume (`None`) the account's applier.
    /// Intents keep accumulating while paused; nothing is claimed.
    pub async fn set_upstream_paused(&self, account_id: &str, paused: Option<&str>) -> Result<(), StoreError> {
        if paused.is_some_and(|p| p != "auth" && p != "operator") {
            return Err(StoreError::Invalid("pause reason is auth or operator"));
        }
        crate::backend::write_job!(self.backend, crate::backend::WriteLane::Normal, |tx| async {
            lock_account(tx, account_id).await?;
            let at = paused.map(|_| now());
            q("INSERT INTO upstream_outbox_head (account_id, paused, paused_at) VALUES (?1, ?2, ?3)
               ON CONFLICT (account_id) DO UPDATE SET paused=excluded.paused, paused_at=excluded.paused_at")
                .bind(account_id).bind(paused).bind(at).execute_tx(tx).await?;
            Ok::<_, StoreError>(())
        })
    }

    /// The account's notices, newest first.
    pub async fn upstream_notices(&self, account_id: &str, limit: i64) -> Result<Vec<UpstreamNotice>, StoreError> {
        let rows = q("SELECT id, account_id, at, stable_id, op, reason, count, seen_at FROM upstream_notice WHERE account_id=?1 ORDER BY at DESC, id LIMIT ?2")
            .bind(account_id)
            .bind(limit)
            .fetch_all(&self.backend)
            .await?;
        Ok(rows
            .iter()
            .map(|r| UpstreamNotice {
                id: r.get_string("id"),
                account_id: r.get_string("account_id"),
                at: r.get_string("at"),
                stable_id: r.get_opt_string("stable_id"),
                op: r.get_string("op"),
                reason: r.get_string("reason"),
                count: r.get_i64("count"),
                seen_at: r.get_opt_string("seen_at"),
            })
            .collect())
    }

    /// Mark one notice seen. Returns whether it existed.
    pub async fn dismiss_upstream_notice(&self, account_id: &str, id: &str) -> Result<bool, StoreError> {
        Ok(q("UPDATE upstream_notice SET seen_at=?3 WHERE account_id=?1 AND id=?2 AND seen_at IS NULL")
            .bind(account_id)
            .bind(id)
            .bind(now())
            .execute(&self.backend)
            .await?
            > 0)
    }

    /// The sealed server text of an intent's last error (operator view).
    pub async fn upstream_error_detail(&self, account_id: &str, stable_id: &str, field: &str) -> Result<Option<String>, StoreError> {
        let rows = q("SELECT error_detail FROM upstream_outbox WHERE account_id=?1 AND stable_id=?2 AND field=?3")
            .bind(account_id)
            .bind(stable_id)
            .bind(field)
            .fetch_all(&self.backend)
            .await?;
        let Some(sealed) = rows.first().and_then(|r| r.get_opt_blob("error_detail")) else {
            return Ok(None);
        };
        let plain = self.key.open_bound(&sealed, SEAL_PURPOSE, &[account_id, stable_id, field])?;
        Ok(Some(String::from_utf8_lossy(&plain).into_owned()))
    }
}

async fn mark_collision(tx: &mut Tx, account_id: &str, stable_id: &str, field: &str, at: &ServerCoords) -> Result<(), StoreError> {
    // Keep the reported coordinates only when no other intent holds them.
    let free = !base_taken(tx, account_id, at, (stable_id, field)).await?;
    if free {
        q("UPDATE upstream_outbox SET state='unresolved', claim=NULL, error_class='collision', base_mailbox_id=?4, base_uidvalidity=?5, base_uid=?6, updated_at=?7
           WHERE account_id=?1 AND stable_id=?2 AND field=?3")
            .bind(account_id).bind(stable_id).bind(field)
            .bind(&at.mailbox_id).bind(i64::from(at.uidvalidity)).bind(i64::from(at.uid)).bind(now())
            .execute_tx(tx).await?;
    } else {
        q("UPDATE upstream_outbox SET state='unresolved', claim=NULL, error_class='collision', updated_at=?4 WHERE account_id=?1 AND stable_id=?2 AND field=?3")
            .bind(account_id).bind(stable_id).bind(field).bind(now())
            .execute_tx(tx).await?;
    }
    Ok(())
}

/// Delete every intent of `stable_ids` inside the caller's transaction (which
/// holds the account lock) and keep the head's backlog in step.
pub(crate) async fn drop_intents_tx(tx: &mut Tx, account_id: &str, stable_ids: &[String]) -> Result<u64, StoreError> {
    if stable_ids.is_empty() {
        return Ok(0);
    }
    let sql = if is_pg(tx) {
        "DELETE FROM upstream_outbox WHERE account_id=?1 AND stable_id IN (SELECT value FROM json_array_elements_text(?2::json))"
    } else {
        "DELETE FROM upstream_outbox WHERE account_id=?1 AND stable_id IN (SELECT value FROM json_each(?2))"
    };
    let n = q(sql).bind(account_id).bind(ids_json(stable_ids)).execute_tx(tx).await?;
    if n > 0 {
        q("UPDATE upstream_outbox_head SET backlog=CASE WHEN backlog>=?2 THEN backlog-?2 ELSE 0 END WHERE account_id=?1")
            .bind(account_id)
            .bind(n as i64)
            .execute_tx(tx)
            .await?;
    }
    Ok(n)
}

#[cfg(test)]
#[path = "upstream_outbox_tests.rs"]
mod tests;
