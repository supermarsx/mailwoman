//! The egress plan, and the class-aware send that consumes it (t32 e31b, plan R2).
//!
//! # What a plan is
//! An [`EgressPlan`] is the deployment's egress configuration in one value: the
//! operator's routes, each with a [`RouteScope`] (global, or one mail domain) and
//! the [`TrafficClass`]es it carries, plus the [`InternalAllowList`]. `mw-server`
//! builds it from `egress_proxy` and `egress_internal_allow` and installs it
//! process-wide ([`install`]); every crate that egresses reads it with
//! [`current`], so crates without a store (`mw-crypto`, `mw-autoconfig`,
//! `mw-admin`, `mw-engine`) honour the same routes as the server does.
//!
//! # How a request picks its route
//! [`EgressPlan::selected_route`]: a route scoped to the request's mail domain that lists
//! the class wins; otherwise the global route that lists it; otherwise the request
//! goes direct. A plan in which two routes would answer the same (scope, class) is
//! refused when it is built ([`EgressPlan::new`]), so the answer is never a choice
//! between rows.
//!
//! The inputs are the class — a constant at the call site — and the mail domain of
//! the account the request is made for, which the server takes from the signed-in
//! session. Neither can name a host: they select among routes an operator saved.
//! That is the property `ProxyRoute::host`'s exemption from the address policy
//! rests on (see `mw_egress::proxy` and `tests/route_construction_sites.rs`).
//!
//! # Fail-closed, still
//! When a route applies, [`send_with`] has one exit on that arm and it is the
//! transport's result: no direct retry, on any error (see
//! [`crate::fetch_remote_routed`] for why that arm is the bug). And a process that
//! has **not** installed a plan refuses every class request rather than going
//! direct — otherwise the first seconds after start, before `mw-server` has read
//! the store, would egress around a configured route.
//!
//! # The address policy per class
//! On the direct arm a resolved address must pass [`crate::ip_allowed`], or — for
//! the `webhooks` and `siem` classes only — match an internal-allowlist entry
//! ([`InternalAllowList::permits`], which still refuses loopback, link-local and
//! metadata). Through a route the transport re-applies [`crate::ip_allowed`]
//! unconditionally, so an allowlisted private destination is **not** reachable
//! through a route: an admin who wants an on-prem collector leaves that class
//! direct. Narrower, and it refuses visibly.

use std::net::IpAddr;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, RwLock};
use std::time::Instant;

use crate::proxy::{self, ProxyRoute};
use crate::{
    Hop, InternalAllowList, MAX_REDIRECTS, OutBody, Outgoing, Refusal, RoutedFetch, Target,
    TrafficClass, ip_allowed, resolve_gated, send_hop_typed,
};

// ── scope ──────────────────────────────────────────────────────────────────────

/// Where a route applies: everywhere, or to accounts of one mail domain.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum RouteScope {
    /// Every account and every server-side request without an account.
    Global,
    /// Accounts of this mail domain (lowercase, no trailing dot). Overrides the
    /// global route for the classes it lists.
    Domain(String),
}

impl RouteScope {
    /// Parse the storage spelling of `egress_proxy.scope`: `global` or
    /// `domain:<name>`.
    pub fn parse(s: &str) -> Option<RouteScope> {
        if s == "global" {
            return Some(RouteScope::Global);
        }
        let name = s.strip_prefix("domain:")?;
        let name = name.trim_end_matches('.').to_ascii_lowercase();
        if name.is_empty() || name.contains(|c: char| c.is_whitespace() || c == '/' || c == ':') {
            return None;
        }
        Some(RouteScope::Domain(name))
    }

    /// The storage spelling.
    pub fn as_storage(&self) -> String {
        match self {
            RouteScope::Global => "global".to_string(),
            RouteScope::Domain(d) => format!("domain:{d}"),
        }
    }
}

// ── the plan ───────────────────────────────────────────────────────────────────

/// One operator route as the plan holds it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PlanRoute {
    /// Where it applies.
    pub scope: RouteScope,
    /// The classes it carries. A class not listed by any applicable route goes
    /// direct.
    pub classes: Vec<TrafficClass>,
    /// The transport route, built by `mw-server`'s one construction site from a
    /// stored row.
    pub route: ProxyRoute,
}

/// Why a plan could not be built.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PlanError {
    /// Two routes answer the same class in the same scope. Carries the two route
    /// ids and the class, for the admin's error message.
    Ambiguous {
        /// The class both routes list.
        class: TrafficClass,
        /// The scope they share.
        scope: RouteScope,
        /// The first route's id.
        first: String,
        /// The second route's id.
        second: String,
    },
}

impl std::fmt::Display for PlanError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PlanError::Ambiguous {
                class,
                scope,
                first,
                second,
            } => write!(
                f,
                "routes {first} and {second} both carry {class} in scope {}",
                scope.as_storage()
            ),
        }
    }
}

/// The deployment's egress configuration. See the module docs.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct EgressPlan {
    routes: Vec<PlanRoute>,
    allow: InternalAllowList,
}

impl EgressPlan {
    /// No routes and an empty allowlist: every class goes direct under
    /// [`crate::ip_allowed`]. What a deployment without egress configuration
    /// installs, and what tests and command-line tools install.
    pub fn direct() -> EgressPlan {
        EgressPlan::default()
    }

    /// A plan from operator routes and the allowlist. Refuses a plan in which two
    /// routes would answer the same class in the same scope.
    pub fn new(routes: Vec<PlanRoute>, allow: InternalAllowList) -> Result<EgressPlan, PlanError> {
        for (i, a) in routes.iter().enumerate() {
            for b in &routes[i + 1..] {
                if a.scope != b.scope {
                    continue;
                }
                if let Some(class) = a.classes.iter().find(|c| b.classes.contains(c)) {
                    return Err(PlanError::Ambiguous {
                        class: *class,
                        scope: a.scope.clone(),
                        first: a.route.id.clone(),
                        second: b.route.id.clone(),
                    });
                }
            }
        }
        Ok(EgressPlan { routes, allow })
    }

    /// The routes, in the order given.
    pub fn routes(&self) -> &[PlanRoute] {
        &self.routes
    }

    /// The internal-destination allowlist.
    pub fn allow(&self) -> &InternalAllowList {
        &self.allow
    }

    /// The route that carries `class` for an account of `domain` (`None` for a
    /// request made for no account), or `None` when it goes direct.
    ///
    /// Selects among the operator's routes; it cannot introduce a host. See the
    /// module docs for why the two inputs are safe ones.
    pub fn selected_route(&self, class: TrafficClass, domain: Option<&str>) -> Option<&ProxyRoute> {
        let domain = domain.map(|d| d.trim_end_matches('.').to_ascii_lowercase());
        let carries = |r: &&PlanRoute| r.classes.contains(&class);
        if let Some(d) = domain.as_deref() {
            let scoped = self
                .routes
                .iter()
                .filter(carries)
                .find(|r| matches!(&r.scope, RouteScope::Domain(s) if s == d));
            if let Some(r) = scoped {
                return Some(&r.route);
            }
        }
        self.routes
            .iter()
            .filter(carries)
            .find(|r| r.scope == RouteScope::Global)
            .map(|r| &r.route)
    }

    /// The direct-path address decision for `class`: [`crate::ip_allowed`], or an
    /// internal-allowlist match for the eligible classes.
    pub fn admits(&self, class: TrafficClass, host: &str, port: u16, ip: &IpAddr) -> bool {
        ip_allowed(ip) || self.allow.permits(class, host, port, ip)
    }

    /// [`crate::validate_and_resolve`] under this plan's policy for `class`: the
    /// scheme, credential and host checks unchanged, one resolution, the first
    /// admitted address pinned.
    ///
    /// For a caller that must make its own connection on the direct path (a
    /// syslog-over-TLS SIEM sink is not HTTP). Such a caller must first check
    /// [`EgressPlan::selected_route`] and refuse when a route applies to its class and it
    /// cannot use one — never connect directly instead.
    pub async fn resolve(&self, class: TrafficClass, url: reqwest::Url) -> Result<Target, Refusal> {
        resolve_gated(url, &|host: &str, port: u16, ip: &IpAddr| {
            self.admits(class, host, port, ip)
        })
        .await
    }
}

// ── the installed plan ─────────────────────────────────────────────────────────

static INSTALLED: RwLock<Option<Arc<EgressPlan>>> = RwLock::new(None);

/// Install `plan` for this process, replacing the previous one. Requests already
/// under way keep the plan they started with.
pub fn install(plan: EgressPlan) -> Arc<EgressPlan> {
    let plan = Arc::new(plan);
    let mut slot = INSTALLED.write().unwrap_or_else(|e| e.into_inner());
    *slot = Some(Arc::clone(&plan));
    plan
}

/// The installed plan, or `None` before [`install`] has run. [`send`] refuses
/// every request while this is `None`.
pub fn current() -> Option<Arc<EgressPlan>> {
    INSTALLED.read().unwrap_or_else(|e| e.into_inner()).clone()
}

// ── the request ────────────────────────────────────────────────────────────────

/// One class request, apart from its URL.
#[derive(Debug, Clone, Copy)]
pub struct Request<'a> {
    authorized_dav: Option<crate::AuthorizedDav<'a>>,
    class: TrafficClass,
    domain: Option<&'a str>,
    accept: &'a str,
    user_agent: Option<&'a str>,
    body: Option<OutBody<'a>>,
    response_limit: usize,
}

impl<'a> Request<'a> {
    /// A `GET` for `class`, accepting anything, with the class's default
    /// `User-Agent` and no account.
    pub fn get(class: TrafficClass) -> Request<'a> {
        Request {
            authorized_dav: None,
            class,
            domain: None,
            accept: "*/*",
            user_agent: None,
            body: None,
            response_limit: crate::MAX_IMAGE_BYTES,
        }
    }

    /// A `POST` of `bytes` as `content_type` for `class`. A `POST` never follows a
    /// redirect: a 3xx answer is [`Refusal::Upstream`].
    pub fn post(class: TrafficClass, content_type: &'a str, bytes: &'a [u8]) -> Request<'a> {
        Request {
            body: Some(OutBody {
                content_type,
                bytes,
            }),
            ..Request::get(class)
        }
    }

    /// Credential-bearing DAV/OCS requests never follow redirects or retry auth.
    pub fn dav(auth: crate::AuthorizedDav<'a>, body: Option<OutBody<'a>>) -> Result<Self, Refusal> {
        auth.validate()?;
        if body.is_some_and(|body| body.bytes.len() > crate::MAX_IMAGE_BYTES) {
            return Err(Refusal::TooLarge);
        }
        Ok(Self {
            authorized_dav: Some(auth),
            body,
            ..Self::get(TrafficClass::Nextcloud)
        })
    }

    /// Made for an account of this mail domain, so a route scoped to the domain
    /// applies. Pass the domain of the signed-in account, never one from a request
    /// body or header.
    pub fn for_domain(mut self, domain: &'a str) -> Request<'a> {
        self.domain = Some(domain);
        self
    }

    /// The `Accept` header.
    pub fn accept(mut self, accept: &'a str) -> Request<'a> {
        self.accept = accept;
        self
    }

    /// Lower the response cap for every hop, including tunneled requests.
    pub fn response_limit(mut self, bytes: usize) -> Request<'a> {
        self.response_limit = bytes.min(crate::MAX_IMAGE_BYTES);
        self
    }

    /// A `User-Agent` other than the class default. A constant, never a browser's.
    pub fn user_agent(mut self, user_agent: &'a str) -> Request<'a> {
        self.user_agent = Some(user_agent);
        self
    }

    /// The class.
    pub fn class(&self) -> TrafficClass {
        self.class
    }

    fn outgoing(&self) -> Outgoing<'a> {
        Outgoing {
            authorized_dav: self.authorized_dav,
            accept: self.accept,
            user_agent: self
                .user_agent
                .unwrap_or_else(|| self.class.default_user_agent()),
            body: self.body,
            response_limit: self.response_limit,
        }
    }
}

// ── send ───────────────────────────────────────────────────────────────────────

/// Send `req` to `url` under the installed plan. Refuses every request when no plan
/// is installed (see the module docs).
pub async fn send(url: reqwest::Url, req: &Request<'_>) -> RoutedFetch {
    match current() {
        Some(plan) => send_with(&plan, url, req).await,
        None => {
            let outcome = Err(Refusal::BadRequest("egress configuration is not loaded"));
            record(req.class, &outcome, Instant::now());
            RoutedFetch {
                traversed_proxy: false,
                content_type: None,
                outcome,
            }
        }
    }
}

/// A `GET` of `url` for `class` (and the account's `domain`, when there is one)
/// under the installed plan.
pub async fn fetch_class(
    class: TrafficClass,
    domain: Option<&str>,
    url: reqwest::Url,
    accept: &str,
) -> RoutedFetch {
    let mut req = Request::get(class).accept(accept);
    if let Some(d) = domain {
        req = req.for_domain(d);
    }
    send(url, &req).await
}

/// Send `req` to `url` under `plan`: through the route [`EgressPlan::selected_route`]
/// picks, fail-closed, or directly under the class's address policy.
pub async fn send_with(plan: &EgressPlan, url: reqwest::Url, req: &Request<'_>) -> RoutedFetch {
    let started = Instant::now();
    let out = req.outgoing();
    let fetched = match plan.selected_route(req.class, req.domain) {
        // A route applies. Its result is the result — one exit, no direct retry.
        Some(route) => {
            let via = proxy::send_via_proxy(url, route, &out).await;
            RoutedFetch {
                traversed_proxy: via.traversed_proxy,
                content_type: via.content_type,
                outcome: via.outcome.map_err(Refusal::from),
            }
        }
        None => {
            let class = req.class;
            let gate = |host: &str, port: u16, ip: &IpAddr| plan.admits(class, host, port, ip);
            match send_direct(url, &out, &gate).await {
                Ok((bytes, content_type)) => RoutedFetch {
                    traversed_proxy: false,
                    content_type,
                    outcome: Ok(bytes),
                },
                Err(r) => RoutedFetch {
                    traversed_proxy: false,
                    content_type: None,
                    outcome: Err(r),
                },
            }
        }
    };
    record(req.class, &fetched.outcome, started);
    fetched
}

/// The direct loop: resolve and pin under `gate`, send, and re-enter the gate for
/// every redirect of a `GET`. A `POST` follows no redirect.
async fn send_direct(
    start: reqwest::Url,
    out: &Outgoing<'_>,
    gate: &crate::AddressGate<'_>,
) -> Result<(Vec<u8>, Option<String>), Refusal> {
    let mut url = start;
    for _ in 0..=MAX_REDIRECTS {
        let target = resolve_gated(url.clone(), gate).await?;
        match send_hop_typed(&target, out).await? {
            (Hop::Body(bytes), content_type) => return Ok((bytes, content_type)),
            (Hop::Redirect(_), _) if out.body.is_some() || out.authorized_dav.is_some() => {
                return Err(Refusal::Upstream);
            }
            (Hop::Redirect(loc), _) => url = url.join(&loc).map_err(|_| Refusal::Upstream)?,
        }
    }
    Err(Refusal::Upstream)
}

// ── counters ───────────────────────────────────────────────────────────────────

/// Per-class request counts since the process started, for metrics and
/// diagnostics. Labels are the class and the result only — never a host, a URL or
/// an account.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct ClassCounts {
    /// Requests that returned a body.
    pub allowed: u64,
    /// Requests refused by Mailwoman's own policy before or instead of the
    /// network: the address gate, the URL checks, a plaintext origin through a
    /// route, or no installed plan.
    pub refused: u64,
    /// Requests that failed at the far end or on the way: timeouts, non-success
    /// statuses, transport and proxy failures, oversize bodies.
    pub failed: u64,
    /// Total wall time of all requests, in milliseconds.
    pub total_ms: u64,
}

struct Counter {
    allowed: AtomicU64,
    refused: AtomicU64,
    failed: AtomicU64,
    total_ms: AtomicU64,
}

#[allow(clippy::declare_interior_mutable_const)]
const ZERO: Counter = Counter {
    allowed: AtomicU64::new(0),
    refused: AtomicU64::new(0),
    failed: AtomicU64::new(0),
    total_ms: AtomicU64::new(0),
};

static COUNTERS: [Counter; TrafficClass::ALL.len()] = [ZERO; TrafficClass::ALL.len()];

fn slot(class: TrafficClass) -> &'static Counter {
    let i = TrafficClass::ALL
        .iter()
        .position(|c| *c == class)
        .expect("every class is in ALL");
    &COUNTERS[i]
}

fn record(class: TrafficClass, outcome: &Result<Vec<u8>, Refusal>, started: Instant) {
    let c = slot(class);
    let bucket = match outcome {
        Ok(_) => &c.allowed,
        Err(Refusal::BadRequest(_) | Refusal::Blocked) => &c.refused,
        Err(_) => &c.failed,
    };
    bucket.fetch_add(1, Ordering::Relaxed);
    let ms = u64::try_from(started.elapsed().as_millis()).unwrap_or(u64::MAX);
    c.total_ms.fetch_add(ms, Ordering::Relaxed);
}

/// The counts for every class, in [`TrafficClass::ALL`] order.
pub fn counts() -> Vec<(TrafficClass, ClassCounts)> {
    TrafficClass::ALL
        .iter()
        .map(|class| {
            let c = slot(*class);
            (
                *class,
                ClassCounts {
                    allowed: c.allowed.load(Ordering::Relaxed),
                    refused: c.refused.load(Ordering::Relaxed),
                    failed: c.failed.load(Ordering::Relaxed),
                    total_ms: c.total_ms.load(Ordering::Relaxed),
                },
            )
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::proxy::ProxyScheme;
    use crate::{AllowDestination, AllowEntry};

    fn route(id: &str) -> ProxyRoute {
        ProxyRoute {
            id: id.into(),
            scheme: ProxyScheme::HttpConnect,
            host: "proxy.example".into(),
            port: 3128,
            auth: None,
            allow_plaintext: false,
        }
    }

    fn planned(id: &str, scope: RouteScope, classes: &[TrafficClass]) -> PlanRoute {
        PlanRoute {
            scope,
            classes: classes.to_vec(),
            route: route(id),
        }
    }

    #[test]
    fn scope_spellings_round_trip_and_refuse_junk() {
        assert_eq!(RouteScope::parse("global"), Some(RouteScope::Global));
        assert_eq!(
            RouteScope::parse("domain:Example.COM."),
            Some(RouteScope::Domain("example.com".into()))
        );
        assert_eq!(
            RouteScope::Domain("example.com".into()).as_storage(),
            "domain:example.com"
        );
        for s in [
            "",
            "Global",
            "domain:",
            "domain:a b",
            "domain:a/b",
            "domain:a:1",
            "example.com",
        ] {
            assert_eq!(RouteScope::parse(s), None, "{s:?}");
        }
    }

    #[test]
    fn a_class_goes_direct_unless_a_route_lists_it() {
        use TrafficClass::*;
        let plan = EgressPlan::new(
            vec![planned(
                "tor",
                RouteScope::Global,
                &[RemoteContent, IcsFeeds],
            )],
            InternalAllowList::empty(),
        )
        .unwrap();
        assert_eq!(
            plan.selected_route(RemoteContent, None)
                .map(|r| r.id.as_str()),
            Some("tor")
        );
        assert_eq!(
            plan.selected_route(IcsFeeds, Some("a.example"))
                .map(|r| r.id.as_str()),
            Some("tor")
        );
        for c in [
            Keyservers,
            Webhooks,
            AdminDetect,
            Pki,
            DnsAuth,
            Unsubscribe,
            Siem,
            Oauth,
            UpdateCheck,
        ] {
            assert!(plan.selected_route(c, None).is_none(), "{c} must go direct");
            assert!(
                plan.selected_route(c, Some("a.example")).is_none(),
                "{c} must go direct"
            );
        }
    }

    #[test]
    fn a_domain_route_overrides_the_global_route_for_its_classes_only() {
        use TrafficClass::*;
        let plan = EgressPlan::new(
            vec![
                planned("global", RouteScope::Global, &[RemoteContent, Keyservers]),
                planned(
                    "corp",
                    RouteScope::Domain("corp.example".into()),
                    &[RemoteContent],
                ),
            ],
            InternalAllowList::empty(),
        )
        .unwrap();
        let id = |c, d| plan.selected_route(c, d).map(|r| r.id.clone());
        assert_eq!(id(RemoteContent, Some("corp.example")), Some("corp".into()));
        assert_eq!(
            id(RemoteContent, Some("CORP.example.")),
            Some("corp".into())
        );
        assert_eq!(id(Keyservers, Some("corp.example")), Some("global".into()));
        assert_eq!(
            id(RemoteContent, Some("other.example")),
            Some("global".into())
        );
        assert_eq!(id(RemoteContent, None), Some("global".into()));
        assert_eq!(id(Pki, Some("corp.example")), None);
    }

    #[test]
    fn two_routes_answering_one_class_in_one_scope_are_refused() {
        use TrafficClass::*;
        let err = EgressPlan::new(
            vec![
                planned("a", RouteScope::Global, &[RemoteContent, Pki]),
                planned("b", RouteScope::Global, &[Pki]),
            ],
            InternalAllowList::empty(),
        )
        .unwrap_err();
        assert_eq!(
            err,
            PlanError::Ambiguous {
                class: Pki,
                scope: RouteScope::Global,
                first: "a".into(),
                second: "b".into(),
            }
        );
        // The same class in different scopes is the override, and is fine.
        EgressPlan::new(
            vec![
                planned("a", RouteScope::Global, &[Pki]),
                planned("b", RouteScope::Domain("x.example".into()), &[Pki]),
            ],
            InternalAllowList::empty(),
        )
        .expect("a domain override is not an ambiguity");
    }

    #[tokio::test]
    async fn resolve_admits_an_allowlisted_private_address_for_its_class_only() {
        let allow = InternalAllowList::new(vec![
            AllowEntry::new(
                TrafficClass::Siem,
                AllowDestination::parse("10.20.0.0/16").unwrap(),
            )
            .unwrap(),
        ]);
        let plan = EgressPlan::new(Vec::new(), allow).unwrap();
        let url = reqwest::Url::parse("https://10.20.0.9:6514/ingest").unwrap();

        let t = plan
            .resolve(TrafficClass::Siem, url.clone())
            .await
            .expect("an allowlisted destination passes for its class");
        assert_eq!(t.addr.ip().to_string(), "10.20.0.9");
        for c in TrafficClass::ALL {
            if c == TrafficClass::Siem {
                continue;
            }
            assert_eq!(
                plan.resolve(c, url.clone()).await.unwrap_err(),
                Refusal::Blocked,
                "{c} must not reach the SIEM's private address"
            );
        }
        // Outside the entry, and metadata, stay refused for the class itself.
        for u in [
            "https://10.21.0.9/",
            "http://169.254.169.254/latest/meta-data/",
            "http://127.0.0.1/",
        ] {
            assert_eq!(
                plan.resolve(TrafficClass::Siem, reqwest::Url::parse(u).unwrap())
                    .await
                    .unwrap_err(),
                Refusal::Blocked,
                "{u}"
            );
        }
    }

    #[tokio::test]
    async fn resolve_keeps_the_url_checks_whatever_the_class() {
        let plan = EgressPlan::direct();
        for u in [
            "file:///etc/passwd",
            "ftp://example.com/",
            "http://u:p@example.com/",
        ] {
            let err = plan
                .resolve(TrafficClass::Webhooks, reqwest::Url::parse(u).unwrap())
                .await
                .unwrap_err();
            assert!(matches!(err, Refusal::BadRequest(_)), "{u}: {err:?}");
        }
    }

    #[test]
    fn the_request_builder_sets_what_it_says() {
        let r = Request::post(
            TrafficClass::Unsubscribe,
            "application/x-www-form-urlencoded",
            b"List-Unsubscribe=One-Click",
        )
        .for_domain("a.example")
        .accept("text/plain");
        let out = r.outgoing();
        assert_eq!(out.accept, "text/plain");
        assert_eq!(out.user_agent, crate::class::CLASS_UA);
        assert_eq!(out.body.unwrap().bytes, b"List-Unsubscribe=One-Click");
        assert_eq!(r.domain, Some("a.example"));

        let g = Request::get(TrafficClass::RemoteContent);
        assert_eq!(g.outgoing().user_agent, crate::PROXY_UA);
        assert!(g.outgoing().body.is_none());
        let u = Request::get(TrafficClass::UpdateCheck).user_agent("Mailwoman/26.20");
        assert_eq!(u.outgoing().user_agent, "Mailwoman/26.20");
    }

    #[test]
    fn a_body_never_reaches_debug_output() {
        let r = Request::post(
            TrafficClass::Webhooks,
            "application/json",
            b"{\"secret\":\"s3cr3t\"}",
        );
        let rendered = format!("{:?}", r.outgoing());
        assert!(!rendered.contains("s3cr3t"), "{rendered}");
        assert!(rendered.contains("application/json"), "{rendered}");
    }
}

/// Open a ManageSieve byte stream using the account domain's selected route.
/// `allow_private` MUST come from the operator's endpoint configuration, never
/// the request body. All DNS answers pass the gate before either socket opens.
/// TLS to the endpoint is applied by the protocol client over the returned pipe.
pub async fn connect_sieve_with(
    plan: &EgressPlan,
    domain: Option<&str>,
    host: &str,
    port: u16,
    allow_private: bool,
    encrypted: bool,
) -> Result<proxy::stream::ProxyConn, proxy::ProxyRefusal> {
    use proxy::ProxyRefusal;
    let route = plan.selected_route(TrafficClass::Sieve, domain);
    if let Some(route) = route {
        if !encrypted && !route.allow_plaintext {
            return Err(ProxyRefusal::PlaintextRefused);
        }
        if route.scheme == proxy::ProxyScheme::Socks5h {
            return Err(ProxyRefusal::RouteInvalid(
                "remote DNS is unavailable for pinned mail transport",
            ));
        }
    }
    tokio::time::timeout(crate::FETCH_TIMEOUT, async {
        let addresses: Vec<_> = tokio::net::lookup_host((host, port))
            .await
            .map_err(|_| ProxyRefusal::Origin(Refusal::Blocked))?
            .collect();
        let Some(destination) = addresses.first().copied() else {
            return Err(ProxyRefusal::Origin(Refusal::Blocked));
        };
        if !addresses
            .iter()
            .all(|address| sieve_ip_allowed(&address.ip(), allow_private))
        {
            return Err(ProxyRefusal::Origin(Refusal::Blocked));
        }
        match route {
            Some(route) => proxy::open_pinned_stream(route, destination).await,
            None => {
                let stream = tokio::net::TcpStream::connect(destination)
                    .await
                    .map_err(|_| ProxyRefusal::Origin(Refusal::Upstream))?;
                let _ = stream.set_nodelay(true);
                Ok(proxy::stream::ProxyConn::Tcp(stream))
            }
        }
    })
    .await
    .map_err(|_| ProxyRefusal::Origin(Refusal::Timeout))?
}

fn sieve_ip_allowed(ip: &IpAddr, allow_private: bool) -> bool {
    crate::operator_mail_ip_allowed(ip, allow_private)
}

#[cfg(test)]
mod sieve_tests {
    use super::*;
    use tokio::io::{AsyncBufReadExt, AsyncReadExt, AsyncWriteExt, BufReader};
    use tokio::net::TcpListener;
    fn plan(port: u16, scope: RouteScope) -> EgressPlan {
        EgressPlan::new(
            vec![PlanRoute {
                scope,
                classes: [TrafficClass::Sieve].into_iter().collect(),
                route: proxy::ProxyRoute {
                    id: "sieve-proxy".into(),
                    scheme: proxy::ProxyScheme::HttpConnect,
                    host: "127.0.0.1".into(),
                    port,
                    auth: None,
                    allow_plaintext: false,
                },
            }],
            InternalAllowList::empty(),
        )
        .unwrap()
    }
    #[test]
    fn private_permission_never_admits_metadata_or_reserved_addresses() {
        for ip in [
            "169.254.169.254",
            "0.0.0.0",
            "255.255.255.255",
            "224.0.0.1",
            "fd00:ec2::254",
            "fe80::1",
            "64:ff9b::a9fe:a9fe",
            "2002:7f00:1::",
        ] {
            assert!(!sieve_ip_allowed(&ip.parse().unwrap(), true), "{ip}");
        }
        for ip in [
            "10.1.2.3",
            "192.168.1.2",
            "fd00::1",
            "127.0.0.1",
            "::1",
            "::ffff:127.0.0.1",
        ] {
            assert!(!sieve_ip_allowed(&ip.parse().unwrap(), false));
            assert!(sieve_ip_allowed(&ip.parse().unwrap(), true));
        }
    }
    #[tokio::test]
    async fn configured_sieve_route_carries_the_pinned_stream() {
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let plan = plan(
            listener.local_addr().unwrap().port(),
            RouteScope::Domain("example.org".into()),
        );
        let recorder = tokio::spawn(async move {
            let (stream, _) = listener.accept().await.unwrap();
            let mut stream = BufReader::new(stream);
            let mut request = String::new();
            loop {
                let mut line = String::new();
                stream.read_line(&mut line).await.unwrap();
                if line == "\r\n" {
                    break;
                }
                request.push_str(&line);
            }
            assert!(
                request.starts_with("CONNECT 10.1.2.3:4190 HTTP/1.1\r\n"),
                "{request}"
            );
            stream
                .get_mut()
                .write_all(b"HTTP/1.1 200 Connection established\r\n\r\nsieve")
                .await
                .unwrap();
        });
        let mut stream =
            connect_sieve_with(&plan, Some("EXAMPLE.ORG"), "10.1.2.3", 4190, true, true)
                .await
                .unwrap();
        let mut payload = [0; 5];
        stream.read_exact(&mut payload).await.unwrap();
        assert_eq!(&payload, b"sieve");
        recorder.await.unwrap();
    }
    #[tokio::test]
    async fn blocked_target_and_plaintext_open_no_proxy_socket() {
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let plan = plan(listener.local_addr().unwrap().port(), RouteScope::Global);
        assert!(
            connect_sieve_with(&plan, None, "10.1.2.3", 4190, false, true)
                .await
                .is_err()
        );
        assert!(
            connect_sieve_with(&plan, None, "8.8.8.8", 4190, false, false)
                .await
                .is_err()
        );
        assert!(
            tokio::time::timeout(std::time::Duration::from_millis(50), listener.accept())
                .await
                .is_err()
        );
    }
    #[tokio::test]
    async fn configured_route_failure_is_final() {
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let plan = plan(listener.local_addr().unwrap().port(), RouteScope::Global);
        drop(listener);
        let error = connect_sieve_with(&plan, None, "8.8.8.8", 4190, false, true)
            .await
            .err()
            .unwrap();
        assert!(matches!(error, proxy::ProxyRefusal::ProxyUnreachable(_)));
    }
}
