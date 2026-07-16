//! t12-e-e2e-backend — autoconfig `.well-known/jmap` rung + SRV resolver (audit #10).
//!
//! Proves mw-autoconfig's new rung-1 JMAP session autodiscovery short-circuits the
//! ladder when the resource is served, over a REAL HTTP round-trip (reqwest → an
//! in-process HTTP responder → real `parse_jmap_session`), and that the SRV rung
//! produces an IMAP-first candidate from records. `.well-known/jmap` must WIN even
//! when SRV records also resolve (rung ordering).
//!
//! An optional live-DNS SRV leg (gated `MW_AUTOCONFIG_LIVE_DNS=1`) resolves a real
//! public `_imaps._tcp` SRV via the shipped `ReqwestFetcher` (hickory) — loud-skip
//! offline.

use async_trait::async_trait;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpListener;

use mw_autoconfig::{DiscoverError, DiscoverySource, Fetcher, SrvRecord, TlsMode, discover_with};

/// Serve a fixed JSON body once over raw HTTP/1.1, returning the bound base URL.
async fn serve_json_once(body: &'static str) -> String {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    tokio::spawn(async move {
        // Serve a few requests (the ladder may probe once; keep the loop small).
        for _ in 0..4 {
            let Ok((mut sock, _)) = listener.accept().await else {
                break;
            };
            let mut buf = [0u8; 2048];
            let _ = sock.read(&mut buf).await;
            let resp = format!(
                "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
                body.len(),
                body
            );
            let _ = sock.write_all(resp.as_bytes()).await;
            let _ = sock.shutdown().await;
        }
    });
    format!("http://{addr}")
}

/// A Fetcher that fulfils the `.well-known/jmap` GET with a REAL HTTP request to the
/// local server, and optionally returns SRV records (to prove JMAP still wins).
struct LiveJmapFetcher {
    base: String,
    srv_records: Vec<SrvRecord>,
}

#[async_trait]
impl Fetcher for LiveJmapFetcher {
    async fn srv(&self, _service: &str) -> Result<Vec<SrvRecord>, DiscoverError> {
        Ok(self.srv_records.clone())
    }
    async fn get(&self, url: &str) -> Result<Option<String>, DiscoverError> {
        if url.ends_with("/.well-known/jmap") {
            let local = format!("{}/.well-known/jmap", self.base);
            return match reqwest::get(&local).await {
                Ok(r) if r.status().is_success() => Ok(Some(r.text().await.unwrap_or_default())),
                _ => Ok(None),
            };
        }
        Ok(None)
    }
}

const JMAP_SESSION: &str = r#"{"capabilities":{"urn:ietf:params:jmap:core":{"maxSizeUpload":50000000}},"apiUrl":"https://jmap.corp.example:443/api","accounts":{}}"#;

#[tokio::test]
async fn well_known_jmap_rung_short_circuits_over_real_http() {
    let base = serve_json_once(JMAP_SESSION).await;
    // Provide an SRV record too — the JMAP rung must still win (rung 1 before SRV).
    let fetcher = LiveJmapFetcher {
        base,
        srv_records: vec![SrvRecord {
            target: "imap.corp.example.".into(),
            port: 993,
            priority: 0,
            weight: 1,
        }],
    };

    let cand = discover_with("user@corp.example", &fetcher)
        .await
        .expect("JMAP rung resolves a candidate");

    assert_eq!(
        cand.source,
        DiscoverySource::Jmap,
        "the .well-known/jmap rung short-circuits ahead of SRV"
    );
    assert_eq!(
        cand.imap.host, "jmap.corp.example",
        "host from the session apiUrl"
    );
    assert_eq!(cand.imap.port, 443);
    assert_eq!(cand.imap.tls, TlsMode::Implicit);
}

/// A Fetcher with NO JMAP resource but live SRV records ⇒ the SRV rung builds the
/// candidate (proves the resolver seam feeds `try_srv`).
struct SrvOnlyFetcher;
#[async_trait]
impl Fetcher for SrvOnlyFetcher {
    async fn srv(&self, service: &str) -> Result<Vec<SrvRecord>, DiscoverError> {
        let rec = |host: &str, port: u16| SrvRecord {
            target: format!("{host}."),
            port,
            priority: 0,
            weight: 1,
        };
        Ok(match service {
            s if s.starts_with("_imaps._tcp") => vec![rec("imap.corp.example", 993)],
            s if s.starts_with("_submissions._tcp") => vec![rec("smtp.corp.example", 465)],
            _ => vec![],
        })
    }
    async fn get(&self, _url: &str) -> Result<Option<String>, DiscoverError> {
        Ok(None)
    }
}

#[tokio::test]
async fn srv_rung_builds_candidate() {
    let cand = discover_with("user@corp.example", &SrvOnlyFetcher)
        .await
        .expect("SRV rung resolves a candidate");
    assert_eq!(cand.source, DiscoverySource::Srv);
    assert_eq!(cand.imap.host, "imap.corp.example");
    assert_eq!(cand.imap.port, 993);
    assert_eq!(cand.smtp.host, "smtp.corp.example");
    assert_eq!(cand.smtp.port, 465);
}

/// Optional: resolve a REAL public SRV record via the shipped hickory-backed
/// `ReqwestFetcher`. Loud-skip when live DNS is not enabled/available.
#[tokio::test]
async fn live_srv_resolves_public_record() {
    if std::env::var("MW_AUTOCONFIG_LIVE_DNS").ok().as_deref() != Some("1") {
        eprintln!(
            "\n[t12 AUTOCONFIG SKIP] MW_AUTOCONFIG_LIVE_DNS!=1 — real DNS SRV not queried \
             (deterministic legs cover the ladder). Set it to resolve a public SRV.\n"
        );
        return;
    }
    let fetcher = match mw_autoconfig::ReqwestFetcher::new() {
        Ok(f) => f,
        Err(e) => {
            eprintln!("\n[t12 AUTOCONFIG SKIP] could not build the live resolver: {e}\n");
            return;
        }
    };
    // gmail.com publishes `_imaps._tcp` SRV → imap.gmail.com:993.
    let recs = fetcher
        .srv("_imaps._tcp.gmail.com")
        .await
        .expect("live SRV query");
    assert!(
        recs.iter().any(|r| r.target.contains("imap.gmail.com")),
        "expected imap.gmail.com in {recs:?}"
    );
}
