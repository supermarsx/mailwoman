//! Worker threads of the server's tokio runtime (`MW_RUNTIME_WORKER_THREADS`).
//!
//! Default: the number of CPUs Rust's `std::thread::available_parallelism`
//! reports. On Linux that is the size of the process's CPU affinity mask (the
//! container's cpuset), lowered to a cgroup CPU quota when one is set (rounded
//! up). That is also what `#[tokio::main]` used before this knob existed (tokio
//! 1.54 reads the same function), so the default is unchanged; the knob makes
//! it explicit, bounded and reported at start-up.
//!
//! `TOKIO_WORKER_THREADS`, tokio's own variable, is still honoured when the
//! Mailwoman knob is not set, under the same bounds (tokio would otherwise
//! panic on a bad value). An invalid value of either refuses start-up.

/// The Mailwoman knob.
pub const ENV: &str = "MW_RUNTIME_WORKER_THREADS";
/// tokio's own variable, honoured when [`ENV`] is unset.
pub const TOKIO_ENV: &str = "TOKIO_WORKER_THREADS";
/// Highest accepted worker count.
pub const MAX_WORKERS: usize = 1024;

/// Where the worker count came from.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Source {
    /// `MW_RUNTIME_WORKER_THREADS`.
    Configured,
    /// `TOKIO_WORKER_THREADS`.
    TokioEnvironment,
    /// `available_parallelism` (cpuset / CPU quota).
    AvailableCpus,
    /// `available_parallelism` failed; one worker.
    Fallback,
}
impl Source {
    pub fn label(self) -> &'static str {
        match self {
            Source::Configured => "MW_RUNTIME_WORKER_THREADS",
            Source::TokioEnvironment => "TOKIO_WORKER_THREADS",
            Source::AvailableCpus => "available CPUs (cpuset or CPU quota)",
            Source::Fallback => "available CPUs unknown: 1",
        }
    }
}

/// The resolved runtime size.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Plan {
    pub workers: usize,
    pub source: Source,
    /// What `available_parallelism` reported, if anything.
    pub available_cpus: Option<usize>,
}

fn parse(name: &str, text: &str) -> Result<usize, String> {
    match text.trim().parse::<usize>() {
        Ok(n) if (1..=MAX_WORKERS).contains(&n) => Ok(n),
        _ => Err(format!(
            "{name} must be auto or a whole number from 1 to {MAX_WORKERS}"
        )),
    }
}

/// Decide the worker count from the two variables and the CPU reading. An
/// unset or empty value, or `auto`, means the default.
pub fn resolve(
    configured: Option<&str>,
    tokio: Option<&str>,
    available_cpus: Option<usize>,
) -> Result<Plan, String> {
    fn set(value: Option<&str>) -> Option<&str> {
        value
            .map(str::trim)
            .filter(|v| !v.is_empty() && !v.eq_ignore_ascii_case("auto"))
    }
    let (workers, source) = if let Some(text) = set(configured) {
        (parse(ENV, text)?, Source::Configured)
    } else if let Some(text) = set(tokio) {
        (parse(TOKIO_ENV, text)?, Source::TokioEnvironment)
    } else {
        match available_cpus {
            Some(n) if n >= 1 => (n.min(MAX_WORKERS), Source::AvailableCpus),
            _ => (1, Source::Fallback),
        }
    };
    Ok(Plan {
        workers,
        source,
        available_cpus,
    })
}

/// [`resolve`] over the process environment. A value that is not Unicode is
/// invalid.
pub fn from_env() -> Result<Plan, String> {
    let read = |name: &str| match std::env::var(name) {
        Ok(value) => Ok(Some(value)),
        Err(std::env::VarError::NotPresent) => Ok(None),
        Err(std::env::VarError::NotUnicode(_)) => Err(format!("{name} is not valid Unicode")),
    };
    let configured = read(ENV)?;
    let tokio = read(TOKIO_ENV)?;
    let available = std::thread::available_parallelism().ok().map(usize::from);
    resolve(configured.as_deref(), tokio.as_deref(), available)
}

/// The multi-thread runtime `main` runs on (what `#[tokio::main]` built, with
/// the worker count set explicitly).
pub fn build(plan: &Plan) -> std::io::Result<tokio::runtime::Runtime> {
    tokio::runtime::Builder::new_multi_thread()
        .worker_threads(plan.workers)
        .enable_all()
        .build()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_is_the_available_cpu_count() {
        for cpus in [1, 4, 26, 56, 80] {
            let plan = resolve(None, None, Some(cpus)).unwrap();
            assert_eq!(plan.workers, cpus);
            assert_eq!(plan.source, Source::AvailableCpus);
            assert_eq!(plan.available_cpus, Some(cpus));
        }
        for unset in ["", "  ", "auto", "AUTO"] {
            assert_eq!(resolve(Some(unset), None, Some(8)).unwrap().workers, 8);
        }
        assert_eq!(resolve(None, None, Some(5000)).unwrap().workers, MAX_WORKERS);
        let unknown = resolve(None, None, None).unwrap();
        assert_eq!((unknown.workers, unknown.source), (1, Source::Fallback));
    }

    #[test]
    fn configured_value_wins_and_is_bounded() {
        let plan = resolve(Some("128"), Some("3"), Some(4)).unwrap();
        assert_eq!((plan.workers, plan.source), (128, Source::Configured));
        assert_eq!(resolve(Some(" 1 "), None, Some(4)).unwrap().workers, 1);
        assert_eq!(resolve(Some("1024"), None, Some(4)).unwrap().workers, 1024);
        for bad in ["0", "1025", "-1", "4.5", "four", "1e3", "99999999999999999999999"] {
            let error = resolve(Some(bad), None, Some(4)).unwrap_err();
            assert!(error.starts_with(ENV), "{bad}: {error}");
        }
    }

    #[test]
    fn tokio_variable_is_honoured_only_without_the_knob_and_under_the_same_bounds() {
        let plan = resolve(None, Some("6"), Some(4)).unwrap();
        assert_eq!((plan.workers, plan.source), (6, Source::TokioEnvironment));
        assert!(resolve(None, Some("0"), Some(4)).unwrap_err().starts_with(TOKIO_ENV));
        assert!(resolve(None, Some("x"), Some(4)).is_err());
        // The knob set to a valid value overrides an invalid tokio value.
        assert_eq!(resolve(Some("2"), Some("x"), Some(4)).unwrap().workers, 2);
        assert_eq!(resolve(Some("auto"), Some("3"), Some(4)).unwrap().workers, 3);
    }

    #[test]
    fn the_built_runtime_has_the_planned_worker_count() {
        let plan = resolve(Some("3"), None, Some(1)).unwrap();
        let runtime = build(&plan).unwrap();
        assert_eq!(runtime.metrics().num_workers(), 3);
    }
}
