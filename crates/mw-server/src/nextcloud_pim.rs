//! Explicit account-owned discovery, selection and synchronization of cloud PIM.
use crate::{
    AppState,
    nextcloud_pim_transport::{ManagedTransport, account_target, collection_url},
};
use axum::{
    Json, Router,
    extract::{Path, Query, State},
    http::{HeaderMap, StatusCode},
    response::{IntoResponse, Response},
    routing::{delete, get, post},
};
use mw_dav::TransportRequest;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::sync::Arc;

pub(crate) fn router() -> Router<AppState> {
    Router::new()
        .route("/api/account/nextcloud/pim", get(list))
        .route(
            "/api/account/nextcloud/pim/discover/{instance}",
            post(discover),
        )
        .route("/api/account/nextcloud/pim/apply/{instance}", post(apply))
        .route("/api/account/nextcloud/pim/create/{instance}", post(create))
        .route("/api/account/nextcloud/pim/sync", post(sync))
        .route(
            "/api/account/nextcloud/pim/collections/{collection}",
            delete(disconnect),
        )
}
fn failure(code: StatusCode, message: &str) -> Response {
    (code, Json(json!({"error":message}))).into_response()
}
#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct Selection {
    account_id: String,
    kind: String,
    href: Option<String>,
}
#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct Creation {
    account_id: String,
    kind: String,
    name: String,
}
#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct Fence {
    account_id: String,
}
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct Discovered {
    href: String,
    name: String,
    kind: String,
}
async fn admitted(
    state: &AppState,
    headers: &HeaderMap,
    expected: &str,
) -> Result<mw_store::Session, Response> {
    let session =
        crate::user_router::feature_session(state, headers, "integrations.nextcloud").await?;
    if session.account_id != expected {
        return Err(failure(StatusCode::CONFLICT, "Account changed; reload."));
    }
    if state.engine.is_none() {
        return Err(failure(
            StatusCode::NOT_IMPLEMENTED,
            "Cloud PIM requires the native mail engine.",
        ));
    }
    Ok(session)
}
async fn preview(
    state: &AppState,
    account: &str,
    instance_id: &str,
    kind: &str,
) -> Result<(Vec<Discovered>, String), Response> {
    let family = match kind {
        "calendar" => "calendar",
        "tasks" => "tasks",
        "contacts" => "contacts",
        _ => return Err(failure(StatusCode::BAD_REQUEST, "Unknown PIM category.")),
    };
    let (instance, link, _) = state
        .store
        .nextcloud_authority(account, instance_id)
        .await
        .map_err(|_| failure(StatusCode::SERVICE_UNAVAILABLE, "Link unavailable."))?
        .ok_or_else(|| failure(StatusCode::CONFLICT, "Link this instance first."))?;
    let transport = ManagedTransport::discovery(
        state.store.clone(),
        account.into(),
        instance_id.into(),
        link.revision.clone(),
        kind.into(),
    );
    let url = collection_url(&instance.base_url, &link.username, kind)
        .map_err(|_| failure(StatusCode::BAD_REQUEST, "Invalid instance."))?;
    let response = transport
        .send(TransportRequest {
            kind: Some(family),
            method: "PROPFIND".into(),
            url: url.to_string(),
            depth: Some("1".into()),
            content_type: Some("application/xml".into()),
            body: Some(mw_dav::request::propfind_collections()),
            if_match: None,
            if_none_match: None,
        })
        .await
        .map_err(|_| {
            failure(
                StatusCode::BAD_GATEWAY,
                "Cloud discovery refused or timed out.",
            )
        })?;
    if !matches!(response.status, 200 | 207) {
        return Err(failure(
            StatusCode::BAD_GATEWAY,
            "Cloud refused discovery; check the account link.",
        ));
    }
    let dav_kind = if kind == "contacts" {
        mw_dav::request::DavKind::CardDav
    } else {
        mw_dav::request::DavKind::CalDav
    };
    let rows = mw_dav::response::parse_collections(&response.body, dav_kind).map_err(|_| {
        failure(
            StatusCode::BAD_GATEWAY,
            "Invalid cloud collection response.",
        )
    })?;
    if rows.len() > 128 {
        return Err(failure(
            StatusCode::BAD_GATEWAY,
            "Cloud returned too many collections.",
        ));
    }
    let mut output = Vec::new();
    for row in rows {
        if kind != "contacts"
            && !row.components.is_empty()
            && !row
                .components
                .iter()
                .any(|c| c == if kind == "tasks" { "VTODO" } else { "VEVENT" })
        {
            continue;
        }
        let href = url
            .join(&row.href)
            .map_err(|_| failure(StatusCode::BAD_GATEWAY, "Invalid collection URL."))?;
        if !account_target(&href, &instance.base_url, &link.username, false).unwrap_or(false)
            || !href.path().starts_with(url.path())
            || !href.path().ends_with('/')
        {
            return Err(failure(
                StatusCode::BAD_GATEWAY,
                "Cloud collection escaped the selected account.",
            ));
        }
        if row.display_name.len() > 512 {
            return Err(failure(
                StatusCode::BAD_GATEWAY,
                "Cloud collection name is too long.",
            ));
        }
        output.push(Discovered {
            href: href.to_string(),
            name: row.display_name,
            kind: kind.into(),
        });
    }
    Ok((output, link.revision))
}
async fn discover(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(instance): Path<String>,
    Json(body): Json<Selection>,
) -> Response {
    let session = match admitted(&state, &headers, &body.account_id).await {
        Ok(s) => s,
        Err(e) => return e,
    };
    match preview(&state, &session.account_id, &instance, &body.kind).await {
        Ok((items, _)) => {
            Json(json!({"accountId":session.account_id,"collections":items})).into_response()
        }
        Err(e) => e,
    }
}
async fn apply(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(instance): Path<String>,
    Json(body): Json<Selection>,
) -> Response {
    let session = match admitted(&state, &headers, &body.account_id).await {
        Ok(s) => s,
        Err(e) => return e,
    };
    connect(
        &state,
        &session,
        &instance,
        &body.kind,
        body.href.as_deref(),
    )
    .await
}
/// A slug for a new collection URL: lowercase ASCII letters, digits and dashes
/// from the display name, then a random suffix so it never names an existing one.
pub(crate) fn collection_slug(name: &str) -> String {
    let mut slug = String::new();
    for ch in name.chars().flat_map(char::to_lowercase) {
        if ch.is_ascii_alphanumeric() {
            slug.push(ch);
        } else if !slug.ends_with('-') && !slug.is_empty() {
            slug.push('-');
        }
        if slug.len() >= 40 {
            break;
        }
    }
    let slug = slug.trim_end_matches('-');
    let suffix = &uuid::Uuid::new_v4().simple().to_string()[..8];
    if slug.is_empty() {
        format!("calendar-{suffix}")
    } else {
        format!("{slug}-{suffix}")
    }
}
/// `POST /api/account/nextcloud/pim/create/{instance} {accountId, kind, name}`:
/// `MKCALENDAR` a new calendar or task list in the linked account's calendar
/// home, then connect it like a discovered one.
async fn create(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(instance): Path<String>,
    Json(body): Json<Creation>,
) -> Response {
    let session = match admitted(&state, &headers, &body.account_id).await {
        Ok(s) => s,
        Err(e) => return e,
    };
    if !matches!(body.kind.as_str(), "calendar" | "tasks") {
        return failure(
            StatusCode::BAD_REQUEST,
            "Only calendars and task lists can be created.",
        );
    }
    let name = body.name.trim();
    if name.is_empty() || name.chars().count() > 200 || name.chars().any(char::is_control) {
        return failure(
            StatusCode::BAD_REQUEST,
            "Use a name of 1 to 200 characters.",
        );
    }
    let (row, link, _) = match state
        .store
        .nextcloud_authority(&session.account_id, &instance)
        .await
    {
        Ok(Some(found)) => found,
        Ok(None) => return failure(StatusCode::CONFLICT, "Link this instance first."),
        Err(_) => return failure(StatusCode::SERVICE_UNAVAILABLE, "Link unavailable."),
    };
    let href = match collection_url(&row.base_url, &link.username, "calendar")
        .ok()
        .and_then(|home| home.join(&format!("{}/", collection_slug(name))).ok())
    {
        Some(href) => href,
        None => return failure(StatusCode::BAD_REQUEST, "Invalid instance."),
    };
    let transport = crate::nextcloud_pim_transport::ManagedTransport::creation(
        state.store.clone(),
        session.account_id.clone(),
        instance.clone(),
        link.revision.clone(),
        body.kind.clone(),
    );
    let client = mw_dav::DavClient::with_transport(config(), transport);
    if let Err(error) = client.make_calendar(href.as_str(), name).await {
        let status = match error {
            mw_dav::DavError::Status { status: 403, .. } => StatusCode::FORBIDDEN,
            _ => StatusCode::BAD_GATEWAY,
        };
        return failure(status, "Nextcloud did not create the collection.");
    }
    connect(&state, &session, &instance, &body.kind, Some(href.as_str())).await
}
async fn connect(
    state: &AppState,
    session: &mw_store::Session,
    instance: &str,
    kind: &str,
    href: Option<&str>,
) -> Response {
    let (items, revision) = match preview(state, &session.account_id, instance, kind).await {
        Ok(v) => v,
        Err(e) => return e,
    };
    let Some(item) = items
        .into_iter()
        .find(|item| Some(item.href.as_str()) == href)
    else {
        return failure(StatusCode::CONFLICT, "Collection changed; discover again.");
    };
    let instance = instance.to_owned();
    let saved = match state
        .store
        .apply_nextcloud_pim(mw_store::nextcloud_pim::NextcloudPimApply {
            account_id: &session.account_id,
            instance_id: &instance,
            link_revision: &revision,
            kind: &item.kind,
            href: &item.href,
            name: &item.name,
        })
        .await
    {
        Ok(Some(v)) => v,
        Ok(None) => {
            return failure(
                StatusCode::CONFLICT,
                "Authority changed or collection limit reached.",
            );
        }
        Err(_) => {
            return failure(
                StatusCode::SERVICE_UNAVAILABLE,
                "Could not save collection.",
            );
        }
    };
    if let Some(engine) = &state.engine {
        install(engine, &state.store, &session.account_id);
    }
    Json(json!({"accountId":session.account_id,"collectionId":saved.collection_id,"href":item.href,"name":item.name,"syncRequired":true})).into_response()
}
pub(crate) fn config() -> mw_dav::DavConfig {
    // Managed requests use absolute bound URLs; this placeholder can never pass authority validation.
    mw_dav::DavConfig {
        base_url: "https://unbound-nextcloud.invalid/".into(),
        ..Default::default()
    }
}
pub(crate) fn install(engine: &Arc<mw_engine::Engine>, store: &mw_store::Store, account: &str) {
    engine.configure_dav_transport(
        account,
        config(),
        ManagedTransport::runtime(store.clone(), account.into()),
    );
}
async fn list(
    State(state): State<AppState>,
    headers: HeaderMap,
    Query(body): Query<Fence>,
) -> Response {
    let session = match crate::authed(&state, &headers).await {
        Ok(s) => s,
        Err(e) => return e,
    };
    if session.account_id != body.account_id {
        return failure(StatusCode::CONFLICT, "Account changed; reload.");
    }
    let rows = match state
        .store
        .nextcloud_pim_bindings(&session.account_id)
        .await
    {
        Ok(rows) => rows,
        Err(_) => return failure(StatusCode::SERVICE_UNAVAILABLE, "Collections unavailable."),
    };
    let links = match state.store.nextcloud_links(&session.account_id).await {
        Ok(links) => links,
        Err(_) => return failure(StatusCode::SERVICE_UNAVAILABLE, "Links unavailable."),
    };
    let instances = match state.store.nextcloud_instances().await {
        Ok(instances) => instances,
        Err(_) => return failure(StatusCode::SERVICE_UNAVAILABLE, "Instances unavailable."),
    };
    let mut collections = Vec::new();
    for row in rows {
        let name = if row.kind == "contacts" {
            state
                .store
                .get_address_book(&row.collection_id)
                .await
                .ok()
                .flatten()
                .map(|book| book.name)
        } else {
            state
                .store
                .get_calendar(&row.collection_id)
                .await
                .ok()
                .flatten()
                .map(|cal| cal.name)
        };
        let ready = row.enabled
            && links.iter().any(|link| {
                link.instance_id == row.instance_id
                    && link.revision == row.link_revision
                    && instances.iter().any(|instance| {
                        instance.id == link.instance_id
                            && instance.enabled
                            && instance.revision == link.instance_revision
                    })
            });
        collections.push(json!({"id":row.collection_id,"instanceId":row.instance_id,"kind":row.kind,"enabled":row.enabled,"name":name.unwrap_or_default(),"ready":ready}));
    }
    Json(json!({"accountId":session.account_id,"supported":state.engine.is_some(),"collections":collections})).into_response()
}
async fn disconnect(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(collection): Path<String>,
    Json(body): Json<Fence>,
) -> Response {
    let session = match crate::authed(&state, &headers).await {
        Ok(s) => s,
        Err(e) => return e,
    };
    if session.account_id != body.account_id {
        return failure(StatusCode::CONFLICT, "Account changed; reload.");
    }
    match state
        .store
        .disconnect_nextcloud_pim(&session.account_id, &collection)
        .await
    {
        Ok(true) => StatusCode::NO_CONTENT.into_response(),
        Ok(false) => StatusCode::NOT_FOUND.into_response(),
        Err(_) => StatusCode::SERVICE_UNAVAILABLE.into_response(),
    }
}
async fn sync(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(body): Json<Fence>,
) -> Response {
    let session = match admitted(&state, &headers, &body.account_id).await {
        Ok(s) => s,
        Err(e) => return e,
    };
    let engine = state.engine.as_ref().unwrap();
    install(engine, &state.store, &session.account_id);
    match tokio::time::timeout(
        std::time::Duration::from_secs(30),
        engine.sync_current_pim(&session.account_id),
    )
    .await
    {
        Ok(Ok(())) => Json(json!({"ok":true})).into_response(),
        Ok(Err(_)) => failure(
            StatusCode::BAD_GATEWAY,
            "Some collections could not synchronize; cached data is preserved.",
        ),
        Err(_) => failure(
            StatusCode::GATEWAY_TIMEOUT,
            "Synchronization timed out; retry explicitly.",
        ),
    }
}
