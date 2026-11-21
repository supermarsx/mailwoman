//! Internal-driver paths must remain usable after build cache mounts disappear.
use std::path::{Path, PathBuf};

pub(super) fn source_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

pub(super) fn internal_http_driver(root: &Path, handoff: &Path, report: &Path) -> tokio::process::Command {
    let mut command = tokio::process::Command::new("node");
    command
        .arg(root.join("scripts/perf/http-real-fixture.mjs"))
        .arg(handoff)
        .arg(report)
        .current_dir(root)
        .kill_on_drop(true);
    command
}

#[tokio::test]
async fn internal_node_driver_uses_runtime_source_root_after_build_mount_release() {
    let directory = tempfile::tempdir().unwrap();
    let handoff = directory.path().join("invalid.private.json");
    let report = directory.path().join("never-created.json");
    // Parsed by the actual driver before its first request; '{}' fails new URL
    // with no target, cookies or credentials, after resolving its entire import
    // closure. This is an IPC/path correctness gate, not a measured campaign.
    std::fs::write(&handoff, b"{}").unwrap();
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(&handoff, std::fs::Permissions::from_mode(0o600)).unwrap();
    }
    let root = source_root();
    let mut command = internal_http_driver(&root, &handoff, &report);
    let output = tokio::time::timeout(std::time::Duration::from_secs(10), command.output())
        .await.expect("bounded internal driver path probe timed out")
        .expect("actual Node driver must start from the compiled source root");
    assert_eq!(output.status.code(), Some(1));
    assert!(output.stdout.is_empty());
    assert_eq!(String::from_utf8(output.stderr).unwrap().trim(), "Invalid URL");
    assert!(!report.exists(), "pre-dispatch rejection cannot create a campaign report");
}
