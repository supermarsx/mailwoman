//! Self-contained desktop mode (SPEC §4.1 / plan §3 e3).
//!
//! A serverless laptop user can run Mailwoman without deploying a separate server:
//! the desktop shell spawns the **bundled `mw-server` binary as a SIBLING PROCESS**
//! on loopback (`127.0.0.1:<ephemeral>`), health-probes it, and points the SPA at
//! that loopback URL exactly like any remote server. **The engine is spawned, never
//! linked into the shell** (SPEC §16 / plan §1.6 / risk #6): this module carries no
//! protocol logic — it manages a child process lifecycle only.
//!
//! Lifecycle: [`LocalServer::start`] picks a free loopback port, spawns the bundled
//! `mw-server serve` with a per-run one-time bootstrap key (`MW_SERVER_KEY`) and a
//! per-profile data dir (`MW_DB_PATH`), then blocks on a `/healthz` probe until the
//! child answers. A supervisor thread restarts the child on crash (bounded) and
//! [`LocalServer::stop`] kills it for a clean quit. The bundled binary is resolved
//! from the Tauri resource dir (`scripts/bundle-server.*` copies the release
//! `mw-server` into `resources/` at build; `tauri.conf.json bundle.resources` ships
//! it). In `cargo test` the debug-built `target/{debug,release}/mw-server` is used.
//!
//! ## Wiring for e7 (this file is intentionally an orphan module, like e1/e4's)
//!
//! `lib.rs`/the shared registration is e7's. To mount, e7 adds `mod selfcontained;`,
//! `app.manage(selfcontained::LocalServer::new());`, and registers the three
//! commands in `tauri::generate_handler![selfcontained::mw_self_contained_status,
//! selfcontained::mw_start_local_server, selfcontained::mw_stop_local_server, …]`
//! (the `mw_`-prefixed names match the frozen `platform/tauri.ts` invoke contract).
//! On `RunEvent::ExitRequested` / window close, e7 calls `LocalServer::stop` so the
//! child is not orphaned on quit.
//!
//! The capability layer (`apps/web/src/platform/tauri.ts`) calls these as the §2.1
//! `selfContainedStatus()` / `startLocalServer()` / `stopLocalServer()` methods;
//! `startLocalServer` returns the loopback URL the SPA transport then points at.

use std::io::{Read, Write};
use std::net::{SocketAddr, TcpListener, TcpStream};
use std::path::PathBuf;
use std::process::{Child, Command, Stdio};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};

use tauri::path::BaseDirectory;
use tauri::{AppHandle, Manager, State};

/// Max ephemeral ports tried per spawn before giving up (covers the tiny
/// pick-free-port → child-bind race).
const PORT_ATTEMPTS: u32 = 3;
/// Bounded crash restarts before the supervisor gives up and reports `error`.
const MAX_RESTARTS: u32 = 5;
/// How long to wait for a freshly spawned child to answer `/healthz`.
const READY_TIMEOUT: Duration = Duration::from_secs(20);
/// Per-request timeout for the loopback probe.
const PROBE_TIMEOUT: Duration = Duration::from_millis(1500);
/// Supervisor poll cadence.
const SUPERVISE_INTERVAL: Duration = Duration::from_millis(400);

/// The frozen §2.1 status the capability layer surfaces.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Status {
    Off,
    Starting,
    Ready,
    Error,
}

impl Status {
    /// The wire string (`selfContainedStatus()` → `"off"|"starting"|"ready"|"error"`).
    pub fn as_str(self) -> &'static str {
        match self {
            Status::Off => "off",
            Status::Starting => "starting",
            Status::Ready => "ready",
            Status::Error => "error",
        }
    }
}

/// How to spawn the sibling `mw-server`.
#[derive(Clone, Debug)]
pub struct SpawnOptions {
    /// The bundled (or debug-built) `mw-server` binary.
    pub binary: PathBuf,
    /// Per-profile data dir; the SQLite DB lives at `<data_dir>/mailwoman.db`.
    pub data_dir: PathBuf,
    /// `MW_MODE` for the child (`engine` = local IMAP/POP3 accounts, the
    /// self-contained-canonical mode; `proxy` = proxy a JMAP upstream).
    pub mode: String,
}

impl SpawnOptions {
    /// Default self-contained options (engine mode). Set `mode` directly (a public
    /// field) for the proxy path.
    pub fn new(binary: PathBuf, data_dir: PathBuf) -> Self {
        Self {
            binary,
            data_dir,
            mode: "engine".into(),
        }
    }

    fn db_path(&self) -> PathBuf {
        self.data_dir.join("mailwoman.db")
    }
}

/// A running sibling `mw-server`.
struct Running {
    child: Child,
    url: String,
}

#[derive(Default)]
struct Shared {
    status: StatusCell,
    url: Option<String>,
    child: Option<Child>,
    opts: Option<SpawnOptions>,
    /// Per-run one-time bootstrap key (`MW_SERVER_KEY`); kept stable across crash
    /// restarts so a mid-session restart does not orphan the sealed DB.
    key: Option<String>,
    stop_requested: bool,
    restarts: u32,
    supervising: bool,
}

/// `Status` with a `Default` (so `Shared` can derive `Default`).
struct StatusCell(Status);
impl Default for StatusCell {
    fn default() -> Self {
        StatusCell(Status::Off)
    }
}

/// Manages the bundled `mw-server` sibling process for self-contained mode. Cheap
/// to clone (shared inner state); stored in Tauri managed state by e7.
#[derive(Clone)]
pub struct LocalServer {
    shared: Arc<Mutex<Shared>>,
}

impl Default for LocalServer {
    fn default() -> Self {
        Self::new()
    }
}

enum Action {
    Exit,
    Idle,
    Restart(SpawnOptions, String),
}

impl LocalServer {
    pub fn new() -> Self {
        Self {
            shared: Arc::new(Mutex::new(Shared::default())),
        }
    }

    /// Current lifecycle status (§2.1 `selfContainedStatus`).
    pub fn status(&self) -> Status {
        self.shared.lock().unwrap().status.0
    }

    /// Spawn the sibling `mw-server` and block until it answers `/healthz`; returns
    /// the loopback URL. Idempotent: if already `ready`, returns the existing URL.
    /// (§2.1 `startLocalServer`.) Blocking — call off the UI thread; the Tauri
    /// command wrapper uses `spawn_blocking`.
    pub fn start(&self, opts: SpawnOptions) -> Result<String, String> {
        {
            let mut s = self.shared.lock().unwrap();
            match s.status.0 {
                Status::Ready => {
                    return s.url.clone().ok_or_else(|| "ready without url".to_string());
                }
                Status::Starting => return Err("local server is already starting".into()),
                _ => {}
            }
            s.status.0 = Status::Starting;
            s.stop_requested = false;
            s.restarts = 0;
            s.opts = Some(opts.clone());
            if s.key.is_none() {
                s.key = Some(generate_key());
            }
        }
        let key = self.shared.lock().unwrap().key.clone().unwrap();

        match spawn_and_probe(&opts, &key) {
            Ok(running) => {
                let url = running.url.clone();
                let mut s = self.shared.lock().unwrap();
                if s.stop_requested {
                    // stop() raced in while we were probing — honor it.
                    let mut child = running.child;
                    let _ = child.kill();
                    let _ = child.wait();
                    s.status.0 = Status::Off;
                    return Err("startup cancelled".into());
                }
                s.child = Some(running.child);
                s.url = Some(url.clone());
                s.status.0 = Status::Ready;
                if !s.supervising {
                    s.supervising = true;
                    let me = self.clone();
                    thread::spawn(move || me.supervise());
                }
                Ok(url)
            }
            Err(e) => {
                let mut s = self.shared.lock().unwrap();
                s.status.0 = Status::Error;
                Err(e)
            }
        }
    }

    /// Kill the sibling and mark the server off (§2.1 `stopLocalServer`). Called on
    /// app quit (e7 wires `RunEvent::ExitRequested`) so the child never orphans.
    pub fn stop(&self) -> Result<(), String> {
        let mut s = self.shared.lock().unwrap();
        s.stop_requested = true;
        if let Some(mut child) = s.child.take() {
            let _ = child.kill();
            let _ = child.wait();
        }
        s.status.0 = Status::Off;
        s.url = None;
        Ok(())
    }

    /// Supervisor loop: restart the child on crash (bounded), exit on `stop`.
    fn supervise(&self) {
        loop {
            thread::sleep(SUPERVISE_INTERVAL);
            let action = {
                let mut s = self.shared.lock().unwrap();
                if s.stop_requested {
                    if let Some(mut c) = s.child.take() {
                        let _ = c.kill();
                        let _ = c.wait();
                    }
                    s.status.0 = Status::Off;
                    s.supervising = false;
                    Action::Exit
                } else {
                    let exited = match s.child.as_mut() {
                        Some(c) => matches!(c.try_wait(), Ok(Some(_))),
                        None => true,
                    };
                    if exited {
                        s.restarts += 1;
                        s.child = None;
                        if s.restarts > MAX_RESTARTS {
                            s.status.0 = Status::Error;
                            s.url = None;
                            s.supervising = false;
                            Action::Exit
                        } else {
                            s.status.0 = Status::Starting;
                            Action::Restart(s.opts.clone().unwrap(), s.key.clone().unwrap())
                        }
                    } else {
                        Action::Idle
                    }
                }
            };
            match action {
                Action::Exit => break,
                Action::Idle => {}
                Action::Restart(opts, key) => match spawn_and_probe(&opts, &key) {
                    Ok(running) => {
                        let mut s = self.shared.lock().unwrap();
                        if s.stop_requested {
                            let mut c = running.child;
                            let _ = c.kill();
                            let _ = c.wait();
                            s.status.0 = Status::Off;
                            s.supervising = false;
                            break;
                        }
                        s.child = Some(running.child);
                        s.url = Some(running.url);
                        s.status.0 = Status::Ready;
                    }
                    Err(_) => {
                        let mut s = self.shared.lock().unwrap();
                        s.status.0 = Status::Error;
                        s.supervising = false;
                        break;
                    }
                },
            }
        }
    }
}

/// Spawn the child on a fresh loopback port and probe `/healthz` until ready.
fn spawn_and_probe(opts: &SpawnOptions, key: &str) -> Result<Running, String> {
    if !opts.binary.exists() {
        return Err(format!(
            "mw-server binary not found at {}",
            opts.binary.display()
        ));
    }
    std::fs::create_dir_all(&opts.data_dir).map_err(|e| e.to_string())?;

    let mut last_err = String::from("no ports tried");
    for _ in 0..PORT_ATTEMPTS {
        let port = pick_free_port().map_err(|e| e.to_string())?;
        let addr: SocketAddr = ([127, 0, 0, 1], port).into();
        let mut child = spawn_child(opts, key, port).map_err(|e| format!("spawn failed: {e}"))?;
        match probe_ready(addr, &mut child) {
            Ok(()) => {
                return Ok(Running {
                    child,
                    url: format!("http://{addr}"),
                });
            }
            Err(e) => {
                let _ = child.kill();
                let _ = child.wait();
                last_err = e;
            }
        }
    }
    Err(format!("mw-server did not become ready: {last_err}"))
}

/// Spawn `mw-server serve` bound to loopback with the self-contained env. The
/// engine runs as an independent process — no linkage, no shared state beyond the
/// per-profile DB path.
fn spawn_child(opts: &SpawnOptions, key: &str, port: u16) -> std::io::Result<Child> {
    Command::new(&opts.binary)
        .arg("serve")
        .env("MW_BIND", format!("127.0.0.1:{port}"))
        .env("MW_DB_PATH", opts.db_path())
        .env("MW_SERVER_KEY", key)
        .env("MW_MODE", &opts.mode)
        .env("MW_COOKIE_SECURE", "false")
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
}

/// Poll `/healthz` until the child answers `200`, it exits early, or we time out.
fn probe_ready(addr: SocketAddr, child: &mut Child) -> Result<(), String> {
    let deadline = Instant::now() + READY_TIMEOUT;
    loop {
        if let Ok(Some(status)) = child.try_wait() {
            return Err(format!("mw-server exited during startup ({status})"));
        }
        if let Ok((200, _)) = http_get(addr, "/healthz", PROBE_TIMEOUT) {
            return Ok(());
        }
        if Instant::now() >= deadline {
            return Err("timed out waiting for /healthz".into());
        }
        thread::sleep(Duration::from_millis(150));
    }
}

/// Reserve a free loopback port by binding `:0` and reading it back. There is a
/// tiny window before the child rebinds it; `spawn_and_probe` retries on failure.
fn pick_free_port() -> std::io::Result<u16> {
    let listener = TcpListener::bind(("127.0.0.1", 0))?;
    Ok(listener.local_addr()?.port())
}

/// Minimal blocking HTTP/1.1 GET over a raw TCP socket — deliberately
/// dependency-free so the thin shell does not pull an HTTP client just to probe a
/// loopback health endpoint. Returns `(status_code, full_response_text)`.
pub(crate) fn http_get(
    addr: SocketAddr,
    path: &str,
    timeout: Duration,
) -> std::io::Result<(u16, String)> {
    let mut stream = TcpStream::connect_timeout(&addr, timeout)?;
    stream.set_read_timeout(Some(timeout))?;
    stream.set_write_timeout(Some(timeout))?;
    let req = format!(
        "GET {path} HTTP/1.1\r\nHost: {addr}\r\nUser-Agent: mailwoman-desktop\r\nAccept: */*\r\nConnection: close\r\n\r\n"
    );
    stream.write_all(req.as_bytes())?;
    stream.flush()?;
    let mut buf = Vec::new();
    stream.read_to_end(&mut buf)?;
    let text = String::from_utf8_lossy(&buf).into_owned();
    let status = parse_status_code(&text)
        .ok_or_else(|| std::io::Error::new(std::io::ErrorKind::InvalidData, "malformed status"))?;
    Ok((status, text))
}

/// Parse the status code from an HTTP response's first line (`HTTP/1.1 <code> …`).
fn parse_status_code(response: &str) -> Option<u16> {
    let line = response.lines().next()?;
    let mut parts = line.split_whitespace();
    let _http = parts.next()?;
    parts.next()?.parse().ok()
}

/// Generate a 32-byte one-time bootstrap key (`MW_SERVER_KEY`) as 64 hex chars,
/// from the OS CSPRNG.
fn generate_key() -> String {
    let mut bytes = [0u8; 32];
    getrandom::fill(&mut bytes).expect("OS RNG unavailable");
    to_hex(&bytes)
}

fn to_hex(bytes: &[u8]) -> String {
    let mut s = String::with_capacity(bytes.len() * 2);
    for b in bytes {
        s.push_str(&format!("{b:02x}"));
    }
    s
}

/// Platform name of the bundled server resource. The `mw-server` crate builds a
/// binary named `mailwoman`; `scripts/bundle-server.*` copies it into `resources/`
/// under this stable, unambiguous name (distinct from the `mailwoman-desktop` shell
/// binary), which `resolve_bundled_server` then looks up.
pub(crate) fn server_bin_name() -> &'static str {
    if cfg!(windows) {
        "mw-server.exe"
    } else {
        "mw-server"
    }
}

/// Resolve the bundled `mw-server` from the Tauri resource dir (`bundle.resources`
/// ships `resources/mw-server*`; `scripts/bundle-server.*` copies it there).
fn resolve_bundled_server(app: &AppHandle) -> Result<PathBuf, String> {
    let rel = format!("resources/{}", server_bin_name());
    let path = app
        .path()
        .resolve(&rel, BaseDirectory::Resource)
        .map_err(|e| format!("cannot resolve bundled mw-server: {e}"))?;
    if path.exists() {
        Ok(path)
    } else {
        Err(format!(
            "bundled mw-server missing at {} (run scripts/bundle-server before packaging)",
            path.display()
        ))
    }
}

/// Per-profile data dir under the app data dir; created if missing.
fn resolve_profile_dir(app: &AppHandle) -> Result<PathBuf, String> {
    let dir = app
        .path()
        .app_data_dir()
        .map_err(|e| e.to_string())?
        .join("self-contained");
    std::fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    Ok(dir)
}

fn resolve_spawn_options(app: &AppHandle) -> Result<SpawnOptions, String> {
    Ok(SpawnOptions::new(
        resolve_bundled_server(app)?,
        resolve_profile_dir(app)?,
    ))
}

// ---- Tauri commands (registered by e7; names/signatures frozen here) ----

/// `selfContainedStatus()` → `"off"|"starting"|"ready"|"error"`. Named
/// `mw_self_contained_status` to match the frozen `platform/tauri.ts` invoke contract.
#[tauri::command]
pub async fn mw_self_contained_status(state: State<'_, LocalServer>) -> Result<String, String> {
    Ok(state.status().as_str().to_string())
}

/// `startLocalServer()` → the loopback URL the SPA transport points at.
#[tauri::command]
pub async fn mw_start_local_server(
    app: AppHandle,
    state: State<'_, LocalServer>,
) -> Result<String, String> {
    let opts = resolve_spawn_options(&app)?;
    let server = state.inner().clone();
    tauri::async_runtime::spawn_blocking(move || server.start(opts))
        .await
        .map_err(|e| e.to_string())?
}

/// `stopLocalServer()` — kill the sibling and mark the server off.
#[tauri::command]
pub async fn mw_stop_local_server(state: State<'_, LocalServer>) -> Result<(), String> {
    let server = state.inner().clone();
    tauri::async_runtime::spawn_blocking(move || server.stop())
        .await
        .map_err(|e| e.to_string())?
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn status_maps_to_frozen_wire_strings() {
        assert_eq!(Status::Off.as_str(), "off");
        assert_eq!(Status::Starting.as_str(), "starting");
        assert_eq!(Status::Ready.as_str(), "ready");
        assert_eq!(Status::Error.as_str(), "error");
    }

    #[test]
    fn spawn_options_default_is_engine_mode_with_profile_db() {
        let opts = SpawnOptions::new(PathBuf::from("mw-server"), PathBuf::from("/tmp/prof"));
        assert_eq!(opts.mode, "engine");
        assert!(opts.db_path().ends_with("mailwoman.db"));
    }

    #[test]
    fn generate_key_is_64_hex_chars_and_random() {
        let a = generate_key();
        let b = generate_key();
        assert_eq!(a.len(), 64);
        assert!(a.bytes().all(|c| c.is_ascii_hexdigit()));
        assert_ne!(a, b, "keys must not repeat");
    }

    #[test]
    fn parse_status_code_reads_the_first_line() {
        assert_eq!(
            parse_status_code("HTTP/1.1 401 Unauthorized\r\n\r\n{}"),
            Some(401)
        );
        assert_eq!(parse_status_code("HTTP/1.1 200 OK\r\n\r\nok"), Some(200));
        assert_eq!(parse_status_code("garbage"), None);
    }

    #[test]
    fn idle_server_reports_off() {
        let s = LocalServer::new();
        assert_eq!(s.status(), Status::Off);
    }

    /// Parse `http://127.0.0.1:<port>` (what `start` returns) into a `SocketAddr`.
    fn addr_from_url(url: &str) -> SocketAddr {
        url.strip_prefix("http://")
            .expect("http url")
            .parse()
            .expect("socket addr")
    }

    /// The acceptance gate (plan §3 e3): spawn the bundled (here: debug-built)
    /// `mw-server` on loopback, health-probe `/healthz`, drive ONE JMAP round-trip
    /// (unauthenticated `/jmap/session` → 401, proving the spawned server serves +
    /// guards the JMAP surface), then shut it down cleanly and assert it is gone.
    #[test]
    fn spawn_probe_jmap_roundtrip_then_clean_shutdown() {
        let binary = locate_mw_server();
        let data_dir = unique_temp_dir();
        let server = LocalServer::new();

        let url = server
            .start(SpawnOptions::new(binary, data_dir.clone()))
            .expect("self-contained mw-server should start");
        assert_eq!(server.status(), Status::Ready);
        assert!(
            url.starts_with("http://127.0.0.1:"),
            "loopback url, got {url}"
        );
        let addr = addr_from_url(&url);

        // /healthz — the readiness endpoint.
        let (code, body) = http_get(addr, "/healthz", Duration::from_secs(3)).expect("healthz");
        assert_eq!(code, 200, "healthz body: {body}");

        // One JMAP round-trip against the spawned server.
        let (code, _) = http_get(addr, "/jmap/session", Duration::from_secs(3)).expect("jmap");
        assert_eq!(code, 401, "unauthenticated /jmap/session must be guarded");

        // Clean shutdown: the child is killed and the loopback stops serving.
        server.stop().expect("stop");
        assert_eq!(server.status(), Status::Off);
        thread::sleep(Duration::from_millis(400));
        match http_get(addr, "/healthz", Duration::from_millis(800)) {
            Err(_) => {}
            Ok((code, _)) => assert_ne!(code, 200, "server still serving after stop"),
        }

        let _ = std::fs::remove_dir_all(&data_dir);
    }

    /// Locate the server binary; prefer the release build, fall back to debug,
    /// building it if an isolated `cargo test` run lacks it. NB: the `mw-server`
    /// crate's binary is named `mailwoman` (`[[bin]] name`); `bundle-server` renames
    /// it to `mw-server` (see [`server_bin_name`]) when copying into resources.
    fn locate_mw_server() -> PathBuf {
        let name = if cfg!(windows) { "mailwoman.exe" } else { "mailwoman" };
        let manifest = PathBuf::from(env!("CARGO_MANIFEST_DIR")); // apps/desktop/src-tauri
        let ws_root = manifest
            .parent()
            .and_then(|p| p.parent())
            .and_then(|p| p.parent())
            .expect("workspace root")
            .to_path_buf();
        let target = std::env::var_os("CARGO_TARGET_DIR")
            .map(PathBuf::from)
            .unwrap_or_else(|| ws_root.join("target"));
        for profile in ["release", "debug"] {
            let candidate = target.join(profile).join(name);
            if candidate.exists() {
                return candidate;
            }
        }
        let status = Command::new(env!("CARGO"))
            .args(["build", "-p", "mw-server"])
            .current_dir(&ws_root)
            .status()
            .expect("invoke cargo build -p mw-server");
        assert!(status.success(), "cargo build -p mw-server failed");
        let built = target.join("debug").join(name);
        assert!(built.exists(), "mw-server not at {}", built.display());
        built
    }

    fn unique_temp_dir() -> PathBuf {
        use std::sync::atomic::{AtomicU64, Ordering};
        static SEQ: AtomicU64 = AtomicU64::new(0);
        let dir = std::env::temp_dir().join(format!(
            "mw-selfcontained-{}-{}",
            std::process::id(),
            SEQ.fetch_add(1, Ordering::Relaxed)
        ));
        std::fs::create_dir_all(&dir).unwrap();
        dir
    }
}
