//! Recorded-fixture replay (host-only; never compiled into the wasm component).
//!
//! Every fixture under `fixtures/` is a request→response pair: a method + a URL
//! substring to match, and the canned status/body to return. Both the pure-module
//! unit tests (via [`FixtureTransport`]) and the in-jail integration test (via an
//! `mw_plugin::HttpFetcher` wrapper) replay the SAME set, so CI never touches a live
//! Microsoft 365 tenant (plan §2.5). Ordering-sensitive flows (delta initial vs the
//! follow-up `deltaLink`) are disambiguated by distinctive URL substrings.

use std::path::Path;

use serde::Deserialize;

use crate::graph::{BridgeError, HttpRequestSpec, HttpResponseData, Result, Transport};

/// A fake bearer token the fixtures accept. Real tokens NEVER live in the guest —
/// the host holds/refreshes them; this only stands in for the host `oauth-token`
/// import so the guest can attach an `Authorization` header.
pub const FIXTURE_TOKEN: &str = "FIXTURE.ACCESS.TOKEN";

/// One recorded request→response pair.
#[derive(Debug, Clone, Deserialize)]
pub struct Fixture {
    /// HTTP method to match (`GET`/`POST`/`PATCH`).
    pub method: String,
    /// A substring that must appear in the request URL for this fixture to match.
    pub url_contains: String,
    /// The status to return.
    pub status: u16,
    /// A JSON body (serialized verbatim), when the response is JSON.
    #[serde(default)]
    pub body_json: Option<serde_json::Value>,
    /// A text body (e.g. `$value` raw MIME), when the response is not JSON.
    #[serde(default)]
    pub body_text: Option<String>,
}

impl Fixture {
    fn body_bytes(&self) -> Vec<u8> {
        if let Some(j) = &self.body_json {
            serde_json::to_vec(j).unwrap_or_default()
        } else if let Some(t) = &self.body_text {
            t.clone().into_bytes()
        } else {
            Vec::new()
        }
    }
}

/// A loaded set of fixtures.
#[derive(Debug, Clone, Default)]
pub struct FixtureSet {
    fixtures: Vec<Fixture>,
}

impl FixtureSet {
    /// Load every `*.json` fixture in the crate's `fixtures/` directory.
    pub fn load_default() -> Self {
        Self::load_dir(concat!(env!("CARGO_MANIFEST_DIR"), "/fixtures"))
    }

    /// Load every `*.json` fixture in `dir`.
    pub fn load_dir(dir: impl AsRef<Path>) -> Self {
        let mut fixtures = Vec::new();
        let read = std::fs::read_dir(dir.as_ref())
            .unwrap_or_else(|e| panic!("read fixtures dir {}: {e}", dir.as_ref().display()));
        for entry in read.flatten() {
            let path = entry.path();
            if path.extension().and_then(|e| e.to_str()) != Some("json") {
                continue;
            }
            let raw = std::fs::read(&path)
                .unwrap_or_else(|e| panic!("read fixture {}: {e}", path.display()));
            let fx: Fixture = serde_json::from_slice(&raw)
                .unwrap_or_else(|e| panic!("parse fixture {}: {e}", path.display()));
            fixtures.push(fx);
        }
        // Longest `url_contains` first so a more specific match (a deltaLink token)
        // beats a broad prefix (`/messages/delta`) regardless of file order.
        fixtures.sort_by_key(|f| std::cmp::Reverse(f.url_contains.len()));
        Self { fixtures }
    }

    /// Find the response for a request, or `None` if nothing matches.
    pub fn match_response(&self, method: &str, url: &str) -> Option<HttpResponseData> {
        self.fixtures
            .iter()
            .find(|f| f.method.eq_ignore_ascii_case(method) && url.contains(&f.url_contains))
            .map(|f| HttpResponseData {
                status: f.status,
                headers: vec![("Content-Type".into(), "application/json".into())],
                body: f.body_bytes(),
            })
    }
}

/// A pure [`Transport`] that replays fixtures — used by the host unit tests to drive
/// the whole Graph mapping without a wasm toolchain.
pub struct FixtureTransport {
    set: FixtureSet,
}

impl FixtureTransport {
    pub fn new(set: FixtureSet) -> Self {
        Self { set }
    }

    pub fn load_default() -> Self {
        Self::new(FixtureSet::load_default())
    }
}

impl Transport for FixtureTransport {
    fn token(&self, _account: &str) -> Result<String> {
        Ok(FIXTURE_TOKEN.to_string())
    }

    fn fetch(&self, req: HttpRequestSpec) -> Result<HttpResponseData> {
        self.set
            .match_response(&req.method, &req.url)
            .ok_or_else(|| {
                BridgeError::Transport(format!("no fixture for {} {}", req.method, req.url))
            })
    }
}
