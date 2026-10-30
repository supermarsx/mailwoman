//! t98: the one route guard behind feature profiles.
//!
//! Every gated feature of [`crate::features::CATALOGUE`] is refused here, by
//! the server, when the subject's profile chain turns it off — the web client
//! hiding it is only presentation. One middleware layer in front of every user
//! route ([`guard`]) does three kinds of check:
//!
//! * **REST prefixes** ([`REST`]): a path (and method set) belongs to a
//!   feature, e.g. `/api/masked/**` to `masked`, writes under
//!   `/api/account/signatures` to `compose.signatures`;
//! * **user setting writes**: `PUT /api/settings/<key>` of a key a feature owns
//!   (`reader.zoom` → `reading.zoom`, `shortcuts.*` → `shortcuts`, ...);
//! * **JMAP method rules** ([`jmap_features`]) on `POST /jmap/api`, before any
//!   method of the batch runs: `CryptoKey/*` → `keys`, a submission with
//!   `sendAt` → `compose.scheduled_send`, an HTML body → `compose.rich_text`,
//!   a non-system keyword → `labels`, `snoozedUntil` → `snooze`, field
//!   operators in a search text → `search.operators`, ...
//!
//! plus three subject-wide checks: native app sessions (`apps.native`), the
//! account's mail protocol (`access.protocols`), and two limits
//! (`limit.linked_mailboxes` on link, `limit.attachment_bytes` on upload,
//! `limit.recipients_per_message` on JMAP writes).
//!
//! A refusal is `403 {"code":"feature_disabled","feature":<id>,"key":<key>}`
//! (a limit: `409`/`413 {"code":"limit_exceeded","limit":<key>,"max":<n>}`).
//! An unauthenticated request passes through so the handler answers it as
//! before (401). Values are resolved by [`crate::features::resolve_features`]'s
//! chain, read live (no cache): a profile change applies to the next request.
//!
//! Features checked inside their handlers already (import, export, Nextcloud,
//! Assist, UI plugins, packages, push, the PIM families in
//! [`crate::feature_jmap`]) are not repeated here.

use crate::{AppState, settings_registry as registry, settings_routes::UserSubject};
use axum::{
    body::Body,
    extract::{Request, State},
    http::{Method, StatusCode},
    middleware::Next,
    response::{IntoResponse, Response},
};
use serde_json::{Value, json};
use std::collections::BTreeSet;

/// Which methods of a prefix are gated.
#[derive(Clone, Copy)]
enum Methods {
    All,
    /// POST, PUT, PATCH, DELETE: reading what exists stays possible.
    Writes,
    Only(&'static [Method]),
}

struct RestGate {
    prefix: &'static str,
    methods: Methods,
    feature: &'static str,
}

const fn gate(prefix: &'static str, methods: Methods, feature: &'static str) -> RestGate {
    RestGate {
        prefix,
        methods,
        feature,
    }
}

const POST: &[Method] = &[Method::POST];
const PUT: &[Method] = &[Method::PUT];

/// The REST entry points of the gated features (see the module docs).
static REST: &[RestGate] = &[
    gate(
        "/api/nextcloud/share-link",
        Methods::All,
        "compose.share_links",
    ),
    gate(
        "/api/account/signatures",
        Methods::Writes,
        "compose.signatures",
    ),
    gate(
        "/api/account/identities",
        Methods::Writes,
        "compose.identities",
    ),
    gate("/api/image-proxy", Methods::All, "privacy.remote_images"),
    gate(
        "/api/remote-images/grant",
        Methods::Only(POST),
        "privacy.remote_images",
    ),
    gate(
        "/api/account/saved-searches",
        Methods::Writes,
        "search.saved",
    ),
    gate("/api/masked", Methods::All, "masked"),
    gate("/api/calendar/subscribe", Methods::All, "calendar"),
    gate("/api/calendar/refresh", Methods::All, "calendar"),
    gate("/api/directory", Methods::All, "directory"),
    gate("/api/keys", Methods::Only(POST), "api_keys"),
    gate("/oauth/consent", Methods::Only(POST), "oauth_apps"),
    gate("/oauth/decision", Methods::Only(POST), "oauth_apps"),
    gate(
        "/api/account/appearance/customization",
        Methods::Only(PUT),
        "customisation.code",
    ),
    gate(
        "/api/account/appearance",
        Methods::Only(PUT),
        "appearance.custom",
    ),
    gate("/api/analytics/events", Methods::Only(POST), "analytics"),
];

/// User setting keys a feature owns: `PUT /api/settings/<key>` is refused while
/// the feature is off (resetting with `DELETE` stays possible).
fn setting_feature(key: &str) -> Option<&'static str> {
    match key {
        "reader.zoom" => Some("reading.zoom"),
        "links.trustedDomains" | "links.trustedSubdomains" => Some("privacy.trusted_links"),
        "mail.focusedInbox" => Some("views.focused"),
        k if k.starts_with("shortcuts.") => Some("shortcuts"),
        k if k.starts_with("appearance.") => Some("appearance.custom"),
        _ => None,
    }
}

fn method_matches(methods: Methods, method: &Method) -> bool {
    match methods {
        Methods::All => true,
        Methods::Writes => matches!(
            *method,
            Method::POST | Method::PUT | Method::PATCH | Method::DELETE
        ),
        Methods::Only(list) => list.contains(method),
    }
}

fn under(path: &str, prefix: &str) -> bool {
    path == prefix
        || path
            .strip_prefix(prefix)
            .is_some_and(|rest| rest.starts_with('/'))
}

/// The features a REST request needs (no JMAP, no subject-wide checks).
pub(crate) fn rest_features(method: &Method, path: &str) -> BTreeSet<&'static str> {
    let mut out = BTreeSet::new();
    // The longest prefix decides, so `/api/account/appearance/customization`
    // is not also `/api/account/appearance`.
    if let Some(g) = REST
        .iter()
        .filter(|g| under(path, g.prefix) && method_matches(g.methods, method))
        .max_by_key(|g| g.prefix.len())
    {
        out.insert(g.feature);
    }
    if *method == Method::PUT
        && let Some(key) = path.strip_prefix("/api/settings/")
        && let Some(feature) = setting_feature(&percent_decode(key))
    {
        out.insert(feature);
    }
    out
}

fn percent_decode(s: &str) -> String {
    let bytes = s.as_bytes();
    let mut out = Vec::with_capacity(bytes.len());
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'%'
            && i + 2 < bytes.len()
            && let Ok(v) = u8::from_str_radix(&s[i + 1..i + 3], 16)
        {
            out.push(v);
            i += 3;
        } else {
            out.push(bytes[i]);
            i += 1;
        }
    }
    String::from_utf8_lossy(&out).into_owned()
}

// ─── JMAP rules ──────────────────────────────────────────────────────────────

/// The search operators `mw-search` understands (`crates/mw-search/src/query.rs`
/// `is_operator`); `name:value` with one of these is an operator search.
const SEARCH_OPERATORS: &[&str] = &[
    "from", "to", "cc", "subject", "body", "text", "has", "filename", "before", "after", "in",
    "is", "larger", "smaller", "tag", "pinned",
];

fn has_operator(text: &str) -> bool {
    text.split_whitespace().any(|token| {
        let token = token.trim_start_matches(['-', '(', '"']);
        token.split_once(':').is_some_and(|(name, value)| {
            !value.is_empty() && SEARCH_OPERATORS.contains(&name.to_ascii_lowercase().as_str())
        })
    })
}

/// Walk a JMAP filter (conditions and operators) for `f`.
fn any_condition(filter: &Value, f: &dyn Fn(&serde_json::Map<String, Value>) -> bool) -> bool {
    let Some(obj) = filter.as_object() else {
        return false;
    };
    if f(obj) {
        return true;
    }
    obj.get("conditions")
        .and_then(Value::as_array)
        .is_some_and(|list| list.iter().any(|c| any_condition(c, f)))
}

/// A user label is any keyword that is not a `$` system keyword.
fn is_label(keyword: &str) -> bool {
    !keyword.starts_with('$') && !keyword.is_empty()
}

fn values_of(args: &Value, field: &str) -> Vec<Value> {
    args.get(field)
        .and_then(Value::as_object)
        .map(|m| m.values().cloned().collect())
        .unwrap_or_default()
}

/// Whether any object in `v` (recursively) has `"kind"` equal to one of `kinds`.
fn has_kind(v: &Value, kinds: &[&str]) -> bool {
    match v {
        Value::Object(m) => {
            m.get("kind")
                .and_then(Value::as_str)
                .is_some_and(|k| kinds.contains(&k))
                || m.values().any(|x| has_kind(x, kinds))
        }
        Value::Array(a) => a.iter().any(|x| has_kind(x, kinds)),
        _ => false,
    }
}

/// The catalogue features a JMAP batch needs, beyond [`crate::feature_jmap`]'s
/// family gates. Clearing a snooze or follow-up, removing a label and sending a
/// scheduled message now (`sendAt: null`) stay possible with the feature off.
pub(crate) fn jmap_features(request: &Value) -> BTreeSet<&'static str> {
    let mut out = BTreeSet::new();
    for call in request
        .get("methodCalls")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
    {
        let Some(name) = call.get(0).and_then(Value::as_str) else {
            continue;
        };
        let args = call.get(1).cloned().unwrap_or(Value::Null);
        let (family, op) = name.split_once('/').unwrap_or((name, ""));
        match (family, op) {
            ("CryptoKey", _) => {
                out.insert("keys");
            }
            ("VacationResponse", _) => {
                out.insert("vacation");
            }
            ("MailRule", "set") => {
                if values_of(&args, "create")
                    .iter()
                    .chain(values_of(&args, "update").iter())
                    .any(|r| has_kind(r, &["forward", "forwardCopy"]))
                {
                    out.insert("rules.forward");
                }
            }
            ("EmailSubmission", "set") => {
                if values_of(&args, "create")
                    .iter()
                    .chain(values_of(&args, "update").iter())
                    .any(|s| s.get("sendAt").is_some_and(|v| !v.is_null()))
                {
                    out.insert("compose.scheduled_send");
                }
            }
            ("Email", "set") => {
                for create in values_of(&args, "create") {
                    if create
                        .get("htmlBody")
                        .and_then(Value::as_array)
                        .is_some_and(|parts| !parts.is_empty())
                    {
                        out.insert("compose.rich_text");
                    }
                    if create
                        .get("attachments")
                        .and_then(Value::as_array)
                        .is_some_and(|parts| {
                            parts.iter().any(|p| {
                                p.get("disposition").and_then(Value::as_str) != Some("inline")
                            })
                        })
                    {
                        out.insert("compose.attachments");
                    }
                    if create.get("mailwomanRequestReadReceipt") == Some(&Value::Bool(true)) {
                        out.insert("compose.read_receipts");
                    }
                    if create
                        .get("keywords")
                        .and_then(Value::as_object)
                        .is_some_and(|k| k.iter().any(|(k, v)| is_label(k) && v == &json!(true)))
                    {
                        out.insert("labels");
                    }
                }
                for patch in values_of(&args, "update") {
                    let Some(patch) = patch.as_object() else {
                        continue;
                    };
                    for (path, value) in patch {
                        if let Some(keyword) = path.strip_prefix("keywords/")
                            && is_label(keyword)
                            && value == &json!(true)
                        {
                            out.insert("labels");
                        }
                        if path == "keywords"
                            && value.as_object().is_some_and(|k| {
                                k.iter().any(|(k, v)| is_label(k) && v == &json!(true))
                            })
                        {
                            out.insert("labels");
                        }
                        if path == "snoozedUntil" && !value.is_null() {
                            out.insert("snooze");
                        }
                        if path == "followUpAt" && !value.is_null() {
                            out.insert("follow_up");
                        }
                    }
                }
            }
            ("Email", "query") | ("SearchSnippet", "get") => {
                let filter = args.get("filter").cloned().unwrap_or(Value::Null);
                if any_condition(&filter, &|c| {
                    c.get("inMailbox").and_then(Value::as_str)
                        == Some(mw_engine::query::EVENTS_VIEW_ID)
                }) {
                    out.insert("views.events");
                }
                if any_condition(&filter, &|c| {
                    ["text", "body", "subject", "from", "to"]
                        .iter()
                        .any(|f| c.get(*f).and_then(Value::as_str).is_some_and(has_operator))
                }) {
                    out.insert("search.operators");
                }
            }
            _ => {}
        }
    }
    out
}

/// The most recipients one message of a JMAP batch names: the `to`, `cc` and
/// `bcc` of an `Email/set` create, or an `EmailSubmission/set` envelope.
pub(crate) fn jmap_max_recipients(request: &Value) -> usize {
    let mut most = 0;
    for call in request
        .get("methodCalls")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
    {
        let name = call.get(0).and_then(Value::as_str).unwrap_or_default();
        let args = call.get(1).cloned().unwrap_or(Value::Null);
        match name {
            "Email/set" => {
                for c in values_of(&args, "create") {
                    let n: usize = ["to", "cc", "bcc"]
                        .iter()
                        .map(|f| c.get(*f).and_then(Value::as_array).map_or(0, Vec::len))
                        .sum();
                    most = most.max(n);
                }
            }
            "EmailSubmission/set" => {
                for s in values_of(&args, "create") {
                    let n = s
                        .pointer("/envelope/rcptTo")
                        .and_then(Value::as_array)
                        .map_or(0, Vec::len);
                    most = most.max(n);
                }
            }
            _ => {}
        }
    }
    most
}

// ─── the middleware ──────────────────────────────────────────────────────────

fn disabled(feature: &str, key: &str) -> Response {
    (
        StatusCode::FORBIDDEN,
        [(axum::http::header::CACHE_CONTROL, "private, no-store")],
        axum::Json(json!({
            "code":"feature_disabled","feature":feature,"key":key,
            "error":"this feature is turned off for your account by the administrator",
        })),
    )
        .into_response()
}

fn over_limit(status: StatusCode, key: &str, max: u64) -> Response {
    (
        status,
        [(axum::http::header::CACHE_CONTROL, "private, no-store")],
        axum::Json(json!({
            "code":"limit_exceeded","limit":key,"max":max,
            "error":"this exceeds a limit the administrator set for your account",
        })),
    )
        .into_response()
}

fn unavailable() -> Response {
    StatusCode::SERVICE_UNAVAILABLE.into_response()
}

/// Whether the request is a native app's (bearer session, not an API key).
fn native_bearer(headers: &axum::http::HeaderMap) -> bool {
    crate::push_relay::bearer_token(headers).is_some_and(|t| !t.starts_with("mwk_"))
}

/// The paths the subject-wide checks (native apps, protocols) cover: the
/// signed-in API. Public, admin and static paths are not.
fn subject_wide(path: &str) -> bool {
    (path.starts_with("/jmap/") || path.starts_with("/api/"))
        && !matches!(
            path,
            "/api/login" | "/api/logout" | "/api/public-config" | "/api/discover" | "/api/me"
        )
        && !path.starts_with("/api/login/")
        && !path.starts_with("/api/sso/")
        && !path.starts_with("/api/signin/")
}

/// Where the account's mail protocol is checked: the JMAP session and API (all
/// mail goes through them; blob ids come from them) and the REST API.
fn protocol_path(path: &str) -> bool {
    matches!(path, "/jmap/session" | "/jmap/api") || path.starts_with("/api/v1/")
}

/// The checks one request needs before any subject lookup.
#[derive(Default)]
struct Needs {
    features: BTreeSet<&'static str>,
    native: bool,
    protocol: bool,
    link_limit: bool,
    upload_bytes: Option<u64>,
    recipients: usize,
    avatars: Option<Value>,
}

impl Needs {
    fn any(&self) -> bool {
        !self.features.is_empty()
            || self.native
            || self.protocol
            || self.link_limit
            || self.upload_bytes.is_some()
            || self.recipients > 0
            || self.avatars.is_some()
    }
}

/// Buffer a JSON body the guard must read, and hand the request back whole.
async fn read_json(req: Request, limit: usize) -> Result<(Request, Option<Value>), Response> {
    let (parts, body) = req.into_parts();
    let bytes = match axum::body::to_bytes(body, limit).await {
        Ok(b) => b,
        Err(_) => {
            return Err((
                StatusCode::PAYLOAD_TOO_LARGE,
                axum::Json(
                    json!({"type":"urn:ietf:params:jmap:error:limit","limit":"maxSizeRequest"}),
                ),
            )
                .into_response());
        }
    };
    let value = serde_json::from_slice(&bytes).ok();
    Ok((Request::from_parts(parts, Body::from(bytes)), value))
}

/// A read failure counts as "in use", so the full check runs (and fails closed).
/// Whether anything is stored for `key` (a policy row, or a value at any admin
/// scope, profiles included): with nothing stored every subject has the default.
async fn key_in_use(state: &AppState, key: &str) -> bool {
    use mw_store::settings_policy::SettingScope;
    match state.store.get_setting_policy(key).await {
        Ok(Some(_)) => return true,
        Ok(None) => {}
        Err(_) => return true,
    }
    for scope in [
        SettingScope::Domain,
        SettingScope::Profile,
        SettingScope::Group,
        SettingScope::AdminUser,
    ] {
        match state.store.list_setting_values(key, scope, None, 1).await {
            Ok(rows) if rows.is_empty() => {}
            _ => return true,
        }
    }
    false
}

/// The guard (see the module docs). Layered innermost on the user router.
pub(crate) async fn guard(State(state): State<AppState>, req: Request, next: Next) -> Response {
    let method = req.method().clone();
    let path = req.uri().path().to_string();
    let mut needs = Needs {
        features: rest_features(&method, &path),
        native: subject_wide(&path) && native_bearer(req.headers()),
        protocol: protocol_path(&path),
        link_limit: method == Method::POST && path == "/api/accounts/link",
        ..Default::default()
    };
    if method == Method::POST && path.starts_with("/jmap/upload/") {
        needs.upload_bytes = req
            .headers()
            .get(axum::http::header::CONTENT_LENGTH)
            .and_then(|v| v.to_str().ok())
            .and_then(|v| v.parse().ok());
    }
    let mut req = req;
    if method == Method::POST && path == "/jmap/api" {
        let declared = req
            .headers()
            .get(axum::http::header::CONTENT_LENGTH)
            .and_then(|v| v.to_str().ok())
            .and_then(|v| v.parse::<usize>().ok());
        // An over-size request is the handler's to refuse; never inspected here.
        if declared.is_none_or(|n| n <= mw_engine::jmap::core::MAX_SIZE_REQUEST) {
            let (rebuilt, body) =
                match read_json(req, mw_engine::jmap::core::MAX_SIZE_REQUEST).await {
                    Ok(v) => v,
                    Err(r) => return r,
                };
            req = rebuilt;
            if let Some(body) = body {
                needs.features.extend(jmap_features(&body));
                needs.recipients = jmap_max_recipients(&body);
            }
        }
    }
    if method == Method::PUT && path == "/api/accounts/layout" {
        let (rebuilt, body) = match read_json(req, 256 * 1024).await {
            Ok(v) => v,
            Err(r) => return r,
        };
        req = rebuilt;
        needs.avatars = body.and_then(|b| b.get("avatars").cloned());
        needs.features.remove("avatars");
    }
    // The two checks that touch most requests are skipped while nothing is
    // stored for their key (every subject then has the default: allowed).
    if needs.native && !key_in_use(&state, "cap.apps.native").await {
        needs.native = false;
    }
    if needs.protocol && !key_in_use(&state, "access.protocols").await {
        needs.protocol = false;
    }
    if !needs.any() {
        return next.run(req).await;
    }
    // Unauthenticated: the handler answers as it always did.
    let Ok(session) = crate::authed(&state, req.headers()).await else {
        return next.run(req).await;
    };
    if let Err(response) = check(&state, &session, &needs).await {
        return response;
    }
    next.run(req).await
}

async fn check(
    state: &AppState,
    session: &mw_store::Session,
    needs: &Needs,
) -> Result<(), Response> {
    let subject = UserSubject::resolved(&state.store, session)
        .await
        .map_err(|_| unavailable())?;
    let mut features = needs.features.clone();
    if needs.native {
        features.insert("apps.native");
    }
    let mut keys: Vec<&str> = features
        .iter()
        .filter_map(|id| crate::features::feature(id))
        .flat_map(|f| f.capability_keys.iter().copied())
        .collect();
    if needs.protocol {
        keys.push("access.protocols");
    }
    if needs.link_limit {
        keys.push("limit.linked_mailboxes");
    }
    if needs.upload_bytes.is_some() {
        keys.push("limit.attachment_bytes");
    }
    if needs.recipients > 0 {
        keys.push("limit.recipients_per_message");
    }
    if let Some(avatars) = &needs.avatars {
        // Only a change of the avatars needs the feature; a layout save that
        // carries them unchanged (reorder, rename) does not.
        let stored = state
            .store
            .linked_layout(&session.id)
            .await
            .map_err(|_| unavailable())?
            .and_then(|row| serde_json::from_str::<Value>(&row.avatars_json).ok())
            .unwrap_or_else(|| json!({}));
        let empty = avatars.as_object().is_some_and(serde_json::Map::is_empty) || avatars.is_null();
        let same_as_stored = avatars == &stored
            || (empty && stored.as_object().is_none_or(serde_json::Map::is_empty));
        if !same_as_stored {
            features.insert("avatars");
            keys.push("cap.avatars");
        }
    }
    if keys.is_empty() {
        return Ok(());
    }
    let values = registry::effective_all(&state.store, &subject.as_subject(), |d| {
        keys.contains(&d.key)
    })
    .await
    .map_err(|_| unavailable())?;
    let get = |key: &str| values.iter().find(|v| v.key == key).map(|v| &v.value);
    for id in &features {
        let Some(f) = crate::features::feature(id) else {
            continue;
        };
        for key in f.capability_keys {
            if get(key).and_then(Value::as_bool) != Some(true) {
                return Err(disabled(id, key));
            }
        }
    }
    if needs.protocol && session.jmap_url == "engine" {
        let allowed = get("access.protocols").and_then(Value::as_array);
        // A policy that cannot be read as a list never locks anyone out.
        if let Some(allowed) = allowed {
            let kind = match state.store.get_account(&session.account_id).await {
                Ok(account) => Some(match account.kind {
                    mw_store::AccountKind::Imap => "imap",
                    mw_store::AccountKind::Pop3 => "pop3",
                    mw_store::AccountKind::Ews => "ews",
                }),
                Err(mw_store::StoreError::NotFound) => None,
                Err(_) => return Err(unavailable()),
            };
            if let Some(kind) = kind
                && !allowed.iter().any(|p| p.as_str() == Some(kind))
            {
                return Err(disabled("access.protocols", "access.protocols"));
            }
        }
    }
    let limit = |key: &str| get(key).and_then(Value::as_u64).filter(|n| *n > 0);
    if needs.link_limit
        && let Some(max) = limit("limit.linked_mailboxes")
    {
        let linked = state
            .store
            .linked_accounts(&session.id)
            .await
            .map_err(|_| unavailable())?
            .len() as u64;
        if linked >= max {
            return Err(over_limit(
                StatusCode::CONFLICT,
                "limit.linked_mailboxes",
                max,
            ));
        }
    }
    if let (Some(bytes), Some(max)) = (needs.upload_bytes, limit("limit.attachment_bytes"))
        && bytes > max
    {
        return Err(over_limit(
            StatusCode::PAYLOAD_TOO_LARGE,
            "limit.attachment_bytes",
            max,
        ));
    }
    if needs.recipients > 0
        && let Some(max) = limit("limit.recipients_per_message")
        && needs.recipients as u64 > max
    {
        return Err(over_limit(
            StatusCode::FORBIDDEN,
            "limit.recipients_per_message",
            max,
        ));
    }
    Ok(())
}

/// The feature check for a key or token principal (no session): the REST API
/// (`api_keys`) and MCP (`mcp`) guards in [`crate::scope_mw`] call it with the
/// key's account. An account the store does not know (an upstream proxy key)
/// has only global policy.
pub(crate) async fn admit_account(
    state: &AppState,
    account_id: &str,
    feature: &str,
) -> Result<(), Response> {
    let username = match state.store.get_account(account_id).await {
        Ok(row) => row.username.to_ascii_lowercase(),
        Err(mw_store::StoreError::NotFound) => String::new(),
        Err(_) => return Err(unavailable()),
    };
    let domain = username
        .rsplit_once('@')
        .map(|(_, d)| d.to_string())
        .unwrap_or_default();
    let subject = mw_store::settings_policy::SettingSubject {
        personal_owner: None,
        domain: &domain,
        account_id,
        address: &username,
    };
    let Some(f) = crate::features::feature(feature) else {
        return Err(unavailable());
    };
    for key in f.capability_keys {
        let value = registry::effective_for(&state.store, key, &subject)
            .await
            .map_err(|_| unavailable())?;
        if value.and_then(|v| v.value.as_bool()) != Some(true) {
            return Err(disabled(feature, key));
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rest_prefixes_map_to_their_feature_by_method() {
        let f = |m: Method, p: &str| rest_features(&m, p).into_iter().collect::<Vec<_>>();
        assert_eq!(f(Method::GET, "/api/masked"), ["masked"]);
        assert_eq!(f(Method::DELETE, "/api/masked/x"), ["masked"]);
        assert!(f(Method::GET, "/api/maskedx").is_empty());
        assert!(f(Method::GET, "/api/account/signatures").is_empty());
        assert_eq!(
            f(Method::PUT, "/api/account/signatures/x"),
            ["compose.signatures"]
        );
        assert_eq!(
            f(Method::PUT, "/api/account/appearance/customization"),
            ["customisation.code"]
        );
        assert_eq!(
            f(Method::PUT, "/api/account/appearance"),
            ["appearance.custom"]
        );
        assert!(f(Method::GET, "/api/account/appearance").is_empty());
        assert_eq!(
            f(Method::PUT, "/api/settings/reader.zoom"),
            ["reading.zoom"]
        );
        assert_eq!(
            f(Method::PUT, "/api/settings/shortcuts.archive"),
            ["shortcuts"]
        );
        assert_eq!(
            f(Method::PUT, "/api/settings/links.trustedDomains"),
            ["privacy.trusted_links"]
        );
        assert!(f(Method::DELETE, "/api/settings/reader.zoom").is_empty());
        assert!(f(Method::PUT, "/api/settings/mail.quoteStyle").is_empty());
        assert_eq!(f(Method::POST, "/api/keys"), ["api_keys"]);
        assert!(f(Method::GET, "/api/keys").is_empty());
        // Every gated feature id is in the catalogue.
        for g in REST {
            assert!(
                crate::features::feature(g.feature).is_some(),
                "{}",
                g.feature
            );
        }
    }

    #[test]
    fn jmap_rules_detect_each_gated_use_and_allow_undoing_it() {
        let req = |calls: Value| json!({ "methodCalls": calls });
        let f = |calls: Value| jmap_features(&req(calls)).into_iter().collect::<Vec<_>>();
        assert_eq!(f(json!([["CryptoKey/get", {}, "a"]])), ["keys"]);
        assert_eq!(f(json!([["VacationResponse/set", {}, "a"]])), ["vacation"]);
        assert_eq!(
            f(
                json!([["EmailSubmission/set", {"create":{"s":{"emailId":"#d","sendAt":"2030-01-01T00:00:00Z"}}}, "a"]])
            ),
            ["compose.scheduled_send"]
        );
        assert!(
            f(json!([["EmailSubmission/set", {"update":{"s":{"sendAt":null}}}, "a"]])).is_empty()
        );
        assert_eq!(
            f(
                json!([["Email/set", {"create":{"d":{"htmlBody":[{"partId":"body","type":"text/html"}]}}}, "a"]])
            ),
            ["compose.rich_text"]
        );
        assert!(f(json!([["Email/set", {"create":{"d":{"textBody":[{"partId":"body"}],"keywords":{"$draft":true,"$seen":true}}}}, "a"]])).is_empty());
        assert_eq!(
            f(
                json!([["Email/set", {"create":{"d":{"attachments":[{"blobId":"b","disposition":"attachment"}]}}}, "a"]])
            ),
            ["compose.attachments"]
        );
        assert!(f(json!([["Email/set", {"create":{"d":{"attachments":[{"blobId":"b","disposition":"inline","cid":"c"}]}}}, "a"]])).is_empty());
        assert_eq!(
            f(json!([["Email/set", {"create":{"d":{"mailwomanRequestReadReceipt":true}}}, "a"]])),
            ["compose.read_receipts"]
        );
        assert_eq!(
            f(json!([["Email/set", {"update":{"m":{"keywords/work":true}}}, "a"]])),
            ["labels"]
        );
        assert!(f(json!([["Email/set", {"update":{"m":{"keywords/work":null,"keywords/$seen":true}}}, "a"]])).is_empty());
        assert_eq!(
            f(
                json!([["Email/set", {"update":{"m":{"snoozedUntil":"2030-01-01T00:00:00Z"}}}, "a"]])
            ),
            ["snooze"]
        );
        assert!(f(json!([["Email/set", {"update":{"m":{"snoozedUntil":null,"followUpAt":null}}}, "a"]])).is_empty());
        assert_eq!(
            f(json!([["Email/set", {"update":{"m":{"followUpAt":"2030-01-01"}}}, "a"]])),
            ["follow_up"]
        );
        assert_eq!(
            f(
                json!([["MailRule/set", {"create":{"r":{"actions":[{"kind":"forward","address":"x@y"}]}}}, "a"]])
            ),
            ["rules.forward"]
        );
        assert!(f(json!([["MailRule/set", {"create":{"r":{"actions":[{"kind":"move","mailbox":"x"}]}}}, "a"]])).is_empty());
        assert_eq!(
            f(json!([["Email/query", {"filter":{"inMailbox":"mwViewEvents"}}, "a"]])),
            ["views.events"]
        );
        assert_eq!(
            f(
                json!([["Email/query", {"filter":{"operator":"AND","conditions":[{"text":"from:ann report"}]}}, "a"]])
            ),
            ["search.operators"]
        );
        assert!(
            f(json!([["Email/query", {"filter":{"text":"meeting at 10:30 re: budget"}}, "a"]]))
                .is_empty()
        );
        assert!(f(json!([["Email/query", {"filter":{"hasAttachment":true}}, "a"]])).is_empty());
    }

    #[test]
    fn recipients_are_counted_per_message() {
        let r = json!({"methodCalls":[
            ["Email/set",{"create":{"d":{"to":[{"email":"a@x"},{"email":"b@x"}],"cc":[{"email":"c@x"}]}}},"a"],
            ["EmailSubmission/set",{"create":{"s":{"envelope":{"rcptTo":[{"email":"a@x"}]}}}},"b"]
        ]});
        assert_eq!(jmap_max_recipients(&r), 3);
    }

    #[test]
    fn percent_decoding_of_setting_keys() {
        assert_eq!(percent_decode("reader%2Ezoom"), "reader.zoom");
        assert_eq!(percent_decode("a%"), "a%");
    }
}
