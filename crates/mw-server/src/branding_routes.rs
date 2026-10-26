//! Global, same-origin branding. Uploaded images are private until published.
use crate::{
    AppState,
    admin_ctx::{AdminApiError, AdminCtx, AdminJson, etag_of, with_etag},
    admin_router::AdminRegistrar,
    capabilities::Capability,
};
use axum::{
    Json, Router,
    extract::{Path, State},
    http::{HeaderMap, Method, StatusCode, header},
    response::{IntoResponse, Response},
    routing::get,
};
use base64::Engine as _;
use mw_store::branding::{BrandAsset, BrandImages, BrandingSnapshot, BrandingUpdate};
use serde::Deserialize;
use serde_json::{Value, json};
use sha2::{Digest, Sha256};

static DECODERS: tokio::sync::Semaphore = tokio::sync::Semaphore::const_new(2);
pub(crate) fn register(r: &mut impl AdminRegistrar) {
    r.admin_endpoint(Method::GET, "/branding", Capability::BrandingRead, read);
    r.admin_endpoint(Method::PUT, "/branding", Capability::BrandingManage, save);
    r.admin_endpoint(
        Method::POST,
        "/branding/assets",
        Capability::BrandingManage,
        upload,
    );
}
pub(crate) fn router() -> Router<AppState> {
    Router::new()
        .route("/api/assets/{id}", get(asset))
        .route("/manifest.webmanifest", get(manifest))
}
pub(crate) fn image_urls(images: &BrandImages) -> Value {
    let url = |id: &Option<String>| id.as_ref().map(|id| format!("/api/assets/{id}"));
    json!({"logoUrl":url(&images.logo_id),"faviconUrl":url(&images.icon_master_id),"loginBackgroundUrl":url(&images.login_background_id),"pwa192Url":url(&images.pwa192_id),"pwa512Url":url(&images.pwa512_id),"pwaMaskableUrl":url(&images.pwa_maskable_id)})
}
/// Relative paths retain the externally mounted application base path. No user
/// identifiers or authentication state are embedded in an installed app identity.
async fn manifest(State(state): State<AppState>) -> Response {
    let snapshot = match state.store.branding_snapshot().await {
        Ok(snapshot) => snapshot,
        Err(_) => return StatusCode::SERVICE_UNAVAILABLE.into_response(),
    };
    let name = view(&snapshot)["brandName"].clone();
    let images = &snapshot.images;
    let icon = |id: &Option<String>, fallback: &str| {
        id.as_ref()
            .map_or_else(|| fallback.to_owned(), |id| format!("api/assets/{id}"))
    };
    let value = json!({
        "name": name, "short_name": name, "start_url":"./", "scope":"./",
        "display":"standalone", "theme_color":"#111111", "background_color":"#111111",
        "icons":[
            {"src":icon(&images.pwa192_id,"icons/icon-192.png"),"sizes":"192x192","type":"image/png","purpose":"any"},
            {"src":icon(&images.pwa512_id,"icons/icon-512.png"),"sizes":"512x512","type":"image/png","purpose":"any"},
            {"src":icon(&images.pwa_maskable_id,"icons/icon-maskable-512.png"),"sizes":"512x512","type":"image/png","purpose":"maskable"}
        ]
    });
    (
        [
            (header::CONTENT_TYPE, "application/manifest+json"),
            (header::CACHE_CONTROL, "no-cache"),
            (header::X_CONTENT_TYPE_OPTIONS, "nosniff"),
        ],
        value.to_string(),
    )
        .into_response()
}
pub(crate) fn view(s: &BrandingSnapshot) -> Value {
    json!({"brandName":s.name_json.as_deref().and_then(|v| serde_json::from_str::<Value>(v).ok()).filter(|v| !v.is_null()).unwrap_or(json!("Mailwoman")),
        "loginNotice":s.notice_json.as_deref().and_then(|v| serde_json::from_str::<Value>(v).ok()),
        "images":s.images,"assets":image_urls(&s.images)})
}
fn tag(s: &BrandingSnapshot) -> String {
    etag_of(&serde_json::to_value(s).expect("branding snapshot"))
}
async fn read(State(state): State<AppState>, ctx: AdminCtx) -> Result<Response, AdminApiError> {
    ctx.require_unscoped(Capability::BrandingRead)?;
    let s = state.store.branding_snapshot().await?;
    Ok(with_etag(Json(view(&s)), &tag(&s)))
}
#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct Save {
    brand_name: String,
    login_notice: Option<String>,
    images: BrandImages,
}
async fn save(
    State(state): State<AppState>,
    ctx: AdminCtx,
    headers: HeaderMap,
    AdminJson(body): AdminJson<Save>,
) -> Result<Response, AdminApiError> {
    ctx.require_unscoped(Capability::BrandingManage)?;
    let name = body.brand_name.trim();
    for id in [
        &body.images.logo_id,
        &body.images.icon_master_id,
        &body.images.login_background_id,
        &body.images.pwa192_id,
        &body.images.pwa512_id,
        &body.images.pwa_maskable_id,
    ]
    .into_iter()
    .flatten()
    {
        if id.len() != 64
            || !id
                .bytes()
                .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
        {
            return Err(AdminApiError::invalid(
                "invalidAsset",
                "Select a locally uploaded branding image",
                "/images",
            ));
        }
    }
    if name.is_empty() || name.chars().count() > 64 || name.chars().any(char::is_control) {
        return Err(AdminApiError::invalid(
            "invalidBrandName",
            "Use 1–64 printable characters",
            "/brandName",
        ));
    }
    if body.login_notice.as_ref().is_some_and(|v| {
        v.chars().count() > 500 || v.chars().any(|c| c.is_control() && c != '\n' && c != '\t')
    }) {
        return Err(AdminApiError::invalid(
            "invalidNotice",
            "Notice is limited to 500 characters",
            "/loginNotice",
        ));
    }
    let before = state.store.branding_snapshot().await?;
    if headers.get(header::IF_MATCH).and_then(|v| v.to_str().ok()) != Some(tag(&before).as_str()) {
        return Err(AdminApiError::stale(None, None));
    }
    if !state
        .store
        .put_branding(
            &before,
            BrandingUpdate {
                images: &body.images,
                name_json: &json!(name).to_string(),
                notice_json: &json!(body.login_notice).to_string(),
                actor: &ctx.username,
                now: chrono::Utc::now().timestamp(),
            },
        )
        .await?
    {
        if state.store.branding_snapshot().await? != before {
            return Err(AdminApiError::stale(None, None));
        }
        return Err(AdminApiError::invalid(
            "brandingConflict",
            "Branding changed or an image is not available for this purpose; reload",
            "/images",
        ));
    }
    crate::admin_accounts::audit(
        &state,
        &ctx,
        "branding-changed",
        "branding:global",
        Some(view(&before)),
        Some(json!({"brandName":name,"images":body.images})),
    )
    .await;
    let s = state.store.branding_snapshot().await?;
    Ok(with_etag(Json(view(&s)), &tag(&s)))
}
#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct Upload {
    kind: String,
    bytes_base64: String,
}
fn invalid_image() -> AdminApiError {
    AdminApiError::invalid(
        "invalidImage",
        "Use a PNG or JPEG within the displayed size limits",
        "/bytesBase64",
    )
}
async fn upload(
    State(state): State<AppState>,
    ctx: AdminCtx,
    AdminJson(body): AdminJson<Upload>,
) -> Result<Response, AdminApiError> {
    ctx.require_unscoped(Capability::BrandingManage)?;
    let max = match body.kind.as_str() {
        "logo" => 512,
        "icon_master" => 1024,
        "background" => 2560,
        "pwa_192" => 192,
        "pwa_512" | "pwa_maskable" => 512,
        _ => return Err(invalid_image()),
    };
    // Keep the JSON request beneath the global 2 MiB request limit as well.
    if body.bytes_base64.len() > 1_400_000 {
        return Err(invalid_image());
    }
    let bytes = base64::engine::general_purpose::STANDARD
        .decode(body.bytes_base64)
        .map_err(|_| invalid_image())?;
    let (width, height) = dimensions(&bytes).ok_or_else(invalid_image)?;
    if width == 0 || height == 0 || width > max || height > max {
        return Err(invalid_image());
    }
    if body.kind.starts_with("pwa_") && (width != max || height != max) {
        return Err(AdminApiError::invalid(
            "invalidIconSize",
            "Installed app icons must use the exact square dimensions shown",
            "/bytesBase64",
        ));
    }
    let permit = DECODERS.try_acquire().map_err(|_| {
        AdminApiError::new(
            StatusCode::TOO_MANY_REQUESTS,
            "busy",
            "Image decoders are busy",
        )
    })?;
    let png = tokio::task::spawn_blocking(move || {
        let _permit = permit;
        mw_render::media_jail::reencode_image(&bytes)
    })
    .await
    .map_err(|_| invalid_image())?
    .map_err(|_| invalid_image())?;
    if png.len() > 4 * 1024 * 1024 || dimensions(&png) != Some((width, height)) {
        return Err(invalid_image());
    }
    // Include kind so a reused picture can independently serve logo and icon roles.
    let mut hash = Sha256::new();
    hash.update(body.kind.as_bytes());
    hash.update(&png);
    let id: String = hash.finalize().iter().map(|b| format!("{b:02x}")).collect();
    state
        .store
        .put_brand_asset(
            &BrandAsset {
                id: id.clone(),
                // Installed-app variants share the existing icon asset class;
                // slot-specific dimensions are also enforced when publishing.
                kind: if body.kind.starts_with("pwa_") {
                    "icon_master".into()
                } else {
                    body.kind.clone()
                },
                data: png,
                width: width.into(),
                height: height.into(),
            },
            &ctx.username,
            chrono::Utc::now().timestamp(),
        )
        .await?;
    crate::admin_accounts::audit(
        &state,
        &ctx,
        "branding-image-uploaded",
        &format!("brand-asset:{id}"),
        None,
        Some(json!({"kind":body.kind,"width":width,"height":height})),
    )
    .await;
    Ok(Json(json!({"id":id,"kind":body.kind,"width":width,"height":height})).into_response())
}
async fn asset(State(state): State<AppState>, Path(id): Path<String>) -> Response {
    if id.len() != 64 || !id.bytes().all(|b| b.is_ascii_hexdigit()) {
        return StatusCode::NOT_FOUND.into_response();
    }
    match state.store.published_brand_asset(&id).await {
        Ok(Some(data)) => (
            [
                (header::CONTENT_TYPE, "image/png"),
                (header::X_CONTENT_TYPE_OPTIONS, "nosniff"),
                (header::CACHE_CONTROL, "public, max-age=31536000, immutable"),
            ],
            data,
        )
            .into_response(),
        Ok(None) => StatusCode::NOT_FOUND.into_response(),
        Err(_) => StatusCode::SERVICE_UNAVAILABLE.into_response(),
    }
}
/// Header-only dimensions, before handing any compressed pixel data to the jail.
fn dimensions(b: &[u8]) -> Option<(u32, u32)> {
    if b.starts_with(b"\x89PNG\r\n\x1a\n") && b.get(12..16) == Some(b"IHDR") {
        return Some((
            u32::from_be_bytes(b.get(16..20)?.try_into().ok()?),
            u32::from_be_bytes(b.get(20..24)?.try_into().ok()?),
        ));
    }
    if b.starts_with(&[0xff, 0xd8]) {
        let mut p = 2;
        while p < b.len() {
            if *b.get(p)? != 0xff {
                return None;
            }
            p += 1;
            while *b.get(p)? == 0xff {
                p += 1;
            }
            let marker = *b.get(p)?;
            p += 1;
            if marker == 0xda || marker == 0xd9 {
                return None;
            }
            let n = u16::from_be_bytes(b.get(p..p + 2)?.try_into().ok()?) as usize;
            if n < 2 || p.checked_add(n)? > b.len() {
                return None;
            }
            if matches!(marker,0xc0..=0xc3|0xc5..=0xc7|0xc9..=0xcb|0xcd..=0xcf) {
                if n < 7 {
                    return None;
                }
                return Some((
                    u16::from_be_bytes(b.get(p + 5..p + 7)?.try_into().ok()?) as u32,
                    u16::from_be_bytes(b.get(p + 3..p + 5)?.try_into().ok()?) as u32,
                ));
            }
            p += n;
        }
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn image_headers_refuse_active_or_truncated_content() {
        assert_eq!(dimensions(b"<svg/>"), None);
        assert_eq!(dimensions(b"\xff\xd8\xff\xc0\x00\x01"), None);
        assert_eq!(dimensions(b"RIFF\x00\x00\x00\x00WEBPVP8X"), None);
        assert_eq!(
            dimensions(include_bytes!(
                "../../../apps/web/public/apple-touch-icon.png"
            )),
            Some((180, 180))
        );
    }
}
