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
    /// Timeouts, redirect cap and per-message/per-account concurrency (t56).
    #[serde(skip)]
    pub fetch: crate::image_proxy::remote_content::FetchLimits,
}

/// The t56 fetch keys, read in one pass (`effective_all`) rather than one query
/// set per key: they are read on every remote image request.
pub(crate) const FETCH_KEYS: [&str; 7] = [
    "privacy.fetchConnectTimeoutMs",
    "privacy.fetchHeadersTimeoutMs",
    "privacy.fetchIdleTimeoutMs",
    "privacy.fetchTotalTimeoutMs",
    "privacy.maxRedirects",
    "privacy.maxConcurrentPerMessage",
    "privacy.maxConcurrentPerAccount",
];

/// The fetch limits from resolved `privacy.*` values; a key with no value keeps
/// the [`FetchLimits`](crate::image_proxy::remote_content::FetchLimits) default.
/// Values are already bounded by the registry; the clamps here only keep a value
/// that bypassed it (a hand-edited row) from producing a zero-length clock or a
/// zero-sized semaphore.
pub(crate) fn fetch_limits(
    value: impl Fn(&str) -> Option<u64>,
) -> crate::image_proxy::remote_content::FetchLimits {
    use std::time::Duration;
    let defaults = crate::image_proxy::remote_content::FetchLimits::default();
    let ms = |key: &str, fallback: Duration, min: u64, max: u64| {
        value(key).map_or(fallback, |v| Duration::from_millis(v.clamp(min, max)))
    };
    let count = |key: &str, fallback: usize, min: u64, max: u64| {
        value(key).map_or(fallback, |v| v.clamp(min, max) as usize)
    };
    let t = defaults.timeouts;
    crate::image_proxy::remote_content::FetchLimits {
        timeouts: mw_egress::FetchTimeouts {
            connect: ms("privacy.fetchConnectTimeoutMs", t.connect, 500, 30_000),
            headers: ms("privacy.fetchHeadersTimeoutMs", t.headers, 500, 60_000),
            idle: ms("privacy.fetchIdleTimeoutMs", t.idle, 500, 60_000),
            total: ms("privacy.fetchTotalTimeoutMs", t.total, 1_000, 120_000),
            max_redirects: count("privacy.maxRedirects", t.max_redirects, 0, 10),
        },
        per_message: count("privacy.maxConcurrentPerMessage", defaults.per_message, 1, 32),
        per_account: count("privacy.maxConcurrentPerAccount", defaults.per_account, 1, 64),
    }
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
    let fetch_values =
        settings_registry::effective_all(&state.store, &subject, |d| FETCH_KEYS.contains(&d.key))
            .await
            .map_err(|_| unavailable())?;
    let fetch = fetch_limits(|key| {
        fetch_values
            .iter()
            .find(|e| e.key == key)
            .and_then(|e| e.value.as_u64())
    });
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
        fetch,
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
pub(crate) struct ReaderFrameResponse {
    pub compatible: bool,
}

#[derive(Default, Deserialize)]
pub(crate) struct ReaderFrameQuery {
    coep: Option<String>,
}

pub(crate) async fn frame(Query(query): Query<ReaderFrameQuery>) -> Response {
    use base64::Engine as _;
    use sha2::Digest;
    let script = include_str!("remote-frame.js")
        .replace("\r\n", "\n")
        .replace('\r', "\n");
    let hash =
        base64::engine::general_purpose::STANDARD.encode(sha2::Sha256::digest(script.as_bytes()));
    let compatible = query.coep.as_deref() == Some("require-corp");
    // Unlike credentialless, require-corp does not omit credentials on direct
    // third-party images. Keep that fallback limited to our image proxy/parts
    // and embedded data, rather than silently sending third-party cookies.
    let images = if compatible {
        "'self' data: blob:"
    } else {
        "'self' https: data: blob:"
    };
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
    response
        .extensions_mut()
        .insert(ReaderFrameResponse { compatible });
    response
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    #[test]
    fn fetch_limits_default_then_honour_values_and_clamp_bypassed_rows() {
        let none = fetch_limits(|_| None);
        assert_eq!(none, crate::image_proxy::remote_content::FetchLimits::default());
        assert_eq!(none.timeouts, mw_egress::FetchTimeouts::REMOTE_IMAGE);

        let set = fetch_limits(|key| match key {
            "privacy.fetchConnectTimeoutMs" => Some(1500),
            "privacy.fetchHeadersTimeoutMs" => Some(2500),
            "privacy.fetchIdleTimeoutMs" => Some(3500),
            "privacy.fetchTotalTimeoutMs" => Some(4500),
            "privacy.maxRedirects" => Some(0),
            "privacy.maxConcurrentPerMessage" => Some(2),
            "privacy.maxConcurrentPerAccount" => Some(3),
            _ => None,
        });
        assert_eq!(set.timeouts.connect, Duration::from_millis(1500));
        assert_eq!(set.timeouts.headers, Duration::from_millis(2500));
        assert_eq!(set.timeouts.idle, Duration::from_millis(3500));
        assert_eq!(set.timeouts.total, Duration::from_millis(4500));
        assert_eq!(set.timeouts.max_redirects, 0);
        assert_eq!((set.per_message, set.per_account), (2, 3));

        let wild = fetch_limits(|key| Some(if key.ends_with("Ms") { 0 } else { u64::MAX }));
        assert_eq!(wild.timeouts.connect, Duration::from_millis(500));
        assert_eq!(wild.timeouts.total, Duration::from_secs(1));
        assert_eq!(wild.timeouts.max_redirects, 10);
        assert_eq!((wild.per_message, wild.per_account), (32, 64));
    }
}
