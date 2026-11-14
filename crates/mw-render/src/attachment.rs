//! Typed boundary for attachment previews. Guest HTML is untrusted until the
//! existing mail sanitizer runs here, before serialization back to the parent.
use base64::Engine as _;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum PreviewFormat {
    Tnef,
    Eml,
    Image,
}
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct AttachmentPreview {
    pub format: PreviewFormat,
    pub subject: Option<String>,
    pub from: Option<String>,
    pub to: Option<String>,
    pub date: Option<String>,
    pub text: String,
    pub html: String,
    pub attachments: Vec<PreviewAttachment>,
    pub warnings: Vec<String>,
}
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct PreviewAttachment {
    pub name: String,
    pub mime: String,
    pub size: usize,
    pub data_base64: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub content_id: Option<String>,
    pub kind: AttachmentKind,
}
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum AttachmentKind {
    File,
    Message,
}

pub fn preview(format: PreviewFormat, bytes: &[u8]) -> Result<AttachmentPreview, String> {
    if bytes.len() > crate::MAX_INPUT_BYTES {
        return Err("attachment exceeds size limit".into());
    }
    if format==PreviewFormat::Image{
        let png=crate::media_jail::reencode_image(bytes)?;
        if png.len()>8*1024*1024{return Err("converted image exceeds preview output limit".into());}
        return Ok(AttachmentPreview{format,subject:None,from:None,to:None,date:None,text:String::new(),html:String::new(),attachments:vec![PreviewAttachment{name:"preview.png".into(),mime:"image/png".into(),size:png.len(),data_base64:base64::engine::general_purpose::STANDARD.encode(png),content_id:None,kind:AttachmentKind::File}],warnings:vec![]});
    }
    let raw = crate::media_jail::preview_payload(format, bytes)?;
    let mut result: AttachmentPreview =
        serde_json::from_slice(&raw).map_err(|_| "invalid guest preview output")?;
    if result.format != format
        || result.attachments.len() > 64
        || result.warnings.len() > 16
        || result.text.len() > 1024 * 1024
        || result.html.len() > 1024 * 1024
    {
        return Err("guest preview exceeds limits".into());
    }
    let mut total = 0usize;
    for attachment in &result.attachments {
        total = total
            .checked_add(attachment.size)
            .ok_or("attachment length overflow")?;
        if total > crate::MAX_INPUT_BYTES
            || attachment.name.len() > 1024
            || attachment.mime.len() > 128
        {
            return Err("attachment output exceeds limit".into());
        }
        let decoded = base64::engine::general_purpose::STANDARD
            .decode(&attachment.data_base64)
            .map_err(|_| "invalid guest attachment encoding")?;
        if decoded.len() != attachment.size {
            return Err("guest attachment size mismatch".into());
        }
    }
    result.html =
        mw_sanitize::sanitize_email_html_reader(&result.html, &mw_sanitize::SanitizeOptions::STRIP);
    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn real_guest_eml_is_sanitized_before_return() {
        let bytes=b"From: sender@example.test\r\nSubject: Attached message\r\nContent-Type: text/html\r\n\r\n<script>alert(1)</script><img src=\"https://tracker.invalid/pixel\"><p>Hello</p>";
        let p = preview(PreviewFormat::Eml, bytes).unwrap();
        assert_eq!(p.subject.as_deref(), Some("Attached message"));
        assert!(!p.html.contains("<script"));
        assert!(!p.html.contains("src=\"https://tracker"));
        assert!(p.html.contains("data-mw-body-blocked=\"true\""));
        assert!(p.html.contains("Hello"));
    }
    #[test]
    fn independent_tnef_attachment_bytes_survive_actual_wasm() {
        let p = preview(
            PreviewFormat::Tnef,
            include_bytes!("../../mw-media-wasm/fixtures/tnef/unicode-mapi-attr.tnef"),
        )
        .unwrap();
        assert_eq!(p.attachments.len(), 1);
        assert_eq!(p.attachments[0].name, "example.dat");
        assert_eq!(
            base64::engine::general_purpose::STANDARD
                .decode(&p.attachments[0].data_base64)
                .unwrap(),
            include_bytes!("../../mw-media-wasm/fixtures/tnef/unicode-mapi-attr/example.dat")
        );
    }
    #[test]
    fn compressed_rtf_body_is_plain_text_through_actual_wasm() {
        let p = preview(
            PreviewFormat::Tnef,
            include_bytes!("../../mw-media-wasm/fixtures/tnef/rtf.tnef"),
        )
        .unwrap();
        assert!(p.text.contains("aafg"));
        assert!(!p.text.contains("fonttbl"));
        assert!(p.html.is_empty());
        assert!(p.warnings.iter().any(|w| w.contains("plain text")));
    }

    #[test]
    fn bitmap_preview_runs_through_actual_media_wasm() {
        let mut bmp=vec![0u8;58];bmp[..2].copy_from_slice(b"BM");bmp[2..6].copy_from_slice(&58u32.to_le_bytes());bmp[10..14].copy_from_slice(&54u32.to_le_bytes());bmp[14..18].copy_from_slice(&40u32.to_le_bytes());bmp[18..22].copy_from_slice(&1u32.to_le_bytes());bmp[22..26].copy_from_slice(&1u32.to_le_bytes());bmp[26..28].copy_from_slice(&1u16.to_le_bytes());bmp[28..30].copy_from_slice(&24u16.to_le_bytes());bmp[34..38].copy_from_slice(&4u32.to_le_bytes());bmp[56]=255;
        let result=preview(PreviewFormat::Image,&bmp).unwrap();assert_eq!(result.attachments.len(),1);assert_eq!(result.attachments[0].mime,"image/png");
        let bytes=base64::engine::general_purpose::STANDARD.decode(&result.attachments[0].data_base64).unwrap();assert_eq!(&bytes[..8],b"\x89PNG\r\n\x1a\n");
        assert!(preview(PreviewFormat::Image,b"not an image").is_err());
    }
    #[test]
    fn invalid_and_oversize_inputs_fail_closed() {
        assert!(preview(PreviewFormat::Tnef, b"corrupt").is_err());
        assert!(preview(PreviewFormat::Eml, &vec![0; crate::MAX_INPUT_BYTES + 1]).is_err());
    }
}
