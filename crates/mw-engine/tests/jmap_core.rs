use mw_engine::Engine;
use mw_store::{ServerKey, Store};
use serde_json::json;

#[tokio::test]
async fn echo_is_account_independent_and_preserves_type_fields() {
    let engine = Engine::new(Store::open_in_memory(ServerKey::generate()).await.unwrap());
    let response=engine.handle_jmap("missing", &json!({"using":["urn:ietf:params:jmap:core"],"methodCalls":[["Core/echo",{"type":"arbitrary","nested":[1,true,null]},"echo"],["Email/get",{},"missing"],["Core/echo",{"after":true},"after"]]})).await;
    assert_eq!(
        response["methodResponses"][0],
        json!(["Core/echo",{"type":"arbitrary","nested":[1,true,null]},"echo"])
    );
    assert_eq!(response["methodResponses"][1][0], "error");
    assert_eq!(response["methodResponses"][2][1], json!({"after":true}));
}

#[tokio::test]
async fn result_reference_failure_is_an_error_and_does_not_stop_later_calls() {
    let engine = Engine::new(Store::open_in_memory(ServerKey::generate()).await.unwrap());
    let response=engine.handle_jmap("missing", &json!({"using":["urn:ietf:params:jmap:core"],"methodCalls":[["Core/echo",{"items":[{"id":"one"},{"id":"two"}]},"first"],["Core/echo",{"#ids":{"resultOf":"first","name":"Wrong/name","path":"/items/*/id"}},"bad"],["Core/echo",{"#ids":{"resultOf":"first","name":"Core/echo","path":"/items/*/id"}},"good"]]})).await;
    assert_eq!(
        response["methodResponses"][1],
        json!(["error",{"type":"invalidResultReference"},"bad"])
    );
    assert_eq!(
        response["methodResponses"][2][1],
        json!({"ids":["one","two"]})
    );
}
#[tokio::test]
async fn limits_and_reference_conflicts_refuse_before_account_dispatch() {
    let engine = Engine::new(Store::open_in_memory(ServerKey::generate()).await.unwrap());
    let response=engine.handle_jmap("missing", &json!({"using":["urn:ietf:params:jmap:core"],"createdIds":{"prior":"stable"},"methodCalls":[
        ["Core/echo",{"values":[{"a/b":[1,2]},{"a/b":[3]}]},"source"],
        ["Core/echo",{"#values":{"resultOf":"source","name":"Core/echo","path":"/values/*/a~1b"}},"mapped"],
        ["Core/echo",{"values":[],"#values":{"resultOf":"source","name":"Core/echo","path":"/values"}},"conflict"],
        ["Email/get",{"ids":vec!["x";mw_engine::jmap::core::MAX_OBJECTS+1]},"get"],
        ["Email/set",{"destroy":vec!["x";mw_engine::jmap::core::MAX_OBJECTS+1]},"set"]
    ]})).await;
    assert_eq!(response["methodResponses"][1][1], json!({"values":[1,2,3]}));
    assert_eq!(
        response["methodResponses"][2][1]["type"],
        "invalidArguments"
    );
    for i in [3, 4] {
        assert_eq!(response["methodResponses"][i][1]["type"], "requestTooLarge");
    }
    assert_eq!(response["createdIds"], json!({"prior":"stable"}));
}
