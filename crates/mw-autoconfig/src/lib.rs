#![forbid(unsafe_code)]
//! `mw-autoconfig` — server discovery for a login email (plan §0/§2.4).
//!
//! [`discover`] walks a fallback ladder and returns the first
//! [`AccountCandidate`] it can build (SPEC §6.3):
//!
//! 1. **JMAP session autodiscovery** — `GET https://<domain>/.well-known/jmap`
//!    (RFC 8620 §2.2). A valid session resource short-circuits the ladder: for a
//!    JMAP-native host it is the authoritative source, so it is tried first.
//! 2. **RFC 6186 SRV** (`_imaps._tcp` / `_submission._tcp`, …), resolved live by
//!    [`HickoryResolver`] (pure-Rust DNS — see [`resolver`]). The resolver sits
//!    behind the injectable [`Fetcher`]/[`resolver::SrvResolver`] seam so the
//!    ladder is exercised against a stub with no live network.
//! 3. **Thunderbird autoconfig XML** — provider-hosted
//!    `https://autoconfig.<domain>/mail/config-v1.1.xml` then the ISPDB
//!    `https://autoconfig.thunderbird.net/v1.1/<domain>`.
//! 4. **MS Autodiscover v2** — `GET /autodiscover/autodiscover.json`.
//! 5. **Offline provider DB** — bundled JSON for the big providers.
//! 6. **Manual** — every rung missed; return [`DiscoverError::NotFound`] and
//!    the UI shows its manual fields.
//!
//! Network access is behind the [`Fetcher`] trait so the ladder is exercised
//! against recorded fixtures with no live network (see the crate tests).

mod provider;
mod resolver;
mod xml;

pub use resolver::{HickoryResolver, SrvResolver};

use async_trait::async_trait;
use serde::{Deserialize, Serialize};

/// Transport security for a discovered server.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum TlsMode {
    /// Implicit TLS from connect (IMAPS 993 / POP3S 995 / SMTPS 465).
    Implicit,
    /// Cleartext then STARTTLS upgrade (143 / 110 / 587).
    StartTls,
    /// No transport security (discouraged; manual only).
    None,
}

/// Authentication method a discovered server expects.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum AuthMethod {
    /// Password over a SASL PLAIN/LOGIN mechanism.
    Password,
    /// OAuth2 bearer via SASL XOAUTH2 (Gmail/Outlook).
    OAuth2,
}

/// Which rung of the discovery ladder produced the candidate.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum DiscoverySource {
    /// JMAP session autodiscovery (`/.well-known/jmap`, RFC 8620). For a
    /// `Jmap` candidate the `imap`/`smtp` [`ServerSpec`]s both carry the JMAP
    /// endpoint host (from the session `apiUrl`); the client re-fetches the
    /// session resource for the full `apiUrl`/capabilities.
    Jmap,
    /// RFC 6186 DNS SRV records.
    Srv,
    /// Thunderbird autoconfig XML (ISPDB / provider-hosted).
    ThunderbirdAutoconfig,
    /// Microsoft Autodiscover v2.
    Autodiscover,
    /// Offline bundled provider database.
    ProviderDb,
    /// User-entered manual configuration.
    Manual,
}

/// One discovered server endpoint.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ServerSpec {
    pub host: String,
    pub port: u16,
    pub tls: TlsMode,
}

/// The discovery result the login flow consumes (plan §2.4).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AccountCandidate {
    pub imap: ServerSpec,
    pub pop3: Option<ServerSpec>,
    pub smtp: ServerSpec,
    pub auth: AuthMethod,
    pub source: DiscoverySource,
}

/// Errors from discovery.
#[derive(Debug, thiserror::Error)]
pub enum DiscoverError {
    /// The email address was not a valid `local@domain`.
    #[error("invalid email address: {0}")]
    InvalidEmail(String),
    /// Every rung of the ladder failed; the UI falls back to manual entry.
    #[error("no configuration discovered for {0}")]
    NotFound(String),
    /// A network/lookup error while probing a discovery source.
    #[error("discovery lookup error: {0}")]
    Lookup(String),
}

/// A single DNS SRV record (RFC 2782), as returned by a [`Fetcher`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SrvRecord {
    pub target: String,
    pub port: u16,
    pub priority: u16,
    pub weight: u16,
}

/// The network seam the discovery ladder depends on. Injecting it lets the
/// ladder run against recorded fixtures with no live network.
#[async_trait]
pub trait Fetcher: Send + Sync {
    /// Resolve SRV records for a service name (e.g. `_imaps._tcp.example.org`).
    /// Return an empty vec when the record is absent or SRV is unsupported.
    async fn srv(&self, service: &str) -> Result<Vec<SrvRecord>, DiscoverError>;

    /// HTTP GET returning the body on `200`, `None` on any non-success or a
    /// recoverable transport failure (so the ladder simply moves on).
    async fn get(&self, url: &str) -> Result<Option<String>, DiscoverError>;
}

/// Split `local@domain`, validating both halves are non-empty.
fn split_email(email: &str) -> Result<(&str, &str), DiscoverError> {
    let (local, domain) = email
        .rsplit_once('@')
        .ok_or_else(|| DiscoverError::InvalidEmail(email.to_string()))?;
    if local.is_empty() || domain.is_empty() || domain.contains('@') || !domain.contains('.') {
        return Err(DiscoverError::InvalidEmail(email.to_string()));
    }
    Ok((local, domain))
}

/// Discover server configuration for `email` using live network access
/// (Thunderbird autoconfig + Autodiscover over HTTPS, then the offline DB).
pub async fn discover(email: &str) -> Result<AccountCandidate, DiscoverError> {
    let fetcher = ReqwestFetcher::new()?;
    discover_with(email, &fetcher).await
}

/// Discover using an injected [`Fetcher`] — the testable core of the ladder.
pub async fn discover_with(
    email: &str,
    fetcher: &dyn Fetcher,
) -> Result<AccountCandidate, DiscoverError> {
    let (_local, domain) = split_email(email)?;

    if let Some(c) = try_well_known_jmap(fetcher, domain).await {
        return Ok(c);
    }
    if let Some(c) = try_srv(fetcher, domain).await {
        return Ok(c);
    }
    if let Some(c) = try_autoconfig(fetcher, domain, email).await {
        return Ok(c);
    }
    if let Some(c) = try_autodiscover(fetcher, domain).await {
        return Ok(c);
    }
    if let Some(c) = provider::lookup(domain) {
        return Ok(c);
    }
    Err(DiscoverError::NotFound(domain.to_string()))
}

// ---- rung 1: JMAP .well-known/jmap session autodiscovery ------------------

async fn try_well_known_jmap(fetcher: &dyn Fetcher, domain: &str) -> Option<AccountCandidate> {
    let url = format!("https://{domain}/.well-known/jmap");
    let body = fetcher.get(&url).await.ok()??;
    parse_jmap_session(&body, domain)
}

/// Parse a JMAP session resource (RFC 8620 §2). A resource is "valid" when it
/// advertises the core capability and an `apiUrl`; the endpoint host becomes the
/// candidate's [`ServerSpec`] (implicit TLS on 443 unless the `apiUrl` says
/// otherwise). Auth is left at the neutral default — the session does not carry
/// a SASL mechanism (JMAP authenticates at the HTTP layer).
fn parse_jmap_session(body: &str, domain: &str) -> Option<AccountCandidate> {
    let v: serde_json::Value = serde_json::from_str(body).ok()?;
    let caps = v.get("capabilities")?.as_object()?;
    if !caps.contains_key("urn:ietf:params:jmap:core") {
        return None;
    }
    let api_url = v.get("apiUrl")?.as_str()?;
    let (host, port) = url_host_port(api_url).unwrap_or_else(|| (domain.to_string(), 443));
    let spec = ServerSpec {
        host,
        port,
        tls: TlsMode::Implicit,
    };
    Some(AccountCandidate {
        imap: spec.clone(),
        pop3: None,
        smtp: spec,
        auth: AuthMethod::Password,
        source: DiscoverySource::Jmap,
    })
}

/// Extract the host and port from an absolute URL (defaulting to 443). Kept
/// deliberately small — JMAP `apiUrl`s are DNS-name HTTPS URLs.
fn url_host_port(url: &str) -> Option<(String, u16)> {
    let rest = url.split_once("://").map(|(_, r)| r).unwrap_or(url);
    let authority = rest.split(['/', '?', '#']).next()?;
    // Drop any userinfo (`user:pass@host`).
    let authority = authority.rsplit_once('@').map_or(authority, |(_, h)| h);
    if authority.is_empty() {
        return None;
    }
    // `host:port` (IPv6 literals carry ':' inside brackets — not expected here).
    if !authority.starts_with('[')
        && let Some((h, p)) = authority.rsplit_once(':')
        && let Ok(port) = p.parse::<u16>()
    {
        return Some((h.to_string(), port));
    }
    Some((authority.to_string(), 443))
}

// ---- rung 2: RFC 6186 SRV -------------------------------------------------

async fn try_srv(fetcher: &dyn Fetcher, domain: &str) -> Option<AccountCandidate> {
    let imap = pick_srv(fetcher, &format!("_imaps._tcp.{domain}"), TlsMode::Implicit).await;
    // Fall back to STARTTLS imap if imaps is absent.
    let imap = match imap {
        Some(s) => Some(s),
        None => pick_srv(fetcher, &format!("_imap._tcp.{domain}"), TlsMode::StartTls).await,
    };
    let imap = imap?; // IMAP is required to build an IMAP-first candidate.

    let smtp = pick_srv(
        fetcher,
        &format!("_submissions._tcp.{domain}"),
        TlsMode::Implicit,
    )
    .await;
    let smtp = match smtp {
        Some(s) => s,
        None => {
            pick_srv(
                fetcher,
                &format!("_submission._tcp.{domain}"),
                TlsMode::StartTls,
            )
            .await?
        }
    };

    let pop3 = pick_srv(fetcher, &format!("_pop3s._tcp.{domain}"), TlsMode::Implicit).await;

    Some(AccountCandidate {
        imap,
        pop3,
        smtp,
        // SRV conveys no auth method; assume password (XOAUTH2 domains resolve
        // via autoconfig/provider-DB, which do carry it).
        auth: AuthMethod::Password,
        source: DiscoverySource::Srv,
    })
}

/// Resolve one service and turn its strongest record into a [`ServerSpec`].
async fn pick_srv(fetcher: &dyn Fetcher, service: &str, tls: TlsMode) -> Option<ServerSpec> {
    let mut records = fetcher.srv(service).await.ok()?;
    // A single "." target means the service is explicitly unavailable (RFC 6186).
    records.retain(|r| !r.target.is_empty() && r.target != ".");
    // Lowest priority wins; higher weight breaks ties.
    records.sort_by(|a, b| a.priority.cmp(&b.priority).then(b.weight.cmp(&a.weight)));
    let best = records.into_iter().next()?;
    Some(ServerSpec {
        host: best.target.trim_end_matches('.').to_string(),
        port: best.port,
        tls,
    })
}

// ---- rung 3: Thunderbird autoconfig XML -----------------------------------

async fn try_autoconfig(
    fetcher: &dyn Fetcher,
    domain: &str,
    email: &str,
) -> Option<AccountCandidate> {
    let urls = [
        format!("https://autoconfig.{domain}/mail/config-v1.1.xml?emailaddress={email}"),
        format!("https://autoconfig.thunderbird.net/v1.1/{domain}"),
    ];
    for url in urls {
        if let Ok(Some(body)) = fetcher.get(&url).await
            && let Some(c) = parse_autoconfig(&body)
        {
            return Some(c);
        }
    }
    None
}

fn parse_autoconfig(body: &str) -> Option<AccountCandidate> {
    let root = xml::parse(body)?;
    let provider = root.child("emailProvider")?;

    let mut imap = None;
    let mut pop3 = None;
    let mut imap_auth = AuthMethod::Password;
    for inc in provider.children_named("incomingServer") {
        let (spec, auth) = server_from_xml(inc)?;
        match inc.attr("type") {
            Some("imap") if imap.is_none() => {
                imap = Some(spec);
                imap_auth = auth;
            }
            Some("pop3") if pop3.is_none() => pop3 = Some(spec),
            _ => {}
        }
    }
    let smtp = provider
        .children_named("outgoingServer")
        .find(|o| o.attr("type") == Some("smtp"))
        .and_then(|o| server_from_xml(o).map(|(s, _)| s))?;

    Some(AccountCandidate {
        imap: imap?,
        pop3,
        smtp,
        auth: imap_auth,
        source: DiscoverySource::ThunderbirdAutoconfig,
    })
}

/// Build a `ServerSpec` + auth from an `<incomingServer>`/`<outgoingServer>`.
fn server_from_xml(node: &xml::Node) -> Option<(ServerSpec, AuthMethod)> {
    let host = node.child_text("hostname")?;
    let port: u16 = node.child_text("port")?.parse().ok()?;
    let tls = match node.child_text("socketType").as_deref() {
        Some("SSL") | Some("TLS") => TlsMode::Implicit,
        Some("STARTTLS") => TlsMode::StartTls,
        _ => TlsMode::None,
    };
    // OAuth2 wins if the server advertises it among its authentication methods.
    let auth = if node
        .children_named("authentication")
        .any(|a| a.text.trim().eq_ignore_ascii_case("OAuth2"))
    {
        AuthMethod::OAuth2
    } else {
        AuthMethod::Password
    };
    Some((ServerSpec { host, port, tls }, auth))
}

// ---- rung 4: MS Autodiscover v2 -------------------------------------------

async fn try_autodiscover(fetcher: &dyn Fetcher, domain: &str) -> Option<AccountCandidate> {
    let urls = [
        format!("https://autodiscover.{domain}/autodiscover/autodiscover.json"),
        format!("https://{domain}/autodiscover/autodiscover.json"),
    ];
    for url in urls {
        if let Ok(Some(body)) = fetcher.get(&url).await
            && let Some(c) = parse_autodiscover(&body)
        {
            return Some(c);
        }
    }
    None
}

fn parse_autodiscover(body: &str) -> Option<AccountCandidate> {
    let v: serde_json::Value = serde_json::from_str(body).ok()?;
    let protocols = v.get("Protocols")?.as_array()?;

    let mut imap = None;
    let mut pop3 = None;
    let mut smtp = None;
    let mut oauth = false;

    for p in protocols {
        let ty = p.get("Type").and_then(|t| t.as_str()).unwrap_or_default();
        let host = p.get("Server").and_then(|s| s.as_str())?.to_string();
        let port = p.get("Port").and_then(|n| n.as_u64())? as u16;
        let ssl = p.get("SSL").and_then(|b| b.as_bool()).unwrap_or(false);
        let starttls = p
            .get("Encryption")
            .and_then(|e| e.as_str())
            .map(|e| e.eq_ignore_ascii_case("STARTTLS"))
            .unwrap_or(false);
        let tls = if ssl {
            TlsMode::Implicit
        } else if starttls {
            TlsMode::StartTls
        } else {
            TlsMode::None
        };
        if p.get("AuthPackage")
            .and_then(|a| a.as_str())
            .map(|a| {
                a.to_ascii_lowercase().contains("oauth")
                    || a.to_ascii_lowercase().contains("bearer")
            })
            .unwrap_or(false)
        {
            oauth = true;
        }
        let spec = ServerSpec { host, port, tls };
        match ty.to_ascii_uppercase().as_str() {
            "IMAP" => imap = Some(spec),
            "POP3" => pop3 = Some(spec),
            "SMTP" => smtp = Some(spec),
            _ => {}
        }
    }

    Some(AccountCandidate {
        imap: imap?,
        pop3,
        smtp: smtp?,
        auth: if oauth {
            AuthMethod::OAuth2
        } else {
            AuthMethod::Password
        },
        source: DiscoverySource::Autodiscover,
    })
}

// ---- default live fetcher -------------------------------------------------

/// Live [`Fetcher`] over `reqwest` (rustls) for the HTTP rungs, with SRV
/// resolved live by a [`resolver::SrvResolver`] (the [`HickoryResolver`] by
/// default). If the system resolver cannot be built, SRV degrades to a no-op
/// and the ladder relies on the HTTP rungs.
pub struct ReqwestFetcher {
    resolver: Box<dyn resolver::SrvResolver>,
}

/// The `User-Agent` autodiscovery announces.
///
/// # Known limit: a ROUTED fetch cannot honour this
/// `mw_egress::proxy::http::exchange` hardcodes `PROXY_UA`, so if a deployment ever
/// routes autodiscovery through a configured egress proxy it will announce
/// `Mailwoman-Image-Proxy` to provider autodiscovery endpoints instead of this.
/// Written down rather than discovered: `t22-e11`'s own comment notes that this is
/// the kind of thing that produces support reports nobody can reproduce. The direct
/// path — which is every deployment today — sends this correctly.
const AUTOCONFIG_UA: &str = "mailwoman-autoconfig";

/// Whether an operator has opted this deployment into reaching PRIVATE addresses
/// during autodiscovery (`MW_AUTOCONFIG_ALLOW_PRIVATE`, **default off**).
///
/// # Why this needs an opt-in when the keyserver lookups do not
/// `POST /api/discover` is **unauthenticated** and CSRF-exempt, and the domain it
/// probes comes straight from an email address in the request body. So an anonymous
/// caller chooses where the server fetches, which is why the strict policy is the
/// default and the widened one has to be a deliberate operator act.
///
/// But a self-hosted deployment legitimately has `autoconfig.corp.internal` on
/// RFC1918, and refusing it would turn an SSRF fix into a functional regression for
/// exactly this project's audience. Hence: opt-in, off by default, release-noted.
///
/// # Why the ManageSieve precedent does NOT apply here, though it looks like it does
/// `mw-server::sieve_sync::sieve_egress_permitted` permits RFC1918 unconditionally,
/// and the temptation is to copy it and drop this flag. **Do not.** That policy is
/// safe because a ManageSieve host is configured by an authenticated user *with
/// credentials* — a deliberate act by someone who already has an account. This
/// domain arrives in an unauthenticated request body. "Your own internal server" and
/// "an anonymous caller's chosen domain" are not the same trust, and the two look
/// identical in the code. The next person to notice the inconsistency between the
/// two policies is meant to read this rather than "fix" it.
fn allow_private_targets() -> bool {
    std::env::var("MW_AUTOCONFIG_ALLOW_PRIVATE")
        .map(|v| matches!(v.trim(), "1" | "true" | "TRUE" | "yes"))
        .unwrap_or(false)
}

/// The address policy this deployment applies to autodiscovery fetches.
///
/// Even under the opt-in, `mw_egress::on_prem_allowed` still refuses loopback,
/// `169.254.0.0/16` (cloud metadata), `fe80::/10`, and every NAT64/6to4/Teredo/ISATAP
/// embedding of those — because it is one implementation in the crate that owns the
/// decode, not a predicate re-authored per caller. The flag widens the address range
/// and **nothing else**: scheme, credential-in-URL, host and port checks live in the
/// shared body and are not parameterised.
fn address_policy() -> fn(&std::net::IpAddr) -> bool {
    if allow_private_targets() {
        mw_egress::on_prem_allowed
    } else {
        mw_egress::ip_allowed
    }
}

impl ReqwestFetcher {
    /// Build the live SRV resolver. HTTP rungs go through [`mw_egress`], so there is
    /// no client here to configure.
    pub fn new() -> Result<Self, DiscoverError> {
        let resolver: Box<dyn resolver::SrvResolver> = match HickoryResolver::new() {
            Ok(r) => Box::new(r),
            Err(_) => Box::new(resolver::NoopResolver),
        };
        Ok(Self { resolver })
    }

    /// Build a fetcher with an injected SRV resolver — the seam the SRV tests
    /// use to exercise the ladder against a stub with no live network.
    #[cfg(test)]
    fn with_resolver(resolver: Box<dyn resolver::SrvResolver>) -> Self {
        Self { resolver }
    }
}

#[async_trait]
impl Fetcher for ReqwestFetcher {
    async fn srv(&self, service: &str) -> Result<Vec<SrvRecord>, DiscoverError> {
        self.resolver.lookup_srv(service).await
    }

    async fn get(&self, url: &str) -> Result<Option<String>, DiscoverError> {
        // Routed through `mw-egress` (t22-e9): the full SSRF gate — scheme and
        // credential checks, one-shot DNS with a pinned connect, every resolved
        // address through the policy, per-hop redirect re-validation, size and
        // timeout caps. This used to be a bare client carrying only an ambient-proxy
        // refusal, on an UNAUTHENTICATED endpoint whose target domain comes from the
        // request body.
        //
        // Collapsing every refusal to `Ok(None)` is not lossy here: this trait's
        // contract is already "body on 200, `None` on any non-success or recoverable
        // transport failure, so the ladder simply moves on". A blocked address and a
        // missing page are the same thing to the ladder — try the next rung.
        match mw_egress::fetch_url_hardened_with(
            url,
            "application/xml, application/json;q=0.9, */*;q=0.8",
            AUTOCONFIG_UA,
            address_policy(),
        )
        .await
        {
            Ok(bytes) => Ok(String::from_utf8(bytes).ok()),
            Err(_) => Ok(None),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    /// Fixture-backed fetcher: a URL→body map plus an SRV name→records map.
    #[derive(Default)]
    struct MockFetcher {
        pages: HashMap<String, String>,
        srv: HashMap<String, Vec<SrvRecord>>,
    }

    impl MockFetcher {
        fn page(mut self, url: &str, body: &str) -> Self {
            self.pages.insert(url.to_string(), body.to_string());
            self
        }
        fn srv_record(mut self, name: &str, recs: Vec<SrvRecord>) -> Self {
            self.srv.insert(name.to_string(), recs);
            self
        }
    }

    #[async_trait]
    impl Fetcher for MockFetcher {
        async fn srv(&self, service: &str) -> Result<Vec<SrvRecord>, DiscoverError> {
            Ok(self.srv.get(service).cloned().unwrap_or_default())
        }
        async fn get(&self, url: &str) -> Result<Option<String>, DiscoverError> {
            Ok(self.pages.get(url).cloned())
        }
    }

    fn load(name: &str) -> String {
        std::fs::read_to_string(format!(
            "{}/../../fixtures/autoconfig/{name}",
            env!("CARGO_MANIFEST_DIR")
        ))
        .unwrap_or_else(|e| panic!("read fixture {name}: {e}"))
    }

    #[test]
    fn split_email_validates() {
        assert!(split_email("a@b.com").is_ok());
        assert!(matches!(
            split_email("nope"),
            Err(DiscoverError::InvalidEmail(_))
        ));
        assert!(matches!(
            split_email("@b.com"),
            Err(DiscoverError::InvalidEmail(_))
        ));
        assert!(matches!(
            split_email("a@localhost"),
            Err(DiscoverError::InvalidEmail(_))
        ));
    }

    const JMAP_SESSION: &str = r#"{
        "capabilities": {
            "urn:ietf:params:jmap:core": { "maxSizeUpload": 50000000 },
            "urn:ietf:params:jmap:mail": {}
        },
        "accounts": {},
        "primaryAccounts": {},
        "username": "user@corp.example",
        "apiUrl": "https://jmap.corp.example/jmap/api/",
        "downloadUrl": "https://jmap.corp.example/jmap/download/",
        "uploadUrl": "https://jmap.corp.example/jmap/upload/",
        "eventSourceUrl": "https://jmap.corp.example/jmap/eventsource/",
        "state": "cyrus-0"
    }"#;

    #[tokio::test]
    async fn well_known_jmap_is_rung_one_and_short_circuits() {
        // A valid session resource wins even when SRV records also resolve.
        let f = MockFetcher::default()
            .page("https://corp.example/.well-known/jmap", JMAP_SESSION)
            .srv_record(
                "_imaps._tcp.corp.example",
                vec![SrvRecord {
                    target: "imap.corp.example.".into(),
                    port: 993,
                    priority: 10,
                    weight: 1,
                }],
            );
        let c = discover_with("user@corp.example", &f).await.unwrap();
        assert_eq!(c.source, DiscoverySource::Jmap);
        // The endpoint host comes from the session apiUrl, on implicit-TLS 443.
        assert_eq!(c.imap.host, "jmap.corp.example");
        assert_eq!(c.imap.port, 443);
        assert_eq!(c.imap.tls, TlsMode::Implicit);
        assert_eq!(c.smtp.host, "jmap.corp.example");
        assert!(c.pop3.is_none());
    }

    #[tokio::test]
    async fn jmap_session_without_core_capability_falls_through() {
        // A resource lacking the mandatory core capability is not a JMAP session;
        // the ladder falls through to the offline provider DB.
        let bogus = r#"{ "capabilities": { "urn:example:other": {} }, "apiUrl": "https://x/" }"#;
        let f = MockFetcher::default().page("https://yahoo.com/.well-known/jmap", bogus);
        let c = discover_with("someone@yahoo.com", &f).await.unwrap();
        assert_eq!(c.source, DiscoverySource::ProviderDb);
    }

    #[tokio::test]
    async fn jmap_api_url_with_explicit_port() {
        let session = JMAP_SESSION.replace(
            "https://jmap.corp.example/jmap/api/",
            "https://jmap.corp.example:8443/jmap/api/",
        );
        let f = MockFetcher::default().page("https://corp.example/.well-known/jmap", &session);
        let c = discover_with("user@corp.example", &f).await.unwrap();
        assert_eq!(c.imap.host, "jmap.corp.example");
        assert_eq!(c.imap.port, 8443);
    }

    #[test]
    fn url_host_port_parses_variants() {
        assert_eq!(
            url_host_port("https://api.example.org/jmap/"),
            Some(("api.example.org".into(), 443))
        );
        assert_eq!(
            url_host_port("https://api.example.org:8443/jmap"),
            Some(("api.example.org".into(), 8443))
        );
        assert_eq!(
            url_host_port("https://user@api.example.org/jmap"),
            Some(("api.example.org".into(), 443))
        );
    }

    #[tokio::test]
    async fn reqwest_fetcher_resolves_srv_via_injected_resolver() {
        // The real ReqwestFetcher's srv() delegates to its SrvResolver; a stub
        // exercises the seam (and pick_srv's host normalization) with no network.
        let f = ReqwestFetcher::with_resolver(Box::new(resolver::StubResolver(vec![SrvRecord {
            target: "imap.corp.example.".into(),
            port: 993,
            priority: 10,
            weight: 1,
        }])));
        let recs = f.srv("_imaps._tcp.corp.example").await.unwrap();
        assert_eq!(recs.len(), 1);
        assert_eq!(recs[0].port, 993);
        let spec = pick_srv(&f, "_imaps._tcp.corp.example", TlsMode::Implicit)
            .await
            .unwrap();
        assert_eq!(spec.host, "imap.corp.example");
        assert_eq!(spec.tls, TlsMode::Implicit);
    }

    #[tokio::test]
    async fn ladder_prefers_srv_when_present() {
        let f = MockFetcher::default()
            .srv_record(
                "_imaps._tcp.corp.example",
                vec![SrvRecord {
                    target: "imap.corp.example.".into(),
                    port: 993,
                    priority: 10,
                    weight: 1,
                }],
            )
            .srv_record(
                "_submission._tcp.corp.example",
                vec![SrvRecord {
                    target: "smtp.corp.example".into(),
                    port: 587,
                    priority: 10,
                    weight: 1,
                }],
            );
        let c = discover_with("user@corp.example", &f).await.unwrap();
        assert_eq!(c.source, DiscoverySource::Srv);
        assert_eq!(c.imap.host, "imap.corp.example");
        assert_eq!(c.imap.tls, TlsMode::Implicit);
        assert_eq!(c.smtp.port, 587);
        assert_eq!(c.smtp.tls, TlsMode::StartTls);
    }

    #[tokio::test]
    async fn srv_picks_lowest_priority() {
        let f = MockFetcher::default()
            .srv_record(
                "_imaps._tcp.corp.example",
                vec![
                    SrvRecord {
                        target: "backup.corp.example".into(),
                        port: 993,
                        priority: 20,
                        weight: 1,
                    },
                    SrvRecord {
                        target: "primary.corp.example".into(),
                        port: 993,
                        priority: 5,
                        weight: 1,
                    },
                ],
            )
            .srv_record(
                "_submissions._tcp.corp.example",
                vec![SrvRecord {
                    target: "smtp.corp.example".into(),
                    port: 465,
                    priority: 1,
                    weight: 1,
                }],
            );
        let c = discover_with("user@corp.example", &f).await.unwrap();
        assert_eq!(c.imap.host, "primary.corp.example");
        assert_eq!(c.smtp.tls, TlsMode::Implicit);
    }

    #[tokio::test]
    async fn falls_back_to_ispdb_autoconfig_xml() {
        // No SRV, no provider-hosted autoconfig; ISPDB serves the Gmail XML.
        let f = MockFetcher::default().page(
            "https://autoconfig.thunderbird.net/v1.1/gmail.com",
            &load("gmail-ispdb.xml"),
        );
        let c = discover_with("someone@gmail.com", &f).await.unwrap();
        assert_eq!(c.source, DiscoverySource::ThunderbirdAutoconfig);
        assert_eq!(c.imap.host, "imap.gmail.com");
        assert_eq!(c.imap.tls, TlsMode::Implicit);
        assert_eq!(c.auth, AuthMethod::OAuth2);
        assert_eq!(c.pop3.as_ref().unwrap().host, "pop.gmail.com");
        assert_eq!(c.smtp.host, "smtp.gmail.com");
    }

    #[tokio::test]
    async fn provider_hosted_autoconfig_starttls_no_pop3() {
        let f = MockFetcher::default().page(
            "https://autoconfig.example.net/mail/config-v1.1.xml?emailaddress=me@example.net",
            &load("generic-autoconfig.xml"),
        );
        let c = discover_with("me@example.net", &f).await.unwrap();
        assert_eq!(c.source, DiscoverySource::ThunderbirdAutoconfig);
        assert_eq!(c.imap.tls, TlsMode::StartTls);
        assert_eq!(c.imap.port, 143);
        assert_eq!(c.auth, AuthMethod::Password);
        assert!(c.pop3.is_none());
        assert_eq!(c.smtp.port, 587);
    }

    #[tokio::test]
    async fn falls_back_to_autodiscover_json() {
        let f = MockFetcher::default().page(
            "https://autodiscover.contoso.example/autodiscover/autodiscover.json",
            &load("outlook-autodiscover.json"),
        );
        let c = discover_with("user@contoso.example", &f).await.unwrap();
        assert_eq!(c.source, DiscoverySource::Autodiscover);
        assert_eq!(c.imap.host, "outlook.office365.com");
        assert_eq!(c.imap.tls, TlsMode::Implicit);
        assert_eq!(c.smtp.tls, TlsMode::StartTls);
        assert_eq!(c.auth, AuthMethod::OAuth2);
    }

    #[tokio::test]
    async fn falls_back_to_offline_provider_db() {
        // Nothing on the network → the bundled DB resolves yahoo.com.
        let f = MockFetcher::default();
        let c = discover_with("someone@yahoo.com", &f).await.unwrap();
        assert_eq!(c.source, DiscoverySource::ProviderDb);
        assert_eq!(c.imap.host, "imap.mail.yahoo.com");
        assert_eq!(c.auth, AuthMethod::Password);
    }

    #[tokio::test]
    async fn unknown_domain_is_not_found() {
        let f = MockFetcher::default();
        let err = discover_with("nobody@unknown.invalid", &f)
            .await
            .unwrap_err();
        assert!(matches!(err, DiscoverError::NotFound(_)));
    }

    #[tokio::test]
    async fn invalid_email_is_rejected() {
        let f = MockFetcher::default();
        assert!(matches!(
            discover_with("not-an-email", &f).await,
            Err(DiscoverError::InvalidEmail(_))
        ));
    }

    // ── the opt-in's address policy ───────────────────────────────────────────

    fn ip(s: &str) -> std::net::IpAddr {
        s.parse().unwrap()
    }

    /// The strict default. Asserted so "the opt-in permits RFC1918" below is a
    /// CHANGE rather than a property the strict policy already had — without this,
    /// a bug that always returned `on_prem_allowed` would pass the opt-in test.
    #[test]
    fn the_default_policy_refuses_private_targets() {
        let strict = mw_egress::ip_allowed;
        assert!(!strict(&ip("10.0.0.1")));
        assert!(!strict(&ip("192.168.1.1")));
        assert!(strict(&ip("1.1.1.1")), "public unicast stays reachable");
    }

    /// The ruling this lane was given: the opt-in widens the address range to
    /// RFC1918 for on-premises autodiscovery, and **still refuses link-local**,
    /// including `169.254.169.254` and every transitional embedding of it.
    ///
    /// This CONFIRMS a property rather than creating it — `t22-e11` factored the rule
    /// into one private `on_prem_v4_allowed` shared by the IPv4 arm, the IPv4-mapped
    /// unwrap and the `embedded_ipv4s` loop, so link-local stays denied because there
    /// is one implementation rather than because three arms each remembered. Asserted
    /// here anyway, at the call site that actually selects the policy: a future edit
    /// that swapped this deployment onto a hand-rolled predicate would pass every test
    /// in `mw-egress` and fail this one.
    #[test]
    fn the_opt_in_permits_rfc1918_and_still_refuses_link_local() {
        let permissive = mw_egress::on_prem_allowed;

        // The point of the opt-in: an on-premises autoconfig host is reachable.
        for reachable in ["10.0.0.1", "192.168.1.1", "172.16.0.1", "fd12:3456::1"] {
            assert!(
                permissive(&ip(reachable)),
                "{reachable} must be reachable under MW_AUTOCONFIG_ALLOW_PRIVATE, or \
                 the opt-in does not do the one thing it exists for"
            );
        }

        // …and the carve-out that must survive it. An "allow private for on-prem"
        // flag that also opened the cloud metadata endpoint would convert a
        // self-hoster convenience into instance-credential theft, on an endpoint
        // anyone on the internet can reach.
        for denied in [
            "169.254.169.254",                      // cloud metadata
            "169.254.1.1",                          // link-local generally
            "127.0.0.1",                            // loopback
            "::1",                                  // v6 loopback
            "fe80::1",                              // v6 link-local
            "::ffff:169.254.169.254",               // IPv4-mapped metadata
            "64:ff9b::a9fe:a9fe",                   // NAT64-embedded metadata
            "2002:a9fe:a9fe::",                     // 6to4-embedded metadata
            "2001:0:4136:e378:8000:ffff:5601:5601", // Teredo-embedded metadata
            "2001:470::5efe:7f00:1",                // ISATAP-embedded loopback
        ] {
            assert!(
                !permissive(&ip(denied)),
                "{denied} must stay refused EVEN WITH the opt-in on"
            );
        }
    }

    /// The selector honours the flag. Read through the same function production uses,
    /// so a change to the variable's name or its accepted spellings fails here.
    ///
    /// Uses the real environment, and therefore asserts only the DEFAULT — setting
    /// the variable would leak into every other test in this binary, and
    /// `std::env::set_var` is `unsafe` in this edition. The two policies themselves
    /// are asserted above; what is checked here is which one an unconfigured
    /// deployment gets.
    #[test]
    fn an_unconfigured_deployment_gets_the_strict_policy() {
        assert!(
            !allow_private_targets(),
            "MW_AUTOCONFIG_ALLOW_PRIVATE must default OFF — the safe deployment is \
             the one you get without configuring anything"
        );
        let selected = address_policy();
        assert!(
            !selected(&ip("10.0.0.1")),
            "the default selection must be the strict policy"
        );
    }
}
