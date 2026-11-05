//! Personal sender identities: the per-owner default and variants.
//!
//! * Exactly one identity per owner is the default (a partial unique index backs
//!   this). Setting one clears the others in the same transaction; the first
//!   identity an owner creates is the default; deleting the default promotes the
//!   oldest remaining base identity (or the oldest remaining row when only
//!   variants are left).
//! * A variant points at a base identity with `variant_of` and carries a
//!   `label`. It has its own name and address; the optional profile fields it
//!   leaves empty are inherited from the base when resolved
//!   ([`resolve_variants`]). Variants are one level deep: a variant cannot have
//!   variants, and a base that has variants cannot become a variant.
//! * Every write runs under a per-owner lock row so the invariants hold under
//!   concurrent requests on SQLite and Postgres alike.

use crate::{
    IdentityRow, Store, StoreError,
    backend::{Row, Tx, q},
};
use chrono::Utc;

macro_rules! cols {
    () => {
        "id, owner_subject, name, email, reply_to, signature_html, signature_text, signature_name, sent_mailbox_id, source, organization, auto_bcc, pgp_key_id, reply_signature_name, is_default, variant_of, label, created_at"
    };
}

/// What deleting a base identity does to its variants.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VariantPolicy {
    /// Refuse while variants exist.
    Reject,
    /// Delete the variants with the base.
    Delete,
    /// Keep the variants as standalone identities; inherited fields are copied in.
    Promote,
}

/// Outcome of [`Store::delete_personal_identity`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IdentityDelete {
    /// Absent or owned by someone else.
    NotFound,
    /// Refused: the identity still has this many variants.
    HasVariants(usize),
    Deleted,
}

fn blank(value: &Option<String>) -> bool {
    value.as_deref().is_none_or(str::is_empty)
}

impl IdentityRow {
    /// The effective profile of a variant: empty optional fields come from `base`.
    /// `name`, `email`, `label` and the identity's own ids always stay its own.
    /// Empty `reply_to`/`auto_bcc`/`organization` inherit; `signature_name` and
    /// `reply_signature_name` inherit only when unset (an empty string means
    /// "no signature" and is kept).
    pub fn resolved_against(&self, base: &IdentityRow) -> IdentityRow {
        let mut out = self.clone();
        let pick = |own: &Option<String>, inherited: &Option<String>| {
            if blank(own) { inherited.clone() } else { own.clone() }
        };
        out.reply_to = pick(&self.reply_to, &base.reply_to);
        out.auto_bcc = pick(&self.auto_bcc, &base.auto_bcc);
        out.organization = pick(&self.organization, &base.organization);
        if self.pgp_key_id.is_none() {
            out.pgp_key_id = base.pgp_key_id.clone();
        }
        if self.signature_name.is_none() {
            out.signature_name = base.signature_name.clone();
            out.signature_html = self.signature_html.clone().or(base.signature_html.clone());
            out.signature_text = self.signature_text.clone().or(base.signature_text.clone());
        }
        if self.reply_signature_name.is_none() {
            out.reply_signature_name = base.reply_signature_name.clone();
        }
        if self.sent_mailbox_id.is_none() {
            out.sent_mailbox_id = base.sent_mailbox_id.clone();
        }
        out
    }
}

/// Replace every variant by its effective profile (flags and `variant_of` stay).
pub fn resolve_variants(rows: Vec<IdentityRow>) -> Vec<IdentityRow> {
    let bases: Vec<IdentityRow> = rows
        .iter()
        .filter(|r| r.variant_of.is_none())
        .cloned()
        .collect();
    rows.into_iter()
        .map(|row| {
            match row
                .variant_of
                .as_deref()
                .and_then(|id| bases.iter().find(|b| b.id == id))
            {
                Some(base) => row.resolved_against(base),
                None => row,
            }
        })
        .collect()
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
        reply_signature_name: r.get_opt_string("reply_signature_name"),
        organization: r.get_opt_string("organization"),
        auto_bcc: r.get_opt_string("auto_bcc"),
        pgp_key_id: r.get_opt_string("pgp_key_id"),
        sent_mailbox_id: r.get_opt_string("sent_mailbox_id"),
        source: r.get_string("source"),
        is_default: r.get_i64("is_default") != 0,
        variant_of: r.get_opt_string("variant_of"),
        label: r.get_opt_string("label"),
    }
}

async fn lock_owner(tx: &mut Tx, owner: &str) -> Result<(), StoreError> {
    let lock = format!("identity-owner:{owner}");
    q("INSERT INTO settings(key,value) VALUES(?1,'0') ON CONFLICT(key) DO NOTHING")
        .bind(&lock)
        .execute_tx(tx)
        .await?;
    q("UPDATE settings SET value=value WHERE key=?1")
        .bind(&lock)
        .execute_tx(tx)
        .await?;
    Ok(())
}

/// The owner's rows, oldest first.
async fn owner_rows(tx: &mut Tx, owner: &str) -> Result<Vec<IdentityRow>, StoreError> {
    let rows = q(concat!("SELECT ", cols!(), " FROM personal_identity WHERE owner_subject = ?1 ORDER BY created_at, source, email, id"))
    .bind(owner)
    .fetch_all_tx(tx)
    .await?;
    Ok(rows.iter().map(identity_from_row).collect())
}

impl Store {
    /// Insert or update an identity. `i.is_default` asks to become the default; a
    /// `false` never clears the flag (move the default by making another one).
    /// Fails with [`StoreError::Invalid`] on a bad `variant_of`.
    pub async fn upsert_personal_identity(&self, i: &IdentityRow) -> Result<(), StoreError> {
        let mut tx = self.backend.begin().await?;
        lock_owner(&mut tx, &i.account_id).await?;
        let rows = owner_rows(&mut tx, &i.account_id).await?;
        let existing = rows.iter().find(|r| r.id == i.id);
        if let Some(base_id) = &i.variant_of {
            if base_id == &i.id {
                return Err(StoreError::Invalid(
                    "an identity cannot be a variant of itself",
                ));
            }
            let Some(base) = rows.iter().find(|r| &r.id == base_id) else {
                return Err(StoreError::Invalid("the base identity does not exist"));
            };
            if base.variant_of.is_some() {
                return Err(StoreError::Invalid(
                    "a variant cannot be the base of another variant",
                ));
            }
            if rows
                .iter()
                .any(|r| r.variant_of.as_deref() == Some(i.id.as_str()))
            {
                return Err(StoreError::Invalid(
                    "an identity that has variants cannot become a variant",
                ));
            }
        }
        let other_default = rows.iter().any(|r| r.id != i.id && r.is_default);
        let make_default = i.is_default || existing.is_some_and(|r| r.is_default) || !other_default;
        if make_default && other_default {
            q("UPDATE personal_identity SET is_default = 0 WHERE owner_subject = ?1 AND id <> ?2 AND is_default = 1")
                .bind(&i.account_id)
                .bind(&i.id)
                .execute_tx(&mut tx)
                .await?;
        }
        let affected = q(
            "INSERT INTO personal_identity
                 (id, owner_subject, name, email, reply_to, signature_html, signature_text, signature_name, sent_mailbox_id, source, organization, auto_bcc, pgp_key_id, reply_signature_name, is_default, variant_of, label, created_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16, ?17, ?18)
             ON CONFLICT(owner_subject,id) DO UPDATE SET
                 name = excluded.name, email = excluded.email, reply_to = excluded.reply_to,
                 signature_html = excluded.signature_html, signature_text = excluded.signature_text,
                 signature_name = excluded.signature_name, organization = excluded.organization, auto_bcc = excluded.auto_bcc, pgp_key_id = excluded.pgp_key_id, reply_signature_name = excluded.reply_signature_name,
                 sent_mailbox_id = excluded.sent_mailbox_id, source = excluded.source,
                 is_default = excluded.is_default, variant_of = excluded.variant_of, label = excluded.label
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
        .bind(i.organization.as_deref())
        .bind(i.auto_bcc.as_deref())
        .bind(i.pgp_key_id.as_deref())
        .bind(i.reply_signature_name.as_deref())
        .bind(i64::from(make_default))
        .bind(i.variant_of.as_deref())
        .bind(i.label.as_deref())
        .bind(Utc::now().timestamp_millis())
        .execute_tx(&mut tx)
        .await?;
        if affected == 0 {
            return Err(StoreError::NotFound);
        }
        tx.commit().await?;
        Ok(())
    }

    /// An owner's identities in creation order (variants are listed raw; see
    /// [`resolve_variants`]).
    pub async fn list_personal_identities(
        &self,
        owner_subject: &str,
    ) -> Result<Vec<IdentityRow>, StoreError> {
        let rows = q(concat!("SELECT ", cols!(), " FROM personal_identity WHERE owner_subject = ?1 ORDER BY created_at, source, email, id"))
        .bind(owner_subject)
        .fetch_all(&self.backend)
        .await?;
        Ok(rows.iter().map(identity_from_row).collect())
    }

    /// Fetch one identity by id (raw, variants unresolved).
    pub async fn get_personal_identity(
        &self,
        owner_subject: &str,
        id: &str,
    ) -> Result<Option<IdentityRow>, StoreError> {
        let row = q(concat!("SELECT ", cols!(), " FROM personal_identity WHERE id = ?1 AND owner_subject = ?2"))
        .bind(id)
        .bind(owner_subject)
        .fetch_optional(&self.backend)
        .await?;
        Ok(row.as_ref().map(identity_from_row))
    }

    /// Make `id` the owner's default, clearing the previous one atomically.
    /// `Ok(false)` = absent or foreign.
    pub async fn set_personal_identity_default(
        &self,
        owner_subject: &str,
        id: &str,
    ) -> Result<bool, StoreError> {
        let mut tx = self.backend.begin().await?;
        lock_owner(&mut tx, owner_subject).await?;
        let rows = owner_rows(&mut tx, owner_subject).await?;
        if !rows.iter().any(|r| r.id == id) {
            return Ok(false);
        }
        q("UPDATE personal_identity SET is_default = 0 WHERE owner_subject = ?1 AND id <> ?2 AND is_default = 1")
            .bind(owner_subject)
            .bind(id)
            .execute_tx(&mut tx)
            .await?;
        q("UPDATE personal_identity SET is_default = 1 WHERE owner_subject = ?1 AND id = ?2")
            .bind(owner_subject)
            .bind(id)
            .execute_tx(&mut tx)
            .await?;
        tx.commit().await?;
        Ok(true)
    }

    /// Delete an identity under its owner. A base with variants needs a
    /// [`VariantPolicy`] other than `Reject`. Deleting the default promotes the
    /// oldest remaining identity (bases first).
    pub async fn delete_personal_identity(
        &self,
        owner_subject: &str,
        id: &str,
        policy: VariantPolicy,
    ) -> Result<IdentityDelete, StoreError> {
        let mut tx = self.backend.begin().await?;
        lock_owner(&mut tx, owner_subject).await?;
        let rows = owner_rows(&mut tx, owner_subject).await?;
        let Some(base) = rows.iter().find(|r| r.id == id) else {
            return Ok(IdentityDelete::NotFound);
        };
        let variants: Vec<&IdentityRow> = rows
            .iter()
            .filter(|r| r.variant_of.as_deref() == Some(id))
            .collect();
        if !variants.is_empty() && policy == VariantPolicy::Reject {
            return Ok(IdentityDelete::HasVariants(variants.len()));
        }
        let mut removed = vec![id.to_string()];
        let mut promoted = Vec::new();
        for variant in &variants {
            if policy == VariantPolicy::Delete {
                q("DELETE FROM personal_identity WHERE id = ?1 AND owner_subject = ?2")
                    .bind(&variant.id)
                    .bind(owner_subject)
                    .execute_tx(&mut tx)
                    .await?;
                removed.push(variant.id.clone());
                continue;
            }
            let own = variant.resolved_against(base);
            q("UPDATE personal_identity SET variant_of = NULL, label = NULL, reply_to = ?3, auto_bcc = ?4, organization = ?5, pgp_key_id = ?6, signature_name = ?7, signature_html = ?8, signature_text = ?9, reply_signature_name = ?10, sent_mailbox_id = ?11 WHERE id = ?1 AND owner_subject = ?2")
                .bind(&variant.id)
                .bind(owner_subject)
                .bind(own.reply_to.as_deref())
                .bind(own.auto_bcc.as_deref())
                .bind(own.organization.as_deref())
                .bind(own.pgp_key_id.as_deref())
                .bind(own.signature_name.as_deref())
                .bind(own.signature_html.as_deref())
                .bind(own.signature_text.as_deref())
                .bind(own.reply_signature_name.as_deref())
                .bind(own.sent_mailbox_id.as_deref())
                .execute_tx(&mut tx)
                .await?;
            promoted.push(variant.id.clone());
        }
        q("DELETE FROM personal_identity WHERE id = ?1 AND owner_subject = ?2")
            .bind(id)
            .bind(owner_subject)
            .execute_tx(&mut tx)
            .await?;
        let remaining: Vec<&IdentityRow> = rows
            .iter()
            .filter(|r| !removed.contains(&r.id))
            .collect();
        if !remaining.iter().any(|r| r.is_default) {
            // `rows` is oldest first. Prefer a row that is (now) a base identity.
            let next = remaining
                .iter()
                .find(|r| r.variant_of.is_none() || promoted.contains(&r.id))
                .or_else(|| remaining.first());
            if let Some(next) = next {
                q("UPDATE personal_identity SET is_default = 1 WHERE owner_subject = ?1 AND id = ?2")
                    .bind(owner_subject)
                    .bind(&next.id)
                    .execute_tx(&mut tx)
                    .await?;
            }
        }
        tx.commit().await?;
        Ok(IdentityDelete::Deleted)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn row(owner: &str, id: &str, email: &str) -> IdentityRow {
        IdentityRow {
            id: id.into(),
            account_id: owner.into(),
            name: id.into(),
            email: email.into(),
            source: "configured".into(),
            ..Default::default()
        }
    }

    fn variant(owner: &str, id: &str, base: &str) -> IdentityRow {
        IdentityRow {
            variant_of: Some(base.into()),
            label: Some(id.into()),
            ..row(owner, id, "v@example.test")
        }
    }

    async fn defaults(store: &Store, owner: &str) -> Vec<String> {
        let mut ids: Vec<String> = store
            .list_personal_identities(owner)
            .await
            .unwrap()
            .into_iter()
            .filter(|r| r.is_default)
            .map(|r| r.id)
            .collect();
        ids.sort();
        ids
    }

    async fn run(store: Store) {
        let o = "pi-owner";
        // The first identity is the default automatically.
        store.upsert_personal_identity(&row(o, "a", "a@example.test")).await.unwrap();
        store.upsert_personal_identity(&row(o, "b", "b@example.test")).await.unwrap();
        assert_eq!(defaults(&store, o).await, ["a"]);
        // Saving without the flag never clears the default.
        store.upsert_personal_identity(&row(o, "a", "a2@example.test")).await.unwrap();
        assert_eq!(defaults(&store, o).await, ["a"]);
        // Asking for default moves it atomically.
        store
            .upsert_personal_identity(&IdentityRow { is_default: true, ..row(o, "b", "b@example.test") })
            .await
            .unwrap();
        assert_eq!(defaults(&store, o).await, ["b"]);
        assert!(store.set_personal_identity_default(o, "a").await.unwrap());
        assert_eq!(defaults(&store, o).await, ["a"]);
        // Foreign / missing ids and other owners are untouched.
        assert!(!store.set_personal_identity_default("other", "a").await.unwrap());
        assert!(!store.set_personal_identity_default(o, "missing").await.unwrap());
        store.upsert_personal_identity(&row("other", "a", "x@example.test")).await.unwrap();
        assert_eq!(defaults(&store, "other").await, ["a"]);
        assert_eq!(defaults(&store, o).await, ["a"]);

        // Variant validation.
        for bad in [
            variant(o, "v", "missing"),
            variant(o, "v", "v"),
        ] {
            assert!(matches!(store.upsert_personal_identity(&bad).await, Err(StoreError::Invalid(_))));
        }
        store.upsert_personal_identity(&variant(o, "v1", "a")).await.unwrap();
        store.upsert_personal_identity(&variant(o, "v2", "a")).await.unwrap();
        assert!(matches!(
            store.upsert_personal_identity(&variant(o, "nested", "v1")).await,
            Err(StoreError::Invalid(_))
        ));
        assert!(matches!(
            store.upsert_personal_identity(&variant(o, "a", "b")).await,
            Err(StoreError::Invalid(_))
        ));
        // A variant of another owner's identity is refused.
        assert!(matches!(
            store.upsert_personal_identity(&variant("other", "v", "b")).await,
            Err(StoreError::Invalid(_))
        ));

        // Inheritance: empty fields come from the base.
        let mut base = row(o, "a", "a2@example.test");
        base.reply_to = Some("base-reply@example.test".into());
        base.organization = Some("Base Org".into());
        base.signature_name = Some("work".into());
        store.upsert_personal_identity(&base).await.unwrap();
        let mut v1 = variant(o, "v1", "a");
        v1.organization = Some("Own Org".into());
        store.upsert_personal_identity(&v1).await.unwrap();
        let resolved = resolve_variants(store.list_personal_identities(o).await.unwrap());
        let v1 = resolved.iter().find(|r| r.id == "v1").unwrap();
        assert_eq!(v1.reply_to.as_deref(), Some("base-reply@example.test"));
        assert_eq!(v1.organization.as_deref(), Some("Own Org"));
        assert_eq!(v1.signature_name.as_deref(), Some("work"));
        assert_eq!(v1.email, "v@example.test");
        let raw = store.get_personal_identity(o, "v1").await.unwrap().unwrap();
        assert_eq!(raw.reply_to, None);

        // A base with variants is not deleted without a choice.
        assert_eq!(
            store.delete_personal_identity(o, "a", VariantPolicy::Reject).await.unwrap(),
            IdentityDelete::HasVariants(2)
        );
        assert!(store.get_personal_identity(o, "a").await.unwrap().is_some());
        // Variants can be the default.
        assert!(store.set_personal_identity_default(o, "v1").await.unwrap());
        assert_eq!(defaults(&store, o).await, ["v1"]);
        // Promote keeps the variants as standalone identities with inherited fields copied.
        assert_eq!(
            store.delete_personal_identity(o, "a", VariantPolicy::Promote).await.unwrap(),
            IdentityDelete::Deleted
        );
        let v1 = store.get_personal_identity(o, "v1").await.unwrap().unwrap();
        assert_eq!(v1.variant_of, None);
        assert_eq!(v1.reply_to.as_deref(), Some("base-reply@example.test"));
        assert_eq!(defaults(&store, o).await, ["v1"]);

        // Deleting the default promotes the oldest remaining base.
        assert_eq!(
            store.delete_personal_identity(o, "v1", VariantPolicy::Reject).await.unwrap(),
            IdentityDelete::Deleted
        );
        assert_eq!(defaults(&store, o).await.len(), 1);
        // Delete policy removes variants with the base; the last row leaves no default.
        store.upsert_personal_identity(&variant(o, "w", "b")).await.unwrap();
        for id in ["v2"] {
            store.delete_personal_identity(o, id, VariantPolicy::Reject).await.unwrap();
        }
        assert_eq!(
            store.delete_personal_identity(o, "b", VariantPolicy::Delete).await.unwrap(),
            IdentityDelete::Deleted
        );
        assert!(store.get_personal_identity(o, "w").await.unwrap().is_none());
        assert!(store.list_personal_identities(o).await.unwrap().is_empty());
        assert_eq!(
            store.delete_personal_identity(o, "b", VariantPolicy::Reject).await.unwrap(),
            IdentityDelete::NotFound
        );
        // Another owner cannot delete.
        assert_eq!(
            store.delete_personal_identity("intruder", "a", VariantPolicy::Delete).await.unwrap(),
            IdentityDelete::NotFound
        );
        assert!(store.get_personal_identity("other", "a").await.unwrap().is_some());
    }

    #[tokio::test]
    async fn identity_defaults_and_variants_sqlite() {
        run(Store::open_in_memory(crate::ServerKey::generate()).await.unwrap()).await;
    }

    #[tokio::test]
    async fn identity_defaults_and_variants_postgres() {
        use sqlx::Connection;
        let Ok(dsn) = std::env::var("MW_TEST_PG") else {
            eprintln!("SKIP: MW_TEST_PG absent");
            return;
        };
        let schema = format!("identity_variants_{}", uuid::Uuid::new_v4().simple());
        let mut admin = sqlx::PgConnection::connect(&dsn).await.unwrap();
        sqlx::query(&format!("CREATE SCHEMA {schema}")).execute(&mut admin).await.unwrap();
        let mut scoped = url::Url::parse(&dsn).unwrap();
        scoped.query_pairs_mut().append_pair("options", &format!("-csearch_path={schema}"));
        run(Store::open(scoped.as_str(), crate::ServerKey::generate()).await.unwrap()).await;
        sqlx::query(&format!("DROP SCHEMA {schema} CASCADE")).execute(&mut admin).await.unwrap();
    }
}
