//! **A `ProxyRoute` may only be built from deployment-wide operator config** —
//! enforced structurally, by counting the places one can come into existence
//! (26.20, `t22-e-sec` item 1, landed as a check rather than as a finding).
//!
//! # Why this is a test and not a paragraph
//!
//! `ProxyRoute::host` is **deliberately exempt** from the SSRF address policy:
//! an egress proxy on RFC1918 is the normal deployment, so `tunnel_fetch_hop`
//! checks the *origin's* address and never the proxy's. That carve-out is safe
//! only while a route is operator configuration that nothing request-derived can
//! select. `Store::active_egress_proxy(&self)` defends one end by taking no
//! request-shaped parameter — a signature that cannot be steered.
//!
//! Nothing defended the other end. `ProxyRoute`'s fields are all `pub`, so the
//! only thing between a request-derived host and an unchecked dial was that
//! **nobody constructs one that way** — true, checkable, and one commit from
//! being false. A finding says there is one constructor today; this fails when a
//! second appears.
//!
//! It joins the family this tree already trusts: `t22-e8`'s `.no_proxy()`
//! scanner that catches the tenth client somebody adds next month, and
//! `t22-e12`'s migration tombstone. Same shape, same reason.
//!
//! # If this test fails
//!
//! It is **not** necessarily a bug — it is a claim that the audit's premise
//! changed. Either the new site is fed by `Store::active_egress_proxy` (update
//! the expectation *and say so in the commit*), or it is fed by something
//! request-shaped, in which case the address carve-out no longer holds and the
//! egress design needs re-reviewing before the tag.
//!
//! # The scanner checks itself first
//!
//! Three of the assertions below run against **embedded fixtures**, not the
//! tree, and each pins a way the scanner could report a false all-clear. Two of
//! them are bugs this scanner actually had, found by calibrating it against a
//! tree whose answer was already known:
//!
//! * `#[cfg(test)]` + `mod tests;` is an **out-of-line declaration** and says
//!   nothing about the code after it. Treating it as opening a test region made
//!   an entire file read as test code — and would have answered "0 production
//!   sites" no matter what the tree contained.
//! * `pub struct ProxyRoute {` and `impl ProxyRoute {` are **definitions**, not
//!   constructions. `impl From<_> for ProxyRoute` really does build one and must
//!   still count.
//!
//! Without a control proving the scanner can *see* a production site, "0 found"
//! is indistinguishable from "the scanner stopped matching".

use std::fs;
use std::path::{Path, PathBuf};

/// One site where a `ProxyRoute` can come into existence.
#[derive(Debug, PartialEq, Eq)]
struct Site {
    where_: String,
    line: String,
    kind: Kind,
}

/// **One site per line, at most.** The classifier reads line by line, so a
/// one-line `fn f() -> ProxyRoute { ProxyRoute { .. } }` is counted once, as a
/// `Factory`. That is a real limitation and is stated rather than hidden: it
/// cannot cause a false all-clear (the line is still *seen*, and a public one
/// still trips the visibility assertion), but it does mean the counts are of
/// lines, not of syntax nodes. `rustfmt` splits the real form across lines, and
/// the production site does have its factory and literal on separate lines.
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
enum Kind {
    /// `ProxyRoute { .. }` — the only way to build one, absent a conversion.
    Literal,
    /// A function whose return type is a `ProxyRoute`. Not itself a
    /// construction; tracked because a **public** one re-opens construction to
    /// any caller.
    Factory,
    /// `impl From<_> for ProxyRoute`, `FromStr`, `TryFrom`, `Default` — a
    /// conversion is a construction, and a `Deserialize` would let a request
    /// body become a route outright.
    Conversion,
}

#[derive(Debug, Default)]
struct Scan {
    production: Vec<Site>,
    test: Vec<Site>,
}

impl Scan {
    fn total(&self) -> usize {
        self.production.len() + self.test.len()
    }

    fn production_of(&self, kind: Kind) -> Vec<&Site> {
        self.production.iter().filter(|s| s.kind == kind).collect()
    }
}

/// Lines from which the rest of the file is test code.
///
/// **Only an INLINE `#[cfg(test)] mod x {` opens a region.** An out-of-line
/// `mod tests;` declaration says nothing about what follows it — see the module
/// docs; this was a real bug in this scanner.
fn test_region_starts(lines: &[&str]) -> Vec<usize> {
    let mut out = Vec::new();
    for (i, l) in lines.iter().enumerate() {
        if !l.contains("#[cfg(test)]") {
            continue;
        }
        let next = lines.get(i + 1).copied().unwrap_or("");
        let t = next.trim_start();
        if t.contains(" mod ") || t.starts_with("mod ") {
            // `mod tests {` opens a region; `mod tests;` does not.
            if t.contains('{') && !t.trim_end().ends_with(';') {
                out.push(i + 1);
            }
        }
    }
    out
}

fn classify(where_: &str, src: &str, is_test_file: bool, scan: &mut Scan) {
    let lines: Vec<&str> = src.lines().collect();
    let regions = test_region_starts(&lines);

    for (idx, raw) in lines.iter().enumerate() {
        let lineno = idx + 1;
        let l = raw.trim_start();
        // Comments mentioning the type are not construction sites.
        if l.starts_with("//") || l.starts_with('*') || l.starts_with("#[") {
            continue;
        }
        // Definitions are not constructions.
        if l.starts_with("pub struct ProxyRoute")
            || l.starts_with("struct ProxyRoute")
            || l.starts_with("pub(crate) struct ProxyRoute")
            || l.starts_with("impl ProxyRoute")
        {
            continue;
        }

        let kind = if l.contains(" for ProxyRoute")
            && (l.contains("impl From<")
                || l.contains("impl TryFrom<")
                || l.contains("impl FromStr")
                || l.contains("impl Default")
                || l.contains("Deserialize"))
        {
            Some(Kind::Conversion)
        } else if raw.contains("-> ProxyRoute")
            || raw.contains("-> Option<ProxyRoute>")
            || raw.contains("-> Result<ProxyRoute")
        {
            Some(Kind::Factory)
        } else if raw.contains("ProxyRoute {") {
            Some(Kind::Literal)
        } else {
            None
        };

        let Some(kind) = kind else { continue };
        let site = Site {
            where_: format!("{where_}:{lineno}"),
            line: l.to_string(),
            kind,
        };
        if is_test_file || regions.iter().any(|r| *r < lineno) {
            scan.test.push(site);
        } else {
            scan.production.push(site);
        }
    }
}

fn is_test_path(p: &Path) -> bool {
    p.components().any(|c| c.as_os_str() == "tests")
        || p.file_name().is_some_and(|n| n == "tests.rs")
}

fn walk(dir: &Path, out: &mut Vec<PathBuf>) {
    let Ok(entries) = fs::read_dir(dir) else {
        return;
    };
    for e in entries.flatten() {
        let p = e.path();
        if p.is_dir() {
            if p.file_name().is_some_and(|n| n == "target") {
                continue;
            }
            walk(&p, out);
        } else if p.extension().is_some_and(|x| x == "rs") {
            out.push(p);
        }
    }
}

fn scan_tree() -> Scan {
    let crates = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("crates/mw-egress has a parent")
        .to_path_buf();
    let mut files = Vec::new();
    walk(&crates, &mut files);
    assert!(
        files.len() > 50,
        "the walker found only {} files under {}; it is not scanning the tree",
        files.len(),
        crates.display()
    );

    let mut scan = Scan::default();
    for f in files {
        let Ok(src) = fs::read_to_string(&f) else {
            continue;
        };
        if !src.contains("ProxyRoute") {
            continue;
        }
        let rel = f
            .strip_prefix(&crates)
            .unwrap_or(&f)
            .to_string_lossy()
            .replace('\\', "/");
        classify(&rel, &src, is_test_path(&f), &mut scan);
    }
    scan
}

// ── the scanner's own controls ───────────────────────────────────────────────

/// **Positive control.** Without this, "0 production sites" is indistinguishable
/// from "the scanner stopped matching".
#[test]
fn the_scanner_can_see_a_production_construction_site() {
    const FIXTURE: &str = r#"
fn to_route(row: &EgressProxyRow) -> ProxyRoute {
    ProxyRoute { id: row.id.clone(), host: row.host.clone() }
}
pub fn route_from_request(headers: &HeaderMap) -> ProxyRoute {
    ProxyRoute { id: "x".into(), host: headers["x-proxy"].to_string() }
}
#[cfg(test)]
mod tests {
    fn t() -> ProxyRoute {
        ProxyRoute { id: "t".into(), host: "h".into() }
    }
}
"#;
    let mut scan = Scan::default();
    classify("fixture.rs", FIXTURE, false, &mut scan);

    assert_eq!(
        scan.production_of(Kind::Literal).len(),
        2,
        "both production literals must be seen: {:#?}",
        scan.production
    );
    assert_eq!(
        scan.production_of(Kind::Factory).len(),
        2,
        "both production factories must be seen: {:#?}",
        scan.production
    );
    assert_eq!(
        scan.test.len(),
        2,
        "the inline #[cfg(test)] mod's factory and literal are test sites: {:#?}",
        scan.test
    );
    // The output must carry enough to judge WHERE a route came from, or a
    // failure sends the reader back to the source to answer the only question
    // that matters.
    assert!(
        scan.production
            .iter()
            .any(|s| s.line.contains("headers[\"x-proxy\"]")),
        "the report must show the request-shaped source inline: {:#?}",
        scan.production
    );
}

/// **Control for a bug this scanner had.** `#[cfg(test)]` + `mod tests;` is an
/// out-of-line *declaration*; the code after it is production. Treating it as a
/// test region made a whole file read as test code and would have answered
/// "0 production sites" regardless of the tree.
#[test]
fn an_out_of_line_test_module_declaration_does_not_hide_the_rest_of_the_file() {
    const FIXTURE: &str = r#"
#[cfg(test)]
mod tests;

fn live_one(row: &EgressProxyRow) -> ProxyRoute {
    ProxyRoute { id: row.id.clone(), host: row.host.clone() }
}
"#;
    let mut scan = Scan::default();
    classify("fixture.rs", FIXTURE, false, &mut scan);
    assert_eq!(
        scan.production_of(Kind::Literal).len(),
        1,
        "a `mod tests;` declaration must not make the rest of the file invisible: {scan:#?}"
    );
    assert!(scan.test.is_empty(), "{scan:#?}");
}

/// **Control for the other bug.** Definitions are not constructions; a
/// conversion is.
#[test]
fn definitions_do_not_count_but_conversions_do() {
    const FIXTURE: &str = r#"
pub struct ProxyRoute {
    pub host: String,
}
impl ProxyRoute {
    pub fn endpoint(&self) -> String { String::new() }
}
impl From<EgressProxyRow> for ProxyRoute {
    fn from(row: EgressProxyRow) -> Self { todo!() }
}
"#;
    let mut scan = Scan::default();
    classify("fixture.rs", FIXTURE, false, &mut scan);
    assert_eq!(
        scan.production_of(Kind::Conversion).len(),
        1,
        "`impl From<_> for ProxyRoute` builds a route and must count: {scan:#?}"
    );
    assert_eq!(
        scan.production_of(Kind::Literal).len(),
        0,
        "`pub struct ProxyRoute {{` and `impl ProxyRoute {{` are definitions: {scan:#?}"
    );
}

// ── the property ─────────────────────────────────────────────────────────────

/// **Exactly one production construction site, and it is fed by operator
/// config.**
///
/// The count is of **literals**: absent a conversion, `ProxyRoute { .. }` is the
/// only way to build one. A factory is reported separately and constrained
/// below — `image_proxy.rs` has one factory *and* one literal, which is one
/// site, not two, and callers of that factory may multiply freely without
/// weakening anything.
#[test]
fn exactly_one_production_site_builds_a_proxy_route() {
    let scan = scan_tree();

    // The floor first: a scan that found nothing must not pass as a scan that
    // found nothing wrong (t22-e8's `sites >= 30` precedent).
    assert!(
        scan.total() >= 8,
        "the scanner found only {} sites in the whole tree — it has stopped \
         matching, and every assertion below is vacuous: {scan:#?}",
        scan.total()
    );

    let literals = scan.production_of(Kind::Literal);
    assert_eq!(
        literals.len(),
        1,
        "exactly one production site may build a ProxyRoute, because its `host` \
         is exempt from the SSRF address policy. Found {}: {:#?}\n\nIf the new \
         one is fed by `Store::active_egress_proxy(&self)`, update this \
         expectation and say so in the commit. If it is fed by anything \
         request-shaped, the carve-out no longer holds.",
        literals.len(),
        literals
    );
    assert!(
        literals[0].where_.contains("mw-server/src/image_proxy.rs"),
        "the one production site moved: {:#?}",
        literals[0]
    );

    // No conversion may exist: a `Deserialize`/`From`/`FromStr` would let a
    // value from anywhere become a route without passing the factory.
    assert!(
        scan.production_of(Kind::Conversion).is_empty(),
        "a conversion into ProxyRoute bypasses the one factory: {:#?}",
        scan.production_of(Kind::Conversion)
    );

    // And construction must not be re-opened to arbitrary callers. `pub(crate)`
    // is included deliberately: within `mw-server` that is every module.
    for f in scan.production_of(Kind::Factory) {
        assert!(
            !f.line.starts_with("pub "),
            "a public factory re-opens construction to any caller — the point of \
             one private constructor is that a route can only be assembled where \
             its input is known to be operator config: {f:#?}"
        );
    }
}
