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
    NotificationRulesRow, OpenBind, PasswdConfigRow, PluginKvLimits, ServerKey, SignatureRow,
    SsoConfigRow, Store, StoreKeyMaterialRow, SubmissionRow, TwofaPolicyRow, WebauthnCredentialRow,
    ZeroAccessRow, open_token_hash,
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
const ALL_TABLES: &str = "identity_principal, identity_binding, session_principal, sessions, settings, accounts, mailboxes, messages, bodies, threads, \
    pop3_uidl, sync_state, message_meta, tags, saved_searches, submissions, identities, changes, \
    calendars, calendar_shares, events, event_instances, tasks, notebooks, notes, address_books, \
    contacts, contact_groups, pim_changes, crypto_keys, key_associations, security_verdicts, \
    dlp_audit, sender_controls, store_key_material, push_subscriptions, push_config, \
    native_sessions, sso_config, sso_login_audit, quotas, zeroaccess_accounts, \
    crypto_changes, audit_log, twofa_policy, totp_secrets, webauthn_credentials, \
    recovery_codes, passwd_config, signatures, notification_rules, \
    remote_image_grants, masked_email, ews_account_cred, bridge_accounts, \
    bridge_oauth_tokens, uploaded_blobs, message_embeddings, plugin_kv, \
    assist_config, cache_scope, password_change_audit, assist_audit, open_tracking, \
    open_events, setting_policy, setting_value, auth_failure, ip_block, ip_allow, \
    admin_account, admin_totp, admin_role, admin_role_capability, admin_account_role, \
    admin_capability_override, admin_saved_query, audit_chain_head, audit_checkpoint, \
    audit_retention, audit_sink, alert_rule, alert_event, signin_event, signin_identity, remembered_device, \
    branding, brand_asset, custom_css, custom_theme, custom_accent, sieve_script, \
    smime_trust_root, pki_revocation_cache, identity_signature_default, \
    domain_signature_template, domain_disclaimer, mdn_sender_rule, receipt_status, \
    child_account, provider_profile, account_oauth, delegation_grant, shared_mailbox, \
    org_view, identity_verification, template, template_version, user_group, \
    user_group_member, feature_profile, feature_profile_assignment, mail_user, \
    mail_user_account, signin_allow, usage_counter, usage_storage, config_snapshot, \
    maintenance_window, scheduled_task, job_run, diag_run, account_ban, metric_rollup, \
    data_export, inbound_webhooks, webhook_nonce, dav_account, admin_webauthn_credentials, admin_recovery_codes";

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
    // 0031: a held submission keeps its hold, origin and onSuccess columns, and
    // a release clears the hold and the timers.
    s.insert_submission_with(
        &SubmissionRow {
            id: "sub-held".into(),
            account_id: account.clone(),
            email_id: sid.clone(),
            identity_id: None,
            send_at: Some("2999-01-01T00:00:00Z".into()),
            undo_status: "pending".into(),
            hold_seconds: 0,
            created_at: "2026-07-01T10:00:01Z".into(),
        },
        Some("manual"),
        Some(r#"{"kind":"apiKey","name":"abcd"}"#),
        Some(r#"{"update":{"keywords/$draft":null}}"#),
    )
    .await
    .unwrap();
    out.push(format!(
        "held_sub={:?}",
        s.get_submission_hold("sub-held").await.unwrap()
    ));
    out.push(format!(
        "released={}:{:?}:{:?}",
        s.release_submission("sub-held").await.unwrap(),
        s.get_submission_hold("sub-held").await.unwrap(),
        s.get_submission("sub-held")
            .await
            .unwrap()
            .map(|r| r.send_at)
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

    // ---- 0032: open tracking — mint, bind, two loads, an unknown token ----
    let t0 = chrono::DateTime::parse_from_rfc3339(OPEN_T0)
        .unwrap()
        .with_timezone(&chrono::Utc);
    let tok = open_token_hash(OPEN_TOKEN);
    s.create_open_token(&tok, &account, OPEN_SUBJECT, r#"["b@example.org"]"#, t0)
        .await
        .unwrap();
    out.push(format!(
        "open_unbound_summary={:?}",
        s.open_summary(&account, OPEN_EMAIL).await.unwrap()
    ));
    out.push(format!(
        "open_bind={:?}:{:?}",
        s.bind_open_token(&account, &tok, OPEN_EMAIL).await.unwrap(),
        s.bind_open_token("other-account", &tok, OPEN_EMAIL)
            .await
            .unwrap()
    ));
    assert_eq!(
        s.bind_open_token(&account, &tok, "another-email")
            .await
            .unwrap(),
        OpenBind::AlreadyBound
    );
    s.record_open(&tok, t0 + chrono::Duration::minutes(5))
        .await
        .unwrap();
    s.record_open(&tok, t0 + chrono::Duration::minutes(9))
        .await
        .unwrap();
    out.push(format!(
        "open_unknown_recorded={}",
        s.record_open(&open_token_hash("no-such-token"), t0)
            .await
            .unwrap()
    ));
    out.push(format!(
        "open_summary={:?}",
        s.open_summary(&account, OPEN_EMAIL).await.unwrap()
    ));
    out.push(format!(
        "open_events={:?}",
        s.open_event_times(&tok).await.unwrap()
    ));
    out.push(format!(
        "open_subject={:?}",
        s.get_open_token(&account, &tok)
            .await
            .unwrap()
            .map(|r| r.subject)
    ));

    out
}

const OPEN_TOKEN: &str = "parity-open-token";
const OPEN_EMAIL: &str = "parity-sent-email";
const OPEN_SUBJECT: &str = "Quarterly report";
const OPEN_T0: &str = "2026-07-01T10:00:00Z";

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
    ("identity_serialization", "Structural singleton seeded by migration 0043; contains no identity or authorization data."),
    ("hosting_connection", "Panel host authority and sealed provisioning credentials are deployment-local, like directory and tunnel configuration; re-approve on the new deployment."),
    (
        "admin_users",
        "0007: the admin panel is a separate identity domain from mail accounts; the \
         operator bootstraps the admin login on the new deployment.",
    ),
    (
        "admin_sessions",
        "0007, 0030: live admin bearer-token hashes and their expiry deadlines. Sessions \
         are re-established after a move.",
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
        "inbound_webhooks",
        "0034: inbound webhooks with their sealed secrets, bound accounts and allowed \
         actions. Like the outbound `webhooks` they belong to the old deployment's \
         integrations; the operator re-creates them and hands out the new secrets.",
    ),
    (
        "webhook_nonce",
        "0034: the replay cache of accepted inbound-webhook nonces, kept for one \
         five-minute signature window. Nothing in it outlives the move.",
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
    (
        "domain_alias",
        "0033: alias -> managed domain, `REFERENCES domains(name)`. `domains` is not copied, \
         so a copied alias would violate its foreign key at commit; it is re-entered with \
         its domain.",
    ),
    (
        "domain_host",
        "0033: host name -> managed domain, `REFERENCES domains(name)`; same reason as \
         `domain_alias`.",
    ),
    (
        "diag_probe_account",
        "0033: the per-domain diagnostics probe login (sealed password), \
         `REFERENCES domains(name)`; same reason as `domain_alias`, and a probe is a test \
         credential the operator saves again.",
    ),
    (
        "ssh_tunnel",
        "0033: SSH bastions, their sealed private keys and host-key pins describe the OLD \
         deployment's network position, like `egress_proxy`.",
    ),
    (
        "egress_internal_allow",
        "0033: internal destinations the SIEM/webhook traffic may reach from the OLD \
         deployment's network, like `egress_proxy`.",
    ),
    (
        "oauth_client",
        "0033: this server's client registrations at upstream OAuth providers; the redirect \
         names the OLD host, like `sso_config`. Re-entered with the same client id, the \
         copied `account_oauth` tokens keep working.",
    ),
    (
        "integration_config",
        "0033: password-change backend, Nextcloud, inbound webhook and DCR endpoints and \
         secrets, the same family as `directory_config`. Disabled by default; the env \
         configuration stays in force until an admin re-enters them.",
    ),
];

/// Formerly the UNCLASSIFIED list: tables whose omission had not been ruled a
/// deliberate design choice. **It is empty, and that is the point.** Every schema
/// table (147 as of 0035) is now either copied or listed in
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

    // 0032: the sent message's open record, its event times and the sealed subject.
    let tok = open_token_hash(OPEN_TOKEN);
    match dest.open_summary(account, OPEN_EMAIL).await.unwrap() {
        Some(sum) if sum.count == 2 && sum.first.is_some() && sum.last.is_some() => {}
        other => lost.push(format!(
            "open_tracking: the sent message's open record is gone or changed ({other:?})"
        )),
    }
    if dest.open_event_times(&tok).await.unwrap().len() != 2 {
        lost.push("open_events: the recorded open times did not travel".into());
    }
    if dest
        .get_open_token(account, &tok)
        .await
        .unwrap()
        .map(|r| r.subject)
        .as_deref()
        != Some(OPEN_SUBJECT)
    {
        lost.push("open_tracking: the sealed subject did not open on the destination".into());
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

// ─────────────────────────────────────────────────────────────────────────────
// Migration 0033 (26.20 t32-e10a): the admin-controls tables.
//
// The spec below is written from the plan (`t32-schedule.md` §4, §12.3, §12.9,
// §13.3, §13.10), not read back from the migration files, so it is an oracle the
// two `0033_admin_controls.sql` files can disagree with. Each dialect is checked
// against it on its own: column set, types, NOT NULL, defaults, CHECK values,
// unique and partial indexes, foreign keys with ON DELETE CASCADE, a full-row
// round trip, the defaults of a minimal row, and the `migrate-store` copy.
// ─────────────────────────────────────────────────────────────────────────────

mod admin_controls_0033 {
    use super::{key, pg_dsn, remove_sqlite_files, test_db};
    use mw_store::Store;
    use sqlx::pool::PoolConnection;
    use sqlx::{Postgres, Sqlite};

    /// `(column, type, rule, sample)`. Type `T` text, `I` integer, `B` blob. Rule
    /// `!` = NOT NULL without a default, `~` = nullable without a default, `=v` =
    /// NOT NULL DEFAULT v (`=` alone is the empty string). Samples differ from the
    /// defaults, so a round trip cannot pass by reading a default back.
    type Col = (&'static str, char, &'static str, &'static str);

    struct NewTable {
        name: &'static str,
        /// Copied by `migrate-store` (else listed in `NOT_MIGRATED_DELIBERATELY`).
        copied: bool,
        cols: &'static [Col],
    }

    const TS: &str = "1767225600";

    /// Every table 0033 creates, FK parents before their children.
    #[rustfmt::skip]
    const NEW_TABLES: &[NewTable] = &[
        // C/AG settings
        NewTable { name: "setting_policy", copied: true, cols: &[
            ("key", 'T', "!", "signin.newUsers"),
            ("global_value", 'T', "~", "open"),
            ("allow_user_override", 'I', "=1", "0"),
            ("global_locked", 'I', "=0", "1"),
            ("updated_by", 'T', "!", "admin:root"),
            ("updated_at", 'I', "!", TS),
        ]},
        NewTable { name: "setting_value", copied: true, cols: &[
            ("scope_kind", 'T', "!", "domain"),
            ("scope_id", 'T', "!", "example.test"),
            ("key", 'T', "!", "signin.newUsers"),
            ("value", 'T', "!", "known"),
            ("locked", 'I', "=0", "1"),
            ("updated_by", 'T', "!", "admin:root"),
            ("updated_at", 'I', "!", TS),
        ]},
        // D security controls
        NewTable { name: "auth_failure", copied: true, cols: &[
            ("surface", 'T', "!", "webmail"),
            ("key_kind", 'T', "!", "ip"),
            ("key", 'T', "!", "192.0.2.1"),
            ("window_start", 'I', "!", TS),
            ("count", 'I', "=0", "3"),
        ]},
        NewTable { name: "ip_block", copied: true, cols: &[
            ("id", 'T', "!", "ipb-1"),
            ("ip_or_cidr", 'T', "!", "192.0.2.0/24"),
            ("surface", 'T', "!", "all"),
            ("reason", 'T', "!", "manual"),
            ("offences", 'I', "=1", "2"),
            ("blocked_until", 'I', "~", "1767229200"),
            ("created_by", 'T', "!", "admin:root"),
            ("created_at", 'I', "!", TS),
            ("note", 'T', "=", "abuse report"),
        ]},
        NewTable { name: "ip_allow", copied: true, cols: &[
            ("id", 'T', "!", "ipa-1"),
            ("surface", 'T', "!", "admin"),
            ("scope_kind", 'T', "=global", "domain"),
            ("scope_id", 'T', "=", "example.test"),
            ("cidr", 'T', "!", "198.51.100.0/24"),
            ("note", 'T', "=", "office"),
            ("created_by", 'T', "!", "admin:root"),
            ("created_at", 'I', "!", TS),
        ]},
        // E/G domains
        NewTable { name: "domain_alias", copied: false, cols: &[
            ("alias", 'T', "!", "alias.test"),
            ("domain", 'T', "!", "example.test"),
        ]},
        NewTable { name: "domain_host", copied: false, cols: &[
            ("host", 'T', "!", "mail.example.test"),
            ("domain", 'T', "!", "example.test"),
        ]},
        // J tunnels
        NewTable { name: "ssh_tunnel", copied: false, cols: &[
            ("id", 'T', "!", "tun-1"),
            ("name", 'T', "!", "bastion"),
            ("host", 'T', "!", "bastion.example.test"),
            ("port", 'I', "!", "2222"),
            ("username", 'T', "!", "mw"),
            ("sealed_private_key", 'B', "!", "sealed-private-key"),
            ("public_key", 'T', "!", "ssh-ed25519 AAAAC3Nz"),
            ("key_algorithm", 'T', "!", "ed25519"),
            ("host_key_pins_json", 'T', "=[]", "[\"SHA256:abc\"]"),
            ("targets_json", 'T', "=[]", "[]"),
            ("allow_private", 'I', "=0", "0"),
            ("auth_kind", 'T', "~=privateKey", "privateKey"),
            ("revision", 'T', "=legacy", "ssh-revision"),
            ("keepalive_secs", 'I', "=30", "60"),
            ("connect_timeout_ms", 'I', "=5000", "3000"),
            ("max_channels", 'I', "=16", "8"),
            ("enabled", 'I', "=1", "0"),
            ("created_by", 'T', "!", "admin:root"),
            ("created_at", 'I', "!", TS),
            ("updated_at", 'I', "!", TS),
        ]},
        // M admin accounts and roles
        NewTable { name: "admin_account", copied: true, cols: &[
            ("id", 'T', "!", "adm-1"),
            ("username", 'T', "!", "ops"),
            ("display_name", 'T', "!", "Operations"),
            ("password_hash", 'T', "~", "$argon2id$v=19$stub"),
            ("disabled", 'I', "=0", "1"),
            ("source", 'T', "!", "db"),
            ("created_at", 'I', "!", TS),
            ("updated_at", 'I', "!", TS),
            ("last_login_at", 'I', "~", "1767229200"),
        ]},
        NewTable { name: "admin_totp", copied: true, cols: &[
            ("account_id", 'T', "!", "adm-1"),
            ("sealed_secret", 'B', "!", "sealed-totp"),
            ("last_step", 'I', "=0", "58907520"),
            ("enrolled_at", 'I', "!", TS),
        ]},
        NewTable { name: "admin_role", copied: true, cols: &[
            ("id", 'T', "!", "role-1"),
            ("name", 'T', "!", "Helpdesk"),
            ("description", 'T', "=", "Handles mailbox support requests."),
            ("builtin", 'I', "=0", "1"),
        ]},
        NewTable { name: "admin_role_capability", copied: true, cols: &[
            ("role_id", 'T', "!", "role-1"),
            ("capability", 'T', "!", "users.read"),
        ]},
        NewTable { name: "admin_account_role", copied: true, cols: &[
            ("account_id", 'T', "!", "adm-1"),
            ("role_id", 'T', "!", "role-1"),
            ("domains_json", 'T', "~", "[\"example.test\"]"),
        ]},
        NewTable { name: "admin_capability_override", copied: true, cols: &[
            ("account_id", 'T', "!", "adm-1"),
            ("capability", 'T', "!", "bans.write"),
            ("effect", 'T', "!", "deny"),
        ]},
        // AD/M audit
        NewTable { name: "audit_chain_head", copied: true, cols: &[
            ("id", 'I', "=1", "1"),
            ("last_id", 'I', "=0", "42"),
            ("hash", 'T', "~", "9f86d081"),
        ]},
        NewTable { name: "audit_checkpoint", copied: true, cols: &[
            ("id", 'T', "!", "ck-1"),
            ("kind", 'T', "!", "prune"),
            ("pruned_through_id", 'I', "=0", "40"),
            ("pruned_through_hash", 'T', "~", "e3b0c442"),
            ("ts", 'I', "!", TS),
        ]},
        NewTable { name: "audit_retention", copied: true, cols: &[
            ("category", 'T', "!", "auth"),
            ("days", 'I', "~", "365"),
            ("legal_hold", 'I', "=0", "1"),
            ("updated_by", 'T', "!", "admin:root"),
            ("updated_at", 'I', "!", TS),
        ]},
        NewTable { name: "audit_sink", copied: true, cols: &[
            ("id", 'T', "!", "sink-1"),
            ("kind", 'T', "!", "webhook"),
            ("sealed_config", 'B', "!", "sealed-sink-config"),
            ("enabled", 'I', "=1", "0"),
            ("cursor_id", 'I', "=0", "17"),
            ("last_ok_at", 'I', "~", "1767229200"),
            ("last_error", 'T', "~", "timeout"),
            ("created_by", 'T', "!", "admin:root"),
            ("created_at", 'I', "!", TS),
        ]},
        NewTable { name: "admin_saved_query", copied: true, cols: &[
            ("id", 'T', "!", "asq-1"),
            ("admin_account_id", 'T', "!", "adm-1"),
            ("name", 'T', "!", "failed sign-ins"),
            ("filter_json", 'T', "!", "{\"action\":\"login\"}"),
            ("created_at", 'I', "!", TS),
        ]},
        NewTable { name: "alert_rule", copied: true, cols: &[
            ("id", 'T', "!", "ar-1"),
            ("name", 'T', "!", "sign-in flood"),
            ("condition_json", 'T', "!", "{\"event\":\"login.failed\"}"),
            ("channels_json", 'T', "!", "[\"email\"]"),
            ("throttle_secs", 'I', "=900", "60"),
            ("enabled", 'I', "=1", "0"),
            ("created_by", 'T', "!", "admin:root"),
            ("created_at", 'I', "!", TS),
            ("updated_at", 'I', "!", TS),
        ]},
        NewTable { name: "alert_event", copied: true, cols: &[
            ("id", 'T', "!", "ae-1"),
            ("rule_id", 'T', "!", "ar-1"),
            ("fired_at", 'I', "!", TS),
            ("summary", 'T', "!", "12 failures in 5 min"),
            ("delivered_json", 'T', "!", "{\"email\":true}"),
            ("audit_id", 'T', "~", "audit-9"),
        ]},
        // AD sessions
        NewTable { name: "signin_event", copied: true, cols: &[
            ("id", 'T', "!", "se-1"),
            ("account_id", 'T', "~", "acct-1"),
            ("username_attempted", 'T', "!", "alice@example.test"),
            ("ts", 'I', "!", TS),
            ("result", 'T', "!", "failure"),
            ("method", 'T', "!", "password"),
            ("ip", 'T', "!", "192.0.2.7"),
            ("user_agent", 'T', "!", "Mozilla/5.0"),
            ("geo_country", 'T', "~", "GB"),
            ("reason", 'T', "!", "badCredentials"),
            ("surface", 'T', "=webmail", "webmail"),
            ("backend", 'T', "~", "imap"),
            ("last_seen", 'I', "=0", TS),
            ("repeat_count", 'I', "=1", "4"),
            ("aggregate_key", 'T', "~", "aggregate-1"),
        ]},
        NewTable { name: "signin_identity", copied: true, cols: &[
            ("account_id", 'T', "!", "acct-1"),
            ("address", 'T', "!", "alice@example.test"),
            ("seen_at", 'I', "!", TS),
            ("event_id", 'T', "!", "success-1"),
        ]},
        NewTable { name: "remembered_device", copied: true, cols: &[
            ("id", 'T', "!", "rd-1"),
            ("account_id", 'T', "!", "acct-1"),
            ("token_hash", 'T', "!", "rd-token-hash"),
            ("label", 'T', "!", "Laptop"),
            ("created_at", 'I', "!", TS),
            ("expires_at", 'I', "!", "1769817600"),
            ("last_used_at", 'I', "!", "1767229200"),
        ]},
        // P/Q/Z/X/AF assets
        NewTable { name: "branding", copied: true, cols: &[
            ("scope", 'T', "!", "global"),
            ("config_json", 'T', "!", "{\"productName\":\"Mail\"}"),
            ("updated_by", 'T', "!", "admin:root"),
            ("updated_at", 'I', "!", TS),
        ]},
        NewTable { name: "brand_asset", copied: true, cols: &[
            ("id", 'T', "!", "ba-1"),
            ("scope", 'T', "!", "global"),
            ("kind", 'T', "!", "user_background"),
            ("owner_account", 'T', "~", "acct-1"),
            ("sha256", 'T', "!", "ab12"),
            ("content_type", 'T', "!", "image/webp"),
            ("bytes", 'I', "!", "2048"),
            ("width", 'I', "~", "1920"),
            ("height", 'I', "~", "1080"),
            ("duration_ms", 'I', "~", "1500"),
            ("licence_ack", 'T', "~", "OFL-1.1"),
            ("data", 'B', "!", "RIFF....WEBP"),
            ("created_by", 'T', "!", "acct-1"),
            ("created_at", 'I', "!", TS),
        ]},
        NewTable { name: "custom_css", copied: true, cols: &[
            ("scope_kind", 'T', "!", "domain_app"),
            ("scope_id", 'T', "!", "example.test"),
            ("source", 'T', "!", ".x{color:red}"),
            ("sanitized", 'T', "!", ".x{color:red}"),
            ("sha256", 'T', "!", "cd34"),
            ("updated_by", 'T', "!", "admin:root"),
            ("updated_at", 'I', "!", TS),
        ]},
        // V2 themes
        NewTable { name: "custom_theme", copied: true, cols: &[
            ("id", 'T', "!", "th-1"),
            ("name", 'T', "!", "Harbour"),
            ("appearance", 'T', "!", "dark"),
            ("tokens_json", 'T', "!", "{\"bg\":\"#000\"}"),
            ("enabled", 'I', "=1", "0"),
            ("scope", 'T', "=global", "example.test"),
            ("created_by", 'T', "!", "admin:root"),
            ("created_at", 'I', "!", TS),
            ("updated_at", 'I', "!", TS),
        ]},
        NewTable { name: "custom_accent", copied: true, cols: &[
            ("id", 'T', "!", "acc-1"),
            ("name", 'T', "!", "Coral"),
            ("shades_json", 'T', "!", "{\"th-1\":\"#ff7f50\"}"),
            ("enabled", 'I', "=1", "0"),
            ("scope", 'T', "=global", "example.test"),
            ("created_by", 'T', "!", "admin:root"),
            ("created_at", 'I', "!", TS),
            ("updated_at", 'I', "!", TS),
        ]},
        // S Sieve
        NewTable { name: "sieve_script", copied: true, cols: &[
            ("account_id", 'T', "!", "acct-1"),
            ("name", 'T', "!", "main"),
            ("content", 'T', "!", "keep;"),
            ("active", 'I', "=0", "1"),
            ("size", 'I', "!", "5"),
            ("state_json", 'T', "={}", "{\"vacation\":{}}"),
            ("updated_at", 'I', "!", TS),
        ]},
        // R2 egress
        NewTable { name: "egress_internal_allow", copied: false, cols: &[
            ("id", 'T', "!", "eia-1"),
            ("traffic_class", 'T', "!", "siem"),
            ("destination", 'T', "!", "10.0.0.5:6514"),
            ("note", 'T', "=", "SIEM collector"),
            ("created_by", 'T', "!", "admin:root"),
            ("created_at", 'I', "!", TS),
        ]},
        // W PKI
        NewTable { name: "smime_trust_root", copied: true, cols: &[
            ("id", 'T', "!", "str-1"),
            ("scope", 'T', "!", "global"),
            ("der", 'B', "!", "0DER-bytes"),
            ("fingerprint", 'T', "!", "sha256:ef56"),
            ("label", 'T', "!", "Corp CA"),
            ("added_by", 'T', "!", "admin:root"),
            ("added_at", 'I', "!", TS),
        ]},
        NewTable { name: "pki_revocation_cache", copied: true, cols: &[
            ("issuer_key_hash", 'T', "!", "ikh-1"),
            ("serial", 'T', "!", "0a1b"),
            ("status", 'T', "!", "revoked"),
            ("source", 'T', "!", "crl"),
            ("this_update", 'I', "!", TS),
            ("next_update", 'I', "~", "1767312000"),
            ("checked_at", 'I', "!", TS),
        ]},
        // X signatures
        NewTable { name: "identity_signature_default", copied: true, cols: &[
            ("identity_id", 'T', "!", "id-1"),
            ("mode", 'T', "!", "reply"),
            ("signature_name", 'T', "!", "Short"),
            ("placement", 'T', "!", "above"),
            ("separator", 'I', "=1", "0"),
        ]},
        NewTable { name: "domain_signature_template", copied: true, cols: &[
            ("domain", 'T', "!", "example.test"),
            ("name", 'T', "!", "Standard"),
            ("html", 'T', "!", "<p>{{name}}</p>"),
            ("text", 'T', "!", "{{name}}"),
            ("user_editable", 'I', "!", "1"),
            ("updated_by", 'T', "!", "admin:root"),
            ("updated_at", 'I', "!", TS),
        ]},
        NewTable { name: "domain_disclaimer", copied: true, cols: &[
            ("domain", 'T', "!", "example.test"),
            ("html", 'T', "!", "<p>Confidential</p>"),
            ("text", 'T', "!", "Confidential"),
            ("enabled", 'I', "!", "1"),
            ("updated_by", 'T', "!", "admin:root"),
            ("updated_at", 'I', "!", TS),
        ]},
        // Y3 receipts
        NewTable { name: "mdn_sender_rule", copied: true, cols: &[
            ("account_id", 'T', "!", "acct-1"),
            ("address", 'T', "!", "boss@example.test"),
            ("decision", 'T', "!", "always"),
            ("updated_at", 'I', "!", TS),
        ]},
        NewTable { name: "receipt_status", copied: true, cols: &[
            ("account_id", 'T', "!", "acct-1"),
            ("sent_email_id", 'T', "!", "em-1"),
            ("recipient", 'T', "!", "bob@example.test"),
            ("kind", 'T', "!", "dsn"),
            ("status", 'T', "!", "delivered"),
            ("reported_at", 'I', "~", TS),
            ("report_email_id", 'T', "~", "em-2"),
        ]},
        // AB linked accounts and delegation
        NewTable { name: "child_account", copied: true, cols: &[
            ("owner_account_id", 'T', "!", "acct-1"),
            ("account_id", 'T', "!", "acct-2"),
            ("provider_id", 'T', "~", "pp-1"),
            ("display_name", 'T', "!", "Old mailbox"),
            ("accent_id", 'T', "~", "acc-1"),
            ("position", 'I', "!", "2"),
            ("paused", 'I', "=0", "1"),
            ("added_at", 'I', "!", TS),
        ]},
        NewTable { name: "provider_profile", copied: true, cols: &[
            ("id", 'T', "!", "pp-1"),
            ("name", 'T', "!", "Example IMAP"),
            ("kind", 'T', "!", "pop3"),
            ("endpoints_json", 'T', "!", "{\"pop3\":\"pop.example.test:995\"}"),
            ("oauth_client_id", 'T', "~", "oc-1"),
            ("tunnel_id", 'T', "~", "tun-1"),
            ("egress_route_id", 'T', "~", "eg-1"),
            ("scope", 'T', "=global", "example.test"),
            ("enabled", 'I', "=0", "1"),
            ("preset", 'T', "~", "generic"),
            ("created_by", 'T', "!", "admin:root"),
            ("updated_at", 'I', "!", TS),
        ]},
        NewTable { name: "oauth_client", copied: false, cols: &[
            ("id", 'T', "!", "oc-1"),
            ("provider_id", 'T', "!", "pp-1"),
            ("client_id", 'T', "!", "client-abc"),
            ("sealed_client_secret", 'B', "~", "sealed-client-secret"),
            ("auth_url", 'T', "!", "https://idp.example.test/auth"),
            ("token_url", 'T', "!", "https://idp.example.test/token"),
            ("device_url", 'T', "~", "https://idp.example.test/device"),
            ("scopes", 'T', "!", "mail offline_access"),
            ("redirect", 'T', "!", "https://mail.example.test/cb"),
            ("created_by", 'T', "!", "admin:root"),
            ("updated_at", 'I', "!", TS),
        ]},
        NewTable { name: "account_oauth", copied: true, cols: &[
            ("account_id", 'T', "!", "acct-2"),
            ("provider_id", 'T', "!", "pp-1"),
            ("sealed_refresh_token", 'B', "!", "sealed-refresh"),
            ("expires_at", 'I', "!", "1767229200"),
            ("scope", 'T', "!", "mail"),
            ("needs_signin", 'I', "=0", "1"),
            ("updated_at", 'I', "!", TS),
        ]},
        NewTable { name: "delegation_grant", copied: true, cols: &[
            ("id", 'T', "!", "dg-1"),
            ("owner_account_id", 'T', "!", "acct-1"),
            ("delegate_owner_account_id", 'T', "~", "acct-3"),
            ("delegate_group_id", 'T', "~", "grp-1"),
            ("scope_kind", 'T', "!", "mailboxes"),
            ("mailbox_ids_json", 'T', "!", "[\"mb-1\"]"),
            ("permissions_json", 'T', "!", "[\"read\"]"),
            ("expires_at", 'I', "~", "1769817600"),
            ("created_at", 'I', "!", TS),
            ("revoked_at", 'I', "~", "1767229200"),
        ]},
        NewTable { name: "shared_mailbox", copied: true, cols: &[
            ("account_id", 'T', "!", "acct-9"),
            ("domain", 'T', "!", "example.test"),
            ("name", 'T', "!", "support"),
            ("created_by", 'T', "!", "admin:root"),
            ("created_at", 'I', "!", TS),
        ]},
        // AC views
        NewTable { name: "org_view", copied: true, cols: &[
            ("id", 'T', "!", "ov-1"),
            ("scope", 'T', "!", "example.test"),
            ("name", 'T', "!", "Invoices"),
            ("query_text", 'T', "!", "subject:invoice"),
            ("settings_json", 'T', "!", "{}"),
            ("mandatory", 'I', "=0", "1"),
            ("created_by", 'T', "!", "admin:root"),
            ("updated_at", 'I', "!", TS),
        ]},
        // AE identities
        NewTable { name: "identity_verification", copied: true, cols: &[
            ("token_hash", 'T', "!", "iv-hash"),
            ("identity_id", 'T', "!", "id-1"),
            ("sent_at", 'I', "!", TS),
            ("expires_at", 'I', "!", "1767312000"),
            ("consumed", 'I', "=0", "1"),
        ]},
        // AF templates
        NewTable { name: "template", copied: true, cols: &[
            ("id", 'T', "!", "tpl-1"),
            ("owner_kind", 'T', "!", "domain"),
            ("owner_id", 'T', "!", "example.test"),
            ("folder", 'T', "!", "Sales"),
            ("name", 'T', "!", "Quote"),
            ("subject", 'T', "!", "Your quote"),
            ("html", 'T', "!", "<p>Hello</p>"),
            ("text", 'T', "!", "Hello"),
            ("attachments_json", 'T', "!", "[]"),
            ("identity_id", 'T', "~", "id-1"),
            ("recipients_json", 'T', "!", "{\"to\":[]}"),
            ("fields_json", 'T', "!", "[\"name\"]"),
            ("mandatory", 'I', "=0", "1"),
            ("user_copyable", 'I', "=1", "0"),
            ("version", 'I', "=1", "3"),
            ("usage_count", 'I', "=0", "7"),
            ("created_by", 'T', "!", "admin:root"),
            ("created_at", 'I', "!", TS),
            ("updated_at", 'I', "!", TS),
        ]},
        NewTable { name: "template_version", copied: true, cols: &[
            ("template_id", 'T', "!", "tpl-1"),
            ("version", 'I', "!", "3"),
            ("subject", 'T', "!", "Your quote"),
            ("html", 'T', "!", "<p>Hello</p>"),
            ("text", 'T', "!", "Hello"),
            ("attachments_json", 'T', "!", "[]"),
            ("created_by", 'T', "!", "admin:root"),
            ("created_at", 'I', "!", TS),
        ]},
        // AG groups and feature profiles
        NewTable { name: "user_group", copied: true, cols: &[
            ("id", 'T', "!", "grp-1"),
            ("name", 'T', "!", "Sales"),
            ("domain", 'T', "!", "example.test"),
            ("source", 'T', "!", "ldap"),
            ("ldap_dn", 'T', "~", "cn=sales,dc=example,dc=test"),
            ("created_at", 'I', "!", TS),
        ]},
        NewTable { name: "user_group_member", copied: true, cols: &[
            ("group_id", 'T', "!", "grp-1"),
            ("account_id", 'T', "!", "acct-1"),
        ]},
        NewTable { name: "feature_profile", copied: true, cols: &[
            ("id", 'T', "!", "fp-1"),
            ("name", 'T', "!", "Restricted"),
            ("values_json", 'T', "!", "{\"assistant.enabled\":false}"),
            ("builtin", 'I', "=0", "1"),
            ("updated_at", 'I', "!", TS),
        ]},
        NewTable { name: "feature_profile_assignment", copied: true, cols: &[
            ("scope_kind", 'T', "!", "group"),
            ("scope_id", 'T', "!", "grp-1"),
            ("profile_id", 'T', "!", "fp-1"),
            ("priority", 'I', "!", "10"),
        ]},
        // Q users, sign-in policy, usage
        NewTable { name: "mail_user", copied: true, cols: &[
            ("address", 'T', "!", "alice@example.test"),
            ("domain", 'T', "!", "example.test"),
            ("local_part", 'T', "!", "alice"),
            ("origin", 'T', "!", "provisioned"),
            ("status", 'T', "=active", "disabled"),
            ("display_name", 'T', "~", "Alice"),
            ("note", 'T', "~", "contractor"),
            ("created_at", 'I', "!", TS),
            ("created_by", 'T', "~", "admin:root"),
            ("first_signin_at", 'I', "~", "1767229200"),
            ("last_signin_at", 'I', "~", "1767232800"),
            ("status_changed_at", 'I', "~", "1767236400"),
            ("status_changed_by", 'T', "~", "admin:root"),
            ("status_reason", 'T', "~", "left"),
            ("expires_at", 'I', "~", "1769817600"),
            ("deleted_at", 'I', "~", "1769904000"),
        ]},
        NewTable { name: "mail_user_account", copied: true, cols: &[
            ("account_id", 'T', "!", "acct-2"),
            ("address", 'T', "!", "alice@example.test"),
            ("mode", 'T', "!", "child"),
            ("first_seen", 'I', "!", TS),
            ("last_seen", 'I', "!", "1767229200"),
        ]},
        NewTable { name: "signin_allow", copied: true, cols: &[
            ("id", 'T', "!", "sa-1"),
            ("domain", 'T', "!", "example.test"),
            ("pattern", 'T', "!", "bot-*"),
            ("note", 'T', "~", "invitation"),
            ("expires_at", 'I', "~", "1769817600"),
            ("max_uses", 'I', "~", "1"),
            ("uses", 'I', "=0", "1"),
            ("created_by", 'T', "!", "admin:root"),
            ("created_at", 'I', "!", TS),
        ]},
        NewTable { name: "usage_counter", copied: true, cols: &[
            ("address", 'T', "!", "alice@example.test"),
            ("domain", 'T', "!", "example.test"),
            ("metric", 'T', "!", "send.messages"),
            ("day", 'I', "!", "20454"),
            ("value", 'I', "=0", "12"),
        ]},
        NewTable { name: "usage_storage", copied: true, cols: &[
            ("address", 'T', "!", "alice@example.test"),
            ("domain", 'T', "!", "example.test"),
            ("blob_bytes", 'I', "=0", "1000"),
            ("blob_count", 'I', "=0", "2"),
            ("content_bytes", 'I', "=0", "300"),
            ("cache_bytes", 'I', "=0", "50000"),
            ("cache_messages", 'I', "=0", "40"),
            ("search_docs", 'I', "~", "40"),
            ("breakdown_json", 'T', "~", "{\"notes\":300}"),
            ("measured_at", 'I', "~", TS),
            ("measuring_until", 'I', "~", "1767226200"),
            ("upstream_used_bytes", 'I', "~", "900000"),
            ("upstream_limit_bytes", 'I', "~", "1000000"),
            ("upstream_json", 'T', "~", "{\"root\":{}}"),
            ("upstream_status", 'T', "~", "ok"),
            ("upstream_checked_at", 'I', "~", TS),
            ("warn_level", 'I', "=0", "80"),
        ]},
        // CFG configuration, maintenance, jobs, diagnostics, bans, metrics, export
        NewTable { name: "config_snapshot", copied: true, cols: &[
            ("id", 'T', "!", "cs-1"),
            ("created_at", 'I', "!", TS),
            ("created_by", 'T', "!", "admin:root"),
            ("reason", 'T', "!", "pre-import"),
            ("sha256", 'T', "!", "0f0f"),
            ("json", 'T', "!", "{\"version\":1}"),
        ]},
        NewTable { name: "integration_config", copied: false, cols: &[
            ("kind", 'T', "!", "nextcloud"),
            ("config_json", 'T', "={}", "{\"url\":\"https://cloud.example.test\"}"),
            ("sealed_secret", 'B', "~", "sealed-app-password"),
            ("secret_hash", 'T', "~", "sha256:1234"),
            ("enabled", 'I', "=0", "1"),
            ("updated_by", 'T', "!", "admin:root"),
            ("updated_at", 'I', "!", TS),
        ]},
        NewTable { name: "maintenance_window", copied: true, cols: &[
            ("id", 'T', "!", "mw-1"),
            ("scope_kind", 'T', "=global", "domain"),
            ("scope_id", 'T', "=", "example.test"),
            ("mode", 'T', "!", "read_only"),
            ("starts_at", 'I', "!", TS),
            ("ends_at", 'I', "~", "1767232800"),
            ("tz", 'T', "=UTC", "Europe/London"),
            ("recurrence_json", 'T', "~", "{\"weekly\":{\"days\":[0]}}"),
            ("announce_minutes", 'I', "=60", "15"),
            ("message_json", 'T', "={}", "{\"en\":\"Upgrade\"}"),
            ("created_by", 'T', "!", "admin:root"),
            ("created_at", 'I', "!", TS),
            ("updated_by", 'T', "!", "admin:root"),
            ("updated_at", 'I', "!", TS),
            ("occurrence_ended_at", 'I', "~", "1767230000"),
            ("cancelled_at", 'I', "~", "1767231000"),
            ("cancelled_by", 'T', "~", "admin:root"),
        ]},
        NewTable { name: "scheduled_task", copied: true, cols: &[
            ("id", 'T', "!", "st-1"),
            ("kind", 'T', "!", "vacuum"),
            ("schedule_json", 'T', "!", "{\"every_secs\":86400}"),
            ("params_json", 'T', "={}", "{\"full\":true}"),
            ("enabled", 'I', "=0", "1"),
            ("only_in_window", 'I', "=0", "1"),
            ("created_by", 'T', "!", "admin:root"),
            ("created_at", 'I', "!", TS),
            ("updated_at", 'I', "!", TS),
            ("last_run_at", 'I', "~", "1767229200"),
            ("last_status", 'T', "~", "ok"),
            ("next_run_at", 'I', "~", "1767312000"),
            ("lease_owner", 'T', "~", "host-a:1767225600"),
            ("lease_until", 'I', "~", "1767229800"),
        ]},
        NewTable { name: "job_run", copied: true, cols: &[
            ("id", 'T', "!", "jr-1"),
            ("task_id", 'T', "~", "st-1"),
            ("kind", 'T', "!", "vacuum"),
            ("instance", 'T', "!", "host-a:1767225600"),
            ("started_at", 'I', "!", TS),
            ("heartbeat_at", 'I', "!", "1767225660"),
            ("finished_at", 'I', "~", "1767225700"),
            ("status", 'T', "!", "abandoned"),
            ("summary", 'T', "~", "lease lost"),
            ("detail_json", 'T', "~", "{}"),
            ("triggered_by", 'T', "!", "schedule"),
        ]},
        NewTable { name: "diag_run", copied: true, cols: &[
            ("id", 'T', "!", "dr-1"),
            ("suite", 'T', "!", "all"),
            ("target", 'T', "~", "example.test"),
            ("started_at", 'I', "!", TS),
            ("finished_at", 'I', "~", "1767225700"),
            ("status", 'T', "!", "warn"),
            ("results_json", 'T', "=[]", "[{\"id\":\"dns.mx\"}]"),
            ("triggered_by", 'T', "!", "admin:root"),
            ("task_id", 'T', "~", "st-1"),
        ]},
        NewTable { name: "diag_probe_account", copied: false, cols: &[
            ("domain", 'T', "!", "example.test"),
            ("address", 'T', "!", "probe@example.test"),
            ("sealed_password", 'B', "!", "sealed-probe-password"),
            ("created_by", 'T', "!", "admin:root"),
            ("updated_at", 'I', "!", TS),
            ("last_loop_at", 'I', "~", "1767229200"),
            ("last_loop_ms", 'I', "~", "840"),
            ("last_loop_status", 'T', "~", "pass"),
        ]},
        NewTable { name: "account_ban", copied: true, cols: &[
            ("id", 'T', "!", "ban-1"),
            ("kind", 'T', "!", "pattern"),
            ("value", 'T', "!", "*@spam.example"),
            ("surface", 'T', "=webmail", "all"),
            ("reason", 'T', "!", "spam run"),
            ("user_note", 'T', "~", "contact support"),
            ("starts_at", 'I', "!", TS),
            ("expires_at", 'I', "~", "1769817600"),
            ("source", 'T', "=manual", "import"),
            ("created_by", 'T', "!", "admin:root"),
            ("created_at", 'I', "!", TS),
            ("lifted_at", 'I', "~", "1767312000"),
            ("lifted_by", 'T', "~", "admin:root"),
            ("lift_reason", 'T', "~", "resolved"),
            ("hits", 'I', "=0", "4"),
            ("last_hit_at", 'I', "~", "1767229200"),
        ]},
        NewTable { name: "metric_rollup", copied: true, cols: &[
            ("series", 'T', "!", "http.requests"),
            ("hour", 'I', "!", "490896"),
            ("instance", 'T', "!", "host-a"),
            ("value_json", 'T', "!", "{\"sum\":10}"),
        ]},
        NewTable { name: "data_export", copied: true, cols: &[
            ("id", 'T', "!", "dx-1"),
            ("address", 'T', "!", "alice@example.test"),
            ("requested_by", 'T', "!", "admin:root"),
            ("status", 'T', "!", "ready"),
            ("item_counts_json", 'T', "={}", "{\"mail\":3}"),
            ("bytes", 'I', "~", "4096"),
            ("blob_ref", 'T', "~", "export/dx-1"),
            ("created_at", 'I', "!", TS),
            ("expires_at", 'I', "!", "1767312000"),
            ("downloaded_at", 'I', "~", "1767229200"),
        ]},
    ];

    /// `(column, type, rule)` of a column added to an existing table.
    type AddedCol = (&'static str, char, &'static str);

    /// `(table, columns)`: the columns 0033 adds to existing tables.
    #[rustfmt::skip]
    const ALTERED: &[(&str, &[AddedCol])] = &[
        ("domains", &[
            ("enabled", 'I', "=1"),
            ("preferred", 'I', "=0"),
            ("settings_json", 'T', "={}"),
            ("updated_at", 'I', "~"),
        ]),
        ("audit_log", &[
            ("admin_role", 'T', "~"),
            ("capability", 'T', "~"),
            ("category", 'T', "~"),
            ("request_id", 'T', "~"),
            ("before_json", 'T', "~"),
            ("after_json", 'T', "~"),
            ("prev_hash", 'T', "~"),
            ("hash", 'T', "~"),
            ("seq", 'I', "~"),
        ]),
        ("sessions", &[
            ("authentication_method", 'T', "=legacy"),
            ("user_agent", 'T', "~"),
            ("ip", 'T', "~"),
            ("client_type", 'T', "~"),
            ("device_label", 'T', "~"),
            ("client_version", 'T', "~"),
        ]),
        ("webauthn_credentials", &[("discoverable", 'I', "=0"), ("backup_eligible", 'I', "~"), ("backup_state", 'I', "=0"), ("uv_initialized", 'I', "=0"), ("state_revision", 'I', "=0")]),
        ("egress_proxy", &[
            ("kind", 'T', "=http"),
            ("tunnel_id", 'T', "~"),
            ("tls_to_proxy", 'I', "=0"),
            ("remote_dns", 'I', "=0"),
            ("remote_dns_risk_accepted_by", 'T', "~"),
            ("remote_dns_risk_accepted_at", 'I', "~"),
            ("chain_next_id", 'T', "~"),
            ("scope", 'T', "=global"),
            ("traffic_classes_json", 'T', "=[\"remote_content\",\"ics_feeds\"]"),
        ]),
        ("signatures", &[
            ("html", 'T', "~"),
            ("text", 'T', "~"),
            ("text_edited", 'I', "=0"),
            ("images_json", 'T', "=[]"),
        ]),
        ("saved_searches", &[
            ("query_text", 'T', "~"),
            ("builder_json", 'T', "~"),
            ("accounts_json", 'T', "~"),
            ("settings_json", 'T', "={}"),
            ("position", 'I', "=0"),
            ("icon", 'T', "~"),
            ("accent_id", 'T', "~"),
            ("start_page", 'I', "=0"),
            ("source", 'T', "=user"),
            ("org_view_id", 'T', "~"),
        ]),
        ("identities", &[
            ("auto_bcc", 'T', "~"),
            ("color", 'T', "~"),
            ("pgp_key_id", 'T', "~"),
            ("smime_cert_id", 'T', "~"),
            ("request_receipt", 'I', "=0"),
            ("verified_at", 'I', "~"),
            ("sender_header", 'T', "~"),
        ]),
        ("directory_config", &[("sealed_bind_password", 'B', "~")]),
    ];

    /// `(table, column, allowed values)`. One value outside the list must be
    /// refused, and every listed value accepted, in both dialects.
    #[rustfmt::skip]
    const CHECKS: &[(&str, &str, &[&str])] = &[
        ("setting_value", "scope_kind", &["domain", "profile", "group", "admin_user", "user"]),
        ("auth_failure", "surface", &["admin", "webmail"]),
        ("auth_failure", "key_kind", &["ip", "account"]),
        ("ip_block", "surface", &["admin", "webmail", "all"]),
        ("ip_block", "reason", &["auto", "manual"]),
        ("ip_allow", "surface", &["admin", "webmail"]),
        ("ip_allow", "scope_kind", &["global", "domain", "account"]),
        ("admin_account", "source", &["db", "env-bootstrap"]),
        ("admin_capability_override", "effect", &["allow", "deny"]),
        ("audit_chain_head", "id", &["1"]),
        ("audit_checkpoint", "kind", &["genesis", "prune"]),
        ("audit_sink", "kind", &["syslog", "webhook"]),
        ("signin_event", "result", &["success", "failure", "blocked"]),
        ("brand_asset", "kind", &[
            "logo", "background", "icon_master", "font", "preset", "user_background",
            "signature", "template_attachment", "sound",
        ]),
        ("custom_css", "scope_kind", &[
            "global_login", "global_app", "domain_login", "domain_app", "user",
        ]),
        ("custom_theme", "appearance", &["light", "dark"]),
        ("egress_internal_allow", "traffic_class", &["siem", "webhooks"]),
        ("pki_revocation_cache", "status", &["good", "revoked", "unknown"]),
        ("pki_revocation_cache", "source", &["ocsp", "crl"]),
        ("identity_signature_default", "mode", &["new", "reply", "forward"]),
        ("identity_signature_default", "placement", &["below", "above"]),
        ("mdn_sender_rule", "decision", &["always", "never"]),
        ("receipt_status", "kind", &["mdn", "dsn"]),
        ("provider_profile", "kind", &["imap", "pop3", "bridge"]),
        ("delegation_grant", "scope_kind", &["account", "mailboxes"]),
        ("template", "owner_kind", &["user", "domain", "group"]),
        ("user_group", "source", &["manual", "ldap"]),
        ("feature_profile_assignment", "scope_kind", &["domain", "group", "user"]),
        ("mail_user", "origin", &["provisioned", "signin", "sso", "import"]),
        ("mail_user", "status", &["active", "disabled", "deleting", "deleted"]),
        ("mail_user_account", "mode", &["engine", "proxy", "header", "sso", "child"]),
        ("config_snapshot", "reason", &["export", "pre-import", "manual"]),
        ("maintenance_window", "scope_kind", &["global", "domain"]),
        ("maintenance_window", "mode", &["announce", "read_only", "signin_closed", "full"]),
        ("job_run", "status", &["running", "ok", "failed", "cancelled", "abandoned"]),
        ("diag_run", "status", &["running", "pass", "warn", "fail", "error"]),
        ("account_ban", "kind", &["user", "pattern", "domain", "asn", "country"]),
        ("account_ban", "surface", &["webmail", "admin", "all"]),
        ("account_ban", "source", &["manual", "import", "rule"]),
        ("data_export", "status", &[
            "queued", "running", "ready", "downloaded", "expired", "failed",
        ]),
    ];

    /// Columns that deliberately carry NO check: any value is stored (A16: a new
    /// integration kind needs no SQLite table rebuild).
    const UNCHECKED: &[(&str, &str, &str)] = &[("integration_config", "kind", "future_kind")];

    /// `(child, child column, parent, parent column)`, all `ON DELETE CASCADE`.
    /// These are the ONLY foreign keys the new tables declare.
    const FKS: &[(&str, &str, &str, &str)] = &[
        ("domain_alias", "domain", "domains", "name"),
        ("domain_host", "domain", "domains", "name"),
        ("admin_totp", "account_id", "admin_account", "id"),
        ("admin_role_capability", "role_id", "admin_role", "id"),
        ("admin_account_role", "account_id", "admin_account", "id"),
        ("admin_account_role", "role_id", "admin_role", "id"),
        (
            "admin_capability_override",
            "account_id",
            "admin_account",
            "id",
        ),
        (
            "admin_saved_query",
            "admin_account_id",
            "admin_account",
            "id",
        ),
        ("template_version", "template_id", "template", "id"),
        ("user_group_member", "group_id", "user_group", "id"),
        (
            "feature_profile_assignment",
            "profile_id",
            "feature_profile",
            "id",
        ),
        ("diag_probe_account", "domain", "domains", "name"),
    ];

    /// `(index, table, unique, partial, columns)`, named identically in both
    /// dialects so they can be compared by name.
    #[rustfmt::skip]
    const INDEXES: &[(&str, &str, bool, bool, &str)] = &[
        ("idx_setting_value_key", "setting_value", false, false, "key"),
        ("idx_auth_failure_window", "auth_failure", false, false, "window_start"),
        ("idx_ip_block_until", "ip_block", false, false, "blocked_until"),
        ("idx_ip_block_ip", "ip_block", false, false, "ip_or_cidr"),
        ("idx_ip_allow_scope", "ip_allow", false, false, "surface,scope_kind,scope_id"),
        ("idx_domains_preferred", "domains", true, true, "preferred"),
        ("idx_domain_alias_domain", "domain_alias", false, false, "domain"),
        ("idx_domain_host_domain", "domain_host", false, false, "domain"),
        ("idx_admin_account_username", "admin_account", true, false, "username"),
        ("idx_admin_role_name", "admin_role", true, false, "name"),
        ("idx_admin_account_role_role", "admin_account_role", false, false, "role_id"),
        ("idx_audit_log_seq", "audit_log", true, false, "seq"),
        ("idx_audit_log_request", "audit_log", false, false, "request_id"),
        ("idx_audit_log_category", "audit_log", false, false, "category,ts"),
        ("idx_admin_saved_query_account", "admin_saved_query", false, false, "admin_account_id"),
        ("idx_alert_event_rule", "alert_event", false, false, "rule_id,fired_at"),
        ("idx_signin_event_account", "signin_event", false, false, "account_id,ts"),
        ("idx_signin_event_ts", "signin_event", false, false, "ts"),
        ("idx_remembered_device_token", "remembered_device", true, false, "token_hash"),
        ("idx_remembered_device_account", "remembered_device", false, false, "account_id"),
        ("idx_brand_asset_owner", "brand_asset", false, false, "owner_account,kind"),
        ("idx_brand_asset_scope", "brand_asset", false, false, "scope,kind"),
        ("idx_sieve_script_active", "sieve_script", true, true, "account_id"),
        ("idx_identity_verification_identity", "identity_verification", false, false, "identity_id"),
        ("idx_child_account_account", "child_account", false, false, "account_id"),
        ("idx_oauth_client_provider", "oauth_client", false, false, "provider_id"),
        ("idx_delegation_grant_owner", "delegation_grant", false, false, "owner_account_id"),
        ("idx_delegation_grant_delegate", "delegation_grant", false, false, "delegate_owner_account_id"),
        ("idx_delegation_grant_group", "delegation_grant", false, false, "delegate_group_id"),
        ("idx_shared_mailbox_domain", "shared_mailbox", false, false, "domain"),
        ("idx_template_owner", "template", false, false, "owner_kind,owner_id"),
        ("idx_user_group_domain", "user_group", false, false, "domain"),
        ("idx_user_group_member_account", "user_group_member", false, false, "account_id"),
        ("idx_feature_profile_name", "feature_profile", true, false, "name"),
        ("idx_feature_profile_assignment_profile", "feature_profile_assignment", false, false, "profile_id"),
        ("idx_mail_user_domain", "mail_user", false, false, "domain,address"),
        ("idx_mail_user_status", "mail_user", false, false, "status"),
        ("idx_mail_user_last_signin", "mail_user", false, false, "last_signin_at"),
        ("idx_mail_user_account_address", "mail_user_account", false, false, "address"),
        ("idx_signin_allow_domain_pattern", "signin_allow", true, false, "domain,pattern"),
        ("idx_usage_counter_domain", "usage_counter", false, false, "domain,metric,day"),
        ("idx_usage_counter_day", "usage_counter", false, false, "day"),
        ("idx_usage_storage_domain", "usage_storage", false, false, "domain"),
        ("idx_bodies_account", "bodies", false, false, "account_id"),
        ("idx_config_snapshot_created", "config_snapshot", false, false, "created_at"),
        ("idx_maintenance_window_starts", "maintenance_window", false, false, "starts_at"),
        ("idx_scheduled_task_next", "scheduled_task", false, false, "next_run_at"),
        ("idx_job_run_kind", "job_run", false, false, "kind,started_at"),
        ("idx_diag_run_suite", "diag_run", false, false, "suite,started_at"),
        ("idx_account_ban_kind", "account_ban", false, false, "kind,value"),
        ("idx_account_ban_expires", "account_ban", false, false, "expires_at"),
        ("idx_metric_rollup_hour", "metric_rollup", false, false, "hour"),
        ("idx_data_export_address", "data_export", false, false, "address"),
        ("idx_data_export_expires", "data_export", false, false, "expires_at"),
    ];

    type Overrides = &'static [(&'static str, &'static str)];

    /// `(table, first row overrides, second row overrides, second must conflict)`.
    #[rustfmt::skip]
    const UNIQUES: &[(&str, Overrides, Overrides, bool)] = &[
        ("admin_account", &[("id", "a1"), ("username", "dup")], &[("id", "a2"), ("username", "dup")], true),
        ("admin_role", &[("id", "r1"), ("name", "dup")], &[("id", "r2"), ("name", "dup")], true),
        ("feature_profile", &[("id", "f1"), ("name", "dup")], &[("id", "f2"), ("name", "dup")], true),
        ("remembered_device", &[("id", "d1"), ("token_hash", "h")], &[("id", "d2"), ("token_hash", "h")], true),
        ("signin_allow", &[("id", "s1")], &[("id", "s2")], true),
        ("signin_allow", &[("id", "s1")], &[("id", "s2"), ("domain", "other.test")], false),
        ("sieve_script", &[("name", "a"), ("active", "1")], &[("name", "b"), ("active", "1")], true),
        ("sieve_script", &[("name", "a"), ("active", "0")], &[("name", "b"), ("active", "0")], false),
        ("sieve_script", &[("name", "a"), ("active", "1")], &[("account_id", "acct-x"), ("active", "1")], false),
    ];

    // ── a small two-dialect SQL layer ────────────────────────────────────────

    #[derive(Clone, Debug)]
    enum Val {
        T(String),
        I(i64),
        B(Vec<u8>),
        /// A typed NULL: Postgres needs the parameter type.
        N(char),
    }

    fn val(ty: char, s: &str) -> Val {
        match ty {
            'T' => Val::T(s.to_string()),
            'I' => Val::I(s.parse().unwrap_or_else(|_| panic!("integer sample {s:?}"))),
            'B' => Val::B(s.as_bytes().to_vec()),
            other => panic!("unknown type {other}"),
        }
    }

    fn render_sample(ty: char, s: &str) -> String {
        match ty {
            'B' => format!("b:{s}"),
            _ => s.to_string(),
        }
    }

    /// `?` → `$n` for Postgres. No SQL here contains a literal `?`.
    fn pg_placeholders(sql: &str) -> String {
        let mut out = String::with_capacity(sql.len() + 8);
        let mut n = 0;
        for ch in sql.chars() {
            if ch == '?' {
                n += 1;
                out.push_str(&format!("${n}"));
            } else {
                out.push(ch);
            }
        }
        out
    }

    enum Conn {
        Lite(PoolConnection<Sqlite>),
        Pg(PoolConnection<Postgres>),
    }

    macro_rules! bind_all {
        ($q:expr, $args:expr) => {{
            let mut q = $q;
            for a in $args {
                q = match a {
                    Val::T(s) => q.bind(s.clone()),
                    Val::I(i) => q.bind(*i),
                    Val::B(b) => q.bind(b.clone()),
                    Val::N('I') => q.bind(Option::<i64>::None),
                    Val::N('B') => q.bind(Option::<Vec<u8>>::None),
                    Val::N(_) => q.bind(Option::<String>::None),
                };
            }
            q
        }};
    }

    /// One cell rendered as a string: `NULL`, the text, the integer, or `b:` and
    /// the bytes.
    macro_rules! cell {
        ($r:expr, $i:expr, $ty:expr) => {{
            use sqlx::Row as _;
            let rendered = match $ty {
                'T' => $r.try_get::<Option<String>, _>($i),
                'I' => $r
                    .try_get::<Option<i64>, _>($i)
                    .map(|v| v.map(|n| n.to_string())),
                'B' => $r
                    .try_get::<Option<Vec<u8>>, _>($i)
                    .map(|v| v.map(|b| format!("b:{}", String::from_utf8_lossy(&b)))),
                other => panic!("unknown type {other}"),
            };
            match rendered {
                Ok(Some(s)) => s,
                Ok(None) => "NULL".to_string(),
                Err(e) => format!("DECODE-ERROR({}): {e}", $ty),
            }
        }};
    }

    impl Conn {
        fn is_pg(&self) -> bool {
            matches!(self, Conn::Pg(_))
        }

        fn dialect(&self) -> &'static str {
            if self.is_pg() { "postgres" } else { "sqlite" }
        }

        async fn exec(&mut self, sql: &str, args: &[Val]) -> Result<u64, String> {
            match self {
                Conn::Lite(c) => bind_all!(sqlx::query(sql), args)
                    .execute(&mut **c)
                    .await
                    .map(|r| r.rows_affected())
                    .map_err(|e| e.to_string()),
                Conn::Pg(c) => {
                    let sql = pg_placeholders(sql);
                    bind_all!(sqlx::query(&sql), args)
                        .execute(&mut **c)
                        .await
                        .map(|r| r.rows_affected())
                        .map_err(|e| e.to_string())
                }
            }
        }

        /// Rows rendered as strings, decoding column `i` as `types[i]`.
        async fn rows(&mut self, sql: &str, args: &[Val], types: &[char]) -> Vec<Vec<String>> {
            match self {
                Conn::Lite(c) => {
                    let rows = bind_all!(sqlx::query(sql), args)
                        .fetch_all(&mut **c)
                        .await
                        .unwrap_or_else(|e| panic!("sqlite: {sql}: {e}"));
                    rows.iter()
                        .map(|r| {
                            types
                                .iter()
                                .enumerate()
                                .map(|(i, t)| cell!(r, i, *t))
                                .collect()
                        })
                        .collect()
                }
                Conn::Pg(c) => {
                    let sql = pg_placeholders(sql);
                    let rows = bind_all!(sqlx::query(&sql), args)
                        .fetch_all(&mut **c)
                        .await
                        .unwrap_or_else(|e| panic!("postgres: {sql}: {e}"));
                    rows.iter()
                        .map(|r| {
                            types
                                .iter()
                                .enumerate()
                                .map(|(i, t)| cell!(r, i, *t))
                                .collect()
                        })
                        .collect()
                }
            }
        }

        async fn count(&mut self, sql: &str, args: &[Val]) -> i64 {
            self.rows(sql, args, &['I']).await[0][0]
                .parse()
                .unwrap_or_else(|e| panic!("{sql}: not a count: {e}"))
        }

        /// Run `stmts` inside a transaction that is always rolled back, and return
        /// the outcome of the LAST statement (earlier ones must succeed), plus
        /// `probe` evaluated as a count inside the same transaction.
        async fn in_rollback(
            &mut self,
            stmts: &[(String, Vec<Val>)],
            probe: Option<&str>,
        ) -> (Result<u64, String>, Option<i64>) {
            self.exec("BEGIN", &[]).await.expect("BEGIN");
            let mut last = Ok(0);
            for (i, (sql, args)) in stmts.iter().enumerate() {
                last = self.exec(sql, args).await;
                if i + 1 < stmts.len()
                    && let Err(e) = &last
                {
                    let _ = self.exec("ROLLBACK", &[]).await;
                    panic!("{}: setup statement failed: {sql}: {e}", self.dialect());
                }
            }
            let probed = match (probe, &last) {
                (Some(p), Ok(_)) => Some(self.count(p, &[]).await),
                _ => None,
            };
            self.exec("ROLLBACK", &[]).await.expect("ROLLBACK");
            (last, probed)
        }
    }

    fn table(name: &str) -> &'static NewTable {
        NEW_TABLES
            .iter()
            .find(|t| t.name == name)
            .unwrap_or_else(|| panic!("no spec for {name}"))
    }

    /// An `INSERT` of the sample row with `overrides`, keeping only the columns
    /// `keep` accepts.
    fn insert_of(
        t: &NewTable,
        overrides: &[(&str, &str)],
        keep: impl Fn(&Col) -> bool,
    ) -> (String, Vec<Val>) {
        let mut names = Vec::new();
        let mut args = Vec::new();
        for c in t.cols.iter().filter(|c| keep(c)) {
            let sample = overrides
                .iter()
                .find(|(n, _)| *n == c.0)
                .map(|(_, v)| *v)
                .unwrap_or(c.3);
            names.push(format!("\"{}\"", c.0));
            args.push(val(c.1, sample));
        }
        if names.is_empty() {
            // Every column has a default (`audit_chain_head`).
            return (format!("INSERT INTO {} DEFAULT VALUES", t.name), args);
        }
        let marks = vec!["?"; names.len()].join(", ");
        (
            format!(
                "INSERT INTO {} ({}) VALUES ({marks})",
                t.name,
                names.join(", ")
            ),
            args,
        )
    }

    fn select_all(t: &NewTable) -> (String, Vec<char>) {
        let cols: Vec<String> = t.cols.iter().map(|c| format!("\"{}\"", c.0)).collect();
        (
            format!("SELECT {} FROM {}", cols.join(", "), t.name),
            t.cols.iter().map(|c| c.1).collect(),
        )
    }

    /// Rows of existing tables the new tables point at.
    const PRELUDE: &[&str] =
        &["INSERT INTO domains (name, upstream_json) VALUES ('example.test', '{}')"];

    // ── schema metadata, per dialect ─────────────────────────────────────────

    type LiveCol = (String, String, bool, Option<String>);

    /// `(name, type, notnull, normalised default)` per column of `table`.
    async fn columns(c: &mut Conn, table: &str) -> Vec<LiveCol> {
        let rows = if c.is_pg() {
            c.rows(
                "SELECT column_name::text, data_type::text, \
                        (CASE WHEN is_nullable = 'NO' THEN 1 ELSE 0 END)::bigint, \
                        column_default::text \
                   FROM information_schema.columns \
                  WHERE table_schema = current_schema() AND table_name = ?",
                &[Val::T(table.to_string())],
                &['T', 'T', 'I', 'T'],
            )
            .await
        } else {
            c.rows(
                &format!(
                    "SELECT name, type, \"notnull\", dflt_value FROM pragma_table_info('{table}')"
                ),
                &[],
                &['T', 'T', 'I', 'T'],
            )
            .await
        };
        rows.into_iter()
            .map(|r| {
                let ty = match r[1].to_ascii_lowercase().as_str() {
                    "text" => "T",
                    "integer" | "bigint" => "I",
                    "blob" | "bytea" => "B",
                    other => panic!("{table}.{}: unexpected type {other}", r[0]),
                }
                .to_string();
                let default = (r[3] != "NULL").then(|| normalise_default(&r[3]));
                (r[0].clone(), ty, r[2] == "1", default)
            })
            .collect()
    }

    /// `'x'::text` (Postgres) and `'x'` (SQLite) both become `x`; `1` stays `1`.
    fn normalise_default(d: &str) -> String {
        let d = d.trim();
        if let Some(rest) = d.strip_prefix('\'') {
            let end = rest.rfind('\'').expect("closing quote");
            rest[..end].replace("''", "'")
        } else {
            d.split("::")
                .next()
                .unwrap()
                .trim_matches(['(', ')'])
                .to_string()
        }
    }

    fn expect_rule(rule: &str) -> (bool, Option<String>) {
        match rule {
            "!" => (true, None),
            "~" => (false, None),
            r if r.starts_with("~=") => (false, Some(r[2..].to_string())),
            r => (true, Some(r.strip_prefix('=').expect("rule").to_string())),
        }
    }

    async fn check_metadata(c: &mut Conn) -> Vec<String> {
        let mut bad = Vec::new();
        let d = c.dialect();
        for t in NEW_TABLES {
            let live = columns(c, t.name).await;
            let mut have: Vec<&str> = live.iter().map(|l| l.0.as_str()).collect();
            let mut want: Vec<&str> = t.cols.iter().map(|c| c.0).collect();
            have.sort_unstable();
            want.sort_unstable();
            if have != want {
                bad.push(format!("{d}: {} columns {have:?}, plan {want:?}", t.name));
                continue;
            }
            for col in t.cols {
                let l = live.iter().find(|l| l.0 == col.0).unwrap();
                let (nn, def) = expect_rule(col.2);
                let got = (l.1.clone(), l.2, l.3.clone());
                let want = (col.1.to_string(), nn, def);
                if got != want {
                    bad.push(format!(
                        "{d}: {}.{} is {got:?}, plan {want:?}",
                        t.name, col.0
                    ));
                }
            }
        }
        for (tname, cols) in ALTERED {
            let live = columns(c, tname).await;
            for (name, ty, rule) in *cols {
                let Some(l) = live.iter().find(|l| l.0 == *name) else {
                    bad.push(format!("{d}: {tname}.{name} missing"));
                    continue;
                };
                let (nn, def) = expect_rule(rule);
                let got = (l.1.clone(), l.2, l.3.clone());
                let want = (ty.to_string(), nn, def);
                if got != want {
                    bad.push(format!("{d}: {tname}.{name} is {got:?}, plan {want:?}"));
                }
            }
        }
        bad
    }

    /// `(unique, partial, columns)` of `index`, or `None` when it is absent.
    async fn index_shape(c: &mut Conn, index: &str, table: &str) -> Option<(bool, bool, String)> {
        if c.is_pg() {
            let head = c
                .rows(
                    "SELECT (CASE WHEN ix.indisunique THEN 1 ELSE 0 END)::bigint, \
                            (CASE WHEN ix.indpred IS NULL THEN 0 ELSE 1 END)::bigint, \
                            t.relname::text \
                       FROM pg_index ix \
                       JOIN pg_class i ON i.oid = ix.indexrelid \
                       JOIN pg_class t ON t.oid = ix.indrelid \
                       JOIN pg_namespace n ON n.oid = i.relnamespace \
                      WHERE i.relname = ? AND n.nspname = current_schema()",
                    &[Val::T(index.to_string())],
                    &['I', 'I', 'T'],
                )
                .await;
            let h = head.first()?;
            if h[2] != table {
                return Some((false, false, format!("on table {}", h[2])));
            }
            let cols = c
                .rows(
                    "SELECT a.attname::text \
                       FROM pg_index ix \
                       JOIN pg_class i ON i.oid = ix.indexrelid \
                       JOIN pg_namespace n ON n.oid = i.relnamespace \
                       CROSS JOIN LATERAL unnest(ix.indkey) WITH ORDINALITY AS k(attnum, ord) \
                       JOIN pg_attribute a ON a.attrelid = ix.indrelid AND a.attnum = k.attnum \
                      WHERE i.relname = ? AND n.nspname = current_schema() \
                      ORDER BY k.ord",
                    &[Val::T(index.to_string())],
                    &['T'],
                )
                .await;
            let cols: Vec<String> = cols.into_iter().map(|r| r[0].clone()).collect();
            Some((h[0] == "1", h[1] == "1", cols.join(",")))
        } else {
            let head = c
                .rows(
                    &format!(
                        "SELECT \"unique\", partial FROM pragma_index_list('{table}') \
                          WHERE name = ?"
                    ),
                    &[Val::T(index.to_string())],
                    &['I', 'I'],
                )
                .await;
            let h = head.first()?;
            let cols = c
                .rows(
                    &format!("SELECT name FROM pragma_index_info('{index}') ORDER BY seqno"),
                    &[],
                    &['T'],
                )
                .await;
            let cols: Vec<String> = cols.into_iter().map(|r| r[0].clone()).collect();
            Some((h[0] == "1", h[1] == "1", cols.join(",")))
        }
    }

    async fn fk_count(c: &mut Conn, table: &str) -> i64 {
        if c.is_pg() {
            c.count(
                "SELECT COUNT(*)::bigint FROM information_schema.table_constraints \
                  WHERE constraint_type = 'FOREIGN KEY' AND table_schema = current_schema() \
                    AND table_name = ?",
                &[Val::T(table.to_string())],
            )
            .await
        } else {
            c.count(
                &format!("SELECT COUNT(*) FROM pragma_foreign_key_list('{table}')"),
                &[],
            )
            .await
        }
    }

    // ── the checks, run against one dialect ──────────────────────────────────

    /// Everything except the `migrate-store` copy. Returns the disagreements with
    /// the plan; empty means this dialect matches it.
    async fn assert_schema(c: &mut Conn) -> Vec<String> {
        let d = c.dialect();
        let mut bad = check_metadata(c).await;

        for (index, tname, unique, partial, cols) in INDEXES {
            match index_shape(c, index, tname).await {
                None => bad.push(format!("{d}: index {index} on {tname} missing")),
                Some(got) => {
                    let want = (*unique, *partial, cols.to_string());
                    if got != want {
                        bad.push(format!("{d}: index {index} is {got:?}, plan {want:?}"));
                    }
                }
            }
        }

        for t in NEW_TABLES {
            let want = FKS.iter().filter(|f| f.0 == t.name).count() as i64;
            let got = fk_count(c, t.name).await;
            if got != want {
                bad.push(format!(
                    "{d}: {} declares {got} foreign keys, plan {want}",
                    t.name
                ));
            }
        }
        if c.is_pg() {
            let not_deferrable = c
                .count(
                    "SELECT COUNT(*)::bigint FROM information_schema.table_constraints \
                      WHERE constraint_type = 'FOREIGN KEY' AND table_schema = current_schema() \
                        AND is_deferrable <> 'YES'",
                    &[],
                )
                .await;
            if not_deferrable != 0 {
                bad.push(format!(
                    "postgres: {not_deferrable} foreign keys are not DEFERRABLE (migrate-store \
                     defers every constraint to commit)"
                ));
            }
        }
        if !bad.is_empty() {
            // The behavioural checks below need the tables to exist.
            return bad;
        }

        // Full-row round trip, parents first.
        for p in PRELUDE {
            c.exec(p, &[])
                .await
                .unwrap_or_else(|e| panic!("{d}: {p}: {e}"));
        }
        for t in NEW_TABLES {
            let (sql, args) = insert_of(t, &[], |_| true);
            if let Err(e) = c.exec(&sql, &args).await {
                bad.push(format!("{d}: full row into {}: {e}", t.name));
                continue;
            }
            let (sel, types) = select_all(t);
            let got = c.rows(&sel, &[], &types).await;
            let want: Vec<String> = t.cols.iter().map(|c| render_sample(c.1, c.3)).collect();
            if got != vec![want.clone()] {
                bad.push(format!(
                    "{d}: {} round trip {got:?}, wrote {want:?}",
                    t.name
                ));
            }
        }
        if !bad.is_empty() {
            return bad;
        }

        // Defaults of a minimal row, and NOT NULL on each required column.
        for t in NEW_TABLES {
            let del = (format!("DELETE FROM {}", t.name), vec![]);
            let minimal = insert_of(t, &[], |c| c.2 == "!");
            let (sel, types) = select_all(t);
            let (res, _) = c.in_rollback(&[del.clone(), minimal.clone()], None).await;
            if let Err(e) = res {
                bad.push(format!("{d}: minimal row into {}: {e}", t.name));
                continue;
            }
            c.exec("BEGIN", &[]).await.unwrap();
            c.exec(&del.0, &[]).await.unwrap();
            c.exec(&minimal.0, &minimal.1).await.unwrap();
            let got = c.rows(&sel, &[], &types).await;
            c.exec("ROLLBACK", &[]).await.unwrap();
            let want: Vec<String> = t
                .cols
                .iter()
                .map(|c| match c.2 {
                    "!" => render_sample(c.1, c.3),
                    "~" => "NULL".to_string(),
                    r if r.starts_with("~=") => r[2..].to_string(),
                    r => r[1..].to_string(),
                })
                .collect();
            if got != vec![want.clone()] {
                bad.push(format!("{d}: {} defaults {got:?}, plan {want:?}", t.name));
            }
            for req in t.cols.iter().filter(|c| c.2 == "!") {
                let without = insert_of(t, &[], |c| c.2 == "!" && c.0 != req.0);
                let (res, _) = c.in_rollback(&[del.clone(), without], None).await;
                if res.is_ok() {
                    bad.push(format!(
                        "{d}: {}.{} accepted a missing value",
                        t.name, req.0
                    ));
                }
            }
        }

        // CHECK constraints.
        for (tname, col, allowed) in CHECKS {
            let t = table(tname);
            let ty = t.cols.iter().find(|c| c.0 == *col).expect("check column").1;
            let del = (format!("DELETE FROM {tname}"), vec![]);
            for v in *allowed {
                let row = insert_of(t, &[(col, v)], |_| true);
                let (res, _) = c.in_rollback(&[del.clone(), row], None).await;
                if let Err(e) = res {
                    bad.push(format!("{d}: {tname}.{col} refused allowed {v:?}: {e}"));
                }
            }
            let outside = if ty == 'I' { "2" } else { "not-a-listed-value" };
            let row = insert_of(t, &[(col, outside)], |_| true);
            let (res, _) = c.in_rollback(&[del.clone(), row], None).await;
            if res.is_ok() {
                bad.push(format!("{d}: {tname}.{col} accepted {outside:?}"));
            }
        }
        for (tname, col, v) in UNCHECKED {
            let t = table(tname);
            let del = (format!("DELETE FROM {tname}"), vec![]);
            let row = insert_of(t, &[(col, v)], |_| true);
            let (res, _) = c.in_rollback(&[del, row], None).await;
            if let Err(e) = res {
                bad.push(format!(
                    "{d}: {tname}.{col} must take any value, refused {v:?}: {e}"
                ));
            }
        }

        // Unique and partial unique indexes.
        for (tname, a, b, conflict) in UNIQUES {
            let t = table(tname);
            let del = (format!("DELETE FROM {tname}"), vec![]);
            let rows = [del, insert_of(t, a, |_| true), insert_of(t, b, |_| true)];
            let (res, _) = c.in_rollback(&rows, None).await;
            if res.is_ok() == *conflict {
                bad.push(format!(
                    "{d}: {tname} {a:?} then {b:?}: expected {}, got {res:?}",
                    if *conflict {
                        "a conflict"
                    } else {
                        "both stored"
                    }
                ));
            }
        }

        // Foreign keys: a missing parent is refused, a deleted parent cascades.
        for (child, ccol, parent, pcol) in FKS {
            let ct = table(child);
            let del = (format!("DELETE FROM {child}"), vec![]);
            let orphan = insert_of(ct, &[(ccol, "no-such-parent")], |_| true);
            let (res, _) = c.in_rollback(&[del, orphan], None).await;
            if res.is_ok() {
                bad.push(format!("{d}: {child}.{ccol} accepted a missing {parent}"));
            }
            let parent_value = ct.cols.iter().find(|c| c.0 == *ccol).unwrap().3;
            let kill = (
                format!("DELETE FROM {parent} WHERE {pcol} = ?"),
                vec![Val::T(parent_value.to_string())],
            );
            let probe = format!("SELECT COUNT(*) FROM {child}");
            let (res, left) = c.in_rollback(&[kill], Some(&probe)).await;
            match (res, left) {
                (Ok(_), Some(0)) => {}
                other => bad.push(format!("{d}: deleting {parent} left {child} at {other:?}")),
            }
        }

        bad.extend(assert_altered_behaviour(c).await);
        bad
    }

    /// Behaviour of the columns added to existing tables that the metadata check
    /// cannot see: the egress kind CHECK, the one-preferred-domain index and the
    /// audit sequence index.
    async fn assert_altered_behaviour(c: &mut Conn) -> Vec<String> {
        let d = c.dialect();
        let mut bad = Vec::new();
        let proxy = |id: &str, kind: &str| {
            (
                "INSERT INTO egress_proxy (id, scheme, host, port, created_at, updated_at, kind) \
                 VALUES (?, 'http', 'proxy.example.test', 3128, 't', 't', ?)"
                    .to_string(),
                vec![Val::T(id.into()), Val::T(kind.into())],
            )
        };
        for (i, k) in ["http", "socks5", "socks5h", "ssh"].iter().enumerate() {
            let (res, _) = c.in_rollback(&[proxy(&format!("p{i}"), k)], None).await;
            if let Err(e) = res {
                bad.push(format!("{d}: egress_proxy.kind refused {k}: {e}"));
            }
        }
        let (res, _) = c.in_rollback(&[proxy("px", "ftp")], None).await;
        if res.is_ok() {
            bad.push(format!("{d}: egress_proxy.kind accepted ftp"));
        }

        let dom = |n: &str, p: i64| {
            (
                "INSERT INTO domains (name, upstream_json, preferred) VALUES (?, '{}', ?)"
                    .to_string(),
                vec![Val::T(n.into()), Val::I(p)],
            )
        };
        let (res, _) = c
            .in_rollback(&[dom("a.test", 1), dom("b.test", 1)], None)
            .await;
        if res.is_ok() {
            bad.push(format!("{d}: two preferred domains were stored"));
        }
        let (res, _) = c
            .in_rollback(&[dom("a.test", 0), dom("b.test", 0)], None)
            .await;
        if let Err(e) = res {
            bad.push(format!("{d}: two non-preferred domains refused: {e}"));
        }

        let audit = |id: &str, seq: Option<i64>| {
            (
                "INSERT INTO audit_log (id, ts, actor, actor_kind, action, detail_json, seq) \
                 VALUES (?, 't', 'root', 'admin', 'x', '{}', ?)"
                    .to_string(),
                vec![Val::T(id.into()), seq.map(Val::I).unwrap_or(Val::N('I'))],
            )
        };
        let (res, _) = c
            .in_rollback(&[audit("a", Some(7)), audit("b", Some(7))], None)
            .await;
        if res.is_ok() {
            bad.push(format!("{d}: two audit rows shared seq 7"));
        }
        let (res, _) = c
            .in_rollback(&[audit("a", None), audit("b", None)], None)
            .await;
        if let Err(e) = res {
            bad.push(format!("{d}: two audit rows without seq refused: {e}"));
        }
        bad
    }

    // ── connections ──────────────────────────────────────────────────────────

    async fn sqlite_conn(path: &str) -> (sqlx::SqlitePool, Conn) {
        let pool = sqlx::sqlite::SqlitePoolOptions::new()
            .max_connections(1)
            .connect(&format!("sqlite://{path}"))
            .await
            .unwrap();
        let conn = Conn::Lite(pool.acquire().await.unwrap());
        (pool, conn)
    }

    async fn pg_conn(dsn: &str) -> (sqlx::PgPool, Conn) {
        let pool = sqlx::postgres::PgPoolOptions::new()
            .max_connections(1)
            .connect(dsn)
            .await
            .unwrap();
        let conn = Conn::Pg(pool.acquire().await.unwrap());
        (pool, conn)
    }

    /// A Postgres schema of its own for one test, so the shared database's other
    /// tests (which TRUNCATE it) and this one cannot see each other's rows.
    /// Returns `(admin pool, schema, scoped dsn)`.
    async fn pg_scratch_schema(dsn: &str, tag: &str) -> (sqlx::PgPool, String, String) {
        let schema = format!(
            "t32_e10a_{tag}_{}_{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        );
        let admin = sqlx::postgres::PgPoolOptions::new()
            .max_connections(1)
            .connect(dsn)
            .await
            .expect("DATABASE_URL_PG is set but Postgres is not reachable");
        sqlx::query(&format!("CREATE SCHEMA {schema}"))
            .execute(&admin)
            .await
            .unwrap();
        let sep = if dsn.contains('?') { '&' } else { '?' };
        let scoped = format!("{dsn}{sep}options=-c%20search_path%3D{schema}");
        (admin, schema, scoped)
    }

    async fn drop_schema(admin: &sqlx::PgPool, schema: &str) {
        let _ = sqlx::query(&format!("DROP SCHEMA {schema} CASCADE"))
            .execute(admin)
            .await;
    }

    fn report(bad: &[String]) {
        assert!(
            bad.is_empty(),
            "migration 0033 disagrees with the plan in {} place(s):\n  - {}",
            bad.len(),
            bad.join("\n  - ")
        );
    }

    #[tokio::test]
    async fn schema_0033_matches_the_plan_on_sqlite() {
        let path = test_db::unique_file_path("mw-store-0033-schema", "s.sqlite");
        let path_str = path.to_string_lossy().to_string();
        drop(Store::open(&path_str, key()).await.unwrap());
        let (pool, mut conn) = sqlite_conn(&path_str).await;
        let bad = assert_schema(&mut conn).await;
        drop(conn);
        pool.close().await;
        remove_sqlite_files(&path_str);
        report(&bad);
        eprintln!(
            "[mw-store] 0033 on SQLite: {} new tables, {} altered tables, {} indexes, {} \
             foreign keys and {} CHECK columns match the plan.",
            NEW_TABLES.len(),
            ALTERED.len(),
            INDEXES.len(),
            FKS.len(),
            CHECKS.len()
        );
    }

    #[tokio::test]
    async fn schema_0033_matches_the_plan_on_postgres() {
        let Some(dsn) = pg_dsn() else {
            super::gate::skip(
                "[mw-store] 0033 schema on Postgres: DATABASE_URL_PG and MW_TEST_PG unset — \
                 the Postgres dialect of migration 0033 is not checked.",
            );
            return;
        };
        let (admin, schema, scoped) = pg_scratch_schema(&dsn, "schema").await;
        let outcome = tokio::spawn(async move {
            drop(Store::open_postgres(&scoped, key()).await.unwrap());
            let (pool, mut conn) = pg_conn(&scoped).await;
            let bad = assert_schema(&mut conn).await;
            drop(conn);
            pool.close().await;
            bad
        })
        .await;
        drop_schema(&admin, &schema).await;
        let bad = outcome.expect("the Postgres leg panicked");
        report(&bad);
        eprintln!("[mw-store] 0033 on Postgres: RAN and matches the plan.");
    }

    /// The destination of a `migrate-store` run from [`seeded_source`]: copied
    /// tables carry the sample row byte for byte, the deliberately left-behind ones
    /// are empty.
    async fn assert_copied(dest: &mut Conn) -> Vec<String> {
        let d = dest.dialect();
        let mut bad = Vec::new();
        for t in NEW_TABLES {
            let (sel, types) = select_all(t);
            let got = dest.rows(&sel, &[], &types).await;
            if t.copied {
                let want: Vec<String> = t.cols.iter().map(|c| render_sample(c.1, c.3)).collect();
                if got != vec![want.clone()] {
                    bad.push(format!(
                        "{d}: migrate-store gave {} {got:?}, source {want:?}",
                        t.name
                    ));
                }
            } else if !got.is_empty() {
                bad.push(format!(
                    "{d}: {} is not copied by design, yet the destination holds {got:?}",
                    t.name
                ));
            }
        }
        bad
    }

    /// A SQLite source holding one sample row in every new table.
    async fn seeded_source(tag: &str) -> String {
        let path = test_db::unique_file_path(tag, "src.sqlite");
        let path_str = path.to_string_lossy().to_string();
        drop(Store::open(&path_str, key()).await.unwrap());
        let (pool, mut c) = sqlite_conn(&path_str).await;
        for p in PRELUDE {
            c.exec(p, &[]).await.unwrap();
        }
        for t in NEW_TABLES {
            let (sql, args) = insert_of(t, &[], |_| true);
            c.exec(&sql, &args)
                .await
                .unwrap_or_else(|e| panic!("seeding {}: {e}", t.name));
        }
        drop(c);
        pool.close().await;
        path_str
    }

    #[tokio::test]
    async fn migrate_store_copies_the_0033_tables_sqlite() {
        let src = seeded_source("mw-store-0033-copy-src").await;
        let dest_path = test_db::unique_file_path("mw-store-0033-copy-dest", "dest.sqlite");
        let dest_str = dest_path.to_string_lossy().to_string();
        let dest = Store::open(&dest_str, key()).await.unwrap();
        dest.migrate_from_sqlite(&src).await.unwrap();
        drop(dest);
        let (pool, mut conn) = sqlite_conn(&dest_str).await;
        let bad = assert_copied(&mut conn).await;
        drop(conn);
        pool.close().await;
        remove_sqlite_files(&src);
        remove_sqlite_files(&dest_str);
        report(&bad);
        eprintln!(
            "[mw-store] migrate-store 0033 (SQLite destination): {} tables copied, {} left \
             behind by design.",
            NEW_TABLES.iter().filter(|t| t.copied).count(),
            NEW_TABLES.iter().filter(|t| !t.copied).count()
        );
    }

    #[tokio::test]
    async fn migrate_store_copies_the_0033_tables_postgres() {
        let Some(dsn) = pg_dsn() else {
            super::gate::skip(
                "[mw-store] migrate-store 0033 to Postgres: DATABASE_URL_PG and MW_TEST_PG \
                 unset — the copy of the 0033 tables into Postgres is not driven.",
            );
            return;
        };
        let src = seeded_source("mw-store-0033-copy-pg-src").await;
        let (admin, schema, scoped) = pg_scratch_schema(&dsn, "copy").await;
        let src2 = src.clone();
        let outcome = tokio::spawn(async move {
            let dest = Store::open_postgres(&scoped, key()).await.unwrap();
            dest.migrate_from_sqlite(&src2).await.unwrap();
            drop(dest);
            let (pool, mut conn) = pg_conn(&scoped).await;
            let bad = assert_copied(&mut conn).await;
            drop(conn);
            pool.close().await;
            bad
        })
        .await;
        drop_schema(&admin, &schema).await;
        remove_sqlite_files(&src);
        let bad = outcome.expect("the Postgres leg panicked");
        report(&bad);
        eprintln!("[mw-store] migrate-store 0033 (Postgres destination): RAN and matched.");
    }

    /// `NOT_MIGRATED_DELIBERATELY` names exactly the new tables marked
    /// `copied: false` here, so the two lists cannot drift apart.
    #[test]
    fn deliberate_list_agrees_with_the_0033_spec() {
        for t in NEW_TABLES {
            let listed = super::NOT_MIGRATED_DELIBERATELY
                .iter()
                .any(|(n, _)| *n == t.name);
            assert_eq!(
                listed, !t.copied,
                "{}: copied={} in the 0033 spec but listed={listed} in NOT_MIGRATED_DELIBERATELY",
                t.name, t.copied
            );
        }
    }
}
