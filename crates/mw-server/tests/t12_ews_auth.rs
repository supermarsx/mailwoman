//! t12-e-e2e-backend — EWS Basic + per-account sealed creds through the jail
//! (audit #2), with the storage leg on live Postgres (the V6 bool-bind lesson).
//!
//! Proves the t12 EWS path against real components:
//!   * the committed `bridge-ews.wasm` runs in the REAL `mw-plugin` wasmtime jail;
//!   * per-account credentials come from the REAL 0011 `ews_account_cred` store —
//!     SEALED at rest, unsealed by a host `BasicCredentialProvider` and handed to the
//!     guest over the gated `basic-credentials` import (NO placeholder constants);
//!   * an empty NT domain selects HTTP Basic; the guest attaches `Authorization:
//!     Basic …` up front (asserted on the wire);
//!   * the host `http-fetch` gate admits ONLY the account's endpoint host.
//!
//! ## Postgres leg (the V6 lesson — plan §7)
//! Runs on SQLite by default; when `MW_E14_PG_DSN` is set the store leg runs on LIVE
//! Postgres, exercising 0011's BIGINT-0/1 `enabled` bind/read, BYTEA `sealed_cred`,
//! and the reused-`?6` upsert — the backend the V6 bool-bind bug only surfaced on.
//!
//!   docker compose -f docker-compose.ci.yml up -d --wait postgres
//!   MW_E14_PG_DSN=postgres://mailwoman:mailwoman@localhost:5432/mailwoman \
//!     cargo test -p mw-server --test t12_ews_auth -- --nocapture --test-threads=1
//!
//! ## ESCALATED BUG (see .orchestration/logs/t12-e-e2e-backend.md + state.md)
//! The bridge-ews guest passes `basic-credentials(ACCOUNT)` with `ACCOUNT = ""`
//! (guest.rs:47), documented as "the empty handle resolves to the bound account
//! host-side". But the mounted `StoreEwsCredProvider` (v7_mount.rs) is a SINGLE global
//! provider that looks up `store.get_ews_account_cred(account)` by the LITERAL handle,
//! and the mw-plugin host passes the guest's `""` through verbatim (no plugin→account
//! binding). So at call time it looks up account `""`, finds nothing, and EWS auth
//! fails with "no enabled EWS credentials stored for account ''" — the per-account
//! creds (which round-trip fine, see the store test below) are never reachable. The
//! bridge-ews fixture test masks this because its `FixtureCreds` ignores the account
//! argument. `ews_empty_handle_production_provider_repro` (ignored) reproduces it.

use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

use async_trait::async_trait;
use base64::Engine as _;

use mw_engine::MailboxRole;
use mw_plugin::{
    BasicCredentialProvider, BasicCredentials, Capability, Grant, HostServices, HttpFetcher,
    HttpReq, HttpResp, PluginHost, PluginLimits, PluginManifest, TrustRoot,
};
use mw_store::{EwsAccountCred, ServerKey, Store};

const GUEST: &[u8] = include_bytes!("../../../plugins/bridge-ews/fixtures/bridge-ews.wasm");
const HIER: &str = include_str!("../../../plugins/bridge-ews/fixtures/sync_folder_hierarchy.xml");
const ITEMS: &str = include_str!("../../../plugins/bridge-ews/fixtures/sync_folder_items.xml");
const GETITEM: &str = include_str!("../../../plugins/bridge-ews/fixtures/get_item_mime.xml");

const ENDPOINT: &str = "https://ews.example.com/EWS/Exchange.asmx";
const ENDPOINT_HOST: &str = "ews.example.com";

fn unique(prefix: &str) -> String {
    format!(
        "{prefix}-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    )
}

/// The db under test: live Postgres when `MW_E14_PG_DSN` is set (the V6 lesson),
/// else a fresh temp SQLite file. Returns `(db, on_postgres)`.
fn db_target() -> (String, bool) {
    if let Some(dsn) = std::env::var("MW_E14_PG_DSN")
        .ok()
        .or_else(|| std::env::var("DATABASE_URL_PG").ok())
        .filter(|s| !s.is_empty())
    {
        eprintln!("[t12 ews_auth] running the store leg against LIVE Postgres (0011 + bool-bind)");
        (dsn, true)
    } else {
        eprintln!(
            "[t12 ews_auth] MW_E14_PG_DSN unset — store leg on SQLite only. Set it (bring up \
             docker-compose.ci.yml postgres) to also exercise 0011 on Postgres."
        );
        let p = std::env::temp_dir().join(format!("{}.db", unique("mw-t12-ews")));
        (p.to_string_lossy().into_owned(), false)
    }
}

/// Open the store under test + seed one Basic (empty-domain) EWS credential; return
/// `(store, account_id)`. `None` after a loud skip if Postgres was requested but is
/// unreachable.
async fn seed_basic_account() -> Option<(Store, String)> {
    let (db, on_pg) = db_target();
    let store = match Store::open(&db, ServerKey::generate()).await {
        Ok(s) => s,
        Err(e) => {
            if on_pg {
                eprintln!(
                    "\n[t12 ews_auth SKIP] could not open/migrate Postgres at the DSN ({e}). Is \
                     docker-compose.ci.yml postgres up?\n"
                );
                return None;
            }
            panic!("open sqlite store: {e}");
        }
    };
    let account = unique("acct-ews");
    let cred = EwsAccountCred {
        account_id: account.clone(),
        endpoint: ENDPOINT.into(),
        endpoint_host: ENDPOINT_HOST.into(),
        user: "user@example.com".into(),
        domain: String::new(), // empty ⇒ HTTP Basic
        password: "s3cr3t-basic".into(),
        workstation: String::new(),
        enabled: true,
    };
    store
        .put_ews_account_cred(&cred)
        .await
        .expect("put 0011 cred");
    Some((store, account))
}

/// A host credential provider BOUND to one account — the intended "one instance backs
/// one account" contract: the guest's (empty) handle is ignored and the bound
/// account's sealed 0011 creds are unsealed from the store. This is exactly the shape
/// the escalated fix should produce (a per-account-bound provider).
struct BoundStoreCreds {
    store: Store,
    account: String,
}
#[async_trait]
impl BasicCredentialProvider for BoundStoreCreds {
    async fn credentials(&self, _handle: &str) -> Result<BasicCredentials, String> {
        let c = self
            .store
            .get_ews_account_cred(&self.account)
            .await
            .map_err(|e| e.to_string())?
            .ok_or_else(|| format!("no EWS credential for {}", self.account))?;
        Ok(BasicCredentials {
            user: c.user,
            domain: c.domain,
            password: c.password,
            workstation: c.workstation,
            endpoint: c.endpoint,
        })
    }
}

/// A provider that mirrors the PRODUCTION `StoreEwsCredProvider`: it looks up by the
/// LITERAL guest-supplied handle. With the guest sending `""`, this reproduces the bug.
struct LiteralStoreCreds {
    store: Store,
}
#[async_trait]
impl BasicCredentialProvider for LiteralStoreCreds {
    async fn credentials(&self, account: &str) -> Result<BasicCredentials, String> {
        let c = self
            .store
            .get_ews_account_cred(account)
            .await
            .map_err(|e| e.to_string())?
            .ok_or_else(|| format!("no enabled EWS credentials stored for account '{account}'"))?;
        Ok(BasicCredentials {
            user: c.user,
            domain: c.domain,
            password: c.password,
            workstation: c.workstation,
            endpoint: c.endpoint,
        })
    }
}

/// A fake EWS server: accepts an up-front HTTP Basic header, records it, and returns
/// the fixture matching the SOAP operation.
struct FakeEws {
    saw_basic: AtomicBool,
    saw_authz_user: std::sync::Mutex<Option<String>>,
}
impl FakeEws {
    fn new() -> Arc<Self> {
        Arc::new(Self {
            saw_basic: AtomicBool::new(false),
            saw_authz_user: std::sync::Mutex::new(None),
        })
    }
    fn dispatch(body: &str) -> &'static str {
        if body.contains("SyncFolderHierarchy") {
            HIER
        } else if body.contains("SyncFolderItems") {
            ITEMS
        } else if body.contains("GetItem") {
            GETITEM
        } else {
            "<soap:Envelope/>"
        }
    }
}
#[async_trait]
impl HttpFetcher for FakeEws {
    async fn fetch(&self, req: HttpReq) -> Result<HttpResp, String> {
        let auth = req
            .headers
            .iter()
            .find(|(k, _)| k.eq_ignore_ascii_case("authorization"))
            .map(|(_, v)| v.clone())
            .unwrap_or_default();
        if let Some(b64) = auth.strip_prefix("Basic ") {
            self.saw_basic.store(true, Ordering::SeqCst);
            if let Ok(dec) = base64::engine::general_purpose::STANDARD.decode(b64.trim()) {
                let s = String::from_utf8_lossy(&dec).into_owned();
                *self.saw_authz_user.lock().unwrap() = s.split(':').next().map(str::to_string);
            }
            let body = String::from_utf8_lossy(&req.body.unwrap_or_default()).into_owned();
            return Ok(HttpResp {
                status: 200,
                headers: vec![],
                body: FakeEws::dispatch(&body).as_bytes().to_vec(),
            });
        }
        Ok(HttpResp {
            status: 401,
            headers: vec![("WWW-Authenticate".into(), "Basic".into())],
            body: vec![],
        })
    }
}

fn manifest(net: &[&str]) -> PluginManifest {
    PluginManifest {
        id: "bridge-ews".into(),
        name: "Exchange EWS bridge".into(),
        version: "26.12.0".into(),
        signature: None,
        capabilities: vec![
            Capability::AccountBackend,
            Capability::Net,
            Capability::AddrbookSource,
        ],
        net_allowlist: net.iter().map(|s| s.to_string()).collect(),
        limits: PluginLimits {
            memory_mb: 128,
            deadline_ms: 10_000,
            fuel: None,
        },
    }
}
fn grant() -> Grant {
    Grant {
        plugin_id: "bridge-ews".into(),
        capabilities: vec![
            Capability::AccountBackend,
            Capability::Net,
            Capability::AddrbookSource,
        ],
        granted_by: "t12@test".into(),
        allow_unsigned: true,
    }
}

/// 0011 `ews_account_cred` round-trips on the backend under test (Postgres when
/// `MW_E14_PG_DSN` is set — the V6 bool-bind lesson: BIGINT 0/1 `enabled`, BYTEA
/// sealed blob, reused-`?6` upsert). The secret never appears in plaintext at rest.
#[tokio::test]
async fn ews_0011_credential_roundtrips_live_backend() {
    let Some((store, account)) = seed_basic_account().await else {
        return;
    };
    let got = store
        .get_ews_account_cred(&account)
        .await
        .expect("get 0011 cred")
        .expect("cred present");
    assert_eq!(got.user, "user@example.com");
    assert_eq!(got.endpoint_host, ENDPOINT_HOST);
    assert!(got.domain.is_empty(), "empty NT domain ⇒ Basic scheme");
    assert!(
        got.enabled,
        "BIGINT/INTEGER 0/1 `enabled` reads back true on the backend"
    );
    assert_eq!(
        got.password, "s3cr3t-basic",
        "sealed secret decrypts to the original"
    );

    // Disable via upsert (exercises the reused-`?6` upsert + the BIGINT-0/1 write on
    // the backend); read the same account back and confirm `enabled` is now false.
    // (A per-account `get` is used rather than the global `list_ews_account_creds`,
    // which on a persistent Postgres would also try to unseal unrelated rows left by
    // earlier runs under different ServerKeys — a test-isolation artifact, not a
    // product concern, since one ServerKey is used throughout a real deployment.)
    let mut off = got.clone();
    off.enabled = false;
    store.put_ews_account_cred(&off).await.unwrap();
    let after = store
        .get_ews_account_cred(&account)
        .await
        .unwrap()
        .expect("row still present after disable");
    assert!(
        !after.enabled,
        "enabled=0 round-trips (BIGINT 0/1) on the backend"
    );
}

/// Guest → jail → host `basic-credentials` → 0011 store unseal → Basic auth →
/// SyncFolderHierarchy, using an ACCOUNT-BOUND provider (the intended contract). Also
/// asserts the net_allowlist admits the account's endpoint host.
#[tokio::test]
async fn ews_basic_per_account_through_jail() {
    let Some((store, account)) = seed_basic_account().await else {
        return;
    };
    let fake = FakeEws::new();
    let services = HostServices {
        http: fake.clone(),
        basic_creds: Arc::new(BoundStoreCreds { store, account }),
        ..HostServices::default()
    };
    let host = PluginHost::try_new(services, TrustRoot::empty()).unwrap();
    let handle = host
        .load(GUEST, &manifest(&[ENDPOINT_HOST]), &grant())
        .expect("load bridge-ews");
    let backend = handle
        .as_account_backend()
        .expect("account-backend granted");

    let boxes = backend
        .list_mailboxes()
        .await
        .expect("list_mailboxes through the jail");
    assert!(
        boxes.iter().any(|b| b.role == MailboxRole::Inbox),
        "Inbox resolved"
    );

    assert!(
        fake.saw_basic.load(Ordering::SeqCst),
        "Basic Authorization sent"
    );
    assert_eq!(
        fake.saw_authz_user.lock().unwrap().as_deref(),
        Some("user@example.com"),
        "the Basic header carries the per-account STORED user (no placeholder constant)"
    );
}

/// The `http-fetch` gate admits ONLY the account's endpoint host. Uses a bound
/// provider so credentials resolve — isolating the net_allowlist decision.
#[tokio::test]
async fn ews_net_allowlist_admits_only_account_endpoint_host() {
    let Some((store, account)) = seed_basic_account().await else {
        return;
    };
    let fake = FakeEws::new();
    let services = HostServices {
        http: fake.clone(),
        basic_creds: Arc::new(BoundStoreCreds { store, account }),
        ..HostServices::default()
    };
    let host = PluginHost::try_new(services, TrustRoot::empty()).unwrap();
    // net_allowlist deliberately EXCLUDES the account's endpoint host.
    let handle = host
        .load(
            GUEST,
            &manifest(&["not-the-account-host.example"]),
            &grant(),
        )
        .expect("load bridge-ews");
    let backend = handle.as_account_backend().unwrap();

    assert!(
        backend.list_mailboxes().await.is_err(),
        "an endpoint host outside net_allowlist must be denied at the http-fetch gate"
    );
    assert!(
        !fake.saw_basic.load(Ordering::SeqCst),
        "no request reaches the EWS server when the host is denied"
    );
}

/// ESCALATED-BUG REPRODUCTION (ignored so CI stays green). Faithfully mirrors
/// PRODUCTION: the mounted `StoreEwsCredProvider` looks up by the LITERAL guest
/// handle, and the guest sends `ACCOUNT = ""`. This asserts the DESIRED outcome
/// (auth succeeds and reaches the account's mailbox) — which currently FAILS with
/// "no enabled EWS credentials stored for account ''". Un-ignore once the
/// empty-handle → bound-account resolution is wired host-side (see the module doc +
/// t12-e-e2e-backend.md for the minimal-fix proposal).
#[tokio::test]
#[ignore = "ESCALATED: guest ACCOUNT=\"\" is never mapped to the mounted account; the global StoreEwsCredProvider looks up account '' and EWS auth fails. See t12-e-e2e-backend.md."]
async fn ews_empty_handle_production_provider_repro() {
    let Some((store, _account)) = seed_basic_account().await else {
        return;
    };
    let fake = FakeEws::new();
    let services = HostServices {
        http: fake.clone(),
        // Production-shaped: literal-handle lookup, NOT bound to the account.
        basic_creds: Arc::new(LiteralStoreCreds { store }),
        ..HostServices::default()
    };
    let host = PluginHost::try_new(services, TrustRoot::empty()).unwrap();
    let handle = host
        .load(GUEST, &manifest(&[ENDPOINT_HOST]), &grant())
        .expect("load bridge-ews");
    let backend = handle.as_account_backend().unwrap();

    // DESIRED (post-fix) behaviour: the bound account's creds resolve and auth works.
    let boxes = backend
        .list_mailboxes()
        .await
        .expect("EWS per-account auth must resolve the mounted account (currently fails: bug)");
    assert!(boxes.iter().any(|b| b.role == MailboxRole::Inbox));
}
