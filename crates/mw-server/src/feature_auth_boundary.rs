//! Cookie JMAP policy work precedes, rather than replaces, final authentication.
use super::{Needs, check};
use crate::AppState;
use axum::{
    extract::Request,
    http::{HeaderMap, Method, StatusCode, header},
    response::{IntoResponse, Response},
};
use mw_store::{Session, settings_policy::SettingsReadRevision};

/// Only non-authoritative request requirements cross middleware/handler.
#[derive(Clone)]
pub(crate) struct DeferredJmapFeatures(
    pub(super) Needs,
    pub(super) SettingsReadRevision,
    std::sync::Arc<ReadRestart>,
);
/// One request's control-flow token, NEVER authority. Private bytes bind it to
/// the exact validated metadata-only body; clones share one consumption bit.
#[derive(Default)]
struct ReadRestart {
    body: std::sync::OnceLock<[u8; 32]>,
    consumed: std::sync::atomic::AtomicBool,
    witness: std::sync::OnceLock<mw_store::primary_policy_snapshot::PrimaryPolicyWitness>,
    knobs: std::sync::OnceLock<crate::rate_leeway::Knobs>,
    final_checked: std::sync::atomic::AtomicBool,
}
impl ReadRestart {
    fn prepare(&self, body: &[u8]) -> bool {
        if super::native_metadata_admission::eligible_request(body).is_none() {
            return false;
        }
        self.body.set(body_digest(body)).is_ok()
    }
    fn take(&self, body: &[u8]) -> Result<bool, ()> {
        let Some(expected) = self.body.get() else {
            return Ok(false);
        };
        if self
            .consumed
            .swap(true, std::sync::atomic::Ordering::AcqRel)
            || *expected != body_digest(body)
            || super::native_metadata_admission::eligible_request(body).is_none()
        {
            return Err(());
        }
        Ok(true)
    }
}
fn body_digest(body: &[u8]) -> [u8; 32] {
    use sha2::Digest;
    sha2::Sha256::digest(body).into()
}
impl DeferredJmapFeatures {
    pub(super) fn new(needs: Needs, revision: SettingsReadRevision) -> Self {
        Self(needs, revision, Default::default())
    }
    /// Called only after old permits/connector have been dropped, before ANY
    /// Engine invocation. Body validation inside this method prevents a forged
    /// extension from requesting a fresh restart for a mutation or mixed batch.
    pub(super) fn restart_metadata_read(&self, body: &[u8]) -> bool {
        self.2.prepare(body)
    }
}
/// Request-only observer retained by the ORIGINAL handler while it performs
/// its own permits/connector awaits. Empty for every ordinary un-restarted call.
pub(crate) struct ReadRestartObserver(std::sync::Arc<ReadRestart>);
pub(crate) fn restart_observer(features: &DeferredJmapFeatures) -> ReadRestartObserver {
    ReadRestartObserver(features.2.clone())
}
impl ReadRestartObserver {
    pub(crate) fn active(&self) -> bool {
        self.0.witness.get().is_some()
    }
    pub(crate) fn record_knobs(&self, knobs: crate::rate_leeway::Knobs) -> Result<(), Response> {
        if self.active() {
            self.0.knobs.set(knobs).map_err(|_| auth_fence_refusal())?;
        }
        Ok(())
    }
    pub(crate) async fn finish(
        &self,
        state: &AppState,
        headers: &HeaderMap,
        session: &Session,
        body: &[u8],
    ) -> Result<(), Response> {
        if self.0.body.get().is_none() {
            return Ok(());
        }
        if self
            .0
            .final_checked
            .swap(true, std::sync::atomic::Ordering::AcqRel)
            || self.0.body.get() != Some(&body_digest(body))
        {
            return Err(auth_fence_refusal());
        }
        let witness = self.0.witness.get().ok_or_else(auth_fence_refusal)?;
        let knobs = self.0.knobs.get().ok_or_else(auth_fence_refusal)?;
        super::native_metadata_admission::finish_restarted_read(
            state, headers, session, body, witness, *knobs,
        )
        .await
    }
    #[cfg(test)]
    pub(crate) fn record_engine_dispatch(&self) {
        if self.active() {
            super::native_metadata_admission::record_restart_dispatch();
        }
    }
}

/// Preserve the handler's admin fallback for authentication failures only.
/// Feature refusal must never be reclassified as an unauthenticated request.
pub(crate) enum BoundaryError {
    Authentication(Response),
    Features(Response),
}

pub(super) fn eligible(state: &AppState, request: &Request) -> bool {
    if request.method() != Method::POST
        || request.uri().path() != "/jmap/api"
        || request.headers().contains_key(header::AUTHORIZATION)
        || request
            .headers()
            .contains_key(crate::linked_accounts::SELECTOR)
        || request
            .extensions()
            .get::<crate::scope_mw::RestSessionSource>()
            .is_some()
        || crate::cookie_value(request.headers()).is_none()
        || request
            .headers()
            .get(header::CONTENT_LENGTH)
            .and_then(|value| value.to_str().ok())
            .and_then(|value| value.parse::<usize>().ok())
            .is_some_and(|length| length > mw_engine::jmap::core::MAX_SIZE_REQUEST)
    {
        return false;
    }
    #[cfg(test)]
    if request.extensions().get::<ForceLegacy>().is_some() {
        return false;
    }
    request
        .extensions()
        .get::<SettingsReadRevision>()
        .is_some_and(|proof| proof.scope() == state.store.cache_scope() && proof.number().is_some())
}
pub(super) fn same_session(hint: &Session, authenticated: &Session) -> bool {
    hint.id == authenticated.id
        && hint.account_id == authenticated.account_id
        && hint.username == authenticated.username
        && hint.jmap_url == authenticated.jmap_url
        && hint.api_url == authenticated.api_url
        && hint.credentials == authenticated.credentials
}
const REFUSAL_TYPE: &str = "urn:mailwoman:problem:authorization-not-confirmed";
const REFUSAL_CODE: &str = "authorizationNotConfirmedBeforeDispatch";
/// Only this module's authenticated pre-dispatch source fence may mint this
/// refusal. It says nothing about Engine/upstream/SMTP uncertainty or errors.
pub(super) fn auth_fence_refusal() -> Response {
    (
        StatusCode::SERVICE_UNAVAILABLE,
        [
            (header::CONTENT_TYPE, "application/problem+json"),
            (header::RETRY_AFTER, "1"),
            (header::CACHE_CONTROL, "no-store"),
        ],
        axum::Json(serde_json::json!({
            "type": REFUSAL_TYPE,
            "title": "Authorization could not be confirmed before dispatch",
            "status": 503,
            "code": REFUSAL_CODE,
            "notDispatched": true,
            "retryAfterSecs": 1,
        })),
    )
        .into_response()
}
/// The first read is an untrusted resolution hint, NEVER an authenticated
/// Session. Even a denied/error feature decision must cross the full final
/// authentication chain before its response can be exposed.
pub(crate) async fn authenticate_deferred(
    state: &AppState,
    headers: &HeaderMap,
    mut features: DeferredJmapFeatures,
    body: &[u8],
) -> Result<Session, BoundaryError> {
    let id = crate::cookie_value(headers)
        .ok_or_else(|| BoundaryError::Authentication(crate::unauthorized()))?;
    let restarting = match features.2.take(body) {
        Ok(true) => {
            if state.engine.is_none()
                || headers.contains_key(header::AUTHORIZATION)
                || headers.contains_key(crate::linked_accounts::SELECTOR)
            {
                return Err(BoundaryError::Features(auth_fence_refusal()));
            }
            // Rebuild UNPRUNED requirements from the same actual validated body.
            // In particular an old absent-protocol optimization is not proof
            // that access.protocols is still absent at this new boundary.
            let Some(mut needs) = super::metadata_read_restart_needs(body) else {
                return Err(BoundaryError::Features(auth_fence_refusal()));
            };
            let revision = match state.store.settings_read_revision().await {
                Ok(revision) if revision.number().is_some() => revision,
                _ => {
                    crate::authed(state, headers)
                        .await
                        .map_err(BoundaryError::Authentication)?;
                    return Err(BoundaryError::Features(auth_fence_refusal()));
                }
            };
            if needs.protocol {
                needs.protocol = state
                    .store
                    .administrator_setting_present_at("access.protocols", &revision)
                    .await
                    .unwrap_or(true);
            }
            features.0 = needs;
            features.1 = revision;
            true
        }
        Ok(false) => false,
        Err(()) => return Err(BoundaryError::Features(auth_fence_refusal())),
    };
    #[cfg(test)]
    wait_pause(PauseAt::BeforeHint, &id).await;
    if let Some(inputs) = crate::request_inputs::current(&state.store).await {
        // Reuse SOURCE, not an authenticated result. The same original pure
        // feature and authentication chain runs below using these readers.
        if restarting { return Err(BoundaryError::Features(auth_fence_refusal())); }
        let source = inputs.source().ok_or_else(|| BoundaryError::Features(auth_fence_refusal()))?;
        let revision = source.primary_policy_revision(&state.store).ok().flatten()
            .filter(|revision| revision.scope() == features.1.scope()
                && revision.number().is_some() && revision.number() == features.1.number())
            .ok_or_else(|| BoundaryError::Features(auth_fence_refusal()))?;
        let _ = revision;
        let hint = inputs.session();
        let decision = check(state, hint, &features.0).await;
        #[cfg(test)] after_prepared(&id).await;
        let authenticated = crate::authed(state, headers).await.map_err(BoundaryError::Authentication)?;
        #[cfg(test)] wait_pause(PauseAt::AfterAuthentication, &id).await;
        crate::request_inputs::refresh().await.map_err(BoundaryError::Features)?;
        if !same_session(hint, &authenticated) {
            return Err(BoundaryError::Features(auth_fence_refusal()));
        }
        decision.map_err(BoundaryError::Features)?;
        return Ok(authenticated);
    }
    let hinted = match state.store.primary_session_policy_snapshot(&id).await {
        Ok(hinted) if hinted.witness.matches_revision(&features.1) => hinted,
        _ => {
            // A read failure cannot turn a stale feature decision into authority.
            // Preserve final auth refusal (including expiry); if it succeeds,
            // require a new request rather than evaluate features after auth.
            crate::authed(state, headers)
                .await
                .map_err(BoundaryError::Authentication)?;
            return Err(BoundaryError::Features(auth_fence_refusal()));
        }
    };
    let decision = check(state, &hinted.session, &features.0).await;
    #[cfg(test)]
    after_prepared(&id).await;
    let authenticated = crate::authed(state, headers)
        .await
        .map_err(BoundaryError::Authentication)?;
    #[cfg(test)]
    wait_pause(PauseAt::AfterAuthentication, &id).await;
    // This is the final authentication source fence, not another asynchronous
    // feature lookup. It observes policy/subject changes DURING domain/clock/
    // account guard awaits without reusing an earlier namespace proof.
    let observed = state
        .store
        .primary_session_policy_witness(&id)
        .await
        .map_err(|_| BoundaryError::Features(auth_fence_refusal()))?;
    let matching_witness = hinted.witness.matches(&observed);
    // Domain upgrade can replace Session fields AFTER the cookie witness read.
    // No async lookup follows final auth; changed subject/source is retryable.
    if !matching_witness || !same_session(&hinted.session, &authenticated) {
        return Err(BoundaryError::Features(auth_fence_refusal()));
    }
    decision.map_err(BoundaryError::Features)?;
    if restarting {
        features
            .2
            .witness
            .set(observed)
            .map_err(|_| BoundaryError::Features(auth_fence_refusal()))?;
    }
    Ok(authenticated)
}

#[cfg(test)]
#[derive(Clone)]
pub(super) struct ForceLegacy;
#[cfg(test)]
#[derive(Clone, Copy, Hash, PartialEq, Eq)]
pub(super) enum PauseAt {
    BeforeHint,
    AfterFeatures,
    AfterDomain,
    AfterAuthentication,
}
#[cfg(test)]
static PAUSES: std::sync::LazyLock<
    std::sync::Mutex<std::collections::HashMap<(PauseAt, String), std::sync::Arc<Pause>>>,
> = std::sync::LazyLock::new(Default::default);
#[cfg(test)]
pub(super) struct Pause {
    pub entered: tokio::sync::Notify,
    pub release: tokio::sync::Notify,
}
#[cfg(test)]
pub(super) fn pause_at(stage: PauseAt, session: &str) -> std::sync::Arc<Pause> {
    let pause = std::sync::Arc::new(Pause {
        entered: Default::default(),
        release: Default::default(),
    });
    PAUSES
        .lock()
        .unwrap()
        .insert((stage, session.into()), pause.clone());
    pause
}
#[cfg(test)]
async fn after_prepared(session: &str) {
    wait_pause(PauseAt::AfterFeatures, session).await;
}
#[cfg(test)]
pub(crate) async fn pause_during_final_auth(session: &str) {
    wait_pause(PauseAt::AfterDomain, session).await;
}
#[cfg(test)]
async fn wait_pause(stage: PauseAt, session: &str) {
    let pause = PAUSES.lock().unwrap().remove(&(stage, session.into()));
    if let Some(pause) = pause {
        pause.entered.notify_one();
        pause.release.notified().await;
    }
}
#[cfg(test)]
tokio::task_local! {static AUTH_CALLS: std::cell::Cell<u64>;}
#[cfg(test)]
pub(crate) fn record_auth() {
    let _ = AUTH_CALLS.try_with(|calls| calls.set(calls.get() + 1));
}
#[cfg(test)]
pub(super) async fn count_auth<F: std::future::Future>(future: F) -> (F::Output, u64) {
    AUTH_CALLS
        .scope(std::cell::Cell::new(0), async {
            let result = future.await;
            let count = AUTH_CALLS.with(std::cell::Cell::get);
            (result, count)
        })
        .await
}
#[cfg(test)]
#[path = "feature_auth_boundary_tests.rs"]
mod tests;
