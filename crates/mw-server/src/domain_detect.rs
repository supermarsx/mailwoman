//! Operator-triggered discovery. Every HTTP hop follows the admin-detect route.
use crate::{
    AppState,
    admin_ctx::{AdminApiError, AdminCtx},
    capabilities::Capability,
};
use axum::{
    Json,
    extract::{Path, State},
};
use mw_autoconfig::{DiscoverError, Fetcher, SrvResolver};
use mw_egress::{
    TrafficClass,
    plan::{EgressPlan, Request},
};
use serde_json::{Value, json};

struct RoutedFetcher {
    plan: EgressPlan,
    domain: String,
    resolver: Box<dyn SrvResolver>,
}

#[async_trait::async_trait]
impl Fetcher for RoutedFetcher {
    async fn srv(&self, service: &str) -> Result<Vec<mw_autoconfig::SrvRecord>, DiscoverError> {
        self.resolver.lookup_srv(service).await
    }

    async fn get(&self, url: &str) -> Result<Option<String>, DiscoverError> {
        let url = reqwest::Url::parse(url)
            .map_err(|_| DiscoverError::Lookup("invalid discovery URL".into()))?;
        let request = Request::get(TrafficClass::AdminDetect)
            .for_domain(&self.domain)
            .accept("application/xml, application/json;q=0.9, */*;q=0.8");
        let result = mw_egress::plan::send_with(&self.plan, url, &request).await;
        Ok(result
            .outcome
            .ok()
            .and_then(|bytes| String::from_utf8(bytes).ok()))
    }
}

pub(crate) async fn detect(
    State(state): State<AppState>,
    ctx: AdminCtx,
    Path(name): Path<String>,
) -> Result<Json<Value>, AdminApiError> {
    ctx.require_unscoped(Capability::DomainsUpstreamWrite)?;
    let name = crate::domain_config::normalize_domain(&name)
        .map_err(|error| AdminApiError::invalid("invalidDomain", error, "/name"))?;
    let plan = crate::image_proxy::load_plan(&state.store)
        .await
        .map_err(|_| {
            AdminApiError::conflict(
                "discoveryUnavailable",
                "Egress configuration is unavailable.",
            )
        })?;
    let resolver = mw_autoconfig::HickoryResolver::new().map_err(|_| {
        AdminApiError::conflict("discoveryUnavailable", "DNS resolver is unavailable.")
    })?;
    let fetcher = RoutedFetcher {
        plan,
        domain: name.clone(),
        resolver: Box::new(resolver),
    };
    let response = propose(&fetcher, &name, state.engine.is_some()).await?;
    crate::admin_accounts::audit(
        &state,
        &ctx,
        "domain-detected",
        &format!("domain:{name}"),
        None,
        None,
    )
    .await;
    Ok(Json(response))
}

/// Runs the discovery ladder for `name` through `fetcher` and maps the result to
/// an upstream proposal for the server mode: IMAP or POP3 plus SMTP when the mail
/// engine runs (`engine_mode`), a JMAP session otherwise. Nothing is saved.
async fn propose(
    fetcher: &dyn Fetcher,
    name: &str,
    engine_mode: bool,
) -> Result<Value, AdminApiError> {
    let candidate = tokio::time::timeout(
        std::time::Duration::from_secs(30),
        mw_autoconfig::discover_with(&format!("postmaster@{name}"), fetcher),
    )
    .await
    .map_err(|_| AdminApiError::conflict("discoveryTimeout", "Discovery timed out."))?
    .map_err(|_| AdminApiError::not_found("No upstream configuration was discovered."))?;
    let endpoint = |spec: &mw_autoconfig::ServerSpec| {
        json!({
            "host":spec.host,"port":spec.port,"tls":match spec.tls {
                mw_autoconfig::TlsMode::Implicit => "implicit",
                mw_autoconfig::TlsMode::StartTls => "starttls",
                mw_autoconfig::TlsMode::None => "none",
            },"auth":"password","allowPrivate":false
        })
    };
    let proposal = if engine_mode && candidate.source != mw_autoconfig::DiscoverySource::Jmap {
        // Every candidate carries IMAP; a published POP3 server is the lesser
        // alternative, so it is never proposed in IMAP's place.
        json!({"imap":endpoint(&candidate.imap),"smtp":endpoint(&candidate.smtp)})
    } else if !engine_mode && candidate.source == mw_autoconfig::DiscoverySource::Jmap {
        json!({"jmap":{"sessionUrl":format!("https://{name}/.well-known/jmap")}})
    } else {
        return Err(AdminApiError::conflict(
            "discoveryModeMismatch",
            "The discovered protocol does not match the server mode.",
        ));
    };
    crate::domain_config::UpstreamConfig::parse(&proposal.to_string()).map_err(|_| {
        AdminApiError::conflict(
            "discoveryUnsafe",
            "The discovered endpoint requires manual review.",
        )
    })?;
    Ok(
        json!({"name":name,"upstream":proposal,"source":candidate.source,
        "requiresOAuth":candidate.auth == mw_autoconfig::AuthMethod::OAuth2,"saved":false}),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::http::StatusCode;
    use mw_autoconfig::SrvRecord;
    use std::collections::HashMap;

    /// Mock DNS (SRV name to records) and mock HTTP (URL to body). A name or URL
    /// that is not listed answers like a missing record or a non-200 page.
    #[derive(Default)]
    struct Mock {
        pages: HashMap<String, String>,
        srv: HashMap<String, Vec<SrvRecord>>,
    }

    impl Mock {
        fn page(mut self, url: &str, body: &str) -> Self {
            self.pages.insert(url.into(), body.into());
            self
        }
        fn srv(mut self, name: &str, records: &[(&str, u16, u16, u16)]) -> Self {
            let records = records
                .iter()
                .map(|&(target, port, priority, weight)| SrvRecord {
                    target: target.into(),
                    port,
                    priority,
                    weight,
                })
                .collect();
            self.srv.insert(name.into(), records);
            self
        }
    }

    #[async_trait::async_trait]
    impl Fetcher for Mock {
        async fn srv(&self, service: &str) -> Result<Vec<SrvRecord>, DiscoverError> {
            Ok(self.srv.get(service).cloned().unwrap_or_default())
        }
        async fn get(&self, url: &str) -> Result<Option<String>, DiscoverError> {
            Ok(self.pages.get(url).cloned())
        }
    }

    #[async_trait::async_trait]
    impl SrvResolver for Mock {
        async fn lookup_srv(&self, service: &str) -> Result<Vec<SrvRecord>, DiscoverError> {
            Fetcher::srv(self, service).await
        }
    }

    const JMAP_SESSION: &str = r#"{"capabilities":{"urn:ietf:params:jmap:core":{}},
        "apiUrl":"https://jmap.example.org/api/"}"#;

    const PROVIDER_AUTOCONFIG: &str =
        "https://autoconfig.example.org/mail/config-v1.1.xml?emailaddress=postmaster@example.org";

    fn endpoint(host: &str, port: u16, tls: &str) -> Value {
        json!({"host":host,"port":port,"tls":tls,"auth":"password","allowPrivate":false})
    }

    async fn refused(mock: Mock, engine_mode: bool) -> AdminApiError {
        match propose(&mock, "example.org", engine_mode).await {
            Ok(proposal) => panic!("expected a refusal, got {proposal}"),
            Err(error) => error,
        }
    }

    #[tokio::test]
    async fn jmap_well_known_proposes_the_session_url_without_the_engine() {
        let mock = Mock::default()
            .page("https://example.org/.well-known/jmap", JMAP_SESSION)
            // A JMAP session wins over SRV records for the same domain.
            .srv(
                "_imaps._tcp.example.org",
                &[("imap.example.org.", 993, 0, 0)],
            )
            .srv(
                "_submission._tcp.example.org",
                &[("smtp.example.org.", 587, 0, 0)],
            );
        let response = propose(&mock, "example.org", false).await.unwrap();
        assert_eq!(
            response,
            json!({"name":"example.org","source":"jmap","requiresOAuth":false,"saved":false,
                "upstream":{"jmap":{"sessionUrl":"https://example.org/.well-known/jmap"}}})
        );
    }

    #[tokio::test]
    async fn the_protocol_must_match_the_server_mode() {
        let jmap = Mock::default().page("https://example.org/.well-known/jmap", JMAP_SESSION);
        let error = refused(jmap, true).await;
        assert_eq!(
            (error.status, &*error.code),
            (StatusCode::CONFLICT, "discoveryModeMismatch")
        );
        let srv = Mock::default()
            .srv(
                "_imaps._tcp.example.org",
                &[("imap.example.org.", 993, 0, 0)],
            )
            .srv(
                "_submission._tcp.example.org",
                &[("smtp.example.org.", 587, 0, 0)],
            );
        let error = refused(srv, false).await;
        assert_eq!(&*error.code, "discoveryModeMismatch");
    }

    #[tokio::test]
    async fn srv_records_pick_the_lowest_priority_then_the_highest_weight() {
        let mock = Mock::default()
            .srv(
                "_imaps._tcp.example.org",
                &[
                    ("backup.example.org.", 993, 20, 100),
                    ("light.example.org.", 9993, 10, 1),
                    ("heavy.example.org.", 993, 10, 50),
                ],
            )
            // No implicit-TLS submission record: STARTTLS submission is used.
            .srv(
                "_submission._tcp.example.org",
                &[("smtp.example.org.", 587, 5, 0)],
            );
        let response = propose(&mock, "example.org", true).await.unwrap();
        assert_eq!(response["source"], "srv");
        assert_eq!(
            response["upstream"],
            json!({"imap":endpoint("heavy.example.org",993,"implicit"),
                "smtp":endpoint("smtp.example.org",587,"starttls")})
        );
    }

    #[tokio::test]
    async fn srv_prefers_implicit_tls_and_skips_an_unavailable_service() {
        let mock = Mock::default()
            // A "." target means the service is not offered (RFC 6186).
            .srv("_imaps._tcp.example.org", &[(".", 0, 0, 0)])
            .srv(
                "_imap._tcp.example.org",
                &[("imap.example.org.", 143, 0, 0)],
            )
            .srv(
                "_submissions._tcp.example.org",
                &[("smtp.example.org.", 465, 0, 0)],
            )
            .srv(
                "_submission._tcp.example.org",
                &[("smtp.example.org.", 587, 0, 0)],
            );
        let response = propose(&mock, "example.org", true).await.unwrap();
        assert_eq!(
            response["upstream"],
            json!({"imap":endpoint("imap.example.org",143,"starttls"),
                "smtp":endpoint("smtp.example.org",465,"implicit")})
        );
    }

    #[tokio::test]
    async fn imap_is_proposed_when_pop3_is_also_published() {
        let srv = Mock::default()
            .srv(
                "_imaps._tcp.example.org",
                &[("imap.example.org.", 993, 5, 0)],
            )
            .srv(
                "_pop3s._tcp.example.org",
                &[("pop.example.org.", 995, 20, 0)],
            )
            .srv(
                "_submission._tcp.example.org",
                &[("smtp.example.org.", 587, 5, 0)],
            );
        let xml = r#"<clientConfig version="1.1"><emailProvider id="example.org">
            <incomingServer type="pop3"><hostname>pop.example.org</hostname><port>995</port>
              <socketType>SSL</socketType></incomingServer>
            <incomingServer type="imap"><hostname>imap.example.org</hostname><port>993</port>
              <socketType>SSL</socketType></incomingServer>
            <outgoingServer type="smtp"><hostname>smtp.example.org</hostname><port>587</port>
              <socketType>STARTTLS</socketType></outgoingServer>
            </emailProvider></clientConfig>"#;
        let autoconfig = Mock::default().page(PROVIDER_AUTOCONFIG, xml);
        for mock in [srv, autoconfig] {
            let response = propose(&mock, "example.org", true).await.unwrap();
            assert_eq!(
                response["upstream"],
                json!({"imap":endpoint("imap.example.org",993,"implicit"),
                    "smtp":endpoint("smtp.example.org",587,"starttls")})
            );
        }
    }

    #[tokio::test]
    async fn provider_autoconfig_xml_is_read_for_the_postmaster_address() {
        let xml = r#"<clientConfig version="1.1"><emailProvider id="example.org">
            <incomingServer type="imap"><hostname>mail.example.org</hostname><port>993</port>
              <socketType>SSL</socketType><authentication>OAuth2</authentication></incomingServer>
            <outgoingServer type="smtp"><hostname>mail.example.org</hostname><port>587</port>
              <socketType>STARTTLS</socketType></outgoingServer>
            </emailProvider></clientConfig>"#;
        let mock = Mock::default().page(PROVIDER_AUTOCONFIG, xml);
        let response = propose(&mock, "example.org", true).await.unwrap();
        assert_eq!(response["source"], "thunderbird-autoconfig");
        assert_eq!(response["requiresOAuth"], true);
        assert_eq!(
            response["upstream"],
            json!({"imap":endpoint("mail.example.org",993,"implicit"),
                "smtp":endpoint("mail.example.org",587,"starttls")})
        );
    }

    #[tokio::test]
    async fn the_ispdb_is_asked_when_the_domain_hosts_no_autoconfig() {
        let xml = r#"<clientConfig version="1.1"><emailProvider id="example.org">
            <incomingServer type="imap"><hostname>imap.isp.example</hostname><port>143</port>
              <socketType>STARTTLS</socketType></incomingServer>
            <outgoingServer type="smtp"><hostname>smtp.isp.example</hostname><port>465</port>
              <socketType>SSL</socketType></outgoingServer>
            </emailProvider></clientConfig>"#;
        let mock = Mock::default()
            .page(
                PROVIDER_AUTOCONFIG,
                "<html>not an autoconfig document</html>",
            )
            .page("https://autoconfig.thunderbird.net/v1.1/example.org", xml);
        let response = propose(&mock, "example.org", true).await.unwrap();
        assert_eq!(response["requiresOAuth"], false);
        assert_eq!(
            response["upstream"],
            json!({"imap":endpoint("imap.isp.example",143,"starttls"),
                "smtp":endpoint("smtp.isp.example",465,"implicit")})
        );
    }

    #[tokio::test]
    async fn autodiscover_v2_json_is_the_last_network_rung() {
        let body = r#"{"Protocols":[
            {"Type":"IMAP","Server":"outlook.example.org","Port":993,"SSL":true},
            {"Type":"SMTP","Server":"smtp.example.org","Port":587,"SSL":false,"Encryption":"STARTTLS"}]}"#;
        let mock = Mock::default().page("https://example.org/autodiscover/autodiscover.json", body);
        let response = propose(&mock, "example.org", true).await.unwrap();
        assert_eq!(response["source"], "autodiscover");
        assert_eq!(
            response["upstream"],
            json!({"imap":endpoint("outlook.example.org",993,"implicit"),
                "smtp":endpoint("smtp.example.org",587,"starttls")})
        );
    }

    #[tokio::test]
    async fn a_private_or_local_discovered_host_needs_manual_review() {
        for target in ["10.0.0.5.", "localhost.", "mail.localhost."] {
            let mock = Mock::default()
                .srv("_imaps._tcp.example.org", &[(target, 993, 0, 0)])
                .srv(
                    "_submission._tcp.example.org",
                    &[("smtp.example.org.", 587, 0, 0)],
                );
            let error = refused(mock, true).await;
            assert_eq!(
                (error.status, &*error.code),
                (StatusCode::CONFLICT, "discoveryUnsafe"),
                "{target}"
            );
        }
    }

    #[tokio::test]
    async fn nothing_published_is_not_found() {
        let error = refused(Mock::default(), true).await;
        assert_eq!(error.status, StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    async fn the_routed_fetcher_resolves_srv_through_its_resolver() {
        let fetcher = RoutedFetcher {
            plan: EgressPlan::direct(),
            domain: "example.org".into(),
            resolver: Box::new(Mock::default().srv(
                "_imaps._tcp.example.org",
                &[("imap.example.org.", 993, 1, 2)],
            )),
        };
        let records = fetcher.srv("_imaps._tcp.example.org").await.unwrap();
        assert_eq!(records.len(), 1);
        assert_eq!(
            (records[0].target.as_str(), records[0].port),
            ("imap.example.org.", 993)
        );
        assert!(
            fetcher
                .srv("_imap._tcp.example.org")
                .await
                .unwrap()
                .is_empty()
        );
    }

    /// Discovery URLs are built from operator input and DNS answers, so every hop
    /// goes through the admin-detect egress gate: a loopback page is never
    /// fetched, even when it answers.
    #[tokio::test]
    async fn the_routed_fetcher_refuses_a_loopback_target() {
        use std::sync::{
            Arc,
            atomic::{AtomicUsize, Ordering},
        };
        use tokio::io::AsyncWriteExt;
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let port = listener.local_addr().unwrap().port();
        let accepted = Arc::new(AtomicUsize::new(0));
        let seen = accepted.clone();
        tokio::spawn(async move {
            while let Ok((mut socket, _)) = listener.accept().await {
                seen.fetch_add(1, Ordering::SeqCst);
                let reply = format!(
                    "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{JMAP_SESSION}",
                    JMAP_SESSION.len()
                );
                let _ = socket.write_all(reply.as_bytes()).await;
            }
        });
        let fetcher = RoutedFetcher {
            plan: EgressPlan::direct(),
            domain: "example.org".into(),
            resolver: Box::new(Mock::default()),
        };
        for url in [
            format!("http://127.0.0.1:{port}/.well-known/jmap"),
            format!("https://127.0.0.1:{port}/.well-known/jmap"),
            format!("http://localhost:{port}/.well-known/jmap"),
        ] {
            assert_eq!(fetcher.get(&url).await.unwrap(), None, "{url}");
        }
        assert_eq!(accepted.load(Ordering::SeqCst), 0);
    }
}
