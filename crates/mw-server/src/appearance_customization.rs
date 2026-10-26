//! Owner-only appearance code documents. No script is executed on the server.
use crate::{
    AppState,
    admin_ctx::{AdminApiError, AdminCtx, AdminJson, check_if_match, etag_of, with_etag},
    admin_router::{Access, AdminRegistrar},
};
use axum::{
    Json, Router,
    extract::{DefaultBodyLimit, Query, State},
    http::{HeaderMap, Method, StatusCode},
    response::{IntoResponse, Response},
    routing::get,
};
use serde::{Deserialize, Serialize};
use serde_json::json;

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct Document {
    pub version: u8,
    pub css: String,
    pub script: String,
}
impl Default for Document {
    fn default() -> Self {
        Self {
            version: 1,
            css: String::new(),
            script: String::new(),
        }
    }
}
#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct Mutation {
    owner_id: String,
    document: Document,
}
#[derive(Deserialize, Default)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct Expected {
    account_id: Option<String>,
    username: Option<String>,
}
#[derive(Clone, Copy)]
enum Context {
    Mailbox,
    Admin,
}
impl Context {
    fn scope(self) -> mw_sanitize::appearance::Scope {
        match self {
            Self::Mailbox => mw_sanitize::appearance::Scope::Mailbox,
            Self::Admin => mw_sanitize::appearance::Scope::Admin,
        }
    }
    fn key(self, owner: &str) -> String {
        format!(
            "appearance-customization:{}:{owner}",
            match self {
                Self::Mailbox => "mailbox",
                Self::Admin => "admin",
            }
        )
    }
}
pub(crate) fn router() -> Router<AppState> {
    Router::new()
        .route("/api/appearance/sandbox", get(sandbox))
        .route(
            "/api/account/appearance/customization",
            get(user_read).put(user_write),
        )
        .layer(DefaultBodyLimit::max(96 * 1024))
}
pub(crate) fn register(r: &mut impl AdminRegistrar) {
    r.admin_endpoint(
        Method::GET,
        "/me/appearance-customization",
        Access::SignedIn,
        admin_read,
    );
    r.admin_endpoint(
        Method::PUT,
        "/me/appearance-customization",
        Access::SignedIn,
        admin_write,
    );
}
fn invalid(message: &'static str) -> AdminApiError {
    AdminApiError::invalid("invalidAppearanceCustomization", message, "/document")
}
fn changed() -> AdminApiError {
    AdminApiError::new(
        StatusCode::CONFLICT,
        "accountChanged",
        "The signed-in owner changed. Reload before saving.",
    )
}
fn validate(document: &Document, context: Context) -> Result<String, AdminApiError> {
    if document.version != 1 || document.script.len() > 16 * 1024 || document.script.contains('\0')
    {
        return Err(invalid(
            "Use document version1 and a script of at most16 KiB without NUL characters.",
        ));
    }
    mw_sanitize::appearance::stylesheet(&document.css, context.scope()).map_err(invalid)
}
async fn stored(state: &AppState, key: &str) -> Result<(Option<String>, Document), AdminApiError> {
    let raw = state
        .store
        .get_setting(key)
        .await
        .map_err(AdminApiError::internal)?;
    let document = match raw.as_deref() {
        Some(raw) => {
            serde_json::from_str(raw).map_err(|_| AdminApiError::internal("appearance document"))?
        }
        None => Document::default(),
    };
    Ok((raw, document))
}
async fn flags(state: &AppState) -> Result<(bool, bool), AdminApiError> {
    let subject = mw_store::settings_policy::SettingSubject::default();
    let mut flags = [false; 2];
    for (index, key) in ["appearance.customCssEnabled", "appearance.customJsEnabled"]
        .into_iter()
        .enumerate()
    {
        flags[index] = crate::settings_registry::effective_for(&state.store, key, &subject)
            .await
            .map_err(AdminApiError::internal)?
            .is_some_and(|value| value.value == true);
    }
    Ok((flags[0], flags[1]))
}
async fn response(
    state: &AppState,
    context: Context,
    owner: &str,
    locks: Vec<String>,
) -> Result<Response, AdminApiError> {
    let (_, document) = stored(state, &context.key(owner)).await?;
    let compiled = validate(&document, context)?;
    let (css_allowed, script_allowed) = flags(state).await?;
    let css_enabled = css_allowed && locks.is_empty();
    let tag = etag_of(&json!(document));
    Ok(with_etag(
        (
            [("cache-control", "private, no-store")],
            Json(json!({
                "ownerId":owner,"document":document,"compiledCss":if css_enabled { compiled } else { String::new() },
                "cssEnabled":css_enabled,"scriptEnabled":script_allowed,"cssAllowed":css_allowed,
                "lockedFields":locks,"scopeClass":context.scope().class()
            })),
        ),
        &tag,
    ))
}
async fn save(
    state: &AppState,
    context: Context,
    owner: &str,
    headers: &HeaderMap,
    mutation: Mutation,
) -> Result<(), AdminApiError> {
    if mutation.owner_id != owner {
        return Err(changed());
    }
    validate(&mutation.document, context)?;
    if !headers.contains_key("if-match") {
        return Err(AdminApiError::new(
            StatusCode::PRECONDITION_REQUIRED,
            "ifMatchRequired",
            "Reload the appearance document before saving.",
        ));
    }
    let key = context.key(owner);
    let (raw, previous) = stored(state, &key).await?;
    check_if_match(headers, &etag_of(&json!(previous)), None, None)?;
    let text = serde_json::to_string(&mutation.document).expect("appearance document");
    if !state
        .store
        .compare_and_set_setting(&key, raw.as_deref(), &text)
        .await
        .map_err(AdminApiError::internal)?
    {
        return Err(AdminApiError::stale(None, None));
    }
    Ok(())
}
async fn user(
    state: &AppState,
    headers: &HeaderMap,
    expected: &Expected,
) -> Result<(mw_store::Session, Vec<String>), Response> {
    let session = crate::authed(state, headers).await?;
    if expected.account_id.as_deref() != Some(&session.account_id) {
        return Err(changed().into_response());
    }
    let locks = crate::prefs_routes::appearance_locks(state, &session)
        .await?
        .into_iter()
        .filter_map(|(key, _)| match key {
            "appearance.theme" => Some("mode".to_owned()),
            "appearance.accent" => Some("accent".to_owned()),
            "appearance.density" => Some("density".to_owned()),
            "appearance.font" => Some("font".to_owned()),
            _ => None,
        })
        .collect();
    Ok((session, locks))
}
async fn user_read(
    State(state): State<AppState>,
    headers: HeaderMap,
    Query(expected): Query<Expected>,
) -> Response {
    let (session, locks) = match user(&state, &headers, &expected).await {
        Ok(value) => value,
        Err(response) => return response,
    };
    response(&state, Context::Mailbox, &session.account_id, locks)
        .await
        .unwrap_or_else(IntoResponse::into_response)
}
async fn user_write(
    State(state): State<AppState>,
    headers: HeaderMap,
    Query(expected): Query<Expected>,
    Json(mutation): Json<Mutation>,
) -> Response {
    let (session, locks) = match user(&state, &headers, &expected).await {
        Ok(value) => value,
        Err(response) => return response,
    };
    if let Err(error) = save(
        &state,
        Context::Mailbox,
        &session.account_id,
        &headers,
        mutation,
    )
    .await
    {
        return error.into_response();
    }
    response(&state, Context::Mailbox, &session.account_id, locks)
        .await
        .unwrap_or_else(IntoResponse::into_response)
}
fn admin_owner(ctx: &AdminCtx, expected: &Expected) -> Result<String, AdminApiError> {
    if expected.username.as_deref() != Some(ctx.username.as_str()) {
        return Err(changed());
    }
    Ok(ctx
        .account_id
        .clone()
        .unwrap_or_else(|| format!("environment:{}", ctx.username)))
}
async fn admin_read(
    State(state): State<AppState>,
    ctx: AdminCtx,
    Query(expected): Query<Expected>,
) -> Result<Response, AdminApiError> {
    let owner = admin_owner(&ctx, &expected)?;
    response(&state, Context::Admin, &owner, Vec::new()).await
}
async fn admin_write(
    State(state): State<AppState>,
    ctx: AdminCtx,
    headers: HeaderMap,
    Query(expected): Query<Expected>,
    AdminJson(mutation): AdminJson<Mutation>,
) -> Result<Response, AdminApiError> {
    let owner = admin_owner(&ctx, &expected)?;
    save(&state, Context::Admin, &owner, &headers, mutation).await?;
    response(&state, Context::Admin, &owner, Vec::new()).await
}

/// Only the fixed public sandbox response may replace the shell CSP.
#[derive(Clone, Copy)]
pub(crate) struct AppearanceSandboxResponse;
async fn sandbox() -> Response {
    use base64::Engine as _;
    use sha2::Digest as _;
    // HTML parsing normalizes CRLF/CR to LF before CSP hashes inline scripts.
    let bootstrap = include_str!("appearance-sandbox.js")
        .replace("\r\n", "\n")
        .replace('\r', "\n");
    let hash = base64::engine::general_purpose::STANDARD
        .encode(sha2::Sha256::digest(bootstrap.as_bytes()));
    let policy = format!(
        "default-src 'none'; script-src 'sha256-{hash}' 'unsafe-eval'; worker-src blob:; connect-src 'none'; img-src 'none'; font-src 'none'; frame-src 'none'; style-src 'none'; form-action 'none'; base-uri 'none'; frame-ancestors 'self'; sandbox allow-scripts"
    );
    let mut response=([( "content-security-policy",policy),("cache-control", "no-store".to_owned())],axum::response::Html(format!("<!doctype html><html><head><meta charset=\"utf-8\"><title>Isolated appearance worker</title></head><body><script>{bootstrap}</script></body></html>"))).into_response();
    response.extensions_mut().insert(AppearanceSandboxResponse);
    response
}
