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
//! ## Every name of a principal
//!
//! A mail server may accept more than one login name for one mailbox (`alice` and
//! `alice@example.org`, or an alias), and may report yet another. The principal is
//! the account id; a flag set under any name that account id is known by applies
//! to every credential of it. The names are:
//!
//! * the account id itself,
//! * the engine account's `username`,
//! * every name recorded at a login whose credentials were accepted — the name
//!   typed and the name the upstream reported ([`remember_login`], kept in the
//!   store by `Store::remember_account_names`), and
//! * for a key or token, the two names on each live session of the account (which
//!   covers sessions opened before 26.20 started recording names).
//!
//! The record outlives the sessions, so a key or token of a disabled account is
//! refused by name after its sessions are gone.
//!
//! ## A session row that does not open
//!
//! A session row sealed under another server key (the server was started with a
//! different `MW_SERVER_KEY`, or with none) cannot be opened. It can never
//! authenticate again: `Store::get_session` fails for it, so its cookie is
//! refused. It is not an unreadable store and it does not refuse the account.
//! [`principal_names`] reads the rows with `Store::session_names_by_account`,
//! which reports such a row with the name stored in the clear and without the
//! name sealed in it.
//!
//! That sealed name — what was typed at the login that wrote the row — is still a
//! name of the principal when it is in the record of names, which is not sealed
//! and is written by every login since 26.20. A name held only in a row that
//! does not open is unknown here, like any name under "What this cannot see",
//! until a login uses it again.
//!
//! Until t30 the gate unsealed every session row of the account and treated one
//! that did not open as unreadable flags: every login to the account, and every
//! request of its keys and tokens, was answered 503 for as long as the row
//! existed.
//!
//! ## Other spellings of a flagged address
//!
//! A mail server can accept a spelling of an address that no login here has
//! used yet, and a login under it would otherwise be a new principal with no
//! flags (t32-h16): with a default realm it takes the bare `alice` for
//! `alice@example.com`, a server that strips tags takes `alice+x@example.com`,
//! and a domain can be written in Unicode or in its ASCII (`xn--`) form. So
//! beside the names themselves, each login name a principal is known by is also
//! read as:
//!
//! * the address with the local part's `+tag` removed (`alice+x@example.com` is
//!   gated under `alice@example.com` too);
//! * the address with its domain in ASCII form: a flag recorded under
//!   `alice@bücher.example` applies to `alice@xn--bcher-kva.example` and the
//!   other way round (the IDNA mapping is the one `reqwest::Url` applies to a
//!   host; no other dependency is used for it);
//! * for a bare name with no `@`, every flagged address with that local part:
//!   the bare `alice` is refused while any `alice@<domain>` is disabled.
//!
//! The last one fails closed. On a server with more than one domain, a bare
//! `alice` that the mail server maps to `alice@example.org` is refused because
//! `alice@example.com` is disabled, though they are two people. The way round it
//! is to sign in with the full address. An account id is not read these ways,
//! only a login name: a proxy-mode account id such as `b` is not the bare name
//! `b`.
//!
//! A hold on a password change (`force_password_change`) set under a flagged
//! address is applied through these spellings too. Changing the password clears
//! it under the session's own names, their `+tag`-less and ASCII-domain forms,
//! but not under an address found only from a bare name: that address may be
//! another person's, whose hold one person's password change must not lift. A
//! user held that way clears it by signing in with the full address, or the
//! admin clears the flag. `zero_access` and `remote_cache_wipe` are not read by
//! this gate.
//!
//! ## What this cannot see
//!
//! A name the mail server would accept for the mailbox, that nobody has logged
//! in with here and that is none of the spellings above (an alias, a tag
//! delimiter other than `+`), is unknown: a flag set under it applies from the
//! first login that uses it. Two upstream servers that hand out the same account
//! id are one principal to this server (as they are to its keys and second
//! factors). Disabling an account here does not disable the mailbox on the mail
//! server.
//!
//! ## Failing closed
//!
//! Every function returns `Err` when the store cannot be read, and every caller
//! treats `Err` as a refusal. A store outage fails logins and requests; it does not
//! let them through unchecked. That covers a database error on any read here —
//! the flags, the record of names, the session rows — and a flag or names record
//! that does not parse.

use axum::Json;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use serde_json::json;

use mw_admin::{ActorKind, AuditEvent, AuditKind, UserFeatureFlags};
use mw_store::{Session, Store, StoreError};

use crate::AppState;

/// The settings-key prefix the flags are stored under (no dedicated table).
const FLAGS_PREFIX: &str = "v6:admin:flags:";

/// The one normalisation of an account's name used by the flag writer and every
/// reader: surrounding whitespace removed, lowercased. The store folds the names
/// it records for an account the same way, so this is that function.
pub(crate) fn flag_subject(username: &str) -> String {
    Store::fold_account_name(username)
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

/// How many times [`update_flags`] re-reads and retries when another writer
/// changed the record between its read and its write.
const UPDATE_ATTEMPTS: usize = 16;

/// Change part of the flags stored for `account_id` without losing a concurrent
/// change to another part: `change` is applied to the record as stored, and the
/// result is written only if the record is still the one that was read
/// (`Store::compare_and_set_setting`); otherwise it is read again and `change`
/// re-applied. Returns the flags before and after.
///
/// [`write_flags`] replaces the whole record, which is what the panel's "save"
/// means. A writer that means one field — clearing `force_password_change` after
/// a password change — must use this instead: read-then-[`write_flags`] would put
/// back a stale `disabled: false` over an admin's `disabled: true` written in
/// between.
pub(crate) async fn update_flags(
    store: &Store,
    account_id: &str,
    change: impl Fn(&mut UserFeatureFlags),
) -> Result<(UserFeatureFlags, UserFeatureFlags), StoreError> {
    let subject = flag_subject(account_id);
    let key = flags_key(&subject);
    for _ in 0..UPDATE_ATTEMPTS {
        let stored = store.get_setting(&key).await?;
        let before = match &stored {
            Some(raw) => parse_flags(raw)?,
            // No record under the normalised key: start from a pre-26.20 record
            // under the exact spelling, if there is one.
            None => read_flags(store, account_id).await?,
        };
        let mut after = before;
        change(&mut after);
        if after == before {
            return Ok((before, after));
        }
        let json = serde_json::to_string(&after)
            .map_err(|e| StoreError::Corrupt(format!("account flags did not serialise: {e}")))?;
        if store
            .compare_and_set_setting(&key, stored.as_deref(), &json)
            .await?
        {
            if account_id != subject && store.get_setting(&flags_key(account_id)).await?.is_some() {
                store.set_setting(&flags_key(account_id), &json).await?;
            }
            return Ok((before, after));
        }
    }
    Err(StoreError::Corrupt(format!(
        "account flags changed under {UPDATE_ATTEMPTS} successive updates"
    )))
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
        if name.is_empty() { continue; }
        let s = flag_subject(name);
        if !s.is_empty() && !out.contains(&s) {
            out.push(s);
        }
    }
    out
}

/// `domain` in ASCII form (IDNA, as `reqwest::Url` maps a host), or `None` when
/// it is not a domain name. Lowercase.
fn ascii_domain(domain: &str) -> Option<String> {
    let bad = |c: char| c.is_whitespace() || "/?#@:[]\\%".contains(c);
    if domain.is_empty() || domain.contains(bad) {
        return None;
    }
    let url = reqwest::Url::parse(&format!("http://{domain}/")).ok()?;
    url.domain().map(str::to_ascii_lowercase)
}

/// Whether `domain` has a Unicode or an ASCII (`xn--`) form other than itself.
fn is_idn(domain: &str) -> bool {
    !domain.is_ascii() || domain.split('.').any(|label| label.starts_with("xn--"))
}

/// The local part of a folded subject without its `+tag`, if it has one.
fn untagged(local: &str) -> Option<&str> {
    match local.split_once('+') {
        Some((base, _)) if !base.is_empty() => Some(base),
        _ => None,
    }
}

/// The other spellings of the folded login names in `subjects` (module docs,
/// "Other spellings of a flagged address"): subjects to read beside them, and
/// local parts whose flagged addresses to scan, each with the ASCII form of the
/// domain an address found must have (`None`: any domain, for a bare name).
fn spellings(subjects: &[String]) -> (Vec<String>, Vec<(String, Option<String>)>) {
    let mut exact: Vec<String> = Vec::new();
    let mut scans: Vec<(String, Option<String>)> = Vec::new();
    for subject in subjects {
        let (local, domain) = match subject.rsplit_once('@') {
            Some((local, domain)) => (local, Some(domain)),
            None => (subject.as_str(), None),
        };
        if local.is_empty() {
            continue;
        }
        let at = domain.map(|d| format!("@{d}")).unwrap_or_default();
        if let Some(base) = untagged(local) {
            exact.push(format!("{base}{at}"));
        }
        // A domain with no other form needs no scan: the record under it is
        // read directly.
        let scan_domain = match domain {
            None => None,
            Some(d) if is_idn(d) => match ascii_domain(d) {
                Some(ascii) => Some(ascii),
                None => continue,
            },
            Some(_) => continue,
        };
        for l in std::iter::once(local).chain(untagged(local)) {
            let scan = (l.to_string(), scan_domain.clone());
            if !scans.contains(&scan) {
                scans.push(scan);
            }
        }
    }
    (exact, scans)
}

/// One flags record that applies to a principal.
struct Applies {
    /// The flag subject it is stored under.
    subject: String,
    flags: UserFeatureFlags,
    /// Stored under one of the principal's names or another spelling of the same
    /// address, not found only from a bare name (module docs).
    same_address: bool,
}

/// Every flags record that applies to a principal with the account ids `ids`,
/// known by the login names `names`: the record under each of them (absent ones
/// as all clear), and under each other spelling of a login name (module docs,
/// "Other spellings of a flagged address").
async fn applicable(
    store: &Store,
    ids: &[&str],
    names: &[&str],
) -> Result<Vec<Applies>, StoreError> {
    let mut out: Vec<Applies> = Vec::new();
    let all: Vec<&str> = ids.iter().chain(names).copied().collect();
    let (exact, scans) = spellings(&subjects(names));
    for subject in subjects(&all).into_iter().chain(exact) {
        if out.iter().any(|a| a.subject == subject) {
            continue;
        }
        let flags = read_flags(store, &subject).await?;
        out.push(Applies {
            subject,
            flags,
            same_address: true,
        });
    }
    for (local, domain) in scans {
        let prefix = flags_key(&format!("{local}@"));
        for (key, raw) in store.settings_with_prefix(&prefix).await? {
            let key = flag_subject(&key);
            let Some(found) = key.strip_prefix(&prefix) else {
                continue;
            };
            if let Some(ascii) = &domain
                && ascii_domain(found).as_ref() != Some(ascii)
            {
                continue;
            }
            let subject = format!("{local}@{found}");
            if let Some(seen) = out.iter_mut().find(|a| a.subject == subject) {
                seen.same_address |= domain.is_some();
                continue;
            }
            out.push(Applies {
                subject,
                flags: parse_flags(&raw)?,
                same_address: domain.is_some(),
            });
        }
    }
    Ok(out)
}

/// The gate for a principal with the account ids `ids`, known by the login names
/// `names`: a flag set under any one of them applies, and under another spelling
/// of a login name.
async fn gate_for(store: &Store, ids: &[&str], names: &[&str]) -> Result<AccountGate, StoreError> {
    let mut gate = AccountGate::default();
    let subject = mw_store::settings_policy::SettingSubject { domain: "", account_id: "", address: "" };
    let listed = crate::settings_registry::effective_for(store, "signin.domainMode", &subject).await?
        .is_some_and(|value| value.value == "listed");
    for name in names {
        let identity = crate::domain_config::canonical(store, name, None).await
            .map_err(StoreError::Corrupt)?;
        for address in [flag_subject(name), identity.address.clone()] {
            if store.directory_access_held(&address).await? {
                gate.disabled = true;
            }
        }
        match store.domain_signin_state(&identity.domain).await? {
            mw_store::DomainSigninState::Disabled => gate.disabled = true,
            mw_store::DomainSigninState::Unlisted if listed => gate.disabled = true,
            _ => {}
        }
    }
    for record in applicable(store, ids, names).await? {
        gate.disabled |= record.flags.disabled;
        gate.password_change_required |= record.flags.force_password_change;
    }
    Ok(gate)
}

/// Record the names a login was made under against its account id, so that a flag
/// set under either applies to every credential of the account from now on (see
/// the module docs). Call once the credentials have been accepted and before the
/// gate is read.
pub(crate) async fn remember_login(
    store: &Store,
    account_id: &str,
    username: &str,
    login_name: &str,
) -> Result<(), StoreError> {
    store
        .remember_account_names(account_id, &[username, login_name])
        .await
}

/// The gate for a login in progress: `account_id` is the principal, `username` the
/// name the session will carry (what the upstream reported, or the asserted
/// identity), `login_name` the name the credentials were presented under. A flag
/// under any name the account is known by applies, not only under these two.
pub(crate) async fn for_login(
    store: &Store,
    account_id: &str,
    username: &str,
    login_name: &str,
) -> Result<AccountGate, StoreError> {
    let mut names = principal_names(store, account_id).await?;
    names.push(username.to_string());
    names.push(login_name.to_string());
    gate_for_names(store, account_id, &names).await
}

/// The login names an established session is gated under: its own two and the
/// names recorded for its account. Its account id is gated beside them.
async fn session_names(store: &Store, session: &Session) -> Result<Vec<String>, StoreError> {
    let mut names = vec![
        session.username.clone(),
        session.credentials.username.clone(),
    ];
    names.extend(store.account_names(&session.account_id).await?);
    Ok(names)
}

/// The gate for an established session.
pub(crate) async fn for_session(
    store: &Store,
    session: &Session,
) -> Result<AccountGate, StoreError> {
    let names = session_names(store, session).await?;
    gate_for_names(store, &session.account_id, &names).await
}

/// Every login name the account `account_id` is known by (module docs, "Every
/// name of a principal"), for principals that carry only the id (an API key, an
/// OAuth token). The id itself is not among them: callers gate it as a subject
/// of its own, and it is not read as a login name. An empty id is no principal
/// and has no names (a proxy-mode upstream that names no mail account leaves
/// sessions with an empty account id, and those must not be read as one
/// account).
async fn principal_names(store: &Store, account_id: &str) -> Result<Vec<String>, StoreError> {
    let mut names = Vec::new();
    if account_id.is_empty() {
        return Ok(names);
    }
    match store.get_account(account_id).await {
        Ok(account) => names.push(account.username),
        Err(StoreError::NotFound) => {}
        Err(e) => return Err(e),
    }
    names.extend(store.account_names(account_id).await?);
    // The names, not the credentials: a row that does not open under this key
    // still gives the name stored in the clear, and is not an error (module
    // docs, "A session row that does not open").
    for session in store.session_names_by_account(account_id).await? {
        names.push(session.username);
        names.extend(session.login_name);
    }
    Ok(names)
}

/// The gate for the account `account_id` known by the login names `names`. The
/// id is read as a subject of its own, not as a login name (module docs).
async fn gate_for_names(
    store: &Store,
    account_id: &str,
    names: &[String],
) -> Result<AccountGate, StoreError> {
    let names: Vec<&str> = names.iter().map(String::as_str).collect();
    gate_for(store, &[account_id], &names).await
}

/// The gate for a key or token principal: a flag under any name the account is
/// known by applies. This does not depend on a live session, so a disabled
/// account's key is refused by name after its sessions have been deleted.
pub(crate) async fn for_account(
    store: &Store,
    account_id: &str,
) -> Result<AccountGate, StoreError> {
    let names = principal_names(store, account_id).await?;
    gate_for_names(store, account_id, &names).await
}

/// Stop the engine's work for a disabled account: drop its runtime, which ends its
/// watch loop and background sync. Proxy mode has no engine and nothing to stop.
///
/// Called where this module's callers refuse a disabled account; the panel's
/// flag write stops the same work when it sets the flag
/// (`stores_v6::AdminBackendAdapter::set_flags`). The account is connected again
/// by the first request after it is re-enabled (`engine_mode::ensure_account`)
/// or at the next start.
pub(crate) fn stop_engine_work(state: &AppState, account_id: &str) {
    if let Some(engine) = &state.engine
        && engine.unregister(account_id).is_some()
    {
        tracing::info!("account {account_id} is disabled; its engine runtime was dropped");
    }
}

/// Clear `force_password_change` for the account behind `session`, after it has
/// changed its password: under every name the session is gated under and every
/// other spelling of the same address, since the hold may come from any of them.
/// A record found only from a bare name is left alone: it may be another
/// person's (module docs, "Other spellings of a flagged address").
///
/// Only that one field is changed ([`update_flags`]); a `disabled` the admin set
/// while the password change was in flight is kept. Each record that was changed
/// gets the audit entry the panel's own "force password change" toggle writes,
/// with `system` as the actor.
pub(crate) async fn clear_password_change(
    state: &AppState,
    session: &Session,
) -> Result<(), mw_admin::AdminError> {
    let store_err = |e: StoreError| mw_admin::AdminError::Store(e.to_string());
    let names = session_names(&state.store, session)
        .await
        .map_err(store_err)?;
    let names: Vec<&str> = names.iter().map(String::as_str).collect();
    let records = applicable(&state.store, &[session.account_id.as_str()], &names)
        .await
        .map_err(store_err)?;
    for subject in records
        .into_iter()
        .filter(|r| r.same_address)
        .map(|r| r.subject)
    {
        let (before, _) = update_flags(&state.store, &subject, |f| {
            f.force_password_change = false;
        })
        .await
        .map_err(store_err)?;
        if before.force_password_change {
            state
                .v6
                .admin
                .record(
                    AuditEvent::new("system", ActorKind::Admin, AuditKind::ForcePasswordChange)
                        .detail(json!({ "enabled": false }))
                        .target(subject),
                )
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
            for_login(&s, "acct", "alice@example.org", "alice@example.org")
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

    fn creds(name: &str) -> Credentials {
        Credentials {
            username: name.into(),
            password: "p".into(),
        }
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
            &creds("erin@example.org"),
        )
        .await
        .unwrap();
        assert!(
            principal_names(&s, "upstream-17")
                .await
                .unwrap()
                .contains(&"erin@example.org".to_string())
        );
        assert!(!for_account(&s, "upstream-17").await.unwrap().disabled);
        write_flags(&s, "erin@example.org", disabled())
            .await
            .unwrap();
        assert!(for_account(&s, "upstream-17").await.unwrap().disabled);
        assert!(principal_names(&s, "nobody").await.unwrap().is_empty());
        // Sessions without an account id are not one principal.
        s.create_session("", "zed@example.org", "http://u", "http://u", &creds("zed"))
            .await
            .unwrap();
        assert!(principal_names(&s, "").await.unwrap().is_empty());
    }

    /// O1 (t27-f1): the admin flags the name the user typed; the upstream reports
    /// another. A key carries only the account id, and must be refused both while
    /// the session lives (through the name sealed in its credentials) and after
    /// it is gone (through the names recorded at login).
    #[tokio::test]
    async fn a_key_is_gated_under_the_name_typed_at_login() {
        let s = store().await;
        remember_login(&s, "upstream-9", "frank@example.org", "Frank")
            .await
            .unwrap();
        let id = s
            .create_session(
                "upstream-9",
                "frank@example.org",
                "http://u",
                "http://u",
                &creds("Frank"),
            )
            .await
            .unwrap();
        assert!(!for_account(&s, "upstream-9").await.unwrap().disabled);
        write_flags(&s, "frank", disabled()).await.unwrap();
        assert!(
            for_account(&s, "upstream-9").await.unwrap().disabled,
            "refused while the session lives"
        );
        s.delete_session(&id).await.unwrap();
        assert!(
            for_account(&s, "upstream-9").await.unwrap().disabled,
            "and after it is gone"
        );
        assert!(!for_account(&s, "upstream-10").await.unwrap().disabled);
    }

    /// Two stores over one database file that hold different keys: what the
    /// server sees after it was started with another `MW_SERVER_KEY`. Returns
    /// the store under the old key and the store under the new one.
    async fn rekeyed() -> (Store, Store) {
        static N: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
        let path = std::env::temp_dir().join(format!(
            "mw-t30-gate-{}-{}.sqlite",
            std::process::id(),
            N.fetch_add(1, std::sync::atomic::Ordering::SeqCst)
        ));
        let _ = std::fs::remove_file(&path);
        let path = path.to_string_lossy().into_owned();
        let old = Store::open(&path, ServerKey::generate()).await.unwrap();
        let new = Store::open(&path, ServerKey::generate()).await.unwrap();
        (old, new)
    }

    /// t30: a session row sealed under another key is not unreadable flags. The
    /// gate answers, and a flag under the name typed at the login that wrote the
    /// row still applies, through the record of names.
    #[tokio::test]
    async fn a_session_row_that_does_not_open_does_not_refuse_the_account() {
        let (old, new) = rekeyed().await;
        remember_login(&old, "upstream-20", "kim@example.org", "Kim")
            .await
            .unwrap();
        let id = old
            .create_session(
                "upstream-20",
                "kim@example.org",
                "http://u",
                "http://u",
                &creds("Kim"),
            )
            .await
            .unwrap();
        // The premise: the row is there and does not open under the new key.
        assert!(old.get_session(&id).await.is_ok());
        assert!(new.get_session(&id).await.is_err());

        let login = for_login(&new, "upstream-20", "kim@example.org", "kim@example.org").await;
        assert_eq!(
            login.expect("a row that does not open is not an unreadable store"),
            AccountGate::default()
        );
        assert_eq!(
            for_account(&new, "upstream-20").await.unwrap(),
            AccountGate::default()
        );

        // "Kim" was typed under the old key; the new key cannot read it from the
        // row, and reads it from the record.
        write_flags(&new, "kim", disabled()).await.unwrap();
        assert!(for_account(&new, "upstream-20").await.unwrap().disabled);
        assert!(
            for_login(&new, "upstream-20", "kim@example.org", "kim@example.org")
                .await
                .unwrap()
                .disabled
        );
        // Unreadable flags still refuse, with the row present.
        new.set_setting("v6:admin:flags:kim", "{not json")
            .await
            .unwrap();
        assert!(for_account(&new, "upstream-20").await.is_err());
        assert!(
            for_login(&new, "upstream-20", "kim@example.org", "kim@example.org")
                .await
                .is_err()
        );
    }

    /// The limit of the above: a session opened before names were recorded keeps
    /// the typed name only inside its sealed row. Under another key that name is
    /// unknown; the name stored in the clear on the same row is still read.
    #[tokio::test]
    async fn a_name_held_only_in_a_row_that_does_not_open_is_unknown() {
        let (old, new) = rekeyed().await;
        old.create_session(
            "upstream-21",
            "lee@example.org",
            "http://u",
            "http://u",
            &creds("lee-alias"),
        )
        .await
        .unwrap();
        write_flags(&new, "lee-alias", disabled()).await.unwrap();
        assert!(
            for_account(&old, "upstream-21").await.unwrap().disabled,
            "control: where the row opens, the sealed name gates the account"
        );
        assert!(
            !for_account(&new, "upstream-21").await.unwrap().disabled,
            "where it does not, the sealed name is not known"
        );
        write_flags(&new, "lee@example.org", disabled())
            .await
            .unwrap();
        assert!(
            for_account(&new, "upstream-21").await.unwrap().disabled,
            "the name in the clear on that row is read under any key"
        );
    }

    /// A session opened before names were recorded has only its own row to go by:
    /// the name sealed in a live session's credentials still gates a key.
    #[tokio::test]
    async fn a_key_is_gated_through_a_live_sessions_presented_name() {
        let s = store().await;
        s.create_session(
            "upstream-11",
            "gina@example.org",
            "http://u",
            "http://u",
            &creds("gina"),
        )
        .await
        .unwrap();
        write_flags(&s, "GINA", disabled()).await.unwrap();
        assert!(for_account(&s, "upstream-11").await.unwrap().disabled);
    }

    /// A flag under one name of a principal applies to a session and to a login
    /// made under another of its names, and to its account id.
    #[tokio::test]
    async fn every_credential_of_a_principal_is_gated_under_any_of_its_names() {
        let s = store().await;
        remember_login(&s, "upstream-12", "hal@example.org", "hal")
            .await
            .unwrap();
        // This session was opened under the reported name only.
        let other = Session {
            id: String::new(),
            account_id: "upstream-12".into(),
            username: "hal@example.org".into(),
            jmap_url: String::new(),
            api_url: String::new(),
            credentials: creds("hal@example.org"),
        };
        assert!(!for_session(&s, &other).await.unwrap().disabled);
        write_flags(&s, "hal", disabled()).await.unwrap();
        assert!(for_session(&s, &other).await.unwrap().disabled);
        assert!(
            for_login(&s, "upstream-12", "hal@example.org", "hal@example.org")
                .await
                .unwrap()
                .disabled
        );
        write_flags(&s, "hal", UserFeatureFlags::default())
            .await
            .unwrap();
        assert!(!for_session(&s, &other).await.unwrap().disabled);
        // The account id, in another case.
        write_flags(&s, "UPSTREAM-12", disabled()).await.unwrap();
        assert!(for_session(&s, &other).await.unwrap().disabled);
        assert!(for_account(&s, "upstream-12").await.unwrap().disabled);
    }

    /// O5 (t27-f1): an admin's `disabled: true` written between the read and the
    /// write of a one-field update is kept. `change` runs once per attempt, so the
    /// competing write is made from inside its first call.
    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn a_one_field_update_keeps_a_concurrent_change_to_another_field() {
        let s = store().await;
        write_flags(
            &s,
            "ivy@example.org",
            UserFeatureFlags {
                force_password_change: true,
                ..UserFeatureFlags::default()
            },
        )
        .await
        .unwrap();

        let calls = std::sync::atomic::AtomicUsize::new(0);
        let (before, after) = update_flags(&s, "Ivy@example.org", |f| {
            if calls.fetch_add(1, std::sync::atomic::Ordering::SeqCst) == 0 {
                // The admin disables the account after this update read the record.
                let s = s.clone();
                tokio::task::block_in_place(|| {
                    tokio::runtime::Handle::current().block_on(async {
                        write_flags(
                            &s,
                            "ivy@example.org",
                            UserFeatureFlags {
                                force_password_change: true,
                                disabled: true,
                                ..UserFeatureFlags::default()
                            },
                        )
                        .await
                        .unwrap();
                    });
                });
            }
            f.force_password_change = false;
        })
        .await
        .unwrap();

        assert_eq!(
            calls.load(std::sync::atomic::Ordering::SeqCst),
            2,
            "the first write was refused and the update re-read"
        );
        assert!(before.disabled && before.force_password_change);
        assert!(after.disabled && !after.force_password_change);
        let stored = read_flags(&s, "ivy@example.org").await.unwrap();
        assert!(stored.disabled, "the admin's disable survived");
        assert!(!stored.force_password_change);
    }

    /// An update that changes nothing writes nothing, and one on an account with
    /// no record creates it.
    #[tokio::test]
    async fn update_flags_on_absent_and_unchanged_records() {
        let s = store().await;
        let (before, after) = update_flags(&s, "jo@example.org", |f| {
            f.force_password_change = false;
        })
        .await
        .unwrap();
        assert_eq!(before, after);
        assert_eq!(
            s.get_setting("v6:admin:flags:jo@example.org")
                .await
                .unwrap(),
            None
        );
        update_flags(&s, "jo@example.org", |f| f.disabled = true)
            .await
            .unwrap();
        assert!(read_flags(&s, "JO@example.org").await.unwrap().disabled);
    }

    #[test]
    fn domains_fold_to_their_ascii_form() {
        assert_eq!(
            ascii_domain("bücher.example").as_deref(),
            Some("xn--bcher-kva.example")
        );
        assert_eq!(
            ascii_domain("XN--BCHER-KVA.example").as_deref(),
            Some("xn--bcher-kva.example")
        );
        assert_eq!(ascii_domain("example.com").as_deref(), Some("example.com"));
        for not_a_domain in ["", "a b", "x/y", "evil.example@other", "127.0.0.1", "[::1]"] {
            assert_eq!(ascii_domain(not_a_domain), None, "{not_a_domain}");
        }
        assert!(is_idn("bücher.example") && is_idn("xn--bcher-kva.example"));
        assert!(!is_idn("example.com"));
    }

    #[test]
    fn spellings_of_tagged_bare_and_idn_names() {
        let s = |v: &[&str]| v.iter().map(|x| x.to_string()).collect::<Vec<_>>();
        let (exact, scans) = spellings(&s(&["alice+x@example.com"]));
        assert_eq!(exact, s(&["alice@example.com"]));
        assert!(scans.is_empty(), "an ASCII domain needs no scan");

        let (exact, scans) = spellings(&s(&["alice+x"]));
        assert_eq!(exact, s(&["alice"]));
        assert_eq!(
            scans,
            vec![("alice+x".to_string(), None), ("alice".to_string(), None)]
        );

        let (exact, scans) = spellings(&s(&["carol@bücher.example"]));
        assert!(exact.is_empty());
        assert_eq!(
            scans,
            vec![(
                "carol".to_string(),
                Some("xn--bcher-kva.example".to_string())
            )]
        );

        // No base before the `+`: not a tag.
        let (exact, _) = spellings(&s(&["+x@example.com", "@example.com"]));
        assert!(exact.is_empty());
    }

    /// The bare name and a tag of a disabled address are refused; another
    /// person with the same local part on another domain is refused too through
    /// the bare name (it fails closed), but not through the full address.
    #[tokio::test]
    async fn bare_and_tagged_spellings_of_a_disabled_address_are_refused() {
        let s = store().await;
        write_flags(&s, "alice@example.com", disabled())
            .await
            .unwrap();
        for name in ["alice", " ALICE ", "alice+x@example.com", "alice+x"] {
            assert!(
                for_login(&s, "", name, name).await.unwrap().disabled,
                "{name}"
            );
        }
        for name in ["alice@example.org", "bob", "alic", "alice_", "ali%"] {
            assert!(
                !for_login(&s, "", name, name).await.unwrap().disabled,
                "{name}"
            );
        }
    }

    /// An account id is not a login name: a proxy-mode id `b` is not refused
    /// because `b@example.com` is disabled, but a login name `b` is.
    #[tokio::test]
    async fn an_account_id_is_not_read_as_a_bare_name() {
        let s = store().await;
        write_flags(&s, "b@example.com", disabled()).await.unwrap();
        assert!(!for_account(&s, "b").await.unwrap().disabled);
        assert!(
            !for_login(&s, "b", "carl@example.com", "carl")
                .await
                .unwrap()
                .disabled
        );
        remember_login(&s, "b", "b", "b").await.unwrap();
        assert!(for_account(&s, "b").await.unwrap().disabled);
    }

    /// Either form of the domain finds a record under the other.
    #[tokio::test]
    async fn the_other_form_of_a_domain_is_one_address() {
        let s = store().await;
        write_flags(&s, "carol@bücher.example", disabled())
            .await
            .unwrap();
        assert!(
            for_login(
                &s,
                "",
                "carol@xn--bcher-kva.example",
                "carol@XN--BCHER-KVA.example"
            )
            .await
            .unwrap()
            .disabled
        );
        write_flags(&s, "carol@bücher.example", UserFeatureFlags::default())
            .await
            .unwrap();
        write_flags(&s, "dan@xn--bcher-kva.example", disabled())
            .await
            .unwrap();
        assert!(
            for_login(&s, "", "dan@bücher.example", "dan+tag@BÜCHER.example")
                .await
                .unwrap()
                .disabled
        );
        assert!(
            !for_login(&s, "", "dan@other.example", "dan@other.example")
                .await
                .unwrap()
                .disabled
        );
    }

    /// A password change clears a hold found under another spelling of the same
    /// address, but not one found only from a bare name.
    #[tokio::test]
    async fn only_same_address_records_are_cleared() {
        let s = store().await;
        let held = UserFeatureFlags {
            force_password_change: true,
            ..UserFeatureFlags::default()
        };
        write_flags(&s, "erin@example.com", held).await.unwrap();
        write_flags(&s, "erin@bücher.example", held).await.unwrap();
        let records = applicable(&s, &["acct"], &["erin", "erin+x@xn--bcher-kva.example"])
            .await
            .unwrap();
        let held: Vec<(&str, bool)> = records
            .iter()
            .filter(|r| r.flags.force_password_change)
            .map(|r| (r.subject.as_str(), r.same_address))
            .collect();
        assert!(held.contains(&("erin@example.com", false)), "{held:?}");
        assert!(held.contains(&("erin@bücher.example", true)), "{held:?}");
    }

    /// A corrupt record found by a scan is an error, like any other.
    #[tokio::test]
    async fn a_corrupt_record_found_by_a_scan_refuses() {
        let s = store().await;
        s.set_setting("v6:admin:flags:fay@example.com", "{not json")
            .await
            .unwrap();
        assert!(for_login(&s, "", "fay", "fay").await.is_err());
        assert!(
            for_login(&s, "", "fay@example.org", "fay@example.org")
                .await
                .is_ok()
        );
    }
}
