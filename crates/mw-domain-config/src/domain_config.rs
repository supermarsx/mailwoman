//! Canonical login identities and operator-owned upstream configuration.
//! No DNS lookup is performed while selecting a domain.
pub use crate::domain_security::ConnectionSecurity;
use mw_store::{DomainConfigRow, Store};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum TlsMode {
    Implicit,
    Starttls,
    None,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Endpoint {
    pub host: String,
    pub port: u16,
    pub tls: TlsMode,
    #[serde(default = "password_auth")]
    pub auth: String,
    #[serde(default)]
    pub allow_private: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub security: Option<ConnectionSecurity>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub authentication: Option<crate::domain_auth::PasswordAuthentication>,
}
fn password_auth() -> String {
    "password".into()
}

impl Endpoint {
    pub fn auth_policy(
        &self,
        protocol: &str,
    ) -> Result<Option<mw_net::PasswordAuthPolicy>, String> {
        self.authentication
            .as_ref()
            .map(|auth| auth.policy(crate::domain_auth::protocol(protocol)?))
            .transpose()
    }

    pub fn validate_for(&self, protocol: &str) -> Result<(), String> {
        self.validate()?;
        if let Some(policy) = self
            .security
            .as_ref()
            .and_then(|security| security.smtp_policy.as_ref())
        {
            if protocol != "smtp" {
                return Err(
                    "DANE and MTA-STS policies are only available for SMTP endpoints".into(),
                );
            }
            if policy.enabled() {
                mw_mail_transport::normalize_domain(&self.host)
                    .map_err(|_| "SMTP policy verification requires a DNS hostname")?;
            }
        }
        self.auth_policy(protocol)?;
        Ok(())
    }
    pub fn validate(&self) -> Result<(), String> {
        if let Some(security) = &self.security {
            security.validate()?;
        }
        if self.port == 0 {
            return Err("the port must be between 1 and 65535".into());
        }
        if self.auth != "password" {
            return Err("only password authentication is supported".into());
        }
        let host = self.host.trim();
        if host.is_empty() || host != self.host || host.contains(['/', '@', '?', '#', '\\']) {
            return Err("the upstream host must be a hostname or IP address".into());
        }
        let host = reqwest::Url::parse(&format!("https://{}/", bracket_host(host)))
            .map_err(|_| "the upstream host is invalid")?;
        if host.host_str().is_none() || host.port().is_some() {
            return Err("put the port in its own field".into());
        }
        if !self.allow_private {
            let name = self.host.trim_matches(['[', ']']);
            if name.eq_ignore_ascii_case("localhost")
                || name.ends_with(".localhost")
                || name
                    .parse::<std::net::IpAddr>()
                    .is_ok_and(|ip| !mw_egress::ip_allowed(&ip))
            {
                return Err("a private upstream needs explicit operator permission".into());
            }
        }
        Ok(())
    }

    pub fn mail_url(&self, protocol: &str) -> String {
        let suffix = if self.tls == TlsMode::Implicit {
            "s"
        } else {
            ""
        };
        format!(
            "{protocol}{suffix}://{}:{}",
            bracket_host(&self.host),
            self.port
        )
    }

    pub fn transport_security(&self) -> &'static str {
        match self.tls {
            TlsMode::Implicit => "implicit",
            TlsMode::Starttls => "starttls",
            TlsMode::None => "plaintext",
        }
    }
}

fn bracket_host(host: &str) -> String {
    if host.contains(':') && !host.starts_with('[') {
        format!("[{host}]")
    } else {
        host.into()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct JmapEndpoint {
    pub session_url: String,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct DavEndpoints {
    pub caldav_url: Option<String>,
    pub carddav_url: Option<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct UpstreamConfig {
    pub imap: Option<Endpoint>,
    pub pop3: Option<Endpoint>,
    pub smtp: Option<Endpoint>,
    pub jmap: Option<JmapEndpoint>,
    #[serde(rename = "jmapSubmission", skip_serializing_if = "Option::is_none")]
    pub jmap_submission: Option<JmapEndpoint>,
    #[serde(skip)]
    pub ews: Option<crate::ews::EwsConfig>,
    pub sieve: Option<Endpoint>,
    pub dav: Option<DavEndpoints>,
    #[serde(skip)]
    pub selection: Option<crate::domain_selection::Selection>,
    /// The failover chain (backend ID, endpoint), selected backend first. Empty
    /// unless failover is on and at least one fallback is enabled.
    #[serde(skip)]
    pub mail_chain: Vec<(String, Endpoint)>,
    /// Set when failover connected to a fallback: the selected backend's
    /// endpoint, which stays the account's identity. `imap`/`pop3` then hold the
    /// fallback that is dialled.
    #[serde(skip)]
    pub mail_primary: Option<Endpoint>,
}

impl UpstreamConfig {
    pub fn parse(json: &str) -> Result<Self, String> {
        let value: serde_json::Value =
            serde_json::from_str(json).map_err(|_| "invalid upstream configuration")?;
        if value.get("version").is_some() {
            let config: crate::domain_selection::BackendConfig = serde_json::from_value(value)
                .map_err(|_| "invalid versioned backend configuration")?;
            return config.resolve();
        }
        let config: Self =
            serde_json::from_str(json).map_err(|_| "invalid upstream configuration".to_string())?;
        for (protocol, endpoint) in [
            ("imap", &config.imap),
            ("pop3", &config.pop3),
            ("smtp", &config.smtp),
            ("sieve", &config.sieve),
        ] {
            if let Some(endpoint) = endpoint {
                endpoint.validate_for(protocol)?;
            }
        }
        if config.imap.is_some() && config.pop3.is_some() {
            return Err("choose either IMAP or POP3".into());
        }
        if config.smtp.is_some() && config.jmap_submission.is_some() {
            return Err("choose SMTP or JMAP for submission".into());
        }
        if let Some(jmap) = &config.jmap_submission {
            validate_http_url(&jmap.session_url)?;
        }
        if let Some(jmap) = &config.jmap {
            validate_http_url(&jmap.session_url)?;
        }
        if let Some(dav) = &config.dav {
            for url in [&dav.caldav_url, &dav.carddav_url].into_iter().flatten() {
                validate_http_url(url)?;
            }
        }
        Ok(config)
    }

    /// The original account identity is preserved; only the wire login is formatted.
    pub fn auth_username(
        &self,
        role: crate::domain_selection::ServiceRole,
        identity: &str,
    ) -> Result<String, String> {
        match self.selection.as_ref().and_then(|s| s.backend(role)) {
            Some(backend) => backend.username.render(identity),
            None => Ok(identity.into()),
        }
    }

    /// A stable, opaque binding to selected authorities and authentication policy.
    pub fn binding(&self) -> Result<Option<String>, String> {
        let bytes = if let Some(selection) = &self.selection {
            serde_json::to_vec(selection)
        } else if self.jmap_submission.is_some()
            || [&self.imap, &self.pop3, &self.smtp, &self.sieve]
                .into_iter()
                .flatten()
                .any(|endpoint| endpoint.security.is_some() || endpoint.authentication.is_some())
        {
            serde_json::to_vec(self)
        } else {
            return Ok(None);
        }
        .map_err(|_| "invalid selected backend configuration")?;
        Ok(Some(
            Sha256::digest(bytes)
                .iter()
                .map(|byte| format!("{byte:02x}"))
                .collect(),
        ))
    }

    /// Host and port that identify the account: the selected backend's, even
    /// when failover dialled a fallback.
    pub fn mail_identity(&self) -> Option<(&str, u16)> {
        self.mail_primary
            .as_ref()
            .map(|e| (e.host.as_str(), e.port))
            .or_else(|| self.mail().map(|(e, _)| (e.host.as_str(), e.port)))
    }

    pub fn mail(&self) -> Option<(&Endpoint, &'static str)> {
        self.imap
            .as_ref()
            .map(|e| (e, "imap"))
            .or_else(|| self.pop3.as_ref().map(|e| (e, "pop3")))
    }
}

pub fn validate_http_url(value: &str) -> Result<(), String> {
    let url = reqwest::Url::parse(value).map_err(|_| "invalid upstream URL")?;
    if !matches!(url.scheme(), "http" | "https")
        || url.host_str().is_none()
        || !url.username().is_empty()
        || url.password().is_some()
        || url.fragment().is_some()
    {
        return Err("an upstream URL must use HTTP(S) without credentials or fragment".into());
    }
    Ok(())
}

/// IDNA-normalized domain, rejecting URLs, paths, ports and mailbox syntax.
pub fn normalize_domain(input: &str) -> Result<String, String> {
    mw_store::settings_policy::normalize_policy_domain(input)
        .map_err(|_| "invalid domain name".to_string())
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Canonical {
    pub address: String,
    pub domain: String,
    pub login_name: String,
}

/// Pure qualified-name parsing shared by fresh domain admission readers.
/// `None` retains the separate bare-name expansion path.
pub struct QualifiedIdentityInput {
    typed: String,
    local: String,
    pub domain: String,
}
impl QualifiedIdentityInput {
    pub fn parse(typed: &str) -> Result<Option<Self>, String> {
        let typed = typed.trim();
        if typed.is_empty() || typed.chars().any(char::is_control) {
            return Err("invalid login name".into());
        }
        let Some((local, domain)) = typed.rsplit_once('@') else { return Ok(None); };
        if local.is_empty() || local.contains('@') { return Err("invalid login name".into()); }
        Ok(Some(Self {typed:typed.into(),local:local.to_lowercase(),domain:normalize_domain(domain)?}))
    }
    pub fn canonical(&self, domain: String) -> Canonical {
        Canonical {address:format!("{}@{domain}",self.local),domain,login_name:self.typed.clone()}
    }
}

#[cfg(test)]
mod qualified_identity_input_tests {
    use super::*;
    #[test]
    fn shared_qualified_parser_preserves_canonical_identity_and_bare_fallback() {
        let input=QualifiedIdentityInput::parse("  NÉW+Tag@BÜCHER.Test  ").unwrap().unwrap();
        assert_eq!(input.domain,"xn--bcher-kva.test");
        assert_eq!(input.canonical("canonical.test".into()),Canonical {
            address:"néw+tag@canonical.test".into(),domain:"canonical.test".into(),login_name:"NÉW+Tag@BÜCHER.Test".into(),
        });
        assert!(QualifiedIdentityInput::parse("alice").unwrap().is_none());
        for name in ["", "a@@example.test", "@example.test", "a@bad/path", "a\n@example.test"] {
            assert!(QualifiedIdentityInput::parse(name).is_err(),"{name:?}");
        }
    }
}

/// Fresh request-local domain inputs, never retained as an authorization cache.
/// Upstream JSON is deliberately unparsed: trusted header overrides still need
/// address admission without consulting the ordinary configured upstream.
pub struct IdentitySnapshot {
    pub identity: Canonical,
    pub row: Option<DomainConfigRow>,
    typed: String,
    existing_transport_compatible: bool,
}

impl IdentitySnapshot {
    /// Assemble the same qualified observation used by the ordinary reader.
    /// The caller must obtain the alias and row together from fresh Store data.
    pub fn from_qualified(
        input: QualifiedIdentityInput,
        domain: String,
        row: Option<DomainConfigRow>,
        second_alias: bool,
    ) -> Self {
        let existing_transport_compatible =
            !second_alias && normalize_domain(&domain).is_ok_and(|normalized| normalized == domain);
        Self {
            typed: input.typed.clone(),
            identity: input.canonical(domain),
            row,
            existing_transport_compatible,
        }
    }
    pub fn matches_existing_identity(&self, address: &str) -> bool {
        self.existing_transport_compatible && address.trim() == self.typed
    }

    pub fn signin_allowed(&self) -> Result<bool, String> {
        signin_allowed_row(&self.identity, self.row.as_ref())
    }
}

pub async fn identity_snapshot(store: &Store, typed: &str) -> Result<IdentitySnapshot, String> {
    let typed = typed.trim();
    if typed.is_empty() || typed.chars().any(char::is_control) {
        return Err("invalid login name".into());
    }
    let Some(input) = QualifiedIdentityInput::parse(typed)? else {
        // Bare login expansion and existing bare transport lookup intentionally
        // differ; preserve the old preferred-domain behavior and never reuse.
        let identity = canonical(store, typed, None).await?;
        let row = configured_row(store, &identity).await?;
        return Ok(IdentitySnapshot {
            identity,
            row,
            typed: typed.into(),
            existing_transport_compatible: false,
        });
    };
    let (domain, row, second_alias) = store
        .domain_alias_config_snapshot(&input.domain)
        .await
        .map_err(|_| "domain lookup failed")?;
    Ok(IdentitySnapshot::from_qualified(input, domain, row, second_alias))
}

pub async fn canonical(
    store: &Store,
    typed: &str,
    host: Option<&str>,
) -> Result<Canonical, String> {
    let typed = typed.trim();
    if typed.is_empty() || typed.chars().any(char::is_control) {
        return Err("invalid login name".into());
    }
    let expanded = if typed.contains('@') {
        typed.to_string()
    } else {
        let mapped = if let Some(host) = host {
            let host = reqwest::Url::parse(&format!("https://{host}/"))
                .ok()
                .and_then(|u| u.host_str().map(str::to_string));
            match host {
                Some(h) => store
                    .domain_for_host(&h)
                    .await
                    .map_err(|_| "domain lookup failed")?,
                None => None,
            }
        } else {
            None
        };
        let preferred = match mapped {
            Some(domain) => Some(domain),
            None => store
                .preferred_domain()
                .await
                .map_err(|_| "domain lookup failed")?,
        };
        preferred.map_or_else(|| typed.to_string(), |domain| format!("{typed}@{domain}"))
    };
    let Some((local, raw_domain)) = expanded.rsplit_once('@') else {
        return Ok(Canonical {
            address: expanded.to_lowercase(),
            domain: String::new(),
            login_name: typed.into(),
        });
    };
    if local.is_empty() || local.contains('@') {
        return Err("invalid login name".into());
    }
    let raw_domain = normalize_domain(raw_domain)?;
    let domain = store
        .domain_alias_target(&raw_domain)
        .await
        .map_err(|_| "domain lookup failed")?
        .unwrap_or(raw_domain);
    Ok(Canonical {
        address: format!("{}@{domain}", local.to_lowercase()),
        domain,
        login_name: expanded,
    })
}

pub async fn upstream_for(store: &Store, address: &str) -> Result<Option<UpstreamConfig>, String> {
    // This is an existing-identity transport lookup, not a new login. Only the
    // login handlers may expand a bare username using Host/preferred domain.
    let address = address.trim();
    let Some((local, domain)) = address.rsplit_once('@') else {
        return Ok(None);
    };
    if local.is_empty()
        || domain.trim().is_empty()
        || local.contains('@')
        || address.chars().any(char::is_control)
    {
        return Err("invalid login name".into());
    }
    let domain = store
        .canonical_policy_domain(domain)
        .await
        .map_err(|_| "domain lookup failed")?;
    let row = store
        .get_domain_config(&domain)
        .await
        .map_err(|_| "domain lookup failed")?;
    match row {
        Some(row) if !row.enabled => Err("domain is disabled".into()),
        Some(row) => UpstreamConfig::parse(&row.upstream_json).map(Some),
        None => Ok(None),
    }
}

pub async fn configured_row(
    store: &Store,
    identity: &Canonical,
) -> Result<Option<DomainConfigRow>, String> {
    store
        .get_domain_config(&identity.domain)
        .await
        .map_err(|_| "domain lookup failed".into())
}

/// Domain lists are checked only after credentials have been proved.
pub async fn signin_allowed(store: &Store, username: &str) -> Result<bool, String> {
    let identity = canonical(store, username, None).await?;
    let row = configured_row(store, &identity).await?;
    signin_allowed_row(&identity, row.as_ref())
}

fn signin_allowed_row(identity: &Canonical, row: Option<&DomainConfigRow>) -> Result<bool, String> {
    let Some(row) = row else {
        return Ok(true);
    };
    let allowed: Vec<String> =
        serde_json::from_str(&row.allowlist_json).map_err(|_| "invalid domain allowlist")?;
    let blocked: Vec<String> =
        serde_json::from_str(&row.blocklist_json).map_err(|_| "invalid domain blocklist")?;
    Ok(row.enabled
        && !blocked
            .iter()
            .any(|pattern| address_matches(pattern, &identity.address))
        && (allowed.is_empty()
            || allowed
                .iter()
                .any(|pattern| address_matches(pattern, &identity.address))))
}

fn address_matches(pattern: &str, address: &str) -> bool {
    let pattern = pattern.to_lowercase();
    let address = address.to_lowercase();
    let parts: Vec<&str> = pattern.split('*').collect();
    if parts.len() == 1 {
        return pattern == address;
    }
    let Some(mut tail) = address.strip_prefix(parts[0]) else {
        return false;
    };
    for part in &parts[1..parts.len() - 1] {
        let Some(index) = tail.find(part) else {
            return false;
        };
        tail = &tail[index + part.len()..];
    }
    tail.ends_with(parts[parts.len() - 1])
}

#[cfg(test)]
mod tests {
    use super::*;
    #[tokio::test]
    async fn fresh_snapshot_matches_login_admission_without_parsing_upstream() {
        let store = Store::open_in_memory(mw_store::ServerKey::generate())
            .await
            .unwrap();
        let mut row = DomainConfigRow {
            name: "example.test".into(),
            upstream_json: "invalid upstream JSON".into(),
            allowlist_json: "[\"*@example.test\"]".into(),
            blocklist_json: "[\"blocked*@example.test\"]".into(),
            enabled: true,
            preferred: false,
            settings_json: "{}".into(),
            updated_at: 1,
        };
        store.put_domain_config(&row).await.unwrap();
        store
            .put_domain_alias("alias.test", "example.test")
            .await
            .unwrap();
        for address in [
            "User@Example.Test",
            "User@ALIAS.TEST.",
            "blocked-user@alias.test",
            "user@missing.test",
        ] {
            let (snapshot, queries) =
                mw_store::pool_telemetry::measure_queries(identity_snapshot(&store, address)).await;
            let snapshot = snapshot.unwrap();
            assert_eq!(queries.read.calls, 1);
            assert_eq!(
                snapshot.identity,
                canonical(&store, address, None).await.unwrap()
            );
            assert_eq!(
                snapshot.signin_allowed(),
                signin_allowed(&store, address).await
            );
            assert!(snapshot.matches_existing_identity(address));
            assert!(!snapshot.matches_existing_identity("other@example.test"));
        }
        // Header override admission must succeed even when ordinary routing JSON
        // is unusable. It is parsed only if a resolver actually needs it.
        assert!(
            identity_snapshot(&store, "user@example.test")
                .await
                .unwrap()
                .signin_allowed()
                .unwrap()
        );
        row.blocklist_json = "invalid".into();
        store.put_domain_config(&row).await.unwrap();
        assert!(
            identity_snapshot(&store, "user@example.test")
                .await
                .unwrap()
                .signin_allowed()
                .is_err()
        );
        row.blocklist_json = "[]".into();
        row.enabled = false;
        store.put_domain_config(&row).await.unwrap();
        assert!(
            !identity_snapshot(&store, "user@example.test")
                .await
                .unwrap()
                .signin_allowed()
                .unwrap()
        );
        row.enabled = true;
        store.put_domain_config(&row).await.unwrap();
        store
            .set_preferred_domain(Some("example.test"))
            .await
            .unwrap();
        let bare = identity_snapshot(&store, "user").await.unwrap();
        assert_eq!(bare.identity.address, "user@example.test");
        assert!(!bare.matches_existing_identity("user"));
        assert!(upstream_for(&store, "user").await.unwrap().is_none());
        for invalid in ["", "user@@example.test", "user@", "user\n@example.test"] {
            assert!(identity_snapshot(&store, invalid).await.is_err());
        }
    }

    #[tokio::test]
    async fn existing_bare_upstream_never_uses_changed_preferred_but_new_login_and_alias_work() {
        let store = Store::open_in_memory(mw_store::ServerKey::generate())
            .await
            .unwrap();
        for domain in ["first.test", "second.test"] {
            store.put_domain_config(&DomainConfigRow {
                name: domain.into(), upstream_json: serde_json::json!({"sieve":{"host":format!("sieve.{domain}"),"port":4190,"tls":"starttls"}}).to_string(),
                allowlist_json: "[]".into(), blocklist_json: "[]".into(), enabled: true, preferred: false, settings_json: "{}".into(), updated_at: 1,
            }).await.unwrap();
        }
        store
            .put_domain_alias("alias.test", "first.test")
            .await
            .unwrap();
        store
            .set_preferred_domain(Some("first.test"))
            .await
            .unwrap();
        assert!(upstream_for(&store, "user").await.unwrap().is_none());
        assert!(upstream_for(&store, "user@").await.is_err());
        store
            .set_preferred_domain(Some("second.test"))
            .await
            .unwrap();
        assert!(upstream_for(&store, "user").await.unwrap().is_none());
        assert_eq!(
            upstream_for(&store, "User@Alias.Test")
                .await
                .unwrap()
                .unwrap()
                .sieve
                .unwrap()
                .host,
            "sieve.first.test"
        );
        let login = canonical(&store, "user", None).await.unwrap();
        assert_eq!(login.login_name, "user@second.test");
        assert_eq!(
            upstream_for(&store, &login.login_name)
                .await
                .unwrap()
                .unwrap()
                .sieve
                .unwrap()
                .host,
            "sieve.second.test"
        );
        let mut first = store
            .get_domain_config("first.test")
            .await
            .unwrap()
            .unwrap();
        first.enabled = false;
        store.put_domain_config(&first).await.unwrap();
        assert!(upstream_for(&store, "user@alias.test").await.is_err());
    }
    #[test]
    fn address_patterns_are_anchored_and_case_insensitive() {
        for (pattern, address, expected) in [
            ("ALICE@example.org", "alice@EXAMPLE.ORG", true),
            ("*@example.org", "alice@example.org", true),
            ("*@example.org", "alice@example.org.evil", false),
            ("alice*smith@example.org", "alice.smith@example.org", true),
            ("alice*smith@example.org", "malice.smith@example.org", false),
            ("a*a", "a", false),
            ("a**a", "aa", true),
        ] {
            assert_eq!(
                address_matches(pattern, address),
                expected,
                "{pattern} {address}"
            );
        }
    }
    #[test]
    fn domains_and_endpoints_reject_authority_confusion() {
        assert_eq!(
            normalize_domain("BÜCHER.Example.").unwrap(),
            "xn--bcher-kva.example"
        );
        for invalid in ["a/b", "a@b", "a:443", "a%2eb", "-bad.test", "a..b"] {
            assert!(normalize_domain(invalid).is_err(), "{invalid}");
        }
        let endpoint = |host: &str| Endpoint {
            host: host.into(),
            port: 993,
            tls: TlsMode::Implicit,
            auth: "password".into(),
            allow_private: false,
            security: None,
            authentication: None,
        };
        assert!(endpoint("imap.example.org").validate().is_ok());
        for invalid in [
            "127.0.0.1",
            "localhost",
            "user@host",
            "host:123",
            "host/path",
        ] {
            assert!(endpoint(invalid).validate().is_err(), "{invalid}");
        }
        assert!(UpstreamConfig::parse(r#"{"imap":{"host":"mail.test","port":993,"tls":"implicit"},"pop3":{"host":"mail.test","port":995,"tls":"implicit"}}"#).is_err());
    }
}
