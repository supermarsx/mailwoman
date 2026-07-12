#![forbid(unsafe_code)]
//! `mw-autoconfig` — server discovery for a login email (plan §0/§2.4).
//!
//! [`discover`] walks a fallback ladder and returns the first
//! [`AccountCandidate`] it can build:
//!
//! 1. **RFC 6186 SRV** (`_imaps._tcp` / `_submission._tcp`, …). Best-effort in
//!    V1: the default [`ReqwestFetcher`] does not ship a DNS resolver (to stay
//!    within the license floor — no async-std/copyleft SRV crate), so its SRV
//!    rung is a no-op and the ladder relies on the HTTP methods. The rung is
//!    fully wired and unit-tested through the injectable [`Fetcher`] seam so a
//!    resolver can be dropped in later without touching the ladder.
//! 2. **Thunderbird autoconfig XML** — provider-hosted
//!    `https://autoconfig.<domain>/mail/config-v1.1.xml` then the ISPDB
//!    `https://autoconfig.thunderbird.net/v1.1/<domain>`.
//! 3. **MS Autodiscover v2** — `GET /autodiscover/autodiscover.json`.
//! 4. **Offline provider DB** — bundled JSON for the big providers.
//! 5. **Manual** — every rung missed; return [`DiscoverError::NotFound`] and
//!    the UI shows its manual fields.
//!
//! Network access is behind the [`Fetcher`] trait so the ladder is exercised
//! against recorded fixtures with no live network (see the crate tests).

mod provider;
mod xml;

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

// ---- rung 1: RFC 6186 SRV -------------------------------------------------

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

// ---- rung 2: Thunderbird autoconfig XML -----------------------------------

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

// ---- rung 3: MS Autodiscover v2 -------------------------------------------

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

/// Live [`Fetcher`] over `reqwest` (rustls). SRV is a no-op in V1 (see the
/// module docs); the HTTP rungs are real.
pub struct ReqwestFetcher {
    client: reqwest::Client,
}

impl ReqwestFetcher {
    /// Build the HTTPS client used for autoconfig/autodiscover fetches.
    pub fn new() -> Result<Self, DiscoverError> {
        let client = reqwest::Client::builder()
            .user_agent("mailwoman-autoconfig")
            .build()
            .map_err(|e| DiscoverError::Lookup(e.to_string()))?;
        Ok(Self { client })
    }
}

#[async_trait]
impl Fetcher for ReqwestFetcher {
    async fn srv(&self, _service: &str) -> Result<Vec<SrvRecord>, DiscoverError> {
        // Best-effort/skippable in V1 (no bundled DNS resolver — plan §0 rung 1).
        Ok(Vec::new())
    }

    async fn get(&self, url: &str) -> Result<Option<String>, DiscoverError> {
        let resp = match self.client.get(url).send().await {
            Ok(r) => r,
            Err(_) => return Ok(None), // treat transport failure as "rung missed"
        };
        if !resp.status().is_success() {
            return Ok(None);
        }
        match resp.text().await {
            Ok(body) => Ok(Some(body)),
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
}
