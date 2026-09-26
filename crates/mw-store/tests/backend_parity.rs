//! Dual-backend parity + `migrate-store` integration tests (t6-e1; plan §2.1, §3
//! e1 acceptance). The SQLite path runs ALWAYS. The Postgres path runs only when
//! `DATABASE_URL_PG` (or `MW_TEST_PG`) points at a live server (CI provides
//! `postgres:16`, plan §11); otherwise it logs a SKIP and the SQLite assertions
//! still run — the suite never silently passes the PG path.
//!
//! `backend_parity` is the table-driven mock-vs-real discipline applied to
//! backends: a scripted sequence of repo calls runs on BOTH backends and the
//! backend-independent result snapshot must be byte-identical.

use mw_store::{
    AccountKind, AddressBookRow, AssistConfigRow, AuditRow, BridgeAccountRow, BridgeOauthTokenRow,
    CacheScopeRow, CalendarRow, ContactRow, Credentials, EventInstanceRow, EventRow,
    EwsAccountCred, MailboxUpsert, MaskedEmailRow, MessageUpsert, NewAccount, NoteRow,
    NotificationRulesRow, PasswdConfigRow, PluginKvLimits, ServerKey, SignatureRow, SsoConfigRow,
    Store, StoreKeyMaterialRow, SubmissionRow, TwofaPolicyRow, WebauthnCredentialRow,
    ZeroAccessRow,
};

// Test-support helper; not part of the shipped `mw-store` library, so it is
// reached by path rather than through the crate root. See its module docs.
#[path = "../src/test_db.rs"]
mod test_db;

// The env-gate helper, reached by path for the same reason `test_db` is: it is
// test support, not library surface. It is a leaf file over `std` alone, so the
// include carries no dependency from `mw-store` to `mw-server`. Both CI jobs that
// run this target — `migrate-store-smoke`, whose only test is below, and
// `store-dual-backend` — boot Postgres, so both set `MW_REQUIRE_LIVE` and need the
// two Postgres legs here to fail rather than skip. See `gate.rs` module docs.
#[allow(dead_code)]
#[path = "../../mw-server/tests/common/gate.rs"]
mod gate;

fn key() -> ServerKey {
    ServerKey::from_bytes(&[7u8; 32]).unwrap()
}

fn pg_dsn() -> Option<String> {
    std::env::var("DATABASE_URL_PG")
        .ok()
        .or_else(|| std::env::var("MW_TEST_PG").ok())
        .filter(|s| !s.trim().is_empty())
}

/// Both PG tests share one database and each `TRUNCATE`s it; this process-wide
/// async lock keeps their PG sections from interleaving (cargo runs tests in
/// parallel threads).
fn pg_lock() -> &'static tokio::sync::Mutex<()> {
    static L: std::sync::OnceLock<tokio::sync::Mutex<()>> = std::sync::OnceLock::new();
    L.get_or_init(|| tokio::sync::Mutex::new(()))
}

/// Every store table, for TRUNCATE between Postgres runs.
const ALL_TABLES: &str = "sessions, settings, accounts, mailboxes, messages, bodies, threads, \
    pop3_uidl, sync_state, message_meta, tags, saved_searches, submissions, identities, changes, \
    calendars, calendar_shares, events, event_instances, tasks, notebooks, notes, address_books, \
    contacts, contact_groups, pim_changes, crypto_keys, key_associations, security_verdicts, \
    dlp_audit, sender_controls, store_key_material, push_subscriptions, push_config, \
    native_sessions, sso_config, sso_login_audit, quotas, zeroaccess_accounts, \
    crypto_changes, audit_log, twofa_policy, totp_secrets, webauthn_credentials, \
    recovery_codes, passwd_config, signatures, notification_rules, \
    remote_image_grants, masked_email, ews_account_cred, bridge_accounts, \
    bridge_oauth_tokens, uploaded_blobs, message_embeddings, plugin_kv, \
    assist_config, cache_scope, password_change_audit, assist_audit";

async fn truncate_pg(dsn: &str) {
    use sqlx::postgres::PgPoolOptions;
    let pool = PgPoolOptions::new()
        .max_connections(1)
        .connect(dsn)
        .await
        .expect("connect pg for truncate");
    sqlx::query(&format!(
        "TRUNCATE TABLE {ALL_TABLES} RESTART IDENTITY CASCADE"
    ))
    .execute(&pool)
    .await
    .expect("truncate pg");
}

/// A scripted sequence exercising a representative method from every repo module,
/// returning a backend-independent snapshot (no server-minted random ids leak in).
async fn run_ops(s: &Store) -> Vec<String> {
    let mut out: Vec<String> = Vec::new();

    // ---- V0: settings + sessions ----
    s.set_setting("theme", "grove-dark").await.unwrap();
    s.set_setting("theme", "grove-light").await.unwrap();
    out.push(format!(
        "setting={:?}",
        s.get_setting("theme").await.unwrap()
    ));

    let creds = Credentials {
        username: "u@e".into(),
        password: "hunter2".into(),
    };
    let sess = s
        .create_session("acctX", "u@e", "http://j", "http://a", &creds)
        .await
        .unwrap();
    out.push(format!(
        "session_creds={:?}",
        s.get_session(&sess).await.unwrap().credentials
    ));

    // ---- V1: account / mailbox / message / body / thread / pop3 / cursor ----
    let account = s
        .create_account(
            &NewAccount {
                kind: AccountKind::Imap,
                host: "imap.example",
                port: 993,
                tls: "implicit",
                username: "u@e",
                sync_policy_json: r#"{"keep":true}"#,
            },
            &creds,
        )
        .await
        .unwrap();
    let acc = s.get_account(&account).await.unwrap();
    out.push(format!(
        "account={}:{}:{}",
        acc.host, acc.port, acc.username
    ));
    out.push(format!(
        "account_creds_ok={}",
        s.account_credentials(&account).await.unwrap() == creds
    ));

    let mbox = s
        .upsert_mailbox(&MailboxUpsert {
            account_id: &account,
            name: "INBOX",
            role: Some("inbox"),
            uidvalidity: 100,
            uidnext: 1,
            highestmodseq: 0,
            total: 0,
            unread: 0,
            parent_id: None,
        })
        .await
        .unwrap();
    // Idempotent upsert refreshes counts, same row.
    let mbox2 = s
        .upsert_mailbox(&MailboxUpsert {
            account_id: &account,
            name: "INBOX",
            role: Some("inbox"),
            uidvalidity: 100,
            uidnext: 42,
            highestmodseq: 9,
            total: 5,
            unread: 2,
            parent_id: None,
        })
        .await
        .unwrap();
    out.push(format!("mailbox_idempotent={}", mbox == mbox2));
    let mb = s.get_mailbox(&mbox).await.unwrap();
    out.push(format!(
        "mailbox_counts={}:{}:{}:{}",
        mb.uidnext, mb.highestmodseq, mb.total, mb.unread
    ));

    let body_ref = s
        .put_body(&account, b"raw\r\n\r\nsecret-body")
        .await
        .unwrap();
    out.push(format!(
        "body_ok={}",
        s.get_body(&body_ref).await.unwrap().as_deref() == Some(&b"raw\r\n\r\nsecret-body"[..])
    ));

    let sid = s
        .upsert_message(&MessageUpsert {
            account_id: &account,
            mailbox_id: &mbox,
            uid: 5,
            uidvalidity: 100,
            message_id: Some("<a@x>"),
            thread_id: None,
            internaldate: Some("2026-07-01T10:00:00Z"),
            size: 1024,
            flags_json: r#"["Seen"]"#,
            envelope: Some(br#"{"subject":"private-subject"}"#),
            blob_ref: Some(&body_ref),
        })
        .await
        .unwrap();
    // Re-key across a UIDVALIDITY change carries the stable id.
    s.revalidate_mailbox(&mbox, 200).await.unwrap();
    let sid2 = s
        .upsert_message(&MessageUpsert {
            account_id: &account,
            mailbox_id: &mbox,
            uid: 9,
            uidvalidity: 200,
            message_id: Some("<a@x>"),
            thread_id: None,
            internaldate: Some("2026-07-01T10:00:00Z"),
            size: 1024,
            flags_json: r#"["Seen"]"#,
            envelope: None,
            blob_ref: None,
        })
        .await
        .unwrap();
    out.push(format!("stable_id_preserved={}", sid == sid2));
    out.push(format!(
        "envelope_ok={}",
        s.get_envelope(&sid).await.unwrap().as_deref()
            == Some(&br#"{"subject":"private-subject"}"#[..])
    ));
    s.set_flags(&sid, r#"["Seen","Flagged"]"#).await.unwrap();
    out.push(format!(
        "flags={}",
        s.get_message(&sid).await.unwrap().flags_json
    ));
    let loc = s.message_location(&sid).await.unwrap().unwrap();
    out.push(format!("loc={}:{}", loc.uidvalidity, loc.uid));

    let t1 = s.assign_thread(&account, "<root@x>").await.unwrap();
    let t2 = s.assign_thread(&account, "<root@x>").await.unwrap();
    out.push(format!("thread_idempotent={}", t1 == t2));

    s.record_uidl(&account, "UID-A", "stable-a").await.unwrap();
    s.record_uidl(&account, "UID-A", "stable-a").await.unwrap();
    out.push(format!(
        "seen_uidls={}",
        s.seen_uidls(&account).await.unwrap().len()
    ));

    s.save_cursor(&account, &mbox, r#"{"k":1}"#).await.unwrap();
    out.push(format!(
        "cursor={:?}",
        s.load_cursor(&account, &mbox).await.unwrap()
    ));

    // ---- V2: change log + submissions + list ordering ----
    let c1 = s
        .record_change(&account, "Email", "e1", "created")
        .await
        .unwrap();
    let c2 = s
        .record_change(&account, "Email", "e2", "created")
        .await
        .unwrap();
    out.push(format!("changes={}:{}", c1, c2));
    out.push(format!(
        "current_state={}",
        s.current_state(&account, "Email").await.unwrap()
    ));
    out.push(format!(
        "changes_since={}",
        s.changes_since(&account, "Email", 1).await.unwrap().len()
    ));
    s.insert_submission(&SubmissionRow {
        id: "sub1".into(),
        account_id: account.clone(),
        email_id: sid.clone(),
        identity_id: None,
        send_at: None,
        undo_status: "pending".into(),
        hold_seconds: 10,
        created_at: "2026-07-01T10:00:00Z".into(),
    })
    .await
    .unwrap();
    out.push(format!(
        "pending_subs={}",
        s.pending_submissions().await.unwrap().len()
    ));

    // ---- V3: calendar / event range / note seal / contact autocomplete ----
    s.upsert_calendar(&CalendarRow {
        id: "cal1".into(),
        account_id: account.clone(),
        name: "Personal".into(),
        color: "#36f".into(),
        sort_order: 0,
        is_visible: true,
        role: Some("default".into()),
        caldav_url: None,
        sync_token: None,
        ctag: None,
        is_overlay: false,
        component: "VEVENT".into(),
    })
    .await
    .unwrap();
    s.upsert_event(&EventRow {
        id: "ev1".into(),
        calendar_id: "cal1".into(),
        uid: "uid-1".into(),
        etag: Some("\"e1\"".into()),
        ical_raw: "BEGIN:VCALENDAR\r\nEND:VCALENDAR\r\n".into(),
        start_utc: Some("2026-07-11T09:00:00Z".into()),
        end_utc: Some("2026-07-11T10:00:00Z".into()),
        tzid: Some("UTC".into()),
        rrule: None,
        status: "confirmed".into(),
        json: Some(b"{}".to_vec()),
    })
    .await
    .unwrap();
    s.replace_event_instances(
        "ev1",
        &[EventInstanceRow {
            event_id: "ev1".into(),
            instance_start_utc: "2026-07-11T09:00:00Z".into(),
            instance_end_utc: "2026-07-11T10:00:00Z".into(),
        }],
    )
    .await
    .unwrap();
    out.push(format!(
        "events_in_range={}",
        s.events_in_range(&account, "2026-07-11T00:00:00Z", "2026-07-12T00:00:00Z")
            .await
            .unwrap()
            .len()
    ));

    s.upsert_note(&NoteRow {
        id: "n1".into(),
        account_id: account.clone(),
        notebook_id: None,
        title: "Groceries".into(),
        tags_json: "[\"home\"]".into(),
        color: "#fc0".into(),
        pinned: true,
        body_html: "<p>milk SUPERSECRET eggs</p>".into(),
        body_text: "milk SUPERSECRET eggs".into(),
        links_json: "[]".into(),
        created_at: "2026-07-11T00:00:00Z".into(),
        updated_at: "2026-07-11T00:00:00Z".into(),
    })
    .await
    .unwrap();
    out.push(format!(
        "note_body={}",
        s.get_note("n1").await.unwrap().unwrap().body_text
    ));

    s.upsert_address_book(&AddressBookRow {
        id: "ab1".into(),
        account_id: account.clone(),
        name: "Contacts".into(),
        is_default: true,
        carddav_url: None,
        sync_token: None,
        ctag: None,
    })
    .await
    .unwrap();
    s.upsert_contact(&ContactRow {
        id: "c1".into(),
        address_book_id: "ab1".into(),
        uid: "c1".into(),
        etag: None,
        vcard_raw: "BEGIN:VCARD\r\nFN:Ada Lovelace\r\nEMAIL:ada@x.test\r\nEND:VCARD\r\n".into(),
        json: None,
        full_name: "Ada Lovelace".into(),
        is_favorite: false,
        photo_blob_id: None,
        pgp_key: None,
        smime_cert: None,
    })
    .await
    .unwrap();
    // Case-insensitive prefix + email-substring scan must match on both dialects.
    out.push(format!(
        "autocomplete_name={}",
        s.autocomplete_contacts(&account, "ada", 10)
            .await
            .unwrap()
            .len()
    ));
    out.push(format!(
        "autocomplete_email={}",
        s.autocomplete_contacts(&account, "ADA@", 10)
            .await
            .unwrap()
            .len()
    ));

    s.record_pim_change(&account, "Note", "n1", "created")
        .await
        .unwrap();
    out.push(format!(
        "pim_state={}",
        s.current_pim_state(&account, "Note").await.unwrap()
    ));

    // ---- V4: crypto change log + store key material ----
    let k1 = s
        .record_crypto_change(&account, "CryptoKey", "k1", "created")
        .await
        .unwrap();
    out.push(format!("crypto_change={}", k1));
    s.upsert_store_key_material(&StoreKeyMaterialRow {
        id: "skm1".into(),
        wrapped_seal_key: vec![1, 2, 3, 4],
        suite: "x25519-ml-kem-768-v1".into(),
        created_at: "2026-07-11T00:00:00Z".into(),
    })
    .await
    .unwrap();
    out.push(format!(
        "store_key={:?}",
        s.get_store_key_material()
            .await
            .unwrap()
            .map(|r| r.wrapped_seal_key)
    ));

    // ---- V5: push subscription + sealed VAPID ----
    s.store_vapid_keypair("PUBLIC", b"vapid-private", "2026-07-11T00:00:00Z")
        .await
        .unwrap();
    out.push(format!(
        "vapid_roundtrip={:?}",
        s.load_vapid_keypair().await.unwrap()
    ));

    // ---- V8: SSO config (sealed secret) + content-free login audit ----
    s.put_sso_config(&SsoConfigRow {
        id: "corp-oidc".into(),
        kind: "oidc".into(),
        display_name: "Acme SSO".into(),
        scope: "deployment".into(),
        enabled: true,
        config_json: r#"{"kind":"oidc","issuer_url":"https://idp.example"}"#.into(),
        secret: Some(b"client-secret".to_vec()),
        claim_map_json: r#"{"email":"email"}"#.into(),
        created_at: "2026-07-14T00:00:00Z".into(),
        updated_at: "2026-07-14T00:00:00Z".into(),
    })
    .await
    .unwrap();
    out.push(format!(
        "sso_secret_unseal_ok={}",
        s.get_sso_config("corp-oidc")
            .await
            .unwrap()
            .unwrap()
            .secret
            .as_deref()
            == Some(&b"client-secret"[..])
    ));
    out.push(format!(
        "sso_scoped={}",
        s.list_sso_config("deployment").await.unwrap().len()
    ));
    s.append_sso_login_audit("corp-oidc", "oidc", "hash-of-subject", "ok")
        .await
        .unwrap();
    out.push("sso_audit_appended".into());

    // ---- V6/2FA surfaces that `migrate-store` now carries (26.20) ----
    // `wrapped_root_key` is opaque to the store, so the test seals a known
    // plaintext with the SAME ServerKey the store is opened under. That makes the
    // migration assertions able to check the key still *opens* after a copy, not
    // merely that some bytes arrived.
    s.upsert_zeroaccess(&ZeroAccessRow {
        account_id: account.clone(),
        enabled: true,
        wrapped_root_key: key().seal(ZA_ROOT_KEY_PLAINTEXT).unwrap(),
        kdf_params_json: r#"{"kdf":"argon2id","m":19456,"t":2,"p":1}"#.into(),
        recovery_wrapped: Some(key().seal(ZA_RECOVERY_PLAINTEXT).unwrap()),
        paired_devices_json: r#"[{"id":"dev-1"}]"#.into(),
    })
    .await
    .unwrap();
    let za = s.get_zeroaccess(&account).await.unwrap().unwrap();
    out.push(format!(
        "zeroaccess_opens={}",
        key().open(&za.wrapped_root_key).unwrap() == ZA_ROOT_KEY_PLAINTEXT
    ));

    s.set_quota(
        &account,
        mw_store::QuotaRow {
            bytes_limit: 12_345_678,
            msg_limit: 4_321,
        },
    )
    .await
    .unwrap();
    out.push(format!(
        "quota={:?}",
        s.get_quota(&account)
            .await
            .unwrap()
            .map(|q| (q.bytes_limit, q.msg_limit))
    ));

    s.set_twofa_policy(&TwofaPolicyRow {
        scope_kind: "global".into(),
        scope_value: String::new(),
        require_2fa: true,
        updated_by: "admin-1".into(),
        // Overwritten with `now` by the setter; the copy still carries whatever
        // value ends up stored.
        updated_at: String::new(),
    })
    .await
    .unwrap();
    out.push(format!(
        "twofa_required={:?}",
        s.get_twofa_policy("global", "")
            .await
            .unwrap()
            .map(|p| p.require_2fa)
    ));

    s.append_audit(&AuditRow {
        id: "audit-1".into(),
        ts: "2026-07-20T00:00:00Z".into(),
        actor: "admin-1".into(),
        actor_kind: "admin".into(),
        action: "quota.set".into(),
        target: Some(account.clone()),
        detail_json: r#"{"bytes_limit":12345678}"#.into(),
        ip: None,
    })
    .await
    .unwrap();
    out.push(format!(
        "audit_rows={}",
        s.list_audit(10).await.unwrap().len()
    ));

    // 2FA enrolments, which must travel with the `twofa_policy` above.
    s.put_totp_secret(&account, TOTP_SECRET, true)
        .await
        .unwrap();
    // A step already consumed on the source: it must stay consumed on the
    // destination, or a spent code could be replayed there.
    assert!(
        s.advance_totp_last_step(&account, 57_000_000)
            .await
            .unwrap()
    );
    out.push(format!(
        "totp_unseals={}",
        s.get_totp_secret(&account).await.unwrap().map(|t| t.secret) == Some(TOTP_SECRET.to_vec())
    ));

    s.add_webauthn_credential(&WebauthnCredentialRow {
        credential_id: "cred-1".into(),
        account_id: account.clone(),
        cose_public_key: b"cose-public-key-bytes".to_vec(),
        sign_count: 41,
        transports: "usb,nfc".into(),
        label: "YubiKey".into(),
        created_at: String::new(), // set to now by the setter
    })
    .await
    .unwrap();
    out.push(format!(
        "webauthn_count={}",
        s.list_webauthn_credentials(&account).await.unwrap().len()
    ));

    // One spent code and one live one, so the copy is checked on both states.
    s.add_recovery_codes(
        &account,
        &[RECOVERY_LIVE.to_string(), RECOVERY_SPENT.to_string()],
    )
    .await
    .unwrap();
    assert!(
        s.consume_recovery_code(&account, RECOVERY_SPENT)
            .await
            .unwrap()
    );
    out.push(format!(
        "recovery_unused={}",
        s.list_unused_recovery_codes(&account).await.unwrap().len()
    ));

    // ---- The remaining surfaces `migrate-store` carries as of 26.20 ----
    // Sealed values are written through the store's own setters, so the migration
    // assertions can require the destination to recover the PLAINTEXT rather than
    // compare opaque bytes.
    s.put_passwd_config(&PasswdConfigRow {
        account_id: account.clone(),
        config_json: r#"{"min_len":12}"#.into(),
        force_change: true,
        updated_at: "2026-07-21T00:00:00Z".into(),
    })
    .await
    .unwrap();
    out.push(format!(
        "passwd_force_change={:?}",
        s.get_passwd_config(&account)
            .await
            .unwrap()
            .map(|c| c.force_change)
    ));

    s.upsert_signature(&SignatureRow {
        account_id: account.clone(),
        name: "work".into(),
        body: "-- \nSent from Mailwoman".into(),
        is_default: true,
        rule_json: r#"{"apply":"replies"}"#.into(),
        updated_at: String::new(), // setter stamps it
    })
    .await
    .unwrap();
    out.push(format!(
        "signatures={}",
        s.list_signatures(&account).await.unwrap().len()
    ));

    s.put_notification_rules(&NotificationRulesRow {
        account_id: account.clone(),
        rule_json: r#"{"vip":["boss@example"]}"#.into(),
        quiet_hours_json: r#"{"from":"22:00","to":"07:00"}"#.into(),
        enabled: true,
        updated_at: String::new(),
    })
    .await
    .unwrap();
    out.push(format!(
        "notif_enabled={:?}",
        s.get_notification_rules(&account)
            .await
            .unwrap()
            .map(|n| n.enabled)
    ));

    // One live grant and one revoked, so the copy is checked on both states.
    s.grant_remote_image(&account, "per-sender", "news@example")
        .await
        .unwrap();
    s.grant_remote_image(&account, "per-domain", "ads.example")
        .await
        .unwrap();
    s.revoke_remote_image(&account, "per-domain", "ads.example")
        .await
        .unwrap();
    out.push(format!(
        "image_grant_live={}",
        s.is_remote_image_granted(&account, "per-sender", "news@example")
            .await
            .unwrap()
    ));

    s.put_masked_email(&MaskedEmailRow {
        id: "mask-1".into(),
        account_id: account.clone(),
        alias_addr: "alias-1@masked.example".into(),
        target_desc: "shopping".into(),
        state: "enabled".into(),
        created_at: "2026-07-21T00:00:00Z".into(),
        last_used_at: None,
    })
    .await
    .unwrap();
    out.push(format!(
        "masked_alias={:?}",
        s.get_masked_email("mask-1")
            .await
            .unwrap()
            .map(|m| m.alias_addr)
    ));

    s.put_ews_account_cred(&EwsAccountCred {
        account_id: account.clone(),
        endpoint: "https://ews.example/EWS/Exchange.asmx".into(),
        endpoint_host: "ews.example".into(),
        user: "ews-user".into(),
        domain: "CORP".into(),
        password: EWS_PASSWORD.into(),
        workstation: "MW".into(),
        enabled: true,
    })
    .await
    .unwrap();
    out.push(format!(
        "ews_cred_opens={}",
        s.get_ews_account_cred(&account)
            .await
            .unwrap()
            .map(|c| c.password)
            .as_deref()
            == Some(EWS_PASSWORD)
    ));

    s.put_bridge_account(&BridgeAccountRow {
        account_id: account.clone(),
        bridge_id: "bridge-plugin-1".into(),
        oauth_ref: Some("oauth-ref-1".into()),
        extra_json: r#"{"folder":"All Mail"}"#.into(),
    })
    .await
    .unwrap();
    out.push(format!(
        "bridge_accounts={}",
        s.list_bridge_accounts().await.unwrap().len()
    ));

    s.put_bridge_oauth_token(&BridgeOauthTokenRow {
        bridge_account_id: account.clone(),
        access_token: BRIDGE_ACCESS_TOKEN.into(),
        refresh_token: Some(BRIDGE_REFRESH_TOKEN.into()),
        expires_at: "2026-12-31T00:00:00Z".into(),
        scope: "mail.read".into(),
        updated_at: "2026-07-21T00:00:00Z".into(),
    })
    .await
    .unwrap();
    out.push(format!(
        "bridge_token_opens={}",
        s.get_bridge_oauth_token(&account)
            .await
            .unwrap()
            .map(|t| t.access_token)
            .as_deref()
            == Some(BRIDGE_ACCESS_TOKEN)
    ));

    s.put_message_embedding(&sid, &account, "text-embedding-3-small", EMBEDDING_VECTOR)
        .await
        .unwrap();
    out.push(format!(
        "embedding_opens={:?}",
        s.get_message_embedding(&sid)
            .await
            .unwrap()
            .map(|e| e.vector)
    ));

    s.plugin_kv_set(
        "plugin-1",
        &account,
        "state",
        PLUGIN_KV_VALUE,
        &PluginKvLimits::default(),
    )
    .await
    .unwrap();
    out.push(format!(
        "plugin_kv_opens={}",
        s.plugin_kv_get("plugin-1", &account, "state")
            .await
            .unwrap()
            .as_deref()
            == Some(PLUGIN_KV_VALUE)
    ));

    s.put_assist_config(&AssistConfigRow {
        scope: "deployment".into(),
        adapters_json: r#"[{"kind":"openai-compatible"}]"#.into(),
        capability_grants_json: r#"["summarize"]"#.into(),
        data_ceilings_json: r#"{"folders":["INBOX"]}"#.into(),
        enabled: true,
    })
    .await
    .unwrap();
    out.push(format!(
        "assist_enabled={:?}",
        s.get_assist_config("deployment")
            .await
            .unwrap()
            .map(|a| a.enabled)
    ));

    s.upsert_cache_scope(&CacheScopeRow {
        class: "mailbox-list".into(),
        layers_json: r#"["memory","redis"]"#.into(),
        ttl_secs: 300,
    })
    .await
    .unwrap();
    out.push(format!(
        "cache_scope={}",
        s.list_cache_scope().await.unwrap().len()
    ));

    s.put_password_change_audit(&account, "local", "ok")
        .await
        .unwrap();
    s.put_assist_audit("u@e", "summarize", "INBOX", "api.example")
        .await
        .unwrap();
    out.push("audits_appended".into());

    out
}

/// Fixtures for the 2FA enrolments, so a migration test can prove they are still
/// usable on the destination rather than merely present.
const TOTP_SECRET: &[u8] = b"totp-shared-secret-bytes";
const RECOVERY_LIVE: &str = "argon2-hash-of-unused-code";
const RECOVERY_SPENT: &str = "argon2-hash-of-already-used-code";

/// Fixtures for the sealed surfaces copied in the final 26.20 round. Each is a
/// plaintext the destination must be able to recover, not just a blob to compare.
const EWS_PASSWORD: &str = "ews-upstream-password";
const BRIDGE_ACCESS_TOKEN: &str = "bridge-access-token-value";
const BRIDGE_REFRESH_TOKEN: &str = "bridge-refresh-token-value";
const PLUGIN_KV_VALUE: &[u8] = b"plugin-sealed-state-value";
const EMBEDDING_VECTOR: &[f32] = &[0.5, -0.25, 0.125, 1.0];

/// Plaintexts sealed into `zeroaccess_accounts` by [`run_ops`], so a migration
/// test can prove the copied key material still opens.
const ZA_ROOT_KEY_PLAINTEXT: &[u8] = b"zero-access-root-key-plaintext";
const ZA_RECOVERY_PLAINTEXT: &[u8] = b"zero-access-recovery-plaintext";

#[tokio::test]
async fn backend_parity_sqlite_and_postgres() {
    let sqlite = Store::open_in_memory(key()).await.unwrap();
    let snap_sqlite = run_ops(&sqlite).await;
    // Sanity: the SQLite snapshot is non-trivial.
    assert!(snap_sqlite.len() > 20);

    match pg_dsn() {
        Some(dsn) => {
            let _guard = pg_lock().lock().await;
            let pg = Store::open_postgres(&dsn, key()).await.unwrap();
            truncate_pg(&dsn).await;
            let snap_pg = run_ops(&pg).await;
            assert_eq!(
                snap_sqlite, snap_pg,
                "SQLite vs Postgres backend-parity snapshot mismatch"
            );
            eprintln!("[mw-store] backend-parity: Postgres path RAN and matched SQLite.");
        }
        None => {
            gate::skip(
                "[mw-store] backend-parity: DATABASE_URL_PG and MW_TEST_PG unset — the \
                 Postgres path is not driven and the parity claim rests on SQLite alone.",
            );
        }
    }
}

#[tokio::test]
async fn migrate_store_sqlite_to_postgres() {
    let Some(dsn) = pg_dsn() else {
        gate::skip(
            "[mw-store] migrate-store: DATABASE_URL_PG and MW_TEST_PG unset — the SQLite → \
             Postgres migration is not driven.",
        );
        return;
    };

    // Populate a temp SQLite file store via the public API.
    let (path_str, account, stable_id) = seed_migration_source("mw-store-migrate").await;

    // Migrate into a freshly-truncated Postgres backend sharing the same key.
    let _guard = pg_lock().lock().await;
    let pg = Store::open_postgres(&dsn, key()).await.unwrap();
    truncate_pg(&dsn).await;
    let report = pg.migrate_from_sqlite(&path_str).await.unwrap();
    assert!(report.total_rows() > 0, "migrate copied nothing");

    // Content parity: sealed columns open under the shared key, and rows match.
    let note = pg.get_note("n1").await.unwrap().unwrap();
    assert_eq!(note.body_text, "milk SUPERSECRET eggs");
    let skm = pg.get_store_key_material().await.unwrap().unwrap();
    assert_eq!(skm.wrapped_seal_key, vec![1, 2, 3, 4]);
    let (vp, vk) = pg.load_vapid_keypair().await.unwrap().unwrap();
    assert_eq!(
        (vp.as_str(), vk.as_slice()),
        ("PUBLIC", &b"vapid-private"[..])
    );

    // Every surface promoted out of `NOT_MIGRATED_UNCLASSIFIED` during 26.20,
    // asserted on the Postgres destination as well as the SQLite one — the blob
    // columns differ by dialect (BLOB vs BYTEA), so a wrapped key that opens on
    // SQLite is not evidence that it opens here.
    assert_carried_surfaces(&pg, &account, &stable_id, &DestRef::Postgres(dsn.clone())).await;

    // Row-count parity for a couple of representative tables (via SQLite source).
    let src_pool = sqlx::sqlite::SqlitePoolOptions::new()
        .max_connections(1)
        .connect(&format!("sqlite://{path_str}?mode=ro"))
        .await
        .unwrap();
    for (table, _n) in &report.tables {
        let src_count: i64 = sqlx::query_scalar(&format!("SELECT COUNT(*) FROM \"{table}\""))
            .fetch_one(&src_pool)
            .await
            .unwrap();
        let copied = report
            .tables
            .iter()
            .find(|(t, _)| t == table)
            .map(|(_, n)| *n as i64)
            .unwrap();
        assert_eq!(src_count, copied, "row-count mismatch for {table}");
    }

    drop(pg);
    remove_sqlite_files(&path_str);
    eprintln!("[mw-store] migrate-store: RAN against Postgres and verified content + counts.");
}

// ─────────────────────────────────────────────────────────────────────────────
// `migrate-store` schema-coverage gate.
//
// `migrate_from_sqlite` copies the tables named in `migrate.rs`'s `TABLES`, and
// builds its `MigrationReport` from that same list. Asserting row-count parity
// over the report therefore cannot detect a table the migrator never mentions —
// the oracle is the thing under test. The gate below asserts instead against the
// LIVE schema (`sqlite_master`), so every table the migrations create must be
// accounted for: copied, or named in exactly one of the two lists here.
//
// The two lists are deliberately separate and are NOT equivalent. The first is a
// decision; the second is an open question that has not been decided yet. Both
// are tolerated by the gate today (t24 landed the detection, not the data fix),
// so the backlog is visible in source rather than silently passing.
// ─────────────────────────────────────────────────────────────────────────────

/// DELIBERATELY NOT MIGRATED — live session state that cannot outlive the old
/// deployment, or an admin/deployment surface the operator re-configures on the
/// new host. `(table, reason)`.
const NOT_MIGRATED_DELIBERATELY: &[(&str, &str)] = &[
    (
        "admin_users",
        "0007: the admin panel is a separate identity domain from mail accounts; the \
         operator bootstraps the admin login on the new deployment.",
    ),
    (
        "admin_sessions",
        "0007: live admin bearer-token hashes. Sessions are re-established after a move.",
    ),
    (
        "oauth_clients",
        "0007: admin-approved OAuth clients whose redirect_uris name the OLD host; \
         re-approved against the new one.",
    ),
    (
        "oauth_tokens",
        "0007: live auth-code/access/refresh token hashes. Clients re-authorize.",
    ),
    (
        "oauth_client_meta",
        "0010: RFC 7591 side table keyed by oauth_clients.client_id — meaningless without \
         its parent, which is itself not copied.",
    ),
    (
        "oauth_dcr",
        "0010: the singleton dynamic-client-registration POLICY row, default-disabled; a \
         deployment setting, not account data.",
    ),
    (
        "api_keys",
        "0007: per-deployment API credentials (Argon2id hash + ip_allowlist + rate_limit). \
         Re-minted against the new host.",
    ),
    (
        "webhooks",
        "0007: outbound webhook endpoints and their sealed HMAC secrets, registered against \
         the old deployment's delivery surface.",
    ),
    (
        "domains",
        "0007 (SPEC §19): managed-domain upstream routing + allow/blocklists — operator \
         configuration of the deployment, re-entered in the admin UI.",
    ),
    (
        "directory_config",
        "0008 (SPEC §13): LDAP/GAL endpoint URLs, bind DNs and attribute maps — deployment \
         configuration.",
    ),
    (
        "egress_proxy",
        "0026: the outbound proxy host/port/sealed credentials of the OLD deployment's \
         network position.",
    ),
    (
        "sso_config",
        "0009: IdP endpoints, sealed client secret and claim map, registered against the \
         old deployment's redirect/ACS URLs.",
    ),
    (
        "plugins",
        "0008: the installed WASM plugin registry (admin-approved, enabled deny-by-default). \
         Plugin bundles live outside the store; the operator re-approves them.",
    ),
    (
        "plugin_allowlist",
        "0014: admin-pinned plugin SHA-256 digests. Omission fails CLOSED — an un-pinned \
         plugin will not load.",
    ),
    (
        "ui_plugins",
        "0010: the UI plugin registry (manifest + signature, enabled deny-by-default). Same \
         re-approval story as `plugins`.",
    ),
    (
        "plugin_grants",
        "0008: capability grants for plugins in `plugins`, which is itself not copied.          Copying them would silently re-arm capabilities for a plugin the admin has          NOT re-approved on the new deployment — the grant would already be there          when the plugin was re-installed under the same id. Fails CLOSED as it is:          no capability until an admin grants it again. Excluded for exactly the          reason `ui_plugin_grants` below already was.",
    ),
    (
        "ui_plugin_grants",
        "0010: admin-granted UI plugin capabilities. Omission fails CLOSED — no capability \
         is granted until an admin grants it again.",
    ),
];

/// Formerly the UNCLASSIFIED list: tables whose omission had not been ruled a
/// deliberate design choice. **It is empty, and that is the point.** Every one of
/// the 76 schema tables is now either copied or listed in
/// `NOT_MIGRATED_DELIBERATELY` with a reason — there is no third category where a
/// table can sit un-argued.
///
/// The list is kept rather than deleted because the gate still reads it, so a new
/// migration can be parked here with an open question instead of being forced into
/// a decision before anyone has made one. An entry here is a debt, not a verdict:
/// it started at 25 in 26.20 and was paid down to zero, 24 tables into `TABLES`
/// and `plugin_grants` out to the deliberate list.
const NOT_MIGRATED_UNCLASSIFIED: &[(&str, &str)] = &[];

/// The completeness gate. Enumerates the LIVE schema and requires every table to
/// be accounted for by exactly one of: copied by the migrator, deliberately
/// skipped, or explicitly unclassified. A table added by a new migration lands in
/// none of the three and fails here until someone classifies it.
///
/// Runs unconditionally: WHICH tables the migrator names is backend-independent,
/// so this needs no Postgres DSN — the destination is a second SQLite store.
/// Row counts are deliberately ignored; an empty table copies trivially and would
/// otherwise mask a missing one.
#[tokio::test]
async fn migrate_store_accounts_for_every_schema_table() {
    use std::collections::BTreeSet;

    // A source store with the full migration chain applied and some rows in it.
    let path = test_db::unique_file_path("mw-store-schema-gate", "src.sqlite");
    let path_str = path.to_string_lossy().to_string();
    let src = Store::open(&path_str, key()).await.unwrap();
    let _ = run_ops(&src).await;
    drop(src);

    let dest = Store::open_in_memory(key()).await.unwrap();
    let report = dest.migrate_from_sqlite(&path_str).await.unwrap();

    let src_pool = sqlx::sqlite::SqlitePoolOptions::new()
        .max_connections(1)
        .connect(&format!("sqlite://{path_str}?mode=ro"))
        .await
        .unwrap();
    let schema: BTreeSet<String> = sqlx::query_scalar::<_, String>(
        "SELECT name FROM sqlite_master WHERE type = 'table' \
           AND name NOT LIKE 'sqlite\\_%' ESCAPE '\\' \
           AND name <> '_sqlx_migrations'",
    )
    .fetch_all(&src_pool)
    .await
    .unwrap()
    .into_iter()
    .collect();
    drop(src_pool);
    let _ = std::fs::remove_file(&path);
    let _ = std::fs::remove_file(format!("{path_str}-wal"));
    let _ = std::fs::remove_file(format!("{path_str}-shm"));

    assert!(
        schema.len() > 50,
        "schema enumeration returned {} tables — the query, not the migrator, is broken",
        schema.len()
    );

    let copied: BTreeSet<&str> = report.tables.iter().map(|(t, _)| t.as_str()).collect();
    let deliberate: BTreeSet<&str> = NOT_MIGRATED_DELIBERATELY.iter().map(|(t, _)| *t).collect();
    let unclassified: BTreeSet<&str> = NOT_MIGRATED_UNCLASSIFIED.iter().map(|(t, _)| *t).collect();

    // The lists stay honest: a renamed or dropped table must not linger in them.
    for t in deliberate.iter().chain(unclassified.iter()) {
        assert!(
            schema.contains(*t),
            "`{t}` is listed as not-migrated but no longer exists in the schema — remove the \
             stale entry"
        );
    }
    // Nothing may be both copied and listed as skipped, or in both lists.
    if let Some(t) = deliberate.intersection(&unclassified).next() {
        panic!("`{t}` appears in BOTH not-migrated lists — it is either decided or not");
    }
    for t in &copied {
        assert!(
            !deliberate.contains(t) && !unclassified.contains(t),
            "`{t}` IS copied by the migrator but is also listed as not migrated"
        );
    }

    // `ALL_TABLES` is a third hand-maintained list, and a copied table missing from
    // it leaves rows behind between Postgres runs — which either breaks the next
    // migration on a duplicate key or, worse, lets a stale row satisfy an
    // assertion the copy should have satisfied. Hold it to the same standard.
    let truncated_list: Vec<&str> = ALL_TABLES.split(',').map(str::trim).collect();
    let truncated: BTreeSet<&str> = truncated_list.iter().copied().collect();
    // Postgres rejects a TRUNCATE naming the same table twice, and the error names a
    // line number rather than the mistake. Catch it here instead.
    assert_eq!(
        truncated_list.len(),
        truncated.len(),
        "ALL_TABLES lists a table more than once; TRUNCATE rejects duplicates"
    );
    let untruncated: Vec<&str> = copied
        .iter()
        .copied()
        .filter(|t| !truncated.contains(t))
        .collect();
    assert!(
        untruncated.is_empty(),
        "these copied table(s) are missing from ALL_TABLES, so Postgres runs do not \
         clear them between tests: {untruncated:?}"
    );

    // The gate itself.
    let unaccounted: Vec<&str> = schema
        .iter()
        .map(String::as_str)
        .filter(|t| !copied.contains(t) && !deliberate.contains(t) && !unclassified.contains(t))
        .collect();
    assert!(
        unaccounted.is_empty(),
        "`migrate-store` does not account for {} schema table(s): {:?}\nEach must be either \
         added to `TABLES` in mw-store/src/migrate.rs (so it is copied), or listed in \
         `NOT_MIGRATED_DELIBERATELY` with the reason it is safe to leave behind, or listed in \
         `NOT_MIGRATED_UNCLASSIFIED` as an open question.",
        unaccounted.len(),
        unaccounted
    );

    eprintln!(
        "[mw-store] migrate-store schema coverage: {} schema tables = {} copied + {} \
         deliberately skipped + {} unclassified (an unclassified table is an open \
         question, not a blessing; zero means every table has been argued either way).",
        schema.len(),
        copied.len(),
        deliberate.len(),
        unclassified.len()
    );
}

/// The migrator's own source, so the column check can read the REAL copy specs
/// rather than a second copy of them maintained here. `TABLES` is private and
/// `mod migrate` is not re-exported, so the specs are not reachable as values;
/// reading the text is the cheapest honest oracle available from an integration
/// test. Every `select:` in that file is a single string literal on one line
/// (rustfmt does not split string literals), which is what `copy_specs` relies on.
const MIGRATE_RS: &str = include_str!("../src/migrate.rs");

/// `(table, columns)` for every `SELECT ... FROM <table>` copy spec in
/// `migrate.rs`, parsed from its source.
fn copy_specs() -> Vec<(String, Vec<String>)> {
    let mut out = Vec::new();
    for line in MIGRATE_RS.lines() {
        let line = line.trim();
        let Some(rest) = line.strip_prefix("select: \"SELECT ") else {
            continue;
        };
        let stmt = rest.trim_end_matches("\",");
        let (cols, table) = stmt
            .split_once(" FROM ")
            .unwrap_or_else(|| panic!("copy spec has no FROM clause: {stmt}"));
        out.push((
            table.trim().to_string(),
            cols.split(',')
                // A reserved word is quoted in the SQL, and that quoting reaches
                // us as `\"` because we are reading Rust source, not SQL.
                .map(|c| c.trim().trim_matches(['"', '\\']).to_string())
                .collect(),
        ));
    }
    assert!(
        out.len() > 30,
        "parsed only {} copy specs out of migrate.rs — the parser has drifted from \
         the file's formatting and is no longer checking anything",
        out.len()
    );
    out
}

/// Columns that exist on a COPIED table but are deliberately left behind.
/// `(table, column, reason)`. Empty today: every column of every copied table is
/// carried. An entry here is a decision that must be argued for, not a way to
/// quiet the gate.
const COLUMNS_NOT_COPIED: &[(&str, &str, &str)] = &[];

/// The column-level companion to the table-level gate.
///
/// The table gate cannot see this class of bug: a table whose copy spec has
/// drifted from a later `ALTER TABLE ... ADD COLUMN` is still present in
/// `report.tables`, so it is counted, compared and passed. That is how 0029's
/// `submissions.attempts` / `last_error` / `next_attempt_at` went uncarried.
///
/// For every table the migrator copies, every column the live schema declares
/// must appear in that table's `SELECT`, unless it is listed in
/// `COLUMNS_NOT_COPIED` with a reason.
///
/// Limitation, stated rather than papered over: this reads the `SELECT` side of
/// each spec. A column that is selected but dropped from the `INSERT` or the
/// `map` would still pass here — though the two are positional and a mismatch in
/// arity fails at runtime. It closes the realistic drift (a migration adds a
/// column and the spec is not updated), not every conceivable one.
#[tokio::test]
async fn migrate_store_copies_every_column_of_every_copied_table() {
    use std::collections::BTreeSet;

    let path = test_db::unique_file_path("mw-store-column-gate", "src.sqlite");
    let path_str = path.to_string_lossy().to_string();
    let src = Store::open(&path_str, key()).await.unwrap();
    let _ = run_ops(&src).await;
    drop(src);

    let src_pool = sqlx::sqlite::SqlitePoolOptions::new()
        .max_connections(1)
        .connect(&format!("sqlite://{path_str}?mode=ro"))
        .await
        .unwrap();

    let specs = copy_specs();
    let mut missing: Vec<String> = Vec::new();
    let mut checked_columns = 0usize;

    for (table, carried) in &specs {
        let schema_cols: Vec<String> = sqlx::query_scalar::<_, String>(&format!(
            "SELECT name FROM pragma_table_info('{table}')"
        ))
        .fetch_all(&src_pool)
        .await
        .unwrap();
        assert!(
            !schema_cols.is_empty(),
            "copy spec names table `{table}`, which the live schema does not have"
        );

        let carried: BTreeSet<&str> = carried.iter().map(String::as_str).collect();
        // A spec must not select a column the schema dropped.
        for c in &carried {
            assert!(
                schema_cols.iter().any(|s| s == c),
                "copy spec for `{table}` selects `{c}`, which the live schema does not have"
            );
        }
        for c in &schema_cols {
            checked_columns += 1;
            let excused = COLUMNS_NOT_COPIED
                .iter()
                .any(|(t, col, _)| t == table && col == c);
            if !carried.contains(c.as_str()) && !excused {
                missing.push(format!("{table}.{c}"));
            }
        }
    }

    drop(src_pool);
    let _ = std::fs::remove_file(&path);
    let _ = std::fs::remove_file(format!("{path_str}-wal"));
    let _ = std::fs::remove_file(format!("{path_str}-shm"));

    assert!(
        missing.is_empty(),
        "`migrate-store` copies these tables but silently drops {} of their column(s): \
         {:?}\nAdd each column to that table's SELECT, INSERT and map in \
         mw-store/src/migrate.rs, or list it in `COLUMNS_NOT_COPIED` with the reason \
         it is safe to leave behind.",
        missing.len(),
        missing
    );

    eprintln!(
        "[mw-store] migrate-store column coverage: {} columns across {} copied tables, \
         all carried ({} deliberately excluded).",
        checked_columns,
        specs.len(),
        COLUMNS_NOT_COPIED.len()
    );
}

// ─────────────────────────────────────────────────────────────────────────────
// The five tables promoted out of `NOT_MIGRATED_UNCLASSIFIED` in 26.20.
//
// Before this, `migrate-store` copied none of them. The gap was invisible to the
// pre-existing assertions for the reason recorded above: the row-count parity
// loop iterates `report.tables`, which is built from the same hardcoded list that
// decides what gets copied, so a table that was never copied was never compared.
// The assertions below read the DESTINATION through the store's public API, so
// they fail on the pre-fix copier — the rows simply are not there.
// ─────────────────────────────────────────────────────────────────────────────

/// A raw handle on the migration DESTINATION, for the handful of copied tables
/// `mw-store` exposes no reader for (`uploaded_blobs` metadata without its object,
/// and the `sso_login_audit` / `password_change_audit` / `assist_audit` append-only
/// logs, which are write-only through the public API). Everything else is asserted
/// through the store's own getters, which is always preferable — a getter proves
/// the row is *usable*, a raw count only proves it is there.
enum DestRef {
    Sqlite(String),
    Postgres(String),
}

impl DestRef {
    /// `COUNT(*)` for `table`, optionally filtered by `where_sql` (no bind params —
    /// callers pass literals they control).
    async fn count(&self, table: &str, where_sql: &str) -> i64 {
        let sql = if where_sql.is_empty() {
            format!("SELECT COUNT(*) FROM \"{table}\"")
        } else {
            format!("SELECT COUNT(*) FROM \"{table}\" WHERE {where_sql}")
        };
        match self {
            Self::Sqlite(path) => {
                let pool = sqlx::sqlite::SqlitePoolOptions::new()
                    .max_connections(1)
                    .connect(&format!("sqlite://{path}?mode=ro"))
                    .await
                    .unwrap();
                sqlx::query_scalar(&sql).fetch_one(&pool).await.unwrap()
            }
            Self::Postgres(dsn) => {
                let pool = sqlx::postgres::PgPoolOptions::new()
                    .max_connections(1)
                    .connect(dsn)
                    .await
                    .unwrap();
                sqlx::query_scalar(&sql).fetch_one(&pool).await.unwrap()
            }
        }
    }
}

/// Write the one `uploaded_blobs` row the migration assertions look for, straight
/// into the SQLite source at `path`.
///
/// Not written through `Store::put_upload` deliberately: that also seals and writes
/// the object to an upload backend, and this suite injects none (a store without one
/// fails closed by design). Wiring a real `FsUploadBackend` through every store
/// construction here would be a larger change than the row it is testing, and the
/// row is what `migrate-store` actually carries.
async fn seed_uploaded_blob(path: &str, account: &str) {
    let pool = sqlx::sqlite::SqlitePoolOptions::new()
        .max_connections(1)
        .connect(&format!("sqlite://{path}"))
        .await
        .unwrap();
    sqlx::query(
        "INSERT INTO uploaded_blobs \
         (blob_id, account_id, content_type, size, storage_key, backend_kind, created_at) \
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
    )
    .bind(UPLOAD_BLOB_ID)
    .bind(account)
    .bind("image/png")
    .bind(2048_i64)
    .bind(UPLOAD_STORAGE_KEY)
    .bind("fs")
    .bind("2026-07-21T00:00:00Z")
    .execute(&pool)
    .await
    .unwrap();
}

const UPLOAD_BLOB_ID: &str = "U0123456789abcdef";
const UPLOAD_STORAGE_KEY: &str = "0123456789abcdef";

/// The one message stable id in a seeded source store, read from the SOURCE so the
/// migration assertions compare against an expectation rather than against whatever
/// the destination happens to hold.
async fn source_stable_id(path: &str) -> String {
    let pool = sqlx::sqlite::SqlitePoolOptions::new()
        .max_connections(1)
        .connect(&format!("sqlite://{path}?mode=ro"))
        .await
        .unwrap();
    sqlx::query_scalar("SELECT stable_id FROM messages LIMIT 1")
        .fetch_one(&pool)
        .await
        .unwrap()
}

/// Assert that everything [`run_ops`] wrote into the five newly-copied tables
/// survived a `migrate-store` into `dest`, which must be open under [`key`].
///
/// For `zeroaccess_accounts` this goes past row presence: the wrapped root key is
/// the only copy of the account's client-derived key, so the test seals a known
/// plaintext into it on the source and requires the destination's bytes to still
/// *open* to that plaintext. A row that arrived corrupt, truncated, or
/// re-encrypted under a different key would pass a presence check and fail here.
async fn assert_carried_surfaces(dest: &Store, account: &str, stable_id: &str, raw: &DestRef) {
    // Collected rather than asserted one at a time, so a copier that drops all
    // five names all five in a single run instead of one per fix cycle.
    let mut lost: Vec<String> = Vec::new();

    // ── zeroaccess_accounts: present, and the key still OPENS ────────────────
    match dest.get_zeroaccess(account).await.unwrap() {
        None => lost.push(
            "zeroaccess_accounts: row absent — the account's wrapped root key exists \
             nowhere else, so zero-access mail on the destination is permanently \
             undecryptable"
                .into(),
        ),
        Some(za) => {
            if !za.enabled {
                lost.push("zeroaccess_accounts: arrived disabled".into());
            }
            if key().open(&za.wrapped_root_key).ok().as_deref() != Some(ZA_ROOT_KEY_PLAINTEXT) {
                lost.push(
                    "zeroaccess_accounts: wrapped_root_key no longer opens — present but \
                     unusable"
                        .into(),
                );
            }
            if za
                .recovery_wrapped
                .as_deref()
                .and_then(|b| key().open(b).ok())
                .as_deref()
                != Some(ZA_RECOVERY_PLAINTEXT)
            {
                lost.push("zeroaccess_accounts: recovery_wrapped no longer opens".into());
            }
            if za.paired_devices_json != r#"[{"id":"dev-1"}]"#
                || !za.kdf_params_json.contains("argon2id")
            {
                lost.push(format!(
                    "zeroaccess_accounts: metadata altered (kdf={:?}, devices={:?})",
                    za.kdf_params_json, za.paired_devices_json
                ));
            }
        }
    }

    // The wrapped key is opaque to the store, so "usable" also requires the
    // destination to hold the same seal key material. It is copied, and this is
    // the assertion that it arrived intact.
    if dest
        .get_store_key_material()
        .await
        .unwrap()
        .map(|r| r.wrapped_seal_key)
        != Some(vec![1, 2, 3, 4])
    {
        lost.push(
            "store_key_material: absent or altered — nothing sealed under the store key \
             would open on the destination"
                .into(),
        );
    }

    // ── twofa_policy: fails OPEN if dropped ──────────────────────────────────
    if dest
        .get_twofa_policy("global", "")
        .await
        .unwrap()
        .map(|p| p.require_2fa)
        != Some(true)
    {
        lost.push(
            "twofa_policy: the require-2FA policy is gone — the destination silently \
             stopped requiring a second factor"
                .into(),
        );
    }

    // ── totp_secrets: the sealed secret must still UNSEAL on the destination ─
    // `get_totp_secret` opens the blob with the destination's own key, so this
    // fails if the row is absent, the bytes are damaged, or the key material did
    // not travel. The same secret bytes generate the same codes, so recovering
    // them is what "the authenticator app still works" reduces to here.
    match dest.get_totp_secret(account).await.unwrap() {
        None => lost.push(
            "totp_secrets: enrolment absent — every user's authenticator app is \
             silently un-enrolled, while twofa_policy still demands a second factor"
                .into(),
        ),
        Some(secret) => {
            if secret.secret != TOTP_SECRET {
                lost.push(
                    "totp_secrets: the sealed secret no longer unseals to the enrolled \
                     value — present but unusable"
                        .into(),
                );
            }
            if !secret.confirmed {
                lost.push("totp_secrets: the enrolment arrived unconfirmed".into());
            }
        }
    }
    // 0021's replay guard: a step spent on the source must stay spent.
    let last_step = dest.totp_last_step(account).await.unwrap();
    if last_step != 57_000_000 {
        lost.push(format!(
            "totp_secrets.last_step: {last_step} instead of 57000000 — a TOTP code \
             already used on the source could be replayed on the destination"
        ));
    }

    // ── webauthn_credentials ─────────────────────────────────────────────────
    // NOTE ON WHAT THIS DOES NOT PROVE: completing a WebAuthn assertion needs a
    // signature from the authenticator's private key, which lives in hardware and
    // is unavailable to any test. So this checks that the verification material
    // arrived intact — not that a login succeeds. Copying is still strictly better
    // than not: the rows are useless on the source once it is retired, and an
    // uncopied credential is a guaranteed lockout rather than a possible one. The
    // separate deployment caveat (credentials are bound to the RP ID, so changing
    // the deployment's domain invalidates them whether or not they are copied) is
    // documented in docs/deploy/postgres.md.
    match dest.get_webauthn_credential("cred-1").await.unwrap() {
        None => lost.push(
            "webauthn_credentials: credential absent — the enrolled security key can \
             no longer be presented"
                .into(),
        ),
        Some(cred) => {
            if cred.cose_public_key != b"cose-public-key-bytes" {
                lost.push(
                    "webauthn_credentials: the COSE public key was altered — no \
                     assertion from this authenticator could verify"
                        .into(),
                );
            }
            if cred.sign_count != 41 {
                lost.push(format!(
                    "webauthn_credentials.sign_count: {} instead of 41 — a counter that \
                     travels backwards stops the destination detecting a cloned \
                     authenticator",
                    cred.sign_count
                ));
            }
            if cred.account_id != account || cred.label != "YubiKey" {
                lost.push("webauthn_credentials: the credential's binding was altered".into());
            }
        }
    }

    // ── recovery_codes: usable, and a spent one still spent ──────────────────
    // `consume_recovery_code` returns true only for a code that is present AND
    // unused, so this is the store's own "would this code authenticate?" answer.
    if !dest
        .consume_recovery_code(account, RECOVERY_LIVE)
        .await
        .unwrap()
    {
        lost.push(
            "recovery_codes: the unused recovery code is gone or arrived spent — the \
             fallback for a lost authenticator no longer works"
                .into(),
        );
    }
    if dest
        .consume_recovery_code(account, RECOVERY_SPENT)
        .await
        .unwrap()
    {
        lost.push(
            "recovery_codes: a code already spent on the source was accepted on the \
             destination — the `used` flag did not travel, so every burned code is \
             live again"
                .into(),
        );
    }

    // ── quotas: also fails OPEN ──────────────────────────────────────────────
    if dest
        .get_quota(account)
        .await
        .unwrap()
        .map(|q| (q.bytes_limit, q.msg_limit))
        != Some((12_345_678, 4_321))
    {
        lost.push("quotas: the account quota is gone — limits silently lifted".into());
    }

    // ── audit_log: append-only by invariant, so a migration must not erase it ─
    let audit = dest.list_audit(10).await.unwrap();
    if !audit.iter().any(|a| {
        a.id == "audit-1" && a.action == "quota.set" && a.target.as_deref() == Some(account)
    }) {
        lost.push(format!(
            "audit_log: the append-only audit entry is gone ({} row(s) present)",
            audit.len()
        ));
    }

    // ── crypto_changes: the state counter is derived from the copied rows ────
    let state = dest
        .current_crypto_state(account, "CryptoKey")
        .await
        .unwrap();
    let feed: Vec<(String, String)> = dest
        .crypto_changes_since(account, "CryptoKey", 0)
        .await
        .unwrap()
        .into_iter()
        .map(|c| (c.object_id, c.op))
        .collect();
    if state != 1 || feed != vec![("k1".to_string(), "created".to_string())] {
        lost.push(format!(
            "crypto_changes: the change-feed did not survive (state={state}, rows={feed:?}) — \
             the destination would replay every crypto object as new"
        ));
    }

    // ── Per-account settings and content ─────────────────────────────────────
    if dest
        .get_passwd_config(account)
        .await
        .unwrap()
        .map(|c| c.force_change)
        != Some(true)
    {
        lost.push(
            "passwd_config: gone — the force-change-on-next-login flag cleared, so a \
             user the operator had flagged walks in unchallenged"
                .into(),
        );
    }

    let sigs = dest.list_signatures(account).await.unwrap();
    if !sigs
        .iter()
        .any(|s| s.name == "work" && s.body.contains("Sent from Mailwoman") && s.is_default)
    {
        lost.push(format!(
            "signatures: the user's authored signature is gone ({} row(s) present)",
            sigs.len()
        ));
    }

    if dest
        .get_notification_rules(account)
        .await
        .unwrap()
        .map(|n| (n.enabled, n.quiet_hours_json.contains("22:00")))
        != Some((true, true))
    {
        lost.push("notification_rules: per-account rules and quiet hours are gone".into());
    }

    // Live grant must survive AND the revoked one must stay revoked.
    if !dest
        .is_remote_image_granted(account, "per-sender", "news@example")
        .await
        .unwrap()
    {
        lost.push("remote_image_grants: the user's live grant is gone".into());
    }
    if dest
        .is_remote_image_granted(account, "per-domain", "ads.example")
        .await
        .unwrap()
    {
        lost.push(
            "remote_image_grants: a REVOKED grant came back live — `revoked` did not \
             travel, so remote images the user turned off are permitted again"
                .into(),
        );
    }

    if dest
        .get_masked_email("mask-1")
        .await
        .unwrap()
        .map(|m| (m.alias_addr, m.state))
        != Some(("alias-1@masked.example".into(), "enabled".into()))
    {
        lost.push(
            "masked_email: the alias record is gone — mail still arrives at the alias \
             but the destination cannot attribute, list or disable it"
                .into(),
        );
    }

    // ── Sealed account credentials: must OPEN, not merely arrive ─────────────
    match dest.get_ews_account_cred(account).await.unwrap() {
        None => lost.push(
            "ews_account_cred: the account's EWS binding is gone, so its upstream \
             mailbox stops syncing until someone re-enters the password"
                .into(),
        ),
        Some(cred) => {
            if cred.password != EWS_PASSWORD || cred.domain != "CORP" || !cred.enabled {
                lost.push(
                    "ews_account_cred: the sealed credential no longer opens to the \
                     enrolled value — present but unusable"
                        .into(),
                );
            }
        }
    }

    if dest.list_bridge_accounts().await.unwrap().iter().all(|b| {
        b.account_id != account
            || b.bridge_id != "bridge-plugin-1"
            || b.oauth_ref.as_deref() != Some("oauth-ref-1")
    }) {
        lost.push("bridge_accounts: the account's bridge binding is gone".into());
    }

    match dest.get_bridge_oauth_token(account).await.unwrap() {
        None => lost.push(
            "bridge_oauth_tokens: the sealed OAuth grant is gone — every bridged \
             account is forced back through interactive re-consent"
                .into(),
        ),
        Some(tok) => {
            if tok.access_token != BRIDGE_ACCESS_TOKEN
                || tok.refresh_token.as_deref() != Some(BRIDGE_REFRESH_TOKEN)
            {
                lost.push(
                    "bridge_oauth_tokens: the sealed tokens no longer open to the \
                     granted values"
                        .into(),
                );
            }
        }
    }

    // ── message_embeddings: the vector must unseal AND survive `dim` validation ─
    // `get_message_embedding` checks the stored `dim` against the decoded vector
    // length, so a truncated or mis-copied blob is rejected rather than returned.
    match dest.get_message_embedding(stable_id).await.unwrap() {
        None => lost.push(
            "message_embeddings: the sealed vector is gone — semantic search must \
             re-embed the whole store to recover"
                .into(),
        ),
        Some(emb) => {
            if emb.vector != EMBEDDING_VECTOR || emb.model != "text-embedding-3-small" {
                lost.push("message_embeddings: the vector did not survive the copy intact".into());
            }
        }
    }

    // ── plugin_kv: sealed plugin state must open ─────────────────────────────
    if dest
        .plugin_kv_get("plugin-1", account, "state")
        .await
        .unwrap()
        .as_deref()
        != Some(PLUGIN_KV_VALUE)
    {
        lost.push(
            "plugin_kv: the plugin's sealed state is gone or no longer opens — a \
             plugin cannot regenerate it"
                .into(),
        );
    }

    // ── Deployment configuration that travels with the data ─────────────────
    if dest
        .get_assist_config("deployment")
        .await
        .unwrap()
        .map(|a| (a.enabled, a.capability_grants_json))
        != Some((true, r#"["summarize"]"#.into()))
    {
        lost.push("assist_config: the Assist configuration and its grants are gone".into());
    }
    if !dest
        .list_cache_scope()
        .await
        .unwrap()
        .iter()
        .any(|c| c.class == "mailbox-list" && c.ttl_secs == 300)
    {
        lost.push("cache_scope: the operator's cache tuning is gone".into());
    }

    // ── The write-only tables, checked raw ───────────────────────────────────
    // `mw-store` exposes no reader for these, so a row count is all that is
    // available. For the three audit logs that is also all that is needed: they are
    // append-only and content-free, and a row either arrived or did not.
    if raw
        .count(
            "uploaded_blobs",
            &format!("blob_id = '{UPLOAD_BLOB_ID}' AND storage_key = '{UPLOAD_STORAGE_KEY}'"),
        )
        .await
        != 1
    {
        lost.push(
            "uploaded_blobs: the attachment metadata is gone — the sealed objects on \
             the upload backend become unreachable and the gc sweep cannot see them"
                .into(),
        );
    }
    // NOTE ON WHAT THIS DOES NOT PROVE: the object itself lives on the upload
    // backend, outside this database, and `migrate-store` does not move it. This
    // asserts the metadata survives with the `storage_key` that locates the object —
    // not that a `get_upload` succeeds, which would additionally require the upload
    // directory to have been carried across. That operator step is documented in
    // docs/deploy/postgres.md.

    for (table, what) in [
        ("sso_login_audit", "SSO login history (hashed subjects)"),
        ("password_change_audit", "password-change history"),
        ("assist_audit", "Assist capability-use history"),
    ] {
        if raw.count(table, "").await < 1 {
            lost.push(format!(
                "{table}: the {what} did not survive the copy — the table is \
                 append-only, so a migration is the only thing that can erase it"
            ));
        }
    }

    assert!(
        lost.is_empty(),
        "`migrate-store` lost {} surface(s) that must survive a store move:\n  - {}",
        lost.len(),
        lost.join("\n  - ")
    );
}

/// Seed a SQLite source store and return `(path, account_id, stable_id)`. The
/// caller migrates it into whichever destination it wants to assert on.
async fn seed_migration_source(tag: &str) -> (String, String, String) {
    let path = test_db::unique_file_path(tag, "src.sqlite");
    let path_str = path.to_string_lossy().to_string();
    let src = Store::open(&path_str, key()).await.unwrap();
    let _ = run_ops(&src).await;
    let account = src.list_accounts().await.unwrap()[0].id.clone();
    drop(src);
    seed_uploaded_blob(&path_str, &account).await;
    let stable_id = source_stable_id(&path_str).await;
    (path_str, account, stable_id)
}

fn remove_sqlite_files(path_str: &str) {
    let _ = std::fs::remove_file(path_str);
    let _ = std::fs::remove_file(format!("{path_str}-wal"));
    let _ = std::fs::remove_file(format!("{path_str}-shm"));
}

/// SQLite destination. Runs unconditionally, so the regression is caught on any
/// machine; the Postgres leg is asserted by `migrate_store_sqlite_to_postgres`.
#[tokio::test]
async fn migrate_store_carries_zero_access_and_policy_rows_sqlite() {
    let (src_path, account, stable_id) = seed_migration_source("mw-store-carry-src").await;

    // A FILE-backed destination, not in-memory: a few copied tables have no reader
    // on `Store`, so the assertions need to reach the destination with raw SQL.
    let dest_path = test_db::unique_file_path("mw-store-carry-dest", "dest.sqlite");
    let dest_path_str = dest_path.to_string_lossy().to_string();
    let dest = Store::open(&dest_path_str, key()).await.unwrap();
    dest.migrate_from_sqlite(&src_path).await.unwrap();

    assert_carried_surfaces(
        &dest,
        &account,
        &stable_id,
        &DestRef::Sqlite(dest_path_str.clone()),
    )
    .await;

    drop(dest);
    remove_sqlite_files(&src_path);
    remove_sqlite_files(&dest_path_str);
    eprintln!(
        "[mw-store] migrate-store: zero-access key, 2FA policy AND its enrolments, \
         per-account settings and content, sealed EWS/bridge credentials, upload \
         metadata, embeddings, plugin state, Assist/cache config and all four \
         append-only audit logs carried (SQLite destination)."
    );
}
