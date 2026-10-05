//! The reader for the admin panel's per-account flags (t27-e3, audit OH-1).
//!
//! The panel writes two flags that decide whether an account may be used:
//! `disabled` and `force_password_change` ([`mw_admin::UserFeatureFlags`]). This
//! module is where they are read, and every path that establishes or uses a mailbox
//! principal goes through it:
//!
//! | path | call site |
//! |---|---|
//! | password / header-auth login, both modes | `twofa_routes::gate_login` |
//! | second-factor completion, forced enrolment | `twofa_routes::complete_login` |
//! | SSO callback | `sso::complete_flow` |
//! | cookie session, native bearer (every `authed` caller) | `crate::authed_with_gate` |
//! | API key, OAuth access / refresh token, authorization code | `stores_v6::OAuthStoreAdapter` |
//! | API key on `/api/v1`, key or token on `/mcp` (the hold) | `scope_mw` guards |
//!
//! ## One identifier
//!
//! The panel names an account `username@domain`. A session's `account_id` is a
//! different thing — a store-generated id in engine mode, the upstream's account id
//! in proxy mode. The identifier the two have in common is the **login name**, so
//! flags are stored and looked up under [`flag_subject`] of it: trimmed and
//! lowercased, the same on the writing and the reading side.
//!
//! ## Every name of a principal
//!
//! A mail server may accept more than one login name for one mailbox (`alice` and
//! `alice@example.org`, or an alias), and may report yet another. The principal is
//! the account id; a flag set under any name that account id is known by applies
//! to every credential of it. The names are:
//!
//! * the account id itself,
//! * the engine account's `username`,
//! * every name recorded at a login whose credentials were accepted — the name
//!   typed and the name the upstream reported ([`remember_login`], kept in the
//!   store by `Store::remember_account_names`), and
//! * for a key or token, the two names on each live session of the account (which
//!   covers sessions opened before 26.20 started recording names).
//!
//! The record outlives the sessions, so a key or token of a disabled account is
//! refused by name after its sessions are gone.
//!
//! ## What this cannot see
//!
//! A name the mail server would accept for the mailbox but that nobody has logged
//! in with here is unknown: a flag set under it applies from the first login that
//! uses it. Two upstream servers that hand out the same account id are one
//! principal to this server (as they are to its keys and second factors).
//! Disabling an account here does not disable the mailbox on the mail server.
//!
//! ## Failing closed
//!
//! Every function returns `Err` when the store cannot be read, and every caller
//! treats `Err` as a refusal. A store outage fails logins and requests; it does not
//! let them through unchecked.

use axum::Json;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use serde_json::json;

use mw_admin::{ActorKind, AuditEvent, AuditKind, UserFeatureFlags};
use mw_store::{Session, Store, StoreError};

use crate::AppState;

/// The settings-key prefix the flags are stored under (no dedicated table).
const FLAGS_PREFIX: &str = "v6:admin:flags:";

/// The one normalisation of an account's name used by the flag writer and every
/// reader: surrounding whitespace removed, lowercased. The store folds the names
/// it records for an account the same way, so this is that function.
pub(crate) fn flag_subject(username: &str) -> String {
    Store::fold_account_name(username)
}

fn flags_key(id: &str) -> String {
    format!("{FLAGS_PREFIX}{id}")
}

fn parse_flags(raw: &str) -> Result<UserFeatureFlags, StoreError> {
    serde_json::from_str(raw)
        .map_err(|e| StoreError::Corrupt(format!("account flags are not valid JSON: {e}")))
}

/// The flags stored for `account_id` (the panel's `username@domain`, or a login
/// name). Absent ⇒ all clear.
///
/// Before 26.20 the key was the id exactly as the panel sent it. A record written
/// then under a mixed-case id is still found when the caller presents that same
/// spelling: the normalised key is read first, the exact spelling second.
pub(crate) async fn read_flags(
    store: &Store,
    account_id: &str,
) -> Result<UserFeatureFlags, StoreError> {
    let subject = flag_subject(account_id);
    if let Some(raw) = store.get_setting(&flags_key(&subject)).await? {
        return parse_flags(&raw);
    }
    if account_id != subject
        && let Some(raw) = store.get_setting(&flags_key(account_id)).await?
    {
        return parse_flags(&raw);
    }
    Ok(UserFeatureFlags::default())
}

/// Store `flags` for `account_id` under its normalised key. A pre-26.20 record
/// under the exact spelling is overwritten with the same value so the two cannot
/// disagree.
pub(crate) async fn write_flags(
    store: &Store,
    account_id: &str,
    flags: UserFeatureFlags,
) -> Result<(), StoreError> {
    let json = serde_json::to_string(&flags)
        .map_err(|e| StoreError::Corrupt(format!("account flags did not serialise: {e}")))?;
    let subject = flag_subject(account_id);
    store.set_setting(&flags_key(&subject), &json).await?;
    if account_id != subject && store.get_setting(&flags_key(account_id)).await?.is_some() {
        store.set_setting(&flags_key(account_id), &json).await?;
    }
    Ok(())
}

/// How many times [`update_flags`] re-reads and retries when another writer
/// changed the record between its read and its write.
const UPDATE_ATTEMPTS: usize = 16;

/// Change part of the flags stored for `account_id` without losing a concurrent
/// change to another part: `change` is applied to the record as stored, and the
/// result is written only if the record is still the one that was read
/// (`Store::compare_and_set_setting`); otherwise it is read again and `change`
/// re-applied. Returns the flags before and after.
///
/// [`write_flags`] replaces the whole record, which is what the panel's "save"
/// means. A writer that means one field — clearing `force_password_change` after
/// a password change — must use this instead: read-then-[`write_flags`] would put
/// back a stale `disabled: false` over an admin's `disabled: true` written in
/// between.
pub(crate) async fn update_flags(
    store: &Store,
    account_id: &str,
    change: impl Fn(&mut UserFeatureFlags),
) -> Result<(UserFeatureFlags, UserFeatureFlags), StoreError> {
    let subject = flag_subject(account_id);
    let key = flags_key(&subject);
    for _ in 0..UPDATE_ATTEMPTS {
        let stored = store.get_setting(&key).await?;
        let before = match &stored {
            Some(raw) => parse_flags(raw)?,
            // No record under the normalised key: start from a pre-26.20 record
            // under the exact spelling, if there is one.
            None => read_flags(store, account_id).await?,
        };
        let mut after = before;
        change(&mut after);
        if after == before {
            return Ok((before, after));
        }
        let json = serde_json::to_string(&after)
            .map_err(|e| StoreError::Corrupt(format!("account flags did not serialise: {e}")))?;
        if store
            .compare_and_set_setting(&key, stored.as_deref(), &json)
            .await?
        {
            if account_id != subject && store.get_setting(&flags_key(account_id)).await?.is_some() {
                store.set_setting(&flags_key(account_id), &json).await?;
            }
            return Ok((before, after));
        }
    }
    Err(StoreError::Corrupt(format!(
        "account flags changed under {UPDATE_ATTEMPTS} successive updates"
    )))
}

/// What the flags mean for one principal.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub(crate) struct AccountGate {
    /// The account is administratively disabled: no login, no session, no key, no
    /// token.
    pub disabled: bool,
    /// The account must change its password before it may do anything else.
    pub password_change_required: bool,
}

/// The distinct, non-empty flag subjects among `names`.
fn subjects(names: &[&str]) -> Vec<String> {
    let mut out: Vec<String> = Vec::with_capacity(names.len());
    for name in names {
        let s = flag_subject(name);
        if !s.is_empty() && !out.contains(&s) {
            out.push(s);
        }
    }
    out
}

/// The gate for a principal known by any of `names`: a flag set under any one of
/// them applies.
async fn gate_for(store: &Store, names: &[&str]) -> Result<AccountGate, StoreError> {
    let mut gate = AccountGate::default();
    for subject in subjects(names) {
        let flags = read_flags(store, &subject).await?;
        gate.disabled |= flags.disabled;
        gate.password_change_required |= flags.force_password_change;
    }
    Ok(gate)
}

/// Record the names a login was made under against its account id, so that a flag
/// set under either applies to every credential of the account from now on (see
/// the module docs). Call once the credentials have been accepted and before the
/// gate is read.
pub(crate) async fn remember_login(
    store: &Store,
    account_id: &str,
    username: &str,
    login_name: &str,
) -> Result<(), StoreError> {
    store
        .remember_account_names(account_id, &[username, login_name])
        .await
}

/// The gate for a login in progress: `account_id` is the principal, `username` the
/// name the session will carry (what the upstream reported, or the asserted
/// identity), `login_name` the name the credentials were presented under. A flag
/// under any name the account is known by applies, not only under these two.
pub(crate) async fn for_login(
    store: &Store,
    account_id: &str,
    username: &str,
    login_name: &str,
) -> Result<AccountGate, StoreError> {
    let mut names = principal_names(store, account_id).await?;
    names.push(username.to_string());
    names.push(login_name.to_string());
    gate_for_names(store, &names).await
}

/// The names an established session is gated under: its own two, its account id,
/// and the names recorded for that account.
async fn session_names(store: &Store, session: &Session) -> Result<Vec<String>, StoreError> {
    let mut names = vec![
        session.username.clone(),
        session.credentials.username.clone(),
        session.account_id.clone(),
    ];
    names.extend(store.account_names(&session.account_id).await?);
    Ok(names)
}

/// The gate for an established session.
pub(crate) async fn for_session(
    store: &Store,
    session: &Session,
) -> Result<AccountGate, StoreError> {
    gate_for_names(store, &session_names(store, session).await?).await
}

/// Every name the account `account_id` is known by (module docs, "Every name of a
/// principal"), for principals that carry only the id (an API key, an OAuth
/// token). Holds the id itself; an empty id is no principal and has no names (a
/// proxy-mode upstream that names no mail account leaves sessions with an empty
/// account id, and those must not be read as one account).
pub(crate) async fn principal_names(
    store: &Store,
    account_id: &str,
) -> Result<Vec<String>, StoreError> {
    if account_id.is_empty() {
        return Ok(Vec::new());
    }
    let mut names = vec![account_id.to_string()];
    match store.get_account(account_id).await {
        Ok(account) => names.push(account.username),
        Err(StoreError::NotFound) => {}
        Err(e) => return Err(e),
    }
    names.extend(store.account_names(account_id).await?);
    for session in store.sessions_by_account(account_id).await? {
        names.push(session.username);
        names.push(session.credentials.username);
    }
    Ok(names)
}

async fn gate_for_names(store: &Store, names: &[String]) -> Result<AccountGate, StoreError> {
    let names: Vec<&str> = names.iter().map(String::as_str).collect();
    gate_for(store, &names).await
}

/// The gate for a key or token principal: a flag under any name the account is
/// known by applies. This does not depend on a live session, so a disabled
/// account's key is refused by name after its sessions have been deleted.
pub(crate) async fn for_account(
    store: &Store,
    account_id: &str,
) -> Result<AccountGate, StoreError> {
    gate_for_names(store, &principal_names(store, account_id).await?).await
}

/// Stop the engine's work for a disabled account: drop its runtime, which ends its
/// watch loop and background sync. Proxy mode has no engine and nothing to stop.
///
/// Called where this module's callers refuse a disabled account. The account is
/// connected again by the first request after it is re-enabled
/// (`engine_mode::ensure_account`) or at the next start.
pub(crate) fn stop_engine_work(state: &AppState, account_id: &str) {
    if let Some(engine) = &state.engine
        && engine.unregister(account_id).is_some()
    {
        tracing::info!("account {account_id} is disabled; its engine runtime was dropped");
    }
}

/// Clear `force_password_change` for the account behind `session`, after it has
/// changed its password: under every name the session is gated under, since the
/// hold may come from any of them.
///
/// Only that one field is changed ([`update_flags`]); a `disabled` the admin set
/// while the password change was in flight is kept. Each record that was changed
/// gets the audit entry the panel's own "force password change" toggle writes,
/// with `system` as the actor.
pub(crate) async fn clear_password_change(
    state: &AppState,
    session: &Session,
) -> Result<(), mw_admin::AdminError> {
    let store_err = |e: StoreError| mw_admin::AdminError::Store(e.to_string());
    let names = session_names(&state.store, session)
        .await
        .map_err(store_err)?;
    let names: Vec<&str> = names.iter().map(String::as_str).collect();
    for subject in subjects(&names) {
        let (before, _) = update_flags(&state.store, &subject, |f| {
            f.force_password_change = false;
        })
        .await
        .map_err(store_err)?;
        if before.force_password_change {
            state
                .v6
                .admin
                .record(
                    AuditEvent::new("system", ActorKind::Admin, AuditKind::ForcePasswordChange)
                        .detail(json!({ "enabled": false }))
                        .target(subject),
                )
                .await?;
        }
    }
    Ok(())
}

/// The refusal a held account gets on everything but the password-change routes.
/// The web client keys on the body field, not the status.
pub(crate) fn password_change_required() -> Response {
    (
        StatusCode::FORBIDDEN,
        Json(json!({ "error": "password change required", "passwordChangeRequired": true })),
    )
        .into_response()
}

/// The refusal when the flags could not be read. Not a 401: the credential may be
/// good, and a client should retry rather than discard its session.
pub(crate) fn unavailable(what: &str, e: impl std::fmt::Display) -> Response {
    tracing::error!("account flags unreadable ({what}), refusing: {e}");
    (
        StatusCode::SERVICE_UNAVAILABLE,
        Json(json!({ "error": "server error" })),
    )
        .into_response()
}

#[cfg(test)]
mod tests {
    use super::*;
    use mw_store::{Credentials, ServerKey};

    async fn store() -> Store {
        Store::open_in_memory(ServerKey::generate()).await.unwrap()
    }

    fn disabled() -> UserFeatureFlags {
        UserFeatureFlags {
            disabled: true,
            ..UserFeatureFlags::default()
        }
    }

    #[test]
    fn subject_is_trimmed_and_lowercased() {
        assert_eq!(flag_subject("  Alice@Example.ORG "), "alice@example.org");
        assert_eq!(
            subjects(&["Alice@x", "alice@X", "", " ", "bob"]),
            vec!["alice@x".to_string(), "bob".to_string()]
        );
    }

    #[tokio::test]
    async fn writer_and_reader_agree_across_case() {
        let s = store().await;
        assert!(!read_flags(&s, "alice@example.org").await.unwrap().disabled);
        write_flags(&s, "Alice@Example.org", disabled())
            .await
            .unwrap();
        assert!(read_flags(&s, "ALICE@example.org").await.unwrap().disabled);
        assert!(
            for_login(&s, "acct", "alice@example.org", "alice@example.org")
                .await
                .unwrap()
                .disabled
        );
        assert!(!read_flags(&s, "bob@example.org").await.unwrap().disabled);
    }

    /// A record written before 26.20 sits under the id exactly as the panel sent
    /// it. It is read under that spelling, and the next write replaces it.
    #[tokio::test]
    async fn a_pre_normalisation_record_is_read_and_then_superseded() {
        let s = store().await;
        s.set_setting(
            "v6:admin:flags:Carol@Example.org",
            &serde_json::to_string(&disabled()).unwrap(),
        )
        .await
        .unwrap();
        assert!(read_flags(&s, "Carol@Example.org").await.unwrap().disabled);

        write_flags(&s, "Carol@Example.org", UserFeatureFlags::default())
            .await
            .unwrap();
        assert!(!read_flags(&s, "Carol@Example.org").await.unwrap().disabled);
        assert!(!read_flags(&s, "carol@example.org").await.unwrap().disabled);
    }

    /// A flag under either the session's name or the name the credentials were
    /// presented under applies to the session.
    #[tokio::test]
    async fn a_session_is_gated_under_either_of_its_names() {
        let s = store().await;
        let session = |username: &str, login: &str| Session {
            id: String::new(),
            account_id: "acct".into(),
            username: username.into(),
            jmap_url: String::new(),
            api_url: String::new(),
            credentials: Credentials {
                username: login.into(),
                password: String::new(),
            },
        };
        let sess = session("alice@example.org", "alice");
        assert_eq!(
            for_session(&s, &sess).await.unwrap(),
            AccountGate::default()
        );
        write_flags(&s, "alice", disabled()).await.unwrap();
        assert!(for_session(&s, &sess).await.unwrap().disabled);
        assert!(
            !for_session(&s, &session("bob@example.org", "bob"))
                .await
                .unwrap()
                .disabled
        );
    }

    /// Unparseable flags are an error, which callers treat as a refusal.
    #[tokio::test]
    async fn corrupt_flags_are_an_error_not_all_clear() {
        let s = store().await;
        s.set_setting("v6:admin:flags:dave@example.org", "{not json")
            .await
            .unwrap();
        assert!(read_flags(&s, "dave@example.org").await.is_err());
        assert!(for_account(&s, "dave@example.org").await.is_err());
    }

    fn creds(name: &str) -> Credentials {
        Credentials {
            username: name.into(),
            password: "p".into(),
        }
    }

    /// A key principal carries an account id; it is gated through the login name
    /// of a live session for that account.
    #[tokio::test]
    async fn an_account_id_is_gated_through_its_sessions_login_name() {
        let s = store().await;
        s.create_session(
            "upstream-17",
            "erin@example.org",
            "http://u",
            "http://u",
            &creds("erin@example.org"),
        )
        .await
        .unwrap();
        assert!(
            principal_names(&s, "upstream-17")
                .await
                .unwrap()
                .contains(&"erin@example.org".to_string())
        );
        assert!(!for_account(&s, "upstream-17").await.unwrap().disabled);
        write_flags(&s, "erin@example.org", disabled())
            .await
            .unwrap();
        assert!(for_account(&s, "upstream-17").await.unwrap().disabled);
        assert_eq!(
            principal_names(&s, "nobody").await.unwrap(),
            vec!["nobody".to_string()]
        );
        // Sessions without an account id are not one principal.
        s.create_session("", "zed@example.org", "http://u", "http://u", &creds("zed"))
            .await
            .unwrap();
        assert!(principal_names(&s, "").await.unwrap().is_empty());
    }

    /// O1 (t27-f1): the admin flags the name the user typed; the upstream reports
    /// another. A key carries only the account id, and must be refused both while
    /// the session lives (through the name sealed in its credentials) and after
    /// it is gone (through the names recorded at login).
    #[tokio::test]
    async fn a_key_is_gated_under_the_name_typed_at_login() {
        let s = store().await;
        remember_login(&s, "upstream-9", "frank@example.org", "Frank")
            .await
            .unwrap();
        let id = s
            .create_session(
                "upstream-9",
                "frank@example.org",
                "http://u",
                "http://u",
                &creds("Frank"),
            )
            .await
            .unwrap();
        assert!(!for_account(&s, "upstream-9").await.unwrap().disabled);
        write_flags(&s, "frank", disabled()).await.unwrap();
        assert!(
            for_account(&s, "upstream-9").await.unwrap().disabled,
            "refused while the session lives"
        );
        s.delete_session(&id).await.unwrap();
        assert!(
            for_account(&s, "upstream-9").await.unwrap().disabled,
            "and after it is gone"
        );
        assert!(!for_account(&s, "upstream-10").await.unwrap().disabled);
    }

    /// A session opened before names were recorded has only its own row to go by:
    /// the name sealed in a live session's credentials still gates a key.
    #[tokio::test]
    async fn a_key_is_gated_through_a_live_sessions_presented_name() {
        let s = store().await;
        s.create_session(
            "upstream-11",
            "gina@example.org",
            "http://u",
            "http://u",
            &creds("gina"),
        )
        .await
        .unwrap();
        write_flags(&s, "GINA", disabled()).await.unwrap();
        assert!(for_account(&s, "upstream-11").await.unwrap().disabled);
    }

    /// A flag under one name of a principal applies to a session and to a login
    /// made under another of its names, and to its account id.
    #[tokio::test]
    async fn every_credential_of_a_principal_is_gated_under_any_of_its_names() {
        let s = store().await;
        remember_login(&s, "upstream-12", "hal@example.org", "hal")
            .await
            .unwrap();
        // This session was opened under the reported name only.
        let other = Session {
            id: String::new(),
            account_id: "upstream-12".into(),
            username: "hal@example.org".into(),
            jmap_url: String::new(),
            api_url: String::new(),
            credentials: creds("hal@example.org"),
        };
        assert!(!for_session(&s, &other).await.unwrap().disabled);
        write_flags(&s, "hal", disabled()).await.unwrap();
        assert!(for_session(&s, &other).await.unwrap().disabled);
        assert!(
            for_login(&s, "upstream-12", "hal@example.org", "hal@example.org")
                .await
                .unwrap()
                .disabled
        );
        write_flags(&s, "hal", UserFeatureFlags::default())
            .await
            .unwrap();
        assert!(!for_session(&s, &other).await.unwrap().disabled);
        // The account id, in another case.
        write_flags(&s, "UPSTREAM-12", disabled()).await.unwrap();
        assert!(for_session(&s, &other).await.unwrap().disabled);
        assert!(for_account(&s, "upstream-12").await.unwrap().disabled);
    }

    /// O5 (t27-f1): an admin's `disabled: true` written between the read and the
    /// write of a one-field update is kept. `change` runs once per attempt, so the
    /// competing write is made from inside its first call.
    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn a_one_field_update_keeps_a_concurrent_change_to_another_field() {
        let s = store().await;
        write_flags(
            &s,
            "ivy@example.org",
            UserFeatureFlags {
                force_password_change: true,
                ..UserFeatureFlags::default()
            },
        )
        .await
        .unwrap();

        let calls = std::sync::atomic::AtomicUsize::new(0);
        let (before, after) = update_flags(&s, "Ivy@example.org", |f| {
            if calls.fetch_add(1, std::sync::atomic::Ordering::SeqCst) == 0 {
                // The admin disables the account after this update read the record.
                let s = s.clone();
                tokio::task::block_in_place(|| {
                    tokio::runtime::Handle::current().block_on(async {
                        write_flags(
                            &s,
                            "ivy@example.org",
                            UserFeatureFlags {
                                force_password_change: true,
                                disabled: true,
                                ..UserFeatureFlags::default()
                            },
                        )
                        .await
                        .unwrap();
                    });
                });
            }
            f.force_password_change = false;
        })
        .await
        .unwrap();

        assert_eq!(
            calls.load(std::sync::atomic::Ordering::SeqCst),
            2,
            "the first write was refused and the update re-read"
        );
        assert!(before.disabled && before.force_password_change);
        assert!(after.disabled && !after.force_password_change);
        let stored = read_flags(&s, "ivy@example.org").await.unwrap();
        assert!(stored.disabled, "the admin's disable survived");
        assert!(!stored.force_password_change);
    }

    /// An update that changes nothing writes nothing, and one on an account with
    /// no record creates it.
    #[tokio::test]
    async fn update_flags_on_absent_and_unchanged_records() {
        let s = store().await;
        let (before, after) = update_flags(&s, "jo@example.org", |f| {
            f.force_password_change = false;
        })
        .await
        .unwrap();
        assert_eq!(before, after);
        assert_eq!(
            s.get_setting("v6:admin:flags:jo@example.org")
                .await
                .unwrap(),
            None
        );
        update_flags(&s, "jo@example.org", |f| f.disabled = true)
            .await
            .unwrap();
        assert!(read_flags(&s, "JO@example.org").await.unwrap().disabled);
    }
}
