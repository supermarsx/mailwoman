//! Security-centre repositories over the existing 0033 schema.
use crate::backend::{Row, q};
use crate::{Store, StoreError};
use serde::Serialize;
use sha2::{Digest, Sha256};

#[derive(Debug, Clone, Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SessionDevice {
    pub user_agent: String,
    pub ip: String,
    pub client_type: String,
    pub device_label: String,
    pub client_version: String,
}
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SecuritySession {
    pub handle: String,
    pub account_id: String,
    pub username: String,
    pub created_at: String,
    pub last_seen: String,
    pub last_activity: String,
    #[serde(flatten)]
    pub device: SessionDevice,
}
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SigninEvent {
    pub id: String,
    pub account_id: Option<String>,
    pub username_attempted: String,
    pub ts: i64,
    pub result: String,
    pub method: String,
    pub ip: String,
    pub user_agent: String,
    pub geo_country: Option<String>,
    pub reason: String,
}
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RememberedDevice {
    pub id: String,
    pub account_id: String,
    pub label: String,
    pub created_at: i64,
    pub expires_at: i64,
    pub last_used_at: i64,
}
fn device_row(row: &Row) -> RememberedDevice {
    RememberedDevice {
        id: row.get_string("id"),
        account_id: row.get_string("account_id"),
        label: row.get_string("label"),
        created_at: row.get_i64("created_at"),
        expires_at: row.get_i64("expires_at"),
        last_used_at: row.get_i64("last_used_at"),
    }
}
impl Store {
    pub async fn security_posture_counts(
        &self,
        stale_before: &str,
    ) -> Result<serde_json::Value, StoreError> {
        let coverage=q("WITH known AS (SELECT id AS account_id FROM accounts UNION SELECT account_id FROM sessions UNION SELECT account_id FROM signin_identity) SELECT COUNT(*) AS total, COALESCE(SUM(CASE WHEN EXISTS(SELECT 1 FROM totp_secrets t WHERE t.account_id=known.account_id AND t.confirmed=1) OR EXISTS(SELECT 1 FROM webauthn_credentials w WHERE w.account_id=known.account_id) THEN 1 ELSE 0 END),0) AS enrolled FROM known").fetch_optional(&self.backend).await?.ok_or(StoreError::NotFound)?;
        let admins=q("SELECT COUNT(*) AS n FROM admin_account a WHERE disabled=0 AND NOT EXISTS(SELECT 1 FROM admin_totp t WHERE t.account_id=a.id) AND NOT EXISTS(SELECT 1 FROM admin_webauthn_credentials w WHERE w.account_id=a.id)").fetch_optional(&self.backend).await?.ok_or(StoreError::NotFound)?.get_i64("n");
        let keys=q("SELECT COUNT(*) AS n FROM api_keys WHERE revoked_at IS NULL AND COALESCE(last_used_at,created_at)<?1").bind(stale_before).fetch_optional(&self.backend).await?.ok_or(StoreError::NotFound)?.get_i64("n");
        Ok(
            serde_json::json!({"knownAccounts":coverage.get_i64("total"),"accountsWithFactor":coverage.get_i64("enrolled"),"adminsWithoutFactor":admins,"staleApiKeys":keys}),
        )
    }
    pub async fn security_account_for_address(
        &self,
        address: &str,
    ) -> Result<Option<String>, StoreError> {
        Ok(q("SELECT account_id FROM signin_identity WHERE lower(address)=?1 ORDER BY seen_at DESC,event_id DESC,account_id DESC LIMIT 1").bind(address.to_ascii_lowercase()).fetch_optional(&self.backend).await?.map(|row|row.get_string("account_id")))
    }
    pub async fn security_account_address(
        &self,
        account: &str,
    ) -> Result<Option<String>, StoreError> {
        Ok(
            q("SELECT address AS username_attempted FROM signin_identity WHERE account_id=?1")
                .bind(account)
                .fetch_optional(&self.backend)
                .await?
                .map(|row| row.get_string("username_attempted")),
        )
    }
    /// Preserve the original authentication lifetime and device while rotating a cookie.
    /// A concurrently revoked old session cannot be resurrected by the replacement.
    pub async fn rotate_session_security(&self, old: &str, new: &str) -> Result<bool, StoreError> {
        if old == new {
            return Ok(false);
        }
        let mut tx = self.backend.begin().await?;
        // Same order as principal retirement: identity lock, then session lock.
        // Otherwise a PostgreSQL DELETE snapshot can miss a concurrently copied
        // replacement session after waiting for the old session's row lock.
        q("UPDATE identity_serialization SET id=id WHERE id=1")
            .execute_tx(&mut tx)
            .await?;
        // Serialize against another rotation/revocation of this same old session.
        if q("UPDATE sessions SET last_seen=last_seen WHERE id=?1 AND linked_internal=0")
            .bind(old)
            .execute_tx(&mut tx)
            .await?
            == 0
        {
            return Ok(false);
        }
        let changed = q("UPDATE sessions SET (created_at,last_seen,activity_at,user_agent,ip,client_type,device_label,client_version,authentication_method)=(SELECT created_at,last_seen,activity_at,user_agent,ip,client_type,device_label,client_version,authentication_method FROM sessions WHERE id=?1) WHERE id=?2 AND linked_internal=0 AND account_id=(SELECT account_id FROM sessions WHERE id=?1)")
            .bind(old).bind(new).execute_tx(&mut tx).await?;
        if changed != 1 {
            return Ok(false);
        }
        // A rotation cannot replace an independently authenticated identity.
        if !q("SELECT session_id FROM session_principal WHERE session_id=?1")
            .bind(new)
            .fetch_all_tx(&mut tx)
            .await?
            .is_empty()
        {
            return Ok(false);
        }
        q("INSERT INTO session_principal(session_id,principal_id,binding_id,primary_account_id,auth_time,authentication_method) SELECT ?2,principal_id,binding_id,primary_account_id,auth_time,authentication_method FROM session_principal WHERE session_id=?1")
            .bind(old).bind(new).execute_tx(&mut tx).await?;
        q("DELETE FROM sessions WHERE id=?1")
            .bind(old)
            .execute_tx(&mut tx)
            .await?;
        let hash: String = Sha256::digest(old.as_bytes())
            .iter()
            .map(|b| format!("{b:02x}"))
            .collect();
        q("DELETE FROM native_sessions WHERE token_hash=?1")
            .bind(hash)
            .execute_tx(&mut tx)
            .await?;
        tx.commit().await?;
        Ok(true)
    }
    pub async fn set_session_authentication(
        &self,
        id: &str,
        method: &str,
    ) -> Result<(), StoreError> {
        q("UPDATE sessions SET authentication_method=?2 WHERE id=?1")
            .bind(id)
            .bind(method)
            .execute(&self.backend)
            .await?;
        Ok(())
    }
    pub async fn session_authentication(&self, id: &str) -> Result<String, StoreError> {
        Ok(q("SELECT authentication_method FROM sessions WHERE id=?1")
            .bind(id)
            .fetch_optional(&self.backend)
            .await?
            .map(|r| r.get_string("authentication_method"))
            .unwrap_or_default())
    }
    pub async fn set_passkey_discoverable(
        &self,
        credential: &str,
        account: &str,
        enabled: bool,
    ) -> Result<(), StoreError> {
        q("UPDATE webauthn_credentials SET discoverable=?3 WHERE credential_id=?1 AND account_id=?2").bind(credential).bind(account).bind(i64::from(enabled)).execute(&self.backend).await?;
        Ok(())
    }
    pub async fn passkey_discoverable(&self, credential: &str) -> Result<bool, StoreError> {
        Ok(
            q("SELECT discoverable FROM webauthn_credentials WHERE credential_id=?1")
                .bind(credential)
                .fetch_optional(&self.backend)
                .await?
                .is_some_and(|r| r.get_i64("discoverable") != 0),
        )
    }

    pub async fn session_clock(&self, id: &str) -> Result<Option<(String, String)>, StoreError> {
        Ok(q("SELECT created_at,last_seen FROM sessions WHERE id=?1")
            .bind(id)
            .fetch_optional(&self.backend)
            .await?
            .map(|r| (r.get_string("created_at"), r.get_string("last_seen"))))
    }

    /// Original sign-in and genuine user activity, independent of background traffic.
    pub async fn session_activity_clock(
        &self,
        id: &str,
    ) -> Result<Option<(String, String)>, StoreError> {
        Ok(q("SELECT created_at,COALESCE(activity_at,created_at) AS activity_at FROM sessions WHERE id=?1")
            .bind(id).fetch_optional(&self.backend).await?
            .map(|row| (row.get_string("created_at"), row.get_string("activity_at"))))
    }

    /// Only an already-authenticated foreground activity request may call this.
    /// UPDATE never recreates a concurrently revoked or rotated credential.
    pub async fn record_session_activity(&self, id: &str, at: &str) -> Result<bool, StoreError> {
        Ok(q("UPDATE sessions SET activity_at=?2 WHERE id=?1")
            .bind(id)
            .bind(at)
            .execute(&self.backend)
            .await?
            == 1)
    }

    /// Persist device metadata and cap concurrent sessions under an account write
    /// lock. The current login always survives; revoked native markers go too.
    pub async fn record_session_device(
        &self,
        id: &str,
        account: &str,
        device: &SessionDevice,
        max_sessions: usize,
    ) -> Result<Vec<String>, StoreError> {
        let mut tx = self.backend.begin().await?;
        let key = format!("sessionWriteLock:{account}");
        q("INSERT INTO settings (key,value) VALUES (?1,'0') ON CONFLICT(key) DO NOTHING")
            .bind(&key)
            .execute_tx(&mut tx)
            .await?;
        q("UPDATE settings SET value=value WHERE key=?1")
            .bind(&key)
            .execute_tx(&mut tx)
            .await?;
        let updated = q("UPDATE sessions SET user_agent=?3,ip=?4,client_type=?5,device_label=?6,client_version=?7 WHERE id=?1 AND account_id=?2")
            .bind(id).bind(account).bind(&device.user_agent).bind(&device.ip).bind(&device.client_type).bind(&device.device_label).bind(&device.client_version).execute_tx(&mut tx).await?;
        if updated == 0 {
            return Err(StoreError::NotFound);
        }
        let rows = q("SELECT id FROM sessions WHERE account_id=?1 AND linked_internal<>2 ORDER BY CASE WHEN id=?2 THEN 0 ELSE 1 END, created_at DESC,id DESC")
            .bind(account).bind(id).fetch_all_tx(&mut tx).await?;
        let evicted: Vec<_> = rows
            .iter()
            .skip(max_sessions.clamp(1, 100))
            .map(|r| r.get_string("id"))
            .collect();
        for old in &evicted {
            q("DELETE FROM sessions WHERE id=?1 AND account_id=?2")
                .bind(old)
                .bind(account)
                .execute_tx(&mut tx)
                .await?;
            let hash: String = Sha256::digest(old.as_bytes())
                .iter()
                .map(|b| format!("{b:02x}"))
                .collect();
            q("DELETE FROM native_sessions WHERE token_hash=?1")
                .bind(hash)
                .execute_tx(&mut tx)
                .await?;
        }
        tx.commit().await?;
        Ok(evicted)
    }
    pub async fn list_security_sessions(
        &self,
        account: Option<&str>,
        limit: u32,
    ) -> Result<Vec<SecuritySession>, StoreError> {
        let rows=q("SELECT id,account_id,username,created_at,last_seen,COALESCE(activity_at,created_at) AS last_activity,user_agent,ip,client_type,device_label,client_version FROM sessions WHERE linked_internal<>2 AND (?1 IS NULL OR account_id=?1) ORDER BY last_seen DESC,id LIMIT ?2")
            .bind(account).bind(i64::from(limit.clamp(1,1000))).fetch_all(&self.backend).await?;
        Ok(rows
            .iter()
            .map(|r| SecuritySession {
                handle: crate::session_handle(&r.get_string("id")),
                account_id: r.get_string("account_id"),
                username: r.get_string("username"),
                created_at: r.get_string("created_at"),
                last_seen: r.get_string("last_seen"),
                last_activity: r.get_string("last_activity"),
                device: SessionDevice {
                    user_agent: r.get_opt_string("user_agent").unwrap_or_default(),
                    ip: r.get_opt_string("ip").unwrap_or_default(),
                    client_type: r
                        .get_opt_string("client_type")
                        .unwrap_or_else(|| "browser".into()),
                    device_label: r.get_opt_string("device_label").unwrap_or_default(),
                    client_version: r.get_opt_string("client_version").unwrap_or_default(),
                },
            })
            .collect())
    }
    pub async fn record_signin_event(&self, event: &SigninEvent) -> Result<(), StoreError> {
        self.record_auth_event(
            event,
            crate::AuthSurface::Webmail,
            None,
            crate::AuthLogLimits::default(),
        )
        .await
    }
    pub async fn signin_history(
        &self,
        account: &str,
        before: i64,
        result: Option<&str>,
        limit: u32,
    ) -> Result<Vec<SigninEvent>, StoreError> {
        let rows=q("SELECT id,account_id,username_attempted,ts,result,method,ip,user_agent,geo_country,reason FROM signin_event WHERE surface='webmail' AND account_id=?1 AND ts<=?2 AND (?3 IS NULL OR result=?3) ORDER BY ts DESC,id DESC LIMIT ?4")
            .bind(account).bind(before).bind(result).bind(i64::from(limit.clamp(1,500))).fetch_all(&self.backend).await?;
        Ok(rows
            .iter()
            .map(|r| SigninEvent {
                id: r.get_string("id"),
                account_id: r.get_opt_string("account_id"),
                username_attempted: r.get_string("username_attempted"),
                ts: r.get_i64("ts"),
                result: r.get_string("result"),
                method: r.get_string("method"),
                ip: r.get_string("ip"),
                user_agent: r.get_string("user_agent"),
                geo_country: r.get_opt_string("geo_country"),
                reason: r.get_string("reason"),
            })
            .collect())
    }
    pub async fn put_remembered_device(
        &self,
        device: &RememberedDevice,
        token_hash: &str,
    ) -> Result<(), StoreError> {
        q("INSERT INTO remembered_device(id,account_id,token_hash,label,created_at,expires_at,last_used_at) VALUES(?1,?2,?3,?4,?5,?6,?7)")
            .bind(&device.id).bind(&device.account_id).bind(token_hash).bind(&device.label).bind(device.created_at).bind(device.expires_at).bind(device.last_used_at).execute(&self.backend).await?;
        Ok(())
    }
    pub async fn remembered_devices(
        &self,
        account: &str,
        now: i64,
    ) -> Result<Vec<RememberedDevice>, StoreError> {
        let rows=q("SELECT id,account_id,label,created_at,expires_at,last_used_at FROM remembered_device WHERE account_id=?1 AND expires_at>?2 ORDER BY last_used_at DESC LIMIT 100")
            .bind(account).bind(now).fetch_all(&self.backend).await?;
        Ok(rows.iter().map(device_row).collect())
    }
    pub async fn use_remembered_device(
        &self,
        account: &str,
        token_hash: &str,
        now: i64,
    ) -> Result<bool, StoreError> {
        Ok(q("UPDATE remembered_device SET last_used_at=?3 WHERE account_id=?1 AND token_hash=?2 AND expires_at>?3")
            .bind(account).bind(token_hash).bind(now).execute(&self.backend).await? != 0)
    }
    pub async fn revoke_remembered_device(
        &self,
        account: &str,
        id: &str,
    ) -> Result<bool, StoreError> {
        Ok(
            q("DELETE FROM remembered_device WHERE account_id=?1 AND id=?2")
                .bind(account)
                .bind(id)
                .execute(&self.backend)
                .await?
                != 0,
        )
    }
    pub async fn account_activity(
        &self,
        account: &str,
        limit: u32,
    ) -> Result<Vec<crate::AuditRow>, StoreError> {
        let target = format!("account:{account}");
        let rows=q("SELECT id,ts,actor,actor_kind,action,target,detail_json,ip FROM audit_log WHERE actor=?1 OR target=?2 ORDER BY ts DESC,id DESC LIMIT ?3")
            .bind(account).bind(target).bind(i64::from(limit.clamp(1,500))).fetch_all(&self.backend).await?;
        Ok(rows
            .iter()
            .map(|r| crate::AuditRow {
                id: r.get_string("id"),
                ts: r.get_string("ts"),
                actor: r.get_string("actor"),
                actor_kind: r.get_string("actor_kind"),
                action: r.get_string("action"),
                target: r.get_opt_string("target"),
                detail_json: r.get_string("detail_json"),
                ip: r.get_opt_string("ip"),
            })
            .collect())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Credentials, ServerKey};
    #[tokio::test]
    async fn posture_counts_password_and_environment_admin_factors_accurately() {
        let store = Store::open_in_memory(ServerKey::generate()).await.unwrap();
        for (id, source, password, disabled) in [
            ("db", "db", Some("hash"), 0_i64),
            ("env", "env-bootstrap", None, 0),
            ("disabled", "db", Some("hash"), 1),
        ] {
            q("INSERT INTO admin_account(id,username,display_name,password_hash,disabled,source,created_at,updated_at) VALUES(?1,?1,?1,?2,?3,?4,0,0)").bind(id).bind(password).bind(disabled).bind(source).execute(&store.backend).await.unwrap();
        }
        assert_eq!(
            store
                .security_posture_counts("2000-01-01T00:00:00Z")
                .await
                .unwrap()["adminsWithoutFactor"],
            2
        );
        q("INSERT INTO admin_webauthn_credentials(credential_id,account_id,cose_public_key,sign_count,transports,label,created_at,backup_eligible,backup_state,uv_initialized) VALUES('key','db',?1,0,'[]','fixture','2026-01-01',0,0,1)").bind(vec![1u8]).execute(&store.backend).await.unwrap();
        assert_eq!(
            store
                .security_posture_counts("2000-01-01T00:00:00Z")
                .await
                .unwrap()["adminsWithoutFactor"],
            1
        );
        q("INSERT INTO admin_totp(account_id,sealed_secret,last_step,enrolled_at) VALUES('env',?1,0,0)").bind(vec![1u8]).execute(&store.backend).await.unwrap();
        assert_eq!(
            store
                .security_posture_counts("2000-01-01T00:00:00Z")
                .await
                .unwrap()["adminsWithoutFactor"],
            0
        );
    }
    #[tokio::test]
    async fn activity_migration_keeps_old_baseline_and_rotation_cannot_extend_it() {
        let pool = sqlx::sqlite::SqlitePoolOptions::new()
            .max_connections(1)
            .connect("sqlite::memory:")
            .await
            .unwrap();
        sqlx::raw_sql("CREATE TABLE sessions(id TEXT,created_at TEXT,last_seen TEXT); INSERT INTO sessions VALUES('old','2020-01-01T00:00:00Z','2020-01-02T00:00:00Z');").execute(&pool).await.unwrap();
        sqlx::raw_sql(include_str!(
            "../migrations/0045_mailbox_session_activity.sql"
        ))
        .execute(&pool)
        .await
        .unwrap();
        let baseline: String =
            sqlx::query_scalar("SELECT activity_at FROM sessions WHERE id='old'")
                .fetch_one(&pool)
                .await
                .unwrap();
        assert_eq!(baseline, "2020-01-02T00:00:00Z");

        let store = Store::open_in_memory(ServerKey::generate()).await.unwrap();
        let creds = Credentials {
            username: "alice".into(),
            password: "fixture".into(),
        };
        let old = store
            .create_session("alice", "alice", "engine", "engine", &creds)
            .await
            .unwrap();
        q("UPDATE sessions SET created_at='2020-01-01T00:00:00Z',activity_at='2020-01-02T00:00:00Z' WHERE id=?1").bind(&old).execute(&store.backend).await.unwrap();
        store.touch_session(&old).await.unwrap(); // background request telemetry
        assert_eq!(
            store.session_activity_clock(&old).await.unwrap().unwrap(),
            ("2020-01-01T00:00:00Z".into(), baseline)
        );
        assert!(
            store
                .record_session_activity(&old, "2020-01-03T00:00:00Z")
                .await
                .unwrap()
        );
        let new = store
            .create_session("alice", "alice", "engine", "engine", &creds)
            .await
            .unwrap();
        assert!(store.rotate_session_security(&old, &new).await.unwrap());
        assert_eq!(
            store.session_activity_clock(&new).await.unwrap().unwrap(),
            ("2020-01-01T00:00:00Z".into(), "2020-01-03T00:00:00Z".into())
        );
        assert!(
            !store
                .record_session_activity(&old, "2020-01-04T00:00:00Z")
                .await
                .unwrap()
        );
        store.delete_session(&new).await.unwrap();
        assert!(
            !store
                .record_session_activity(&new, "2020-01-04T00:00:00Z")
                .await
                .unwrap()
        );
    }

    #[tokio::test]
    async fn upgrade_revokes_legacy_mail_sessions_but_preserves_accounts_and_admin_sessions() {
        let pool = sqlx::sqlite::SqlitePoolOptions::new()
            .max_connections(1)
            .connect("sqlite::memory:")
            .await
            .unwrap();
        let mut before = sqlx::migrate!("./migrations");
        before.migrations.to_mut().retain(|m| m.version < 36);
        before.run(&pool).await.unwrap();
        sqlx::query("INSERT INTO accounts(id,kind,host,port,tls,username,sealed_creds) VALUES('a','imap','example.org',993,'implicit','alice',X'01')").execute(&pool).await.unwrap();
        sqlx::query("INSERT INTO sessions(id,account_id,username,jmap_url,api_url,sealed_creds,created_at,last_seen) VALUES('old','a','alice','engine','engine',X'01','2026-01-01','2026-01-01')").execute(&pool).await.unwrap();
        sqlx::query("INSERT INTO native_sessions(token_hash,account_id,client_type,created_at,last_seen) VALUES('hash','a','native','2026-01-01','2026-01-01')").execute(&pool).await.unwrap();
        sqlx::query("INSERT INTO admin_sessions(token_hash,admin_id,created_at,last_seen,expires_at,absolute_expires_at) VALUES('admin-hash','root','2026-01-01','2026-01-01',2000000000,2000000000)").execute(&pool).await.unwrap();
        let store = Store::init_sqlite(pool, ServerKey::generate())
            .await
            .unwrap();
        for (query, expected) in [
            ("SELECT COUNT(*) AS n FROM sessions", 0),
            ("SELECT COUNT(*) AS n FROM native_sessions", 0),
            ("SELECT COUNT(*) AS n FROM accounts", 1),
            ("SELECT COUNT(*) AS n FROM admin_sessions", 1),
        ] {
            let row = q(query)
                .fetch_optional(&store.backend)
                .await
                .unwrap()
                .unwrap();
            assert_eq!(row.get_i64("n"), expected, "{query}");
        }
    }
    #[tokio::test]
    async fn rotation_preserves_security_lifetime_and_cannot_resurrect_revoked_session() {
        let store = Store::open_in_memory(ServerKey::generate()).await.unwrap();
        let creds = Credentials {
            username: "alice".into(),
            password: "service-password".into(),
        };
        let old = store
            .create_session("alice", "alice", "engine", "engine", &creds)
            .await
            .unwrap();
        let new = store
            .create_session("alice", "alice", "engine", "engine", &creds)
            .await
            .unwrap();
        q("UPDATE sessions SET created_at='2020-01-01T00:00:00Z',user_agent='browser',authentication_method='header-auth' WHERE id=?1").bind(&old).execute(&store.backend).await.unwrap();
        assert!(store.rotate_session_security(&old, &new).await.unwrap());
        assert!(store.get_session(&old).await.is_err());
        assert_eq!(
            store.session_authentication(&new).await.unwrap(),
            "header-auth"
        );
        let rows = store
            .list_security_sessions(Some("alice"), 10)
            .await
            .unwrap();
        assert_eq!(rows[0].created_at, "2020-01-01T00:00:00Z");
        assert_eq!(rows[0].device.user_agent, "browser");
        assert!(!store.rotate_session_security(&old, &new).await.unwrap());
    }
    #[tokio::test]
    async fn session_cap_keeps_new_login_and_metadata_never_returns_credentials() {
        let store = Store::open_in_memory(ServerKey::generate()).await.unwrap();
        let creds = Credentials {
            username: "alice".into(),
            password: "secret-password".into(),
        };
        let old = store
            .create_session("alice", "alice", "engine", "engine", &creds)
            .await
            .unwrap();
        let new = store
            .create_session("alice", "alice", "engine", "engine", &creds)
            .await
            .unwrap();
        let device = SessionDevice {
            client_version: "browser/1.2.3".into(),
            ..SessionDevice::default()
        };
        assert_eq!(
            store
                .record_session_device(&new, "alice", &device, 1)
                .await
                .unwrap(),
            vec![old.clone()]
        );
        assert!(matches!(
            store.get_session(&old).await,
            Err(StoreError::NotFound)
        ));
        let rows = store
            .list_security_sessions(Some("alice"), 10)
            .await
            .unwrap();
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].device.client_version, "browser/1.2.3");
        let json = serde_json::to_string(&rows).unwrap();
        assert!(!json.contains(&new));
        assert!(!json.contains("secret-password"));
        assert!(
            store
                .list_security_sessions(Some("bob"), 10)
                .await
                .unwrap()
                .is_empty()
        );
    }
    #[tokio::test]
    async fn remembered_tokens_are_scoped_expiring_and_not_in_listings() {
        let store = Store::open_in_memory(ServerKey::generate()).await.unwrap();
        let device = RememberedDevice {
            id: "d".into(),
            account_id: "alice".into(),
            label: "Laptop".into(),
            created_at: 10,
            expires_at: 20,
            last_used_at: 10,
        };
        store
            .put_remembered_device(&device, "hashed-only")
            .await
            .unwrap();
        assert!(
            !store
                .use_remembered_device("bob", "hashed-only", 11)
                .await
                .unwrap()
        );
        assert!(
            store
                .use_remembered_device("alice", "hashed-only", 11)
                .await
                .unwrap()
        );
        assert!(
            !store
                .use_remembered_device("alice", "hashed-only", 20)
                .await
                .unwrap()
        );
        assert!(
            !serde_json::to_string(&store.remembered_devices("alice", 11).await.unwrap())
                .unwrap()
                .contains("hashed-only")
        );
        assert!(!store.revoke_remembered_device("bob", "d").await.unwrap());
        assert!(store.revoke_remembered_device("alice", "d").await.unwrap());
    }
}
