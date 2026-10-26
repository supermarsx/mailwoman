//! Verified provider enrollment. This repository is called only after upstream proof.
use crate::{Credentials, Store, StoreError, q};
use sha2::{Digest, Sha256};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ProviderAccount {
    pub account_id: String,
    pub endpoint_url: String,
    pub backend_username: String,
    pub login_address: String,
    pub domain_binding: String,
    pub ca_pem: Option<String>,
    pub revision: String,
}

/// Contains a password; deliberately has no Debug or Serialize implementation.
pub struct VerifiedEwsEnrollment<'a> {
    pub endpoint_url: &'a str,
    pub backend_username: &'a str,
    pub login_address: &'a str,
    pub domain_binding: &'a str,
    pub ca_pem: Option<&'a str>,
    pub password: &'a str,
}

fn projection(row: &crate::backend::Row) -> ProviderAccount {
    ProviderAccount {
        account_id: row.get_string("account_id"),
        endpoint_url: row.get_string("endpoint_url"),
        backend_username: row.get_string("backend_username"),
        login_address: row.get_string("login_address"),
        domain_binding: row.get_string("domain_binding"),
        ca_pem: row.get_opt_string("ca_pem"),
        revision: row.get_string("revision"),
    }
}

pub fn ews_authority(endpoint: &str) -> String {
    Sha256::digest(endpoint.as_bytes())
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect()
}

async fn policy_domain_tx(
    transaction: &mut crate::backend::Tx,
    address: &str,
) -> Result<String, StoreError> {
    let domain = address
        .rsplit_once('@')
        .map(|(_, domain)| domain.trim_end_matches('.'))
        .ok_or_else(|| StoreError::Corrupt("invalid provider login address".into()))?;
    let domain = url::Host::parse(domain)
        .map_err(|_| StoreError::Corrupt("invalid provider login domain".into()))?
        .to_string();
    Ok(q("SELECT domain FROM domain_alias WHERE alias=?1")
        .bind(&domain)
        .fetch_opt_scalar_string_tx(transaction)
        .await?
        .unwrap_or(domain))
}

impl Store {
    pub async fn provider_account(
        &self,
        account: &str,
    ) -> Result<Option<ProviderAccount>, StoreError> {
        Ok(q("SELECT account_id,endpoint_url,backend_username,login_address,domain_binding,ca_pem,revision FROM account_provider WHERE account_id=?1 AND provider='ews'")
            .bind(account).fetch_optional(&self.backend).await?.as_ref().map(projection))
    }

    pub async fn ews_account_for_login(
        &self,
        endpoint: &str,
        username: &str,
    ) -> Result<Option<ProviderAccount>, StoreError> {
        Ok(q("SELECT account_id,endpoint_url,backend_username,login_address,domain_binding,ca_pem,revision FROM account_provider WHERE provider='ews' AND endpoint_url=?1 AND backend_username=?2")
            .bind(endpoint).bind(username.to_ascii_lowercase()).fetch_optional(&self.backend).await?.as_ref().map(projection))
    }

    /// Exact endpoint and authenticated login define the realm. No email-only lookup.
    /// The transaction serializes first enrollment across replicas and preserves IDs/factors.
    pub async fn enroll_verified_ews(
        &self,
        verified: VerifiedEwsEnrollment<'_>,
    ) -> Result<ProviderAccount, StoreError> {
        let endpoint = url::Url::parse(verified.endpoint_url)
            .map_err(|_| StoreError::Corrupt("invalid provider endpoint".into()))?;
        if endpoint.scheme() != "https"
            || endpoint.host_str().is_none()
            || endpoint.as_str() != verified.endpoint_url
            || !endpoint.username().is_empty()
            || endpoint.password().is_some()
            || endpoint.query().is_some()
            || endpoint.fragment().is_some()
            || verified.endpoint_url.len() > 2048
            || verified.endpoint_url.chars().any(char::is_control)
            || verified.backend_username.is_empty()
            || verified.backend_username.len() > 512
            || verified
                .backend_username
                .chars()
                .any(|value| value.is_control() || value == ':')
            || verified.login_address.len() > 512
            || !verified.login_address.contains('@')
            || verified.login_address.chars().any(char::is_control)
            || verified.password.len() > 4096
            || verified.domain_binding.len() != 64
            || !verified
                .domain_binding
                .bytes()
                .all(|value| value.is_ascii_hexdigit())
            || verified
                .ca_pem
                .is_some_and(|value| value.len() > 65536 || value.contains("PRIVATE KEY"))
        {
            return Err(StoreError::Corrupt(
                "invalid verified provider enrollment".into(),
            ));
        }
        let backend_username = verified.backend_username.to_ascii_lowercase();
        let authority = ews_authority(verified.endpoint_url);
        let mut transaction = self.backend.begin().await?;
        q("INSERT INTO settings(key,value) VALUES('provider:enrollment-lock','0') ON CONFLICT(key) DO NOTHING").execute_tx(&mut transaction).await?;
        q("UPDATE settings SET value=value WHERE key='provider:enrollment-lock'")
            .execute_tx(&mut transaction)
            .await?;
        let existing = q("SELECT account_id,endpoint_url,backend_username,login_address,domain_binding,ca_pem,revision FROM account_provider WHERE provider='ews' AND endpoint_url=?1 AND backend_username=?2")
            .bind(verified.endpoint_url).bind(&backend_username).fetch_all_tx(&mut transaction).await?.pop();
        let previous = existing.as_ref().map(projection);
        if let Some(previous) = &previous {
            let original_domain =
                policy_domain_tx(&mut transaction, &previous.login_address).await?;
            let incoming_domain =
                policy_domain_tx(&mut transaction, verified.login_address).await?;
            if original_domain != incoming_domain {
                return Err(StoreError::Corrupt(
                    "provider account belongs to another configured domain".into(),
                ));
            }
        }
        let account = previous
            .as_ref()
            .map(|row| row.account_id.clone())
            .unwrap_or_else(crate::seal::random_token);
        let login_address = previous
            .as_ref()
            .map(|row| row.login_address.as_str())
            .unwrap_or(verified.login_address);
        let revision = crate::seal::random_token();
        let credentials = Credentials {
            username: verified.backend_username.into(),
            password: verified.password.into(),
        };
        let sealed = self.key.seal_bound(
            &crate::encode_creds(&credentials),
            "provider-ews-credential-v1",
            &[
                &account,
                verified.endpoint_url,
                &backend_username,
                verified.domain_binding,
            ],
        )?;
        // Legacy physical kind is a storage compatibility detail. It contains no usable password.
        // Native readers project the effective backend and socket lookup excludes this row.
        let placeholder = self.key.seal(&crate::encode_creds(&Credentials {
            username: login_address.into(),
            password: String::new(),
        }))?;
        q("UPDATE accounts SET sync_policy_json=sync_policy_json WHERE id=?1")
            .bind(&account)
            .execute_tx(&mut transaction)
            .await?;
        let old_policy = q("SELECT sync_policy_json FROM accounts WHERE id=?1")
            .bind(&account)
            .fetch_all_tx(&mut transaction)
            .await?
            .pop();
        let mut policy: serde_json::Value = match old_policy {
            Some(row) => serde_json::from_str(&row.get_string("sync_policy_json"))
                .map_err(|_| StoreError::Corrupt("invalid provider account policy".into()))?,
            None => serde_json::json!({}),
        };
        let object = policy
            .as_object_mut()
            .ok_or_else(|| StoreError::Corrupt("invalid provider account policy".into()))?;
        object.insert("domain_binding".into(), verified.domain_binding.into());
        object.remove("credential_rejected");
        let policy = policy.to_string();
        q("INSERT INTO accounts(id,kind,host,port,tls,username,sealed_creds,sync_policy_json,backend_kind,backend_authority) VALUES(?1,'imap',?2,?3,'implicit',?4,?5,?6,'ews',?7) ON CONFLICT(id) DO UPDATE SET sealed_creds=excluded.sealed_creds,sync_policy_json=excluded.sync_policy_json")
            .bind(&account).bind(endpoint.host_str().unwrap_or_default()).bind(i64::from(endpoint.port_or_known_default().unwrap_or(443)))
            .bind(login_address).bind(placeholder).bind(policy).bind(&authority).execute_tx(&mut transaction).await?;
        let now = chrono::Utc::now().timestamp();
        q("INSERT INTO account_provider(account_id,provider,endpoint_url,backend_username,login_address,domain_binding,ca_pem,sealed_credential,revision,created_at,updated_at) VALUES(?1,'ews',?2,?3,?4,?5,?6,?7,?8,?9,?9) ON CONFLICT(account_id) DO UPDATE SET domain_binding=excluded.domain_binding,ca_pem=excluded.ca_pem,sealed_credential=excluded.sealed_credential,revision=excluded.revision,updated_at=excluded.updated_at")
            .bind(&account).bind(verified.endpoint_url).bind(&backend_username).bind(login_address).bind(verified.domain_binding)
            .bind(verified.ca_pem).bind(sealed).bind(&revision).bind(now).execute_tx(&mut transaction).await?;
        q("INSERT INTO bridge_accounts(account_id,bridge_id,oauth_ref,extra) VALUES(?1,'bridge-ews',NULL,'{}') ON CONFLICT(account_id) DO UPDATE SET bridge_id=excluded.bridge_id,oauth_ref=NULL,extra=excluded.extra")
            .bind(&account).execute_tx(&mut transaction).await?;
        transaction.commit().await?;
        Ok(ProviderAccount {
            account_id: account,
            endpoint_url: verified.endpoint_url.into(),
            backend_username,
            login_address: login_address.into(),
            domain_binding: verified.domain_binding.into(),
            ca_pem: verified.ca_pem.map(str::to_owned),
            revision,
        })
    }

    pub async fn provider_ews_credentials(
        &self,
        account: &str,
    ) -> Result<Option<Credentials>, StoreError> {
        let Some(row) = q("SELECT endpoint_url,backend_username,domain_binding,sealed_credential FROM account_provider WHERE account_id=?1 AND provider='ews'").bind(account).fetch_optional(&self.backend).await? else { return Ok(None) };
        let clear = self.key.open_bound(
            &row.get_blob("sealed_credential"),
            "provider-ews-credential-v1",
            &[
                account,
                &row.get_string("endpoint_url"),
                &row.get_string("backend_username"),
                &row.get_string("domain_binding"),
            ],
        )?;
        Ok(Some(crate::decode_creds(&clear)?))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn endpoint_realms_preserve_factors_and_sealed_credential_ownership() {
        exercise(
            Store::open_in_memory(crate::ServerKey::generate())
                .await
                .unwrap(),
        )
        .await;
    }

    #[tokio::test]
    async fn provider_enrollment_postgres() {
        use sqlx::Connection;
        let Ok(dsn) = std::env::var("MW_DIRECTORY_TEST_PG_DSN") else {
            return;
        };
        let schema = format!("provider_{}", uuid::Uuid::new_v4().simple());
        let mut admin = sqlx::PgConnection::connect(&dsn).await.unwrap();
        sqlx::query(&format!("CREATE SCHEMA {schema}"))
            .execute(&mut admin)
            .await
            .unwrap();
        let mut scoped = url::Url::parse(&dsn).unwrap();
        scoped
            .query_pairs_mut()
            .append_pair("options", &format!("-csearch_path={schema}"));
        exercise(
            Store::open(scoped.as_str(), crate::ServerKey::generate())
                .await
                .unwrap(),
        )
        .await;
        sqlx::query(&format!("DROP SCHEMA {schema} CASCADE"))
            .execute(&mut admin)
            .await
            .unwrap();
    }

    async fn exercise(store: Store) {
        let binding = "a".repeat(64);
        let enroll = |endpoint, username, address| VerifiedEwsEnrollment {
            endpoint_url: endpoint,
            backend_username: username,
            login_address: address,
            domain_binding: binding.as_str(),
            ca_pem: None,
            password: "synthetic-secret",
        };
        let first = store
            .enroll_verified_ews(enroll(
                "https://exchange.example/EWS/a",
                "Alice@example.test",
                "alice@example.test",
            ))
            .await
            .unwrap();
        let (racing_a, racing_b) = tokio::join!(
            store.enroll_verified_ews(enroll(
                "https://exchange.example/EWS/race",
                "race@example.test",
                "race@example.test"
            )),
            store.enroll_verified_ews(enroll(
                "https://exchange.example/EWS/race",
                "RACE@example.test",
                "race@example.test"
            ))
        );
        assert_eq!(racing_a.unwrap().account_id, racing_b.unwrap().account_id);
        q("UPDATE accounts SET sync_policy_json=?2 WHERE id=?1")
            .bind(&first.account_id).bind(serde_json::json!({"domain_binding":binding,"credential_rejected":true,"operatorMarker":"preserve"}).to_string())
            .execute(&store.backend).await.unwrap();
        let again = store
            .enroll_verified_ews(enroll(
                "https://exchange.example/EWS/a",
                "alice@example.test",
                "alias@example.test",
            ))
            .await
            .unwrap();
        assert_eq!(first.account_id, again.account_id);
        assert_eq!(again.login_address, "alice@example.test");
        assert_ne!(first.revision, again.revision);
        let policy: serde_json::Value = serde_json::from_str(
            &store
                .get_account(&first.account_id)
                .await
                .unwrap()
                .sync_policy_json,
        )
        .unwrap();
        assert_eq!(policy["operatorMarker"], "preserve");
        assert!(policy.get("credential_rejected").is_none());
        assert!(
            store
                .enroll_verified_ews(enroll(
                    "https://exchange.example/EWS/a",
                    "alice@example.test",
                    "alice@other.test"
                ))
                .await
                .is_err()
        );
        assert_eq!(
            store
                .provider_account(&first.account_id)
                .await
                .unwrap()
                .unwrap()
                .revision,
            again.revision
        );
        let other = store
            .enroll_verified_ews(enroll(
                "https://exchange.example/EWS/b",
                "alice@example.test",
                "alice@example.test",
            ))
            .await
            .unwrap();
        assert_ne!(first.account_id, other.account_id);
        assert_eq!(
            store.get_account(&first.account_id).await.unwrap().kind,
            crate::AccountKind::Ews
        );
        let opened = store
            .provider_ews_credentials(&first.account_id)
            .await
            .unwrap()
            .unwrap();
        assert_eq!(opened.password, "synthetic-secret");
        assert!(
            store
                .account_credentials(&first.account_id)
                .await
                .unwrap()
                .password
                .is_empty()
        );
        let native = store
            .create_account(
                &crate::NewAccount {
                    kind: crate::AccountKind::Imap,
                    host: "exchange.example",
                    port: 443,
                    tls: "implicit",
                    username: "alice@example.test",
                    sync_policy_json: "{}",
                },
                &Credentials {
                    username: "alice@example.test".into(),
                    password: "native-secret".into(),
                },
            )
            .await
            .unwrap();
        assert_eq!(
            store
                .account_id_by_identity(
                    crate::AccountKind::Imap,
                    "exchange.example",
                    443,
                    "alice@example.test"
                )
                .await
                .unwrap(),
            Some(native)
        );
        let sealed = q("SELECT sealed_credential FROM account_provider WHERE account_id=?1")
            .bind(&first.account_id)
            .fetch_optional(&store.backend)
            .await
            .unwrap()
            .unwrap()
            .get_blob("sealed_credential");
        q("UPDATE account_provider SET sealed_credential=?2 WHERE account_id=?1")
            .bind(&other.account_id)
            .bind(sealed)
            .execute(&store.backend)
            .await
            .unwrap();
        assert!(
            store
                .provider_ews_credentials(&other.account_id)
                .await
                .is_err()
        );
    }
}
