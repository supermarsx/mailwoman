//! Process memory for benchmark records (t62-n12): resident set split into
//! anonymous / file-backed / shared memory, peak resident set and thread count,
//! from `/proc/self/status` (Linux). `None` elsewhere.

/// `VmRSS`, `RssAnon`, `RssFile`, `RssShmem`, `VmHWM` in bytes and `Threads`.
pub fn process_memory() -> Option<serde_json::Value> {
    #[cfg(target_os = "linux")]
    {
        let status = std::fs::read_to_string("/proc/self/status").ok()?;
        return parse_status(&status);
    }
    #[allow(unreachable_code)]
    None
}

pub fn parse_status(status: &str) -> Option<serde_json::Value> {
    let field = |name: &str| -> Option<u64> {
        let line = status.lines().find(|line| line.starts_with(name))?;
        let mut parts = line[name.len()..].split_whitespace();
        let value = parts.next()?.parse::<u64>().ok()?;
        Some(match parts.next() {
            Some("kB") => value.checked_mul(1024)?,
            _ => value,
        })
    };
    Some(serde_json::json!({
        "residentBytes": field("VmRSS:")?,
        "anonymousBytes": field("RssAnon:"),
        "fileBackedBytes": field("RssFile:"),
        "sharedMemoryBytes": field("RssShmem:"),
        "peakResidentBytes": field("VmHWM:"),
        "threads": field("Threads:"),
        "source": "/proc/self/status",
    }))
}

#[test]
fn status_fields_are_parsed_in_bytes() {
    let sample = "Name:\tx\nVmHWM:\t  2048 kB\nVmRSS:\t  1024 kB\nRssAnon:\t   600 kB\nRssFile:\t   400 kB\nRssShmem:\t    24 kB\nThreads:\t17\n";
    let value = parse_status(sample).unwrap();
    assert_eq!(value["residentBytes"], 1024 * 1024);
    assert_eq!(value["anonymousBytes"], 600 * 1024);
    assert_eq!(value["fileBackedBytes"], 400 * 1024);
    assert_eq!(value["peakResidentBytes"], 2048 * 1024);
    assert_eq!(value["threads"], 17);
    assert!(parse_status("Name:\tx\n").is_none());
    #[cfg(target_os = "linux")]
    assert!(process_memory().unwrap()["residentBytes"].as_u64().unwrap() > 0);
}
