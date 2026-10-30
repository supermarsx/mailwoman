//! Verified-owner identity profiles, notification preferences and push subscriptions.
use crate::{
    IdentityRow, NotificationRulesRow, PushSubscriptionRow, Store, StoreError,
    backend::{Row, q},
};
use chrono::Utc;
impl Store {
    pub async fn upsert_personal_identity(&self, i: &IdentityRow) -> Result<(), StoreError> {
        let affected = q(
            "INSERT INTO personal_identity
                 (id, owner_subject, name, email, reply_to, signature_html, signature_text, signature_name, sent_mailbox_id, source)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)
             ON CONFLICT(owner_subject,id) DO UPDATE SET
                 name = excluded.name, email = excluded.email, reply_to = excluded.reply_to,
                 signature_html = excluded.signature_html, signature_text = excluded.signature_text,
                 signature_name = excluded.signature_name,
                 sent_mailbox_id = excluded.sent_mailbox_id, source = excluded.source
             WHERE personal_identity.owner_subject = excluded.owner_subject",
        )
        .bind(&i.id)
        .bind(&i.account_id)
        .bind(&i.name)
        .bind(&i.email)
        .bind(i.reply_to.as_deref())
        .bind(i.signature_html.as_deref())
        .bind(i.signature_text.as_deref())
        .bind(i.signature_name.as_deref())
        .bind(i.sent_mailbox_id.as_deref())
        .bind(&i.source)
        .execute(&self.backend)
        .await?;
        if affected == 0 {
            return Err(StoreError::NotFound);
        }
        Ok(())
    }

    /// List an account's sending personal_identity.
    pub async fn list_personal_identities(
        &self,
        owner_subject: &str,
    ) -> Result<Vec<IdentityRow>, StoreError> {
        let rows = q(
            "SELECT id, owner_subject, name, email, reply_to, signature_html, signature_text, signature_name, sent_mailbox_id, source
             FROM personal_identity WHERE owner_subject = ?1 ORDER BY source, email",
        )
        .bind(owner_subject)
        .fetch_all(&self.backend)
        .await?;
        Ok(rows.iter().map(identity_from_row).collect())
    }

    /// Fetch one identity by id.
    pub async fn get_personal_identity(
        &self,
        owner_subject: &str,
        id: &str,
    ) -> Result<Option<IdentityRow>, StoreError> {
        let row = q(
            "SELECT id, owner_subject, name, email, reply_to, signature_html, signature_text, signature_name, sent_mailbox_id, source
             FROM personal_identity WHERE id = ?1 AND owner_subject = ?2",
        )
        .bind(id).bind(owner_subject)
        .fetch_optional(&self.backend)
        .await?;
        Ok(row.as_ref().map(identity_from_row))
    }

    /// Delete a sending identity only under its current account owner.
    pub async fn delete_personal_identity(
        &self,
        owner_subject: &str,
        id: &str,
    ) -> Result<bool, StoreError> {
        let affected = q("DELETE FROM personal_identity WHERE id = ?1 AND owner_subject = ?2")
            .bind(id)
            .bind(owner_subject)
            .execute(&self.backend)
            .await?;
        Ok(affected == 1)
    }

    pub async fn put_personal_notification_rules(
        &self,
        rules: &NotificationRulesRow,
    ) -> Result<(), StoreError> {
        let now = Utc::now().to_rfc3339();
        q("INSERT INTO personal_notification_rules (owner_subject, rule_json, quiet_hours_json, enabled, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5)
             ON CONFLICT(owner_subject) DO UPDATE SET
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
    pub async fn get_personal_notification_rules(
        &self,
        owner_subject: &str,
    ) -> Result<Option<NotificationRulesRow>, StoreError> {
        let row = q(
            "SELECT owner_subject, rule_json, quiet_hours_json, enabled, updated_at
                     FROM personal_notification_rules WHERE owner_subject = ?1",
        )
        .bind(owner_subject)
        .fetch_optional(&self.backend)
        .await?;
        Ok(row.as_ref().map(personal_notification_rules_from_row))
    }
    pub async fn upsert_personal_push_subscription(
        &self,
        row: &PushSubscriptionRow,
    ) -> Result<(), StoreError> {
        self.upsert_personal_push_subscription_shared(row, &[])
            .await
    }

    /// Store a subscription, keyed by owner and endpoint (t87 D3). One browser
    /// has one endpoint, so the signed-in account and each linked mailbox open in
    /// it keep their own row for it. An endpoint already held by any owner other
    /// than `row`'s and those in `may_share_with` is refused with `NotFound`, so
    /// a leaked endpoint cannot be bound to an unrelated account.
    pub async fn upsert_personal_push_subscription_shared(
        &self,
        row: &PushSubscriptionRow,
        may_share_with: &[String],
    ) -> Result<(), StoreError> {
        let mut tx = self.backend.begin().await?;
        let owners =
            q("SELECT DISTINCT owner_subject FROM personal_push_subscription WHERE endpoint=?1")
                .bind(&row.endpoint)
                .fetch_all_tx(&mut tx)
                .await?;
        if owners
            .iter()
            .map(|o| o.get_string("owner_subject"))
            .any(|owner| owner != row.account_id && !may_share_with.contains(&owner))
        {
            return Err(StoreError::NotFound);
        }
        q("INSERT INTO personal_push_subscription
                 (id, owner_subject, transport, endpoint, p256dh, auth, app_id,
                  expires_at, created_at, last_wake_at)
             VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9,?10)
             ON CONFLICT(owner_subject, endpoint) DO UPDATE SET
                 transport=excluded.transport,
                 p256dh=excluded.p256dh, auth=excluded.auth, app_id=excluded.app_id,
                 expires_at=excluded.expires_at")
        .bind(&row.id)
        .bind(&row.account_id)
        .bind(&row.transport)
        .bind(&row.endpoint)
        .bind(row.p256dh.as_deref())
        .bind(row.auth.as_deref())
        .bind(row.app_id.as_deref())
        .bind(row.expires_at.as_deref())
        .bind(&row.created_at)
        .bind(row.last_wake_at.as_deref())
        .execute_tx(&mut tx)
        .await?;
        tx.commit().await?;
        Ok(())
    }

    /// All active subscriptions for an account (the dispatcher's fan-out set).
    pub async fn list_personal_push_subscriptions(
        &self,
        owner_subject: &str,
    ) -> Result<Vec<PushSubscriptionRow>, StoreError> {
        let rows = q(
            "SELECT * FROM personal_push_subscription WHERE owner_subject = ?1 ORDER BY created_at",
        )
        .bind(owner_subject)
        .fetch_all(&self.backend)
        .await?;
        Ok(rows.iter().map(push_sub_from_row).collect())
    }

    /// Account-authorized unsubscribe. Unknown/foreign identifiers are no-ops.
    pub async fn delete_personal_push_subscription(
        &self,
        account: &str,
        endpoint: Option<&str>,
        id: Option<&str>,
    ) -> Result<(), StoreError> {
        q("DELETE FROM personal_push_subscription WHERE owner_subject=?1 AND ((?2 IS NOT NULL AND endpoint=?2) OR (?2 IS NULL AND id=?3))")
            .bind(account).bind(endpoint).bind(id).execute(&self.backend).await?;
        Ok(())
    }

    /// Record that an opaque wake was last sent to a subscription (rate/telemetry).
    pub async fn touch_personal_push_wake(
        &self,
        owner: &str,
        id: &str,
        at: &str,
    ) -> Result<(), StoreError> {
        q("UPDATE personal_push_subscription SET last_wake_at = ?2 WHERE id = ?1 AND owner_subject = ?3")
            .bind(id)
            .bind(at).bind(owner)
            .execute(&self.backend)
            .await?;
        Ok(())
    }

    /// Native-only adapters; session routes must derive their verified personal subject.
    pub async fn list_native_personal_identities(
        &self,
        account: &str,
    ) -> Result<Vec<IdentityRow>, StoreError> {
        let owner = self
            .native_personal_subject(account)
            .await?
            .ok_or(StoreError::NotFound)?;
        let mut rows = self.list_personal_identities(&owner).await?;
        for row in &mut rows {
            row.account_id = account.into();
        }
        Ok(rows)
    }
    pub async fn get_native_personal_identity(
        &self,
        account: &str,
        id: &str,
    ) -> Result<Option<IdentityRow>, StoreError> {
        let owner = self
            .native_personal_subject(account)
            .await?
            .ok_or(StoreError::NotFound)?;
        Ok(self
            .get_personal_identity(&owner, id)
            .await?
            .map(|mut row| {
                row.account_id = account.into();
                row
            }))
    }
    pub async fn upsert_native_personal_identity(
        &self,
        row: &IdentityRow,
    ) -> Result<(), StoreError> {
        let mut owned = row.clone();
        owned.account_id = self
            .native_personal_subject(&row.account_id)
            .await?
            .ok_or(StoreError::NotFound)?;
        self.upsert_personal_identity(&owned).await
    }
}

fn identity_from_row(r: &Row) -> IdentityRow {
    IdentityRow {
        id: r.get_string("id"),
        account_id: r.get_string("owner_subject"),
        name: r.get_string("name"),
        email: r.get_string("email"),
        reply_to: r.get_opt_string("reply_to"),
        signature_html: r.get_opt_string("signature_html"),
        signature_text: r.get_opt_string("signature_text"),
        signature_name: r.get_opt_string("signature_name"),
        sent_mailbox_id: r.get_opt_string("sent_mailbox_id"),
        source: r.get_string("source"),
    }
}
fn personal_notification_rules_from_row(r: &Row) -> NotificationRulesRow {
    NotificationRulesRow {
        account_id: r.get_string("owner_subject"),
        rule_json: r.get_string("rule_json"),
        quiet_hours_json: r.get_string("quiet_hours_json"),
        enabled: r.get_i64("enabled") != 0,
        updated_at: r.get_string("updated_at"),
    }
}
fn push_sub_from_row(r: &Row) -> PushSubscriptionRow {
    PushSubscriptionRow {
        id: r.get_string("id"),
        account_id: r.get_string("owner_subject"),
        transport: r.get_string("transport"),
        endpoint: r.get_string("endpoint"),
        p256dh: r.get_opt_string("p256dh"),
        auth: r.get_opt_string("auth"),
        app_id: r.get_opt_string("app_id"),
        expires_at: r.get_opt_string("expires_at"),
        created_at: r.get_string("created_at"),
        last_wake_at: r.get_opt_string("last_wake_at"),
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    async fn isolation(store: Store) {
        let identity = |owner: &str, name: &str| IdentityRow {
            id: "shared-provider-id".into(),
            account_id: owner.into(),
            name: name.into(),
            email: "authorized@example.test".into(),
            reply_to: None,
            signature_html: Some("<p>Rich</p>".into()),
            signature_text: Some("Plain".into()),
            signature_name: Some("work".into()),
            sent_mailbox_id: None,
            source: "configured".into(),
        };
        store
            .upsert_personal_identity(&identity("po1-a", "Realm A"))
            .await
            .unwrap();
        store
            .upsert_personal_identity(&identity("po1-b", "Realm B"))
            .await
            .unwrap();
        assert_eq!(
            store.list_personal_identities("po1-a").await.unwrap()[0].name,
            "Realm A"
        );
        assert_eq!(
            store
                .get_personal_identity("po1-b", "shared-provider-id")
                .await
                .unwrap()
                .unwrap()
                .name,
            "Realm B"
        );
        assert!(
            store
                .delete_personal_identity("po1-b", "shared-provider-id")
                .await
                .unwrap()
        );
        assert!(
            store
                .get_personal_identity("po1-a", "shared-provider-id")
                .await
                .unwrap()
                .is_some()
        );
        let rules = NotificationRulesRow {
            account_id: "po1-a".into(),
            enabled: true,
            rule_json: "[]".into(),
            quiet_hours_json: "{}".into(),
            updated_at: String::new(),
        };
        store.put_notification_rules(&rules).await.unwrap();
        assert!(
            store
                .get_personal_notification_rules("po1-a")
                .await
                .unwrap()
                .is_none()
        );
        store.put_personal_notification_rules(&rules).await.unwrap();
        assert!(
            store
                .get_personal_notification_rules("po1-b")
                .await
                .unwrap()
                .is_none()
        );
        let mut push = PushSubscriptionRow {
            id: "old-subscription".into(),
            account_id: "po1-a".into(),
            transport: "unifiedpush".into(),
            endpoint: "https://push.example.test/endpoint".into(),
            p256dh: None,
            auth: None,
            app_id: None,
            expires_at: None,
            created_at: "2026-01-01T00:00:00Z".into(),
            last_wake_at: None,
        };
        store.upsert_push_subscription(&push).await.unwrap();
        assert!(
            store
                .list_personal_push_subscriptions("po1-a")
                .await
                .unwrap()
                .is_empty()
        );
        store
            .upsert_personal_push_subscription(&push)
            .await
            .unwrap();
        push.account_id = "po1-b".into();
        push.id = "new-subscription".into();
        assert!(matches!(
            store.upsert_personal_push_subscription(&push).await,
            Err(StoreError::NotFound)
        ));
        store
            .delete_personal_push_subscription("po1-b", Some(&push.endpoint), None)
            .await
            .unwrap();
        assert_eq!(
            store
                .list_personal_push_subscriptions("po1-a")
                .await
                .unwrap()
                .len(),
            1
        );
        store
            .delete_personal_push_subscription("po1-a", Some(&push.endpoint), None)
            .await
            .unwrap();
        store
            .upsert_personal_push_subscription(&push)
            .await
            .unwrap();
        // A stale dispatcher result from the old owner cannot remove or touch the replacement.
        store
            .delete_personal_push_subscription("po1-a", None, Some("old-subscription"))
            .await
            .unwrap();
        store
            .touch_personal_push_wake("po1-a", "new-subscription", "later")
            .await
            .unwrap();
        let rows = store
            .list_personal_push_subscriptions("po1-b")
            .await
            .unwrap();
        assert_eq!(rows.len(), 1);
        assert!(rows[0].last_wake_at.is_none());
        assert_eq!(
            store.list_push_subscriptions("po1-a").await.unwrap().len(),
            1,
            "legacy bytes stay intact"
        );
    }
    #[tokio::test]
    async fn personal_delivery_sqlite() {
        isolation(
            Store::open_in_memory(crate::ServerKey::generate())
                .await
                .unwrap(),
        )
        .await;
    }
    #[tokio::test]
    async fn personal_delivery_postgres() {
        use sqlx::Connection;
        let Ok(dsn) = std::env::var("MW_TEST_PG") else {
            eprintln!("SKIP: MW_TEST_PG absent");
            return;
        };
        let schema = format!("personal_delivery_{}", uuid::Uuid::new_v4().simple());
        let mut admin = sqlx::PgConnection::connect(&dsn).await.unwrap();
        sqlx::query(&format!("CREATE SCHEMA {schema}"))
            .execute(&mut admin)
            .await
            .unwrap();
        let mut scoped = url::Url::parse(&dsn).unwrap();
        scoped
            .query_pairs_mut()
            .append_pair("options", &format!("-csearch_path={schema}"));
        isolation(
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
}
