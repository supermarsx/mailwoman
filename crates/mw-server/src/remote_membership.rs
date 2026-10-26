//! Bounded server-derived references for deliberate message resource downloads.
use super::{AppState, fetch_semaphore, forbidden, internal};
use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
};
use serde_json::{Value, json};
use std::collections::{BTreeMap, BTreeSet};

const MAX_HTML_BYTES: usize = 1024 * 1024;
const MAX_JSON_BYTES: usize = 6 * MAX_HTML_BYTES + 256 * 1024;
const MAX_REFERENCES: usize = 4096;
#[derive(Default)]
pub(super) struct References {
    pub images: BTreeMap<String, bool>,
    pub links: BTreeSet<String>,
}
pub(super) fn normalized_url(value: &str) -> Option<String> {
    if value.len() > super::MAX_URL_LEN {
        return None;
    }
    let mut url = reqwest::Url::parse(value).ok()?;
    if !matches!(url.scheme(), "http" | "https")
        || !url.username().is_empty()
        || url.password().is_some()
    {
        return None;
    }
    url.set_fragment(None);
    Some(url.to_string())
}
fn too_large() -> Response {
    (
        StatusCode::PAYLOAD_TOO_LARGE,
        "message reference inspection limit exceeded",
    )
        .into_response()
}
fn html_values(response: &Value, account: &str, email: &str) -> Result<Vec<String>, Response> {
    let result = response["methodResponses"]
        .as_array()
        .and_then(|calls| {
            calls
                .iter()
                .find(|call| call[0] == "Email/get" && call[2] == "download-refs")
        })
        .ok_or_else(|| forbidden("message body unavailable"))?;
    if result[1]["accountId"] != account {
        return Err(forbidden("message account mismatch"));
    }
    let message = result[1]["list"]
        .as_array()
        .and_then(|list| list.iter().find(|value| value["id"] == email))
        .ok_or_else(|| forbidden("no such message"))?;
    let parts = message["htmlBody"]
        .as_array()
        .ok_or_else(|| forbidden("message HTML body unavailable"))?;
    if parts.len() > 64 {
        return Err(too_large());
    }
    let mut total = 0usize;
    let mut bodies = Vec::new();
    for part in parts {
        let id = part["partId"]
            .as_str()
            .filter(|id| id.len() <= 512)
            .ok_or_else(|| forbidden("invalid HTML part"))?;
        let body = &message["bodyValues"][id];
        if body["isTruncated"] == true || body["isEncodingProblem"] == true {
            return Err(forbidden("incomplete HTML body"));
        }
        let value = body["value"]
            .as_str()
            .ok_or_else(|| forbidden("HTML body unavailable"))?;
        total = total.checked_add(value.len()).ok_or_else(too_large)?;
        if total > MAX_HTML_BYTES {
            return Err(too_large());
        }
        bodies.push(value.to_owned());
    }
    Ok(bodies)
}
fn collect(bodies: Vec<String>, clean_links: bool) -> Result<References, Response> {
    let mut result = References::default();
    for body in bodies {
        let (_, refs) = mw_sanitize::sanitize_email_html_and_references(
            &body,
            &mw_sanitize::SanitizeOptions {
                remote_images: mw_sanitize::RemoteImages::Placeholder,
                block_trackers: false,
                clean_links,
            },
        );
        for image in refs.images {
            if let Some(url) = normalized_url(&image.url) {
                result
                    .images
                    .entry(url)
                    .and_modify(|tracker| *tracker |= image.tracker)
                    .or_insert(image.tracker);
            }
        }
        for link in refs.links {
            if let Some(url) = normalized_url(&link) {
                result.links.insert(url);
            }
        }
        if result.images.len() + result.links.len() > MAX_REFERENCES {
            return Err(too_large());
        }
    }
    Ok(result)
}
pub(super) async fn owned(
    state: &AppState,
    session: &mw_store::Session,
    email: &str,
    clean_links: bool,
) -> Result<References, Response> {
    let _permit = fetch_semaphore()
        .try_acquire()
        .map_err(|_| (StatusCode::SERVICE_UNAVAILABLE, "remote content busy").into_response())?;
    let bodies = if let Some(engine) = &state.engine {
        engine
            .message_html_limited(
                &session.account_id,
                email,
                super::MAX_PART_RAW_BYTES,
                MAX_HTML_BYTES,
            )
            .await
            .map_err(|_| forbidden("message body unavailable or too large"))?
            .ok_or_else(|| forbidden("no owned stored message body"))?
    } else {
        let request = json!({"using":["urn:ietf:params:jmap:core","urn:ietf:params:jmap:mail"],"methodCalls":[["Email/get",{"accountId":session.account_id,"ids":[email],"properties":["id","htmlBody","bodyValues"],"bodyProperties":["partId","type"],"fetchHTMLBodyValues":true,"maxBodyValueBytes":MAX_HTML_BYTES},"download-refs"]]});
        let client = crate::upstream_client(
            &state.security,
            &session.jmap_url,
            &session.api_url,
            &session.credentials,
        )
        .await
        .map_err(|_| internal("message reference upstream"))?;
        let (status, bytes) = tokio::time::timeout(
            std::time::Duration::from_secs(10),
            client.request_raw_limited(
                &session.api_url,
                axum::body::Bytes::from(request.to_string()),
                MAX_JSON_BYTES,
            ),
        )
        .await
        .map_err(|_| {
            (
                StatusCode::GATEWAY_TIMEOUT,
                "message reference lookup timed out",
            )
                .into_response()
        })?
        .map_err(|_| internal("message reference lookup"))?;
        if !status.is_success() {
            return Err(forbidden("message body unavailable"));
        }
        let account = session.account_id.clone();
        let email = email.to_owned();
        tokio::task::spawn_blocking(move || {
            let parsed: Value = serde_json::from_slice(&bytes)
                .map_err(|_| internal("invalid message reference response"))?;
            html_values(&parsed, &account, &email)
        })
        .await
        .map_err(|_| internal("message reference task"))??
    };
    tokio::task::spawn_blocking(move || collect(bodies, clean_links))
        .await
        .map_err(|_| internal("message reference parse task"))?
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn owned_body_projection_refuses_foreign_truncated_and_oversized_values() {
        let mut response = json!({"methodResponses":[["Email/get",{"accountId":"a","list":[{"id":"m","htmlBody":[{"partId":"1"}],"bodyValues":{"1":{"value":"<img src=\"https://image.example/a.png\">","isTruncated":false}}}]},"download-refs"]]});
        assert!(html_values(&response, "other", "m").is_err());
        assert!(html_values(&response, "a", "other").is_err());
        assert_eq!(html_values(&response, "a", "m").unwrap().len(), 1);
        response["methodResponses"][0][1]["list"][0]["bodyValues"]["1"]["isTruncated"] =
            json!(true);
        assert!(html_values(&response, "a", "m").is_err());
        response["methodResponses"][0][1]["list"][0]["bodyValues"]["1"] =
            json!({"value":"x".repeat(MAX_HTML_BYTES+1)});
        assert!(html_values(&response, "a", "m").is_err());
    }
    #[test]
    fn references_are_exact_cleaned_and_conservatively_mark_trackers() {
        let refs = collect(vec!["<img src='https://image.example/a.png'><img src='https://image.example/a.png' width=1 height=1><a href='https://file.example/a.pdf?utm_source=x&amp;id=1'>file</a>".into()],true).unwrap();
        assert_eq!(refs.images.get("https://image.example/a.png"), Some(&true));
        assert!(!refs.images.contains_key("https://image.example/other.png"));
        assert!(refs.links.contains("https://file.example/a.pdf?id=1"));
        assert!(normalized_url("https://user:secret@image.example/a.png").is_none());
    }
}
