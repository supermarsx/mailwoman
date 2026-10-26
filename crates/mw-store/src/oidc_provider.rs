//! OIDC issuer authority. Identity codes/tokens never share the mail OAuth table.
//! Writes serialize with principal retirement and session identity rotation.
use crate::{
    Store, StoreError,
    backend::{Row, q},
};
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OidcProviderConfig {
    pub issuer: String,
    pub enabled: bool,
    pub revision: String,
    pub updated_at: i64,
}
#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OidcClientRow {
    pub client_id: String,
    pub config: Value,
    pub enabled: bool,
    pub revision: String,
    pub updated_at: i64,
    #[serde(skip)]
    pub secret_hash: Option<String>,
}
pub struct OidcKeyRotation<'a> {
    pub kid: &'a str,
    pub issuer: &'a str,
    pub private_pem: &'a [u8],
    pub public_jwk: &'a Value,
    pub expected_kid: Option<&'a str>,
    pub now: i64,
}
pub struct OidcSigningKey {
    pub kid: String,
    pub issuer: String,
    pub private_pem: Vec<u8>,
    pub public_jwk: Value,
    pub active: bool,
    pub retain_until: i64,
}
#[derive(Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OidcGrant {
    pub id: String,
    pub client_id: String,
    pub client_revision: String,
    pub provider_revision: String,
    pub principal_id: String,
    pub session_id: String,
    pub claims: Value,
    pub scopes: Vec<String>,
    pub nonce: Option<String>,
    pub auth_time: i64,
    pub created_at: i64,
    pub expires_at: i64,
}
pub struct OidcAuthorization {
    pub request_hash: String,
    pub session_id: String,
    pub client_id: String,
    pub client_revision: String,
    pub provider_revision: String,
    pub request: Value,
    pub expires_at: i64,
}
pub struct OidcBrowserFlow {
    pub flow_hash: String,
    pub browser_hash: String,
    pub source_hash: String,
    pub request: Value,
    pub initial_session_id: Option<String>,
    pub created_at: i64,
    pub expires_at: i64,
}
pub struct OidcIssueCode<'a> {
    pub request_hash: &'a str,
    pub grant: &'a OidcGrant,
    pub code_hash: &'a str,
    pub redirect_uri: &'a str,
    pub pkce_challenge: &'a str,
    pub now: i64,
}
#[derive(Clone, Copy)]
pub enum OidcRedeemKind<'a> {
    Code {
        redirect_uri: &'a str,
        pkce_challenge: &'a str,
    },
    Refresh,
}
pub struct OidcRedeem<'a> {
    pub presented_hash: &'a str,
    pub client_id: &'a str,
    pub client_revision: &'a str,
    pub kind: OidcRedeemKind<'a>,
    pub scope: Option<&'a [String]>,
    pub access_hash: &'a str,
    pub refresh_hash: Option<&'a str>,
    pub now: i64,
}
pub enum OidcRedemption {
    Issued(Box<OidcGrant>),
    Invalid,
    Replay,
    InvalidScope,
}
fn parse(value: &str) -> Result<Value, StoreError> {
    serde_json::from_str(value).map_err(|_| StoreError::Corrupt("invalid OIDC record".into()))
}
fn hash_valid(value: &str) -> bool {
    value.len() == 64 && value.bytes().all(|b| b.is_ascii_hexdigit())
}
fn client(row: &Row) -> Result<OidcClientRow, StoreError> {
    Ok(OidcClientRow {
        client_id: row.get_string("client_id"),
        config: parse(&row.get_string("config_json"))?,
        enabled: row.get_i64("enabled") == 1,
        revision: row.get_string("revision"),
        updated_at: row.get_i64("updated_at"),
        secret_hash: row.get_opt_string("secret_hash"),
    })
}
fn grant(row: &Row) -> Result<OidcGrant, StoreError> {
    Ok(OidcGrant {
        id: row.get_string("id"),
        client_id: row.get_string("client_id"),
        client_revision: row.get_string("client_revision"),
        provider_revision: row.get_string("provider_revision"),
        principal_id: row.get_string("principal_id"),
        session_id: row.get_string("session_id"),
        claims: parse(&row.get_string("claims_json"))?,
        scopes: serde_json::from_str(&row.get_string("scopes_json"))
            .map_err(|_| StoreError::Corrupt("invalid OIDC scopes".into()))?,
        nonce: row.get_opt_string("nonce"),
        auth_time: row.get_i64("auth_time"),
        created_at: row.get_i64("created_at"),
        expires_at: row.get_i64("expires_at"),
    })
}
// This query deliberately joins the current verified primary binding. A retired
// principal, revoked session, disabled provider/client or edited client invalidates
// outstanding opaque authority immediately.
const LIVE_GRANT: &str = "SELECT g.* FROM oidc_grant g JOIN oidc_client c ON c.client_id=g.client_id AND c.revision=g.client_revision AND c.enabled=1 JOIN oidc_provider_config cfg ON cfg.id=1 AND cfg.enabled=1 AND cfg.revision=g.provider_revision JOIN session_principal sp ON sp.session_id=g.session_id AND sp.principal_id=g.principal_id JOIN identity_principal p ON p.id=g.principal_id AND p.state='active' JOIN identity_binding b ON b.id=sp.binding_id AND b.principal_id=sp.principal_id AND b.retired_at IS NULL JOIN sessions s ON s.id=sp.session_id WHERE g.id=?1 AND g.revoked_at IS NULL AND g.expires_at>?2";
impl Store {
    pub async fn begin_oidc_browser_flow(
        &self,
        row: &OidcBrowserFlow,
        now: i64,
    ) -> Result<bool, StoreError> {
        if !hash_valid(&row.flow_hash)
            || !hash_valid(&row.browser_hash)
            || !hash_valid(&row.source_hash)
            || row.request.to_string().len() > 16384
            || row.expires_at <= now
            || row.expires_at > now + 600
        {
            return Ok(false);
        }
        let mut tx = self.backend.begin().await?;
        q("UPDATE identity_serialization SET id=id WHERE id=1")
            .execute_tx(&mut tx)
            .await?;
        q("DELETE FROM oidc_browser_flow WHERE flow_hash IN (SELECT flow_hash FROM oidc_browser_flow WHERE expires_at<=?1 ORDER BY expires_at LIMIT 200)").bind(now).execute_tx(&mut tx).await?;
        let count = q("SELECT COUNT(*) AS n FROM oidc_browser_flow")
            .fetch_all_tx(&mut tx)
            .await?;
        if count[0].get_i64("n") >= 20000 {
            return Ok(false);
        }
        let count =
            q("SELECT COUNT(*) AS n FROM oidc_browser_flow WHERE source_hash=?1 AND created_at>?2")
                .bind(&row.source_hash)
                .bind(now - 60)
                .fetch_all_tx(&mut tx)
                .await?;
        if count[0].get_i64("n") >= 30 {
            return Ok(false);
        }
        let count=q("SELECT COUNT(*) AS n FROM oidc_browser_flow WHERE browser_hash=?1 AND expires_at>?2 AND consumed_at IS NULL").bind(&row.browser_hash).bind(now).fetch_all_tx(&mut tx).await?;
        if count[0].get_i64("n") >= 16 {
            return Ok(false);
        }
        q("INSERT INTO oidc_browser_flow(flow_hash,browser_hash,source_hash,request_json,initial_session_id,created_at,expires_at) VALUES(?1,?2,?3,?4,?5,?6,?7)").bind(&row.flow_hash).bind(&row.browser_hash).bind(&row.source_hash).bind(row.request.to_string()).bind(row.initial_session_id.as_deref()).bind(now).bind(row.expires_at).execute_tx(&mut tx).await?;
        tx.commit().await?;
        Ok(true)
    }
    pub async fn consume_oidc_browser_flow(
        &self,
        flow_hash: &str,
        browser_hash: &str,
        now: i64,
    ) -> Result<bool, StoreError> {
        Ok(q("UPDATE oidc_browser_flow SET consumed_at=?3 WHERE flow_hash=?1 AND browser_hash=?2 AND expires_at>?3 AND consumed_at IS NULL").bind(flow_hash).bind(browser_hash).bind(now).execute(&self.backend).await? == 1)
    }
    pub async fn oidc_browser_flow(
        &self,
        flow_hash: &str,
        browser_hash: &str,
        now: i64,
    ) -> Result<Option<OidcBrowserFlow>, StoreError> {
        q("SELECT * FROM oidc_browser_flow WHERE flow_hash=?1 AND browser_hash=?2 AND expires_at>?3 AND consumed_at IS NULL").bind(flow_hash).bind(browser_hash).bind(now).fetch_optional(&self.backend).await?.map(|row|Ok(OidcBrowserFlow{flow_hash:row.get_string("flow_hash"),browser_hash:row.get_string("browser_hash"),source_hash:row.get_string("source_hash"),request:parse(&row.get_string("request_json"))?,initial_session_id:row.get_opt_string("initial_session_id"),created_at:row.get_i64("created_at"),expires_at:row.get_i64("expires_at")})).transpose()
    }
    pub async fn deny_oidc_authorization(
        &self,
        request: &str,
        session: &str,
        now: i64,
    ) -> Result<bool, StoreError> {
        let mut tx = self.backend.begin().await?;
        q("UPDATE identity_serialization SET id=id WHERE id=1")
            .execute_tx(&mut tx)
            .await?;
        let changed=q("UPDATE oidc_authorization SET consumed_at=?3 WHERE request_hash=?1 AND session_id=?2 AND consumed_at IS NULL AND expires_at>?3").bind(request).bind(session).bind(now).execute_tx(&mut tx).await?;
        if changed == 1 {
            q("UPDATE oidc_browser_flow SET consumed_at=?2 WHERE flow_hash=?1")
                .bind(request)
                .bind(now)
                .execute_tx(&mut tx)
                .await?;
        }
        tx.commit().await?;
        Ok(changed == 1)
    }

    pub async fn oidc_provider_config(&self) -> Result<Option<OidcProviderConfig>, StoreError> {
        Ok(
            q("SELECT issuer,enabled,revision,updated_at FROM oidc_provider_config WHERE id=1")
                .fetch_optional(&self.backend)
                .await?
                .map(|r| OidcProviderConfig {
                    issuer: r.get_string("issuer"),
                    enabled: r.get_i64("enabled") == 1,
                    revision: r.get_string("revision"),
                    updated_at: r.get_i64("updated_at"),
                }),
        )
    }
    pub async fn put_oidc_provider_config(
        &self,
        issuer: &str,
        enabled: bool,
        expected: Option<&str>,
        now: i64,
    ) -> Result<Option<String>, StoreError> {
        let mut tx = self.backend.begin().await?;
        q("UPDATE identity_serialization SET id=id WHERE id=1")
            .execute_tx(&mut tx)
            .await?;
        let old = q("SELECT revision FROM oidc_provider_config WHERE id=1")
            .fetch_all_tx(&mut tx)
            .await?;
        if old.first().map(|r| r.get_string("revision")).as_deref() != expected {
            return Ok(None);
        }
        let revision = uuid::Uuid::new_v4().to_string();
        q("INSERT INTO oidc_provider_config(id,issuer,enabled,revision,updated_at) VALUES(1,?1,?2,?3,?4) ON CONFLICT(id) DO UPDATE SET issuer=excluded.issuer,enabled=excluded.enabled,revision=excluded.revision,updated_at=excluded.updated_at").bind(issuer).bind(i64::from(enabled)).bind(&revision).bind(now).execute_tx(&mut tx).await?;
        tx.commit().await?;
        Ok(Some(revision))
    }
    pub async fn create_oidc_client(
        &self,
        row: &OidcClientRow,
        name: &str,
        approved_by: &str,
    ) -> Result<Option<String>, StoreError> {
        let mut tx = self.backend.begin().await?;
        q("UPDATE identity_serialization SET id=id WHERE id=1")
            .execute_tx(&mut tx)
            .await?;
        if !q("SELECT client_id FROM oauth_clients WHERE client_id=?1")
            .bind(&row.client_id)
            .fetch_all_tx(&mut tx)
            .await?
            .is_empty()
        {
            return Ok(None);
        }
        let count = q("SELECT COUNT(*) AS n FROM oidc_client")
            .fetch_all_tx(&mut tx)
            .await?;
        if count[0].get_i64("n") >= 256 {
            return Ok(None);
        }
        let revision = uuid::Uuid::new_v4().to_string();
        let redirects = row
            .config
            .get("redirectUris")
            .ok_or_else(|| StoreError::Corrupt("missing OIDC redirects".into()))?
            .to_string();
        q("INSERT INTO oauth_clients(client_id,name,redirect_uris,approved_by,created_at) VALUES(?1,?2,?3,?4,?5)").bind(&row.client_id).bind(name).bind(redirects).bind(approved_by).bind(chrono::Utc::now().to_rfc3339()).execute_tx(&mut tx).await?;
        q("INSERT INTO oidc_client(client_id,config_json,enabled,secret_hash,revision,updated_at) VALUES(?1,?2,?3,?4,?5,?6)").bind(&row.client_id).bind(row.config.to_string()).bind(i64::from(row.enabled)).bind(row.secret_hash.as_deref()).bind(&revision).bind(row.updated_at).execute_tx(&mut tx).await?;
        tx.commit().await?;
        Ok(Some(revision))
    }
    pub async fn delete_oidc_client(&self, id: &str, expected: &str) -> Result<bool, StoreError> {
        let mut tx = self.backend.begin().await?;
        q("UPDATE identity_serialization SET id=id WHERE id=1")
            .execute_tx(&mut tx)
            .await?;
        let changed = q("DELETE FROM oidc_client WHERE client_id=?1 AND revision=?2")
            .bind(id)
            .bind(expected)
            .execute_tx(&mut tx)
            .await?;
        if changed == 1 {
            q("DELETE FROM oauth_clients WHERE client_id=?1")
                .bind(id)
                .execute_tx(&mut tx)
                .await?;
        }
        tx.commit().await?;
        Ok(changed == 1)
    }
    pub async fn oidc_public_keys(&self, issuer: &str, now: i64) -> Result<Vec<Value>, StoreError> {
        q("SELECT public_jwk_json FROM oidc_signing_key WHERE issuer=?1 AND (active=1 OR retain_until>?2) ORDER BY created_at DESC,kid").bind(issuer).bind(now).fetch_all(&self.backend).await?.iter().map(|r|parse(&r.get_string("public_jwk_json"))).collect()
    }
    pub async fn oidc_consents(&self, principal: &str) -> Result<Vec<Value>, StoreError> {
        q("SELECT c.client_id,c.scopes_json,c.updated_at,o.name FROM oidc_consent c JOIN oauth_clients o ON o.client_id=c.client_id WHERE c.principal_id=?1 ORDER BY c.updated_at DESC").bind(principal).fetch_all(&self.backend).await?.iter().map(|r|Ok(serde_json::json!({"clientId":r.get_string("client_id"),"name":r.get_string("name"),"scopes":parse(&r.get_string("scopes_json"))?,"updatedAt":r.get_i64("updated_at")}))).collect()
    }
    pub async fn oidc_clients(&self) -> Result<Vec<OidcClientRow>, StoreError> {
        q("SELECT * FROM oidc_client ORDER BY client_id")
            .fetch_all(&self.backend)
            .await?
            .iter()
            .map(client)
            .collect()
    }
    pub async fn oidc_client(&self, id: &str) -> Result<Option<OidcClientRow>, StoreError> {
        q("SELECT * FROM oidc_client WHERE client_id=?1")
            .bind(id)
            .fetch_optional(&self.backend)
            .await?
            .as_ref()
            .map(client)
            .transpose()
    }
    /// The caller supplies an already validated complete sidecar. A registry
    /// client must exist; mail-client registration alone never enables OIDC.
    pub async fn put_oidc_client(
        &self,
        row: &OidcClientRow,
        expected: Option<&str>,
    ) -> Result<Option<String>, StoreError> {
        self.put_oidc_client_named(row, expected, None).await
    }
    pub async fn put_oidc_client_named(
        &self,
        row: &OidcClientRow,
        expected: Option<&str>,
        name: Option<&str>,
    ) -> Result<Option<String>, StoreError> {
        if name.is_some_and(|name| {
            name.trim().is_empty() || name.len() > 120 || name.chars().any(char::is_control)
        }) {
            return Ok(None);
        }
        let mut tx = self.backend.begin().await?;
        q("UPDATE identity_serialization SET id=id WHERE id=1")
            .execute_tx(&mut tx)
            .await?;
        if q("SELECT client_id FROM oauth_clients WHERE client_id=?1")
            .bind(&row.client_id)
            .fetch_all_tx(&mut tx)
            .await?
            .is_empty()
        {
            return Ok(None);
        }
        let old = q("SELECT revision FROM oidc_client WHERE client_id=?1")
            .bind(&row.client_id)
            .fetch_all_tx(&mut tx)
            .await?;
        if old.first().map(|r| r.get_string("revision")).as_deref() != expected {
            return Ok(None);
        }
        let revision = uuid::Uuid::new_v4().to_string();
        q("INSERT INTO oidc_client(client_id,config_json,enabled,secret_hash,revision,updated_at) VALUES(?1,?2,?3,?4,?5,?6) ON CONFLICT(client_id) DO UPDATE SET config_json=excluded.config_json,enabled=excluded.enabled,secret_hash=excluded.secret_hash,revision=excluded.revision,updated_at=excluded.updated_at").bind(&row.client_id).bind(row.config.to_string()).bind(i64::from(row.enabled)).bind(row.secret_hash.as_deref()).bind(&revision).bind(row.updated_at).execute_tx(&mut tx).await?;
        if let Some(name) = name {
            q("UPDATE oauth_clients SET name=?2 WHERE client_id=?1")
                .bind(&row.client_id)
                .bind(name.trim())
                .execute_tx(&mut tx)
                .await?;
        }
        tx.commit().await?;
        Ok(Some(revision))
    }
    pub async fn rotate_oidc_signing_key(
        &self,
        kid: &str,
        issuer: &str,
        private_pem: &[u8],
        public_jwk: &Value,
        now: i64,
    ) -> Result<(), StoreError> {
        self.rotate_oidc_key(
            OidcKeyRotation {
                kid,
                issuer,
                private_pem,
                public_jwk,
                expected_kid: None,
                now,
            },
            false,
        )
        .await?;
        Ok(())
    }
    pub async fn rotate_oidc_signing_key_if_current(
        &self,
        change: OidcKeyRotation<'_>,
    ) -> Result<bool, StoreError> {
        self.rotate_oidc_key(change, true).await
    }
    async fn rotate_oidc_key(
        &self,
        change: OidcKeyRotation<'_>,
        compare: bool,
    ) -> Result<bool, StoreError> {
        let OidcKeyRotation {
            kid,
            issuer,
            private_pem,
            public_jwk,
            expected_kid,
            now,
        } = change;
        let expected = compare.then_some(expected_kid);
        let sealed = self
            .key
            .seal_bound(private_pem, "oidc-signing-key", &[issuer, kid])?;
        let mut tx = self.backend.begin().await?;
        q("UPDATE identity_serialization SET id=id WHERE id=1")
            .execute_tx(&mut tx)
            .await?;
        if let Some(expected) = expected {
            let current = q("SELECT kid FROM oidc_signing_key WHERE issuer=?1 AND active=1")
                .bind(issuer)
                .fetch_all_tx(&mut tx)
                .await?;
            if current.first().map(|row| row.get_string("kid")).as_deref() != expected {
                return Ok(false);
            }
        }
        q("UPDATE oidc_signing_key SET active=0,retain_until=?2 WHERE issuer=?1 AND active=1")
            .bind(issuer)
            .bind(now + 900)
            .execute_tx(&mut tx)
            .await?;
        q("INSERT INTO oidc_signing_key(kid,issuer,private_sealed,public_jwk_json,active,created_at,retain_until) VALUES(?1,?2,?3,?4,1,?5,?6)").bind(kid).bind(issuer).bind(sealed).bind(public_jwk.to_string()).bind(now).bind(i64::MAX).execute_tx(&mut tx).await?;
        q("DELETE FROM oidc_signing_key WHERE active=0 AND retain_until<=?1")
            .bind(now)
            .execute_tx(&mut tx)
            .await?;
        tx.commit().await?;
        Ok(true)
    }
    pub async fn oidc_signing_keys(
        &self,
        issuer: &str,
        now: i64,
    ) -> Result<Vec<OidcSigningKey>, StoreError> {
        let rows=q("SELECT * FROM oidc_signing_key WHERE issuer=?1 AND (active=1 OR retain_until>?2) ORDER BY created_at DESC,kid").bind(issuer).bind(now).fetch_all(&self.backend).await?;
        rows.iter()
            .map(|r| {
                let kid = r.get_string("kid");
                Ok(OidcSigningKey {
                    private_pem: self.key.open_bound(
                        &r.get_blob("private_sealed"),
                        "oidc-signing-key",
                        &[issuer, &kid],
                    )?,
                    kid,
                    issuer: issuer.into(),
                    public_jwk: parse(&r.get_string("public_jwk_json"))?,
                    active: r.get_i64("active") == 1,
                    retain_until: r.get_i64("retain_until"),
                })
            })
            .collect()
    }
    pub async fn begin_oidc_authorization(
        &self,
        row: &OidcAuthorization,
        now: i64,
    ) -> Result<bool, StoreError> {
        if !hash_valid(&row.request_hash)
            || row.request.to_string().len() > 16384
            || row.expires_at > now + 600
            || row.expires_at <= now
        {
            return Ok(false);
        }
        let mut tx = self.backend.begin().await?;
        q("UPDATE identity_serialization SET id=id WHERE id=1")
            .execute_tx(&mut tx)
            .await?;
        q("DELETE FROM oidc_authorization WHERE expires_at<=?1")
            .bind(now)
            .execute_tx(&mut tx)
            .await?;
        let valid=q("SELECT sp.session_id FROM session_principal sp JOIN identity_principal p ON p.id=sp.principal_id AND p.state='active' JOIN identity_binding b ON b.id=sp.binding_id AND b.principal_id=sp.principal_id AND b.retired_at IS NULL JOIN oidc_client c ON c.client_id=?2 AND c.enabled=1 AND c.revision=?3 JOIN oidc_provider_config cfg ON cfg.id=1 AND cfg.enabled=1 AND cfg.revision=?4 WHERE sp.session_id=?1").bind(&row.session_id).bind(&row.client_id).bind(&row.client_revision).bind(&row.provider_revision).fetch_all_tx(&mut tx).await?;
        if valid.is_empty() {
            return Ok(false);
        }
        let existing = q("SELECT * FROM oidc_authorization WHERE request_hash=?1")
            .bind(&row.request_hash)
            .fetch_all_tx(&mut tx)
            .await?;
        if let Some(existing) = existing.first() {
            return Ok(existing.get_string("session_id") == row.session_id
                && existing.get_string("client_id") == row.client_id
                && existing.get_string("client_revision") == row.client_revision
                && existing.get_string("provider_revision") == row.provider_revision
                && parse(&existing.get_string("request_json"))? == row.request
                && existing.get_opt_i64("consumed_at").is_none()
                && existing.get_i64("expires_at") > now);
        }
        let count = q("SELECT COUNT(*) AS n FROM oidc_authorization WHERE session_id=?1")
            .bind(&row.session_id)
            .fetch_all_tx(&mut tx)
            .await?;
        if count[0].get_i64("n") >= 16 {
            return Ok(false);
        }
        q("INSERT INTO oidc_authorization(request_hash,session_id,client_id,client_revision,provider_revision,request_json,expires_at) VALUES(?1,?2,?3,?4,?5,?6,?7)").bind(&row.request_hash).bind(&row.session_id).bind(&row.client_id).bind(&row.client_revision).bind(&row.provider_revision).bind(row.request.to_string()).bind(row.expires_at).execute_tx(&mut tx).await?;
        tx.commit().await?;
        Ok(true)
    }
    pub async fn oidc_authorization(
        &self,
        hash: &str,
        session: &str,
        now: i64,
    ) -> Result<Option<OidcAuthorization>, StoreError> {
        let row=q("SELECT * FROM oidc_authorization WHERE request_hash=?1 AND session_id=?2 AND consumed_at IS NULL AND expires_at>?3").bind(hash).bind(session).bind(now).fetch_optional(&self.backend).await?;
        row.map(|r| {
            Ok(OidcAuthorization {
                request_hash: r.get_string("request_hash"),
                session_id: r.get_string("session_id"),
                client_id: r.get_string("client_id"),
                client_revision: r.get_string("client_revision"),
                provider_revision: r.get_string("provider_revision"),
                request: parse(&r.get_string("request_json"))?,
                expires_at: r.get_i64("expires_at"),
            })
        })
        .transpose()
    }
    pub async fn issue_oidc_code(&self, input: OidcIssueCode<'_>) -> Result<bool, StoreError> {
        let g = input.grant;
        let now = input.now;
        if !hash_valid(input.code_hash)
            || g.expires_at <= now
            || g.expires_at > g.auth_time.saturating_add(1296000)
            || g.claims.to_string().len() > 16384
        {
            return Ok(false);
        }
        let mut tx = self.backend.begin().await?;
        q("UPDATE identity_serialization SET id=id WHERE id=1")
            .execute_tx(&mut tx)
            .await?;
        let valid=q("SELECT sp.principal_id,sp.auth_time FROM session_principal sp JOIN identity_principal p ON p.id=sp.principal_id AND p.state='active' JOIN identity_binding b ON b.id=sp.binding_id AND b.principal_id=sp.principal_id AND b.retired_at IS NULL JOIN oidc_client c ON c.client_id=?2 AND c.enabled=1 AND c.revision=?3 JOIN oidc_provider_config cfg ON cfg.id=1 AND cfg.enabled=1 AND cfg.revision=?4 WHERE sp.session_id=?1 AND sp.principal_id=?5").bind(&g.session_id).bind(&g.client_id).bind(&g.client_revision).bind(&g.provider_revision).bind(&g.principal_id).fetch_all_tx(&mut tx).await?;
        if valid
            .first()
            .is_none_or(|r| r.get_i64("auth_time") != g.auth_time)
        {
            return Ok(false);
        }
        if q("UPDATE oidc_authorization SET consumed_at=?6 WHERE request_hash=?1 AND session_id=?2 AND client_id=?3 AND client_revision=?4 AND provider_revision=?5 AND consumed_at IS NULL AND expires_at>?6").bind(input.request_hash).bind(&g.session_id).bind(&g.client_id).bind(&g.client_revision).bind(&g.provider_revision).bind(now).execute_tx(&mut tx).await?!=1{return Ok(false)}
        q("DELETE FROM oidc_grant WHERE expires_at<=?1 OR revoked_at<=?2")
            .bind(now)
            .bind(now - 300)
            .execute_tx(&mut tx)
            .await?;
        let count = q("SELECT COUNT(*) AS n FROM oidc_grant WHERE principal_id=?1")
            .bind(&g.principal_id)
            .fetch_all_tx(&mut tx)
            .await?;
        if count[0].get_i64("n") >= 256 {
            return Ok(false);
        }
        q("UPDATE oidc_browser_flow SET consumed_at=?2 WHERE flow_hash=?1")
            .bind(input.request_hash)
            .bind(now)
            .execute_tx(&mut tx)
            .await?;
        let scopes = serde_json::to_string(&g.scopes)
            .map_err(|_| StoreError::Corrupt("invalid OIDC scopes".into()))?;
        q("INSERT INTO oidc_grant(id,client_id,client_revision,provider_revision,principal_id,session_id,claims_json,scopes_json,nonce,auth_time,created_at,expires_at) VALUES(?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11,?12)").bind(&g.id).bind(&g.client_id).bind(&g.client_revision).bind(&g.provider_revision).bind(&g.principal_id).bind(&g.session_id).bind(g.claims.to_string()).bind(&scopes).bind(g.nonce.as_deref()).bind(g.auth_time).bind(now).bind(g.expires_at).execute_tx(&mut tx).await?;
        q("INSERT INTO oidc_code(code_hash,grant_id,redirect_uri,pkce_challenge,expires_at) VALUES(?1,?2,?3,?4,?5)").bind(input.code_hash).bind(&g.id).bind(input.redirect_uri).bind(input.pkce_challenge).bind(now+60).execute_tx(&mut tx).await?;
        q("INSERT INTO oidc_consent(principal_id,client_id,client_revision,scopes_json,updated_at) VALUES(?1,?2,?3,?4,?5) ON CONFLICT(principal_id,client_id) DO UPDATE SET client_revision=excluded.client_revision,scopes_json=excluded.scopes_json,updated_at=excluded.updated_at").bind(&g.principal_id).bind(&g.client_id).bind(&g.client_revision).bind(&scopes).bind(now).execute_tx(&mut tx).await?;
        tx.commit().await?;
        Ok(true)
    }
    pub async fn redeem_oidc_token(
        &self,
        input: OidcRedeem<'_>,
    ) -> Result<OidcRedemption, StoreError> {
        let now = input.now;
        if !hash_valid(input.presented_hash)
            || !hash_valid(input.access_hash)
            || input.refresh_hash.is_some_and(|h| !hash_valid(h))
        {
            return Ok(OidcRedemption::Invalid);
        }
        let mut tx = self.backend.begin().await?;
        q("UPDATE identity_serialization SET id=id WHERE id=1")
            .execute_tx(&mut tx)
            .await?;
        let sql = match input.kind {
            OidcRedeemKind::Code { .. } => "SELECT * FROM oidc_code WHERE code_hash=?1",
            OidcRedeemKind::Refresh => {
                "SELECT * FROM oidc_token WHERE token_hash=?1 AND kind='refresh'"
            }
        };
        let rows = q(sql)
            .bind(input.presented_hash)
            .fetch_all_tx(&mut tx)
            .await?;
        let Some(token) = rows.first() else {
            return Ok(OidcRedemption::Invalid);
        };
        let rows = q(LIVE_GRANT)
            .bind(token.get_string("grant_id"))
            .bind(now)
            .fetch_all_tx(&mut tx)
            .await?;
        let Some(row) = rows.first() else {
            return Ok(OidcRedemption::Invalid);
        };
        let mut g = grant(row)?;
        if g.client_id != input.client_id || g.client_revision != input.client_revision {
            return Ok(OidcRedemption::Invalid);
        }
        if let OidcRedeemKind::Code {
            redirect_uri,
            pkce_challenge,
        } = input.kind
            && (token.get_string("redirect_uri") != redirect_uri
                || token.get_string("pkce_challenge") != pkce_challenge)
        {
            return Ok(OidcRedemption::Invalid);
        }
        if token.get_opt_i64("consumed_at").is_some() {
            q("UPDATE oidc_grant SET revoked_at=?2 WHERE id=?1")
                .bind(&g.id)
                .bind(now)
                .execute_tx(&mut tx)
                .await?;
            tx.commit().await?;
            return Ok(OidcRedemption::Replay);
        }
        if token.get_i64("expires_at") <= now {
            return Ok(OidcRedemption::Invalid);
        }
        if let Some(scope) = input.scope {
            if !matches!(input.kind, OidcRedeemKind::Refresh)
                || scope.is_empty()
                || scope.len() > 4
                || scope
                    .iter()
                    .collect::<std::collections::BTreeSet<_>>()
                    .len()
                    != scope.len()
                || scope.iter().any(|value| !g.scopes.contains(value))
            {
                return Ok(OidcRedemption::InvalidScope);
            }
            g.scopes = scope.to_vec();
            q("UPDATE oidc_grant SET scopes_json=?2 WHERE id=?1")
                .bind(&g.id)
                .bind(
                    serde_json::to_string(&g.scopes)
                        .map_err(|_| StoreError::Corrupt("invalid OIDC scopes".into()))?,
                )
                .execute_tx(&mut tx)
                .await?;
        }
        // Access credentials expire quickly; refresh replay tombstones must
        // remain until the family expires. Bound that history without silently
        // forgetting replay evidence: exhaustion revokes the whole family.
        q("DELETE FROM oidc_token WHERE token_hash IN (SELECT token_hash FROM oidc_token WHERE kind='access' AND expires_at<=?1 ORDER BY expires_at LIMIT 256)").bind(now).execute_tx(&mut tx).await?;
        let retained =
            q("SELECT COUNT(*) AS n FROM oidc_token WHERE grant_id=?1 AND kind='refresh'")
                .bind(&g.id)
                .fetch_all_tx(&mut tx)
                .await?;
        if retained[0].get_i64("n") >= 8192 {
            q("UPDATE oidc_grant SET revoked_at=?2 WHERE id=?1")
                .bind(&g.id)
                .bind(now)
                .execute_tx(&mut tx)
                .await?;
            tx.commit().await?;
            return Ok(OidcRedemption::Invalid);
        }
        let sql = match input.kind {
            OidcRedeemKind::Code { .. } => {
                "UPDATE oidc_code SET consumed_at=?2 WHERE code_hash=?1 AND consumed_at IS NULL"
            }
            OidcRedeemKind::Refresh => {
                "UPDATE oidc_token SET consumed_at=?2 WHERE token_hash=?1 AND consumed_at IS NULL"
            }
        };
        if q(sql)
            .bind(input.presented_hash)
            .bind(now)
            .execute_tx(&mut tx)
            .await?
            != 1
        {
            return Ok(OidcRedemption::Invalid);
        }
        q("INSERT INTO oidc_token(token_hash,grant_id,kind,expires_at) VALUES(?1,?2,'access',?3)")
            .bind(input.access_hash)
            .bind(&g.id)
            .bind((now + 300).min(g.expires_at))
            .execute_tx(&mut tx)
            .await?;
        if let Some(refresh) = input.refresh_hash
            && g.scopes.iter().any(|s| s == "offline_access")
        {
            q("INSERT INTO oidc_token(token_hash,grant_id,kind,expires_at) VALUES(?1,?2,'refresh',?3)").bind(refresh).bind(&g.id).bind(g.expires_at).execute_tx(&mut tx).await?;
        }
        tx.commit().await?;
        Ok(OidcRedemption::Issued(Box::new(g)))
    }
    pub async fn oidc_access_grant(
        &self,
        hash: &str,
        now: i64,
    ) -> Result<Option<OidcGrant>, StoreError> {
        let row=q("SELECT grant_id FROM oidc_token WHERE token_hash=?1 AND kind='access' AND consumed_at IS NULL AND expires_at>?2").bind(hash).bind(now).fetch_optional(&self.backend).await?;
        let Some(row) = row else { return Ok(None) };
        q(LIVE_GRANT)
            .bind(row.get_string("grant_id"))
            .bind(now)
            .fetch_optional(&self.backend)
            .await?
            .as_ref()
            .map(grant)
            .transpose()
    }
    pub async fn revoke_oidc_token(
        &self,
        hash: &str,
        client_id: &str,
        now: i64,
    ) -> Result<(), StoreError> {
        let mut tx = self.backend.begin().await?;
        q("UPDATE identity_serialization SET id=id WHERE id=1")
            .execute_tx(&mut tx)
            .await?;
        q("UPDATE oidc_grant SET revoked_at=?3 WHERE client_id=?2 AND id IN (SELECT grant_id FROM oidc_token WHERE token_hash=?1)").bind(hash).bind(client_id).bind(now).execute_tx(&mut tx).await?;
        tx.commit().await?;
        Ok(())
    }
    pub async fn revoke_oidc_consent(
        &self,
        principal: &str,
        client_id: &str,
        now: i64,
    ) -> Result<(), StoreError> {
        let mut tx = self.backend.begin().await?;
        q("UPDATE identity_serialization SET id=id WHERE id=1")
            .execute_tx(&mut tx)
            .await?;
        q("UPDATE oidc_grant SET revoked_at=?3 WHERE principal_id=?1 AND client_id=?2")
            .bind(principal)
            .bind(client_id)
            .bind(now)
            .execute_tx(&mut tx)
            .await?;
        q("DELETE FROM oidc_consent WHERE principal_id=?1 AND client_id=?2")
            .bind(principal)
            .bind(client_id)
            .execute_tx(&mut tx)
            .await?;
        tx.commit().await?;
        Ok(())
    }
}
