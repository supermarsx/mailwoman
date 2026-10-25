#![forbid(unsafe_code)]
//! `mw-dav` — the shared DAV/HTTP core plus the CalDAV surface for Mailwoman V3
//! (plan §0.1, §1.3, SPEC §6.2/§11).
//!
//! Hand-rolled over `reqwest` (rustls) + `quick-xml` — no DAV framework (plan
//! §1.3). This crate owns the **shared DAV core** that `mw-carddav` (e3) reuses
//! verbatim: discovery (`.well-known` → current-user-principal → home-set),
//! `PROPFIND`, `REPORT`, `MKCALENDAR`, `PUT`/`GET`/`DELETE` with `ETag`
//! `If-Match`/`If-None-Match`, `sync-collection` (RFC 6578) with `sync-token`,
//! and the `ctag` + etag-diff fallback (§2.3). On top of it sits the **CalDAV**
//! surface (list calendars, pull events/tasks by component, push,
//! `calendar-query`/`calendar-multiget`, `free-busy-query`). Bodies
//! (de)serialize through `mw-ics`.
//!
//! ## Shared DAV core (for `mw-carddav`, e3)
//! The reusable core is exposed as two public modules — [`request`] (pure XML
//! request-body builders, parameterised by [`request::DavKind`]) and
//! [`response`] (pure `multistatus` parsers) — plus the generic, kind-parameter
//! client methods on [`DavClient`]: [`DavClient::discover`],
//! [`DavClient::sync_collection`], [`DavClient::list_etags`],
//! [`DavClient::multiget`], [`DavClient::put_resource`],
//! [`DavClient::delete_resource`]. `mw-carddav` constructs its own
//! [`DavClient`] and drives these with [`request::DavKind::CardDav`]; the CalDAV
//! methods below (`make_calendar`, `calendar_query`, `calendar_multiget`,
//! `free_busy_query`) are the calendar-only additions.

pub mod request;
pub mod response;

use request::DavKind;
use serde::{Deserialize, Serialize};

/// A recoverable DAV/HTTP failure.
#[derive(Debug, thiserror::Error)]
pub enum DavError {
    #[error("http transport error: {0}")]
    Transport(String),
    #[error("DAV XML parse error: {0}")]
    Xml(String),
    #[error("unexpected DAV status {status}: {body}")]
    Status { status: u16, body: String },
    #[error("etag precondition failed (412) — remote changed, re-pull required")]
    Conflict,
    #[error("discovery failed: {0}")]
    Discovery(String),
    #[error("serialization error: {0}")]
    Ics(#[from] mw_ics::IcsError),
}

/// The convenience result alias for this crate.
pub type Result<T> = std::result::Result<T, DavError>;

/// The `ctag` + member-etag fallback listing: the collection `getctag` plus
/// every member `(href, etag)` the engine diffs against its stored etags when
/// `sync-collection` is unadvertised (§2.3).
pub type EtagList = (Option<String>, Vec<(String, String)>);

/// Credentials + base URL for a DAV account. The shared core is protocol-
/// agnostic; CalDAV/CardDAV layer their collection semantics on top.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct DavConfig {
    /// The server base or discovery URL (`.well-known/caldav` is tried).
    pub base_url: String,
    pub username: String,
    pub password: String,
    /// The address the base URL's host was checked against the egress policy as
    /// (t28-e19). When set, every request connects to this address only, never
    /// follows a redirect, and is refused for a URL on any other host — so a DAV
    /// server cannot point the client at a second host the policy never saw.
    /// `None`: the host is resolved per request (tests, and callers that do their
    /// own checking).
    pub pin: Option<std::net::SocketAddr>,
}

/// The host of `url` (lower-cased), if it has one.
fn host_of(url: &str) -> Option<String> {
    reqwest::Url::parse(url)
        .ok()?
        .host_str()
        .map(str::to_ascii_lowercase)
}

/// The HTTP client a DAV account's requests use: no ambient proxy (it carries the
/// account's credentials), and, with [`DavConfig::pin`], connected to the checked
/// address only and with redirects off. `mw-carddav` builds its transport with it.
pub fn http_client(config: &DavConfig) -> Result<reqwest::Client> {
    // `.no_proxy()`: this client carries account credentials to a configured base
    // URL; an ambient `HTTP_PROXY` would route them through an unrelated third
    // party. See `mw_egress::harden_client`.
    let mut builder = reqwest::Client::builder().no_proxy();
    if let Some(addr) = config.pin {
        let host = host_of(&config.base_url)
            .ok_or_else(|| DavError::Transport("the DAV base URL has no host".into()))?;
        builder = builder
            .resolve(&host, addr)
            .redirect(reqwest::redirect::Policy::none());
    }
    builder
        .build()
        .map_err(|e| DavError::Transport(e.to_string()))
}

/// Whether a request to `url` may be sent under `config`: always without a pin;
/// with one, only to the base URL's own host.
pub fn url_allowed(config: &DavConfig, url: &str) -> bool {
    config.pin.is_none() || (host_of(url).is_some() && host_of(url) == host_of(&config.base_url))
}

/// A discovered DAV collection (calendar or address book): its href plus the
/// sync capabilities the engine feature-detects on (`sync-token` vs `ctag`).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Collection {
    pub href: String,
    pub display_name: String,
    /// `calendar-color` / `addressbook` colour, if advertised.
    pub color: Option<String>,
    /// `getctag`, when the server exposes it (the fallback sync key).
    pub ctag: Option<String>,
    /// `sync-token`, when the server advertises RFC 6578 sync-collection.
    pub sync_token: Option<String>,
    /// Supported component set (`VEVENT` / `VTODO`) for a calendar collection.
    pub components: Vec<String>,
}

/// One remote resource in a collection: its href + `ETag` + body bytes. Bodies
/// are opaque here; `mw-ics` parses them at the engine seam.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Resource {
    pub href: String,
    pub etag: Option<String>,
    /// The iCalendar / vCard body (absent for a delete-tombstone href).
    pub body: Option<String>,
}

/// The result of an incremental `sync-collection` (RFC 6578) pull: the new
/// `sync-token` plus the changed and removed hrefs (§2.3).
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct SyncDelta {
    pub new_sync_token: Option<String>,
    pub changed: Vec<Resource>,
    pub removed: Vec<String>,
}

/// Map a bare HTTP status to a core result, folding `412` to
/// [`DavError::Conflict`] (the etag precondition-failed path, §2.3).
fn check_status(status: u16, body: &str) -> Result<()> {
    match status {
        200..=299 => Ok(()),
        412 => Err(DavError::Conflict),
        _ => Err(DavError::Status {
            status,
            body: body.to_string(),
        }),
    }
}

/// Resolve `href` against `base` into an absolute URL. Absolute hrefs (which
/// Google returns) pass through; root-relative hrefs are joined to the base
/// origin; anything else is appended to the base path.
fn resolve(base: &str, href: &str) -> String {
    if href.starts_with("http://") || href.starts_with("https://") {
        return href.to_string();
    }
    if let Some(scheme_end) = base.find("://") {
        let after = &base[scheme_end + 3..];
        let origin_len = scheme_end + 3 + after.find('/').unwrap_or(after.len());
        let origin = &base[..origin_len];
        if href.starts_with('/') {
            return format!("{origin}{href}");
        }
    }
    format!(
        "{}/{}",
        base.trim_end_matches('/'),
        href.trim_start_matches('/')
    )
}

/// A completed DAV/HTTP response reduced to what the core parsers need.
struct DavResponse {
    status: u16,
    etag: Option<String>,
    body: String,
}

/// The shared DAV/HTTP client (plan §1.3). `mw-carddav` constructs its own
/// surface over the same core methods.
pub struct DavClient {
    config: DavConfig,
    http: reqwest::Client,
}

impl DavClient {
    /// Construct a DAV client for an account (rustls `reqwest`).
    pub fn new(config: DavConfig) -> Result<Self> {
        let http = http_client(&config)?;
        Ok(Self { config, http })
    }

    /// Issue a raw DAV request (custom method, optional `Depth`, optional XML
    /// body) with HTTP basic auth, returning the status, `ETag`, and body text.
    async fn send(
        &self,
        method: &str,
        url: &str,
        depth: Option<&str>,
        extra_headers: &[(&str, &str)],
        body: Option<String>,
    ) -> Result<DavResponse> {
        self.send_typed(
            method,
            url,
            depth,
            extra_headers,
            body.map(|b| ("application/xml; charset=utf-8", b)),
        )
        .await
    }

    /// [`DavClient::send`] with the body's `Content-Type` named by the caller (a
    /// scheduling POST is `text/calendar`, not XML).
    async fn send_typed(
        &self,
        method: &str,
        url: &str,
        depth: Option<&str>,
        extra_headers: &[(&str, &str)],
        body: Option<(&str, String)>,
    ) -> Result<DavResponse> {
        if !url_allowed(&self.config, url) {
            return Err(DavError::Transport(
                "refused: the DAV server named a URL on another host".into(),
            ));
        }
        let method = reqwest::Method::from_bytes(method.as_bytes())
            .map_err(|e| DavError::Transport(e.to_string()))?;
        let mut req = self
            .http
            .request(method, url)
            .basic_auth(&self.config.username, Some(&self.config.password));
        if let Some(d) = depth {
            req = req.header("Depth", d);
        }
        for (k, v) in extra_headers {
            req = req.header(*k, *v);
        }
        if let Some((content_type, b)) = body {
            req = req.header("Content-Type", content_type).body(b);
        }
        let resp = req
            .send()
            .await
            .map_err(|e| DavError::Transport(e.to_string()))?;
        let status = resp.status().as_u16();
        let etag = resp
            .headers()
            .get(reqwest::header::ETAG)
            .and_then(|v| v.to_str().ok())
            .map(str::to_string);
        let body = resp
            .text()
            .await
            .map_err(|e| DavError::Transport(e.to_string()))?;
        Ok(DavResponse { status, etag, body })
    }

    // ── shared DAV core (reused by mw-carddav) ──────────────────────────────

    /// Discover the principal's collection home-set and enumerate collections
    /// of `kind` (`.well-known` → current-user-principal → home-set →
    /// Depth:1 `PROPFIND`, §2.3). Returned hrefs are absolute.
    pub async fn discover(&self, kind: DavKind) -> Result<Vec<Collection>> {
        let start = resolve(&self.config.base_url, kind.well_known());
        let r = self
            .send(
                "PROPFIND",
                &start,
                Some("0"),
                &[],
                Some(request::propfind_current_user_principal()),
            )
            .await?;
        let principal = response::parse_current_user_principal(&r.body)?
            .ok_or_else(|| DavError::Discovery("no current-user-principal".into()))?;

        let principal_url = resolve(&self.config.base_url, &principal);
        let r = self
            .send(
                "PROPFIND",
                &principal_url,
                Some("0"),
                &[],
                Some(request::propfind_home_set(kind)),
            )
            .await?;
        let home = response::parse_home_set(&r.body, kind)?
            .ok_or_else(|| DavError::Discovery("no home-set".into()))?;

        let home_url = resolve(&self.config.base_url, &home);
        let r = self
            .send(
                "PROPFIND",
                &home_url,
                Some("1"),
                &[],
                Some(request::propfind_collections()),
            )
            .await?;
        let mut cols = response::parse_collections(&r.body, kind)?;
        for c in &mut cols {
            c.href = resolve(&self.config.base_url, &c.href);
        }
        Ok(cols)
    }

    /// Incremental pull via `sync-collection` (RFC 6578) from `sync_token` (an
    /// empty/`None` token requests the initial full enumeration). Returns the
    /// changed/removed hrefs + new token; bodies are then fetched via
    /// [`DavClient::multiget`]. Use [`DavClient::list_etags`] for the ctag +
    /// etag-diff fallback when a server does not advertise sync-collection.
    pub async fn sync_collection(
        &self,
        collection_href: &str,
        sync_token: Option<&str>,
    ) -> Result<SyncDelta> {
        let url = resolve(&self.config.base_url, collection_href);
        let r = self
            .send(
                "REPORT",
                &url,
                Some("1"),
                &[],
                Some(request::report_sync_collection(sync_token)),
            )
            .await?;
        check_status(r.status, &r.body)?;
        response::parse_sync_delta(&r.body)
    }

    /// The `ctag` + member-etag fallback pull: a Depth:1 `PROPFIND` returning
    /// the collection `getctag` plus every member `(href, etag)`, which the
    /// engine diffs against its stored etags when `sync-collection` is
    /// unadvertised (§2.3).
    pub async fn list_etags(&self, collection_href: &str) -> Result<EtagList> {
        let url = resolve(&self.config.base_url, collection_href);
        let r = self
            .send(
                "PROPFIND",
                &url,
                Some("1"),
                &[],
                Some(request::propfind_etag_list()),
            )
            .await?;
        check_status(r.status, &r.body)?;
        response::parse_etag_list(&r.body)
    }

    /// Generic multiget of resource bodies for `kind` over a set of hrefs
    /// (`calendar-multiget` / `addressbook-multiget`, §2.3).
    pub async fn multiget(
        &self,
        kind: DavKind,
        collection_href: &str,
        hrefs: &[String],
    ) -> Result<Vec<Resource>> {
        if hrefs.is_empty() {
            return Ok(Vec::new());
        }
        let url = resolve(&self.config.base_url, collection_href);
        let r = self
            .send(
                "REPORT",
                &url,
                Some("1"),
                &[],
                Some(request::multiget(kind, hrefs)),
            )
            .await?;
        check_status(r.status, &r.body)?;
        response::parse_multiget(&r.body, kind)
    }

    /// `GET` a single resource body (returns the body + `ETag`).
    pub async fn get_resource(&self, href: &str) -> Result<Resource> {
        let url = resolve(&self.config.base_url, href);
        let r = self.send("GET", &url, None, &[], None).await?;
        check_status(r.status, &r.body)?;
        Ok(Resource {
            href: href.to_string(),
            etag: r.etag,
            body: Some(r.body),
        })
    }

    /// `PUT` a resource with `If-Match:<etag>` (update) or `If-None-Match:*`
    /// (create); a `412` maps to [`DavError::Conflict`] (§2.3). Returns the new
    /// `ETag` when the server supplies one.
    pub async fn put_resource(
        &self,
        href: &str,
        body: &str,
        if_match: Option<&str>,
    ) -> Result<Option<String>> {
        let url = resolve(&self.config.base_url, href);
        let cond = match if_match {
            Some(etag) => [("If-Match", etag)],
            None => [("If-None-Match", "*")],
        };
        let r = self
            .send("PUT", &url, None, &cond, Some(body.to_string()))
            .await?;
        check_status(r.status, &r.body)?;
        Ok(r.etag)
    }

    /// `DELETE` a resource with `If-Match:<etag>` (§2.3); `412` ⇒ conflict.
    pub async fn delete_resource(&self, href: &str, if_match: Option<&str>) -> Result<()> {
        let url = resolve(&self.config.base_url, href);
        let headers: Vec<(&str, &str)> = match if_match {
            Some(etag) => vec![("If-Match", etag)],
            None => vec![],
        };
        let r = self.send("DELETE", &url, None, &headers, None).await?;
        check_status(r.status, &r.body)
    }

    // ── CalDAV surface ──────────────────────────────────────────────────────

    /// Discover CalDAV calendar collections (`.well-known/caldav`, §2.3).
    pub async fn discover_calendars(&self) -> Result<Vec<Collection>> {
        self.discover(DavKind::CalDav).await
    }

    /// `MKCALENDAR` a new calendar collection with a display name (§2.3).
    pub async fn make_calendar(&self, href: &str, display_name: &str) -> Result<()> {
        let url = resolve(&self.config.base_url, href);
        let r = self
            .send(
                "MKCALENDAR",
                &url,
                None,
                &[],
                Some(request::mkcalendar(display_name)),
            )
            .await?;
        check_status(r.status, &r.body)
    }

    /// `calendar-query` a collection for one component type (`VEVENT`/`VTODO`)
    /// over an optional UTC window, returning matching hrefs + etags (RFC 4791,
    /// §2.3). Bodies are then fetched via [`DavClient::calendar_multiget`].
    pub async fn calendar_query(
        &self,
        collection_href: &str,
        component: &str,
        window: Option<(&str, &str)>,
    ) -> Result<Vec<Resource>> {
        let url = resolve(&self.config.base_url, collection_href);
        let r = self
            .send(
                "REPORT",
                &url,
                Some("1"),
                &[],
                Some(request::calendar_query(component, window)),
            )
            .await?;
        check_status(r.status, &r.body)?;
        response::parse_resource_list(&r.body)
    }

    /// `calendar-multiget` bodies for a set of hrefs (RFC 4791, §2.3).
    pub async fn calendar_multiget(
        &self,
        collection_href: &str,
        hrefs: &[String],
    ) -> Result<Vec<Resource>> {
        self.multiget(DavKind::CalDav, collection_href, hrefs).await
    }

    /// `free-busy-query` a collection over a window, returning merged busy
    /// intervals (RFC 4791, feeds `Calendar/freeBusy`, §2.2).
    pub async fn free_busy_query(
        &self,
        collection_href: &str,
        window_start: &str,
        window_end: &str,
    ) -> Result<Vec<mw_ics::BusyInterval>> {
        let url = resolve(&self.config.base_url, collection_href);
        let r = self
            .send(
                "REPORT",
                &url,
                Some("1"),
                &[],
                Some(request::free_busy_query(window_start, window_end)),
            )
            .await?;
        check_status(r.status, &r.body)?;
        response::parse_free_busy(&r.body)
    }

    /// Ask the server for other people's free/busy (RFC 6638 §5): find the
    /// signed-in principal's `schedule-outbox-URL` and POST a `VFREEBUSY`
    /// request naming `attendees`. Returns one [`FreeBusyReply`] per recipient
    /// the server answered for. `start`/`end` are RFC 3339 UTC.
    ///
    /// [`DavError::Discovery`] when the server advertises no outbox — it does not
    /// do scheduling, and the caller reports those addresses as unknown.
    pub async fn schedule_free_busy(
        &self,
        organizer: &str,
        attendees: &[String],
        start: &str,
        end: &str,
    ) -> Result<Vec<FreeBusyReply>> {
        let start_url = resolve(&self.config.base_url, DavKind::CalDav.well_known());
        let r = self
            .send(
                "PROPFIND",
                &start_url,
                Some("0"),
                &[],
                Some(request::propfind_current_user_principal()),
            )
            .await?;
        let principal = response::parse_current_user_principal(&r.body)?
            .ok_or_else(|| DavError::Discovery("no current-user-principal".into()))?;
        let principal_url = resolve(&self.config.base_url, &principal);
        let r = self
            .send(
                "PROPFIND",
                &principal_url,
                Some("0"),
                &[],
                Some(request::propfind_schedule_outbox()),
            )
            .await?;
        let outbox = response::parse_schedule_outbox(&r.body)?
            .ok_or_else(|| DavError::Discovery("no schedule-outbox-URL".into()))?;
        let outbox_url = resolve(&self.config.base_url, &outbox);
        let stamp = |rfc: &str| rfc.replace(['-', ':'], "");
        let now = chrono_like_now();
        let body = request::free_busy_request(
            organizer,
            attendees,
            &stamp(start),
            &stamp(end),
            &format!("mw-fb-{now}"),
            &now,
        );
        let r = self
            .send_typed(
                "POST",
                &outbox_url,
                None,
                &[],
                Some(("text/calendar; charset=utf-8; method=REQUEST", body)),
            )
            .await?;
        check_status(r.status, &r.body)?;
        let replies = response::parse_schedule_response(&r.body)?;
        Ok(replies
            .into_iter()
            .map(|reply| {
                let busy = if reply.ok() {
                    reply
                        .calendar_data
                        .as_deref()
                        .and_then(|ics| response::parse_free_busy(ics).ok())
                } else {
                    None
                };
                FreeBusyReply {
                    recipient: reply.recipient,
                    busy,
                }
            })
            .collect())
    }
}

/// One recipient's free/busy from [`DavClient::schedule_free_busy`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FreeBusyReply {
    /// The address, lower-cased, without `mailto:`.
    pub recipient: String,
    /// The busy blocks, or `None` when the server could not answer for this
    /// address (unknown user, no permission).
    pub busy: Option<Vec<mw_ics::BusyInterval>>,
}

/// The current UTC time as an iCalendar basic-format stamp (`20260701T090000Z`),
/// without a date crate: seconds since the epoch, converted by the civil-from-days
/// algorithm.
fn chrono_like_now() -> String {
    let secs = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0);
    let days = secs.div_euclid(86_400);
    let rem = secs.rem_euclid(86_400);
    // Howard Hinnant's civil_from_days.
    let z = days + 719_468;
    let era = z.div_euclid(146_097);
    let doe = z.rem_euclid(146_097);
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let y = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    let y = if m <= 2 { y + 1 } else { y };
    format!(
        "{y:04}{m:02}{d:02}T{:02}{:02}{:02}Z",
        rem / 3600,
        (rem % 3600) / 60,
        rem % 60
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn status_412_is_conflict() {
        assert!(matches!(check_status(412, ""), Err(DavError::Conflict)));
    }

    #[test]
    fn status_2xx_ok_others_error() {
        assert!(check_status(201, "").is_ok());
        assert!(check_status(207, "").is_ok());
        assert!(matches!(
            check_status(404, "gone"),
            Err(DavError::Status { status: 404, .. })
        ));
    }

    #[test]
    fn resolve_absolute_root_relative_and_path() {
        assert_eq!(
            resolve("https://dav.example.com/base/", "https://x.test/cal/1.ics"),
            "https://x.test/cal/1.ics"
        );
        assert_eq!(
            resolve("https://dav.example.com/base/", "/cal/1.ics"),
            "https://dav.example.com/cal/1.ics"
        );
        assert_eq!(
            resolve("https://dav.example.com/base", "sub/1.ics"),
            "https://dav.example.com/base/sub/1.ics"
        );
    }
}
