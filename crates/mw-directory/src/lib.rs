#![forbid(unsafe_code)]
//! `mw-directory` — LDAP/GAL directory (plan §2.2, SPEC §13). **Read-only at 1.0.**
//!
//! GAL search over every recipient field, distribution-group read + expand-before-
//! send, **S/MIME cert lookup** (feeds `mw-crypto`'s cert path, §8.2), photo
//! attributes, paged search, StartTLS/LDAPS, **multiple directories with priority
//! order**, and an offline GAL cache (via `mw-cache::CacheClass::GalDirectory`).
//! LDAP-bind **login** (§18.3) reuses this crate's connection layer.
//!
//! `ldap3` is configured **rustls-only** (`default-features=false,
//! features=["tls-rustls-ring"]`) so **no openssl/native-tls** enters the tree
//! (deny.toml ban); the `no_openssl_in_tree` test asserts it structurally from
//! `Cargo.lock`. The `-ring` provider matches the workspace's rustls provider (no
//! aws-lc-sys C).
//!
//! ## Architecture
//! The public [`DirectorySource`] logic (priority merge, paging, group-expand,
//! cert/photo extraction, bind decision, caching) lives here over an internal
//! [`ldap_backend::LdapBackend`] seam. The seam's real impl ([`ldap_backend::Ldap3Backend`])
//! is a thin `ldap3` wrapper; a mock impl lets the whole orchestration be unit-
//! tested with **no live server**. The live OpenLDAP run is e16.

use std::collections::HashSet;
use std::sync::Arc;
use std::time::Duration;

use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use mw_cache::{Cache, CacheClass, CacheError};

mod ldap_backend;
use ldap_backend::{
    Ldap3Backend, LdapBackend, RawEntry, SearchReq, attr_cert, attr_display, attr_mail,
    attr_member, attr_photo, escape_filter_value, to_gal,
};

/// Errors surfaced by directory operations (plan §2.2).
#[derive(Debug, thiserror::Error)]
pub enum DirectoryError {
    #[error("ldap protocol error: {0}")]
    Protocol(String),
    #[error("bind/auth failed: {0}")]
    Auth(String),
    #[error("transport/TLS error: {0}")]
    Transport(String),
    #[error("no directory configured")]
    NotConfigured,
}

pub type Result<T> = std::result::Result<T, DirectoryError>;

/// DER-encoded bytes (an X.509 cert / photo blob).
pub type Der = Vec<u8>;

/// TLS mode for an LDAP endpoint (plan §2.2). rustls throughout.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum LdapTls {
    /// Plain LDAP (no TLS) — dev/on-prem only.
    None,
    /// StartTLS on the LDAP port.
    StartTls,
    /// Implicit LDAPS.
    Ldaps,
}

/// Attribute-name mapping so a deployment's schema maps onto GAL fields.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct AttrMap {
    pub display_name: Option<String>,
    pub mail: Option<String>,
    pub member: Option<String>,
    pub user_cert: Option<String>,
    pub photo: Option<String>,
}

/// One LDAP endpoint in a priority-ordered directory list (plan §2.2).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LdapEndpoint {
    pub url: String,
    pub base_dn: String,
    pub bind_dn: Option<String>,
    pub tls: LdapTls,
    /// Lower = queried first; results merge in priority order.
    pub priority: i32,
    #[serde(default)]
    pub attr_map: AttrMap,
}

/// Directory config: an ordered set of endpoints merged by priority (plan §2.2).
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct DirectoryConfig {
    pub endpoints: Vec<LdapEndpoint>,
}

/// A resolved GAL entry (plan §2.2).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GalEntry {
    pub dn: String,
    pub display_name: String,
    pub mail: String,
    /// Whether this entry is a distribution group (expandable).
    #[serde(default)]
    pub is_group: bool,
}

/// The outcome of an LDAP-bind authentication (plan §2.2/§18.3).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum BindOutcome {
    Ok { dn: String },
    Denied,
}

/// Runtime knobs owned by this crate (NOT part of the frozen serde
/// [`DirectoryConfig`], which stays byte-stable for `mw-server`/e9).
///
/// * `page_size` — entries per page for [`DirectorySource::search_gal`] and the
///   server-side paged-results chunk.
/// * `cache_ttl` — GAL cache refresh horizon. **Zero disables the cache** (always
///   fresh); a non-zero value enables caching. Note the cache tier's own TTL is
///   governed by `mw-cache`'s `GalDirectory` matrix (3600 s default) — this knob
///   is the crate-level on/off + refresh policy.
/// * `expand_depth` — max nested-group recursion for expand-before-send.
#[derive(Debug, Clone, Copy)]
pub struct DirectoryOptions {
    pub page_size: u32,
    pub cache_ttl: Duration,
    pub expand_depth: u32,
}

impl Default for DirectoryOptions {
    fn default() -> Self {
        Self {
            page_size: 50,
            cache_ttl: Duration::from_secs(3_600),
            expand_depth: 10,
        }
    }
}

/// The read-only directory seam (plan §2.2). [`Directory`] backs this with
/// `ldap3` over the priority-ordered [`DirectoryConfig`], caching via
/// `mw-cache::GalDirectory`.
#[async_trait]
pub trait DirectorySource: Send + Sync {
    /// GAL search across every recipient field; `page` is a 0-based page index.
    async fn search_gal(&self, query: &str, page: u32) -> Result<Vec<GalEntry>>;
    /// Expand a distribution group DN to its members (recursively upstream).
    async fn expand_group(&self, dn: &str) -> Result<Vec<GalEntry>>;
    /// S/MIME certificate lookup for a recipient (feeds mw-crypto §8.2).
    async fn lookup_cert(&self, email: &str) -> Result<Vec<Der>>;
    /// Photo attribute for a recipient.
    async fn lookup_photo(&self, email: &str) -> Result<Option<Der>>;
    /// LDAP-bind authentication (§18.3 login backend).
    async fn bind_auth(&self, user: &str, pass: &str) -> Result<BindOutcome>;
}

/// One configured endpoint paired with its live connection backend.
struct EndpointRuntime {
    endpoint: LdapEndpoint,
    backend: Arc<dyn LdapBackend>,
    /// Service-bind password for search binds. `None` ⇒ anonymous search bind
    /// (the frozen config carries no password field; inject via
    /// [`Directory::with_service_password`]).
    service_bind_pw: Option<String>,
}

/// The concrete multi-directory client (plan §2.2).
pub struct Directory {
    config: DirectoryConfig,
    options: DirectoryOptions,
    cache: Option<Cache>,
    runtimes: Vec<EndpointRuntime>,
}

impl Directory {
    /// Build a directory over a priority-ordered config with default options and
    /// no cache. Endpoints are sorted by ascending `priority` (lower first).
    #[must_use]
    pub fn new(config: DirectoryConfig) -> Self {
        Self::with_options(config, DirectoryOptions::default(), None)
    }

    /// Build a directory with explicit options and an optional GAL cache.
    #[must_use]
    pub fn with_options(
        config: DirectoryConfig,
        options: DirectoryOptions,
        cache: Option<Cache>,
    ) -> Self {
        let mut sorted = config.endpoints.clone();
        sorted.sort_by_key(|e| e.priority);
        let runtimes = sorted
            .into_iter()
            .map(|ep| EndpointRuntime {
                backend: Arc::new(Ldap3Backend::new(&ep)) as Arc<dyn LdapBackend>,
                endpoint: ep,
                service_bind_pw: None,
            })
            .collect();
        Self {
            config,
            options,
            cache,
            runtimes,
        }
    }

    /// Provide a service-bind password for the endpoint(s) whose `bind_dn` equals
    /// `bind_dn` (the frozen serde config has no secret field, so passwords are
    /// injected out-of-band from sealed store credentials by the caller). Without
    /// this, search binds are anonymous.
    pub fn with_service_password(mut self, bind_dn: &str, password: impl Into<String>) -> Self {
        let password = password.into();
        for rt in &mut self.runtimes {
            if rt.endpoint.bind_dn.as_deref() == Some(bind_dn) {
                rt.service_bind_pw = Some(password.clone());
            }
        }
        self
    }

    /// The configured endpoints (priority order).
    #[must_use]
    pub fn config(&self) -> &DirectoryConfig {
        &self.config
    }

    /// Whether a GAL cache is attached and enabled (non-zero `cache_ttl`).
    fn cache_enabled(&self) -> bool {
        self.cache.is_some() && !self.options.cache_ttl.is_zero()
    }

    /// Cache-aside wrapper: on a hit returns the cached value; on a miss runs
    /// `fetch`, caches, and returns. When caching is disabled `fetch` runs
    /// directly. Fetch errors are never cached.
    async fn cached<T, F, Fut>(&self, key: String, fetch: F) -> Result<T>
    where
        T: Serialize + for<'de> Deserialize<'de>,
        F: FnOnce() -> Fut,
        Fut: std::future::Future<Output = Result<T>>,
    {
        if self.cache_enabled() {
            let cache = self.cache.as_ref().expect("cache_enabled");
            cache
                .get(CacheClass::GalDirectory, &key, || async {
                    fetch().await.map_err(|e| CacheError::Store(e.to_string()))
                })
                .await
                .map_err(map_cache_err)
        } else {
            fetch().await
        }
    }

    /// Read a single entry by DN across endpoints (base-object scope), returning
    /// the first match with its owning endpoint index (for attr-map context).
    async fn read_entry(&self, dn: &str, attrs: Vec<String>) -> Option<(usize, RawEntry)> {
        for (idx, rt) in self.runtimes.iter().enumerate() {
            let req = SearchReq {
                base: dn,
                subtree: false,
                filter: "(objectClass=*)",
                attrs: attrs.clone(),
                page_size: 1,
                limit: 1,
                service_bind: service_bind(rt),
            };
            if let Ok(mut entries) = rt.backend.search(req).await
                && let Some(e) = entries.drain(..).next()
            {
                return Some((idx, e));
            }
        }
        None
    }

    // ── Uncached fetch implementations ───────────────────────────────────────

    async fn fetch_gal(&self, query: &str, page: u32) -> Result<Vec<GalEntry>> {
        let page_size = self.options.page_size.max(1) as usize;
        let start = (page as usize) * page_size;
        // Fetch enough to fill the requested page after cross-endpoint dedup.
        let per_endpoint_limit = start + page_size + page_size;
        let esc = escape_filter_value(query);

        let mut merged: Vec<GalEntry> = Vec::new();
        let mut seen: HashSet<String> = HashSet::new();
        for rt in &self.runtimes {
            let map = &rt.endpoint.attr_map;
            let filter = format!(
                "(|({d}=*{q}*)({m}=*{q}*)(cn=*{q}*)(givenName=*{q}*)(sn=*{q}*)(sAMAccountName=*{q}*))",
                d = attr_display(map),
                m = attr_mail(map),
                q = esc,
            );
            let req = SearchReq {
                base: &rt.endpoint.base_dn,
                subtree: true,
                filter: &filter,
                attrs: gal_attrs(map),
                page_size: self.options.page_size.max(1),
                limit: per_endpoint_limit.max(1),
                service_bind: service_bind(rt),
            };
            let entries = rt.backend.search(req).await?;
            for raw in &entries {
                if let Some(g) = to_gal(raw, map) {
                    // Priority merge: the earlier (lower-priority-number) endpoint
                    // wins on a duplicate mail/dn.
                    let dk = dedup_key(&g);
                    if seen.insert(dk) {
                        merged.push(g);
                    }
                }
            }
        }
        // Stable page slice over the priority-ordered merge.
        Ok(merged.into_iter().skip(start).take(page_size).collect())
    }

    async fn fetch_expand(&self, dn: &str) -> Result<Vec<GalEntry>> {
        let mut result: Vec<GalEntry> = Vec::new();
        let mut seen: HashSet<String> = HashSet::new();
        let mut visited: HashSet<String> = HashSet::new();
        // (dn, depth)
        let mut stack: Vec<(String, u32)> = vec![(dn.to_string(), 0)];
        const TOTAL_CAP: usize = 5_000;

        while let Some((cur, depth)) = stack.pop() {
            if !visited.insert(cur.to_ascii_lowercase()) {
                continue;
            }
            if depth > self.options.expand_depth || result.len() >= TOTAL_CAP {
                continue;
            }
            let Some((idx, entry)) = self.read_entry(&cur, group_attrs()).await else {
                continue;
            };
            let map = &self.runtimes[idx].endpoint.attr_map;
            // Collect member DNs (member / uniqueMember / memberUid-as-dn).
            let mut members: Vec<String> = Vec::new();
            for a in [attr_member(map), "uniqueMember", "member"] {
                if let Some(vals) = entry
                    .attrs
                    .iter()
                    .find(|(k, _)| k.eq_ignore_ascii_case(a))
                    .map(|(_, v)| v)
                {
                    members.extend(vals.iter().cloned());
                }
            }
            for m in members {
                // Resolve each member entry to classify person vs nested group.
                let Some((midx, mraw)) = self.read_entry(&m, group_attrs()).await else {
                    continue;
                };
                let mmap = &self.runtimes[midx].endpoint.attr_map;
                if let Some(g) = to_gal(&mraw, mmap) {
                    if g.is_group {
                        stack.push((g.dn.clone(), depth + 1));
                    } else if seen.insert(dedup_key(&g)) {
                        result.push(g);
                    }
                }
            }
        }
        Ok(result)
    }

    // ── Bind DN resolution ───────────────────────────────────────────────────

    async fn resolve_user_dn(&self, rt: &EndpointRuntime, user: &str) -> Option<String> {
        if user.contains('=') {
            // Already a DN.
            return Some(user.to_string());
        }
        let map = &rt.endpoint.attr_map;
        let esc = escape_filter_value(user);
        let filter = format!(
            "(|({m}={q})(uid={q})(sAMAccountName={q})(userPrincipalName={q}))",
            m = attr_mail(map),
            q = esc,
        );
        let req = SearchReq {
            base: &rt.endpoint.base_dn,
            subtree: true,
            filter: &filter,
            attrs: vec!["1.1".to_string()],
            page_size: 2,
            limit: 2,
            service_bind: service_bind(rt),
        };
        let entries = rt.backend.search(req).await.ok()?;
        // Exactly one match resolves an unambiguous DN.
        if entries.len() == 1 {
            Some(entries[0].dn.clone())
        } else {
            None
        }
    }
}

#[async_trait]
impl DirectorySource for Directory {
    async fn search_gal(&self, query: &str, page: u32) -> Result<Vec<GalEntry>> {
        if self.runtimes.is_empty() {
            return Err(DirectoryError::NotConfigured);
        }
        let key = format!("gal:{}:{}:{}", page, self.options.page_size, query);
        self.cached(key, || self.fetch_gal(query, page)).await
    }

    async fn expand_group(&self, dn: &str) -> Result<Vec<GalEntry>> {
        if self.runtimes.is_empty() {
            return Err(DirectoryError::NotConfigured);
        }
        let key = format!("grp:{dn}");
        self.cached(key, || self.fetch_expand(dn)).await
    }

    async fn lookup_cert(&self, email: &str) -> Result<Vec<Der>> {
        if self.runtimes.is_empty() {
            return Err(DirectoryError::NotConfigured);
        }
        let esc = escape_filter_value(email);
        let mut out: Vec<Der> = Vec::new();
        let mut seen: HashSet<Vec<u8>> = HashSet::new();
        for rt in &self.runtimes {
            let map = &rt.endpoint.attr_map;
            let cert = attr_cert(map);
            let filter = format!("({m}={q})", m = attr_mail(map), q = esc);
            let req = SearchReq {
                base: &rt.endpoint.base_dn,
                subtree: true,
                filter: &filter,
                attrs: vec![cert.to_string()],
                page_size: 8,
                limit: 8,
                service_bind: service_bind(rt),
            };
            let entries = rt.backend.search(req).await?;
            for raw in &entries {
                for der in raw.bin(cert) {
                    if seen.insert(der.clone()) {
                        out.push(der);
                    }
                }
            }
        }
        Ok(out)
    }

    async fn lookup_photo(&self, email: &str) -> Result<Option<Der>> {
        if self.runtimes.is_empty() {
            return Err(DirectoryError::NotConfigured);
        }
        let esc = escape_filter_value(email);
        for rt in &self.runtimes {
            let map = &rt.endpoint.attr_map;
            let photo = attr_photo(map);
            let filter = format!("({m}={q})", m = attr_mail(map), q = esc);
            let req = SearchReq {
                base: &rt.endpoint.base_dn,
                subtree: true,
                filter: &filter,
                attrs: vec![photo.to_string()],
                page_size: 1,
                limit: 1,
                service_bind: service_bind(rt),
            };
            let entries = rt.backend.search(req).await?;
            if let Some(raw) = entries.first()
                && let Some(bytes) = raw.bin(photo).into_iter().next()
            {
                return Ok(Some(bytes));
            }
        }
        Ok(None)
    }

    async fn bind_auth(&self, user: &str, pass: &str) -> Result<BindOutcome> {
        if self.runtimes.is_empty() {
            return Err(DirectoryError::NotConfigured);
        }
        // SECURITY: reject an empty password. LDAP treats a non-empty DN with an
        // empty password as an "unauthenticated bind" that many servers accept as
        // anonymous — that would be an auth bypass. Never allow it.
        if pass.is_empty() {
            return Ok(BindOutcome::Denied);
        }
        for rt in &self.runtimes {
            let Some(dn) = self.resolve_user_dn(rt, user).await else {
                continue;
            };
            match rt.backend.bind(&dn, pass).await {
                Ok(true) => return Ok(BindOutcome::Ok { dn }),
                Ok(false) => continue,
                Err(_) => continue,
            }
        }
        Ok(BindOutcome::Denied)
    }
}

// ── Free helpers ─────────────────────────────────────────────────────────────

fn service_bind(rt: &EndpointRuntime) -> Option<(&str, &str)> {
    match (
        rt.endpoint.bind_dn.as_deref(),
        rt.service_bind_pw.as_deref(),
    ) {
        (Some(dn), Some(pw)) => Some((dn, pw)),
        _ => None,
    }
}

fn dedup_key(g: &GalEntry) -> String {
    if g.mail.is_empty() {
        format!("dn:{}", g.dn.to_ascii_lowercase())
    } else {
        format!("mail:{}", g.mail.to_ascii_lowercase())
    }
}

fn gal_attrs(map: &AttrMap) -> Vec<String> {
    vec![
        attr_display(map).to_string(),
        attr_mail(map).to_string(),
        attr_member(map).to_string(),
        "cn".to_string(),
        "objectClass".to_string(),
    ]
}

fn group_attrs() -> Vec<String> {
    vec![
        "displayName".to_string(),
        "mail".to_string(),
        "member".to_string(),
        "uniqueMember".to_string(),
        "cn".to_string(),
        "objectClass".to_string(),
    ]
}

fn map_cache_err(e: CacheError) -> DirectoryError {
    DirectoryError::Protocol(e.to_string())
}

#[cfg(test)]
mod tests;
