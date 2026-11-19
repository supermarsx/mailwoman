//! Read-only bounded server resource observations. Percentages never guess at missing data.
use serde::Serialize;
use std::{
    path::Path,
    sync::{Mutex, OnceLock},
    time::{Instant, SystemTime, UNIX_EPOCH},
};

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ResourceSnapshot {
    pub observed_at: u64,
    /// Affinity, cgroup quota and host capacity combined. Fractional cores are preserved.
    pub effective_cpu_cores: f64,
    pub effective_memory_limit_bytes: Option<u64>,
    pub effective_memory_available_bytes: Option<u64>,
    /// Entire-host memory, which may exceed the container's entitlement.
    pub host_memory_total_bytes: Option<u64>,
    pub host_memory_available_bytes: Option<u64>,
    pub process_resident_bytes: Option<u64>,
    /// Process CPU use: 100% means one logical core, not the whole host/container.
    pub process_cpu_percent: Option<f64>,
    pub cpu_sample_seconds: Option<f64>,
    pub filesystem_total_bytes: Option<u64>,
    pub filesystem_available_bytes: Option<u64>,
    pub cgroup_detected: bool,
}
#[derive(Default)]
struct CpuSample {
    previous: Option<(Instant, f64)>,
    percent: Option<f64>,
    seconds: Option<f64>,
}
impl CpuSample {
    fn observe(&mut self, now: Instant, cpu_seconds: Option<f64>) -> (Option<f64>, Option<f64>) {
        let Some(total) = cpu_seconds.filter(|value| value.is_finite() && *value >= 0.0) else {
            self.previous = None;
            self.percent = None;
            self.seconds = None;
            return (None, None);
        };
        if let Some((at, before)) = self.previous {
            let elapsed = now.saturating_duration_since(at).as_secs_f64();
            if elapsed < 0.1 {
                return (self.percent, self.seconds);
            }
            self.previous = Some((now, total));
            if elapsed <= 120.0 && total >= before {
                self.percent = Some((total - before) / elapsed * 100.0);
                self.seconds = Some(elapsed)
            } else {
                self.percent = None;
                self.seconds = None
            }
        } else {
            self.previous = Some((now, total));
        }
        (self.percent, self.seconds)
    }
}
/// Observe the process and the filesystem containing the operator-owned data path.
/// No subprocesses, network calls, directory traversals or writes are performed.
pub fn snapshot(data_path: &Path) -> ResourceSnapshot {
    static CPU: OnceLock<Mutex<CpuSample>> = OnceLock::new();
    let capacity = crate::resources::snapshot();
    let (resident, cpu) = process_usage();
    let (percent, seconds) = CPU
        .get_or_init(|| Mutex::new(CpuSample::default()))
        .lock()
        .unwrap_or_else(|p| p.into_inner())
        .observe(Instant::now(), cpu);
    let (host_total, host_available) = host_memory();
    let disk = disk_space(data_path);
    ResourceSnapshot {
        observed_at: SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs(),
        effective_cpu_cores: f64::from(capacity.cpu_millis) / 1000.0,
        effective_memory_limit_bytes: capacity.memory_limit_bytes,
        effective_memory_available_bytes: capacity.memory_available_bytes,
        host_memory_total_bytes: host_total,
        host_memory_available_bytes: host_available,
        process_resident_bytes: resident,
        process_cpu_percent: percent,
        cpu_sample_seconds: seconds,
        filesystem_total_bytes: disk.map(|d| d.0),
        filesystem_available_bytes: disk.map(|d| d.1),
        cgroup_detected: cgroup_detected(),
    }
}
#[cfg(target_os = "linux")]
fn read_small(path: &str) -> Option<String> {
    use std::io::Read;
    let mut value = String::new();
    std::fs::File::open(path)
        .ok()?
        .take(131073)
        .read_to_string(&mut value)
        .ok()?;
    (value.len() <= 131072).then_some(value)
}
#[cfg(target_os = "linux")]
fn kib_field(text: &str, key: &str) -> Option<u64> {
    text.lines().find_map(|line| {
        let mut fields = line.split_whitespace();
        (fields.next()? == key)
            .then(|| fields.next()?.parse::<u64>().ok()?.checked_mul(1024))
            .flatten()
    })
}
#[cfg(target_os = "linux")]
fn host_memory() -> (Option<u64>, Option<u64>) {
    let text = read_small("/proc/meminfo").unwrap_or_default();
    (
        kib_field(&text, "MemTotal:"),
        kib_field(&text, "MemAvailable:").or_else(|| kib_field(&text, "MemFree:")),
    )
}
#[cfg(windows)]
fn host_memory() -> (Option<u64>, Option<u64>) {
    let resources = crate::resources::snapshot();
    (
        resources.memory_limit_bytes,
        resources.memory_available_bytes,
    )
}
#[cfg(not(any(windows, target_os = "linux")))]
fn host_memory() -> (Option<u64>, Option<u64>) {
    (None, None)
}
#[cfg(target_os = "linux")]
fn cgroup_detected() -> bool {
    read_small("/proc/self/cgroup").is_some_and(|value| {
        value
            .lines()
            .any(|line| line.splitn(3, ':').nth(2).is_some_and(|path| path != "/"))
    }) || std::path::Path::new("/.dockerenv").exists()
}
#[cfg(not(target_os = "linux"))]
fn cgroup_detected() -> bool {
    false
}
#[cfg(any(target_os = "linux", test))]
fn linux_cpu_ticks(stat: &str) -> Option<u64> {
    // comm may contain spaces and parentheses; fields after its final ')' start at field 3.
    let tail = stat.rsplit_once(')')?.1;
    let fields: Vec<_> = tail.split_whitespace().take(15).collect();
    fields
        .get(11)?
        .parse::<u64>()
        .ok()?
        .checked_add(fields.get(12)?.parse::<u64>().ok()?)
}
#[cfg(target_os = "linux")]
fn process_usage() -> (Option<u64>, Option<f64>) {
    let resident = read_small("/proc/self/status").and_then(|text| kib_field(&text, "VmRSS:"));
    // SAFETY: sysconf has no pointer inputs and does not mutate process state.
    let ticks = unsafe { libc::sysconf(libc::_SC_CLK_TCK) };
    let cpu = (ticks > 0)
        .then(|| {
            read_small("/proc/self/stat")
                .as_deref()
                .and_then(linux_cpu_ticks)
                .map(|value| value as f64 / ticks as f64)
        })
        .flatten();
    (resident, cpu)
}
#[cfg(windows)]
fn process_usage() -> (Option<u64>, Option<f64>) {
    #[repr(C)]
    #[derive(Default)]
    struct FileTime {
        low: u32,
        high: u32,
    }
    #[repr(C)]
    #[derive(Default)]
    struct ProcessMemory {
        cb: u32,
        page_faults: u32,
        peak_working: usize,
        working: usize,
        quota_peak_paged: usize,
        quota_paged: usize,
        quota_peak_nonpaged: usize,
        quota_nonpaged: usize,
        pagefile: usize,
        peak_pagefile: usize,
    }
    #[link(name = "kernel32")]
    unsafe extern "system" {
        fn GetCurrentProcess() -> *mut core::ffi::c_void;
        fn GetProcessTimes(
            handle: *mut core::ffi::c_void,
            creation: *mut FileTime,
            exit: *mut FileTime,
            kernel: *mut FileTime,
            user: *mut FileTime,
        ) -> i32;
    }
    #[link(name = "psapi")]
    unsafe extern "system" {
        fn GetProcessMemoryInfo(
            handle: *mut core::ffi::c_void,
            memory: *mut ProcessMemory,
            size: u32,
        ) -> i32;
    }
    let mut creation = FileTime::default();
    let mut exit = FileTime::default();
    let mut kernel = FileTime::default();
    let mut user = FileTime::default();
    let mut memory = ProcessMemory {
        cb: std::mem::size_of::<ProcessMemory>() as u32,
        ..Default::default()
    };
    // SAFETY: Windows' process pseudo-handle needs no closing; each ABI-compatible
    // output structure stays writable for the call, and no pointer is retained.
    unsafe {
        let handle = GetCurrentProcess();
        let cpu = (GetProcessTimes(handle, &mut creation, &mut exit, &mut kernel, &mut user) != 0)
            .then(|| {
                (((u64::from(kernel.high) << 32) | u64::from(kernel.low)) as f64
                    + ((u64::from(user.high) << 32) | u64::from(user.low)) as f64)
                    / 10_000_000.0
            });
        let size = memory.cb;
        let resident =
            (GetProcessMemoryInfo(handle, &mut memory, size) != 0).then_some(memory.working as u64);
        (resident, cpu)
    }
}
#[cfg(not(any(windows, target_os = "linux")))]
fn process_usage() -> (Option<u64>, Option<f64>) {
    (None, None)
}
#[cfg(unix)]
fn disk_space(path: &Path) -> Option<(u64, u64)> {
    use std::os::unix::ffi::OsStrExt;
    let path = std::ffi::CString::new(path.as_os_str().as_bytes()).ok()?;
    let mut stat = std::mem::MaybeUninit::<libc::statvfs>::uninit();
    // SAFETY: NUL-terminated filename and correctly sized writable output; only
    // inspect initialized output after statvfs has reported success.
    if unsafe { libc::statvfs(path.as_ptr(), stat.as_mut_ptr()) } != 0 {
        return None;
    }
    let stat = unsafe { stat.assume_init() };
    Some((
        (stat.f_blocks as u64).checked_mul(stat.f_frsize as u64)?,
        (stat.f_bavail as u64).checked_mul(stat.f_frsize as u64)?,
    ))
}
#[cfg(windows)]
fn disk_space(path: &Path) -> Option<(u64, u64)> {
    use std::os::windows::ffi::OsStrExt;
    #[link(name = "kernel32")]
    unsafe extern "system" {
        fn GetDiskFreeSpaceExW(
            path: *const u16,
            available: *mut u64,
            total: *mut u64,
            free: *mut u64,
        ) -> i32;
    }
    let mut path: Vec<u16> = path.as_os_str().encode_wide().collect();
    if path.contains(&0) {
        return None;
    }
    path.push(0);
    let (mut available, mut total, mut free) = (0, 0, 0);
    // SAFETY: valid NUL-terminated UTF-16 path; outputs are writable u64 values.
    (unsafe { GetDiskFreeSpaceExW(path.as_ptr(), &mut available, &mut total, &mut free) } != 0)
        .then_some((total, available))
}
#[cfg(not(any(unix, windows)))]
fn disk_space(_path: &Path) -> Option<(u64, u64)> {
    None
}
#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;
    #[test]
    fn cpu_rates_are_interval_based_with_honest_warmup_and_reset() {
        let mut sample = CpuSample::default();
        let now = Instant::now();
        assert_eq!(sample.observe(now, Some(1.0)), (None, None));
        assert_eq!(
            sample.observe(now + Duration::from_secs(2), Some(4.0)),
            (Some(150.0), Some(2.0))
        );
        assert_eq!(
            sample.observe(now + Duration::from_secs(3), Some(2.0)),
            (None, None)
        );
        assert_eq!(
            sample.observe(now + Duration::from_secs(4), None),
            (None, None)
        );
    }
    #[test]
    fn process_stat_supports_spaces_and_parentheses_in_command() {
        assert_eq!(
            linux_cpu_ticks("123 (mail worker)name) R 1 2 3 4 5 6 7 8 9 10 25 15 0"),
            Some(40)
        );
        assert_eq!(linux_cpu_ticks("invalid"), None);
    }
    #[test]
    fn filesystem_and_capacity_snapshot_is_read_only_and_bounded() {
        let report = snapshot(&std::env::temp_dir());
        assert!(report.effective_cpu_cores > 0.0);
        assert!(report.filesystem_total_bytes.is_some());
        assert!(report.filesystem_available_bytes <= report.filesystem_total_bytes);
        let json = serde_json::to_value(report).unwrap();
        assert!(json.get("observedAt").is_some());
    }
}
