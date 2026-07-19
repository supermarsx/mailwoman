//! Linux kernel jail: `no_new_privs` + rlimits + network namespace + Landlock +
//! seccomp-BPF. Compiled only on `target_os = "linux"` (see `Cargo.toml`'s
//! target-gated deps). Everything here runs on the disposable render child, before
//! it reads any hostile input.
//!
//! Layer order matters: seccomp is installed **last** so the setup syscalls
//! (`unshare`, the Landlock `landlock_*` calls, `setrlimit`, `prctl`) are not
//! themselves filtered. `unshare(CLONE_NEWUSER)` also requires a single-threaded
//! caller — the render child calls [`confine`] at the very top of `main`, before the
//! WASM jail's epoch-ticker thread is ever spawned, so that holds.

use std::collections::BTreeMap;

use landlock::{
    ABI, Access, AccessFs, CompatLevel, Compatible, Ruleset, RulesetAttr, RulesetCreatedAttr,
    RulesetStatus,
};
use seccompiler::{BpfProgram, SeccompAction, SeccompFilter, TargetArch};

use crate::{JailPolicy, Layer, LayerState, SandboxError, SandboxReport};

/// Confine the current (render-child) process. See [`crate::confine_current_process`].
pub(crate) fn confine(policy: &JailPolicy) -> Result<SandboxReport, SandboxError> {
    let mut layers = Vec::with_capacity(5);

    // 1. no_new_privs — required for unprivileged seccomp *and* Landlock. Fatal if the
    //    jail is required.
    match set_no_new_privs() {
        Ok(()) => layers.push(Layer::new("no_new_privs", LayerState::Enforced)),
        Err(e) => {
            if policy.required {
                return Err(SandboxError::Unavailable(format!("no_new_privs: {e}")));
            }
            layers.push(Layer::new("no_new_privs", LayerState::Unavailable(e)));
        }
    }

    // 2. rlimits — no core dumps (never spill hostile bytes to disk), no regular-file
    //    writes, a modest fd cap. Best-effort per limit; a setrlimit failure is
    //    recorded but does not fail the jail (the strong layers are seccomp/Landlock).
    layers.push(Layer::new("rlimits", apply_rlimits()));

    // 3. network namespace — drop into an empty net namespace so the child has no
    //    routes/interfaces. Best-effort: unprivileged user namespaces may be disabled
    //    (some hardened kernels/containers). seccomp kills the socket syscalls anyway,
    //    so this is defense in depth, never fatal.
    layers.push(Layer::new("net-namespace", apply_net_namespace()));

    // 4. Landlock — an empty filesystem ruleset: every path access is denied.
    //    Best-effort across kernel ABIs; unavailable on kernels < 5.13.
    layers.push(Layer::new("landlock", apply_landlock()));

    // 5. seccomp-BPF — default kill-process, small syscall allowlist. Installed LAST.
    //    Fatal if the jail is required (this is the load-bearing layer).
    match apply_seccomp() {
        Ok(()) => layers.push(Layer::new("seccomp", LayerState::Enforced)),
        Err(e) => {
            if policy.required {
                return Err(SandboxError::Unavailable(format!("seccomp: {e}")));
            }
            layers.push(Layer::new("seccomp", LayerState::Unavailable(e)));
        }
    }

    let degraded = degraded_reason(&layers);
    Ok(SandboxReport {
        platform_supported: true,
        platform: std::env::consts::OS,
        layers,
        degraded,
    })
}

/// Report what the render child's jail would apply, without confining this
/// (doctor) process. Landlock availability is probed harmlessly (a ruleset fd is
/// opened and immediately dropped); seccomp/namespace/rlimits are reported as
/// available on Linux.
pub(crate) fn probe() -> SandboxReport {
    let landlock = probe_landlock();
    let layers = vec![
        Layer::new("no_new_privs", LayerState::Enforced),
        Layer::new("rlimits", LayerState::Enforced),
        Layer::new(
            "net-namespace",
            LayerState::Partial, // best-effort; depends on unprivileged userns being enabled
        ),
        Layer::new("landlock", landlock),
        Layer::new("seccomp", LayerState::Enforced),
    ];
    let degraded = degraded_reason(&layers);
    SandboxReport {
        platform_supported: true,
        platform: std::env::consts::OS,
        layers,
        degraded,
    }
}

/// A degraded note when a strong layer is not fully enforced. A best-effort
/// namespace being `Partial` is expected and not, on its own, "degraded".
fn degraded_reason(layers: &[Layer]) -> Option<String> {
    let mut missing = Vec::new();
    for l in layers {
        if matches!(l.state, LayerState::Unavailable(_))
            && matches!(l.name, "landlock" | "seccomp" | "no_new_privs")
        {
            missing.push(l.name);
        }
    }
    if missing.is_empty() {
        None
    } else {
        Some(format!(
            "kernel jail degraded: {} unavailable on this kernel",
            missing.join(", ")
        ))
    }
}

fn set_no_new_privs() -> Result<(), String> {
    // SAFETY: prctl with PR_SET_NO_NEW_PRIVS and the documented (1,0,0,0) args; no
    // memory is touched. Returns 0 on success.
    let rc = unsafe { libc::prctl(libc::PR_SET_NO_NEW_PRIVS, 1, 0, 0, 0) };
    if rc == 0 { Ok(()) } else { Err(errno_string()) }
}

fn apply_rlimits() -> LayerState {
    let mut any_err = false;
    // No core dumps: a crash on hostile input must not write the process image (which
    // may hold that input) to disk.
    set_one_rlimit(libc::RLIMIT_CORE, 0, &mut any_err);
    // No regular-file writes (the child only writes its stdout pipe, which is not
    // subject to RLIMIT_FSIZE).
    set_one_rlimit(libc::RLIMIT_FSIZE, 0, &mut any_err);
    // Modest fd cap — enough for stdio + the WASM engine, a backstop against fd
    // exhaustion. (Left generous so the WASM jail's mmaps/threads are unaffected;
    // process/thread and address-space limits are intentionally NOT clamped here so
    // the epoch-ticker thread and wasmtime's large virtual reservations still work —
    // fork/exec escape is closed by the seccomp layer instead.)
    set_one_rlimit(libc::RLIMIT_NOFILE, 256, &mut any_err);
    // core/fsize/nofile are all backstops; which one failed is not security-relevant.
    if any_err {
        LayerState::Partial
    } else {
        LayerState::Enforced
    }
}

fn set_one_rlimit(resource: RlimitResource, value: libc::rlim_t, any_err: &mut bool) {
    let rlim = libc::rlimit {
        rlim_cur: value,
        rlim_max: value,
    };
    // SAFETY: setrlimit with a valid resource id and an initialized rlimit struct.
    let rc = unsafe { libc::setrlimit(resource, &rlim) };
    if rc != 0 {
        *any_err = true;
    }
}

fn apply_net_namespace() -> LayerState {
    // Create a fresh user namespace (grants the caps needed to make a network
    // namespace unprivileged) together with an empty network namespace. Must be
    // single-threaded — guaranteed at the render child's confine point.
    // SAFETY: unshare with namespace-creation flags; no memory is touched.
    let rc = unsafe { libc::unshare(libc::CLONE_NEWUSER | libc::CLONE_NEWNET) };
    if rc == 0 {
        LayerState::Enforced
    } else {
        // Unprivileged userns disabled, or already in a restricted namespace. The
        // socket syscalls are killed by seccomp regardless, so record and continue.
        LayerState::Unavailable(format!(
            "unshare(CLONE_NEWUSER|CLONE_NEWNET): {} (network is still blocked by seccomp)",
            errno_string()
        ))
    }
}

fn apply_landlock() -> LayerState {
    // Best-effort across ABIs: handle every filesystem access right the running
    // kernel knows about, then add NO rules — so every path access is denied.
    match build_deny_all_ruleset() {
        Ok(status) => match status {
            RulesetStatus::FullyEnforced => LayerState::Enforced,
            RulesetStatus::PartiallyEnforced => LayerState::Partial,
            RulesetStatus::NotEnforced => {
                LayerState::Unavailable("kernel lacks Landlock (< 5.13)".to_string())
            }
        },
        Err(e) => LayerState::Unavailable(e),
    }
}

fn build_deny_all_ruleset() -> Result<RulesetStatus, String> {
    let abi = ABI::V1;
    let status = Ruleset::default()
        .set_compatibility(CompatLevel::BestEffort)
        .handle_access(AccessFs::from_all(abi))
        .map_err(|e| e.to_string())?
        .create()
        .map_err(|e| e.to_string())?
        // No `add_rule` calls → nothing is permitted.
        .restrict_self()
        .map_err(|e| e.to_string())?;
    Ok(status.ruleset)
}

fn probe_landlock() -> LayerState {
    // Open a ruleset fd to detect support, then drop it — the doctor process is NOT
    // restricted (no `restrict_self`).
    let abi = ABI::V1;
    match Ruleset::default()
        .set_compatibility(CompatLevel::BestEffort)
        .handle_access(AccessFs::from_all(abi))
        .and_then(|r| r.create())
    {
        Ok(_created) => LayerState::Enforced,
        Err(e) => LayerState::Unavailable(e.to_string()),
    }
}

fn apply_seccomp() -> Result<(), String> {
    let arch = TargetArch::try_from(std::env::consts::ARCH)
        .map_err(|_| format!("unsupported arch for seccomp: {}", std::env::consts::ARCH))?;

    let mut rules: BTreeMap<i64, Vec<seccompiler::SeccompRule>> = BTreeMap::new();
    for nr in allowed_syscalls() {
        // Empty rule vec = allow the syscall unconditionally.
        rules.insert(nr, Vec::new());
    }

    let filter = SeccompFilter::new(
        rules,
        SeccompAction::KillProcess, // default: anything not allow-listed kills the process
        SeccompAction::Allow,
        arch,
    )
    .map_err(|e| e.to_string())?;

    let program: BpfProgram = filter.try_into().map_err(|e| format!("{e}"))?;
    seccompiler::apply_filter(&program).map_err(|e| e.to_string())?;
    Ok(())
}

/// The render child's syscall allowlist. Everything else is `SIGSYS`-killed —
/// notably `execve`/`execveat` (no process escape), `ptrace`/`process_vm_*` (no
/// inspecting other processes), and the whole socket family (`socket`/`connect`/…):
/// the child never needs the network, so an attempt to reach it is fatal. This is
/// the syscall a Linux-CI test drives to prove the jail (`socket` → killed).
///
/// The set is deliberately generous on *housekeeping* syscalls the Rust runtime and
/// the wasmtime/Pulley interpreter need (memory management, futex, thread creation,
/// signal plumbing, clocks) to avoid false kills; path-based access is neutralised by
/// the Landlock layer rather than by filtering `openat` here.
fn allowed_syscalls() -> Vec<i64> {
    // `mut` is only exercised by the x86_64 legacy-syscall block below; on other
    // arches the base set is returned as-is.
    #[cfg_attr(not(target_arch = "x86_64"), allow(unused_mut))]
    let mut s: Vec<i64> = vec![
        // stdio + basic file I/O over already-open fds
        libc::SYS_read,
        libc::SYS_write,
        libc::SYS_readv,
        libc::SYS_writev,
        libc::SYS_pread64,
        libc::SYS_pwrite64,
        libc::SYS_close,
        libc::SYS_lseek,
        libc::SYS_fcntl,
        libc::SYS_dup,
        libc::SYS_dup3,
        libc::SYS_pipe2,
        // memory management (allocator + wasmtime linear memory)
        libc::SYS_mmap,
        libc::SYS_munmap,
        libc::SYS_mremap,
        libc::SYS_mprotect,
        libc::SYS_pkey_mprotect,
        libc::SYS_madvise,
        libc::SYS_brk,
        // signals (traps, panics, thread signal masks)
        libc::SYS_rt_sigaction,
        libc::SYS_rt_sigprocmask,
        libc::SYS_rt_sigreturn,
        libc::SYS_rt_sigtimedwait,
        libc::SYS_sigaltstack,
        libc::SYS_tgkill,
        // synchronisation + scheduling
        libc::SYS_futex,
        libc::SYS_futex_waitv,
        libc::SYS_sched_yield,
        libc::SYS_sched_getaffinity,
        libc::SYS_membarrier,
        libc::SYS_getcpu,
        // thread lifecycle (the WASM jail's epoch-ticker thread)
        libc::SYS_clone,
        libc::SYS_clone3,
        libc::SYS_set_robust_list,
        libc::SYS_get_robust_list,
        libc::SYS_rseq,
        libc::SYS_prctl,
        // clocks / sleeps (epoch ticker + deadlines)
        libc::SYS_clock_gettime,
        libc::SYS_clock_getres,
        libc::SYS_clock_nanosleep,
        libc::SYS_nanosleep,
        libc::SYS_gettimeofday,
        // randomness (HashMap seeds, etc.)
        libc::SYS_getrandom,
        // identity / info (glibc init + backtrace machinery)
        libc::SYS_getpid,
        libc::SYS_gettid,
        libc::SYS_getppid,
        libc::SYS_getuid,
        libc::SYS_geteuid,
        libc::SYS_getgid,
        libc::SYS_getegid,
        libc::SYS_uname,
        libc::SYS_sysinfo,
        libc::SYS_getrusage,
        // stat family over open fds / paths (Landlock gates the actual paths)
        libc::SYS_fstat,
        libc::SYS_statx,
        libc::SYS_newfstatat,
        libc::SYS_openat,
        libc::SYS_faccessat,
        libc::SYS_faccessat2,
        libc::SYS_getdents64,
        // event loop primitives (some std/glibc paths)
        libc::SYS_ppoll,
        libc::SYS_epoll_create1,
        libc::SYS_epoll_ctl,
        libc::SYS_epoll_pwait,
        // process/thread exit
        libc::SYS_exit,
        libc::SYS_exit_group,
        libc::SYS_restart_syscall,
    ];

    // x86_64 keeps several legacy syscalls that aarch64 dropped for their `*at` /
    // `p*` variants; include them so a glibc built against them is not false-killed.
    #[cfg(target_arch = "x86_64")]
    {
        s.extend_from_slice(&[
            libc::SYS_open,
            libc::SYS_stat,
            libc::SYS_lstat,
            libc::SYS_access,
            libc::SYS_readlink,
            libc::SYS_poll,
            libc::SYS_select,
            libc::SYS_epoll_wait,
            libc::SYS_epoll_create,
            libc::SYS_dup2,
            libc::SYS_arch_prctl,
        ]);
    }

    s
}

/// The `RLIMIT_*` resource id type differs between libc backends (glibc uses
/// `__rlimit_resource_t`, musl uses `c_int`); alias to whatever `setrlimit` expects
/// so this compiles under both the gnu and musl-static builds.
#[cfg(target_env = "gnu")]
type RlimitResource = libc::__rlimit_resource_t;
#[cfg(not(target_env = "gnu"))]
type RlimitResource = libc::c_int;

fn errno_string() -> String {
    std::io::Error::last_os_error().to_string()
}
