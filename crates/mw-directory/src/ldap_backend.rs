//! The low-level LDAP connection seam (`LdapBackend`) + its real `ldap3` (rustls)
//! implementation, plus the pure helpers (`tls_plan`, filter escaping, attr
//! mapping) that carry the search/paging/merge logic in [`crate`].
//!
//! Splitting the connection behind a trait keeps the orchestration in `lib.rs`
//! (priority merge, paging, group-expand, cert/photo extraction, bind decision,
//! caching) **unit-testable against an in-memory mock** — no live server. The real
//! `ldap3` path (connect/StartTLS/bind/search) is thin here and is exercised
//! end-to-end against real OpenLDAP in e16.

use std::collections::HashMap;

use async_trait::async_trait;

use crate::{AttrMap, DirectoryError, LdapEndpoint, LdapTls, Result};

/// A raw LDAP entry as returned by a search: text attrs + binary attrs (certs,
/// photos) keyed by attribute name.
#[derive(Debug, Clone, Default)]
pub(crate) struct RawEntry {
    pub dn: String,
    pub attrs: HashMap<String, Vec<String>>,
    pub bin_attrs: HashMap<String, Vec<Vec<u8>>>,
}

impl RawEntry {
    /// First text value for `attr` (case-insensitive attribute name).
    pub(crate) fn first(&self, attr: &str) -> Option<&str> {
        self.attrs
            .iter()
            .find(|(k, _)| k.eq_ignore_ascii_case(attr))
            .and_then(|(_, v)| v.first())
            .map(String::as_str)
    }

    /// All binary values for `attr` or its base name (strips a `;binary` option),
    /// matched case-insensitively — AD returns `userCertificate` in `bin_attrs`.
    pub(crate) fn bin(&self, attr: &str) -> Vec<Vec<u8>> {
        let base = attr.split(';').next().unwrap_or(attr);
        let mut out = Vec::new();
        for (k, v) in &self.bin_attrs {
            let kbase = k.split(';').next().unwrap_or(k);
            if k.eq_ignore_ascii_case(attr) || kbase.eq_ignore_ascii_case(base) {
                out.extend(v.iter().cloned());
            }
        }
        out
    }
}

/// One search request against a single endpoint.
pub(crate) struct SearchReq<'a> {
    pub base: &'a str,
    /// `true` = subtree scope; `false` = base-object scope (single entry).
    pub subtree: bool,
    pub filter: &'a str,
    pub attrs: Vec<String>,
    /// Server-side page chunk size (RFC 2696 paged results).
    pub page_size: u32,
    /// Stop once this many entries are collected (bounds a wide GAL search).
    pub limit: usize,
    /// Optional service bind `(dn, password)` before the search; `None` = anon.
    pub service_bind: Option<(&'a str, &'a str)>,
}

/// The connection seam. The real impl is [`Ldap3Backend`]; tests inject a mock.
#[async_trait]
pub(crate) trait LdapBackend: Send + Sync {
    async fn search(&self, req: SearchReq<'_>) -> Result<Vec<RawEntry>>;
    /// Simple-bind as `dn` with `password`; `Ok(true)` = credentials accepted.
    async fn bind(&self, dn: &str, password: &str) -> Result<bool>;
}

/// How an endpoint's `tls` maps onto a connection plan. `ldaps` is also implied by
/// an `ldaps://` URL scheme; `starttls` is negotiated on a plain `ldap://` port.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct TlsPlan {
    pub starttls: bool,
    pub ldaps: bool,
}

/// Pure mapping from an endpoint's TLS mode to a connection plan (unit-tested;
/// the negotiation itself is a live-server concern, e16).
pub(crate) fn tls_plan(ep: &LdapEndpoint) -> TlsPlan {
    let scheme_ldaps = ep
        .url
        .trim_start()
        .to_ascii_lowercase()
        .starts_with("ldaps://");
    match ep.tls {
        LdapTls::None => TlsPlan {
            starttls: false,
            ldaps: scheme_ldaps,
        },
        LdapTls::StartTls => TlsPlan {
            starttls: true,
            ldaps: scheme_ldaps,
        },
        LdapTls::Ldaps => TlsPlan {
            starttls: false,
            ldaps: true,
        },
    }
}

/// Escape a user-supplied value for use inside an LDAP filter (RFC 4515) so a
/// query like `a)(uid=*` cannot alter the filter's structure.
pub(crate) fn escape_filter_value(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for c in s.chars() {
        match c {
            '*' => out.push_str("\\2a"),
            '(' => out.push_str("\\28"),
            ')' => out.push_str("\\29"),
            '\\' => out.push_str("\\5c"),
            '\0' => out.push_str("\\00"),
            _ => out.push(c),
        }
    }
    out
}

// ── Attribute-name resolution (deployment attr_map → concrete LDAP attr) ──────

pub(crate) fn attr_display(map: &AttrMap) -> &str {
    map.display_name.as_deref().unwrap_or("displayName")
}
pub(crate) fn attr_mail(map: &AttrMap) -> &str {
    map.mail.as_deref().unwrap_or("mail")
}
pub(crate) fn attr_member(map: &AttrMap) -> &str {
    map.member.as_deref().unwrap_or("member")
}
pub(crate) fn attr_cert(map: &AttrMap) -> &str {
    map.user_cert.as_deref().unwrap_or("userCertificate;binary")
}
pub(crate) fn attr_photo(map: &AttrMap) -> &str {
    map.photo.as_deref().unwrap_or("jpegPhoto")
}

/// The object classes that mark an entry as a distribution group.
fn looks_like_group(raw: &RawEntry, map: &AttrMap) -> bool {
    const GROUP_CLASSES: [&str; 4] = ["group", "groupofnames", "groupofuniquenames", "groupofurls"];
    if let Some(classes) = raw
        .attrs
        .iter()
        .find(|(k, _)| k.eq_ignore_ascii_case("objectClass"))
        .map(|(_, v)| v)
        && classes
            .iter()
            .any(|c| GROUP_CLASSES.contains(&c.to_ascii_lowercase().as_str()))
    {
        return true;
    }
    // A member attribute present ⇒ it is a group even without a matched class.
    raw.first(attr_member(map)).is_some()
        || raw
            .attrs
            .keys()
            .any(|k| k.eq_ignore_ascii_case(attr_member(map)))
}

/// Convert a raw entry to a GAL entry via the deployment attr map. Non-group
/// entries without a mail address are dropped (not addressable); groups are kept
/// even with no mail so they can be expanded before send.
pub(crate) fn to_gal(raw: &RawEntry, map: &AttrMap) -> Option<crate::GalEntry> {
    let is_group = looks_like_group(raw, map);
    let mail = raw.first(attr_mail(map)).unwrap_or("").to_string();
    if mail.is_empty() && !is_group {
        return None;
    }
    let display_name = raw
        .first(attr_display(map))
        .map(str::to_string)
        .or_else(|| raw.first("cn").map(str::to_string))
        .unwrap_or_else(|| {
            if mail.is_empty() {
                raw.dn.clone()
            } else {
                mail.clone()
            }
        });
    Some(crate::GalEntry {
        dn: raw.dn.clone(),
        display_name,
        mail,
        is_group,
    })
}

// ── Real ldap3 (rustls) backend ──────────────────────────────────────────────

/// The real connection backed by `ldap3` with **rustls only**. Constructed
/// eagerly (no I/O); each operation opens a fresh connection.
pub(crate) struct Ldap3Backend {
    url: String,
    plan: TlsPlan,
}

impl Ldap3Backend {
    pub(crate) fn new(ep: &LdapEndpoint) -> Self {
        Self {
            url: ep.url.clone(),
            plan: tls_plan(ep),
        }
    }

    async fn connect(&self) -> Result<ldap3::Ldap> {
        let settings = ldap3::LdapConnSettings::new().set_starttls(self.plan.starttls);
        let (conn, ldap) = ldap3::LdapConnAsync::with_settings(settings, &self.url)
            .await
            .map_err(|e| DirectoryError::Transport(e.to_string()))?;
        // Drive the connection in the background; it ends when `ldap` is dropped.
        tokio::spawn(async move {
            if let Err(e) = conn.drive().await {
                tracing::warn!(target: "mw_directory", error = %e, "ldap connection driver ended");
            }
        });
        Ok(ldap)
    }
}

#[async_trait]
impl LdapBackend for Ldap3Backend {
    async fn search(&self, req: SearchReq<'_>) -> Result<Vec<RawEntry>> {
        use ldap3::adapters::{Adapter, EntriesOnly, PagedResults};

        let mut ldap = self.connect().await?;
        if let Some((dn, pw)) = req.service_bind {
            ldap.simple_bind(dn, pw)
                .await
                .map_err(|e| DirectoryError::Auth(e.to_string()))?
                .success()
                .map_err(|e| DirectoryError::Auth(e.to_string()))?;
        }

        let scope = if req.subtree {
            ldap3::Scope::Subtree
        } else {
            ldap3::Scope::Base
        };
        let adapters: Vec<Box<dyn Adapter<String, Vec<String>>>> = vec![
            Box::new(EntriesOnly::new()),
            Box::new(PagedResults::new(req.page_size.max(1) as i32)),
        ];
        let mut stream = ldap
            .streaming_search_with(adapters, req.base, scope, req.filter, req.attrs.clone())
            .await
            .map_err(|e| DirectoryError::Protocol(e.to_string()))?;

        let mut out = Vec::new();
        while out.len() < req.limit {
            match stream.next().await {
                Ok(Some(re)) => {
                    let se = ldap3::SearchEntry::construct(re);
                    out.push(RawEntry {
                        dn: se.dn,
                        attrs: se.attrs,
                        bin_attrs: se.bin_attrs,
                    });
                }
                Ok(None) => break,
                Err(e) => return Err(DirectoryError::Protocol(e.to_string())),
            }
        }
        let _ = stream.finish().await;
        let _ = ldap.unbind().await;
        Ok(out)
    }

    async fn bind(&self, dn: &str, password: &str) -> Result<bool> {
        let mut ldap = self.connect().await?;
        let res = ldap
            .simple_bind(dn, password)
            .await
            .map_err(|e| DirectoryError::Auth(e.to_string()))?;
        let ok = res.rc == 0;
        let _ = ldap.unbind().await;
        Ok(ok)
    }
}
