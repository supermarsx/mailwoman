//! t39-e7 (L11): a vacation reply is accepted only where something sends it.
//!
//! - With a Sieve context and the `vacation` extension allowed, the reply is
//!   written into the active Sieve script: uploaded to ManageSieve (live
//!   Pigeonhole below, `MW_SIEVE_JMAP_LIVE=host:port`) or run by the engine at
//!   delivery (`tests/engine.rs` `local_vacation_submits_once_…`).
//! - With the administrator's `sieve.vacationAllowed` off, the session
//!   capability does not list `vacation` (the web client hides the tab) and an
//!   enabling `set` is refused, nothing stored.
//! - Without a Sieve context (MCP and other in-process callers) the enabling
//!   `set` is refused (`mail_ext/tests.rs`).
use mw_engine::{Engine, sieve_jmap::SieveContext};
use mw_sieve::policy::Policy;
use mw_store::{ServerKey, Store};
use serde_json::{Value, json};

async fn engine() -> Engine {
    Engine::new(
        Store::open_in_memory(ServerKey::generate())
            .await
            .unwrap()
            .with_upload_backend(std::sync::Arc::new(MemoryUploads::default())),
    )
}

#[derive(Default)]
struct MemoryUploads(std::sync::Mutex<std::collections::HashMap<(String, String), Vec<u8>>>);
#[async_trait::async_trait]
impl mw_store::UploadBackend for MemoryUploads {
    fn kind(&self) -> &'static str {
        "memory-test"
    }
    async fn put(&self, a: &str, k: &str, v: Vec<u8>) -> Result<(), mw_store::UploadError> {
        self.0.lock().unwrap().insert((a.into(), k.into()), v);
        Ok(())
    }
    async fn get(&self, a: &str, k: &str) -> Result<Option<Vec<u8>>, mw_store::UploadError> {
        Ok(self.0.lock().unwrap().get(&(a.into(), k.into())).cloned())
    }
    async fn delete(&self, a: &str, k: &str) -> Result<(), mw_store::UploadError> {
        self.0.lock().unwrap().remove(&(a.into(), k.into()));
        Ok(())
    }
}

fn context(vacation_allowed: bool, client: Option<mw_sieve::ManageSieveClient>) -> SieveContext {
    SieveContext::new(
        Policy {
            vacation_allowed,
            max_scripts: Some(5),
            max_script_bytes: Some(65536),
            ..Default::default()
        },
        client,
    )
}

async fn call(e: &Engine, c: &mut SieveContext, name: &str, args: Value) -> Value {
    e.handle_jmap_with_sieve(
        "alice",
        &json!({"using":["urn:ietf:params:jmap:core","urn:ietf:params:jmap:vacationresponse"],"methodCalls":[[name,args,"c"]]}),
        Some(c),
    )
    .await["methodResponses"][0]
        .clone()
}

const ENABLE: &str = r#"{"accountId":"alice","update":{"singleton":{"isEnabled":true,"subject":"Away","textBody":"Back soon"}}}"#;

fn extensions(c: &SieveContext) -> Vec<String> {
    c.account_capability()["sieveExtensions"]
        .as_array()
        .unwrap()
        .iter()
        .filter_map(|v| v.as_str().map(str::to_owned))
        .collect()
}

#[tokio::test]
async fn vacation_disallowed_is_not_advertised_and_an_enabling_set_is_refused() {
    let e = engine().await;
    let mut c = context(false, None);
    assert!(!extensions(&c).iter().any(|x| x == "vacation"));
    let set = call(
        &e,
        &mut c,
        "VacationResponse/set",
        serde_json::from_str(ENABLE).unwrap(),
    )
    .await;
    assert!(set[1]["updated"].get("singleton").is_none(), "{set}");
    assert!(set[1]["notUpdated"].get("singleton").is_some(), "{set}");
    assert!(
        e.store()
            .list_sieve_scripts("alice")
            .await
            .unwrap()
            .is_empty()
    );
    let get = call(
        &e,
        &mut c,
        "VacationResponse/get",
        json!({"accountId":"alice"}),
    )
    .await;
    assert_eq!(get[1]["list"][0]["isEnabled"], false);
}

#[tokio::test]
async fn vacation_allowed_without_managesieve_becomes_an_engine_run_script() {
    let e = engine().await;
    let mut c = context(true, None);
    assert!(extensions(&c).iter().any(|x| x == "vacation"));
    assert_eq!(
        c.account_capability()["urn:mailwoman:policy"]["runsAt"],
        "engine"
    );
    let set = call(
        &e,
        &mut c,
        "VacationResponse/set",
        serde_json::from_str(ENABLE).unwrap(),
    )
    .await;
    assert!(set[1]["updated"].get("singleton").is_some(), "{set}");
    let scripts = e.store().list_sieve_scripts("alice").await.unwrap();
    let active = scripts.iter().find(|s| s.active).expect("an active script");
    assert!(
        active.content.contains("vacation :days 7"),
        "{}",
        active.content
    );
}

#[tokio::test]
async fn live_pigeonhole_installs_the_vacation_script() {
    let Ok(target) = std::env::var("MW_SIEVE_JMAP_LIVE") else {
        eprintln!("MW_SIEVE_JMAP_LIVE not set; skipping live Pigeonhole");
        return;
    };
    let (host, port) = target.rsplit_once(':').unwrap();
    let connect = || async {
        mw_sieve::ManageSieveClient::connect(
            host,
            port.parse().unwrap(),
            mw_sieve::TlsMode::Plaintext,
            mw_sieve::Credentials::Plain {
                username: "testuser".into(),
                password: "testpass".into(),
            },
        )
        .await
        .unwrap()
    };
    let e = engine().await;
    let mut c = context(true, Some(connect().await));
    assert_eq!(
        c.account_capability()["urn:mailwoman:policy"]["runsAt"],
        "server-sieve"
    );
    assert!(
        extensions(&c).iter().any(|x| x == "vacation"),
        "Pigeonhole advertises vacation: {:?}",
        extensions(&c)
    );
    let set = call(
        &e,
        &mut c,
        "VacationResponse/set",
        serde_json::from_str(ENABLE).unwrap(),
    )
    .await;
    assert!(set[1]["updated"].get("singleton").is_some(), "{set}");

    // The reply is on the server: Pigeonhole compiled and activated it.
    let mut check = connect().await;
    let listed = check.list_scripts().await.unwrap();
    eprintln!("scripts on Pigeonhole: {listed:?}");
    let active = listed
        .iter()
        .find(|s| s.active)
        .expect("an active server script");
    let source = check.get_script(&active.name).await.unwrap();
    assert!(source.contains("vacation :days 7"), "{source}");
    assert!(source.contains("Back soon"), "{source}");

    // Turning it off removes the action from the server script.
    let off = call(
        &e,
        &mut c,
        "VacationResponse/set",
        json!({"accountId":"alice","update":{"singleton":{"isEnabled":false}}}),
    )
    .await;
    assert!(off[1]["updated"].get("singleton").is_some(), "{off}");
    let source = connect().await.get_script(&active.name).await.unwrap();
    assert!(!source.contains("vacation :days"), "{source}");

    // Clean up so the fixture account is reusable.
    let mut cleanup = connect().await;
    let _ = cleanup.set_active("").await;
    for s in cleanup.list_scripts().await.unwrap() {
        let _ = cleanup.delete_script(&s.name).await;
    }
}
