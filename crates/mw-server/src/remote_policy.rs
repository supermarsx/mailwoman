//! Effective reader privacy controls, resolved from the shared settings policy.
use crate::{AppState, settings_registry};
use axum::{
    Json,
    extract::{Query, State},
    http::HeaderMap,
    response::{IntoResponse, Response},
};
use serde::Deserialize;
use serde_json::Value;

pub(crate) use mw_remote_content::policy::*;

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
    // Resolve one current policy snapshot for the entire image request. Repeated
    // per-key reads used to reload the same subject/profile rows ten times.
    let resolved = settings_registry::effective_all(&state.store, &subject, |d| {
        FETCH_KEYS.contains(&d.key) || POLICY_KEYS.contains(&d.key)
    })
    .await
    .map_err(|_| unavailable())?;
    let values: std::collections::BTreeMap<_, _> = resolved
        .into_iter()
        .map(|value| (value.key.clone(), value))
        .collect();
    let boolean = |key| {
        values
            .get(key)
            .and_then(|v| v.value.as_bool())
            .unwrap_or(false)
    };
    let locked = |key| values.get(key).is_none_or(|v| v.locked());
    let hosts = |key: &str| -> Vec<String> {
        values
            .get(key)
            .and_then(|v| v.value.as_array())
            .map(|list| {
                list.iter()
                    .filter_map(|item| item.as_str().map(str::to_owned))
                    .take(100)
                    .collect()
            })
            .unwrap_or_default()
    };
    let number = |key: &str, fallback: usize| {
        values
            .get(key)
            .and_then(|v| v.value.as_u64())
            .map(|n| n as usize)
            .unwrap_or(fallback)
    };
    let sizing = |key: &str| {
        crate::image_proxy::adaptive::Sizing::parse(
            values
                .get(key)
                .and_then(|v| v.value.as_str())
                .unwrap_or("auto"),
        )
    };
    let mut fetch = fetch_limits(|key| values.get(key).and_then(|e| e.value.as_u64()));
    fetch.concurrency_mode = crate::image_proxy::adaptive::Mode::parse(
        values
            .get("privacy.imageConcurrencyMode")
            .and_then(|e| e.value.as_str())
            .unwrap_or("auto"),
    );
    fetch.image_bytes = number("privacy.maxBytes", 8 * 1024 * 1024);
    let allow_direct = boolean("privacy.allowDirect") && boolean("cap.remote_content.direct");
    let mut mode = values
        .get("privacy.remoteContent")
        .and_then(|v| Value::as_str(&v.value))
        .unwrap_or("ask")
        .to_owned();
    if mode == "direct" && !allow_direct {
        mode = "ask".into();
    }
    let (image_rate, image_cache) = image_configs(&number, &sizing);
    let resources = crate::image_proxy::resources::snapshot();
    let budget = crate::image_proxy::adaptive::cache_budget(
        image_cache,
        crate::image_proxy::adaptive::calculate(
            fetch.concurrency_mode,
            fetch.ceilings(),
            resources,
            fetch.image_bytes,
            mw_render::media_jail::IMAGE_DECODE_TYPICAL_BYTES,
        ),
        resources,
        fetch.image_bytes,
        mw_render::media_jail::IMAGE_DECODE_TYPICAL_BYTES,
        crate::image_proxy::hit_percent(),
    );
    Ok(PrivacyPolicy {
        mode,
        trackers: boolean("privacy.trackers"),
        tracker_override_allowed: boolean("privacy.trackerOverrideAllowed")
            && !(boolean("privacy.trackers") && locked("privacy.trackers")),
        clean_links: boolean("privacy.cleanLinks"),
        allow_direct,
        inline_parts: state.engine.is_some(),
        remote_image_downloads: boolean("privacy.remoteImageDownloads"),
        trusted_hosts: hosts("images.trustedHosts"),
        blocked_hosts: hosts("images.blockedHosts"),
        locked: Locks {
            mode: locked("privacy.remoteContent"),
            trackers: locked("privacy.trackers"),
            clean_links: locked("privacy.cleanLinks"),
        },
        max_bytes: number("privacy.maxBytes", 8 * 1024 * 1024),
        rate_per_min: number("privacy.ratePerMin", 1200),
        rate_burst: number("privacy.rateBurst", 1024),
        image_rate,
        image_tier: tier_config(&|key| values.get(key).map(|v| v.value.clone()), &budget),
        negative_cache_ttl: std::time::Duration::from_secs(number(
            "privacy.negativeCacheTtlSecs",
            crate::image_proxy::remote_content::fetch_limits::NEGATIVE_TTL.as_secs() as usize,
        ) as u64),
        track_latency: values
            .get("privacy.imageLatencyTracking")
            .and_then(|v| v.value.as_bool())
            .unwrap_or(true),
        image_load_budget_ms: fetch.overall_budget().as_millis() as u64,
        image_retry_attempts: number("privacy.imageRetryAttempts", 3),
        image_retry_delay_ms: number("privacy.imageRetryDelayMs", 5000),
        image_resize: resize_config(&|key| values.get(key).map(|v| v.value.clone())),
        image_resize_max_width: {
            let r = resize_config(&|key| values.get(key).map(|v| v.value.clone()));
            r.enabled.then_some(r.max_width)
        },
        fetch,
    })
}
/// Instance-wide tier configuration for the admin readout and diagnostics.
pub(crate) async fn instance_tier_config(
    state: &AppState,
    budget: &crate::image_proxy::adaptive::CacheBudget,
) -> Result<crate::image_proxy::tier::TierConfig, mw_store::StoreError> {
    let subject = mw_store::settings_policy::SettingSubject::default();
    let values =
        settings_registry::effective_all(&state.store, &subject, |d| POLICY_KEYS.contains(&d.key))
            .await?;
    Ok(tier_config(
        &|key| values.iter().find(|v| v.key == key).map(|v| v.value.clone()),
        budget,
    ))
}

/// Global sizing for administrative diagnostics. No account/domain preference
/// can raise these replica-wide resource ceilings.
pub(crate) async fn instance_limits(
    state: &AppState,
) -> Result<crate::image_proxy::remote_content::FetchLimits, mw_store::StoreError> {
    let subject = mw_store::settings_policy::SettingSubject::default();
    let values = settings_registry::effective_all(&state.store, &subject, |d| {
        FETCH_KEYS.contains(&d.key) || POLICY_KEYS.contains(&d.key)
    })
    .await?;
    let mut fetch = fetch_limits(|key| {
        values
            .iter()
            .find(|value| value.key == key)
            .and_then(|value| value.value.as_u64())
    });
    fetch.concurrency_mode = crate::image_proxy::adaptive::Mode::parse(
        values
            .iter()
            .find(|value| value.key == "privacy.imageConcurrencyMode")
            .and_then(|value| value.value.as_str())
            .unwrap_or("auto"),
    );
    fetch.image_bytes = values
        .iter()
        .find(|value| value.key == "privacy.maxBytes")
        .and_then(|value| value.value.as_u64())
        .unwrap_or(8 * 1024 * 1024) as usize;
    Ok(fetch)
}

/// A tracker exception is independent from image display grants and applies to
/// exactly one message. Re-read permission and grant on every capability use,
/// before cache access, so revocation or a new administrator lock takes effect.
pub(crate) async fn effective_for_message(
    state: &AppState,
    session: &mw_store::Session,
    email_id: &str,
) -> Result<PrivacyPolicy, Response> {
    let mut policy = effective(state, session).await?;
    if policy.tracker_override_allowed
        && policy.mode != "block"
        && state
            .store
            .is_remote_image_granted(&session.account_id, "trackers-single", email_id)
            .await
            .map_err(|_| unavailable())?
    {
        policy.trackers = false;
        // Explicit tracker consent always retains the server proxy: later
        // revocation must not leave direct-origin URLs in the message frame.
        if policy.mode == "direct" {
            policy.mode = "proxy".into();
        }
    }
    Ok(policy)
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
    #[serde(rename = "parentOrigin")]
    parent_origin: Option<String>,
}

fn reader_parent_origin(config: &crate::push_relay::NativeAuthConfig, value: &str) -> bool {
    if value.is_empty()
        || value == "null"
        || value.chars().any(|c| {
            c.is_whitespace()
                || c.is_control()
                || matches!(c, '*' | '\'' | '"' | ';' | '<' | '>' | '\\')
        })
        || !config.allows(value)
    {
        return false;
    }
    let Ok(url) = reqwest::Url::parse(value) else {
        return false;
    };
    if url.host_str().is_none()
        || !url.username().is_empty()
        || url.password().is_some()
        || url.query().is_some()
        || url.fragment().is_some()
        || !matches!(url.path(), "" | "/")
    {
        return false;
    }
    // Accept canonical origins only, including configured non-special shell
    // schemes whose URL.origin() is opaque. No CSP source grammar is accepted.
    url.as_str().trim_end_matches('/') == value && !value.ends_with('/')
}

pub(crate) async fn frame(
    State(state): State<AppState>,
    Query(query): Query<ReaderFrameQuery>,
) -> Response {
    use base64::Engine as _;
    use sha2::Digest;
    let parent_origin = match query.parent_origin.as_deref() {
        Some(origin) if reader_parent_origin(&state.native_auth, origin) => Some(origin),
        Some(_) => {
            return (
                axum::http::StatusCode::FORBIDDEN,
                "reader parent origin refused",
            )
                .into_response();
        }
        None => None,
    };
    let handler = include_str!("reader-address-handler.js")
        .replace("\r\n", "\n")
        .replace('\r', "\n");
    let handler_hash =
        base64::engine::general_purpose::STANDARD.encode(sha2::Sha256::digest(handler.as_bytes()));
    let script = include_str!("remote-frame.js")
        .replace("\r\n", "\n")
        .replace('\r', "\n")
        .replace(
            "__MW_PARENT_ORIGIN__",
            &serde_json::to_string(&parent_origin).expect("origin JSON"),
        )
        .replace(
            "__MW_ADDRESS_HANDLER_SOURCE__",
            &serde_json::to_string(&handler).expect("handler JSON"),
        )
        .replace(
            "__MW_ADDRESS_HANDLER_HASH__",
            &serde_json::to_string(&handler_hash).expect("hash JSON"),
        );
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
    let ancestor = parent_origin.unwrap_or("'self'");
    let csp = format!(
        "default-src 'none'; script-src 'sha256-{hash}' 'sha256-{handler_hash}'; style-src 'unsafe-inline'; frame-src 'self'; img-src {images}; frame-ancestors {ancestor}; base-uri 'none'; form-action 'none'"
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
    fn reader_parent_origin_is_canonical_and_exactly_native_allowlisted() {
        let config = crate::push_relay::NativeAuthConfig {
            extra_origins: vec![
                "app://mailwoman".into(),
                "https://native.example:8443".into(),
                "https://bad.example/path".into(),
            ],
        };
        for value in [
            "tauri://localhost",
            "http://tauri.localhost",
            "https://tauri.localhost",
            "app://mailwoman",
            "https://native.example:8443",
        ] {
            assert!(reader_parent_origin(&config, value), "{value}");
        }
        for value in [
            "null",
            "*",
            "https://unknown.example",
            "http://tauri.localhost/",
            "http://tauri.localhost?x",
            "http://tauri.localhost#x",
            "https://bad.example/path",
            "http://tauri.localhost\n",
            "https://native.example:8443.evil",
        ] {
            assert!(!reader_parent_origin(&config, value), "{value}");
        }
    }

    #[test]
    fn fetch_limits_default_then_honour_values_and_clamp_bypassed_rows() {
        let none = fetch_limits(|_| None);
        assert_eq!(
            none,
            crate::image_proxy::remote_content::FetchLimits::default()
        );
        assert_eq!(none.timeouts, mw_egress::FetchTimeouts::REMOTE_IMAGE);
        assert_eq!(none.queue_timeout, Duration::from_secs(60));

        let set = fetch_limits(|key| match key {
            "privacy.fetchConnectTimeoutMs" => Some(1500),
            "privacy.fetchHeadersTimeoutMs" => Some(2500),
            "privacy.fetchIdleTimeoutMs" => Some(3500),
            "privacy.fetchTotalTimeoutMs" => Some(4500),
            "privacy.fetchQueueTimeoutMs" => Some(6500),
            "privacy.maxRedirects" => Some(0),
            "privacy.maxConcurrentPerMessage" => Some(2),
            "privacy.maxConcurrentPerAccount" => Some(3),
            _ => None,
        });
        assert_eq!(set.timeouts.connect, Duration::from_millis(1500));
        assert_eq!(set.timeouts.headers, Duration::from_millis(2500));
        assert_eq!(set.timeouts.idle, Duration::from_millis(3500));
        assert_eq!(set.timeouts.total, Duration::from_millis(4500));
        assert_eq!(set.queue_timeout, Duration::from_millis(6500));
        assert_eq!(set.overall_budget(), Duration::from_secs(21));
        assert_eq!(set.timeouts.max_redirects, 0);
        assert_eq!((set.per_message, set.per_account), (2, 3));

        let wild = fetch_limits(|key| Some(if key.ends_with("Ms") { 0 } else { u64::MAX }));
        assert_eq!(wild.timeouts.connect, Duration::from_millis(500));
        assert_eq!(wild.timeouts.total, Duration::from_secs(1));
        assert_eq!(wild.queue_timeout, Duration::from_secs(1));
        assert_eq!(
            fetch_limits(|key| (key == "privacy.fetchQueueTimeoutMs").then_some(u64::MAX))
                .queue_timeout,
            Duration::from_secs(180)
        );
        assert_eq!(wild.timeouts.max_redirects, 10);
        assert_eq!((wild.per_message, wild.per_account), (128, 256));
    }
}

/// Global (no user override) rate and cache configuration, for the admin readout.
pub(crate) async fn instance_image_configs(
    state: &AppState,
) -> Result<
    (
        crate::image_proxy::adaptive::RateConfig,
        crate::image_proxy::adaptive::CacheConfig,
    ),
    mw_store::StoreError,
> {
    let subject = mw_store::settings_policy::SettingSubject::default();
    let values =
        settings_registry::effective_all(&state.store, &subject, |d| POLICY_KEYS.contains(&d.key))
            .await?;
    let number = |key: &str, fallback: usize| {
        values
            .iter()
            .find(|v| v.key == key)
            .and_then(|v| v.value.as_u64())
            .map_or(fallback, |n| n as usize)
    };
    let sizing = |key: &str| {
        crate::image_proxy::adaptive::Sizing::parse(
            values
                .iter()
                .find(|v| v.key == key)
                .and_then(|v| v.value.as_str())
                .unwrap_or("auto"),
        )
    };
    Ok(image_configs(&number, &sizing))
}
