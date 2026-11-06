//! Anonymizing remote-image proxy + remote-image display grants + the tightened
//! shell CSP constant (t16 26.16 S7/S8/S10, plan §DQ3).
//!
//! # Why a proxy (S7/S8)
//! Email HTML embeds `<img src>` pointing at attacker-controlled hosts. Loading them
//! directly leaks the reader's IP, User-Agent, and open-time to the sender (the
//! classic tracking pixel) and can smuggle requests to internal services. Instead the
//! sanitizer strips remote images by default; when the user grants a scope
//! ([`Store::grant_remote_image`](mw_store::Store)) the web rewrites the granted
//! images to **`GET /api/image-proxy?url=…`**, and THIS server fetches them — so the
//! only host the reader's browser ever contacts is Mailwoman itself.
//!
//! # SSRF hardening (DQ3 — this fetches attacker-controlled URLs; treat as hostile)
//! The deny-by-default egress policy itself lives in the [`mw_egress`] crate as of
//! 26.20 (t22-e6) — it was `pub(crate)` here, which is why the workspace's other
//! egress surfaces could not reuse it. Nothing about it changed in the move:
//!   * scheme ∈ {`http`,`https`} only; URLs carrying credentials are refused;
//!   * DNS is resolved **once, by us**, and the fetch is PINNED to the resolved IP
//!     (reqwest `resolve`) so a name cannot rebind to a new address between our check
//!     and the connect (anti-DNS-rebinding). The client also sets `no_proxy` — an
//!     ambient `HTTP_PROXY`/`HTTPS_PROXY`/`ALL_PROXY` would otherwise hand the
//!     HOSTNAME to a proxy that resolves it itself, bypassing the pin
//!     ([`mw_egress::harden_client`]);
//!   * every resolved address is checked against [`mw_egress::ip_allowed`] —
//!     loopback, private, link-local (incl. the `169.254.169.254` cloud-metadata
//!     address), CGNAT, unique-local/link-local IPv6, multicast, unspecified and
//!     reserved ranges are REFUSED; IPv4-mapped IPv6 is unwrapped and re-checked;
//!   * redirects are NOT auto-followed — each hop's `Location` is re-parsed and
//!     re-validated through the same gate (a redirect to a private target is refused);
//!   * hard caps bound response size and the per-request timeout.
//!
//! One consequence of the move worth knowing before optimizing: this module no longer
//! builds an HTTP client at all — [`mw_egress::fetch_hop`] builds one per hop, because
//! the pin is `.resolve(host, addr)` baked into the builder and each redirect hop is a
//! different `(host, addr)` discovered only after the previous hop answers. A client
//! cannot be shared across hops without either dropping the pin for hops 2..n or
//! replacing the pinning mechanism with a custom resolver. Any such change belongs in
//! `mw-egress` and must keep the per-hop pin (t22-e7 P4, escalated there).
//!
//! What stays HERE is what is specific to serving remote images to a browser:
//!   * the global concurrency ceiling and the per-account token bucket;
//!   * fetched bytes are re-encoded through the wasm media jail
//!     ([`mw_render::media_jail::reencode_image`], t16-e5) to a metadata-stripped PNG
//!     before serving — a hostile codec never runs natively in this process;
//!   * re-encoded bytes are cached in memory under `(egress route, URL)`, bounded by
//!     **bytes** and by a TTL, and served with a content-hash `ETag` ([`ProxyCache`]);
//!   * the request originates upstream with a normalized `User-Agent` and no forwarded
//!     `Cookie`/`Referer`/`Authorization` (nothing from the browser is proxied).
//!
//! The proxy REQUIRES a session ([`crate::authed`]) so it is never an open relay.
//!
//! # The grants are enforced HERE, not only in the client (t22-e7 P3)
//! The four grant scopes gate what the client rewrites AND what this server will
//! fetch. Until 26.20 only the former was true: `proxy_image` required a session and
//! then fetched any URL, so any authenticated session was an anonymizing fetch relay
//! for arbitrary public URLs — rate-limited, but not scoped to a message the reader
//! had actually consented to load images for.
//!
//! Enforcing it needed the wire to carry what the scopes are keyed on. Every scope in
//! [`mw_store::Store::remote_image_allowed`] is MESSAGE context — `single` is the
//! message id, `per-sender` and `per-domain` come from that message's SENDER — and an
//! image URL's host (`cdn.example`) has no relation to a sender's domain, so nothing
//! in a bare `?url=` request could be resolved into any of them. `t22-e4` added
//! `&emailId=` (`apps/web/src/api/remote-images.ts::imageProxyUrl`, threaded from
//! `Reader.tsx`); [`grant_covers`] resolves it here.
//!
//! Deny-by-default, and the order is load-bearing — see [`grant_covers`] for the
//! decision and [`ungranted_response`] for what a refusal costs and reveals. The gate
//! sits ahead of the cache, so an ungranted session cannot read images a granted one
//! fetched.
//!
//! What it does NOT do: the scopes are per-account and per-message, not per-URL, so a
//! session holding a covering grant may still proxy any *public* URL it names under
//! that message's id. Narrowing the fetch to URLs that actually occur in the message
//! body would need the body at fetch time and is not attempted here.
//!
//! # Ownership
//! This module exposes [`image_proxy_router`]; `crate::lib` (t16-e10, chain link 3)
//! MOUNTS it and applies [`SHELL_CSP_TIGHTENED`] at the shell-CSP site — this module
//! does not edit `lib.rs`. It also re-exports the `mw-egress` items in-tree callers
//! reach for through this path ([`ip_allowed`], [`embedded_ipv4s`]), so
//! `sieve_sync.rs` was unchanged by the extraction.
//!
//! As of 26.20 t22-e14 it additionally owns the **egress route** for the whole
//! crate: [`proxy_route`] is the only production construction site of a
//! `mw_egress::proxy::ProxyRoute` in the workspace, [`load_plan`] is the only way
//! to obtain the live ones (as an egress plan, t32 e31b), and
//! [`fetch_url_hardened_routed`] is how `import_routes.rs`'s
//! `webcal://`/ICS fetch takes that route. Both hardened-fetch surfaces in this
//! crate therefore share one route lookup, one fail-closed dispatch and one audit
//! emission, rather than two implementations that can drift apart.

use std::collections::HashMap;
use std::sync::{Arc, Mutex, OnceLock};
use std::time::{Duration, Instant};

use axum::extract::{Query, State};
use axum::http::{HeaderMap, StatusCode, header};
use axum::response::{IntoResponse, Response};
use axum::routing::{get, post};
use axum::{Json, Router};
use serde::Deserialize;
use serde_json::json;
use sha2::{Digest, Sha256};

use mw_egress::plan::{self, EgressPlan, PlanRoute, Request, RouteScope};
use mw_egress::proxy::{ProxyAuth, ProxyRoute, ProxyScheme};
use mw_egress::{AllowDestination, AllowEntry, InternalAllowList, Refusal, TrafficClass};
use mw_store::EgressProxyRow;

use crate::AppState;

/// Signed remote-content URLs (t32 e31a). A child module in its own file, so no
/// `lib.rs` mount is needed; it becomes `remote_content.rs` at the crate root when
/// this module is renamed (plan R).
#[path = "remote_content.rs"]
pub(crate) mod remote_content;

// ── S10: tightened shell CSP (delivered here; applied at lib.rs:102 by e10) ──────

/// The tightened Content-Security-Policy for the SPA shell (SPEC §7.4, t16 S10).
/// Delivered as a constant so t16-e10 applies it in `lib.rs` with a one-line change,
/// keeping this milestone's single `lib.rs` editor on the chain.
///
/// Two changes vs the prior shell CSP:
///   * **`require-trusted-types-for 'script'`** — DOM-XSS sink injection must go
///     through a Trusted Types policy. No `trusted-types` allow-list directive is
///     added, so the SPA may keep naming its own policy; only the enforcement is
///     turned on.
///   * **`style-src 'self'`** — the `'unsafe-inline'` style source is dropped.
///
/// NOTE for e10 / e-e2e: dropping style `'unsafe-inline'` blocks inline `style="…"`
/// attributes the SPA framework may emit. If the shell renders broken under this
/// value, the minimal fallback that still satisfies S10's intent is to re-admit
/// inline styles for the *attribute* sink ONLY (`style-src-elem 'self'; style-src-attr
/// 'unsafe-inline'`) rather than restoring the blanket `'unsafe-inline'`. Verify in
/// the live web gate before release. The per-message body CSP (`MESSAGE_CSP`) is
/// unaffected by this constant.
pub(crate) const SHELL_CSP_TIGHTENED: &str = "default-src 'none'; \
     script-src 'self' 'wasm-unsafe-eval'; style-src 'self'; \
     img-src 'self' blob: data:; font-src 'self'; manifest-src 'self'; \
     connect-src 'self' blob:; frame-src 'self'; worker-src 'self' blob:; \
     base-uri 'none'; form-action 'none'; require-trusted-types-for 'script'";

// ── fetch caps ───────────────────────────────────────────────────────────────
//
// The size cap, the per-hop timeout and the redirect-hop ceiling moved to
// `mw_egress` with the fetch they bound (`MAX_IMAGE_BYTES`, `FETCH_TIMEOUT`,
// `MAX_REDIRECTS`). The two below are the image proxy's own.

/// Global concurrent-fetch ceiling — bounds proxy load + upstream fan-out.
const MAX_CONCURRENT: usize = 128;

// ── re-encoded-image cache defaults (t22-e7 P2) ────────────────────────────────
//
// The cache is bounded by BYTES as well as entries: the cached value is the
// re-encoded PNG, whose size is a function of the decoded image, not of the
// source. The live bounds come from the policy (`privacy.image*` keys) and are
// enforced together by [`tier::ImageTier`]; the constants below are the manual-mode
// defaults.

/// Total re-encoded RAM bytes in manual mode unless configured.
pub(crate) const MAX_CACHE_BYTES: usize = 32 * 1024 * 1024;
/// How long a cached re-encode stays servable. Bounds both staleness (the upstream
/// bytes behind a URL can change) and retention of an image no one is reading any
/// more; the `Cache-Control` we hand the browser is separate and unaffected.
pub(crate) const CACHE_TTL: Duration = Duration::from_secs(3600);

/// The cache-key route token for a **direct** fetch — no egress route configured.
///
/// The route is the first half of the cache key because the cache is GLOBAL across
/// accounts: with a URL-only key, bytes fetched over one egress path could be served
/// to a request that was supposed to take another. t22-e7 built that in while the
/// route was still constant; as of t22-e14 it stops being constant, and
/// the plan's selected route supplies a real route id here whenever one applies.
///
/// It is a reserved token rather than an empty string so a direct entry and a
/// configured route are always distinguishable in the key space. An operator route
/// literally named `direct` would collide — the consequence is that images cached
/// before that route was activated could be served after, which is the same
/// staleness [`CACHE_TTL`] already bounds, so it is not defended against further.
const DIRECT_ROUTE: &str = "direct";

// ── the configured egress route (t22-e14) ─────────────────────────────────────
//
// Everything below exists so that a route saved in `/admin/egress` actually routes
// something. Until 26.20 the transport, the config store, the admin API and the
// admin UI had all landed and NOTHING consumed a configured route: saving one routed
// nothing at all.

/// Map a stored route onto the transport's [`ProxyRoute`].
///
/// # 🔒 THIS IS THE ONLY PRODUCTION CONSTRUCTION SITE OF A `ProxyRoute` IN THE
/// # WORKSPACE. A SECOND ONE IS A SECURITY DECISION, NOT A REFACTOR.
///
/// [`ProxyRoute`]'s fields are all `pub` and its `host` is deliberately **exempt**
/// from the SSRF address policy, because an egress proxy on loopback or RFC1918 (a
/// Squid, a Tor SOCKS port) is the normal operator deployment. Compare
/// [`mw_egress::Target`], whose `pub` fields carry the same hazard and whose risk is
/// **discharged** by `tunnel_fetch_hop` re-applying `ip_allowed` unconditionally to
/// whatever it is handed.
///
/// **`ProxyRoute` has no equivalent discharge, and cannot have one** — the whole
/// point of the carve-out is that its host is *not* address-checked. So the only
/// thing standing between a request-derived host and an unchecked dial is that
/// nothing request-derived ever builds one. That is a property of **how many
/// constructors exist and what they take**, not of any check inside this function,
/// and it is one commit away from being false at any time (t22-e-sec).
///
/// Hence the shape, which is the enforcement:
///   * the only input is an [`EgressProxyRow`], which comes only from
///     [`load_plan`] → `Store::live_egress_routes(&self)` or from [`route_by_id`];
///   * this function is **private**, and [`load_plan`] is `pub(crate)` and takes
///     only a store handle, so no caller anywhere can supply the fields;
///   * `ProxyRoute` derives no `Deserialize`, `FromStr` or `TryFrom`, so there is no
///     deserialisation path into one either.
///
/// `crates/mw-egress/tests/route_construction_sites.rs` **enforces this by scanning
/// the tree**: it fails if a second production construction site appears, or if any
/// fully-public function returns a `ProxyRoute`. A comment saying "there is one
/// constructor today" would not notice the second one; that test does.
///
/// # Why this function asserts nothing about its input
/// Stated rather than left as an omission. There is no check available here that
/// would mean anything: a `host` is a string, and "came from the operator" is not a
/// property of the value — an attacker-chosen host and an operator-chosen one are
/// the same bytes. Validating the string would produce a check that always passes
/// and reads as though it protects something. The provenance is carried by the type
/// of the argument and by the absence of other constructors, which is checkable; a
/// runtime assertion here would not be.
///
/// # Behaviour
/// The stored kind and flags map onto the transport's kinds (t32 e31b):
///
/// | `kind` | `tls_to_proxy` | `remote_dns` | transport |
/// |---|---|---|---|
/// | `http` | off | off | `HttpConnect` |
/// | `http` | on | off | `HttpsConnect` (TLS to the proxy) |
/// | `socks5` | off | off | `Socks5` |
/// | `socks5h` | off | on, **and an acceptance is recorded** | `Socks5h` (remote DNS) |
///
/// Every other combination returns `None`: an `ssh` route or a chained second hop
/// (not carried by this build yet), a `socks5h` route whose remote-DNS acceptance
/// is missing or was cleared by an edit, and the inconsistent ones. The caller turns
/// `None` into a refusal rather than into a direct fetch — and never into a
/// different kind: carrying a route without what it asked for (TLS to the proxy,
/// say) would send the credential in the clear, which is what the operator switched
/// it on to prevent. See [`load_plan`].
///
/// The password travels from the sealed column into [`ProxyAuth`], which redacts it
/// in `Debug`; `EgressProxyRow` redacts it too, so there is no point on this path
/// where a formatted value carries the credential.
fn proxy_route(row: &EgressProxyRow) -> Option<ProxyRoute> {
    let scheme = match (row.kind.as_str(), row.tls_to_proxy, row.remote_dns) {
        ("http", false, false) => ProxyScheme::HttpConnect,
        ("http", true, false) => ProxyScheme::HttpsConnect,
        ("socks5", false, false) => ProxyScheme::Socks5,
        // The one kind that hands the proxy a name exists only behind the
        // recorded acceptance (`Store::accept_egress_remote_dns_risk`).
        ("socks5h", false, true) if row.remote_dns_risk_accepted_by.is_some() => {
            ProxyScheme::Socks5h
        }
        _ => return None,
    };
    if row.chain_next_id.is_some() {
        return None;
    }
    Some(ProxyRoute {
        id: row.id.clone(),
        scheme,
        host: row.host.clone(),
        port: row.port,
        // A username with no password is still a credential pair the proxy may
        // accept; a row with neither is an unauthenticated route.
        auth: (!row.username.is_empty() || row.password.is_some()).then(|| ProxyAuth {
            username: row.username.clone(),
            password: row.password.clone().unwrap_or_default(),
        }),
        allow_plaintext: row.allow_plaintext,
    })
}

/// The deployment's egress plan, built from the store: every live route with its
/// scope and classes, and the internal-destination allowlist (t32 e31b).
///
/// # This function takes a store handle and nothing else
/// The routes come from `Store::live_egress_routes(&self)` and the allowlist from
/// `Store::list_egress_internal_allow(&self)`; neither takes a request-shaped
/// argument. [`ProxyRoute::host`] is deliberately **exempt** from the SSRF address
/// policy, which is safe only while no request can construct or choose a host.
/// Which of the plan's routes a request takes is decided later, by
/// `EgressPlan::selected_route(class, domain)`: a traffic-class constant and the
/// signed-in account's domain, selecting among these operator rows — never a
/// destination or a header.
///
/// # `Err` means "do not fetch", never "fetch directly"
/// A store failure, a live route this build cannot carry ([`proxy_route`] says
/// `None`), a corrupt scope or class, or two live routes answering the same class
/// in one scope all return `Err(())`, and the caller refuses. Falling back to a
/// direct fetch would mean a transient database error silently disables the
/// operator's egress control.
///
/// An allowlist row that does not parse is **skipped** with a warning instead: a
/// missing entry only narrows what `webhooks`/`siem` may reach, which is the safe
/// direction, and one bad row must not stop all egress.
///
/// `pub(crate)` rather than `pub`: see `route_construction_sites.rs`.
pub(crate) async fn load_plan(store: &mw_store::Store) -> Result<EgressPlan, ()> {
    let rows = store.live_egress_routes().await.map_err(|e| {
        tracing::warn!("egress route lookup failed: {e}");
    })?;
    let mut routes = Vec::with_capacity(rows.len());
    for row in &rows {
        // Never render the row: it holds the credential, and these are error paths,
        // which is the exit people forget.
        let Some(route) = proxy_route(row) else {
            tracing::warn!(
                "egress route {} (kind {:?}) cannot be carried by this build as configured — \
                 refusing rather than egressing directly",
                row.id,
                row.kind
            );
            return Err(());
        };
        let Some(scope) = RouteScope::parse(&row.scope) else {
            tracing::warn!("egress route {} has an unreadable scope", row.id);
            return Err(());
        };
        // Older/configuration writers may name an alias. Treat it as the same
        // namespace; conflicting alias/canonical routes fail plan validation.
        let scope = match scope {
            RouteScope::Domain(domain) => RouteScope::Domain(
                store
                    .canonical_policy_domain(&domain)
                    .await
                    .map_err(|_| ())?,
            ),
            RouteScope::Global => RouteScope::Global,
        };
        let mut classes = Vec::with_capacity(row.traffic_classes.len());
        for c in &row.traffic_classes {
            let Some(class) = TrafficClass::parse(c) else {
                tracing::warn!("egress route {} lists an unknown traffic class", row.id);
                return Err(());
            };
            classes.push(class);
        }
        routes.push(PlanRoute {
            scope,
            classes,
            route,
        });
    }

    let allow_rows = store.list_egress_internal_allow().await.map_err(|e| {
        tracing::warn!("egress allowlist lookup failed: {e}");
    })?;
    let mut entries = Vec::with_capacity(allow_rows.len());
    for row in &allow_rows {
        let entry = TrafficClass::parse(&row.traffic_class)
            .ok_or("unknown class")
            .and_then(|class| {
                AllowDestination::parse(&row.destination)
                    .and_then(|dest| AllowEntry::new(class, dest))
            });
        match entry {
            Ok(e) => entries.push(e),
            Err(why) => tracing::warn!("egress allowlist entry {} skipped: {why}", row.id),
        }
    }

    EgressPlan::new(routes, InternalAllowList::new(entries)).map_err(|e| {
        tracing::warn!("egress plan refused: {e}");
    })
}

/// [`load_plan`], then install it for this process (`mw_egress::plan::install`), so
/// crates without a store handle (`mw-crypto`, `mw-autoconfig`, `mw-admin`) send
/// under the same routes. On `Err` the installed plan is left as it was.
///
/// A plan that differs from the installed one also clears the remembered fetch
/// failures (t56 negative cache): they were measured under the old routes, and
/// an admin who fixes a route expects the next fetch to try it.
pub(crate) async fn refresh_plan(store: &mw_store::Store) -> Result<Arc<EgressPlan>, ()> {
    let next = load_plan(store).await?;
    let changed = plan::current().is_none_or(|installed| *installed != next);
    let installed = plan::install(next);
    if changed {
        remote_content::fetch_limits::forget_all();
    }
    Ok(installed)
}

/// How often every process re-reads the egress plan (R-CFG 6: hot settings apply
/// within 30 s on every process). The admin API also refreshes it at once after a
/// change, on the process that served the change.
const PLAN_REFRESH_EVERY: std::time::Duration = std::time::Duration::from_secs(15);

/// Install the plan now and keep it fresh. Called once at start
/// (`admin_apply::boot`), before the server accepts requests, because a process
/// with no installed plan refuses every class request.
///
/// If the first read fails, an empty plan is **not** installed in its place: the
/// process keeps refusing class requests until a read succeeds, rather than sending
/// direct around routes it could not read. A later failed read keeps the last good
/// plan (still the operator's routes) and logs.
pub(crate) async fn start_plan_refresh(store: mw_store::Store) {
    history::ensure_sampler();
    if refresh_plan(&store).await.is_err() {
        tracing::warn!(
            "egress plan not installed at start; class requests are refused until it loads"
        );
    }
    tokio::spawn(async move {
        let mut tick = tokio::time::interval(PLAN_REFRESH_EVERY);
        tick.tick().await;
        loop {
            tick.tick().await;
            let _ = refresh_plan(&store).await;
        }
    });
}

/// One stored route by id, mapped for the transport, or `Ok(None)` when no row has
/// that id. For the admin "test this route" endpoint
/// (`egress_admin::test_proxy`), which probes a named route that is usually not the
/// active one.
///
/// `id` is request-shaped, and this is the one factory allowed to take such a
/// parameter: it only **selects among rows the operator created** through the admin
/// API, so the `host` that reaches [`ProxyRoute::host`] is operator configuration
/// whichever row is chosen. It cannot introduce a host. That argument is recorded
/// as the named `fn route_by_id` exception in
/// `crates/mw-egress/tests/route_construction_sites.rs`; the literal is still built
/// only by [`proxy_route`].
///
/// Nothing that serves mail traffic may call this — those paths take the plan from
/// [`load_plan`], which has no parameter to steer. `Err(())` has the meaning it has
/// there: the route could not be loaded or mapped, so do not fetch.
#[rustfmt::skip] // one line: `route_construction_sites.rs` reads a signature per line
pub(crate) async fn route_by_id(store: &mw_store::Store, id: &str) -> Result<Option<ProxyRoute>, ()> {
    match store.get_egress_proxy(id).await {
        Ok(None) => Ok(None),
        Ok(Some(row)) => match proxy_route(&row) {
            Some(route) => Ok(Some(route)),
            None => {
                tracing::warn!(
                    "egress route {} (kind {:?}) cannot be carried by this build as configured",
                    row.id,
                    row.kind
                );
                Err(())
            }
        },
        Err(e) => {
            tracing::warn!("egress route lookup failed: {e}");
            Err(())
        }
    }
}

/// The last `(configured route, actually traversed)` pair this replica observed, so
/// an audit row is written when egress **changes** rather than once per image.
static LAST_EGRESS: OnceLock<Mutex<Option<(String, bool)>>> = OnceLock::new();

fn last_egress() -> &'static Mutex<Option<(String, bool)>> {
    LAST_EGRESS.get_or_init(|| Mutex::new(None))
}

/// Record what egress actually did, when it changes.
///
/// # Why on transition and not per fetch
/// `audit_log` is append-only **by design** — `mw-store`'s `v6.rs` has no update or
/// delete method — so a row written per proxied fetch is a row per remote image,
/// kept for the life of the deployment. Opening one rich mailbox would write dozens.
/// The operationally useful facts are the transitions (*egress began traversing
/// `corp`*; *`corp` stopped being traversed*), and those are what this emits.
///
/// De-duplication is **per replica**, exactly as the image proxy's rate limiter
/// already is: a second replica writes its own first row. That is stated rather than
/// implied because a reader counting rows across a cluster would otherwise
/// mis-interpret the gaps.
///
/// # `traversed_proxy` is a measurement, `configured_route` is an intention
/// The two are separate fields on purpose. `traversed` comes from
/// [`mw_egress::RoutedFetch::traversed_proxy`], which the transport sets from its own
/// progress and carries on the error paths too — so a row written after a **failed**
/// fetch still truthfully says the bytes did not go through the proxy. A row that
/// claimed traversal because a route was configured would be intent recorded as
/// fact, and it would be wrong in precisely the situation audit rows exist for.
///
/// # `traversedProxy: false` does NOT mean "went direct" — read both fields
/// A `CONNECT` the proxy **refuses** sends our bytes to that proxy and still reports
/// `traversed: false`, because no origin traffic traversed it. That is the right
/// value, and it is also the one an operator is most likely to misread — a refused
/// tunnel is exactly when they are trying to work out whether their proxy is in the
/// path at all, and "false" invites the conclusion that the fetch went around it
/// (t22-e-sec).
///
/// The row is readable only because **both** fields are on it, and they separate the
/// three states unambiguously:
///
/// | `configuredRoute` | `traversedProxy` | what happened |
/// |---|---|---|
/// | `"direct"` | `false` | no route configured; the fetch went direct |
/// | `"corp"` | `true`  | the fetch went through `corp` |
/// | `"corp"` | `false` | `corp` was live and was **not** traversed — the fetch was **refused**, not re-routed |
///
/// The third row can never mean "went direct", because `plan::send_with` is
/// fail-closed: once a route is configured there is no path to the direct fetcher.
/// `DIRECT_ROUTE` is a reserved token precisely so the first and third rows are
/// distinguishable by value rather than by inference.
///
/// The stop transition therefore fires on **failure** as well as success: this is
/// called before the refusal is returned, and the de-duplication key includes
/// `traversed`, so a route going `true` → `false` because its tunnel broke writes a
/// row. That is the single most useful row in the file and the easiest to lose to a
/// de-dup that only updates on success.
async fn audit_egress_transition(state: &AppState, configured_route: &str, traversed: bool) {
    let observed = (configured_route.to_string(), traversed);
    {
        // Scoped so the lock is never held across an await.
        let mut last = last_egress().lock().expect("egress transition lock");
        if last.as_ref() == Some(&observed) {
            return;
        }
        *last = Some(observed.clone());
    }
    let entry = mw_admin::AuditEvent::new(
        "egress",
        mw_admin::ActorKind::System,
        mw_admin::AuditKind::SecurityPolicyChanged,
    )
    .target(configured_route)
    .detail(json!({
        "configuredRoute": configured_route,
        "traversedProxy": traversed,
    }))
    .into_entry();
    let row = mw_store::AuditRow {
        id: entry.id,
        ts: entry.ts,
        actor: entry.actor,
        actor_kind: "system".to_string(),
        action: entry.action,
        target: entry.target,
        detail_json: entry.detail_json,
        ip: entry.ip,
    };
    if let Err(e) = state.store.append_audit(&row).await {
        tracing::warn!("egress transition audit append failed: {e}");
    }
}

/// [`fetch_url_hardened`] over the deployment's egress route, with the same
/// fail-closed behaviour and the same audit row as the image proxy's own fetch.
///
/// This exists so `import_routes.rs`'s `webcal://`/ICS subscription fetcher — the
/// workspace's **second** hardened-fetch surface, and the *more*
/// attacker-influenceable of the two, since the user supplies the URL — takes the
/// configured route too. Wiring the image proxy and not this one would give an
/// operator an egress control that is honoured for remote images and **silently
/// bypassed for calendar subscriptions**, which is worse than not shipping the
/// feature: they would believe their egress is controlled.
///
/// It is a function here rather than a second copy over there for the same reason
/// `import_routes.rs` reaches for [`fetch_url_hardened`] instead of hand-rolling a
/// fetcher: the route lookup, the fail-closed dispatch and the audit emission are
/// one implementation, so the two surfaces cannot drift into behaving differently.
///
/// The `String` error shape and every message are [`fetch_url_hardened`]'s
/// unchanged, so the ICS caller's existing `502` bodies are byte-identical for every
/// refusal it could already produce. A proxy failure arrives as `Refusal::Upstream`
/// → *"upstream fetch failed"*, which is the fail-closed refusal: visible, and
/// carrying nothing about internal reachability.
pub(crate) async fn fetch_url_hardened_routed(
    state: &AppState,
    url_str: &str,
    accept: &str,
) -> Result<Vec<u8>, String> {
    let plan = load_plan(&state.store)
        .await
        .map_err(|()| "egress route is not usable".to_string())?;
    let route_id = plan
        .selected_route(TrafficClass::IcsFeeds, None)
        .map_or(DIRECT_ROUTE, |r| r.id.as_str());
    let url = reqwest::Url::parse(url_str).map_err(|_| "malformed URL".to_string())?;
    let routed = plan::send_with(
        &plan,
        url,
        &Request::get(TrafficClass::IcsFeeds).accept(accept),
    )
    .await;
    audit_egress_transition(state, route_id, routed.traversed_proxy).await;
    routed.outcome.map_err(|r| match r {
        Refusal::BadRequest(m) => m.to_string(),
        Refusal::Blocked => "target address is not permitted".to_string(),
        Refusal::Timeout => "upstream timed out".to_string(),
        // Matched to `fetch_url_hardened`'s arm CHARACTER FOR CHARACTER, including
        // t22-e11's `HTTP {code}` suffix. This is a transport swap, not a change of
        // what the ICS caller reports, and a message that drifted here would be a
        // behaviour change smuggled in under a wiring commit. `is_upstream_failure`
        // is not used: it would collapse the two arms and lose the code that the
        // existing text carries.
        Refusal::Status(code) => format!("upstream fetch failed: HTTP {code}"),
        Refusal::Upstream => "upstream fetch failed".to_string(),
        Refusal::TooLarge => "upstream response too large".to_string(),
    })
}

// ── router ───────────────────────────────────────────────────────────────────

/// The image-proxy + remote-image-grant routes (mounted by t16-e10). Every route is
/// session-authed; the proxy fetch is additionally SSRF-gated.
pub(crate) fn image_proxy_router() -> Router<AppState> {
    Router::new()
        .route("/api/image-proxy", get(proxy_image))
        .route("/api/remote-images/grants", get(list_grants))
        .route("/api/remote-images/grant", post(grant))
        .route("/api/remote-images/revoke", post(revoke))
        // Signed remote-content URLs for the sandboxed body frame (t32 e31a).
        .route("/api/rc/sign", post(remote_content::sign))
        .route("/api/rc/batch", post(remote_content::image_batch::batch))
        .route("/api/rc/stats", get(remote_content::message_stats))
        .route("/api/rc/policy", get(crate::remote_policy::get))
        .route("/api/rc/frame", get(crate::remote_policy::frame))
        .merge(signed_content_router())
}

/// The capability-signed token routes the opaque-origin body frame loads.
///
/// Every answer on them, **whatever its status**, carries
/// `Cross-Origin-Resource-Policy: cross-origin` (t56). The frame is
/// `sandbox=""`, so its origin is opaque and therefore cross-origin to this
/// server; the global hardening layer stamps `same-origin` on any response that
/// does not set its own CORP. The 200 image already set `cross-origin`, but every
/// refusal (an expired or revoked link 403, a 429, a 502/504 upstream failure, a
/// 415) went out `same-origin`, so the browser reported a CORP block instead of the
/// real status and the reader could not tell a timeout from a misconfiguration.
/// Those bodies are short fixed strings and a token is the authorisation, so
/// letting the frame see them reveals nothing a same-origin caller cannot. CORP
/// stays `same-origin` on every other route, `/api/rc/sign` and `/api/rc/stats`
/// included.
fn signed_content_router() -> Router<AppState> {
    Router::new()
        .route("/api/rc/i/{token}", get(remote_content::image))
        .route("/api/rc/p/{token}", get(remote_content::part))
        .route("/api/rc/f/{token}", get(remote_content::file))
        .route("/api/rc/d/{token}", get(remote_content::image_download))
        .layer(axum::middleware::map_response(signed_content_corp))
}

async fn signed_content_corp(mut response: Response) -> Response {
    let h = response.headers_mut();
    h.insert(
        "cross-origin-resource-policy",
        axum::http::HeaderValue::from_static("cross-origin"),
    );
    h.insert(
        header::X_CONTENT_TYPE_OPTIONS,
        axum::http::HeaderValue::from_static("nosniff"),
    );
    // A token answer is never a reason for the browser to store a cookie.
    h.remove(header::SET_COOKIE);
    response
}

// ── SSRF refusal → HTTP ────────────────────────────────────────────────────────

/// Map an [`mw_egress::Refusal`] onto the wire response, unchanged from when the
/// enum lived here and carried its own `IntoResponse`: the same status for the same
/// discriminant and the same body text. It is a free function rather than a trait
/// impl because both the type and `IntoResponse` are now foreign to this crate; the
/// HTTP mapping is this module's concern in either case, since `mw-egress` serves no
/// HTTP and must not link axum.
///
/// Every arm maps to a client error or bad-gateway — an image request never reveals
/// internal reachability beyond a coarse status + reason.
fn refusal_response(refusal: Refusal) -> Response {
    let (code, msg) = match refusal {
        Refusal::BadRequest(m) => (StatusCode::BAD_REQUEST, m),
        Refusal::Blocked => (StatusCode::FORBIDDEN, "target address is not permitted"),
        Refusal::Timeout => (StatusCode::GATEWAY_TIMEOUT, "upstream timed out"),
        // The upstream status is DISCARDED here, deliberately, and this arm must
        // stay `_`. `mw_egress::Refusal::Status` carries the code because
        // `mw-crypto` renders a 404 keyserver lookup as "no key published for that
        // lookup"; forwarding it *here* would make an image request a reachability
        // oracle for internal URLs — the caller could distinguish 404 from 403 from
        // 500 on a host it cannot otherwise see — through the one endpoint whose
        // design goal is to reveal nothing, quietly undoing `Refusal::Blocked`'s
        // deliberate coarseness. Every status therefore collapses to the same 502
        // this module returned before the variant existed.
        Refusal::Status(_) => (StatusCode::BAD_GATEWAY, "upstream fetch failed"),
        Refusal::Upstream => (StatusCode::BAD_GATEWAY, "upstream fetch failed"),
        Refusal::TooLarge => (StatusCode::BAD_GATEWAY, "upstream image too large"),
    };
    (code, msg).into_response()
}

// ── content-hash cache ─────────────────────────────────────────────────────────

/// What a cached re-encode is filed under: the egress route that produced the bytes
/// ([`DIRECT_ROUTE`] today) and the requested URL. Never the account — the cache is
/// deliberately shared, since the bytes are a public resource fetched with nothing of
/// the reader's attached, and a per-account cache would multiply the resident set by
/// the account count for identical images. `owner` and `ctx` are NOT part of the key:
/// they only tell the tier which account's RAM share an insert counts against and
/// how the request relates to an open message, for scoring.
///
/// The cache itself is the two-tier [`tier::ImageTier`]; handlers authenticate and
/// check the display grant first, and only then build a key and look it up.
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
struct CacheKey {
    route: String,
    url: String,
    owner: String,
    ctx: tier::Context,
}

impl CacheKey {
    fn new(route: &str, url: &str) -> Self {
        Self {
            route: route.to_string(),
            url: url.to_string(),
            owner: String::new(),
            ctx: tier::Context::Background,
        }
    }

    /// Attribute the request: `owner` is the authorised account, and a request that
    /// names a message is `Open`.
    fn by(mut self, owner: &str, has_message: bool) -> Self {
        self.owner = owner.to_string();
        self.ctx = if has_message {
            tier::Context::Open
        } else {
            tier::Context::Background
        };
        self
    }
}

static CACHE_HITS: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
static CACHE_MISSES: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);

/// Count a served cache hit (`true`) or a fetch that had to go upstream.
pub(crate) fn record_cache(hit: bool) {
    use std::sync::atomic::Ordering::Relaxed;
    let (counter, other) = if hit {
        (&CACHE_HITS, &CACHE_MISSES)
    } else {
        (&CACHE_MISSES, &CACHE_HITS)
    };
    let n = counter.fetch_add(1, Relaxed) + 1;
    // Halve both once the window is large, so the rate follows recent traffic.
    if n > 10_000 {
        counter.store(n / 2, Relaxed);
        other.store(other.load(Relaxed) / 2, Relaxed);
    }
}

/// Recent hit rate in percent; `None` before 20 lookups.
pub(crate) fn hit_percent() -> Option<u8> {
    use std::sync::atomic::Ordering::Relaxed;
    let (h, m) = (CACHE_HITS.load(Relaxed), CACHE_MISSES.load(Relaxed));
    (h + m >= 20).then(|| (h * 100 / (h + m)) as u8)
}

#[path = "remote_image_tier.rs"]
pub(super) mod tier;

#[path = "remote_image_adaptive.rs"]
pub(super) mod adaptive;
#[path = "remote_image_history.rs"]
pub(crate) mod history;
#[path = "remote_image_latency.rs"]
pub(crate) mod latency;
#[path = "remote_image_capacity.rs"]
pub(super) mod capacity;
#[path = "remote_image_demand.rs"]
pub(super) mod demand;
#[path = "remote_image_resources.rs"]
pub(super) mod resources;
#[path = "remote_image_slots.rs"]
pub(super) mod slots;

/// Stable capacity is resized in place: a hot reduction drains existing work.
fn fetch_semaphore() -> &'static Arc<capacity::Capacity> {
    static SEM: OnceLock<Arc<capacity::Capacity>> = OnceLock::new();
    SEM.get_or_init(|| capacity::Capacity::new(MAX_CONCURRENT))
}

/// Respect the replica's configured network ceiling while allowing a bounded
/// burst to wait within its original budget instead of failing immediately.
pub(crate) async fn acquire_fetch_permit(deadline: Instant) -> Option<capacity::Permit> {
    acquire_bounded_permit(fetch_semaphore(), deadline).await
}

fn admission_waiters() -> &'static Arc<capacity::Capacity> {
    static WAITERS: OnceLock<Arc<capacity::Capacity>> = OnceLock::new();
    WAITERS.get_or_init(|| capacity::Capacity::new(2048))
}

pub(super) async fn acquire_bounded_permit(
    semaphore: &Arc<capacity::Capacity>,
    deadline: Instant,
) -> Option<capacity::Permit> {
    if Instant::now() >= deadline {
        return None;
    }
    if let Ok(permit) = semaphore.try_acquire() {
        return Some(permit);
    }
    // One shared finite waiting budget across image-pipeline and network slots.
    let _waiting = admission_waiters().try_acquire().ok()?;
    let permit = latency::timed(
        latency::Phase::Queue,
        tokio::time::timeout_at(
            tokio::time::Instant::from_std(deadline),
            semaphore.acquire_priority(is_high_priority()),
        ),
    )
    .await
    .ok()?
    .ok()?;
    (Instant::now() < deadline).then_some(permit)
}

tokio::task_local! {
    static HIGH_PRIORITY: bool;
}
static HIGH_SERVED: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
/// Requests that asked for first-viewport priority, cumulative since start.
pub(crate) fn high_priority_requests() -> u64 {
    HIGH_SERVED.load(std::sync::atomic::Ordering::Relaxed)
}

/// Whether the request being served asked to be scheduled first (first-viewport
/// image of the open message). Tasks that never set it are normal priority.
pub(crate) fn is_high_priority() -> bool {
    HIGH_PRIORITY.try_with(|high| *high).unwrap_or(false)
}

/// `x-mw-priority: high`, `?priority=high` (an `<img>` cannot set headers) or the
/// RFC 9218 `Priority: u=0|1` field a browser sends for `fetchpriority="high"`.
pub(crate) fn requested_priority(headers: &HeaderMap, query: Option<&str>) -> bool {
    let named = headers
        .get("x-mw-priority")
        .and_then(|v| v.to_str().ok())
        .is_some_and(|v| v.trim().eq_ignore_ascii_case("high"));
    let rfc9218 = headers
        .get("priority")
        .and_then(|v| v.to_str().ok())
        .is_some_and(|v| {
            v.split(',')
                .any(|p| matches!(p.trim(), "u=0" | "u=1"))
        });
    let queried = query.is_some_and(|q| q.split('&').any(|p| p == "priority=high"));
    named || rfc9218 || queried
}

/// Run `work` with the priority the request asked for.
pub(crate) async fn with_priority<F: std::future::Future>(high: bool, work: F) -> F::Output {
    if high {
        HIGH_SERVED.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        HIGH_PRIORITY.scope(true, work).await
    } else {
        work.await
    }
}

/// Wait for a token from the account's bucket, in arrival order, for at most
/// until `deadline`. Authentication and the grant check have already passed when
/// this runs. `false` means the wait would not fit the queue budget (answer 429).
pub(crate) async fn charge_waiting(
    account_id: &str,
    burst: usize,
    per_min: usize,
    deadline: Instant,
) -> bool {
    history::note_rate(burst, per_min);
    let wait = rate_limiter()
        .lock()
        .expect("image rate-limit lock")
        .reserve(
            account_id,
            burst as f64,
            per_min as f64 / 60.0,
            deadline.saturating_duration_since(Instant::now()),
        );
    match wait {
        Some(wait) if wait.is_zero() => true,
        Some(wait) => {
            history::note_wait(wait.as_micros() as u64);
            latency::timed(latency::Phase::Queue, tokio::time::sleep(wait)).await;
            true
        }
        None => false,
    }
}

// ── per-account rate limit (R6, t18) ────────────────────────────────────────────

/// Fixed bucket the unit tests exercise; production sizing is `adaptive::rate_budget`.
#[cfg(test)]
const RATE_REFILL_PER_SEC: f64 = 1.0;
#[cfg(test)]
const RATE_BURST: f64 = 120.0;

/// A single account's token bucket.
struct TokenBucket {
    tokens: f64,
    last: std::time::Instant,
}

/// A coarse in-memory per-account token-bucket limiter for the image-proxy FETCH
/// path (R6). It caps how fast one account can drive DISTINCT upstream fetches
/// (cache hits are free — see [`proxy_image`]), a fan-out/abuse limit rather than a
/// security boundary (the SSRF gate is that).
///
/// Caveat (documented, by design): state lives in a process-local `OnceLock` static,
/// so it resets on restart and is **per-replica** — N replicas each admit the full
/// rate. A cluster-global limit would need a shared store and a hot-path write, not
/// warranted for an abuse cap. The account map is bounded by the deployment's account
/// count (one small bucket per account that has used the proxy); no eviction needed.
pub(crate) struct AccountRateLimiter {
    buckets: HashMap<String, TokenBucket>,
}

impl AccountRateLimiter {
    /// Charge one token to `account_id`, refilling for elapsed time first. Returns
    /// `true` if a token was available (request allowed), `false` if the bucket is
    /// exhausted (→ `429`). A never-seen account starts with a full burst.
    #[cfg(test)]
    fn check(&mut self, account_id: &str) -> bool {
        self.check_with_limit(account_id, RATE_BURST, RATE_REFILL_PER_SEC)
    }

    /// Accounts that spent or refilled a bucket within the last minute.
    pub(crate) fn active_accounts(&self) -> usize {
        let now = std::time::Instant::now();
        self.buckets
            .values()
            .filter(|b| now.saturating_duration_since(b.last) < Duration::from_secs(60))
            .count()
    }

    /// Reserve one token, allowing the balance to go negative so that waiters are
    /// served in arrival order. Returns how long the caller must wait before it
    /// may proceed (zero when a token was free), or `None` when that wait would
    /// pass `max_wait`: then nothing is reserved and the caller should refuse.
    fn reserve(
        &mut self,
        account_id: &str,
        burst: f64,
        refill_per_sec: f64,
        max_wait: Duration,
    ) -> Option<Duration> {
        let now = std::time::Instant::now();
        let b = self
            .buckets
            .entry(account_id.to_string())
            .or_insert(TokenBucket {
                tokens: burst,
                last: now,
            });
        let elapsed = now.saturating_duration_since(b.last).as_secs_f64();
        b.tokens = (b.tokens + elapsed * refill_per_sec).min(burst);
        b.last = now;
        if b.tokens >= 1.0 {
            b.tokens -= 1.0;
            return Some(Duration::ZERO);
        }
        let wait = Duration::from_secs_f64((1.0 - b.tokens) / refill_per_sec.max(1e-9));
        if wait > max_wait {
            return None;
        }
        b.tokens -= 1.0;
        Some(wait)
    }

    fn check_with_limit(&mut self, account_id: &str, burst: f64, refill_per_sec: f64) -> bool {
        let now = std::time::Instant::now();
        let b = self
            .buckets
            .entry(account_id.to_string())
            .or_insert(TokenBucket {
                tokens: burst,
                last: now,
            });
        let elapsed = now.saturating_duration_since(b.last).as_secs_f64();
        b.tokens = (b.tokens + elapsed * refill_per_sec).min(burst);
        b.last = now;
        if b.tokens >= 1.0 {
            b.tokens -= 1.0;
            true
        } else {
            false
        }
    }
}

pub(crate) fn rate_limiter() -> &'static Mutex<AccountRateLimiter> {
    static RL: OnceLock<Mutex<AccountRateLimiter>> = OnceLock::new();
    RL.get_or_init(|| {
        Mutex::new(AccountRateLimiter {
            buckets: HashMap::new(),
        })
    })
}

/// Quoted-hex `ETag` of the re-encoded bytes.
fn etag_for(bytes: &[u8]) -> String {
    let digest = Sha256::digest(bytes);
    let mut s = String::with_capacity(2 + digest.len() * 2 + 1);
    s.push('"');
    for b in digest {
        s.push_str(&format!("{b:02x}"));
    }
    s.push('"');
    s
}

// ── handlers ───────────────────────────────────────────────────────────────────

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ProxyQuery {
    url: String,
    /// The message this image is being loaded FOR — what every grant scope is keyed
    /// on. Optional on the wire (a request may simply omit it) but NOT optional for
    /// the fetch: without it no scope can match, so [`grant_covers`] refuses.
    #[serde(default)]
    email_id: Option<String>,
}

/// Whether `account_id` holds a grant covering the message `email_id` — the
/// server-side half of the remote-image grant model (t22-e7 P3).
///
/// Deny-by-default at every step: no id, an id that does not resolve, an id
/// belonging to ANOTHER account, or no covering grant all return `false`. The
/// account check matters because grants are per-account — an id borrowed from
/// another account must not unlock that account's sender scopes.
///
/// All four scopes are resolved by [`mw_store::Store::remote_image_allowed`] in one
/// query rather than reassembled here, so this cannot drift from the model the grant
/// endpoints below write.
///
/// `Err(())` is a store failure, distinct from "not granted", so a database problem
/// surfaces as a `500` instead of silently reading as a denied grant.
async fn grant_covers(
    store: &mw_store::Store,
    account_id: &str,
    email_id: Option<&str>,
) -> Result<bool, ()> {
    let Some(email_id) = email_id.filter(|s| !s.is_empty()) else {
        return Ok(false);
    };
    let msg = match store.get_message(email_id).await {
        Ok(m) => m,
        Err(mw_store::StoreError::NotFound) => return Ok(false),
        Err(_) => return Err(()),
    };
    if msg.account_id != account_id {
        return Ok(false);
    }
    let (sender, domain) = sender_of(store, email_id).await;
    store
        .remote_image_allowed(account_id, email_id, &sender, &domain)
        .await
        .map_err(|_| ())
}

/// A message's sender address and its domain, both lower-cased.
///
/// Derived exactly as the client derives the values it GRANTS, or the two would
/// never match: the first `from` address (`Reader.tsx`'s
/// `props.email.from?.[0]?.email ?? ''`) and the part after its last `@`
/// (`remote-images.ts::senderDomain`).
///
/// Empty when the message has no stored envelope or no `from`. Such a message can
/// then only be covered by an `all` or `single` grant — which is the honest answer,
/// since nothing is known about its sender.
async fn sender_of(store: &mw_store::Store, email_id: &str) -> (String, String) {
    let Ok(Some(bytes)) = store.get_envelope(email_id).await else {
        return (String::new(), String::new());
    };
    let Ok(email) = serde_json::from_slice::<mw_jmap::Email>(&bytes) else {
        return (String::new(), String::new());
    };
    let sender = email
        .from
        .as_ref()
        .and_then(|v| v.first())
        .map(|a| a.email.trim().to_lowercase())
        .unwrap_or_default();
    let domain = match sender.rfind('@') {
        Some(at) => sender[at + 1..].to_string(),
        None => String::new(),
    };
    (sender, domain)
}

/// `GET /api/image-proxy?url=…` — session-authed, SSRF-gated fetch → wasm-jail
/// re-encode → PNG. Served same-origin so the shell's `img-src 'self'` covers it.
async fn proxy_image(
    State(state): State<AppState>,
    headers: HeaderMap,
    Query(q): Query<ProxyQuery>,
    uri: axum::http::Uri,
) -> Response {
    let high = requested_priority(&headers, uri.query());
    with_priority(high, latency::track(proxy_image_inner(state, headers, q))).await
}

async fn proxy_image_inner(state: AppState, headers: HeaderMap, q: ProxyQuery) -> Response {
    // Require a session — never an open relay. Capture it for the per-account rate
    // limit below.
    let session = match crate::authed(&state, &headers).await {
        Ok(s) => s,
        Err(r) => return r,
    };
    if q.url.len() > remote_content::MAX_URL_LEN {
        return (StatusCode::URI_TOO_LONG, remote_content::URL_TOO_LONG).into_response();
    }
    let policy = match crate::remote_policy::effective_for_message(
        &state,
        &session,
        q.email_id.as_deref().unwrap_or(""),
    )
    .await
    {
        Ok(p) => p,
        Err(r) => return r,
    };
    latency::set_enabled(policy.track_latency);
    if policy.mode == "block" || (policy.trackers && mw_sanitize::remote::is_tracker_url(&q.url)) {
        return (StatusCode::FORBIDDEN, "remote content blocked by policy").into_response();
    }

    // P3: the grant gate comes BEFORE the cache, or a session with no grant could
    // read images another session fetched.
    //
    // MOVING THIS CHANGES BEHAVIOUR TWO SUITES DEPEND ON — the constraint is not
    // obvious from the code, so it is written here rather than left to be
    // rediscovered:
    //   * it must stay AFTER the session check (`authed`, above), or the refusal
    //     bodies below become readable without a session — an unauthenticated
    //     reachability oracle. `t17_ssrf_nat64` and `t18_e2e_ssrf_teredo_isatap`
    //     assert an anonymous caller sees neither body;
    //   * the EGRESS POLICY must run before this gate reports "not granted", which
    //     is why the refusal path calls it (see `ungranted_response`) rather than
    //     answering here. Refuse earlier and `t16_image_proxy`'s expected 400s for
    //     `file://`/`ftp://`/credential URLs become 403s;
    //   * the RATE LIMITER must still be charged on the refusal path, or
    //     `t18_e2e_ratelimit`'s 403 → 429 progression starves — requests would be
    //     refused here and never reach the counter.
    // All three suites stay green on the happy path either way, so they will not
    // catch a reordering for you; they catch it only through those specific legs.
    match grant_covers(&state.store, &session.account_id, q.email_id.as_deref()).await {
        Ok(true) => {}
        Ok(false) => return ungranted_response(&session.account_id, &q.url, &policy).await,
        Err(()) => return internal("remote-image grant check"),
    }

    // Which egress path this fetch takes. Read BEFORE the cache lookup because the
    // route is half the cache key — bytes fetched over one egress path must never be
    // served to a request that was supposed to take another, and that decision
    // cannot be made after the lookup. Costs one indexed read (and one AEAD open, on
    // a configured route) per request, cache hits included.
    let plan = match load_plan(&state.store).await {
        Ok(p) => p,
        Err(()) => return internal("egress route lookup"),
    };
    // A route scoped to the account's mail domain overrides the global one.
    let domain = match crate::remote_policy::canonical_domain(&state, &session).await {
        Ok(domain) => domain,
        Err(response) => return response,
    };
    let route_id = plan
        .selected_route(TrafficClass::RemoteContent, domain.as_deref())
        .map_or(DIRECT_ROUTE, |r| r.id.as_str());

    // Serve a cache hit before doing any work (and honor If-None-Match). A cache hit
    // performs no upstream fetch, so it does NOT consume the per-account rate budget.
    let key = CacheKey::new(
        &format!("{route_id}:legacy:{}:{}", policy.trackers, policy.max_bytes),
        &q.url,
    )
    .by(&session.account_id, q.email_id.is_some());
    // Authentication, the grant and the egress plan are all behind this point; the
    // tier is only reached by an authorised request.
    let tier = tier::prepare(&policy.image_tier).await;
    let cached_response = || async {
        let (etag, png) = tier.lookup(&key.route, &key.url, key.ctx).await?;
        record_cache(true);
        if if_none_match(&headers, &etag) {
            return Some(not_modified(&etag));
        }
        Some(image_response(png, etag))
    };
    if let Some(response) = cached_response().await {
        return response;
    }
    let current = match remote_content::configure_capacity(&state, Some(&policy.fetch)).await {
        Ok(current) => current,
        Err(response) => return response,
    };
    let deadline = Instant::now() + policy.fetch.overall_budget();
    let queue_deadline = policy.fetch.queue_deadline(deadline);
    let Some(flight) = latency::timed(
        latency::Phase::Queue,
        remote_content::image_flight::acquire(
            &session.account_id,
            &key.route,
            &q.url,
            queue_deadline,
            current.queued,
        ),
    )
    .await
    else {
        return remote_content::busy_response();
    };
    if let Some(response) = cached_response().await {
        return response;
    }

    // A failure remembered for this URL (t56) answers at once, before the rate
    // limit, like a cache hit.
    if let Some(failure) = remote_content::fetch_limits::remembered_for(
        &session.account_id,
        &q.url,
        policy.negative_cache_ttl,
    ) {
        return remote_content::failed_response(failure.refusal(), policy.negative_cache_ttl);
    }

    // Per-account fan-out rate limit (R6): a cache MISS will fetch upstream, so
    // charge the account one token here. Exhaustion -> 429 (per-replica; see
    // `AccountRateLimiter`).
    record_cache(false);
    if !remote_content::charge_image_fetch(
        &session.account_id,
        q.email_id.as_deref().unwrap_or(""),
        &policy,
        queue_deadline,
    )
    .await
    {
        return (
            StatusCode::TOO_MANY_REQUESTS,
            "image proxy rate limit exceeded",
        )
            .into_response();
    }

    let url = match reqwest::Url::parse(&q.url) {
        Ok(u) => u,
        Err(_) => return refusal_response(Refusal::BadRequest("malformed URL")),
    };

    // The fetch, under the t56 slots, timeouts and negative cache. With a route
    // configured this is FAIL-CLOSED: `plan::send_with` has no arm that reaches
    // the direct path once a route is in play, so a proxy that is down produces a
    // `502` rather than a quiet direct fetch that defeats the egress control the
    // operator configured. The egress audit is written inside, before a refusal
    // is returned.
    let request = Request::get(TrafficClass::RemoteContent)
        .response_limit(policy.max_bytes)
        .accept("image/*");
    let (raw, content_type, admission) = match remote_content::guarded_fetch(
        &state,
        &plan,
        route_id,
        url,
        request,
        domain.as_deref(),
        (
            &session.account_id,
            q.email_id.as_deref().unwrap_or(""),
            &q.url,
        ),
        &policy,
        deadline,
    )
    .await
    {
        Ok(fetched) => fetched,
        Err(response) => return response,
    };

    if let Some(why) = remote_content::type_refusal(content_type.as_deref(), &raw) {
        return (StatusCode::UNSUPPORTED_MEDIA_TYPE, why).into_response();
    }
    remote_content::reencode_and_answer(
        raw,
        Some(key),
        &policy,
        deadline,
        admission.with_flight(flight),
    )
    .await
}

/// Refuse without DNS or network I/O; preserve pure URL validation and throttling.
async fn ungranted_response(
    account_id: &str,
    raw_url: &str,
    policy: &crate::remote_policy::PrivacyPolicy,
) -> Response {
    // Refusals never wait: they are charged against the same bucket, and a drained
    // bucket answers 429 at once so probing cannot hold a connection open.
    let budget = remote_content::rate_budget_for(policy, None);
    if !rate_limiter()
        .lock()
        .expect("image rate-limit lock")
        .check_with_limit(
            account_id,
            budget.burst as f64,
            budget.per_min as f64 / 60.0,
        )
    {
        return (
            StatusCode::TOO_MANY_REQUESTS,
            "image proxy rate limit exceeded",
        )
            .into_response();
    }
    let url = match reqwest::Url::parse(raw_url) {
        Ok(u) => u,
        Err(_) => return refusal_response(Refusal::BadRequest("malformed URL")),
    };
    if !matches!(url.scheme(), "http" | "https") {
        return refusal_response(Refusal::BadRequest("URL scheme must be http or https"));
    }
    if !url.username().is_empty() || url.password().is_some() {
        return refusal_response(Refusal::BadRequest("URL credentials are not allowed"));
    }
    (
        StatusCode::FORBIDDEN,
        "no remote-image grant covers this message",
    )
        .into_response()
}

/// Browser caching of a proxied image. The URL carries the signed handle, which
/// names one (account, source URL) pair and expires; the bytes behind a handle
/// do not change while it lives, so a re-opened message or reply is answered
/// from the browser's per-user cache for ten minutes (well inside the one-hour
/// handle lifetime) and revalidated by `ETag` after that. `private`: never a
/// shared cache. The web client sends `Clear-Site-Data: "cache"` on sign-out.
pub const IMAGE_CACHE_CONTROL: &str = "private, max-age=600";

/// Build a `200` image response with the content-hash `ETag` + private caching.
fn image_response(png: bytes::Bytes, etag: String) -> Response {
    (
        StatusCode::OK,
        [
            (header::CONTENT_TYPE, "image/png".to_string()),
            (header::CACHE_CONTROL, IMAGE_CACHE_CONTROL.to_string()),
            (header::ETAG, etag),
            (header::X_CONTENT_TYPE_OPTIONS, "nosniff".to_string()),
        ],
        png,
    )
        .into_response()
}

fn not_modified(etag: &str) -> Response {
    (
        StatusCode::NOT_MODIFIED,
        [
            (header::ETAG, etag.to_string()),
            (header::CACHE_CONTROL, IMAGE_CACHE_CONTROL.to_string()),
        ],
    )
        .into_response()
}

/// Whether the request's `If-None-Match` covers `etag`.
fn if_none_match(headers: &HeaderMap, etag: &str) -> bool {
    headers
        .get(header::IF_NONE_MATCH)
        .and_then(|v| v.to_str().ok())
        .map(|inm| inm == "*" || inm.split(',').any(|t| t.trim() == etag))
        .unwrap_or(false)
}

// ── grant endpoints (S8, over the 0016 4-scope model) ──────────────────────────

/// The four image grants and one exact-message tracker exception.
fn valid_scope_kind(kind: &str) -> bool {
    matches!(
        kind,
        "single" | "all" | "per-sender" | "per-domain" | "trackers-single"
    )
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct GrantReq {
    scope_kind: String,
    #[serde(default)]
    scope_value: String,
}

/// `GET /api/remote-images/grants` — the caller's active (non-revoked) grants.
async fn list_grants(State(state): State<AppState>, headers: HeaderMap) -> Response {
    let session = match crate::authed(&state, &headers).await {
        Ok(s) => s,
        Err(r) => return r,
    };
    match state
        .store
        .list_active_image_grants(&session.account_id)
        .await
    {
        Ok(rows) => {
            let list: Vec<_> = rows
                .iter()
                .map(|g| {
                    json!({
                        "scopeKind": g.scope_kind,
                        "scopeValue": g.scope_value,
                        "grantedAt": g.granted_at,
                    })
                })
                .collect();
            Json(json!({ "accountId": session.account_id, "list": list })).into_response()
        }
        Err(_) => internal("list grants"),
    }
}

/// `POST /api/remote-images/grant` — grant remote-image loading for a scope
/// (idempotent; un-revokes). The scope is applied to the CALLER's account; a
/// client-supplied account id is never trusted.
async fn grant(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(body): Json<GrantReq>,
) -> Response {
    let session = match crate::authed(&state, &headers).await {
        Ok(s) => s,
        Err(r) => return r,
    };
    if !valid_scope_kind(&body.scope_kind) {
        return (StatusCode::BAD_REQUEST, "unknown grant scope").into_response();
    }
    // `all` is account-wide: pin its value to "" so it cannot masquerade as a
    // narrower scope.
    let value = if body.scope_kind == "all" {
        ""
    } else {
        body.scope_value.trim()
    };
    if body.scope_kind == "trackers-single" {
        if value.is_empty() || value.len() > 512 {
            return (StatusCode::BAD_REQUEST, "emailId is required").into_response();
        }
        let policy = match crate::remote_policy::effective(&state, &session).await {
            Ok(policy) => policy,
            Err(response) => return response,
        };
        if !policy.tracker_override_allowed || policy.mode == "block" {
            return (StatusCode::FORBIDDEN, "tracker consent is disabled").into_response();
        }
        match remote_content::message_context(&state, &session, value).await {
            Ok(Some(_)) => {}
            Ok(None) => return (StatusCode::FORBIDDEN, "no such message").into_response(),
            Err(()) => return internal("tracker grant ownership check"),
        }
    }
    match state
        .store
        .grant_remote_image(&session.account_id, &body.scope_kind, value)
        .await
    {
        Ok(()) => Json(json!({ "ok": true })).into_response(),
        Err(_) => internal("grant remote image"),
    }
}

/// `POST /api/remote-images/revoke` — soft-revoke a grant (blocks again next load).
async fn revoke(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(body): Json<GrantReq>,
) -> Response {
    let session = match crate::authed(&state, &headers).await {
        Ok(s) => s,
        Err(r) => return r,
    };
    if !valid_scope_kind(&body.scope_kind) {
        return (StatusCode::BAD_REQUEST, "unknown grant scope").into_response();
    }
    let value = if body.scope_kind == "all" {
        ""
    } else {
        body.scope_value.trim()
    };
    match state
        .store
        .revoke_remote_image(&session.account_id, &body.scope_kind, value)
        .await
    {
        Ok(()) => Json(json!({ "ok": true })).into_response(),
        Err(_) => internal("revoke remote image"),
    }
}

fn internal(ctx: &str) -> Response {
    tracing::error!("image proxy: {ctx} failed");
    (StatusCode::INTERNAL_SERVER_ERROR, "internal error").into_response()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn priority_is_requested_by_header_query_or_rfc9218_field_only() {
        let mut h = HeaderMap::new();
        assert!(!requested_priority(&h, None));
        assert!(!requested_priority(&h, Some("priority=low&x=1")));
        assert!(requested_priority(&h, Some("a=1&priority=high")));
        h.insert("x-mw-priority", "High".parse().unwrap());
        assert!(requested_priority(&h, None));
        let mut h = HeaderMap::new();
        h.insert("priority", "u=1, i".parse().unwrap());
        assert!(requested_priority(&h, None));
        h.insert("priority", "u=4".parse().unwrap());
        assert!(!requested_priority(&h, None));
    }

    #[tokio::test]
    async fn the_priority_scope_is_visible_only_inside_the_request() {
        assert!(!is_high_priority());
        with_priority(true, async { assert!(is_high_priority()) }).await;
        with_priority(false, async { assert!(!is_high_priority()) }).await;
        assert!(!is_high_priority());
    }

    #[tokio::test]
    async fn image_queue_wait_longer_than_network_total_preserves_a_real_fetch_budget() {
        use tokio::io::{AsyncReadExt, AsyncWriteExt};
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let address = listener.local_addr().unwrap();
        let server = tokio::spawn(async move {
            let (mut stream, _) = listener.accept().await.unwrap();
            let mut request = [0; 2048];
            stream.read(&mut request).await.unwrap();
            stream
                .write_all(b"HTTP/1.1 200 OK\r\nContent-Length: 2\r\nConnection: close\r\n\r\nok")
                .await
                .unwrap();
        });
        let limits = remote_content::FetchLimits {
            queue_timeout: Duration::from_secs(2),
            timeouts: mw_egress::FetchTimeouts {
                total: Duration::from_millis(150),
                ..mw_egress::FetchTimeouts::REMOTE_IMAGE
            },
            ..remote_content::FetchLimits::default()
        };
        let gate: &'static Arc<capacity::Capacity> =
            Box::leak(Box::new(capacity::Capacity::new(1)));
        let held = gate.acquire().await.unwrap();
        let started = Instant::now();
        let deadline = started + limits.overall_budget();
        let queued = tokio::spawn(async move {
            let _permit = acquire_bounded_permit(gate, limits.queue_deadline(deadline))
                .await
                .expect("queue headroom");
            assert!(started.elapsed() > limits.timeouts.total);
            let network = limits
                .network_timeouts(deadline)
                .expect("fetch budget remains");
            assert_eq!(
                network, limits.timeouts,
                "waiting must not consume the network total"
            );
            reqwest::Client::builder()
                .no_proxy()
                .build()
                .unwrap()
                .get(format!("http://{address}/image"))
                .timeout(network.total)
                .send()
                .await
                .unwrap()
                .text()
                .await
                .unwrap()
        });
        tokio::time::sleep(Duration::from_millis(300)).await;
        assert!(!queued.is_finished());
        drop(held);
        assert_eq!(queued.await.unwrap(), "ok");
        server.await.unwrap();
    }

    #[tokio::test]
    async fn image_queue_stops_at_its_own_deadline_and_releases_waiter_before_network_budget() {
        let limits = remote_content::FetchLimits {
            queue_timeout: Duration::from_millis(30),
            timeouts: mw_egress::FetchTimeouts {
                total: Duration::from_secs(5),
                ..mw_egress::FetchTimeouts::REMOTE_IMAGE
            },
            ..remote_content::FetchLimits::default()
        };
        let gate: &'static Arc<capacity::Capacity> =
            Box::leak(Box::new(capacity::Capacity::new(1)));
        let held = gate.acquire().await.unwrap();
        let before = admission_waiters().available_permits();
        let overall = Instant::now() + limits.overall_budget();
        assert!(
            acquire_bounded_permit(gate, limits.queue_deadline(overall))
                .await
                .is_none()
        );
        assert!(
            Instant::now() < overall,
            "admission must not consume reserved network time"
        );
        assert_eq!(admission_waiters().available_permits(), before);
        drop(held);
    }

    #[tokio::test]
    async fn image_admission_http2_burst_waits_without_early_busy_responses() {
        let gate: &'static Arc<capacity::Capacity> =
            Box::leak(Box::new(capacity::Capacity::new(1)));
        let held = gate.acquire().await.unwrap();
        let before = admission_waiters().available_permits();
        let mut jobs = Vec::new();
        for _ in 0..300 {
            jobs.push(tokio::spawn(async move {
                acquire_bounded_permit(gate, Instant::now() + Duration::from_secs(3))
                    .await
                    .is_some()
            }));
        }
        tokio::time::timeout(Duration::from_secs(1), async {
            while admission_waiters().available_permits() > before.saturating_sub(300) {
                tokio::task::yield_now().await;
            }
        })
        .await
        .unwrap();
        assert!(
            jobs.iter().all(|job| !job.is_finished()),
            "legitimate image requests must wait while their deadlines remain"
        );
        drop(held);
        for job in jobs {
            assert!(job.await.unwrap());
        }
        assert_eq!(admission_waiters().available_permits(), before);
    }

    #[tokio::test]
    async fn image_admission_queue_is_finite_and_cancellation_reclaims_waiters() {
        let gate: &'static Arc<capacity::Capacity> =
            Box::leak(Box::new(capacity::Capacity::new(1)));
        let held = gate.acquire().await.unwrap();
        let capacity = admission_waiters().available_permits();
        let mut jobs = Vec::new();
        for _ in 0..capacity {
            jobs.push(tokio::spawn(acquire_bounded_permit(
                gate,
                Instant::now() + Duration::from_secs(2),
            )));
        }
        tokio::time::timeout(Duration::from_secs(1), async {
            while admission_waiters().available_permits() != 0 {
                tokio::task::yield_now().await;
            }
        })
        .await
        .unwrap();
        let start = Instant::now();
        assert!(
            acquire_bounded_permit(gate, Instant::now() + Duration::from_secs(2))
                .await
                .is_none()
        );
        assert!(
            start.elapsed() < Duration::from_millis(100),
            "overflow must shed rather than enqueue unbounded work"
        );
        for job in jobs {
            job.abort();
            let _ = job.await;
        }
        assert_eq!(admission_waiters().available_permits(), capacity);
        drop(held);
        assert!(
            acquire_bounded_permit(gate, Instant::now() + Duration::from_secs(1))
                .await
                .is_some()
        );
    }
    use std::net::SocketAddr;
    use std::time::Duration;

    use mw_egress::{Hop, Target, fetch_hop};

    // ── a LOCAL origin for the re-encode leg ──────────────────────────────────
    //
    // The SSRF gate, the URL/scheme checks and the fetch mechanics (size cap,
    // timeout, redirect surfacing, and the `no_proxy` assertion) moved to
    // `mw-egress` with the code they cover and are tested there — including
    // through the crate's public API in `mw-egress/tests/policy_through_crate.rs`.
    //
    // What remains here is the leg that is this module's own: fetched bytes →
    // wasm media jail → stripped PNG. It calls the low-level `fetch_hop` with a
    // pinned loopback address DIRECTLY, deliberately bypassing the gate (which —
    // correctly — would refuse the 127.0.0.1 test server).

    async fn spawn_origin(
        body: Vec<u8>,
        delay: Option<Duration>,
        status: StatusCode,
        location: Option<String>,
    ) -> SocketAddr {
        use axum::routing::get as aget;
        let handler = move || async move {
            if let Some(d) = delay {
                tokio::time::sleep(d).await;
            }
            let mut resp = Response::new(axum::body::Body::from(body));
            *resp.status_mut() = status;
            if let Some(loc) = location {
                resp.headers_mut()
                    .insert(header::LOCATION, loc.parse().unwrap());
            }
            resp
        };
        let app: Router = Router::new().route("/img", aget(handler));
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();
        tokio::spawn(async move {
            axum::serve(listener, app).await.unwrap();
        });
        addr
    }

    fn target_for(addr: SocketAddr) -> Target {
        Target {
            url: reqwest::Url::parse(&format!("http://{addr}/img")).unwrap(),
            host: addr.ip().to_string(),
            addr,
        }
    }

    // ── re-encode integration: fetched bytes → wasm jail → PNG ────────────────

    #[tokio::test]
    async fn fetched_image_reencodes_to_stripped_png() {
        // 1×1 GIF served by a local origin; fetched (gate-bypassed) then re-encoded
        // in the media jail → a normalized PNG (metadata stripped by the jail).
        let gif: Vec<u8> = vec![
            0x47, 0x49, 0x46, 0x38, 0x39, 0x61, 0x01, 0x00, 0x01, 0x00, 0x80, 0x00, 0x00, 0x00,
            0x00, 0x00, 0xFF, 0xFF, 0xFF, 0x21, 0xF9, 0x04, 0x00, 0x00, 0x00, 0x00, 0x00, 0x2C,
            0x00, 0x00, 0x00, 0x00, 0x01, 0x00, 0x01, 0x00, 0x00, 0x02, 0x02, 0x44, 0x01, 0x00,
            0x3B,
        ];
        let addr = spawn_origin(gif, None, StatusCode::OK, None).await;
        let bytes = match fetch_hop(&target_for(addr), "image/*").await.unwrap() {
            Hop::Body(b) => b,
            Hop::Redirect(_) => panic!("unexpected redirect"),
        };
        let png = mw_render::media_jail::reencode_image(&bytes).expect("re-encode");
        assert_eq!(&png[..8], &[0x89, b'P', b'N', b'G', 0x0D, 0x0A, 0x1A, 0x0A]);
    }

    // The cache itself is the two-tier ImageTier; its behaviour is tested in
    // remote_image_tier_tests.rs.

    // ── R6: per-account token-bucket rate limit ──────────────────────────────────

    #[test]
    fn rate_limiter_allows_burst_then_429s_and_is_per_account() {
        let mut rl = AccountRateLimiter {
            buckets: HashMap::new(),
        };
        // A fresh account spends its full burst, then the next request is refused
        // (no measurable time elapses inside the loop → no refill).
        for i in 0..RATE_BURST as usize {
            assert!(rl.check("acct-a"), "burst token {i} should be admitted");
        }
        assert!(
            !rl.check("acct-a"),
            "exhausted bucket must return false (429)"
        );
        // A different account has an independent budget.
        assert!(
            rl.check("acct-b"),
            "a second account is limited independently"
        );
    }

    #[test]
    fn reserve_queues_in_arrival_order_and_refuses_past_the_deadline() {
        let mut rl = AccountRateLimiter {
            buckets: HashMap::new(),
        };
        let max = Duration::from_secs(5);
        // Burst of 2 at 1 token/s: two free, then waits of ~1s, ~2s, ...
        assert_eq!(rl.reserve("a", 2.0, 1.0, max), Some(Duration::ZERO));
        assert_eq!(rl.reserve("a", 2.0, 1.0, max), Some(Duration::ZERO));
        let first = rl.reserve("a", 2.0, 1.0, max).expect("fits the deadline");
        let second = rl.reserve("a", 2.0, 1.0, max).expect("fits the deadline");
        assert!(first > Duration::from_millis(900) && first <= Duration::from_secs(1));
        assert!(second > first, "later arrivals wait longer: FIFO");
        // A wait beyond the deadline reserves nothing.
        assert_eq!(rl.reserve("a", 2.0, 1.0, Duration::from_millis(10)), None);
        // Another account is unaffected.
        assert_eq!(rl.reserve("b", 2.0, 1.0, max), Some(Duration::ZERO));
    }

    #[test]
    fn rate_limiter_refills_over_time() {
        let mut rl = AccountRateLimiter {
            buckets: HashMap::new(),
        };
        for _ in 0..RATE_BURST as usize {
            assert!(rl.check("a"));
        }
        assert!(!rl.check("a"), "bucket drained");
        // Simulate ~2 seconds elapsed: at 1 token/s that is ~2 refilled tokens.
        if let Some(b) = rl.buckets.get_mut("a") {
            b.last = b
                .last
                .checked_sub(std::time::Duration::from_secs(2))
                .unwrap_or(b.last);
        }
        assert!(rl.check("a"), "first refilled token available");
        assert!(rl.check("a"), "second refilled token available");
        assert!(!rl.check("a"), "only ~2 tokens refilled, third is refused");
    }

    #[test]
    fn image_responses_are_privately_cacheable_and_never_shared() {
        for response in [
            image_response(bytes::Bytes::from_static(b"png"), "\"t\"".to_string()),
            not_modified("\"t\""),
        ] {
            let cc = response.headers()[header::CACHE_CONTROL].to_str().unwrap();
            assert!(cc.starts_with("private"), "{cc}");
            assert!(cc.contains("max-age=600"), "{cc}");
            assert!(!cc.contains("public") && !cc.contains("immutable"), "{cc}");
        }
    }

    #[test]
    fn etag_is_stable_content_hash() {
        assert_eq!(etag_for(b"abc"), etag_for(b"abc"));
        assert_ne!(etag_for(b"abc"), etag_for(b"abd"));
        assert!(etag_for(b"abc").starts_with('"'));
    }

    #[test]
    fn csp_tightens_style_and_adds_trusted_types() {
        assert!(SHELL_CSP_TIGHTENED.contains("require-trusted-types-for 'script'"));
        assert!(SHELL_CSP_TIGHTENED.contains("style-src 'self';"));
        assert!(!SHELL_CSP_TIGHTENED.contains("style-src 'self' 'unsafe-inline'"));
    }

    // ── P3: server-side grant enforcement, BOTH directions ─────────────────────
    //
    // These drive `grant_covers` against a real store, which is where the decision
    // is made; the handler's only job is to call it before the cache and the fetch.
    // A refusal-only test would pass with the feature deleted, so every case below
    // asserts the pair: the same request refused without a grant and admitted with
    // one.

    use mw_store::{
        AccountKind, Credentials, MailboxUpsert, MessageUpsert, NewAccount, ServerKey, Store,
    };

    /// A real store with a real account + mailbox, so messages carry the account id
    /// the FK and the grant check both read.
    struct Fixture {
        store: Store,
        account: String,
        mailbox: String,
    }

    async fn fixture() -> Fixture {
        let store = Store::open_in_memory(ServerKey::generate()).await.unwrap();
        let (account, mailbox) = seed_account(&store, "reader@example.org", 1).await;
        Fixture {
            store,
            account,
            mailbox,
        }
    }

    async fn seed_account(store: &Store, username: &str, uidvalidity: u32) -> (String, String) {
        let account = store
            .create_account(
                &NewAccount {
                    kind: AccountKind::Imap,
                    host: "imap.example.org",
                    port: 993,
                    tls: "implicit",
                    username,
                    sync_policy_json: "{}",
                },
                &Credentials {
                    username: username.to_string(),
                    password: "pw".into(),
                },
            )
            .await
            .unwrap();
        let mailbox = store
            .upsert_mailbox(&MailboxUpsert {
                account_id: &account,
                name: "INBOX",
                role: Some("inbox"),
                uidvalidity,
                uidnext: 1,
                highestmodseq: 0,
                total: 0,
                unread: 0,
                parent_id: None,
            })
            .await
            .unwrap();
        (account, mailbox)
    }

    /// Store a message and return the stable id the store assigned it.
    async fn seed_message(
        store: &Store,
        account: &str,
        mailbox: &str,
        uid: u32,
        from: Option<&str>,
    ) -> String {
        let envelope = from.map(|addr| {
            serde_json::to_vec(&json!({ "from": [{ "email": addr }] })).expect("envelope")
        });
        store
            .upsert_message(&MessageUpsert {
                account_id: account,
                mailbox_id: mailbox,
                uid,
                uidvalidity: 1,
                message_id: None,
                thread_id: None,
                internaldate: None,
                size: 10,
                flags_json: "[]",
                envelope: envelope.as_deref(),
                blob_ref: None,
            })
            .await
            .expect("seed message")
    }

    impl Fixture {
        async fn message(&self, uid: u32, from: Option<&str>) -> String {
            seed_message(&self.store, &self.account, &self.mailbox, uid, from).await
        }
    }

    #[tokio::test]
    async fn an_account_without_a_grant_is_refused_and_with_one_is_admitted() {
        let f = fixture().await;
        let m = f.message(1, Some("sales@shop.example")).await;

        // Deny-by-default: the message exists and belongs to the caller, but nothing
        // is granted.
        assert!(
            !grant_covers(&f.store, &f.account, Some(&m)).await.unwrap(),
            "no grant must refuse"
        );

        // The SAME request, once a covering grant exists, is admitted. Without this
        // direction the test would pass with the whole gate deleted.
        f.store
            .grant_remote_image(&f.account, "single", &m)
            .await
            .unwrap();
        assert!(
            grant_covers(&f.store, &f.account, Some(&m)).await.unwrap(),
            "a covering grant must admit"
        );

        // ...and revoking puts it back to refused.
        f.store
            .revoke_remote_image(&f.account, "single", &m)
            .await
            .unwrap();
        assert!(
            !grant_covers(&f.store, &f.account, Some(&m)).await.unwrap(),
            "a revoked grant must refuse again"
        );
    }

    #[tokio::test]
    async fn every_one_of_the_four_scopes_admits_and_only_when_it_matches() {
        // Each scope: refused before, admitted after, and NOT admitted for a message
        // the scope does not cover — so no scope can be read as account-wide except
        // `all`, which is the one that means it.
        for kind in ["single", "per-sender", "per-domain", "all"] {
            let f = fixture().await;
            let m = f.message(1, Some("sales@shop.example")).await;
            // A second message from a DIFFERENT sender, same account.
            let other = f.message(2, Some("noreply@other.example")).await;

            let value = match kind {
                "single" => m.clone(),
                "per-sender" => "sales@shop.example".to_string(),
                "per-domain" => "shop.example".to_string(),
                _ => String::new(),
            };

            assert!(!grant_covers(&f.store, &f.account, Some(&m)).await.unwrap());
            f.store
                .grant_remote_image(&f.account, kind, &value)
                .await
                .unwrap();
            assert!(
                grant_covers(&f.store, &f.account, Some(&m)).await.unwrap(),
                "a {kind} grant must cover the message it was granted for"
            );
            assert_eq!(
                grant_covers(&f.store, &f.account, Some(&other))
                    .await
                    .unwrap(),
                kind == "all",
                "{kind} must not leak onto an unrelated message"
            );
        }
    }

    #[tokio::test]
    async fn a_grant_does_not_cross_accounts() {
        let f = fixture().await;
        let m = f.message(1, Some("sales@shop.example")).await;
        let (other_account, _) = seed_account(&f.store, "someone@else.example", 2).await;

        // The other account grants itself everything...
        f.store
            .grant_remote_image(&other_account, "all", "")
            .await
            .unwrap();

        // ...which must not let it proxy for a message it does not own, even though
        // its own grant is as broad as grants get.
        assert!(
            !grant_covers(&f.store, &other_account, Some(&m))
                .await
                .unwrap(),
            "another account's message id must not be usable"
        );
        // ...and must not carry over to the message's real owner, who granted nothing.
        assert!(
            !grant_covers(&f.store, &f.account, Some(&m)).await.unwrap(),
            "the owner holds no grant of its own"
        );
    }

    #[tokio::test]
    async fn a_missing_or_unknown_message_id_is_refused_even_with_an_account_grant() {
        let f = fixture().await;
        let m = f.message(1, Some("sales@shop.example")).await;
        // The broadest grant there is.
        f.store
            .grant_remote_image(&f.account, "all", "")
            .await
            .unwrap();

        // No id, an empty id, and an id that resolves to nothing all name no message
        // — refused. This is what stops the gate degrading into "the account holds
        // some grant", which would admit any URL for any message.
        assert!(!grant_covers(&f.store, &f.account, None).await.unwrap());
        assert!(!grant_covers(&f.store, &f.account, Some("")).await.unwrap());
        assert!(
            !grant_covers(&f.store, &f.account, Some("no-such-id"))
                .await
                .unwrap()
        );
        // The real id under that same grant IS admitted — the control that proves
        // the refusals above are about the id, not about the grant being missing.
        assert!(grant_covers(&f.store, &f.account, Some(&m)).await.unwrap());
    }

    #[tokio::test]
    async fn sender_scopes_are_derived_the_way_the_client_grants_them() {
        // The client grants `sender.toLowerCase()` and `senderDomain(sender)`; the
        // server must derive the same strings from the envelope or the two never
        // match. Mixed case in the envelope, lower-case in the grant.
        let f = fixture().await;
        let m = f.message(1, Some("Sales@Shop.Example")).await;

        let (sender, domain) = sender_of(&f.store, &m).await;
        assert_eq!(sender, "sales@shop.example");
        assert_eq!(domain, "shop.example");

        f.store
            .grant_remote_image(&f.account, "per-sender", "sales@shop.example")
            .await
            .unwrap();
        assert!(grant_covers(&f.store, &f.account, Some(&m)).await.unwrap());
    }

    #[tokio::test]
    async fn a_message_with_no_envelope_falls_back_to_all_and_single_only() {
        let f = fixture().await;
        let m = f.message(1, None).await;
        assert_eq!(
            sender_of(&f.store, &m).await,
            (String::new(), String::new())
        );

        // Nothing is known about the sender, so a per-domain grant cannot cover it...
        f.store
            .grant_remote_image(&f.account, "per-domain", "shop.example")
            .await
            .unwrap();
        assert!(!grant_covers(&f.store, &f.account, Some(&m)).await.unwrap());
        // ...but a single-message grant still can.
        f.store
            .grant_remote_image(&f.account, "single", &m)
            .await
            .unwrap();
        assert!(grant_covers(&f.store, &f.account, Some(&m)).await.unwrap());
    }

    #[test]
    fn only_image_grants_and_the_single_message_tracker_scope_are_valid() {
        for k in [
            "single",
            "all",
            "per-sender",
            "per-domain",
            "trackers-single",
        ] {
            assert!(valid_scope_kind(k));
        }
        for k in [
            "",
            "global",
            "sender",
            "domain",
            "ALL",
            "trackers-all",
            "trackers-domain",
        ] {
            assert!(!valid_scope_kind(k));
        }
    }

    // ── the egress plan from the store (t32 e31b) ──────────────────────────────

    fn stored(kind: &str) -> EgressProxyRow {
        EgressProxyRow {
            id: "r1".into(),
            kind: kind.into(),
            host: "proxy.corp.example".into(),
            port: 3128,
            username: "svc".into(),
            password: Some("pw".into()),
            allow_plaintext: false,
            active: false,
            tls_to_proxy: false,
            remote_dns: false,
            remote_dns_risk_accepted_by: None,
            remote_dns_risk_accepted_at: None,
            tunnel_id: None,
            chain_next_id: None,
            scope: "global".into(),
            traffic_classes: vec!["remote_content".into(), "ics_feeds".into()],
            created_at: String::new(),
            updated_at: String::new(),
        }
    }

    #[test]
    fn each_stored_kind_maps_to_its_transport_or_to_a_refusal() {
        let scheme = |r: &EgressProxyRow| proxy_route(r).map(|p| p.scheme);
        assert_eq!(scheme(&stored("http")), Some(ProxyScheme::HttpConnect));
        assert_eq!(scheme(&stored("socks5")), Some(ProxyScheme::Socks5));

        let mut tls = stored("http");
        tls.tls_to_proxy = true;
        assert_eq!(scheme(&tls), Some(ProxyScheme::HttpsConnect));

        // Remote DNS only with the recorded acceptance.
        let mut h = stored("socks5h");
        h.remote_dns = true;
        assert_eq!(scheme(&h), None, "no acceptance recorded");
        h.remote_dns_risk_accepted_by = Some("root".into());
        h.remote_dns_risk_accepted_at = Some(1);
        assert_eq!(scheme(&h), Some(ProxyScheme::Socks5h));

        // Not carried by this build yet, or inconsistent: refused, never downgraded.
        let mut chained = stored("http");
        chained.chain_next_id = Some("r2".into());
        let mut socks_tls = stored("socks5");
        socks_tls.tls_to_proxy = true;
        let mut socks_remote = stored("socks5");
        socks_remote.remote_dns = true;
        socks_remote.remote_dns_risk_accepted_by = Some("root".into());
        let mut h_off = stored("socks5h");
        h_off.remote_dns_risk_accepted_by = Some("root".into());
        let mut http_remote = stored("http");
        http_remote.remote_dns = true;
        http_remote.remote_dns_risk_accepted_by = Some("root".into());
        for (what, r) in [
            ("ssh", stored("ssh")),
            ("unknown kind", stored("ftp")),
            ("chained", chained),
            ("socks5 with TLS", socks_tls),
            ("socks5 with remote DNS", socks_remote),
            ("socks5h without remote DNS", h_off),
            ("http with remote DNS", http_remote),
        ] {
            assert_eq!(scheme(&r), None, "{what}");
        }
    }

    async fn plan_store() -> mw_store::Store {
        mw_store::Store::open_in_memory(mw_store::ServerKey::generate())
            .await
            .unwrap()
    }

    #[tokio::test]
    async fn the_plan_carries_the_live_routes_classes_and_the_allowlist() {
        let store = plan_store().await;
        let mut r = stored("http");
        r.traffic_classes = vec!["remote_content".into(), "keyservers".into()];
        store.put_egress_proxy(&r).await.unwrap();

        // Configured but not live: every class goes direct.
        let plan = load_plan(&store).await.unwrap();
        assert!(
            plan.selected_route(TrafficClass::RemoteContent, None)
                .is_none()
        );

        store.set_active_egress_proxy(Some("r1")).await.unwrap();
        store
            .add_egress_internal_allow(&mw_store::EgressAllowRow {
                id: "a1".into(),
                traffic_class: "siem".into(),
                destination: "10.20.0.0/16".into(),
                note: String::new(),
                created_by: "root".into(),
                created_at: 1,
            })
            .await
            .unwrap();
        let plan = load_plan(&store).await.unwrap();
        let id = |c| {
            plan.selected_route(c, Some("any.example"))
                .map(|r| r.id.clone())
        };
        assert_eq!(id(TrafficClass::RemoteContent), Some("r1".into()));
        assert_eq!(id(TrafficClass::Keyservers), Some("r1".into()));
        assert_eq!(id(TrafficClass::IcsFeeds), None, "not listed, so direct");
        assert!(plan.allow().permits(
            TrafficClass::Siem,
            "siem",
            6514,
            &"10.20.1.1".parse().unwrap()
        ));
    }

    #[tokio::test]
    async fn a_live_route_the_build_cannot_carry_refuses_the_plan() {
        let store = plan_store().await;
        let mut h = stored("socks5h");
        h.remote_dns = true;
        store.put_egress_proxy(&h).await.unwrap();
        store.set_active_egress_proxy(Some("r1")).await.unwrap();
        assert!(
            load_plan(&store).await.is_err(),
            "a live remote-DNS route without the acceptance must stop egress, not go direct"
        );
        assert!(
            store
                .accept_egress_remote_dns_risk("r1", "root", 1)
                .await
                .unwrap()
        );
        let plan = load_plan(&store).await.unwrap();
        assert_eq!(
            plan.selected_route(TrafficClass::RemoteContent, None)
                .map(|r| r.scheme),
            Some(ProxyScheme::Socks5h)
        );
    }

    #[tokio::test]
    async fn an_unknown_class_on_a_live_route_refuses_the_plan() {
        let store = plan_store().await;
        let mut r = stored("http");
        r.traffic_classes = vec!["remote_content".into(), "everything".into()];
        store.put_egress_proxy(&r).await.unwrap();
        store.set_active_egress_proxy(Some("r1")).await.unwrap();
        assert!(load_plan(&store).await.is_err());
    }

    #[tokio::test]
    async fn a_bad_allowlist_row_is_skipped_and_the_rest_still_applies() {
        let store = plan_store().await;
        let row = |id: &str, dest: &str| mw_store::EgressAllowRow {
            id: id.into(),
            traffic_class: "webhooks".into(),
            destination: dest.into(),
            note: String::new(),
            created_by: "root".into(),
            created_at: 1,
        };
        store
            .add_egress_internal_allow(&row("bad", "169.254.169.254"))
            .await
            .unwrap();
        store
            .add_egress_internal_allow(&row("good", "10.1.0.0/16"))
            .await
            .unwrap();
        let plan = load_plan(&store).await.unwrap();
        assert_eq!(plan.allow().entries().len(), 1);
        assert!(!plan.allow().permits(
            TrafficClass::Webhooks,
            "x",
            80,
            &"169.254.169.254".parse().unwrap()
        ));
    }
}
