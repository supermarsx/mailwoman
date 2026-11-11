//! Account-scoped sender exceptions over the existing 0033 receipt tables.
//! A per-account revision and write lock make editor saves and quotas atomic.
use crate::backend::{Tx, q};
use crate::{Store, StoreError};

pub const MAX_SENDER_RULES: usize = 500;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SenderDecision {
    Always,
    Never,
}
impl SenderDecision {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Always => "always",
            Self::Never => "never",
        }
    }
    pub fn parse(value: &str) -> Option<Self> {
        match value {
            "always" => Some(Self::Always),
            "never" => Some(Self::Never),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SenderRule {
    pub address: String,
    pub decision: SenderDecision,
    pub updated_at: i64,
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SenderRules {
    pub revision: String,
    pub rules: Vec<SenderRule>,
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SenderRuleWrite {
    Applied(String),
    Conflict,
    LimitReached,
}

/// Exact mailbox matching: preserve the local part, canonicalize only its domain.
/// This is intentionally not a wildcard or a display-name parser.
pub fn normalize_sender(address: &str) -> Result<String, StoreError> {
    let invalid = || StoreError::Corrupt("invalid receipt sender mailbox".into());
    let address = address.trim();
    if address.len() > 254 || address.chars().any(|c| c.is_control() || c.is_whitespace()) {
        return Err(invalid());
    }
    let (local, domain) = address.rsplit_once('@').ok_or_else(invalid)?;
    if local.is_empty()
        || local.len() > 64
        || local.starts_with('.')
        || local.ends_with('.')
        || local.contains("..")
        || local.chars().any(|c| "@<>()[]:;\\,\"".contains(c))
    {
        return Err(invalid());
    }
    let domain = crate::settings_policy::normalize_policy_domain(domain)?;
    let canonical = format!("{local}@{domain}");
    if canonical.len() > 254 {
        return Err(invalid());
    }
    Ok(canonical)
}

impl Store {
    /// Bound candidate count and bytes in SQL, before materializing contact data.
    /// Callers must parse these cards and compare exact mailbox values.
    pub async fn receipt_contact_cards(
        &self,
        account: &str,
        address: &str,
    ) -> Result<Vec<String>, StoreError> {
        let address = normalize_sender(address)?;
        let pattern = format!("%{}%", address.replace('%', "\\%").replace('_', "\\_"));
        let sql = match self.backend.dialect() {
            crate::backend::Dialect::Sqlite => {
                "SELECT c.vcard_raw FROM contacts c JOIN address_books b ON b.id=c.address_book_id WHERE b.account_id=?1 AND length(CAST(c.vcard_raw AS BLOB))<=65536 AND c.vcard_raw LIKE ?2 ESCAPE '\\' ORDER BY c.id LIMIT 64"
            }
            crate::backend::Dialect::Postgres => {
                "SELECT c.vcard_raw FROM contacts c JOIN address_books b ON b.id=c.address_book_id WHERE b.account_id=?1 AND octet_length(c.vcard_raw)<=65536 AND c.vcard_raw ILIKE ?2 ESCAPE '\\' ORDER BY c.id LIMIT 64"
            }
        };
        Ok(q(sql)
            .bind(account)
            .bind(pattern)
            .fetch_all(&self.backend)
            .await?
            .into_iter()
            .map(|row| row.get_string("vcard_raw"))
            .collect())
    }

    async fn receipt_rule_tx(&self, account: &str) -> Result<Tx, StoreError> {
        let mut tx = self.backend.begin().await?;
        // This also prevents a rule save racing deletion of its native account.
        if q("UPDATE accounts SET id=id WHERE id=?1")
            .bind(account)
            .execute_tx(&mut tx)
            .await?
            == 0
        {
            return Err(StoreError::NotFound);
        }
        Ok(tx)
    }

    async fn sender_rules_tx(&self, account: &str, tx: &mut Tx) -> Result<SenderRules, StoreError> {
        let key = format!("mdn_sender_revision:{account}");
        let candidate = crate::seal::random_token();
        q("INSERT INTO settings(key,value) VALUES(?1,?2) ON CONFLICT(key) DO NOTHING")
            .bind(&key)
            .bind(candidate)
            .execute_tx(tx)
            .await?;
        let revision = q("SELECT value FROM settings WHERE key=?1")
            .bind(key)
            .fetch_all_tx(tx)
            .await?
            .first()
            .ok_or(StoreError::NotFound)?
            .get_string("value");
        let rows=q("SELECT address,decision,updated_at FROM mdn_sender_rule WHERE account_id=?1 ORDER BY address LIMIT 501").bind(account).fetch_all_tx(tx).await?;
        if rows.len() > MAX_SENDER_RULES {
            return Err(StoreError::Corrupt(
                "receipt sender rule limit exceeded".into(),
            ));
        }
        let mut rules = Vec::with_capacity(rows.len());
        for row in rows {
            let address = row.get_string("address");
            if normalize_sender(&address)? != address {
                return Err(StoreError::Corrupt(
                    "noncanonical receipt sender rule".into(),
                ));
            }
            let decision = SenderDecision::parse(&row.get_string("decision"))
                .ok_or_else(|| StoreError::Corrupt("invalid receipt sender decision".into()))?;
            rules.push(SenderRule {
                address,
                decision,
                updated_at: row.get_i64("updated_at"),
            });
        }
        Ok(SenderRules { revision, rules })
    }

    pub async fn receipt_sender_rules(&self, account: &str) -> Result<SenderRules, StoreError> {
        let mut tx = self.receipt_rule_tx(account).await?;
        let result = self.sender_rules_tx(account, &mut tx).await?;
        tx.commit().await?;
        Ok(result)
    }

    /// Read one exact exception at the actual receipt decision boundary.
    pub async fn receipt_sender_decision(
        &self,
        account: &str,
        address: &str,
    ) -> Result<Option<SenderDecision>, StoreError> {
        let address = normalize_sender(address)?;
        let value=q("SELECT r.decision FROM mdn_sender_rule r JOIN accounts a ON a.id=r.account_id WHERE r.account_id=?1 AND r.address=?2").bind(account).bind(address).fetch_opt_scalar_string(&self.backend).await?;
        value
            .map(|value| {
                SenderDecision::parse(&value)
                    .ok_or_else(|| StoreError::Corrupt("invalid receipt sender decision".into()))
            })
            .transpose()
    }

    /// None removes an exception. Every mutation requires the editor's revision.
    pub async fn write_receipt_sender_rule(
        &self,
        account: &str,
        address: &str,
        decision: Option<SenderDecision>,
        expected: &str,
    ) -> Result<SenderRuleWrite, StoreError> {
        let address = normalize_sender(address)?;
        let mut tx = self.receipt_rule_tx(account).await?;
        let current = self.sender_rules_tx(account, &mut tx).await?;
        if current.revision != expected {
            return Ok(SenderRuleWrite::Conflict);
        }
        let existing = current.rules.iter().find(|rule| rule.address == address);
        if existing.map(|rule| rule.decision) == decision {
            tx.commit().await?;
            return Ok(SenderRuleWrite::Applied(current.revision));
        }
        if decision.is_some() && existing.is_none() && current.rules.len() >= MAX_SENDER_RULES {
            return Ok(SenderRuleWrite::LimitReached);
        }
        if let Some(decision) = decision {
            q("INSERT INTO mdn_sender_rule(account_id,address,decision,updated_at) VALUES(?1,?2,?3,?4) ON CONFLICT(account_id,address) DO UPDATE SET decision=excluded.decision,updated_at=excluded.updated_at").bind(account).bind(&address).bind(decision.as_str()).bind(chrono::Utc::now().timestamp()).execute_tx(&mut tx).await?;
        } else {
            q("DELETE FROM mdn_sender_rule WHERE account_id=?1 AND address=?2")
                .bind(account)
                .bind(&address)
                .execute_tx(&mut tx)
                .await?;
        }
        let revision = crate::seal::random_token();
        q("UPDATE settings SET value=?2 WHERE key=?1")
            .bind(format!("mdn_sender_revision:{account}"))
            .bind(&revision)
            .execute_tx(&mut tx)
            .await?;
        tx.commit().await?;
        Ok(SenderRuleWrite::Applied(revision))
    }
}

/// A due receipt to be revalidated before contacting the submitter.
pub struct ScheduledMdn {
    pub account_id: String,
    pub email_id: String,
    pub requested_by: String,
    pub send_at: i64,
}

impl Store {
    pub async fn scheduled_mdn_time(&self, account: &str, email: &str) -> Result<Option<i64>, StoreError> {
        let row = q("SELECT send_at FROM scheduled_mdn WHERE account_id=?1 AND email_id=?2")
            .bind(account).bind(email).fetch_optional(&self.backend).await?;
        Ok(row.map(|row| row.get_i64("send_at")))
    }

    pub async fn mdn_sender_schedule(&self, account: &str, address: &str, now: i64) -> Result<Option<i64>, StoreError> {
        let row = q("SELECT send_at FROM mdn_sender_schedule WHERE account_id=?1 AND address=?2 AND send_at>?3")
            .bind(account).bind(address).bind(now).fetch_optional(&self.backend).await?;
        Ok(row.map(|row| row.get_i64("send_at")))
    }

    /// Serialise the cap with other writes for this account.
    pub async fn schedule_mdn(
        &self, account: &str, email: &str, address: &str, at: i64, sender_rule: Option<&str>, expected_rule: Option<&str>,
    ) -> Result<bool, StoreError> {
        let mut tx = self.receipt_rule_tx(account).await?;
        if let Some(sender) = expected_rule {
            let current = q("SELECT send_at FROM mdn_sender_schedule WHERE account_id=?1 AND address=?2")
                .bind(account).bind(sender).fetch_all_tx(&mut tx).await?;
            if current.first().is_none_or(|row| row.get_i64("send_at") != at) {
                return Ok(false);
            }
        }
        let changed = q("INSERT INTO scheduled_mdn(account_id,email_id,requested_by,send_at)
            SELECT ?1,?2,?3,?4 FROM messages
            WHERE stable_id=?2 AND account_id=?1 AND (
                EXISTS(SELECT 1 FROM scheduled_mdn WHERE account_id=?1 AND email_id=?2)
                OR (SELECT COUNT(*) FROM scheduled_mdn WHERE account_id=?1)<200)
            ON CONFLICT(account_id,email_id) DO UPDATE SET requested_by=excluded.requested_by,send_at=excluded.send_at,retry_at=NULL
            WHERE (scheduled_mdn.lease_until IS NULL OR scheduled_mdn.lease_until<=?6) AND ?5=CAST(0 AS BIGINT)")
            .bind(account).bind(email).bind(address).bind(at).bind(i64::from(expected_rule.is_some())).bind(chrono::Utc::now().timestamp()).execute_tx(&mut tx).await?;
        if changed == 0 {
            return Ok(false);
        }
        if let Some(sender) = sender_rule {
            q("DELETE FROM mdn_sender_schedule WHERE account_id=?1 AND send_at<=?2")
                .bind(account).bind(chrono::Utc::now().timestamp()).execute_tx(&mut tx).await?;
            let changed = q("INSERT INTO mdn_sender_schedule(account_id,address,send_at)
                SELECT ?1,?2,?3 FROM accounts WHERE id=?1 AND (
                    EXISTS(SELECT 1 FROM mdn_sender_schedule WHERE account_id=?1 AND address=?2)
                    OR (SELECT COUNT(*) FROM mdn_sender_schedule WHERE account_id=?1)<100)
                ON CONFLICT(account_id,address) DO UPDATE SET send_at=excluded.send_at")
                .bind(account).bind(sender).bind(at).execute_tx(&mut tx).await?;
            if changed == 0 {
                return Ok(false);
            }
        }
        tx.commit().await?;
        Ok(true)
    }

    pub async fn cancel_scheduled_mdn(&self, account: &str, email: &str) -> Result<bool, StoreError> {
        Ok(q("DELETE FROM scheduled_mdn WHERE account_id=?1 AND email_id=?2 AND (lease_until IS NULL OR lease_until<=?3)")
            .bind(account).bind(email).bind(chrono::Utc::now().timestamp()).execute(&self.backend).await? > 0)
    }

    pub async fn clear_mdn_sender_schedule(&self, account: &str, sender: &str) -> Result<(), StoreError> {
        q("DELETE FROM mdn_sender_schedule WHERE account_id=?1 AND address=?2")
            .bind(account).bind(sender).execute(&self.backend).await?;
        Ok(())
    }

    pub async fn due_scheduled_mdn(&self, now: i64) -> Result<Vec<ScheduledMdn>, StoreError> {
        let rows = q("SELECT account_id,email_id,requested_by,send_at FROM scheduled_mdn
            WHERE send_at<=?1 AND (lease_until IS NULL OR lease_until<=?1) AND (retry_at IS NULL OR retry_at<=?1)
            ORDER BY send_at,account_id,email_id LIMIT 24")
            .bind(now).fetch_all(&self.backend).await?;
        Ok(rows.into_iter().map(|row| ScheduledMdn {
            account_id: row.get_string("account_id"),
            email_id: row.get_string("email_id"),
            requested_by: row.get_string("requested_by"),
            send_at: row.get_i64("send_at"),
        }).collect())
    }

    pub async fn claim_scheduled_mdn(&self, row: &ScheduledMdn, now: i64, lease: i64) -> Result<bool, StoreError> {
        Ok(q("UPDATE scheduled_mdn SET lease_until=?1 WHERE account_id=?2 AND email_id=?3
            AND send_at=?4 AND requested_by=?5 AND (lease_until IS NULL OR lease_until<=?6) AND (retry_at IS NULL OR retry_at<=?6)")
            .bind(lease).bind(&row.account_id).bind(&row.email_id).bind(row.send_at)
            .bind(&row.requested_by).bind(now).execute(&self.backend).await? > 0)
    }

    pub async fn defer_scheduled_mdn(&self, row: &ScheduledMdn, lease: i64, retry: i64) -> Result<(), StoreError> {
        q("UPDATE scheduled_mdn SET retry_at=?1,lease_until=NULL WHERE account_id=?2 AND email_id=?3 AND lease_until=?4")
            .bind(retry).bind(&row.account_id).bind(&row.email_id).bind(lease).execute(&self.backend).await?;
        Ok(())
    }

    pub async fn finish_scheduled_mdn(&self, row: &ScheduledMdn, lease: i64) -> Result<(), StoreError> {
        q("DELETE FROM scheduled_mdn WHERE account_id=?1 AND email_id=?2 AND lease_until=?3")
            .bind(&row.account_id).bind(&row.email_id).bind(lease).execute(&self.backend).await?;
        Ok(())
    }

}

#[cfg(test)]
mod tests {
    use super::*;
    async fn account(store: &Store) -> String {
        let username = format!("receipt-{}@example.org", crate::seal::random_token());
        store
            .create_account(
                &crate::NewAccount {
                    kind: crate::AccountKind::Imap,
                    host: "imap.example.org",
                    port: 993,
                    tls: "implicit",
                    username: &username,
                    sync_policy_json: "{}",
                },
                &crate::Credentials {
                    username: username.clone(),
                    password: "test".into(),
                },
            )
            .await
            .unwrap()
    }
    async fn exercise(store: Store) {
        let a = account(&store).await;
        let b = account(&store).await;
        let book = format!("receipt-{a}");
        store
            .upsert_address_book(&crate::AddressBookRow {
                id: book.clone(),
                account_id: a.clone(),
                name: "Receipts".into(),
                is_default: true,
                carddav_url: None,
                sync_token: None,
                ctag: None,
            })
            .await
            .unwrap();
        let mut card = crate::ContactRow {
            id: format!("receipt-card-{a}"),
            address_book_id: book,
            uid: "one".into(),
            etag: None,
            vcard_raw:
                "BEGIN:VCARD\r\nVERSION:3.0\r\nFN:Ann\r\nEMAIL:ann@example.org\r\nEND:VCARD\r\n"
                    .into(),
            json: None,
            full_name: "Ann".into(),
            is_favorite: false,
            photo_blob_id: None,
            pgp_key: None,
            smime_cert: None,
        };
        store.upsert_contact(&card).await.unwrap();
        assert_eq!(
            store
                .receipt_contact_cards(&a, "ann@example.org")
                .await
                .unwrap()
                .len(),
            1
        );
        assert!(
            store
                .receipt_contact_cards(&b, "ann@example.org")
                .await
                .unwrap()
                .is_empty()
        );
        // Enforce bytes, not characters: this fits 64Ki characters but not bytes.
        card.vcard_raw.push_str(&"é".repeat(40_000));
        store.upsert_contact(&card).await.unwrap();
        assert!(
            store
                .receipt_contact_cards(&a, "ann@example.org")
                .await
                .unwrap()
                .is_empty()
        );
        let first = store.receipt_sender_rules(&a).await.unwrap();
        let (one, two) = tokio::join!(
            store.write_receipt_sender_rule(
                &a,
                "one@EXAMPLE.org",
                Some(SenderDecision::Always),
                &first.revision
            ),
            store.write_receipt_sender_rule(
                &a,
                "two@example.org",
                Some(SenderDecision::Never),
                &first.revision
            )
        );
        let results = [one.unwrap(), two.unwrap()];
        assert_eq!(
            results
                .iter()
                .filter(|value| matches!(value, SenderRuleWrite::Applied(_)))
                .count(),
            1
        );
        assert_eq!(
            results
                .iter()
                .filter(|value| **value == SenderRuleWrite::Conflict)
                .count(),
            1
        );
        let saved = store.receipt_sender_rules(&a).await.unwrap();
        assert_eq!(saved.rules.len(), 1);
        let rule = &saved.rules[0];
        assert_eq!(
            store
                .receipt_sender_decision(&a, &rule.address)
                .await
                .unwrap(),
            Some(rule.decision)
        );
        assert_eq!(
            store
                .receipt_sender_decision(&b, &rule.address)
                .await
                .unwrap(),
            None
        );
        assert_eq!(
            store
                .write_receipt_sender_rule(&b, &rule.address, None, &saved.revision)
                .await
                .unwrap(),
            SenderRuleWrite::Conflict
        );
        assert_eq!(
            store
                .receipt_sender_decision(&a, &rule.address)
                .await
                .unwrap(),
            Some(rule.decision)
        );
        assert!(matches!(
            store
                .write_receipt_sender_rule(&a, &rule.address, None, &saved.revision)
                .await
                .unwrap(),
            SenderRuleWrite::Applied(_)
        ));
        assert!(
            store
                .receipt_sender_rules(&a)
                .await
                .unwrap()
                .rules
                .is_empty()
        );
        let mut tx = store.receipt_rule_tx(&a).await.unwrap();
        for index in 0..MAX_SENDER_RULES - 1 {
            q("INSERT INTO mdn_sender_rule(account_id,address,decision,updated_at) VALUES(?1,?2,'never',1)").bind(&a).bind(format!("sender{index}@example.org")).execute_tx(&mut tx).await.unwrap();
        }
        tx.commit().await.unwrap();
        let near_limit = store.receipt_sender_rules(&a).await.unwrap();
        let (one, two) = tokio::join!(
            store.write_receipt_sender_rule(
                &a,
                "last-one@example.org",
                Some(SenderDecision::Never),
                &near_limit.revision
            ),
            store.write_receipt_sender_rule(
                &a,
                "last-two@example.org",
                Some(SenderDecision::Never),
                &near_limit.revision
            )
        );
        assert_eq!(
            [one.unwrap(), two.unwrap()]
                .iter()
                .filter(|value| matches!(value, SenderRuleWrite::Applied(_)))
                .count(),
            1
        );
        let full = store.receipt_sender_rules(&a).await.unwrap();
        assert_eq!(full.rules.len(), MAX_SENDER_RULES);
        assert_eq!(
            store
                .write_receipt_sender_rule(
                    &a,
                    "over@example.org",
                    Some(SenderDecision::Always),
                    &full.revision
                )
                .await
                .unwrap(),
            SenderRuleWrite::LimitReached
        );
        assert!(matches!(
            store
                .write_receipt_sender_rule(
                    &a,
                    &full.rules[0].address,
                    Some(SenderDecision::Always),
                    &full.revision
                )
                .await
                .unwrap(),
            SenderRuleWrite::Applied(_)
        ));
        assert!(
            store
                .receipt_sender_rules(&b)
                .await
                .unwrap()
                .rules
                .is_empty(),
            "another account's rows never consume this account's limit"
        );
        assert!(matches!(
            store.receipt_sender_rules("nonexistent").await,
            Err(StoreError::NotFound)
        ));
    }
    #[tokio::test]
    async fn sender_rules_sqlite_authority_and_competing_editors() {
        exercise(
            Store::open_in_memory(crate::ServerKey::generate())
                .await
                .unwrap(),
        )
        .await;
    }
    #[tokio::test]
    async fn sender_rules_postgres_authority_and_competing_editors() {
        let Ok(dsn) = std::env::var("MW_TEST_PG") else {
            eprintln!("MW_TEST_PG unset: live PostgreSQL gate not run");
            return;
        };
        exercise(
            Store::open(&dsn, crate::ServerKey::generate())
                .await
                .unwrap(),
        )
        .await;
    }
    #[test]
    fn sender_addresses_are_exact_not_patterns_or_headers() {
        assert_eq!(
            normalize_sender(" Person@EXAMPLE.org ").unwrap(),
            "Person@example.org"
        );
        for value in [
            "a@example.org\r\nBcc:victim@example.org",
            "a,b@example.org",
            "a@b@example.org",
            "a@https://example.org",
            "a..b@example.org",
        ] {
            assert!(normalize_sender(value).is_err(), "{value}");
        }
    }
}
