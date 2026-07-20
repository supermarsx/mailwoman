//! t17-e-e2e — the SHIPPED SPA bundle boots under the enforced Trusted-Types CSP.
//!
//! 26.17 re-enabled `require-trusted-types-for 'script'` in the shell CSP AND
//! registered a default TT policy in `apps/web/src/main.tsx` so the Solid boot
//! `innerHTML` sinks don't throw under enforcement. `t16_csp_shell.rs` proves the
//! served CSP header carries the directive (against a stub index). THIS leg serves the
//! REAL built `apps/web/dist` through `build_app` and asserts the shipped artifact is
//! internally consistent for a TT-enforced boot:
//!   * `GET /` serves the real shell (200) under the enforced CSP (`require-trusted-
//!     types-for 'script'`, `script-src 'self' 'wasm-unsafe-eval'`);
//!   * the entry bundle it references is SERVED (200) and REGISTERS the default TT
//!     policy (`trustedTypes.createPolicy("default", …)`) — so a browser installs the
//!     passthrough policy before the boot innerHTML sinks run, and no TrustedTypes
//!     violation is thrown at startup.
//!
//! HONEST SCOPE / LOUD-FLAG: a real browser boot (pixel-level "the SPA paints, zero TT
//! violations in the console") needs a live Chromium with the CSP enforced. No browser
//! is drivable from this agent host (no connected Chrome extension), so the pixel-level
//! assertion is NOT driven here — it is the web/Playwright gate's job. This leg proves
//! the served-artifact half: the enforced CSP + the policy-registering entry are both
//! shipped and served together, which is the wiring a browser boot depends on.
//!
//! Loud-SKIPs (never silently passes) when `apps/web/dist` is absent (web not built).

use std::path::PathBuf;

use mw_server::{AppConfig, build_app};

fn dist_dir() -> PathBuf {
    // crates/mw-server → ../../apps/web/dist
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
        .join("apps")
        .join("web")
        .join("dist")
}

async fn spawn_server(web: PathBuf) -> String {
    let db = std::env::temp_dir().join(format!(
        "mw-t17-tt-{}-{}.db",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    let config = AppConfig {
        db_path: db.to_string_lossy().into_owned(),
        server_key_hex: None,
        web_dir: Some(web),
        cookie_secure: false,
        mode: mw_server::ServerMode::Proxy,
        hardening: mw_server::HardeningConfig::default(),
        security: mw_server::SecurityConfig::default(),
    };
    let app = build_app(config).await.unwrap();
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    tokio::spawn(async move {
        axum::serve(listener, app).await.unwrap();
    });
    format!("http://{addr}")
}

/// Extract the `/assets/index-*.js` module entry from the shell HTML.
fn entry_src(html: &str) -> Option<String> {
    let i = html.find("/assets/index-")?;
    let rest = &html[i..];
    let end = rest.find(".js")? + 3;
    Some(rest[..end].to_string())
}

#[tokio::test]
async fn shipped_bundle_is_served_under_enforced_trusted_types_and_registers_the_default_policy() {
    let dist = dist_dir();
    if !dist.join("index.html").exists() {
        eprintln!(
            "\n[t17 TT SKIP] {} not built (no apps/web/dist/index.html) — build the web \
             SPA (pnpm -C apps/web build) to exercise the served-artifact TT boot.\n",
            dist.display()
        );
        return;
    }

    let base = spawn_server(dist).await;

    // GET / — the REAL shell served under the enforced CSP.
    let resp = reqwest::get(format!("{base}/")).await.unwrap();
    assert_eq!(resp.status(), 200, "the real shell is served");
    let csp = resp
        .headers()
        .get("content-security-policy")
        .and_then(|v| v.to_str().ok())
        .expect("shell carries a CSP header")
        .to_string();
    assert!(
        csp.contains("require-trusted-types-for 'script'"),
        "the shipped shell CSP enforces Trusted-Types: {csp}"
    );
    assert!(
        csp.contains("script-src 'self' 'wasm-unsafe-eval'"),
        "script-src permits self + wasm only: {csp}"
    );
    let html = resp.text().await.unwrap();
    let entry = entry_src(&html)
        .unwrap_or_else(|| panic!("the shell references an /assets/index-*.js entry: {html}"));

    // The entry bundle is SERVED (200) under the same enforced CSP…
    let asset = reqwest::get(format!("{base}{entry}")).await.unwrap();
    assert_eq!(asset.status(), 200, "the entry bundle {entry} is served");
    let entry_csp = asset
        .headers()
        .get("content-security-policy")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("")
        .to_string();
    assert!(
        entry_csp.contains("require-trusted-types-for 'script'"),
        "the entry asset is also served under the enforced CSP: {entry_csp}"
    );

    // …and it REGISTERS the default TT policy (the passthrough `createHTML`), which a
    // browser installs before the Solid boot innerHTML sinks run — so `require-trusted-
    // types-for 'script'` does not throw a TrustedTypes violation at startup.
    let js = asset.text().await.unwrap();
    assert!(
        js.contains("createPolicy(\"default\"") || js.contains("createPolicy('default'"),
        "the shipped entry bundle registers the default Trusted-Types policy"
    );
    assert!(
        js.contains("defaultPolicy"),
        "the default-policy guard (defaultPolicy === null) is present in the shipped bundle"
    );

    eprintln!(
        "[t17 TT] shipped shell + entry {entry} served under enforced CSP; entry registers the \
         default TT policy. NOTE: pixel-level browser boot (zero TT violations in-console) is the \
         web/Playwright gate's job — no Chrome is drivable from this host."
    );
}
