//! t16-e-e2e — route-wiring proofs: inbound webhook (A5), MSG/DOCX export (W7),
//! attachment-text search (W19), + documented coverage for the engine-mode legs.
//!
//! "unit-green != wired." These drive the actual shipped seams:
//!   * A5 — a signed inbound webhook, through the REAL `/api/webhooks/inbound` route
//!     (`build_app`), fires the wired action sink (a bad signature is refused).
//!   * W7 — the MSG (CFB) + DOCX (OOXML/ZIP) export transform the export route serves
//!     (`mw_export::export_one`) produces the right container bytes from a real message.
//!   * W19 — decoded attachment text is genuinely searchable: a term present ONLY in an
//!     attachment matches a full-text query over the real `mw-search` index.
//!
//! Engine-mode legs that need a logged-in IMAP account (mbox/EML/Maildir import
//! round-trip; webcal subscribe; server-side prefs persistence) require an engine-mode
//! server + a real IMAP backend (Dovecot) and a session, which is outside this file's
//! in-process reach; see the LOUD-NOTES below for where each is covered. The webcal
//! subscribe fetch reuses e6's SSRF-hardened fetcher (`validate_and_resolve`) — the
//! EXACT gate proven to refuse loopback/metadata LIVE in `t16_image_proxy`.

use std::sync::Arc;

use serde_json::json;

use mw_server::webhooks::{
    self, InMemoryWebhookActionSink, SIGNATURE_HEADER, WebhookActionSink, set_inbound_dispatcher,
    set_inbound_secret,
};
use mw_server::{AppConfig, build_app};

fn unique() -> String {
    static SEQ: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
    format!(
        "{}-{}",
        std::process::id(),
        SEQ.fetch_add(1, std::sync::atomic::Ordering::Relaxed)
    )
}

async fn spawn_server() -> String {
    let base = std::env::temp_dir().join(format!("mw-t16-routes-{}", unique()));
    let web = base.join("web");
    std::fs::create_dir_all(&web).unwrap();
    std::fs::write(
        web.join("index.html"),
        "<!doctype html><div id=app>MW</div>",
    )
    .unwrap();
    let config = AppConfig {
        db_path: base.join("mw.db").to_string_lossy().into_owned(),
        server_key_hex: None,
        web_dir: Some(web),
        cookie_secure: false,
        mode: mw_server::ServerMode::Proxy,
        hardening: mw_server::HardeningConfig::default(),
        security: mw_server::SecurityConfig::default(),
    };
    let app = build_app(config).await.unwrap();
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    tokio::spawn(async move {
        axum::serve(listener, app).await.unwrap();
    });
    format!("http://{addr}")
}

// ── A5: inbound webhook fires a wired rule action (LIVE via build_app) ────────

#[tokio::test]
async fn inbound_webhook_fires_the_wired_action_sink() {
    // The inbound dispatcher + secret are process-global; this test owns them, so run
    // it single-threaded (the conformance job passes --test-threads=1).
    let base = spawn_server().await;

    let secret = b"t16-inbound-secret".to_vec();
    set_inbound_secret(Some(secret.clone()));
    let sink = Arc::new(InMemoryWebhookActionSink::new());
    set_inbound_dispatcher(Some(sink.clone() as Arc<dyn WebhookActionSink>));

    let payload = json!({
        "account": "alice@example.org",
        "action": {
            "kind": "run_rules",
            "envelope": {
                "from": "boss@example.org",
                "to": "alice@example.org",
                "subject": "quarterly numbers",
            }
        }
    });
    let body = serde_json::to_vec(&payload).unwrap();
    let signature = webhooks::sign(&secret, &body);
    let c = reqwest::Client::new();

    // A correctly-signed inbound webhook is accepted and DISPATCHED to the sink.
    let resp = c
        .post(format!("{base}/api/webhooks/inbound"))
        .header(SIGNATURE_HEADER, &signature)
        .header("content-type", "application/json")
        .body(body.clone())
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 202, "signed inbound webhook accepted");
    let rbody: serde_json::Value = resp.json().await.unwrap();
    assert_eq!(
        rbody["dispatched"],
        json!(true),
        "the action was dispatched: {rbody}"
    );

    let fired = sink.dispatched();
    assert_eq!(fired.len(), 1, "exactly one action fired");
    assert_eq!(fired[0].0, "alice@example.org", "for the right account");

    // A BAD signature is refused (never reaches the sink).
    let bad = c
        .post(format!("{base}/api/webhooks/inbound"))
        .header(SIGNATURE_HEADER, "sha256=deadbeef")
        .header("content-type", "application/json")
        .body(body)
        .send()
        .await
        .unwrap();
    assert_eq!(bad.status(), 401, "an invalid signature is rejected");
    assert_eq!(
        sink.dispatched().len(),
        1,
        "the rejected call did not fire the sink"
    );

    // Clean up the process-global wiring for any peer test.
    set_inbound_dispatcher(None);
    set_inbound_secret(None);
}

// ── W7: the MSG (CFB) + DOCX (OOXML) export transform the route serves ────────

#[test]
fn export_transform_emits_msg_and_docx_containers() {
    let raw =
        b"Message-ID: <w7@x>\r\nFrom: a@x\r\nTo: b@x\r\nSubject: W7 export\r\n\r\nhello world\r\n"
            .to_vec();
    let email = mw_export::RawEmail::new(raw);

    // .msg is a Microsoft Compound File Binary (OLE2) — magic D0 CF 11 E0 A1 B1 1A E1.
    let msg = mw_export::export_one(&email, mw_export::Format::Msg).expect("to_msg");
    assert_eq!(
        &msg[..8],
        &[0xD0, 0xCF, 0x11, 0xE0, 0xA1, 0xB1, 0x1A, 0xE1],
        ".msg is a CFB/OLE2 container"
    );

    // .docx is Office Open XML — a ZIP (magic PK\x03\x04).
    let docx = mw_export::export_one(&email, mw_export::Format::Docx).expect("to_docx");
    assert_eq!(&docx[..4], b"PK\x03\x04", ".docx is a ZIP/OOXML container");

    // .eml round-trips too (the existing allow-list, sanity).
    let eml = mw_export::export_one(&email, mw_export::Format::Eml).expect("to_eml");
    assert!(
        String::from_utf8_lossy(&eml).contains("Subject: W7 export"),
        ".eml carries the message"
    );
}

// ── W19: decoded attachment text is searchable ───────────────────────────────

#[test]
fn attachment_text_is_full_text_searchable() {
    let idx = mw_search::Index::open_in_ram().expect("open index");
    let doc = mw_search::IndexDoc {
        stable_id: "m-w19".into(),
        account_id: "acct".into(),
        mailbox_id: "mb".into(),
        from: "biller@example.com".into(),
        to: "me@example.com".into(),
        cc: String::new(),
        subject: "Invoice".into(),
        body: "see attached".into(),
        date: 0,
        has_attachment: true,
        keywords: Vec::new(),
        size: 100,
        filenames: vec!["invoice.txt".into()],
        pinned: false,
        // A term that exists ONLY in the attachment (W19).
        attachment_text: "netsuite purchaseorder 4471 grandtotal".into(),
    };
    idx.upsert(&doc).expect("index");

    let hits = |q: &str| -> Vec<String> {
        let parsed = mw_search::parse_query(q).expect("parse");
        idx.search(&parsed, 10).expect("search")
    };

    // The attachment-only term matches (W19 wiring: attachment_text → body field).
    assert_eq!(
        hits("purchaseorder"),
        vec!["m-w19".to_string()],
        "attachment term is searchable"
    );
    assert_eq!(
        hits("has:attachment"),
        vec!["m-w19".to_string()],
        "has:attachment matches"
    );
    assert!(
        hits("nonexistentterm").is_empty(),
        "a non-matching term returns nothing"
    );
}

// ── documented coverage for the engine-mode legs ─────────────────────────────

#[test]
fn engine_mode_route_legs_coverage_note() {
    eprintln!(
        "\n[t16 routes NOTE] engine-mode route legs (need an engine-mode server + a real\n\
         IMAP backend + a session), covered as follows:\n\
         * mbox/EML/Maildir import round-trip — `import_routes.rs` over the shipped\n\
           `mw-mbox::split_mbox` parser; unit-covered in the import lane.\n\
         * webcal subscribe fetch + SSRF-block — the webcal driver reuses e6's\n\
           `image_proxy::fetch_url_hardened` → `validate_and_resolve`, the SAME gate\n\
           this lane proves LIVE-refuses loopback/metadata in `t16_image_proxy`.\n\
         * server-side prefs persistence (signatures/notifications/saved-searches/\n\
           identities) — `prefs_routes.rs` over the 0017/0003 store methods; unit-covered\n\
           in the e18 lane. The 0017 SQL is exercised on live Postgres by the other\n\
           t16 store round-trips.\n"
    );
}
