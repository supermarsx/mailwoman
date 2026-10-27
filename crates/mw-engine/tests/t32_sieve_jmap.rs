//! RFC9661 through the public engine envelope; no mail backend is needed for scripts.
use mw_engine::{Engine, sieve_jmap::SieveContext};
use mw_sieve::policy::Policy;
use mw_store::{ServerKey, Store};
use serde_json::{Value, json};
async fn setup() -> (Engine, SieveContext) {
    let engine = Engine::new(
        Store::open_in_memory(ServerKey::generate())
            .await
            .unwrap()
            .with_upload_backend(std::sync::Arc::new(MemoryUploads::default())),
    );
    let context = SieveContext::new(
        Policy {
            raw_edit: true,
            own_domains: vec!["example.test".into()],
            max_scripts: Some(2),
            max_script_bytes: Some(1024),
            max_redirects: Some(1),
            ..Default::default()
        },
        None,
    );
    (engine, context)
}
async fn call(e: &Engine, c: &mut SieveContext, name: &str, args: Value) -> Value {
    e.handle_jmap_with_sieve("alice",&json!({"using":["urn:ietf:params:jmap:core","urn:ietf:params:jmap:sieve"],"methodCalls":[[name,args,"c"]]}),Some(c)).await["methodResponses"][0].clone()
}

#[tokio::test]
async fn builder_edits_preserve_manual_source_without_raw_edit_authority() {
    let (e, mut c) = setup().await;
    let handwritten =
        "# hand written, preserve these bytes\r\nif header :is \"X-Manual\" \"yes\" { keep; }\r\n";
    let blob = e
        .store_upload("alice", "application/sieve", handwritten.as_bytes())
        .await
        .unwrap();
    let made = call(&e, &mut c, "SieveScript/set", json!({"accountId":"alice","create":{"a":{"name":"manual","blobId":blob}},"onSuccessActivateScript":"#a"})).await;
    let script = made[1]["created"]["a"]["id"].as_str().unwrap().to_string();
    c.policy.raw_edit = false;
    let made = call(&e, &mut c, "MailRule/set", json!({"accountId":"alice","create":{"a":{"name":"News","conditions":[{"type":"from","op":"contains","value":"news@example.test"}],"actions":[{"type":"move","value":"News"}]}}})).await;
    assert_eq!(made[0], "MailRule/set", "{made}");
    let rule = made[1]["created"]["a"]["id"].as_str().unwrap();
    let rows = e.store().list_sieve_scripts("alice").await.unwrap();
    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0].id(), script);
    assert!(rows[0].active);
    assert_eq!(
        mw_sieve::managed::hand_written(&rows[0].content).unwrap(),
        handwritten
    );
    assert_eq!(
        mw_sieve::managed::rules(&rows[0].content).unwrap()[0].id,
        rule
    );
    let old_state = made[1]["newState"].clone();
    let denied = call(
        &e,
        &mut c,
        "SieveScript/set",
        json!({"accountId":"alice","update":{script.clone():{"blobId":blob}}}),
    )
    .await;
    assert_eq!(denied[1]["notUpdated"][&script]["type"], "forbidden");
    let stale = call(
        &e,
        &mut c,
        "MailRule/set",
        json!({"accountId":"alice","ifInState":"stale","destroy":[rule]}),
    )
    .await;
    assert_eq!(stale[1]["type"], "stateMismatch");
    let got = call(&e, &mut c, "MailRule/get", json!({"accountId":"alice"})).await;
    assert_eq!(got[1]["state"], old_state);
    assert_eq!(got[1]["list"][0]["id"], rule);
    call(
        &e,
        &mut c,
        "SieveScript/set",
        json!({"accountId":"alice","onSuccessDeactivateScript":true}),
    )
    .await;
    assert!(
        e.get_rules("alice").await.unwrap().is_empty(),
        "deactivation must not resurrect the legacy JSON mirror"
    );
    call(
        &e,
        &mut c,
        "SieveScript/set",
        json!({"accountId":"alice","destroy":[script]}),
    )
    .await;
    assert!(
        e.get_rules("alice").await.unwrap().is_empty(),
        "deleting the last script must not resurrect old rules"
    );
}

#[tokio::test]
async fn builder_policy_failure_does_not_persist_or_activate() {
    let (e, mut c) = setup().await;
    c.policy.raw_edit = false;
    c.policy.max_script_bytes = Some(1);
    let failed = call(&e, &mut c, "MailRule/set", json!({"accountId":"alice","create":{"a":{"name":"News","conditions":[{"type":"from","op":"contains","value":"news@example.test"}],"actions":[{"type":"move","value":"News"}]}}})).await;
    assert_eq!(failed[1]["type"], "tooLarge", "{failed}");
    assert!(e.get_rules("alice").await.unwrap().is_empty());
    assert!(
        e.store()
            .list_sieve_scripts("alice")
            .await
            .unwrap()
            .is_empty()
    );
    let cross = call(&e, &mut c, "MailRule/get", json!({"accountId":"bob"})).await;
    assert_eq!(cross[1]["type"], "accountNotFound");
}

#[tokio::test]
async fn editing_a_partial_builder_projection_cannot_drop_hidden_conditions() {
    let (e, mut c) = setup().await;
    let source = "require [\"fileinto\",\"body\"];\n# mailwoman:rule legacy begin\n# rule: Rich rule\nif allof (address :contains \"from\" \"news\", body :text :contains \"private\") { fileinto \"News\"; }\n# mailwoman:rule legacy end\n";
    e.store()
        .put_sieve_script("alice", "legacy", source, true)
        .await
        .unwrap();
    let response=call(&e,&mut c,"MailRule/set",json!({"accountId":"alice","update":{"legacy":{"name":"Renamed","conditions":[{"type":"from","op":"contains","value":"news"}],"actions":[{"type":"move","value":"News"}]}}})).await;
    assert_eq!(
        response[1]["notUpdated"]["legacy"]["type"], "invalidProperties",
        "{response}"
    );
    assert_eq!(
        e.store().list_sieve_scripts("alice").await.unwrap()[0].content,
        source
    );
}

#[tokio::test]
async fn local_vacation_obeys_extension_policy_before_enabling() {
    let (e, mut c) = setup().await;
    c.policy.vacation_allowed = true;
    c.policy.raw_edit = false;
    c.policy.allowed_extensions = Some(["fileinto".to_string()].into_iter().collect());
    let saved = call(
        &e,
        &mut c,
        "VacationResponse/set",
        json!({"accountId":"alice","update":{"singleton":{"isEnabled":true,"textBody":"Away","fromDate":"2026-01-01T00:00:00Z"}}}),
    )
    .await;
    assert_eq!(
        saved[1]["notUpdated"]["singleton"]["type"], "invalidSieve",
        "{saved}"
    );
    assert!(
        e.store()
            .list_sieve_scripts("alice")
            .await
            .unwrap()
            .is_empty()
    );
    let got = call(
        &e,
        &mut c,
        "VacationResponse/get",
        json!({"accountId":"alice"}),
    )
    .await;
    assert_eq!(got[1]["list"][0]["isEnabled"], false);
    c.policy.allowed_extensions = None;
    let saved = call(&e,&mut c,"VacationResponse/set",json!({"accountId":"alice","update":{"singleton":{"isEnabled":true,"textBody":"Away","fromDate":"2026-01-01T00:00:00Z"}}})).await;
    assert!(saved[1]["updated"].get("singleton").is_some(), "{saved}");
}
#[tokio::test]
async fn script_ids_survive_rename_and_activation_obeys_all_success() {
    let (e, mut c) = setup().await;
    let blob = e
        .store_upload("alice", "application/sieve", b"keep;")
        .await
        .unwrap();
    let made=call(&e,&mut c,"SieveScript/set",json!({"accountId":"alice","create":{"a":{"name":"first","blobId":blob}},"onSuccessActivateScript":"#a"})).await;
    assert_eq!(made[0], "SieveScript/set");
    let id = made[1]["created"]["a"]["id"].as_str().unwrap();
    assert_eq!(made[1]["created"]["a"]["isActive"], true);
    let rename = call(
        &e,
        &mut c,
        "SieveScript/set",
        json!({"accountId":"alice","update":{id:{"name":"renamed"}}}),
    )
    .await;
    assert!(rename[1]["updated"].get(id).is_some());
    let got = call(
        &e,
        &mut c,
        "SieveScript/get",
        json!({"accountId":"alice","ids":[id]}),
    )
    .await;
    assert_eq!(got[1]["list"][0]["id"], id);
    assert_eq!(got[1]["list"][0]["name"], "renamed");
    let refused = call(
        &e,
        &mut c,
        "SieveScript/set",
        json!({"accountId":"alice","destroy":[id],"onSuccessDeactivateScript":true}),
    )
    .await;
    assert_eq!(refused[1]["notDestroyed"][id]["type"], "sieveIsActive");
    let got = call(&e, &mut c, "SieveScript/get", json!({"accountId":"alice"})).await;
    assert_eq!(got[1]["list"][0]["isActive"], true);
    call(
        &e,
        &mut c,
        "SieveScript/set",
        json!({"accountId":"alice","onSuccessDeactivateScript":true}),
    )
    .await;
    let deleted = call(
        &e,
        &mut c,
        "SieveScript/set",
        json!({"accountId":"alice","destroy":[id]}),
    )
    .await;
    assert_eq!(deleted[1]["destroyed"], json!([id]));
}

#[tokio::test]
async fn managed_forward_copy_roundtrips_and_only_real_new_forwarding_sets_alert() {
    let (e, mut c) = setup().await;
    c.policy.raw_edit = false;
    let failed=call(&e,&mut c,"MailRule/set",json!({"accountId":"alice","create":{"a":{"conditions":[],"actions":[{"type":"forwardCopy","value":"outside@elsewhere.test"}]}}})).await;
    assert_eq!(failed[1]["type"], "forbidden", "{failed}");
    assert!(!c.forwarding_added);
    let made=call(&e,&mut c,"MailRule/set",json!({"accountId":"alice","create":{"a":{"conditions":[],"actions":[{"type":"forwardCopy","value":"other@example.test"}]}}})).await;
    assert_eq!(made[0], "MailRule/set", "{made}");
    assert!(c.forwarding_added);
    let got = call(&e, &mut c, "MailRule/get", json!({"accountId":"alice"})).await;
    assert_eq!(got[1]["list"][0]["actions"][0]["type"], "forwardCopy");
    assert_eq!(
        got[1]["list"][0]["actions"][0]["value"],
        "other@example.test"
    );
    c.forwarding_added = false;
    c.policy.vacation_allowed = true;
    c.policy.max_script_bytes = Some(65536);
    let saved = call(
        &e,
        &mut c,
        "VacationResponse/set",
        json!({"accountId":"alice","update":{"singleton":{"isEnabled":true,"textBody":"Away"}}}),
    )
    .await;
    assert!(saved[1]["updated"].get("singleton").is_some(), "{saved}");
    assert!(
        !c.forwarding_added,
        "vacation changes preserving redirect targets must not emit a forwarding alert"
    );
}
#[tokio::test]
async fn cross_account_blob_and_account_access_fail_closed() {
    let (e, mut c) = setup().await;
    let blob = e
        .store_upload("bob", "application/sieve", b"keep;")
        .await
        .unwrap();
    assert_eq!(
        call(&e, &mut c, "SieveScript/get", json!({"accountId":"bob"})).await[1]["type"],
        "accountNotFound"
    );
    assert_eq!(
        call(
            &e,
            &mut c,
            "SieveScript/validate",
            json!({"accountId":"alice","blobId":blob})
        )
        .await[1]["type"],
        "blobNotFound"
    );
    let result = call(
        &e,
        &mut c,
        "SieveScript/set",
        json!({"accountId":"alice","create":{"a":{"blobId":blob}}}),
    )
    .await;
    assert_eq!(result[1]["notCreated"]["a"]["type"], "blobNotFound");
    assert!(
        e.store()
            .list_sieve_scripts("alice")
            .await
            .unwrap()
            .is_empty()
    );
}
#[tokio::test]
async fn quotas_policy_invalid_scripts_and_cas_are_enforced() {
    let (e, mut c) = setup().await;
    let blob = e
        .store_upload("alice", "application/sieve", b"keep;")
        .await
        .unwrap();
    let made=call(&e,&mut c,"SieveScript/set",json!({"accountId":"alice","create":{"a":{"name":"same","blobId":blob},"b":{"name":"same","blobId":blob},"c":{"name":"other","blobId":blob},"d":{"blobId":blob}},"onSuccessActivateScript":"#a"})).await;
    assert_eq!(made[1]["notCreated"]["b"]["type"], "alreadyExists");
    assert_eq!(made[1]["notCreated"]["d"]["type"], "overQuota");
    assert_eq!(made[1]["created"]["a"]["isActive"], false);
    assert_eq!(
        call(
            &e,
            &mut c,
            "SieveScript/set",
            json!({"accountId":"alice","ifInState":"stale"})
        )
        .await[1]["type"],
        "stateMismatch"
    );
    for (source, expected) in [
        ("redirect \"x@outside.test\";", "forbidden"),
        ("require [\"vacation\"]; vacation \"away\";", "forbidden"),
        ("this is invalid;", "invalidSieve"),
    ] {
        let b = e
            .store_upload("alice", "application/sieve", source.as_bytes())
            .await
            .unwrap();
        let r = call(
            &e,
            &mut c,
            "SieveScript/validate",
            json!({"accountId":"alice","blobId":b}),
        )
        .await;
        assert_eq!(r[1]["error"]["type"], expected, "{r}");
    }
    c.policy.raw_edit = false;
    assert_eq!(
        call(
            &e,
            &mut c,
            "SieveScript/validate",
            json!({"accountId":"alice","blobId":blob})
        )
        .await[1]["error"]["type"],
        "forbidden"
    );
}
#[tokio::test]
async fn query_filter_sort_anchor_and_result_references_work() {
    let (e, mut c) = setup().await;
    let blob = e
        .store_upload("alice", "application/sieve", b"keep;")
        .await
        .unwrap();
    call(&e,&mut c,"SieveScript/set",json!({"accountId":"alice","create":{"a":{"name":"alpha","blobId":blob},"b":{"name":"beta","blobId":blob}}})).await;
    let response=e.handle_jmap_with_sieve("alice",&json!({"using":["urn:ietf:params:jmap:core","urn:ietf:params:jmap:sieve"],"methodCalls":[["SieveScript/query",{"accountId":"alice","filter":{"operator":"NOT","conditions":[{"name":"beta"}]},"sort":[{"property":"name","isAscending":false}],"calculateTotal":true},"q"],["SieveScript/get",{"accountId":"alice","#ids":{"resultOf":"q","name":"SieveScript/query","path":"/ids"},"properties":["name"]},"g"]]}),Some(&mut c)).await;
    assert_eq!(response["methodResponses"][0][1]["total"], 1);
    assert_eq!(
        response["methodResponses"][1][1]["list"][0]["name"],
        "alpha"
    );
    assert_eq!(
        call(
            &e,
            &mut c,
            "SieveScript/query",
            json!({"accountId":"alice","filter":{"bogus":true}})
        )
        .await[1]["type"],
        "unsupportedFilter"
    );
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

#[tokio::test]
async fn managesieve_wire_crud_uses_remote_authority_and_preserves_ids() {
    use tokio::io::{AsyncReadExt, AsyncWriteExt};
    let (e, mut c) = setup().await;
    let (client, mut server) = tokio::io::duplex(65536);
    let mut responses = String::from(
        "\"IMPLEMENTATION\" \"test\"\r\n\"SIEVE\" \"fileinto\"\r\n\"SASL\" \"PLAIN\"\r\n\"VERSION\" \"1.0\"\r\nOK\r\nOK\r\n",
    );
    // create: list, check, space, put, activate
    responses.push_str("OK\r\nOK\r\nOK\r\nOK\r\nOK\r\n");
    // rename: list/get/check/space/rename
    responses.push_str("\"main\" ACTIVE\r\nOK\r\n{5}\r\nkeep;\r\nOK\r\nOK\r\nOK\r\nOK\r\n");
    // get: list/get
    responses.push_str("\"renamed\" ACTIVE\r\nOK\r\n{5}\r\nkeep;\r\nOK\r\n");
    // deactivate: list/get/setactive
    responses.push_str("\"renamed\" ACTIVE\r\nOK\r\n{5}\r\nkeep;\r\nOK\r\nOK\r\n");
    // destroy: list/get/delete
    responses.push_str("\"renamed\"\r\nOK\r\n{5}\r\nkeep;\r\nOK\r\nOK\r\n");
    server.write_all(responses.as_bytes()).await.unwrap();
    c.client = Some(
        mw_sieve::ManageSieveClient::from_stream(
            Box::new(client),
            "sieve.example",
            mw_sieve::TlsMode::Plaintext,
            mw_sieve::Credentials::Plain {
                username: "alice".into(),
                password: "synthetic".into(),
            },
        )
        .await
        .unwrap(),
    );
    let blob = e
        .store_upload("alice", "application/sieve", b"keep;")
        .await
        .unwrap();
    let made=call(&e,&mut c,"SieveScript/set",json!({"accountId":"alice","create":{"a":{"name":"main","blobId":blob}},"onSuccessActivateScript":"#a"})).await;
    let id = made[1]["created"]["a"]["id"]
        .as_str()
        .unwrap_or_else(|| panic!("{made}"));
    let renamed = call(
        &e,
        &mut c,
        "SieveScript/set",
        json!({"accountId":"alice","update":{id:{"name":"renamed"}}}),
    )
    .await;
    assert!(renamed[1]["updated"].get(id).is_some(), "{renamed}");
    let got = call(&e, &mut c, "SieveScript/get", json!({"accountId":"alice"})).await;
    assert_eq!(got[1]["list"][0]["id"], id);
    assert_eq!(got[1]["list"][0]["name"], "renamed");
    assert_eq!(
        serde_json::from_str::<Value>(
            &e.store().list_sieve_scripts("alice").await.unwrap()[0].state_json
        )
        .unwrap()["_remote"],
        true
    );
    call(
        &e,
        &mut c,
        "SieveScript/set",
        json!({"accountId":"alice","onSuccessDeactivateScript":true}),
    )
    .await;
    let deleted = call(
        &e,
        &mut c,
        "SieveScript/set",
        json!({"accountId":"alice","destroy":[id]}),
    )
    .await;
    assert_eq!(deleted[1]["destroyed"], json!([id]));
    drop(c);
    let mut wire = String::new();
    server.read_to_string(&mut wire).await.unwrap();
    for command in [
        "CHECKSCRIPT",
        "HAVESPACE \"main\" 5",
        "PUTSCRIPT \"main\"",
        "SETACTIVE \"main\"",
        "RENAMESCRIPT \"main\" \"renamed\"",
        "SETACTIVE \"\"",
        "DELETESCRIPT \"renamed\"",
    ] {
        assert!(wire.contains(command), "missing {command}: {wire}");
    }
}

#[tokio::test]
async fn blob_identity_is_stable_and_activation_rechecks_current_policy() {
    let (e, mut c) = setup().await;
    e.store()
        .put_sieve_script(
            "alice",
            "legacy",
            "redirect \"outside@elsewhere.test\";",
            false,
        )
        .await
        .unwrap();
    let got = call(&e, &mut c, "SieveScript/get", json!({"accountId":"alice"})).await;
    let again = call(&e, &mut c, "SieveScript/get", json!({"accountId":"alice"})).await;
    assert_eq!(got[1]["list"][0]["blobId"], again[1]["list"][0]["blobId"]);
    let id = got[1]["list"][0]["id"].as_str().unwrap();
    let activated = call(
        &e,
        &mut c,
        "SieveScript/set",
        json!({"accountId":"alice","onSuccessActivateScript":id}),
    )
    .await;
    assert_eq!(activated[1]["type"], "forbidden", "{activated}");
    assert!(!e.store().list_sieve_scripts("alice").await.unwrap()[0].active);
}

#[tokio::test]
async fn live_pigeonhole_jmap_crud_validation_and_activation() {
    let Ok(target) = std::env::var("MW_SIEVE_JMAP_LIVE") else {
        eprintln!("MW_SIEVE_JMAP_LIVE not set; skipping live Pigeonhole");
        return;
    };
    let (host, port) = target.rsplit_once(':').unwrap();
    let (e, mut c) = setup().await;
    let client = mw_sieve::ManageSieveClient::connect(
        host,
        port.parse().unwrap(),
        mw_sieve::TlsMode::Plaintext,
        mw_sieve::Credentials::Plain {
            username: "testuser".into(),
            password: "testpass".into(),
        },
    )
    .await
    .unwrap();
    c.client = Some(client);
    let source = "require [\"fileinto\"]; if header :contains \"subject\" \"newsletter\" { fileinto \"INBOX/News\"; }";
    let blob = e
        .store_upload("alice", "application/sieve", source.as_bytes())
        .await
        .unwrap();
    assert!(
        call(
            &e,
            &mut c,
            "SieveScript/validate",
            json!({"accountId":"alice","blobId":blob})
        )
        .await[1]["error"]
            .is_null()
    );
    let made=call(&e,&mut c,"SieveScript/set",json!({"accountId":"alice","create":{"a":{"name":"codex-jmap","blobId":blob}},"onSuccessActivateScript":"#a"})).await;
    let id = made[1]["created"]["a"]["id"]
        .as_str()
        .unwrap_or_else(|| panic!("{made}"));
    assert_eq!(made[1]["created"]["a"]["isActive"], true);
    let renamed = call(
        &e,
        &mut c,
        "SieveScript/set",
        json!({"accountId":"alice","update":{id:{"name":"codex-renamed"}}}),
    )
    .await;
    assert!(renamed[1]["updated"].get(id).is_some(), "{renamed}");
    let got = call(
        &e,
        &mut c,
        "SieveScript/get",
        json!({"accountId":"alice","ids":[id]}),
    )
    .await;
    assert_eq!(got[1]["list"][0]["id"], id);
    assert_eq!(got[1]["list"][0]["isActive"], true);
    let downloaded = e
        .fetch_blob("alice", got[1]["list"][0]["blobId"].as_str().unwrap())
        .await
        .unwrap()
        .unwrap();
    assert_eq!(downloaded.bytes, source.as_bytes());
    c.policy.vacation_allowed = true;
    c.policy.max_script_bytes = Some(65536);
    c.policy.raw_edit = false;
    let vacation = call(&e,&mut c,"VacationResponse/set",json!({"accountId":"alice","update":{"singleton":{"isEnabled":true,"subject":"Away","textBody":"Back soon","fromDate":"2026-01-01T00:00:00Z","toDate":"2027-01-01T00:00:00Z"}}})).await;
    assert!(
        vacation[1]["updated"].get("singleton").is_some(),
        "{vacation}"
    );
    let vacation_get = call(
        &e,
        &mut c,
        "VacationResponse/get",
        json!({"accountId":"alice"}),
    )
    .await;
    assert_eq!(vacation_get[1]["list"][0]["isEnabled"], true);
    c.policy.raw_edit = true;
    let tampered = call(
        &e,
        &mut c,
        "SieveScript/set",
        json!({"accountId":"alice","update":{id:{"blobId":blob}}}),
    )
    .await;
    assert_eq!(tampered[1]["notUpdated"][id]["type"], "forbidden");
    let disabled = call(
        &e,
        &mut c,
        "VacationResponse/set",
        json!({"accountId":"alice","update":{"singleton":{"isEnabled":false}}}),
    )
    .await;
    assert!(
        disabled[1]["updated"].get("singleton").is_some(),
        "{disabled}"
    );
    let restored = c
        .client
        .as_mut()
        .unwrap()
        .get_script("codex-renamed")
        .await
        .unwrap();
    assert!(restored.contains(source));
    let disabled_get = call(
        &e,
        &mut c,
        "VacationResponse/get",
        json!({"accountId":"alice"}),
    )
    .await;
    assert_eq!(disabled_get[1]["list"][0]["isEnabled"], false);
    assert_eq!(disabled_get[1]["list"][0]["textBody"], "Back soon");
    let failed = call(
        &e,
        &mut c,
        "SieveScript/set",
        json!({"accountId":"alice","destroy":[id],"onSuccessDeactivateScript":true}),
    )
    .await;
    assert_eq!(failed[1]["notDestroyed"][id]["type"], "forbidden");
    call(
        &e,
        &mut c,
        "SieveScript/set",
        json!({"accountId":"alice","onSuccessDeactivateScript":true}),
    )
    .await;
    let deleted = call(
        &e,
        &mut c,
        "SieveScript/set",
        json!({"accountId":"alice","destroy":[id]}),
    )
    .await;
    assert_eq!(deleted[1]["notDestroyed"][id]["type"], "forbidden");
    c.client
        .as_mut()
        .unwrap()
        .delete_script("codex-renamed")
        .await
        .unwrap();
    assert!(
        c.client
            .as_mut()
            .unwrap()
            .list_scripts()
            .await
            .unwrap()
            .is_empty()
    );
}

#[tokio::test]
async fn broken_remote_session_never_falls_back_or_reuses_the_stream() {
    use tokio::io::AsyncWriteExt;
    let (e, mut c) = setup().await;
    e.store()
        .put_sieve_script("alice", "local", "keep;", false)
        .await
        .unwrap();
    let (client, mut server) = tokio::io::duplex(4096);
    server
        .write_all(b"\"SIEVE\" \"fileinto\"\r\n\"SASL\" \"PLAIN\"\r\nOK\r\nOK\r\n")
        .await
        .unwrap();
    c.client = Some(
        mw_sieve::ManageSieveClient::from_stream(
            Box::new(client),
            "sieve.example",
            mw_sieve::TlsMode::Plaintext,
            mw_sieve::Credentials::Plain {
                username: "alice".into(),
                password: "synthetic".into(),
            },
        )
        .await
        .unwrap(),
    );
    drop(server);
    for _ in 0..2 {
        let result = call(&e, &mut c, "SieveScript/get", json!({"accountId":"alice"})).await;
        assert_eq!(result[1]["type"], "serverUnavailable");
    }
    assert_eq!(
        e.store().list_sieve_scripts("alice").await.unwrap()[0].name,
        "local"
    );
}
