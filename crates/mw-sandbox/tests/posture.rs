//! The sandbox posture surface as an outside caller (`mw-render`, `mailwoman
//! doctor`) sees it.
//!
//! **Platform honesty — read this before trusting any number from this file.**
//! `mw-sandbox` is a security boundary whose entire implementation is
//! `#[cfg(target_os = "linux")]`. On Windows and macOS the kernel jail *does not
//! exist*, so nothing here can exercise enforcement; what it can pin is the
//! contract that surrounds it:
//!
//! * a required jail on a platform without one is **refused**, never degraded;
//! * a degraded posture claims no layer as enforced (a `doctor` table that said
//!   "enforced" off Linux would be a false security claim);
//! * the rendering and summary logic (`render_posture`, `fully_enforced`) behaves
//!   correctly for an enforced report — checked against a **hand-constructed**
//!   report, which tests the reporting code and says nothing about any kernel.
//!
//! Real Linux enforcement — that a confined process is actually killed for making a
//! forbidden syscall — is in `tests/linux_enforcement.rs`, which runs only on Linux
//! and loudly skips elsewhere.

use mw_sandbox::{
    JailPolicy, Layer, LayerState, SandboxError, SandboxReport, jail_expected, jail_strict, probe,
    render_posture,
};

/// `MW_RENDER_JAIL` is process-global; serialize every test that writes it.
static ENV_LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());

/// The layer set `doctor` shows, in application order. Seccomp is last because it is
/// installed last (so the setup syscalls are not themselves filtered).
const LAYERS: [&str; 5] = [
    "no_new_privs",
    "rlimits",
    "net-namespace",
    "landlock",
    "seccomp",
];

// ── the env contract, from outside the crate ─────────────────────────────────

#[test]
fn the_jail_policy_env_contract_holds_for_every_accepted_spelling() {
    let _guard = ENV_LOCK.lock().unwrap_or_else(|e| e.into_inner());
    // SAFETY: single-threaded within the lock, and the prior value is restored.
    let restore = std::env::var("MW_RENDER_JAIL").ok();

    // `yes`/`no` are accepted spellings the inline tests do not cover, and an
    // operator who writes one must get what they asked for rather than the default.
    for (value, expected) in [("yes", true), ("no", false), ("YES", true), ("No", false)] {
        unsafe { std::env::set_var("MW_RENDER_JAIL", value) };
        assert_eq!(jail_expected(), expected, "MW_RENDER_JAIL={value}");
    }

    // An unrecognised value must fall back to the PLATFORM DEFAULT, never to "off":
    // a typo in a deployment's env must not silently disable the jail on Linux.
    for junk in ["", "  ", "true", "enabled", "ON!", "strictly"] {
        unsafe { std::env::set_var("MW_RENDER_JAIL", junk) };
        assert_eq!(
            jail_expected(),
            cfg!(target_os = "linux"),
            "MW_RENDER_JAIL={junk:?} must fall back to the platform default"
        );
        assert!(!jail_strict(), "MW_RENDER_JAIL={junk:?} is not strict");
    }

    // `JailPolicy::render_child()` is the render child's policy and must track
    // `jail_expected()` exactly — the two agreeing is what makes "fail closed" and
    // "server refuses the in-process fallback" the same decision.
    for value in ["require", "off", "strict"] {
        unsafe { std::env::set_var("MW_RENDER_JAIL", value) };
        assert_eq!(
            JailPolicy::render_child().required,
            jail_expected(),
            "MW_RENDER_JAIL={value}"
        );
    }

    // `strict` always implies a required jail — it only upgrades Landlock from
    // best-effort to load-bearing, it never turns the jail off.
    unsafe { std::env::set_var("MW_RENDER_JAIL", "strict") };
    assert!(jail_strict() && jail_expected() && JailPolicy::render_child().required);

    match restore {
        Some(v) => unsafe { std::env::set_var("MW_RENDER_JAIL", v) },
        None => unsafe { std::env::remove_var("MW_RENDER_JAIL") },
    }
}

// ── reporting logic, over hand-constructed reports ───────────────────────────

fn report(
    platform_supported: bool,
    states: [LayerState; 5],
    degraded: Option<&str>,
) -> SandboxReport {
    SandboxReport {
        platform_supported,
        platform: "linux",
        layers: LAYERS
            .iter()
            .zip(states)
            .map(|(name, state)| Layer { name, state })
            .collect(),
        degraded: degraded.map(str::to_string),
    }
}

#[test]
fn fully_enforced_requires_both_the_platform_and_an_undegraded_report() {
    // NOTE: these reports are CONSTRUCTED, not observed. This pins the summary rule
    // `doctor` prints, not the behaviour of any kernel.
    let all = || {
        [
            LayerState::Enforced,
            LayerState::Enforced,
            LayerState::Enforced,
            LayerState::Enforced,
            LayerState::Enforced,
        ]
    };
    assert!(report(true, all(), None).fully_enforced());
    // A degraded note beats an all-enforced layer list: the note is set when a
    // *strong* layer is missing, and summarising over it would hide that.
    assert!(!report(true, all(), Some("landlock unavailable")).fully_enforced());
    // Off a supported platform nothing is ever fully enforced, whatever the layers.
    assert!(!report(false, all(), None).fully_enforced());
}

#[test]
fn the_posture_table_spells_out_partial_and_unavailable_layers() {
    // `doctor` output is how an operator learns which protections are missing, so
    // each state needs its own word and an unavailable layer needs its reason.
    let text = render_posture(&report(
        true,
        [
            LayerState::Enforced,
            LayerState::Partial,
            LayerState::Unavailable("unprivileged userns disabled".into()),
            LayerState::Partial,
            LayerState::Enforced,
        ],
        Some("kernel jail degraded: landlock unavailable on this kernel"),
    ));

    assert!(
        text.contains("Render sandbox posture (SPEC §7.5)"),
        "{text}"
    );
    assert!(
        text.contains("kernel jail active for the render child"),
        "{text}"
    );
    for name in LAYERS {
        assert!(
            text.contains(name),
            "{name} missing from the table:\n{text}"
        );
    }
    assert!(text.contains("partial"), "{text}");
    assert!(
        text.contains("unavailable (unprivileged userns disabled)"),
        "an unavailable layer must carry its reason:\n{text}"
    );
    assert!(
        text.contains("note: kernel jail degraded"),
        "the degraded note belongs in the table:\n{text}"
    );
}

#[test]
fn an_unsupported_platform_is_described_without_hype_or_a_false_claim() {
    let text = render_posture(&report(
        false,
        [
            LayerState::NotApplicable,
            LayerState::NotApplicable,
            LayerState::NotApplicable,
            LayerState::NotApplicable,
            LayerState::NotApplicable,
        ],
        Some("kernel jail unavailable on this platform (non-Linux)"),
    ));
    assert!(text.contains("no kernel jail"), "{text}");
    assert!(
        text.contains("process-isolated only"),
        "the remaining isolation must be stated plainly:\n{text}"
    );
    assert!(
        text.contains("WASM media jail still applies"),
        "the layer that DOES still apply must not be omitted:\n{text}"
    );
    assert!(
        !text.contains("enforced"),
        "nothing may read as enforced:\n{text}"
    );
}

#[test]
fn a_required_jail_failure_says_why_it_could_not_be_installed() {
    let e = SandboxError::Unavailable("seccomp: Operation not permitted".into());
    assert_eq!(
        e.to_string(),
        "kernel jail unavailable: seccomp: Operation not permitted"
    );
    // It is a real error type, so `?` and `Box<dyn Error>` work at the call site.
    let boxed: Box<dyn std::error::Error> = Box::new(e);
    assert!(boxed.to_string().contains("seccomp"));
}

// ── what this host actually observed ─────────────────────────────────────────

#[test]
fn probe_never_claims_more_than_the_running_platform_provides() {
    let report = probe();
    assert_eq!(report.platform, std::env::consts::OS);
    assert_eq!(
        report.layers.iter().map(|l| l.name).collect::<Vec<_>>(),
        LAYERS,
        "the table must always list every layer, so a missing one is visible"
    );
    assert_eq!(report.platform_supported, cfg!(target_os = "linux"));

    #[cfg(not(target_os = "linux"))]
    {
        // Windows/macOS: the ONLY thing proved here is that nothing is claimed.
        assert!(report.degraded.is_some());
        assert!(!report.fully_enforced());
        for layer in &report.layers {
            assert_eq!(layer.state, LayerState::NotApplicable, "{}", layer.name);
        }
        eprintln!(
            "[mw-sandbox] NOTE: running on {} — kernel-jail ENFORCEMENT is not exercised \
             by this test binary. See tests/linux_enforcement.rs.",
            std::env::consts::OS
        );
    }
    #[cfg(target_os = "linux")]
    {
        // Linux: `probe` must not restrict the calling (doctor) process, so a second
        // probe and an ordinary file open both still work afterwards.
        let again = probe();
        assert!(again.platform_supported);
        assert!(
            std::fs::metadata(std::env::current_exe().expect("exe")).is_ok(),
            "probe() must not confine the caller — it is called by `doctor`"
        );
    }
}

// NOTE — deliberately NOT tested here: `confine_current_process(required: false)`.
// On Linux it confines the CALLING process, and seccomp's filter is applied with
// TSYNC, so calling it inside this binary would silently place every test that ran
// afterwards inside the jail. `src/tests.rs` already covers that call; the confined
// paths belong in a child process, which is what `tests/linux_enforcement.rs` does.
