//! The admin route registry (t32-ext §M, e24a seam; mini-slot `t32-admin-registry`).
//!
//! Every admin route is mounted here through
//! [`AdminRegistrar::admin_endpoint`]`(method, suffix, access, handler)`:
//!
//! * the path is built only with h15's helper, `mw_admin::config::admin_route`
//!   (the console moves with `MW_ADMIN_PATH`); `suffix` starts with `/`;
//! * `access` is a [`Capability`] (or [`Access::SignedIn`] / [`Access::Public`]).
//!   Before the handler runs, the gate resolves the caller's [`AdminCtx`]: no admin
//!   session → `401`, capability missing → `403`, an unscoped-only capability held
//!   for some domains only → `403`. The handler receives the context as an
//!   extractor (`ctx: AdminCtx`) and checks the target's domain with
//!   `ctx.require_domain(..)`;
//! * the `(method, suffix, access)` triple is recorded, so [`registry`] lists every
//!   admin route. `tests/t32_admin_route_scan.rs` fails on an admin route mounted
//!   any other way, and `tests/t32_rbac_matrix.rs` calls every registered route
//!   without the capability.
//!
//! Adding a route: one `r.admin_endpoint(..)` line in [`endpoints`] (or in a
//! function of your module that takes `&mut impl AdminRegistrar`, called from
//! there). Write errors use [`crate::admin_ctx::AdminApiError`], reads send an
//! `ETag` and writes honour `If-Match` ([`crate::admin_ctx::check_if_match`]).

use axum::Router;
use axum::extract::{Request, State};
use axum::handler::Handler;
use axum::http::Method;
use axum::middleware::{self, Next};
use axum::response::{IntoResponse, Response};
use axum::routing::{MethodFilter, on};
use serde_json::{Value, json};

use crate::AppState;
use crate::admin_ctx::AdminCtx;
use crate::capabilities::Capability;

/// Who may call a route.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Access {
    /// No admin session needed (sign-in steps). The handler does its own checks.
    Public,
    /// Any signed-in admin (their own profile, sign-out).
    SignedIn,
    /// A signed-in admin holding this capability.
    Cap(Capability),
}

impl From<Capability> for Access {
    fn from(c: Capability) -> Self {
        Access::Cap(c)
    }
}

/// One registered admin route.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Endpoint {
    pub method: Method,
    /// The path under the admin path (`/users/{id}`).
    pub suffix: &'static str,
    pub access: Access,
}

/// Something admin routes are registered with: the router that mounts them, or
/// the list [`registry`] returns.
pub(crate) trait AdminRegistrar {
    /// Register `handler` for `method` at `admin_route(suffix)`, behind `access`.
    fn admin_endpoint<H, T>(
        &mut self,
        method: Method,
        suffix: &'static str,
        access: impl Into<Access>,
        handler: H,
    ) where
        H: Handler<T, AppState>,
        T: 'static;
}

/// Every admin route. Landing order follows `t32-admin-registry` (t32-fast §4).
fn endpoints(r: &mut impl AdminRegistrar) {
    // f02 (e24a): the catalogue.
    r.admin_endpoint(Method::GET, "/capabilities", Access::SignedIn, capabilities);
    // f01 (e10b): the settings model C (`/settings/**`).
    crate::settings_routes::register(r);
}

/// The mounted admin routes (merged by `router()` in `lib.rs`).
pub(crate) fn router(state: &AppState) -> Router<AppState> {
    let mut m = Mount {
        router: Router::new(),
        state: state.clone(),
    };
    endpoints(&mut m);
    m.router
}

/// Every registered admin route, in registration order.
pub fn registry() -> Vec<Endpoint> {
    let mut l = List(Vec::new());
    endpoints(&mut l);
    l.0
}

struct Mount {
    router: Router<AppState>,
    state: AppState,
}

impl AdminRegistrar for Mount {
    fn admin_endpoint<H, T>(
        &mut self,
        method: Method,
        suffix: &'static str,
        access: impl Into<Access>,
        handler: H,
    ) where
        H: Handler<T, AppState>,
        T: 'static,
    {
        assert!(
            suffix.starts_with('/'),
            "admin route {suffix} must start with /"
        );
        let access = access.into();
        let filter = MethodFilter::try_from(method.clone())
            .unwrap_or_else(|_| panic!("admin route {method} {suffix}: unsupported method"));
        let gate = middleware::from_fn_with_state(
            self.state.clone(),
            move |State(state): State<AppState>, req: Request, next: Next| {
                gate(state, access, req, next)
            },
        );
        let route = on(filter, handler).route_layer(gate);
        let router = std::mem::take(&mut self.router);
        self.router = router.route(&mw_admin::config::admin_route(suffix), route);
    }
}

struct List(Vec<Endpoint>);

impl AdminRegistrar for List {
    fn admin_endpoint<H, T>(
        &mut self,
        method: Method,
        suffix: &'static str,
        access: impl Into<Access>,
        _handler: H,
    ) where
        H: Handler<T, AppState>,
        T: 'static,
    {
        self.0.push(Endpoint {
            method,
            suffix,
            access: access.into(),
        });
    }
}

/// The check in front of every registered handler.
async fn gate(state: AppState, access: Access, mut req: Request, next: Next) -> Response {
    let cap = match access {
        Access::Public => return next.run(req).await,
        Access::SignedIn => None,
        Access::Cap(cap) => Some(cap),
    };
    let ctx = match AdminCtx::resolve(&state, req.headers()).await {
        Ok(ctx) => ctx,
        Err(e) => return e.into_response(),
    };
    if let Some(cap) = cap
        && let Err(e) = ctx.require(cap)
    {
        return e.into_response();
    }
    req.extensions_mut().insert(ctx.for_route(cap));
    next.run(req).await
}

// ─── catalogue ──────────────────────────────────────────────────────────────────

/// `GET /admin/capabilities` — every capability id, and whether only a grant for
/// all domains carries it.
async fn capabilities(_ctx: AdminCtx) -> axum::Json<Value> {
    let list: Vec<Value> = Capability::ALL
        .iter()
        .map(|c| json!({ "id": c.as_str(), "unscopedOnly": c.unscoped_only() }))
        .collect();
    axum::Json(json!({ "capabilities": list }))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeSet;

    #[test]
    fn registry_has_no_duplicates() {
        let mut seen = BTreeSet::new();
        for e in registry() {
            assert!(e.suffix.starts_with('/'), "{}", e.suffix);
            assert!(
                seen.insert((e.method.to_string(), e.suffix)),
                "{} {} registered twice",
                e.method,
                e.suffix
            );
        }
    }
}
