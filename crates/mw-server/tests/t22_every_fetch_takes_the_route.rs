//! **Every hardened fetch in `mw-server` goes through the egress route** (t22-e14).
//!
//! # The failure this exists to prevent
//! 26.20 wires the operator's egress route into the image proxy and into
//! `import_routes.rs`'s `webcal://`/ICS subscription fetch. Both were wired because
//! honouring the route on one surface and not the other gives an operator a control
//! that is **silently bypassed** for the surface that was missed — worse than not
//! shipping the feature, because they believe their egress is controlled.
//!
//! Nothing structural stopped that from happening. `mw_egress::fetch_url_hardened`,
//! `fetch_remote` and friends are still public and still correct — they are simply
//! **unrouted**, and reaching for one is the natural thing to do when adding a third
//! fetch surface. A reviewer would have to know that the routed wrapper exists.
//!
//! So this asks the falsification question directly: *if someone added an unrouted
//! hardened fetch to `mw-server` tomorrow, or reverted the ICS call site, would
//! anything fail?* Without this file, no. With it, this.
//!
//! # Scope, stated rather than implied
//! It scans **`crates/mw-server/src/` only**. Other workspace crates
//! (`mw-crypto`, `mw-autoconfig`, `mw-carddav`, `mw-dav`, `mw-jmap`, `mw-mcp`,
//! `mw-sso`, `mw-passwd`, `mw-assist`) build their own `reqwest` clients through
//! `mw_egress::harden_client` and do not call the fetch entry points at all — so
//! they are a **separate, larger question** (they would each need a store handle to
//! read a route), and pretending to cover them here would be a check that passes
//! because it is looking at the wrong place.
//!
//! Run:
//!   cargo test -p mw-server --test t22_every_fetch_takes_the_route -- --test-threads=1

use std::fs;
use std::path::{Path, PathBuf};

/// The unrouted entry points. Each is public, correct, and **bypasses the configured
/// egress route** — which is exactly why calling one from a server fetch surface is
/// a decision rather than an implementation detail.
const UNROUTED: [&str; 5] = [
    "fetch_url_hardened(",
    "fetch_url_hardened_with(",
    "fetch_remote(",
    "fetch_remote_accepting(",
    "fetch_remote_with(",
];

/// The routed wrappers that must be used instead. Substrings of the unrouted names
/// would otherwise match them (`fetch_url_hardened_routed` contains
/// `fetch_url_hardened`), so they are stripped before the search.
const ROUTED: [&str; 2] = ["fetch_url_hardened_routed(", "fetch_remote_routed("];

fn server_src() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("src")
}

fn rust_files(root: &Path) -> Vec<PathBuf> {
    let mut out = Vec::new();
    let mut stack = vec![root.to_path_buf()];
    while let Some(dir) = stack.pop() {
        let Ok(entries) = fs::read_dir(&dir) else {
            continue;
        };
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                stack.push(path);
            } else if path.extension().is_some_and(|e| e == "rs") {
                out.push(path);
            }
        }
    }
    out.sort();
    out
}

fn strip_comment(line: &str) -> &str {
    match line.find("//") {
        Some(i) => &line[..i],
        None => line,
    }
}

/// Call sites of the unrouted entry points, as `(file, line, text)`.
fn unrouted_call_sites() -> Vec<(String, usize, String)> {
    let root = server_src();
    let mut hits = Vec::new();
    for path in rust_files(&root) {
        let Ok(body) = fs::read_to_string(&path) else {
            continue;
        };
        let rel = path
            .strip_prefix(&root)
            .unwrap_or(&path)
            .to_string_lossy()
            .replace('\\', "/");
        let mut in_tests = false;
        let mut depth = 0i32;
        let mut region_depth = 0i32;
        let mut pending = false;
        for (i, raw) in body.lines().enumerate() {
            let code = strip_comment(raw);

            // Inline `#[cfg(test)] mod … { … }` only — an out-of-line `mod tests;`
            // ends in `;` and opens no region (the bug t22-e-sec caught in its own
            // scanner, reproduced faithfully here rather than re-derived).
            if code.trim() == "#[cfg(test)]" {
                pending = true;
            } else if pending && !code.trim().is_empty() {
                if code.contains('{') {
                    in_tests = true;
                    region_depth = depth;
                }
                pending = false;
            }

            if !in_tests {
                // Remove the routed names first, so `fetch_url_hardened_routed(` is
                // never read as `fetch_url_hardened(`.
                let mut probe = code.to_string();
                for r in ROUTED {
                    probe = probe.replace(r, "«routed»");
                }
                // The re-export/definition sites in `mw-egress` are not in scope, and
                // `image_proxy.rs`'s own wrapper legitimately names the routed
                // function; only genuine unrouted CALLS matter.
                for u in UNROUTED {
                    if probe.contains(u) {
                        hits.push((rel.clone(), i + 1, raw.trim().to_string()));
                        break;
                    }
                }
            }

            depth += code.matches('{').count() as i32;
            depth -= code.matches('}').count() as i32;
            if in_tests && depth <= region_depth {
                in_tests = false;
            }
        }
    }
    hits
}

#[test]
fn the_scanner_can_see_the_names_it_is_looking_for() {
    // Calibration, and the reason anything below can be believed: search for the
    // ROUTED names and prove they are found. A scanner that could not read the tree
    // would report "no unrouted calls" — a clean-looking answer from a check that
    // saw nothing.
    let root = server_src();
    let mut routed_hits = 0usize;
    for path in rust_files(&root) {
        let Ok(body) = fs::read_to_string(&path) else {
            continue;
        };
        for line in body.lines() {
            let code = strip_comment(line);
            if ROUTED.iter().any(|r| code.contains(r)) {
                routed_hits += 1;
            }
        }
    }
    assert!(
        routed_hits >= 3,
        "expected at least the three routed call sites this tag added (the image \
         proxy's fetch, the ICS wrapper's own body, and `import_routes`'s call), \
         found {routed_hits} — the scanner is not reading `mw-server/src/`, so the \
         assertion below would pass vacuously"
    );
}

#[test]
fn no_server_fetch_surface_bypasses_the_configured_egress_route() {
    let hits = unrouted_call_sites();
    assert!(
        hits.is_empty(),
        "These call `mw-egress`'s UNROUTED fetch entry points, so a configured egress \
         route does not apply to them.\n\n\
         An operator who configures a route in /admin/egress expects it to hold for \
         the deployment, not for whichever surfaces happened to be wired. A fetch \
         that quietly goes direct defeats network policy, egress-IP control and \
         reader anonymity at exactly the moment nobody is looking.\n\n\
         Use `image_proxy::fetch_url_hardened_routed` (or `mw_egress::\
         fetch_remote_routed` with `image_proxy::active_route`), which adds the route \
         lookup, the fail-closed dispatch and the audit row.\n\n\
         If a surface genuinely must bypass the route, that is a security decision: \
         allow it here explicitly, with the reason.\n\n\
         found: {hits:#?}"
    );
}

#[test]
fn the_ics_subscription_fetch_is_routed() {
    // Named separately from the sweep above because it is the surface this tag
    // nearly shipped unrouted, and a reader deserves to see it asserted by name
    // rather than inferred from an empty list.
    let body =
        fs::read_to_string(server_src().join("import_routes.rs")).expect("read import_routes.rs");
    assert!(
        body.contains("fetch_url_hardened_routed("),
        "the webcal/ICS fetch must take the configured egress route"
    );
    assert!(
        body.contains("fn fetch_ics(state: &AppState"),
        "and it must receive the state that carries the store the route is read \
         from — the route is deployment configuration, so a store handle is the \
         ONLY thing it needs; a parameter derived from the request would be the \
         steering hazard the whole design excludes"
    );
}
