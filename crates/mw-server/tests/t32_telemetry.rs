use mw_server::{AppConfig, HardeningConfig, SecurityConfig, ServerMode, V6Config, build_app_full};
use serde_json::{Value, json};
mod common;

/// Run against a loopback OpenTelemetry Collector and inspect its debug output.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
#[ignore = "requires MW_ACCEPTANCE_OTLP_ENDPOINT and a local collector"]
async fn persisted_configuration_exports_request_traces_and_metrics() {
    use tracing_subscriber::prelude::*;
    let endpoint = std::env::var("MW_ACCEPTANCE_OTLP_ENDPOINT").expect("local collector endpoint");
    assert!(endpoint.starts_with("http://127.0.0.1:"));
    let path = common::test_db::unique_db_path("telemetry-collector");
    let key = "22".repeat(32);
    let store = mw_store::Store::open(&path.to_string_lossy(),mw_store::ServerKey::from_hex(&key).unwrap()).await.unwrap();
    let config = mw_server::telemetry_config::TelemetryConfig {
        enabled:true,endpoint,service_name:"mailwoman-local-acceptance".into(),environment:"local-test".into(),
        sample_ratio:1.0,metrics_interval_secs:10,span_export_delay_ms:100,..Default::default()
    };
    store.set_setting(mw_server::telemetry_config::SETTINGS_KEY,&serde_json::to_string(&config).unwrap()).await.unwrap();
    let (layer, reload) = tracing_subscriber::reload::Layer::new(None::<tracing_opentelemetry::OpenTelemetryLayer<tracing_subscriber::Registry, opentelemetry_sdk::trace::SdkTracer>>);
    mw_server::observability::install_tracer_bridge(move |tracer| reload.reload(Some(tracing_opentelemetry::layer().with_tracer(tracer))).map_err(|e|e.to_string()));
    tracing_subscriber::registry().with(layer.with_filter(tracing_subscriber::filter::filter_fn(|metadata|metadata.name()=="mailwoman.http.request"))).init();
    let (app,_) = build_app_full(AppConfig {db_path:path.to_string_lossy().into(),server_key_hex:Some(key),web_dir:None,cookie_secure:false,mode:ServerMode::Proxy,hardening:HardeningConfig::default(),security:SecurityConfig::default()},V6Config {admin_enabled:false,admin_username:None,admin_password:None,redis_url:None}).await.unwrap();
    let listener=tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let base=format!("http://{}",listener.local_addr().unwrap());
    let server=tokio::spawn(async move {axum::serve(listener,app.into_make_service_with_connect_info::<std::net::SocketAddr>()).await.unwrap();});
    let client=reqwest::Client::builder().no_proxy().build().unwrap();
    for _ in 0..3 {assert_eq!(client.get(format!("{base}/healthz?private-query-must-not-export=1")).send().await.unwrap().status(),200);}
    tracing::info!(private_field="private-event-must-not-export", "local diagnostic event");
    tokio::time::sleep(std::time::Duration::from_secs(11)).await;
    tokio::task::spawn_blocking(mw_server::observability::shutdown_otlp).await.unwrap();
    server.abort();
}

#[tokio::test]
async fn telemetry_is_disabled_requires_step_up_and_cas_and_applies_on_restart() {
    let path = common::test_db::unique_db_path("telemetry");
    let config = AppConfig {
        db_path:path.to_string_lossy().into(), server_key_hex:Some("11".repeat(32)), web_dir:None,
        cookie_secure:false, mode:ServerMode::Proxy, hardening:HardeningConfig::default(), security:SecurityConfig::default(),
    };
    let (app, _) = build_app_full(config, V6Config {admin_enabled:true,admin_username:Some("root".into()),admin_password:Some("hunter2".into()),redis_url:None}).await.unwrap();
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let base = format!("http://{}", listener.local_addr().unwrap());
    let server = tokio::spawn(async move { axum::serve(listener, app.into_make_service_with_connect_info::<std::net::SocketAddr>()).await.unwrap(); });
    let client = reqwest::Client::builder().no_proxy().cookie_store(true).build().unwrap();
    let url = format!("{base}/admin/telemetry");
    assert_eq!(client.get(&url).send().await.unwrap().status(),401);
    assert_eq!(client.post(format!("{base}/admin/login")).json(&json!({"username":"root","password":"hunter2"})).send().await.unwrap().status(),200);
    let response = client.get(&url).send().await.unwrap();
    assert_eq!(response.status(),200);
    assert_eq!(response.headers()["cache-control"],"private, no-store");
    let tag = response.headers()["etag"].to_str().unwrap().to_owned();
    let view:Value = response.json().await.unwrap();
    assert_eq!(view["config"]["enabled"],false);
    assert_eq!(view["active"]["enabled"],false);
    assert_eq!(view["restartRequired"],false);
    let mut value = view["config"].clone();
    value["environment"] = json!("acceptance");
    assert_eq!(client.put(&url).header("if-match",&tag).json(&value).send().await.unwrap().status(),403);
    assert_eq!(client.post(format!("{base}/admin/step-up")).json(&json!({"password":"hunter2"})).send().await.unwrap().status(),200);
    assert_eq!(client.put(&url).json(&value).send().await.unwrap().status(),428);
    let mut invalid = value.clone(); invalid["spanBatchSize"] = json!(32769);
    assert_eq!(client.put(&url).header("if-match",&tag).json(&invalid).send().await.unwrap().status(),422);
    let response = client.put(&url).header("if-match",&tag).json(&value).send().await.unwrap();
    assert_eq!(response.status(),200);
    let saved:Value = response.json().await.unwrap();
    assert_eq!(saved["restartRequired"],true);
    assert_eq!(saved["active"]["environment"],"");
    assert_eq!(client.put(&url).header("if-match",&tag).json(&value).send().await.unwrap().status(),412);
    let store = mw_store::Store::open(&path.to_string_lossy(),mw_store::ServerKey::from_hex(&"11".repeat(32)).unwrap()).await.unwrap();
    let restarted = mw_server::observability::ObservabilityConfig::from_store(&store).await.unwrap();
    assert_eq!(restarted.telemetry.environment,"acceptance");
    assert!(restarted.otlp_endpoint.is_none());
    assert!(restarted.metrics_token.is_none());
    server.abort();
}
