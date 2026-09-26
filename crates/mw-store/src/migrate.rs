//! `mailwoman migrate-store` engine (t6-e1; plan §1.1, §2.1, §4.2): copy a
//! populated SQLite store into the (Postgres) backend held by `self`, row-for-row,
//! preserving every id, timestamp, and sealed blob byte-for-byte.
//!
//! The copy runs inside a single destination transaction. On Postgres it issues
//! `SET CONSTRAINTS ALL DEFERRED` first (the schema declares every foreign key
//! `DEFERRABLE`), so cross-table and self-referential (`mailboxes.parent_id`)
//! references need not be inserted in dependency order — the whole graph is
//! validated at commit.
//!
//! **The copy is not the whole schema, but it is now nearly all of it.** `TABLES`
//! below is the complete list of what is copied: as of migration 0029 that is 59
//! tables out of the 76 the migrations create. The other 17 are left behind
//! deliberately, each with its reason recorded at its entry in
//! `NOT_MIGRATED_DELIBERATELY` in `tests/backend_parity.rs`: the admin panel's
//! separate identity domain, OAuth clients and tokens, API keys, webhooks, the
//! managed-domain/directory/egress/SSO configuration that names the OLD
//! deployment's hosts, and the plugin registries and their capability grants,
//! which are deny-by-default and re-approved by an operator.
//!
//! There is no longer an "undecided" category. An earlier version of this note
//! claimed "only the 0001–0006 tables are copied; the 0007 admin/OAuth/webhook
//! tables are provisioned empty"; that was never a full account of the split, and
//! both halves were wrong — `crypto_changes` is a 0005 table that was *not*
//! copied, while migrations 0008–0029 added tables the note never mentioned.
//!
//! Two gates in `tests/backend_parity.rs` keep this honest, and both assert
//! against the LIVE schema rather than against this module's own report:
//! `migrate_store_accounts_for_every_schema_table` fails if a table appears in
//! neither `TABLES` nor the deliberate list, and
//! `migrate_store_copies_every_column_of_every_copied_table` does the same one
//! level down for the columns of each copied table. A new migration therefore
//! cannot quietly join the left-behind set.
//!
//! How the left-behind set shrank from 41 tables to 17 during 26.20, in the order
//! the reasons were established: `zeroaccess_accounts` first, because it holds the
//! only copy of each account's wrapped root key and without it the destination
//! cannot decrypt zero-access mail at all; then `crypto_changes`, `audit_log`
//! (append-only by invariant), and `twofa_policy`/`quotas`, whose absence silently
//! *relaxed* a protection. Copying the require-2FA policy without the enrolments
//! then produced a lockout of its own, so `totp_secrets`, `webauthn_credentials`
//! and `recovery_codes` followed. The remaining sixteen — per-account settings and
//! content, sealed account and bridge credentials, upload metadata, embeddings,
//! plugin state, Assist and cache configuration, and the last three append-only
//! audit logs — were copied once it was clear that every one of them was data a
//! store move should carry rather than configuration an operator re-enters.
//!
//! `plugin_grants` is the one table that moved the *other* way, to deliberate: it
//! grants capabilities to plugins in `plugins`, which is not copied, so copying it
//! would silently re-arm capabilities for a plugin the admin has not re-approved on
//! the new deployment. Its sibling `ui_plugin_grants` was already excluded for the
//! same fail-closed reason.

use crate::backend::{Arg, Backend, IntoArg, Row, Tx};
use crate::{MigrationReport, Store, StoreError, backend, q};

use sqlx::sqlite::SqlitePoolOptions;

impl Store {
    /// Copy every row from the SQLite store at `src_dsn` into this store's
    /// backend, returning a per-table row-count report for count + content parity
    /// assertions. `src_dsn` may be a bare path or a `sqlite:` URL.
    pub async fn migrate_from_sqlite(&self, src_dsn: &str) -> Result<MigrationReport, StoreError> {
        let url = if src_dsn.starts_with("sqlite:") {
            src_dsn.to_string()
        } else {
            format!("sqlite://{src_dsn}?mode=ro")
        };
        let src_pool = SqlitePoolOptions::new()
            .max_connections(1)
            .connect(&url)
            .await?;
        let src = Backend::Sqlite(src_pool);

        let mut tx = self.backend.begin().await?;
        if self.backend.dialect() == backend::Dialect::Postgres {
            // Defer FK validation to commit so copy order is irrelevant.
            q("SET CONSTRAINTS ALL DEFERRED")
                .execute_tx(&mut tx)
                .await?;
        }

        let mut report = MigrationReport::default();
        for spec in TABLES {
            let n = copy_table(&src, &mut tx, spec).await?;
            report.tables.push((spec.name.to_string(), n));
        }

        tx.commit().await?;
        Ok(report)
    }
}

/// A table to copy: its name, the ordered `SELECT`, the matching `INSERT`, and a
/// mapper turning a source [`Row`] into the `INSERT`'s positional [`Arg`]s.
struct TableSpec {
    name: &'static str,
    select: &'static str,
    insert: &'static str,
    map: fn(&Row) -> Vec<Arg>,
}

async fn copy_table(src: &Backend, tx: &mut Tx, spec: &TableSpec) -> Result<u64, StoreError> {
    let rows = q(spec.select).fetch_all(src).await?;
    let n = rows.len() as u64;
    for r in &rows {
        let mut query = q(spec.insert);
        for a in (spec.map)(r) {
            query = query.bind(a);
        }
        query.execute_tx(tx).await?;
    }
    Ok(n)
}

// Short helpers to keep the mappers legible.
fn t(r: &Row, c: &str) -> Arg {
    r.get_string(c).into_arg()
}
fn ot(r: &Row, c: &str) -> Arg {
    r.get_opt_string(c).into_arg()
}
fn i(r: &Row, c: &str) -> Arg {
    r.get_i64(c).into_arg()
}
fn b(r: &Row, c: &str) -> Arg {
    r.get_blob(c).into_arg()
}
fn ob(r: &Row, c: &str) -> Arg {
    r.get_opt_blob(c).into_arg()
}

/// Every table `migrate-store` copies, in FK-parent-first order (belt-and-braces
/// alongside the deferred constraints). This list is not the whole schema — see
/// the module note above for the two lists that account for the rest.
const TABLES: &[TableSpec] = &[
    TableSpec {
        name: "settings",
        select: "SELECT key, value FROM settings",
        insert: "INSERT INTO settings (key, value) VALUES (?1, ?2)",
        map: |r| vec![t(r, "key"), t(r, "value")],
    },
    TableSpec {
        name: "sessions",
        select: "SELECT id, account_id, username, jmap_url, api_url, sealed_creds, created_at, last_seen FROM sessions",
        insert: "INSERT INTO sessions (id, account_id, username, jmap_url, api_url, sealed_creds, created_at, last_seen) VALUES (?1,?2,?3,?4,?5,?6,?7,?8)",
        map: |r| {
            vec![
                t(r, "id"),
                t(r, "account_id"),
                t(r, "username"),
                t(r, "jmap_url"),
                t(r, "api_url"),
                b(r, "sealed_creds"),
                t(r, "created_at"),
                t(r, "last_seen"),
            ]
        },
    },
    TableSpec {
        name: "accounts",
        select: "SELECT id, kind, host, port, tls, username, sealed_creds, sync_policy_json FROM accounts",
        insert: "INSERT INTO accounts (id, kind, host, port, tls, username, sealed_creds, sync_policy_json) VALUES (?1,?2,?3,?4,?5,?6,?7,?8)",
        map: |r| {
            vec![
                t(r, "id"),
                t(r, "kind"),
                t(r, "host"),
                i(r, "port"),
                t(r, "tls"),
                t(r, "username"),
                b(r, "sealed_creds"),
                t(r, "sync_policy_json"),
            ]
        },
    },
    // `quotas` and `zeroaccess_accounts` are keyed by `account_id` but declare no
    // `REFERENCES accounts(id)` in either dialect, so nothing constrains their
    // position; they sit next to `accounts` because that is what they describe.
    TableSpec {
        name: "quotas",
        select: "SELECT account_id, bytes_limit, msg_limit FROM quotas",
        insert: "INSERT INTO quotas (account_id, bytes_limit, msg_limit) VALUES (?1,?2,?3)",
        map: |r| vec![t(r, "account_id"), i(r, "bytes_limit"), i(r, "msg_limit")],
    },
    TableSpec {
        name: "zeroaccess_accounts",
        // `wrapped_root_key` and `recovery_wrapped` are opaque to the store — it
        // never wraps or unwraps them. They are also the ONLY copy of the account's
        // client-derived root key, so leaving them behind made zero-access mail on
        // the destination permanently undecryptable. Copied byte-for-byte.
        select: "SELECT account_id, enabled, wrapped_root_key, kdf_params, recovery_wrapped, paired_devices FROM zeroaccess_accounts",
        insert: "INSERT INTO zeroaccess_accounts (account_id, enabled, wrapped_root_key, kdf_params, recovery_wrapped, paired_devices) VALUES (?1,?2,?3,?4,?5,?6)",
        map: |r| {
            vec![
                t(r, "account_id"),
                i(r, "enabled"),
                b(r, "wrapped_root_key"),
                t(r, "kdf_params"),
                ob(r, "recovery_wrapped"),
                t(r, "paired_devices"),
            ]
        },
    },
    // The 0015 login-2FA enrolments, copied alongside the `twofa_policy` that
    // requires them. Like `quotas`/`zeroaccess_accounts` above they are keyed by
    // `account_id` and declare no `REFERENCES` in either dialect, so they sit with
    // `accounts` for the same reason. Copying the policy without these three left
    // a migrated deployment demanding a second factor while holding no enrolment
    // and no recovery code — a lockout, and worse than either consistent state.
    TableSpec {
        name: "totp_secrets",
        // `sealed_secret` is sealed under the store's `ServerKey`, so it is only
        // usable on a destination holding the same key — which is already required
        // for every other sealed column. 0021's `last_step` is the replay guard: if
        // it reset to 0, a TOTP code already spent on the source could be presented
        // again on the destination, so it is copied with the secret.
        select: "SELECT account_id, sealed_secret, confirmed, created_at, last_step FROM totp_secrets",
        insert: "INSERT INTO totp_secrets (account_id, sealed_secret, confirmed, created_at, last_step) VALUES (?1,?2,?3,?4,?5)",
        map: |r| {
            vec![
                t(r, "account_id"),
                b(r, "sealed_secret"),
                i(r, "confirmed"),
                t(r, "created_at"),
                i(r, "last_step"),
            ]
        },
    },
    TableSpec {
        name: "webauthn_credentials",
        // `cose_public_key` is a public verification key, not a secret. `sign_count`
        // is the clone-detection counter and must never travel backwards, so it is
        // copied rather than defaulted: a destination that reset it to 0 would stop
        // noticing a cloned authenticator.
        select: "SELECT credential_id, account_id, cose_public_key, sign_count, transports, label, created_at FROM webauthn_credentials",
        insert: "INSERT INTO webauthn_credentials (credential_id, account_id, cose_public_key, sign_count, transports, label, created_at) VALUES (?1,?2,?3,?4,?5,?6,?7)",
        map: |r| {
            vec![
                t(r, "credential_id"),
                t(r, "account_id"),
                b(r, "cose_public_key"),
                i(r, "sign_count"),
                t(r, "transports"),
                t(r, "label"),
                t(r, "created_at"),
            ]
        },
    },
    TableSpec {
        name: "recovery_codes",
        // `used` is copied with the hash: a spent code must stay spent, or the copy
        // would silently un-consume every recovery code the account had burned.
        select: "SELECT account_id, code_hash, used, created_at FROM recovery_codes",
        insert: "INSERT INTO recovery_codes (account_id, code_hash, used, created_at) VALUES (?1,?2,?3,?4)",
        map: |r| {
            vec![
                t(r, "account_id"),
                t(r, "code_hash"),
                i(r, "used"),
                t(r, "created_at"),
            ]
        },
    },
    // ── Remaining per-account surfaces (26.20). None of these declares a
    // `REFERENCES` in either dialect — checked in `migrations/` and
    // `migrations_pg/` table by table — so their position is unconstrained; they
    // sit with `accounts` because that is what they hang off.
    TableSpec {
        name: "passwd_config",
        // `force_change` is the force-change-on-next-login flag: uncopied it cleared
        // silently, so a user the operator had flagged walked in unchallenged.
        select: "SELECT account_id, config, force_change, updated_at FROM passwd_config",
        insert: "INSERT INTO passwd_config (account_id, config, force_change, updated_at) VALUES (?1,?2,?3,?4)",
        map: |r| {
            vec![
                t(r, "account_id"),
                t(r, "config"),
                i(r, "force_change"),
                t(r, "updated_at"),
            ]
        },
    },
    TableSpec {
        name: "signatures",
        // User-authored content. The `identities.signature_*` columns beside it were
        // always copied, so leaving these behind lost half of one feature.
        select: "SELECT account_id, name, body, is_default, rule_json, updated_at FROM signatures",
        insert: "INSERT INTO signatures (account_id, name, body, is_default, rule_json, updated_at) VALUES (?1,?2,?3,?4,?5,?6)",
        map: |r| {
            vec![
                t(r, "account_id"),
                t(r, "name"),
                t(r, "body"),
                i(r, "is_default"),
                t(r, "rule_json"),
                t(r, "updated_at"),
            ]
        },
    },
    TableSpec {
        name: "notification_rules",
        select: "SELECT account_id, rule_json, quiet_hours_json, enabled, updated_at FROM notification_rules",
        insert: "INSERT INTO notification_rules (account_id, rule_json, quiet_hours_json, enabled, updated_at) VALUES (?1,?2,?3,?4,?5)",
        map: |r| {
            vec![
                t(r, "account_id"),
                t(r, "rule_json"),
                t(r, "quiet_hours_json"),
                i(r, "enabled"),
                t(r, "updated_at"),
            ]
        },
    },
    TableSpec {
        name: "remote_image_grants",
        // `revoked` is copied with the grant: a revoked grant must stay revoked, or
        // the copy would silently re-permit remote images the user had turned off.
        select: "SELECT account_id, scope_kind, scope_value, granted_at, revoked FROM remote_image_grants",
        insert: "INSERT INTO remote_image_grants (account_id, scope_kind, scope_value, granted_at, revoked) VALUES (?1,?2,?3,?4,?5)",
        map: |r| {
            vec![
                t(r, "account_id"),
                t(r, "scope_kind"),
                t(r, "scope_value"),
                t(r, "granted_at"),
                i(r, "revoked"),
            ]
        },
    },
    TableSpec {
        name: "masked_email",
        // The alias records are the user's. Mail already flowing to an alias keeps
        // arriving whether or not these rows travel; without them the destination
        // cannot attribute, disable or list the alias. Note that alias DELIVERY also
        // needs the `domains` routing, which is deliberately not copied — see the
        // operator note in docs/deploy/postgres.md.
        select: "SELECT id, account_id, alias_addr, target_desc, state, created_at, last_used_at FROM masked_email",
        insert: "INSERT INTO masked_email (id, account_id, alias_addr, target_desc, state, created_at, last_used_at) VALUES (?1,?2,?3,?4,?5,?6,?7)",
        map: |r| {
            vec![
                t(r, "id"),
                t(r, "account_id"),
                t(r, "alias_addr"),
                t(r, "target_desc"),
                t(r, "state"),
                t(r, "created_at"),
                ot(r, "last_used_at"),
            ]
        },
    },
    TableSpec {
        name: "ews_account_cred",
        // `sealed_cred` is the sealed {user, domain, password, workstation} quad — an
        // account binding in the same family as `accounts.sealed_creds`, which was
        // always copied. Opens on the destination under the shared `MW_SERVER_KEY`.
        select: "SELECT account_id, endpoint, endpoint_host, sealed_cred, enabled, created_at, updated_at FROM ews_account_cred",
        insert: "INSERT INTO ews_account_cred (account_id, endpoint, endpoint_host, sealed_cred, enabled, created_at, updated_at) VALUES (?1,?2,?3,?4,?5,?6,?7)",
        map: |r| {
            vec![
                t(r, "account_id"),
                t(r, "endpoint"),
                t(r, "endpoint_host"),
                b(r, "sealed_cred"),
                i(r, "enabled"),
                t(r, "created_at"),
                t(r, "updated_at"),
            ]
        },
    },
    TableSpec {
        name: "bridge_accounts",
        // `bridge_id` names a `plugins.id`, and `plugins` is deliberately NOT copied
        // (the operator re-approves plugins on the new deployment). Copying the
        // binding is still right: it is the account's configuration, and a bridge
        // cannot run until it is re-approved, so nothing is armed early.
        select: "SELECT account_id, bridge_id, oauth_ref, extra FROM bridge_accounts",
        insert: "INSERT INTO bridge_accounts (account_id, bridge_id, oauth_ref, extra) VALUES (?1,?2,?3,?4)",
        map: |r| {
            vec![
                t(r, "account_id"),
                t(r, "bridge_id"),
                ot(r, "oauth_ref"),
                t(r, "extra"),
            ]
        },
    },
    TableSpec {
        name: "bridge_oauth_tokens",
        // Sealed access + refresh tokens. Uncopied, every bridged account was forced
        // back through interactive re-consent after a backend swap.
        select: "SELECT bridge_account_id, sealed_access_token, sealed_refresh_token, expires_at, scope, updated_at FROM bridge_oauth_tokens",
        insert: "INSERT INTO bridge_oauth_tokens (bridge_account_id, sealed_access_token, sealed_refresh_token, expires_at, scope, updated_at) VALUES (?1,?2,?3,?4,?5,?6)",
        map: |r| {
            vec![
                t(r, "bridge_account_id"),
                b(r, "sealed_access_token"),
                b(r, "sealed_refresh_token"),
                t(r, "expires_at"),
                t(r, "scope"),
                t(r, "updated_at"),
            ]
        },
    },
    TableSpec {
        name: "uploaded_blobs",
        // Metadata for sealed attachment objects that live on the UPLOAD BACKEND,
        // not in this database. `storage_key` + `backend_kind` are how an object is
        // found, so without these rows the objects are unreachable and the
        // gc-uploads sweep has nothing to sweep. `migrate-store` moves the database
        // only: if the deployment also changes host or upload directory, the objects
        // must be moved alongside or these rows dangle. See docs/deploy/postgres.md.
        select: "SELECT blob_id, account_id, content_type, size, storage_key, backend_kind, created_at FROM uploaded_blobs",
        insert: "INSERT INTO uploaded_blobs (blob_id, account_id, content_type, size, storage_key, backend_kind, created_at) VALUES (?1,?2,?3,?4,?5,?6,?7)",
        map: |r| {
            vec![
                t(r, "blob_id"),
                t(r, "account_id"),
                t(r, "content_type"),
                i(r, "size"),
                t(r, "storage_key"),
                t(r, "backend_kind"),
                t(r, "created_at"),
            ]
        },
    },
    TableSpec {
        name: "mailboxes",
        select: "SELECT id, account_id, name, role, uidvalidity, uidnext, highestmodseq, total, unread, parent_id FROM mailboxes",
        insert: "INSERT INTO mailboxes (id, account_id, name, role, uidvalidity, uidnext, highestmodseq, total, unread, parent_id) VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9,?10)",
        map: |r| {
            vec![
                t(r, "id"),
                t(r, "account_id"),
                t(r, "name"),
                ot(r, "role"),
                i(r, "uidvalidity"),
                i(r, "uidnext"),
                i(r, "highestmodseq"),
                i(r, "total"),
                i(r, "unread"),
                ot(r, "parent_id"),
            ]
        },
    },
    TableSpec {
        name: "messages",
        select: "SELECT stable_id, account_id, mailbox_id, uid, uidvalidity, message_id, thread_id, internaldate, size, flags_json, envelope_json, blob_ref FROM messages",
        insert: "INSERT INTO messages (stable_id, account_id, mailbox_id, uid, uidvalidity, message_id, thread_id, internaldate, size, flags_json, envelope_json, blob_ref) VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11,?12)",
        map: |r| {
            vec![
                t(r, "stable_id"),
                t(r, "account_id"),
                t(r, "mailbox_id"),
                i(r, "uid"),
                i(r, "uidvalidity"),
                ot(r, "message_id"),
                ot(r, "thread_id"),
                ot(r, "internaldate"),
                i(r, "size"),
                t(r, "flags_json"),
                ob(r, "envelope_json"),
                ot(r, "blob_ref"),
            ]
        },
    },
    TableSpec {
        name: "bodies",
        select: "SELECT blob_ref, account_id, sealed_bytes FROM bodies",
        insert: "INSERT INTO bodies (blob_ref, account_id, sealed_bytes) VALUES (?1,?2,?3)",
        map: |r| vec![t(r, "blob_ref"), t(r, "account_id"), b(r, "sealed_bytes")],
    },
    TableSpec {
        name: "threads",
        select: "SELECT thread_id, account_id, root_message_id FROM threads",
        insert: "INSERT INTO threads (thread_id, account_id, root_message_id) VALUES (?1,?2,?3)",
        map: |r| {
            vec![
                t(r, "thread_id"),
                t(r, "account_id"),
                ot(r, "root_message_id"),
            ]
        },
    },
    TableSpec {
        name: "pop3_uidl",
        select: "SELECT account_id, uidl, stable_id FROM pop3_uidl",
        insert: "INSERT INTO pop3_uidl (account_id, uidl, stable_id) VALUES (?1,?2,?3)",
        map: |r| vec![t(r, "account_id"), t(r, "uidl"), t(r, "stable_id")],
    },
    TableSpec {
        name: "sync_state",
        select: "SELECT account_id, mailbox_id, cursor_json, last_sync_at FROM sync_state",
        insert: "INSERT INTO sync_state (account_id, mailbox_id, cursor_json, last_sync_at) VALUES (?1,?2,?3,?4)",
        map: |r| {
            vec![
                t(r, "account_id"),
                t(r, "mailbox_id"),
                t(r, "cursor_json"),
                ot(r, "last_sync_at"),
            ]
        },
    },
    TableSpec {
        name: "message_meta",
        select: "SELECT stable_id, pinned, snoozed_until, follow_up_at FROM message_meta",
        insert: "INSERT INTO message_meta (stable_id, pinned, snoozed_until, follow_up_at) VALUES (?1,?2,?3,?4)",
        map: |r| {
            vec![
                t(r, "stable_id"),
                i(r, "pinned"),
                ot(r, "snoozed_until"),
                ot(r, "follow_up_at"),
            ]
        },
    },
    TableSpec {
        name: "message_embeddings",
        // Per-message SEALED vectors, keyed by `messages.stable_id` (by value; no
        // declared FK). Derived data, but only recomputable by re-running the
        // embedder over the whole store, so a migration silently threw away work
        // that costs real money and time to rebuild. `dim` is validated against the
        // vector length on read, so a truncated copy is rejected rather than used.
        select: "SELECT stable_id, account_id, model, dim, vector_sealed, updated_at FROM message_embeddings",
        insert: "INSERT INTO message_embeddings (stable_id, account_id, model, dim, vector_sealed, updated_at) VALUES (?1,?2,?3,?4,?5,?6)",
        map: |r| {
            vec![
                t(r, "stable_id"),
                t(r, "account_id"),
                t(r, "model"),
                i(r, "dim"),
                b(r, "vector_sealed"),
                t(r, "updated_at"),
            ]
        },
    },
    TableSpec {
        name: "tags",
        select: "SELECT id, \"user\", name, color, icon FROM tags",
        insert: "INSERT INTO tags (id, \"user\", name, color, icon) VALUES (?1,?2,?3,?4,?5)",
        map: |r| {
            vec![
                t(r, "id"),
                t(r, "user"),
                t(r, "name"),
                t(r, "color"),
                ot(r, "icon"),
            ]
        },
    },
    TableSpec {
        name: "saved_searches",
        select: "SELECT id, \"user\", name, query_json, as_folder FROM saved_searches",
        insert: "INSERT INTO saved_searches (id, \"user\", name, query_json, as_folder) VALUES (?1,?2,?3,?4,?5)",
        map: |r| {
            vec![
                t(r, "id"),
                t(r, "user"),
                t(r, "name"),
                t(r, "query_json"),
                i(r, "as_folder"),
            ]
        },
    },
    TableSpec {
        name: "submissions",
        // 0029 (26.20): `attempts` / `last_error` / `next_attempt_at` are copied
        // alongside the 0003 columns. Without them a submission still awaiting
        // dispatch would cross a `migrate-store` having forgotten how many times it
        // has already failed and how long it agreed to wait — the retry budget
        // resets to zero and the backoff is dropped.
        select: "SELECT id, account_id, email_id, identity_id, send_at, undo_status, hold_seconds, created_at, attempts, last_error, next_attempt_at FROM submissions",
        insert: "INSERT INTO submissions (id, account_id, email_id, identity_id, send_at, undo_status, hold_seconds, created_at, attempts, last_error, next_attempt_at) VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11)",
        map: |r| {
            vec![
                t(r, "id"),
                t(r, "account_id"),
                t(r, "email_id"),
                ot(r, "identity_id"),
                ot(r, "send_at"),
                t(r, "undo_status"),
                i(r, "hold_seconds"),
                t(r, "created_at"),
                i(r, "attempts"),
                ot(r, "last_error"),
                ot(r, "next_attempt_at"),
            ]
        },
    },
    TableSpec {
        name: "identities",
        // 0020 (26.17): `signature_name` copied alongside the 0003 columns.
        select: "SELECT id, account_id, name, email, reply_to, signature_html, signature_text, sent_mailbox_id, source, signature_name FROM identities",
        insert: "INSERT INTO identities (id, account_id, name, email, reply_to, signature_html, signature_text, sent_mailbox_id, source, signature_name) VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9,?10)",
        map: |r| {
            vec![
                t(r, "id"),
                t(r, "account_id"),
                t(r, "name"),
                t(r, "email"),
                ot(r, "reply_to"),
                ot(r, "signature_html"),
                ot(r, "signature_text"),
                ot(r, "sent_mailbox_id"),
                t(r, "source"),
                ot(r, "signature_name"),
            ]
        },
    },
    TableSpec {
        name: "changes",
        select: "SELECT account_id, type, state, stable_id, op, at FROM changes",
        insert: "INSERT INTO changes (account_id, type, state, stable_id, op, at) VALUES (?1,?2,?3,?4,?5,?6)",
        map: |r| {
            vec![
                t(r, "account_id"),
                t(r, "type"),
                i(r, "state"),
                t(r, "stable_id"),
                t(r, "op"),
                t(r, "at"),
            ]
        },
    },
    TableSpec {
        name: "calendars",
        select: "SELECT id, account_id, name, color, sort_order, is_visible, role, caldav_url, sync_token, ctag, is_overlay, component FROM calendars",
        insert: "INSERT INTO calendars (id, account_id, name, color, sort_order, is_visible, role, caldav_url, sync_token, ctag, is_overlay, component) VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11,?12)",
        map: |r| {
            vec![
                t(r, "id"),
                t(r, "account_id"),
                t(r, "name"),
                t(r, "color"),
                i(r, "sort_order"),
                i(r, "is_visible"),
                ot(r, "role"),
                ot(r, "caldav_url"),
                ot(r, "sync_token"),
                ot(r, "ctag"),
                i(r, "is_overlay"),
                t(r, "component"),
            ]
        },
    },
    TableSpec {
        name: "calendar_shares",
        select: "SELECT calendar_id, principal, access FROM calendar_shares",
        insert: "INSERT INTO calendar_shares (calendar_id, principal, access) VALUES (?1,?2,?3)",
        map: |r| vec![t(r, "calendar_id"), t(r, "principal"), t(r, "access")],
    },
    TableSpec {
        name: "events",
        select: "SELECT id, calendar_id, uid, etag, ical_raw, start_utc, end_utc, tzid, rrule, status, json FROM events",
        insert: "INSERT INTO events (id, calendar_id, uid, etag, ical_raw, start_utc, end_utc, tzid, rrule, status, json) VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11)",
        map: |r| {
            vec![
                t(r, "id"),
                t(r, "calendar_id"),
                t(r, "uid"),
                ot(r, "etag"),
                t(r, "ical_raw"),
                ot(r, "start_utc"),
                ot(r, "end_utc"),
                ot(r, "tzid"),
                ot(r, "rrule"),
                t(r, "status"),
                ob(r, "json"),
            ]
        },
    },
    TableSpec {
        name: "event_instances",
        select: "SELECT event_id, instance_start_utc, instance_end_utc FROM event_instances",
        insert: "INSERT INTO event_instances (event_id, instance_start_utc, instance_end_utc) VALUES (?1,?2,?3)",
        map: |r| {
            vec![
                t(r, "event_id"),
                t(r, "instance_start_utc"),
                t(r, "instance_end_utc"),
            ]
        },
    },
    TableSpec {
        name: "tasks",
        select: "SELECT id, list_id, uid, etag, due_utc, start_utc, priority, percent_complete, status, parent_id, my_day_date, ical_raw, json FROM tasks",
        insert: "INSERT INTO tasks (id, list_id, uid, etag, due_utc, start_utc, priority, percent_complete, status, parent_id, my_day_date, ical_raw, json) VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11,?12,?13)",
        map: |r| {
            vec![
                t(r, "id"),
                t(r, "list_id"),
                t(r, "uid"),
                ot(r, "etag"),
                ot(r, "due_utc"),
                ot(r, "start_utc"),
                i(r, "priority"),
                i(r, "percent_complete"),
                t(r, "status"),
                ot(r, "parent_id"),
                ot(r, "my_day_date"),
                t(r, "ical_raw"),
                ob(r, "json"),
            ]
        },
    },
    TableSpec {
        name: "notebooks",
        select: "SELECT id, account_id, name FROM notebooks",
        insert: "INSERT INTO notebooks (id, account_id, name) VALUES (?1,?2,?3)",
        map: |r| vec![t(r, "id"), t(r, "account_id"), t(r, "name")],
    },
    TableSpec {
        name: "notes",
        // 0019 (26.17): the four `*_sealed` metadata columns are copied verbatim
        // alongside the frozen plaintext columns (which stay for the backfill window).
        select: "SELECT id, account_id, notebook_id, title, tags_json, color, pinned, body_html_sealed, body_text_sealed, links_json, created_at, updated_at, title_sealed, tags_json_sealed, color_sealed, pinned_sealed FROM notes",
        insert: "INSERT INTO notes (id, account_id, notebook_id, title, tags_json, color, pinned, body_html_sealed, body_text_sealed, links_json, created_at, updated_at, title_sealed, tags_json_sealed, color_sealed, pinned_sealed) VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11,?12,?13,?14,?15,?16)",
        map: |r| {
            vec![
                t(r, "id"),
                t(r, "account_id"),
                ot(r, "notebook_id"),
                t(r, "title"),
                t(r, "tags_json"),
                t(r, "color"),
                i(r, "pinned"),
                ob(r, "body_html_sealed"),
                ob(r, "body_text_sealed"),
                t(r, "links_json"),
                t(r, "created_at"),
                t(r, "updated_at"),
                ob(r, "title_sealed"),
                ob(r, "tags_json_sealed"),
                ob(r, "color_sealed"),
                ob(r, "pinned_sealed"),
            ]
        },
    },
    TableSpec {
        name: "address_books",
        select: "SELECT id, account_id, name, is_default, carddav_url, sync_token, ctag FROM address_books",
        insert: "INSERT INTO address_books (id, account_id, name, is_default, carddav_url, sync_token, ctag) VALUES (?1,?2,?3,?4,?5,?6,?7)",
        map: |r| {
            vec![
                t(r, "id"),
                t(r, "account_id"),
                t(r, "name"),
                i(r, "is_default"),
                ot(r, "carddav_url"),
                ot(r, "sync_token"),
                ot(r, "ctag"),
            ]
        },
    },
    TableSpec {
        name: "contacts",
        select: "SELECT id, address_book_id, uid, etag, vcard_raw, json, full_name, is_favorite, photo_blob_id, pgp_key, smime_cert FROM contacts",
        insert: "INSERT INTO contacts (id, address_book_id, uid, etag, vcard_raw, json, full_name, is_favorite, photo_blob_id, pgp_key, smime_cert) VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11)",
        map: |r| {
            vec![
                t(r, "id"),
                t(r, "address_book_id"),
                t(r, "uid"),
                ot(r, "etag"),
                t(r, "vcard_raw"),
                ob(r, "json"),
                t(r, "full_name"),
                i(r, "is_favorite"),
                ot(r, "photo_blob_id"),
                ot(r, "pgp_key"),
                ot(r, "smime_cert"),
            ]
        },
    },
    TableSpec {
        name: "contact_groups",
        select: "SELECT id, address_book_id, name, member_ids_json FROM contact_groups",
        insert: "INSERT INTO contact_groups (id, address_book_id, name, member_ids_json) VALUES (?1,?2,?3,?4)",
        map: |r| {
            vec![
                t(r, "id"),
                t(r, "address_book_id"),
                t(r, "name"),
                t(r, "member_ids_json"),
            ]
        },
    },
    TableSpec {
        name: "pim_changes",
        select: "SELECT account_id, type, state, object_id, op, at FROM pim_changes",
        insert: "INSERT INTO pim_changes (account_id, type, state, object_id, op, at) VALUES (?1,?2,?3,?4,?5,?6)",
        map: |r| {
            vec![
                t(r, "account_id"),
                t(r, "type"),
                i(r, "state"),
                t(r, "object_id"),
                t(r, "op"),
                t(r, "at"),
            ]
        },
    },
    TableSpec {
        name: "crypto_changes",
        // 0005. `account_id REFERENCES accounts(id) ON DELETE CASCADE` — the only
        // one of these five with a declared foreign key, so it must follow
        // `accounts`. Placed beside its copied siblings `changes` and
        // `pim_changes`, whose change-feed shape it shares.
        select: "SELECT account_id, type, state, object_id, op, at FROM crypto_changes",
        insert: "INSERT INTO crypto_changes (account_id, type, state, object_id, op, at) VALUES (?1,?2,?3,?4,?5,?6)",
        map: |r| {
            vec![
                t(r, "account_id"),
                t(r, "type"),
                i(r, "state"),
                t(r, "object_id"),
                t(r, "op"),
                t(r, "at"),
            ]
        },
    },
    TableSpec {
        name: "crypto_keys",
        select: "SELECT id, account_id, kind, is_own, addresses_json, fingerprint, key_id, algorithm, created_at, expires_at, public_key, cert_pem, trust, autocrypt, source, encrypted_private_backup, verified_at, key_history_json FROM crypto_keys",
        insert: "INSERT INTO crypto_keys (id, account_id, kind, is_own, addresses_json, fingerprint, key_id, algorithm, created_at, expires_at, public_key, cert_pem, trust, autocrypt, source, encrypted_private_backup, verified_at, key_history_json) VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11,?12,?13,?14,?15,?16,?17,?18)",
        map: |r| {
            vec![
                t(r, "id"),
                t(r, "account_id"),
                t(r, "kind"),
                i(r, "is_own"),
                t(r, "addresses_json"),
                t(r, "fingerprint"),
                t(r, "key_id"),
                t(r, "algorithm"),
                t(r, "created_at"),
                ot(r, "expires_at"),
                ot(r, "public_key"),
                ot(r, "cert_pem"),
                t(r, "trust"),
                i(r, "autocrypt"),
                t(r, "source"),
                ob(r, "encrypted_private_backup"),
                ot(r, "verified_at"),
                t(r, "key_history_json"),
            ]
        },
    },
    TableSpec {
        name: "key_associations",
        select: "SELECT account_id, address, crypto_key_id, seen_at FROM key_associations",
        insert: "INSERT INTO key_associations (account_id, address, crypto_key_id, seen_at) VALUES (?1,?2,?3,?4)",
        map: |r| {
            vec![
                t(r, "account_id"),
                t(r, "address"),
                t(r, "crypto_key_id"),
                t(r, "seen_at"),
            ]
        },
    },
    TableSpec {
        name: "security_verdicts",
        select: "SELECT email_id, account_id, raw_hash, verdict_json, computed_at FROM security_verdicts",
        insert: "INSERT INTO security_verdicts (email_id, account_id, raw_hash, verdict_json, computed_at) VALUES (?1,?2,?3,?4,?5)",
        map: |r| {
            vec![
                t(r, "email_id"),
                t(r, "account_id"),
                t(r, "raw_hash"),
                b(r, "verdict_json"),
                t(r, "computed_at"),
            ]
        },
    },
    TableSpec {
        name: "dlp_audit",
        select: "SELECT id, account_id, at, rule_id, rule_name, action, matched_detectors_json, blocked FROM dlp_audit",
        insert: "INSERT INTO dlp_audit (id, account_id, at, rule_id, rule_name, action, matched_detectors_json, blocked) VALUES (?1,?2,?3,?4,?5,?6,?7,?8)",
        map: |r| {
            vec![
                t(r, "id"),
                t(r, "account_id"),
                t(r, "at"),
                t(r, "rule_id"),
                t(r, "rule_name"),
                t(r, "action"),
                t(r, "matched_detectors_json"),
                i(r, "blocked"),
            ]
        },
    },
    TableSpec {
        name: "sender_controls",
        select: "SELECT account_id, address, thread_id, action, mail_rule_id, at FROM sender_controls",
        insert: "INSERT INTO sender_controls (account_id, address, thread_id, action, mail_rule_id, at) VALUES (?1,?2,?3,?4,?5,?6)",
        map: |r| {
            vec![
                t(r, "account_id"),
                ot(r, "address"),
                ot(r, "thread_id"),
                t(r, "action"),
                ot(r, "mail_rule_id"),
                t(r, "at"),
            ]
        },
    },
    TableSpec {
        name: "store_key_material",
        select: "SELECT id, wrapped_seal_key, suite, created_at FROM store_key_material",
        insert: "INSERT INTO store_key_material (id, wrapped_seal_key, suite, created_at) VALUES (?1,?2,?3,?4)",
        map: |r| {
            vec![
                t(r, "id"),
                b(r, "wrapped_seal_key"),
                t(r, "suite"),
                t(r, "created_at"),
            ]
        },
    },
    TableSpec {
        name: "push_subscriptions",
        select: "SELECT id, account_id, transport, endpoint, p256dh, auth, app_id, expires_at, created_at, last_wake_at FROM push_subscriptions",
        insert: "INSERT INTO push_subscriptions (id, account_id, transport, endpoint, p256dh, auth, app_id, expires_at, created_at, last_wake_at) VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9,?10)",
        map: |r| {
            vec![
                t(r, "id"),
                t(r, "account_id"),
                t(r, "transport"),
                t(r, "endpoint"),
                ot(r, "p256dh"),
                ot(r, "auth"),
                ot(r, "app_id"),
                ot(r, "expires_at"),
                t(r, "created_at"),
                ot(r, "last_wake_at"),
            ]
        },
    },
    TableSpec {
        name: "push_config",
        select: "SELECT id, vapid_public, vapid_private_sealed, created_at FROM push_config",
        insert: "INSERT INTO push_config (id, vapid_public, vapid_private_sealed, created_at) VALUES (?1,?2,?3,?4)",
        map: |r| {
            vec![
                i(r, "id"),
                t(r, "vapid_public"),
                b(r, "vapid_private_sealed"),
                t(r, "created_at"),
            ]
        },
    },
    TableSpec {
        name: "native_sessions",
        select: "SELECT token_hash, account_id, client_type, created_at, last_seen, rotated_from FROM native_sessions",
        insert: "INSERT INTO native_sessions (token_hash, account_id, client_type, created_at, last_seen, rotated_from) VALUES (?1,?2,?3,?4,?5,?6)",
        map: |r| {
            vec![
                t(r, "token_hash"),
                t(r, "account_id"),
                t(r, "client_type"),
                t(r, "created_at"),
                t(r, "last_seen"),
                ot(r, "rotated_from"),
            ]
        },
    },
    // `audit_log` and `twofa_policy` reference nothing and are referenced by
    // nothing, so their position is unconstrained; they go last rather than being
    // interleaved with the mail graph they have no part in.
    TableSpec {
        name: "audit_log",
        // 0007, SPEC §21: append-only, with no update or delete path anywhere in
        // the store. A migration that dropped it was the only way to erase it.
        select: "SELECT id, ts, actor, actor_kind, action, target, detail_json, ip FROM audit_log",
        insert: "INSERT INTO audit_log (id, ts, actor, actor_kind, action, target, detail_json, ip) VALUES (?1,?2,?3,?4,?5,?6,?7,?8)",
        map: |r| {
            vec![
                t(r, "id"),
                t(r, "ts"),
                t(r, "actor"),
                t(r, "actor_kind"),
                t(r, "action"),
                ot(r, "target"),
                t(r, "detail_json"),
                ot(r, "ip"),
            ]
        },
    },
    TableSpec {
        name: "twofa_policy",
        // 0015. Not copying this failed OPEN: a deployment that required a second
        // factor silently stopped requiring one the moment it was migrated.
        select: "SELECT scope_kind, scope_value, require_2fa, updated_by, updated_at FROM twofa_policy",
        insert: "INSERT INTO twofa_policy (scope_kind, scope_value, require_2fa, updated_by, updated_at) VALUES (?1,?2,?3,?4,?5)",
        map: |r| {
            vec![
                t(r, "scope_kind"),
                t(r, "scope_value"),
                i(r, "require_2fa"),
                t(r, "updated_by"),
                t(r, "updated_at"),
            ]
        },
    },
    // ── Deployment surfaces and the remaining append-only audit logs (26.20).
    // No declared `REFERENCES` in either dialect; nothing references them either,
    // so they go last rather than into the mail graph.
    TableSpec {
        name: "plugin_kv",
        // Per-plugin, per-account SEALED plugin state with quota accounting —
        // application data a plugin cannot regenerate. Rows for a plugin the operator
        // never re-approves are inert (a plugin reads only its own namespace), so
        // copying costs nothing and not copying destroyed the plugin's data.
        select: "SELECT plugin_id, account_id, key, sealed_value, size, updated_at FROM plugin_kv",
        insert: "INSERT INTO plugin_kv (plugin_id, account_id, key, sealed_value, size, updated_at) VALUES (?1,?2,?3,?4,?5,?6)",
        map: |r| {
            vec![
                t(r, "plugin_id"),
                t(r, "account_id"),
                t(r, "key"),
                b(r, "sealed_value"),
                i(r, "size"),
                t(r, "updated_at"),
            ]
        },
    },
    TableSpec {
        name: "assist_config",
        // Keyed by scope: 'deployment' AND 'user:<account_id>'. The per-user rows are
        // user configuration, not deployment config, so the table could not be
        // dismissed as re-configurable. `enabled` travels with it, so a deployment
        // that had Assist off stays off.
        select: "SELECT scope, adapters, capability_grants, data_ceilings, enabled FROM assist_config",
        insert: "INSERT INTO assist_config (scope, adapters, capability_grants, data_ceilings, enabled) VALUES (?1,?2,?3,?4,?5)",
        map: |r| {
            vec![
                t(r, "scope"),
                t(r, "adapters"),
                t(r, "capability_grants"),
                t(r, "data_ceilings"),
                i(r, "enabled"),
            ]
        },
    },
    TableSpec {
        name: "cache_scope",
        // The per-CacheClass layer/TTL matrix: admin tuning of a pure accelerator.
        // Copying preserves the operator's tuning; the cache is never authoritative,
        // so nothing here can be stale in a way that matters.
        select: "SELECT class, layers, ttl_secs FROM cache_scope",
        insert: "INSERT INTO cache_scope (class, layers, ttl_secs) VALUES (?1,?2,?3)",
        map: |r| vec![t(r, "class"), t(r, "layers"), i(r, "ttl_secs")],
    },
    TableSpec {
        name: "sso_login_audit",
        // Append-only (hashed subjects, never raw). Same reasoning as `audit_log`:
        // a table with no delete path must not lose its history to a backend swap.
        select: "SELECT id, ts, provider_id, kind, subject_hash, outcome FROM sso_login_audit",
        insert: "INSERT INTO sso_login_audit (id, ts, provider_id, kind, subject_hash, outcome) VALUES (?1,?2,?3,?4,?5,?6)",
        map: |r| {
            vec![
                t(r, "id"),
                t(r, "ts"),
                t(r, "provider_id"),
                t(r, "kind"),
                t(r, "subject_hash"),
                t(r, "outcome"),
            ]
        },
    },
    TableSpec {
        name: "password_change_audit",
        select: "SELECT id, ts, account_id, backend, outcome FROM password_change_audit",
        insert: "INSERT INTO password_change_audit (id, ts, account_id, backend, outcome) VALUES (?1,?2,?3,?4,?5)",
        map: |r| {
            vec![
                t(r, "id"),
                t(r, "ts"),
                t(r, "account_id"),
                t(r, "backend"),
                t(r, "outcome"),
            ]
        },
    },
    TableSpec {
        name: "assist_audit",
        select: "SELECT id, ts, actor, capability, scope_summary, endpoint_host FROM assist_audit",
        insert: "INSERT INTO assist_audit (id, ts, actor, capability, scope_summary, endpoint_host) VALUES (?1,?2,?3,?4,?5,?6)",
        map: |r| {
            vec![
                t(r, "id"),
                t(r, "ts"),
                t(r, "actor"),
                t(r, "capability"),
                t(r, "scope_summary"),
                t(r, "endpoint_host"),
            ]
        },
    },
];
