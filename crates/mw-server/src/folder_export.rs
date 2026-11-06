//! `GET /api/export/folder/{id}.zip` — download a whole folder as a ZIP.
//!
//! ```text
//! /api/export/folder/<mailboxId>.zip?format=eml|mbox&subfolders=1|0[&estimate=1]
//! ```
//!
//! The archive is built as it is sent: one `.eml` entry per message (or one
//! `.mbox` entry per folder), a directory per subfolder when subfolders are
//! included. Entries are stored, not compressed, so the response is
//! byte-for-byte the messages plus ZIP framing.
//!
//! This is the user's own mail, so the attachment policy does not apply. What
//! does: the `export` feature gate, `export.folder.enabled`, the formats of
//! `export.folder.format`, and the caps `export.folder.maxBytes` and
//! `export.folder.maxMessages`, all checked against the cached folder before
//! the first byte and again while streaming. A folder over a cap is refused with
//! `413` and the sizes, never truncated. Each completed or refused export writes
//! an audit row, and the whole export holds one `jmapExport` slot of the account
//! until its last byte is sent.
//!
//! The classic ZIP format caps an archive at 65,535 entries, so
//! `export.folder.maxMessages` is never applied above 65,000.
//!
//! The download itself needs the `dataExport` step-up, like exporting one message.
//! `estimate=1` answers the sizes and the verdict as JSON and sends no mail.
//! Engine mode only; a proxied account has no local store to read.

use axum::{
    Json,
    body::Body,
    extract::{Path as UrlPath, Query, State},
    http::{HeaderMap, HeaderValue, StatusCode, header},
    response::{IntoResponse, Response},
};
use serde::Deserialize;
use serde_json::{Value, json};

use crate::{AppState, dsr_data::ZipWriter, jmap_limits, rate_leeway::Held, settings_registry};

/// The most entries a classic ZIP holds, with room for the folder entries.
const ZIP_ENTRY_CEILING: u64 = 65_000;
/// Messages read from the store per page.
const PAGE: i64 = 100;

#[derive(Debug, Deserialize)]
pub(crate) struct ExportQuery {
    format: Option<String>,
    subfolders: Option<String>,
    estimate: Option<String>,
}

/// What the effective settings allow this user.
#[derive(Debug, Clone)]
pub(crate) struct Policy {
    pub enabled: bool,
    pub formats: Vec<&'static str>,
    pub max_bytes: u64,
    pub max_messages: u64,
    pub subfolders_default: bool,
}

impl Policy {
    pub(crate) fn from_values(
        enabled: bool,
        format: &str,
        max_bytes: u64,
        max_messages: u64,
        subfolders_default: bool,
    ) -> Self {
        Self {
            enabled,
            formats: match format {
                "zip-eml" => vec!["eml"],
                "zip-mbox" => vec!["mbox"],
                _ => vec!["eml", "mbox"],
            },
            max_bytes,
            max_messages: max_messages.min(ZIP_ENTRY_CEILING),
            subfolders_default,
        }
    }
}

fn problem(status: StatusCode, code: &str, extra: Value) -> Response {
    let mut body = json!({ "error": code });
    if let (Some(map), Some(more)) = (body.as_object_mut(), extra.as_object()) {
        map.extend(more.clone());
    }
    (status, Json(body)).into_response()
}

async fn policy(state: &AppState, session: &mw_store::Session) -> Result<Policy, Response> {
    let unavailable = || problem(StatusCode::SERVICE_UNAVAILABLE, "exportPolicyUnavailable", json!({}));
    let identity = crate::settings_routes::UserSubject::resolved(&state.store, session)
        .await
        .map_err(|_| unavailable())?;
    let subject = identity.as_subject();
    let get = |key: &'static str| {
        let subject = &subject;
        async move {
            settings_registry::effective_for(&state.store, key, subject)
                .await
                .ok()
                .flatten()
                .map(|e| e.value)
        }
    };
    let enabled = get("export.folder.enabled").await.and_then(|v| v.as_bool());
    let format = get("export.folder.format").await;
    let max_bytes = get("export.folder.maxBytes").await.and_then(|v| v.as_u64());
    let max_messages = get("export.folder.maxMessages").await.and_then(|v| v.as_u64());
    let subfolders = get("export.folder.includeSubfolders").await.and_then(|v| v.as_bool());
    match (enabled, format, max_bytes, max_messages, subfolders) {
        (Some(enabled), Some(format), Some(bytes), Some(messages), Some(subfolders)) => {
            Ok(Policy::from_values(
                enabled,
                format.as_str().unwrap_or("both"),
                bytes,
                messages,
                subfolders,
            ))
        }
        _ => Err(unavailable()),
    }
}

/// One folder of the archive.
#[derive(Debug, Clone)]
struct Part {
    mailbox_id: String,
    /// Directory inside the archive, `""` for the exported folder itself.
    dir: String,
}

/// A path segment that cannot climb out of the archive or carry a control character.
fn safe_segment(segment: &str) -> String {
    let cleaned: String = segment
        .chars()
        .map(|c| if c.is_control() || matches!(c, '\\' | '/' | ':' | '*' | '?' | '"' | '<' | '>' | '|') { '_' } else { c })
        .collect();
    let trimmed = cleaned.trim().trim_matches('.');
    if trimmed.is_empty() { "_".to_string() } else { trimmed.chars().take(120).collect() }
}

/// The exported folder and, when asked, every folder below it.
fn parts_of(all: &[mw_store::Mailbox], target: &mw_store::Mailbox, subfolders: bool) -> Vec<Part> {
    let mut parts = vec![Part { mailbox_id: target.id.clone(), dir: String::new() }];
    if !subfolders {
        return parts;
    }
    let mut below: Vec<(&mw_store::Mailbox, String)> = all
        .iter()
        .filter(|m| m.id != target.id)
        .filter_map(|m| {
            let rest = m.name.strip_prefix(target.name.as_str())?;
            let mut chars = rest.chars();
            let sep = chars.next()?;
            if !matches!(sep, '/' | '.' | '\\') {
                return None;
            }
            let path = chars.as_str();
            (!path.is_empty()).then(|| {
                let dir = path
                    .split(['/', '.', '\\'])
                    .filter(|s| !s.is_empty())
                    .map(safe_segment)
                    .collect::<Vec<_>>()
                    .join("/");
                (m, dir)
            })
        })
        .collect();
    below.sort_by(|a, b| a.1.cmp(&b.1));
    parts.extend(below.into_iter().map(|(m, dir)| Part { mailbox_id: m.id.clone(), dir }));
    parts
}

/// Totals over the cached folders of an export.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub(crate) struct Totals {
    pub folders: u64,
    pub messages: u64,
    pub bytes: u64,
}

/// `Some(code)` when the totals are over a cap.
pub(crate) fn over_cap(totals: Totals, policy: &Policy) -> Option<&'static str> {
    if totals.messages > policy.max_messages {
        Some("exportTooManyMessages")
    } else if totals.bytes > policy.max_bytes {
        Some("exportTooLarge")
    } else {
        None
    }
}

async fn audit(state: &AppState, session: &mw_store::Session, mailbox_id: &str, outcome: &str, detail: Value) {
    let mut detail = detail;
    detail["outcome"] = json!(outcome);
    let row = mw_store::AuditRow {
        id: uuid::Uuid::new_v4().to_string(),
        ts: chrono::Utc::now().to_rfc3339(),
        actor: session.username.clone(),
        actor_kind: "user".to_string(),
        action: "user-folder-export".to_string(),
        target: Some(mailbox_id.to_string()),
        detail_json: detail.to_string(),
        ip: None,
    };
    if let Err(e) = state.store.append_audit(&row).await {
        tracing::warn!("folder export audit append failed: {e}");
    }
}

pub(crate) async fn export_folder(
    State(state): State<AppState>,
    headers: HeaderMap,
    UrlPath(file): UrlPath<String>,
    Query(query): Query<ExportQuery>,
) -> Response {
    let session = match crate::user_router::feature_session(&state, &headers, "export").await {
        Ok(s) => s,
        Err(resp) => return resp,
    };
    let Some(mailbox_id) = file.strip_suffix(".zip").filter(|id| !id.is_empty()) else {
        return problem(StatusCode::NOT_FOUND, "folderNotFound", json!({}));
    };
    let Some(engine) = state.engine.clone() else {
        return problem(StatusCode::NOT_IMPLEMENTED, "exportRequiresEngine", json!({}));
    };
    let policy = match policy(&state, &session).await {
        Ok(p) => p,
        Err(r) => return r,
    };
    if !policy.enabled {
        audit(&state, &session, mailbox_id, "refused-disabled", json!({})).await;
        return problem(StatusCode::FORBIDDEN, "exportDisabled", json!({}));
    }
    let format = query.format.as_deref().unwrap_or(policy.formats[0]);
    let Some(format) = policy.formats.iter().copied().find(|f| *f == format) else {
        return problem(
            StatusCode::BAD_REQUEST,
            "exportFormatNotAllowed",
            json!({ "allowed": policy.formats }),
        );
    };
    let subfolders = match query.subfolders.as_deref() {
        Some("1" | "true") => true,
        Some("0" | "false") => false,
        _ => policy.subfolders_default,
    };
    let all = match state.store.list_mailboxes(&session.account_id).await {
        Ok(all) => all,
        Err(e) => {
            tracing::warn!("folder export: listing folders failed: {e}");
            return problem(StatusCode::INTERNAL_SERVER_ERROR, "exportFailed", json!({}));
        }
    };
    let Some(target) = all.iter().find(|m| m.id == mailbox_id) else {
        return problem(StatusCode::NOT_FOUND, "folderNotFound", json!({}));
    };
    let parts = parts_of(&all, target, subfolders);
    let mut totals = Totals { folders: parts.len() as u64, ..Totals::default() };
    for part in &parts {
        match state.store.folder_stats(&part.mailbox_id).await {
            Ok(stats) => {
                totals.messages += stats.cached;
                totals.bytes += stats.size_bytes;
            }
            Err(e) => {
                tracing::warn!("folder export: sizing failed: {e}");
                return problem(StatusCode::INTERNAL_SERVER_ERROR, "exportFailed", json!({}));
            }
        }
    }
    let verdict = over_cap(totals, &policy);
    let sizes = json!({
        "folders": totals.folders,
        "messages": totals.messages,
        "bytes": totals.bytes,
        "maxBytes": policy.max_bytes,
        "maxMessages": policy.max_messages,
        "format": format,
        "formats": policy.formats,
        "subfolders": subfolders,
    });
    if query.estimate.as_deref() == Some("1") {
        let mut body = sizes;
        body["allowed"] = json!(verdict.is_none());
        if let Some(code) = verdict {
            body["reason"] = json!(code);
        }
        return Json(body).into_response();
    }
    if let Some(code) = verdict {
        audit(&state, &session, mailbox_id, "refused-size", sizes.clone()).await;
        return problem(StatusCode::PAYLOAD_TOO_LARGE, code, sizes);
    }
    // Taking a whole folder out is a data export: it needs the same fresh password
    // proof as exporting one message (the web client's request wrapper asks for it).
    if let Err(response) =
        crate::step_up::require_user(&state, &session, crate::step_up::StepUpAction::DataExport).await
    {
        return response;
    }
    let held = match jmap_limits::export(&state, &session.account_id, &session.username).await {
        Ok(h) => h,
        Err(r) => return r,
    };
    audit(&state, &session, mailbox_id, "started", sizes).await;
    let top = safe_segment(
        target
            .name
            .rsplit(['/', '.', '\\'])
            .find(|s| !s.is_empty())
            .unwrap_or("folder"),
    );
    let body = Body::from_stream(stream(StreamState {
        state: state.clone(),
        engine,
        account_id: session.account_id.clone(),
        actor: session.username.clone(),
        mailbox_id: mailbox_id.to_string(),
        top: top.clone(),
        mbox: format == "mbox",
        policy,
        parts: parts.into_iter(),
        current: None,
        zip: ZipWriter::new(chrono::Utc::now().timestamp()),
        bytes: 0,
        messages: 0,
        done: false,
        _held: held,
    }));
    let mut response = Response::new(body);
    let h = response.headers_mut();
    h.insert(header::CONTENT_TYPE, HeaderValue::from_static("application/zip"));
    let name = format!("{top}.zip");
    if let Ok(v) = HeaderValue::from_str(&format!(
        "attachment; filename=\"{}\"; filename*=UTF-8''{}",
        name.replace(|c: char| !c.is_ascii() || c == '"', "_"),
        percent_encode(&name)
    )) {
        h.insert(header::CONTENT_DISPOSITION, v);
    }
    h.insert(header::CACHE_CONTROL, HeaderValue::from_static("no-store"));
    response
}

fn percent_encode(s: &str) -> String {
    s.bytes()
        .map(|b| {
            if b.is_ascii_alphanumeric() || b"-_.~".contains(&b) {
                (b as char).to_string()
            } else {
                format!("%{b:02X}")
            }
        })
        .collect()
}

struct Current {
    part: Part,
    ids: std::vec::IntoIter<String>,
    offset: i64,
    exhausted: bool,
    index: u32,
    mbox: Vec<u8>,
}

struct StreamState {
    state: AppState,
    engine: std::sync::Arc<mw_engine::Engine>,
    account_id: String,
    actor: String,
    mailbox_id: String,
    top: String,
    mbox: bool,
    policy: Policy,
    parts: std::vec::IntoIter<Part>,
    current: Option<Current>,
    zip: ZipWriter,
    bytes: u64,
    messages: u64,
    done: bool,
    _held: Held,
}

type Chunk = Result<axum::body::Bytes, std::io::Error>;

fn stream(st: StreamState) -> impl futures_util::Stream<Item = Chunk> {
    futures_util::stream::unfold(st, |mut st| async move {
        if st.done {
            return None;
        }
        loop {
            let Some(cur) = st.current.as_mut() else {
                let Some(part) = st.parts.next() else {
                    st.done = true;
                    let tail = st.zip.finish();
                    finish(&st, "completed").await;
                    return Some((Ok(axum::body::Bytes::from(tail)), st));
                };
                st.current = Some(Current {
                    part,
                    ids: Vec::new().into_iter(),
                    offset: 0,
                    exhausted: false,
                    index: 0,
                    mbox: Vec::new(),
                });
                continue;
            };
            let dir = if cur.part.dir.is_empty() {
                st.top.clone()
            } else {
                format!("{}/{}", st.top, cur.part.dir)
            };
            if let Some(id) = cur.ids.next() {
                let raw = match st.engine.fetch_blob(&st.account_id, &id).await {
                    Ok(Some(blob)) => blob.bytes,
                    // A message that went away since the page was read is skipped.
                    Ok(None) => continue,
                    Err(e) => {
                        st.done = true;
                        finish(&st, "failed").await;
                        return Some((Err(std::io::Error::other(e.to_string())), st));
                    }
                };
                st.bytes += raw.len() as u64;
                st.messages += 1;
                if st.bytes > st.policy.max_bytes || st.messages > st.policy.max_messages {
                    st.done = true;
                    finish(&st, "aborted-cap").await;
                    return Some((Err(std::io::Error::other("export cap reached")), st));
                }
                cur.index += 1;
                if st.mbox {
                    match mw_export::export_one(&mw_export::RawEmail::new(raw), mw_export::Format::Mbox) {
                        Ok(entry) => cur.mbox.extend_from_slice(&entry),
                        Err(_) => continue,
                    }
                    continue;
                }
                let name = format!("{dir}/{:06}.eml", cur.index);
                let chunk = st.zip.entry(&name, &raw);
                return Some((Ok(axum::body::Bytes::from(chunk)), st));
            }
            if !cur.exhausted {
                match st.state.store.list_message_ids(&cur.part.mailbox_id, PAGE, cur.offset).await {
                    Ok(ids) => {
                        cur.exhausted = (ids.len() as i64) < PAGE;
                        cur.offset += ids.len() as i64;
                        cur.ids = ids.into_iter();
                    }
                    Err(e) => {
                        st.done = true;
                        finish(&st, "failed").await;
                        return Some((Err(std::io::Error::other(e.to_string())), st));
                    }
                }
                continue;
            }
            // The folder is done: an mbox archive writes its one entry now.
            let finished = st.current.take().expect("current folder");
            if st.mbox && !finished.mbox.is_empty() {
                let leaf = if finished.part.dir.is_empty() {
                    st.top.clone()
                } else {
                    finished.part.dir.rsplit('/').next().unwrap_or("folder").to_string()
                };
                let name = if finished.part.dir.is_empty() {
                    format!("{leaf}.mbox")
                } else {
                    format!("{}/{}.mbox", finished.part.dir, leaf)
                };
                let chunk = st.zip.entry(&name, &finished.mbox);
                return Some((Ok(axum::body::Bytes::from(chunk)), st));
            }
        }
    })
}

async fn finish(st: &StreamState, outcome: &str) {
    let row = mw_store::AuditRow {
        id: uuid::Uuid::new_v4().to_string(),
        ts: chrono::Utc::now().to_rfc3339(),
        actor: st.actor.clone(),
        actor_kind: "user".to_string(),
        action: "user-folder-export".to_string(),
        target: Some(st.mailbox_id.clone()),
        detail_json: json!({ "outcome": outcome, "messages": st.messages, "bytes": st.bytes }).to_string(),
        ip: None,
    };
    if let Err(e) = st.state.store.append_audit(&row).await {
        tracing::warn!("folder export audit append failed: {e}");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn mailbox(id: &str, name: &str) -> mw_store::Mailbox {
        mw_store::Mailbox {
            id: id.into(),
            account_id: "a".into(),
            name: name.into(),
            role: None,
            uidvalidity: 1,
            uidnext: 1,
            highestmodseq: 0,
            total: 0,
            unread: 0,
            parent_id: None,
        }
    }

    #[test]
    fn subfolders_become_directories_and_unrelated_prefixes_are_left_out() {
        let all = vec![
            mailbox("1", "Work"),
            mailbox("2", "Work/Invoices"),
            mailbox("3", "Work/Invoices/2026"),
            mailbox("4", "Workshop"),
            mailbox("5", "Other"),
        ];
        let parts = parts_of(&all, &all[0], true);
        let dirs: Vec<_> = parts.iter().map(|p| (p.mailbox_id.as_str(), p.dir.as_str())).collect();
        assert_eq!(dirs, vec![("1", ""), ("2", "Invoices"), ("3", "Invoices/2026")]);
        assert_eq!(parts_of(&all, &all[0], false).len(), 1);
    }

    #[test]
    fn path_segments_cannot_climb_or_carry_separators() {
        assert_eq!(safe_segment(".."), "_");
        assert_eq!(safe_segment("a/b\\c"), "a_b_c");
        assert_eq!(safe_segment("  "), "_");
        assert_eq!(safe_segment("Invoices 2026"), "Invoices 2026");
    }

    #[test]
    fn caps_refuse_by_messages_then_bytes_and_never_exceed_the_zip_entry_limit() {
        let policy = Policy::from_values(true, "both", 1000, 1_000_000, true);
        assert_eq!(policy.max_messages, ZIP_ENTRY_CEILING);
        assert_eq!(over_cap(Totals { folders: 1, messages: 10, bytes: 1000 }, &policy), None);
        assert_eq!(over_cap(Totals { folders: 1, messages: 10, bytes: 1001 }, &policy), Some("exportTooLarge"));
        assert_eq!(
            over_cap(Totals { folders: 1, messages: ZIP_ENTRY_CEILING + 1, bytes: 1 }, &policy),
            Some("exportTooManyMessages")
        );
    }

    #[test]
    fn the_format_setting_limits_what_is_offered() {
        assert_eq!(Policy::from_values(true, "zip-eml", 1, 1, true).formats, vec!["eml"]);
        assert_eq!(Policy::from_values(true, "zip-mbox", 1, 1, true).formats, vec!["mbox"]);
        assert_eq!(Policy::from_values(true, "both", 1, 1, true).formats, vec!["eml", "mbox"]);
    }
}
