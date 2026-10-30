//! t32-e24a — no admin route skips the capability check.
//!
//! Every admin route is mounted through `admin_router::admin_endpoint`, which puts
//! the capability gate in front of the handler and records the route in
//! `admin_router::registry()`. This test reads `crates/mw-server/src/**` and finds
//! every route-mounting call (`.route(`, `.route_service(`, `.nest(`,
//! `.nest_service(`), wherever its path argument starts. A path built with
//! `mw_admin::config::admin_route(..)` outside the registrar is an admin route that
//! bypasses the gate, and fails the test. Since t32-h15 no admin route is written
//! as a `"/admin…"` literal (`t32_h15_admin_path.rs` fails on one), so the scan
//! keys on the helper call, not on a literal, and a path argument that is neither a
//! literal nor the helper must be listed in [`KNOWN_DYNAMIC`].
//!
//! [`LEGACY`] lists the routes mounted the old way that e24b moves into the
//! registry; the list may only shrink, and an entry that is no longer in the source
//! fails the test so it cannot linger.
//!
//! The second half boots the app and calls every registered route without an admin
//! session: each one that is not a sign-in step answers `401`.

use std::collections::BTreeSet;
use std::net::SocketAddr;
use std::path::{Path, PathBuf};

use mw_server::admin_router::{Access, registry};
use mw_server::{AppConfig, HardeningConfig, SecurityConfig, ServerMode, V6Config, build_app_full};

mod common;
use common::test_db;

const KEY_HEX: &str = "00112233445566778899aabbccddeeff00112233445566778899aabbccddeeff";

/// Admin routes still mounted outside the registry, as `(file, suffix)`. e24b
/// moves them into `admin_router.rs` and deletes the line here.
const LEGACY: &[(&str, &str)] = &[];

/// Route-mounting calls whose path is neither a literal nor the admin helper, as
/// `(file, argument)`. Each one is checked by hand: it is not an admin route, or it
/// is the registrar itself.
const KNOWN_DYNAMIC: &[(&str, &str)] = &[
    // The deploy-prefix mount of the whole app (`MW_BASE_PATH`).
    ("lib.rs", "base"),
    // The registrar: the one place an admin path reaches `.route(`.
    ("admin_router.rs", "&mw_admin::config::admin_route(suffix)"),
];

const CALLS: &[&str] = &[".route(", ".route_service(", ".nest(", ".nest_service("];

fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .nth(2)
        .unwrap()
        .to_path_buf()
}

fn rust_files(dir: &Path, out: &mut Vec<PathBuf>) {
    for e in std::fs::read_dir(dir).unwrap() {
        let p = e.unwrap().path();
        if p.is_dir() {
            rust_files(&p, out);
        } else if p.extension().is_some_and(|x| x == "rs") {
            out.push(p);
        }
    }
}

/// `line` without its `//` comment. A `//` inside a string literal (a URL) is
/// kept.
fn strip_comment(line: &str) -> &str {
    let bytes = line.as_bytes();
    let mut in_str = false;
    let mut i = 0;
    while i < bytes.len() {
        match bytes[i] {
            b'\\' if in_str => i += 1,
            b'"' => in_str = !in_str,
            b'/' if !in_str && bytes.get(i + 1) == Some(&b'/') => return &line[..i],
            _ => {}
        }
        i += 1;
    }
    line
}

/// A file's shipped code: comments removed, `#[cfg(test)] mod … { … }` blocks
/// removed, lines joined with `\n`.
fn shipped_code(text: &str) -> String {
    let mut out = String::new();
    let mut depth: Option<i32> = None;
    let mut armed = false;
    for line in text.lines() {
        let code = strip_comment(line).trim();
        if let Some(d) = depth.as_mut() {
            *d += code.matches('{').count() as i32 - code.matches('}').count() as i32;
            if *d <= 0 {
                depth = None;
            }
            continue;
        }
        if code.starts_with("#[cfg(test)]") {
            armed = true;
            continue;
        }
        if armed && code.starts_with("mod ") && code.ends_with('{') {
            depth = Some(1);
            armed = false;
            continue;
        }
        armed = false;
        out.push_str(code);
        out.push('\n');
    }
    out
}

/// The first argument of the call whose `(` ends at `open`: everything up to the
/// first `,` (or `)`) at depth 0, whitespace collapsed.
fn first_arg(code: &str, open: usize) -> String {
    let mut depth = 0i32;
    let mut in_str = false;
    let mut prev = '\0';
    let mut out = String::new();
    for ch in code[open..].chars() {
        if in_str {
            out.push(ch);
            if ch == '"' && prev != '\\' {
                in_str = false;
            }
        } else {
            match ch {
                '"' => {
                    in_str = true;
                    out.push(ch);
                }
                '(' | '[' | '{' => {
                    depth += 1;
                    out.push(ch);
                }
                ')' | ']' | '}' if depth == 0 => break,
                ')' | ']' | '}' => {
                    depth -= 1;
                    out.push(ch);
                }
                ',' if depth == 0 => break,
                c if c.is_whitespace() => {}
                c => out.push(c),
            }
        }
        prev = ch;
    }
    out
}

/// What a route call's path argument is.
#[derive(Debug, PartialEq, Eq)]
enum PathArg {
    Literal,
    Admin(String),
    Dynamic(String),
}

fn classify(arg: &str) -> PathArg {
    if arg.starts_with('"') {
        return PathArg::Literal;
    }
    let bare = arg.trim_start_matches('&');
    for helper in ["mw_admin::config::admin_route(", "admin_route("] {
        if let Some(rest) = bare.strip_prefix(helper)
            && let Some(lit) = rest.strip_prefix('"')
            && let Some(end) = lit.find('"')
            && lit[end..] == *"\")"
        {
            return PathArg::Admin(lit[..end].to_string());
        }
    }
    PathArg::Dynamic(arg.to_string())
}

/// Every route call in `crates/mw-server/src`: `(file name, line, argument)`.
fn route_calls() -> Vec<(String, usize, PathArg)> {
    let src = repo_root().join("crates/mw-server/src");
    let mut files = Vec::new();
    rust_files(&src, &mut files);
    let mut out = Vec::new();
    for f in &files {
        let name = f.file_name().unwrap().to_string_lossy().into_owned();
        let code = shipped_code(&std::fs::read_to_string(f).unwrap());
        for call in CALLS {
            for (i, _) in code.match_indices(call) {
                let open = i + call.len();
                let line = code[..i].matches('\n').count() + 1;
                out.push((name.clone(), line, classify(&first_arg(&code, open))));
            }
        }
    }
    out
}

#[test]
fn every_admin_route_is_registered() {
    let calls = route_calls();
    assert!(calls.len() > 80, "only {} route calls found", calls.len());
    let legacy: BTreeSet<(&str, &str)> = LEGACY.iter().copied().collect();
    let registered: BTreeSet<&str> = registry().iter().map(|e| e.suffix).collect();
    let mut bypass = Vec::new();
    let mut unknown = Vec::new();
    let mut legacy_seen = BTreeSet::new();
    for (file, line, arg) in &calls {
        match arg {
            PathArg::Literal => {}
            PathArg::Admin(suffix) => {
                if legacy.contains(&(file.as_str(), suffix.as_str())) {
                    legacy_seen.insert((file.clone(), suffix.clone()));
                } else {
                    bypass.push(format!(
                        "{file}:{line} mounts {suffix} outside admin_router::admin_endpoint"
                    ));
                }
            }
            PathArg::Dynamic(text) => {
                if !KNOWN_DYNAMIC.contains(&(file.as_str(), text.as_str())) {
                    unknown.push(format!("{file}:{line} mounts a computed path: {text}"));
                }
            }
        }
    }
    assert!(
        bypass.is_empty(),
        "admin routes that skip the capability gate:\n{}",
        bypass.join("\n")
    );
    assert!(
        unknown.is_empty(),
        "route paths that are neither a literal nor the admin helper \
         (register admin routes in admin_router.rs; list a checked non-admin one in KNOWN_DYNAMIC):\n{}",
        unknown.join("\n")
    );
    let stale: Vec<String> = LEGACY
        .iter()
        .filter(|(f, s)| !legacy_seen.contains(&(f.to_string(), s.to_string())))
        .map(|(f, s)| format!("{f} {s}"))
        .collect();
    assert!(
        stale.is_empty(),
        "LEGACY entries no longer mounted the old way; delete them:\n{}",
        stale.join("\n")
    );
    let both: Vec<&str> = LEGACY
        .iter()
        .map(|(_, s)| *s)
        .filter(|s| registered.contains(s))
        .collect();
    assert!(
        both.is_empty(),
        "registered and still mounted the old way: {both:?}"
    );
    for (file, dynamic) in KNOWN_DYNAMIC {
        assert!(
            calls
                .iter()
                .any(|(f, _, a)| f == file && *a == PathArg::Dynamic(dynamic.to_string())),
            "KNOWN_DYNAMIC entry {file} {dynamic} is gone; delete it"
        );
    }
}

/// The scanner itself: the call forms it must see, wherever the argument starts.
#[test]
fn the_scanner_sees_every_form() {
    let code = shipped_code(
        r#"
        Router::new()
            .route(&admin_route("/a"), get(a))
            .route(
                &mw_admin::config::admin_route("/b/{id}"),
                post(b),
            )
            .route("/api/c", get(c)) // .route(&admin_route("/commented"), get(x))
            .nest_service(&admin_route("/d"), svc)
            .route(&path, get(e))
            .route(&format!("{}/f", admin_path()), get(f));
        #[cfg(test)]
        mod tests {
            fn t() { Router::new().route(&admin_route("/in-test"), get(t)); }
        }
        "#,
    );
    let mut found = Vec::new();
    for call in CALLS {
        for (i, _) in code.match_indices(call) {
            found.push(classify(&first_arg(&code, i + call.len())));
        }
    }
    found.sort_by_key(|a| format!("{a:?}"));
    let mut want = vec![
        PathArg::Admin("/a".into()),
        PathArg::Admin("/b/{id}".into()),
        PathArg::Literal,
        PathArg::Admin("/d".into()),
        PathArg::Dynamic("&path".into()),
        PathArg::Dynamic("&format!(\"{}/f\",admin_path())".into()),
    ];
    want.sort_by_key(|a| format!("{a:?}"));
    assert_eq!(found, want);
}

#[tokio::test]
async fn every_registered_route_refuses_a_caller_without_a_session() {
    // t100: this test calls every route from one address on purpose; admin-probe
    // protection would (correctly) block it partway. Its own tests are t100_*.
    unsafe { std::env::set_var("MW_ADMIN_PROBE_ENABLED", "false") };
    let dir = test_db::unique_dir("mw-t32-e24a-scan");
    let config = AppConfig {
        db_path: dir.join("mw.db").to_string_lossy().into_owned(),
        server_key_hex: Some(KEY_HEX.to_string()),
        web_dir: None,
        cookie_secure: false,
        mode: ServerMode::Proxy,
        hardening: HardeningConfig::default(),
        security: SecurityConfig::default(),
    };
    let v6 = V6Config {
        admin_enabled: true,
        admin_username: Some("root".into()),
        admin_password: Some("hunter2".into()),
        redis_url: None,
    };
    let app = build_app_full(config, v6).await.expect("server boots").0;
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    tokio::spawn(async move {
        let _ = axum::serve(
            listener,
            app.into_make_service_with_connect_info::<SocketAddr>(),
        )
        .await;
    });
    let base = format!("http://{addr}");
    let client = reqwest::Client::new();
    let mut checked = 0;
    for e in registry() {
        if e.access == Access::Public {
            continue;
        }
        let path = e
            .suffix
            .split('/')
            .map(|s| if s.starts_with('{') { "x" } else { s })
            .collect::<Vec<_>>()
            .join("/");
        let url = format!("{base}{}", mw_admin::config::admin_route(&path));
        let resp = client
            .request(e.method.clone(), &url)
            .header("origin", &base)
            .header("content-type", "application/json")
            .body("{}")
            .send()
            .await
            .unwrap();
        assert_eq!(resp.status(), 401, "{} {}", e.method, e.suffix);
        let body: serde_json::Value = resp.json().await.unwrap();
        assert_eq!(body["code"], "unauthenticated", "{} {}", e.method, e.suffix);
        checked += 1;
    }
    assert!(checked >= 1);

    // Signed in as the environment admin, the catalogue answers.
    let login = client
        .post(format!("{base}{}", mw_admin::config::admin_route("/login")))
        .header("origin", &base)
        .json(&serde_json::json!({"username": "root", "password": "hunter2"}))
        .send()
        .await
        .unwrap();
    assert_eq!(login.status(), 200);
    let cookie = login
        .headers()
        .get_all("set-cookie")
        .iter()
        .map(|v| v.to_str().unwrap().split(';').next().unwrap().to_string())
        .collect::<Vec<_>>()
        .join("; ");
    let caps: serde_json::Value = client
        .get(format!(
            "{base}{}",
            mw_admin::config::admin_route("/capabilities")
        ))
        .header("cookie", cookie)
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    let ids: Vec<&str> = caps["capabilities"]
        .as_array()
        .unwrap()
        .iter()
        .map(|c| c["id"].as_str().unwrap())
        .collect();
    assert!(ids.contains(&"users.read") && ids.contains(&"data_export.run"));
}
