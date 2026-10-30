//! Where the country database comes from (t96): the providers, their download
//! URLs and licences, and the administrator's stored choice.
//!
//! The URLs and terms were checked on 2026-10-29; `docs/admin/country-database.md`
//! records them with their sources.

use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

/// The `settings` row that holds [`Config`].
pub(crate) const KEY: &str = "admin.geoip.v1";
/// Purpose label of the key that seals the licence key or token.
const SEAL_LABEL: &str = "geoip-licence-v1";

/// MaxMind's downloads redirect to presigned URLs on this storage host.
pub(crate) const MAXMIND_STORAGE_HOST: &str =
    "mm-prod-geoip-databases.a2649acb697e2c09b632799562c076f2.r2.cloudflarestorage.com";

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub(crate) enum Provider {
    /// DB-IP "IP to Country Lite": CC BY 4.0, monthly, no account.
    #[default]
    Dbip,
    /// MaxMind GeoLite2 Country: GeoLite2 EULA, needs a licence key.
    Maxmind,
    /// IPinfo Lite: CC BY-SA 4.0, needs an access token.
    Ipinfo,
    /// An operator mirror or another source, HTTPS only.
    Custom,
}

impl Provider {
    pub(crate) const ALL: [Provider; 4] = [
        Provider::Dbip,
        Provider::Maxmind,
        Provider::Ipinfo,
        Provider::Custom,
    ];

    pub(crate) fn as_str(self) -> &'static str {
        match self {
            Provider::Dbip => "dbip",
            Provider::Maxmind => "maxmind",
            Provider::Ipinfo => "ipinfo",
            Provider::Custom => "custom",
        }
    }

    /// Whether downloads need the sealed licence key (MaxMind) or token (IPinfo).
    pub(crate) fn needs_key(self) -> bool {
        matches!(self, Provider::Maxmind | Provider::Ipinfo)
    }

    /// What the console shows about the provider: its licence and the attribution
    /// the licence asks for. Static text; never a key.
    pub(crate) fn describe(self) -> Value {
        match self {
            Provider::Dbip => json!({
                "id": "dbip", "name": "DB-IP IP to Country Lite", "needsKey": false,
                "homepage": "https://db-ip.com/db/download/ip-to-country-lite",
                "licence": "CC BY 4.0", "licenceUrl": "https://creativecommons.org/licenses/by/4.0/",
                "attribution": "IP Geolocation by DB-IP", "attributionUrl": "https://db-ip.com",
                "updates": "monthly", "checksum": "structural"
            }),
            Provider::Maxmind => json!({
                "id": "maxmind", "name": "MaxMind GeoLite2 Country", "needsKey": true,
                "homepage": "https://dev.maxmind.com/geoip/geolite2-free-geolocation-data",
                "licence": "GeoLite2 End User License Agreement",
                "licenceUrl": "https://www.maxmind.com/en/geolite2/eula",
                "attribution": "This product includes GeoLite2 data created by MaxMind, available from https://www.maxmind.com",
                "attributionUrl": "https://www.maxmind.com",
                "updates": "twice weekly", "checksum": "sha256"
            }),
            Provider::Ipinfo => json!({
                "id": "ipinfo", "name": "IPinfo Lite", "needsKey": true,
                "homepage": "https://ipinfo.io/lite",
                "licence": "CC BY-SA 4.0", "licenceUrl": "https://creativecommons.org/licenses/by-sa/4.0/",
                "attribution": "IP address data powered by IPinfo", "attributionUrl": "https://ipinfo.io",
                "updates": "daily", "checksum": "structural"
            }),
            Provider::Custom => json!({
                "id": "custom", "name": "Custom URL", "needsKey": false,
                "homepage": null, "licence": null, "licenceUrl": null,
                "attribution": null, "attributionUrl": null,
                "updates": null, "checksum": "optional sha256"
            }),
        }
    }
}

/// The administrator's choice. Stored as JSON in `settings`; the key is sealed.
#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default, rename_all = "camelCase", deny_unknown_fields)]
pub(crate) struct Config {
    pub provider: Provider,
    /// MaxMind licence key or IPinfo token, sealed and bound to the provider.
    pub licence_key_sealed: Option<Vec<u8>>,
    pub custom_url: String,
    pub custom_checksum_url: String,
    pub custom_redirect_hosts: Vec<String>,
    /// PEM of an extra root certificate for a custom mirror (public data).
    pub custom_ca_pem: String,
    pub auto_update: bool,
    pub interval_days: u32,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            provider: Provider::Dbip,
            licence_key_sealed: None,
            custom_url: String::new(),
            custom_checksum_url: String::new(),
            custom_redirect_hosts: Vec::new(),
            custom_ca_pem: String::new(),
            auto_update: true,
            interval_days: 7,
        }
    }
}

impl std::fmt::Debug for Config {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("GeoipConfig")
            .field("provider", &self.provider)
            .field("autoUpdate", &self.auto_update)
            .field("intervalDays", &self.interval_days)
            .finish_non_exhaustive()
    }
}

fn https_url(raw: &str) -> Result<reqwest::Url, &'static str> {
    let url = reqwest::Url::parse(raw).map_err(|_| "Enter an absolute HTTPS URL.")?;
    if url.scheme() != "https"
        || url.host_str().is_none()
        || !url.username().is_empty()
        || url.password().is_some()
        || url.fragment().is_some()
    {
        return Err("Use an HTTPS URL without credentials or fragment.");
    }
    Ok(url)
}

fn valid_host(host: &str) -> bool {
    !host.is_empty()
        && host.len() <= 253
        && host
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || b == b'-' || b == b'.')
        && !host.starts_with('.')
        && !host.ends_with('.')
}

impl Config {
    pub(crate) fn validate(&self) -> Result<(), &'static str> {
        if !(1..=90).contains(&self.interval_days) {
            return Err("Choose an update interval from 1 to 90 days.");
        }
        if self.custom_url.len() > 2048
            || self.custom_checksum_url.len() > 2048
            || self.custom_ca_pem.len() > 16384
            || self.custom_redirect_hosts.len() > 8
        {
            return Err("The custom source settings are too long.");
        }
        if self
            .custom_redirect_hosts
            .iter()
            .any(|h| !valid_host(h) || h.to_ascii_lowercase() != *h)
        {
            return Err("Redirect hosts are lower-case host names, one per entry.");
        }
        if self.provider == Provider::Custom {
            if self.custom_url.is_empty() {
                return Err("Enter the download URL of the custom source.");
            }
            https_url(&self.custom_url)?;
            if !self.custom_checksum_url.is_empty() {
                https_url(&self.custom_checksum_url)?;
            }
            if !self.custom_ca_pem.trim().is_empty()
                && pem_certificates(&self.custom_ca_pem).is_err()
            {
                return Err("The CA certificate is not a PEM certificate.");
            }
        }
        Ok(())
    }

    /// The configuration as the console sees it: never the key itself, only
    /// whether one is stored.
    pub(crate) fn public(&self) -> Value {
        json!({
            "provider": self.provider,
            "licenceKeyConfigured": self.licence_key_sealed.is_some(),
            "customUrl": self.custom_url,
            "customChecksumUrl": self.custom_checksum_url,
            "customRedirectHosts": self.custom_redirect_hosts,
            "customCaConfigured": !self.custom_ca_pem.trim().is_empty(),
            "autoUpdate": self.auto_update,
            "intervalDays": self.interval_days,
        })
    }

    /// The licence key or token, opened for one download. Never logged.
    pub(crate) fn licence_key(&self, store: &mw_store::Store) -> Option<String> {
        let sealed = self.licence_key_sealed.as_ref()?;
        let key = sealing_key(store);
        let plain = key
            .open_bound(sealed, SEAL_LABEL, &[self.provider.as_str()])
            .ok()?;
        String::from_utf8(plain).ok()
    }

    /// The extra root certificates (DER) of a custom source.
    pub(crate) fn custom_roots(&self) -> Vec<Vec<u8>> {
        if self.provider != Provider::Custom || self.custom_ca_pem.trim().is_empty() {
            return Vec::new();
        }
        pem_certificates(&self.custom_ca_pem).unwrap_or_default()
    }
}

fn sealing_key(store: &mw_store::Store) -> mw_store::ServerKey {
    mw_store::ServerKey::from_bytes(&store.derive_key(SEAL_LABEL)).expect("32-byte derived key")
}

/// Parse PEM `CERTIFICATE` blocks into DER.
pub(crate) fn pem_certificates(pem: &str) -> Result<Vec<Vec<u8>>, ()> {
    use base64::Engine as _;
    let mut out = Vec::new();
    let mut body: Option<String> = None;
    for line in pem.lines().map(str::trim) {
        match (line, body.as_mut()) {
            ("-----BEGIN CERTIFICATE-----", None) => body = Some(String::new()),
            ("-----END CERTIFICATE-----", Some(b)) => {
                let der = base64::engine::general_purpose::STANDARD
                    .decode(b.as_bytes())
                    .map_err(|_| ())?;
                if der.is_empty() {
                    return Err(());
                }
                out.push(der);
                body = None;
            }
            (l, Some(b)) => b.push_str(l),
            ("", None) => {}
            (_, None) => return Err(()),
        }
    }
    if body.is_some() || out.is_empty() || out.len() > 4 {
        return Err(());
    }
    Ok(out)
}

/// A configuration change from the console.
#[derive(Deserialize)]
#[serde(default, rename_all = "camelCase", deny_unknown_fields)]
pub(crate) struct Update {
    pub provider: Provider,
    /// A new licence key or token; `None` keeps the stored one (same provider).
    pub licence_key: Option<String>,
    pub clear_licence_key: bool,
    pub custom_url: String,
    pub custom_checksum_url: String,
    pub custom_redirect_hosts: Vec<String>,
    /// `None` keeps the stored CA; `Some("")` removes it.
    pub custom_ca_pem: Option<String>,
    pub auto_update: bool,
    pub interval_days: u32,
}

impl Default for Update {
    fn default() -> Self {
        let c = Config::default();
        Self {
            provider: c.provider,
            licence_key: None,
            clear_licence_key: false,
            custom_url: String::new(),
            custom_checksum_url: String::new(),
            custom_redirect_hosts: Vec::new(),
            custom_ca_pem: None,
            auto_update: c.auto_update,
            interval_days: c.interval_days,
        }
    }
}

impl Update {
    pub(crate) fn prepare(
        self,
        previous: &Config,
        store: &mw_store::Store,
    ) -> Result<Config, &'static str> {
        if self.clear_licence_key && self.licence_key.is_some() {
            return Err("Choose either a new licence key or its removal.");
        }
        let mut config = Config {
            provider: self.provider,
            licence_key_sealed: None,
            custom_url: self.custom_url.trim().to_string(),
            custom_checksum_url: self.custom_checksum_url.trim().to_string(),
            custom_redirect_hosts: self
                .custom_redirect_hosts
                .iter()
                .map(|h| h.trim().to_ascii_lowercase())
                .filter(|h| !h.is_empty())
                .collect(),
            custom_ca_pem: self
                .custom_ca_pem
                .unwrap_or_else(|| previous.custom_ca_pem.clone()),
            auto_update: self.auto_update,
            interval_days: self.interval_days,
        };
        if let Some(key) = self.licence_key {
            if !config.provider.needs_key() {
                return Err("Only MaxMind and IPinfo use a licence key.");
            }
            let key = key.trim();
            if key.is_empty()
                || key.len() > 128
                || !key
                    .bytes()
                    .all(|b| b.is_ascii_alphanumeric() || b == b'_' || b == b'-')
            {
                return Err("Enter a licence key of up to 128 letters, digits, '-' or '_'.");
            }
            config.licence_key_sealed = Some(
                sealing_key(store)
                    .seal_bound(key.as_bytes(), SEAL_LABEL, &[config.provider.as_str()])
                    .map_err(|_| "Unable to protect the licence key.")?,
            );
        } else if !self.clear_licence_key && previous.provider == config.provider {
            config.licence_key_sealed = previous.licence_key_sealed.clone();
        }
        config.validate()?;
        Ok(config)
    }
}

pub(crate) async fn load(store: &mw_store::Store) -> Result<Config, ()> {
    let config: Config = match store.get_setting(KEY).await.map_err(|_| ())? {
        Some(raw) => serde_json::from_str(&raw).map_err(|_| ())?,
        None => Config::default(),
    };
    config.validate().map_err(|_| ())?;
    Ok(config)
}

/// One download to make: the URL (which may carry the key, so it is never
/// logged, stored or shown), how to unpack the result and how to check it.
pub(crate) struct Plan {
    pub url: reqwest::Url,
    /// A label for status and audit rows: the URL without its query.
    pub label: String,
    pub checksum_url: Option<reqwest::Url>,
    pub redirect_hosts: Vec<String>,
    pub roots: Vec<Vec<u8>>,
    /// The provider's release name when the URL names one (DB-IP's `YYYY-MM`).
    pub release: Option<String>,
}

fn label_of(url: &reqwest::Url) -> String {
    let mut u = url.clone();
    u.set_query(None);
    u.to_string()
}

/// DB-IP publishes one file per month, `dbip-country-lite-YYYY-MM.mmdb.gz`.
pub(crate) fn dbip_url(year: i32, month: u32) -> String {
    format!("https://download.db-ip.com/free/dbip-country-lite-{year:04}-{month:02}.mmdb.gz")
}

/// The downloads to try, in order (DB-IP: this month, then last month, because a
/// new month's file appears during its first day).
pub(crate) fn plans(
    config: &Config,
    store: &mw_store::Store,
    now: chrono::DateTime<chrono::Utc>,
) -> Result<Vec<Plan>, &'static str> {
    use chrono::Datelike;
    let plan = |url: reqwest::Url,
                checksum: Option<reqwest::Url>,
                hosts: Vec<String>,
                release: Option<String>| Plan {
        label: label_of(&url),
        url,
        checksum_url: checksum,
        redirect_hosts: hosts,
        roots: config.custom_roots(),
        release,
    };
    match config.provider {
        Provider::Dbip => {
            let (y, m) = (now.year(), now.month());
            let (py, pm) = if m == 1 { (y - 1, 12) } else { (y, m - 1) };
            Ok([(y, m), (py, pm)]
                .into_iter()
                .map(|(y, m)| {
                    plan(
                        reqwest::Url::parse(&dbip_url(y, m)).expect("static URL"),
                        None,
                        Vec::new(),
                        Some(format!("{y:04}-{m:02}")),
                    )
                })
                .collect())
        }
        Provider::Maxmind => {
            let key = config
                .licence_key(store)
                .ok_or("Enter a MaxMind licence key first.")?;
            let mut url = reqwest::Url::parse("https://download.maxmind.com/app/geoip_download")
                .expect("static URL");
            url.query_pairs_mut()
                .append_pair("edition_id", "GeoLite2-Country")
                .append_pair("license_key", &key)
                .append_pair("suffix", "tar.gz");
            let mut sum = url.clone();
            sum.query_pairs_mut().clear();
            sum.query_pairs_mut()
                .append_pair("edition_id", "GeoLite2-Country")
                .append_pair("license_key", &key)
                .append_pair("suffix", "tar.gz.sha256");
            Ok(vec![plan(
                url,
                Some(sum),
                vec![MAXMIND_STORAGE_HOST.to_string()],
                None,
            )])
        }
        Provider::Ipinfo => {
            let key = config
                .licence_key(store)
                .ok_or("Enter an IPinfo access token first.")?;
            let mut url =
                reqwest::Url::parse("https://ipinfo.io/data/ipinfo_lite.mmdb").expect("static URL");
            url.query_pairs_mut().append_pair("token", &key);
            Ok(vec![plan(url, None, Vec::new(), None)])
        }
        Provider::Custom => {
            let url = https_url(&config.custom_url)?;
            let checksum = if config.custom_checksum_url.is_empty() {
                None
            } else {
                Some(https_url(&config.custom_checksum_url)?)
            };
            Ok(vec![plan(
                url,
                checksum,
                config.custom_redirect_hosts.clone(),
                None,
            )])
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    async fn store() -> mw_store::Store {
        mw_store::Store::open_in_memory(mw_store::ServerKey::generate())
            .await
            .unwrap()
    }

    #[test]
    fn dbip_url_names_the_month() {
        assert_eq!(
            dbip_url(2026, 10),
            "https://download.db-ip.com/free/dbip-country-lite-2026-10.mmdb.gz"
        );
    }

    #[tokio::test]
    async fn dbip_plans_try_this_month_then_last_month() {
        let store = store().await;
        let now = chrono::DateTime::parse_from_rfc3339("2027-01-01T00:30:00Z")
            .unwrap()
            .with_timezone(&chrono::Utc);
        let plans = plans(&Config::default(), &store, now).unwrap();
        let urls: Vec<_> = plans.iter().map(|p| p.url.to_string()).collect();
        assert_eq!(
            urls,
            [
                "https://download.db-ip.com/free/dbip-country-lite-2027-01.mmdb.gz",
                "https://download.db-ip.com/free/dbip-country-lite-2026-12.mmdb.gz"
            ]
        );
        assert_eq!(plans[1].release.as_deref(), Some("2026-12"));
    }

    #[tokio::test]
    async fn licence_key_is_sealed_bound_to_provider_and_never_public() {
        let store = store().await;
        let update = Update {
            provider: Provider::Maxmind,
            licence_key: Some("SeCrEt_Key-123".into()),
            ..Update::default()
        };
        let config = update.prepare(&Config::default(), &store).unwrap();
        let raw = serde_json::to_string(&config).unwrap();
        assert!(!raw.contains("SeCrEt_Key-123"));
        assert!(!config.public().to_string().contains("SeCrEt_Key-123"));
        assert!(!format!("{config:?}").contains("SeCrEt_Key-123"));
        assert_eq!(
            config.licence_key(&store).as_deref(),
            Some("SeCrEt_Key-123")
        );
        let plans = plans(&config, &store, chrono::Utc::now()).unwrap();
        assert!(plans[0].url.as_str().contains("license_key=SeCrEt_Key-123"));
        assert!(!plans[0].label.contains("SeCrEt"));
        assert!(!plans[0].checksum_url.as_ref().unwrap().as_str().is_empty());
        // Changing provider drops the key; it was bound to MaxMind.
        let moved = Update {
            provider: Provider::Ipinfo,
            ..Update::default()
        }
        .prepare(&config, &store)
        .unwrap();
        assert!(moved.licence_key_sealed.is_none());
        // A key moved across providers by hand does not open.
        let mut forged = config.clone();
        forged.provider = Provider::Ipinfo;
        assert!(forged.licence_key(&store).is_none());
    }

    #[test]
    fn custom_source_needs_https_and_valid_hosts() {
        let mut c = Config {
            provider: Provider::Custom,
            custom_url: "http://mirror.example/db.mmdb".into(),
            ..Config::default()
        };
        assert!(c.validate().is_err());
        c.custom_url = "https://user:pw@mirror.example/db.mmdb".into();
        assert!(c.validate().is_err());
        c.custom_url = "https://mirror.example/db.mmdb".into();
        assert!(c.validate().is_ok());
        c.custom_redirect_hosts = vec!["Bad Host".into()];
        assert!(c.validate().is_err());
        c.custom_redirect_hosts = vec!["cdn.mirror.example".into()];
        assert!(c.validate().is_ok());
        c.interval_days = 0;
        assert!(c.validate().is_err());
    }
}
