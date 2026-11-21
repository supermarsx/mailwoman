//! Narrow native Email/get with optional immutable source DATA publication.
//! Session/credentials/allow decisions are never cached.
//! Unsupported or cold inputs return to the complete original path BEFORE
//! taking permits. Before any Engine dispatch, late source changes may restart
//! one complete fresh read chain after dropping all old permits/connector state.
//! No preparation-time observation authorizes that restarted dispatch.
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

#[path = "native_metadata_admission/publication.rs"]
pub(crate) mod publication;

fn publication_enabled()->bool {
    #[cfg(test)] {test_control::publication_enabled()}
    #[cfg(not(test))] {true}
}
type Unsupported = ();
fn unsupported<T>(value: Result<T, impl Sized>) -> Result<T, Unsupported> {
    value.map_err(|_| ())
}

pub(super) fn eligible_request(body: &[u8]) -> Option<Value> {
    if body.len() > mw_engine::jmap::core::MAX_SIZE_REQUEST {
        return None;
    }
    let request: Value = serde_json::from_slice(body).ok()?;
    eligible_parsed_request(&request, body.len())?;
    Some(request)
}

/// The feature guard already owns the bounded parsed body. Share the complete
/// validator rather than defining a weaker shape-only deferral eligibility.
pub(super) fn eligible_parsed_request(request: &Value, bytes: usize) -> Option<&Value> {
    if bytes > mw_engine::jmap::core::MAX_SIZE_REQUEST {
        return None;
    }
    mw_engine::jmap::core::validate(request, bytes, true).ok()?;
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
    revision: Option<&SettingsReadRevision>,
    published: Option<&publication::Published>,
    filter: impl Fn(&registry::SettingDef) -> bool,
) -> Result<Vec<registry::Effective>, Unsupported> {
    if let Some(published)=published {return published.values(subject,filter).ok_or(());}
    let revision=revision.ok_or(())?;
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
    revision: Option<&SettingsReadRevision>,
    published: Option<&publication::Published>,
) -> Result<bool, Unsupported> {
    Ok(value(
        &warm_values(state, subject, revision, published, |def| {
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
    prepared_mail: mw_engine::PreparedMailOptions,
    prepared_revision: SettingsReadRevision,
}
impl Loaded {
    fn target_matches(&self, headers: &HeaderMap) -> bool {
        crate::mailbox_target_matches(&self.session, headers, Some(&self.authority))
    }
}

fn names_for(snapshot:&NativeAdmissionSnapshot,state:&AppState,session:&Session)->Result<(Vec<String>,Vec<String>),Unsupported> {
    names_for_store(snapshot,&state.store,session)
}
fn names_for_store(
    snapshot: &NativeAdmissionSnapshot,
    store: &mw_store::Store,
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
    for source in unsupported(snapshot.account_session_names(store))?.ok_or(())? {
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

fn evaluate(state:&AppState,snapshot:&NativeAdmissionSnapshot,needs:&Needs)->Result<Loaded,Unsupported> {
    evaluate_inputs(state,snapshot,needs,None,None)
}
fn evaluate_inputs(
    state: &AppState,
    snapshot: &NativeAdmissionSnapshot,
    needs: &Needs,
    published:Option<&publication::Published>,
    current:Option<&mw_store::authority_publication::NativeAuthorityFence>,
) -> Result<Loaded, Unsupported> {
    if (published.is_none() && !snapshot.bounded_shape())
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
            current.map(|f|f.last_seen()).unwrap_or(source.last_seen.as_deref()),
            crate::session_persistence::flush_seconds(),
            chrono::Utc::now(),
        )
        || unsupported(snapshot.linked_policy_required())?
    {
        return Err(());
    }
    let revision=if published.is_some() {None}else {
        let revision=unsupported(snapshot.revision(&state.store))?;
        if revision.number().is_none() {return Err(());}
        Some(revision)
    };
    // Require an active, native, same-owner principal before selecting any
    // personal namespace. No caller-supplied account selects that namespace.
    let subjects = unsupported(snapshot.native_subjects(&state.store))?.ok_or(())?;
    if crate::websocket_actions::has_frame_actor() {
        let actor_principal=unsupported(snapshot.principal())?.ok_or(())?;
        if !crate::websocket_actions::native_actor_ok(&state.store,&session,&actor_principal.principal_id,&actor_principal.binding_id){return Err(());}
    }

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
    let values = warm_values(state, &feature_subject, revision.as_ref(), published, |def| {
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
    let clock_values = warm_values(state, &clock_subject, revision.as_ref(), published, |def| {
        matches!(def.key, "session.idleSecs" | "session.absoluteSecs")
    })?;
    let clock = crate::security_center::clock_policy_from_values(
        &clock_values,
        state.hardening.idle_timeout.as_secs(),
        state.hardening.absolute_timeout.as_secs(),
    );
    let (created,activity)=match current {
        Some(fence)=>{let (a,b)=fence.clocks();(a.to_owned(),b.to_owned())},
        None=>unsupported(snapshot.clocks())?.ok_or(())?,
    };
    if crate::security_center::session_deadline_passed(
        &created,
        &activity,
        chrono::Utc::now(),
        clock.idle_secs,
        clock.absolute_secs,
    ) {
        return Err(());
    }
    let global_listed = listed(state, &SettingSubject::default(), revision.as_ref(), published)?;
    if let Some(published)=published {
        for rules in &published.gates {
            let gate=rules.evaluate(&published.directory,global_listed);
            if gate.disabled || gate.password_change_required {return Err(());}
        }
    } else {
        let (session_names, account_names) = names_for(snapshot, state, &session)?;
        for names in [&session_names,&account_names] {
            let gate=unsupported(prepared(&session.account_id,names)?.from_native(&state.store,snapshot,global_listed))?.ok_or(())?;
            if gate.disabled || gate.password_change_required {return Err(());}
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
    let domain_listed = listed(state, &domain_subject, revision.as_ref(), published)?;
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
    let rate_values = warm_values(state, &rate_subject, revision.as_ref(), published, |def| {
        def.key.starts_with("ratelimit.")
    })?;
    let prepared_values=warm_values(state,&SettingSubject::default(),revision.as_ref(),published,|def|def.key=="performance.preparedMail")?;
    let prepared_mail=match value(&prepared_values,"performance.preparedMail") {
        Some(value)=>serde_json::from_value::<mw_engine::PreparedMailOptions>(value.clone()).map_err(|_|())?,
        None if registry::lookup("performance.preparedMail").is_none()=>mw_engine::PreparedMailOptions::default(),
        None=>return Err(()), // Registered-but-uncollected DATA is not a default.
    };
    if !prepared_mail.valid() {return Err(());}
    Ok(Loaded {
        session,
        knobs: rate_leeway::Knobs::from_rows(&rate_values),
        authority: subjects.storage_authority,
        prepared_mail,
        prepared_revision:unsupported(snapshot.revision(&state.store))?,
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

async fn restart_or_refuse(state:&AppState,headers:&HeaderMap,body:&[u8],deferred:&DeferredJmapFeatures)->Option<Response> {
    #[cfg(test)]
    if test_control::legacy_late_refusal() {return Some(late_refusal(state,headers).await);}
    if deferred.restart_metadata_read(body) {
        #[cfg(test)] test_control::record_restart();
        return None;
    }
    Some(late_refusal(state,headers).await)
}

/// Final source observation for the single restarted read. This remains the
/// LAST asynchronous source read before dispatch. Never follow it with a narrower
/// primary-only SELECT that could hide later raw flag/credential/config changes.
pub(super) async fn finish_restarted_read(
    state:&AppState,headers:&HeaderMap,authenticated:&Session,body:&[u8],
    witness:&mw_store::primary_policy_snapshot::PrimaryPolicyWitness,admitted:rate_leeway::Knobs,
)->Result<(),Response> {
    let refusal=super::auth_boundary::auth_fence_refusal;
    let request=eligible_request(body).ok_or_else(refusal)?;
    let needs=super::metadata_read_restart_needs(body).ok_or_else(refusal)?;
    let id=crate::cookie_value(headers).ok_or_else(refusal)?;
    if id!=authenticated.id || request["methodCalls"][0][1]["accountId"].as_str()!=Some(authenticated.account_id.as_str()) {return Err(refusal());}
    #[cfg(test)] test_control::wait(test_control::Stage::RestartBeforeFinal,&id).await;
    let names=vec![authenticated.username.clone(),authenticated.credentials.username.clone()];
    let query=shaping(&authenticated.account_id,&names).map_err(|_|refusal())?;
    let mut snapshot=observe(state,&id,&query).await.map_err(|_|refusal())?;
    let source=snapshot.session(&state.store).map_err(|_|refusal())?.ok_or_else(refusal)?.session;
    let names=names_for(&snapshot,state,&source).map_err(|_|refusal())?;
    let names=names.0.into_iter().chain(names.1).collect::<Vec<_>>();
    let complete=shaping(&source.account_id,&names).map_err(|_|refusal())?;
    if !snapshot.covers_domains(complete.domains()) || !snapshot.covers_keys(complete.keys()) {
        snapshot=observe(state,&id,&complete).await.map_err(|_|refusal())?;
    }
    // No successful path may await ANY further authority read after this point.
    if !snapshot.matches_primary_policy_witness(&state.store,witness).map_err(|_|refusal())? {return Err(refusal());}
    let source=snapshot.session(&state.store).map_err(|_|refusal())?.ok_or_else(refusal)?.session;
    if !super::auth_boundary::same_session(authenticated,&source) {return Err(refusal());}
    let (session_names,account_names)=names_for(&snapshot,state,&source).map_err(|_|refusal())?;
    let complete=shaping(&source.account_id,&session_names.into_iter().chain(account_names).collect::<Vec<_>>()).map_err(|_|refusal())?;
    if !snapshot.covers_domains(complete.domains()) || !snapshot.covers_keys(complete.keys()) {return Err(refusal());}
    let revision=snapshot.revision(&state.store).map_err(|_|refusal())?;
    let subject=SettingSubject {personal_owner:None,domain:source.username.rsplit_once('@').map_or("",|(_,d)|d),account_id:&source.account_id,address:&source.username};
    let values=warm_values(state,&subject,Some(&revision),None,|d|matches!(d.key,"session.idleSecs"|"session.absoluteSecs")).map_err(|_|refusal())?;
    let clock=crate::security_center::clock_policy_from_values(&values,state.hardening.idle_timeout.as_secs(),state.hardening.absolute_timeout.as_secs());
    let (created,activity)=snapshot.clocks().map_err(|_|refusal())?.ok_or_else(refusal)?;
    if crate::security_center::session_deadline_passed(&created,&activity,chrono::Utc::now(),clock.idle_secs,clock.absolute_secs) {
        // Failure-only cleanup may await writes; there is no subsequent dispatch.
        let _=state.store.delete_session(&id).await;
        let _=state.store.delete_native_session(&crate::push_relay::hash_token(&id)).await;
        state.sessions.forget(&id);state.step_up.forget_user(&id);
        return Err(crate::session_expired());
    }
    if state.sessions.check(&id,&state.hardening).is_err() {
        let _=state.store.delete_session(&id).await;state.sessions.forget(&id);
        return Err(crate::session_expired());
    }
    let current=evaluate(state,&snapshot,&needs).map_err(|_|refusal())?;
    if !super::auth_boundary::same_session(authenticated,&current.session) || current.knobs!=admitted || !current.target_matches(headers) {return Err(refusal());}
    Ok(())
}
#[cfg(test)]
pub(super) fn record_restart_dispatch() {test_control::record_restart_dispatch();}
#[cfg(test)]
pub(crate) fn record_ordinary_dispatch() {test_control::record_ordinary_dispatch();}

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
    let mut published=None;
    let mut first=None;
    let mut published_current=false;
    let mut observed_published=false;
    if publication_enabled() {
        if let Some(worker)=&state.native_publication {
            if let Some(entry)=worker.get(&id,&deferred.0) {
                observed_published=true;
                if let Ok(Some(fence))=state.store.native_publication_fence(&id,&entry.domains,&entry.subjects).await {
                    #[cfg(test)] test_control::record_fence();
                    if entry.matches(&fence) {
                        published_current=true;
                        if let Ok(loaded)=evaluate_inputs(state,&entry.snapshot,&deferred.0,Some(&entry),Some(&fence)) {
                            first=Some(loaded);published=Some(entry);
                        }
                    }
                }
            }
            // A contended nonblocking lookup is just a miss. It must not evict
            // an otherwise current entry once the memory mutex becomes free.
            if observed_published && !published_current {worker.invalidate(&id,&deferred.0);}
            if !published_current && request["methodCalls"][0][1]["accountId"].as_str()==Some(hint.session.account_id.as_str()) {
                worker.hint(&id,&hint.session.account_id,&deferred.0);
            }
        }
    }
    let first=match first {
        Some(first)=>first,
        None=>{
            let mut snapshot = observe(state, &id, &query).await.ok()?;
            let source = snapshot.session(&state.store).ok()??;
            let (session_names, account_names) = names_for(&snapshot, state, &source.session).ok()?;
            names = session_names.into_iter().chain(account_names).collect();
            let current = shaping(&source.session.account_id, &names).ok()?;
            if !snapshot.covers_domains(current.domains()) || !snapshot.covers_keys(current.keys()) {
                query = current;
                snapshot = observe(state, &id, &query).await.ok()?;
            }
            evaluate(state,&snapshot,&deferred.0).ok()?
        }
    };
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
    // Prepare only DATA, before the final authority observation. Configuration
    // comes from the same validated policy inputs and is compared again below.
    // Unsupported/cold representations retain the existing dispatch path.
    let engine=state.engine.as_ref()?;
    let candidate=if published.is_some() && request.get("createdIds").is_none() && engine.configure_prepared_mail_at_revision(first.prepared_mail,&first.prepared_revision) {
        engine.prepare_metadata_candidate(&first.session.account_id,&request["methodCalls"][0][1]).await
    }else {None};
    #[cfg(test)]
    test_control::wait(test_control::Stage::BeforeConnector, &id).await;
    let lock = engine_mode::connect_lock(&first.session.account_id);
    let connecting = lock.lock().await;
    let mut final_data=None;
    let final_source=if let Some(entry)=&published {
        let observed=if let Some(candidate)=&candidate {
            state.store.native_publication_fence_with_mail_data(&id,&entry.domains,&entry.subjects,candidate.descriptor()).await
        }else {state.store.native_publication_fence(&id,&entry.domains,&entry.subjects).await};
        match observed {
            Ok(Some(fence))=>{
                #[cfg(test)] test_control::record_fence();
                if entry.matches(&fence) {
                    final_data=fence.mail_data().cloned();
                    evaluate_inputs(state,&entry.snapshot,&deferred.0,Some(entry),Some(&fence)).ok()
                } else {None}
            },
            _=>None,
        }
    } else {
        match observe(state,&id,&query).await {
            Ok(snapshot)=>evaluate(state,&snapshot,&deferred.0).ok(),
            Err(_)=>None,
        }
    };
    let Some(final_source) = final_source.filter(|source| {
        super::auth_boundary::same_session(&first.session, &source.session)
            && first.knobs == source.knobs
            && first.prepared_mail == source.prepared_mail
            && source.target_matches(headers)
    }) else {
        drop(connecting);
        drop(permit);
        return restart_or_refuse(state,headers,body,deferred).await;
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
    // No async operation may intervene between this fresh joint proof, the
    // remaining pure checks and prepared emission. The Engine verifies its
    // logical publication/configuration proof again around byte assembly.
    if let (Some(candidate),Some(data))=(candidate,final_data) {
        if let Some(call_id)=request["methodCalls"][0][2].as_str() {
            if let Some(bytes)=engine.finish_prepared_metadata(candidate,&data,call_id) {
                #[cfg(test)] test_control::record_prepared_result();
                drop(permit);
                return Some(([(header::CONTENT_TYPE,"application/json")],bytes.into_bytes()).into_response());
            }
        }
    }
    // No further asynchronous policy/credential lookup occurs before dispatch.
    // Engine retains its own fresh ownership and posture checks for data reads.
    #[cfg(test)]
    test_control::record_dispatch();
    let dispatch=state.engine.as_ref()?.handle_jmap_with_sieve(&final_source.session.account_id,&request,None);
    #[cfg(feature="sql-shape-diagnostics")]
    let dispatch=mw_store::pool_telemetry::shapes::in_phase(mw_store::pool_telemetry::shapes::Phase::Engine,dispatch);
    let response=dispatch.await;
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
        pub publication:bool,
        pub fences:u64,
        pub legacy_late_refusal:bool,
        pub restarts:u64,
        pub restart_dispatches:u64,
        pub ordinary_dispatches:u64,
        pub prepared_results:u64,
    }
    tokio::task_local! { static COUNTS: RefCell<Counts>; }
    pub(super) fn force_original() -> bool {
        COUNTS
            .try_with(|counts| counts.borrow().original)
            .unwrap_or(false)
    }
    pub(super) fn publication_enabled()->bool {COUNTS.try_with(|c|c.borrow().publication).unwrap_or(false)}
    pub(super) fn record_fence() {let _=COUNTS.try_with(|c|c.borrow_mut().fences+=1);}
    pub(super) async fn count_published<F:std::future::Future>(future:F)->(F::Output,Counts) {
        COUNTS.scope(RefCell::new(Counts {publication:true,..Default::default()}),async {
            let value=future.await;(value,COUNTS.with(|counts|*counts.borrow()))
        }).await
    }
    pub(super) fn legacy_late_refusal()->bool {COUNTS.try_with(|c|c.borrow().legacy_late_refusal).unwrap_or(false)}
    pub(super) fn record_ordinary_dispatch() {let _=COUNTS.try_with(|c|c.borrow_mut().ordinary_dispatches+=1);}
    pub(super) fn record_prepared_result() {let _=COUNTS.try_with(|c|c.borrow_mut().prepared_results+=1);}
    pub(super) fn record_restart_dispatch() {let _=COUNTS.try_with(|c|c.borrow_mut().restart_dispatches+=1);}
    pub(super) fn record_restart() {let _=COUNTS.try_with(|c|c.borrow_mut().restarts+=1);}
    pub(super) async fn count_legacy_late<F:std::future::Future>(future:F)->(F::Output,Counts) {
        COUNTS.scope(RefCell::new(Counts {legacy_late_refusal:true,..Default::default()}),async {
            let output=future.await;(output,COUNTS.with(|c|*c.borrow()))
        }).await
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
        RestartBeforeFinal,
        PublicationBeforeFinal,
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
