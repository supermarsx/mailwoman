//! The reader for the admin panel's per-account flags (t27-e3, audit OH-1).
//!
//! The panel writes two flags that decide whether an account may be used:
//! `disabled` and `force_password_change` ([`mw_admin::UserFeatureFlags`]). This
//! module is where they are read, and every path that establishes or uses a mailbox
//! principal goes through it:
//!
//! | path | call site |
//! |---|---|
//! | password / header-auth login, both modes | `twofa_routes::gate_login` |
//! | second-factor completion, forced enrolment | `twofa_routes::complete_login` |
//! | SSO callback | `sso::complete_flow` |
//! | cookie session, native bearer (every `authed` caller) | `crate::authed_with_gate` |
//! | API key, OAuth access / refresh token, authorization code | `stores_v6::OAuthStoreAdapter` |
//! | API key on `/api/v1`, key or token on `/mcp` (the hold) | `scope_mw` guards |
//!
//! ## One identifier
//!
//! The panel names an account `username@domain`. A session's `account_id` is a
//! different thing — a store-generated id in engine mode, the upstream's account id
//! in proxy mode. The identifier the two have in common is the **login name**, so
//! flags are stored and looked up under [`flag_subject`] of it: trimmed and
//! lowercased, the same on the writing and the reading side.
//!
//! ## What this cannot see
//!
//! A mail server may accept more than one login name for one mailbox (`alice` and
//! `alice@example.org`, or an alias). Mailwoman cannot know two names are one
//! mailbox, so a flag set for one does not apply to the other; the name typed at
//! login and the name the upstream reports are both checked, which covers the case
//! where they differ within one login. Disabling an account here also does not
//! disable the mailbox on the mail server.
//!
//! ## Failing closed
//!
//! Every function returns `Err` when the store cannot be read, and every caller
//! treats `Err` as a refusal. A store outage fails logins and requests; it does not
//! let them through unchecked.

use axum::Json;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use serde_json::json;

use mw_admin::UserFeatureFlags;
use mw_store::{Session, Store, StoreError};

use crate::AppState;

/// The settings-key prefix the flags are stored under (no dedicated table).
const FLAGS_PREFIX: &str = "v6:admin:flags:";

/// The one normalisation of an account's name used by the flag writer and every
/// reader: surrounding whitespace removed, lowercased.
pub(crate) fn flag_subject(username: &str) -> String {
    username.trim().to_lowercase()
}

fn flags_key(id: &str) -> String {
    format!("{FLAGS_PREFIX}{id}")
}

fn parse_flags(raw: &str) -> Result<UserFeatureFlags, StoreError> {
    serde_json::from_str(raw)
        .map_err(|e| StoreError::Corrupt(format!("account flags are not valid JSON: {e}")))
}

/// The flags stored for `account_id` (the panel's `username@domain`, or a login
/// name). Absent ⇒ all clear.
///
/// Before 26.20 the key was the id exactly as the panel sent it. A record written
/// then under a mixed-case id is still found when the caller presents that same
/// spelling: the normalised key is read first, the exact spelling second.
pub(crate) async fn read_flags(
    store: &Store,
    account_id: &str,
) -> Result<UserFeatureFlags, StoreError> {
    let subject = flag_subject(account_id);
    if let Some(raw) = store.get_setting(&flags_key(&subject)).await? {
        return parse_flags(&raw);
    }
    if account_id != subject
        && let Some(raw) = store.get_setting(&flags_key(account_id)).await?
    {
        return parse_flags(&raw);
    }
    Ok(UserFeatureFlags::default())
}

/// Store `flags` for `account_id` under its normalised key. A pre-26.20 record
/// under the exact spelling is overwritten with the same value so the two cannot
/// disagree.
pub(crate) async fn write_flags(
    store: &Store,
    account_id: &str,
    flags: UserFeatureFlags,
) -> Result<(), StoreError> {
    let json = serde_json::to_string(&flags)
        .map_err(|e| StoreError::Corrupt(format!("account flags did not serialise: {e}")))?;
    let subject = flag_subject(account_id);
    store.set_setting(&flags_key(&subject), &json).await?;
    if account_id != subject && store.get_setting(&flags_key(account_id)).await?.is_some() {
        store.set_setting(&flags_key(account_id), &json).await?;
    }
    Ok(())
}

/// What the flags mean for one principal.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub(crate) struct AccountGate {
    /// The account is administratively disabled: no login, no session, no key, no
    /// token.
    pub disabled: bool,
    /// The account must change its password before it may do anything else.
    pub password_change_required: bool,
}

/// The distinct, non-empty flag subjects among `names`.
fn subjects(names: &[&str]) -> Vec<String> {
    let mut out: Vec<String> = Vec::with_capacity(names.len());
    for name in names {
        let s = flag_subject(name);
        if !s.is_empty() && !out.contains(&s) {
            out.push(s);
        }
    }
    out
}

/// The gate for a principal known by any of `names`: a flag set under any one of
/// them applies.
async fn gate_for(store: &Store, names: &[&str]) -> Result<AccountGate, StoreError> {
    let mut gate = AccountGate::default();
    for subject in subjects(names) {
        let flags = read_flags(store, &subject).await?;
        gate.disabled |= flags.disabled;
        gate.password_change_required |= flags.force_password_change;
    }
    Ok(gate)
}

/// The gate for a login in progress: `username` is the name the session will carry
/// (what the upstream reported, or the asserted identity), `login_name` the name
/// the credentials were presented under.
pub(crate) async fn for_login(
    store: &Store,
    username: &str,
    login_name: &str,
) -> Result<AccountGate, StoreError> {
    gate_for(store, &[username, login_name]).await
}

/// The gate for an established session.
pub(crate) async fn for_session(
    store: &Store,
    session: &Session,
) -> Result<AccountGate, StoreError> {
    gate_for(store, &[&session.username, &session.credentials.username]).await
}

/// The login name behind an account id, for principals that carry only the id (an
/// API key, an OAuth token): the engine account's `username`, else the name on a
/// live session for that account. `None` when neither exists.
pub(crate) async fn username_for_account(
    store: &Store,
    account_id: &str,
) -> Result<Option<String>, StoreError> {
    match store.get_account(account_id).await {
        Ok(account) => return Ok(Some(account.username)),
        Err(StoreError::NotFound) => {}
        Err(e) => return Err(e),
    }
    Ok(store
        .sessions_by_account(account_id)
        .await?
        .into_iter()
        .next()
        .map(|s| s.username))
}

/// The gate for a key or token principal.
///
/// The account id itself is also tried as a name: under header-auth the two are the
/// same string. In proxy mode an account id maps to a login name only through a
/// live session, so once a disabled account's sessions are deleted its keys resolve
/// to no name and are not refused here — they have no upstream credentials left to
/// read with (`scope_mw::rest_session`).
pub(crate) async fn for_account(
    store: &Store,
    account_id: &str,
) -> Result<AccountGate, StoreError> {
    let username = username_for_account(store, account_id).await?;
    gate_for(store, &[username.as_deref().unwrap_or(""), account_id]).await
}

/// Clear `force_password_change` for the account behind `session`, after it has
/// changed its password. Goes through [`mw_admin::Admin`] so the change is audited
/// like the panel's own.
pub(crate) async fn clear_password_change(
    state: &AppState,
    session: &Session,
) -> Result<(), mw_admin::AdminError> {
    for subject in subjects(&[&session.username, &session.credentials.username]) {
        let flagged = state
            .v6
            .admin
            .get_feature_flags(&subject)
            .await?
            .force_password_change;
        if flagged {
            state
                .v6
                .admin
                .force_password_change("system", &subject, false)
                .await?;
        }
    }
    Ok(())
}

/// The refusal a held account gets on everything but the password-change routes.
/// The web client keys on the body field, not the status.
pub(crate) fn password_change_required() -> Response {
    (
        StatusCode::FORBIDDEN,
        Json(json!({ "error": "password change required", "passwordChangeRequired": true })),
    )
        .into_response()
}

/// The refusal when the flags could not be read. Not a 401: the credential may be
/// good, and a client should retry rather than discard its session.
pub(crate) fn unavailable(what: &str, e: impl std::fmt::Display) -> Response {
    tracing::error!("account flags unreadable ({what}), refusing: {e}");
    (
        StatusCode::SERVICE_UNAVAILABLE,
        Json(json!({ "error": "server error" })),
    )
        .into_response()
}

#[cfg(test)]
mod tests {
    use super::*;
    use mw_store::{Credentials, ServerKey};

    async fn store() -> Store {
        Store::open_in_memory(ServerKey::generate()).await.unwrap()
    }

    fn disabled() -> UserFeatureFlags {
        UserFeatureFlags {
            disabled: true,
            ..UserFeatureFlags::default()
        }
    }

    #[test]
    fn subject_is_trimmed_and_lowercased() {
        assert_eq!(flag_subject("  Alice@Example.ORG "), "alice@example.org");
        assert_eq!(
            subjects(&["Alice@x", "alice@X", "", " ", "bob"]),
            vec!["alice@x".to_string(), "bob".to_string()]
        );
    }

    #[tokio::test]
    async fn writer_and_reader_agree_across_case() {
        let s = store().await;
        assert!(!read_flags(&s, "alice@example.org").await.unwrap().disabled);
        write_flags(&s, "Alice@Example.org", disabled())
            .await
            .unwrap();
        assert!(read_flags(&s, "ALICE@example.org").await.unwrap().disabled);
        assert!(
            for_login(&s, "alice@example.org", "alice@example.org")
                .await
                .unwrap()
                .disabled
        );
        assert!(!read_flags(&s, "bob@example.org").await.unwrap().disabled);
    }

    /// A record written before 26.20 sits under the id exactly as the panel sent
    /// it. It is read under that spelling, and the next write replaces it.
    #[tokio::test]
    async fn a_pre_normalisation_record_is_read_and_then_superseded() {
        let s = store().await;
        s.set_setting(
            "v6:admin:flags:Carol@Example.org",
            &serde_json::to_string(&disabled()).unwrap(),
        )
        .await
        .unwrap();
        assert!(read_flags(&s, "Carol@Example.org").await.unwrap().disabled);

        write_flags(&s, "Carol@Example.org", UserFeatureFlags::default())
            .await
            .unwrap();
        assert!(!read_flags(&s, "Carol@Example.org").await.unwrap().disabled);
        assert!(!read_flags(&s, "carol@example.org").await.unwrap().disabled);
    }

    /// A flag under either the session's name or the name the credentials were
    /// presented under applies to the session.
    #[tokio::test]
    async fn a_session_is_gated_under_either_of_its_names() {
        let s = store().await;
        let session = |username: &str, login: &str| Session {
            id: String::new(),
            account_id: "acct".into(),
            username: username.into(),
            jmap_url: String::new(),
            api_url: String::new(),
            credentials: Credentials {
                username: login.into(),
                password: String::new(),
            },
        };
        let sess = session("alice@example.org", "alice");
        assert_eq!(
            for_session(&s, &sess).await.unwrap(),
            AccountGate::default()
        );
        write_flags(&s, "alice", disabled()).await.unwrap();
        assert!(for_session(&s, &sess).await.unwrap().disabled);
        assert!(
            !for_session(&s, &session("bob@example.org", "bob"))
                .await
                .unwrap()
                .disabled
        );
    }

    /// Unparseable flags are an error, which callers treat as a refusal.
    #[tokio::test]
    async fn corrupt_flags_are_an_error_not_all_clear() {
        let s = store().await;
        s.set_setting("v6:admin:flags:dave@example.org", "{not json")
            .await
            .unwrap();
        assert!(read_flags(&s, "dave@example.org").await.is_err());
        assert!(for_account(&s, "dave@example.org").await.is_err());
    }

    /// A key principal carries an account id; it is gated through the login name
    /// of a live session for that account.
    #[tokio::test]
    async fn an_account_id_is_gated_through_its_sessions_login_name() {
        let s = store().await;
        s.create_session(
            "upstream-17",
            "erin@example.org",
            "http://u",
            "http://u",
            &Credentials {
                username: "erin@example.org".into(),
                password: "p".into(),
            },
        )
        .await
        .unwrap();
        assert_eq!(
            username_for_account(&s, "upstream-17").await.unwrap(),
            Some("erin@example.org".to_string())
        );
        assert!(!for_account(&s, "upstream-17").await.unwrap().disabled);
        write_flags(&s, "erin@example.org", disabled())
            .await
            .unwrap();
        assert!(for_account(&s, "upstream-17").await.unwrap().disabled);
        assert_eq!(username_for_account(&s, "nobody").await.unwrap(), None);
    }
}
