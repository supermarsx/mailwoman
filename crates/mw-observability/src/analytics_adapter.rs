//! Fixed event schemas. No browser URL, identity or arbitrary metadata crosses this seam.
use super::analytics_config::{Config, Provider};
use serde::{Deserialize, Serialize};
use serde_json::json;

pub const USER_AGENT: &str = "Mailwoman/1.0 (consented application events)";
#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Screen {
    Mail,
    Calendar,
    Tasks,
    Contacts,
    Notes,
    Settings,
}
impl Screen {
    pub fn name(self) -> &'static str {
        match self {
            Self::Mail => "mail",
            Self::Calendar => "calendar",
            Self::Tasks => "tasks",
            Self::Contacts => "contacts",
            Self::Notes => "notes",
            Self::Settings => "settings",
        }
    }
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Event {
    pub screen: Screen,
}

pub fn payload(config: &Config, screen: Screen) -> (&'static str, Vec<u8>) {
    match config.provider {
        Provider::Ga4 => {
            let random = uuid::Uuid::new_v4().as_u128();
            let client_id = format!("{}.{}", ((random >> 64) as u64).max(1), (random as u64).max(1));
            ("application/json", serde_json::to_vec(&json!({
                "client_id": client_id,
                "consent": {"ad_user_data":"DENIED","ad_personalization":"DENIED"},
                "events":[{"name":"screen_view","params":{"screen_name":screen.name()}}]
            })).expect("fixed analytics payload"))
        },
        Provider::Umami => ("application/json", serde_json::to_vec(&json!({
            "type":"event", "payload": { "website":config.site_id, "hostname":"mailwoman.invalid",
                "url":format!("/app/{}",screen.name()), "title":"Mailwoman", "name":"screen-view",
                "referrer":"", "data":{"screen":screen.name()} }
        })).expect("fixed analytics payload")),
        Provider::Matomo => {
            let mut body = reqwest::Url::parse("https://mailwoman.invalid/").expect("constant URL");
            body.query_pairs_mut().append_pair("idsite", &config.site_id).append_pair("rec", "1")
                .append_pair("apiv", "1").append_pair("url", &format!("https://mailwoman.invalid/app/{}", screen.name()))
                .append_pair("action_name", "Mailwoman").append_pair("e_c", "application")
                .append_pair("e_a", "screen-view").append_pair("e_n", screen.name());
            ("application/x-www-form-urlencoded", body.query().unwrap_or_default().as_bytes().to_vec())
        }
    }
}

pub async fn deliver(
    store: &mw_store::Store,
    config: &Config,
    screen: Screen,
    domain: Option<&str>,
) -> Result<(), &'static str> {
    let plan = mw_egress::plan::current().ok_or("configuration")?;
    let url = collector_url(store, config)?;
    deliver_url(&plan, config, screen, domain, url).await
}

fn collector_url(store: &mw_store::Store, config: &Config) -> Result<reqwest::Url, &'static str> {
    config.validate().map_err(|_| "configuration")?;
    let mut url = reqwest::Url::parse(&config.endpoint).map_err(|_| "configuration")?;
    if config.provider == Provider::Ga4 {
        let sealed = config.api_secret_sealed.as_deref().ok_or("configuration")?;
        let secret = store
            .open_analytics_secret(sealed, &config.endpoint, &config.site_id)
            .map_err(|_| "configuration")?;
        let secret = std::str::from_utf8(&secret).map_err(|_| "configuration")?;
        url.query_pairs_mut()
            .append_pair("measurement_id", &config.site_id)
            .append_pair("api_secret", secret);
    }
    Ok(url)
}

#[cfg(test)]
async fn deliver_with(
    plan: &mw_egress::plan::EgressPlan,
    config: &Config,
    screen: Screen,
    domain: Option<&str>,
) -> Result<(), &'static str> {
    let url = reqwest::Url::parse(&config.endpoint).map_err(|_| "configuration")?;
    deliver_url(plan, config, screen, domain, url).await
}

async fn deliver_url(
    plan: &mw_egress::plan::EgressPlan,
    config: &Config,
    screen: Screen,
    domain: Option<&str>,
    url: reqwest::Url,
) -> Result<(), &'static str> {
    let (content_type, bytes) = payload(config, screen);
    let mut request =
        mw_egress::plan::Request::post(mw_egress::TrafficClass::Analytics, content_type, &bytes)
            .response_limit(8192)
            .user_agent(USER_AGENT);
    if let Some(domain) = domain {
        request = request.for_domain(domain);
    }
    let response = tokio::time::timeout(
        std::time::Duration::from_secs(3),
        mw_egress::plan::send_with(plan, url, &request),
    )
    .await
    .map_err(|_| "timeout")?;
    let bytes = response.outcome.map_err(|_| "collector-refused")?;
    collector_response(config.provider, &bytes)
}

fn collector_response(provider: Provider, bytes: &[u8]) -> Result<(), &'static str> {
    if provider == Provider::Umami
        && serde_json::from_slice::<serde_json::Value>(bytes)
            .ok()
            .is_some_and(|body| body["beep"] == "boop")
    {
        return Err("collector-filtered");
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::io::{AsyncBufReadExt, AsyncReadExt, AsyncWriteExt, BufReader};

    #[tokio::test]
    async fn real_mock_collectors_receive_only_fixed_payloads_through_selected_proxy() {
        for provider in [Provider::Umami, Provider::Matomo, Provider::Ga4] {
            let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
            let plan = mw_egress::plan::EgressPlan::new(
                vec![mw_egress::plan::PlanRoute {
                    scope: mw_egress::plan::RouteScope::Domain("example.test".into()),
                    classes: [mw_egress::TrafficClass::Analytics].into_iter().collect(),
                    route: mw_egress::proxy::ProxyRoute {
                        id: "collector-test".into(),
                        scheme: mw_egress::proxy::ProxyScheme::HttpConnect,
                        host: "127.0.0.1".into(),
                        port: listener.local_addr().unwrap().port(),
                        auth: None,
                        allow_plaintext: true,
                    },
                }],
                mw_egress::InternalAllowList::empty(),
            )
            .unwrap();
            let recorder = tokio::spawn(async move {
                let (socket, _) = listener.accept().await.unwrap();
                let mut socket = BufReader::new(socket);
                let mut first = String::new();
                socket.read_line(&mut first).await.unwrap();
                assert_eq!(first, "CONNECT 8.8.8.8:80 HTTP/1.1\r\n");
                loop {
                    let mut line = String::new();
                    socket.read_line(&mut line).await.unwrap();
                    if line == "\r\n" {
                        break;
                    }
                }
                socket
                    .get_mut()
                    .write_all(b"HTTP/1.1 200 Connection established\r\n\r\n")
                    .await
                    .unwrap();
                let mut headers = String::new();
                let mut length = 0;
                loop {
                    let mut line = String::new();
                    socket.read_line(&mut line).await.unwrap();
                    if line == "\r\n" {
                        break;
                    }
                    if let Some(value) = line.to_ascii_lowercase().strip_prefix("content-length:") {
                        length = value.trim().parse::<usize>().unwrap();
                    }
                    headers.push_str(&line);
                }
                assert!(length < 2048);
                let mut body = vec![0; length];
                socket.read_exact(&mut body).await.unwrap();
                socket
                    .get_mut()
                    .write_all(
                        b"HTTP/1.1 200 OK\r\nContent-Length: 2\r\nConnection: close\r\n\r\n{}",
                    )
                    .await
                    .unwrap();
                (headers, body)
            });
            let path = match provider {
                Provider::Umami => "api/send",
                Provider::Matomo => "matomo.php",
                Provider::Ga4 => "mp/collect?measurement_id=G-TEST123&api_secret=fixture_secret",
            };
            let config = Config {
                enabled: true,
                provider,
                endpoint: format!("http://8.8.8.8/{path}"),
                site_id: if provider == Provider::Umami {
                    "f2997167-36cd-4e06-9054-abcc5c8cf476"
                } else {
                    "2"
                }
                .into(),
                allow_http: true,
                api_secret_sealed: None,
            };
            deliver_with(&plan, &config, Screen::Mail, Some("example.test"))
                .await
                .unwrap();
            let (headers, body) = recorder.await.unwrap();
            assert!(headers.starts_with(&format!("POST /{path} HTTP/1.1")));
            assert!(
                headers
                    .to_ascii_lowercase()
                    .contains("user-agent: mailwoman/1.0 (consented application events)")
            );
            assert!(!headers.to_ascii_lowercase().contains("cookie:"));
            assert!(!headers.to_ascii_lowercase().contains("authorization:"));
            if provider == Provider::Ga4 {
                let body: serde_json::Value = serde_json::from_slice(&body).unwrap();
                assert_eq!(body["events"][0]["name"], "screen_view");
                assert_eq!(body["events"][0]["params"], json!({"screen_name":"mail"}));
                assert_eq!(body["consent"]["ad_user_data"], "DENIED");
                assert_eq!(body.as_object().unwrap().len(), 3);
                assert!(!body.to_string().contains("fixture_secret"));
            } else {
                assert_eq!(body, payload(&config, Screen::Mail).1);
            }
        }
    }
    #[test]
    fn schemas_cannot_accept_identity_or_freeform_events() {
        assert_eq!(
            collector_response(Provider::Umami, br#"{"beep":"boop"}"#),
            Err("collector-filtered")
        );
        assert!(
            collector_response(
                Provider::Umami,
                br#"{"cache":"accepted","sessionId":"unused"}"#
            )
            .is_ok()
        );
        for raw in [
            r#"{"screen":"user@example.com"}"#,
            r#"{"screen":"mail","url":"/mail?token=secret"}"#,
            r#"{"screen":"mail","account":"private"}"#,
        ] {
            assert!(serde_json::from_str::<Event>(raw).is_err());
        }
        let config = Config {
            site_id: "f2997167-36cd-4e06-9054-abcc5c8cf476".into(),
            ..Config::default()
        };
        let (mime, bytes) = payload(&config, Screen::Mail);
        assert_eq!(mime, "application/json");
        let body: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
        assert_eq!(
            body,
            json!({"type":"event","payload":{"website":config.site_id,"hostname":"mailwoman.invalid","url":"/app/mail","title":"Mailwoman","name":"screen-view","referrer":"","data":{"screen":"mail"}}})
        );
        let (mime, bytes) = payload(
            &Config {
                provider: Provider::Matomo,
                site_id: "2".into(),
                ..Config::default()
            },
            Screen::Contacts,
        );
        assert_eq!(mime, "application/x-www-form-urlencoded");
        let mut url = reqwest::Url::parse("https://test.invalid").unwrap();
        url.set_query(Some(std::str::from_utf8(&bytes).unwrap()));
        let pairs: std::collections::BTreeMap<_, _> = url.query_pairs().into_owned().collect();
        assert_eq!(pairs.len(), 8);
        assert_eq!(pairs["e_n"], "contacts");
        assert_eq!(pairs["url"], "https://mailwoman.invalid/app/contacts");
        assert!(!pairs.contains_key("uid"));
        assert!(!pairs.contains_key("cip"));
    }
    #[test]
    fn ga4_contract_has_fresh_event_identifier_and_denied_advertising_consent() {
        let config = Config {
            provider: Provider::Ga4,
            ..Config::default()
        };
        let first: serde_json::Value =
            serde_json::from_slice(&payload(&config, Screen::Mail).1).unwrap();
        let second: serde_json::Value =
            serde_json::from_slice(&payload(&config, Screen::Mail).1).unwrap();
        assert_ne!(first["client_id"], second["client_id"]);
        assert!(
            first["client_id"]
                .as_str()
                .unwrap()
                .split('.')
                .all(|part| part.parse::<u64>().is_ok_and(|n| n > 0))
        );
        assert_eq!(first.as_object().unwrap().len(), 3);
        assert_eq!(
            first["consent"],
            json!({"ad_user_data":"DENIED", "ad_personalization":"DENIED"})
        );
        assert_eq!(
            first["events"],
            json!([{"name":"screen_view","params":{"screen_name":"mail"}}])
        );
        assert!(collector_response(Provider::Ga4, b"").is_ok());
    }
    #[tokio::test]
    async fn ga4_url_opens_only_the_bound_secret_at_the_fixed_authority() {
        let store = mw_store::Store::open_in_memory(mw_store::ServerKey::generate())
            .await
            .unwrap();
        let update: crate::analytics_config::Update = serde_json::from_value(json!({
            "enabled":true,"provider":"ga4","endpoint":"https://region1.google-analytics.com/mp/collect",
            "siteId":"G-TEST123","apiSecret":"test_secret"
        })).unwrap();
        let mut config = update.prepare(&Config::default(), &store).unwrap();
        let url = collector_url(&store, &config).unwrap();
        assert_eq!(url.host_str(), Some("region1.google-analytics.com"));
        assert_eq!(url.scheme(), "https");
        assert_eq!(url.path(), "/mp/collect");
        let pairs: std::collections::BTreeMap<_, _> = url.query_pairs().into_owned().collect();
        assert_eq!(pairs.len(), 2);
        assert_eq!(pairs["measurement_id"], "G-TEST123");
        assert_eq!(pairs["api_secret"], "test_secret");
        config.endpoint = "https://www.google-analytics.com/mp/collect".into();
        assert_eq!(collector_url(&store, &config).unwrap_err(), "configuration");
    }
}
