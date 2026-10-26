//! Attachment access decisions use server-owned MIME metadata, never URL filenames.
use crate::{AppState, settings_registry};
use axum::{Json, http::StatusCode, response::{IntoResponse, Response}};
use serde::Serialize;
use serde_json::json;

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct Policy {
    pub preview_allowed: bool,
    pub download_allowed: bool,
    pub max_bytes: u64,
    pub allowed_extensions: Vec<String>,
    pub blocked_extensions: Vec<String>,
    pub warn_extensions: Vec<String>,
    pub allowed_mime_types: Vec<String>,
    pub blocked_mime_types: Vec<String>,
    pub warn_mime_types: Vec<String>,
}

#[derive(Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct Decision {
    pub download_allowed: bool,
    pub preview_allowed: bool,
    pub warning: bool,
    pub reason: Option<&'static str>,
}

impl Policy {
    pub fn decide(&self, filename: &str, mime: &str, size: u64) -> Decision {
        let name = filename.rsplit(['/', '\\']).next().unwrap_or("").trim_end_matches([' ', '.']).to_ascii_lowercase();
        let mime = mime.split(';').next().unwrap_or("").trim().to_ascii_lowercase();
        // Match compound extensions as well as the final suffix. A misleading
        // double extension must not hide a configured dangerous suffix.
        let extension = |list: &[String]| list.iter().any(|ext| name.ends_with(&format!(".{ext}")));
        let mime_matches = |list: &[String]| list.iter().any(|rule| rule == &mime || rule.strip_suffix("/*").is_some_and(|top| mime.starts_with(&format!("{top}/"))));
        let reason = if !self.download_allowed { Some("downloadsDisabled") }
            else if self.max_bytes > 0 && size > self.max_bytes { Some("attachmentTooLarge") }
            else if extension(&self.blocked_extensions) || mime_matches(&self.blocked_mime_types) { Some("attachmentBlocked") }
            else if (!self.allowed_extensions.is_empty() && !extension(&self.allowed_extensions)) || (!self.allowed_mime_types.is_empty() && !mime_matches(&self.allowed_mime_types)) { Some("attachmentNotAllowed") }
            else { None };
        Decision { download_allowed: reason.is_none(), preview_allowed: reason.is_none() && self.preview_allowed,
            warning: reason.is_none() && (extension(&self.warn_extensions) || mime_matches(&self.warn_mime_types)), reason }
    }

    pub fn restricts_content(&self) -> bool {
        !self.download_allowed || self.max_bytes > 0 || !self.allowed_extensions.is_empty()
            || !self.blocked_extensions.is_empty() || !self.allowed_mime_types.is_empty()
            || !self.blocked_mime_types.is_empty() || !self.warn_extensions.is_empty() || !self.warn_mime_types.is_empty()
    }
}

pub(crate) fn refusal(code: &'static str) -> Response {
    let mut response = (StatusCode::FORBIDDEN, Json(json!({"code":code,"error":"Attachment access is restricted by administrator policy"}))).into_response();
    response.headers_mut().insert(axum::http::header::CACHE_CONTROL, axum::http::HeaderValue::from_static("private, no-store"));
    response
}

/// Validate administrator rules independently from filenames supplied by mail.
pub(crate) fn valid_rule(rule: &str, mime: bool) -> bool {
    if rule.is_empty() || rule.len() > 127 || rule != rule.to_ascii_lowercase() { return false; }
    if mime {
        let Some((top, sub)) = rule.split_once('/') else { return false; };
        let token = |s: &str| !s.is_empty() && s.bytes().all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || matches!(b,b'!'|b'#'|b'$'|b'&'|b'^'|b'_'|b'.'|b'+'|b'-'));
        token(top) && (sub == "*" || token(sub))
    } else {
        rule.len() <= 32 && rule.split('.').all(|part| !part.is_empty() && part.bytes().all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || matches!(b,b'_'|b'-')))
    }
}

#[derive(Clone, Debug)]
pub(crate) struct Metadata { pub name:String, pub mime:String, pub size:u64 }

#[derive(Default, serde::Deserialize)]
#[serde(rename_all="camelCase")]
pub(crate) struct DownloadQuery {
    pub email_id: Option<String>,
    pub preview: Option<bool>,
    pub acknowledge_warning: Option<bool>,
}

pub(crate) fn authorize(policy:&Policy, metadata:&Metadata, query:&DownloadQuery) -> Result<Decision,Response> {
    let decision=policy.decide(&metadata.name,&metadata.mime,metadata.size);
    if let Some(reason)=decision.reason { return Err(refusal(reason)); }
    if query.preview==Some(true) && !decision.preview_allowed { return Err(refusal("attachmentPreviewDisabled")); }
    if decision.warning && query.acknowledge_warning!=Some(true) { return Err(refusal("attachmentWarningRequired")); }
    // Nested messages/TNEF can carry attachments that outer metadata does not
    // enumerate. Do not provide an export bypass under restrictive rules.
    let mime=metadata.mime.split(';').next().unwrap_or("").trim().to_ascii_lowercase();
    if policy.restricts_content() && (matches!(mime.as_str(),"message/rfc822"|"message/global"|"application/ms-tnef"|"application/vnd.ms-tnef") || metadata.name.to_ascii_lowercase().ends_with(".eml")) {
        return Err(refusal("attachmentContainerRestricted"));
    }
    Ok(decision)
}

pub(crate) fn response_headers(response:&mut Response, decision:&Decision) {
    response.headers_mut().insert("x-mailwoman-preview-allowed",axum::http::HeaderValue::from_static(if decision.preview_allowed {"true"} else {"false"}));
    response.headers_mut().insert(axum::http::header::CACHE_CONTROL,axum::http::HeaderValue::from_static("private, no-store"));
}

pub(crate) async fn proxy_metadata(state:&AppState, session:&mw_store::Session, blob:&str, query:&DownloadQuery) -> Result<Vec<Metadata>,Response> {
    let email=query.email_id.as_deref().filter(|id|!id.is_empty() && id.len()<=512).ok_or_else(||refusal("attachmentMetadataRequired"))?;
    let message=crate::image_proxy::remote_content::message_metadata(state,session,email,&["id","blobId","bodyStructure","attachments"]).await
        .map_err(|_|refusal("attachmentMetadataUnavailable"))?.ok_or_else(||refusal("attachmentMetadataUnavailable"))?;
    if message["blobId"].as_str()==Some(blob) { return Err(refusal("attachmentContainerRestricted")); }
    let parts=matching_parts(&message,blob).map_err(|_|refusal("attachmentMetadataUnavailable"))?;
    if parts.is_empty() {return Err(refusal("attachmentMetadataUnavailable"));}
    Ok(parts)
}

pub(crate) struct AuthorizedAttachment {
    pub bytes:Vec<u8>,
    pub content_type:String,
    pub filename:String,
}

/// Export seam for integrations. The caller supplies only an owned message/blob
/// reference and its own byte ceiling, never a supposedly authoritative filename.
pub(crate) async fn read_owned_part(state:&AppState,session:&mw_store::Session,email_id:&str,blob_id:&str,limit:usize,acknowledge_warning:bool)->Result<Option<AuthorizedAttachment>,Response> {
    let policy=effective(state,session).await?;
    if !policy.download_allowed {return Err(refusal("downloadsDisabled"));}
    let query=DownloadQuery{email_id:Some(email_id.into()),preview:Some(false),acknowledge_warning:Some(acknowledge_warning)};
    let metadata=if state.engine.is_some() {
        if blob_id.split_once('.').map(|(message,_)|message)!=Some(email_id) {return Err(refusal("attachmentMetadataUnavailable"));}
        Vec::new()
    } else {proxy_metadata(state,session,blob_id,&query).await?};
    for part in &metadata {authorize(&policy,part,&query)?;}
    let ceiling=if policy.max_bytes>0 {limit.min(policy.max_bytes as usize)} else {limit};
    let Some((bytes,mime,name))=crate::image_proxy::remote_content::read_part(state,session,blob_id,ceiling).await? else {return Ok(None)};
    let content_type=mime.unwrap_or_else(||"application/octet-stream".into());
    let filename=name.or_else(||metadata.first().map(|part|part.name.clone())).ok_or_else(||refusal("attachmentMetadataUnavailable"))?;
    authorize(&policy,&Metadata{name:filename.clone(),mime:content_type.clone(),size:bytes.len() as u64},&query)?;
    for part in &metadata {authorize(&policy,&Metadata{name:part.name.clone(),mime:content_type.clone(),size:bytes.len() as u64},&query)?;}
    Ok(Some(AuthorizedAttachment{bytes,content_type,filename}))
}

/// A blob may occur more than once with different sender-supplied names/types.
/// Evaluate every matching occurrence rather than choosing the easiest name.
pub(crate) fn matching_parts(message: &serde_json::Value, blob: &str) -> Result<Vec<Metadata>, ()> {
    fn visit(value: &serde_json::Value, blob:&str, depth:usize, budget:&mut usize, out:&mut Vec<Metadata>) -> Result<(),()> {
        if depth > 32 || *budget == 0 { return Err(()); }
        *budget -= 1;
        if let Some(parts)=value.as_array() { for part in parts { visit(part,blob,depth+1,budget,out)?; } }
        else if value.is_object() {
            if value["blobId"].as_str()==Some(blob) {
                out.push(Metadata { name:value["name"].as_str().unwrap_or("").into(), mime:value["type"].as_str().ok_or(())?.into(), size:value["size"].as_u64().ok_or(())? });
            }
            if let Some(parts)=value.get("subParts") { visit(parts,blob,depth+1,budget,out)?; }
        }
        Ok(())
    }
    let mut out=Vec::new(); let mut budget=4096;
    for field in ["bodyStructure","attachments"] { if let Some(value)=message.get(field) {visit(value,blob,0,&mut budget,&mut out)?;} }
    Ok(out)
}

pub(crate) async fn effective(state: &AppState, session: &mw_store::Session) -> Result<Policy, Response> {
    let identity = crate::settings_routes::UserSubject::resolved(&state.store, session).await.map_err(|_| refusal("attachmentPolicyUnavailable"))?;
    let subject = identity.as_subject();
    let mut values = serde_json::Map::new();
    for field in ["previewAllowed", "downloadAllowed", "maxBytes", "allowedExtensions", "blockedExtensions", "warnExtensions", "allowedMimeTypes", "blockedMimeTypes", "warnMimeTypes"] {
        let value = settings_registry::effective_for(&state.store, &format!("attachments.{field}"), &subject).await
            .map_err(|_| refusal("attachmentPolicyUnavailable"))?.ok_or_else(|| refusal("attachmentPolicyUnavailable"))?;
        values.insert(field.into(), value.value);
    }
    let boolean = |key: &str| values[key].as_bool().unwrap_or(false);
    let list = |key: &str| values[key].as_array().into_iter().flatten().filter_map(|v| v.as_str().map(str::to_owned)).collect();
    Ok(Policy { preview_allowed: boolean("previewAllowed"), download_allowed: boolean("downloadAllowed"), max_bytes: values["maxBytes"].as_u64().unwrap_or(0),
        allowed_extensions:list("allowedExtensions"), blocked_extensions:list("blockedExtensions"), warn_extensions:list("warnExtensions"),
        allowed_mime_types:list("allowedMimeTypes"), blocked_mime_types:list("blockedMimeTypes"), warn_mime_types:list("warnMimeTypes") })
}

pub(crate) async fn get(axum::extract::State(state): axum::extract::State<AppState>, headers: axum::http::HeaderMap) -> Response {
    let session = match crate::authed(&state, &headers).await { Ok(s)=>s, Err(r)=>return r };
    match effective(&state,&session).await {
        Ok(policy)=> { let mut response=Json(policy).into_response(); response.headers_mut().insert(axum::http::header::CACHE_CONTROL,axum::http::HeaderValue::from_static("private, no-store")); response },
        Err(response)=>response,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn policy() -> Policy { Policy { preview_allowed:true, download_allowed:true, max_bytes:0, allowed_extensions:vec![], blocked_extensions:vec![], warn_extensions:vec![], allowed_mime_types:vec![], blocked_mime_types:vec![], warn_mime_types:vec![] } }
    #[test]
    fn blocks_override_allows_and_warnings_with_normalized_metadata() {
        let mut p=policy(); p.allowed_extensions=vec!["exe".into()]; p.blocked_extensions=vec!["exe".into()]; p.warn_extensions=vec!["exe".into()];
        assert_eq!(p.decide("safe.pdf.EXE. ","application/octet-stream",10).reason,Some("attachmentBlocked"));
        p.blocked_extensions.clear(); p.allowed_mime_types=vec!["image/*".into()];
        assert_eq!(p.decide("app.exe","application/octet-stream",10).reason,Some("attachmentNotAllowed"));
        assert!(p.decide("app.exe"," IMAGE/PNG; charset=binary",10).warning);
    }
    #[test]
    fn byte_limits_and_raw_delivery_take_precedence_over_preview() {
        let mut p=policy(); p.max_bytes=10;
        assert!(p.decide("x","text/plain",10).download_allowed);
        assert!(!p.decide("x","text/plain",11).preview_allowed);
        p.download_allowed=false;
        assert!(!p.decide("x","text/plain",1).preview_allowed);
        p.download_allowed=true; p.preview_allowed=false;
        assert!(p.decide("x","text/plain",1).download_allowed);
        assert!(!p.decide("x","text/plain",1).preview_allowed);
    }
    #[test]
    fn repeated_blob_metadata_cannot_hide_a_dangerous_filename() {
        let message=json!({"attachments":[{"blobId":"b","name":"safe.txt","type":"text/plain","size":1},{"blobId":"b","name":"bad.exe","type":"application/octet-stream","size":1}]});
        let parts=matching_parts(&message,"b").unwrap();
        let mut p=policy(); p.blocked_extensions=vec!["exe".into()];
        assert_eq!(parts.len(),2);
        assert!(parts.iter().any(|part| !p.decide(&part.name,&part.mime,part.size).download_allowed));
        assert!(matching_parts(&message,"foreign").unwrap().is_empty());
        assert!(matching_parts(&json!({"attachments":[{"blobId":"b","name":"safe.txt"}]}),"b").is_err());
    }
    #[test]
    fn rules_are_bounded_and_do_not_accept_ambiguous_globs() {
        assert!(valid_rule("application/vnd.ms-excel",true)); assert!(valid_rule("image/*",true)); assert!(valid_rule("tar.gz",false));
        for rule in ["*/*","text/*html","image/png;foo=x","image/PNG","text/","text/plain\n"] {assert!(!valid_rule(rule,true));}
        for rule in [".exe","*","a..b","exe ","../exe"] {assert!(!valid_rule(rule,false));}
    }
}
