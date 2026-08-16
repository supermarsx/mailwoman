//! The HTTP/1.1 exchange that runs **inside** the tunnel.
//!
//! This is the tunnelled twin of [`crate::fetch_hop`] and it deliberately mirrors
//! it field for field: the same normalized `User-Agent`, the same caller-chosen
//! `Accept`, the same `Accept-Encoding: identity`, the same [`MAX_IMAGE_BYTES`] cap
//! applied **while streaming**, and **no automatic redirect following** — a
//! `Location` is handed back to the caller so it re-enters the address gate from
//! the top.
//!
//! The request is constructed here, from the origin URL alone. Nothing from the
//! tunnel-setup phase is carried into it, which is why a proxy credential cannot
//! reach an origin: `Proxy-Authorization` is written by [`super::connect`] onto the
//! `CONNECT` head and never appears in this builder.

use bytes::Bytes;
use http_body_util::{BodyExt, Empty};
use hyper::header;
use hyper_util::rt::TokioIo;
use tokio::io::{AsyncRead, AsyncWrite};

use super::ProxyRefusal;
use crate::{Hop, MAX_IMAGE_BYTES, PROXY_UA, Refusal};

/// The `Host` header value for an origin URL: the hostname, plus the port when it
/// is not the scheme default. `reqwest::Url::port` is already `None` for a default
/// port and `host_str` already brackets an IPv6 literal.
fn host_header(url: &reqwest::Url) -> Result<String, ProxyRefusal> {
    let host = url
        .host_str()
        .ok_or(ProxyRefusal::Origin(Refusal::BadRequest("URL has no host")))?;
    Ok(match url.port() {
        Some(p) => format!("{host}:{p}"),
        None => host.to_string(),
    })
}

/// The origin-form request target: path plus query, never the absolute URL (which
/// is the proxy-facing form and would put the hostname back on the wire).
fn request_target(url: &reqwest::Url) -> String {
    match url.query() {
        Some(q) => format!("{}?{}", url.path(), q),
        None => url.path().to_string(),
    }
}

/// Perform one HTTP/1.1 request/response over an established tunnel.
///
/// Returns [`Hop::Redirect`] with the raw `Location` for any 3xx, exactly as the
/// direct path does, so the caller re-validates the new URL rather than this
/// function following it.
pub async fn exchange<S>(stream: S, url: &reqwest::Url, accept: &str) -> Result<Hop, ProxyRefusal>
where
    S: AsyncRead + AsyncWrite + Unpin + Send + 'static,
{
    let (mut sender, conn) = hyper::client::conn::http1::handshake(TokioIo::new(stream))
        .await
        .map_err(|e| ProxyRefusal::OriginHttp(format!("HTTP/1.1 handshake failed: {e}")))?;
    // The connection future must be driven for the exchange to make progress. It is
    // aborted when this hop ends — there is no pooling, so each hop (and each
    // redirect) gets a fresh tunnel, matching the direct path's per-hop isolation.
    let pump = tokio::spawn(async move {
        let _ = conn.await;
    });
    let result = exchange_inner(&mut sender, url, accept).await;
    pump.abort();
    result
}

async fn exchange_inner(
    sender: &mut hyper::client::conn::http1::SendRequest<Empty<Bytes>>,
    url: &reqwest::Url,
    accept: &str,
) -> Result<Hop, ProxyRefusal> {
    let request = hyper::Request::builder()
        .method(hyper::Method::GET)
        .uri(request_target(url))
        .header(header::HOST, host_header(url)?)
        .header(header::USER_AGENT, PROXY_UA)
        .header(header::ACCEPT, accept)
        // One less decompression-bomb surface, identical to the direct path.
        .header(header::ACCEPT_ENCODING, "identity")
        .body(Empty::<Bytes>::new())
        .map_err(|e| ProxyRefusal::OriginHttp(format!("request could not be built: {e}")))?;

    let response = sender
        .send_request(request)
        .await
        .map_err(|e| ProxyRefusal::OriginHttp(format!("request failed: {e}")))?;

    let status = response.status();
    if status.is_redirection() {
        let location = response
            .headers()
            .get(header::LOCATION)
            .and_then(|v| v.to_str().ok())
            .ok_or(ProxyRefusal::Origin(Refusal::Upstream))?
            .to_string();
        return Ok(Hop::Redirect(location));
    }
    if !status.is_success() {
        return Err(ProxyRefusal::Origin(Refusal::Upstream));
    }

    // Early refusal from Content-Length when the origin declares one, then the
    // streaming cap regardless of what it declared.
    if let Some(declared) = response
        .headers()
        .get(header::CONTENT_LENGTH)
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.parse::<u64>().ok())
        && declared as usize > MAX_IMAGE_BYTES
    {
        return Err(ProxyRefusal::Origin(Refusal::TooLarge));
    }

    let mut body = response.into_body();
    let mut buf: Vec<u8> = Vec::new();
    while let Some(frame) = body.frame().await {
        let frame = frame.map_err(|_| ProxyRefusal::Origin(Refusal::Upstream))?;
        if let Some(chunk) = frame.data_ref() {
            if buf.len() + chunk.len() > MAX_IMAGE_BYTES {
                return Err(ProxyRefusal::Origin(Refusal::TooLarge));
            }
            buf.extend_from_slice(chunk);
        }
    }
    Ok(Hop::Body(buf))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn url(s: &str) -> reqwest::Url {
        reqwest::Url::parse(s).unwrap()
    }

    #[test]
    fn host_header_omits_the_default_port_and_keeps_a_custom_one() {
        assert_eq!(
            host_header(&url("https://cdn.example/x.png")).unwrap(),
            "cdn.example"
        );
        assert_eq!(
            host_header(&url("http://cdn.example/x.png")).unwrap(),
            "cdn.example"
        );
        assert_eq!(
            host_header(&url("https://cdn.example:8443/x.png")).unwrap(),
            "cdn.example:8443"
        );
        assert_eq!(
            host_header(&url("http://[2001:db8::1]/x")).unwrap(),
            "[2001:db8::1]"
        );
    }

    #[test]
    fn request_target_is_origin_form_and_never_absolute() {
        assert_eq!(
            request_target(&url("https://cdn.example/a/b.png")),
            "/a/b.png"
        );
        assert_eq!(
            request_target(&url("https://cdn.example/a/b.png?v=2&w=1")),
            "/a/b.png?v=2&w=1"
        );
        assert_eq!(request_target(&url("https://cdn.example")), "/");
        // The hostname must not appear in the request target — that form is what a
        // forward proxy reads, and it would put the name back on the wire.
        assert!(!request_target(&url("https://cdn.example/a")).contains("cdn.example"));
    }
}
