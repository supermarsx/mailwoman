//! LDAP administration: redacted configuration, atomic writes and guarded startup wiring.
use crate::{
    AppState,
    admin_ctx::{AdminApiError, AdminCtx, AdminJson, check_if_match, etag_of, with_etag},
    admin_router::AdminRegistrar,
    capabilities::Capability,
};
use axum::{
    Json,
    extract::State,
    http::{HeaderMap, Method, StatusCode},
    response::Response,
};
use mw_directory::{
    AttrMap, Directory, DirectoryConfig, DirectoryConnector, DirectoryError, LdapEndpoint, LdapTls,
};
use mw_store::{
    DirectoryAdminRow, DirectoryAdminWrite, DirectoryConfigRow, DirectoryPasswordChange, Store,
};
use serde::Deserialize;
use serde_json::{Value, json};
use std::{collections::HashSet, sync::Arc, time::Duration};

pub(crate) fn register(r: &mut impl AdminRegistrar) {
    r.admin_endpoint(
        Method::GET,
        "/integrations/directory",
        Capability::IntegrationsRead,
        get,
    );
    r.admin_endpoint(
        Method::PUT,
        "/integrations/directory",
        Capability::IntegrationsWrite,
        put,
    );
}
fn env_bind() -> Option<String> {
    std::env::var("MW_DIRECTORY_BIND_DN")
        .ok()
        .filter(|s| !s.is_empty())
}
fn env_password() -> Option<String> {
    std::env::var("MW_DIRECTORY_BIND_PW")
        .ok()
        .filter(|s| !s.is_empty())
}
fn redacted_url(raw: &str) -> String {
    let Ok(mut url) = reqwest::Url::parse(raw) else {
        return "<invalid URL>".into();
    };
    let _ = url.set_username("");
    let _ = url.set_password(None);
    url.set_query(None);
    url.set_fragment(None);
    url.to_string()
}
fn rows_json(rows: &[DirectoryAdminRow]) -> Value {
    Value::Array(rows.iter().map(|r| json!({
        "id":r.config.id,"url":redacted_url(&r.config.url),"baseDn":r.config.base_dn,"bindDn":r.config.bind_dn,
        "tls":r.config.tls,"priority":r.config.priority,"attrMap":serde_json::from_str::<Value>(&r.config.attr_map_json).unwrap_or(json!({})),
        "enabled":r.config.enabled,"connectionMode":r.connection_mode,"allowPrivate":r.allow_private,
        "password":{"set":r.password_set},"revision":r.revision,
    })).collect())
}
pub(crate) fn revision(rows: &[DirectoryAdminRow]) -> String {
    etag_of(&json!(format!("{rows:?}")))
}
async fn get(State(state): State<AppState>, ctx: AdminCtx) -> Result<Response, AdminApiError> {
    ctx.require_unscoped(Capability::IntegrationsRead)?;
    let rows = state
        .store
        .directory_admin_rows()
        .await
        .map_err(|_| AdminApiError::internal("directory operation failed"))?;
    let etag = revision(&rows);
    let active = state.directory.configuration_revision();
    let proxy = state
        .store
        .active_egress_proxy()
        .await
        .map_err(|_| AdminApiError::internal("directory operation failed"))?
        .is_some();
    let body = json!({"entries":rows_json(&rows),"etag":etag,"activeRevision":active,
        "restartRequired":active != Some(etag.as_str()),"reload":"restart",
        "transportStatus":if proxy {"proxyUnsupported"} else {"directExplicitOnly"},
        "environment":{"bindDn":env_bind(),"passwordSet":env_password().is_some(),"pinned":env_bind().is_some() && env_password().is_some()}});
    Ok(with_etag(Json(body), &etag))
}
#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct Input {
    id: String,
    url: String,
    base_dn: String,
    bind_dn: Option<String>,
    tls: String,
    priority: i32,
    attr_map: AttrMap,
    enabled: bool,
    connection_mode: String,
    allow_private: bool,
    #[serde(default)]
    password: Password,
}
#[derive(Default, Deserialize)]
#[serde(tag = "action", rename_all = "lowercase", deny_unknown_fields)]
enum Password {
    #[default]
    Keep,
    Set {
        value: String,
    },
    Clear,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Update {
    entries: Vec<Input>,
}
fn invalid(detail: &str, field: &str) -> AdminApiError {
    AdminApiError::invalid("invalidDirectory", detail, field)
}
fn validate_url(value: &str, tls: &str) -> Result<reqwest::Url, AdminApiError> {
    let u = reqwest::Url::parse(value)
        .map_err(|_| invalid("An absolute LDAP URL is required", "/url"))?;
    if !matches!(
        (u.scheme(), tls),
        ("ldap", "none" | "starttls") | ("ldaps", "ldaps")
    ) || u.host_str().is_none()
        || !u.username().is_empty()
        || u.password().is_some()
        || u.query().is_some()
        || u.fragment().is_some()
        || !matches!(u.path(), "" | "/")
    {
        return Err(invalid(
            "Use ldap:// with none/starttls or ldaps:// with ldaps; credentials, query and path are not allowed",
            "/url",
        ));
    }
    Ok(u)
}
fn writes(
    inputs: Vec<Input>,
    old: &[DirectoryAdminRow],
    pinned_dn: Option<&str>,
) -> Result<Vec<DirectoryAdminWrite>, AdminApiError> {
    if inputs.len() > 32 {
        return Err(invalid("At most 32 directories are supported", "/entries"));
    }
    let mut ids = HashSet::new();
    let mut urls = HashSet::new();
    let mut out = Vec::new();
    for input in inputs {
        if input.id.is_empty()
            || input.id.len() > 64
            || !input
                .id
                .bytes()
                .all(|b| b.is_ascii_alphanumeric() || b"-_".contains(&b))
            || !ids.insert(input.id.clone())
        {
            return Err(invalid(
                "Directory IDs must be unique ASCII identifiers",
                "/entries/id",
            ));
        }
        let url = validate_url(&input.url, &input.tls)?.to_string();
        if !urls.insert(url.clone()) {
            return Err(invalid("Each endpoint URL must be unique", "/entries/url"));
        }
        if !matches!(input.connection_mode.as_str(), "direct" | "blocked") {
            return Err(invalid(
                "Choose explicit direct access or blocked",
                "/connectionMode",
            ));
        }
        if input.base_dn.len() > 4096
            || input.base_dn.contains('\0')
            || input
                .bind_dn
                .as_ref()
                .is_some_and(|s| s.len() > 4096 || s.contains('\0'))
        {
            return Err(invalid("Distinguished name is invalid", "/baseDn"));
        }
        for attr in [
            &input.attr_map.display_name,
            &input.attr_map.mail,
            &input.attr_map.member,
            &input.attr_map.user_cert,
            &input.attr_map.photo,
        ]
        .into_iter()
        .flatten()
        {
            if attr.is_empty()
                || attr.len() > 128
                || !attr
                    .bytes()
                    .all(|b| b.is_ascii_alphanumeric() || b";-".contains(&b))
            {
                return Err(invalid("Invalid LDAP attribute name", "/attrMap"));
            }
        }
        let previous = old.iter().find(|r| r.config.id == input.id);
        let pinned = input
            .bind_dn
            .as_deref()
            .is_some_and(|dn| Some(dn) == pinned_dn)
            || previous.is_some_and(|r| {
                r.config
                    .bind_dn
                    .as_deref()
                    .is_some_and(|dn| Some(dn) == pinned_dn)
            });
        if pinned
            && (!matches!(input.password, Password::Keep)
                || !previous.is_some_and(|r| {
                    r.config.url.trim_end_matches('/') == url.trim_end_matches('/')
                        && r.config.bind_dn == input.bind_dn
                }))
        {
            return Err(invalid(
                "This endpoint credential is pinned by the environment; change the environment and restart to retarget it",
                "/password",
            ));
        }
        let retaining =
            matches!(input.password, Password::Keep) && previous.is_some_and(|r| r.password_set);
        if retaining
            && previous.is_some_and(|r| {
                r.config.url.trim_end_matches('/') != url.trim_end_matches('/')
                    || r.config.bind_dn != input.bind_dn
            })
        {
            return Err(invalid(
                "Replace or clear the saved credential when changing its server or bind DN",
                "/password",
            ));
        }
        let has_password = retaining || pinned || matches!(input.password, Password::Set { .. });
        if has_password
            && (input.tls == "none" || input.bind_dn.as_ref().is_none_or(|s| s.is_empty()))
        {
            return Err(invalid(
                "Service credentials require TLS and a bind DN",
                "/tls",
            ));
        }
        let password = match input.password {
            Password::Keep => DirectoryPasswordChange::Keep,
            Password::Clear => DirectoryPasswordChange::Clear,
            Password::Set { value } => {
                if value.is_empty() || value.len() > 4096 {
                    return Err(invalid(
                        "Password must contain 1 to 4096 bytes",
                        "/password",
                    ));
                }
                DirectoryPasswordChange::Set(value)
            }
        };
        out.push(DirectoryAdminWrite {
            row: DirectoryAdminRow {
                config: DirectoryConfigRow {
                    id: input.id,
                    url,
                    base_dn: input.base_dn,
                    bind_dn: input.bind_dn,
                    tls: input.tls,
                    priority: i64::from(input.priority),
                    attr_map_json: serde_json::to_string(&input.attr_map)
                        .map_err(|_| AdminApiError::internal("directory operation failed"))?,
                    enabled: input.enabled,
                },
                connection_mode: input.connection_mode,
                allow_private: input.allow_private,
                password_set: has_password,
                revision: String::new(),
            },
            password,
        });
    }
    Ok(out)
}
async fn put(
    State(state): State<AppState>,
    ctx: AdminCtx,
    headers: HeaderMap,
    AdminJson(body): AdminJson<Update>,
) -> Result<Response, AdminApiError> {
    ctx.require_unscoped(Capability::IntegrationsWrite)?;
    let old = state
        .store
        .directory_admin_rows()
        .await
        .map_err(|_| AdminApiError::internal("directory operation failed"))?;
    if headers
        .get(axum::http::header::IF_MATCH)
        .is_none_or(|h| h.to_str().ok() == Some("*"))
    {
        return Err(AdminApiError::new(
            StatusCode::PRECONDITION_REQUIRED,
            "ifMatchRequired",
            "Reload the configuration and send its exact ETag",
        ));
    }
    check_if_match(&headers, &revision(&old), None, None)?;
    let pin = env_password().and_then(|_| env_bind());
    let writes = writes(body.entries, &old, pin.as_deref())?;
    if !state
        .store
        .replace_directory_admin(&old, &writes)
        .await
        .map_err(|_| AdminApiError::internal("directory operation failed"))?
    {
        return Err(AdminApiError::new(
            StatusCode::PRECONDITION_FAILED,
            "stale",
            "Directory configuration changed; reload before saving",
        ));
    }
    let rows = state
        .store
        .directory_admin_rows()
        .await
        .map_err(|_| AdminApiError::internal("directory operation failed"))?;
    crate::admin_accounts::audit(
        &state,
        &ctx,
        "directory.configure",
        "integration:directory",
        Some(rows_json(&old)),
        Some(rows_json(&rows)),
    )
    .await;
    let etag = revision(&rows);
    Ok(with_etag(
        Json(
            json!({"entries":rows_json(&rows),"etag":etag,"restartRequired":state.directory.configuration_revision()!=Some(etag.as_str()),"reload":"restart"}),
        ),
        &etag,
    ))
}

struct Connector {
    store: Store,
    url: String,
    allow_private: bool,
    direct: bool,
}
#[async_trait::async_trait]
impl DirectoryConnector for Connector {
    async fn connect(&self, url: &str) -> mw_directory::Result<std::net::TcpStream> {
        let refused =
            || DirectoryError::Transport("LDAP outbound policy refused the connection".into());
        if url != self.url
            || !self.direct
            || self
                .store
                .active_egress_proxy()
                .await
                .map_err(|_| refused())?
                .is_some()
        {
            return Err(refused());
        }
        let u = reqwest::Url::parse(url).map_err(|_| refused())?;
        let host = u.host_str().ok_or_else(refused)?.trim_matches(['[', ']']);
        let port = u
            .port()
            .unwrap_or(if u.scheme() == "ldaps" { 636 } else { 389 });
        tokio::time::timeout(Duration::from_secs(10), async {
            let addresses: Vec<_> = tokio::net::lookup_host((host, port))
                .await
                .map_err(|_| refused())?
                .collect();
            if addresses.is_empty()
                || !addresses
                    .iter()
                    .all(|a| mw_egress::operator_mail_ip_allowed(&a.ip(), self.allow_private))
            {
                return Err(refused());
            }
            for address in addresses {
                if let Ok(stream) = tokio::net::TcpStream::connect(address).await {
                    return stream.into_std().map_err(|_| refused());
                }
            }
            Err(refused())
        })
        .await
        .map_err(|_| refused())?
    }
}
/// Startup snapshot; changes require restart and are compared with this revision by GET.
pub(crate) async fn build_directory(store: &Store) -> Arc<Directory> {
    let rows = store.directory_admin_rows().await.unwrap_or_default();
    let blocked = rows
        .iter()
        .filter(|r| r.config.enabled && r.connection_mode != "direct")
        .count();
    if blocked > 0 {
        tracing::warn!(
            count = blocked,
            "LDAP endpoints are blocked pending explicit direct-access consent in Admin Integrations; restart after saving"
        );
    }
    let endpoints = rows
        .iter()
        .filter(|r| r.config.enabled)
        .filter_map(|r| {
            validate_url(&r.config.url, &r.config.tls).ok()?;
            if rows
                .iter()
                .filter(|other| other.config.url == r.config.url)
                .count()
                != 1
            {
                return None;
            }
            Some(LdapEndpoint {
                url: r.config.url.clone(),
                base_dn: r.config.base_dn.clone(),
                bind_dn: r.config.bind_dn.clone(),
                tls: match r.config.tls.as_str() {
                    "ldaps" => LdapTls::Ldaps,
                    "starttls" => LdapTls::StartTls,
                    _ => LdapTls::None,
                },
                priority: i32::try_from(r.config.priority).ok()?,
                attr_map: serde_json::from_str(&r.config.attr_map_json).ok()?,
            })
        })
        .collect();
    let mut dir =
        Directory::new(DirectoryConfig { endpoints }).with_configuration_revision(revision(&rows));
    for row in &rows {
        let cfg = &row.config;
        let mut direct = row.connection_mode == "direct";
        if let Some(dn) = &cfg.bind_dn {
            let password = if env_bind().as_deref() == Some(dn.as_str()) {
                env_password()
            } else {
                None
            };
            let password = match password {
                Some(p) => Some(p),
                None => match store.directory_bind_password(&cfg.id).await {
                    Ok(p) => p,
                    Err(_) => {
                        direct = false;
                        None
                    }
                },
            };
            if let Some(password) = password {
                // Never send service credentials on an inherited plaintext configuration.
                if cfg.tls != "none" {
                    dir = dir.with_endpoint_password(&cfg.url, dn, password);
                } else {
                    direct = false;
                }
            }
        }
        dir = dir.with_connector(
            &cfg.url,
            Arc::new(Connector {
                store: store.clone(),
                url: cfg.url.clone(),
                allow_private: row.allow_private,
                direct,
            }),
        );
    }
    Arc::new(dir)
}

#[cfg(test)]
mod tests {
    use super::*;
    fn input() -> Input {
        serde_json::from_value(json!({"id":"primary","url":"ldaps://directory.example","baseDn":"dc=example","bindDn":"cn=service","tls":"ldaps","priority":0,"attrMap":{},"enabled":true,"connectionMode":"direct","allowPrivate":false})).unwrap()
    }
    #[test]
    fn rejects_url_tls_mismatch_and_embedded_secrets() {
        for (url, tls) in [
            ("ldap://directory.example", "ldaps"),
            ("ldaps://directory.example", "none"),
            ("ldaps://user:secret@directory.example", "ldaps"),
            ("ldaps://directory.example/dc=example", "ldaps"),
            ("http://directory.example", "none"),
        ] {
            assert!(validate_url(url, tls).is_err(), "{url}");
        }
        assert!(validate_url("ldap://directory.example:389", "starttls").is_ok());
    }
    #[test]
    fn secrets_require_tls_and_retarget_requires_replacement() {
        let mut i = input();
        i.password = Password::Set {
            value: "secret".into(),
        };
        let old = writes(vec![i], &[], None).unwrap().remove(0).row;
        let mut changed = input();
        changed.url = "ldaps://other.example".into();
        assert!(writes(vec![changed], std::slice::from_ref(&old), None).is_err());
        let mut plaintext = input();
        plaintext.url = "ldap://directory.example".into();
        plaintext.tls = "none".into();
        plaintext.password = Password::Set {
            value: "secret".into(),
        };
        assert!(writes(vec![plaintext], &[], None).is_err());
        assert!(
            writes(vec![input()], &[], Some("cn=service")).is_err(),
            "new endpoint cannot receive pinned environment credential"
        );
        let mut attr = input();
        attr.attr_map.mail = Some("mail)(uid=*".into());
        assert!(writes(vec![attr], &[], None).is_err());
    }
    #[tokio::test]
    async fn guarded_connector_refuses_private_without_consent_and_pins_target() {
        let store = Store::open_in_memory(mw_store::ServerKey::generate())
            .await
            .unwrap();
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let url = format!("ldap://{}/", listener.local_addr().unwrap());
        let mut connector = Connector {
            store,
            url: url.clone(),
            allow_private: false,
            direct: true,
        };
        assert!(connector.connect(&url).await.is_err());
        connector.allow_private = true;
        assert!(connector.connect("ldap://127.0.0.1:1").await.is_err());
        let stream = connector.connect(&url).await.unwrap();
        drop(stream);
        assert!(
            tokio::time::timeout(Duration::from_secs(1), listener.accept())
                .await
                .is_ok()
        );
        connector.direct = false;
        assert!(connector.connect(&url).await.is_err());
    }
    #[tokio::test]
    async fn legacy_rows_require_consent_and_live_tls_directory_uses_sealed_password() {
        use mw_directory::DirectorySource;
        let store = Store::open_in_memory(mw_store::ServerKey::generate())
            .await
            .unwrap();
        let row = DirectoryConfigRow {
            id: "legacy".into(),
            url: "ldaps://localhost:42412/".into(),
            base_dn: "dc=example,dc=com".into(),
            bind_dn: Some("cn=admin,dc=example,dc=com".into()),
            tls: "ldaps".into(),
            priority: 0,
            attr_map_json: "{}".into(),
            enabled: true,
        };
        store.put_directory_config(&row).await.unwrap();
        let old = store.directory_admin_rows().await.unwrap();
        assert_eq!(old[0].connection_mode, "blocked");
        assert!(!old[0].allow_private);
        let blocked = build_directory(&store).await;
        assert!(blocked.search_gal("alice", 0).await.is_err());
        let Ok(url) = std::env::var("MW_DIRECTORY_TEST_URL") else {
            return;
        };
        let mut updated = old[0].clone();
        updated.config.url = url;
        updated.connection_mode = "direct".into();
        updated.allow_private = true;
        let password = std::env::var("MW_DIRECTORY_TEST_PASSWORD").expect("fixture password");
        assert!(
            store
                .replace_directory_admin(
                    &old,
                    &[DirectoryAdminWrite {
                        row: updated,
                        password: DirectoryPasswordChange::Set(password)
                    }]
                )
                .await
                .unwrap()
        );
        let live = build_directory(&store).await;
        let entries = live
            .search_gal("alice", 0)
            .await
            .expect("real LDAP TLS service bind/search");
        assert!(
            entries.iter().any(|e| e.mail.contains("alice")),
            "{entries:?}"
        );
        assert_ne!(
            live.configuration_revision(),
            blocked.configuration_revision()
        );
        let current = store.directory_admin_rows().await.unwrap();
        let mut wrong_name = current[0].clone();
        wrong_name.config.url = wrong_name.config.url.replace("localhost", "127.0.0.1");
        store
            .replace_directory_admin(
                &current,
                &[DirectoryAdminWrite {
                    row: wrong_name,
                    password: DirectoryPasswordChange::Keep,
                }],
            )
            .await
            .unwrap();
        assert!(
            build_directory(&store)
                .await
                .search_gal("alice", 0)
                .await
                .is_err(),
            "pinned socket must still verify the original TLS hostname"
        );
        // An active proxy blocks a previously built directory immediately.
        let proxy = mw_store::EgressProxyRow {
            id: "directory-test".into(),
            kind: "http".into(),
            host: "127.0.0.1".into(),
            port: 9,
            username: String::new(),
            password: None,
            allow_plaintext: false,
            active: true,
            tls_to_proxy: false,
            remote_dns: false,
            remote_dns_risk_accepted_by: None,
            remote_dns_risk_accepted_at: None,
            tunnel_id: None,
            chain_next_id: None,
            scope: "global".into(),
            traffic_classes: vec![],
            created_at: String::new(),
            updated_at: String::new(),
        };
        store.put_egress_proxy(&proxy).await.unwrap();
        assert!(
            store
                .set_active_egress_proxy(Some(&proxy.id))
                .await
                .unwrap()
        );
        assert!(
            live.search_gal("alice", 0).await.is_err(),
            "a running directory must not bypass a newly active proxy"
        );
    }
}
