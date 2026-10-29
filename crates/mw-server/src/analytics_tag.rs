//! Browser analytics tags (t63): presets for common providers and an admin-supplied
//! custom tag, loaded into the mail shell only.
//!
//! This is separate from the server-side event exporter in `analytics_config`
//! (Umami / Matomo / GA4 Measurement Protocol), which never runs provider code in
//! the browser. A browser tag does: the provider's script runs in the mail shell
//! with full access to the page. So:
//!
//! * The server writes the loader. A preset is built from typed fields (site ID,
//!   host, region); a custom tag is either a script URL or a stored snippet. Either
//!   way the shell references one same-origin file, `assets/analytics/<sha256>.js`,
//!   whose name is the SHA-256 of its bytes. No inline script is ever added, so the
//!   shell CSP keeps `script-src 'self'` with no `'unsafe-inline'`/`'unsafe-eval'`.
//! * The CSP is widened only by the provider's documented origins (or, for a
//!   custom tag, the origins the admin declared), in `script-src`, `connect-src`
//!   and `img-src`, and only on the mail shell document served to a signed-in user
//!   who consented, without Do Not Track or Global Privacy Control. Every other
//!   response keeps the fixed shell CSP: the admin console, the sign-in page (no
//!   session), OAuth/OIDC consent pages, the detached workspace popup, and the
//!   reader / appearance frames (which carry their own policy).
//! * Saving a configuration that turns a tag on or changes an enabled one needs a
//!   full Super admin, a fresh step-up proof and the loaded revision; a custom tag
//!   also needs an explicit acknowledgement that it runs with full page access. The
//!   audit entry records the loader's SHA-256 and the CSP origins, never the snippet.
use crate::{
    AppState,
    admin_ctx::{AdminApiError, AdminCtx, AdminJson, check_if_match, etag_of, with_etag},
    admin_router::AdminRegistrar,
    capabilities::Capability,
};
use axum::{
    Json,
    extract::State,
    http::{HeaderMap, HeaderValue, Method, StatusCode, header},
    response::{IntoResponse, Response},
};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};

pub const KEY: &str = "admin.analytics.tag.v1";
/// The largest stored custom snippet. A tag is a loader, not an application.
pub const MAX_SNIPPET_BYTES: usize = 32 * 1024;
/// Origins per directive for a custom tag.
pub const MAX_ORIGINS: usize = 16;
/// Served path prefix, relative to the deploy base.
pub const ASSET_PREFIX: &str = "assets/analytics/";
/// The `<meta>` the shell carries when a tag is active; its content is the
/// space-separated script origins the app's Trusted Types policy admits.
pub const META_NAME: &str = "mw-analytics";

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Provider {
    #[default]
    Plausible,
    Matomo,
    Umami,
    Fathom,
    Posthog,
    SimpleAnalytics,
    Cloudflare,
    Custom,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum CustomKind {
    #[default]
    Url,
    Inline,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default, rename_all = "camelCase", deny_unknown_fields)]
pub struct Config {
    pub enabled: bool,
    pub provider: Provider,
    /// Plausible domain, Matomo site ID, Umami website UUID, Fathom site ID,
    /// PostHog project key, Simple Analytics hostname (optional), Cloudflare token.
    pub site_id: String,
    /// Self-hosted base URL for Plausible, Matomo and Umami.
    pub host: String,
    /// PostHog cloud region: `us` or `eu`.
    pub region: String,
    pub custom_kind: CustomKind,
    pub script_url: String,
    pub snippet: String,
    pub script_origins: Vec<String>,
    pub connect_origins: Vec<String>,
    pub img_origins: Vec<String>,
}

/// What a valid, enabled configuration adds to the shell.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Plan {
    pub script_src: Vec<String>,
    pub connect_src: Vec<String>,
    pub img_src: Vec<String>,
    /// The exact bytes served at [`Plan::path`].
    pub script: Vec<u8>,
    /// Lowercase hex SHA-256 of `script`.
    pub sha256: String,
    /// Script URLs the loader fetches (empty for an inline snippet), for the Test
    /// button.
    pub fetches: Vec<String>,
}
impl Plan {
    /// `assets/analytics/<sha256>.js`, relative to the deploy base.
    pub fn path(&self) -> String {
        format!("{ASSET_PREFIX}{}.js", self.sha256)
    }
}

// ── origin validation ─────────────────────────────────────────────────────────

/// Registrable suffixes under which a `*.` wildcard would admit other tenants'
/// hosts. Not the full Public Suffix List: a short list of the common two-label
/// country suffixes and the shared hosting platforms where anyone can publish.
const SHARED_SUFFIXES: &[&str] = &[
    "co.uk",
    "org.uk",
    "ac.uk",
    "gov.uk",
    "com.au",
    "net.au",
    "org.au",
    "co.nz",
    "co.jp",
    "co.in",
    "co.za",
    "com.br",
    "com.cn",
    "com.mx",
    "com.tr",
    "com.pt",
    "github.io",
    "gitlab.io",
    "herokuapp.com",
    "vercel.app",
    "netlify.app",
    "pages.dev",
    "workers.dev",
    "web.app",
    "firebaseapp.com",
    "appspot.com",
    "blogspot.com",
    "cloudfront.net",
    "azurewebsites.net",
    "amazonaws.com",
    "s3.amazonaws.com",
    "googleusercontent.com",
    "glitch.me",
    "repl.co",
];

fn valid_label(label: &str) -> bool {
    !label.is_empty()
        && label.len() <= 63
        && !label.starts_with('-')
        && !label.ends_with('-')
        && label
            .bytes()
            .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'-')
}

fn valid_domain(host: &str) -> bool {
    host.len() <= 253 && host.split('.').all(valid_label)
}

fn valid_ipv4(host: &str) -> bool {
    host.parse::<std::net::Ipv4Addr>().is_ok()
}

/// Parse one CSP source origin. Accepts `https://host[:port]` (a trailing `/` is
/// dropped), and with `wildcard`, `https://*.example.com`. Returns the normalized
/// form, which is the only text ever written into the CSP.
///
/// Refused: any scheme but https (so `http:`, `data:`, `blob:`, `javascript:`,
/// `filesystem:`), a bare `*` or `https://*`, a wildcard over a TLD or a shared
/// suffix, a wildcard anywhere but the leftmost label, paths, queries, fragments,
/// credentials, CSP keywords and anything with whitespace, quotes, `;` or `,`.
pub fn parse_origin(raw: &str, wildcard: bool) -> Result<String, &'static str> {
    let raw = raw.trim();
    if raw.is_empty() || raw.len() > 300 {
        return Err("Enter an origin such as https://stats.example.com.");
    }
    if raw
        .bytes()
        .any(|b| b.is_ascii_whitespace() || b.is_ascii_control() || b"'\";,\\<>".contains(&b))
    {
        return Err("An origin cannot contain spaces, quotes, semicolons or commas.");
    }
    let lower = raw.to_ascii_lowercase();
    if raw == "*" || lower == "https://*" || lower == "https://*/" {
        return Err("A bare * is not allowed. Name each origin.");
    }
    for scheme in ["data:", "blob:", "javascript:", "filesystem:", "about:", "ws:", "wss:"] {
        if lower.starts_with(scheme) {
            return Err("Only https origins are allowed (no data:, blob:, javascript: or other schemes).");
        }
    }
    let Some(rest) = lower.strip_prefix("https://") else {
        return Err("Only https origins are allowed.");
    };
    let rest = rest.strip_suffix('/').unwrap_or(rest);
    if rest.contains(['/', '?', '#', '@', '%']) {
        return Err("An origin is scheme, host and optional port only: no path, query, fragment or credentials.");
    }
    let (host, port) = match rest.rsplit_once(':') {
        Some((host, port)) => {
            let port = port
                .parse::<u16>()
                .ok()
                .filter(|p| *p > 0 && port.bytes().all(|b| b.is_ascii_digit()))
                .ok_or("Enter a valid port number.")?;
            (host, (port != 443).then_some(port))
        }
        None => (rest, None),
    };
    if let Some(base) = host.strip_prefix("*.") {
        if !wildcard {
            return Err("Wildcards are not allowed here.");
        }
        if base.contains('*') || !valid_domain(base) {
            return Err("A wildcard is allowed only as the leftmost label, as in https://*.example.com.");
        }
        let labels = base.split('.').count();
        let tld_numeric = base
            .rsplit('.')
            .next()
            .is_some_and(|tld| tld.bytes().all(|b| b.is_ascii_digit()));
        if labels < 2 || tld_numeric || SHARED_SUFFIXES.contains(&base) {
            return Err("A wildcard must sit under a domain you control, not a top-level or shared suffix.");
        }
    } else if host.contains('*') {
        return Err("A wildcard is allowed only as the leftmost label, as in https://*.example.com.");
    } else if !(valid_ipv4(host) || valid_domain(host)) {
        return Err("Enter a valid host name.");
    }
    Ok(match port {
        Some(port) => format!("https://{host}:{port}"),
        None => format!("https://{host}"),
    })
}

/// Does `origin` (normalized `https://host[:port]`) fall under `source`, the way a
/// CSP host source with an optional leading `*.` matches?
pub fn origin_matches(source: &str, origin: &str) -> bool {
    if source == origin {
        return true;
    }
    let (Some(s), Some(o)) = (
        source.strip_prefix("https://*."),
        origin.strip_prefix("https://"),
    ) else {
        return false;
    };
    let (s_host, s_port) = s.split_once(':').unwrap_or((s, ""));
    let (o_host, o_port) = o.split_once(':').unwrap_or((o, ""));
    s_port == o_port && o_host.len() > s_host.len() && o_host.ends_with(&format!(".{s_host}"))
}

/// `https://host[:port]` of an absolute https URL.
fn origin_of(url: &reqwest::Url) -> Option<String> {
    if url.scheme() != "https" {
        return None;
    }
    let host = url.host_str()?;
    Some(match url.port() {
        Some(port) => format!("https://{host}:{port}"),
        None => format!("https://{host}"),
    })
}

/// A provider base URL: `https://host[:port][/path]`, no query, fragment or
/// credentials, path limited to plain characters. Returned without a trailing `/`,
/// with its origin.
fn parse_base(raw: &str) -> Result<(String, String), &'static str> {
    const MSG: &str = "Enter the provider's https base URL, without credentials, query or fragment.";
    let raw = raw.trim();
    if raw.len() > 512 {
        return Err(MSG);
    }
    let url = reqwest::Url::parse(raw).map_err(|_| MSG)?;
    if url.scheme() != "https"
        || !url.username().is_empty()
        || url.password().is_some()
        || url.query().is_some()
        || url.fragment().is_some()
        || !url
            .path()
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || b"/-._~".contains(&b))
    {
        return Err(MSG);
    }
    let origin = parse_origin(&origin_of(&url).ok_or(MSG)?, false)?;
    let path = url.path().trim_end_matches('/');
    Ok((format!("{origin}{path}"), origin))
}

fn valid_hostname_id(id: &str) -> bool {
    id.contains('.') && valid_domain(id)
}

fn origins(raw: &[String], label: &'static str) -> Result<Vec<String>, String> {
    if raw.len() > MAX_ORIGINS {
        return Err(format!("{label}: at most {MAX_ORIGINS} origins."));
    }
    let mut out: Vec<String> = Vec::new();
    for entry in raw {
        let origin = parse_origin(entry, true).map_err(|e| format!("{label}: {e}"))?;
        if !out.contains(&origin) {
            out.push(origin);
        }
    }
    Ok(out)
}

// ── loaders ───────────────────────────────────────────────────────────────────

fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}

fn js(s: &str) -> String {
    serde_json::to_string(s).expect("string literal")
}

/// The guard every preset loader starts with. Defence in depth: the server only
/// references the loader from a consented mail shell, but the file itself also
/// refuses to act in a frame, in the admin console, under DNT/GPC or in a document
/// the server did not mark.
const PRESET_HEAD: &str = r#"/* Mailwoman analytics loader. Generated by the server; do not edit. */
(function () {
  "use strict";
  var n = navigator;
  if (n.doNotTrack === "1" || n.globalPrivacyControl === true) return;
  if (window.top !== window || document.querySelector('meta[name="mw-console"]') || !document.querySelector('meta[name="mw-analytics"]')) return;
  function load(src, attrs) {
    var s = document.createElement("script");
    s.async = true;
    s.crossOrigin = "anonymous";
    s.referrerPolicy = "no-referrer";
    for (var k in attrs) if (Object.prototype.hasOwnProperty.call(attrs, k)) s.setAttribute(k, attrs[k]);
    s.src = src;
    document.head.appendChild(s);
  }
"#;
const PRESET_TAIL: &str = "})();\n";

/// The guard in front of a custom snippet. It throws rather than wrapping the
/// snippet in a block, so the snippet's top-level declarations stay global.
const INLINE_HEAD: &str = r#"/* Mailwoman custom analytics tag. Stored by an administrator. */
if (navigator.doNotTrack === "1" || navigator.globalPrivacyControl === true || window.top !== window || document.querySelector('meta[name="mw-console"]') || !document.querySelector('meta[name="mw-analytics"]')) throw new Error("analytics tag not permitted in this document");
"#;

fn attrs(pairs: &[(&str, String)]) -> String {
    let body: Vec<String> = pairs.iter().map(|(k, v)| format!("{}: {}", js(k), js(v))).collect();
    format!("{{{}}}", body.join(", "))
}

struct Built {
    script_src: Vec<String>,
    connect_src: Vec<String>,
    img_src: Vec<String>,
    body: String,
    fetches: Vec<String>,
    inline: bool,
}

fn preset(script_src: &[&str], connect_src: &[&str], img_src: &[&str], body: String, fetches: Vec<String>) -> Built {
    let own = |v: &[&str]| v.iter().map(|s| s.to_string()).collect::<Vec<_>>();
    Built {
        script_src: own(script_src),
        connect_src: own(connect_src),
        img_src: own(img_src),
        body,
        fetches,
        inline: false,
    }
}

fn dedup(mut v: Vec<String>) -> Vec<String> {
    let mut seen = Vec::new();
    v.retain(|s| {
        if seen.contains(s) {
            false
        } else {
            seen.push(s.clone());
            true
        }
    });
    v
}

impl Config {
    /// Validate and build. `Ok(None)` when disabled. Errors are user-facing.
    pub fn plan(&self) -> Result<Option<Plan>, String> {
        if !self.enabled {
            // A disabled draft may be incomplete, but nothing oversized or
            // unsafe is ever stored.
            self.limits()?;
            return Ok(None);
        }
        let built = self.build()?;
        let script = if built.inline {
            format!("{INLINE_HEAD}{}\n", built.body)
        } else {
            format!("{PRESET_HEAD}{}{PRESET_TAIL}", built.body)
        }
        .into_bytes();
        let sha256 = hex(&Sha256::digest(&script));
        Ok(Some(Plan {
            script_src: dedup(built.script_src),
            connect_src: dedup(built.connect_src),
            img_src: dedup(built.img_src),
            script,
            sha256,
            fetches: built.fetches,
        }))
    }

    /// Plan ignoring `enabled`, for the Test button and the CSP preview.
    pub fn preview(&self) -> Result<Plan, String> {
        let mut on = self.clone();
        on.enabled = true;
        on.plan().map(|p| p.expect("enabled"))
    }

    fn limits(&self) -> Result<(Vec<String>, Vec<String>, Vec<String>), String> {
        if self.site_id.trim().len() > 128 || self.host.len() > 512 || self.region.len() > 8 || self.script_url.len() > 2048 {
            return Err("A field is too long.".into());
        }
        if self.snippet.len() > MAX_SNIPPET_BYTES {
            return Err(format!(
                "The custom snippet is larger than {} KiB. Host the script elsewhere and use its URL.",
                MAX_SNIPPET_BYTES / 1024
            ));
        }
        Ok((
            origins(&self.script_origins, "Script origins")?,
            origins(&self.connect_origins, "Connection origins")?,
            origins(&self.img_origins, "Image origins")?,
        ))
    }

    fn build(&self) -> Result<Built, String> {
        let site = self.site_id.trim();
        let (script_origins, connect_origins, img_origins) = self.limits()?;
        let required = |ok: bool, msg: &str| -> Result<(), String> {
            if ok { Ok(()) } else { Err(msg.to_string()) }
        };
        let host_or = |default: &str| -> Result<(String, String), String> {
            parse_base(if self.host.trim().is_empty() { default } else { &self.host })
                .map_err(str::to_string)
        };
        Ok(match self.provider {
            Provider::Plausible => {
                required(valid_hostname_id(site), "Enter the site domain registered in Plausible, such as mail.example.com.")?;
                let (base, origin) = host_or("https://plausible.io")?;
                let src = format!("{base}/js/script.js");
                preset(
                    &[&origin],
                    &[&origin],
                    &[],
                    format!(
                        "  load({}, {});\n",
                        js(&src),
                        attrs(&[("data-domain", site.into()), ("data-api", format!("{base}/api/event"))])
                    ),
                    vec![src],
                )
            }
            Provider::Matomo => {
                required(site.parse::<u32>().is_ok_and(|id| id > 0), "Matomo uses a positive numeric site ID.")?;
                required(!self.host.trim().is_empty(), "Enter your Matomo base URL.")?;
                let (base, origin) = host_or("")?;
                let src = format!("{base}/matomo.js");
                preset(
                    &[&origin],
                    &[&origin],
                    &[&origin],
                    format!(
                        "  var q = window._paq = window._paq || [];\n  q.push([\"disableCookies\"]);\n  q.push([\"trackPageView\"]);\n  q.push([\"enableLinkTracking\"]);\n  q.push([\"setTrackerUrl\", {}]);\n  q.push([\"setSiteId\", {}]);\n  load({}, {{}});\n",
                        js(&format!("{base}/matomo.php")),
                        js(site),
                        js(&src)
                    ),
                    vec![src],
                )
            }
            Provider::Umami => {
                required(uuid::Uuid::parse_str(site).is_ok(), "Umami uses a website UUID.")?;
                let (base, origin) = host_or("https://cloud.umami.is")?;
                let src = format!("{base}/script.js");
                preset(
                    &[&origin],
                    &[&origin],
                    &[],
                    format!(
                        "  load({}, {});\n",
                        js(&src),
                        attrs(&[
                            ("data-website-id", site.to_ascii_lowercase()),
                            ("data-host-url", base.clone()),
                            ("data-exclude-hash", "true".into()),
                            ("data-exclude-search", "true".into()),
                        ])
                    ),
                    vec![src],
                )
            }
            Provider::Fathom => {
                required(
                    (4..=16).contains(&site.len()) && site.bytes().all(|b| b.is_ascii_uppercase() || b.is_ascii_digit()),
                    "Fathom uses a site ID of 4 to 16 capital letters and digits.",
                )?;
                let src = "https://cdn.usefathom.com/script.js".to_string();
                let o = "https://cdn.usefathom.com";
                preset(
                    &[o],
                    &[o],
                    &[o],
                    format!("  load({}, {});\n", js(&src), attrs(&[("data-site", site.into())])),
                    vec![src],
                )
            }
            Provider::Posthog => {
                required(
                    site.strip_prefix("phc_").is_some_and(|k| (16..=64).contains(&k.len()) && k.bytes().all(|b| b.is_ascii_alphanumeric())),
                    "Enter the PostHog project API key (phc_…).",
                )?;
                let region = match self.region.as_str() {
                    "us" | "" => "us",
                    "eu" => "eu",
                    _ => return Err("PostHog region is us or eu.".into()),
                };
                let api = format!("https://{region}.i.posthog.com");
                let assets = format!("https://{region}-assets.i.posthog.com");
                let src = format!("{assets}/static/array.js");
                preset(
                    &[&assets, &api],
                    &[&api, &assets],
                    &[],
                    format!(
                        "  if (!window.posthog) window.posthog = {{ __SV: 1, _i: [[{}, {{ api_host: {}, disable_session_recording: true }}, undefined]] }};\n  load({}, {{}});\n",
                        js(site),
                        js(&api),
                        js(&src)
                    ),
                    vec![src],
                )
            }
            Provider::SimpleAnalytics => {
                required(site.is_empty() || valid_hostname_id(site), "Enter the hostname registered in Simple Analytics, or leave it blank.")?;
                let src = "https://scripts.simpleanalyticscdn.com/latest.js".to_string();
                let q = "https://queue.simpleanalyticscdn.com";
                let a: Vec<(&str, String)> = if site.is_empty() { vec![] } else { vec![("data-hostname", site.into())] };
                preset(
                    &["https://scripts.simpleanalyticscdn.com"],
                    &[q],
                    &[q],
                    format!("  load({}, {});\n", js(&src), attrs(&a)),
                    vec![src],
                )
            }
            Provider::Cloudflare => {
                required(
                    site.len() == 32 && site.bytes().all(|b| b.is_ascii_hexdigit() && !b.is_ascii_uppercase()),
                    "Cloudflare Web Analytics uses a 32-character lowercase hex token.",
                )?;
                let src = "https://static.cloudflareinsights.com/beacon.min.js".to_string();
                preset(
                    &["https://static.cloudflareinsights.com"],
                    &["https://cloudflareinsights.com"],
                    &[],
                    format!(
                        "  load({}, {});\n",
                        js(&src),
                        attrs(&[("data-cf-beacon", json!({"token": site, "spa": false}).to_string())])
                    ),
                    vec![src],
                )
            }
            Provider::Custom => match self.custom_kind {
                CustomKind::Url => {
                    let url = reqwest::Url::parse(self.script_url.trim()).map_err(|_| "Enter the script's absolute https URL.")?;
                    if url.scheme() != "https" || !url.username().is_empty() || url.password().is_some() || url.fragment().is_some() {
                        return Err("The script URL must be https, without credentials or a fragment.".into());
                    }
                    let origin = parse_origin(&origin_of(&url).ok_or("The script URL must be https.")?, false)?;
                    if !script_origins.iter().any(|s| origin_matches(s, &origin)) {
                        return Err(format!("Declare {origin} under script origins: the script is loaded from there."));
                    }
                    let src = url.to_string();
                    Built {
                        script_src: script_origins,
                        connect_src: connect_origins,
                        img_src: img_origins,
                        body: format!("  load({}, {{}});\n", js(&src)),
                        fetches: vec![src],
                        inline: false,
                    }
                }
                CustomKind::Inline => {
                    let snippet = self.snippet.trim();
                    if snippet.is_empty() {
                        return Err("Paste the provider's JavaScript snippet.".into());
                    }
                    if snippet.starts_with('<') || snippet.contains('\0') {
                        return Err("Paste only the JavaScript, without <script> tags, or use the script URL option.".into());
                    }
                    Built {
                        script_src: script_origins,
                        connect_src: connect_origins,
                        img_src: img_origins,
                        body: snippet.to_string(),
                        fetches: Vec::new(),
                        inline: true,
                    }
                }
            },
        })
    }

    /// The admin view. The snippet is returned to administrators who may read
    /// integrations: it is configuration, not a credential.
    pub fn public(&self) -> Value {
        json!(self)
    }
}

/// `base` with the plan's origins appended to `script-src`, `connect-src` and
/// `img-src`. A directive the base lacks is added. Nothing else changes.
pub fn widen(base: &str, plan: &Plan) -> String {
    let mut directives: Vec<String> = base
        .split(';')
        .map(str::trim)
        .filter(|d| !d.is_empty())
        .map(str::to_string)
        .collect();
    for (name, extra) in [
        ("script-src", &plan.script_src),
        ("connect-src", &plan.connect_src),
        ("img-src", &plan.img_src),
    ] {
        if extra.is_empty() {
            continue;
        }
        let add = extra.join(" ");
        match directives
            .iter_mut()
            .find(|d| d.split_whitespace().next() == Some(name))
        {
            Some(d) => {
                d.push(' ');
                d.push_str(&add);
            }
            None => directives.push(format!("{name} 'self' {add}")),
        }
    }
    directives.join("; ")
}

pub async fn load(store: &mw_store::Store) -> Result<Config, ()> {
    match store.get_setting(KEY).await.map_err(|_| ())? {
        Some(raw) => serde_json::from_str(&raw).map_err(|_| ()),
        None => Ok(Config::default()),
    }
}

// ── the mail shell ────────────────────────────────────────────────────────────

/// The shell's whole CSP when a tag is active. Set only by [`mail_shell`]; a
/// response extension cannot be forged by a request or an upstream.
#[derive(Clone)]
pub(crate) struct ShellCsp(pub HeaderValue);

/// What a user consents to: this provider running this exact loader.
pub fn consent_key(provider: Provider, plan: &Plan) -> String {
    let wire = serde_json::to_value(provider).expect("provider");
    format!("{}:{}", wire.as_str().expect("provider name"), plan.sha256)
}

/// A stored consent value: empty (withdrawn) or `<provider>:<64 hex>`.
pub fn valid_consent_key(value: &str) -> bool {
    value.is_empty()
        || value.split_once(':').is_some_and(|(provider, hash)| {
            serde_json::from_value::<Provider>(json!(provider)).is_ok()
                && hash.len() == 64
                && hash.bytes().all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
        })
}

/// Has `owner` consented to exactly `key`? Read from the same realm-scoped
/// preferences document as the counts consent, but a different field: agreeing
/// to server-side counts never implies a browser tag.
pub(crate) async fn tag_consent(store: &mw_store::Store, owner: &str, key: &str) -> bool {
    store
        .get_setting(&crate::prefs_routes::personal_storage::key("ui-preferences", owner))
        .await
        .ok()
        .flatten()
        .and_then(|raw| serde_json::from_str::<Value>(&raw).ok())
        .is_some_and(|value| {
            value["document"]["version"] == 1
                && value["document"]["preferences"]["analyticsTagConsent"] == key
        })
}

/// Does the request carry an admin console session cookie? The console shares
/// this origin, so a provider script in the mail shell could call the admin API
/// with that cookie: the tag is never served to such a browser.
fn has_admin_session(headers: &HeaderMap) -> bool {
    crate::hardening::cookie(headers, &mw_admin::config::session_cookie_name()).is_some()
}

/// The plan for this document request, when every gate passes: an enabled valid
/// tag, no DNT/GPC header, no admin session cookie, a signed-in mailbox session,
/// and that account's consent to this provider and loader hash.
pub(crate) async fn active_for(state: &AppState, headers: &HeaderMap) -> Option<(Plan, String)> {
    if crate::analytics_runtime::privacy_signal(headers) || has_admin_session(headers) {
        return None;
    }
    let config = load(&state.store).await.ok()?;
    let plan = config.plan().ok()??;
    let key = consent_key(config.provider, &plan);
    let session = crate::authed(state, headers).await.ok()?;
    let owner = crate::prefs_routes::personal_storage::owner(state, &session, headers, false)
        .await
        .ok()?;
    tag_consent(&state.store, &owner, &key).await.then_some((plan, key))
}

/// Insert the marker and the loader reference into the shell HTML.
pub fn inject(html: &str, base: &str, plan: &Plan, key: &str) -> String {
    let meta = format!(
        r#"<meta name="{META_NAME}" content="{}" data-consent="{key}">"#,
        plan.script_src.join(" ")
    );
    let tag = format!(
        r#"<script defer src="{base}/{}" data-mw-analytics></script>"#,
        plan.path()
    );
    let mut out = html.to_string();
    match out.find("<head>") {
        Some(i) => out.insert_str(i + "<head>".len(), &meta),
        None => out.insert_str(0, &meta),
    }
    // After every module entry, so the app's Trusted Types policy exists first:
    // deferred classic and module scripts run in document order.
    match out.rfind("</body>") {
        Some(i) => out.insert_str(i, &tag),
        None => out.push_str(&tag),
    }
    out
}

/// The mail shell with the tag, or `None` to serve the unchanged shell.
pub(crate) async fn mail_shell(state: &AppState, method: &Method, headers: &HeaderMap) -> Option<Response> {
    if method != Method::GET {
        return None;
    }
    let (plan, key) = active_for(state, headers).await?;
    let csp = HeaderValue::from_str(&widen(crate::CSP, &plan)).ok()?;
    let base: &str = &state.base_path;
    let bytes = crate::adapt_asset_to_base("index.html", crate::read_asset(state, "index.html")?, base);
    let html = String::from_utf8(bytes).ok()?;
    let mut response = crate::asset_response("index.html", "text/html", inject(&html, base, &plan, &key).into_bytes(), headers);
    let h = response.headers_mut();
    h.insert(header::CACHE_CONTROL, HeaderValue::from_static("private, no-cache"));
    h.insert(header::VARY, HeaderValue::from_static("cookie, dnt, sec-gpc"));
    // The service worker never stores a tagged shell (it keys on this header).
    h.insert("x-mw-analytics-tag", HeaderValue::from_static("1"));
    response.extensions_mut().insert(ShellCsp(csp));
    Some(response)
}

/// `GET assets/analytics/<sha256>.js`: the active loader, to a signed-in session,
/// only while its hash is the current one.
pub(crate) async fn asset(state: &AppState, path: &str, headers: &HeaderMap) -> Option<Response> {
    let name = path.strip_prefix(ASSET_PREFIX)?;
    let not_found = || Some(StatusCode::NOT_FOUND.into_response());
    let Some(hash) = name.strip_suffix(".js") else {
        return not_found();
    };
    if hash.len() != 64 || !hash.bytes().all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b)) {
        return not_found();
    }
    let Some(plan) = load(&state.store).await.ok().and_then(|c| c.plan().ok().flatten()) else {
        return not_found();
    };
    if plan.sha256 != hash || active_for(state, headers).await.is_none_or(|(p, _)| p.sha256 != hash) {
        return not_found();
    }
    Some(script_response(&plan))
}

pub fn script_response(plan: &Plan) -> Response {
    let mut response = plan.script.clone().into_response();
    let h = response.headers_mut();
    h.insert(header::CONTENT_TYPE, HeaderValue::from_static("text/javascript; charset=utf-8"));
    h.insert("x-content-type-options", HeaderValue::from_static("nosniff"));
    h.insert(header::CACHE_CONTROL, HeaderValue::from_static("private, max-age=31536000, immutable"));
    if let Ok(v) = HeaderValue::from_str(&format!("\"{}\"", plan.sha256)) {
        h.insert(header::ETAG, v);
    }
    response
}

// ── admin API ─────────────────────────────────────────────────────────────────

pub(crate) fn register(r: &mut impl AdminRegistrar) {
    r.admin_endpoint(Method::GET, "/analytics/tag", Capability::IntegrationsRead, read);
    r.admin_endpoint(Method::PUT, "/analytics/tag", Capability::IntegrationsWrite, save);
    r.admin_endpoint(Method::POST, "/analytics/tag/test", Capability::IntegrationsWrite, test);
}

/// A save: the [`Config`] fields plus the one-time acknowledgement, which is
/// never stored.
struct Update {
    config: Config,
    acknowledge_full_access: bool,
}
impl Update {
    fn parse(mut body: Value) -> Result<Self, AdminApiError> {
        let acknowledge_full_access = match body.as_object_mut().and_then(|o| o.remove("acknowledgeFullAccess")) {
            None => false,
            Some(Value::Bool(b)) => b,
            Some(_) => return Err(AdminApiError::invalid("invalidBody", "acknowledgeFullAccess is a boolean.", "/acknowledgeFullAccess")),
        };
        let config = serde_json::from_value(body)
            .map_err(|e| AdminApiError::invalid("invalidBody", e.to_string(), "/config"))?;
        Ok(Self { config, acknowledge_full_access })
    }
}

fn refusal_code(refusal: &mw_egress::Refusal) -> String {
    match refusal {
        mw_egress::Refusal::BadRequest(_) => "badRequest".into(),
        mw_egress::Refusal::Blocked => "blocked".into(),
        mw_egress::Refusal::Timeout => "timeout".into(),
        mw_egress::Refusal::Status(code) => format!("status {code}"),
        mw_egress::Refusal::Upstream => "upstream".into(),
        mw_egress::Refusal::TooLarge => "tooLarge".into(),
    }
}

fn preview_json(config: &Config) -> Value {
    match config.preview() {
        Ok(plan) => json!({
            "valid": true,
            "csp": {"scriptSrc": plan.script_src, "connectSrc": plan.connect_src, "imgSrc": plan.img_src},
            "path": format!("/{}", plan.path()),
            "sha256": plan.sha256,
            "bytes": plan.script.len(),
        }),
        Err(error) => json!({"valid": false, "error": error}),
    }
}

async fn view(state: &AppState) -> Result<Response, AdminApiError> {
    let config = load(&state.store).await.map_err(|_| AdminApiError::internal("analytics tag config"))?;
    let tag = etag_of(&json!(config));
    Ok(with_etag(
        (
            [("cache-control", "private, no-store")],
            Json(json!({"config": config.public(), "preview": preview_json(&config), "maxSnippetBytes": MAX_SNIPPET_BYTES})),
        ),
        &tag,
    ))
}

async fn read(State(state): State<AppState>, ctx: AdminCtx) -> Result<Response, AdminApiError> {
    ctx.require_unscoped(Capability::IntegrationsRead)?;
    view(&state).await
}

fn super_required() -> AdminApiError {
    AdminApiError::new(
        StatusCode::FORBIDDEN,
        "superAdminRequired",
        "Only a full Super admin can turn on or change a browser analytics tag.",
    )
}

async fn require_full_super(state: &AppState, ctx: &AdminCtx) -> Result<(), AdminApiError> {
    let id = ctx.account_id.as_deref().ok_or_else(super_required)?;
    let account = state.store.get_admin_account(id).await?.ok_or_else(super_required)?;
    if crate::admin_roles::is_full_super(state, &account).await? {
        Ok(())
    } else {
        Err(super_required())
    }
}

async fn save(
    State(state): State<AppState>,
    ctx: AdminCtx,
    headers: HeaderMap,
    AdminJson(body): AdminJson<Value>,
) -> Result<Response, AdminApiError> {
    ctx.require_unscoped(Capability::IntegrationsWrite)?;
    let update = Update::parse(body)?;
    let raw = state.store.get_setting(KEY).await.map_err(AdminApiError::internal)?;
    let previous = load(&state.store).await.map_err(|_| AdminApiError::internal("analytics tag config"))?;
    if !headers.contains_key("if-match") {
        return Err(AdminApiError::new(
            StatusCode::PRECONDITION_REQUIRED,
            "ifMatchRequired",
            "Reload the configuration before saving.",
        ));
    }
    check_if_match(&headers, &etag_of(&json!(previous)), None, None)?;
    let mut config = update.config;
    config.site_id = config.site_id.trim().to_string();
    config.host = config.host.trim().to_string();
    config.script_url = config.script_url.trim().to_string();
    let plan = config
        .plan()
        .map_err(|message| AdminApiError::invalid("invalidAnalyticsTag", message, "/config"))?;
    let previous_plan = previous.plan().ok().flatten();
    // Turning a tag on, or changing what an enabled tag loads or where it may
    // connect, is a decision about code that runs in every consenting user's mail.
    let changes_code = plan.as_ref().is_some_and(|p| previous_plan.as_ref() != Some(p));
    if changes_code {
        require_full_super(&state, &ctx).await?;
        if !update.acknowledge_full_access {
            return Err(AdminApiError::invalid(
                "acknowledgeFullAccess",
                "Confirm that this provider's script runs with full access to users' webmail pages, including the admin API of anyone signed in to the console in the same browser.",
                "/acknowledgeFullAccess",
            ));
        }
    }
    // Every save needs a fresh proof, after the cheaper refusals above.
    crate::step_up::require_admin(&state, &ctx, crate::step_up::StepUpAction::IntegrationManage).await?;
    let serialized = serde_json::to_string(&config).expect("analytics tag config");
    if !state
        .store
        .compare_and_set_setting(KEY, raw.as_deref(), &serialized)
        .await
        .map_err(AdminApiError::internal)?
    {
        return Err(AdminApiError::stale(None, None));
    }
    let detail = |config: &Config, plan: Option<&Plan>| {
        json!({
            "enabled": config.enabled,
            "provider": config.provider,
            "customKind": (config.provider == Provider::Custom).then_some(config.custom_kind),
            "scriptSha256": plan.map(|p| p.sha256.clone()),
            "scriptSrc": plan.map(|p| p.script_src.clone()),
            "connectSrc": plan.map(|p| p.connect_src.clone()),
            "imgSrc": plan.map(|p| p.img_src.clone()),
        })
    };
    crate::admin_accounts::audit(
        &state,
        &ctx,
        "analytics-tag-configured",
        "analytics:browser-tag",
        Some(detail(&previous, previous_plan.as_ref())),
        Some(detail(&config, plan.as_ref())),
    )
    .await;
    view(&state).await
}

/// Check that the script the form would load is reachable: each script URL is
/// fetched through the analytics egress class (so private and metadata addresses
/// stay refused), and a stored snippet reports the path it is served at.
async fn test(
    State(state): State<AppState>,
    ctx: AdminCtx,
    AdminJson(config): AdminJson<Config>,
) -> Result<Response, AdminApiError> {
    ctx.require_unscoped(Capability::IntegrationsWrite)?;
    let plan = config
        .preview()
        .map_err(|message| AdminApiError::invalid("invalidAnalyticsTag", message, "/config"))?;
    let served = load(&state.store)
        .await
        .ok()
        .and_then(|c| c.plan().ok().flatten())
        .is_some_and(|p| p.sha256 == plan.sha256);
    let mut results = Vec::new();
    if let Some(egress) = mw_egress::plan::current() {
        for url in &plan.fetches {
            let Ok(parsed) = reqwest::Url::parse(url) else { continue };
            let request = mw_egress::plan::Request::get(mw_egress::TrafficClass::Analytics)
                .accept("*/*")
                .response_limit(1024 * 1024)
                .user_agent(crate::analytics_adapter::USER_AGENT);
            let fetched = tokio::time::timeout(
                std::time::Duration::from_secs(5),
                mw_egress::plan::send_with(&egress, parsed, &request),
            )
            .await;
            results.push(match fetched {
                Err(_) => json!({"url": url, "ok": false, "error": "timeout"}),
                Ok(response) => match response.outcome {
                    Ok(bytes) => json!({"url": url, "ok": true, "bytes": bytes.len(), "contentType": response.content_type}),
                    Err(refusal) => json!({"url": url, "ok": false, "error": refusal_code(&refusal)}),
                },
            });
        }
    } else {
        for url in &plan.fetches {
            results.push(json!({"url": url, "ok": false, "error": "egress configuration is not loaded"}));
        }
    }
    Ok(Json(json!({
        "path": format!("/{}", plan.path()),
        "sha256": plan.sha256,
        "bytes": plan.script.len(),
        "served": served,
        "fetches": results,
    }))
    .into_response())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn cfg(provider: Provider, site: &str) -> Config {
        Config { enabled: true, provider, site_id: site.into(), ..Config::default() }
    }
    fn text(plan: &Plan) -> String {
        String::from_utf8(plan.script.clone()).unwrap()
    }

    #[test]
    fn disabled_by_default_and_disabled_adds_nothing() {
        assert!(!Config::default().enabled);
        assert_eq!(Config::default().plan().unwrap(), None);
        let mut off = cfg(Provider::Plausible, "mail.example.com");
        off.enabled = false;
        assert_eq!(off.plan().unwrap(), None);
    }

    #[test]
    fn each_preset_has_exact_csp_additions_and_loader() {
        let cases: Vec<(Config, &[&str], &[&str], &[&str], &[&str])> = vec![
            (cfg(Provider::Plausible, "mail.example.com"), &["https://plausible.io"], &["https://plausible.io"], &[],
                &[r#"load("https://plausible.io/js/script.js", {"data-domain": "mail.example.com", "data-api": "https://plausible.io/api/event"});"#]),
            (Config { host: "https://stats.example.org/".into(), ..cfg(Provider::Plausible, "mail.example.com") },
                &["https://stats.example.org"], &["https://stats.example.org"], &[],
                &[r#"load("https://stats.example.org/js/script.js""#]),
            (Config { host: "https://example.matomo.cloud/".into(), ..cfg(Provider::Matomo, "3") },
                &["https://example.matomo.cloud"], &["https://example.matomo.cloud"], &["https://example.matomo.cloud"],
                &[r#"q.push(["setTrackerUrl", "https://example.matomo.cloud/matomo.php"]);"#, r#"q.push(["setSiteId", "3"]);"#, r#"q.push(["disableCookies"]);"#, r#"load("https://example.matomo.cloud/matomo.js", {});"#]),
            (cfg(Provider::Umami, "F2997167-36cd-4e06-9054-abcc5c8cf476"), &["https://cloud.umami.is"], &["https://cloud.umami.is"], &[],
                &[r#"load("https://cloud.umami.is/script.js", {"data-website-id": "f2997167-36cd-4e06-9054-abcc5c8cf476", "data-host-url": "https://cloud.umami.is", "data-exclude-hash": "true", "data-exclude-search": "true"});"#]),
            (cfg(Provider::Fathom, "ABCDEFGH"), &["https://cdn.usefathom.com"], &["https://cdn.usefathom.com"], &["https://cdn.usefathom.com"],
                &[r#"load("https://cdn.usefathom.com/script.js", {"data-site": "ABCDEFGH"});"#]),
            (Config { region: "eu".into(), ..cfg(Provider::Posthog, "phc_abcdefghijklmnopqrstuvwxyz012345") },
                &["https://eu-assets.i.posthog.com", "https://eu.i.posthog.com"], &["https://eu.i.posthog.com", "https://eu-assets.i.posthog.com"], &[],
                &[r#"_i: [["phc_abcdefghijklmnopqrstuvwxyz012345", { api_host: "https://eu.i.posthog.com", disable_session_recording: true }, undefined]]"#, r#"load("https://eu-assets.i.posthog.com/static/array.js", {});"#]),
            (cfg(Provider::SimpleAnalytics, ""), &["https://scripts.simpleanalyticscdn.com"], &["https://queue.simpleanalyticscdn.com"], &["https://queue.simpleanalyticscdn.com"],
                &[r#"load("https://scripts.simpleanalyticscdn.com/latest.js", {});"#]),
            (cfg(Provider::Cloudflare, "0123456789abcdef0123456789abcdef"), &["https://static.cloudflareinsights.com"], &["https://cloudflareinsights.com"], &[],
                &[r#"load("https://static.cloudflareinsights.com/beacon.min.js", {"data-cf-beacon": "{\"spa\":false,\"token\":\"0123456789abcdef0123456789abcdef\"}"});"#]),
        ];
        for (config, script, connect, img, loader) in cases {
            let plan = config.plan().unwrap().unwrap();
            assert_eq!(plan.script_src, script, "{:?}", config.provider);
            assert_eq!(plan.connect_src, connect, "{:?}", config.provider);
            assert_eq!(plan.img_src, img, "{:?}", config.provider);
            let body = text(&plan);
            assert!(body.starts_with(PRESET_HEAD) && body.ends_with(PRESET_TAIL));
            for line in loader {
                assert!(body.contains(line), "{:?} loader lacks {line}:\n{body}", config.provider);
            }
            assert_eq!(plan.sha256, hex(&Sha256::digest(&plan.script)));
            assert_eq!(plan.path(), format!("assets/analytics/{}.js", plan.sha256));
        }
    }

    #[test]
    fn preset_fields_are_typed() {
        for bad in [
            cfg(Provider::Plausible, "not a domain"),
            cfg(Provider::Plausible, "\";alert(1);//"),
            Config { host: "http://plausible.io".into(), ..cfg(Provider::Plausible, "a.example") },
            Config { host: "https://p.example/?x=1".into(), ..cfg(Provider::Plausible, "a.example") },
            Config { host: "https://u:p@p.example".into(), ..cfg(Provider::Plausible, "a.example") },
            cfg(Provider::Matomo, "1"),
            Config { host: "https://m.example".into(), ..cfg(Provider::Matomo, "0") },
            cfg(Provider::Umami, "not-a-uuid"),
            cfg(Provider::Fathom, "abc"),
            cfg(Provider::Posthog, "key"),
            Config { region: "cn".into(), ..cfg(Provider::Posthog, "phc_abcdefghijklmnopqrstuvwxyz012345") },
            cfg(Provider::Cloudflare, "0123"),
            cfg(Provider::SimpleAnalytics, "bad host"),
        ] {
            assert!(bad.plan().is_err(), "{bad:?}");
        }
    }

    #[test]
    fn custom_origins_refuse_unsafe_sources() {
        for bad in [
            "http://stats.example.com",
            "*",
            "https://*",
            "https://*.com",
            "https://*.co.uk",
            "https://*.github.io",
            "https://a.*.example.com",
            "https://*example.com",
            "data:",
            "data:text/javascript,alert(1)",
            "blob:https://example.com/x",
            "javascript:alert(1)",
            "https://example.com/path",
            "https://example.com?q",
            "https://user@example.com",
            "https://example.com 'unsafe-inline'",
            "'unsafe-inline'",
            "'unsafe-eval'",
            "https://example.com;script-src *",
            "https://example.com,https://b.example",
            "https://exa mple.com",
            "https://example.com:0",
            "https://example.com:99999",
            "",
        ] {
            assert!(parse_origin(bad, true).is_err(), "{bad}");
        }
        assert_eq!(parse_origin("https://Stats.Example.com/", true).unwrap(), "https://stats.example.com");
        assert_eq!(parse_origin("https://stats.example.com:443", true).unwrap(), "https://stats.example.com");
        assert_eq!(parse_origin("https://127.0.0.1:8443", true).unwrap(), "https://127.0.0.1:8443");
        assert_eq!(parse_origin("https://*.example.com", true).unwrap(), "https://*.example.com");
        assert!(parse_origin("https://*.example.com", false).is_err());
        assert!(origin_matches("https://*.example.com", "https://a.example.com"));
        assert!(!origin_matches("https://*.example.com", "https://example.com"));
        assert!(!origin_matches("https://*.example.com", "https://a.example.com:8443"));
        assert!(!origin_matches("https://*.example.com", "https://evilexample.com"));
    }

    fn custom_url(url: &str, script: &[&str]) -> Config {
        Config {
            enabled: true,
            provider: Provider::Custom,
            custom_kind: CustomKind::Url,
            script_url: url.into(),
            script_origins: script.iter().map(|s| s.to_string()).collect(),
            ..Config::default()
        }
    }

    #[test]
    fn custom_tag_rules() {
        let ok = custom_url("https://cdn.example.com/t.js?id=7", &["https://cdn.example.com"]);
        let plan = ok.plan().unwrap().unwrap();
        assert_eq!(plan.script_src, ["https://cdn.example.com"]);
        assert!(text(&plan).contains(r#"load("https://cdn.example.com/t.js?id=7", {});"#));
        // The script origin must be declared.
        assert!(custom_url("https://cdn.example.com/t.js", &["https://other.example"]).plan().is_err());
        assert!(custom_url("http://cdn.example.com/t.js", &["https://cdn.example.com"]).plan().is_err());
        assert!(custom_url("javascript:alert(1)", &["https://cdn.example.com"]).plan().is_err());
        assert!(custom_url("data:text/javascript,1", &["https://cdn.example.com"]).plan().is_err());
        assert!(custom_url("https://cdn.example.com/t.js", &["http://cdn.example.com"]).plan().is_err());
        let mut many = ok.clone();
        many.connect_origins = (0..=MAX_ORIGINS).map(|i| format!("https://h{i}.example.com")).collect();
        assert!(many.plan().is_err());
        let inline = |snippet: String| Config {
            enabled: true,
            provider: Provider::Custom,
            custom_kind: CustomKind::Inline,
            snippet,
            connect_origins: vec!["https://collect.example.com".into()],
            ..Config::default()
        };
        let plan = inline("window.t = 1;".into()).plan().unwrap().unwrap();
        assert!(plan.script_src.is_empty());
        assert_eq!(plan.connect_src, ["https://collect.example.com"]);
        assert_eq!(text(&plan), format!("{INLINE_HEAD}window.t = 1;\n"));
        assert!(inline("<script>window.t=1</script>".into()).plan().is_err());
        assert!(inline(String::new()).plan().is_err());
        assert!(inline("a".repeat(MAX_SNIPPET_BYTES + 1)).plan().is_err());
        assert!(inline("a".repeat(MAX_SNIPPET_BYTES)).plan().is_ok());
        // Oversize is refused even while disabled: it is never stored.
        let mut off = inline("a".repeat(MAX_SNIPPET_BYTES + 1));
        off.enabled = false;
        assert!(off.plan().is_err());
    }

    #[test]
    fn widen_appends_only_declared_origins() {
        let plan = cfg(Provider::Fathom, "ABCDEFGH").plan().unwrap().unwrap();
        let widened = widen(crate::CSP, &plan);
        assert_eq!(
            widened,
            "default-src 'none'; script-src 'self' 'wasm-unsafe-eval' https://cdn.usefathom.com; \
             style-src 'self'; img-src 'self' blob: data: https://cdn.usefathom.com; font-src 'self'; manifest-src 'self'; \
             connect-src 'self' blob: https://cdn.usefathom.com; frame-src 'self'; worker-src 'self' blob:; \
             base-uri 'none'; form-action 'none'; require-trusted-types-for 'script'"
        );
        assert!(!widened.contains("unsafe-inline"));
        assert!(!widened.contains("'unsafe-eval'"));
        let none = Plan { script_src: vec![], connect_src: vec![], img_src: vec![], script: vec![], sha256: String::new(), fetches: vec![] };
        assert_eq!(widen(crate::CSP, &none), crate::CSP);
    }

    #[test]
    fn inject_marks_head_and_defers_after_entry() {
        let plan = cfg(Provider::Fathom, "ABCDEFGH").plan().unwrap().unwrap();
        let html = "<!doctype html><html><head><script type=\"module\" src=\"./assets/index.js\"></script></head><body><div id=\"root\"></div></body></html>";
        let key = consent_key(Provider::Fathom, &plan);
        assert_eq!(key, format!("fathom:{}", plan.sha256));
        let out = inject(html, "/mail", &plan, &key);
        assert!(out.contains(&format!(r#"<head><meta name="mw-analytics" content="https://cdn.usefathom.com" data-consent="{key}">"#)), "{out}");
        let tag = format!(r#"<script defer src="/mail/assets/analytics/{}.js" data-mw-analytics></script></body>"#, plan.sha256);
        assert!(out.contains(&tag), "{out}");
        assert!(out.find("index.js").unwrap() < out.find("data-mw-analytics").unwrap());
    }

    #[test]
    fn consent_keys_bind_provider_and_hash() {
        let plan = cfg(Provider::SimpleAnalytics, "").plan().unwrap().unwrap();
        let key = consent_key(Provider::SimpleAnalytics, &plan);
        assert!(key.starts_with("simple-analytics:"));
        assert!(valid_consent_key(&key));
        assert!(valid_consent_key(""));
        for bad in ["true", "plausible", "plausible:abc", "clarity:".to_string().as_str(), &format!("evil:{}", plan.sha256), &format!("plausible:{}", plan.sha256.to_uppercase())] {
            assert!(!valid_consent_key(bad), "{bad}");
        }
        // A different loader is a different consent.
        let other = cfg(Provider::SimpleAnalytics, "mail.example.com").plan().unwrap().unwrap();
        assert_ne!(consent_key(Provider::SimpleAnalytics, &other), key);
    }

    #[test]
    fn script_response_headers() {
        let plan = cfg(Provider::Fathom, "ABCDEFGH").plan().unwrap().unwrap();
        let response = script_response(&plan);
        let h = response.headers();
        assert_eq!(h["content-type"], "text/javascript; charset=utf-8");
        assert_eq!(h["x-content-type-options"], "nosniff");
        assert_eq!(h["cache-control"], "private, max-age=31536000, immutable");
        assert_eq!(h["etag"], format!("\"{}\"", plan.sha256));
    }

    #[test]
    fn update_refuses_unknown_fields() {
        let update = Update::parse(json!({"enabled":true,"provider":"plausible","siteId":"a.example","acknowledgeFullAccess":true})).ok().unwrap();
        assert!(update.acknowledge_full_access);
        assert!(Update::parse(json!({"enabled":true,"csp":"script-src *"})).is_err());
        assert!(Update::parse(json!({"acknowledgeFullAccess":"yes"})).is_err());
        assert!(serde_json::from_value::<Config>(json!({"provider":"clarity"})).is_err());
    }
}
