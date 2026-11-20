//! Revision-bearing build facts belong only to this thin facade/CLI package.
use std::{env, fs, path::Path, process::Command};

fn git(root: &Path, args: &[&str]) -> String {
    Command::new("git")
        .args(args)
        .current_dir(root)
        .output()
        .ok()
        .filter(|output| output.status.success())
        .map(|output| String::from_utf8_lossy(&output.stdout).trim().into())
        .unwrap_or_default()
}

fn write_if_changed(path: &Path, bytes: &[u8]) {
    if fs::read(path).ok().as_deref() != Some(bytes) {
        fs::write(path, bytes).expect("write product build artifact");
    }
}

fn main() {
    let manifest = env::var("CARGO_MANIFEST_DIR").unwrap();
    let root = Path::new(&manifest).parent().unwrap().parent().unwrap();
    let output = env::var("OUT_DIR").unwrap();
    for file in ["Cargo.lock", "apps/web/pnpm-lock.yaml", "SECURITY.md"] {
        println!("cargo:rerun-if-changed={}", root.join(file).display());
    }
    for key in [
        "SOURCE_DATE_EPOCH",
        "MW_PRODUCT_SUPPORT_UNTIL",
        "MW_BUILD_REVISION",
        "MW_BUILD_SHA",
        "MW_BUILD_RELEASE_TAG",
    ] {
        println!("cargo:rerun-if-env-changed={key}");
    }
    for name in ["HEAD", "packed-refs", "refs/tags"] {
        let path = git(root, &["rev-parse", "--git-path", name]);
        if !path.is_empty() {
            println!("cargo:rerun-if-changed={}", root.join(path).display());
        }
    }
    let branch = git(root, &["symbolic-ref", "-q", "HEAD"]);
    if !branch.is_empty() {
        let path = git(root, &["rev-parse", "--git-path", &branch]);
        println!("cargo:rerun-if-changed={}", root.join(path).display());
    }
    let explicit = env::var("MW_BUILD_REVISION").ok();
    let legacy = env::var("MW_BUILD_SHA").ok();
    let revision = mw_build_metadata::resolve_revision(
        explicit.as_deref(),
        legacy.as_deref(),
        &git(root, &["rev-parse", "HEAD"]),
    )
    .expect("immutable product build provenance");
    let git_tag = git(root, &["describe", "--tags", "--exact-match"]);
    let release_tag = env::var("MW_BUILD_RELEASE_TAG")
        .ok()
        .filter(|tag| !tag.is_empty())
        .unwrap_or_else(|| git_tag.clone());
    assert!(
        git_tag.is_empty() || release_tag == git_tag,
        "Explicit release tag and Git tag disagree"
    );
    let lock =
        fs::read(root.join("Cargo.lock")).expect("Cargo.lock required for product inventory");
    let pnpm = fs::read(root.join("apps/web/pnpm-lock.yaml")).ok();
    let policy =
        fs::read(root.join("SECURITY.md")).expect("SECURITY.md required for product inventory");
    let version = env::var("CARGO_PKG_VERSION").unwrap();
    let target = env::var("TARGET").unwrap();
    println!("cargo:rustc-env=MW_PRODUCT_REVISION={revision}");
    println!("cargo:rustc-env=MW_PRODUCT_TARGET={target}");
    let epoch = env::var("SOURCE_DATE_EPOCH").ok();
    let support = env::var("MW_PRODUCT_SUPPORT_UNTIL").ok();
    let built = mw_build_metadata::generate(mw_build_metadata::Inputs {
        version: &version,
        target: &target,
        revision: &revision,
        release_tag: &release_tag,
        source_date_epoch: epoch.as_deref(),
        support_until: support.as_deref(),
        cargo_lock: &lock,
        pnpm_lock: pnpm.as_deref(),
        security_policy: &policy,
    })
    .expect("complete product build inventory");
    let output = Path::new(&output);
    write_if_changed(&output.join("product-security.json"), &built.metadata);
    write_if_changed(&output.join("software-bom.cdx.json"), &built.sbom);
    write_if_changed(&output.join("security-policy.txt"), &built.security_policy);
}
