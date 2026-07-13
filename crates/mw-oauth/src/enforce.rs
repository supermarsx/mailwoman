//! Enforcement core (plan §2.3): `require_scope` resolves a presented credential
//! (API key **or** OAuth access token) to a [`Scope`], then checks expiry, IP
//! allowlist, per-key rate limit, resource/audience binding, and the requested
//! capability — emitting an audit event either way. `mw-server` (e11) mounts this
//! behind axum middleware; here it is a plain callable core.

use std::collections::HashMap;
use std::net::IpAddr;
use std::sync::Mutex;
use std::time::{Duration, Instant};

use chrono::Utc;

use crate::keys;
use crate::oauth::{AuthServer, is_expired};
use crate::store::OAuthStore;
use crate::util::sha256_hex;
use crate::{OAuthError, Scope, TokenKind};

/// How a caller authenticated.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CredentialKind {
    ApiKey,
    OAuthToken,
}

impl CredentialKind {
    fn as_str(self) -> &'static str {
        match self {
            CredentialKind::ApiKey => "api-key",
            CredentialKind::OAuthToken => "oauth-token",
        }
    }
}

/// The inbound request as far as enforcement is concerned.
#[derive(Debug, Clone)]
pub struct RequestContext<'a> {
    /// The presented credential: `mwk_<prefix>.<secret>` or an OAuth access token.
    pub credential: &'a str,
    /// Source IP (for allowlist checks). `None` when unknown.
    pub source_ip: Option<IpAddr>,
    /// The resource-server audience this request targets (RFC 8707). When set, a
    /// resource-bound token must match it.
    pub resource: Option<&'a str>,
}

/// A successful authorization.
#[derive(Debug, Clone)]
pub struct Granted {
    pub account_id: String,
    pub scope: Scope,
    pub resource: Option<String>,
    pub via: CredentialKind,
}

/// An emitted audit record (the hook `mw-admin`/`mw-server` persists to
/// `audit_log`). Deliberately carries no secret material.
#[derive(Debug, Clone)]
pub struct AuditEvent {
    /// Key prefix or OAuth client id (`unknown` if resolution failed).
    pub actor: String,
    pub actor_kind: &'static str,
    pub action: String,
    pub allowed: bool,
    pub reason: Option<String>,
    pub ip: Option<String>,
    pub ts: String,
}

/// Sink for audit events. `require_scope` calls [`AuditSink::emit`] exactly once.
pub trait AuditSink {
    fn emit(&self, event: &AuditEvent);
}

/// Drops audit events (default when the caller does not wire a sink).
pub struct NoopAudit;
impl AuditSink for NoopAudit {
    fn emit(&self, _event: &AuditEvent) {}
}

/// Collects audit events in memory (tests / diagnostics).
#[derive(Default)]
pub struct CollectingAudit {
    events: Mutex<Vec<AuditEvent>>,
}
impl CollectingAudit {
    pub fn new() -> Self {
        Self::default()
    }
    pub fn events(&self) -> Vec<AuditEvent> {
        self.events.lock().expect("lock").clone()
    }
}
impl AuditSink for CollectingAudit {
    fn emit(&self, event: &AuditEvent) {
        self.events.lock().expect("lock").push(event.clone());
    }
}

/// Fixed-window per-key rate limiter (requests/min). Ephemeral, in-process.
pub(crate) struct RateLimiter {
    inner: Mutex<HashMap<String, Window>>,
    window: Duration,
}

struct Window {
    start: Instant,
    count: u32,
}

impl RateLimiter {
    pub(crate) fn new() -> Self {
        Self {
            inner: Mutex::new(HashMap::new()),
            window: Duration::from_secs(60),
        }
    }

    /// Record a request for `key`; returns `false` when `limit` is exceeded within
    /// the current window.
    pub(crate) fn check(&self, key: &str, limit: u32) -> bool {
        let mut map = self.inner.lock().expect("lock");
        let now = Instant::now();
        let w = map.entry(key.to_string()).or_insert(Window {
            start: now,
            count: 0,
        });
        if now.duration_since(w.start) >= self.window {
            w.start = now;
            w.count = 0;
        }
        if w.count >= limit {
            return false;
        }
        w.count += 1;
        true
    }
}

/// Match a source IP against one allowlist entry (a plain IP or `addr/prefix`
/// CIDR). Malformed entries never match. IPv4 and IPv6 both supported.
fn cidr_match(entry: &str, ip: IpAddr) -> bool {
    match entry.split_once('/') {
        None => entry
            .trim()
            .parse::<IpAddr>()
            .map(|a| a == ip)
            .unwrap_or(false),
        Some((net_s, pfx_s)) => {
            let (Ok(prefix), Ok(net)) =
                (pfx_s.trim().parse::<u32>(), net_s.trim().parse::<IpAddr>())
            else {
                return false;
            };
            match (net, ip) {
                (IpAddr::V4(n), IpAddr::V4(a)) => {
                    if prefix > 32 {
                        return false;
                    }
                    let mask = if prefix == 0 {
                        0
                    } else {
                        u32::MAX << (32 - prefix)
                    };
                    (u32::from(n) & mask) == (u32::from(a) & mask)
                }
                (IpAddr::V6(n), IpAddr::V6(a)) => {
                    if prefix > 128 {
                        return false;
                    }
                    let mask = if prefix == 0 {
                        0
                    } else {
                        u128::MAX << (128 - prefix)
                    };
                    (u128::from(n) & mask) == (u128::from(a) & mask)
                }
                _ => false,
            }
        }
    }
}

fn ip_allowed(ip: Option<IpAddr>, allow: &[String]) -> bool {
    if allow.is_empty() {
        return true; // empty allowlist = any source IP
    }
    match ip {
        None => false, // allowlist set but source IP unknown → deny
        Some(ip) => allow.iter().any(|e| cidr_match(e, ip)),
    }
}

/// A credential resolved to its effective grant, before policy checks.
struct Resolved {
    kind: CredentialKind,
    actor: String,
    account_id: String,
    scope: Scope,
    resource: Option<String>,
    /// Expiry to enforce (API key's scope expiry or the token's expiry).
    expires_at: Option<String>,
}

impl<S: OAuthStore> AuthServer<S> {
    /// Resolve + authorize a request, emitting one audit event.
    ///
    /// Returns [`Granted`] when the credential is valid **and** its scope covers
    /// `required` under all policy checks; otherwise an [`OAuthError`].
    pub async fn require_scope(
        &self,
        ctx: &RequestContext<'_>,
        required: &Scope,
        audit: &dyn AuditSink,
    ) -> Result<Granted, OAuthError> {
        let resolved = self.resolve(ctx.credential).await;
        let (actor, actor_kind) = match &resolved {
            Ok(r) => (r.actor.clone(), r.kind.as_str()),
            Err(_) => ("unknown".to_string(), "unknown"),
        };

        let outcome = match resolved {
            Err(e) => Err(e),
            Ok(r) => self.check_policy(&r, ctx, required).map(|()| Granted {
                account_id: r.account_id,
                scope: r.scope,
                resource: r.resource,
                via: r.kind,
            }),
        };

        audit.emit(&AuditEvent {
            actor,
            actor_kind,
            action: "require_scope".to_string(),
            allowed: outcome.is_ok(),
            reason: outcome.as_ref().err().map(|e| e.to_string()),
            ip: ctx.source_ip.map(|i| i.to_string()),
            ts: Utc::now().to_rfc3339(),
        });

        outcome
    }

    async fn resolve(&self, credential: &str) -> Result<Resolved, OAuthError> {
        if credential.starts_with(keys::KEY_SCHEME) {
            let (prefix, _) = keys::parse(credential).ok_or(OAuthError::InvalidGrant)?;
            let key = self
                .store
                .get_api_key(&prefix)
                .await?
                .ok_or(OAuthError::InvalidGrant)?;
            if !keys::verify(credential, &key) {
                return Err(OAuthError::InvalidGrant);
            }
            // Best-effort last-used bookkeeping.
            self.store
                .touch_api_key(&prefix, &Utc::now().to_rfc3339())
                .await?;
            Ok(Resolved {
                kind: CredentialKind::ApiKey,
                actor: key.prefix,
                account_id: key.account_id,
                expires_at: key.scope.expires_at.clone(),
                resource: None,
                scope: key.scope,
            })
        } else {
            let token = self
                .store
                .get_token(&sha256_hex(credential))
                .await?
                .ok_or(OAuthError::InvalidGrant)?;
            if token.kind != TokenKind::Access {
                return Err(OAuthError::InvalidGrant);
            }
            if token.revoked_at.is_some() {
                return Err(OAuthError::Expired);
            }
            Ok(Resolved {
                kind: CredentialKind::OAuthToken,
                actor: token.client_id,
                account_id: token.account_id,
                expires_at: Some(token.expires_at),
                resource: token.resource,
                scope: token.scope,
            })
        }
    }

    fn check_policy(
        &self,
        r: &Resolved,
        ctx: &RequestContext<'_>,
        required: &Scope,
    ) -> Result<(), OAuthError> {
        // 1. Expiry.
        if let Some(exp) = &r.expires_at
            && is_expired(exp)
        {
            return Err(OAuthError::Expired);
        }
        // 2. IP allowlist.
        if !ip_allowed(ctx.source_ip, &r.scope.ip_allowlist) {
            return Err(OAuthError::IpDenied);
        }
        // 3. Per-key rate limit.
        if let Some(limit) = r.scope.rate_limit
            && !self.rate.check(&r.actor, limit)
        {
            return Err(OAuthError::RateLimited);
        }
        // 4. Resource/audience binding (RFC 8707): a resource-bound token may only
        //    be used against the resource server it was issued for.
        if let (Some(bound), Some(want)) = (&r.resource, ctx.resource)
            && bound != want
        {
            return Err(OAuthError::InvalidScope);
        }
        // 5. Capability check — no scope escalation.
        if !r.scope.allows(required) {
            return Err(OAuthError::InvalidScope);
        }
        Ok(())
    }
}
