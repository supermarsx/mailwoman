//! Process CPU time, resident memory and processor facts.
use serde_json::{Value, json};

/// (user, system) CPU microseconds of this process.
#[cfg(unix)]
pub fn cpu_micros() -> (u64, u64) {
    let mut usage: libc::rusage = unsafe { std::mem::zeroed() };
    unsafe { libc::getrusage(libc::RUSAGE_SELF, &mut usage) };
    let us = |t: libc::timeval| t.tv_sec as u64 * 1_000_000 + t.tv_usec as u64;
    (us(usage.ru_utime), us(usage.ru_stime))
}
#[cfg(unix)]
pub fn rss_bytes() -> u64 {
    std::fs::read_to_string("/proc/self/statm")
        .ok()
        .and_then(|s| {
            s.split_whitespace()
                .nth(1)
                .and_then(|v| v.parse::<u64>().ok())
        })
        .map_or(0, |pages| pages * 4096)
}

#[cfg(windows)]
pub fn cpu_micros() -> (u64, u64) {
    use windows_sys::Win32::Foundation::FILETIME;
    use windows_sys::Win32::System::Threading::{GetCurrentProcess, GetProcessTimes};
    let zero = FILETIME {
        dwLowDateTime: 0,
        dwHighDateTime: 0,
    };
    let (mut c, mut e, mut k, mut u) = (zero, zero, zero, zero);
    unsafe { GetProcessTimes(GetCurrentProcess(), &mut c, &mut e, &mut k, &mut u) };
    let t = |f: FILETIME| ((f.dwHighDateTime as u64) << 32 | f.dwLowDateTime as u64) / 10;
    (t(u), t(k))
}
#[cfg(windows)]
pub fn rss_bytes() -> u64 {
    use windows_sys::Win32::System::ProcessStatus::{
        GetProcessMemoryInfo, PROCESS_MEMORY_COUNTERS,
    };
    use windows_sys::Win32::System::Threading::GetCurrentProcess;
    let mut counters: PROCESS_MEMORY_COUNTERS = unsafe { std::mem::zeroed() };
    counters.cb = std::mem::size_of::<PROCESS_MEMORY_COUNTERS>() as u32;
    unsafe { GetProcessMemoryInfo(GetCurrentProcess(), &mut counters, counters.cb) };
    counters.WorkingSetSize as u64
}

/// What this process can use: `available_parallelism` and, on Windows, the processor groups.
pub fn processors() -> Value {
    let available = std::thread::available_parallelism().map_or(1, |n| n.get());
    #[cfg(windows)]
    {
        use windows_sys::Win32::System::Threading::{
            GetActiveProcessorCount, GetActiveProcessorGroupCount, GetCurrentProcess,
            GetProcessGroupAffinity,
        };
        let groups = unsafe { GetActiveProcessorGroupCount() } as u32;
        let all = unsafe { GetActiveProcessorCount(0xffff) };
        let mut count: u16 = 8;
        let mut array = [0u16; 8];
        let ok =
            unsafe { GetProcessGroupAffinity(GetCurrentProcess(), &mut count, array.as_mut_ptr()) }
                != 0;
        let process_groups: Vec<u16> = if ok {
            array[..count as usize].to_vec()
        } else {
            Vec::new()
        };
        json!({"availableParallelism": available, "platform": "windows", "processorGroups": groups, "logicalProcessorsAllGroups": all, "processGroupAffinity": process_groups,
            "note": "available_parallelism counts the process's primary group only; on Windows 11 / Server 2022 threads are scheduled across all groups unless an affinity restricts them, so --threads may exceed it"})
    }
    #[cfg(not(windows))]
    {
        json!({"availableParallelism": available, "platform": std::env::consts::OS})
    }
}
