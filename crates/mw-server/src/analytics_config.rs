//! Deliberately separate from operational OpenTelemetry configuration.
use serde::{Deserialize, Serialize};

pub const KEY: &str = "admin.analytics.v1";
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Provider {
    #[default]
    Umami,
    Matomo,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default, rename_all = "camelCase", deny_unknown_fields)]
pub struct Config {
    pub enabled: bool,
    pub provider: Provider,
    pub endpoint: String,
    pub site_id: String,
    pub allow_http: bool,
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
                "Umami needs /api/send and a website UUID; Matomo needs /matomo.php and a positive site ID.",
            );
        }
        Ok(())
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
}
