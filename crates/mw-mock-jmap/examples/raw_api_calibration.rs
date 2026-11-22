//! Feature-only raw Axum/Hyper ceiling fixture. No Store, app router or users.
//! The parent supplies a cryptographic token. Never print it.
use axum::{
    extract::State, http::{HeaderMap, Method, StatusCode},
    response::{IntoResponse, Response}, routing::{get, post}, Json, Router,
};
use serde_json::json;
use std::{
    io::Write, path::PathBuf,
    sync::{atomic::{AtomicU64, Ordering}, Arc, Mutex},
    time::{Duration, Instant, SystemTime, UNIX_EPOCH},
};
use tokio::sync::watch;
const TINY: &str = r#"{"ok":true}"#;
struct Config { token: String, handoff: PathBuf, workers: usize, lifetime: Duration }
fn bounded(value: Option<&str>, default: u64, min: u64, max: u64) -> Result<u64, &'static str> {
    let value = match value {
        Some(raw) => raw.parse().map_err(|_| "invalid raw calibration integer")?,
        None => default,
    };
    if !(min..=max).contains(&value) { return Err("raw calibration bound exceeded"); }
    Ok(value)
}
fn token_valid(value: &str) -> bool {
    value.len() == 64 && value.bytes().all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
}
fn opt_in(value: Option<&str>) -> Result<(), &'static str> {
    if value == Some("1") { Ok(()) } else { Err("explicit raw Rust calibration opt-in required") }
}
fn optional(name: &str) -> Result<Option<String>, &'static str> {
    match std::env::var(name) {
        Ok(value) => Ok(Some(value)),
        Err(std::env::VarError::NotPresent) => Ok(None),
        Err(_) => Err("raw calibration option must be Unicode"),
    }
}
impl Config {
    fn from_environment() -> Result<Self, &'static str> {
        opt_in(optional("MW_RUN_RAW_RUST_API_CALIBRATION")?.as_deref())?;
        let token = optional("MW_RAW_API_CALIBRATION_TOKEN")?.ok_or("private calibration token required")?;
        if !token_valid(&token) { return Err("invalid private calibration token"); }
        let handoff = std::env::var_os("MW_RAW_API_CALIBRATION_HANDOFF")
            .map(PathBuf::from).filter(|p| p.is_absolute()).ok_or("absolute private handoff path required")?;
        let workers = optional("MW_RAW_API_RUNTIME_WORKERS")?;
        let seconds = optional("MW_RAW_API_LIFETIME_SECONDS")?;
        Ok(Self {
            token, handoff,
            workers: bounded(workers.as_deref(), 4, 1, 32)? as usize,
            lifetime: Duration::from_secs(bounded(seconds.as_deref(), 120, 10, 900)?),
        })
    }
}
struct Shared {
    token: String, expires: Instant, reset: Mutex<Instant>,
    requests: AtomicU64, bytes: AtomicU64, workers: usize, stop: watch::Sender<bool>,
}
type SharedState = Arc<Shared>;
fn allowed(state: &Shared, headers: &HeaderMap) -> bool {
    let supplied = headers.get("x-calibration-token").map(|v| v.as_bytes()).unwrap_or_default();
    supplied.len() == 64 && supplied.iter().zip(state.token.as_bytes())
        .fold(0u8, |difference, (a, b)| difference | (a ^ b)) == 0
}
fn gate(state: &Shared, headers: &HeaderMap) -> Option<Response> {
    if Instant::now() >= state.expires { Some(StatusCode::GONE.into_response()) }
    else if !allowed(state, headers) { Some(StatusCode::NOT_FOUND.into_response()) }
    else { None }
}
async fn tiny(State(state): State<SharedState>, method: Method, headers: HeaderMap) -> Response {
    if let Some(response) = gate(&state, &headers) { return response; }
    if method != Method::GET { return StatusCode::NOT_FOUND.into_response(); }
    state.requests.fetch_add(1, Ordering::Relaxed);
    state.bytes.fetch_add(TINY.len() as u64, Ordering::Relaxed);
    ([("content-type", "application/json"), ("cache-control", "no-store")], TINY).into_response()
}
async fn empty(State(state): State<SharedState>, method: Method, headers: HeaderMap) -> Response {
    if let Some(response) = gate(&state, &headers) { return response; }
    if method != Method::GET { return StatusCode::NOT_FOUND.into_response(); }
    state.requests.fetch_add(1, Ordering::Relaxed);
    (StatusCode::NO_CONTENT, [("cache-control", "no-store")]).into_response()
}
async fn reset(State(state): State<SharedState>, headers: HeaderMap) -> Response {
    if let Some(response) = gate(&state, &headers) { return response; }
    let Ok(mut start) = state.reset.lock() else { return StatusCode::SERVICE_UNAVAILABLE.into_response(); };
    state.requests.store(0, Ordering::Relaxed); state.bytes.store(0, Ordering::Relaxed);
    *start = Instant::now(); StatusCode::NO_CONTENT.into_response()
}
async fn metrics(State(state): State<SharedState>, headers: HeaderMap) -> Response {
    if let Some(response) = gate(&state, &headers) { return response; }
    let Ok(start) = state.reset.lock() else { return StatusCode::SERVICE_UNAVAILABLE.into_response(); };
    Json(json!({
        "schema": 1, "kind": "raw-rust-axum-api",
        "handlerInvocations": state.requests.load(Ordering::Relaxed),
        "handlerBytesOffered": state.bytes.load(Ordering::Relaxed),
        "elapsedSeconds": start.elapsed().as_secs_f64(), "runtimeWorkers": state.workers,
        "cpuUserMs": null, "cpuSystemMs": null, "rssBytes": null
    })).into_response()
}
async fn stop(State(state): State<SharedState>, headers: HeaderMap) -> Response {
    if !allowed(&state, &headers) { return StatusCode::NOT_FOUND.into_response(); }
    let _ = state.stop.send(true); StatusCode::NO_CONTENT.into_response()
}
async fn stopped(mut signal: watch::Receiver<bool>) {
    while !*signal.borrow() { if signal.changed().await.is_err() { break; } }
}
struct Running {
    data: String, control: String, expires_at_ms: u64, state: SharedState,
    data_task: tokio::task::JoinHandle<Result<(), std::io::Error>>,
    control_task: tokio::task::JoinHandle<Result<(), std::io::Error>>,
    timer: tokio::task::JoinHandle<()>,
}
async fn start(config: &Config) -> Result<Running, &'static str> {
    let data = tokio::net::TcpListener::bind("127.0.0.1:0").await.map_err(|_| "raw data listener failed")?;
    let control = tokio::net::TcpListener::bind("127.0.0.1:0").await.map_err(|_| "raw control listener failed")?;
    let data_base = format!("http://{}", data.local_addr().map_err(|_| "raw address unavailable")?);
    let control_base = format!("http://{}", control.local_addr().map_err(|_| "raw address unavailable")?);
    let (stop, signal) = watch::channel(false); let now = Instant::now();
    let state = Arc::new(Shared {
        token: config.token.clone(), expires: now + config.lifetime, reset: Mutex::new(now),
        requests: AtomicU64::new(0), bytes: AtomicU64::new(0), workers: config.workers, stop: stop.clone(),
    });
    let data_router = Router::new().route("/json", get(tiny)).route("/empty", get(empty)).with_state(state.clone());
    let control_router = Router::new().route("/__reset", post(reset)).route("/__metrics", get(metrics))
        .route("/__stop", post(self::stop)).with_state(state.clone());
    let data_signal = signal.clone();
    let data_task = tokio::spawn(async move { axum::serve(data, data_router).with_graceful_shutdown(stopped(data_signal)).await });
    let control_task = tokio::spawn(async move { axum::serve(control, control_router).with_graceful_shutdown(stopped(signal)).await });
    let lifetime = config.lifetime;
    let timer = tokio::spawn(async move { tokio::time::sleep(lifetime).await; let _ = stop.send(true); });
    let expires_at_ms = SystemTime::now().duration_since(UNIX_EPOCH).map_err(|_| "system clock unavailable")?.as_millis() as u64
        + config.lifetime.as_millis() as u64;
    Ok(Running { data: data_base, control: control_base, expires_at_ms, state, data_task, control_task, timer })
}
async fn run(config: Config) -> Result<(), &'static str> {
    let running = start(&config).await?;
    let bytes = serde_json::to_vec(&json!({
        "schema": 1, "kind": "raw-rust-axum-api", "dataBase": running.data, "controlBase": running.control,
        "token": config.token, "expiresAtMs": running.expires_at_ms, "runtimeWorkers": config.workers,
    })).map_err(|_| "handoff serialization failed")?;
    if bytes.len() > 4096 { return Err("handoff bound exceeded"); }
    let mut options = std::fs::OpenOptions::new(); options.write(true).create_new(true);
    #[cfg(unix)] { use std::os::unix::fs::OpenOptionsExt; options.mode(0o600); }
    let result = options.open(&config.handoff).and_then(|mut file| { file.write_all(&bytes)?; file.sync_all() });
    if result.is_err() {
        let _ = running.state.stop.send(true);
        running.data_task.abort(); running.control_task.abort(); running.timer.abort();
        return Err("private handoff publication failed");
    }
    println!("{{\"ready\":true,\"rawRustAxum\":true}}");
    let joined = tokio::try_join!(running.data_task, running.control_task); running.timer.abort();
    match joined { Ok((Ok(()), Ok(()))) => Ok(()), _ => Err("raw Rust listener stopped unexpectedly") }
}
fn main() {
    let result = Config::from_environment().and_then(|config| {
        tokio::runtime::Builder::new_multi_thread().worker_threads(config.workers).enable_all().build()
            .map_err(|_| "raw runtime creation failed")?.block_on(run(config))
    });
    if let Err(reason) = result { eprintln!("{reason}"); std::process::exit(1); }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn bounds_and_private_token_are_strict() {
        assert!(token_valid(&"a".repeat(64)));
        for bad in ["a".repeat(63), "A".repeat(64), "g".repeat(64)] { assert!(!token_valid(&bad)); }
        assert_eq!(bounded(None, 4, 1, 32), Ok(4));
        for bad in ["0", "33", "-1", "x"] { assert!(bounded(Some(bad), 4, 1, 32).is_err()); }
        assert!(opt_in(Some("1")).is_ok());
        for bad in [None, Some(""), Some("0"), Some("true"), Some(" 1")] { assert!(opt_in(bad).is_err()); }
        assert!(bounded(Some("9"), 120, 10, 900).is_err());
        assert!(bounded(Some("901"), 120, 10, 900).is_err());
    }
    #[tokio::test]
    async fn actual_raw_json_empty_wrong_token_and_shutdown() {
        let config = Config { token: "a".repeat(64), handoff: PathBuf::new(), workers: 1, lifetime: Duration::from_secs(10) };
        let running = start(&config).await.unwrap(); let client = reqwest::Client::new();
        assert_eq!(client.get(format!("{}/json", running.data)).send().await.unwrap().status(), 404);
        assert_eq!(client.get(format!("{}/json", running.data)).header("x-calibration-token", "b".repeat(64)).send().await.unwrap().status(), 404);
        let json = client.get(format!("{}/json", running.data)).header("x-calibration-token", &config.token).send().await.unwrap();
        assert_eq!(json.status(), 200); assert_eq!(json.bytes().await.unwrap().as_ref(), TINY.as_bytes());
        let empty = client.get(format!("{}/empty", running.data)).header("x-calibration-token", &config.token).send().await.unwrap();
        assert_eq!(empty.status(), 204); assert!(empty.bytes().await.unwrap().is_empty());
        assert_eq!(running.state.requests.load(Ordering::Relaxed), 2); assert_eq!(running.state.bytes.load(Ordering::Relaxed), 11);
        assert_eq!(client.post(format!("{}/__stop", running.control)).header("x-calibration-token", &config.token).send().await.unwrap().status(), 204);
        tokio::time::timeout(Duration::from_secs(2), async {
            running.data_task.await.unwrap().unwrap(); running.control_task.await.unwrap().unwrap();
        }).await.unwrap(); running.timer.abort();
    }
    #[tokio::test]
    async fn expiry_stops_both_original_listeners() {
        let config = Config { token: "a".repeat(64), handoff: PathBuf::new(), workers: 1, lifetime: Duration::from_millis(20) };
        let running = start(&config).await.unwrap();
        tokio::time::timeout(Duration::from_secs(2), async {
            running.data_task.await.unwrap().unwrap(); running.control_task.await.unwrap().unwrap();
        }).await.unwrap(); running.timer.abort();
    }
}
