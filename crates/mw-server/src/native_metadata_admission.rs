//! Narrow, fresh native Email/get integration. No source/allow memo is kept.
//! Unsupported or cold inputs return to the complete original path BEFORE
//! taking permits. After that boundary failures must authenticate again and
//! refuse, never dispatch using a preparation-time observation.
use super::{
    DeferredJmapFeatures, Needs, check_feature_values, check_protocol_kind, check_required_values,
};
use crate::{
    AppState, account_gate, domain_config, engine_mode, rate_leeway, settings_registry as registry,
};
use axum::{
    Json,
    http::{HeaderMap, header},
    response::{IntoResponse, Response},
};
use mw_store::{
    Session,
    native_admission_snapshot::{NativeAdmissionRequest, NativeAdmissionSnapshot},
    settings_policy::{SettingSubject, SettingsReadRevision},
};
use serde_json::Value;

type Unsupported = ();
fn unsupported<T>(value: Result<T, impl Sized>) -> Result<T, Unsupported> {
    value.map_err(|_| ())
}

fn eligible_request(body: &[u8]) -> Option<Value> {
    if body.len() > mw_engine::jmap::core::MAX_SIZE_REQUEST {
        return None;
    }
    let request: Value = serde_json::from_slice(body).ok()?;
    mw_engine::jmap::core::validate(&request, body.len(), true).ok()?;
    let calls = request.get("methodCalls")?.as_array()?;
    if calls.len() != 1 {
        return None;
    }
    let call = &calls[0];
    if call.get(0)?.as_str()? != "Email/get" {
        return None;
    }
    let args = call.get(1)?;
    let ids = args.get("ids")?.as_array()?;
    let properties = args.get("properties")?.as_array()?;
    if ids.is_empty()
        || ids.len() > 50
        || properties.len() > 64
        || ids.iter().any(|id| {
            !id.as_str()
                .is_some_and(|id| !id.is_empty() && id.len() <= 1024)
        })
        || properties.iter().any(|property| {
            !property
                .as_str()
                .is_some_and(|property| property.len() <= 128)
        })
        || !mw_engine::jmap::metadata_envelope_only(args)
        || args.as_object()?.keys().any(|key| key.starts_with('#'))
    {
        return None;
    }
    Some(request)
}

fn warm_values(
    state: &AppState,
    subject: &SettingSubject<'_>,
    revision: &SettingsReadRevision,
    filter: impl Fn(&registry::SettingDef) -> bool,
) -> Result<Vec<registry::Effective>, Unsupported> {
    let inputs = unsupported(
        state
            .store
            .try_settings_resolution_inputs_at(subject, revision),
    )?
    .ok_or(())?;
    Ok(registry::effective_all_from_inputs(inputs, filter, &[]))
}
fn value<'a>(values: &'a [registry::Effective], key: &str) -> Option<&'a Value> {
    values
        .iter()
        .find(|value| value.key == key)
        .map(|value| &value.value)
}
fn listed(
    state: &AppState,
    subject: &SettingSubject<'_>,
    revision: &SettingsReadRevision,
) -> Result<bool, Unsupported> {
    Ok(value(
        &warm_values(state, subject, revision, |def| {
            def.key == "signin.domainMode"
        })?,
        "signin.domainMode",
    )
    .is_some_and(|value| value == "listed"))
}

struct Loaded {
    session: Session,
    knobs: rate_leeway::Knobs,
    authority: String,
}
impl Loaded {
    fn target_matches(&self, headers: &HeaderMap) -> bool {
        crate::mailbox_target_matches(&self.session, headers, Some(&self.authority))
    }
}

fn names_for(
    snapshot: &NativeAdmissionSnapshot,
    state: &AppState,
    session: &Session,
) -> Result<(Vec<String>, Vec<String>), Unsupported> {
    let mut session_names = vec![
        session.username.clone(),
        session.credentials.username.clone(),
    ];
    session_names.extend(unsupported(snapshot.names())?);
    let account = unsupported(snapshot.account())?.ok_or(())?;
    let mut account_names = vec![account.username];
    account_names.extend(unsupported(snapshot.names())?);
    for source in unsupported(snapshot.account_session_names(&state.store))?.ok_or(())? {
        account_names.push(source.username);
        account_names.extend(source.login_name);
    }
    if session_names.len() > 256 || account_names.len() > 256 {
        return Err(());
    }
    for names in [&mut session_names, &mut account_names] {
        let mut seen = std::collections::HashSet::new();
        names.retain(|name| seen.insert(name.clone()));
    }
    Ok((session_names, account_names))
}
fn prepared(
    account: &str,
    names: &[String],
) -> Result<account_gate::final_snapshot::Prepared, Unsupported> {
    let names = names.iter().map(String::as_str).collect::<Vec<_>>();
    account_gate::final_snapshot::Prepared::new(&[account], &names).ok_or(())
}
fn shaping(account: &str, names: &[String]) -> Result<NativeAdmissionRequest, Unsupported> {
    let prepared = prepared(account, names)?;
    unsupported(NativeAdmissionRequest::new(
        prepared.domains().to_vec(),
        prepared.keys().to_vec(),
    ))
}

fn evaluate(
    state: &AppState,
    snapshot: &NativeAdmissionSnapshot,
    needs: &Needs,
) -> Result<Loaded, Unsupported> {
    if !snapshot.bounded_shape()
        || needs.native
        || needs.link_limit
        || needs.upload_bytes.is_some()
        || needs.recipients != 0
        || needs.avatars.is_some()
    {
        return Err(());
    }
    let engine = state.engine.as_ref().ok_or(())?;
    let source = unsupported(snapshot.session(&state.store))?.ok_or(())?;
    let session = source.session;
    if session.jmap_url != "engine"
        || session.api_url != "engine"
        || session.id.is_empty()
        || state
            .sessions
            .expired(&session.id, &state.hardening)
            .is_some()
        || !crate::request_protection::native_account_already_admitted(&session.account_id)
        || state.store.session_touch_may_be_due_at(
            source.last_seen.as_deref(),
            crate::session_persistence::flush_seconds(),
            chrono::Utc::now(),
        )
        || unsupported(snapshot.linked_policy_required())?
    {
        return Err(());
    }
    let revision = unsupported(snapshot.revision(&state.store))?;
    if revision.number().is_none() {
        return Err(());
    }
    // Require an active, native, same-owner principal before selecting any
    // personal namespace. No caller-supplied account selects that namespace.
    let subjects = unsupported(snapshot.native_subjects(&state.store))?.ok_or(())?;
    let account = unsupported(snapshot.account())?.ok_or(())?;
    if account.id != session.account_id || account.kind == mw_store::AccountKind::Ews {
        return Err(());
    }
    let raw_name = session.username.trim().to_ascii_lowercase();
    let (_, raw_domain) = raw_name.rsplit_once('@').ok_or(())?;
    let feature_subject = SettingSubject {
        personal_owner: None,
        domain: raw_domain,
        account_id: &session.account_id,
        address: &raw_name,
    };
    let mut features = needs.features.clone();
    if needs.native {
        features.insert("apps.native");
    }
    let mut keys = features
        .iter()
        .filter_map(|id| crate::features::feature(id))
        .flat_map(|feature| feature.capability_keys.iter().copied())
        .collect::<Vec<_>>();
    // Recheck protocol even if the earlier presence optimization saw no policy.
    keys.push("access.protocols");
    keys.extend(needs.required.iter().copied());
    let values = warm_values(state, &feature_subject, &revision, |def| {
        keys.contains(&def.key)
    })?;
    unsupported(check_feature_values(&features, &values))?;
    if let Some(allowed) = value(&values, "access.protocols").and_then(Value::as_array) {
        let kind = match account.kind {
            mw_store::AccountKind::Imap => "imap",
            mw_store::AccountKind::Pop3 => "pop3",
            mw_store::AccountKind::Ews => return Err(()),
        };
        unsupported(check_protocol_kind(allowed, Some(kind)))?;
    }
    unsupported(check_required_values(&needs.required, &values))?;
    let clock_subject = SettingSubject {
        personal_owner: None,
        domain: session
            .username
            .rsplit_once('@')
            .map_or("", |(_, domain)| domain),
        account_id: &session.account_id,
        address: &session.username,
    };
    let clock_values = warm_values(state, &clock_subject, &revision, |def| {
        matches!(def.key, "session.idleSecs" | "session.absoluteSecs")
    })?;
    let clock = crate::security_center::clock_policy_from_values(
        &clock_values,
        state.hardening.idle_timeout.as_secs(),
        state.hardening.absolute_timeout.as_secs(),
    );
    let (created, activity) = unsupported(snapshot.clocks())?.ok_or(())?;
    if crate::security_center::session_deadline_passed(
        &created,
        &activity,
        chrono::Utc::now(),
        clock.idle_secs,
        clock.absolute_secs,
    ) {
        return Err(());
    }
    let (session_names, account_names) = names_for(snapshot, state, &session)?;
    let global_listed = listed(state, &SettingSubject::default(), &revision)?;
    for names in [&session_names, &account_names] {
        let gate = unsupported(prepared(&session.account_id, names)?.from_native(
            &state.store,
            snapshot,
            global_listed,
        ))?
        .ok_or(())?;
        if gate.disabled || gate.password_change_required {
            return Err(());
        }
    }
    // Qualified, non-chained identity reuse only. Both the sign-in resolver and
    // connector use this exact same current domain configuration below.
    if session.credentials.username.trim() != account.username.trim() {
        return Err(());
    }
    let input = unsupported(domain_config::QualifiedIdentityInput::parse(
        &session.credentials.username,
    ))?
    .ok_or(())?;
    let mut domains = snapshot.domains();
    let row = unsupported(domains.remove(&input.domain).ok_or(())?)?;
    if row.second_alias {
        return Err(());
    }
    let identity = domain_config::IdentitySnapshot::from_qualified(
        input,
        row.canonical,
        row.configuration,
        row.second_alias,
    );
    if !identity.matches_existing_identity(&account.username)
        || !unsupported(identity.signin_allowed())?
    {
        return Err(());
    }
    let domain_subject = SettingSubject {
        personal_owner: None,
        domain: &identity.identity.domain,
        account_id: "",
        address: &identity.identity.address,
    };
    let domain_listed = listed(state, &domain_subject, &revision)?;
    let (upstream, config) = match unsupported(crate::upstream_resolve::resolve_configured(
        &identity,
        true,
        domain_listed,
    ))? {
        crate::upstream_resolve::Configured::Resolved(
            crate::upstream_resolve::Upstream::Mail(url),
            Some(config),
        ) => (url, config),
        _ => return Err(()),
    };
    if !unsupported(engine_mode::account_matches_mail_configuration(
        &account,
        &upstream,
        Some(&config),
    ))? {
        return Err(());
    }
    if engine_mode::stored_credentials_rejected(&account) {
        return Err(());
    }
    let credentials = unsupported(snapshot.account_credentials(&state.store))?.ok_or(())?;
    if engine.credential_revision_rejected(&account.id, credentials.revision()) {
        return Err(());
    }
    unsupported(engine_mode::checked_account_policy(&account, Some(&config)))?;
    if config.selection.is_some() && config.mail().is_none() || !engine.is_registered(&account.id) {
        return Err(());
    }
    let domain = crate::rate_leeway::domain_of(&session.username);
    let rate_subject = SettingSubject {
        domain,
        ..Default::default()
    };
    let rate_values = warm_values(state, &rate_subject, &revision, |def| {
        def.key.starts_with("ratelimit.")
    })?;
    Ok(Loaded {
        session,
        knobs: rate_leeway::Knobs::from_rows(&rate_values),
        authority: subjects.storage_authority,
    })
}

async fn observe(
    state: &AppState,
    id: &str,
    query: &NativeAdmissionRequest,
) -> Result<NativeAdmissionSnapshot, mw_store::StoreError> {
    let result = state
        .store
        .native_cookie_admission_snapshot(id, query)
        .await;
    #[cfg(test)]
    if result.is_ok() {
        test_control::record_observation();
    }
    result
}

async fn late_refusal(state: &AppState, headers: &HeaderMap) -> Response {
    // Preserve ordinary revocation/expiry/error handling. A successful fresh
    // chain still cannot revive the earlier prepared dispatch after a race.
    match crate::authed(state, headers).await {
        Err(response) => response,
        Ok(_) => super::auth_boundary::auth_fence_refusal(),
    }
}

pub(crate) async fn try_dispatch(
    state: &AppState,
    headers: &HeaderMap,
    body: &[u8],
    deferred: &DeferredJmapFeatures,
) -> Option<Response> {
    #[cfg(test)]
    if test_control::force_original() {
        return None;
    }
    if state.engine.is_none()
        || headers.contains_key(header::AUTHORIZATION)
        || headers.contains_key(crate::linked_accounts::SELECTOR)
    {
        return None;
    }
    let request = eligible_request(body)?;
    let id = crate::cookie_value(headers)?;
    let hint = state
        .store
        .primary_session_policy_snapshot(&id)
        .await
        .ok()?;
    #[cfg(test)]
    test_control::record_hint();
    if !hint.witness.matches_revision(&deferred.1) {
        return None;
    }
    let mut names = vec![
        hint.session.username.clone(),
        hint.session.credentials.username.clone(),
    ];
    let mut query = shaping(&hint.session.account_id, &names).ok()?;
    let mut snapshot = observe(state, &id, &query).await.ok()?;
    // At most one additional shaping read for current account/sibling names.
    let source = snapshot.session(&state.store).ok()??;
    let (session_names, account_names) = names_for(&snapshot, state, &source.session).ok()?;
    names = session_names.into_iter().chain(account_names).collect();
    let current = shaping(&source.session.account_id, &names).ok()?;
    if !snapshot.covers_domains(current.domains()) || !snapshot.covers_keys(current.keys()) {
        query = current;
        snapshot = observe(state, &id, &query).await.ok()?;
    }
    let first = evaluate(state, &snapshot, &deferred.0).ok()?;
    if request["methodCalls"][0][1]["accountId"].as_str() != Some(first.session.account_id.as_str())
        || !first.target_matches(headers)
    {
        return None;
    }
    #[cfg(test)]
    test_control::wait(test_control::Stage::AfterPrepared, &id).await;
    let permit = match crate::jmap_limits::request_with_knobs(
        &first.session.account_id,
        first.knobs,
    )
    .await
    {
        Ok(permit) => permit,
        Err(response) => return Some(response),
    };
    // Keep the original permit -> connector mutex ordering. Helpers run BEFORE
    // the final read, so any newly introduced await cannot stale the last fence.
    if let Err(response) = crate::feature_jmap::admit(state, &first.session, &request).await {
        return Some(response);
    }
    let mut checked = request.clone();
    if let Err(response) = crate::message_headers::apply(state, &first.session, &mut checked).await
    {
        return Some(response);
    }
    if let Err(response) = crate::delete_policy::admit(state, &first.session, &mut checked).await {
        return Some(response);
    }
    #[cfg(test)]
    test_control::wait(test_control::Stage::BeforeConnector, &id).await;
    let lock = engine_mode::connect_lock(&first.session.account_id);
    let connecting = lock.lock().await;
    let final_snapshot = match observe(state, &id, &query).await {
        Ok(snapshot) => snapshot,
        Err(_) => {
            drop(connecting);
            drop(permit);
            return Some(late_refusal(state, headers).await);
        }
    };
    let final_source = evaluate(state, &final_snapshot, &deferred.0).ok();
    let Some(final_source) = final_source.filter(|source| {
        super::auth_boundary::same_session(&first.session, &source.session)
            && first.knobs == source.knobs
            && source.target_matches(headers)
    }) else {
        drop(connecting);
        drop(permit);
        return Some(late_refusal(state, headers).await);
    };
    if state.sessions.check(&id, &state.hardening).is_err() {
        drop(connecting);
        drop(permit);
        // check() removes an expired process clock. Do not run it again via
        // authed(), which would treat the removed entry as a new sighting.
        // Preserve the ordinary cookie path's cleanup and expiry response.
        let _ = state.store.delete_session(&id).await;
        state.sessions.forget(&id);
        return Some(crate::session_expired());
    }
    drop(connecting);
    // No further asynchronous policy/credential lookup occurs before dispatch.
    // Engine retains its own fresh ownership and posture checks for data reads.
    #[cfg(test)]
    test_control::record_dispatch();
    let response = state
        .engine
        .as_ref()?
        .handle_jmap_with_sieve(&final_source.session.account_id, &request, None)
        .await;
    drop(permit);
    Some(Json(response).into_response())
}

#[cfg(test)]
mod test_control {
    use std::{
        cell::RefCell,
        collections::HashMap,
        sync::{Arc, LazyLock, Mutex},
    };
    #[derive(Default, Clone, Copy, Debug)]
    pub(super) struct Counts {
        pub hints: u64,
        pub observations: u64,
        pub dispatches: u64,
        pub original: bool,
    }
    tokio::task_local! { static COUNTS: RefCell<Counts>; }
    pub(super) fn force_original() -> bool {
        COUNTS
            .try_with(|counts| counts.borrow().original)
            .unwrap_or(false)
    }
    pub(super) fn record_hint() {
        let _ = COUNTS.try_with(|counts| counts.borrow_mut().hints += 1);
    }
    pub(super) fn record_observation() {
        let _ = COUNTS.try_with(|counts| counts.borrow_mut().observations += 1);
    }
    pub(super) fn record_dispatch() {
        let _ = COUNTS.try_with(|counts| counts.borrow_mut().dispatches += 1);
    }
    pub(super) async fn count<F: std::future::Future>(
        original: bool,
        future: F,
    ) -> (F::Output, Counts) {
        COUNTS
            .scope(
                RefCell::new(Counts {
                    original,
                    ..Default::default()
                }),
                async {
                    let value = future.await;
                    (value, COUNTS.with(|counts| *counts.borrow()))
                },
            )
            .await
    }
    #[derive(Clone, Copy, PartialEq, Eq, Hash)]
    pub(super) enum Stage {
        AfterPrepared,
        BeforeConnector,
    }
    pub(super) struct Pause {
        pub entered: tokio::sync::Notify,
        pub release: tokio::sync::Notify,
    }
    static PAUSES: LazyLock<Mutex<HashMap<(Stage, String), Arc<Pause>>>> =
        LazyLock::new(Default::default);
    pub(super) fn pause(stage: Stage, session: &str) -> Arc<Pause> {
        let pause = Arc::new(Pause {
            entered: Default::default(),
            release: Default::default(),
        });
        PAUSES
            .lock()
            .unwrap()
            .insert((stage, session.to_owned()), pause.clone());
        pause
    }
    pub(super) async fn wait(stage: Stage, session: &str) {
        let pause = PAUSES.lock().unwrap().remove(&(stage, session.to_owned()));
        if let Some(pause) = pause {
            pause.entered.notify_one();
            pause.release.notified().await;
        }
    }
}
#[cfg(test)]
#[path = "native_metadata_admission_tests.rs"]
mod tests;
