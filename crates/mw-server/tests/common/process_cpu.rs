//! Process CPU time for benchmark records (t60). Whole process, all threads.

/// User + system CPU seconds of the whole process.
pub fn process_cpu_seconds() -> Option<f64> {
    #[cfg(target_os = "linux")]
    {
        let stat = std::fs::read_to_string("/proc/self/stat").ok()?;
        let fields: Vec<&str> = stat.rsplit_once(')')?.1.split_whitespace().collect();
        let ticks = fields.get(11)?.parse::<f64>().ok()? + fields.get(12)?.parse::<f64>().ok()?;
        return Some(ticks / 100.0);
    }
    #[cfg(windows)]
    {
        #[repr(C)]
        #[derive(Default)]
        struct FileTime {
            low: u32,
            high: u32,
        }
        #[link(name = "kernel32")]
        unsafe extern "system" {
            fn GetCurrentProcess() -> *mut core::ffi::c_void;
            fn GetProcessTimes(h: *mut core::ffi::c_void, c: *mut FileTime, e: *mut FileTime, k: *mut FileTime, u: *mut FileTime) -> i32;
        }
        let (mut c, mut e, mut k, mut u) = (FileTime::default(), FileTime::default(), FileTime::default(), FileTime::default());
        // SAFETY: the pseudo-handle needs no closing; the outputs live for the call.
        let ok = unsafe { GetProcessTimes(GetCurrentProcess(), &mut c, &mut e, &mut k, &mut u) } != 0;
        let secs = |t: &FileTime| ((u64::from(t.high) << 32) | u64::from(t.low)) as f64 / 10_000_000.0;
        return ok.then(|| secs(&k) + secs(&u));
    }
    #[allow(unreachable_code)]
    None
}

#[test]
fn process_cpu_time_is_observable_and_monotonic() {
    let Some(first) = process_cpu_seconds() else { return };
    let mut x = 0u64;
    for i in 0..5_000_000u64 { x = x.wrapping_mul(31).wrapping_add(i); }
    assert!(x != 1);
    assert!(process_cpu_seconds().unwrap() >= first);
}
