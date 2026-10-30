//! Store-backed SMTP transport policy state, shared across sessions and restarts.
use mw_mail_transport::{BoxFuture, CachedSts, PolicyError, StsCache};

pub(crate) struct DurableStsCache(pub mw_store::Store);

pub(crate) async fn for_endpoint(
    store: &mw_store::Store,
    endpoint: Option<&crate::domain_config::Endpoint>,
    identity: &str,
) -> Result<Option<std::sync::Arc<mw_mail_transport::TransportPolicy>>, String> {
    let Some(config) = endpoint
        .and_then(|value| value.security.as_ref())
        .and_then(|security| security.smtp_policy.as_ref())
        .filter(|config| config.enabled())
    else {
        return Ok(None);
    };
    let domain = crate::upstream_resolve::domain_of(identity)
        .ok_or("SMTP transport policy requires an account domain")?;
    let domain = store
        .canonical_policy_domain(&domain)
        .await
        .map_err(|e| e.to_string())?;
    let context = mw_mail_transport::PolicyContext {
        doh_url: std::env::var("MW_SMTP_POLICY_DOH_URL")
            .unwrap_or_else(|_| "https://cloudflare-dns.com/dns-query".into()),
        egress_domain: Some(domain.clone()),
        cache_namespace: domain,
    };
    mw_mail_transport::TransportPolicy::new(
        config.clone(),
        context,
        std::sync::Arc::new(DurableStsCache(store.clone())),
    )
    .map(|policy| Some(std::sync::Arc::new(policy)))
    .map_err(|e| e.to_string())
}

impl StsCache for DurableStsCache {
    fn load<'a>(&'a self, key: &'a str) -> BoxFuture<'a, Result<Option<CachedSts>, PolicyError>> {
        Box::pin(async move {
            if key.len() != 64 || !key.bytes().all(|byte| byte.is_ascii_hexdigit()) {
                return Err(PolicyError::Cache);
            }
            let value = self
                .0
                .get_setting(&format!("smtp.transport.sts.{}", key.to_ascii_lowercase()))
                .await
                .map_err(|_| PolicyError::Cache)?;
            value
                .map(|json| serde_json::from_str(&json).map_err(|_| PolicyError::Cache))
                .transpose()
        })
    }

    fn store<'a>(
        &'a self,
        key: &'a str,
        value: &'a CachedSts,
    ) -> BoxFuture<'a, Result<(), PolicyError>> {
        Box::pin(async move {
            let json = serde_json::to_string(value).map_err(|_| PolicyError::Cache)?;
            let now = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map_err(|_| PolicyError::Cache)?
                .as_secs();
            self.0
                .cache_smtp_policy(key, &json, now)
                .await
                .map_err(|_| PolicyError::Cache)
        })
    }
}
