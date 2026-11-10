//! JMAP Session discovery for clients connecting through Mailwoman.
use crate::AppState;
use axum::{
    Json,
    body::to_bytes,
    extract::State,
    http::{Extensions, HeaderMap, HeaderValue, StatusCode, header},
    response::{IntoResponse, Response},
};
use serde_json::{Value, json};

#[cfg(test)]
#[allow(dead_code)]
#[path = "../tests/common/mock_mail.rs"]
mod mock_mail;
#[cfg(test)]
#[allow(dead_code)]
#[path = "../../mw-store/src/test_db.rs"]
mod test_db;

fn origin(state: &AppState, headers: &HeaderMap, extensions: &Extensions) -> Option<String> {
    let base = state.external.base_url(headers, extensions)?;
    let url = reqwest::Url::parse(&base).ok()?;
    if !matches!(url.scheme(), "https" | "http")
        || url.host_str().is_none()
        || !url.username().is_empty()
        || url.password().is_some()
        || url.path() != "/"
        || url.query().is_some()
        || url.fragment().is_some()
    {
        return None;
    }
    Some(url.origin().ascii_serialization())
}

fn private(mut response: Response) -> Response {
    response
        .headers_mut()
        .insert(header::CACHE_CONTROL, HeaderValue::from_static("no-store"));
    response.headers_mut().insert(
        header::VARY,
        HeaderValue::from_static("Authorization, Cookie, X-Mailwoman-Link"),
    );
    if response.status() == StatusCode::UNAUTHORIZED {
        response.headers_mut().insert(
            header::WWW_AUTHENTICATE,
            HeaderValue::from_static("Bearer realm=\"Mailwoman\""),
        );
    }
    response
}

fn session_urls(document: &mut Value, base: &str) {
    // Never expose an upstream address: every advertised operation stays on our
    // authenticated proxy. A download URI includes all four RFC 8620 variables.
    document["apiUrl"] = format!("{base}/jmap/api").into();
    document["uploadUrl"] = format!("{base}/jmap/upload/{{accountId}}").into();
    document["downloadUrl"] =
        format!("{base}/jmap/download/{{accountId}}/{{blobId}}/{{name}}?type={{type}}").into();
    document["eventSourceUrl"] = format!("{base}/jmap/eventsource").into();
}

pub(crate) async fn session(
    State(state): State<AppState>,
    headers: HeaderMap,
    extensions: Extensions,
) -> Response {
    // Reuse the existing account, linked-mailbox, native bearer, password gate,
    // and restricted Sieve-key checks. Discovery grants no additional authority.
    let response =
        crate::jmap_session(State(state.clone()), headers.clone(), extensions.clone()).await;
    if !response.status().is_success() {
        return private(response);
    }
    let Some(origin) = origin(&state, &headers, &extensions) else {
        return private(StatusCode::BAD_REQUEST.into_response());
    };
    let bytes = match to_bytes(response.into_body(), 2 * 1024 * 1024).await {
        Ok(bytes) => bytes,
        Err(_) => return private(StatusCode::BAD_GATEWAY.into_response()),
    };
    let mut document: Value = match serde_json::from_slice(&bytes) {
        Ok(Value::Object(object)) => Value::Object(object),
        _ => return private(StatusCode::BAD_GATEWAY.into_response()),
    };
    session_urls(&mut document, &format!("{origin}{}", state.base_path));
    private(Json(document).into_response())
}

pub(crate) async fn connection(
    State(state): State<AppState>,
    headers: HeaderMap,
    extensions: Extensions,
) -> Response {
    let Some(origin) = origin(&state, &headers, &extensions) else {
        return StatusCode::BAD_REQUEST.into_response();
    };
    let base = format!("{origin}{}", state.base_path);
    let mut response = Json(json!({
        "version":1,
        "jmap":{"sessionUrl":format!("{base}/.well-known/jmap")},
        "authentication":{"native":{
            "scheme":"Bearer","loginUrl":format!("{base}/api/login"),
            "logoutUrl":format!("{base}/api/logout"),"clientType":"native",
            "twoFactorChallenges":true
        }}
    }))
    .into_response();
    response
        .headers_mut()
        .insert(header::CACHE_CONTROL, HeaderValue::from_static("no-store"));
    response
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn session_templates_are_absolute_and_stay_on_the_proxy() {
        let mut document = json!({"apiUrl":"https://upstream.invalid/jmap","capabilities":{"mail":{}},"accounts":{"a":{}}});
        session_urls(&mut document, "https://mail.example.org/mail");
        assert_eq!(document["apiUrl"], "https://mail.example.org/mail/jmap/api");
        assert_eq!(
            document["uploadUrl"],
            "https://mail.example.org/mail/jmap/upload/{accountId}"
        );
        assert_eq!(
            document["downloadUrl"],
            "https://mail.example.org/mail/jmap/download/{accountId}/{blobId}/{name}?type={type}"
        );
        assert_eq!(
            document["eventSourceUrl"],
            "https://mail.example.org/mail/jmap/eventsource"
        );
        assert!(document["capabilities"]["mail"].is_object());
        assert!(document["accounts"]["a"].is_object());
    }
    #[test]
    fn discovery_is_private_and_challenges_unauthenticated_clients() {
        let response = private(StatusCode::UNAUTHORIZED.into_response());
        assert_eq!(response.headers()[header::CACHE_CONTROL], "no-store");
        assert_eq!(
            response.headers()[header::WWW_AUTHENTICATE],
            "Bearer realm=\"Mailwoman\""
        );
        assert!(
            response.headers()[header::VARY]
                .to_str()
                .unwrap()
                .contains("X-Mailwoman-Link")
        );
    }

    #[tokio::test]
    async fn native_client_discovers_authenticates_and_calls_mail_over_http() {
        use crate::{AppConfig, SecurityConfig, ServerMode};
        use mw_store::{DomainConfigRow, ServerKey, Store};
        const KEY: &str = "00112233445566778899aabbccddeeff00112233445566778899aabbccddeeff";
        let mail = mock_mail::MockPop3::start("alice@example.org", "testpass").await;
        let db = test_db::unique_dir("jmap-discovery-http")
            .join("db.sqlite")
            .to_string_lossy()
            .into_owned();
        let store = Store::open(&db, ServerKey::from_hex(KEY).unwrap())
            .await
            .unwrap();
        store.put_domain_config(&DomainConfigRow {
            name: "example.org".into(),
            upstream_json: json!({"pop3":{"host":"127.0.0.1","port":mail.addr.port(),"tls":"none","allowPrivate":true}}).to_string(),
            allowlist_json:"[]".into(),blocklist_json:"[]".into(),enabled:true,preferred:false,settings_json:"{}".into(),updated_at:1,
        }).await.unwrap();
        let app = crate::build_app(AppConfig {
            db_path: db,
            server_key_hex: Some(KEY.into()),
            mode: ServerMode::Engine,
            cookie_secure: false,
            web_dir: None,
            hardening: Default::default(),
            security: SecurityConfig {
                upstream_default: Some(mail.url()),
                ..Default::default()
            },
        })
        .await
        .unwrap();
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let base = format!("http://{}", listener.local_addr().unwrap());
        let server = tokio::spawn(async move {
            axum::serve(listener, app).await.unwrap();
        });
        let client = reqwest::Client::builder()
            .no_proxy()
            .timeout(std::time::Duration::from_secs(30))
            .build()
            .unwrap();
        let manifest: Value = client
            .get(format!("{base}/.well-known/mailwoman"))
            .send()
            .await
            .unwrap()
            .error_for_status()
            .unwrap()
            .json()
            .await
            .unwrap();
        assert_eq!(
            manifest["jmap"]["sessionUrl"],
            format!("{base}/.well-known/jmap")
        );
        let discovery = manifest["jmap"]["sessionUrl"].as_str().unwrap();
        let anonymous = client.get(discovery).send().await.unwrap();
        assert_eq!(anonymous.status(), 401);
        assert_eq!(anonymous.headers()[header::CACHE_CONTROL], "no-store");
        assert!(anonymous.headers().contains_key(header::WWW_AUTHENTICATE));
        let login:Value=client.post(manifest["authentication"]["native"]["loginUrl"].as_str().unwrap()).header("origin","http://tauri.localhost")
            .json(&json!({"username":"alice@example.org","password":"testpass","clientType":"native"})).send().await.unwrap().error_for_status().unwrap().json().await.unwrap();
        let token = login["token"]
            .as_str()
            .expect("native login returns a bearer token");
        let settings: Value = client
            .get(format!("{base}/api/settings"))
            .bearer_auth(token)
            .send()
            .await
            .unwrap()
            .error_for_status()
            .unwrap()
            .json()
            .await
            .unwrap();
        assert_eq!(
            settings["settings"]["mail.conversations.allowed"]["value"],
            true
        );
        assert_eq!(
            settings["settings"]["mail.conversations.enabled"]["value"],
            false
        );
        assert_eq!(
            settings["settings"]["mail.cache.keepIndefinitely"]["value"],
            false
        );
        let forbidden = client
            .put(format!("{base}/api/settings/mail.conversations.allowed"))
            .bearer_auth(token)
            .header(
                "x-mailwoman-storage-authority",
                login["storageAuthority"].as_str().unwrap(),
            )
            .json(&json!({"value":false}))
            .send()
            .await
            .unwrap();
        assert_eq!(
            forbidden.status(),
            403,
            "users cannot change administrator availability"
        );
        let response = client
            .get(discovery)
            .bearer_auth(token)
            .send()
            .await
            .unwrap()
            .error_for_status()
            .unwrap();
        assert_eq!(response.headers()[header::CACHE_CONTROL], "no-store");
        let session: Value = response.json().await.unwrap();
        let account = session["primaryAccounts"]["urn:ietf:params:jmap:mail"]
            .as_str()
            .unwrap();
        assert!(session["accounts"][account].is_object());
        let download = session["downloadUrl"].as_str().unwrap();
        for variable in ["{accountId}", "{blobId}", "{name}", "{type}"] {
            assert!(download.contains(variable));
        }
        let answer:Value=client.post(session["apiUrl"].as_str().unwrap()).bearer_auth(token)
            .json(&json!({"using":["urn:ietf:params:jmap:core","urn:ietf:params:jmap:mail"],"methodCalls":[["Mailbox/get",{"accountId":account},"boxes"],["Email/query",{"accountId":account},"messages"]]}))
            .send().await.unwrap().error_for_status().unwrap().json().await.unwrap();
        assert_eq!(answer["methodResponses"][0][0], "Mailbox/get");
        assert_eq!(answer["methodResponses"][1][0], "Email/query");
        let invalid = client
            .get(discovery)
            .bearer_auth("invalid-token")
            .send()
            .await
            .unwrap();
        assert_eq!(invalid.status(), 401);
        server.abort();
    }
}
