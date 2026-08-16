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

/// Whether a signature's **return type** mentions `ProxyRoute`.
///
/// Position matters: `fn f(route: &ProxyRoute) -> ProxyHop` consumes one and
/// builds none, so "the line mentions ProxyRoute" is the wrong question. The
/// text after the last `->` is the return type on a signature line.
fn returns_a_route(line: &str) -> bool {
    line.rsplit("->")
        .next()
        .is_some_and(|ret| ret.contains("ProxyRoute"))
        && line.contains("->")
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
        } else if l.contains("fn ") && returns_a_route(raw) {
            // Any RETURN TYPE mentioning `ProxyRoute`, however nested — and the
            // return type only.
            //
            // Two bugs live here, both found by running the both-ways control
            // rather than by reading:
            //
            // 1. This was an enumeration of shapes (`-> ProxyRoute`,
            //    `-> Option<ProxyRoute>`, `-> Result<ProxyRoute`) and it missed
            //    the real one: `active_route` returns
            //    `Result<Option<ProxyRoute>, ()>`, which matches none of them. The
            //    scanner did not see it as a factory at all, so the visibility
            //    rule below never ran on it — a `pub fn f() -> Result<Option<
            //    ProxyRoute>, E>` would have walked straight through.
            // 2. Broadening it to "the line mentions ProxyRoute" then caught
            //    `tunnel_fetch_hop(target, route: &ProxyRoute, ..) -> ProxyHop`,
            //    which **consumes** a route and builds none.
            //
            // Enumerating shapes is one bug and ignoring position is the other;
            // the property is "the value coming *out* is a route".
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

    // Both known production factories must be SEEN, by name. This is the floor
    // for the visibility rule below, and it is not redundant with the total: the
    // factory detector has already narrowed silently once — it enumerated return
    // shapes and so never saw `active_route`'s `Result<Option<ProxyRoute>, ()>`,
    // which meant the visibility rule ran on one factory instead of two while
    // every assertion still passed.
    let factories = scan.production_of(Kind::Factory);
    for name in ["fn proxy_route", "fn active_route"] {
        assert!(
            factories.iter().any(|f| f.line.contains(name)),
            "`{name}` is a production factory and must be seen as one, or the \
             visibility rule below silently skips it: {factories:#?}"
        );
    }

    // No conversion may exist: a `Deserialize`/`From`/`FromStr` would let a
    // value from anywhere become a route without passing the factory.
    assert!(
        scan.production_of(Kind::Conversion).is_empty(),
        "a conversion into ProxyRoute bypasses the one factory: {:#?}",
        scan.production_of(Kind::Conversion)
    );

    // And a factory visible beyond its module must be **unsteerable by its
    // signature**.
    //
    // "No public factory" is the wrong rule, and stating the right one matters
    // because the wrong one fails on a legitimate change. What makes a factory
    // safe to share is not its visibility but **what it accepts**:
    //
    //   * `fn active_route(store: &Store) -> Option<ProxyRoute>` takes a handle
    //     to the database and nothing else. A caller cannot use it to name a
    //     host, so sharing it shares the *configured* route — which is the whole
    //     point of that function existing, and `import_routes.rs` needs exactly
    //     that.
    //   * `fn proxy_route(row: &EgressProxyRow) -> Option<ProxyRoute>` takes a
    //     plain struct any module can build. Sharing THAT hands every caller the
    //     ability to turn a value it invented into a route whose `host` is
    //     exempt from the address policy.
    //
    // So: a factory may be `pub`/`pub(crate)` only if **every** parameter is a
    // store handle — or if it is on [`SHARED_FACTORY_EXCEPTIONS`], each entry of
    // which carries its own reason.
    //
    // "Contains a store handle" is not enough, and the difference is the whole
    // point: `f(store: &Store, host: &str)` contains one and can still introduce
    // a host. The invariant is not "no request-derived parameter anywhere" — it
    // is **nothing request-derived reaches `ProxyRoute::host`**.
    for f in scan.production_of(Kind::Factory) {
        if !f.line.starts_with("pub ") {
            continue; // private: its module owns the provenance of its input.
        }
        if SHARED_FACTORY_EXCEPTIONS
            .iter()
            .any(|(name, _)| f.line.contains(name))
        {
            continue;
        }
        assert!(
            store_only(&f.line),
            "a factory visible outside its module must take ONLY a store handle, \
             so no caller can choose what it builds — or be listed in \
             SHARED_FACTORY_EXCEPTIONS with a reason. `ProxyRoute::host` is exempt \
             from the SSRF address policy, so a shared factory that accepts \
             anything a caller can author hands every module a way past it. \
             Parameters read as {:?}: {f:#?}",
            params_of(&f.line)
        );
    }
}

/// The parameter list of a factory signature, or `""`.
///
/// **The first `(` on the line is not the parameter list.** This was
/// `split_once('(')`, and on `pub(crate) async fn active_route(store: &Store)`
/// it split inside `pub(crate)` and returned
/// `"crate) async fn active_route(store: &Store) -> Result<Option<ProxyRoute>, ("`
/// — which contains `&Store`, so the safety rule read garbage and happened to
/// pass. Every `pub(crate)` factory would have been judged on nonsense, and
/// `pub(crate)` is exactly the visibility this rule exists to police.
///
/// So: find the `(` that follows `fn <name>`, and walk to its matching `)`
/// rather than to the last one on the line.
fn params_of(line: &str) -> String {
    let after_fn = match line.find("fn ") {
        Some(i) => &line[i + 3..],
        None => return String::new(),
    };
    let Some(open) = after_fn.find('(') else {
        return String::new();
    };
    let mut depth = 0usize;
    for (i, c) in after_fn[open..].char_indices() {
        match c {
            '(' => depth += 1,
            ')' => {
                depth -= 1;
                if depth == 0 {
                    return after_fn[open + 1..open + i].trim().to_string();
                }
            }
            _ => {}
        }
    }
    String::new()
}

/// Whether **every** parameter is a store handle.
///
/// Deliberately not "contains a store handle": `f(store: &Store, host: &str)`
/// contains one and can still introduce a host, which is the exact thing being
/// prevented.
fn store_only(line: &str) -> bool {
    let p = params_of(line);
    if p.is_empty() {
        return true;
    }
    p.split(',').all(|arg| {
        let a = arg.trim();
        a.is_empty()
            || a.contains("&Store")
            || a.contains("&mw_store::Store")
            || a == "&self"
            || a == "&mut self"
    })
}

/// Shared factories permitted despite taking more than a store handle, each
/// with the argument for why it cannot reach `ProxyRoute::host`.
///
/// **A named exception rather than a loosened rule**: relaxing the rule
/// generally would let the next factory through silently; an entry here makes
/// the next one an explicit decision with a name attached.
const SHARED_FACTORY_EXCEPTIONS: &[(&str, &str)] = &[(
    "fn route_by_id",
    // t22-e12's admin "test this route" button tests a SPECIFIC route by id —
    // usually NOT the active one, because staging a replacement and testing it
    // before switching is the whole workflow, and `active_route` returns only
    // the active row.
    //
    // The `id` is request-shaped, and that is permitted here because it only
    // SELECTS AMONG ROWS THE OPERATOR CREATED. It cannot introduce a host: every
    // candidate `host` already came from operator configuration through the
    // admin API, so the value that ends up in `ProxyRoute::host` is operator's
    // either way. A selector over operator-supplied rows preserves the
    // invariant; a constructor over caller-supplied rows does not.
    //
    // This is the distinction the rule above cannot see from a signature — `id:
    // &str` and `host: &str` are the same shape — which is why it is written
    // down here instead of inferred.
    "selects among operator-created rows; cannot introduce a host",
)];

/// The rule above, **both ways round**, so it is not merely satisfied by
/// today's visibility.
///
/// This is the control for the assertion about to be exercised for real:
/// `import_routes.rs` needs a route, so `active_route` becomes crate-visible.
/// That is safe and must pass. Widening the *row-fed* constructor alongside it
/// is the thing that must not, and nothing about their visibility distinguishes
/// them — only their parameters do.
#[test]
fn a_shared_factory_may_take_a_store_but_not_a_row() {
    const SAFE: &str = "pub(crate) async fn active_route(store: &mw_store::Store) -> Result<Option<ProxyRoute>, ()> {";
    const UNSAFE: &str = "pub(crate) fn proxy_route(row: &EgressProxyRow) -> Option<ProxyRoute> {";

    // The shape the exception must NOT be wide enough to admit: a store handle,
    // exactly like `route_by_id`, AND a host, which `route_by_id` does not. If
    // the rule were "contains a store handle", this would pass.
    const SMUGGLER: &str =
        "pub(crate) async fn route_for(store: &Store, host: &str) -> Option<ProxyRoute> {";

    assert!(
        store_only(SAFE),
        "sharing the store-fed accessor is what lets another module use the \
         CONFIGURED route, and must be permitted; params read as {:?}",
        params_of(SAFE)
    );
    assert!(
        !store_only(UNSAFE),
        "sharing the row-fed constructor hands every module the ability to build \
         a route from a value it invented, and must be refused; params read as {:?}",
        params_of(UNSAFE)
    );
    assert!(
        !store_only(SMUGGLER),
        "a factory taking a store handle AND a host must not be waved through by \
         the presence of the store handle — that is why the rule is EVERY \
         parameter, not ANY: params read as {:?}",
        params_of(SMUGGLER)
    );

    // The exception is by NAME and is narrow: it admits `route_by_id` and not
    // the same-shaped smuggler.
    let covered = |line: &str| {
        SHARED_FACTORY_EXCEPTIONS
            .iter()
            .any(|(n, _)| line.contains(n))
    };
    assert!(
        covered("pub(crate) async fn route_by_id(store: &Store, id: &str) -> Option<ProxyRoute> {"),
        "route_by_id must be the named exception"
    );
    assert!(
        !covered(SMUGGLER),
        "a same-shaped function must not be covered by the exception: {SMUGGLER}"
    );
    for (name, why) in SHARED_FACTORY_EXCEPTIONS {
        assert!(
            why.len() > 20,
            "exception `{name}` must state why it cannot reach ProxyRoute::host"
        );
    }

    // `params_of` must find the parameter list, not the first `(` on the line.
    // `pub(crate)` puts a paren before the one that matters, and this rule
    // exists precisely to police `pub(crate)` factories.
    assert_eq!(
        params_of(SAFE),
        "store: &mw_store::Store",
        "the parameter list must be read from `fn name(`, not from `pub(`"
    );
    assert_eq!(params_of(SMUGGLER), "store: &Store, host: &str");

    // Both are classified as factories in the first place — otherwise the rule
    // above never runs on them and this control proves nothing.
    for src in [SAFE, UNSAFE] {
        let mut scan = Scan::default();
        classify("fixture.rs", src, false, &mut scan);
        assert_eq!(
            scan.production_of(Kind::Factory).len(),
            1,
            "must be seen as a factory at all: {src}"
        );
    }
}
