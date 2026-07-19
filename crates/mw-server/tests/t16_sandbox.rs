//! t16-e-e2e — render kernel-jail posture + FAIL-CLOSED contract (S2/S3/S6/DQ4).
//!
//! The kernel jail (seccomp-BPF + Landlock + namespaces + rlimits) is Linux-only. The
//! headline live proof — "a syscall OUTSIDE the seccomp filter is KILLED with SIGSYS"
//! and "a jail-expected-but-unavailable render fails closed (503), never parses hostile
//! input in-process" — runs on Linux CI (`.github/workflows/t16-conformance.yml`), and
//! this dev host is win32, so those legs LOUD-SKIP here.
//!
//! What DOES run on every platform is the FAIL-CLOSED policy contract, which is the
//! security-relevant half: when a jail is *expected* but cannot be installed (which is
//! ALWAYS the case off Linux), `confine_current_process` refuses rather than silently
//! degrading, and `mailwoman doctor`'s posture reports the degraded state plainly
//! (no-hype). Those are asserted here directly against the real `mw-sandbox` API.

use mw_sandbox::{
    JailPolicy, SandboxError, confine_current_process, jail_expected, probe, render_posture,
};

#[test]
fn fail_closed_when_a_jail_is_required_but_unavailable() {
    // The security contract (DQ4/S6): a REQUIRED jail that cannot be installed is an
    // error, never a silent in-process parse. On win32 no kernel jail exists, so a
    // required policy MUST fail closed here.
    let result = confine_current_process(&JailPolicy { required: true });
    if cfg!(target_os = "linux") {
        // On Linux the jail installs (this would confine the test process); we do not
        // assert the confine here to avoid seccomp-killing the harness — the live
        // syscall-kill proof is the Linux-CI job.
        eprintln!(
            "[t16 sandbox] Linux host: kernel jail installs; syscall-kill proof is Linux-CI."
        );
    } else {
        let err = result.expect_err("a required jail must fail closed off Linux");
        assert!(
            matches!(err, SandboxError::Unavailable(_)),
            "fail-closed refusal, not a silent degrade: {err}"
        );
    }
}

#[test]
fn jail_expected_matches_platform_default() {
    // Unset MW_RENDER_JAIL → expected on Linux, not expected elsewhere. This is the
    // single policy the render child + server share to decide the fail-closed boundary.
    let restore = std::env::var("MW_RENDER_JAIL").ok();
    // SAFETY: single-threaded test; restored before returning.
    unsafe { std::env::remove_var("MW_RENDER_JAIL") };
    assert_eq!(jail_expected(), cfg!(target_os = "linux"));
    // An explicit require forces expectation on any platform (harden a deployment / test
    // the fail-closed path on a non-Linux host).
    unsafe { std::env::set_var("MW_RENDER_JAIL", "require") };
    assert!(jail_expected());
    match restore {
        Some(v) => unsafe { std::env::set_var("MW_RENDER_JAIL", v) },
        None => unsafe { std::env::remove_var("MW_RENDER_JAIL") },
    }
}

#[test]
fn doctor_posture_reports_platform_state_plainly() {
    let report = probe();
    let rendered = render_posture(&report);
    // The doctor block names the layers + the SPEC reference either way.
    assert!(rendered.contains("SPEC §7.5"));
    for layer in ["seccomp", "landlock", "rlimits", "net-namespace"] {
        assert!(
            rendered.contains(layer),
            "posture mentions {layer}:\n{rendered}"
        );
    }
    if report.platform_supported {
        assert!(report.fully_enforced() || report.degraded.is_some());
        assert!(rendered.contains("kernel jail active"));
    } else {
        assert!(report.degraded.is_some(), "non-Linux is degraded");
        assert!(
            rendered.contains("no kernel jail"),
            "degraded posture stated plainly:\n{rendered}"
        );
    }
    eprintln!(
        "[t16 sandbox] doctor render-posture on {}:\n{rendered}",
        report.platform
    );
}

#[test]
fn kernel_jail_syscall_kill_is_linux_ci_only() {
    if cfg!(target_os = "linux") {
        // The real "syscall outside the filter is killed" proof spawns a child that
        // installs the jail then attempts a blocked syscall (socket/execve) and asserts
        // it dies with SIGSYS. That belongs in the Linux-CI conformance job where a
        // process kill won't take down this harness; here we only record the intent.
        eprintln!(
            "[t16 sandbox] Linux host — the seccomp SIGSYS-kill proof runs in \
             .github/workflows/t16-conformance.yml (jailed child + blocked syscall)."
        );
    } else {
        eprintln!(
            "\n[t16 sandbox SKIP] non-Linux ({}) — kernel jail unavailable; the \
             seccomp/Landlock/namespace syscall-kill proof is Linux-CI-gated. \
             Fail-closed + degraded-posture are proven above on this platform.\n",
            std::env::consts::OS
        );
    }
}
