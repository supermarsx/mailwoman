use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

#[derive(Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct EditorState {
    pub version: u8,
    #[serde(default)]
    pub identity_id: String,
    #[serde(default)]
    pub to: String,
    #[serde(default)]
    pub cc: String,
    #[serde(default)]
    pub bcc: String,
    #[serde(default)]
    pub subject: String,
    pub body_format: String,
    #[serde(default)]
    pub body_html: String,
    #[serde(default)]
    pub body_text: String,
    #[serde(default)]
    pub inserted_signatures: Vec<String>,
    #[serde(default)]
    pub in_reply_to: Vec<String>,
    #[serde(default)]
    pub references: Vec<String>,
    #[serde(default)]
    pub attachments: Vec<File>,
    #[serde(default)]
    pub inline: Vec<Inline>,
    #[serde(default)]
    pub request_receipt: bool,
    #[serde(default)]
    pub request_delivery_receipt: bool,
    #[serde(default)]
    pub open_tracking: bool,
    #[serde(default)]
    pub protected_content: bool,
}
#[derive(Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct File {
    pub blob_id: String,
    pub name: String,
    pub size: u64,
    pub content_type: Option<String>,
}
#[derive(Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Inline {
    pub blob_id: String,
    pub name: String,
    pub size: u64,
    pub cid: String,
    #[serde(rename = "type")]
    pub content_type: String,
}
fn header(value: &str, max: usize) -> bool {
    value.len() <= max && !value.chars().any(char::is_control)
}
fn ids(values: &[String], max: usize) -> bool {
    values.len() <= max && values.iter().all(|s| header(s, 512))
}
impl EditorState {
    pub fn validate(&self) -> Result<(), &'static str> {
        if self.protected_content {
            return Err("protectedDraftUnsupported");
        }
        if self.version != 1
            || !matches!(self.body_format.as_str(), "plain" | "html")
            || !header(&self.subject, 4096)
            || !header(&self.identity_id, 512)
            || [&self.to, &self.cc, &self.bcc]
                .iter()
                .any(|s| s.len() > 16384)
            || self.body_html.len() > 786432
            || self.body_text.len() > 786432
            || !ids(&self.inserted_signatures, 128)
            || !ids(&self.in_reply_to, 100)
            || !ids(&self.references, 100)
            || self.attachments.len() > 100
            || self.inline.len() > 32
        {
            return Err("invalidDraftState");
        }
        if self.attachments.iter().any(|file| {
            file.blob_id.is_empty()
                || !header(&file.blob_id, 512)
                || !header(&file.name, 512)
                || file.size > 32 * 1024 * 1024
                || file.content_type.as_ref().is_some_and(|s| !header(s, 128))
        }) || self.inline.iter().any(|file| {
            file.blob_id.is_empty()
                || !header(&file.blob_id, 512)
                || !header(&file.name, 512)
                || !header(&file.cid, 256)
                || !header(&file.content_type, 128)
                || file.size > 32 * 1024 * 1024
        }) {
            return Err("invalidDraftAttachments");
        }
        Ok(())
    }
    pub fn summary(&self) -> Value {
        json!({"subject":self.subject,"bodyFormat":self.body_format})
    }
    pub fn compose_spec(&self) -> Value {
        let mut spec = json!({"keywords":{"$draft":true,"$seen":true},"to":recipients(&self.to),"cc":recipients(&self.cc),"bcc":recipients(&self.bcc),"subject":self.subject,"inReplyTo":self.in_reply_to,"references":self.references});
        let plain = self.body_format == "plain";
        spec[if plain { "textBody" } else { "htmlBody" }] =
            json!([{"partId":"body","type":if plain{"text/plain"}else{"text/html"}}]);
        spec["bodyValues"] =
            json!({"body":{"value":if plain{&self.body_text}else{&self.body_html}}});
        let mut attachments:Vec<Value>=self.attachments.iter().map(|file|json!({"blobId":file.blob_id,"name":file.name,"size":file.size,"type":file.content_type,"disposition":"attachment"})).collect();
        attachments.extend(self.inline.iter().map(|file|json!({"blobId":file.blob_id,"name":file.name,"size":file.size,"type":file.content_type,"cid":file.cid,"disposition":"inline"})));
        if !attachments.is_empty() {
            spec["attachments"] = json!(attachments)
        }
        // Receipt/tracking intent remains editor state. Saving must not mint a
        // tracking token or create an outbound disposition request on its own.
        spec
    }
}
/// Preserve partial recipient text in sealed editor state. Only complete valid
/// mailbox tokens enter the unsent MIME projection; names are intentionally not
/// guessed from an incomplete token. No raw text becomes a header value.
fn recipients(value: &str) -> Vec<Value> {
    value
        .split([',', ';', '\r', '\n'])
        .filter_map(|token| {
            let token = token.trim();
            let address = if let Some((_, rest)) = token.rsplit_once('<') {
                rest.strip_suffix('>')?.trim()
            } else {
                token
            };
            mw_smtp::validate_mailbox(address).ok()?;
            Some(json!({"name":null,"email":address}))
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn partial_recipients_are_preserved_but_never_injected_into_mime() {
        let state:EditorState=serde_json::from_value(json!({"version":1,"bodyFormat":"plain","to":"unfinished, Person <valid@example.test>\r\nBcc: injected@example.test","bcc":"private@example.test","subject":"draft","bodyText":"unsent"})).unwrap();
        state.validate().unwrap();
        let spec = state.compose_spec();
        assert_eq!(
            spec["to"],
            json!([{"name":null,"email":"valid@example.test"}])
        );
        assert_eq!(spec["bcc"][0]["email"], "private@example.test");
        assert!(state.to.contains("unfinished"));
        assert!(spec.get("htmlBody").is_none());
    }
    #[test]
    fn protected_content_and_header_injection_are_refused() {
        let mut state: EditorState = serde_json::from_value(
            json!({"version":1,"bodyFormat":"html","protectedContent":true}),
        )
        .unwrap();
        assert_eq!(state.validate(), Err("protectedDraftUnsupported"));
        state.protected_content = false;
        state.subject = "draft\r\nBcc: victim@example.test".into();
        assert_eq!(state.validate(), Err("invalidDraftState"));
    }
}
