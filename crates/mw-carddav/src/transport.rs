//! The async DAV/HTTP transport for the CardDAV REPORTs (plan §1.3/§2.3).
//!
//! Rides an in-tree `reqwest` (rustls) client directly — the same primitive
//! `mw-dav`'s shared core uses — issuing the CardDAV `PROPFIND`/`REPORT`/`PUT`/
//! `DELETE` requests with basic auth. [`CardDavClient`](crate::CardDavClient)
//! also keeps a live [`mw_dav::DavClient`] (see `CardDavClient::dav`) for the
//! shared account config; the CardDAV request/response shapes live in
//! `crate::request` / `crate::response`.

use mw_dav::DavConfig;
use reqwest::Method;

use crate::{Error, Result};

/// A minimal HTTP response projection: the status, the `ETag` header, and the
/// decoded body — all the CardDAV layer needs.
pub(crate) struct HttpResponse {
    pub status: u16,
    pub etag: Option<String>,
    pub body: String,
}

/// The CardDAV HTTP transport (basic-auth `reqwest`, rustls).
pub(crate) struct Http {
    client: reqwest::Client,
    config: DavConfig,
}

impl Http {
    pub(crate) fn new(config: DavConfig) -> Result<Self> {
        let client = reqwest::Client::builder()
            .build()
            .map_err(|e| Error::Transport(e.to_string()))?;
        Ok(Self { client, config })
    }

    /// Resolve a collection/resource href against the account base URL. Absolute
    /// hrefs (Google returns full `https://…` URLs) replace the base; absolute
    /// paths (Radicale returns `/addressbooks/…`) replace the base path.
    fn resolve(&self, href: &str) -> Result<reqwest::Url> {
        let base = reqwest::Url::parse(&self.config.base_url)
            .map_err(|e| Error::Transport(e.to_string()))?;
        if href.is_empty() {
            return Ok(base);
        }
        base.join(href).map_err(|e| Error::Transport(e.to_string()))
    }

    fn method(token: &[u8]) -> Result<Method> {
        Method::from_bytes(token).map_err(|e| Error::Transport(e.to_string()))
    }

    #[allow(clippy::too_many_arguments)]
    async fn send(
        &self,
        method: Method,
        href: &str,
        depth: Option<&str>,
        content_type: Option<&str>,
        body: Option<String>,
        if_match: Option<&str>,
        if_none_match: Option<&str>,
    ) -> Result<HttpResponse> {
        let url = self.resolve(href)?;
        let mut req = self
            .client
            .request(method, url)
            .basic_auth(&self.config.username, Some(&self.config.password));
        if let Some(d) = depth {
            req = req.header("Depth", d);
        }
        if let Some(ct) = content_type {
            req = req.header("Content-Type", ct);
        }
        if let Some(m) = if_match {
            req = req.header("If-Match", m);
        }
        if let Some(m) = if_none_match {
            req = req.header("If-None-Match", m);
        }
        if let Some(b) = body {
            req = req.body(b);
        }
        let resp = req
            .send()
            .await
            .map_err(|e| Error::Transport(e.to_string()))?;
        let status = resp.status().as_u16();
        let etag = resp
            .headers()
            .get("ETag")
            .and_then(|v| v.to_str().ok())
            .map(str::to_string);
        let text = resp
            .text()
            .await
            .map_err(|e| Error::Transport(e.to_string()))?;
        Ok(HttpResponse {
            status,
            etag,
            body: text,
        })
    }

    /// `PROPFIND` with an explicit `Depth` and an XML body.
    pub(crate) async fn propfind(
        &self,
        href: &str,
        depth: &str,
        body: String,
    ) -> Result<HttpResponse> {
        let m = Self::method(b"PROPFIND")?;
        self.send(
            m,
            href,
            Some(depth),
            Some("application/xml; charset=utf-8"),
            Some(body),
            None,
            None,
        )
        .await
    }

    /// `REPORT` (addressbook-query / addressbook-multiget / sync-collection).
    pub(crate) async fn report(
        &self,
        href: &str,
        depth: &str,
        body: String,
    ) -> Result<HttpResponse> {
        let m = Self::method(b"REPORT")?;
        self.send(
            m,
            href,
            Some(depth),
            Some("application/xml; charset=utf-8"),
            Some(body),
            None,
            None,
        )
        .await
    }

    /// `PUT` a vCard with `If-Match:<etag>` (update) or `If-None-Match:*`
    /// (create). `412` maps to [`Error::Conflict`].
    pub(crate) async fn put(
        &self,
        href: &str,
        vcard: String,
        if_match: Option<&str>,
    ) -> Result<HttpResponse> {
        let m = Self::method(b"PUT")?;
        let if_none = if if_match.is_none() { Some("*") } else { None };
        self.send(
            m,
            href,
            None,
            Some("text/vcard; charset=utf-8"),
            Some(vcard),
            if_match,
            if_none,
        )
        .await
    }

    /// `DELETE` a resource with `If-Match:<etag>`. `412` maps to [`Error::Conflict`].
    pub(crate) async fn delete(&self, href: &str, if_match: Option<&str>) -> Result<HttpResponse> {
        let m = Self::method(b"DELETE")?;
        self.send(m, href, None, None, None, if_match, None).await
    }
}

/// Map a REPORT/PROPFIND status to an error unless it is `207 Multi-Status`
/// (or a lenient `200`).
pub(crate) fn expect_multistatus(resp: &HttpResponse) -> Result<()> {
    match resp.status {
        207 | 200 => Ok(()),
        s => Err(Error::Status {
            status: s,
            body: truncate(&resp.body),
        }),
    }
}

/// Map a write (`PUT`/`DELETE`) status: 2xx ok, `412` ⇒ conflict, else error.
pub(crate) fn expect_write(resp: &HttpResponse) -> Result<()> {
    match resp.status {
        200 | 201 | 204 => Ok(()),
        412 => Err(Error::Conflict),
        s => Err(Error::Status {
            status: s,
            body: truncate(&resp.body),
        }),
    }
}

fn truncate(body: &str) -> String {
    body.chars().take(512).collect()
}

// ── Live, env-gated integration smoke test (RADICALE_URL) ───────────────────
// Ignored by default; run against a real Radicale with:
//   RADICALE_URL=… RADICALE_USER=… RADICALE_PASS=… \
//     cargo test -p mw-carddav -- --ignored live_carddav_roundtrip
#[cfg(test)]
mod live {
    use crate::CardDavClient;
    use mw_dav::DavConfig;

    #[tokio::test]
    #[ignore = "requires a live Radicale (set RADICALE_URL/RADICALE_USER/RADICALE_PASS)"]
    async fn live_carddav_roundtrip() {
        let base = std::env::var("RADICALE_URL").expect("RADICALE_URL");
        let user = std::env::var("RADICALE_USER").unwrap_or_default();
        let pass = std::env::var("RADICALE_PASS").unwrap_or_default();
        let client = CardDavClient::new(DavConfig {
            base_url: base,
            username: user,
            password: pass,
        })
        .expect("client");

        let books = client
            .discover_addressbooks()
            .await
            .expect("discover address books");
        assert!(!books.is_empty(), "expected at least one address book");
        let book = &books[0];

        // Enumerate, then multiget one card back as a ContactCard.
        let cards = client.addressbook_query(&book.href).await.expect("query");
        if let Some(first) = cards.first() {
            let projected = client
                .fetch_cards(&book.href, std::slice::from_ref(&first.href))
                .await
                .expect("fetch");
            assert_eq!(projected.len(), 1);
        }
    }
}
