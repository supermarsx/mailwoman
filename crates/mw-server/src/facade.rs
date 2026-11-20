//! Stable public host API. Revision-bearing build facts compile only here;
//! HTTP implementation and request authority remain in the independent core.
pub use server_core::*;

use std::sync::OnceLock;

const METADATA: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/product-security.json"));
const SBOM: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/software-bom.cdx.json"));
const POLICY: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/security-policy.txt"));

fn embedded_build_info() -> anyhow::Result<ProductBuildInfo> {
    static INFO: OnceLock<Result<ProductBuildInfo, String>> = OnceLock::new();
    INFO.get_or_init(|| {
        ProductBuildInfo::new(
            bytes::Bytes::from_static(METADATA),
            bytes::Bytes::from_static(SBOM),
            bytes::Bytes::from_static(POLICY),
        )
        .map_err(|error| error.to_string())
    })
    .as_ref()
    .cloned()
    .map_err(|error| anyhow::anyhow!("Invalid compiled product inventory: {error}"))
}

/// Build the fully-wired application with this executable's complete build facts.
pub async fn build_app(config: AppConfig) -> anyhow::Result<axum::Router> {
    server_core::build_app_with_info(config, embedded_build_info()?).await
}

/// Existing realtime-handle API, with immutable build facts injected by the host.
pub async fn build_app_with_push(config: AppConfig) -> anyhow::Result<(axum::Router, PushHandle)> {
    server_core::build_app_with_push_and_info(config, embedded_build_info()?).await
}

pub async fn build_app_full(
    config: AppConfig,
    v6config: V6Config,
) -> anyhow::Result<(axum::Router, PushHandle)> {
    server_core::build_app_full_with_info(config, v6config, embedded_build_info()?).await
}

#[cfg(feature = "load-test-fixtures")]
#[doc(hidden)]
pub async fn build_app_with_fixture_engine(
    config: AppConfig,
    v6config: V6Config,
    factory: impl FnOnce(&mw_store::Store) -> std::sync::Arc<mw_engine::Engine> + Send + 'static,
) -> anyhow::Result<(axum::Router, PushHandle)> {
    server_core::build_app_with_fixture_engine_and_info(
        config,
        v6config,
        factory,
        embedded_build_info()?,
    )
    .await
}

#[doc(hidden)]
pub async fn build_app_with_sso_mock(
    config: AppConfig,
    v6config: V6Config,
    providers: Vec<(String, sso::SsoEntry)>,
) -> anyhow::Result<(axum::Router, PushHandle)> {
    server_core::build_app_with_sso_mock_and_info(
        config,
        v6config,
        providers,
        embedded_build_info()?,
    )
    .await
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn compiled_build_info_is_complete_and_shared() {
        let info = embedded_build_info().unwrap();
        assert_eq!(info.revision(), env!("MW_PRODUCT_REVISION"));
        let facts: serde_json::Value = serde_json::from_slice(METADATA).unwrap();
        assert_eq!(facts["target"], env!("MW_PRODUCT_TARGET"));
        assert!(facts["componentCount"].as_u64().unwrap() > 100);
        assert_eq!(facts["inventoryScope"], "workspace-lock");
        assert_eq!(facts["conformityAssessed"], false);
        assert_eq!(std::any::type_name::<AppConfig>(), "mw_server::AppConfig");
        assert_eq!(embedded_build_info().unwrap().revision(), info.revision());
    }
}
