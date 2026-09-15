//! **Real kernel-jail enforcement.** Linux only, and loudly skipped everywhere else.
//!
//! `src/lib.rs` says of the seccomp allowlist: *"the whole socket family
//! (`socket`/`connect`/…): the child never needs the network, so an attempt to reach
//! it is fatal. This is the syscall a Linux-CI test drives to prove the jail."* This
//! is that test.
//!
//! ## Why a child process
//!
//! [`mw_sandbox::confine_current_process`] confines the **caller**: seccomp, Landlock
//! and `no_new_privs` bind the calling thread (seccomp is installed without `TSYNC`)
//! and every thread it creates afterwards, and the rlimits bind the whole process.
//! Calling it inside a test binary would jail that test's thread and leave the
//! harness running under `RLIMIT_FSIZE=0` and a 256-fd cap. So each probe
//! re-executes *this same test binary*, filtered to the `#[ignore]`d helper below,
//! with `MW_SANDBOX_TEST_CHILD` naming the probe to run. The helper does nothing
//! unless that variable is set, so `cargo test -- --include-ignored` stays safe.
//!
//! ## What is and is not proved
//!
//! On **Linux**: that `confine_current_process` with a required policy actually
//! installs (`no_new_privs` + seccomp reported `enforced`), that a confined process
//! is **killed by `SIGSYS`** when it attempts a socket syscall, and — when the
//! running kernel reports Landlock `enforced` — that a file readable *before* the
//! jail is unreadable *after* it. The parent reads the child's own layer report, so
//! the assertions adapt to the kernel rather than assuming one; every skipped
//! assertion is printed with its reason.
//!
//! On **non-Linux**: nothing. `mw-sandbox`'s non-Linux arm is a documented no-op, and
//! a test that passed there would prove only that the no-op no-ops. The single test
//! compiled off Linux says exactly that and does not pretend otherwise.

// ── the child helper (shared by both platforms so the binary always builds) ──

/// Env var naming the probe the re-executed child should run.
const CHILD_ENV: &str = "MW_SANDBOX_TEST_CHILD";
/// Env var carrying the pre-created file the Landlock probe re-opens. Only the
/// Linux arm has a Landlock probe, so it does not exist off Linux.
#[cfg(target_os = "linux")]
const CHILD_PROBE_FILE: &str = "MW_SANDBOX_TEST_PROBE_FILE";
/// The name of the helper test, used as the child's filter.
const HELPER: &str = "jail_child_helper";

/// Not a test in its own right: the body the parent tests re-execute. It is
/// `#[ignore]`d so a normal run skips it, and it returns immediately unless
/// [`CHILD_ENV`] is set — so even `--include-ignored` cannot confine this binary.
#[test]
#[ignore = "internal child process for the Linux jail probes; driven by the tests in this file"]
fn jail_child_helper() {
    let Ok(mode) = std::env::var(CHILD_ENV) else {
        eprintln!("[mw-sandbox] {HELPER} invoked without {CHILD_ENV}: nothing to do");
        return;
    };
    #[cfg(target_os = "linux")]
    linux_child::run(&mode);
    #[cfg(not(target_os = "linux"))]
    {
        println!("UNSUPPORTED {mode}");
    }
}

#[cfg(target_os = "linux")]
mod linux_child {
    use std::io::Write as _;

    use mw_sandbox::{JailPolicy, LayerState, confine_current_process};

    fn flush() {
        let _ = std::io::stdout().flush();
    }

    /// Leave with `code` without running exit handlers. The jail sets
    /// `RLIMIT_FSIZE=0`, and a coverage-instrumented build writes its profile from an
    /// exit handler through a descriptor it opened before the jail, so a plain
    /// `process::exit` turned every probe's result into SIGXFSZ under `cargo llvm-cov`
    /// (t24-e10). The probe has already printed and flushed everything the parent
    /// reads; only the child's own coverage profile is lost.
    fn leave(code: i32) -> ! {
        flush();
        // SAFETY: `_exit` takes no pointers and does not return.
        unsafe { libc::_exit(code) }
    }

    /// Confine this process for real, publish the resulting layer states on stdout so
    /// the parent can adapt its assertions to the kernel, then run one probe.
    pub(super) fn run(mode: &str) -> ! {
        let report = match confine_current_process(&JailPolicy { required: true }) {
            Ok(r) => r,
            Err(e) => {
                println!("CONFINE_ERR {e}");
                leave(20);
            }
        };
        for layer in &report.layers {
            let word = match &layer.state {
                LayerState::Enforced => "enforced",
                LayerState::Partial => "partial",
                LayerState::Unavailable(_) => "unavailable",
                LayerState::NotApplicable => "n/a",
            };
            println!("LAYER {} {word}", layer.name);
        }
        println!("PLATFORM_SUPPORTED {}", report.platform_supported);
        // The marker the parent requires before believing any probe result: without
        // it, a process that died on the way to the probe would look like a kill.
        println!("CONFINED");
        flush();

        match mode {
            // The documented proof: the socket family is outside the allowlist, so
            // reaching for the network must terminate the process with SIGSYS.
            "socket" => {
                let _ = std::net::TcpStream::connect("127.0.0.1:9");
                println!("SURVIVED");
                leave(21);
            }
            // Landlock: an empty ruleset denies every path. `openat` is *allowed* by
            // seccomp, so a refusal here is Landlock's doing, not the syscall filter's.
            "open" => {
                let path = std::env::var(super::CHILD_PROBE_FILE).unwrap_or_default();
                match std::fs::read(&path) {
                    Ok(_) => println!("OPEN ok"),
                    Err(e) => println!("OPEN err {:?}", e.kind()),
                }
                leave(0);
            }
            // Just report and leave: proves a required jail INSTALLS on this kernel.
            "posture" => {
                print!("{}", mw_sandbox::render_posture(&report));
                leave(0);
            }
            other => {
                println!("UNKNOWN_MODE {other}");
                leave(22);
            }
        }
    }
}

// ── the Linux tests ──────────────────────────────────────────────────────────

#[cfg(target_os = "linux")]
mod linux {
    use std::collections::BTreeMap;
    use std::os::unix::process::ExitStatusExt as _;

    use super::{CHILD_ENV, CHILD_PROBE_FILE, HELPER};

    /// `SIGSYS` — what `SeccompAction::KillProcess` raises. 31 on x86_64/aarch64
    /// Linux, which is every architecture this project builds for.
    const SIGSYS: i32 = 31;

    struct ChildRun {
        status: std::process::ExitStatus,
        stdout: String,
        layers: BTreeMap<String, String>,
    }

    impl ChildRun {
        fn confined(&self) -> bool {
            self.stdout.contains("\nCONFINED") || self.stdout.starts_with("CONFINED")
        }
        fn layer(&self, name: &str) -> &str {
            self.layers
                .get(name)
                .map(String::as_str)
                .unwrap_or("missing")
        }
    }

    fn run_child(mode: &str, extra_env: &[(&str, &str)]) -> ChildRun {
        let exe = std::env::current_exe().expect("path to this test binary");
        let mut cmd = std::process::Command::new(exe);
        cmd.args([HELPER, "--exact", "--ignored", "--nocapture"])
            .env(CHILD_ENV, mode);
        for (k, v) in extra_env {
            cmd.env(k, v);
        }
        let out = cmd.output().expect("spawn the confined child");
        let stdout = String::from_utf8_lossy(&out.stdout).into_owned();
        let layers = stdout
            .lines()
            .filter_map(|l| l.strip_prefix("LAYER "))
            .filter_map(|l| l.split_once(' '))
            .map(|(name, state)| (name.to_string(), state.to_string()))
            .collect();
        ChildRun {
            status: out.status,
            stdout,
            layers,
        }
    }

    #[test]
    fn a_required_jail_installs_on_this_kernel() {
        let run = run_child("posture", &[]);
        assert!(
            run.confined(),
            "confine_current_process(required) FAILED on this kernel:\n{}",
            run.stdout
        );
        assert_eq!(run.status.code(), Some(0), "child:\n{}", run.stdout);
        assert!(
            run.stdout.contains("PLATFORM_SUPPORTED true"),
            "\n{}",
            run.stdout
        );

        // The two layers a required jail treats as fatal must both be enforced —
        // anything less and `confine` would have returned Err, so this also pins that
        // the report agrees with the decision.
        assert_eq!(run.layer("no_new_privs"), "enforced", "\n{}", run.stdout);
        assert_eq!(run.layer("seccomp"), "enforced", "\n{}", run.stdout);

        // Best-effort layers vary by kernel and by whether unprivileged user
        // namespaces are permitted. Report, do not assert.
        eprintln!(
            "[mw-sandbox] kernel jail on this host: net-namespace={} landlock={} rlimits={}",
            run.layer("net-namespace"),
            run.layer("landlock"),
            run.layer("rlimits"),
        );
        assert!(
            run.stdout.contains("Render sandbox posture"),
            "\n{}",
            run.stdout
        );
    }

    #[test]
    fn a_confined_process_is_killed_for_reaching_the_network() {
        // THE enforcement proof. `socket` is outside the allowlist and the filter's
        // default action is kill-process, so this must die by SIGSYS — not return an
        // error, not connect, not be silently permitted.
        let run = run_child("socket", &[]);
        assert!(
            run.confined(),
            "the jail did not install, so nothing about enforcement is proved:\n{}",
            run.stdout
        );
        assert!(
            !run.stdout.contains("SURVIVED"),
            "the socket syscall was NOT blocked by the jail:\n{}",
            run.stdout
        );
        assert_eq!(
            run.status.signal(),
            Some(SIGSYS),
            "expected the child to be killed by SIGSYS ({SIGSYS}); got status {:?}, code {:?}\n{}",
            run.status,
            run.status.code(),
            run.stdout
        );
    }

    #[test]
    fn landlock_denies_a_file_that_was_readable_before_the_jail() {
        // Written and read BEFORE the child is spawned, so a failure inside the jail
        // cannot be confused with "the file was never there".
        let path = std::env::temp_dir().join(format!(
            "mw-sandbox-landlock-probe-{}.txt",
            std::process::id()
        ));
        std::fs::write(&path, b"readable before the jail").expect("seed the probe file");
        assert!(
            std::fs::read(&path).is_ok(),
            "the probe file must be readable first"
        );

        let run = run_child("open", &[(CHILD_PROBE_FILE, &path.to_string_lossy())]);
        let _ = std::fs::remove_file(&path);

        assert!(run.confined(), "the jail did not install:\n{}", run.stdout);
        assert_eq!(run.status.code(), Some(0), "child:\n{}", run.stdout);

        match run.layer("landlock") {
            "enforced" => assert!(
                run.stdout.contains("OPEN err"),
                "Landlock reported ENFORCED, so an empty ruleset must deny this read:\n{}",
                run.stdout
            ),
            other => eprintln!(
                "[mw-sandbox] SKIP landlock deny-all assertion: this kernel reports \
                 landlock={other} (needs >= 5.13 with Landlock enabled). Child said: {}",
                run.stdout
                    .lines()
                    .find(|l| l.starts_with("OPEN"))
                    .unwrap_or("(none)")
            ),
        }
    }
}

// ── the non-Linux arm ────────────────────────────────────────────────────────

#[cfg(not(target_os = "linux"))]
#[test]
fn kernel_jail_enforcement_is_not_exercised_on_this_platform() {
    // Deliberately asserts nothing about enforcement. `mw-sandbox`'s non-Linux arm is
    // a single shared `cfg(not(target_os = "linux"))` block with no kernel calls in
    // it, so a green run here would only prove that a documented no-op no-ops.
    // `tests/posture.rs` covers the contract that IS testable off Linux: a required
    // jail is refused rather than degraded, and no layer claims enforcement.
    eprintln!(
        "[mw-sandbox] SKIP kernel-jail enforcement on {}: seccomp/Landlock/namespaces \
         are Linux-only. The socket-kill and Landlock-deny probes run on Linux CI.",
        std::env::consts::OS
    );
}
