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
    // t87/t99: a linked request checks this up to four times (parent and source,
    // in `resolve` and again in `enforce_context`), and each resolution costs
    // about 25 settings queries. A session that passed recently, under the same
    // settings write generation, passes again without resolving. Only a pass is
    // kept, so a refusal is always fresh and any settings write ends every pass.
    let key = memo::Key::of(session);
    if memo::passed(&key) {
        return Ok(());
    }
    let generation = mw_store::settings_policy::settings_generation();
    memo::note_resolution();
    if !supported(state, session).await? {
        return Err(failure(StatusCode::CONFLICT, "linkedBackendUnsupported"));
    }
    if enabled(state, session).await? {
        memo::remember(key, generation);
        Ok(())
    } else {
        Err(failure(StatusCode::FORBIDDEN, "linkingDisabled"))
    }
}

/// The per-session memo of a passed linking-policy check (t87, from t99's
/// profile). An entry holds for [`memo::TTL`] at most, and only while no
/// statement has written a settings-resolution table since it was resolved
/// (`mw_store::settings_policy::settings_generation`), so an admin change
/// applies to the next request. The TTL bounds what another server process
/// sharing the database can leave stale.
pub(crate) mod memo {
    use std::collections::HashMap;
    use std::sync::Mutex;
    use std::sync::atomic::{AtomicU64, Ordering};
    use std::time::{Duration, Instant};

    pub(crate) const TTL: Duration = Duration::from_secs(5);
    const CAPACITY: usize = 4096;

    /// What the check reads from the session itself, besides its id.
    #[derive(Clone, PartialEq, Eq, Hash, Debug)]
    pub(crate) struct Key {
        session: String,
        jmap_url: String,
        api_url: String,
    }
    impl Key {
        pub(crate) fn of(session: &mw_store::Session) -> Self {
            Key {
                session: session.id.clone(),
                jmap_url: session.jmap_url.clone(),
                api_url: session.api_url.clone(),
            }
        }
    }

    struct Entry {
        generation: u64,
        at: Instant,
    }
    static PASSED: Mutex<Option<HashMap<Key, Entry>>> = Mutex::new(None);
    static RESOLUTIONS: AtomicU64 = AtomicU64::new(0);

    fn fresh(entry: &Entry, generation: u64, now: Instant) -> bool {
        entry.generation == generation && now.duration_since(entry.at) < TTL
    }

    pub(crate) fn passed(key: &Key) -> bool {
        let generation = mw_store::settings_policy::settings_generation();
        let now = Instant::now();
        let mut guard = PASSED.lock().unwrap_or_else(|e| e.into_inner());
        let map = guard.get_or_insert_with(HashMap::new);
        match map.get(key) {
            Some(entry) if fresh(entry, generation, now) => true,
            Some(_) => {
                map.remove(key);
                false
            }
            None => false,
        }
    }

    /// Keep a pass resolved under `generation` (read BEFORE resolving, so a
    /// write that raced the resolution leaves the entry already stale).
    pub(crate) fn remember(key: Key, generation: u64) {
        let now = Instant::now();
        let current = mw_store::settings_policy::settings_generation();
        let mut guard = PASSED.lock().unwrap_or_else(|e| e.into_inner());
        let map = guard.get_or_insert_with(HashMap::new);
        if map.len() >= CAPACITY {
            map.retain(|_, entry| fresh(entry, current, now));
            if map.len() >= CAPACITY {
                map.clear();
            }
        }
        map.insert(
            key,
            Entry {
                generation,
                at: now,
            },
        );
    }

    /// Count a real resolution: here and as the Prometheus counter
    /// `mw_linked_policy_resolutions_total` (no labels, no identifiers).
    pub(crate) fn note_resolution() {
        RESOLUTIONS.fetch_add(1, Ordering::Relaxed);
        metrics::counter!("mw_linked_policy_resolutions_total").increment(1);
    }

    /// How many times this process resolved the linking policy (not answered
    /// from the memo).
    #[allow(dead_code)]
    pub(crate) fn resolutions() -> u64 {
        RESOLUTIONS.load(Ordering::Relaxed)
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        fn key(id: &str) -> Key {
            Key {
                session: id.into(),
                jmap_url: "engine".into(),
                api_url: "engine".into(),
            }
        }

        #[test]
        fn a_pass_is_kept_until_a_settings_write_or_the_ttl() {
            let generation = mw_store::settings_policy::settings_generation();
            remember(key("memo-a"), generation);
            assert!(passed(&key("memo-a")));
            assert!(!passed(&key("memo-other")));
            // Another URL pair is another key (a domain refresh changed it).
            let mut moved = key("memo-a");
            moved.jmap_url = "https://jmap.example".into();
            assert!(!passed(&moved));
            // A pass resolved under an older generation is stale.
            remember(key("memo-b"), generation.wrapping_sub(1));
            assert!(!passed(&key("memo-b")));
            // The TTL.
            let old = Entry {
                generation,
                at: Instant::now() - TTL - Duration::from_millis(1),
            };
            assert!(!fresh(&old, generation, Instant::now()));
        }
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
