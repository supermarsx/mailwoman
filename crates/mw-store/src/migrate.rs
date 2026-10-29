//! OIDC0056 deployment configuration, keys and grants follow the existing
//! oauth_clients exclusion: an operator reapproves the issuer and clients on the
//! destination, and users consent again. Durable primary principals are copied.
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
//! below is the complete list of what is copied: as of migration 0035 that is 121
//! tables out of the 147 the migrations create. The other 26 are left behind
//! deliberately, each with its reason recorded at its entry in
//! `NOT_MIGRATED_DELIBERATELY` in `tests/backend_parity.rs`: the admin panel's
//! separate identity domain, OAuth clients and tokens, API keys, webhooks (and,
//! from 0034, inbound webhooks with their nonce cache), the
//! managed-domain/directory/egress/SSO/tunnel/integration configuration that
//! names the OLD deployment's hosts or network position (and the 0033 tables
//! that hang off `domains`), and the plugin registries and their capability
//! grants, which are deny-by-default and re-approved by an operator.
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
//! Migration 0033 (t32 admin controls) added 66 tables and was classified in the
//! same commit: 59 copied, 7 deliberate, none left as an open question. The
//! reasons sit at each `TableSpec` below and each deliberate entry.
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
fn oi(r: &Row, c: &str) -> Arg {
    use sqlx::Row as _;
    let v: Option<i64> = match r {
        Row::Sqlite(r) => r.get(c),
        Row::Postgres(r) => r.get(c),
    };
    v.into_arg()
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
    // Verified durable realm subject and sealed editor bytes survive store copies.
    TableSpec {
        name: "account_draft",
        select: "SELECT owner,id,account_id,revision,email_id,sealed_state,sealed_summary,updated_at,operation_id,request_hash,operation_state,sealed_pending_state,sealed_pending_summary,cleanup_email_id FROM account_draft",
        insert: "INSERT INTO account_draft(owner,id,account_id,revision,email_id,sealed_state,sealed_summary,updated_at,operation_id,request_hash,operation_state,sealed_pending_state,sealed_pending_summary,cleanup_email_id) VALUES(?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11,?12,?13,?14)",
        map: |r| {
            vec![
                t(r, "owner"),
                t(r, "id"),
                t(r, "account_id"),
                ot(r, "revision"),
                ot(r, "email_id"),
                ob(r, "sealed_state"),
                ob(r, "sealed_summary"),
                i(r, "updated_at"),
                t(r, "operation_id"),
                t(r, "request_hash"),
                t(r, "operation_state"),
                ob(r, "sealed_pending_state"),
                ob(r, "sealed_pending_summary"),
                ot(r, "cleanup_email_id"),
            ]
        },
    },
    TableSpec {
        name: "identity_principal",
        select: "SELECT id,state,created_at,retired_at FROM identity_principal",
        insert: "INSERT INTO identity_principal(id,state,created_at,retired_at) VALUES(?1,?2,?3,?4)",
        map: |r| {
            vec![
                t(r, "id"),
                t(r, "state"),
                i(r, "created_at"),
                oi(r, "retired_at"),
            ]
        },
    },
    TableSpec {
        name: "identity_binding",
        select: "SELECT id,principal_id,kind,authority,subject,created_at,retired_at FROM identity_binding",
        insert: "INSERT INTO identity_binding(id,principal_id,kind,authority,subject,created_at,retired_at) VALUES(?1,?2,?3,?4,?5,?6,?7)",
        map: |r| {
            vec![
                t(r, "id"),
                t(r, "principal_id"),
                t(r, "kind"),
                t(r, "authority"),
                t(r, "subject"),
                i(r, "created_at"),
                oi(r, "retired_at"),
            ]
        },
    },
    TableSpec {
        name: "settings",
        select: "SELECT key, value FROM settings",
        insert: "INSERT INTO settings (key, value) VALUES (?1, ?2)",
        map: |r| vec![t(r, "key"), t(r, "value")],
    },
    TableSpec {
        name: "sessions",
        // 0033 (26.20): the device columns travel with the session, so "sign out
        // this device" still names the right device after a move.
        select: "SELECT id, account_id, username, jmap_url, api_url, sealed_creds, created_at, last_seen, user_agent, ip, client_type, device_label, client_version, authentication_method, activity_at, linked_internal FROM sessions",
        insert: "INSERT INTO sessions (id, account_id, username, jmap_url, api_url, sealed_creds, created_at, last_seen, user_agent, ip, client_type, device_label, client_version, authentication_method, activity_at, linked_internal) VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11,?12,?13,?14,?15,?16)",
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
                ot(r, "user_agent"),
                ot(r, "ip"),
                ot(r, "client_type"),
                ot(r, "device_label"),
                ot(r, "client_version"),
                t(r, "authentication_method"),
                ot(r, "activity_at"),
                i(r, "linked_internal"),
            ]
        },
    },
    TableSpec {
        name: "session_principal",
        select: "SELECT session_id,principal_id,binding_id,primary_account_id,auth_time,authentication_method FROM session_principal",
        insert: "INSERT INTO session_principal(session_id,principal_id,binding_id,primary_account_id,auth_time,authentication_method) VALUES(?1,?2,?3,?4,?5,?6)",
        map: |r| {
            vec![
                t(r, "session_id"),
                t(r, "principal_id"),
                t(r, "binding_id"),
                t(r, "primary_account_id"),
                i(r, "auth_time"),
                t(r, "authentication_method"),
            ]
        },
    },
    TableSpec {
        name: "linked_account",
        select: "SELECT id,owner_principal_id,child_principal_id,child_subject,account_id,display_name,sealed_session,revision,created_at FROM linked_account",
        insert: "INSERT INTO linked_account(id,owner_principal_id,child_principal_id,child_subject,account_id,display_name,sealed_session,revision,created_at) VALUES(?1,?2,?3,?4,?5,?6,?7,?8,?9)",
        map: |r| {
            vec![
                t(r, "id"),
                t(r, "owner_principal_id"),
                t(r, "child_principal_id"),
                t(r, "child_subject"),
                t(r, "account_id"),
                t(r, "display_name"),
                b(r, "sealed_session"),
                t(r, "revision"),
                i(r, "created_at"),
            ]
        },
    },
    TableSpec {
        name: "linked_context",
        select: "SELECT session_id,owner_session_id,link_id,link_revision FROM linked_context",
        insert: "INSERT INTO linked_context(session_id,owner_session_id,link_id,link_revision) VALUES(?1,?2,?3,?4)",
        map: |r| {
            vec![
                t(r, "session_id"),
                t(r, "owner_session_id"),
                t(r, "link_id"),
                t(r, "link_revision"),
            ]
        },
    },
    // t40's owner arrangement (first mailbox, order, nicknames): plain rows
    // keyed by the owner principal, which travels above.
    TableSpec {
        name: "linked_primary",
        select: "SELECT owner_principal_id,link_id,updated_at FROM linked_primary",
        insert: "INSERT INTO linked_primary(owner_principal_id,link_id,updated_at) VALUES(?1,?2,?3)",
        map: |r| {
            vec![
                t(r, "owner_principal_id"),
                t(r, "link_id"),
                i(r, "updated_at"),
            ]
        },
    },
    TableSpec {
        name: "linked_layout",
        select: "SELECT owner_principal_id,order_json,nicknames_json,revision,updated_at FROM linked_layout",
        insert: "INSERT INTO linked_layout(owner_principal_id,order_json,nicknames_json,revision,updated_at) VALUES(?1,?2,?3,?4,?5)",
        map: |r| {
            vec![
                t(r, "owner_principal_id"),
                t(r, "order_json"),
                t(r, "nicknames_json"),
                t(r, "revision"),
                i(r, "updated_at"),
            ]
        },
    },
    // t48: sealed under the same server key and bound to owner+link, so the
    // ciphertext moves as-is and opens on the destination.
    TableSpec {
        name: "linked_credential",
        select: "SELECT link_id,owner_principal_id,owner_account_id,child_authority,login_name,state,reason,sealed_secret,updated_at FROM linked_credential",
        insert: "INSERT INTO linked_credential(link_id,owner_principal_id,owner_account_id,child_authority,login_name,state,reason,sealed_secret,updated_at) VALUES(?1,?2,?3,?4,?5,?6,?7,?8,?9)",
        map: |r| {
            vec![
                t(r, "link_id"),
                t(r, "owner_principal_id"),
                t(r, "owner_account_id"),
                t(r, "child_authority"),
                t(r, "login_name"),
                t(r, "state"),
                ot(r, "reason"),
                ob(r, "sealed_secret"),
                i(r, "updated_at"),
            ]
        },
    },
    // MFA continuations are process-local. Drop challenges on store migration;
    // never copy a stale pending token and pretend it can complete on a new node.
    TableSpec {
        name: "accounts",
        select: "SELECT id, kind, host, port, tls, username, sealed_creds, sync_policy_json, backend_kind, backend_authority FROM accounts",
        insert: "INSERT INTO accounts (id, kind, host, port, tls, username, sealed_creds, sync_policy_json, backend_kind, backend_authority) VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9,?10)",
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
                t(r, "backend_kind"),
                t(r, "backend_authority"),
            ]
        },
    },
    TableSpec {
        name: "account_provider",
        select: "SELECT account_id,provider,endpoint_url,backend_username,login_address,domain_binding,ca_pem,sealed_credential,revision,created_at,updated_at FROM account_provider",
        insert: "INSERT INTO account_provider(account_id,provider,endpoint_url,backend_username,login_address,domain_binding,ca_pem,sealed_credential,revision,created_at,updated_at) VALUES(?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11)",
        map: |r| {
            vec![
                t(r, "account_id"),
                t(r, "provider"),
                t(r, "endpoint_url"),
                t(r, "backend_username"),
                t(r, "login_address"),
                t(r, "domain_binding"),
                ot(r, "ca_pem"),
                b(r, "sealed_credential"),
                t(r, "revision"),
                i(r, "created_at"),
                i(r, "updated_at"),
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
        // 0033 (26.20): `discoverable` marks a passkey usable without a username;
        // dropped, passwordless sign-in would stop finding it.
        select: "SELECT credential_id, account_id, cose_public_key, sign_count, transports, label, created_at, discoverable, backup_eligible, backup_state, uv_initialized, state_revision FROM webauthn_credentials",
        insert: "INSERT INTO webauthn_credentials (credential_id, account_id, cose_public_key, sign_count, transports, label, created_at, discoverable, backup_eligible, backup_state, uv_initialized, state_revision) VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11,?12)",
        map: |r| {
            vec![
                t(r, "credential_id"),
                t(r, "account_id"),
                b(r, "cose_public_key"),
                i(r, "sign_count"),
                t(r, "transports"),
                t(r, "label"),
                t(r, "created_at"),
                i(r, "discoverable"),
                oi(r, "backup_eligible"),
                i(r, "backup_state"),
                i(r, "uv_initialized"),
                i(r, "state_revision"),
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
        name: "personal_identity",
        select: "SELECT id, owner_subject, name, email, reply_to, signature_html, signature_text, sent_mailbox_id, source, signature_name FROM personal_identity",
        insert: "INSERT INTO personal_identity (id, owner_subject, name, email, reply_to, signature_html, signature_text, sent_mailbox_id, source, signature_name) VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9,?10)",
        map: |r| {
            vec![
                t(r, "id"),
                t(r, "owner_subject"),
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
        name: "personal_notification_rules",
        select: "SELECT owner_subject, rule_json, quiet_hours_json, enabled, updated_at FROM personal_notification_rules",
        insert: "INSERT INTO personal_notification_rules (owner_subject, rule_json, quiet_hours_json, enabled, updated_at) VALUES (?1,?2,?3,?4,?5)",
        map: |r| {
            vec![
                t(r, "owner_subject"),
                t(r, "rule_json"),
                t(r, "quiet_hours_json"),
                i(r, "enabled"),
                t(r, "updated_at"),
            ]
        },
    },
    TableSpec {
        name: "personal_push_subscription",
        select: "SELECT id, owner_subject, transport, endpoint, p256dh, auth, app_id, expires_at, created_at, last_wake_at FROM personal_push_subscription",
        insert: "INSERT INTO personal_push_subscription (id, owner_subject, transport, endpoint, p256dh, auth, app_id, expires_at, created_at, last_wake_at) VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9,?10)",
        map: |r| {
            vec![
                t(r, "id"),
                t(r, "owner_subject"),
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
        name: "personal_setting_value",
        select: "SELECT owner_subject,key,value,updated_at FROM personal_setting_value",
        insert: "INSERT INTO personal_setting_value(owner_subject,key,value,updated_at) VALUES(?1,?2,?3,?4)",
        map: |r| {
            vec![
                t(r, "owner_subject"),
                t(r, "key"),
                t(r, "value"),
                i(r, "updated_at"),
            ]
        },
    },
    TableSpec {
        name: "personal_signature",
        // User-authored content. The `identities.signature_*` columns beside it were
        // always copied, so leaving these behind lost half of one feature.
        // 0033 (26.20): the HTML/text pair, the edited flag and the images.
        select: "SELECT owner_subject, name, body, is_default, rule_json, updated_at, html, text, text_edited, images_json FROM personal_signature",
        insert: "INSERT INTO personal_signature (owner_subject, name, body, is_default, rule_json, updated_at, html, text, text_edited, images_json) VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9,?10)",
        map: |r| {
            vec![
                t(r, "owner_subject"),
                t(r, "name"),
                t(r, "body"),
                i(r, "is_default"),
                t(r, "rule_json"),
                t(r, "updated_at"),
                ot(r, "html"),
                ot(r, "text"),
                i(r, "text_edited"),
                t(r, "images_json"),
            ]
        },
    },
    TableSpec {
        name: "signatures",
        // User-authored content. The `identities.signature_*` columns beside it were
        // always copied, so leaving these behind lost half of one feature.
        // 0033 (26.20): the HTML/text pair, the edited flag and the images.
        select: "SELECT account_id, name, body, is_default, rule_json, updated_at, html, text, text_edited, images_json FROM signatures",
        insert: "INSERT INTO signatures (account_id, name, body, is_default, rule_json, updated_at, html, text, text_edited, images_json) VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9,?10)",
        map: |r| {
            vec![
                t(r, "account_id"),
                t(r, "name"),
                t(r, "body"),
                i(r, "is_default"),
                t(r, "rule_json"),
                t(r, "updated_at"),
                ot(r, "html"),
                ot(r, "text"),
                i(r, "text_edited"),
                t(r, "images_json"),
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
        name: "open_tracking",
        // 0032 (26.20): a sender's open-tracking tokens and their totals. Left
        // behind, every sent message's "opened" record would vanish in a store move,
        // and images already delivered would stop counting. The subject and
        // recipients travel sealed, byte for byte, under the same server key.
        select: "SELECT token_hash, account_id, email_id, subject_sealed, recipients_sealed, created_at, open_count, first_open_at, last_open_at FROM open_tracking",
        insert: "INSERT INTO open_tracking (token_hash, account_id, email_id, subject_sealed, recipients_sealed, created_at, open_count, first_open_at, last_open_at) VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9)",
        map: |r| {
            vec![
                t(r, "token_hash"),
                t(r, "account_id"),
                ot(r, "email_id"),
                b(r, "subject_sealed"),
                b(r, "recipients_sealed"),
                t(r, "created_at"),
                i(r, "open_count"),
                ot(r, "first_open_at"),
                ot(r, "last_open_at"),
            ]
        },
    },
    TableSpec {
        name: "open_events",
        select: "SELECT token_hash, at FROM open_events",
        insert: "INSERT INTO open_events (token_hash, at) VALUES (?1,?2)",
        map: |r| vec![t(r, "token_hash"), t(r, "at")],
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
        name: "personal_saved_search",
        // 0033 (26.20): the view columns (query text, builder, accounts,
        // settings, order, icon, accent, start page, source, org view link).
        select: "SELECT id, owner_subject, name, query_json, as_folder, query_text, builder_json, accounts_json, settings_json, position, icon, accent_id, start_page, source, org_view_id FROM personal_saved_search",
        insert: "INSERT INTO personal_saved_search (id, owner_subject, name, query_json, as_folder, query_text, builder_json, accounts_json, settings_json, position, icon, accent_id, start_page, source, org_view_id) VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11,?12,?13,?14,?15)",
        map: |r| {
            vec![
                t(r, "id"),
                t(r, "owner_subject"),
                t(r, "name"),
                t(r, "query_json"),
                i(r, "as_folder"),
                ot(r, "query_text"),
                ot(r, "builder_json"),
                ot(r, "accounts_json"),
                t(r, "settings_json"),
                i(r, "position"),
                ot(r, "icon"),
                ot(r, "accent_id"),
                i(r, "start_page"),
                t(r, "source"),
                ot(r, "org_view_id"),
            ]
        },
    },
    TableSpec {
        name: "saved_searches",
        // 0033 (26.20): the view columns (query text, builder, accounts,
        // settings, order, icon, accent, start page, source, org view link).
        select: "SELECT id, \"user\", name, query_json, as_folder, query_text, builder_json, accounts_json, settings_json, position, icon, accent_id, start_page, source, org_view_id FROM saved_searches",
        insert: "INSERT INTO saved_searches (id, \"user\", name, query_json, as_folder, query_text, builder_json, accounts_json, settings_json, position, icon, accent_id, start_page, source, org_view_id) VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11,?12,?13,?14,?15)",
        map: |r| {
            vec![
                t(r, "id"),
                t(r, "user"),
                t(r, "name"),
                t(r, "query_json"),
                i(r, "as_folder"),
                ot(r, "query_text"),
                ot(r, "builder_json"),
                ot(r, "accounts_json"),
                t(r, "settings_json"),
                i(r, "position"),
                ot(r, "icon"),
                ot(r, "accent_id"),
                i(r, "start_page"),
                t(r, "source"),
                ot(r, "org_view_id"),
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
        // 0031 (26.20): `hold` / `origin` / `on_success` are copied too. Dropping
        // `hold` would turn a submission that waits for its owner's release into
        // one that is due at once: the copy would send mail nobody approved.
        select: "SELECT id, account_id, email_id, identity_id, send_at, undo_status, hold_seconds, created_at, attempts, last_error, next_attempt_at, hold, origin, on_success FROM submissions",
        insert: "INSERT INTO submissions (id, account_id, email_id, identity_id, send_at, undo_status, hold_seconds, created_at, attempts, last_error, next_attempt_at, hold, origin, on_success) VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11,?12,?13,?14)",
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
                ot(r, "hold"),
                ot(r, "origin"),
                ot(r, "on_success"),
            ]
        },
    },
    TableSpec {
        name: "identities",
        // 0020 (26.17): `signature_name` copied alongside the 0003 columns.
        // 0033 (26.20): auto-Bcc, colour, key links, receipt request, the
        // verification time (an unverified copy would block sending as it) and
        // the Sender header.
        select: "SELECT id, account_id, name, email, reply_to, signature_html, signature_text, sent_mailbox_id, source, signature_name, auto_bcc, color, pgp_key_id, smime_cert_id, request_receipt, verified_at, sender_header FROM identities",
        insert: "INSERT INTO identities (id, account_id, name, email, reply_to, signature_html, signature_text, sent_mailbox_id, source, signature_name, auto_bcc, color, pgp_key_id, smime_cert_id, request_receipt, verified_at, sender_header) VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11,?12,?13,?14,?15,?16,?17)",
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
                ot(r, "auto_bcc"),
                ot(r, "color"),
                ot(r, "pgp_key_id"),
                ot(r, "smime_cert_id"),
                i(r, "request_receipt"),
                oi(r, "verified_at"),
                ot(r, "sender_header"),
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
        // 0035: `href` is the DAV server's name for the resource; without it the
        // destination would address every synced event by a guessed href.
        select: "SELECT id, calendar_id, uid, etag, ical_raw, start_utc, end_utc, tzid, rrule, status, json, href FROM events",
        insert: "INSERT INTO events (id, calendar_id, uid, etag, ical_raw, start_utc, end_utc, tzid, rrule, status, json, href) VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11,?12)",
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
                ot(r, "href"),
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
        select: "SELECT id, list_id, uid, etag, due_utc, start_utc, priority, percent_complete, status, parent_id, my_day_date, ical_raw, json, href FROM tasks",
        insert: "INSERT INTO tasks (id, list_id, uid, etag, due_utc, start_utc, priority, percent_complete, status, parent_id, my_day_date, ical_raw, json, href) VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11,?12,?13,?14)",
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
                ot(r, "href"),
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
        select: "SELECT id, address_book_id, uid, etag, vcard_raw, json, full_name, is_favorite, photo_blob_id, pgp_key, smime_cert, href FROM contacts",
        insert: "INSERT INTO contacts (id, address_book_id, uid, etag, vcard_raw, json, full_name, is_favorite, photo_blob_id, pgp_key, smime_cert, href) VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11,?12)",
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
                ot(r, "href"),
            ]
        },
    },
    TableSpec {
        name: "dav_account",
        // 0035: an account's CalDAV/CardDAV sign-in (a sealed password when it is
        // not the mail one) and its sync backoff. Copied like the other sealed
        // account credentials: without it every DAV-syncing account stops syncing
        // until its owner enters the password again. The server URLs are not here;
        // they come from the domain on the destination.
        select: "SELECT account_id, enabled, use_mail_credentials, username, sealed_password, last_sync_at, last_error, failures, next_attempt_at, created_at, updated_at FROM dav_account",
        insert: "INSERT INTO dav_account (account_id, enabled, use_mail_credentials, username, sealed_password, last_sync_at, last_error, failures, next_attempt_at, created_at, updated_at) VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11)",
        map: |r| {
            vec![
                t(r, "account_id"),
                i(r, "enabled"),
                i(r, "use_mail_credentials"),
                t(r, "username"),
                ob(r, "sealed_password"),
                oi(r, "last_sync_at"),
                t(r, "last_error"),
                i(r, "failures"),
                oi(r, "next_attempt_at"),
                i(r, "created_at"),
                i(r, "updated_at"),
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
    TableSpec {
        name: "request_metrics_bucket",
        select: "SELECT instance_id,bucket_seconds,bucket_start,observed_from,observed_to,counts_json FROM request_metrics_bucket",
        insert: "INSERT INTO request_metrics_bucket(instance_id,bucket_seconds,bucket_start,observed_from,observed_to,counts_json) VALUES(?1,?2,?3,?4,?5,?6)",
        map: |r| {
            vec![
                t(r, "instance_id"),
                i(r, "bucket_seconds"),
                i(r, "bucket_start"),
                i(r, "observed_from"),
                i(r, "observed_to"),
                t(r, "counts_json"),
            ]
        },
    },
    TableSpec {
        name: "package_provider",
        select: "SELECT id,enabled,refresh_secs,sealed_key,revision FROM package_provider",
        insert: "INSERT INTO package_provider(id,enabled,refresh_secs,sealed_key,revision) VALUES(?1,?2,?3,?4,?5)",
        map: |r| {
            vec![
                t(r, "id"),
                i(r, "enabled"),
                i(r, "refresh_secs"),
                b(r, "sealed_key"),
                t(r, "revision"),
            ]
        },
    },
    TableSpec {
        name: "package_account",
        select: "SELECT account_id,state,revision,native_account_id FROM package_account",
        insert: "INSERT INTO package_account(account_id,state,revision,native_account_id) VALUES(?1,?2,?3,?4)",
        map: |r| {
            vec![
                t(r, "account_id"),
                t(r, "state"),
                t(r, "revision"),
                ot(r, "native_account_id"),
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
        // 0033 (26.20): the RBAC, category, request-id, before/after and hash
        // chain columns, with `seq`: the chain only verifies if every link
        // arrives unchanged.
        select: "SELECT id, ts, actor, actor_kind, action, target, detail_json, ip, admin_role, capability, category, request_id, before_json, after_json, prev_hash, hash, seq FROM audit_log",
        insert: "INSERT INTO audit_log (id, ts, actor, actor_kind, action, target, detail_json, ip, admin_role, capability, category, request_id, before_json, after_json, prev_hash, hash, seq) VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11,?12,?13,?14,?15,?16,?17)",
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
                ot(r, "admin_role"),
                ot(r, "capability"),
                ot(r, "category"),
                ot(r, "request_id"),
                ot(r, "before_json"),
                ot(r, "after_json"),
                ot(r, "prev_hash"),
                ot(r, "hash"),
                oi(r, "seq"),
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
    // t39-e11 (0073): AI provider profiles. The key is sealed under the same
    // server key and bound to id + dialect + endpoint, so the ciphertext moves
    // as-is and opens on the destination.
    TableSpec {
        name: "assist_profile",
        select: "SELECT id, name, preset, dialect, endpoint, model, params, api_key_sealed, created_by, created_at, updated_by, updated_at FROM assist_profile",
        insert: "INSERT INTO assist_profile (id, name, preset, dialect, endpoint, model, params, api_key_sealed, created_by, created_at, updated_by, updated_at) VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11,?12)",
        map: |r| {
            vec![
                t(r, "id"),
                t(r, "name"),
                t(r, "preset"),
                t(r, "dialect"),
                t(r, "endpoint"),
                t(r, "model"),
                t(r, "params"),
                ob(r, "api_key_sealed"),
                t(r, "created_by"),
                i(r, "created_at"),
                t(r, "updated_by"),
                i(r, "updated_at"),
            ]
        },
    },
    // t39-e11 (0073): the admin's egress grants for local profile endpoints.
    TableSpec {
        name: "assist_egress_grant",
        select: "SELECT id, destination, note, created_by, created_at FROM assist_egress_grant",
        insert: "INSERT INTO assist_egress_grant (id, destination, note, created_by, created_at) VALUES (?1,?2,?3,?4,?5)",
        map: |r| {
            vec![
                t(r, "id"),
                t(r, "destination"),
                t(r, "note"),
                t(r, "created_by"),
                i(r, "created_at"),
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
    // ── 0033 (26.20 t32-e10a): admin controls, users and sign-in policy,
    // configuration. 59 of its 66 new tables are copied; the other seven are in
    // `NOT_MIGRATED_DELIBERATELY` with their reasons (children of the uncopied
    // `domains`; tunnels, egress allowances, upstream OAuth client registrations
    // and integration endpoints, which describe the OLD deployment's network
    // position like `egress_proxy` and `directory_config`). FKs exist only
    // inside the copied set, parents first.
    TableSpec {
        name: "setting_policy",
        // Global values and locks of the settings registry. Left behind, a locked
        // setting silently unlocks on the destination, so this fails OPEN like
        // `twofa_policy` did.
        select: "SELECT key, global_value, allow_user_override, global_locked, updated_by, updated_at FROM setting_policy",
        insert: "INSERT INTO setting_policy (key, global_value, allow_user_override, global_locked, updated_by, updated_at) VALUES (?1,?2,?3,?4,?5,?6)",
        map: |r| {
            vec![
                t(r, "key"),
                ot(r, "global_value"),
                i(r, "allow_user_override"),
                i(r, "global_locked"),
                t(r, "updated_by"),
                i(r, "updated_at"),
            ]
        },
    },
    TableSpec {
        name: "setting_value",
        select: "SELECT scope_kind, scope_id, key, value, locked, updated_by, updated_at FROM setting_value",
        insert: "INSERT INTO setting_value (scope_kind, scope_id, key, value, locked, updated_by, updated_at) VALUES (?1,?2,?3,?4,?5,?6,?7)",
        map: |r| {
            vec![
                t(r, "scope_kind"),
                t(r, "scope_id"),
                t(r, "key"),
                t(r, "value"),
                i(r, "locked"),
                t(r, "updated_by"),
                i(r, "updated_at"),
            ]
        },
    },
    TableSpec {
        name: "auth_failure",
        // Live failure counters, copied like `sessions`: dropping them resets every
        // lockout window in the middle of an attack.
        select: "SELECT surface, key_kind, key, window_start, count FROM auth_failure",
        insert: "INSERT INTO auth_failure (surface, key_kind, key, window_start, count) VALUES (?1,?2,?3,?4,?5)",
        map: |r| {
            vec![
                t(r, "surface"),
                t(r, "key_kind"),
                t(r, "key"),
                i(r, "window_start"),
                i(r, "count"),
            ]
        },
    },
    TableSpec {
        name: "ip_block",
        // Omission would lift every ban and every allowlist restriction — both fail
        // OPEN. `ip_allow` follows for the same reason.
        select: "SELECT id, ip_or_cidr, surface, reason, offences, blocked_until, created_by, created_at, note FROM ip_block",
        insert: "INSERT INTO ip_block (id, ip_or_cidr, surface, reason, offences, blocked_until, created_by, created_at, note) VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9)",
        map: |r| {
            vec![
                t(r, "id"),
                t(r, "ip_or_cidr"),
                t(r, "surface"),
                t(r, "reason"),
                i(r, "offences"),
                oi(r, "blocked_until"),
                t(r, "created_by"),
                i(r, "created_at"),
                t(r, "note"),
            ]
        },
    },
    TableSpec {
        name: "ip_allow",
        select: "SELECT id, surface, scope_kind, scope_id, cidr, note, created_by, created_at FROM ip_allow",
        insert: "INSERT INTO ip_allow (id, surface, scope_kind, scope_id, cidr, note, created_by, created_at) VALUES (?1,?2,?3,?4,?5,?6,?7,?8)",
        map: |r| {
            vec![
                t(r, "id"),
                t(r, "surface"),
                t(r, "scope_kind"),
                t(r, "scope_id"),
                t(r, "cidr"),
                t(r, "note"),
                t(r, "created_by"),
                i(r, "created_at"),
            ]
        },
    },
    TableSpec {
        name: "admin_account",
        // Admin accounts, their TOTP enrolments and their roles travel together, for
        // the reason the mail 2FA tables do: a role or an enrolment without its
        // account (or the reverse) is a lockout or a silent change of rights. FK
        // parents first: the SQLite destination enforces them row by row.
        select: "SELECT id, username, display_name, password_hash, disabled, source, created_at, updated_at, last_login_at FROM admin_account",
        insert: "INSERT INTO admin_account (id, username, display_name, password_hash, disabled, source, created_at, updated_at, last_login_at) VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9)",
        map: |r| {
            vec![
                t(r, "id"),
                t(r, "username"),
                t(r, "display_name"),
                ot(r, "password_hash"),
                i(r, "disabled"),
                t(r, "source"),
                i(r, "created_at"),
                i(r, "updated_at"),
                oi(r, "last_login_at"),
            ]
        },
    },
    TableSpec {
        name: "admin_totp",
        // `last_step` is the replay guard and is copied with the secret (see
        // `totp_secrets`).
        select: "SELECT account_id, sealed_secret, last_step, enrolled_at FROM admin_totp",
        insert: "INSERT INTO admin_totp (account_id, sealed_secret, last_step, enrolled_at) VALUES (?1,?2,?3,?4)",
        map: |r| {
            vec![
                t(r, "account_id"),
                b(r, "sealed_secret"),
                i(r, "last_step"),
                i(r, "enrolled_at"),
            ]
        },
    },
    TableSpec {
        name: "admin_webauthn_credentials",
        select: "SELECT credential_id,account_id,cose_public_key,sign_count,transports,label,created_at,backup_eligible,backup_state,uv_initialized,state_revision FROM admin_webauthn_credentials",
        insert: "INSERT INTO admin_webauthn_credentials(credential_id,account_id,cose_public_key,sign_count,transports,label,created_at,backup_eligible,backup_state,uv_initialized,state_revision) VALUES(?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11)",
        map: |r| {
            vec![
                t(r, "credential_id"),
                t(r, "account_id"),
                b(r, "cose_public_key"),
                i(r, "sign_count"),
                t(r, "transports"),
                t(r, "label"),
                t(r, "created_at"),
                i(r, "backup_eligible"),
                i(r, "backup_state"),
                i(r, "uv_initialized"),
                i(r, "state_revision"),
            ]
        },
    },
    TableSpec {
        name: "admin_recovery_codes",
        select: "SELECT account_id,code_hash,used FROM admin_recovery_codes",
        insert: "INSERT INTO admin_recovery_codes(account_id,code_hash,used) VALUES(?1,?2,?3)",
        map: |r| vec![t(r, "account_id"), t(r, "code_hash"), i(r, "used")],
    },
    TableSpec {
        name: "admin_role",
        select: "SELECT id, name, builtin, description FROM admin_role",
        insert: "INSERT INTO admin_role (id, name, builtin, description) VALUES (?1,?2,?3,?4)",
        map: |r| {
            vec![
                t(r, "id"),
                t(r, "name"),
                i(r, "builtin"),
                t(r, "description"),
            ]
        },
    },
    TableSpec {
        name: "admin_role_capability",
        select: "SELECT role_id, capability FROM admin_role_capability",
        insert: "INSERT INTO admin_role_capability (role_id, capability) VALUES (?1,?2)",
        map: |r| vec![t(r, "role_id"), t(r, "capability")],
    },
    TableSpec {
        name: "admin_account_role",
        select: "SELECT account_id, role_id, domains_json FROM admin_account_role",
        insert: "INSERT INTO admin_account_role (account_id, role_id, domains_json) VALUES (?1,?2,?3)",
        map: |r| vec![t(r, "account_id"), t(r, "role_id"), ot(r, "domains_json")],
    },
    TableSpec {
        name: "admin_capability_override",
        select: "SELECT account_id, capability, effect FROM admin_capability_override",
        insert: "INSERT INTO admin_capability_override (account_id, capability, effect) VALUES (?1,?2,?3)",
        map: |r| vec![t(r, "account_id"), t(r, "capability"), t(r, "effect")],
    },
    TableSpec {
        name: "admin_saved_query",
        select: "SELECT id, admin_account_id, name, filter_json, created_at FROM admin_saved_query",
        insert: "INSERT INTO admin_saved_query (id, admin_account_id, name, filter_json, created_at) VALUES (?1,?2,?3,?4,?5)",
        map: |r| {
            vec![
                t(r, "id"),
                t(r, "admin_account_id"),
                t(r, "name"),
                t(r, "filter_json"),
                i(r, "created_at"),
            ]
        },
    },
    TableSpec {
        name: "audit_chain_head",
        // `audit_log` is copied with its `prev_hash`/`hash`/`seq`; the chain head,
        // checkpoints, retention (incl. legal hold) and sinks with their cursors must
        // follow, or the destination either fails chain verification or prunes rows
        // under legal hold, and a SIEM sink re-sends or skips rows.
        select: "SELECT id, last_id, hash FROM audit_chain_head",
        insert: "INSERT INTO audit_chain_head (id, last_id, hash) VALUES (?1,?2,?3)",
        map: |r| vec![i(r, "id"), i(r, "last_id"), ot(r, "hash")],
    },
    TableSpec {
        name: "audit_checkpoint",
        select: "SELECT id, kind, pruned_through_id, pruned_through_hash, ts FROM audit_checkpoint",
        insert: "INSERT INTO audit_checkpoint (id, kind, pruned_through_id, pruned_through_hash, ts) VALUES (?1,?2,?3,?4,?5)",
        map: |r| {
            vec![
                t(r, "id"),
                t(r, "kind"),
                i(r, "pruned_through_id"),
                ot(r, "pruned_through_hash"),
                i(r, "ts"),
            ]
        },
    },
    TableSpec {
        name: "audit_retention",
        select: "SELECT category, days, legal_hold, updated_by, updated_at FROM audit_retention",
        insert: "INSERT INTO audit_retention (category, days, legal_hold, updated_by, updated_at) VALUES (?1,?2,?3,?4,?5)",
        map: |r| {
            vec![
                t(r, "category"),
                oi(r, "days"),
                i(r, "legal_hold"),
                t(r, "updated_by"),
                i(r, "updated_at"),
            ]
        },
    },
    TableSpec {
        name: "audit_sink",
        select: "SELECT id, kind, sealed_config, enabled, cursor_id, last_ok_at, last_error, created_by, created_at FROM audit_sink",
        insert: "INSERT INTO audit_sink (id, kind, sealed_config, enabled, cursor_id, last_ok_at, last_error, created_by, created_at) VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9)",
        map: |r| {
            vec![
                t(r, "id"),
                t(r, "kind"),
                b(r, "sealed_config"),
                i(r, "enabled"),
                i(r, "cursor_id"),
                oi(r, "last_ok_at"),
                ot(r, "last_error"),
                t(r, "created_by"),
                i(r, "created_at"),
            ]
        },
    },
    TableSpec {
        name: "alert_rule",
        // Omission silently stops every admin alert.
        select: "SELECT id, name, condition_json, channels_json, throttle_secs, enabled, created_by, created_at, updated_at FROM alert_rule",
        insert: "INSERT INTO alert_rule (id, name, condition_json, channels_json, throttle_secs, enabled, created_by, created_at, updated_at) VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9)",
        map: |r| {
            vec![
                t(r, "id"),
                t(r, "name"),
                t(r, "condition_json"),
                t(r, "channels_json"),
                i(r, "throttle_secs"),
                i(r, "enabled"),
                t(r, "created_by"),
                i(r, "created_at"),
                i(r, "updated_at"),
            ]
        },
    },
    TableSpec {
        name: "alert_event",
        select: "SELECT id, rule_id, fired_at, summary, delivered_json, audit_id FROM alert_event",
        insert: "INSERT INTO alert_event (id, rule_id, fired_at, summary, delivered_json, audit_id) VALUES (?1,?2,?3,?4,?5,?6)",
        map: |r| {
            vec![
                t(r, "id"),
                t(r, "rule_id"),
                i(r, "fired_at"),
                t(r, "summary"),
                t(r, "delivered_json"),
                ot(r, "audit_id"),
            ]
        },
    },
    TableSpec {
        name: "signin_event",
        // Sign-in history the user sees in My activity; append-only like the other
        // audit logs.
        select: "SELECT id, account_id, username_attempted, ts, result, method, ip, user_agent, geo_country, reason, surface, backend, last_seen, repeat_count, aggregate_key FROM signin_event",
        insert: "INSERT INTO signin_event (id, account_id, username_attempted, ts, result, method, ip, user_agent, geo_country, reason, surface, backend, last_seen, repeat_count, aggregate_key) VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11,?12,?13,?14,?15)",
        map: |r| {
            vec![
                t(r, "id"),
                ot(r, "account_id"),
                t(r, "username_attempted"),
                i(r, "ts"),
                t(r, "result"),
                t(r, "method"),
                t(r, "ip"),
                t(r, "user_agent"),
                ot(r, "geo_country"),
                t(r, "reason"),
                t(r, "surface"),
                ot(r, "backend"),
                i(r, "last_seen"),
                i(r, "repeat_count"),
                ot(r, "aggregate_key"),
            ]
        },
    },
    TableSpec {
        name: "signin_identity",
        select: "SELECT account_id,address,seen_at,event_id FROM signin_identity",
        insert: "INSERT INTO signin_identity(account_id,address,seen_at,event_id) VALUES(?1,?2,?3,?4)",
        map: |r| {
            vec![
                t(r, "account_id"),
                t(r, "address"),
                i(r, "seen_at"),
                t(r, "event_id"),
            ]
        },
    },
    TableSpec {
        name: "auth_source_failure",
        select: "SELECT surface,source_ip,attempts_json,locked_until,expires_at,revision FROM auth_source_failure",
        insert: "INSERT INTO auth_source_failure(surface,source_ip,attempts_json,locked_until,expires_at,revision) VALUES(?1,?2,?3,?4,?5,?6)",
        map: |r| {
            vec![
                t(r, "surface"),
                t(r, "source_ip"),
                t(r, "attempts_json"),
                i(r, "locked_until"),
                i(r, "expires_at"),
                t(r, "revision"),
            ]
        },
    },
    TableSpec {
        name: "request_protection_subject",
        select: "SELECT subject_key,subject_kind,state_json,updated_at,expires_at,revision FROM request_protection_subject",
        insert: "INSERT INTO request_protection_subject(subject_key,subject_kind,state_json,updated_at,expires_at,revision) VALUES(?1,?2,?3,?4,?5,?6)",
        map: |r| {
            vec![
                t(r, "subject_key"),
                t(r, "subject_kind"),
                t(r, "state_json"),
                i(r, "updated_at"),
                i(r, "expires_at"),
                t(r, "revision"),
            ]
        },
    },
    TableSpec {
        name: "request_protection_event",
        select: "SELECT id,subject_key,subject_kind,observed_at,reason,action,enforced,retry_after_secs,counts_json FROM request_protection_event",
        insert: "INSERT INTO request_protection_event(id,subject_key,subject_kind,observed_at,reason,action,enforced,retry_after_secs,counts_json) VALUES(?1,?2,?3,?4,?5,?6,?7,?8,?9)",
        map: |r| {
            vec![
                t(r, "id"),
                t(r, "subject_key"),
                t(r, "subject_kind"),
                i(r, "observed_at"),
                t(r, "reason"),
                t(r, "action"),
                i(r, "enforced"),
                i(r, "retry_after_secs"),
                t(r, "counts_json"),
            ]
        },
    },
    TableSpec {
        name: "remembered_device",
        select: "SELECT id, account_id, token_hash, label, created_at, expires_at, last_used_at FROM remembered_device",
        insert: "INSERT INTO remembered_device (id, account_id, token_hash, label, created_at, expires_at, last_used_at) VALUES (?1,?2,?3,?4,?5,?6,?7)",
        map: |r| {
            vec![
                t(r, "id"),
                t(r, "account_id"),
                t(r, "token_hash"),
                t(r, "label"),
                i(r, "created_at"),
                i(r, "expires_at"),
                i(r, "last_used_at"),
            ]
        },
    },
    TableSpec {
        name: "branding",
        // Branding, user backgrounds, signature images, themes: content, nothing that
        // names the old host.
        select: "SELECT scope, config_json, updated_by, updated_at FROM branding",
        insert: "INSERT INTO branding (scope, config_json, updated_by, updated_at) VALUES (?1,?2,?3,?4)",
        map: |r| {
            vec![
                t(r, "scope"),
                t(r, "config_json"),
                t(r, "updated_by"),
                i(r, "updated_at"),
            ]
        },
    },
    TableSpec {
        name: "brand_asset",
        select: "SELECT id, scope, kind, owner_account, sha256, content_type, bytes, width, height, duration_ms, licence_ack, data, created_by, created_at FROM brand_asset",
        insert: "INSERT INTO brand_asset (id, scope, kind, owner_account, sha256, content_type, bytes, width, height, duration_ms, licence_ack, data, created_by, created_at) VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11,?12,?13,?14)",
        map: |r| {
            vec![
                t(r, "id"),
                t(r, "scope"),
                t(r, "kind"),
                ot(r, "owner_account"),
                t(r, "sha256"),
                t(r, "content_type"),
                i(r, "bytes"),
                oi(r, "width"),
                oi(r, "height"),
                oi(r, "duration_ms"),
                ot(r, "licence_ack"),
                b(r, "data"),
                t(r, "created_by"),
                i(r, "created_at"),
            ]
        },
    },
    TableSpec {
        name: "custom_css",
        select: "SELECT scope_kind, scope_id, source, sanitized, sha256, updated_by, updated_at FROM custom_css",
        insert: "INSERT INTO custom_css (scope_kind, scope_id, source, sanitized, sha256, updated_by, updated_at) VALUES (?1,?2,?3,?4,?5,?6,?7)",
        map: |r| {
            vec![
                t(r, "scope_kind"),
                t(r, "scope_id"),
                t(r, "source"),
                t(r, "sanitized"),
                t(r, "sha256"),
                t(r, "updated_by"),
                i(r, "updated_at"),
            ]
        },
    },
    TableSpec {
        name: "custom_theme",
        select: "SELECT id, name, appearance, tokens_json, enabled, scope, created_by, created_at, updated_at FROM custom_theme",
        insert: "INSERT INTO custom_theme (id, name, appearance, tokens_json, enabled, scope, created_by, created_at, updated_at) VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9)",
        map: |r| {
            vec![
                t(r, "id"),
                t(r, "name"),
                t(r, "appearance"),
                t(r, "tokens_json"),
                i(r, "enabled"),
                t(r, "scope"),
                t(r, "created_by"),
                i(r, "created_at"),
                i(r, "updated_at"),
            ]
        },
    },
    TableSpec {
        name: "custom_accent",
        select: "SELECT id, name, shades_json, enabled, scope, created_by, created_at, updated_at FROM custom_accent",
        insert: "INSERT INTO custom_accent (id, name, shades_json, enabled, scope, created_by, created_at, updated_at) VALUES (?1,?2,?3,?4,?5,?6,?7,?8)",
        map: |r| {
            vec![
                t(r, "id"),
                t(r, "name"),
                t(r, "shades_json"),
                i(r, "enabled"),
                t(r, "scope"),
                t(r, "created_by"),
                i(r, "created_at"),
                i(r, "updated_at"),
            ]
        },
    },
    TableSpec {
        name: "sieve_script",
        select: "SELECT account_id, name, content, active, size, state_json, updated_at FROM sieve_script",
        insert: "INSERT INTO sieve_script (account_id, name, content, active, size, state_json, updated_at) VALUES (?1,?2,?3,?4,?5,?6,?7)",
        map: |r| {
            vec![
                t(r, "account_id"),
                t(r, "name"),
                t(r, "content"),
                i(r, "active"),
                i(r, "size"),
                t(r, "state_json"),
                i(r, "updated_at"),
            ]
        },
    },
    TableSpec {
        name: "smime_trust_root",
        select: "SELECT id, scope, der, fingerprint, label, added_by, added_at FROM smime_trust_root",
        insert: "INSERT INTO smime_trust_root (id, scope, der, fingerprint, label, added_by, added_at) VALUES (?1,?2,?3,?4,?5,?6,?7)",
        map: |r| {
            vec![
                t(r, "id"),
                t(r, "scope"),
                b(r, "der"),
                t(r, "fingerprint"),
                t(r, "label"),
                t(r, "added_by"),
                i(r, "added_at"),
            ]
        },
    },
    TableSpec {
        name: "pki_revocation_cache",
        // A cached `revoked` status must not be forgotten while the responder is
        // unreachable from the new host.
        select: "SELECT issuer_key_hash, serial, status, source, this_update, next_update, checked_at FROM pki_revocation_cache",
        insert: "INSERT INTO pki_revocation_cache (issuer_key_hash, serial, status, source, this_update, next_update, checked_at) VALUES (?1,?2,?3,?4,?5,?6,?7)",
        map: |r| {
            vec![
                t(r, "issuer_key_hash"),
                t(r, "serial"),
                t(r, "status"),
                t(r, "source"),
                i(r, "this_update"),
                oi(r, "next_update"),
                i(r, "checked_at"),
            ]
        },
    },
    TableSpec {
        name: "identity_signature_default",
        select: "SELECT identity_id, mode, signature_name, placement, separator FROM identity_signature_default",
        insert: "INSERT INTO identity_signature_default (identity_id, mode, signature_name, placement, separator) VALUES (?1,?2,?3,?4,?5)",
        map: |r| {
            vec![
                t(r, "identity_id"),
                t(r, "mode"),
                t(r, "signature_name"),
                t(r, "placement"),
                i(r, "separator"),
            ]
        },
    },
    TableSpec {
        name: "domain_signature_template",
        select: "SELECT domain, name, html, text, user_editable, updated_by, updated_at FROM domain_signature_template",
        insert: "INSERT INTO domain_signature_template (domain, name, html, text, user_editable, updated_by, updated_at) VALUES (?1,?2,?3,?4,?5,?6,?7)",
        map: |r| {
            vec![
                t(r, "domain"),
                t(r, "name"),
                t(r, "html"),
                t(r, "text"),
                i(r, "user_editable"),
                t(r, "updated_by"),
                i(r, "updated_at"),
            ]
        },
    },
    TableSpec {
        name: "domain_disclaimer",
        select: "SELECT domain, html, text, enabled, updated_by, updated_at FROM domain_disclaimer",
        insert: "INSERT INTO domain_disclaimer (domain, html, text, enabled, updated_by, updated_at) VALUES (?1,?2,?3,?4,?5,?6)",
        map: |r| {
            vec![
                t(r, "domain"),
                t(r, "html"),
                t(r, "text"),
                i(r, "enabled"),
                t(r, "updated_by"),
                i(r, "updated_at"),
            ]
        },
    },
    TableSpec {
        name: "mdn_sender_rule",
        select: "SELECT account_id, address, decision, updated_at FROM mdn_sender_rule",
        insert: "INSERT INTO mdn_sender_rule (account_id, address, decision, updated_at) VALUES (?1,?2,?3,?4)",
        map: |r| {
            vec![
                t(r, "account_id"),
                t(r, "address"),
                t(r, "decision"),
                i(r, "updated_at"),
            ]
        },
    },
    TableSpec {
        name: "receipt_status",
        select: "SELECT account_id, sent_email_id, recipient, kind, status, reported_at, report_email_id FROM receipt_status",
        insert: "INSERT INTO receipt_status (account_id, sent_email_id, recipient, kind, status, reported_at, report_email_id) VALUES (?1,?2,?3,?4,?5,?6,?7)",
        map: |r| {
            vec![
                t(r, "account_id"),
                t(r, "sent_email_id"),
                t(r, "recipient"),
                t(r, "kind"),
                t(r, "status"),
                oi(r, "reported_at"),
                ot(r, "report_email_id"),
            ]
        },
    },
    TableSpec {
        name: "child_account",
        select: "SELECT owner_account_id, account_id, provider_id, display_name, accent_id, position, paused, added_at FROM child_account",
        insert: "INSERT INTO child_account (owner_account_id, account_id, provider_id, display_name, accent_id, position, paused, added_at) VALUES (?1,?2,?3,?4,?5,?6,?7,?8)",
        map: |r| {
            vec![
                t(r, "owner_account_id"),
                t(r, "account_id"),
                ot(r, "provider_id"),
                t(r, "display_name"),
                ot(r, "accent_id"),
                i(r, "position"),
                i(r, "paused"),
                i(r, "added_at"),
            ]
        },
    },
    TableSpec {
        name: "provider_profile",
        select: "SELECT id, name, kind, endpoints_json, oauth_client_id, tunnel_id, egress_route_id, scope, enabled, preset, created_by, updated_at FROM provider_profile",
        insert: "INSERT INTO provider_profile (id, name, kind, endpoints_json, oauth_client_id, tunnel_id, egress_route_id, scope, enabled, preset, created_by, updated_at) VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11,?12)",
        map: |r| {
            vec![
                t(r, "id"),
                t(r, "name"),
                t(r, "kind"),
                t(r, "endpoints_json"),
                ot(r, "oauth_client_id"),
                ot(r, "tunnel_id"),
                ot(r, "egress_route_id"),
                t(r, "scope"),
                i(r, "enabled"),
                ot(r, "preset"),
                t(r, "created_by"),
                i(r, "updated_at"),
            ]
        },
    },
    TableSpec {
        name: "account_oauth",
        // Sealed refresh tokens of linked accounts, copied like
        // `bridge_oauth_tokens`. The `oauth_client` registration they were issued to
        // is deliberately NOT copied (its redirect names the old host); the operator
        // re-enters it with the same client id and these tokens keep working.
        select: "SELECT account_id, provider_id, sealed_refresh_token, expires_at, scope, needs_signin, updated_at FROM account_oauth",
        insert: "INSERT INTO account_oauth (account_id, provider_id, sealed_refresh_token, expires_at, scope, needs_signin, updated_at) VALUES (?1,?2,?3,?4,?5,?6,?7)",
        map: |r| {
            vec![
                t(r, "account_id"),
                t(r, "provider_id"),
                b(r, "sealed_refresh_token"),
                i(r, "expires_at"),
                t(r, "scope"),
                i(r, "needs_signin"),
                i(r, "updated_at"),
            ]
        },
    },
    TableSpec {
        name: "delegation_grant",
        select: "SELECT id, owner_account_id, delegate_owner_account_id, delegate_group_id, scope_kind, mailbox_ids_json, permissions_json, expires_at, created_at, revoked_at FROM delegation_grant",
        insert: "INSERT INTO delegation_grant (id, owner_account_id, delegate_owner_account_id, delegate_group_id, scope_kind, mailbox_ids_json, permissions_json, expires_at, created_at, revoked_at) VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9,?10)",
        map: |r| {
            vec![
                t(r, "id"),
                t(r, "owner_account_id"),
                ot(r, "delegate_owner_account_id"),
                ot(r, "delegate_group_id"),
                t(r, "scope_kind"),
                t(r, "mailbox_ids_json"),
                t(r, "permissions_json"),
                oi(r, "expires_at"),
                i(r, "created_at"),
                oi(r, "revoked_at"),
            ]
        },
    },
    TableSpec {
        name: "shared_mailbox",
        select: "SELECT account_id, domain, name, created_by, created_at FROM shared_mailbox",
        insert: "INSERT INTO shared_mailbox (account_id, domain, name, created_by, created_at) VALUES (?1,?2,?3,?4,?5)",
        map: |r| {
            vec![
                t(r, "account_id"),
                t(r, "domain"),
                t(r, "name"),
                t(r, "created_by"),
                i(r, "created_at"),
            ]
        },
    },
    TableSpec {
        name: "org_view",
        select: "SELECT id, scope, name, query_text, settings_json, mandatory, created_by, updated_at FROM org_view",
        insert: "INSERT INTO org_view (id, scope, name, query_text, settings_json, mandatory, created_by, updated_at) VALUES (?1,?2,?3,?4,?5,?6,?7,?8)",
        map: |r| {
            vec![
                t(r, "id"),
                t(r, "scope"),
                t(r, "name"),
                t(r, "query_text"),
                t(r, "settings_json"),
                i(r, "mandatory"),
                t(r, "created_by"),
                i(r, "updated_at"),
            ]
        },
    },
    TableSpec {
        name: "identity_verification",
        select: "SELECT token_hash, identity_id, sent_at, expires_at, consumed FROM identity_verification",
        insert: "INSERT INTO identity_verification (token_hash, identity_id, sent_at, expires_at, consumed) VALUES (?1,?2,?3,?4,?5)",
        map: |r| {
            vec![
                t(r, "token_hash"),
                t(r, "identity_id"),
                i(r, "sent_at"),
                i(r, "expires_at"),
                i(r, "consumed"),
            ]
        },
    },
    TableSpec {
        name: "template",
        // FK parent of `template_version`.
        select: "SELECT id, owner_kind, owner_id, folder, name, subject, html, text, attachments_json, identity_id, recipients_json, fields_json, mandatory, user_copyable, version, usage_count, created_by, created_at, updated_at FROM template",
        insert: "INSERT INTO template (id, owner_kind, owner_id, folder, name, subject, html, text, attachments_json, identity_id, recipients_json, fields_json, mandatory, user_copyable, version, usage_count, created_by, created_at, updated_at) VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11,?12,?13,?14,?15,?16,?17,?18,?19)",
        map: |r| {
            vec![
                t(r, "id"),
                t(r, "owner_kind"),
                t(r, "owner_id"),
                t(r, "folder"),
                t(r, "name"),
                t(r, "subject"),
                t(r, "html"),
                t(r, "text"),
                t(r, "attachments_json"),
                ot(r, "identity_id"),
                t(r, "recipients_json"),
                t(r, "fields_json"),
                i(r, "mandatory"),
                i(r, "user_copyable"),
                i(r, "version"),
                i(r, "usage_count"),
                t(r, "created_by"),
                i(r, "created_at"),
                i(r, "updated_at"),
            ]
        },
    },
    TableSpec {
        name: "template_version",
        select: "SELECT template_id, version, subject, html, text, attachments_json, created_by, created_at FROM template_version",
        insert: "INSERT INTO template_version (template_id, version, subject, html, text, attachments_json, created_by, created_at) VALUES (?1,?2,?3,?4,?5,?6,?7,?8)",
        map: |r| {
            vec![
                t(r, "template_id"),
                i(r, "version"),
                t(r, "subject"),
                t(r, "html"),
                t(r, "text"),
                t(r, "attachments_json"),
                t(r, "created_by"),
                i(r, "created_at"),
            ]
        },
    },
    TableSpec {
        name: "user_group",
        // FK parent of `user_group_member`; `feature_profile` likewise precedes its
        // assignments.
        select: "SELECT id, name, domain, source, ldap_dn, created_at FROM user_group",
        insert: "INSERT INTO user_group (id, name, domain, source, ldap_dn, created_at) VALUES (?1,?2,?3,?4,?5,?6)",
        map: |r| {
            vec![
                t(r, "id"),
                t(r, "name"),
                t(r, "domain"),
                t(r, "source"),
                ot(r, "ldap_dn"),
                i(r, "created_at"),
            ]
        },
    },
    TableSpec {
        name: "user_group_member",
        select: "SELECT group_id, account_id FROM user_group_member",
        insert: "INSERT INTO user_group_member (group_id, account_id) VALUES (?1,?2)",
        map: |r| vec![t(r, "group_id"), t(r, "account_id")],
    },
    TableSpec {
        name: "feature_profile",
        select: "SELECT id, name, values_json, builtin, updated_at FROM feature_profile",
        insert: "INSERT INTO feature_profile (id, name, values_json, builtin, updated_at) VALUES (?1,?2,?3,?4,?5)",
        map: |r| {
            vec![
                t(r, "id"),
                t(r, "name"),
                t(r, "values_json"),
                i(r, "builtin"),
                i(r, "updated_at"),
            ]
        },
    },
    TableSpec {
        name: "feature_profile_assignment",
        select: "SELECT scope_kind, scope_id, profile_id, priority FROM feature_profile_assignment",
        insert: "INSERT INTO feature_profile_assignment (scope_kind, scope_id, profile_id, priority) VALUES (?1,?2,?3,?4)",
        map: |r| {
            vec![
                t(r, "scope_kind"),
                t(r, "scope_id"),
                t(r, "profile_id"),
                i(r, "priority"),
            ]
        },
    },
    TableSpec {
        name: "mail_user",
        // t32-q: the user directory, its account mapping, allow patterns and usage.
        // Left behind, a `known`/`provisioned` sign-in policy would refuse everyone on
        // the destination and a disabled user would lose the mirror of that state.
        select: "SELECT address, domain, local_part, origin, status, display_name, note, created_at, created_by, first_signin_at, last_signin_at, status_changed_at, status_changed_by, status_reason, expires_at, deleted_at FROM mail_user",
        insert: "INSERT INTO mail_user (address, domain, local_part, origin, status, display_name, note, created_at, created_by, first_signin_at, last_signin_at, status_changed_at, status_changed_by, status_reason, expires_at, deleted_at) VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11,?12,?13,?14,?15,?16)",
        map: |r| {
            vec![
                t(r, "address"),
                t(r, "domain"),
                t(r, "local_part"),
                t(r, "origin"),
                t(r, "status"),
                ot(r, "display_name"),
                ot(r, "note"),
                i(r, "created_at"),
                ot(r, "created_by"),
                oi(r, "first_signin_at"),
                oi(r, "last_signin_at"),
                oi(r, "status_changed_at"),
                ot(r, "status_changed_by"),
                ot(r, "status_reason"),
                oi(r, "expires_at"),
                oi(r, "deleted_at"),
            ]
        },
    },
    TableSpec {
        name: "mail_user_account",
        select: "SELECT account_id, address, mode, first_seen, last_seen FROM mail_user_account",
        insert: "INSERT INTO mail_user_account (account_id, address, mode, first_seen, last_seen) VALUES (?1,?2,?3,?4,?5)",
        map: |r| {
            vec![
                t(r, "account_id"),
                t(r, "address"),
                t(r, "mode"),
                i(r, "first_seen"),
                i(r, "last_seen"),
            ]
        },
    },
    TableSpec {
        name: "signin_allow",
        select: "SELECT id, domain, pattern, note, expires_at, max_uses, uses, created_by, created_at FROM signin_allow",
        insert: "INSERT INTO signin_allow (id, domain, pattern, note, expires_at, max_uses, uses, created_by, created_at) VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9)",
        map: |r| {
            vec![
                t(r, "id"),
                t(r, "domain"),
                t(r, "pattern"),
                ot(r, "note"),
                oi(r, "expires_at"),
                oi(r, "max_uses"),
                i(r, "uses"),
                t(r, "created_by"),
                i(r, "created_at"),
            ]
        },
    },
    TableSpec {
        name: "usage_counter",
        select: "SELECT address, domain, metric, day, value FROM usage_counter",
        insert: "INSERT INTO usage_counter (address, domain, metric, day, value) VALUES (?1,?2,?3,?4,?5)",
        map: |r| {
            vec![
                t(r, "address"),
                t(r, "domain"),
                t(r, "metric"),
                i(r, "day"),
                i(r, "value"),
            ]
        },
    },
    TableSpec {
        name: "usage_storage",
        select: "SELECT address, domain, blob_bytes, blob_count, content_bytes, cache_bytes, cache_messages, search_docs, breakdown_json, measured_at, measuring_until, upstream_used_bytes, upstream_limit_bytes, upstream_json, upstream_status, upstream_checked_at, warn_level FROM usage_storage",
        insert: "INSERT INTO usage_storage (address, domain, blob_bytes, blob_count, content_bytes, cache_bytes, cache_messages, search_docs, breakdown_json, measured_at, measuring_until, upstream_used_bytes, upstream_limit_bytes, upstream_json, upstream_status, upstream_checked_at, warn_level) VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11,?12,?13,?14,?15,?16,?17)",
        map: |r| {
            vec![
                t(r, "address"),
                t(r, "domain"),
                i(r, "blob_bytes"),
                i(r, "blob_count"),
                i(r, "content_bytes"),
                i(r, "cache_bytes"),
                i(r, "cache_messages"),
                oi(r, "search_docs"),
                ot(r, "breakdown_json"),
                oi(r, "measured_at"),
                oi(r, "measuring_until"),
                oi(r, "upstream_used_bytes"),
                oi(r, "upstream_limit_bytes"),
                ot(r, "upstream_json"),
                ot(r, "upstream_status"),
                oi(r, "upstream_checked_at"),
                i(r, "warn_level"),
            ]
        },
    },
    TableSpec {
        name: "config_snapshot",
        select: "SELECT id, created_at, created_by, reason, sha256, json FROM config_snapshot",
        insert: "INSERT INTO config_snapshot (id, created_at, created_by, reason, sha256, json) VALUES (?1,?2,?3,?4,?5,?6)",
        map: |r| {
            vec![
                t(r, "id"),
                i(r, "created_at"),
                t(r, "created_by"),
                t(r, "reason"),
                t(r, "sha256"),
                t(r, "json"),
            ]
        },
    },
    TableSpec {
        name: "maintenance_window",
        // t32-cfg operational state and history. A scheduled window or task keeps
        // its schedule; `lease_owner`/`lease_until` name the old process and simply
        // expire. `account_ban` omission would lift every identity ban (fails OPEN).
        select: "SELECT id, scope_kind, scope_id, mode, starts_at, ends_at, tz, recurrence_json, announce_minutes, message_json, created_by, created_at, updated_by, updated_at, occurrence_ended_at, cancelled_at, cancelled_by FROM maintenance_window",
        insert: "INSERT INTO maintenance_window (id, scope_kind, scope_id, mode, starts_at, ends_at, tz, recurrence_json, announce_minutes, message_json, created_by, created_at, updated_by, updated_at, occurrence_ended_at, cancelled_at, cancelled_by) VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11,?12,?13,?14,?15,?16,?17)",
        map: |r| {
            vec![
                t(r, "id"),
                t(r, "scope_kind"),
                t(r, "scope_id"),
                t(r, "mode"),
                i(r, "starts_at"),
                oi(r, "ends_at"),
                t(r, "tz"),
                ot(r, "recurrence_json"),
                i(r, "announce_minutes"),
                t(r, "message_json"),
                t(r, "created_by"),
                i(r, "created_at"),
                t(r, "updated_by"),
                i(r, "updated_at"),
                oi(r, "occurrence_ended_at"),
                oi(r, "cancelled_at"),
                ot(r, "cancelled_by"),
            ]
        },
    },
    TableSpec {
        name: "scheduled_task",
        select: "SELECT id, kind, schedule_json, params_json, enabled, only_in_window, created_by, created_at, updated_at, last_run_at, last_status, next_run_at, lease_owner, lease_until FROM scheduled_task",
        insert: "INSERT INTO scheduled_task (id, kind, schedule_json, params_json, enabled, only_in_window, created_by, created_at, updated_at, last_run_at, last_status, next_run_at, lease_owner, lease_until) VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11,?12,?13,?14)",
        map: |r| {
            vec![
                t(r, "id"),
                t(r, "kind"),
                t(r, "schedule_json"),
                t(r, "params_json"),
                i(r, "enabled"),
                i(r, "only_in_window"),
                t(r, "created_by"),
                i(r, "created_at"),
                i(r, "updated_at"),
                oi(r, "last_run_at"),
                ot(r, "last_status"),
                oi(r, "next_run_at"),
                ot(r, "lease_owner"),
                oi(r, "lease_until"),
            ]
        },
    },
    TableSpec {
        name: "job_run",
        select: "SELECT id, task_id, kind, instance, started_at, heartbeat_at, finished_at, status, summary, detail_json, triggered_by FROM job_run",
        insert: "INSERT INTO job_run (id, task_id, kind, instance, started_at, heartbeat_at, finished_at, status, summary, detail_json, triggered_by) VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11)",
        map: |r| {
            vec![
                t(r, "id"),
                ot(r, "task_id"),
                t(r, "kind"),
                t(r, "instance"),
                i(r, "started_at"),
                i(r, "heartbeat_at"),
                oi(r, "finished_at"),
                t(r, "status"),
                ot(r, "summary"),
                ot(r, "detail_json"),
                t(r, "triggered_by"),
            ]
        },
    },
    TableSpec {
        name: "diag_run",
        select: "SELECT id, suite, target, started_at, finished_at, status, results_json, triggered_by, task_id FROM diag_run",
        insert: "INSERT INTO diag_run (id, suite, target, started_at, finished_at, status, results_json, triggered_by, task_id) VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9)",
        map: |r| {
            vec![
                t(r, "id"),
                t(r, "suite"),
                ot(r, "target"),
                i(r, "started_at"),
                oi(r, "finished_at"),
                t(r, "status"),
                t(r, "results_json"),
                t(r, "triggered_by"),
                ot(r, "task_id"),
            ]
        },
    },
    TableSpec {
        name: "account_ban",
        select: "SELECT id, kind, value, surface, reason, user_note, starts_at, expires_at, source, created_by, created_at, lifted_at, lifted_by, lift_reason, hits, last_hit_at FROM account_ban",
        insert: "INSERT INTO account_ban (id, kind, value, surface, reason, user_note, starts_at, expires_at, source, created_by, created_at, lifted_at, lifted_by, lift_reason, hits, last_hit_at) VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11,?12,?13,?14,?15,?16)",
        map: |r| {
            vec![
                t(r, "id"),
                t(r, "kind"),
                t(r, "value"),
                t(r, "surface"),
                t(r, "reason"),
                ot(r, "user_note"),
                i(r, "starts_at"),
                oi(r, "expires_at"),
                t(r, "source"),
                t(r, "created_by"),
                i(r, "created_at"),
                oi(r, "lifted_at"),
                ot(r, "lifted_by"),
                ot(r, "lift_reason"),
                i(r, "hits"),
                oi(r, "last_hit_at"),
            ]
        },
    },
    // Approved cloud authority and account-bound sealed credentials must move
    // together. The server key is preserved by the store migration contract.
    TableSpec {
        name: "nextcloud_instance",
        select: "SELECT id,name,base_url,enabled,revision,ca_pem FROM nextcloud_instance",
        insert: "INSERT INTO nextcloud_instance(id,name,base_url,enabled,revision,ca_pem) VALUES(?1,?2,?3,?4,?5,?6)",
        map: |r| {
            vec![
                t(r, "id"),
                t(r, "name"),
                t(r, "base_url"),
                i(r, "enabled"),
                t(r, "revision"),
                t(r, "ca_pem"),
            ]
        },
    },
    TableSpec {
        name: "nextcloud_account_link",
        select: "SELECT account_id,instance_id,instance_revision,username,sealed_password,revision FROM nextcloud_account_link",
        insert: "INSERT INTO nextcloud_account_link(account_id,instance_id,instance_revision,username,sealed_password,revision) VALUES(?1,?2,?3,?4,?5,?6)",
        map: |r| {
            vec![
                t(r, "account_id"),
                t(r, "instance_id"),
                t(r, "instance_revision"),
                t(r, "username"),
                b(r, "sealed_password"),
                t(r, "revision"),
            ]
        },
    },
    TableSpec {
        name: "nextcloud_pim_binding",
        select: "SELECT account_id,collection_id,kind,instance_id,link_revision,href,enabled FROM nextcloud_pim_binding",
        insert: "INSERT INTO nextcloud_pim_binding(account_id,collection_id,kind,instance_id,link_revision,href,enabled) VALUES(?1,?2,?3,?4,?5,?6,?7)",
        map: |r| {
            vec![
                t(r, "account_id"),
                t(r, "collection_id"),
                t(r, "kind"),
                t(r, "instance_id"),
                t(r, "link_revision"),
                t(r, "href"),
                i(r, "enabled"),
            ]
        },
    },
    TableSpec {
        name: "metric_rollup",
        select: "SELECT series, hour, instance, value_json FROM metric_rollup",
        insert: "INSERT INTO metric_rollup (series, hour, instance, value_json) VALUES (?1,?2,?3,?4)",
        map: |r| {
            vec![
                t(r, "series"),
                i(r, "hour"),
                t(r, "instance"),
                t(r, "value_json"),
            ]
        },
    },
    TableSpec {
        name: "data_export",
        // `blob_ref` points at the sealed archive on the upload backend, like
        // `uploaded_blobs.storage_key`: if the move also changes the upload
        // directory, the archive must move with it.
        select: "SELECT id, address, requested_by, status, item_counts_json, bytes, blob_ref, created_at, expires_at, downloaded_at FROM data_export",
        insert: "INSERT INTO data_export (id, address, requested_by, status, item_counts_json, bytes, blob_ref, created_at, expires_at, downloaded_at) VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9,?10)",
        map: |r| {
            vec![
                t(r, "id"),
                t(r, "address"),
                t(r, "requested_by"),
                t(r, "status"),
                t(r, "item_counts_json"),
                oi(r, "bytes"),
                ot(r, "blob_ref"),
                i(r, "created_at"),
                i(r, "expires_at"),
                oi(r, "downloaded_at"),
            ]
        },
    },
];
