//! 0017 user-preferences repository (t16 e1 skeleton; e15/e11 fill the endpoint
//! callers): additive, dual-backend `Store` methods over `signatures` and
//! `notification_rules` (0017, both dialects).
//!
//! Saved searches (W13) are NOT here — they already exist FROZEN in 0003 (`v2.rs`,
//! `upsert_saved_search`/`list_saved_searches`), so W13 reuses that table. These rows
//! are non-secret user preferences (no sealed columns): signature bodies and the
//! `*_json` rule/quiet-hours blobs are app-owned opaque JSON. Authored in the SQLite
//! `?n` style so they run identically on SQLite or Postgres through [`crate::backend`].

use chrono::Utc;

use crate::backend::q;
use crate::{Store, StoreError};

/// A signature template (0017). At most one per account is the default (enforced
/// app-side). `rule_json` carries optional auto-apply rules (opaque JSON).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SignatureRow {
    pub account_id: String,
    pub name: String,
    pub body: String,
    pub html: Option<String>,
    pub text: Option<String>,
    pub text_edited: bool,
    pub is_default: bool,
    pub rule_json: String,
    pub updated_at: String,
}

/// The per-account notification configuration (0017): a rule set + quiet-hours
/// window + an enabled switch. `rule_json` / `quiet_hours_json` are opaque JSON.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NotificationRulesRow {
    pub account_id: String,
    pub rule_json: String,
    pub quiet_hours_json: String,
    pub enabled: bool,
    pub updated_at: String,
}

impl Store {
    // ---- signatures ----------------------------------------------------------

    /// Upsert a signature template (by `(account_id, name)`), bumping `updated_at`.
    pub async fn upsert_signature(&self, sig: &SignatureRow) -> Result<(), StoreError> {
        let now = Utc::now().to_rfc3339();
        q(
            "INSERT INTO signatures (account_id, name, body, is_default, rule_json, updated_at, html, text, text_edited)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)
             ON CONFLICT(account_id, name) DO UPDATE SET
                 body = excluded.body, is_default = excluded.is_default,
                 rule_json = excluded.rule_json, updated_at = excluded.updated_at, html = excluded.html, text = excluded.text, text_edited = excluded.text_edited",
        )
        .bind(&sig.account_id)
        .bind(&sig.name)
        .bind(&sig.body)
        .bind(i64::from(sig.is_default))
        .bind(&sig.rule_json)
        .bind(&now)
        .bind(sig.html.as_deref())
        .bind(sig.text.as_deref())
        .bind(i64::from(sig.text_edited))
        .execute(&self.backend)
        .await?;
        Ok(())
    }

    /// A user's signature templates (name-ordered).
    pub async fn list_signatures(&self, account_id: &str) -> Result<Vec<SignatureRow>, StoreError> {
        let rows = q(
            "SELECT account_id, name, body, is_default, rule_json, updated_at, html, text, text_edited
                      FROM signatures WHERE account_id = ?1 ORDER BY name",
        )
        .bind(account_id)
        .fetch_all(&self.backend)
        .await?;
        Ok(rows.iter().map(signature_from_row).collect())
    }

    /// Delete a signature template by `(account_id, name)`.
    pub async fn delete_signature(&self, account_id: &str, name: &str) -> Result<(), StoreError> {
        q("DELETE FROM signatures WHERE account_id = ?1 AND name = ?2")
            .bind(account_id)
            .bind(name)
            .execute(&self.backend)
            .await?;
        Ok(())
    }

    // ---- notification rules --------------------------------------------------

    /// Upsert an account's notification configuration, bumping `updated_at`.
    pub async fn put_notification_rules(
        &self,
        rules: &NotificationRulesRow,
    ) -> Result<(), StoreError> {
        let now = Utc::now().to_rfc3339();
        q("INSERT INTO notification_rules (account_id, rule_json, quiet_hours_json, enabled, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5)
             ON CONFLICT(account_id) DO UPDATE SET
                 rule_json = excluded.rule_json, quiet_hours_json = excluded.quiet_hours_json,
                 enabled = excluded.enabled, updated_at = excluded.updated_at")
        .bind(&rules.account_id)
        .bind(&rules.rule_json)
        .bind(&rules.quiet_hours_json)
        .bind(i64::from(rules.enabled))
        .bind(&now)
        .execute(&self.backend)
        .await?;
        Ok(())
    }

    /// Read an account's notification configuration, if set.
    pub async fn get_notification_rules(
        &self,
        account_id: &str,
    ) -> Result<Option<NotificationRulesRow>, StoreError> {
        let row = q(
            "SELECT account_id, rule_json, quiet_hours_json, enabled, updated_at
                     FROM notification_rules WHERE account_id = ?1",
        )
        .bind(account_id)
        .fetch_optional(&self.backend)
        .await?;
        Ok(row.as_ref().map(notification_rules_from_row))
    }
}

fn signature_from_row(r: &crate::backend::Row) -> SignatureRow {
    SignatureRow {
        account_id: r.get_string("account_id"),
        name: r.get_string("name"),
        body: r.get_string("body"),
        html: r.get_opt_string("html"),
        text: r.get_opt_string("text"),
        text_edited: r.get_i64("text_edited") != 0,
        is_default: r.get_i64("is_default") != 0,
        rule_json: r.get_string("rule_json"),
        updated_at: r.get_string("updated_at"),
    }
}

fn notification_rules_from_row(r: &crate::backend::Row) -> NotificationRulesRow {
    NotificationRulesRow {
        account_id: r.get_string("account_id"),
        rule_json: r.get_string("rule_json"),
        quiet_hours_json: r.get_string("quiet_hours_json"),
        enabled: r.get_i64("enabled") != 0,
        updated_at: r.get_string("updated_at"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ServerKey;

    async fn store() -> Store {
        Store::open_in_memory(ServerKey::generate()).await.unwrap()
    }

    async fn assert_signature_formats(store: &Store) {
        let owner=format!("signature-{}",crate::seal::random_token());
        let row=SignatureRow{account_id:owner.clone(),name:"work".into(),body:"legacy".into(),html:Some("<p>Rich</p>".into()),text:Some("Plain alternative".into()),text_edited:true,is_default:false,rule_json:String::new(),updated_at:String::new()};
        store.upsert_signature(&row).await.unwrap();
        let saved=store.list_signatures(&owner).await.unwrap().remove(0);
        assert_eq!(saved.html,row.html);assert_eq!(saved.text,row.text);assert!(saved.text_edited);
        assert!(store.list_signatures(&format!("{owner}-foreign")).await.unwrap().is_empty());
        let mut plain=row;plain.html=None;plain.text=Some("Revised plain".into());store.upsert_signature(&plain).await.unwrap();
        let saved=store.list_signatures(&owner).await.unwrap().remove(0);assert!(saved.html.is_none());assert_eq!(saved.text.as_deref(),Some("Revised plain"));
    }
    #[tokio::test]
    async fn signature_formats_sqlite() {assert_signature_formats(&store().await).await;}
    #[tokio::test]
    async fn signature_formats_postgres() {
        let Ok(dsn)=std::env::var("MW_TEST_PG_DSN") else {return;};
        let store=Store::open_postgres(&dsn,ServerKey::generate()).await.unwrap();assert_signature_formats(&store).await;
    }

    #[tokio::test]
    async fn signatures_crud() {
        let s = store().await;
        assert!(s.list_signatures("a1").await.unwrap().is_empty());

        s.upsert_signature(&SignatureRow { html: None, text: None, text_edited: false,
            account_id: "a1".into(),
            name: "work".into(),
            body: "Regards".into(),
            is_default: true,
            rule_json: "{}".into(),
            updated_at: String::new(),
        })
        .await
        .unwrap();
        s.upsert_signature(&SignatureRow { html: None, text: None, text_edited: false,
            account_id: "a1".into(),
            name: "personal".into(),
            body: "Cheers".into(),
            is_default: false,
            rule_json: String::new(),
            updated_at: String::new(),
        })
        .await
        .unwrap();

        let list = s.list_signatures("a1").await.unwrap();
        assert_eq!(list.len(), 2);
        let work = list.iter().find(|x| x.name == "work").unwrap();
        assert!(work.is_default);
        assert!(!work.updated_at.is_empty());

        // Upsert updates in place.
        s.upsert_signature(&SignatureRow { html: None, text: None, text_edited: false,
            account_id: "a1".into(),
            name: "work".into(),
            body: "Best".into(),
            is_default: true,
            rule_json: "{}".into(),
            updated_at: String::new(),
        })
        .await
        .unwrap();
        assert_eq!(s.list_signatures("a1").await.unwrap().len(), 2);

        s.delete_signature("a1", "personal").await.unwrap();
        assert_eq!(s.list_signatures("a1").await.unwrap().len(), 1);
    }

    #[tokio::test]
    async fn notification_rules_round_trip() {
        let s = store().await;
        assert!(s.get_notification_rules("a1").await.unwrap().is_none());

        s.put_notification_rules(&NotificationRulesRow {
            account_id: "a1".into(),
            rule_json: "[]".into(),
            quiet_hours_json: "{\"start\":\"22:00\"}".into(),
            enabled: true,
            updated_at: String::new(),
        })
        .await
        .unwrap();
        let got = s.get_notification_rules("a1").await.unwrap().unwrap();
        assert!(got.enabled);
        assert_eq!(got.quiet_hours_json, "{\"start\":\"22:00\"}");

        s.put_notification_rules(&NotificationRulesRow {
            account_id: "a1".into(),
            rule_json: "[]".into(),
            quiet_hours_json: String::new(),
            enabled: false,
            updated_at: String::new(),
        })
        .await
        .unwrap();
        assert!(
            !s.get_notification_rules("a1")
                .await
                .unwrap()
                .unwrap()
                .enabled
        );
    }
}
