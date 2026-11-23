//! t48: every settings registry key has a reader.
//!
//! The settings console lists every key in `settings_registry::REGISTRY`, so a key
//! that nothing reads is a switch that does nothing. This test fails when a
//! registered key has no reader in production source, and when a key removed by
//! t48 (`REMOVED_KEYS`) is still read.
//!
//! **This is a weak oracle.** It scans source text; it does not run anything.
//! A key counts as read when its full name appears as a quoted string (`"key"`
//! or `'key'`) in a production file under `crates/*/src` or `apps/web/src`,
//! leaving out the registry itself, the knob catalogue (`knobs.rs`), the console
//! label list (`Settings/keys.ts`), test files and inline `#[cfg(test)] mod`
//! blocks. So it catches a key that nothing mentions; it does not catch a key
//! that is mentioned but has no effect (a name in an editor list, a value that
//! is read and then ignored, a reader that only echoes the value back). The
//! behaviour tests of each key are the real oracle; this one only stops a new
//! key from being registered with no code behind it.
//!
//! Keys whose name is built at run time are covered by [`BUILT`]: each entry names
//! the keys it covers and a string that must still appear in production source
//! (the `format!` or template literal that builds the name). If that string goes
//! away, the entry stops covering anything and its keys fail here.
//!
//! [`PENDING`] lists keys that other t48 executors are wiring, or that are
//! waiting for a decision; each entry says who and why. A pending key that has
//! gained a reader is reported on stderr so the entry can be dropped.

use std::path::{Path, PathBuf};

use mw_server::settings_registry::{REGISTRY, REMOVED_KEYS};

/// Which keys a [`Built`] entry covers.
enum Keys {
    /// Every key that starts with this.
    Prefix(&'static str),
    /// Every key that starts with the first and ends with the second.
    Shape(&'static str, &'static str),
    /// Exactly these.
    Exact(&'static [&'static str]),
}

/// Keys whose name a reader builds at run time.
struct Built {
    keys: Keys,
    /// Text that must appear in production source for the entry to count.
    evidence: &'static str,
}

const BUILT: &[Built] = &[
    Built { keys: Keys::Prefix("web.assets.cache."), evidence: "\"web.assets.cache.\"" },
    Built { keys: Keys::Prefix("security.protection."), evidence: "\"security.protection.\"" },
    Built { keys: Keys::Prefix("security.protocol."), evidence: "\"security.protocol.\"" },
    Built { keys: Keys::Prefix("security.trust."), evidence: "\"security.trust.\"" },
    Built { keys: Keys::Prefix("security.adminProbe."), evidence: "\"security.adminProbe.\"" },
    Built { keys: Keys::Prefix("mail.cache."), evidence: "`mail.cache.${" },
    Built { keys: Keys::Prefix("mail.folderRole."), evidence: "\"mail.folderRole.{role}\"" },
    Built { keys: Keys::Prefix("views."), evidence: "`views.${" },
    Built { keys: Keys::Prefix("reader.actions."), evidence: "`reader.actions.${" },
    Built { keys: Keys::Prefix("shortcuts."), evidence: "`shortcuts.${" },
    Built { keys: Keys::Prefix("ui.bar."), evidence: "`ui.bar.${" },
    Built {
        keys: Keys::Exact(&[
            "mail.allowReply",
            "mail.allowForward",
            "mail.allowPrint",
            "mail.showSpam",
            "mail.showAuthStatus",
            "mail.closeOnDelete",
            "mail.showGoToTop",
            "mail.showCreateFolderButton",
        ]),
        evidence: "`mail.${feature}`",
    },
    Built { keys: Keys::Prefix("mail.badge."), evidence: "`mail.badge.${id}`" },
    Built {
        keys: Keys::Exact(&["time.region", "time.dateFormat", "time.timeFormat"]),
        evidence: "`time.${key}`",
    },
    Built { keys: Keys::Prefix("security.health."), evidence: "\"security.health.\"" },
    Built { keys: Keys::Prefix("restriction.scope."), evidence: "\"restriction.scope.{s}\"" },
    Built { keys: Keys::Prefix("dsr.type."), evidence: "\"dsr.type.{kind}\"" },
    Built {
        keys: Keys::Prefix("dsr.export.include."),
        evidence: "\"dsr.export.include.{other}\"",
    },
    Built { keys: Keys::Shape("consent.", ".basis"), evidence: "\"consent.{}.basis\"" },
    Built { keys: Keys::Shape("consent.", ".version"), evidence: "\"consent.{}.version\"" },
    Built { keys: Keys::Shape("consent.", ".text"), evidence: "\"consent.{}.text\"" },
];

/// Keys with no reader yet, each with the reason. Keep this list short: an entry
/// here is a switch in the console that does nothing today.
const PENDING: &[(&str, &str)] = &[
    // t48-e2 (password policy, assistant, plugin storage).
    ("passwd.policy.minLength", "t48-e2 wires passwd.policy.*"),
    ("passwd.policy.requireUpper", "t48-e2 wires passwd.policy.*"),
    ("passwd.policy.requireLower", "t48-e2 wires passwd.policy.*"),
    ("passwd.policy.requireDigit", "t48-e2 wires passwd.policy.*"),
    ("passwd.policy.requireSymbol", "t48-e2 wires passwd.policy.*"),
    ("plugins.kv.maxKeyBytes", "t48-e2 reads it through the registry instead of the environment"),
    ("plugins.kv.maxValueBytes", "t48-e2 reads it through the registry instead of the environment"),
    ("plugins.kv.maxTotalBytes", "t48-e2 reads it through the registry instead of the environment"),
    ("plugins.kv.maxKeys", "t48-e2 reads it through the registry instead of the environment"),
    // t48-e7 (lib.rs, webhooks, masked addresses, security.txt).
    ("webhooks.maxAttempts", "t48-e7 reads it at delivery time"),
    ("webhooks.timeoutSecs", "t48-e7 reads it at delivery time"),
    ("upstream.proxyTimeoutSecs", "t48-e7 reads it when the upstream client is built"),
    ("signin.ldapBindPrecheck", "t48-e7 reads it through the registry instead of the environment"),
    ("maskedEmail.domain", "t48-e7 reads it through the registry instead of the environment"),
    ("abuse.reportAddress", "t48-e7 reads it through the registry instead of the environment"),
    ("about.securityContact", "t48-e7 serves /.well-known/security.txt"),
    ("about.securityPolicyUrl", "t48-e7 serves /.well-known/security.txt"),
    // t48-e8 (engine, health).
    ("outbox.maxAttempts", "t48-e8 reads it in the dispatcher"),
    ("dlp.adminAddress", "t48-e8 reads it through the registry instead of the environment"),
    ("health.diskFreeMinPercent", "t48-e8 adds the /readyz disk floor"),
    ("about.projectLinks", "t48-e1 wave 3: removed with the knobs.rs test that names it (no About screen reads it)"),
    // Read only from the environment today; t48-e1 removes them from the registry
    // together with their knob rows (knobs.rs, wave 3) so the variables stay
    // catalogued.
    ("privacy.watermark.enabled", "t48-e1 wave 3: removed with the lib.rs env read"),
    ("privacy.watermark.opacity", "t48-e1 wave 3: removed with the lib.rs env read"),
    ("observability.errorForwardUrl", "t48-e1 wave 3: stays environment-only (outbound target)"),
    ("egress.probeUrl", "t48-e1 wave 3: stays environment-only (outbound target)"),
    // Deferred: its only reader is push.rs, which t48 does not edit.
    ("realtime.recheckSecs", "deferred until the push.rs owner reads it through the registry"),
];

fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

/// Whether a file is production source for this scan.
fn production(path: &Path) -> bool {
    let name = path.file_name().and_then(|n| n.to_str()).unwrap_or_default();
    let rust = name.ends_with(".rs");
    let ts = name.ends_with(".ts") || name.ends_with(".tsx");
    if !(rust || ts) {
        return false;
    }
    if matches!(name, "settings_registry.rs" | "knobs.rs" | "keys.ts" | "tests.rs") {
        return false;
    }
    if name.ends_with("_tests.rs") || name.ends_with(".d.ts") {
        return false;
    }
    if [".test.ts", ".test.tsx", ".spec.ts", ".spec.tsx"].iter().any(|s| name.ends_with(s)) {
        return false;
    }
    true
}

/// `source` without its inline `#[cfg(test)] mod … { … }` blocks.
fn without_test_modules(source: &str) -> String {
    let mut out = String::with_capacity(source.len());
    let mut rest = source;
    while let Some(at) = rest.find("#[cfg(test)]") {
        out.push_str(&rest[..at]);
        let after = &rest[at + "#[cfg(test)]".len()..];
        let trimmed = after.trim_start();
        let item = trimmed
            .strip_prefix("pub(crate) ")
            .or_else(|| trimmed.strip_prefix("pub "))
            .unwrap_or(trimmed);
        let inline_mod = item.starts_with("mod ")
            && item.find(['{', ';']).is_some_and(|i| item.as_bytes()[i] == b'{');
        if !inline_mod {
            out.push_str("#[cfg(test)]");
            rest = after;
            continue;
        }
        let open = after.len() - item.len() + item.find('{').unwrap();
        let mut depth = 0usize;
        let mut end = after.len();
        for (i, b) in after.bytes().enumerate().skip(open) {
            match b {
                b'{' => depth += 1,
                b'}' => {
                    depth -= 1;
                    if depth == 0 {
                        end = i + 1;
                        break;
                    }
                }
                _ => {}
            }
        }
        rest = &after[end..];
    }
    out.push_str(rest);
    out
}

fn collect(dir: &Path, skip_dirs: &[&str], out: &mut String) {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return;
    };
    for entry in entries {
        let path = entry.unwrap().path();
        let name = path.file_name().and_then(|n| n.to_str()).unwrap_or_default().to_string();
        if path.is_dir() {
            if !skip_dirs.contains(&name.as_str()) {
                collect(&path, skip_dirs, out);
            }
        } else if production(&path) {
            let source = std::fs::read_to_string(&path).unwrap();
            if name.ends_with(".rs") {
                out.push_str(&without_test_modules(&source));
            } else {
                out.push_str(&source);
            }
            out.push('\n');
        }
    }
}

/// All production source the scan reads, as one string.
fn production_source() -> String {
    let root = root();
    let mut out = String::new();
    for entry in std::fs::read_dir(root.join("crates")).unwrap() {
        let src = entry.unwrap().path().join("src");
        if src.is_dir() {
            collect(&src, &["tests", "testdata", "fixtures"], &mut out);
        }
    }
    collect(
        &root.join("apps/web/src"),
        &["__tests__", "test", "tests", "node_modules"],
        &mut out,
    );
    assert!(out.len() > 1_000_000, "the scan found the source tree ({} bytes)", out.len());
    out
}

fn quoted(source: &str, key: &str) -> bool {
    source.contains(&format!("\"{key}\"")) || source.contains(&format!("'{key}'"))
}

fn built_reader(source: &str, key: &str) -> Option<&'static str> {
    BUILT.iter().find_map(|b| {
        let covers = match b.keys {
            Keys::Prefix(p) => key.starts_with(p),
            Keys::Shape(p, s) => key.starts_with(p) && key.ends_with(s),
            Keys::Exact(list) => list.contains(&key),
        };
        (covers && source.contains(b.evidence)).then_some(b.evidence)
    })
}

#[test]
fn every_registered_key_has_a_reader() {
    let source = production_source();
    let mut unread = Vec::new();
    let mut now_read = Vec::new();
    for def in REGISTRY.iter().filter(|d| !d.is_family()) {
        let read = quoted(&source, def.key) || built_reader(&source, def.key).is_some();
        let pending = PENDING.iter().any(|(k, _)| *k == def.key);
        match (read, pending) {
            (false, false) => unread.push(def.key),
            (true, true) => now_read.push(def.key),
            _ => {}
        }
    }
    if !now_read.is_empty() {
        eprintln!("pending keys that now have a reader (drop them from PENDING): {now_read:?}");
    }
    assert!(
        unread.is_empty(),
        "registered keys with no reader in production source; wire them, remove them \
         (and add them to REMOVED_KEYS), or list them in PENDING with the reason: {unread:#?}"
    );
}

#[test]
fn every_built_entry_still_has_its_evidence() {
    let source = production_source();
    let stale: Vec<_> = BUILT.iter().filter(|b| !source.contains(b.evidence)).map(|b| b.evidence).collect();
    assert!(stale.is_empty(), "BUILT entries whose builder is gone: {stale:?}");
}

#[test]
fn pending_keys_are_still_registered() {
    let gone: Vec<_> = PENDING
        .iter()
        .filter(|(k, _)| !REGISTRY.iter().any(|d| d.key == *k))
        .map(|(k, _)| *k)
        .collect();
    assert!(gone.is_empty(), "PENDING names keys that are not registered: {gone:?}");
}

/// Removed setting keys whose name is also used for something else.
const NAMESAKES: &[(&str, &str)] = &[
    ("updates.check", "an admin capability (capabilities.rs, can.ts)"),
    ("updates.apply", "an admin capability (capabilities.rs, can.ts)"),
];

#[test]
fn removed_keys_are_not_registered_or_read() {
    let source = production_source();
    let mut problems = Vec::new();
    for key in REMOVED_KEYS {
        if mw_server::settings_registry::lookup(key).is_some() {
            problems.push(format!("{key}: still registered"));
        }
        if quoted(&source, key) && !NAMESAKES.iter().any(|(k, _)| k == key) {
            problems.push(format!("{key}: still read by name"));
        }
    }
    assert!(problems.is_empty(), "{problems:#?}");
}

#[test]
fn test_modules_are_left_out_of_the_scan() {
    let source = "fn a() { \"x.read\"; }\n#[cfg(test)]\nmod tests {\n    fn b() { let _ = \"x.test\"; { } }\n}\nfn c() {}\n#[cfg(test)]\nuse y;\n";
    let kept = without_test_modules(source);
    assert!(kept.contains("\"x.read\""));
    assert!(!kept.contains("x.test"));
    assert!(kept.contains("fn c()"));
    assert!(kept.contains("use y;"));
    // The oracle sees a key named in production code and misses one that is not.
    assert!(quoted(&kept, "x.read"));
    assert!(!quoted(&kept, "x.test"));
}
