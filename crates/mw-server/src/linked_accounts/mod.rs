//! Explicit mailbox context selection; the browser's primary identity never changes.
mod credential;
mod live;
mod routes;
mod scope;
use crate::AppState;
use axum::{
    Json,
    http::{HeaderMap, StatusCode},
    response::{IntoResponse, Response},
};
pub(crate) use credential::STORE_KEY as STORE_CREDENTIALS_KEY;
pub(crate) use live::{
    LiveLink, inbox_unread, links_changed, live_links, subscribe_links, verify_link,
};
pub(crate) use routes::router;
pub(crate) use scope::guard;
use serde_json::json;

pub(crate) const SELECTOR: &str = "x-mailwoman-link";

/// The link a request selected, for a handler that has already resolved its
/// session through `crate::authed` (which proves the link belongs to the
/// signed-in session, and refuses the request otherwise). t87 uses it to tie
/// the durable credentials a linked mailbox creates to the link, so unlinking
/// revokes them (D1, D3).
pub(crate) fn selected_link(headers: &HeaderMap) -> Option<String> {
    headers
        .get(SELECTOR)
        .and_then(|v| v.to_str().ok())
        .filter(|v| uuid::Uuid::parse_str(v).is_ok())
        .map(str::to_owned)
}
#[derive(Clone)]
pub(crate) struct PasswordProof;

fn failure(status: StatusCode, code: &str) -> Response {
    (
        status,
        [(axum::http::header::CACHE_CONTROL, "private, no-store")],
        Json(json!({"code":code,"error":code})),
    )
        .into_response()
}
fn unavailable(_: impl std::fmt::Display) -> Response {
    failure(StatusCode::SERVICE_UNAVAILABLE, "linkedAccountUnavailable")
}
fn child_error(response: Response) -> Response {
    if response.status() == StatusCode::UNAUTHORIZED {
        failure(
            StatusCode::CONFLICT,
            "linkedAccountReauthenticationRequired",
        )
    } else {
        response
    }
}
async fn effective(
    state: &AppState,
    session: &mw_store::Session,
    key: &str,
) -> Result<bool, Response> {
    Ok(
        crate::settings_routes::effective_for_session(state, session, key)
            .await
            .map_err(unavailable)?
            .and_then(|v| v.value.as_bool())
            .unwrap_or(false),
    )
}
pub(crate) async fn enabled(
    state: &AppState,
    session: &mw_store::Session,
) -> Result<bool, Response> {
    if !supported(state, session).await? {
        return Ok(false);
    }
    let global = crate::settings_registry::effective_for(
        &state.store,
        "accounts.linkingEnabled",
        &Default::default(),
    )
    .await
    .map_err(unavailable)?
    .and_then(|v| v.value.as_bool())
    .unwrap_or(false);
    Ok(global
        && effective(state, session, "accounts.linkingEnabled").await?
        && effective(state, session, "accounts.useLinked").await?)
}
async fn supported(state: &AppState, session: &mw_store::Session) -> Result<bool, Response> {
    Ok(session.jmap_url == "engine"
        && session.api_url == "engine"
        && state
            .store
            .session_policy_subject(&session.id)
            .await
            .map_err(unavailable)?
            .is_some_and(|subject| subject.native_account.is_some()))
}
async fn maximum(state: &AppState) -> Result<usize, Response> {
    Ok(crate::settings_registry::effective_for(
        &state.store,
        "accounts.maxLinked",
        &Default::default(),
    )
    .await
    .map_err(unavailable)?
    .and_then(|v| v.value.as_u64())
    .unwrap_or(5)
    .clamp(1, 16) as usize)
}
async fn require_enabled(state: &AppState, session: &mw_store::Session) -> Result<(), Response> {
    if !supported(state, session).await? {
        return Err(failure(StatusCode::CONFLICT, "linkedBackendUnsupported"));
    }
    if enabled(state, session).await? {
        Ok(())
    } else {
        Err(failure(StatusCode::FORBIDDEN, "linkingDisabled"))
    }
}

/// Normal authentication establishes the primary session first. This function
/// then resolves an independent selector, never the expected-account headers.
pub(crate) async fn resolve(
    state: &AppState,
    headers: &HeaderMap,
    parent: &mw_store::Session,
) -> Result<Option<mw_store::Session>, Response> {
    let Some(link) = headers.get(SELECTOR) else {
        return Ok(None);
    };
    let link = link
        .to_str()
        .map_err(|_| failure(StatusCode::BAD_REQUEST, "invalidLinkedContext"))?;
    require_enabled(state, parent).await?;
    // t48: a lapsed or missing child session is re-established from the link's
    // sealed credential when one is kept; otherwise the owner renews access.
    let reauth = || {
        failure(
            StatusCode::CONFLICT,
            "linkedAccountReauthenticationRequired",
        )
    };
    let Some((_, source)) = credential::live_source(state, parent, link).await? else {
        return Err(reauth());
    };
    require_enabled(state, &source).await?;
    // A delegated context copies the source's clocks; one that lapsed on its
    // own is deleted by the check and rebuilt once from the live source.
    for attempt in 0..2 {
        let context = state
            .store
            .linked_context(&parent.id, link)
            .await
            .map_err(unavailable)?
            .ok_or_else(reauth)?;
        match crate::security_center::enforce_session(state, &context).await {
            Ok(()) => return Ok(Some(context)),
            Err(response) if attempt == 0 && response.status() == StatusCode::UNAUTHORIZED => {
                if credential::live_source(state, parent, link)
                    .await?
                    .is_none()
                {
                    return Err(reauth());
                }
            }
            Err(response) => return Err(child_error(response)),
        }
    }
    Err(reauth())
}

/// Signed resources reach this through central session enforcement as well.
pub(crate) async fn enforce_context(
    state: &AppState,
    session: &mw_store::Session,
) -> Result<(), Response> {
    let authority = state
        .store
        .linked_context_authority(&session.id)
        .await
        .map_err(unavailable)?
        .ok_or_else(crate::unauthorized)?;
    for mut original in [authority.parent, authority.source] {
        crate::security_center::enforce_session_clock(state, &original).await?;
        crate::domain_upgrade::refresh_session(state, &mut original).await?;
        let gate = crate::account_gate::for_session(&state.store, &original)
            .await
            .map_err(unavailable)?;
        if gate.disabled || gate.password_change_required {
            return Err(crate::unauthorized());
        }
        require_enabled(state, &original).await?;
    }
    // Recheck after domain refresh: a changed endpoint must not preserve an old
    // link's authority, even when the upstream raw account ID is unchanged.
    if state
        .store
        .linked_context_authority(&session.id)
        .await
        .map_err(unavailable)?
        .is_none()
    {
        return Err(crate::unauthorized());
    }
    Ok(())
}

pub(crate) async fn record_activity(
    state: &AppState,
    session: &mw_store::Session,
    now: &str,
) -> Result<bool, mw_store::StoreError> {
    if state.store.linked_session_kind(&session.id).await? == Some(2) {
        let Some(authority) = state.store.linked_context_authority(&session.id).await? else {
            return Ok(false);
        };
        // UPDATE-only: an unlink/rotation racing this request cannot revive a row.
        if !state
            .store
            .record_session_activity(&authority.parent.id, now)
            .await?
            || !state
                .store
                .record_session_activity(&authority.source.id, now)
                .await?
        {
            return Ok(false);
        }
    }
    state.store.record_session_activity(&session.id, now).await
}
