//! Strict account Drafts persistence. Unlike composition staging, no local-only
//! fallback and no provider `submit` operation is permitted here.
use super::*;
use std::sync::Arc;

pub struct PreparedAccountDraft {
    account_id: String,
    runtime: AccountRuntime,
    mailbox: mw_store::Mailbox,
    raw: Vec<u8>,
    received_at: String,
}

impl Engine {
    pub async fn account_draft_support(&self, account_id: &str) -> Result<()> {
        if self
            .store()
            .get_zeroaccess(account_id)
            .await?
            .is_some_and(|row| row.enabled)
        {
            return Err(EngineError::Unsupported(
                "protected accounts cannot upload plaintext drafts".into(),
            ));
        }
        let runtime = self
            .runtime(account_id)
            .ok_or_else(|| EngineError::Unsupported("account is not connected".into()))?;
        if self.is_plugin_backed(account_id) || !runtime.backend.supports_dated_import() {
            return Err(EngineError::Unsupported(
                "this backend cannot save drafts to an upstream folder".into(),
            ));
        }
        let caps = runtime.backend.capabilities().await?;
        if !caps.uidplus && !caps.imap4rev2 {
            return Err(EngineError::Unsupported(
                "draft saving requires confirmed APPENDUID".into(),
            ));
        }
        let view = self.mailbox_role_view(account_id).await?;
        if view.problems.iter().any(|p| p.role == "drafts")
            || !view
                .mailboxes
                .iter()
                .any(|m| m.role.as_deref() == Some("drafts") && m.uidvalidity > 0)
        {
            return Err(EngineError::Unsupported(
                "map an existing upstream Drafts folder before saving".into(),
            ));
        }
        Ok(())
    }
    /// Resolve current identity, attachments and role without performing an upstream write.
    /// This method performs no APPEND and never transmits a message.
    pub async fn prepare_account_draft(
        &self,
        account_id: &str,
        spec: &Value,
        identity_id: Option<&str>,
    ) -> Result<PreparedAccountDraft> {
        self.account_draft_support(account_id).await?;
        let runtime = self
            .runtime(account_id)
            .ok_or_else(|| EngineError::Unsupported("account is not connected".into()))?;
        let mut spec = spec.clone();
        if !spec.is_object() {
            return Err(EngineError::Protocol(
                "draft specification must be an object".into(),
            ));
        }
        let (name, email) = if let Some(id) = identity_id.filter(|id| !id.is_empty()) {
            let row = self
                .store()
                .get_native_personal_identity(account_id, id)
                .await?
                .filter(|row| row.account_id == account_id)
                .ok_or_else(|| {
                    EngineError::Protocol("the selected identity is unavailable".into())
                })?;
            if !crate::identity::sender_address_allowed(
                self.store(),
                account_id,
                &runtime.identity,
                &row.email,
            )
            .await?
            {
                return Err(EngineError::Protocol(
                    "the selected identity is no longer authorized".into(),
                ));
            }
            (row.name, row.email)
        } else {
            (String::new(), runtime.identity.clone())
        };
        spec["from"] = json!([{"name":name,"email":email}]);
        check_compose_spec(&spec)
            .map_err(|_| EngineError::Protocol("invalid draft content".into()))?;
        // Bound actual immutable blobs before the general MIME composer reads
        // them, including inline parts (client-declared sizes are not trusted).
        let mut remaining = 32 * 1024 * 1024;
        for attachment in spec
            .get("attachments")
            .and_then(Value::as_array)
            .into_iter()
            .flatten()
        {
            let id = attachment
                .get("blobId")
                .and_then(Value::as_str)
                .ok_or_else(|| EngineError::Protocol("draft attachment needs a blob ID".into()))?;
            let bytes = if id.starts_with('U') {
                self.store()
                    .get_upload_limited(account_id, id, remaining)
                    .await?
                    .map(|upload| upload.bytes)
            } else if id.contains('.') {
                self.fetch_blob_limited(account_id, id, 32 * 1024 * 1024, remaining)
                    .await?
                    .map(|blob| blob.bytes)
            } else {
                self.stored_raw_limited(account_id, id, remaining).await?
            };
            let bytes = bytes.ok_or_else(|| {
                EngineError::Protocol(
                    "draft attachment is unavailable or exceeds the bounded size".into(),
                )
            })?;
            remaining = remaining
                .checked_sub(bytes.len())
                .ok_or_else(|| EngineError::Protocol("draft attachments exceed 32 MiB".into()))?;
        }
        let extras = self
            .compose_extras(account_id, &spec, &runtime.identity)
            .await
            .map_err(|error| match error {
                ComposeRefusal::Engine(error) => error,
                ComposeRefusal::Invalid(_) => {
                    EngineError::Protocol("invalid draft attachments or headers".into())
                }
            })?;
        let request = self
            .compose_from_spec(account_id, &spec, &runtime.identity, &gen_message_id())
            .await?;
        let raw = if request
            .from
            .as_ref()
            .is_none_or(|from| mw_smtp::validate_mailbox(&from.email).is_ok())
        {
            mw_mime::build_with(&request, &extras)
        } else if extras.inline_parts.is_empty() && extras.receipt_to.is_none() {
            mw_mime::build(&request)
        } else {
            return Err(EngineError::Protocol(
                "inline attachments require a valid From address".into(),
            ));
        }
        .map_err(|_| EngineError::Protocol("draft MIME could not be composed".into()))?;
        if raw.len() > 32 * 1024 * 1024 {
            return Err(EngineError::Protocol(
                "draft exceeds the 32 MiB limit".into(),
            ));
        }
        let view = self.mailbox_role_view(account_id).await?;
        let mailbox = view
            .mailboxes
            .into_iter()
            .find(|m| m.role.as_deref() == Some("drafts") && m.uidvalidity > 0)
            .ok_or_else(|| {
                EngineError::Unsupported("upstream Drafts folder is unavailable".into())
            })?;
        Ok(PreparedAccountDraft {
            account_id: account_id.into(),
            runtime,
            mailbox,
            raw,
            received_at: chrono::Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Secs, true),
        })
    }
    /// Any error after entering this method is conservatively uncertain. The
    /// caller must retain its durable claim and must not automatically retry.
    pub async fn persist_account_draft(&self, prepared: PreparedAccountDraft) -> Result<String> {
        let current = self
            .runtime(&prepared.account_id)
            .ok_or_else(|| EngineError::Unsupported("account disconnected before saving".into()))?;
        if !Arc::ptr_eq(&current.backend, &prepared.runtime.backend) {
            return Err(EngineError::Unsupported(
                "account backend changed before saving".into(),
            ));
        }
        let reference = prepared
            .runtime
            .backend
            .append_with_date(
                &RawMailboxRef {
                    name: prepared.mailbox.name.clone(),
                    uidvalidity: prepared.mailbox.uidvalidity,
                },
                &prepared.raw,
                &[Flag::Draft],
                &prepared.received_at,
            )
            .await?;
        if self
            .runtime(&prepared.account_id)
            .is_none_or(|current| !Arc::ptr_eq(&current.backend, &prepared.runtime.backend))
        {
            return Err(EngineError::Unsupported(
                "account backend changed while saving; upstream acceptance is uncertain".into(),
            ));
        }
        if !matches!(
            reference,
            MessageRef::Imap {
                uid: 1..,
                uidvalidity: 1..,
                ..
            }
        ) {
            return Err(EngineError::Protocol(
                "APPEND completed without durable coordinates".into(),
            ));
        }
        self.ingest_import(
            &prepared.account_id,
            &prepared.mailbox.id,
            &RawMessage {
                message_ref: reference,
                raw: prepared.raw,
                flags: vec![Flag::Draft],
                internaldate: Some(prepared.received_at),
            },
        )
        .await
    }
    pub(super) async fn account_draft_submission_reference(
        &self,
        account_id: &str,
        value: Option<&Value>,
    ) -> Result<Option<Value>> {
        let Some(value) = value else { return Ok(None) };
        let id = value
            .get("id")
            .and_then(Value::as_str)
            .filter(|s| s.len() <= 128)
            .ok_or_else(|| EngineError::Protocol("invalid account draft reference".into()))?;
        let revision = value
            .get("revision")
            .and_then(Value::as_str)
            .filter(|s| s.len() <= 128)
            .ok_or_else(|| EngineError::Protocol("invalid account draft revision".into()))?;
        let owner = self
            .store()
            .native_personal_subject(account_id)
            .await?
            .ok_or_else(|| {
                EngineError::Unsupported("verified account draft owner unavailable".into())
            })?;
        let row = self
            .store()
            .account_draft(&owner, id)
            .await?
            .ok_or_else(|| EngineError::Protocol("account draft no longer exists".into()))?;
        if row.account_id != account_id
            || row.revision.as_deref() != Some(revision)
            || row.status != "saved"
            || row.cleanup_email_id.is_some()
        {
            return Err(EngineError::Protocol(
                "account draft changed or needs cleanup; reopen it before sending".into(),
            ));
        }
        Ok(Some(json!({"owner":owner,"id":id,"revision":revision})))
    }
    pub(super) async fn cleanup_account_draft_after_send(
        &self,
        account_id: &str,
        value: &Value,
    ) -> Result<()> {
        use mw_store::account_drafts::DraftDiscard;
        let owner = self
            .store()
            .native_personal_subject(account_id)
            .await?
            .ok_or_else(|| EngineError::Unsupported("draft owner unavailable".into()))?;
        if value.get("owner").and_then(Value::as_str) != Some(owner.as_str()) {
            return Err(EngineError::Protocol(
                "draft owner changed; saved copy retained".into(),
            ));
        }
        let id = value["id"]
            .as_str()
            .ok_or_else(|| EngineError::Protocol("draft reference unavailable".into()))?;
        let revision = value["revision"]
            .as_str()
            .ok_or_else(|| EngineError::Protocol("draft revision unavailable".into()))?;
        let row = match self
            .store()
            .claim_account_draft_discard(&owner, id, Some(revision))
            .await?
        {
            DraftDiscard::Gone => return Ok(()),
            DraftDiscard::Ready(row) => row,
            _ => {
                return Err(EngineError::Protocol(
                    "saved draft changed or cleanup is pending; copy retained".into(),
                ));
            }
        };
        let email = row
            .email_id
            .as_deref()
            .ok_or_else(|| EngineError::Protocol("draft message unavailable".into()))?;
        self.cleanup_account_draft(account_id, email).await?;
        if !self
            .store()
            .complete_account_draft_discard(&owner, id, revision, email)
            .await?
        {
            return Err(EngineError::Protocol(
                "draft cleanup confirmation unavailable".into(),
            ));
        }
        Ok(())
    }
    pub async fn account_draft_parts(&self, account_id: &str, email_id: &str) -> Result<Value> {
        let result = self
            .email_get(
                account_id,
                &json!({"ids":[email_id],"properties":["id","attachments","mailwomanInlineParts"]}),
            )
            .await;
        result
            .get("list")
            .and_then(Value::as_array)
            .and_then(|list| list.first())
            .filter(|email| email["id"] == email_id)
            .cloned()
            .ok_or_else(|| EngineError::Protocol("saved draft parts could not be confirmed".into()))
    }
    /// Delete only the old, still-owned Drafts copy after replacement committed.
    /// Moving or editing its flags elsewhere refuses cleanup instead of deleting
    /// a message the user now treats as ordinary mail.
    pub async fn cleanup_account_draft(&self, account_id: &str, email_id: &str) -> Result<()> {
        let row = match self.store().get_message(email_id).await {
            Ok(row) if row.account_id == account_id => row,
            Err(mw_store::StoreError::NotFound) => return Ok(()),
            Ok(_) => {
                return Err(EngineError::Unsupported(
                    "draft belongs to another account".into(),
                ));
            }
            Err(error) => return Err(error.into()),
        };
        if row.uid == 0
            || row.uidvalidity == 0
            || !flags_from_json(&row.flags_json).contains(&Flag::Draft)
        {
            return Err(EngineError::Unsupported(
                "old draft is no longer an upstream draft".into(),
            ));
        }
        let view = self.mailbox_role_view(account_id).await?;
        let mailbox = view
            .mailboxes
            .iter()
            .find(|m| m.id == row.mailbox_id && m.role.as_deref() == Some("drafts"))
            .ok_or_else(|| {
                EngineError::Unsupported("old draft was moved; retain it for review".into())
            })?;
        let runtime = self
            .runtime(account_id)
            .ok_or_else(|| EngineError::Unsupported("account is not connected".into()))?;
        let at = MessageRef::Imap {
            mailbox: RawMailboxRef {
                name: mailbox.name.clone(),
                uidvalidity: row.uidvalidity,
            },
            uidvalidity: row.uidvalidity,
            uid: row.uid,
        };
        runtime.backend.expunge_messages(&[at]).await?;
        self.remove_local_email(account_id, email_id, &row.mailbox_id)
            .await
    }
}
