//! Request-local policy hint fence. This does not authenticate or grant access.
use crate::{Session, Store, StoreError, backend::q};
use sha2::{Digest, Sha256};

/// Opaque, Store-bound equality witness for a primary session and its policy
/// namespace. Keep only for the same request; always authenticate again.
#[derive(Clone)]
pub struct PrimaryPolicyWitness {
    scope: uuid::Uuid,
    revision: Option<i64>,
    source: [u8; 32],
}
impl PrimaryPolicyWitness {
    pub fn eligible(&self) -> bool {
        self.revision.is_some()
    }
    pub fn matches_revision(
        &self,
        revision: &crate::settings_policy::SettingsReadRevision,
    ) -> bool {
        self.eligible() && revision.scope() == self.scope && revision.number() == self.revision
    }
    pub fn matches(&self, fresh: &Self) -> bool {
        self.eligible()
            && fresh.eligible()
            && self.scope == fresh.scope
            && self.revision == fresh.revision
            && self.source == fresh.source
    }
}
impl std::fmt::Debug for PrimaryPolicyWitness {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PrimaryPolicyWitness")
            .field("eligible", &self.eligible())
            .finish_non_exhaustive()
    }
}
/// Plaintext session used only as a policy-resolution hint until a full, fresh
/// authentication observes a matching witness. Debug never prints credentials.
pub struct PrimaryPolicySnapshot {
    pub session: Session,
    pub last_seen: Option<String>,
    pub witness: PrimaryPolicyWitness,
}
impl std::fmt::Debug for PrimaryPolicySnapshot {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PrimaryPolicySnapshot")
            .field("witness", &self.witness)
            .finish_non_exhaustive()
    }
}
struct RawPrimarySnapshot {
    id: String,
    account_id: String,
    username: String,
    jmap_url: String,
    api_url: String,
    sealed: Vec<u8>,
    last_seen: Option<String>,
    witness: PrimaryPolicyWitness,
}
impl Store {
    pub(crate) fn primary_policy_witness_from_parts(&self,ids:[&str;5],sealed:&[u8],kind:Option<&str>,revision:Option<i64>)->PrimaryPolicyWitness {
        let mut source=Sha256::new();source.update(b"mailwoman-primary-policy-witness-v1\0");
        for value in ids {source.update((value.len() as u64).to_be_bytes());source.update(value.as_bytes());}
        source.update((sealed.len() as u64).to_be_bytes());source.update(sealed);
        match kind {Some(kind)=>{source.update([1]);source.update((kind.len() as u64).to_be_bytes());source.update(kind.as_bytes());},None=>source.update([0])}
        PrimaryPolicyWitness {scope:self.cache_scope(),revision,source:source.finalize().into()}
    }

    /// Original primary-session predicate/decode, plus one coherent namespace
    /// witness. The returned plaintext session is still only a policy hint.
    pub async fn primary_session_policy_snapshot(
        &self,
        id: &str,
    ) -> Result<PrimaryPolicySnapshot, StoreError> {
        let raw = self.read_primary_policy_source(id).await?;
        let credentials = crate::decode_creds(&self.key.open(&raw.sealed)?)?;
        Ok(PrimaryPolicySnapshot {
            session: Session {
                id: raw.id,
                account_id: raw.account_id,
                username: raw.username,
                jmap_url: raw.jmap_url,
                api_url: raw.api_url,
                credentials,
            },
            last_seen: raw.last_seen,
            witness: raw.witness,
        })
    }
    /// Fresh final source fence with exactly the SAME SELECT/fingerprint as the
    /// hint, but no extra credential decryption. Never authenticates by itself:
    /// the complete ordinary auth chain must already have decoded the session.
    pub async fn primary_session_policy_witness(
        &self,
        id: &str,
    ) -> Result<PrimaryPolicyWitness, StoreError> {
        Ok(self.read_primary_policy_source(id).await?.witness)
    }
    /// One snapshot: revision protects policies, aliases, groups, profiles and
    /// verified principal/binding/session-principal namespaces (0078). Account
    /// kind/presence protects protocol/native-subject classification. Ignore
    /// last_seen for equality: throttled telemetry is not authority.
    async fn read_primary_policy_source(&self, id: &str) -> Result<RawPrimarySnapshot, StoreError> {
        let row=q("SELECT s.id,s.account_id,s.username,s.jmap_url,s.api_url,s.sealed_creds,s.last_seen,r.n AS policy_revision,a.kind AS account_kind FROM sessions s LEFT JOIN settings_revision r ON r.id=1 LEFT JOIN accounts a ON a.id=s.account_id WHERE s.id=?1 AND s.linked_internal=0")
            .bind(id).fetch_optional(&self.backend).await?.ok_or(StoreError::NotFound)?;
        let id = row.get_string("id");
        let account_id = row.get_string("account_id");
        let username = row.get_string("username");
        let jmap_url = row.get_string("jmap_url");
        let api_url = row.get_string("api_url");
        let sealed = row.get_blob("sealed_creds");
        let account_kind=row.try_get_opt_string("account_kind")?;
        let revision = match row.get_json("policy_revision") {
            serde_json::Value::Null => None,
            serde_json::Value::Number(number) => Some(
                number
                    .as_i64()
                    .filter(|n| *n >= 0)
                    .ok_or_else(|| StoreError::Corrupt("invalid policy revision".into()))?,
            ),
            _ => return Err(StoreError::Corrupt("invalid policy revision".into())),
        };
        let witness=self.primary_policy_witness_from_parts([&id,&account_id,&username,&jmap_url,&api_url],&sealed,account_kind.as_deref(),revision);
        Ok(RawPrimarySnapshot {
            id,
            account_id,
            username,
            jmap_url,
            api_url,
            sealed,
            last_seen: row.try_get_opt_string("last_seen").ok().flatten(),
            witness,
        })
    }
}

#[cfg(test)]
#[path = "primary_policy_snapshot_tests.rs"]
mod tests;
