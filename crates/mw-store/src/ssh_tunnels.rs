//! Deployment-local SSH authority; credentials are sealed and never part of metadata.
use crate::{
    Store, StoreError,
    backend::{Row, q},
    seal,
};

#[derive(Clone, Debug)]
pub struct SshTunnelRow {
    pub id: String,
    pub name: String,
    pub host: String,
    pub port: u16,
    pub username: String,
    pub public_key: String,
    pub key_algorithm: String,
    pub pins_json: String,
    pub targets_json: String,
    pub allow_private: bool,
    pub keepalive_seconds: u32,
    pub connect_timeout_ms: u32,
    pub max_channels: u32,
    pub enabled: bool,
    pub credential_configured: bool,
    pub auth_kind: Option<String>,
    pub revision: String,
    pub created_by: String,
    pub created_at: i64,
    pub updated_at: i64,
}

/// Deliberately neither Debug nor Serialize.
pub enum SshCredentialChange {
    Keep,
    Clear,
    Set {
        secret: String,
        kind: String,
        public_key: String,
        algorithm: String,
    },
}
pub enum SshTunnelMutation {
    Applied(Box<SshTunnelRow>),
    Missing,
    Conflict,
    CredentialRequired,
}
pub enum SshTunnelDelete {
    Deleted,
    Missing,
    Conflict,
    Bound,
}
const ALL: &str = "SELECT *,CAST(CASE WHEN length(sealed_private_key)>0 THEN 1 ELSE 0 END AS BIGINT) AS credential_configured FROM ssh_tunnel ORDER BY name,id";
const BY_ID: &str = "SELECT *,CAST(CASE WHEN length(sealed_private_key)>0 THEN 1 ELSE 0 END AS BIGINT) AS credential_configured FROM ssh_tunnel WHERE id=?1";
fn row(r: &Row) -> SshTunnelRow {
    SshTunnelRow {
        id: r.get_string("id"),
        name: r.get_string("name"),
        host: r.get_string("host"),
        port: u16::try_from(r.get_i64("port")).unwrap_or(0),
        username: r.get_string("username"),
        public_key: r.get_string("public_key"),
        key_algorithm: r.get_string("key_algorithm"),
        pins_json: r.get_string("host_key_pins_json"),
        targets_json: r.get_string("targets_json"),
        allow_private: r.get_i64("allow_private") != 0,
        keepalive_seconds: u32::try_from(r.get_i64("keepalive_secs")).unwrap_or(0),
        connect_timeout_ms: u32::try_from(r.get_i64("connect_timeout_ms")).unwrap_or(0),
        max_channels: u32::try_from(r.get_i64("max_channels")).unwrap_or(0),
        enabled: r.get_i64("enabled") != 0,
        credential_configured: r.get_i64("credential_configured") != 0,
        auth_kind: r.get_opt_string("auth_kind"),
        revision: r.get_string("revision"),
        created_by: r.get_string("created_by"),
        created_at: r.get_i64("created_at"),
        updated_at: r.get_i64("updated_at"),
    }
}
impl Store {
    pub async fn ssh_tunnels(&self) -> Result<Vec<SshTunnelRow>, StoreError> {
        Ok(q(ALL)
            .fetch_all(&self.backend)
            .await?
            .iter()
            .map(row)
            .collect())
    }
    pub async fn ssh_tunnel(&self, id: &str) -> Result<Option<SshTunnelRow>, StoreError> {
        Ok(q(BY_ID)
            .bind(id)
            .fetch_optional(&self.backend)
            .await?
            .as_ref()
            .map(row))
    }
    /// The metadata and secret come from one database snapshot, never two reads.
    pub async fn ssh_tunnel_authority(
        &self,
        id: &str,
    ) -> Result<Option<(SshTunnelRow, Option<String>)>, StoreError> {
        let Some(r) = q(BY_ID).bind(id).fetch_optional(&self.backend).await? else {
            return Ok(None);
        };
        let metadata = row(&r);
        let secret = if metadata.credential_configured {
            Some(
                String::from_utf8(self.key.open(&r.get_blob("sealed_private_key"))?)
                    .map_err(|_| seal::SealError::Decrypt)?,
            )
        } else {
            None
        };
        Ok(Some((metadata, secret)))
    }
    pub async fn put_ssh_tunnel(
        &self,
        desired: &SshTunnelRow,
        expected: Option<&str>,
        credential: SshCredentialChange,
    ) -> Result<SshTunnelMutation, StoreError> {
        let mut tx = self.backend.begin().await?;
        q("INSERT INTO settings(key,value) VALUES('ssh:authority-lock','0') ON CONFLICT(key) DO NOTHING").execute_tx(&mut tx).await?;
        q("UPDATE settings SET value=value WHERE key='ssh:authority-lock'")
            .execute_tx(&mut tx)
            .await?;
        let old = q(BY_ID)
            .bind(&desired.id)
            .fetch_all_tx(&mut tx)
            .await?
            .first()
            .map(row);
        match (expected, old.as_ref()) {
            (None, Some(_)) => return Ok(SshTunnelMutation::Conflict),
            (Some(_), None) => return Ok(SshTunnelMutation::Missing),
            (Some(expected), Some(old)) if expected != old.revision => {
                return Ok(SshTunnelMutation::Conflict);
            }
            _ => {}
        }
        let mut saved = desired.clone();
        saved.revision = seal::random_token();
        if let Some(old) = &old {
            saved.created_by = old.created_by.clone();
            saved.created_at = old.created_at;
        }
        let sealed = match credential {
            SshCredentialChange::Keep => {
                saved.credential_configured = old.as_ref().is_some_and(|r| r.credential_configured);
                saved.auth_kind = old.as_ref().and_then(|r| r.auth_kind.clone());
                saved.public_key = old
                    .as_ref()
                    .map(|r| r.public_key.clone())
                    .unwrap_or_default();
                saved.key_algorithm = old
                    .as_ref()
                    .map(|r| r.key_algorithm.clone())
                    .unwrap_or_default();
                None
            }
            SshCredentialChange::Clear => {
                saved.credential_configured = false;
                saved.auth_kind = None;
                saved.public_key.clear();
                saved.key_algorithm.clear();
                Some(Vec::new())
            }
            SshCredentialChange::Set {
                secret,
                kind,
                public_key,
                algorithm,
            } => {
                saved.credential_configured = true;
                saved.auth_kind = Some(kind);
                saved.public_key = public_key;
                saved.key_algorithm = algorithm;
                Some(self.key.seal(secret.as_bytes())?)
            }
        };
        if saved.enabled && !saved.credential_configured {
            return Ok(SshTunnelMutation::CredentialRequired);
        }
        q("INSERT INTO ssh_tunnel(id,name,host,port,username,sealed_private_key,public_key,key_algorithm,host_key_pins_json,keepalive_secs,connect_timeout_ms,max_channels,enabled,created_by,created_at,updated_at,targets_json,allow_private,auth_kind,revision) VALUES(?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11,?12,?13,?14,?15,?16,?17,?18,?19,?20) ON CONFLICT(id) DO UPDATE SET name=excluded.name,host=excluded.host,port=excluded.port,username=excluded.username,public_key=excluded.public_key,key_algorithm=excluded.key_algorithm,host_key_pins_json=excluded.host_key_pins_json,keepalive_secs=excluded.keepalive_secs,connect_timeout_ms=excluded.connect_timeout_ms,max_channels=excluded.max_channels,enabled=excluded.enabled,updated_at=excluded.updated_at,targets_json=excluded.targets_json,allow_private=excluded.allow_private,auth_kind=excluded.auth_kind,revision=excluded.revision")
            .bind(&saved.id).bind(&saved.name).bind(&saved.host).bind(i64::from(saved.port)).bind(&saved.username).bind(Vec::<u8>::new()).bind(&saved.public_key).bind(&saved.key_algorithm).bind(&saved.pins_json).bind(i64::from(saved.keepalive_seconds)).bind(i64::from(saved.connect_timeout_ms)).bind(i64::from(saved.max_channels)).bind(i64::from(saved.enabled)).bind(&saved.created_by).bind(saved.created_at).bind(saved.updated_at).bind(&saved.targets_json).bind(i64::from(saved.allow_private)).bind(saved.auth_kind.clone()).bind(&saved.revision).execute_tx(&mut tx).await?;
        if let Some(sealed) = sealed {
            q("UPDATE ssh_tunnel SET sealed_private_key=?2 WHERE id=?1")
                .bind(&saved.id)
                .bind(sealed)
                .execute_tx(&mut tx)
                .await?;
        }
        tx.commit().await?;
        Ok(SshTunnelMutation::Applied(Box::new(saved)))
    }
    pub async fn delete_ssh_tunnel(
        &self,
        id: &str,
        expected: &str,
    ) -> Result<SshTunnelDelete, StoreError> {
        let mut tx = self.backend.begin().await?;
        q("INSERT INTO settings(key,value) VALUES('ssh:authority-lock','0') ON CONFLICT(key) DO NOTHING").execute_tx(&mut tx).await?;
        q("UPDATE settings SET value=value WHERE key='ssh:authority-lock'")
            .execute_tx(&mut tx)
            .await?;
        let old = q("SELECT revision FROM ssh_tunnel WHERE id=?1")
            .bind(id)
            .fetch_opt_scalar_string_tx(&mut tx)
            .await?;
        match old {
            None => return Ok(SshTunnelDelete::Missing),
            Some(old) if old != expected => return Ok(SshTunnelDelete::Conflict),
            _ => {}
        }
        // Parse stored JSON instead of interpolating a JSON path or substring match.
        for r in q("SELECT config_json FROM hosting_connection")
            .fetch_all_tx(&mut tx)
            .await?
        {
            let config: serde_json::Value = serde_json::from_str(&r.get_string("config_json"))
                .map_err(|_| StoreError::Corrupt("invalid hosting authority".into()))?;
            if config
                .pointer("/connection/tunnelId")
                .and_then(|v| v.as_str())
                == Some(id)
            {
                return Ok(SshTunnelDelete::Bound);
            }
        }
        if !q("SELECT id FROM egress_proxy WHERE tunnel_id=?1")
            .bind(id)
            .fetch_all_tx(&mut tx)
            .await?
            .is_empty()
        {
            return Ok(SshTunnelDelete::Bound);
        }
        q("DELETE FROM ssh_tunnel WHERE id=?1")
            .bind(id)
            .execute_tx(&mut tx)
            .await?;
        tx.commit().await?;
        Ok(SshTunnelDelete::Deleted)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn applied(value: SshTunnelMutation) -> SshTunnelRow {
        match value {
            SshTunnelMutation::Applied(row) => *row,
            _ => panic!("write rejected"),
        }
    }
    async fn exercise(store: Store) {
        let id = format!("ssh-{}", rand::random::<u64>());
        let desired = SshTunnelRow {
            id: id.clone(),
            name: "Jump".into(),
            host: "jump.example.test".into(),
            port: 22,
            username: "operator".into(),
            public_key: String::new(),
            key_algorithm: String::new(),
            pins_json: "[]".into(),
            targets_json: "[]".into(),
            allow_private: false,
            keepalive_seconds: 30,
            connect_timeout_ms: 5000,
            max_channels: 4,
            enabled: true,
            credential_configured: false,
            auth_kind: None,
            revision: String::new(),
            created_by: "root".into(),
            created_at: 1,
            updated_at: 1,
        };
        assert!(matches!(
            store
                .put_ssh_tunnel(&desired, None, SshCredentialChange::Keep)
                .await
                .unwrap(),
            SshTunnelMutation::CredentialRequired
        ));
        let first = applied(
            store
                .put_ssh_tunnel(
                    &desired,
                    None,
                    SshCredentialChange::Set {
                        secret: "test-secret-password".into(),
                        kind: "password".into(),
                        public_key: String::new(),
                        algorithm: String::new(),
                    },
                )
                .await
                .unwrap(),
        );
        assert!(!format!("{first:?}").contains("test-secret-password"));
        assert_eq!(
            store
                .ssh_tunnel_authority(&id)
                .await
                .unwrap()
                .unwrap()
                .1
                .as_deref(),
            Some("test-secret-password")
        );
        let (a, b) = tokio::join!(
            store.put_ssh_tunnel(&desired, Some(&first.revision), SshCredentialChange::Keep),
            store.put_ssh_tunnel(&desired, Some(&first.revision), SshCredentialChange::Keep)
        );
        assert!(
            matches!(a.unwrap(), SshTunnelMutation::Applied(_))
                ^ matches!(b.unwrap(), SshTunnelMutation::Applied(_))
        );
        let current = store.ssh_tunnel(&id).await.unwrap().unwrap();
        assert!(matches!(
            store
                .put_ssh_tunnel(
                    &desired,
                    Some(&current.revision),
                    SshCredentialChange::Clear
                )
                .await
                .unwrap(),
            SshTunnelMutation::CredentialRequired
        ));
        assert!(
            store
                .ssh_tunnel(&id)
                .await
                .unwrap()
                .unwrap()
                .credential_configured
        );
        let panel = format!("panel-{id}");
        let config = serde_json::json!({"connection":{"kind":"ssh","tunnelId":id}}).to_string();
        let (delete, bind) = tokio::join!(
            store.delete_ssh_tunnel(&id, &current.revision),
            store.put_hosting_connection(
                &panel,
                &config,
                None,
                crate::HostingCredentialChange::Keep,
                2
            )
        );
        match (delete.unwrap(), bind.unwrap()) {
            (SshTunnelDelete::Deleted, crate::HostingMutation::Conflict) => {
                assert!(store.hosting_connection(&panel).await.unwrap().is_none())
            }
            (SshTunnelDelete::Bound, crate::HostingMutation::Applied(row)) => {
                assert!(store.ssh_tunnel(&id).await.unwrap().is_some());
                assert!(
                    store
                        .delete_hosting_connection(&panel, &row.revision)
                        .await
                        .unwrap()
                );
                assert!(matches!(
                    store
                        .delete_ssh_tunnel(&id, &current.revision)
                        .await
                        .unwrap(),
                    SshTunnelDelete::Deleted
                ));
            }
            _ => panic!("binding and deletion must serialize"),
        }
    }
    #[tokio::test]
    async fn ssh_authority_sqlite() {
        exercise(
            Store::open_in_memory(crate::ServerKey::generate())
                .await
                .unwrap(),
        )
        .await;
    }
    #[tokio::test]
    async fn ssh_authority_postgres() {
        if let Ok(url) = std::env::var("MW_TEST_PG") {
            exercise(
                Store::open(&url, crate::ServerKey::generate())
                    .await
                    .unwrap(),
            )
            .await;
        }
    }
}
