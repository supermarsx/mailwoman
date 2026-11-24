use super::*;

const GIB: u64 = 1024 * 1024 * 1024;

#[test]
fn no_readings_give_todays_fixed_values() {
    let auto = derive(&Resources::default(), 8, 256);
    assert_eq!(
        (auto.connections, auto.requests, auto.http2_streams),
        (1024, 256, 32)
    );
    assert_eq!(auto.fd_reserve, None);
    assert!(!auto.fd_short);
}

#[test]
fn a_large_host_derives_from_memory_fds_and_cpus() {
    let resources = Resources {
        cpus: Some(16),
        memory_bytes: Some(32 * GIB),
        open_files: Some(1_048_576),
    };
    let auto = derive(&resources, 64, 256);
    // R = 512 + 64 + 256; fd term (1048576 - 832) / 2 = 523872; memory term 32768.
    assert_eq!(auto.fd_reserve, Some(832));
    assert_eq!(auto.connections, 32768);
    // max(64*16, 16*64) = 1024; memory ceiling 32 GiB / 4 MiB = 8192.
    assert_eq!(auto.requests, 1024);
    assert_eq!(auto.http2_streams, 100);
}

#[test]
fn cgroup_limits_lower_than_the_host_win() {
    // A 2-CPU, 1 GiB container on a large host: std reports the quota, and the
    // cgroup memory limit is lower than MemTotal.
    let meminfo = "MemTotal:       263842108 kB\nMemFree: 1 kB\n";
    let memory = memory_from(Some(meminfo), &[Some("1073741824\n".into()), None]);
    assert_eq!(memory, Some(GIB));
    let auto = derive(
        &Resources {
            cpus: Some(2),
            memory_bytes: memory,
            open_files: Some(1_048_576),
        },
        db_pool_estimate(Some("postgres://db/mail"), None, 2),
        256,
    );
    assert_eq!(auto.connections, 1024, "memory term 1024, floor 1024");
    assert_eq!(auto.requests, 256, "floor; memory ceiling 256");
    assert_eq!(auto.http2_streams, 32);
}

#[test]
fn the_floor_is_never_undercut_even_by_tiny_limits() {
    for resources in [
        Resources {
            cpus: Some(1),
            memory_bytes: Some(64 * 1024 * 1024),
            open_files: Some(256),
        },
        Resources {
            cpus: Some(1),
            memory_bytes: None,
            open_files: Some(1024),
        },
    ] {
        let auto = derive(&resources, 2, 256);
        assert_eq!((auto.connections, auto.requests), (1024, 256));
        assert!(auto.fd_short, "{resources:?}");
    }
}

#[test]
fn hard_ceilings_hold_on_huge_readings() {
    let auto = derive(
        &Resources {
            cpus: Some(4096),
            memory_bytes: Some(u64::MAX / 2),
            open_files: Some(u64::MAX),
        },
        2000,
        2048,
    );
    assert_eq!(auto.connections, CONNECTIONS_CEILING);
    assert_eq!(auto.requests, REQUESTS_CEILING);
    assert_eq!(auto.http2_streams, STREAMS_AUTO_CEILING);
    // A reserve that saturates leaves no fd room: the floor, no overflow.
    let saturated = derive(
        &Resources {
            open_files: Some(u64::MAX),
            ..Resources::default()
        },
        u32::MAX,
        u64::MAX,
    );
    assert_eq!(saturated.connections, CONNECTIONS_FLOOR);
}

#[test]
fn the_pool_term_scales_requests() {
    let resources = Resources {
        cpus: Some(4),
        memory_bytes: Some(64 * GIB),
        open_files: None,
    };
    let pool = db_pool_estimate(
        Some("postgresql://x"),
        Some(r#"{"postgresMaxConnections":200}"#),
        4,
    );
    assert_eq!(pool, 200);
    assert_eq!(derive(&resources, pool, 256).requests, 3200);
    // Memory alone (no fd reading) bounds connections.
    assert_eq!(derive(&resources, pool, 256).connections, 65536);
}

#[test]
fn pool_estimate_follows_the_store_defaults() {
    assert_eq!(db_pool_estimate(None, None, 16), 8);
    assert_eq!(db_pool_estimate(Some("mailwoman.db"), None, 1), 2);
    assert_eq!(db_pool_estimate(Some("postgres://h/d"), None, 16), 64);
    assert_eq!(db_pool_estimate(Some("postgres://h/d"), None, 0), 4);
    assert_eq!(
        db_pool_estimate(Some("x.db"), Some(r#"{"sqliteMaxConnections":3}"#), 16),
        3
    );
    assert_eq!(db_pool_estimate(Some("x.db"), Some("not json"), 16), 8);
}

#[test]
fn readings_parse_and_unreadable_values_are_ignored() {
    assert_eq!(parse_cgroup_memory("max\n"), None);
    assert_eq!(parse_cgroup_memory("9223372036854771712"), None);
    assert_eq!(parse_cgroup_memory("garbage"), None);
    assert_eq!(parse_cgroup_memory("0"), None);
    assert_eq!(parse_cgroup_memory("536870912"), Some(512 * 1024 * 1024));
    assert_eq!(parse_meminfo("MemTotal: 2048 kB"), Some(2048 * 1024));
    assert_eq!(parse_meminfo("nothing"), None);
    assert_eq!(memory_from(None, &[None, Some("max".into())]), None);
    let files = cgroup_memory_files("0::/system.slice/mw.service\n");
    assert_eq!(
        files[0],
        "/sys/fs/cgroup/system.slice/mw.service/memory.max"
    );
    let v1 = cgroup_memory_files("4:memory:/docker/abc\n3:cpu,cpuacct:/docker/abc\n");
    assert_eq!(
        v1[0],
        "/sys/fs/cgroup/memory/docker/abc/memory.limit_in_bytes"
    );
}

#[test]
fn streams_are_an_eighth_of_requests_within_bounds() {
    assert_eq!(streams_for(1), 32);
    assert_eq!(streams_for(256), 32);
    assert_eq!(streams_for(400), 50);
    assert_eq!(streams_for(800), 100);
    assert_eq!(streams_for(16384), 100);
}

#[test]
fn probing_the_running_system_does_not_fail() {
    let resources = Resources::probe(false);
    assert!(resources.cpus.is_some_and(|n| n >= 1));
}
