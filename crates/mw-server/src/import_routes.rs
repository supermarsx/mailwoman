//! Mailbox import (W18) + the server-side webcal sync driver (P6, e11 handoff)
//! — t16 26.16, e10.
//!
//! # Import (W18)
//! Bulk-ingest a user's existing mail into their Mailwoman account from the three
//! portable on-disk formats:
//!   * **mbox** — one `From `-delimited stream ([`mw_export::split_mbox`]);
//!   * **EML** — a single RFC822 message;
//!   * **Maildir** — the per-message files of a maildir, uploaded as one
//!     uncompressed `tar` (the standard way to move a maildir over HTTP); the
//!     regular files under `cur/`/`new/` are imported.
//!
//! Every parsed message is sealed to the upload backend ([`Engine::store_upload`])
//! and then handed to `Email/import` (RFC 8621 §4.8) via the engine's JMAP surface,
//! so import reuses the exact ingest path a client `Email/import` would — no second
//! ingest code path. Requires engine mode + a configured upload backend
//! (`MW_UPLOAD_DIR`); otherwise it fails loudly rather than dropping bytes.
//!
//! ## Request size (t20 26.19, B7)
//! Each body-carrying import route declares an **explicit**
//! [`DefaultBodyLimit`] of [`MAX_MESSAGE_BYTES`]. Without it axum applies its
//! implicit 2 MB default to the `Bytes` extractor, so every import over 2 MB was
//! refused — with a bare, non-JSON `413` — no matter how the fronting proxy was
//! configured, and this module's own `MAX_*` constants were unreachable. The
//! limit is now the module constant, and an overflow answers with the
//! crate-shared JSON body ([`crate::payload_too_large`]) that `/jmap/upload` and
//! the blob-download path already use, so a refusal by the *app* stays
//! distinguishable from one by a proxy.
//!
//! # Webcal sync driver (P6)
//! The engine holds no general HTTP client, so `Calendar/subscribe` /
//! `Calendar/refreshSubscription` take the fetched `.ics` as a `blob`. This driver
//! performs that GET **through e6's SSRF-hardened fetcher**
//! ([`crate::image_proxy::fetch_url_hardened_routed`], which adds the configured
//! egress route and its fail-closed behaviour) — a `webcal://` URL is
//! attacker-influenceable exactly like a remote-image URL, so it must not get a
//! second, weaker fetcher — then forwards the body to the engine.
//!
//! All routes are session-authed and ride the normal CSRF/same-origin guard.

use axum::body::Bytes;
use axum::extract::rejection::BytesRejection;
use axum::extract::{DefaultBodyLimit, Query, State};
use axum::http::{HeaderMap, StatusCode};
use axum::response::{IntoResponse, Response};
use axum::routing::{get, post};
use axum::{Json, Router};
use serde::Deserialize;
use serde_json::{Value, json};

use crate::{AppState, authed, payload_too_large};

/// Hard cap on messages accepted in one import call — bounds work + upload writes
/// for a single request (a larger archive is split client-side).
const MAX_IMPORT_MESSAGES: usize = 20_000;
/// Hard cap on one message's size (also the per-file cap for a maildir tar entry)
/// **and, since t20 (26.19, B7), the whole-request body limit of every import
/// route** — see [`import_router`]. An mbox/maildir archive therefore travels in
/// slices of at most this size; the same "split the archive" advice that
/// [`MAX_IMPORT_MESSAGES`] gives applies to bytes as well as message count.
const MAX_MESSAGE_BYTES: usize = 64 * 1024 * 1024;
/// The `limit` name reported in the JSON `413` — the constant that was hit, as
/// `/jmap/upload` reports `maxSizeUpload` for its own cap.
const MESSAGE_LIMIT_NAME: &str = "maxMessageBytes";

/// The mailbox-import + webcal sync-driver routes (mounted by `lib.rs`/e10).
///
/// The three body-carrying import routes each carry an explicit
/// [`DefaultBodyLimit`] of [`MAX_MESSAGE_BYTES`]. This is deliberately per-route
/// rather than router-wide: the JSON routes below (webcal subscribe/refresh)
/// carry a URL, not an archive, and keep axum's small default. The layer must be
/// declared here — a body limit is applied by the extractor, so a handler can
/// never raise one it was not given.
pub(crate) fn import_router() -> Router<AppState> {
    Router::new()
        .route(
            "/api/import/mbox",
            post(import_mbox).layer(DefaultBodyLimit::max(MAX_MESSAGE_BYTES)),
        )
        .route(
            "/api/import/eml",
            post(import_eml).layer(DefaultBodyLimit::max(MAX_MESSAGE_BYTES)),
        )
        .route(
            "/api/import/maildir",
            post(import_maildir).layer(DefaultBodyLimit::max(MAX_MESSAGE_BYTES)),
        )
        .route("/api/import/formats", get(import_formats))
        .route("/api/calendar/subscribe", post(calendar_subscribe))
        .route("/api/calendar/refresh", post(calendar_refresh))
}

// ── import (W18) ────────────────────────────────────────────────────────────────

/// Optional target-mailbox selector; absent → the account's inbox.
#[derive(Debug, Default, Deserialize)]
struct ImportQuery {
    #[serde(default)]
    mailbox_id: Option<String>,
}

/// `GET /api/import/formats` — the migration-wizard descriptor: which formats this
/// server accepts and how each body is expected. Session-authed so the wizard is
/// only reachable to a logged-in user, and so it doubles as an engine-mode probe.
async fn import_formats(State(state): State<AppState>, headers: HeaderMap) -> Response {
    if let Err(r) = authed(&state, &headers).await {
        return r;
    }
    let engine_ready = state.engine.is_some();
    Json(json!({
        "engineMode": engine_ready,
        "note": if engine_ready {
            "Import requires a configured upload backend (MW_UPLOAD_DIR)."
        } else {
            "Import is unavailable: the server is in proxy mode."
        },
        "formats": [
            { "id": "mbox", "endpoint": "/api/import/mbox",
              "contentType": "application/mbox", "body": "one From_-delimited mbox stream" },
            { "id": "eml", "endpoint": "/api/import/eml",
              "contentType": "message/rfc822", "body": "a single RFC822 message" },
            { "id": "maildir", "endpoint": "/api/import/maildir",
              "contentType": "application/x-tar",
              "body": "an uncompressed tar of the maildir (cur/ and new/ files are imported)" }
        ],
        // The real, enforced limits — so the wizard can slice an archive to fit
        // instead of discovering the cap as a 413 (t20 B7).
        "maxBytes": MAX_MESSAGE_BYTES,
        "maxMessages": MAX_IMPORT_MESSAGES,
        "targetMailbox": "pass ?mailboxId=<id>; omitted → the account inbox"
    }))
    .into_response()
}

/// `POST /api/import/mbox` — split an mbox stream and import each message.
async fn import_mbox(
    State(state): State<AppState>,
    headers: HeaderMap,
    Query(q): Query<ImportQuery>,
    body: Result<Bytes, BytesRejection>,
) -> Response {
    let body = match body {
        Ok(b) => b,
        Err(rej) => return body_rejected(rej),
    };
    let messages = mw_export::split_mbox(&body);
    ingest_messages(&state, &headers, q.mailbox_id.as_deref(), messages).await
}

/// `POST /api/import/eml` — import one RFC822 message (the whole body).
async fn import_eml(
    State(state): State<AppState>,
    headers: HeaderMap,
    Query(q): Query<ImportQuery>,
    body: Result<Bytes, BytesRejection>,
) -> Response {
    let body = match body {
        Ok(b) => b,
        Err(rej) => return body_rejected(rej),
    };
    if body.is_empty() {
        return bad_request("empty message body");
    }
    ingest_messages(
        &state,
        &headers,
        q.mailbox_id.as_deref(),
        vec![body.to_vec()],
    )
    .await
}

/// `POST /api/import/maildir` — import the message files of a maildir, uploaded as
/// one uncompressed tar. Only regular files under `cur/`/`new/` are imported (the
/// `tmp/` staging dir and non-file entries are skipped).
async fn import_maildir(
    State(state): State<AppState>,
    headers: HeaderMap,
    Query(q): Query<ImportQuery>,
    body: Result<Bytes, BytesRejection>,
) -> Response {
    let body = match body {
        Ok(b) => b,
        Err(rej) => return body_rejected(rej),
    };
    let messages = match maildir_tar_messages(&body) {
        Ok(m) => m,
        Err(e) => return bad_request(&e),
    };
    if messages.is_empty() {
        return bad_request(
            "no maildir messages found in the tar (expected files under cur/ or new/)",
        );
    }
    ingest_messages(&state, &headers, q.mailbox_id.as_deref(), messages).await
}

/// The shared import core: authenticate, resolve the target mailbox, seal each
/// message to the upload backend, then ingest them all through one `Email/import`.
async fn ingest_messages(
    state: &AppState,
    headers: &HeaderMap,
    mailbox_id: Option<&str>,
    messages: Vec<Vec<u8>>,
) -> Response {
    let session = match authed(state, headers).await {
        Ok(s) => s,
        Err(resp) => return resp,
    };
    let Some(engine) = &state.engine else {
        return (
            StatusCode::NOT_IMPLEMENTED,
            Json(json!({ "error": "import requires engine mode" })),
        )
            .into_response();
    };
    if messages.is_empty() {
        return bad_request("no messages to import");
    }
    if messages.len() > MAX_IMPORT_MESSAGES {
        return bad_request("too many messages in one import (split the archive)");
    }
    // Defence in depth: the route's body limit already caps the whole request at
    // `MAX_MESSAGE_BYTES`, so a single message cannot exceed it — but the check
    // stays, and answers with the same `413` shape as the body limit rather than
    // a `400`, so a size refusal looks identical wherever it is raised.
    if messages.iter().any(|m| m.len() > MAX_MESSAGE_BYTES) {
        return import_too_large();
    }
    let account = &session.account_id;

    // Resolve the destination mailbox id (explicit, or the account inbox).
    let mailbox_id = match resolve_target_mailbox(engine, account, mailbox_id).await {
        Ok(Some(id)) => id,
        Ok(None) => return bad_request("no target mailbox (and no inbox found)"),
        Err(e) => return upstream(&e),
    };

    // Seal each message to the upload backend, collecting its blobId. A store_upload
    // failure (e.g. no MW_UPLOAD_DIR) aborts loudly rather than dropping bytes.
    let mut emails = serde_json::Map::new();
    for (i, msg) in messages.iter().enumerate() {
        let blob_id = match engine.store_upload(account, "message/rfc822", msg).await {
            Ok(id) => id,
            Err(e) => {
                return upstream(&format!(
                    "upload backend rejected message {i} (is MW_UPLOAD_DIR configured?): {e}"
                ));
            }
        };
        emails.insert(
            format!("i{i}"),
            json!({ "blobId": blob_id, "mailboxIds": { &mailbox_id: true } }),
        );
    }

    // One Email/import for the whole batch — the same ingest path a client uses.
    let request = json!({
        "methodCalls": [[
            "Email/import",
            { "accountId": account, "emails": Value::Object(emails) },
            "imp0"
        ]]
    });
    let resp = engine.handle_jmap(account, &request).await;
    let (imported, failed) = import_counts(&resp);
    Json(json!({
        "imported": imported,
        "failed": failed,
        "total": messages.len(),
        "mailboxId": mailbox_id,
    }))
    .into_response()
}

/// Count `created` vs `notCreated` in an `Email/import` response.
fn import_counts(resp: &Value) -> (usize, usize) {
    let call = resp
        .get("methodResponses")
        .and_then(Value::as_array)
        .and_then(|r| r.first())
        .and_then(|c| c.as_array())
        .and_then(|c| c.get(1));
    let created = call
        .and_then(|a| a.get("created"))
        .and_then(Value::as_object)
        .map(|m| m.len())
        .unwrap_or(0);
    let failed = call
        .and_then(|a| a.get("notCreated"))
        .and_then(Value::as_object)
        .map(|m| m.len())
        .unwrap_or(0);
    (created, failed)
}

/// Resolve the destination mailbox id: an explicit id is used verbatim; otherwise
/// pick the account's `inbox`-role mailbox (falling back to a mailbox literally
/// named "Inbox") via `Mailbox/get`.
async fn resolve_target_mailbox(
    engine: &mw_engine::Engine,
    account: &str,
    explicit: Option<&str>,
) -> Result<Option<String>, String> {
    if let Some(id) = explicit.filter(|s| !s.is_empty()) {
        return Ok(Some(id.to_string()));
    }
    let request = json!({
        "methodCalls": [["Mailbox/get", { "accountId": account, "ids": null }, "mb0"]]
    });
    let resp = engine.handle_jmap(account, &request).await;
    let list = resp
        .get("methodResponses")
        .and_then(Value::as_array)
        .and_then(|r| r.first())
        .and_then(|c| c.as_array())
        .and_then(|c| c.get(1))
        .and_then(|a| a.get("list"))
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default();
    let by_role = list.iter().find(|m| {
        m.get("role")
            .and_then(Value::as_str)
            .is_some_and(|r| r.eq_ignore_ascii_case("inbox"))
    });
    let chosen = by_role.or_else(|| {
        list.iter().find(|m| {
            m.get("name")
                .and_then(Value::as_str)
                .is_some_and(|n| n.eq_ignore_ascii_case("inbox"))
        })
    });
    Ok(chosen
        .and_then(|m| m.get("id").and_then(Value::as_str))
        .map(str::to_string))
}

// ── uncompressed tar (ustar) reader for maildir import ──────────────────────────

/// Extract the regular-file bodies of an uncompressed POSIX tar that hold maildir
/// messages (paths containing a `/cur/` or `/new/` segment; if none match, every
/// non-empty regular file is taken). Pure, allocation-bounded, and panic-free — a
/// truncated or malformed archive yields the entries parsed so far or an error, but
/// never reads out of bounds.
fn maildir_tar_messages(data: &[u8]) -> Result<Vec<Vec<u8>>, String> {
    const BLOCK: usize = 512;
    if data.len() < BLOCK {
        return Err("not a tar archive (too short)".to_string());
    }
    let mut all: Vec<(String, Vec<u8>)> = Vec::new();
    let mut off = 0usize;
    while off + BLOCK <= data.len() {
        let header = &data[off..off + BLOCK];
        // Two consecutive zero blocks mark the end of the archive.
        if header.iter().all(|&b| b == 0) {
            break;
        }
        let name = tar_str(&header[0..100]);
        let size = tar_octal(&header[124..136]).ok_or("malformed tar size field")?;
        if size > MAX_MESSAGE_BYTES as u64 {
            return Err("a tar entry exceeds the per-message size limit".to_string());
        }
        let size = size as usize;
        let typeflag = header[156];
        off += BLOCK;
        // Body starts after the header, padded up to the next 512 boundary.
        let end = off.checked_add(size).ok_or("tar size overflow")?;
        if end > data.len() {
            return Err("truncated tar entry body".to_string());
        }
        // typeflag '0' or NUL is a regular file; skip everything else (dirs, links…).
        if (typeflag == b'0' || typeflag == 0) && size > 0 && !name.ends_with('/') {
            all.push((name, data[off..end].to_vec()));
        }
        off = end.div_ceil(BLOCK) * BLOCK;
        if all.len() > MAX_IMPORT_MESSAGES {
            return Err("too many entries in the maildir tar".to_string());
        }
    }
    // Prefer real maildir message paths; fall back to every regular file if the tar
    // wasn't rooted at the maildir (so a bare `cur/`-less dump still imports).
    let maildir: Vec<Vec<u8>> = all
        .iter()
        .filter(|(name, _)| name.contains("/cur/") || name.contains("/new/"))
        .map(|(_, body)| body.clone())
        .collect();
    if maildir.is_empty() {
        Ok(all.into_iter().map(|(_, body)| body).collect())
    } else {
        Ok(maildir)
    }
}

/// Read a NUL-terminated tar header string field as UTF-8 (lossy).
fn tar_str(field: &[u8]) -> String {
    let end = field.iter().position(|&b| b == 0).unwrap_or(field.len());
    String::from_utf8_lossy(&field[..end]).into_owned()
}

/// Parse a tar octal numeric field (ASCII octal, space/NUL padded).
fn tar_octal(field: &[u8]) -> Option<u64> {
    let s: String = field
        .iter()
        .take_while(|&&b| b != 0 && b != b' ')
        .skip_while(|&&b| b == b' ')
        .map(|&b| b as char)
        .collect();
    if s.is_empty() {
        return Some(0);
    }
    u64::from_str_radix(s.trim(), 8).ok()
}

// ── webcal sync driver (P6) ─────────────────────────────────────────────────────

/// `Accept` sent for a `.ics` fetch — prefer calendar, tolerate text.
const ICS_ACCEPT: &str = "text/calendar, text/plain;q=0.8, */*;q=0.1";

#[derive(Debug, Deserialize)]
struct SubscribeReq {
    url: String,
    #[serde(default)]
    name: Option<String>,
}

#[derive(Debug, Deserialize)]
struct RefreshReq {
    #[serde(rename = "calendarId")]
    calendar_id: String,
    url: String,
}

/// `POST /api/calendar/subscribe {url, name?}` — fetch the feed through the
/// SSRF-hardened fetcher and register the subscription (importing its contents).
async fn calendar_subscribe(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(req): Json<SubscribeReq>,
) -> Response {
    let (session, engine) = match auth_engine(&state, &headers).await {
        Ok(v) => v,
        Err(resp) => return resp,
    };
    let ics = match fetch_ics(&state, &req.url).await {
        Ok(b) => b,
        Err(e) => return upstream(&e),
    };
    let mut args = json!({ "url": req.url, "blob": ics });
    if let Some(name) = req.name.filter(|s| !s.is_empty()) {
        args["name"] = Value::String(name);
    }
    forward_pim(engine, &session.account_id, "Calendar/subscribe", args).await
}

/// `POST /api/calendar/refresh {calendarId, url}` — re-fetch the feed and re-import
/// the overlay's events.
async fn calendar_refresh(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(req): Json<RefreshReq>,
) -> Response {
    let (session, engine) = match auth_engine(&state, &headers).await {
        Ok(v) => v,
        Err(resp) => return resp,
    };
    let ics = match fetch_ics(&state, &req.url).await {
        Ok(b) => b,
        Err(e) => return upstream(&e),
    };
    let args = json!({ "calendarId": req.calendar_id, "blob": ics });
    forward_pim(
        engine,
        &session.account_id,
        "Calendar/refreshSubscription",
        args,
    )
    .await
}

/// Fetch a `.ics` feed through e6's SSRF-hardened fetcher, **over the deployment's
/// configured egress route** (26.20 t22-e14). `webcal://`/`webcals://` are normalized
/// to `https://` first (reqwest only speaks http/https); the fetched bytes are
/// returned as a UTF-8 (lossy) string for the engine's parser.
///
/// # Why this takes `state`
/// The route is deployment-wide operator configuration read from the store, so this
/// needs a store handle — and **nothing else**. `state` supplies the store; no part
/// of the request reaches route selection, which is the invariant the egress design
/// rests on (see `image_proxy::active_route`). The subscription URL is passed to the
/// fetcher as a fetch target, exactly as before, and never as a route selector.
///
/// # Why this surface had to be wired, not just the image proxy
/// A subscription URL is attacker-influenceable in the same way a remote-image URL
/// is — which is why this module already refused to hand-roll a second, weaker
/// fetcher. Honouring the egress route for images and not here would give an
/// operator a control that is silently bypassed for calendar subscriptions, which is
/// worse than not shipping it: they would believe their egress is controlled. The
/// previous author's care in sharing the fetcher would have been quietly undone by
/// wiring only the sibling.
///
/// Fail-closed comes with it: if a route is configured and its tunnel cannot be
/// established, this **fails** rather than fetching directly. Every error string is
/// `fetch_url_hardened`'s unchanged, so the `502` bodies below are what they were.
async fn fetch_ics(state: &AppState, url: &str) -> Result<String, String> {
    let fetch_url = normalize_webcal(url);
    let bytes =
        crate::image_proxy::fetch_url_hardened_routed(state, &fetch_url, ICS_ACCEPT).await?;
    Ok(String::from_utf8_lossy(&bytes).into_owned())
}

/// `webcal://` / `webcals://` → `https://` (both are HTTPS iCalendar feeds); an
/// `http(s)` URL is left as-is (the SSRF gate re-checks the scheme).
fn normalize_webcal(url: &str) -> String {
    if let Some(rest) = url.strip_prefix("webcals://") {
        format!("https://{rest}")
    } else if let Some(rest) = url.strip_prefix("webcal://") {
        format!("https://{rest}")
    } else {
        url.to_string()
    }
}

/// Forward a single PIM method to the engine and surface its result / error.
async fn forward_pim(
    engine: &mw_engine::Engine,
    account: &str,
    method: &str,
    args: Value,
) -> Response {
    let request = json!({ "methodCalls": [[method, args, "cal0"]] });
    let resp = engine.handle_jmap(account, &request).await;
    let call = resp
        .get("methodResponses")
        .and_then(Value::as_array)
        .and_then(|r| r.first())
        .and_then(|c| c.as_array())
        .and_then(|c| c.get(1))
        .cloned()
        .unwrap_or_else(|| json!({}));
    // A method-level error object carries a `type`; surface it as a 502.
    if let Some(err) = call.get("type").and_then(Value::as_str) {
        return upstream(&format!("{method} failed: {err}"));
    }
    Json(call).into_response()
}

// ── small shared helpers ────────────────────────────────────────────────────────

/// Authenticate and require engine mode, returning both.
async fn auth_engine<'a>(
    state: &'a AppState,
    headers: &HeaderMap,
) -> Result<(mw_store::Session, &'a mw_engine::Engine), Response> {
    let session = authed(state, headers).await?;
    let Some(engine) = &state.engine else {
        return Err((
            StatusCode::NOT_IMPLEMENTED,
            Json(json!({ "error": "requires engine mode" })),
        )
            .into_response());
    };
    Ok((session, engine))
}

/// Turn a failed body read on a size-capped import route into a response.
///
/// The `Bytes` extractor is fallible once an explicit [`DefaultBodyLimit`] is in
/// play, and its own rejection is a bare `text/plain` `413`. Taking the
/// `Result` in the handler lets the limit answer with the crate-shared JSON body
/// instead, so a client sees the same `{error, limit, maxBytes}` shape here as on
/// `/jmap/upload` and the blob-download path. Any other read failure (a client
/// that hung up mid-body) is a `400` — the request never arrived, it was not
/// too big.
fn body_rejected(rej: BytesRejection) -> Response {
    if rej.status() == StatusCode::PAYLOAD_TOO_LARGE {
        return import_too_large();
    }
    bad_request(&format!(
        "could not read the request body: {}",
        rej.body_text()
    ))
}

/// The JSON `413` for an import body over [`MAX_MESSAGE_BYTES`].
fn import_too_large() -> Response {
    payload_too_large(MESSAGE_LIMIT_NAME, MAX_MESSAGE_BYTES)
}

fn bad_request(msg: &str) -> Response {
    (StatusCode::BAD_REQUEST, Json(json!({ "error": msg }))).into_response()
}

fn upstream(msg: &str) -> Response {
    tracing::warn!("import/webcal upstream: {msg}");
    (StatusCode::BAD_GATEWAY, Json(json!({ "error": msg }))).into_response()
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Build a minimal ustar archive of the given `(name, body)` files.
    fn make_tar(files: &[(&str, &[u8])]) -> Vec<u8> {
        let mut out = Vec::new();
        for (name, body) in files {
            let mut header = [0u8; 512];
            let nb = name.as_bytes();
            header[..nb.len()].copy_from_slice(nb);
            // size octal at 124..136 (11 digits + NUL).
            let size = format!("{:011o}\0", body.len());
            header[124..124 + size.len()].copy_from_slice(size.as_bytes());
            header[156] = b'0'; // regular file
            out.extend_from_slice(&header);
            out.extend_from_slice(body);
            let pad = (512 - (body.len() % 512)) % 512;
            out.extend(std::iter::repeat_n(0u8, pad));
        }
        out.extend([0u8; 1024]); // two zero blocks terminate the archive
        out
    }

    #[test]
    fn maildir_tar_extracts_cur_and_new() {
        let tar = make_tar(&[
            ("maildir/cur/1.msg", b"From: a@x\r\n\r\nhi cur"),
            ("maildir/new/2.msg", b"From: b@x\r\n\r\nhi new"),
            ("maildir/tmp/3.msg", b"From: c@x\r\n\r\nhi tmp"),
            ("maildir/", b""),
        ]);
        let msgs = maildir_tar_messages(&tar).expect("parse tar");
        assert_eq!(msgs.len(), 2, "only cur/ and new/ files import");
        assert!(msgs.iter().any(|m| m.ends_with(b"hi cur")));
        assert!(msgs.iter().any(|m| m.ends_with(b"hi new")));
    }

    #[test]
    fn maildir_tar_falls_back_to_all_regular_files() {
        let tar = make_tar(&[("dump/msg1", b"one"), ("dump/msg2", b"two")]);
        let msgs = maildir_tar_messages(&tar).expect("parse tar");
        assert_eq!(msgs.len(), 2);
    }

    #[test]
    fn truncated_tar_is_rejected_not_panicked() {
        let body = vec![b'x'; 1000];
        let mut tar = make_tar(&[("maildir/cur/1", &body)]);
        tar.truncate(600); // header (512) present but the 1000-byte body is cut off
        assert!(maildir_tar_messages(&tar).is_err());
    }

    #[test]
    fn tar_octal_parses_and_defaults() {
        assert_eq!(tar_octal(b"00000000012\0"), Some(10));
        assert_eq!(tar_octal(b"          \0"), Some(0));
        assert_eq!(tar_octal(b"\0\0\0\0"), Some(0));
    }

    #[test]
    fn webcal_normalizes_to_https() {
        assert_eq!(normalize_webcal("webcal://h/c.ics"), "https://h/c.ics");
        assert_eq!(normalize_webcal("webcals://h/c.ics"), "https://h/c.ics");
        assert_eq!(normalize_webcal("https://h/c.ics"), "https://h/c.ics");
    }

    // ── request-size limits (t20 B7) ────────────────────────────────────────────
    //
    // Driven through a REAL `build_app` router over a real socket, because the
    // thing under test is a router *layer*: a limit asserted on the constant
    // alone would still pass with the layer deleted, which is exactly the bug
    // this closes. Proxy mode is enough — the size verdict is reached in the
    // extractor, before the handler needs an engine.

    /// axum's implicit `DefaultBodyLimit`. Not imported from axum (it is not
    /// public API); pinned here as the number that made the module's own
    /// constants unreachable.
    const AXUM_DEFAULT_BODY_LIMIT: usize = 2 * 1024 * 1024;

    const IMPORT_PATHS: [&str; 3] = ["/api/import/mbox", "/api/import/eml", "/api/import/maildir"];

    /// A temp dir unique per call AND per process — a pid-only or name-only path
    /// collides between concurrently running test binaries sharing `target/`.
    fn unique_dir(tag: &str) -> std::path::PathBuf {
        use std::sync::atomic::{AtomicU64, Ordering};
        static SEQ: AtomicU64 = AtomicU64::new(0);
        let nanos = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("clock after epoch")
            .as_nanos();
        let dir = std::env::temp_dir().join(format!(
            "mw-t20e5-{tag}-{}-{nanos}-{}",
            std::process::id(),
            SEQ.fetch_add(1, Ordering::Relaxed)
        ));
        std::fs::create_dir_all(&dir).expect("create temp dir");
        dir
    }

    /// Boot the real app on an ephemeral port and return its base URL.
    async fn spawn_app(tag: &str) -> String {
        let dir = unique_dir(tag);
        let config = crate::AppConfig {
            db_path: dir.join("mw.db").to_string_lossy().into_owned(),
            server_key_hex: None,
            web_dir: None,
            cookie_secure: false,
            mode: crate::ServerMode::Proxy,
            hardening: crate::HardeningConfig::default(),
            security: crate::SecurityConfig::default(),
        };
        let app = crate::build_app(config).await.expect("build_app");
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
            .await
            .expect("bind ephemeral port");
        let addr = listener.local_addr().expect("local addr");
        tokio::spawn(async move {
            let _ = axum::serve(listener, app).await;
        });
        format!("http://{addr}")
    }

    async fn post_bytes(base: &str, path: &str, body: Vec<u8>) -> reqwest::Response {
        reqwest::Client::builder()
            .no_proxy()
            .build()
            .expect("reqwest client builds")
            .post(format!("{base}{path}"))
            .header("content-type", "application/octet-stream")
            .body(body)
            .send()
            .await
            .expect("request completed")
    }

    /// A body over axum's implicit 2 MB default but under the module's own cap is
    /// accepted for reading on every import route — it reaches the handler and
    /// fails on *authentication*, not on size. Before the explicit per-route
    /// limit this was refused outright, no matter how the fronting proxy was
    /// configured — and refused mid-upload, so the client saw an aborted
    /// connection rather than a status it could report.
    #[tokio::test]
    async fn import_reads_a_body_over_the_implicit_axum_default() {
        let base = spawn_app("over-default").await;
        let payload = vec![b'x'; 3 * 1024 * 1024];
        assert!(payload.len() > AXUM_DEFAULT_BODY_LIMIT && payload.len() < MAX_MESSAGE_BYTES);
        // A well-formed tar so the maildir route also gets as far as the auth
        // check rather than stopping at "malformed archive".
        let tar = make_tar(&[("maildir/cur/1.msg", &payload)]);

        for path in IMPORT_PATHS {
            let body = if path.ends_with("maildir") {
                tar.clone()
            } else {
                payload.clone()
            };
            let sent = body.len();
            let resp = post_bytes(&base, path, body).await;
            assert_eq!(
                resp.status(),
                reqwest::StatusCode::UNAUTHORIZED,
                "{path} should have read {sent} bytes and stopped at auth, got {}",
                resp.status()
            );
        }
    }

    /// A 60 MB import — the size the 2 MB default made impossible — still reaches
    /// the handler.
    #[tokio::test]
    async fn import_reads_a_sixty_megabyte_body() {
        let base = spawn_app("sixty-mb").await;
        let resp = post_bytes(&base, "/api/import/eml", vec![b'x'; 60 * 1024 * 1024]).await;
        assert_eq!(resp.status(), reqwest::StatusCode::UNAUTHORIZED);
    }

    /// One byte past the module's own cap is refused on every import route, with
    /// the crate-shared JSON `413` — the same body `/jmap/upload` and the blob
    /// download path answer with, which is what lets a client tell an app refusal
    /// from a fronting proxy's HTML error page.
    #[tokio::test]
    async fn import_refuses_a_body_over_max_message_bytes() {
        let base = spawn_app("too-large").await;
        for path in IMPORT_PATHS {
            let resp = post_bytes(&base, path, vec![b'x'; MAX_MESSAGE_BYTES + 1]).await;
            assert_eq!(
                resp.status(),
                reqwest::StatusCode::PAYLOAD_TOO_LARGE,
                "{path} accepted a body over the limit"
            );
            let body: Value = resp.json().await.expect("413 carries a JSON body");
            assert_eq!(body["error"], "payload too large", "{path}");
            assert_eq!(body["limit"], MESSAGE_LIMIT_NAME, "{path}");
            assert_eq!(body["maxBytes"], MAX_MESSAGE_BYTES, "{path}");
        }
    }

    /// Pin the advertised limit. `/api/import/formats` reports this constant and
    /// the route layer is built from it, so the number a client is told is the
    /// number it will be held to.
    #[test]
    fn message_limit_is_pinned_at_64_mib() {
        assert_eq!(MAX_MESSAGE_BYTES, 64 * 1024 * 1024);
        const { assert!(MAX_MESSAGE_BYTES > AXUM_DEFAULT_BODY_LIMIT) };
    }
}
