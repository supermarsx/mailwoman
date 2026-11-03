//! Bounded local resource discovery for adaptive image admission. No commands,
//! network requests or optional dependencies; unknown memory stays unknown.
use std::sync::{Mutex, OnceLock};
use std::time::{Duration, Instant};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct ImageResources {
    /// Effective logical CPU capacity, preserving fractional cgroup quotas.
    pub cpu_millis: u32,
    pub memory_limit_bytes: Option<u64>,
    pub memory_available_bytes: Option<u64>,
}

const REFRESH: Duration = Duration::from_secs(5);

#[derive(Default)]
struct Cached(Option<(Instant, ImageResources)>);
impl Cached {
    fn get(&mut self, now: Instant, load: impl FnOnce() -> ImageResources) -> ImageResources {
        if let Some((sampled, value)) = self.0
            && now.saturating_duration_since(sampled) < REFRESH
        {
            return value;
        }
        let value = load();
        self.0 = Some((now, value));
        value
    }
}

pub(crate) fn snapshot() -> ImageResources {
    static SAMPLE: OnceLock<Mutex<Cached>> = OnceLock::new();
    SAMPLE
        .get_or_init(|| Mutex::new(Cached::default()))
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
        .get(Instant::now(), discover)
}

fn discover() -> ImageResources {
    let cpu = std::thread::available_parallelism()
        .map(|count| count.get().saturating_mul(1000).min(u32::MAX as usize) as u32)
        .unwrap_or(1000);
    #[cfg(target_os = "linux")]
    {
        linux_resources(cpu, &read_small)
    }
    #[cfg(windows)]
    {
        let memory = windows_memory();
        ImageResources {
            cpu_millis: cpu,
            memory_limit_bytes: memory.map(|memory| memory.0),
            memory_available_bytes: memory.map(|memory| memory.1),
        }
    }
    #[cfg(not(any(target_os = "linux", windows)))]
    {
        ImageResources {
            cpu_millis: cpu,
            memory_limit_bytes: None,
            memory_available_bytes: None,
        }
    }
}

#[cfg(target_os = "linux")]
fn read_small(path: &str) -> Option<String> {
    use std::io::Read;
    const MAX_BYTES: u64 = 128 * 1024;
    let mut text = String::new();
    std::fs::File::open(path)
        .ok()?
        .take(MAX_BYTES + 1)
        .read_to_string(&mut text)
        .ok()?;
    (text.len() as u64 <= MAX_BYTES).then_some(text)
}

#[cfg(windows)]
fn windows_memory() -> Option<(u64, u64)> {
    // The fixed Win32 ABI avoids adding a system-information dependency or
    // changing dependency features for the other platforms.
    #[repr(C)]
    #[derive(Default)]
    struct MemoryStatus {
        length: u32,
        load: u32,
        total_physical: u64,
        available_physical: u64,
        total_page_file: u64,
        available_page_file: u64,
        total_virtual: u64,
        available_virtual: u64,
        available_extended_virtual: u64,
    }
    #[link(name = "kernel32")]
    unsafe extern "system" {
        fn GlobalMemoryStatusEx(status: *mut MemoryStatus) -> i32;
    }
    let mut status = MemoryStatus {
        length: std::mem::size_of::<MemoryStatus>() as u32,
        ..Default::default()
    };
    // SAFETY: the writable, correctly sized repr(C) structure lives through the
    // synchronous call; the function retains no pointer.
    let valid = unsafe { GlobalMemoryStatusEx(&mut status) } != 0;
    (valid && status.total_physical > 0).then_some((
        status.total_physical,
        status.available_physical.min(status.total_physical),
    ))
}

#[cfg(any(target_os = "linux", test))]
mod linux {
    use super::ImageResources;

    fn smaller(current: &mut Option<u64>, value: u64) {
        *current = Some(current.map_or(value, |previous| previous.min(value)));
    }

    fn cpu_ranges(text: &str) -> Option<Vec<(u32, u32)>> {
        if text.len() > 32 * 1024 {
            return None;
        }
        let mut ranges = Vec::new();
        for entry in text.trim().split(',').take(1025) {
            if ranges.len() == 1024 {
                return None;
            }
            let (first, last) = entry.split_once('-').unwrap_or((entry, entry));
            let first: u32 = first.parse().ok()?;
            let last: u32 = last.parse().ok()?;
            if first > last {
                return None;
            }
            ranges.push((first, last));
        }
        ranges.sort_unstable();
        let mut merged: Vec<(u32, u32)> = Vec::new();
        for (first, last) in ranges {
            if let Some(previous) = merged.last_mut()
                && first as u64 <= previous.1 as u64 + 1
            {
                previous.1 = previous.1.max(last);
            } else {
                merged.push((first, last));
            }
        }
        Some(merged)
    }

    fn cpu_count(ranges: &[(u32, u32)]) -> Option<u32> {
        let count: u64 = ranges
            .iter()
            .map(|(first, last)| *last as u64 - *first as u64 + 1)
            .sum();
        (count > 0).then_some((count.saturating_mul(1000)).min(u32::MAX as u64) as u32)
    }

    pub(super) fn cpus(text: &str) -> Option<u32> {
        cpu_count(&cpu_ranges(text)?)
    }

    fn affinity(status: &str, online: Option<&str>) -> Option<u32> {
        let allowed = status
            .lines()
            .take(4096)
            .find_map(|line| line.strip_prefix("Cpus_allowed_list:"))?;
        let allowed = cpu_ranges(allowed)?;
        let Some(online) = online.and_then(cpu_ranges) else {
            return cpu_count(&allowed);
        };
        let (mut left, mut right) = (0, 0);
        let mut intersection = Vec::new();
        while left < allowed.len() && right < online.len() {
            let start = allowed[left].0.max(online[right].0);
            let end = allowed[left].1.min(online[right].1);
            if start <= end {
                intersection.push((start, end));
            }
            if allowed[left].1 < online[right].1 {
                left += 1;
            } else {
                right += 1;
            }
        }
        cpu_count(&intersection)
    }

    pub(super) fn quota(quota: &str, period: &str) -> Option<u32> {
        let quota: u64 = quota.trim().parse().ok()?;
        let period: u64 = period.trim().parse().ok()?;
        if quota == 0 || period == 0 {
            return None;
        }
        Some(((quota as u128 * 1000) / period as u128).clamp(1, u32::MAX as u128) as u32)
    }

    fn meminfo(text: &str, name: &str) -> Option<u64> {
        let line = text
            .lines()
            .take(4096)
            .find_map(|line| line.strip_prefix(name))?;
        let mut words = line.split_whitespace();
        let value: u64 = words.next()?.parse().ok()?;
        (words.next()? == "kB" && words.next().is_none())
            .then(|| value.checked_mul(1024))
            .flatten()
    }

    fn components(path: &str) -> Option<Vec<&str>> {
        if path.len() > 4096 || !path.starts_with('/') || path.contains(['\0', '\n', '\r']) {
            return None;
        }
        let parts: Vec<_> = path.split('/').filter(|part| !part.is_empty()).collect();
        (parts.len() <= 64 && parts.iter().all(|part| *part != "." && *part != ".."))
            .then_some(parts)
    }

    fn unescape(path: &str) -> Option<String> {
        let mut output = String::new();
        let mut remaining = path;
        while let Some(index) = remaining.find('\\') {
            output.push_str(&remaining[..index]);
            let escape = remaining.get(index..index + 4)?;
            output.push(match escape {
                "\\040" => ' ',
                "\\011" => '\t',
                "\\012" => '\n',
                "\\134" => '\\',
                _ => return None,
            });
            remaining = &remaining[index + 4..];
        }
        output.push_str(remaining);
        components(&output)?;
        Some(output)
    }

    /// Resolve a process path against a mount's delegated root, then walk only
    /// within that mount. A cgroup namespace may report `/` for its mount root.
    pub(super) fn hierarchy(root: &str, mount: &str, member: &str) -> Option<Vec<String>> {
        let root = components(root)?;
        let mount = components(mount)?;
        let member = components(member)?;
        let relative = if member.starts_with(&root) {
            &member[root.len()..]
        } else if member.is_empty() {
            &[]
        } else {
            return None;
        };
        let mut paths = Vec::new();
        for length in (0..=relative.len()).rev() {
            paths.push(format!(
                "/{}",
                mount
                    .iter()
                    .chain(relative[..length].iter())
                    .copied()
                    .collect::<Vec<_>>()
                    .join("/")
            ));
        }
        Some(paths)
    }

    fn limit(text: &str, unified: bool) -> Option<Option<u64>> {
        let text = text.trim();
        if unified && text == "max" {
            return Some(None);
        }
        let number: u64 = text.parse().ok()?;
        // v1's unlimited sentinel is near i64::MAX (page-size dependent).
        Some((unified || number < 1 << 60).then_some(number))
    }

    pub(super) fn resources(cpu: u32, read: &impl Fn(&str) -> Option<String>) -> ImageResources {
        let host = read("/proc/meminfo").unwrap_or_default();
        // available_parallelism rounds fractional Linux quotas down. Start
        // from the real affinity/online intersection, then apply quota minima.
        let affinity_cpu = read("/proc/self/status").and_then(|status| {
            affinity(&status, read("/sys/devices/system/cpu/online").as_deref())
        });
        let mut result = ImageResources {
            cpu_millis: affinity_cpu.unwrap_or(cpu).max(1),
            memory_limit_bytes: meminfo(&host, "MemTotal:"),
            memory_available_bytes: meminfo(&host, "MemAvailable:")
                .or_else(|| meminfo(&host, "MemFree:")),
        };
        let members = read("/proc/self/cgroup").unwrap_or_default();
        let mounts = read("/proc/self/mountinfo").unwrap_or_default();
        let needs_memory = members.lines().take(64).any(|line| {
            line.split(':').nth(1).is_some_and(|controllers| {
                controllers.is_empty()
                    || controllers
                        .split(',')
                        .any(|controller| controller == "memory")
            })
        });
        let (mut inspected_memory, mut unknown_headroom) = (false, false);
        for line in mounts
            .lines()
            .take(4096)
            .filter(|line| line.contains(" - cgroup"))
            .take(64)
        {
            let Some((before, after)) = line.split_once(" - ") else {
                continue;
            };
            let fields: Vec<_> = before.split_whitespace().collect();
            let options: Vec<_> = after.split_whitespace().collect();
            if fields.len() < 5 || options.len() < 3 {
                continue;
            }
            let unified = options[0] == "cgroup2";
            if !unified && options[0] != "cgroup" {
                continue;
            }
            let Some(root) = unescape(fields[3]) else {
                continue;
            };
            let Some(mount) = unescape(fields[4]) else {
                continue;
            };
            let controls: Vec<_> = options[2].split(',').collect();
            for member in members.lines().take(64) {
                let mut member = member.splitn(3, ':');
                let _ = member.next();
                let (Some(controllers), Some(path)) = (member.next(), member.next()) else {
                    continue;
                };
                if unified != controllers.is_empty() {
                    continue;
                }
                if !unified
                    && !controllers
                        .split(',')
                        .any(|controller| controls.contains(&controller))
                {
                    continue;
                }
                let Some(paths) = hierarchy(&root, &mount, path) else {
                    continue;
                };
                let has = |controller| unified || controls.contains(&controller);
                for path in paths {
                    if has("cpu") {
                        let value = if unified {
                            read(&format!("{path}/cpu.max")).and_then(|text| {
                                let mut fields = text.split_whitespace();
                                let value = quota(fields.next()?, fields.next()?);
                                fields.next().is_none().then_some(value).flatten()
                            })
                        } else {
                            read(&format!("{path}/cpu.cfs_quota_us"))
                                .zip(read(&format!("{path}/cpu.cfs_period_us")))
                                .and_then(|(q, p)| quota(&q, &p))
                        };
                        if let Some(value) = value {
                            result.cpu_millis = result.cpu_millis.min(value);
                        }
                    }
                    if has("cpuset") {
                        let value = read(&format!("{path}/cpuset.cpus.effective"))
                            .and_then(|text| cpus(&text))
                            .or_else(|| {
                                read(&format!("{path}/cpuset.cpus")).and_then(|text| cpus(&text))
                            });
                        if let Some(value) = value {
                            result.cpu_millis = result.cpu_millis.min(value);
                        }
                    }
                    if has("memory") {
                        let name = if unified {
                            "memory.max"
                        } else {
                            "memory.limit_in_bytes"
                        };
                        let Some(memory_limit) =
                            read(&format!("{path}/{name}")).and_then(|text| limit(&text, unified))
                        else {
                            continue;
                        };
                        inspected_memory = true;
                        if let Some(memory_limit) = memory_limit {
                            smaller(&mut result.memory_limit_bytes, memory_limit);
                            let usage = if unified {
                                "memory.current"
                            } else {
                                "memory.usage_in_bytes"
                            };
                            if let Some(usage) = read(&format!("{path}/{usage}"))
                                .and_then(|text| text.trim().parse::<u64>().ok())
                            {
                                smaller(
                                    &mut result.memory_available_bytes,
                                    memory_limit.saturating_sub(usage),
                                );
                            } else {
                                unknown_headroom = true;
                            }
                        }
                    }
                }
            }
        }
        if needs_memory && !inspected_memory {
            result.memory_limit_bytes = None;
            result.memory_available_bytes = None;
        } else if unknown_headroom {
            result.memory_available_bytes = None;
        }
        if let Some(limit) = result.memory_limit_bytes {
            result.memory_available_bytes = result
                .memory_available_bytes
                .map(|available| available.min(limit));
        }
        result
    }
}

#[cfg(target_os = "linux")]
use linux::resources as linux_resources;

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;
    fn sample(files: &[(&str, &str)]) -> ImageResources {
        let files: HashMap<_, _> = files.iter().copied().collect();
        linux::resources(64000, &|path| {
            files.get(path).map(|value| value.to_string())
        })
    }

    #[test]
    fn nested_v2_applies_fractional_cpu_cpuset_and_ancestor_memory_headroom() {
        let result = sample(&[
            (
                "/proc/meminfo",
                "MemTotal: 8388608 kB\nMemAvailable: 4194304 kB\n",
            ),
            ("/proc/self/cgroup", "0::/tenant/mail\n"),
            (
                "/proc/self/mountinfo",
                "30 20 0:1 / /sys/fs/cgroup rw - cgroup2 cgroup rw\n",
            ),
            ("/sys/fs/cgroup/tenant/mail/cpu.max", "max 100000"),
            ("/sys/fs/cgroup/tenant/cpu.max", "150000 100000"),
            ("/sys/fs/cgroup/tenant/mail/cpuset.cpus.effective", "0-7"),
            ("/sys/fs/cgroup/tenant/mail/memory.max", "1073741824"),
            ("/sys/fs/cgroup/tenant/mail/memory.current", "268435456"),
            ("/sys/fs/cgroup/tenant/memory.max", "2147483648"),
            ("/sys/fs/cgroup/tenant/memory.current", "2013265920"),
        ]);
        assert_eq!(result.cpu_millis, 1500);
        assert_eq!(result.memory_limit_bytes, Some(1073741824));
        assert_eq!(result.memory_available_bytes, Some(134217728));
    }

    #[test]
    fn actual_affinity_preserves_fractional_quota_and_intersects_online_cpus() {
        let mut files = HashMap::from([
            ("/proc/self/status", "Cpus_allowed_list:\t0-3\n"),
            ("/sys/devices/system/cpu/online", "2-5"),
            ("/proc/self/cgroup", "0::/"),
            (
                "/proc/self/mountinfo",
                "30 20 0:1 / /cg rw - cgroup2 cgroup rw",
            ),
            ("/cg/cpu.max", "150000 100000"),
            ("/cg/memory.max", "max"),
        ]);
        let read = |path: &str| files.get(path).map(|value| value.to_string());
        assert_eq!(linux::resources(1000, &read).cpu_millis, 1500);
        files.insert("/cg/cpu.max", "400000 100000");
        assert_eq!(
            linux::resources(1000, &|path| files.get(path).map(|value| value.to_string()))
                .cpu_millis,
            2000
        );
    }

    #[test]
    fn v1_controller_mounts_and_delegated_roots_are_resolved() {
        let result = sample(&[
            (
                "/proc/self/cgroup",
                "2:cpu,cpuacct:/docker/abc/mail\n3:memory:/docker/abc/mail\n4:cpuset:/docker/abc/mail\n",
            ),
            (
                "/proc/self/mountinfo",
                "30 20 0:1 /docker/abc /cg/cpu rw - cgroup cgroup rw,cpu,cpuacct\n31 20 0:2 /docker/abc /cg/mem rw - cgroup cgroup rw,memory\n32 20 0:3 /docker/abc /cg/set rw - cgroup cgroup rw,cpuset\n",
            ),
            ("/cg/cpu/mail/cpu.cfs_quota_us", "50000"),
            ("/cg/cpu/mail/cpu.cfs_period_us", "100000"),
            ("/cg/mem/mail/memory.limit_in_bytes", "1073741824"),
            ("/cg/mem/mail/memory.usage_in_bytes", "536870912"),
            ("/cg/mem/memory.limit_in_bytes", "9223372036854771712"),
            ("/cg/set/cpuset.cpus", "0-1"),
        ]);
        assert_eq!(result.cpu_millis, 500);
        assert_eq!(result.memory_limit_bytes, Some(1073741824));
        assert_eq!(result.memory_available_bytes, Some(536870912));
    }

    #[test]
    fn namespaced_root_and_mount_escapes_never_escape_the_mount() {
        assert_eq!(
            linux::hierarchy("/docker/id", "/sys/fs/cgroup", "/"),
            Some(vec!["/sys/fs/cgroup".into()])
        );
        assert_eq!(
            linux::hierarchy("/docker/id", "/sys/fs/cgroup", "/other/group"),
            None
        );
        assert_eq!(linux::hierarchy("/", "/sys/fs/cgroup", "/../escape"), None);
        let result = sample(&[
            ("/proc/self/cgroup", "0::/\n"),
            (
                "/proc/self/mountinfo",
                "30 20 0:1 /docker/id /cg/my\\040mount rw - cgroup2 cgroup rw\n",
            ),
            ("/cg/my mount/cpu.max", "200000 100000"),
            ("/cg/my mount/memory.max", "max"),
        ]);
        assert_eq!(result.cpu_millis, 2000);
    }

    #[test]
    fn unknown_or_unreadable_memory_does_not_treat_host_ram_as_container_headroom() {
        let files = [
            (
                "/proc/meminfo",
                "MemTotal: 8388608 kB\nMemAvailable: 4194304 kB",
            ),
            ("/proc/self/cgroup", "0::/tenant"),
        ];
        let result = sample(&files);
        assert_eq!(result.memory_available_bytes, None);
        assert_eq!(result.memory_limit_bytes, None);
        let result = sample(&[]);
        assert_eq!(result.cpu_millis, 64000);
        assert_eq!(result.memory_available_bytes, None);
        let result = sample(&[
            ("/proc/self/cgroup", "0::/"),
            (
                "/proc/self/mountinfo",
                "30 20 0:1 / /cg rw - cgroup2 cgroup rw",
            ),
            ("/cg/memory.max", "4096"),
        ]);
        assert_eq!(result.memory_limit_bytes, Some(4096));
        assert_eq!(result.memory_available_bytes, None);
    }

    #[test]
    fn cpuset_ranges_are_merged_and_malformed_or_unbounded_inputs_are_ignored() {
        assert_eq!(linux::cpus("0-3,2-5,9,9"), Some(7000));
        for text in ["", "2-1", "0-", "one", "0,,2", "-1"] {
            assert_eq!(linux::cpus(text), None);
        }
        assert_eq!(linux::quota("15000", "100000"), Some(150));
        assert_eq!(linux::quota("max", "100000"), None);
        assert_eq!(linux::quota("-1", "100000"), None);
        assert_eq!(linux::quota("1000", "0"), None);
        assert_eq!(linux::quota("1", "1000000"), Some(1));
        assert_eq!(linux::cpus(&vec!["1"; 1025].join(",")), None);
        assert_eq!(
            linux::hierarchy("/", "/cg", &format!("/{}", vec!["deep"; 65].join("/"))),
            None
        );
    }

    #[test]
    fn unlimited_cgroups_retain_host_bounds_and_memory_pressure_saturates_at_zero() {
        let mut files = vec![
            (
                "/proc/meminfo",
                "MemTotal: 1048576 kB\nMemAvailable: 65536 kB\n",
            ),
            ("/proc/self/cgroup", "0::/"),
            (
                "/proc/self/mountinfo",
                "30 20 0:1 / /cg rw - cgroup2 cgroup rw",
            ),
            ("/cg/cpu.max", "max 100000"),
            ("/cg/cpuset.cpus.effective", "0-1"),
            ("/cg/memory.max", "max"),
        ];
        let unlimited = sample(&files);
        assert_eq!(unlimited.cpu_millis, 2000);
        assert_eq!(unlimited.memory_limit_bytes, Some(1073741824));
        assert_eq!(unlimited.memory_available_bytes, Some(67108864));
        files.pop();
        files.push(("/cg/memory.max", "1024"));
        files.push(("/cg/memory.current", "4096"));
        let pressure = sample(&files);
        assert_eq!(pressure.memory_limit_bytes, Some(1024));
        assert_eq!(pressure.memory_available_bytes, Some(0));
    }

    #[test]
    fn sampler_reuses_values_for_five_seconds_then_refreshes() {
        let mut cached = Cached::default();
        let now = Instant::now();
        let first = ImageResources {
            cpu_millis: 1000,
            memory_limit_bytes: None,
            memory_available_bytes: None,
        };
        let next = ImageResources {
            cpu_millis: 2000,
            ..first
        };
        assert_eq!(cached.get(now, || first), first);
        assert_eq!(
            cached.get(now + Duration::from_secs(4), || panic!("cached")),
            first
        );
        assert_eq!(cached.get(now + Duration::from_secs(5), || next), next);
    }
}
