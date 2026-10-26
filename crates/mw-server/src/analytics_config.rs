//! Deliberately separate from operational OpenTelemetry configuration.
use serde::{Deserialize, Serialize};

pub const KEY: &str = "admin.analytics.v1";
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Provider {
    #[default]
    Umami,
    Matomo,
    Ga4,
}

#[derive(Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default, rename_all = "camelCase", deny_unknown_fields)]
pub struct Config {
    pub enabled: bool,
    pub provider: Provider,
    pub endpoint: String,
    pub site_id: String,
    pub allow_http: bool,
    pub api_secret_sealed: Option<Vec<u8>>,
}
impl Config {
    pub fn validate(&self) -> Result<(), &'static str> {
        if self.endpoint.len() > 2048 || self.site_id.len() > 64 {
            return Err("Analytics configuration is too long.");
        }
        if self.endpoint.is_empty() {
            return if self.enabled {
                Err("Enter a collector endpoint before enabling analytics.")
            } else {
                Ok(())
            };
        }
        let url =
            reqwest::Url::parse(&self.endpoint).map_err(|_| "Enter an absolute collector URL.")?;
        if !(url.scheme() == "https" || (self.allow_http && url.scheme() == "http"))
            || url.host_str().is_none()
            || !url.username().is_empty()
            || url.password().is_some()
            || url.query().is_some()
            || url.fragment().is_some()
        {
            return Err(
                "Use HTTPS without credentials, query or fragment. HTTP needs explicit opt-in.",
            );
        }
        let valid = match self.provider {
            Provider::Ga4 => {
                !self.allow_http
                    && matches!(
                        self.endpoint.as_str(),
                        "https://www.google-analytics.com/mp/collect"
                            | "https://region1.google-analytics.com/mp/collect"
                    )
                    && self.site_id.strip_prefix("G-").is_some_and(|id| {
                        !id.is_empty()
                            && id
                                .bytes()
                                .all(|b| b.is_ascii_uppercase() || b.is_ascii_digit())
                    })
                    && (!self.enabled || self.api_secret_sealed.is_some())
            }
            Provider::Umami => {
                url.path().ends_with("/api/send") && uuid::Uuid::parse_str(&self.site_id).is_ok()
            }
            Provider::Matomo => {
                url.path().ends_with("/matomo.php")
                    && self.site_id.parse::<u32>().is_ok_and(|id| id > 0)
            }
        };
        if !valid {
            return Err(
                "Use the provider collector and valid site ID. GA4 requires an approved HTTPS endpoint, G- measurement ID and API secret when enabled.",
            );
        }
        Ok(())
    }
}
impl std::fmt::Debug for Config {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AnalyticsConfig")
            .field("enabled", &self.enabled)
            .field("provider", &self.provider)
            .finish_non_exhaustive()
    }
}
impl Config {
    pub fn public(&self) -> serde_json::Value {
        serde_json::json!({"enabled":self.enabled,"provider":self.provider,"endpoint":self.endpoint,
            "siteId":self.site_id,"allowHttp":self.allow_http,"apiSecretConfigured":self.api_secret_sealed.is_some()})
    }
}

#[derive(Default, Deserialize)]
#[serde(default, rename_all = "camelCase", deny_unknown_fields)]
pub struct Update {
    pub enabled: bool,
    pub provider: Provider,
    pub endpoint: String,
    pub site_id: String,
    pub allow_http: bool,
    pub api_secret: Option<String>,
    pub clear_api_secret: bool,
}
impl Update {
    pub fn prepare(
        self,
        previous: &Config,
        store: &mw_store::Store,
    ) -> Result<Config, &'static str> {
        if self.clear_api_secret && self.api_secret.is_some() {
            return Err("Choose either replacement or removal of the API secret.");
        }
        let mut config = Config {
            enabled: self.enabled,
            provider: self.provider,
            endpoint: self.endpoint,
            site_id: self.site_id,
            allow_http: self.allow_http,
            api_secret_sealed: None,
        };
        if config.provider == Provider::Ga4 {
            if let Some(secret) = self.api_secret {
                if secret.is_empty()
                    || secret.len() > 256
                    || !secret
                        .bytes()
                        .all(|b| b.is_ascii_alphanumeric() || b == b'-' || b == b'_')
                {
                    return Err(
                        "Enter a valid GA4 API secret (up to 256 letters, digits, hyphens or underscores).",
                    );
                }
                config.api_secret_sealed = Some(
                    store
                        .seal_analytics_secret(secret.as_bytes(), &config.endpoint, &config.site_id)
                        .map_err(|_| "Unable to protect analytics credentials.")?,
                );
            } else if !self.clear_api_secret
                && previous.provider == config.provider
                && previous.endpoint == config.endpoint
                && previous.site_id == config.site_id
            {
                config.api_secret_sealed = previous.api_secret_sealed.clone();
            }
        } else if self.api_secret.is_some() {
            return Err("API secrets are only supported for GA4.");
        }
        config.validate()?;
        Ok(config)
    }
}
pub async fn load(store: &mw_store::Store) -> Result<Config, ()> {
    let config: Config = match store.get_setting(KEY).await.map_err(|_| ())? {
        Some(raw) => serde_json::from_str(&raw).map_err(|_| ())?,
        None => Config::default(),
    };
    config.validate().map_err(|_| ())?;
    Ok(config)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn disabled_default_and_explicit_transport_contract() {
        assert!(!Config::default().enabled);
        assert!(Config::default().validate().is_ok());
        let mut config = Config {
            enabled: true,
            endpoint: "https://stats.example/api/send".into(),
            site_id: "f2997167-36cd-4e06-9054-abcc5c8cf476".into(),
            ..Config::default()
        };
        assert!(config.validate().is_ok());
        for endpoint in [
            "http://stats.example/api/send",
            "https://user:secret@stats.example/api/send",
            "https://stats.example/api/send?token=secret",
            "https://stats.example/api/send#secret",
            "file:///api/send",
        ] {
            config.endpoint = endpoint.into();
            assert!(config.validate().is_err(), "{endpoint}");
        }
        config.endpoint = "http://stats.example/api/send".into();
        config.allow_http = true;
        assert!(config.validate().is_ok());
        config.provider = Provider::Matomo;
        config.endpoint = "https://stats.example/matomo.php".into();
        config.site_id = "1".into();
        assert!(config.validate().is_ok());
        config.site_id = "0".into();
        assert!(config.validate().is_err());
    }
    #[tokio::test]
    async fn ga4_secret_is_sealed_bound_redacted_and_revision_sensitive() {
        let store = mw_store::Store::open_in_memory(mw_store::ServerKey::generate())
            .await
            .unwrap();
        let update = |endpoint: &str, secret: Option<&str>, enabled: bool| -> Update {
            serde_json::from_value(
                serde_json::json!({"enabled":enabled,"provider":"ga4", "endpoint":endpoint,
                "siteId":"G-TEST123", "apiSecret":secret}),
            )
            .unwrap()
        };
        let endpoint = "https://www.google-analytics.com/mp/collect";
        let config = update(endpoint, Some("secret_fixture_only"), true)
            .prepare(&Config::default(), &store)
            .unwrap();
        let persisted = serde_json::to_string(&config).unwrap();
        assert!(!persisted.contains("secret_fixture_only"));
        store.set_setting(KEY, &persisted).await.unwrap();
        assert_eq!(load(&store).await.unwrap(), config);
        let sealed = config.api_secret_sealed.as_ref().unwrap();
        assert!(
            !sealed
                .windows(19)
                .any(|window| window == b"secret_fixture_only")
        );
        assert_eq!(
            store
                .open_analytics_secret(sealed, endpoint, "G-TEST123")
                .unwrap(),
            b"secret_fixture_only"
        );
        assert!(
            store
                .open_analytics_secret(sealed, endpoint, "G-OTHER")
                .is_err()
        );
        assert!(
            store
                .open_analytics_secret(
                    sealed,
                    "https://region1.google-analytics.com/mp/collect",
                    "G-TEST123"
                )
                .is_err()
        );
        let public = config.public().to_string();
        assert!(!public.contains("secret_fixture_only"));
        assert!(!public.contains("apiSecretSealed"));
        assert_eq!(config.public()["apiSecretConfigured"], true);
        assert!(!format!("{config:?}").contains("secret_fixture_only"));
        assert_eq!(
            update(endpoint, None, true)
                .prepare(&config, &store)
                .unwrap(),
            config
        );
        let rotated = update(endpoint, Some("replacement"), true)
            .prepare(&config, &store)
            .unwrap();
        assert_ne!(rotated, config);
        assert!(
            update(
                "https://region1.google-analytics.com/mp/collect",
                None,
                true
            )
            .prepare(&config, &store)
            .is_err()
        );
        let changed = update(
            "https://region1.google-analytics.com/mp/collect",
            None,
            false,
        )
        .prepare(&config, &store)
        .unwrap();
        assert!(changed.api_secret_sealed.is_none());
        for target in [
            "http://www.google-analytics.com/mp/collect",
            "https://www.google-analytics.com.evil.test/mp/collect",
            "https://www.google-analytics.com/mp/collect?api_secret=x",
            "https://www.google-analytics.com:443/mp/collect",
            "https://127.0.0.1/mp/collect",
        ] {
            assert!(
                update(target, Some("secret"), true)
                    .prepare(&config, &store)
                    .is_err()
            );
        }
        let mut clear = update(endpoint, None, false);
        clear.clear_api_secret = true;
        assert!(
            clear
                .prepare(&config, &store)
                .unwrap()
                .api_secret_sealed
                .is_none()
        );
        assert!(serde_json::from_str::<Update>(r#"{"apiSecretSealed":[1,2]}"#).is_err());
    }
}
