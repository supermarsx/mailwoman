//! Durable linked-mailbox access (t48).
//!
//! A link points at a private child session (`linked_internal=1`). That session
//! has the child's own idle and absolute clocks, and the clock check deletes it
//! when either lapses, so before t48 a link stopped working thirty minutes after
//! its mailbox was last opened. Now the child's password, as the link flow
//! already collected it, is kept sealed in `linked_credential` (server key,
//! XChaCha20-Poly1305, AAD = owner principal + link id + child subject + child
//! storage authority + login name). When the child session has lapsed or is
//! gone, [`live_source`] signs the child in again from it with the normal
//! engine login, checks that the result is the same principal, subject, account
//! and storage authority, and rebinds the link to the new session.
//!
//! What is never stored or replayed: a second factor. A child that has one
//! enrolled, or for which a policy requires one, keeps no secret here; its link
//! asks the owner to renew access (inline, no unlink) when its session ends.
//! Also not stored: zero-access accounts (owner or child), links of an owner or
//! child for whom `accounts.storeLinkedCredentials` is off, and EWS children.
//!
//! The secret never leaves the server: no route returns it, and it is not
//! logged. When upstream refuses it the secret is erased and the link shows
//! "renew access"; every store, refusal and deletion writes an audit row
//! without it.
use super::*;
use serde_json::Value;
use std::collections::HashMap;
use std::sync::{Arc, LazyLock, Mutex};
use zeroize::Zeroize;

pub(crate) const STORE_KEY: &str = "accounts.storeLinkedCredentials";

/// Why a link's access does or does not renew by itself. Shown to the owner.
pub(crate) async fn renewal(
    state: &AppState,
    parent: &mw_store::Session,
    link: &str,
) -> Result<(Value, Value), Response> {
    let status = state
        .store
        .linked_credential_status(&parent.id, link)
        .await
        .map_err(unavailable)?;
    Ok(match status {
        Some(s) if s.state == "active" => (json!("automatic"), Value::Null),
        Some(s) => (json!("signIn"), json!(s.reason)),
        None => (json!("signIn"), Value::Null),
    })
}

/// The login name to offer in "Renew access", when one is known.
pub(crate) async fn login_name(
    state: &AppState,
    parent: &mw_store::Session,
    link: &str,
) -> Option<String> {
    state
        .store
        .linked_credential_status(&parent.id, link)
        .await
        .ok()
        .flatten()
        .map(|s| s.login_name)
}

async fn bool_setting(
    state: &AppState,
    subject: &mw_store::settings_policy::SettingSubject<'_>,
    key: &str,
) -> Result<bool, Response> {
    Ok(
        crate::settings_registry::effective_for(&state.store, key, subject)
            .await
            .map_err(unavailable)?
            .and_then(|v| v.value.as_bool())
            .unwrap_or(false),
    )
}

/// May a secret be kept for a link of `parent` to the child `account`/`login`?
/// `Err(code)` names why not; a store error fails closed (not allowed).
async fn storing_allowed(
    state: &AppState,
    parent: &mw_store::Session,
    account: &str,
    login: &str,
) -> Result<Result<(), &'static str>, Response> {
    let global = bool_setting(state, &Default::default(), STORE_KEY).await?;
    let child = mw_store::settings_policy::SettingSubject {
        personal_owner: None,
        domain: login.rsplit_once('@').map_or("", |(_, d)| d),
        account_id: account,
        address: login,
    };
    if !global
        || !effective(state, parent, STORE_KEY).await?
        || !bool_setting(state, &child, STORE_KEY).await?
    {
        return Ok(Err("policyOff"));
    }
    // Linking itself off for the owner or for the linked mailbox's subject.
    if !enabled(state, parent).await?
        || !bool_setting(state, &child, "accounts.linkingEnabled").await?
    {
        return Ok(Err("linkingDisabled"));
    }
    for id in [parent.account_id.as_str(), account] {
        // Unreadable reads as zero-access: fail closed.
        let zero = state
            .store
            .get_zeroaccess(id)
            .await
            .map_or(true, |row| row.is_some_and(|r| r.enabled));
        if zero {
            return Ok(Err("zeroAccess"));
        }
    }
    Ok(Ok(()))
}

/// Whether new links of `parent` keep a sealed credential: `(true, null)` or
/// `(false, "policyOff" | "zeroAccess")`. The child's own policy and factors
/// are only known once it signs in, so this is the owner's half.
pub(crate) async fn owner_policy(
    state: &AppState,
    parent: &mw_store::Session,
) -> Result<(bool, Value), Response> {
    let global = bool_setting(state, &Default::default(), STORE_KEY).await?;
    if !global || !effective(state, parent, STORE_KEY).await? {
        return Ok((false, json!("policyOff")));
    }
    let zero = state
        .store
        .get_zeroaccess(&parent.account_id)
        .await
        .map_or(true, |row| row.is_some_and(|r| r.enabled));
    Ok(if zero {
        (false, json!("zeroAccess"))
    } else {
        (true, Value::Null)
    })
}

pub(crate) async fn audit(state: &AppState, actor: &str, owner: &str, action: &str, detail: Value) {
    let row = mw_store::AuditRow {
        id: uuid::Uuid::new_v4().to_string(),
        ts: chrono::Utc::now().to_rfc3339(),
        actor: actor.into(),
        actor_kind: if actor == "system" { "system" } else { "user" }.into(),
        action: action.into(),
        target: Some(format!("account:{owner}")),
        detail_json: detail.to_string(),
        ip: None,
    };
    if let Err(e) = state.store.append_audit(&row).await {
        tracing::warn!("linked credential audit not written: {e}");
    }
}

/// After a link was made or renewed with `child` (a fresh, verified login of
/// the linked mailbox): keep its password sealed, or record why not.
pub(crate) async fn remember(
    state: &AppState,
    parent: &mw_store::Session,
    link: &str,
    child: &mw_store::Session,
) -> Result<(), Response> {
    let now = chrono::Utc::now().timestamp();
    let login = child.credentials.username.as_str();
    let authority = state
        .store
        .session_storage_authority(&child.id)
        .await
        .map_err(unavailable)?
        .ok_or_else(|| unavailable("child authority"))?;
    let method = state
        .store
        .primary_principal(&child.id)
        .await
        .map_err(unavailable)?
        .map(|p| p.authentication_method)
        .unwrap_or_default();
    let ews = state
        .store
        .get_account(&child.account_id)
        .await
        .is_ok_and(|a| a.kind == mw_store::AccountKind::Ews);
    let why = if let Err(why) = storing_allowed(state, parent, &child.account_id, login).await? {
        Some(why)
    } else if method != "password"
        || crate::twofa_routes::factor_required(state, &child.account_id, &child.username)
            .await
            .map_err(unavailable)?
    {
        Some("factorRequired")
    } else if ews {
        Some("unsupported")
    } else {
        None
    };
    let detail = json!({"link":link,"reason":why});
    match why {
        None => {
            let stored = state
                .store
                .put_linked_credential(
                    &parent.id,
                    link,
                    login,
                    &child.credentials.password,
                    &authority,
                    now,
                )
                .await
                .map_err(unavailable)?;
            if stored {
                audit(
                    state,
                    &parent.account_id,
                    &parent.account_id,
                    "linked-credential-stored",
                    detail,
                )
                .await;
            }
        }
        Some(why) => {
            let had = state
                .store
                .linked_credential_status(&parent.id, link)
                .await
                .map_err(unavailable)?
                .is_some_and(|s| s.state == "active");
            state
                .store
                .note_linked_credential_unavailable(&parent.id, link, login, &authority, why, now)
                .await
                .map_err(unavailable)?;
            if had {
                audit(
                    state,
                    &parent.account_id,
                    &parent.account_id,
                    "linked-credential-deleted",
                    detail,
                )
                .await;
            }
        }
    }
    Ok(())
}

/// The link's source session while its clocks hold.
async fn current(
    state: &AppState,
    parent: &mw_store::Session,
    link: &str,
) -> Result<Option<(mw_store::linked_accounts::LinkedAccount, mw_store::Session)>, Response> {
    let Some(found) = state
        .store
        .linked_source(&parent.id, link)
        .await
        .map_err(unavailable)?
    else {
        return Ok(None);
    };
    match crate::security_center::enforce_session_clock(state, &found.1).await {
        Ok(()) => Ok(Some(found)),
        Err(response) if response.status() == StatusCode::UNAUTHORIZED => Ok(None),
        Err(response) => Err(response),
    }
}

/// The link's live source session, signing the child in again from its stored
/// credential when the old one lapsed or is gone. `Ok(None)`: the owner has to
/// renew access. `Err(503)`: upstream could not be reached; nothing changed.
pub(crate) async fn live_source(
    state: &AppState,
    parent: &mw_store::Session,
    link: &str,
) -> Result<Option<(mw_store::linked_accounts::LinkedAccount, mw_store::Session)>, Response> {
    if let Some(found) = current(state, parent, link).await? {
        // A secret kept while policy, zero-access or linking changed since is
        // erased now, not only at the next lapse.
        enforce_policy(
            state,
            parent,
            link,
            &found.0.account_id,
            &found.1.credentials.username,
        )
        .await?;
        return Ok(Some(found));
    }
    // After a transient upstream failure, wait before signing in again, so
    // listings and realtime checks do not hammer (or lock out) the mailbox.
    if BACKOFF
        .lock()
        .unwrap_or_else(|e| e.into_inner())
        .get(link)
        .is_some_and(|until| std::time::Instant::now() < *until)
    {
        return Err(failure(
            StatusCode::SERVICE_UNAVAILABLE,
            "linkedAccountUnavailable",
        ));
    }
    // One renewal per link at a time (per process): a listing, a mailbox
    // request and the realtime stream may all notice the lapse together. Across
    // nodes the rebind is serialized in the store and a loser's session is
    // discarded.
    static RENEWING: LazyLock<Mutex<HashMap<String, Arc<tokio::sync::Mutex<()>>>>> =
        LazyLock::new(Default::default);
    let gate = RENEWING
        .lock()
        .unwrap_or_else(|e| e.into_inner())
        .entry(link.to_string())
        .or_default()
        .clone();
    let held = gate.lock().await;
    let result = match current(state, parent, link).await? {
        Some(found) => Ok(Some(found)),
        None => renew(state, parent, link).await,
    };
    drop(held);
    {
        let mut map = RENEWING.lock().unwrap_or_else(|e| e.into_inner());
        if map.get(link).is_some_and(|g| Arc::strong_count(g) <= 2) {
            map.remove(link);
        }
    }
    {
        let mut backoff = BACKOFF.lock().unwrap_or_else(|e| e.into_inner());
        match &result {
            Err(r) if r.status() == StatusCode::SERVICE_UNAVAILABLE => {
                backoff.insert(
                    link.to_string(),
                    std::time::Instant::now() + std::time::Duration::from_secs(60),
                );
            }
            _ => {
                backoff.remove(link);
            }
        }
    }
    result
}

static BACKOFF: LazyLock<Mutex<HashMap<String, std::time::Instant>>> =
    LazyLock::new(Default::default);

/// Erase a still-stored secret that may no longer be kept.
async fn enforce_policy(
    state: &AppState,
    parent: &mw_store::Session,
    link: &str,
    account: &str,
    login: &str,
) -> Result<(), Response> {
    let active = state
        .store
        .linked_credential_status(&parent.id, link)
        .await
        .map_err(unavailable)?
        .is_some_and(|s| s.state == "active");
    if !active {
        return Ok(());
    }
    if let Err(why) = storing_allowed(state, parent, account, login).await?
        && state
            .store
            .reject_linked_credential(&parent.id, link, why, chrono::Utc::now().timestamp())
            .await
            .map_err(unavailable)?
    {
        audit(
            state,
            "system",
            &parent.account_id,
            "linked-credential-deleted",
            json!({"link":link,"reason":why}),
        )
        .await;
    }
    Ok(())
}

fn linked_device() -> mw_store::SessionDevice {
    crate::security_center::device(
        &axum::http::HeaderMap::new(),
        &axum::http::Extensions::new(),
        Some("linked"),
    )
}

async fn refuse(state: &AppState, parent: &mw_store::Session, link: &str, reason: &str) {
    let now = chrono::Utc::now().timestamp();
    if state
        .store
        .reject_linked_credential(&parent.id, link, reason, now)
        .await
        .unwrap_or(false)
    {
        audit(
            state,
            "system",
            &parent.account_id,
            "linked-credential-refresh-failed",
            json!({"link":link,"reason":reason}),
        )
        .await;
    }
}

async fn renew(
    state: &AppState,
    parent: &mw_store::Session,
    link: &str,
) -> Result<Option<(mw_store::linked_accounts::LinkedAccount, mw_store::Session)>, Response> {
    let Some(credential) = state
        .store
        .linked_credential(&parent.id, link)
        .await
        .map_err(unavailable)?
    else {
        return Ok(None);
    };
    if let Err(why) = storing_allowed(
        state,
        parent,
        &credential.account_id,
        &credential.login_name,
    )
    .await?
    {
        // Erase the secret, keep why (shown with "Renew access").
        if state
            .store
            .note_linked_credential_unavailable(
                &parent.id,
                link,
                &credential.login_name,
                &credential.child_authority,
                why,
                chrono::Utc::now().timestamp(),
            )
            .await
            .map_err(unavailable)?
        {
            audit(
                state,
                "system",
                &parent.account_id,
                "linked-credential-deleted",
                json!({"link":link,"reason":why}),
            )
            .await;
        }
        return Ok(None);
    }
    let Some(engine) = state.engine.as_ref() else {
        return Ok(None);
    };
    let upstream = crate::upstream_resolve::resolve_upstream(state, &credential.login_name)
        .await
        .map_err(|_| failure(StatusCode::SERVICE_UNAVAILABLE, "linkedAccountUnavailable"))?;
    let attempt = crate::engine_mode::engine_login(
        engine,
        upstream.url(),
        &credential.login_name,
        &credential.password,
    )
    .await;
    let (account_id, username) = match attempt {
        Ok(pair) => pair,
        Err(error) => {
            use crate::login_failure::LoginFailureReason as R;
            return match error.reason {
                R::Auth | R::Disabled | R::Policy | R::NotConfigured => {
                    // Visible to the linked mailbox's owner like any failed sign-in.
                    crate::security_center::record_failure_backend(
                        state,
                        None,
                        &credential.login_name,
                        &linked_device(),
                        "password",
                        error.reason.code(),
                        Some("engine"),
                    )
                    .await;
                    refuse(state, parent, link, "credentialRejected").await;
                    Ok(None)
                }
                _ => Err(failure(
                    StatusCode::SERVICE_UNAVAILABLE,
                    "linkedAccountUnavailable",
                )),
            };
        }
    };
    if account_id != credential.account_id {
        refuse(state, parent, link, "mailboxChanged").await;
        return Ok(None);
    }
    match crate::domain_config::signin_allowed(&state.store, &credential.login_name).await {
        Ok(true) => {}
        Ok(false) => {
            refuse(state, parent, link, "signinNotAllowed").await;
            return Ok(None);
        }
        Err(e) => return Err(unavailable(e)),
    }
    let gate = crate::account_gate::for_login(
        &state.store,
        &account_id,
        &username,
        &credential.login_name,
    )
    .await
    .map_err(unavailable)?;
    if gate.disabled {
        crate::account_gate::stop_engine_work(state, &account_id);
        refuse(state, parent, link, "accountDisabled").await;
        return Ok(None);
    }
    if gate.password_change_required {
        refuse(state, parent, link, "passwordChangeRequired").await;
        return Ok(None);
    }
    if crate::twofa_routes::factor_required(state, &account_id, &username)
        .await
        .map_err(unavailable)?
    {
        refuse(state, parent, link, "factorRequired").await;
        return Ok(None);
    }
    let mut creds = mw_store::Credentials {
        username: credential.login_name.clone(),
        password: (*credential.password).clone(),
    };
    let created = state
        .store
        .create_session(&account_id, &username, "engine", "engine", &creds)
        .await;
    creds.password.zeroize();
    let id = created.map_err(unavailable)?;
    let bound = async {
        state
            .store
            .set_session_authentication(&id, "password")
            .await
            .map_err(unavailable)?;
        crate::primary_identity::VerifiedLoginIdentity::engine(state, &account_id)
            .await
            .map_err(unavailable)?
            .bind(state, &id, &account_id)
            .await
            .map_err(unavailable)?;
        // The same bookkeeping as a normal sign-in: directory registration
        // (and its expiry), the per-account session cap, and the device record
        // the linked mailbox's owner sees in their sessions.
        if let Err(response) = crate::security_center::record_login(
            state,
            &id,
            &account_id,
            &username,
            &linked_device(),
            "password",
        )
        .await
        {
            return if response.status() == StatusCode::FORBIDDEN {
                Ok(false)
            } else {
                Err(response)
            };
        }
        let authority = state
            .store
            .session_storage_authority(&id)
            .await
            .map_err(unavailable)?;
        if authority.as_deref() != Some(credential.child_authority.as_str()) {
            return Ok(false);
        }
        state
            .store
            .rebind_linked_source(&parent.id, link, &id, true)
            .await
            .map_err(unavailable)
    }
    .await;
    match bound {
        Ok(true) => {}
        other => {
            let _ = state.store.delete_session(&id).await;
            if matches!(other, Ok(false)) {
                // Only an active credential is turned into "mailboxChanged";
                // one revoked meanwhile stays as the revoke left it.
                refuse(state, parent, link, "mailboxChanged").await;
                return Ok(None);
            }
            return other.map(|_| None);
        }
    }
    audit(
        state,
        "system",
        &parent.account_id,
        "linked-credential-renewed",
        json!({"link":link}),
    )
    .await;
    current(state, parent, link).await
}
