//! `mailwoman migrate-store` engine (t6-e1; plan §1.1, §2.1, §4.2): copy a
//! populated SQLite store into the (Postgres) backend held by `self`, row-for-row,
//! preserving every id, timestamp, and sealed blob byte-for-byte.
//!
//! The copy runs inside a single destination transaction. On Postgres it issues
//! `SET CONSTRAINTS ALL DEFERRED` first (the schema declares every foreign key
//! `DEFERRABLE`), so cross-table and self-referential (`mailboxes.parent_id`)
//! references need not be inserted in dependency order — the whole graph is
//! validated at commit. Only the 0001–0006 tables the store manages are copied;
//! the 0007 admin/OAuth/webhook tables are provisioned empty (a `migrate-store`
//! moves an existing *mail* store — those surfaces are configured post-migration).

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

/// Every 0001–0006 table, in FK-parent-first order (belt-and-braces alongside the
/// deferred constraints).
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
        select: "SELECT id, account_id, email_id, identity_id, send_at, undo_status, hold_seconds, created_at FROM submissions",
        insert: "INSERT INTO submissions (id, account_id, email_id, identity_id, send_at, undo_status, hold_seconds, created_at) VALUES (?1,?2,?3,?4,?5,?6,?7,?8)",
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
            ]
        },
    },
    TableSpec {
        name: "identities",
        select: "SELECT id, account_id, name, email, reply_to, signature_html, signature_text, sent_mailbox_id, source FROM identities",
        insert: "INSERT INTO identities (id, account_id, name, email, reply_to, signature_html, signature_text, sent_mailbox_id, source) VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9)",
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
        select: "SELECT id, account_id, notebook_id, title, tags_json, color, pinned, body_html_sealed, body_text_sealed, links_json, created_at, updated_at FROM notes",
        insert: "INSERT INTO notes (id, account_id, notebook_id, title, tags_json, color, pinned, body_html_sealed, body_text_sealed, links_json, created_at, updated_at) VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11,?12)",
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
];
