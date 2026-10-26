//! RFC 9007 adapter. Capability advertisement is owned by the completed host
//! contract; declaring these helpers alone must not advertise support.
use super::{refusal, same_mailbox};
use crate::engine::Engine;
use mw_mime::{MdnActionMode, MdnDisposition, MdnSendingMode};
use serde_json::{Value, json};
use std::collections::BTreeMap;

const LIMIT: usize = 32;

pub(super) struct SendOptions {
    pub identity: String,
    pub identity_id: String,
    pub name: String,
    pub subject: Option<String>,
    pub text: Option<String>,
    pub reporting_ua: Option<String>,
    pub include_original: bool,
    pub action: MdnActionMode,
    pub sending: MdnSendingMode,
    pub disposition: MdnDisposition,
    pub extensions: Vec<(String, String)>,
}

fn optional_string(value: &Value, key: &str, limit: usize) -> Result<Option<String>, Value> {
    match value.get(key) {
        None | Some(Value::Null) => Ok(None),
        Some(Value::String(text)) if text.len() <= limit => Ok(Some(text.clone())),
        _ => Err(json!({"type":"invalidProperties","properties":[key]})),
    }
}
impl SendOptions {
    fn parse(value: &Value, identity: &mw_store::IdentityRow) -> Result<Self, Value> {
        let Some(object) = value.as_object() else {
            return Err(refusal("invalidProperties", "MDN must be an object"));
        };
        if object.keys().any(|key| {
            ![
                "forEmailId",
                "subject",
                "textBody",
                "includeOriginalMessage",
                "reportingUA",
                "disposition",
                "finalRecipient",
                "extensionFields",
                "mdnGateway",
                "originalRecipient",
                "originalMessageId",
                "error",
            ]
            .contains(&key.as_str())
        }) {
            return Err(refusal("invalidProperties", "unknown MDN property"));
        }
        for name in [
            "mdnGateway",
            "originalRecipient",
            "originalMessageId",
            "error",
        ] {
            if value.get(name).is_some_and(|v| !v.is_null()) {
                return Err(json!({"type":"invalidProperties","properties":[name]}));
            }
        }
        if let Some(final_recipient) = optional_string(value, "finalRecipient", 1024)? {
            let Some((kind, address)) = final_recipient.split_once(';') else {
                return Err(refusal(
                    "forbiddenFrom",
                    "finalRecipient must be an allowed typed address",
                ));
            };
            if !kind.trim().eq_ignore_ascii_case("rfc822")
                || !same_mailbox(address.trim(), &identity.email)
            {
                return Err(refusal(
                    "forbiddenFrom",
                    "finalRecipient does not match the selected identity",
                ));
            }
        }
        let disposition = value
            .get("disposition")
            .and_then(Value::as_object)
            .ok_or_else(|| refusal("invalidProperties", "disposition is required"))?;
        if disposition.len() != 3 {
            return Err(refusal(
                "invalidProperties",
                "invalid disposition properties",
            ));
        }
        let action = match disposition.get("actionMode").and_then(Value::as_str) {
            Some("manual-action") => MdnActionMode::Manual,
            Some("automatic-action") => MdnActionMode::Automatic,
            _ => return Err(refusal("invalidProperties", "invalid actionMode")),
        };
        let sending = match disposition.get("sendingMode").and_then(Value::as_str) {
            Some("mdn-sent-manually") => MdnSendingMode::Manual,
            Some("mdn-sent-automatically") => MdnSendingMode::Automatic,
            _ => return Err(refusal("invalidProperties", "invalid sendingMode")),
        };
        let disposition = match disposition.get("type").and_then(Value::as_str) {
            Some("displayed") => MdnDisposition::Displayed,
            Some("deleted") => MdnDisposition::Deleted,
            Some("dispatched") => MdnDisposition::Dispatched,
            Some("processed") => MdnDisposition::Processed,
            _ => return Err(refusal("invalidProperties", "invalid disposition type")),
        };
        let include_original = match value.get("includeOriginalMessage") {
            None => false,
            Some(Value::Bool(value)) => *value,
            _ => {
                return Err(refusal(
                    "invalidProperties",
                    "includeOriginalMessage must be boolean",
                ));
            }
        };
        let extensions = match value.get("extensionFields") {
            None | Some(Value::Null) => Vec::new(),
            Some(Value::Object(values)) if values.len() <= 32 => values
                .iter()
                .map(|(key, value)| {
                    value
                        .as_str()
                        .map(|value| (key.clone(), value.to_owned()))
                        .ok_or_else(|| {
                            refusal(
                                "invalidProperties",
                                "extensionFields values must be strings",
                            )
                        })
                })
                .collect::<Result<Vec<_>, _>>()?,
            _ => return Err(refusal("invalidProperties", "invalid extensionFields")),
        };
        Ok(Self {
            identity: identity.email.clone(),
            identity_id: identity.id.clone(),
            name: identity.name.clone(),
            subject: optional_string(value, "subject", 4096)?,
            text: optional_string(value, "textBody", 65536)?,
            reporting_ua: optional_string(value, "reportingUA", 900)?,
            include_original,
            action,
            sending,
            disposition,
            extensions,
        })
    }
}

pub(super) fn document_value(document: mw_mime::MdnDocument) -> Value {
    json!({"forEmailId":null,"subject":document.subject,"textBody":document.text_body,"includeOriginalMessage":document.include_original_message,"reportingUA":document.reporting_ua,
    "disposition":{"actionMode":match document.action_mode{MdnActionMode::Manual=>"manual-action",MdnActionMode::Automatic=>"automatic-action"},"sendingMode":match document.sending_mode{MdnSendingMode::Manual=>"mdn-sent-manually",MdnSendingMode::Automatic=>"mdn-sent-automatically"},"type":document.disposition.as_str()},
    "mdnGateway":document.mdn_gateway,"originalRecipient":document.original_recipient,"finalRecipient":document.final_recipient,"originalMessageId":document.original_message_id,"error":if document.error.is_empty(){None}else{Some(document.error)},"extensionFields":if document.extension_fields.is_empty(){None}else{Some(document.extension_fields)}})
}

impl Engine {
    /// RFC 9007 identifier. Hosts advertise it only when the entire envelope
    /// and implicit Email/set contract is available for the account.
    pub const STANDARD_MDN_CAPABILITY: &'static str = "urn:ietf:params:jmap:mdn";

    /// Execute RFC 9007 data methods after the host has checked the request's
    /// `using` capabilities. Returns the primary response and optional implicit
    /// Email/set response; this helper does not advertise the capability.
    pub async fn mdn_standard_dispatch(
        &self,
        account_id: &str,
        name: &str,
        args: &Value,
    ) -> (Value, Option<Value>) {
        if args
            .get("accountId")
            .is_some_and(|value| value.as_str() != Some(account_id))
        {
            return (
                refusal(
                    "invalidArguments",
                    "accountId does not match the authorized account",
                ),
                None,
            );
        }
        if self.runtime(account_id).is_none() || self.store().get_account(account_id).await.is_err()
        {
            return (refusal("invalidArguments", "unknown account"), None);
        }
        if name == "MDN/parse" {
            return (self.mdn_standard_parse(account_id, args).await, None);
        }
        if name != "MDN/send" {
            return (refusal("unknownMethod", "unknown MDN method"), None);
        }
        self.mdn_standard_send(account_id, args).await
    }
    async fn mdn_standard_parse(&self, account_id: &str, args: &Value) -> Value {
        let Some(ids) = args.get("blobIds").and_then(Value::as_array) else {
            return refusal("invalidArguments", "blobIds must be an array");
        };
        if ids.len() > LIMIT {
            return refusal("requestTooLarge", "too many blobs");
        }
        if ids
            .iter()
            .any(|id| id.as_str().is_none_or(|id| id.is_empty() || id.len() > 256))
        {
            return refusal("invalidArguments", "invalid blob id");
        }
        let mut parsed = BTreeMap::new();
        let mut missing = Vec::new();
        let mut invalid = Vec::new();
        for id in ids.iter().filter_map(Value::as_str) {
            let size = if id.starts_with('U') {
                match self.store().upload_size(account_id, id).await {
                    Ok(size) => size,
                    Err(_) => return refusal("serverFail", "MDN metadata could not be read"),
                }
            } else {
                let stable = id.split_once('.').map_or(id, |(stable, _)| stable);
                match self.store().get_message(stable).await {
                    Ok(message) if message.account_id == account_id => {
                        Some(i64::try_from(message.size).unwrap_or(i64::MAX))
                    }
                    Ok(_) | Err(mw_store::StoreError::NotFound) => None,
                    Err(_) => return refusal("serverFail", "MDN metadata could not be read"),
                }
            };
            if size.is_some_and(|size| !(0..=9 * 1024 * 1024).contains(&size)) {
                invalid.push(id.to_owned());
                continue;
            }
            let bytes = if id.starts_with('U') {
                self.store()
                    .get_upload_limited(account_id, id, 9 * 1024 * 1024)
                    .await
                    .map(|upload| upload.map(|upload| upload.bytes))
                    .map_err(crate::backend::EngineError::from)
            } else if id.contains('.') {
                self.fetch_blob_limited(account_id, id, 9 * 1024 * 1024, 9 * 1024 * 1024)
                    .await
                    .map(|blob| blob.map(|blob| blob.bytes))
            } else {
                self.stored_raw_limited(account_id, id, 9 * 1024 * 1024)
                    .await
            };
            match bytes {
                Ok(Some(bytes)) => {
                    if let Some(document) = mw_mime::parse_mdn_document(&bytes) {
                        parsed.insert(id.to_owned(), document_value(document));
                    } else {
                        invalid.push(id.to_owned());
                    }
                }
                Ok(None) => missing.push(id.to_owned()),
                Err(_) => return refusal("serverFail", "MDN blob could not be read"),
            }
        }
        json!({"accountId":account_id,"parsed":if parsed.is_empty(){None}else{Some(parsed)},"notParsable":if invalid.is_empty(){None}else{Some(invalid)},"notFound":if missing.is_empty(){None}else{Some(missing)}})
    }
}

impl SendOptions {
    pub(super) fn build(
        &self,
        request: &super::ReceiptRequest,
        message: &mw_store::Message,
        original: &mw_jmap::Email,
        raw: &[u8],
    ) -> Result<Vec<u8>, mw_mime::MimeError> {
        mw_mime::build_mdn_with_options(
            &mw_mime::MdnInput {
                from: mw_mime::EmailAddress {
                    name: (!self.name.is_empty()).then(|| self.name.clone()),
                    email: self.identity.clone(),
                },
                to: request.to.clone(),
                final_recipient: self.identity.clone(),
                original_recipient: None,
                original_message_id: message
                    .message_id
                    .as_deref()
                    .map(|id| id.trim().trim_start_matches('<').trim_end_matches('>'))
                    .filter(|id| super::writable_message_id(id))
                    .map(str::to_owned),
                original_subject: original.subject.clone(),
                action_mode: self.action,
                sending_mode: self.sending,
                disposition: self.disposition.clone(),
                message_id: Some(crate::jmap::gen_message_id()),
            },
            &mw_mime::MdnOptions {
                subject: self.subject.as_deref(),
                text_body: self.text.as_deref(),
                reporting_ua: self.reporting_ua.as_deref(),
                original_message: self.include_original.then_some(raw),
                extension_fields: &self.extensions,
            },
        )
    }
}

impl Engine {
    async fn mdn_standard_send(&self, account_id: &str, args: &Value) -> (Value, Option<Value>) {
        let Some(identity_id) = args.get("identityId").and_then(Value::as_str) else {
            return (refusal("invalidArguments", "identityId is required"), None);
        };
        let identity = match self.store().list_identities(account_id).await {
            Ok(identities) => match identities
                .into_iter()
                .find(|identity| identity.id == identity_id)
            {
                Some(identity) => identity,
                None => return (refusal("invalidArguments", "unknown identity"), None),
            },
            Err(_) => return (refusal("serverFail", "identities could not be read"), None),
        };
        let Some(send) = args.get("send").and_then(Value::as_object) else {
            return (refusal("invalidArguments", "send must be a map"), None);
        };
        if send.len() > LIMIT {
            return (refusal("requestTooLarge", "too many MDNs"), None);
        }
        if send.is_empty() {
            return (
                json!({"accountId":account_id,"sent":null,"notSent":null}),
                None,
            );
        }
        let Some(patches) = args.get("onSuccessUpdateEmail").and_then(Value::as_object) else {
            return (
                refusal("invalidArguments", "onSuccessUpdateEmail must set $mdnsent"),
                None,
            );
        };
        if patches.len() != send.len()
            || send.keys().any(|id| {
                id.is_empty()
                    || id.len() > 256
                    || !patches.get(&format!("#{id}")).is_some_and(|patch| {
                        patch.as_object().is_some_and(|patch| {
                            patch.get("keywords/$mdnsent") == Some(&Value::Bool(true))
                                && !patch.contains_key("keywords")
                                && !patch.iter().any(|(key, value)| {
                                    key.strip_prefix("keywords/")
                                        .is_some_and(|word| word.eq_ignore_ascii_case("$mdnsent"))
                                        && value != &Value::Bool(true)
                                })
                        })
                    })
            })
        {
            return (
                refusal(
                    "invalidArguments",
                    "every MDN must set keywords/$mdnsent through its creation reference",
                ),
                None,
            );
        }
        let mut sent = BTreeMap::new();
        let mut refused = BTreeMap::new();
        let mut updates = serde_json::Map::new();
        for (creation, value) in send {
            let options = match SendOptions::parse(value, &identity) {
                Ok(options) => options,
                Err(error) => {
                    refused.insert(creation.clone(), error);
                    continue;
                }
            };
            let Some(email_id) = value
                .get("forEmailId")
                .and_then(Value::as_str)
                .filter(|id| !id.is_empty() && id.len() <= 256)
            else {
                refused.insert(
                    creation.clone(),
                    refusal("invalidProperties", "forEmailId is required"),
                );
                continue;
            };
            let result=self.mdn_send_common(account_id,&json!({"emailId":email_id,"automatic":options.sending==MdnSendingMode::Automatic}),Some(&options)).await;
            if result.get("sent") == Some(&Value::Bool(true)) {
                let mut defaults = result["standardDefaults"]
                    .as_object()
                    .cloned()
                    .unwrap_or_default();
                defaults.remove("forEmailId");
                for key in value.as_object().unwrap().keys() {
                    if !value[key].is_null() {
                        defaults.remove(key);
                    }
                }
                sent.insert(creation.clone(), Value::Object(defaults));
                updates.insert(
                    email_id.to_owned(),
                    patches[&format!("#{creation}")].clone(),
                );
            } else {
                let mut error = result;
                let kind = match error["type"].as_str().unwrap_or("serverFail") {
                    "mdnNotRequested" => "notFound",
                    "mdnNoRecipientAddress" => "forbiddenFrom",
                    "mdnAutomaticRefused" | "mdnNotAllowed" | "mdnNotSent" => "forbidden",
                    kind => kind,
                }
                .to_owned();
                error["type"] = Value::String(kind);
                refused.insert(creation.clone(), error);
            }
        }
        let implicit = if updates.is_empty() {
            None
        } else {
            let request = json!({"using":["urn:ietf:params:jmap:core","urn:ietf:params:jmap:mail"],"methodCalls":[["Email/set",{"accountId":account_id,"update":updates},"mdn"]]});
            let response = Box::pin(self.handle_jmap(account_id, &request)).await;
            Some(response["methodResponses"][0][1].clone())
        };
        (
            json!({"accountId":account_id,"sent":if sent.is_empty(){None}else{Some(sent)},"notSent":if refused.is_empty(){None}else{Some(refused)}}),
            implicit,
        )
    }
}
