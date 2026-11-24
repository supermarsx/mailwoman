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
    /// `identity`: this session's verified principal row, exactly as the
    /// native admission composite's `principal` section reads it. Since 0123
    /// the settings revision no longer moves on identity writes, so the
    /// witness carries the identity values itself.
    pub(crate) fn primary_policy_witness_from_parts(&self,ids:[&str;5],sealed:&[u8],kind:Option<&str>,identity:Option<&crate::native_admission_snapshot::NativePrincipalSource>,revision:Option<i64>)->PrimaryPolicyWitness {
        let mut source=Sha256::new();source.update(b"mailwoman-primary-policy-witness-v2\0");
        let text=|source:&mut Sha256,value:&str|{source.update((value.len() as u64).to_be_bytes());source.update(value.as_bytes());};
        for value in ids {text(&mut source,value);}
        source.update((sealed.len() as u64).to_be_bytes());source.update(sealed);
        match kind {Some(kind)=>{source.update([1]);text(&mut source,kind);},None=>source.update([0])}
        match identity {
            None=>source.update([0]),
            Some(p)=>{
                source.update([1]);
                for value in [&p.primary_account_id,&p.principal_id,&p.binding_id,&p.binding_principal_id,&p.state,&p.binding_kind,&p.authentication_method] {text(&mut source,value);}
                source.update(p.auth_time.to_be_bytes());
                match p.retired_at {Some(at)=>{source.update([1]);source.update(at.to_be_bytes());},None=>source.update([0])}
            }
        }
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
    /// One snapshot: revision protects policies, aliases and groups/profiles
    /// (0078); the verified principal/binding/session-principal row is read
    /// here and hashed itself (0123 took identity writes out of the revision).
    /// Account kind/presence protects protocol/native-subject classification.
    /// Ignore last_seen for equality: throttled telemetry is not authority.
    async fn read_primary_policy_source(&self, id: &str) -> Result<RawPrimarySnapshot, StoreError> {
        // The identity columns use the native admission composite's `principal`
        // section joins (session_principal, identity_principal, identity_binding
        // by sp.binding_id), present only when all three rows exist.
        let row=q("SELECT s.id,s.account_id,s.username,s.jmap_url,s.api_url,s.sealed_creds,s.last_seen,r.n AS policy_revision,a.kind AS account_kind,CAST(CASE WHEN p.id IS NULL OR b.id IS NULL THEN 0 ELSE 1 END AS BIGINT) AS has_principal,sp.primary_account_id AS principal_account_id,sp.principal_id,sp.binding_id,b.principal_id AS binding_principal_id,p.state AS principal_state,b.kind AS binding_kind,sp.auth_time AS principal_auth_time,sp.authentication_method AS principal_auth_method,b.retired_at AS binding_retired_at FROM sessions s LEFT JOIN settings_revision r ON r.id=1 LEFT JOIN accounts a ON a.id=s.account_id LEFT JOIN session_principal sp ON sp.session_id=s.id LEFT JOIN identity_principal p ON p.id=sp.principal_id LEFT JOIN identity_binding b ON b.id=sp.binding_id WHERE s.id=?1 AND s.linked_internal=0")
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
        let identity = if row.get_i64("has_principal") == 1 {
            let required = |key: &str| row.try_get_opt_string(key)?.ok_or_else(|| StoreError::Corrupt(format!("missing {key}")));
            Some(crate::native_admission_snapshot::NativePrincipalSource {
                primary_account_id: required("principal_account_id")?,
                principal_id: required("principal_id")?,
                binding_id: required("binding_id")?,
                binding_principal_id: required("binding_principal_id")?,
                state: required("principal_state")?,
                binding_kind: required("binding_kind")?,
                retired_at: row.get_opt_i64("binding_retired_at"),
                auth_time: row.get_opt_i64("principal_auth_time")
                    .ok_or_else(|| StoreError::Corrupt("missing principal auth time".into()))?,
                authentication_method: required("principal_auth_method")?,
            })
        } else {
            None
        };
        let witness=self.primary_policy_witness_from_parts([&id,&account_id,&username,&jmap_url,&api_url],&sealed,account_kind.as_deref(),identity.as_ref(),revision);
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
