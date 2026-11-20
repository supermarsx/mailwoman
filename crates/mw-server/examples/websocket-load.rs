//! Separate-process, opt-in authenticated WebSocket fixture generator.
//! No credentials/account identifiers are copied to reports or stderr.
use futures_util::{SinkExt, StreamExt};
use serde::Deserialize;
use serde_json::{Value, json};
use std::{
    collections::HashSet,
    fs::{self, File, OpenOptions},
    io::{Read, Write},
    net::Ipv4Addr,
    path::PathBuf,
    sync::{
        Arc,
        atomic::{AtomicU64, Ordering},
    },
    time::{Duration, Instant, SystemTime, UNIX_EPOCH},
};
use tokio::{
    sync::{Semaphore, watch},
    task::JoinSet,
};
use tokio_tungstenite::{
    connect_async_with_config,
    tungstenite::{
        client::IntoClientRequest,
        http::{HeaderValue, header},
        protocol::{Message, WebSocketConfig},
    },
};
const MAX_MANIFEST: u64 = 32 * 1024 * 1024;
#[derive(Clone, Copy, PartialEq, Eq)]
enum Protocol {
    Push,
    Actions,
}
#[derive(Clone)]
struct Config {
    manifest: PathBuf,
    report: PathBuf,
    connections: usize,
    ramp: u64,
    pending: usize,
    warm: u64,
    hold: u64,
    timeout: u64,
    workers: usize,
    trigger_interval: u64,
    ready_file: Option<PathBuf>,
}
impl Config {
    fn parse(args: impl IntoIterator<Item = String>) -> Result<(Self, Protocol), &'static str> {
        let mut config = Self {
            manifest: PathBuf::new(),
            report: PathBuf::new(),
            connections: 10000,
            ramp: 100,
            pending: 128,
            warm: 10,
            hold: 60,
            timeout: 30,
            workers: 2,
            trigger_interval: 10,
            ready_file: None,
        };
        let mut protocol = Protocol::Push;
        let mut args = args.into_iter();
        while let Some(flag) = args.next() {
            let value = args.next().ok_or("Every option requires a value")?;
            match flag.as_str() {
                "--manifest" => config.manifest = value.into(),
                "--report" => config.report = value.into(),
                "--ready-file" => config.ready_file = Some(value.into()),
                "--protocol" => {
                    protocol = match value.as_str() {
                        "push" => Protocol::Push,
                        "actions" => Protocol::Actions,
                        _ => return Err("Unknown protocol"),
                    }
                }
                "--connections" => {
                    config.connections = value.parse().map_err(|_| "Invalid count")?
                }
                "--ramp-per-second" => config.ramp = value.parse().map_err(|_| "Invalid rate")?,
                "--max-pending" => {
                    config.pending = value.parse().map_err(|_| "Invalid pending bound")?
                }
                "--warm-seconds" => config.warm = value.parse().map_err(|_| "Invalid warmup")?,
                "--hold-seconds" => config.hold = value.parse().map_err(|_| "Invalid hold")?,
                "--connect-timeout-seconds" => {
                    config.timeout = value.parse().map_err(|_| "Invalid timeout")?
                }
                "--workers" => config.workers = value.parse().map_err(|_| "Invalid workers")?,
                "--trigger-interval-ms" => {
                    config.trigger_interval =
                        value.parse().map_err(|_| "Invalid trigger interval")?
                }
                _ => return Err("Unknown option"),
            }
        }
        if config.manifest.as_os_str().is_empty()
            || config.report.as_os_str().is_empty()
            || !(1..=10000).contains(&config.connections)
            || !(1..=5000).contains(&config.ramp)
            || !(1..=512).contains(&config.pending)
            || config.warm > 300
            || !(1..=1800).contains(&config.hold)
            || !(1..=120).contains(&config.timeout)
            || !(1..=32).contains(&config.workers)
            || !(1..=1000).contains(&config.trigger_interval)
            || config.ready_file.as_ref().is_some_and(|path| {
                path.as_os_str().is_empty() || path == &config.manifest || path == &config.report
            })
        {
            return Err("Arguments exceed fixture bounds");
        }
        if config.connections as u64 / config.ramp
            + config.warm
            + config.hold
            + config.connections as u64 * config.trigger_interval / 1000
            + config.timeout
            > 3500
        {
            return Err("Campaign exceeds bounded lifetime");
        }
        if config.connections == 10000 && config.hold < 60 {
            return Err("Ten-thousand-user acceptance requires at least sixty steady seconds");
        }
        Ok((config, protocol))
    }
}
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct Manifest {
    base: String,
    engine_mode: String,
    backend: String,
    build_profile: String,
    principals: Vec<Principal>,
    push_trigger: Option<PushTrigger>,
    client_simulation: Option<ClientSimulation>,
}
#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct ClientSimulation {
    mode: String,
    client_count: usize,
}
#[derive(Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
struct Principal {
    cookie: String,
    #[serde(default)]
    csrf_token: String,
    account_id: String,
    owner_account_id: Option<String>,
    client_address: Option<String>,
}
#[derive(Clone, Deserialize)]
struct PushTrigger {
    path: String,
    token: String,
}
fn validate_manifest(
    manifest: &Manifest,
    config: &Config,
    protocol: Protocol,
) -> Result<reqwest::Url, &'static str> {
    if manifest.engine_mode != "native"
        || !matches!(manifest.backend.as_str(), "sqlite" | "postgres")
        || !matches!(manifest.build_profile.as_str(), "debug" | "release" | "fast")
        || manifest.principals.len() < config.connections
        || manifest.principals.len() > 10000
    {
        return Err("Invalid native fixture manifest");
    }
    let mut base = reqwest::Url::parse(&manifest.base).map_err(|_| "Invalid fixture origin")?;
    if !matches!(base.scheme(), "http" | "https")
        || base.host_str().is_none()
        || !base.username().is_empty()
        || base.password().is_some()
        || base.query().is_some()
        || base.fragment().is_some()
    {
        return Err("Invalid fixture origin");
    }
    if !base.path().ends_with('/') {
        base.set_path(&format!("{}/", base.path()));
    }
    if let Some(simulation) = &manifest.client_simulation {
        if simulation.mode != "fixture-loopback-trusted-proxy"
            || simulation.client_count != manifest.principals.len()
            || base.scheme() != "http"
            || base.host_str() != Some("127.0.0.1")
            || base.path() != "/"
        {
            return Err("Invalid isolated fixture client simulation");
        }
    } else if manifest
        .principals
        .iter()
        .any(|principal| principal.client_address.is_some())
    {
        return Err("Forwarded clients require explicit fixture proxy simulation");
    }
    let mut accounts = HashSet::new();
    let mut cookies = HashSet::new();
    let mut sessions = HashSet::new();
    let mut clients = HashSet::new();
    for principal in &manifest.principals {
        if principal.account_id.is_empty()
            || principal.account_id.len() > 512
            || principal
                .owner_account_id
                .as_ref()
                .is_some_and(|owner| owner != &principal.account_id)
            || principal.cookie.len() > 8192
            || !principal.cookie.split(';').any(|part| {
                part.trim()
                    .strip_prefix("mw_session=")
                    .is_some_and(|token| !token.is_empty())
            })
            || HeaderValue::from_str(&principal.cookie).is_err()
            || !accounts.insert(&principal.account_id)
            || !cookies.insert(&principal.cookie)
        {
            return Err("Principals must have distinct accounts and session credentials");
        }
        let sessions_for_principal = principal
            .cookie
            .split(';')
            .filter_map(|part| part.trim().strip_prefix("mw_session="))
            .collect::<Vec<_>>();
        if sessions_for_principal.len() != 1 || !sessions.insert(sessions_for_principal[0]) {
            return Err("Distinct fixture session tokens are required");
        }
        if manifest.client_simulation.is_some() {
            let address = principal
                .client_address
                .as_deref()
                .ok_or("Missing fixture client address")?;
            let parsed: Ipv4Addr = address
                .parse()
                .map_err(|_| "Invalid benchmark client address")?;
            let octets = parsed.octets();
            if octets[0] != 198
                || !matches!(octets[1], 18 | 19)
                || parsed.to_string() != address
                || !clients.insert(parsed)
            {
                return Err("Distinct canonical benchmark client addresses are required");
            }
        }
        if protocol == Protocol::Actions
            && (principal.csrf_token.is_empty()
                || HeaderValue::from_str(&principal.csrf_token).is_err())
        {
            return Err("Actions proof requires fixture CSRF credentials");
        }
    }
    if protocol == Protocol::Push {
        let trigger = manifest
            .push_trigger
            .as_ref()
            .ok_or("Push delivery requires a fixture-owned trigger")?;
        if !trigger
            .path
            .strip_prefix("/__fixture-push-")
            .is_some_and(|suffix| uuid::Uuid::parse_str(suffix).is_ok())
            || !trigger.path.starts_with('/')
            || trigger.path.starts_with("//")
            || trigger.path.contains(['?', '#', '\\', '\r', '\n'])
            || trigger.token.is_empty()
            || trigger.token.len() > 1024
            || HeaderValue::from_str(&trigger.token).is_err()
        {
            return Err("Invalid private fixture push trigger");
        }
    }
    Ok(base)
}
fn now_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis()
        .min(u64::MAX as u128) as u64
}
fn read_manifest(path: &PathBuf) -> Result<Manifest, &'static str> {
    let metadata = fs::symlink_metadata(path).map_err(|_| "Cannot read private manifest")?;
    if !metadata.is_file() || metadata.file_type().is_symlink() || metadata.len() > MAX_MANIFEST {
        return Err("Manifest is not a bounded regular file");
    }
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        if metadata.permissions().mode() & 0o077 != 0 {
            return Err("Private manifest must not be group/world readable");
        }
    }
    let mut bytes = Vec::new();
    File::open(path)
        .map_err(|_| "Cannot read private manifest")?
        .take(MAX_MANIFEST + 1)
        .read_to_end(&mut bytes)
        .map_err(|_| "Cannot read private manifest")?;
    if bytes.len() as u64 > MAX_MANIFEST {
        return Err("Manifest exceeds size bound");
    }
    serde_json::from_slice(&bytes).map_err(|_| "Invalid private authentication manifest")
}
#[derive(Default)]
struct Stats {
    settled: AtomicU64,
    ready: AtomicU64,
    live: AtomicU64,
    peak: AtomicU64,
    closed_early: AtomicU64,
    errors: AtomicU64,
    notifications: AtomicU64,
    verified: AtomicU64,
    pongs: AtomicU64,
    close_sent: AtomicU64,
    ready_micros: AtomicU64,
    max_ready_micros: AtomicU64,
    foreign: AtomicU64,
    connect_errors: AtomicU64,
    protocol_errors: AtomicU64,
    read_errors: AtomicU64,
    simulated_ready: AtomicU64,
    simulated_live: AtomicU64,
}
impl Stats {
    fn error(&self, kind: &str) {
        self.errors.fetch_add(1, Ordering::Relaxed);
        match kind {
            "connect" => {
                self.connect_errors.fetch_add(1, Ordering::Relaxed);
            }
            "foreign" => {
                self.foreign.fetch_add(1, Ordering::Relaxed);
            }
            "read" => {
                self.read_errors.fetch_add(1, Ordering::Relaxed);
            }
            _ => {
                self.protocol_errors.fetch_add(1, Ordering::Relaxed);
            }
        }
    }
    fn json(&self) -> Value {
        macro_rules! n {
            ($field:ident) => {
                self.$field.load(Ordering::Relaxed)
            };
        }
        json!({"settled":n!(settled),"ready":n!(ready),"live":n!(live),"peakLive":n!(peak),"closedBeforeStop":n!(closed_early),"errors":n!(errors),"stateChangeFrames":n!(notifications),"verifiedPushRecipients":n!(verified),"pongs":n!(pongs),"intentionalCloseFramesSent":n!(close_sent),"readyTotalMicros":n!(ready_micros),"readyMaxMicros":n!(max_ready_micros),"errorClasses":{"connect":n!(connect_errors),"foreignAccount":n!(foreign),"read":n!(read_errors),"protocol":n!(protocol_errors)}})
    }
}
#[derive(Clone, Default)]
struct Control {
    stop: bool,
    marker: Option<Arc<str>>,
}
struct Live(Arc<Stats>, bool, bool);
impl Drop for Live {
    fn drop(&mut self) {
        self.0.live.fetch_sub(1, Ordering::Relaxed);
        if self.2 {
            self.0.simulated_live.fetch_sub(1, Ordering::Relaxed);
        }
        if !self.1 {
            self.0.closed_early.fetch_add(1, Ordering::Relaxed);
        }
    }
}
fn own_email(frame: &Value, account: &str) -> Result<Option<String>, &'static str> {
    if frame.get("@type").and_then(Value::as_str) != Some("StateChange") {
        return Ok(None);
    }
    let changed = frame
        .get("changed")
        .and_then(Value::as_object)
        .ok_or("protocol")?;
    if changed.keys().any(|key| key != account) {
        return Err("foreign");
    }
    let email = changed
        .get(account)
        .and_then(|types| types.get("Email"))
        .and_then(Value::as_str);
    if email.is_some_and(|email| email.len() > 512) {
        return Err("protocol");
    }
    Ok(email.map(str::to_owned))
}
fn action_proof(frame: &Value, account: &str) -> bool {
    if frame.get("id").and_then(Value::as_str) != Some("load-proof")
        || frame.get("status").and_then(Value::as_u64) != Some(200)
    {
        return false;
    }
    let Some(body) = frame
        .get("body")
        .and_then(Value::as_str)
        .and_then(|body| serde_json::from_str::<Value>(body).ok())
    else {
        return false;
    };
    body.get("methodResponses")
        .and_then(Value::as_array)
        .is_some_and(|responses| {
            responses.len() == 1
                && responses[0][0] == "Mailbox/get"
                && responses[0][1]["accountId"] == account
                && responses[0][1]["list"].is_array()
        })
}
async fn connection(
    base: reqwest::Url,
    principal: Principal,
    protocol: Protocol,
    timeout: u64,
    permit: tokio::sync::OwnedSemaphorePermit,
    mut control: watch::Receiver<Control>,
    stats: Arc<Stats>,
) {
    let started = Instant::now();
    let mut last_email = None;
    let admission=tokio::time::timeout(Duration::from_secs(timeout),async{
  let mut url=base.join(if protocol==Protocol::Push{"jmap/ws"}else{"jmap/actions"}).map_err(|_|"protocol")?;url.set_scheme(if base.scheme()=="https"{"wss"}else{"ws"}).map_err(|_|"protocol")?;
  let mut request=url.as_str().into_client_request().map_err(|_|"protocol")?;
  let mut cookie=HeaderValue::from_str(&principal.cookie).map_err(|_|"protocol")?;cookie.set_sensitive(true);request.headers_mut().insert(header::COOKIE,cookie);request.headers_mut().insert(header::ORIGIN,base.origin().ascii_serialization().parse().map_err(|_|"protocol")?);
  request.headers_mut().insert(header::USER_AGENT,HeaderValue::from_static("Mozilla/5.0 MailwomanFixture/1.0"));
  if let Some(address)=&principal.client_address{request.headers_mut().insert("x-forwarded-for",HeaderValue::from_str(address).map_err(|_|"protocol")?);}
  if protocol==Protocol::Push{request.headers_mut().insert(header::SEC_WEBSOCKET_PROTOCOL,HeaderValue::from_static("jmap"));}
  let config=WebSocketConfig::default().read_buffer_size(4096).write_buffer_size(0).max_write_buffer_size(65536).max_message_size(Some(65536)).max_frame_size(Some(65536));
  let (mut socket,response)=connect_async_with_config(request,Some(config),true).await.map_err(|_|"connect")?;
  if protocol==Protocol::Push{
   if response.headers().get(header::SEC_WEBSOCKET_PROTOCOL).and_then(|value|value.to_str().ok())!=Some("jmap"){return Err("protocol");}
   socket.send(Message::Ping(b"mw-load-ready-v1".to_vec().into())).await.map_err(|_|"connect")?;
  }
  let mut actions_ready=false;
  for _ in 0..128 {
   match socket.next().await.ok_or("connect")?.map_err(|_|"connect")?{
    Message::Pong(payload) if protocol==Protocol::Push&&payload.as_ref()==b"mw-load-ready-v1"=>{return Ok(socket);},
    Message::Ping(payload)=>socket.send(Message::Pong(payload)).await.map_err(|_|"connect")?,
    Message::Text(text)=>{
     let frame:Value=serde_json::from_str(&text).map_err(|_|"protocol")?;
     if protocol==Protocol::Push{if frame["@type"]=="StateChange"{stats.notifications.fetch_add(1,Ordering::Relaxed);}if let Some(email)=own_email(&frame,&principal.account_id)?{last_email=Some(email);}}
     else if !actions_ready&&frame["type"]=="ready"&&frame["protocol"]=="mailwoman-actions-v1"{
      actions_ready=true;let body=json!({"using":["urn:ietf:params:jmap:core","urn:ietf:params:jmap:mail"],"methodCalls":[["Mailbox/get",{"accountId":principal.account_id},"proof"]]});
      socket.send(Message::Text(json!({"id":"load-proof","method":"POST","path":"/jmap/api","headers":{"content-type":"application/json","x-csrf-token":principal.csrf_token},"body":body.to_string()}).to_string().into())).await.map_err(|_|"connect")?;
     }else if actions_ready&&action_proof(&frame,&principal.account_id){return Ok(socket);}else if frame["id"]=="load-proof"{return Err("protocol");}
    },Message::Close(_)=>return Err("connect"),_=>{}
   }
  }Err("protocol")
 }).await;
    stats.settled.fetch_add(1, Ordering::Relaxed);
    drop(permit);
    let mut socket = match admission {
        Ok(Ok(socket)) => socket,
        Ok(Err(reason)) => {
            stats.error(reason);
            return;
        }
        Err(_) => {
            stats.error("connect");
            return;
        }
    };
    stats.ready.fetch_add(1, Ordering::Relaxed);
    if principal.client_address.is_some() {
        stats.simulated_ready.fetch_add(1, Ordering::Relaxed);
        stats.simulated_live.fetch_add(1, Ordering::Relaxed);
    }
    let live = stats.live.fetch_add(1, Ordering::Relaxed) + 1;
    stats.peak.fetch_max(live, Ordering::Relaxed);
    let mut live_guard = Live(stats.clone(), false, principal.client_address.is_some());
    let micros = started.elapsed().as_micros().min(u64::MAX as u128) as u64;
    stats.ready_micros.fetch_add(micros, Ordering::Relaxed);
    stats.max_ready_micros.fetch_max(micros, Ordering::Relaxed);
    let mut verified = false;
    let mut heartbeat = tokio::time::interval(Duration::from_secs(15));
    heartbeat.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Delay);
    loop {
        let current = control.borrow().clone();
        if !verified
            && current
                .marker
                .as_deref()
                .is_some_and(|marker| last_email.as_deref() == Some(marker))
        {
            verified = true;
            stats.verified.fetch_add(1, Ordering::Relaxed);
        }
        if current.stop {
            live_guard.1 = true;
            if tokio::time::timeout(Duration::from_secs(2), socket.close(None))
                .await
                .is_ok_and(|result| result.is_ok())
            {
                stats.close_sent.fetch_add(1, Ordering::Relaxed);
            }
            return;
        }
        tokio::select! {
         changed=control.changed()=>{if changed.is_err(){return;}},
         _=heartbeat.tick()=>{if socket.send(Message::Ping(b"mw-load-hold".to_vec().into())).await.is_err(){stats.error("read");return;}},
         frame=socket.next()=>match frame{
          Some(Ok(Message::Pong(_)))=>{stats.pongs.fetch_add(1,Ordering::Relaxed);},Some(Ok(Message::Ping(payload)))=>{if socket.send(Message::Pong(payload)).await.is_err(){stats.error("read");return;}},
          Some(Ok(Message::Text(text)))=>{let Ok(frame)=serde_json::from_str::<Value>(&text)else{stats.error("protocol");return;};if protocol==Protocol::Push{if frame["@type"]=="StateChange"{stats.notifications.fetch_add(1,Ordering::Relaxed);}match own_email(&frame,&principal.account_id){Ok(Some(email))=>last_email=Some(email),Ok(None)=>{},Err(reason)=>{stats.error(reason);return;}}}},
          Some(Ok(Message::Close(_)))|None=>{return;},Some(Err(_))=>{stats.error("read");return;},_=>{}
         }
        }
    }
}
async fn trigger(
    base: &reqwest::Url,
    hook: &PushTrigger,
    config: &Config,
) -> Result<Arc<str>, &'static str> {
    let url = base
        .join(hook.path.trim_start_matches('/'))
        .map_err(|_| "push-trigger")?;
    let client = reqwest::Client::builder()
        .no_proxy()
        .redirect(reqwest::redirect::Policy::none())
        .timeout(Duration::from_secs(
            config.connections as u64 * config.trigger_interval / 1000 + 30,
        ))
        .build()
        .map_err(|_| "push-trigger")?;
    let mut response = client
        .post(url)
        .header("x-fixture-token", &hook.token)
        .json(&json!({"count":config.connections,"intervalMs":config.trigger_interval}))
        .send()
        .await
        .map_err(|_| "push-trigger")?;
    if !response.status().is_success() {
        return Err("push-trigger");
    }
    let mut bytes = Vec::new();
    while let Some(chunk) = response.chunk().await.map_err(|_| "push-trigger")? {
        if bytes.len() + chunk.len() > 65536 {
            return Err("push-trigger");
        }
        bytes.extend_from_slice(&chunk);
    }
    let value: Value = serde_json::from_slice(&bytes).map_err(|_| "push-trigger")?;
    let marker = value
        .get("marker")
        .and_then(Value::as_str)
        .filter(|marker| !marker.is_empty() && marker.len() <= 128)
        .ok_or("push-trigger")?;
    if value.get("count").and_then(Value::as_u64) != Some(config.connections as u64) {
        return Err("push-trigger");
    }
    Ok(Arc::from(marker))
}
async fn campaign(
    config: &Config,
    protocol: Protocol,
    manifest: &Manifest,
    base: reqwest::Url,
    stats: Arc<Stats>,
) -> Result<Value, &'static str> {
    let (control, receiver) = watch::channel(Control::default());
    let semaphore = Arc::new(Semaphore::new(config.pending));
    let mut tasks = JoinSet::new();
    let started = Instant::now();
    let mut pace = tokio::time::interval(Duration::from_secs_f64(1.0 / config.ramp as f64));
    pace.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Delay);
    for principal in manifest.principals.iter().take(config.connections) {
        pace.tick().await;
        let permit = semaphore
            .clone()
            .acquire_owned()
            .await
            .map_err(|_| "ramp")?;
        tasks.spawn(connection(
            base.clone(),
            principal.clone(),
            protocol,
            config.timeout,
            permit,
            receiver.clone(),
            stats.clone(),
        ));
    }
    while stats.settled.load(Ordering::Relaxed) < config.connections as u64 {
        tokio::time::sleep(Duration::from_millis(20)).await;
    }
    let ramp_seconds = started.elapsed().as_secs_f64();
    tokio::time::sleep(Duration::from_secs(config.warm)).await;
    let mut triggered = false;
    let mut trigger_error = false;
    if protocol == Protocol::Push {
        match trigger(
            &base,
            manifest.push_trigger.as_ref().ok_or("push-trigger")?,
            config,
        )
        .await
        {
            Ok(marker) => {
                control
                    .send(Control {
                        stop: false,
                        marker: Some(marker),
                    })
                    .map_err(|_| "control")?;
                triggered = true;
            }
            Err(_) => {
                stats.error("protocol");
                trigger_error = true;
            }
        }
    }
    let steady_started_at_ms = now_ms();
    let steady_until_ms = steady_started_at_ms.saturating_add(config.hold * 1000);
    if let Some(path) = &config.ready_file {
        if stats.ready.load(Ordering::Relaxed) == config.connections as u64
            && stats.live.load(Ordering::Relaxed) == config.connections as u64
            && stats.errors.load(Ordering::Relaxed) == 0
            && (protocol == Protocol::Actions || triggered && !trigger_error)
        {
            let receipt = json!({"schema":1,"phase":"steady-hold","startedAtMs":steady_started_at_ms,"untilMs":steady_until_ms,"authenticatedDistinctPrincipals":config.connections,"simulatedDistinctClients":stats.simulated_live.load(Ordering::Relaxed)});
            if write_report(path, &receipt).is_err() {
                stats.error("protocol");
            }
        }
    }
    tokio::time::sleep(Duration::from_secs(config.hold)).await;
    let steady_ended_at_ms = now_ms();
    let live_at_end = stats.live.load(Ordering::Relaxed);
    let verified = stats.verified.load(Ordering::Relaxed);
    let active_distinct = live_at_end;
    let simulated_at_end = stats.simulated_live.load(Ordering::Relaxed);
    control.send_modify(|value| value.stop = true);
    let drained = tokio::time::timeout(Duration::from_secs(30), async {
        while let Some(result) = tasks.join_next().await {
            if result.is_err() {
                stats.error("protocol");
            }
        }
    })
    .await
    .is_ok();
    if !drained {
        tasks.abort_all();
        while tasks.join_next().await.is_some() {}
    }
    let passed = stats.ready.load(Ordering::Relaxed) == config.connections as u64
        && active_distinct == config.connections as u64
        && stats.errors.load(Ordering::Relaxed) == 0
        && stats.closed_early.load(Ordering::Relaxed) == 0
        && drained
        && (protocol == Protocol::Actions
            || (triggered && !trigger_error && verified == config.connections as u64));
    Ok(
        json!({"passed":passed,"rampSeconds":ramp_seconds,"steadyHoldSeconds":config.hold,"steadyStartedAtMs":steady_started_at_ms,"steadyUntilMs":steady_until_ms,"steadyEndedAtMs":steady_ended_at_ms,"warmSeconds":config.warm,"liveAtHoldEnd":live_at_end,"authenticatedDistinctPrincipalsReady":stats.ready.load(Ordering::Relaxed),"authenticatedDistinctPrincipalsAtHoldEnd":active_distinct,"simulatedDistinctClientsReady":stats.simulated_ready.load(Ordering::Relaxed),"simulatedDistinctClientsAtHoldEnd":simulated_at_end,"pushDeliveryRequired":protocol==Protocol::Push,"pushDeliveryVerified":protocol==Protocol::Push&&verified==config.connections as u64,"pushTriggerFailed":trigger_error,"stateChangeAccountIsolationChecked":protocol==Protocol::Push,"cleanupDrained":drained,"elapsedSeconds":started.elapsed().as_secs_f64()}),
    )
}
fn write_report(path: &PathBuf, value: &Value) -> Result<(), &'static str> {
    let parent = path
        .parent()
        .filter(|path| !path.as_os_str().is_empty())
        .unwrap_or(std::path::Path::new("."));
    fs::create_dir_all(parent).map_err(|_| "Cannot create report directory")?;
    let temp = parent.join(format!(".ws-load-report-{}.tmp", std::process::id()));
    let mut options = OpenOptions::new();
    options.write(true).create_new(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        options.mode(0o600);
    }
    let mut file = options.open(&temp).map_err(|_| "Cannot write report")?;
    file.write_all(&serde_json::to_vec_pretty(value).map_err(|_| "Cannot serialize report")?)
        .map_err(|_| "Cannot write report")?;
    file.sync_all().map_err(|_| "Cannot persist report")?;
    fs::rename(temp, path).map_err(|_| "Cannot publish report")
}
fn main() -> std::process::ExitCode {
    if std::env::args().any(|argument| argument == "--help") {
        println!(
            "websocket-load --manifest PRIVATE.json --report REPORT.json [--protocol push|actions] [--connections 10000] [--ramp-per-second 100] [--max-pending 128] [--warm-seconds 10] [--hold-seconds 60] [--connect-timeout-seconds 30] [--workers 2] [--trigger-interval-ms 10] [--ready-file PRIVATE.json]"
        );
        return std::process::ExitCode::SUCCESS;
    }
    let result = (|| -> Result<bool, &'static str> {
        let (config, protocol) = Config::parse(std::env::args().skip(1))?;
        if config.ready_file.as_ref().is_some_and(|path| path.exists()) {
            return Err("Readiness receipt must be a fresh path");
        }
        let manifest = read_manifest(&config.manifest)?;
        let base = validate_manifest(&manifest, &config, protocol)?;
        let stats = Arc::new(Stats::default());
        let runtime = tokio::runtime::Builder::new_multi_thread()
            .worker_threads(config.workers)
            .enable_all()
            .build()
            .map_err(|_| "Cannot start generator runtime")?;
        let outcome = runtime.block_on(async {
            tokio::time::timeout(
                Duration::from_secs(3600),
                campaign(&config, protocol, &manifest, base, stats.clone()),
            )
            .await
        });
        let summary = match outcome {
            Ok(Ok(summary)) => summary,
            Ok(Err(reason)) => json!({"passed":false,"failureClass":reason}),
            Err(_) => json!({"passed":false,"failureClass":"campaign-timeout"}),
        };
        let passed = summary["passed"] == true;
        let report = json!({"schema":1,"scenario":"independent unique-principal authenticated WebSocket fixture","protocol":if protocol==Protocol::Push{"jmap-state-change"}else{"mailwoman-actions-v1"},"backend":manifest.backend,"buildProfile":manifest.build_profile,"requestedConnections":config.connections,"inputDistinctPrincipals":config.connections,"clientModel":{"simulationMode":if manifest.client_simulation.is_some(){"fixture-loopback-trusted-proxy"}else{"single-loopback-peer"},"configuredSimulatedClients":if manifest.client_simulation.is_some(){config.connections}else{0},"distinctPhysicalGeneratorHostsMeasured":false},"rampPerSecond":config.ramp,"maximumPendingConnects":config.pending,"generatorWorkers":config.workers,"generatorMemoryBound":"64KiB frame/message limit;4KiB readbuffer,64KiB writebuffer cap and one512byte Emailstate per socket; task/HTTP/TLS/OS overhead additional; no raw frames retained","actionsOnlyDoesNotProvePush":protocol==Protocol::Actions,"summary":summary,"counters":stats.json()});
        write_report(&config.report, &report)?;
        Ok(passed)
    })();
    match result {
        Ok(true) => {
            println!("WebSocket fixture PASS (credential-free report written)");
            std::process::ExitCode::SUCCESS
        }
        Ok(false) => {
            eprintln!("WebSocket fixture FAIL (see credential-free report)");
            std::process::ExitCode::FAILURE
        }
        Err(reason) => {
            eprintln!("WebSocket fixture refused: {reason}");
            std::process::ExitCode::FAILURE
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn config() -> Config {
        Config::parse(
            [
                "--manifest",
                "private.json",
                "--report",
                "report.json",
                "--connections",
                "2",
            ]
            .map(String::from),
        )
        .unwrap()
        .0
    }
    fn manifest() -> Manifest {
        Manifest {
            base: "http://127.0.0.1:1234/".into(),
            engine_mode: "native".into(),
            backend: "sqlite".into(),
            build_profile: "debug".into(),
            principals: (0..2)
                .map(|n| Principal {
                    cookie: format!("mw_session=private-{n}"),
                    csrf_token: "csrf".into(),
                    account_id: format!("account-{n}"),
                    owner_account_id: None,
                    client_address: Some(format!("198.18.0.{}", n + 1)),
                })
                .collect(),
            push_trigger: Some(PushTrigger {
                path: "/__fixture-push-29d63f0a-ef7d-4144-86c9-811207420cc4".into(),
                token: "private".into(),
            }),
            client_simulation: Some(ClientSimulation {
                mode: "fixture-loopback-trusted-proxy".into(),
                client_count: 2,
            }),
        }
    }
    #[test]
    fn parser_bounds_pending_users_and_campaign_lifetime() {
        for (flag, value) in [
            ("--connections", "10001"),
            ("--max-pending", "513"),
            ("--workers", "0"),
            ("--hold-seconds", "1801"),
            ("--protocol", "wrong"),
        ] {
            assert!(
                Config::parse(["--manifest", "p", "--report", "r", flag, value].map(String::from))
                    .is_err()
            );
        }
    }
    #[test]
    fn distinct_accounts_credentials_and_origin_are_required() {
        let config = config();
        let mut input = manifest();
        assert!(validate_manifest(&input, &config, Protocol::Push).is_ok());
        input.principals[1].account_id = input.principals[0].account_id.clone();
        assert!(validate_manifest(&input, &config, Protocol::Push).is_err());
        input = manifest();
        input.principals[1].cookie = input.principals[0].cookie.clone();
        assert!(validate_manifest(&input, &config, Protocol::Push).is_err());
        input = manifest();
        input.base = "http://name:secret@localhost/".into();
        assert!(validate_manifest(&input, &config, Protocol::Push).is_err());
        input = manifest();
        input.push_trigger = None;
        assert!(validate_manifest(&input, &config, Protocol::Push).is_err());
        assert!(validate_manifest(&input, &config, Protocol::Actions).is_ok());
    }
    #[test]
    fn client_simulation_is_explicit_bounded_and_cannot_assert_external_addresses() {
        let config = config();
        for address in [
            "8.8.8.8",
            "198.18.00.2",
            "198.18.0.1",
            "198.18.0.2\r\nInjected: true",
        ] {
            let mut input = manifest();
            input.principals[1].client_address = Some(address.into());
            assert!(validate_manifest(&input, &config, Protocol::Push).is_err());
        }
        let mut input = manifest();
        input.client_simulation = None;
        assert!(validate_manifest(&input, &config, Protocol::Push).is_err());
        for principal in &mut input.principals {
            principal.client_address = None;
        }
        assert!(validate_manifest(&input, &config, Protocol::Push).is_ok());
        input = manifest();
        input.client_simulation.as_mut().unwrap().client_count = 1;
        assert!(validate_manifest(&input, &config, Protocol::Push).is_err());
        input = manifest();
        input.base = "https://example.invalid/".into();
        assert!(validate_manifest(&input, &config, Protocol::Push).is_err());
        input = manifest();
        input.principals[1].cookie = format!("{}; mw_csrf=different", input.principals[0].cookie);
        assert!(validate_manifest(&input, &config, Protocol::Push).is_err());
    }
    #[test]
    fn state_change_proof_rejects_other_accounts_and_oversized_tokens() {
        assert_eq!(
            own_email(
                &json!({"@type":"StateChange","changed":{"owned":{"Email":"marker"}}}),
                "owned"
            )
            .unwrap()
            .as_deref(),
            Some("marker")
        );
        assert!(
            own_email(
                &json!({"@type":"StateChange","changed":{"other":{"Email":"marker"}}}),
                "owned"
            )
            .is_err()
        );
        assert!(
            own_email(
                &json!({"@type":"StateChange","changed":{"owned":{"Email":"x".repeat(513)}}}),
                "owned"
            )
            .is_err()
        );
        assert!(
            own_email(&json!({"@type":"ping"}), "owned")
                .unwrap()
                .is_none()
        );
    }
    #[test]
    fn actions_proof_requires_successful_authenticated_account_response() {
        let frame = json!({"id":"load-proof","status":200,"body":json!({"methodResponses":[["Mailbox/get",{"accountId":"owned","list":[]},"proof"]]}).to_string()});
        assert!(action_proof(&frame, "owned"));
        assert!(!action_proof(&frame, "other"));
        let mut failed = frame;
        failed["status"] = 403.into();
        assert!(!action_proof(&failed, "owned"));
    }
    async fn fake_push(foreign: bool) -> (reqwest::Url, tokio::task::JoinHandle<()>) {
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let address = listener.local_addr().unwrap();
        let task = tokio::spawn(async move {
            let (stream, _) = listener.accept().await.unwrap();
            let mut socket=tokio_tungstenite::accept_hdr_async(stream,|request:&tokio_tungstenite::tungstenite::handshake::server::Request,mut response:tokio_tungstenite::tungstenite::handshake::server::Response| {
                assert!(request.headers().get(header::COOKIE).is_some());assert!(request.headers().get(header::ORIGIN).is_some());
                assert_eq!(request.headers().get("x-forwarded-for").unwrap(), "198.18.0.1");
                response.headers_mut().insert(header::SEC_WEBSOCKET_PROTOCOL,HeaderValue::from_static("jmap"));Ok(response)
            }).await.unwrap();
            let account = if foreign {
                "other-account"
            } else {
                "account-0"
            };
            socket
                .send(Message::Text(
                    json!({"@type":"StateChange","changed":{account:{"Email":"fixture-proof"}}})
                        .to_string()
                        .into(),
                ))
                .await
                .unwrap();
            while let Some(Ok(message)) = socket.next().await {
                match message {
                    Message::Ping(data) => {
                        if socket.send(Message::Pong(data)).await.is_err() {
                            break;
                        }
                    }
                    Message::Close(_) => {
                        let _ = socket.flush().await;
                        break;
                    }
                    _ => {}
                }
            }
        });
        (
            reqwest::Url::parse(&format!("http://{address}/")).unwrap(),
            task,
        )
    }
    #[tokio::test]
    async fn local_fake_socket_proves_ready_delivery_and_bounded_cleanup() {
        let (base, server) = fake_push(false).await;
        let stats = Arc::new(Stats::default());
        let (control, receiver) = watch::channel(Control {
            stop: false,
            marker: Some(Arc::from("fixture-proof")),
        });
        let semaphore = Arc::new(Semaphore::new(1));
        let permit = semaphore.acquire_owned().await.unwrap();
        let task = tokio::spawn(connection(
            base,
            manifest().principals.remove(0),
            Protocol::Push,
            2,
            permit,
            receiver,
            stats.clone(),
        ));
        tokio::time::timeout(Duration::from_secs(3), async {
            while stats.ready.load(Ordering::Relaxed) != 1
                || stats.verified.load(Ordering::Relaxed) != 1
            {
                tokio::time::sleep(Duration::from_millis(5)).await;
            }
        })
        .await
        .unwrap();
        assert_eq!(stats.live.load(Ordering::Relaxed), 1);
        assert_eq!(stats.simulated_ready.load(Ordering::Relaxed), 1);
        assert_eq!(stats.simulated_live.load(Ordering::Relaxed), 1);
        assert_eq!(stats.errors.load(Ordering::Relaxed), 0);
        control.send_modify(|value| value.stop = true);
        tokio::time::timeout(Duration::from_secs(3), task)
            .await
            .unwrap()
            .unwrap();
        tokio::time::timeout(Duration::from_secs(3), server)
            .await
            .unwrap()
            .unwrap();
        assert_eq!(stats.live.load(Ordering::Relaxed), 0);
        assert_eq!(stats.close_sent.load(Ordering::Relaxed), 1);
        let serialized = stats.json().to_string();
        assert!(!serialized.contains("private-0"));
        assert!(!serialized.contains("account-0"));
    }
    #[tokio::test]
    async fn local_fake_socket_rejects_cross_account_frames_without_a_false_ready() {
        let (base, server) = fake_push(true).await;
        let stats = Arc::new(Stats::default());
        let (_control, receiver) = watch::channel(Control::default());
        let permit = Arc::new(Semaphore::new(1)).acquire_owned().await.unwrap();
        connection(
            base,
            manifest().principals.remove(0),
            Protocol::Push,
            2,
            permit,
            receiver,
            stats.clone(),
        )
        .await;
        assert_eq!(stats.ready.load(Ordering::Relaxed), 0);
        assert_eq!(stats.foreign.load(Ordering::Relaxed), 1);
        assert_eq!(stats.errors.load(Ordering::Relaxed), 1);
        tokio::time::timeout(Duration::from_secs(3), server)
            .await
            .unwrap()
            .unwrap();
    }
}
