//! Kernel jail for the disposable render child (SPEC §7.5, plan t16 S2/S3/S6/DQ4).
//!
//! The `mw-render` child parses hostile input (message HTML, `.msg`/`.oft` compound
//! files, remote images) in a throwaway process. Before it reads a single byte of
//! that input it calls [`confine_current_process`], which — **on Linux** — locks the
//! process down with four layers:
//!
//! 1. **`no_new_privs`** (`prctl`) — required for unprivileged seccomp/Landlock.
//! 2. **rlimits** — no core dumps (never spill hostile bytes), no regular-file writes,
//!    a modest file-descriptor cap.
//! 3. **network namespace** — `unshare(CLONE_NEWUSER|CLONE_NEWNET)` drops the child
//!    into an empty net namespace (best-effort; the socket syscalls are killed by
//!    seccomp regardless, so this is defense in depth).
//! 4. **Landlock** — an empty filesystem ruleset: every path access is denied
//!    (best-effort across kernel ABIs).
//! 5. **seccomp-BPF** — a default-**kill-process** syscall allowlist. Anything outside
//!    the render child's small working set (notably `execve`, `ptrace`, `socket`,
//!    `connect`) terminates the process with `SIGSYS`.
//!
//! On **non-Linux** targets (the win32 dev host, macOS, non-Linux CI) there is no
//! kernel jail: [`confine_current_process`] is a documented no-op that returns a
//! **degraded** [`SandboxReport`]. The render child still runs process-isolated and
//! still parses media inside the WASM jail — it just lacks the kernel layer. This is
//! reported plainly by `mailwoman doctor` (no-hype).
//!
//! ## Fail-closed (DQ4 / S6)
//!
//! [`jail_expected`] is the single policy the render child *and* the server share.
//! When a jail is expected but cannot be installed, the child exits fail-closed
//! (never parsing hostile input unconfined) and the server refuses the in-process
//! fallback rather than parsing in the trusted process.

#[cfg(target_os = "linux")]
mod linux;

use std::fmt;

/// Whether a kernel render-jail is **expected** on this deployment (DQ4).
///
/// This is the shared contract between the render child (which fails closed if the
/// jail is expected but cannot be installed) and the server (which refuses the
/// in-process render fallback when a jail was expected). It is driven by the
/// `MW_RENDER_JAIL` environment variable:
///
/// - `require` / `required` / `on` / `1` → expected on **every** platform (harden a
///   deployment, or exercise the fail-closed path on a non-Linux dev host).
/// - `off` / `degraded` / `0` → **not** expected (run the documented degraded mode).
/// - unset (the default) → expected on Linux, not expected on other platforms.
pub fn jail_expected() -> bool {
    match std::env::var("MW_RENDER_JAIL") {
        Ok(v) => match v.trim().to_ascii_lowercase().as_str() {
            "require" | "required" | "on" | "1" | "yes" => true,
            "off" | "degraded" | "0" | "no" => false,
            // An unrecognised value falls back to the platform default rather than
            // silently disabling the jail.
            _ => cfg!(target_os = "linux"),
        },
        Err(_) => cfg!(target_os = "linux"),
    }
}

/// How the render child should treat a jail it cannot fully install.
#[derive(Debug, Clone, Copy)]
pub struct JailPolicy {
    /// When `true`, [`confine_current_process`] returns [`SandboxError::Unavailable`]
    /// if a *required* layer (`no_new_privs` or seccomp) cannot be installed, so the
    /// caller fails closed. Best-effort layers (namespace, Landlock) never fail the
    /// call; they are recorded in the report as degraded instead.
    pub required: bool,
}

impl JailPolicy {
    /// The policy the render child uses: fail-closed exactly when a jail is expected
    /// on this platform ([`jail_expected`]).
    pub fn render_child() -> Self {
        Self {
            required: jail_expected(),
        }
    }
}

/// One isolation layer's outcome.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LayerState {
    /// The layer is fully active.
    Enforced,
    /// The layer applied only partially (e.g. Landlock on an older kernel ABI).
    Partial,
    /// The layer is unavailable on this kernel; recorded, not fatal for a
    /// best-effort layer.
    Unavailable(String),
    /// The layer does not exist on this platform (non-Linux).
    NotApplicable,
}

impl LayerState {
    /// A short word for the `doctor` table.
    fn word(&self) -> &str {
        match self {
            LayerState::Enforced => "enforced",
            LayerState::Partial => "partial",
            LayerState::Unavailable(_) => "unavailable",
            LayerState::NotApplicable => "n/a",
        }
    }
}

/// One named layer and its outcome.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Layer {
    pub name: &'static str,
    pub state: LayerState,
}

impl Layer {
    fn new(name: &'static str, state: LayerState) -> Self {
        Self { name, state }
    }
}

/// The isolation actually applied to (or, for [`probe`], available to) the process.
#[derive(Debug, Clone)]
pub struct SandboxReport {
    /// `true` only on a platform with the kernel jail (Linux).
    pub platform_supported: bool,
    /// `std::env::consts::OS` for the running target.
    pub platform: &'static str,
    /// Per-layer outcome, in application order.
    pub layers: Vec<Layer>,
    /// Present when the jail is degraded (a missing layer or an unsupported
    /// platform); the plain-language reason `doctor` prints.
    pub degraded: Option<String>,
}

impl SandboxReport {
    /// Whether every kernel layer is fully enforced (Linux, nothing degraded).
    pub fn fully_enforced(&self) -> bool {
        self.platform_supported && self.degraded.is_none()
    }
}

/// The report returned on a platform with no kernel jail. Only referenced off Linux
/// (the Linux path always produces a real report via `linux::`).
#[cfg(not(target_os = "linux"))]
fn degraded_report(reason: &str) -> SandboxReport {
    SandboxReport {
        platform_supported: false,
        platform: std::env::consts::OS,
        layers: vec![
            Layer::new("no_new_privs", LayerState::NotApplicable),
            Layer::new("rlimits", LayerState::NotApplicable),
            Layer::new("net-namespace", LayerState::NotApplicable),
            Layer::new("landlock", LayerState::NotApplicable),
            Layer::new("seccomp", LayerState::NotApplicable),
        ],
        degraded: Some(reason.to_string()),
    }
}

/// Why a required jail could not be installed.
#[derive(Debug, Clone)]
pub enum SandboxError {
    /// A required layer could not be installed and the policy demanded the jail.
    Unavailable(String),
}

impl fmt::Display for SandboxError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SandboxError::Unavailable(m) => write!(f, "kernel jail unavailable: {m}"),
        }
    }
}

impl std::error::Error for SandboxError {}

/// Confine the **current** process before it touches hostile input.
///
/// On Linux this installs `no_new_privs` + rlimits + a network namespace + Landlock +
/// a seccomp-BPF kill-process allowlist, in that order (seccomp last, so the setup
/// syscalls themselves are not filtered). On non-Linux it is a no-op.
///
/// Fail-closed: when `policy.required` is set and a required layer (`no_new_privs` or
/// seccomp) cannot be installed — including "not on Linux at all" — this returns
/// [`SandboxError::Unavailable`] so the caller can refuse to run unconfined. A
/// best-effort layer that degrades (namespace/Landlock unavailable on the kernel) is
/// recorded in the report and never fails a required jail on its own.
pub fn confine_current_process(policy: &JailPolicy) -> Result<SandboxReport, SandboxError> {
    #[cfg(target_os = "linux")]
    {
        linux::confine(policy)
    }
    #[cfg(not(target_os = "linux"))]
    {
        let reason = "kernel jail unavailable on this platform (non-Linux)";
        if policy.required {
            return Err(SandboxError::Unavailable(reason.into()));
        }
        Ok(degraded_report(reason))
    }
}

/// Report the render sandbox posture **without confining the caller** — for
/// `mailwoman doctor`. On Linux it reports the kernel jail as available (probing
/// Landlock's supported ABI harmlessly); on non-Linux it reports the degraded
/// posture plainly.
pub fn probe() -> SandboxReport {
    #[cfg(target_os = "linux")]
    {
        linux::probe()
    }
    #[cfg(not(target_os = "linux"))]
    {
        degraded_report("kernel jail unavailable on this platform (non-Linux)")
    }
}

/// Render the sandbox posture as the aligned block `mailwoman doctor` prints,
/// mirroring [`mw_cache::render_posture`]. Factual, no-hype.
pub fn render_posture(report: &SandboxReport) -> String {
    use std::fmt::Write as _;
    let mut out = String::new();
    let _ = writeln!(out, "Render sandbox posture (SPEC §7.5):");
    if report.platform_supported {
        let _ = writeln!(
            out,
            "  platform: {} — kernel jail active for the render child",
            report.platform
        );
    } else {
        let _ = writeln!(
            out,
            "  platform: {} — no kernel jail; the render child is process-isolated only \
             (degraded). The WASM media jail still applies.",
            report.platform
        );
    }
    for layer in &report.layers {
        match &layer.state {
            LayerState::Unavailable(why) => {
                let _ = writeln!(out, "  {:<14} {} ({})", layer.name, layer.state.word(), why);
            }
            _ => {
                let _ = writeln!(out, "  {:<14} {}", layer.name, layer.state.word());
            }
        }
    }
    if let Some(reason) = &report.degraded {
        let _ = writeln!(out, "  note: {reason}");
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn jail_expected_env_overrides() {
        // Serialize env mutation within this test; MW_RENDER_JAIL is process-global.
        // SAFETY: single-threaded test, restored before returning.
        let restore = std::env::var("MW_RENDER_JAIL").ok();
        for (val, want) in [
            ("require", true),
            ("required", true),
            ("on", true),
            ("1", true),
            ("off", false),
            ("degraded", false),
            ("0", false),
        ] {
            unsafe { std::env::set_var("MW_RENDER_JAIL", val) };
            assert_eq!(jail_expected(), want, "MW_RENDER_JAIL={val}");
        }
        // Unrecognised value → platform default.
        unsafe { std::env::set_var("MW_RENDER_JAIL", "banana") };
        assert_eq!(jail_expected(), cfg!(target_os = "linux"));
        match restore {
            Some(v) => unsafe { std::env::set_var("MW_RENDER_JAIL", v) },
            None => unsafe { std::env::remove_var("MW_RENDER_JAIL") },
        }
    }

    #[test]
    fn non_required_confine_is_ok_everywhere() {
        // On the non-Linux dev host this exercises the degraded no-op; on Linux it
        // actually confines the test process (still returns Ok).
        let report = confine_current_process(&JailPolicy { required: false }).unwrap();
        assert_eq!(report.platform, std::env::consts::OS);
    }

    #[cfg(not(target_os = "linux"))]
    #[test]
    fn required_confine_fails_closed_off_linux() {
        // The fail-closed contract, unit-testable on the win32 dev host: a required
        // jail on a platform without one is refused, never silently degraded.
        let err = confine_current_process(&JailPolicy { required: true }).unwrap_err();
        assert!(matches!(err, SandboxError::Unavailable(_)));
    }

    #[cfg(not(target_os = "linux"))]
    #[test]
    fn probe_reports_degraded_off_linux() {
        let report = probe();
        assert!(!report.platform_supported);
        assert!(report.degraded.is_some());
        let rendered = render_posture(&report);
        assert!(rendered.contains("no kernel jail"));
        assert!(rendered.contains("SPEC §7.5"));
    }

    #[test]
    fn render_posture_lists_layers() {
        let report = probe();
        let rendered = render_posture(&report);
        for layer in ["seccomp", "landlock", "rlimits"] {
            assert!(rendered.contains(layer), "posture should mention {layer}");
        }
    }
}
