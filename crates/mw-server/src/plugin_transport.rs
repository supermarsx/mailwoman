//! Host-bound plugin requests through the current administrator egress policy.
use mw_plugin::{HttpReq, HttpResp};
use mw_store::Store;

pub(super) async fn fetch(
    store: &Store,
    plugin: &str,
    account: Option<&str>,
    req: HttpReq,
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
            let address = store
                .security_account_address(account)
                .await
                .map_err(|_| "account identity unavailable")?
                .ok_or("account identity unavailable")?;
            let canonical = store
                .canonical_policy_address(&address)
                .await
                .map_err(|_| "account identity unavailable")?;
            if plugin == "bridge-ews" {
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
        let metadata = mw_egress::ResponseMetadata::default();
        let body = req.body.as_deref().map(|bytes| mw_egress::OutBody {
            content_type: "application/octet-stream",
            bytes,
        });
        let mut request = mw_egress::plan::Request::plugin(
            mw_egress::TrafficClass::Plugins,
            mw_egress::PluginHttp {
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
        let fetched = mw_egress::plan::send_with(&plan, url, &request).await;
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
