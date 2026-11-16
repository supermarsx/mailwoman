//! Host-bound plugin requests through the current administrator egress policy.
use base64::Engine as _;
use mw_plugin::{HttpReq, HttpResp};
use mw_store::Store;
use std::{
    collections::HashMap,
    sync::{Arc, Mutex, OnceLock},
    time::{Duration, Instant},
};

struct AuthConnection {
    connection: Arc<mw_egress::PluginConnection>,
    url: String,
    created: Instant,
}
type AuthKey = (String, String, String);
fn connections() -> &'static Mutex<HashMap<AuthKey, AuthConnection>> {
    static SESSIONS: OnceLock<Mutex<HashMap<AuthKey, AuthConnection>>> = OnceLock::new();
    SESSIONS.get_or_init(Mutex::default)
}
/// Correlation is host-only: strip it before any upstream request. One NTLM
/// NEGOTIATE creates the lease; AUTHENTICATE consumes it exactly once.
pub(crate) fn authentication_connection(
    plugin: &str,
    account: Option<&str>,
    url: &str,
    headers: &mut Vec<(String, String)>,
) -> Result<Option<Arc<mw_egress::PluginConnection>>, String> {
    // Apply the transport's limits before cloning correlation fields or
    // decoding an untrusted base64 token. The one extra field is stripped here.
    if headers.len() > 17
        || headers.iter().any(|(_, value)| value.len() > 16384)
        || headers
            .iter()
            .map(|(name, value)| name.len() + value.len())
            .sum::<usize>()
            > 32824
    {
        return Err("authentication headers exceed transport limits".into());
    }
    let markers = headers
        .iter()
        .filter(|(name, _)| name.eq_ignore_ascii_case("x-mailwoman-auth-session"))
        .map(|(_, value)| value.clone())
        .collect::<Vec<_>>();
    headers.retain(|(name, _)| !name.eq_ignore_ascii_case("x-mailwoman-auth-session"));
    if markers.is_empty() {
        return Ok(None);
    }
    let token = &markers[0];
    if plugin != "bridge-ews"
        || account.is_none()
        || markers.len() != 1
        || token.len() != 32
        || !token
            .bytes()
            .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
    {
        return Err("invalid authentication connection".into());
    }
    let authorizations = headers
        .iter()
        .filter(|(name, _)| name.eq_ignore_ascii_case("authorization"))
        .map(|(_, value)| value.as_str())
        .collect::<Vec<_>>();
    if authorizations.len() != 1 {
        return Err("authentication requires one credential".into());
    }
    let value = authorizations[0]
        .strip_prefix("NTLM ")
        .ok_or("authentication scheme mismatch")?;
    let message = base64::engine::general_purpose::STANDARD
        .decode(value)
        .map_err(|_| "invalid authentication token")?;
    if message.len() < 12 || message.len() > 16384 || &message[..8] != b"NTLMSSP\0" {
        return Err("invalid authentication token".into());
    }
    let phase = u32::from_le_bytes(message[8..12].try_into().unwrap());
    let key = (
        plugin.to_owned(),
        account.unwrap().to_owned(),
        token.clone(),
    );
    let now = Instant::now();
    let mut sessions = connections()
        .lock()
        .map_err(|_| "authentication connection unavailable")?;
    sessions.retain(|_, value| now.duration_since(value.created) < Duration::from_secs(30));
    match phase {
        1 => {
            if sessions.len() >= 256 || sessions.contains_key(&key) {
                return Err("authentication connection limit".into());
            }
            let connection = Arc::new(mw_egress::PluginConnection::default());
            sessions.insert(
                key.clone(),
                AuthConnection {
                    connection: connection.clone(),
                    url: url.into(),
                    created: now,
                },
            );
            tokio::spawn(async move {
                tokio::time::sleep(Duration::from_secs(30)).await;
                if let Ok(mut held) = connections().lock() {
                    if held.get(&key).is_some_and(|value| value.created == now) {
                        held.remove(&key);
                    }
                }
            });
            Ok(Some(connection))
        }
        3 => {
            let held = sessions
                .remove(&key)
                .ok_or("authentication connection expired")?;
            if held.url != url {
                return Err("authentication endpoint changed".into());
            }
            Ok(Some(held.connection))
        }
        _ => Err("invalid authentication phase".into()),
    }
}

pub(super) async fn fetch(
    store: &Store,
    plugin: &str,
    account: Option<&str>,
    mut req: HttpReq,
) -> Result<HttpResp, String> {
    let operation = async {
        crate::features::plugin_policy_admit(store, plugin, account).await?;
        let row = store
            .get_plugin(plugin)
            .await
            .map_err(|_| "plugin registry unavailable")?
            .ok_or("plugin unavailable")?;
        if !row.enabled || row.approved_by.is_none() {
            return Err("plugin unavailable".into());
        }
        let grants = store
            .plugin_grants_scoped(plugin, account)
            .await
            .map_err(|_| "plugin grants unavailable")?;
        if !grants.iter().any(|grant| grant == "net") {
            return Err("plugin network capability revoked".into());
        }
        let url = reqwest::Url::parse(&req.url).map_err(|_| "invalid plugin URL")?;
        let host = url.host_str().ok_or("invalid plugin host")?;
        let allowed: Vec<String> = serde_json::from_str(&row.net_allowlist_json)
            .map_err(|_| "invalid plugin permissions")?;
        let mut endpoint_grant = false;
        let mut additional_roots = Vec::new();
        let canonical = if let Some(account) = account {
            if matches!(plugin, "bridge-ews" | "bridge-graph" | "bridge-gmail") {
                let binding = store
                    .get_bridge_account(account)
                    .await
                    .map_err(|_| "account binding unavailable")?;
                if !binding.is_some_and(|binding| binding.bridge_id == plugin) {
                    return Err("account provider binding changed".into());
                }
            }
            let provider = if plugin == "bridge-ews" {
                crate::domain_provider::stored_selection(store, account).await?
            } else {
                None
            };
            // Verified provider enrollment precedes the login audit/session row.
            // Initial indexing must use that durable identity, not race the later
            // signin_identity projection. Legacy bridges retain their old lookup.
            let address = if let Some(selected) = &provider {
                selected.login_address.clone()
            } else {
                store
                    .security_account_address(account)
                    .await
                    .map_err(|_| "account identity unavailable")?
                    .ok_or("account identity unavailable")?
            };
            let canonical = store
                .canonical_policy_address(&address)
                .await
                .map_err(|_| "account identity unavailable")?;
            if plugin == "bridge-ews" {
                if let Some(selected) = provider {
                    endpoint_grant = url == selected.config.endpoint()?;
                    additional_roots = selected.config.roots()?;
                } else {
                    let credential = store
                        .get_ews_account_cred(account)
                        .await
                        .map_err(|_| "account credentials unavailable")?
                        .ok_or("account credentials unavailable")?;
                    let endpoint = reqwest::Url::parse(&credential.endpoint)
                        .map_err(|_| "invalid account endpoint")?;
                    endpoint_grant = credential.enabled
                        && url.origin() == endpoint.origin()
                        && url.path() == endpoint.path()
                        && url.query() == endpoint.query();
                }
                if !endpoint_grant {
                    return Err("account endpoint mismatch".into());
                }
            }
            Some(canonical)
        } else {
            None
        };
        if !endpoint_grant && !allowed.iter().any(|h| h.eq_ignore_ascii_case(host)) {
            return Err("plugin host permission revoked".into());
        }
        if req
            .headers
            .iter()
            .any(|(name, _)| name.eq_ignore_ascii_case("authorization"))
        {
            if account.is_none() {
                return Err("credential request requires a bound account".into());
            }
            let authority = match plugin {
                "bridge-ews" => endpoint_grant,
                "bridge-graph" => {
                    url.scheme() == "https"
                        && host == "graph.microsoft.com"
                        && url.port_or_known_default() == Some(443)
                }
                "bridge-gmail" => {
                    url.scheme() == "https"
                        && host == "gmail.googleapis.com"
                        && url.port_or_known_default() == Some(443)
                }
                _ => false,
            };
            if !authority {
                return Err("credential authority refused".into());
            }
        }
        let plan = crate::image_proxy::load_plan(store)
            .await
            .map_err(|_| "plugin egress unavailable")?;
        let connection = authentication_connection(plugin, account, &req.url, &mut req.headers)?;
        let metadata = mw_egress::ResponseMetadata::default();
        let body = req.body.as_deref().map(|bytes| mw_egress::OutBody {
            content_type: "application/octet-stream",
            bytes,
        });
        let mut request = mw_egress::plan::Request::plugin(
            mw_egress::TrafficClass::Plugins,
            mw_egress::PluginHttp {
                connection: connection.as_deref(),
                additional_roots: &additional_roots,
                method: &req.method,
                headers: &req.headers,
            },
            body,
        )
        .map_err(|_| "invalid plugin HTTP request")?
        .capture_response(&metadata);
        if let Some(domain) = canonical
            .as_deref()
            .and_then(|address| address.rsplit_once('@').map(|(_, domain)| domain))
        {
            request = request.for_domain(domain);
        }
        let fetched = mw_egress::plan::send_with(&plan, url.clone(), &request).await;
        let body = fetched
            .outcome
            .map_err(|_| "plugin egress request refused or failed")?;
        let head = metadata.snapshot().ok_or("plugin response unavailable")?;
        let mut headers = Vec::new();
        if let Some(kind) = fetched.content_type {
            headers.push(("content-type".into(), kind));
        }
        if let Some(etag) = head.etag {
            headers.push(("etag".into(), etag));
        }
        if plugin == "bridge-ews" {
            headers.extend(
                metadata
                    .authentication_challenges()
                    .into_iter()
                    .map(|value| ("www-authenticate".into(), value)),
            );
            if let Some(connection) = connection {
                if let Some(binding) = connection.tls_server_end_point().await {
                    headers.push((
                        "x-mailwoman-tls-server-end-point".into(),
                        binding.iter().map(|byte| format!("{byte:02x}")).collect(),
                    ));
                }
                if let Some(host) = url.host_str() {
                    headers.push((
                        "x-mailwoman-ntlm-target-name".into(),
                        format!("HTTP/{host}"),
                    ));
                }
            }
        }
        Ok(HttpResp {
            status: head.status,
            headers,
            body,
        })
    };
    tokio::time::timeout(std::time::Duration::from_secs(15), operation)
        .await
        .map_err(|_| "plugin request timed out".to_string())?
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ntlm_headers(id: &str, phase: u32) -> Vec<(String, String)> {
        let mut token = b"NTLMSSP\0".to_vec();
        token.extend_from_slice(&phase.to_le_bytes());
        vec![
            ("X-Mailwoman-Auth-Session".into(), id.into()),
            (
                "Authorization".into(),
                format!(
                    "NTLM {}",
                    base64::engine::general_purpose::STANDARD.encode(token)
                ),
            ),
        ]
    }
    #[tokio::test]
    async fn ntlm_connection_is_bound_stripped_and_consumed_once() {
        let id = uuid::Uuid::new_v4().simple().to_string();
        let url = "https://exchange.test/EWS/Exchange.asmx";
        let mut headers = ntlm_headers(&id, 1);
        let first = authentication_connection("bridge-ews", Some("account-a"), url, &mut headers)
            .unwrap()
            .unwrap();
        assert_eq!(headers.len(), 1);
        assert_eq!(headers[0].0, "Authorization");
        assert!(
            authentication_connection(
                "bridge-ews",
                Some("account-b"),
                url,
                &mut ntlm_headers(&id, 3)
            )
            .is_err()
        );
        let second = authentication_connection(
            "bridge-ews",
            Some("account-a"),
            url,
            &mut ntlm_headers(&id, 3),
        )
        .unwrap()
        .unwrap();
        assert!(Arc::ptr_eq(&first, &second));
        assert!(
            authentication_connection(
                "bridge-ews",
                Some("account-a"),
                url,
                &mut ntlm_headers(&id, 3)
            )
            .is_err()
        );
    }
    #[tokio::test]
    async fn ntlm_connection_refuses_changed_endpoint_unbound_or_wrong_provider() {
        let id = uuid::Uuid::new_v4().simple().to_string();
        let url = "https://exchange.test/EWS/Exchange.asmx";
        let mut oversized = ntlm_headers(&id, 1);
        oversized[1].1 = "NTLM ".to_string() + &"A".repeat(16384);
        assert!(
            authentication_connection("bridge-ews", Some("account-a"), url, &mut oversized)
                .unwrap_err()
                .contains("limits")
        );
        assert!(
            authentication_connection(
                "bridge-graph",
                Some("account-a"),
                url,
                &mut ntlm_headers(&id, 1)
            )
            .is_err()
        );
        assert!(
            authentication_connection("bridge-ews", None, url, &mut ntlm_headers(&id, 1)).is_err()
        );
        authentication_connection(
            "bridge-ews",
            Some("account-a"),
            url,
            &mut ntlm_headers(&id, 1),
        )
        .unwrap();
        assert!(
            authentication_connection(
                "bridge-ews",
                Some("account-a"),
                "https://other.test/EWS/Exchange.asmx",
                &mut ntlm_headers(&id, 3)
            )
            .is_err()
        );
        assert!(
            authentication_connection(
                "bridge-ews",
                Some("account-a"),
                url,
                &mut ntlm_headers("injected\r\nheader", 1)
            )
            .is_err()
        );
    }

    async fn fixture() -> Store {
        let store = Store::open_in_memory(mw_store::ServerKey::generate())
            .await
            .unwrap();
        store
            .put_plugin(&mw_store::PluginRow {
                id: "bridge-ews".into(),
                name: "EWS".into(),
                version: "1".into(),
                signature_hex: None,
                approved_by: Some("fixture-admin".into()),
                enabled: true,
                capabilities_json: "[\"net\",\"account-backend\"]".into(),
                net_allowlist_json: "[\"127.0.0.1\",\"169.254.169.254\"]".into(),
                limits_json: "{}".into(),
                created_at: "2026-10-26T00:00:00Z".into(),
            })
            .await
            .unwrap();
        store
            .put_plugin_grant(&mw_store::PluginGrantRow {
                plugin_id: "bridge-ews".into(),
                account_id: String::new(),
                capability: "net".into(),
                granted_by: "fixture-admin".into(),
                created_at: "2026-10-26T00:00:00Z".into(),
            })
            .await
            .unwrap();
        store
    }
    fn request(url: &str) -> HttpReq {
        HttpReq {
            method: "GET".into(),
            url: url.into(),
            headers: vec![],
            body: None,
        }
    }

    #[tokio::test]
    async fn hostname_grants_do_not_authorize_private_network_or_guest_credentials() {
        let store = fixture().await;
        for url in ["http://127.0.0.1:9/", "http://169.254.169.254/"] {
            let error = fetch(&store, "bridge-ews", None, request(url))
                .await
                .unwrap_err();
            assert!(error.contains("egress"), "{error}");
        }
        let mut credentials = request("http://127.0.0.1:9/");
        credentials
            .headers
            .push(("Authorization".into(), "Basic fixture".into()));
        assert!(
            fetch(&store, "bridge-ews", None, credentials)
                .await
                .unwrap_err()
                .contains("bound account")
        );
        assert!(
            fetch(
                &store,
                "bridge-ews",
                Some("foreign-account"),
                request("http://127.0.0.1:9/")
            )
            .await
            .is_err()
        );
    }

    #[tokio::test]
    async fn stale_account_provider_binding_is_refused_before_any_socket_opens() {
        let store = fixture().await;
        let account = store
            .create_account(
                &mw_store::NewAccount {
                    kind: mw_store::AccountKind::Imap,
                    host: "mail.test",
                    port: 993,
                    tls: "implicit",
                    username: "owner@example.test",
                    sync_policy_json: "{}",
                },
                &mw_store::Credentials {
                    username: "owner@example.test".into(),
                    password: "fixture".into(),
                },
            )
            .await
            .unwrap();
        store
            .put_bridge_account(&mw_store::BridgeAccountRow {
                account_id: account.clone(),
                bridge_id: "bridge-gmail".into(),
                oauth_ref: None,
                extra_json: "{}".into(),
            })
            .await
            .unwrap();
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let url = format!("http://{}/ews", listener.local_addr().unwrap());
        let error = fetch(&store, "bridge-ews", Some(&account), request(&url))
            .await
            .unwrap_err();
        assert_eq!(error, "account provider binding changed");
        assert!(
            tokio::time::timeout(std::time::Duration::from_millis(20), listener.accept())
                .await
                .is_err()
        );
        store
            .replace_plugin_grants("bridge-ews", "", &[], "fixture-admin")
            .await
            .unwrap();
        assert_eq!(
            fetch(&store, "bridge-ews", None, request(&url))
                .await
                .unwrap_err(),
            "plugin network capability revoked"
        );
    }

    #[tokio::test]
    async fn verified_provider_can_apply_egress_before_login_audit_projection() {
        let store = fixture().await;
        let endpoint = "https://127.0.0.1:9/EWS.asmx";
        store
            .put_domain_config(&mw_store::DomainConfigRow {
                name: "example.test".into(),
                enabled: true,
                preferred: false,
                upstream_json: serde_json::json!({"version":2,"backends":[{
                    "id":"ews","role":"mail","protocol":"ews",
                    "ews":{"endpointUrl":endpoint,"authentication":"basic"}
                }]})
                .to_string(),
                allowlist_json: "[]".into(),
                blocklist_json: "[]".into(),
                settings_json: "{}".into(),
                updated_at: 1,
            })
            .await
            .unwrap();
        let selected = crate::domain_provider::SelectedEws::for_login(&store, "owner@example.test")
            .await
            .unwrap()
            .unwrap();
        let account = store
            .enroll_verified_ews(mw_store::provider_accounts::VerifiedEwsEnrollment {
                endpoint_url: endpoint,
                backend_username: &selected.wire_username,
                login_address: &selected.login_address,
                domain_binding: &selected.binding,
                ca_pem: None,
                password: "fixture",
            })
            .await
            .unwrap();
        assert!(
            store
                .security_account_address(&account.account_id)
                .await
                .unwrap()
                .is_none()
        );
        // The request reaches the egress refusal using authoritative enrollment;
        // it neither needs a later sign-in audit row nor bypasses private-IP policy.
        let error = fetch(
            &store,
            "bridge-ews",
            Some(&account.account_id),
            request(endpoint),
        )
        .await
        .unwrap_err();
        assert!(error.contains("egress"), "{error}");
    }

    #[tokio::test]
    async fn a_previously_loaded_plugin_cannot_reuse_revoked_host_permissions() {
        let store = fixture().await;
        let mut row = store.get_plugin("bridge-ews").await.unwrap().unwrap();
        row.net_allowlist_json = "[]".into();
        store.put_plugin(&row).await.unwrap();
        assert!(
            fetch(&store, "bridge-ews", None, request("http://127.0.0.1:9/"))
                .await
                .unwrap_err()
                .contains("permission revoked")
        );
        row.enabled = false;
        store.put_plugin(&row).await.unwrap();
        assert!(
            fetch(&store, "bridge-ews", None, request("https://example.test/"))
                .await
                .unwrap_err()
                .contains("unavailable")
        );
    }
}
