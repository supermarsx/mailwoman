//! One static contract shared with the browser. No authority comes from it.
use axum::http::Uri;
use serde::Deserialize;
use std::sync::OnceLock;

#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(super) struct Contract {
    pub schema: u8,
    pub protocol: String,
    pub ack: serde_json::Value,
    pub limits: Limits,
    pub methods: Vec<String>,
    pub request_headers: Vec<String>,
    pub webmail: Webmail,
    pub admin: Admin,
    pub cookie_handoff: CookieHandoff,
}
#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(super) struct Limits {
    pub frame_bytes: usize,
    pub body_bytes: usize,
    pub response_bytes: usize,
    pub reply_frame_bytes: usize,
    pub path_bytes: usize,
    pub header_bytes: usize,
    pub headers: usize,
    pub id_bytes: usize,
}
#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(super) struct Webmail {
    pub endpoint: String,
    pub max_in_flight: usize,
    pub json_exact: Vec<String>,
    pub json_prefixes: Vec<String>,
    pub http_required: Vec<Rule>,
}
#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(super) struct Admin {
    pub endpoint_suffix: String,
    pub cookie_delivery_suffix: String,
    pub max_in_flight: usize,
    pub scope: String,
    pub http_required: Vec<Rule>,
}
#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(super) struct CookieHandoff {
    pub capacity: usize,
    pub max_sealed_bytes: usize,
    pub receipt_ttl_ms: u64,
    pub token_bytes: usize,
    pub token_encoding: String,
    pub token_characters: usize,
    pub action_ack_field: String,
    pub request_body_field: String,
    pub success_body: serde_json::Value,
}
#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(super) struct Rule {
    pub exact: Option<String>,
    pub prefix: Option<String>,
    pub methods: Option<Vec<String>>,
    pub reason: String,
}
impl Rule {
    fn matches(&self, path: &str, method: &str) -> bool {
        self.methods
            .as_ref()
            .is_none_or(|ms| ms.iter().any(|m| m == method))
            && (self.exact.as_deref() == Some(path)
                || self.prefix.as_ref().is_some_and(|p| path.starts_with(p)))
    }
}
pub(super) fn get() -> &'static Contract {
    static VALUE: OnceLock<Contract> = OnceLock::new();
    VALUE.get_or_init(|| {
        let value: Contract = serde_json::from_str(include_str!(
            "../../../../apps/web/src/api/websocket-action-contract.json"
        ))
        .expect("static action contract JSON");
        assert!(value.schema == 1 && value.protocol == "mailwoman-actions-v1");
        assert!((1..=8).contains(&value.webmail.max_in_flight) && value.admin.max_in_flight == 1);
        assert!(
            value.limits.frame_bytes <= 2 * 1024 * 1024
                && value.limits.body_bytes <= value.limits.frame_bytes
        );
        assert!(
            value.limits.response_bytes <= 64 * 1024 * 1024
                && value.limits.reply_frame_bytes <= 64 * 1024 * 1024
        );
        assert!(
            (1..=16384).contains(&value.limits.path_bytes) && value.limits.header_bytes <= 16384
        );
        assert!(
            (1..=32).contains(&value.limits.headers) && (1..=128).contains(&value.limits.id_bytes)
        );
        assert!((1..=256).contains(&value.cookie_handoff.capacity));
        assert!(
            value.cookie_handoff.max_sealed_bytes <= 8192
                && value.cookie_handoff.token_bytes == 32
                && value.cookie_handoff.token_encoding == "hex-lower"
                && value.cookie_handoff.token_characters == 64
        );
        assert!(
            value.cookie_handoff.receipt_ttl_ms <= 30000 && value.cookie_handoff.receipt_ttl_ms > 0
        );
        assert!(value.admin.scope == "dynamic-console-descendants");
        assert!(
            value.cookie_handoff.action_ack_field == "cookieDelivery"
                && value.cookie_handoff.request_body_field == "token"
        );
        assert!(value.cookie_handoff.success_body == serde_json::json!({"delivered":true}));
        assert!(value.request_headers.iter().all(|h| {
            h.len() <= 64
                && h.bytes().all(|c| c.is_ascii_lowercase() || c == b'-')
                && ![
                    "cookie",
                    "authorization",
                    "host",
                    "origin",
                    "forwarded",
                    "user-agent",
                ]
                .contains(&h.as_str())
                && !h.starts_with("x-forwarded-")
        }));
        value
    })
}
pub(super) struct Target {
    pub uri: Uri,
    pub path: String,
}
pub(super) fn target(input: &str) -> Option<Target> {
    if input.len() > get().limits.path_bytes
        || !input.starts_with('/')
        || input.starts_with("//")
        || input.contains(['\\', '\r', '\n', '#'])
    {
        return None;
    }
    let uri: Uri = input.parse().ok()?;
    if uri.scheme().is_some() || uri.authority().is_some() {
        return None;
    }
    let path = decode(uri.path())?;
    if path.starts_with("//") || path.contains(['\\', '%']) || path.chars().any(char::is_control) {
        return None;
    }
    // A percent-encoded path separator must not change route segmentation.
    if path.bytes().filter(|c| *c == b'/').count()
        != uri.path().bytes().filter(|c| *c == b'/').count()
    {
        return None;
    }
    if path.split('/').any(|s| matches!(s, "." | "..")) {
        return None;
    }
    if let Some(query) = uri.query() {
        if decode(query)?.chars().any(char::is_control) {
            return None;
        }
    }
    Some(Target { uri, path })
}
fn decode(input: &str) -> Option<String> {
    let bytes = input.as_bytes();
    let mut output = Vec::with_capacity(bytes.len());
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'%' {
            let hex = |b: u8| match b {
                b'0'..=b'9' => Some(b - b'0'),
                b'a'..=b'f' => Some(b - b'a' + 10),
                b'A'..=b'F' => Some(b - b'A' + 10),
                _ => None,
            };
            output.push(hex(*bytes.get(i + 1)?)? * 16 + hex(*bytes.get(i + 2)?)?);
            i += 3;
        } else {
            output.push(bytes[i]);
            i += 1;
        }
    }
    String::from_utf8(output).ok()
}
pub(super) fn requires_http(path: &str, method: &str, admin: bool) -> bool {
    let rules = if admin {
        &get().admin.http_required
    } else {
        &get().webmail.http_required
    };
    rules.iter().any(|r| r.matches(path, method))
}
pub(super) fn webmail_path(path: &str, admin_base: &str) -> bool {
    if path == admin_base
        || path
            .strip_prefix(admin_base)
            .is_some_and(|s| s.starts_with('/'))
    {
        return false;
    }
    let value = &get().webmail;
    path != value.endpoint
        && !path.starts_with(&(value.endpoint.clone() + "/"))
        && (value.json_exact.iter().any(|p| p == path)
            || value.json_prefixes.iter().any(|p| path.starts_with(p)))
}
