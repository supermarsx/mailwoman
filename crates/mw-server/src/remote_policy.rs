//! Effective reader privacy controls, resolved from the shared settings policy.
use crate::{AppState, settings_registry};
use axum::{
    Json,
    extract::{Query, State},
    http::HeaderMap,
    response::{IntoResponse, Response},
};
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct PrivacyPolicy {
    pub mode: String,
    pub trackers: bool,
    pub clean_links: bool,
    pub allow_direct: bool,
    pub inline_parts: bool,
    pub remote_image_downloads: bool,
    pub locked: Locks,
    #[serde(skip)]
    pub max_bytes: usize,
    #[serde(skip)]
    pub cache_bytes: usize,
    #[serde(skip)]
    pub rate_per_min: usize,
}
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct Locks {
    pub mode: bool,
    pub trackers: bool,
    pub clean_links: bool,
}

fn unavailable() -> Response {
    (
        axum::http::StatusCode::INTERNAL_SERVER_ERROR,
        "privacy policy unavailable",
    )
        .into_response()
}

/// Mail aliases inherit the canonical domain route; an existing bare username
/// never acquires the preferred domain's route.
pub(crate) async fn canonical_domain(
    state: &AppState,
    session: &mw_store::Session,
) -> Result<Option<String>, Response> {
    match crate::upstream_resolve::domain_of(&session.username) {
        Some(domain) => state
            .store
            .canonical_policy_domain(&domain)
            .await
            .map(Some)
            .map_err(|_| unavailable()),
        None => Ok(None),
    }
}

pub(crate) async fn effective(
    state: &AppState,
    session: &mw_store::Session,
) -> Result<PrivacyPolicy, Response> {
    let identity = crate::settings_routes::UserSubject::resolved(&state.store, session)
        .await
        .map_err(|_| unavailable())?;
    let subject = identity.as_subject();
    let mut values = std::collections::BTreeMap::new();
    for key in [
        "privacy.remoteContent",
        "privacy.remoteImageDownloads",
        "privacy.trackers",
        "privacy.cleanLinks",
        "privacy.allowDirect",
        "cap.remote_content.direct",
        "privacy.maxBytes",
        "privacy.cacheBytes",
        "privacy.ratePerMin",
    ] {
        let value = settings_registry::effective_for(&state.store, key, &subject)
            .await
            .map_err(|_| {
                (
                    axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                    "privacy policy unavailable",
                )
                    .into_response()
            })?;
        if let Some(value) = value {
            values.insert(key, value);
        }
    }
    let boolean = |key| {
        values
            .get(key)
            .and_then(|v| v.value.as_bool())
            .unwrap_or(false)
    };
    let locked = |key| values.get(key).is_none_or(|v| v.locked());
    let number = |key, fallback| {
        values
            .get(key)
            .and_then(|v| v.value.as_u64())
            .map(|n| n as usize)
            .unwrap_or(fallback)
    };
    let allow_direct = boolean("privacy.allowDirect") && boolean("cap.remote_content.direct");
    let mut mode = values
        .get("privacy.remoteContent")
        .and_then(|v| Value::as_str(&v.value))
        .unwrap_or("ask")
        .to_owned();
    if mode == "direct" && !allow_direct {
        mode = "ask".into();
    }
    Ok(PrivacyPolicy {
        mode,
        trackers: boolean("privacy.trackers"),
        clean_links: boolean("privacy.cleanLinks"),
        allow_direct,
        inline_parts: state.engine.is_some(),
        remote_image_downloads: boolean("privacy.remoteImageDownloads"),
        locked: Locks {
            mode: locked("privacy.remoteContent"),
            trackers: locked("privacy.trackers"),
            clean_links: locked("privacy.cleanLinks"),
        },
        max_bytes: number("privacy.maxBytes", 8 * 1024 * 1024),
        cache_bytes: number("privacy.cacheBytes", crate::image_proxy::MAX_CACHE_BYTES),
        rate_per_min: number("privacy.ratePerMin", 120),
    })
}
pub(crate) async fn get(State(state): State<AppState>, headers: HeaderMap) -> Response {
    let session = match crate::authed(&state, &headers).await {
        Ok(s) => s,
        Err(r) => return r,
    };
    match effective(&state, &session).await {
        Ok(policy) => (
            [(axum::http::header::CACHE_CONTROL, "no-store")],
            Json(policy),
        )
            .into_response(),
        Err(r) => r,
    }
}

/// Static, public wrapper: message HTML is delivered only through browser
/// postMessage and never enters this endpoint or a server log.
#[derive(Clone)]
pub(crate) struct ReaderFrameResponse { pub compatible: bool }

#[derive(Default, Deserialize)]
pub(crate) struct ReaderFrameQuery { coep: Option<String> }

pub(crate) async fn frame(Query(query): Query<ReaderFrameQuery>) -> Response {
    use base64::Engine as _;
    use sha2::Digest;
    let script = include_str!("remote-frame.js").replace("\r\n", "\n").replace('\r', "\n");
    let hash = base64::engine::general_purpose::STANDARD.encode(sha2::Sha256::digest(script.as_bytes()));
    let compatible = query.coep.as_deref() == Some("require-corp");
    // Unlike credentialless, require-corp does not omit credentials on direct
    // third-party images. Keep that fallback limited to our image proxy/parts
    // and embedded data, rather than silently sending third-party cookies.
    let images = if compatible { "'self' data: blob:" } else { "'self' https: data: blob:" };
    let csp = format!(
        "default-src 'none'; script-src 'sha256-{hash}'; style-src 'unsafe-inline'; frame-src 'self'; img-src {images}; frame-ancestors 'self'; base-uri 'none'; form-action 'none'"
    );
    let html = format!(
        "<!doctype html><html><head><meta charset=utf-8><meta name=referrer content=no-referrer><style>html,body{{margin:0;width:100%;height:100%;overflow:hidden}}</style></head><body><script>{script}</script></body></html>"
    );
    let mut response = (
        [
            (
                axum::http::header::CONTENT_TYPE,
                "text/html; charset=utf-8".to_string(),
            ),
            (axum::http::header::CONTENT_SECURITY_POLICY, csp),
            (
                axum::http::header::CACHE_CONTROL,
                "public, max-age=300".to_string(),
            ),
            (
                axum::http::header::X_CONTENT_TYPE_OPTIONS,
                "nosniff".to_string(),
            ),
            (
                axum::http::header::REFERRER_POLICY,
                "no-referrer".to_string(),
            ),
        ],
        html,
    )
        .into_response();
    response.extensions_mut().insert(ReaderFrameResponse { compatible });
    response
}
