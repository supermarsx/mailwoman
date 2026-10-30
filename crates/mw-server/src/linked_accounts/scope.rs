use super::*;
use axum::{
    extract::{Request, State},
    middleware::Next,
};

/// The linked-mailbox scope (t87): ONE list, shared with the web client's
/// `linkedAccountRouting.ts`. Only its `mailbox` rules are allowed with a
/// selector; `primary`, `public` and `signed` are client routing only and are
/// refused here like any unlisted path.
const MANIFEST: &str = include_str!("../../../../apps/web/src/api/linked-scope.json");

#[derive(serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct Rule {
    path: Option<String>,
    prefix: Option<String>,
    segments: Option<usize>,
    suffix: Option<String>,
    scope: String,
    #[serde(default)]
    resource: bool,
    #[serde(default)]
    #[allow(dead_code)]
    note: Option<String>,
}
#[derive(serde::Deserialize)]
struct Manifest {
    routes: Vec<Rule>,
}

impl Rule {
    fn matches(&self, path: &str) -> bool {
        if let Some(exact) = &self.path {
            return path == exact;
        }
        let (Some(prefix), Some(segments)) = (&self.prefix, self.segments) else {
            return false;
        };
        path.strip_prefix(prefix.as_str()).is_some_and(|tail| {
            let parts: Vec<&str> = tail.split('/').collect();
            parts.len() == segments
                && parts.iter().all(|v| !v.is_empty())
                && self
                    .suffix
                    .as_deref()
                    .is_none_or(|suffix| parts.last() == Some(&suffix))
        })
    }
}

/// The `mailbox` rules, parsed once. A malformed manifest panics at first use
/// (and in the unit test below), never widening the scope.
fn mailbox_rules() -> &'static [Rule] {
    static RULES: std::sync::LazyLock<Vec<Rule>> = std::sync::LazyLock::new(|| {
        let manifest: Manifest =
            serde_json::from_str(MANIFEST).expect("linked-scope.json is valid");
        for rule in &manifest.routes {
            assert!(
                matches!(
                    rule.scope.as_str(),
                    "mailbox" | "primary" | "public" | "signed"
                ) && rule.path.is_some() != rule.prefix.is_some()
                    && (rule.prefix.is_none() || rule.segments.is_some_and(|n| n > 0))
                    && (!rule.resource || rule.scope == "mailbox"),
                "invalid linked scope rule {:?} {:?}",
                rule.path,
                rule.prefix
            );
        }
        manifest
            .routes
            .into_iter()
            .filter(|rule| rule.scope == "mailbox")
            .collect()
    });
    &RULES
}

pub(crate) fn allowed(path: &str) -> bool {
    mailbox_rules().iter().any(|rule| rule.matches(path))
}
fn resource(path: &str) -> bool {
    mailbox_rules()
        .iter()
        .any(|rule| rule.resource && rule.matches(path))
}
pub(crate) async fn guard(
    State(state): State<AppState>,
    mut request: Request,
    next: Next,
) -> Response {
    if !request.headers().contains_key(SELECTOR) && request.uri().query().is_none() {
        return next.run(request).await;
    }
    let mut query_url = reqwest::Url::parse("http://linked.invalid/").expect("fixed URL");
    query_url.set_query(request.uri().query());
    let pairs: Vec<(String, String)> = query_url.query_pairs().into_owned().collect();
    let links: Vec<_> = pairs.iter().filter(|(key, _)| key == "mwLink").collect();
    let header_count = request.headers().get_all(SELECTOR).iter().count();
    if header_count > 1 || links.len() > 1 {
        return failure(StatusCode::BAD_REQUEST, "invalidLinkedContext");
    }
    if let Some((_, link)) = links.first() {
        if !resource(request.uri().path())
            || request.method() != axum::http::Method::GET
            || header_count > 0
            || request
                .headers()
                .contains_key(axum::http::header::AUTHORIZATION)
        {
            return failure(StatusCode::BAD_REQUEST, "invalidLinkedContext");
        }
        let Ok(value) = axum::http::HeaderValue::from_str(link) else {
            return failure(StatusCode::BAD_REQUEST, "invalidLinkedContext");
        };
        request.headers_mut().insert(SELECTOR, value);
        query_url.set_query(None);
        query_url
            .query_pairs_mut()
            .extend_pairs(pairs.iter().filter(|(key, _)| key != "mwLink"));
        let query = query_url.query().unwrap_or("");
        let uri = if query.is_empty() {
            request.uri().path().to_string()
        } else {
            format!("{}?{}", request.uri().path(), query)
        };
        let Ok(uri) = uri.parse() else {
            return failure(StatusCode::BAD_REQUEST, "invalidLinkedContext");
        };
        *request.uri_mut() = uri;
    }
    if let Some(link) = request.headers().get(SELECTOR) {
        if !allowed(request.uri().path()) {
            return failure(StatusCode::FORBIDDEN, "linkedContextUnsupported");
        }
        if link
            .to_str()
            .ok()
            .and_then(|v| uuid::Uuid::parse_str(v).ok())
            .is_none()
        {
            return failure(StatusCode::BAD_REQUEST, "invalidLinkedContext");
        }
        // Scoped API keys/OAuth adapters may bypass normal session resolution.
        // They cannot select links. Establish a real primary cookie/native
        // session here before any handler-specific scoped adapter can execute.
        if request.headers().contains_key("x-api-key") {
            return failure(StatusCode::FORBIDDEN, "linkedContextUnsupported");
        }
        let mut primary = request.headers().clone();
        primary.remove(SELECTOR);
        if let Err(response) = crate::authed(&state, &primary).await {
            return response;
        }
    }
    next.run(request).await
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Hand-written, reviewed expectations shared with the web client's test
    /// (`linkedAccountRouting.test.ts`). Both matchers must agree with this
    /// list, so they cannot drift apart (t87).
    const CASES: &str = include_str!("../../../../apps/web/src/api/linked-scope.cases.json");

    fn cases(key: &str) -> Vec<String> {
        let value: serde_json::Value = serde_json::from_str(CASES).expect("cases parse");
        value[key]
            .as_array()
            .unwrap_or_else(|| panic!("cases.{key}"))
            .iter()
            .map(|v| v.as_str().expect("path").to_owned())
            .collect()
    }

    #[test]
    fn linked_scope_is_explicit_and_excludes_primary_security_and_streams() {
        for path in [
            "/api/me",
            "/api/policy",
            "/jmap/api",
            "/api/account/identities/x",
            "/jmap/download/a/b/name",
            "/api/settings/mail.folderRole.sent",
            "/api/remote-images/grants",
            "/api/remote-images/grant",
            "/api/remote-images/revoke",
        ] {
            assert!(allowed(path), "{path}");
        }
        for path in [
            "/api/login",
            "/api/accounts",
            "/api/security/passkeys",
            "/api/session/rotate",
            "/oidc/token",
            "/oauth/token",
            "/jmap/ws",
            "/jmap/eventsource",
            "/admin/settings",
            "/api/unknown",
            "/api/image-proxy",
            "/api/remote-images/grant/x",
            "/api/account/identities/x/more",
            "/api/masked",
            "/api/zeroaccess/enable",
            "/api/password",
            "/api/account/2fa/totp/begin",
        ] {
            assert!(!allowed(path), "{path}");
        }
    }

    #[test]
    fn the_server_agrees_with_the_shared_case_list() {
        for path in cases("mailbox") {
            assert!(allowed(&path), "mailbox: {path}");
            assert!(!resource(&path), "not a resource: {path}");
        }
        for path in cases("resource") {
            assert!(allowed(&path) && resource(&path), "resource: {path}");
        }
        // Client-only routing classes are refused WITH a selector.
        for key in ["primary", "public", "signed", "unsupported"] {
            for path in cases(key) {
                assert!(
                    !allowed(&path),
                    "{key} must be refused with a selector: {path}"
                );
            }
        }
    }

    #[test]
    fn documented_refusals_match_no_rule() {
        let value: serde_json::Value = serde_json::from_str(MANIFEST).expect("manifest parses");
        let refused = value["refused"].as_array().expect("refused list");
        assert!(!refused.is_empty());
        for entry in refused {
            let path = entry["path"].as_str().expect("path");
            assert!(!allowed(path), "{path}");
        }
    }
}
