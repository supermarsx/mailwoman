//! The unchanged facade API must inject complete metadata into the real router.
use axum::{
    body::{Body, to_bytes},
    extract::ConnectInfo,
    http::{Method, Request, StatusCode, header},
};
use mw_server::{AppConfig, HardeningConfig, SecurityConfig, ServerMode, V6Config};
use serde_json::Value;
use sha2::{Digest, Sha256};
use std::net::{Ipv4Addr, SocketAddr};
use tower::ServiceExt;

fn sha(bytes: &[u8]) -> String {
    Sha256::digest(bytes)
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect()
}

fn request(method: Method, path: &str) -> Request<Body> {
    Request::builder()
        .method(method)
        .uri(path)
        .header(header::HOST, "localhost")
        .header(header::USER_AGENT, "Mozilla/5.0 Mailwoman Product Fixture")
        .extension(ConnectInfo(SocketAddr::from((Ipv4Addr::LOCALHOST, 41234))))
        .body(Body::empty())
        .unwrap()
}

#[test]
fn facade_router_serves_complete_source_bound_build_inventory_policy_and_head() {
    let worker = std::thread::Builder::new()
        .name("product-build-info-fixture".into())
        .stack_size(16 * 1024 * 1024)
        .spawn(|| {
            tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()
                .unwrap()
                .block_on(async {
                    let directory = tempfile::tempdir().unwrap();
                    let (router, _) = mw_server::build_app_full(
                        AppConfig {
                            db_path: directory
                                .path()
                                .join("private-data.sqlite")
                                .to_str()
                                .unwrap()
                                .into(),
                            server_key_hex: Some("11".repeat(32)),
                            web_dir: None,
                            cookie_secure: false,
                            mode: ServerMode::Proxy,
                            hardening: HardeningConfig::default(),
                            security: SecurityConfig::default(),
                        },
                        V6Config {
                            admin_enabled: false,
                            admin_username: None,
                            admin_password: None,
                            redis_url: None,
                        },
                    )
                    .await
                    .unwrap();
                    // Real peer metadata activates protocol guards. Keep that
                    // boundary in this fixture, with a valid HTTP authority.
                    let mut missing_host = request(Method::GET, "/api/product-security");
                    missing_host.headers_mut().remove(header::HOST);
                    assert_eq!(
                        router.clone().oneshot(missing_host).await.unwrap().status(),
                        StatusCode::BAD_REQUEST
                    );
                    let response = router
                        .clone()
                        .oneshot(request(Method::GET, "/api/product-security"))
                        .await
                        .unwrap();
                    assert_eq!(response.status(), StatusCode::OK);
                    assert_eq!(response.headers()[header::CACHE_CONTROL], "no-cache");
                    assert_eq!(response.headers()[header::CONTENT_TYPE], "application/json");
                    let bytes = to_bytes(response.into_body(), 64 * 1024).await.unwrap();
                    let facts: Value = serde_json::from_slice(&bytes).unwrap();
                    assert_eq!(facts["revision"], env!("MW_PRODUCT_REVISION"));
                    assert_eq!(facts["target"], env!("MW_PRODUCT_TARGET"));
                    assert!(facts["componentCount"].as_u64().unwrap() > 100);
                    assert_eq!(facts["conformityAssessed"], false);
                    assert_eq!(facts["inventoryScope"], "workspace-lock");
                    assert!(
                        !String::from_utf8_lossy(&bytes)
                            .contains(directory.path().to_str().unwrap())
                    );

                    let response = router
                        .clone()
                        .oneshot(request(Method::GET, "/api/product-security/sbom"))
                        .await
                        .unwrap();
                    assert_eq!(response.status(), StatusCode::OK);
                    assert_eq!(
                        response.headers()[header::CONTENT_TYPE],
                        "application/vnd.cyclonedx+json"
                    );
                    assert_eq!(
                        response.headers()[header::CONTENT_DISPOSITION],
                        "attachment; filename=mailwoman-workspace.cdx.json"
                    );
                    let bom = to_bytes(response.into_body(), 8 * 1024 * 1024)
                        .await
                        .unwrap();
                    assert_eq!(facts["sbomSha256"], sha(&bom));
                    let parsed: Value = serde_json::from_slice(&bom).unwrap();
                    assert_eq!(
                        parsed["components"].as_array().unwrap().len() as u64,
                        facts["componentCount"].as_u64().unwrap()
                    );
                    assert_eq!(parsed["bomFormat"], "CycloneDX");

                    let response = router
                        .clone()
                        .oneshot(request(Method::GET, "/api/product-security/policy"))
                        .await
                        .unwrap();
                    assert_eq!(response.status(), StatusCode::OK);
                    assert_eq!(
                        response.headers()[header::CONTENT_DISPOSITION],
                        "attachment; filename=SECURITY.md"
                    );
                    let policy = to_bytes(response.into_body(), 512 * 1024).await.unwrap();
                    assert_eq!(&policy[..], include_bytes!("../../../SECURITY.md"));
                    assert_eq!(facts["securityPolicySha256"], sha(&policy));

                    let response = router
                        .oneshot(request(Method::HEAD, "/api/product-security"))
                        .await
                        .unwrap();
                    assert_eq!(response.status(), StatusCode::OK);
                    assert_eq!(response.headers()[header::CONTENT_TYPE], "application/json");
                    assert!(to_bytes(response.into_body(), 1).await.unwrap().is_empty());
                });
        })
        .unwrap();
    if let Err(panic) = worker.join() {
        std::panic::resume_unwind(panic);
    }
}
