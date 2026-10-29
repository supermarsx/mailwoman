//! Owner-bound, cancellable diagnostic jobs. Only saved targets are accepted.
use super::*;
use crate::domain_selection::{Backend, BackendConfig, Protocol, ServiceRole};
use axum::extract::Path;
use serde_json::Value;
use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
};
use tokio::sync::{Notify, Semaphore};

const MAX_CHECKS: usize = 128;
static JOBS: OnceLock<Mutex<HashMap<String, Arc<Job>>>> = OnceLock::new();
static PROBES: OnceLock<Arc<Semaphore>> = OnceLock::new();
struct Job {
    owner: String,
    created: Instant,
    cancelled: AtomicBool,
    wake: Notify,
    snapshot: Mutex<Value>,
}
fn jobs() -> &'static Mutex<HashMap<String, Arc<Job>>> {
    JOBS.get_or_init(Default::default)
}
fn slots() -> Arc<Semaphore> {
    PROBES.get_or_init(|| Arc::new(Semaphore::new(4))).clone()
}
pub(super) fn register(r: &mut impl AdminRegistrar) {
    r.admin_endpoint(
        Method::GET,
        "/diag/catalogue",
        Capability::DiagnosticsRead,
        catalogue,
    );
    r.admin_endpoint(
        Method::POST,
        "/diag/jobs",
        Capability::DiagnosticsRun,
        start,
    );
    r.admin_endpoint(
        Method::GET,
        "/diag/jobs/{id}",
        Capability::DiagnosticsRead,
        poll,
    );
    r.admin_endpoint(
        Method::POST,
        "/diag/jobs/{id}/cancel",
        Capability::DiagnosticsRun,
        cancel,
    );
    r.admin_endpoint(
        Method::GET,
        "/diag/report",
        Capability::DiagnosticsRead,
        report,
    );
}
fn response(value: Value) -> Response {
    ([(header::CACHE_CONTROL, "private, no-store")], Json(value)).into_response()
}
async fn catalogue(ctx: AdminCtx) -> Result<Response, AdminApiError> {
    ctx.require_unscoped(Capability::DiagnosticsRead)?;
    Ok(response(
        json!({"canRunBackends":ctx.require_unscoped(Capability::DomainsUpstreamWrite).is_ok(),"suites":[{"id":"server","supported":true,"description":"Database, migrations, encryption and default egress policy"},{"id":"backends","supported":true,"description":"Every saved mail/submission/rules backend and DAV URL, including non-selected priorities; no credentials sent"},{"id":"services","supported":false,"reason":"Arbitrary local services, pipes and provider-authenticated operations have no safe diagnostic adapter yet"}],"limits":{"concurrency":4,"checks":MAX_CHECKS,"runSeconds":60,"checkSeconds":15}}),
    ))
}
fn owned(id: &str, ctx: &AdminCtx) -> Result<Arc<Job>, AdminApiError> {
    jobs()
        .lock()
        .unwrap_or_else(|e| e.into_inner())
        .get(id)
        .filter(|job| {
            job.owner == ctx.session_hash && job.created.elapsed() < Duration::from_secs(1800)
        })
        .cloned()
        .ok_or_else(|| AdminApiError::not_found("Diagnostic job not found for this session."))
}
async fn poll(ctx: AdminCtx, Path(id): Path<String>) -> Result<Response, AdminApiError> {
    ctx.require_unscoped(Capability::DiagnosticsRead)?;
    let job = owned(&id, &ctx)?;
    let snapshot = job
        .snapshot
        .lock()
        .unwrap_or_else(|e| e.into_inner())
        .clone();
    Ok(response(snapshot))
}
async fn cancel(ctx: AdminCtx, Path(id): Path<String>) -> Result<Response, AdminApiError> {
    ctx.require_unscoped(Capability::DiagnosticsRun)?;
    let job = owned(&id, &ctx)?;
    job.cancelled.store(true, Ordering::Release);
    job.wake.notify_one();
    Ok(response(json!({"cancellationRequested":true})))
}
#[derive(Clone)]
struct Target {
    domain: String,
    revision: i64,
    id: String,
    protocol: String,
    enabled: bool,
    backend: Option<Backend>,
    http_url: Option<String>,
}
fn configured(row: &mw_store::DomainConfigRow) -> Result<Vec<Target>, String> {
    let value: Value =
        serde_json::from_str(&row.upstream_json).map_err(|_| "invalid saved JSON")?;
    let dav = crate::domain_config::UpstreamConfig::parse(&row.upstream_json)?.dav;
    let backends = if value.get("version").is_some() {
        let config: BackendConfig =
            serde_json::from_value(value).map_err(|_| "invalid saved backend list")?;
        config.validate()?;
        config.backends
    } else {
        let config = crate::domain_config::UpstreamConfig::parse(&row.upstream_json)?;
        let mut result = Vec::new();
        for (id, protocol, role, endpoint) in [
            ("imap", Protocol::Imap, ServiceRole::Mail, config.imap),
            ("pop3", Protocol::Pop3, ServiceRole::Mail, config.pop3),
            ("smtp", Protocol::Smtp, ServiceRole::Submission, config.smtp),
            ("sieve", Protocol::Sieve, ServiceRole::Rules, config.sieve),
        ] {
            if let Some(endpoint) = endpoint {
                result.push(Backend {
                    id: id.into(),
                    role,
                    protocol,
                    enabled: true,
                    preferred: false,
                    priority: 100,
                    endpoint: Some(endpoint),
                    session_url: None,
                    ews: None,
                    username: Default::default(),
                });
            }
        }
        if let Some(jmap) = config.jmap {
            result.push(Backend {
                id: "jmap".into(),
                role: ServiceRole::Mail,
                protocol: Protocol::Jmap,
                enabled: true,
                preferred: false,
                priority: 100,
                endpoint: None,
                session_url: Some(jmap.session_url),
                ews: None,
                username: Default::default(),
            });
        }
        result
    };
    let mut targets: Vec<Target> = backends
        .into_iter()
        .map(|mut backend| {
            backend.enabled &= row.enabled;
            Target {
                domain: row.name.clone(),
                revision: row.updated_at,
                id: backend.id.clone(),
                protocol: serde_json::to_value(backend.protocol)
                    .unwrap()
                    .as_str()
                    .unwrap()
                    .into(),
                enabled: backend.enabled,
                backend: Some(backend),
                http_url: None,
            }
        })
        .collect();
    if let Some(dav) = dav {
        for (protocol, url) in [("caldav", dav.caldav_url), ("carddav", dav.carddav_url)] {
            if let Some(url) = url {
                targets.push(Target {
                    domain: row.name.clone(),
                    revision: row.updated_at,
                    id: protocol.into(),
                    protocol: protocol.into(),
                    enabled: row.enabled,
                    backend: None,
                    http_url: Some(url),
                });
            }
        }
    }
    Ok(targets)
}
/// A backend row. `elapsed` is `None` for a check that never ran: `durationUs`
/// is then null, so the console shows no time rather than a measured zero.
fn item(target: &Target, status: &str, elapsed: Option<Duration>, summary: &str) -> Value {
    let ms = elapsed.map_or(0, |d| d.as_millis().min(u64::MAX as u128) as u64);
    let us = elapsed.map(|d| d.as_micros().min(u64::MAX as u128) as u64);
    json!({"id":format!("{}:{}",target.domain,target.id),"domain":target.domain,"backendId":target.id,"revision":target.revision,"category":"backend","protocol":target.protocol,"status":status,"durationMs":ms,"durationUs":us,"summary":summary,"fixId":"saved-backend-connection"})
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct StartInput {
    suite: String,
}
async fn start(
    State(state): State<AppState>,
    ctx: AdminCtx,
    AdminJson(input): AdminJson<StartInput>,
) -> Result<Response, AdminApiError> {
    ctx.require_unscoped(Capability::DiagnosticsRun)?;
    ctx.require_unscoped(Capability::DomainsUpstreamWrite)?;
    if input.suite != "backends" {
        return Err(AdminApiError::invalid(
            "unsupported_suite",
            "Choose the saved backends suite.",
            "/suite",
        ));
    }
    super::limit(&ctx.session_hash)?;
    let rows = state.store.list_domain_configs_limited(129).await?;
    let mut targets = Vec::new();
    let mut results = Vec::new();
    let mut limited = rows.len() > 128;
    for row in rows.into_iter().take(128) {
        if targets.len() + results.len() >= MAX_CHECKS {
            limited = true;
            break;
        }
        match configured(&row){
            Ok(items)=>for target in items{if targets.len()+results.len()>=MAX_CHECKS{limited=true;break;}
if target.enabled{targets.push(target);}else{results.push(item(&target,"skipped",None,"Disabled domain or backend; no connection attempted"));}},
            Err(_)=>results.push(json!({"id":format!("{}:configuration",row.name),"domain":row.name,"category":"configuration","status":"fail","durationMs":0,"durationUs":null,"summary":"Saved backend configuration is invalid","fixId":"domain-configuration"})),
        }
    }
    let plan = crate::image_proxy::load_plan(&state.store)
        .await
        .map_err(|_| {
            AdminApiError::conflict("egress_unavailable", "Saved egress policy is unavailable.")
        })?;
    let id = uuid::Uuid::new_v4().to_string();
    let started = chrono::Utc::now().timestamp();
    let job = Arc::new(Job {
        owner: ctx.session_hash.clone(),
        created: Instant::now(),
        cancelled: AtomicBool::new(false),
        wake: Notify::new(),
        snapshot: Mutex::new(
            json!({"run":{"id":id,"suite":"backends","startedAt":started,"status":"running","results":results},"total":targets.len()+results.len(),"completed":results.len(),"limited":limited,"credentialsSent":false,"historySaved":false}),
        ),
    });
    {
        let mut all = jobs().lock().unwrap_or_else(|e| e.into_inner());
        all.retain(|_, j| j.created.elapsed() < Duration::from_secs(1800));
        let running = |j: &&Arc<Job>| {
            j.snapshot.lock().unwrap_or_else(|e| e.into_inner())["run"]["status"] == "running"
        };
        if all.values().filter(running).count() >= 8
            || all
                .values()
                .filter(|j| j.owner == ctx.session_hash)
                .filter(running)
                .count()
                >= 2
            || all.len() >= 64
        {
            return Err(AdminApiError::conflict(
                "diagnostics_busy",
                "Diagnostic capacity is occupied; retry after existing runs finish.",
            ));
        }
        all.insert(id.clone(), job.clone());
    }
    let audit = state
        .v6
        .admin
        .audit(mw_admin::AuditLogEntry {
            id: uuid::Uuid::new_v4().to_string(),
            ts: chrono::Utc::now().to_rfc3339(),
            actor: ctx.username.clone(),
            actor_kind: mw_admin::ActorKind::Admin,
            action: "diagnostics.run".into(),
            target: Some(id.clone()),
            detail_json: json!({"suite":"backends","checks":targets.len()}).to_string(),
            ip: None,
        })
        .await
        .is_ok();
    let snapshot = job
        .snapshot
        .lock()
        .unwrap_or_else(|e| e.into_inner())
        .clone();
    tokio::spawn(execute(
        state,
        job,
        targets,
        plan,
        id,
        started,
        ctx.username,
        audit,
    ));
    Ok((StatusCode::ACCEPTED, response(snapshot)).into_response())
}
#[allow(clippy::too_many_arguments)]
async fn execute(
    state: AppState,
    job: Arc<Job>,
    targets: Vec<Target>,
    plan: mw_egress::plan::EgressPlan,
    id: String,
    started: i64,
    actor: String,
    audit: bool,
) {
    let mut work = tokio::task::JoinSet::new();
    for target in targets {
        let plan = plan.clone();
        let slots = slots();
        let store = state.store.clone();
        work.spawn(async move {
            let start=Instant::now();
            let result=tokio::time::timeout(Duration::from_secs(15),async{
                let _permit=slots.acquire_owned().await.map_err(|_|())?;
                let current=store.get_domain_config(&target.domain).await.map_err(|_|())?;
                if current.is_none_or(|row| !row.enabled || row.updated_at!=target.revision){return Err(());}
                let detail=if let Some(backend)=&target.backend {crate::domain_probe::diagnostic_backend(&target.domain,backend,&plan).await} else {crate::domain_probe::diagnostic_http_service(&target.domain,&target.protocol,target.http_url.as_deref().unwrap_or_default(),&plan).await};
                Ok::<_,()>(detail)
            }).await;
            let duration=Some(start.elapsed());
            match result {Ok(Ok(detail))=>{let pass=detail["reachable"]==true;let mut row=item(&target,if pass{"pass"}else{"fail"},duration,if pass{"Saved endpoint handshake completed; authentication not tested"}else{"Connection or security handshake failed; review DNS, route, certificate and saved endpoint"});row["detail"]=detail;row},_=>item(&target,"fail",duration,"Check timed out or capacity was unavailable")}
        });
    }
    let deadline = tokio::time::sleep(Duration::from_secs(60));
    tokio::pin!(deadline);
    let mut terminal = "complete";
    loop {
        if job.cancelled.load(Ordering::Acquire) {
            terminal = "cancelled";
            break;
        }
        tokio::select! {
            _=job.wake.notified()=>{terminal="cancelled";break;},
            _=&mut deadline=>{terminal="timed_out";break;},
            result=work.join_next()=>match result{None=>break,Some(result)=>{
                let row=result.unwrap_or_else(|_|json!({"id":"worker","status":"fail","durationMs":0,"durationUs":null,"summary":"Diagnostic worker did not complete"}));
                let mut value=job.snapshot.lock().unwrap_or_else(|e|e.into_inner());value["run"]["results"].as_array_mut().unwrap().push(row);value["completed"]=json!(value["run"]["results"].as_array().unwrap().len());
            }}
        }
    }
    work.abort_all();
    while work.join_next().await.is_some() {}
    let mut value = job
        .snapshot
        .lock()
        .unwrap_or_else(|e| e.into_inner())
        .clone();
    let failed = value["run"]["results"]
        .as_array()
        .unwrap()
        .iter()
        .any(|r| r["status"] == "fail");
    if terminal != "complete" {
        value["run"]["results"].as_array_mut().unwrap().push(json!({"id":"run-termination","category":"run","status":"skipped","durationMs":0,"summary":if terminal=="cancelled"{"Cancelled by the requesting session; unfinished checks were stopped"}else{"Run deadline reached; unfinished checks were stopped"},"termination":terminal}));
    }
    let mut omitted = 0;
    while serde_json::to_vec(&value["run"]["results"]).is_ok_and(|bytes| bytes.len() > 60 * 1024) {
        value["run"]["results"].as_array_mut().unwrap().pop();
        omitted += 1;
    }
    if omitted > 0 {
        value["limited"] = json!(true);
        value["run"]["results"].as_array_mut().unwrap().push(json!({"id":"report-limit","status":"skipped","durationMs":0,"summary":"Additional results omitted from the bounded report","omitted":omitted}));
    }
    let status = if terminal == "timed_out" {
        "error"
    } else if terminal == "cancelled" {
        "warn"
    } else if failed {
        "fail"
    } else if value["limited"] == true
        || !value["run"]["results"]
            .as_array()
            .unwrap()
            .iter()
            .any(|row| row["status"] == "pass")
    {
        "warn"
    } else {
        "pass"
    };
    let run = DiagnosticRun {
        id,
        suite: "backends".into(),
        started_at: started,
        finished_at: chrono::Utc::now().timestamp(),
        status: status.into(),
        results: value["run"]["results"].clone(),
        triggered_by: actor,
    };
    let saved = state.store.save_diagnostic_run(&run).await.is_ok();
    value["run"] = serde_json::to_value(run).unwrap();
    value["historySaved"] = json!(saved);
    value["retentionApplied"] = json!(super::prune_history(&state.store).await.is_ok());
    value["auditSaved"] = json!(audit);
    value["termination"] = json!(terminal);
    *job.snapshot.lock().unwrap_or_else(|e| e.into_inner()) = value;
}
#[derive(Deserialize, Default)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct ReportInput {
    #[serde(default)]
    omit_domains: bool,
    #[serde(default)]
    selected: Option<String>,
}
async fn report(
    State(state): State<AppState>,
    ctx: AdminCtx,
    Query(input): Query<ReportInput>,
) -> Result<Response, AdminApiError> {
    ctx.require_unscoped(Capability::DiagnosticsRead)?;
    let runs = state.store.diagnostic_runs(None, 10).await?;
    let selected = input
        .selected
        .as_deref()
        .and_then(|id| runs.iter().find(|run| run.id == id))
        .map(|run| serde_json::to_value(run).unwrap_or(Value::Null));
    let included = if input.omit_domains {
        "Allowlisted runtime flags and the latest ten diagnostic results; domain names and backend identifiers are replaced with placeholders"
    } else {
        "Allowlisted runtime flags and the latest ten diagnostic results; domain names and backend identifiers may appear"
    };
    let mut value = json!({"schemaVersion":1,"generatedAt":chrono::Utc::now().to_rfc3339(),"server":{"version":env!("CARGO_PKG_VERSION"),"os":std::env::consts::OS,"architecture":std::env::consts::ARCH,"mode":if state.engine.is_some(){"engine"}else{"proxy"},"uptimeSeconds":state.started_at.elapsed().map(|v|v.as_secs()).unwrap_or_default()},"security":{"strictCsrf":state.hardening.csrf_strict,"coep":state.hardening.coep,"configuredHttps":state.external.declares_https(),"idleTimeoutSeconds":state.hardening.idle_timeout.as_secs(),"absoluteTimeoutSeconds":state.hardening.absolute_timeout.as_secs()},"redaction":{"included":included,"domainsOmitted":input.omit_domains,"excluded":["credentials","tokens","cookies","private keys","raw configuration","user mail","raw upstream greetings"]},"runs":runs});
    if let Some(selected) = selected {
        value["selectedRun"] = selected;
    }
    if input.omit_domains {
        let names = state
            .store
            .list_domain_configs_limited(512)
            .await?
            .into_iter()
            .map(|row| row.name)
            .collect::<Vec<_>>();
        omit_domains(&mut value, &names);
    }
    Ok(response(value))
}

/// Replace domain names (in `domain` fields, `domain:backend` IDs and any text)
/// and backend IDs (in `backendId` fields and those IDs) with stable placeholders
/// `domain-N` / `backend-N`, numbered in order of first appearance.
fn omit_domains(value: &mut Value, saved: &[String]) {
    fn collect(value: &Value, domains: &mut Vec<String>, backends: &mut Vec<String>) {
        match value {
            Value::Object(map) => {
                for (key, item) in map {
                    if let Value::String(text) = item {
                        let list = match key.as_str() {
                            "domain" => &mut *domains,
                            "backendId" => &mut *backends,
                            _ => continue,
                        };
                        if !text.is_empty() && !list.contains(text) {
                            list.push(text.clone());
                        }
                    } else {
                        collect(item, domains, backends);
                    }
                }
            }
            Value::Array(items) => items.iter().for_each(|i| collect(i, domains, backends)),
            _ => {}
        }
    }
    let (mut domains, mut backends) = (Vec::new(), Vec::new());
    collect(value, &mut domains, &mut backends);
    for name in saved {
        if !domains.contains(name) {
            domains.push(name.clone());
        }
    }
    let label = |list: &[String], text: &str, prefix: &str| {
        list.iter()
            .position(|n| n == text)
            .map(|i| format!("{prefix}-{}", i + 1))
    };
    // Longest first, so `mail.example.org` is not half-replaced by `example.org`.
    let mut by_length: Vec<(usize, &String)> = domains.iter().enumerate().collect();
    by_length.sort_by_key(|(_, n)| std::cmp::Reverse(n.len()));
    let scrub = |text: &str| {
        let mut out = text.to_string();
        for (i, name) in &by_length {
            if !name.is_empty() {
                out = replace_ignore_case(&out, name, &format!("domain-{}", i + 1));
            }
        }
        out
    };
    fn walk(value: &mut Value, key: Option<&str>, f: &dyn Fn(Option<&str>, &str) -> String) {
        match value {
            Value::Object(map) => map
                .iter_mut()
                .for_each(|(k, v)| walk(v, Some(k.as_str()), f)),
            Value::Array(items) => items.iter_mut().for_each(|v| walk(v, key, f)),
            Value::String(text) => *text = f(key, text),
            _ => {}
        }
    }
    walk(value, None, &|key, text| match key {
        Some("domain") => label(&domains, text, "domain").unwrap_or_else(|| scrub(text)),
        Some("backendId") => label(&backends, text, "backend").unwrap_or_else(|| "backend".into()),
        Some("id") => match text.split_once(':') {
            Some((d, b)) if domains.iter().any(|n| n == d) => format!(
                "{}:{}",
                label(&domains, d, "domain").unwrap_or_default(),
                label(&backends, b, "backend").unwrap_or_else(|| scrub(b))
            ),
            _ => scrub(text),
        },
        _ => scrub(text),
    });
}

fn replace_ignore_case(text: &str, needle: &str, with: &str) -> String {
    let lower = text.to_ascii_lowercase();
    let needle = needle.to_ascii_lowercase();
    let mut out = String::with_capacity(text.len());
    let mut at = 0;
    while let Some(found) = lower[at..].find(&needle) {
        out.push_str(&text[at..at + found]);
        out.push_str(with);
        at += found + needle.len();
    }
    out.push_str(&text[at..]);
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn inventory_keeps_nonpreferred_and_disabled_without_credentials() {
        let row=mw_store::DomainConfigRow{name:"example.test".into(),upstream_json:json!({"version":2,"backends":[{"id":"primary","protocol":"imap","role":"mail","preferred":true,"endpoint":{"host":"imap.example.test","port":993,"tls":"implicit"}},{"id":"backup","protocol":"pop3","role":"mail","priority":200,"endpoint":{"host":"pop.example.test","port":995,"tls":"implicit"}},{"id":"off","protocol":"smtp","role":"submission","enabled":false,"endpoint":{"host":"smtp.example.test","port":465,"tls":"implicit"}}]}).to_string(),allowlist_json:"[]".into(),blocklist_json:"[]".into(),settings_json:"{}".into(),enabled:true,preferred:false,updated_at:7};
        let inventory = configured(&row).unwrap();
        assert_eq!(inventory.len(), 3);
        assert_eq!(inventory[1].id, "backup");
        assert!(!inventory[2].enabled);
        assert_eq!(inventory[0].revision, 7);
        let report = item(
            &inventory[0],
            "fail",
            Some(Duration::from_micros(1_250)),
            "Handshake failed",
        );
        assert_eq!(report["durationMs"], 1);
        assert_eq!(report["durationUs"], 1_250);
        let never = item(&inventory[2], "skipped", None, "Disabled");
        assert_eq!(never["durationMs"], 0);
        assert!(never["durationUs"].is_null());
        let text = report.to_string();
        assert!(!text.contains("imap.example.test"));
        assert!(!text.contains("username"));
    }
    #[test]
    fn omitting_domains_replaces_names_and_backend_ids_everywhere() {
        let mut value = json!({"runs":[{"results":[
            {"id":"mail.example.org:primary","domain":"mail.example.org","backendId":"primary","summary":"ok"},
            {"id":"example.org:imap","domain":"example.org","backendId":"imap","summary":"Example.ORG and mail.example.org failed"},
            {"id":"database-write","summary":"Scratch write succeeds"}
        ]}],"selectedRun":{"results":[{"domain":"example.org","backendId":"imap"}]},
        "note":"secret.example.net appears"});
        omit_domains(&mut value, &["secret.example.net".into()]);
        let text = value.to_string();
        for leaked in [
            "example.org",
            "Example.ORG",
            "secret.example.net",
            "\"primary\"",
            "\"imap\"",
        ] {
            assert!(!text.contains(leaked), "{leaked} in {text}");
        }
        let rows = &value["runs"][0]["results"];
        assert_eq!(rows[0]["id"], "domain-1:backend-1");
        assert_eq!(rows[1]["domain"], "domain-2");
        assert_eq!(rows[1]["summary"], "domain-2 and domain-1 failed");
        assert_eq!(rows[2]["id"], "database-write", "other IDs are kept");
        assert_eq!(value["selectedRun"]["results"][0]["backendId"], "backend-2");
        assert_eq!(value["note"], "domain-3 appears");
    }
    #[test]
    fn another_session_cannot_observe_or_cancel_job() {
        let mut owner = AdminCtx::all_capabilities("owner");
        owner.session_hash = uuid::Uuid::new_v4().to_string();
        let mut other = AdminCtx::all_capabilities("owner");
        other.session_hash = uuid::Uuid::new_v4().to_string();
        let id = uuid::Uuid::new_v4().to_string();
        let job = Arc::new(Job {
            owner: owner.session_hash.clone(),
            created: Instant::now(),
            cancelled: AtomicBool::new(false),
            wake: Notify::new(),
            snapshot: Mutex::new(json!({})),
        });
        jobs().lock().unwrap().insert(id.clone(), job);
        assert!(owned(&id, &owner).is_ok());
        assert!(owned(&id, &other).is_err());
        jobs().lock().unwrap().remove(&id);
    }
}
