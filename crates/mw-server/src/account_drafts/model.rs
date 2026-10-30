use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

#[derive(Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct EditorState {
    pub version: u8,
    #[serde(default)]
    pub identity_id: String,
    /// Editable intent only; never grants sender authority or enters draft MIME.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub from_override: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub importance: Option<String>,
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
    pub attach_public_key: bool,
    #[serde(default)]
    pub include_file_hashes: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub sent_mailbox_id: Option<String>,
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
            || self
                .sent_mailbox_id
                .as_ref()
                .is_some_and(|value| value.is_empty() || !header(value, 512))
            || self
                .importance
                .as_deref()
                .is_some_and(|value| !matches!(value, "high" | "normal" | "low"))
            || self
                .from_override
                .as_ref()
                .is_some_and(|value| !header(value, 254))
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
        let importance = self.importance.as_deref().unwrap_or("normal");
        spec["header:Importance:asText"] = json!(importance);
        spec["header:X-Priority:asText"] = json!(match importance {
            "high" => "1",
            "low" => "5",
            _ => "3",
        });
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
    let mut tokens = Vec::new();
    let (mut quoted, mut escaped, mut start) = (false, false, 0);
    for (at, c) in value.char_indices() {
        if escaped {
            escaped = false;
            continue;
        }
        if c == '\\' && quoted {
            escaped = true;
            continue;
        }
        if c == '"' {
            quoted = !quoted;
        }
        if !quoted && matches!(c, ',' | ';' | '\r' | '\n') {
            tokens.push(&value[start..at]);
            start = at + c.len_utf8();
        }
    }
    if !quoted && !escaped {
        tokens.push(&value[start..]);
    }
    tokens
        .into_iter()
        .filter_map(|token| {
            let address = crate::prefs_routes::mailbox_list_address(token)?;
            Some(json!({"name":null,"email":address}))
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn quoted_utf8_draft_recipients_remain_complete_and_separate() {
        let raw = "Name <\"jörg,a<>@x\"@bücher.example>, unfinished";
        assert_eq!(
            recipients(raw),
            vec![json!({"name":null,"email":"\"jörg,a<>@x\"@bücher.example"})]
        );
        assert_eq!(
            recipients("a@example.org\r\nBcc: victim@example.org"),
            vec![json!({"name":null,"email":"a@example.org"})]
        );
        assert!(recipients("\"a\r\nBcc: victim\"@example.org").is_empty());
    }
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
    fn outgoing_options_roundtrip_without_affecting_unsent_mime() {
        let mut state: EditorState = serde_json::from_value(json!({"version":1,"bodyFormat":"plain","attachPublicKey":true,"includeFileHashes":true,"sentMailboxId":"archive"})).unwrap();
        state.validate().unwrap();
        let encoded = serde_json::to_value(&state).unwrap();
        assert_eq!(encoded["attachPublicKey"], true);
        assert_eq!(encoded["includeFileHashes"], true);
        assert_eq!(encoded["sentMailboxId"], "archive");
        assert!(state.compose_spec().get("sentMailboxId").is_none());
        for invalid in ["".to_string(), "x".repeat(513), "bad\r\nfield".into()] {
            state.sent_mailbox_id = Some(invalid);
            assert_eq!(state.validate(), Err("invalidDraftState"));
        }
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
    #[test]
    fn from_override_roundtrips_as_editor_intent_without_mime_authority() {
        let mut state: EditorState = serde_json::from_value(
            json!({"version":1,"bodyFormat":"plain","fromOverride":"alias@example.test"}),
        )
        .unwrap();
        state.validate().unwrap();
        assert_eq!(
            serde_json::to_value(&state).unwrap()["fromOverride"],
            "alias@example.test"
        );
        assert!(state.compose_spec().get("from").is_none());
        state.from_override = Some(String::new());
        state.validate().unwrap();
        for invalid in [
            "x".repeat(255),
            "alias@example.test\r\nBcc: victim@example.test".into(),
        ] {
            state.from_override = Some(invalid);
            assert_eq!(state.validate(), Err("invalidDraftState"));
        }
    }
    #[test]
    fn draft_importance_is_bounded_and_uses_matching_headers() {
        let mut state: EditorState =
            serde_json::from_value(json!({"version":1,"bodyFormat":"plain"})).unwrap();
        assert_eq!(state.compose_spec()["header:X-Priority:asText"], "3");
        for (importance, priority) in [("high", "1"), ("normal", "3"), ("low", "5")] {
            state.importance = Some(importance.into());
            state.validate().unwrap();
            let spec = state.compose_spec();
            assert_eq!(spec["header:Importance:asText"], importance);
            assert_eq!(spec["header:X-Priority:asText"], priority);
        }
        state.importance = Some("high\r\nBcc: victim@example.test".into());
        assert_eq!(state.validate(), Err("invalidDraftState"));
    }
}
