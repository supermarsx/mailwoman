//! This crate must link nothing.
//!
//! It is a dev-dependency of `mw-engine` and `mw-store` — two crates almost everything
//! else in the workspace builds on — so anything it linked would enter the
//! dev-dependency closure of nearly every `cargo test` in the tree, and the project's
//! net-zero-third-party-crates property is checked by diffing `Cargo.lock`, which is
//! easy to read as noise once a lane is already touching it. The implementation needs
//! only `std`; this test fails if that ever quietly stops being true.
//!
//! It reads the manifest as text rather than asking Cargo, so it also fails for a
//! commented-in entry that a `cargo tree` would show only after someone ran it.

#[test]
fn the_manifest_declares_no_dependencies() {
    let manifest = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("Cargo.toml");
    let text = std::fs::read_to_string(&manifest).expect("read mw-test-gate/Cargo.toml");

    let mut section = String::new();
    let mut offenders = Vec::new();
    for (n, raw) in text.lines().enumerate() {
        let line = raw.trim();
        if let Some(name) = line.strip_prefix('[').and_then(|s| s.strip_suffix(']')) {
            section = name.to_string();
            continue;
        }
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        // Any `key = …` under a dependency table is a dependency, whatever its form.
        if section.ends_with("dependencies") {
            offenders.push(format!(
                "{}:{}  [{section}] {line}",
                manifest.display(),
                n + 1
            ));
        }
    }

    assert!(
        offenders.is_empty(),
        "mw-test-gate must declare no dependencies of any kind — it is a dev-dependency \
         of mw-engine and mw-store, so its closure is paid by most of the workspace, and \
         `Cargo.lock` is the only other place this would show. Remove these, or move the \
         code that needs them to the crate that owns it:\n{}",
        offenders.join("\n")
    );
}
