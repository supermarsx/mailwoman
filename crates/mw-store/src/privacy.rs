//! Art.18 restrictions of processing and the purpose consent ledger.
//!
//! A restriction pauses processing categories ("scopes") for one account. The
//! hot path is [`Store::restriction_blocks`], called from the places that process
//! an account's data (indexing, assistant, push, tracking, rules); it is one
//! indexed query that returns false when nothing is active.
//!
//! The consent ledger is append-only. The current decision for a purpose is the
//! newest row of the account; a withdrawal is a row with decision `withdrawn`.
use crate::{Store, StoreError, backend::Row, q};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

/// Processing categories a restriction can pause.
pub const RESTRICTION_SCOPES: [&str; 6] = [
    "indexing",
    "assistant",
    "analytics",
    "classification",
    "push",
    "thirdParty",
];
/// What happens to mail arriving for a restricted account.
pub const MAIL_DELIVERY: [&str; 3] = ["keep", "queue", "reject"];
/// Most rows one listing or export reads.
pub const MAX_ROWS: i64 = 20_000;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RestrictionRow {
    pub id: String,
    pub account_id: String,
    pub address: String,
    pub domain: String,
    pub scopes: Vec<String>,
    pub mail_delivery: String,
    pub reason: String,
    pub source: String,
    pub request_id: Option<String>,
    pub created_by: String,
    pub created_at: i64,
    pub expires_at: Option<i64>,
    pub lifted_at: Option<i64>,
    pub lifted_by: Option<String>,
    pub lift_reason: Option<String>,
}

impl RestrictionRow {
    /// Whether the restriction is in force at `now`.
    pub fn active_at(&self, now: i64) -> bool {
        self.lifted_at.is_none() && self.expires_at.is_none_or(|e| e > now)
    }

    /// Whether the restriction pauses `scope` (`all` pauses every scope).
    pub fn covers(&self, scope: &str) -> bool {
        self.scopes.iter().any(|s| s == scope || s == "all")
    }
}

macro_rules! rcols {
    () => {
        "id,account_id,address,domain,scopes_json,mail_delivery,reason,source,request_id,created_by,created_at,expires_at,lifted_at,lifted_by,lift_reason"
    };
}

fn restriction(r: &Row) -> RestrictionRow {
    RestrictionRow {
        id: r.get_string("id"),
        account_id: r.get_string("account_id"),
        address: r.get_string("address"),
        domain: r.get_string("domain"),
        scopes: serde_json::from_str(&r.get_string("scopes_json")).unwrap_or_default(),
        mail_delivery: r.get_string("mail_delivery"),
        reason: r.get_string("reason"),
        source: r.get_string("source"),
        request_id: r.get_opt_string("request_id"),
        created_by: r.get_string("created_by"),
        created_at: r.get_i64("created_at"),
        expires_at: r.get_opt_i64("expires_at"),
        lifted_at: r.get_opt_i64("lifted_at"),
        lifted_by: r.get_opt_string("lifted_by"),
        lift_reason: r.get_opt_string("lift_reason"),
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ConsentRecord {
    pub id: String,
    pub account_id: String,
    pub domain: String,
    pub purpose: String,
    pub decision: String,
    pub version: i64,
    pub basis: String,
    pub source: String,
    pub created_at: i64,
}

macro_rules! ccols {
    () => {
        "id,account_id,domain,purpose,decision,version,basis,source,created_at"
    };
}

fn consent(r: &Row) -> ConsentRecord {
    ConsentRecord {
        id: r.get_string("id"),
        account_id: r.get_string("account_id"),
        domain: r.get_string("domain"),
        purpose: r.get_string("purpose"),
        decision: r.get_string("decision"),
        version: r.get_i64("version"),
        basis: r.get_string("basis"),
        source: r.get_string("source"),
        created_at: r.get_i64("created_at"),
    }
}

/// Row counts and approximate sizes of what is stored about one account.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Footprint {
    /// Category -> (rows, approximate bytes).
    pub items: BTreeMap<String, (i64, i64)>,
}

impl Store {
    /// Place an account under a restriction. An existing active restriction of
    /// the account is replaced (lifted with the reason `replaced`).
    pub async fn restriction_apply(&self, row: &RestrictionRow) -> Result<(), StoreError> {
        q("UPDATE account_restriction SET lifted_at=?2, lifted_by=?3, lift_reason='replaced' WHERE account_id=?1 AND lifted_at IS NULL")
            .bind(row.account_id.as_str())
            .bind(row.created_at)
            .bind(row.created_by.as_str())
            .execute(&self.backend)
            .await?;
        let scopes = serde_json::to_string(&row.scopes).unwrap_or_else(|_| "[]".into());
        q(concat!("INSERT INTO account_restriction(", rcols!(), ") VALUES(?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11,?12,NULL,NULL,NULL)"))
            .bind(row.id.as_str())
            .bind(row.account_id.as_str())
            .bind(row.address.as_str())
            .bind(row.domain.as_str())
            .bind(scopes)
            .bind(row.mail_delivery.as_str())
            .bind(row.reason.as_str())
            .bind(row.source.as_str())
            .bind(row.request_id.clone())
            .bind(row.created_by.as_str())
            .bind(row.created_at)
            .bind(row.expires_at)
            .execute(&self.backend)
            .await?;
        Ok(())
    }

    /// Lift the active restriction of an account. Returns the lifted row.
    pub async fn restriction_lift(
        &self,
        account_id: &str,
        by: &str,
        reason: &str,
        now: i64,
    ) -> Result<Option<RestrictionRow>, StoreError> {
        let Some(active) = self.restriction_active(account_id, now).await? else {
            return Ok(None);
        };
        q("UPDATE account_restriction SET lifted_at=?2, lifted_by=?3, lift_reason=?4 WHERE id=?1 AND lifted_at IS NULL")
            .bind(active.id.as_str())
            .bind(now)
            .bind(by)
            .bind(reason)
            .execute(&self.backend)
            .await?;
        self.restriction_get(&active.id).await
    }

    pub async fn restriction_get(&self, id: &str) -> Result<Option<RestrictionRow>, StoreError> {
        Ok(
            q(concat!("SELECT ", rcols!(), " FROM account_restriction WHERE id=?1"))
                .bind(id)
                .fetch_optional(&self.backend)
                .await?
                .as_ref()
                .map(restriction),
        )
    }

    /// The restriction in force for an account at `now`, if any.
    pub async fn restriction_active(
        &self,
        account_id: &str,
        now: i64,
    ) -> Result<Option<RestrictionRow>, StoreError> {
        let rows = q(concat!("SELECT ", rcols!(), " FROM account_restriction WHERE account_id=?1 AND lifted_at IS NULL ORDER BY created_at DESC"))
            .bind(account_id)
            .fetch_all(&self.backend)
            .await?;
        Ok(rows.iter().map(restriction).find(|r| r.active_at(now)))
    }

    /// Whether processing of `scope` is paused for the account right now. A store
    /// error counts as not restricted so a database fault cannot silently stop an
    /// unrelated feature; callers that must fail closed use
    /// [`Store::restriction_active`].
    pub async fn restriction_blocks(&self, account_id: &str, scope: &str) -> bool {
        let now = chrono::Utc::now().timestamp();
        matches!(
            self.restriction_active(account_id, now).await,
            Ok(Some(r)) if r.covers(scope)
        )
    }

    /// Restrictions, newest first. `active_only` hides lifted and expired ones.
    pub async fn restriction_list(
        &self,
        domain: Option<&str>,
        active_only: bool,
        now: i64,
    ) -> Result<Vec<RestrictionRow>, StoreError> {
        let rows = match domain {
            Some(d) => {
                q(concat!("SELECT ", rcols!(), " FROM account_restriction WHERE domain=?1 ORDER BY created_at DESC LIMIT ?2"))
                    .bind(d)
                    .bind(MAX_ROWS)
                    .fetch_all(&self.backend)
                    .await?
            }
            None => {
                q(concat!("SELECT ", rcols!(), " FROM account_restriction ORDER BY created_at DESC LIMIT ?1"))
                    .bind(MAX_ROWS)
                    .fetch_all(&self.backend)
                    .await?
            }
        };
        Ok(rows
            .iter()
            .map(restriction)
            .filter(|r| !active_only || r.active_at(now))
            .collect())
    }

    /// Erase the restriction history of an account (an erasure request).
    pub async fn restriction_delete_for_account(
        &self,
        account_id: &str,
    ) -> Result<u64, StoreError> {
        Ok(q("DELETE FROM account_restriction WHERE account_id=?1")
            .bind(account_id)
            .execute(&self.backend)
            .await?)
    }

    // ── consent ledger ──────────────────────────────────────────────────────

    pub async fn consent_append(&self, row: &ConsentRecord) -> Result<(), StoreError> {
        q(concat!("INSERT INTO consent_record(", ccols!(), ") VALUES(?1,?2,?3,?4,?5,?6,?7,?8,?9)"))
            .bind(row.id.as_str())
            .bind(row.account_id.as_str())
            .bind(row.domain.as_str())
            .bind(row.purpose.as_str())
            .bind(row.decision.as_str())
            .bind(row.version)
            .bind(row.basis.as_str())
            .bind(row.source.as_str())
            .bind(row.created_at)
            .execute(&self.backend)
            .await?;
        Ok(())
    }

    /// The newest decision of an account for one purpose.
    pub async fn consent_latest(
        &self,
        account_id: &str,
        purpose: &str,
    ) -> Result<Option<ConsentRecord>, StoreError> {
        Ok(q(concat!("SELECT ", ccols!(), " FROM consent_record WHERE account_id=?1 AND purpose=?2 ORDER BY created_at DESC, id DESC LIMIT 1"))
            .bind(account_id)
            .bind(purpose)
            .fetch_optional(&self.backend)
            .await?
            .as_ref()
            .map(consent))
    }

    /// Every receipt of an account, oldest first.
    pub async fn consent_receipts(
        &self,
        account_id: &str,
    ) -> Result<Vec<ConsentRecord>, StoreError> {
        let rows = q(concat!("SELECT ", ccols!(), " FROM consent_record WHERE account_id=?1 ORDER BY created_at, id LIMIT ?2"))
            .bind(account_id)
            .bind(MAX_ROWS)
            .fetch_all(&self.backend)
            .await?;
        Ok(rows.iter().map(consent).collect())
    }

    /// Ledger rows with `from <= created_at < to`, oldest first, capped.
    pub async fn consent_between(
        &self,
        from: i64,
        to: i64,
    ) -> Result<Vec<ConsentRecord>, StoreError> {
        let rows = q(concat!("SELECT ", ccols!(), " FROM consent_record WHERE created_at>=?1 AND created_at<?2 ORDER BY created_at, id LIMIT ?3"))
            .bind(from)
            .bind(to)
            .bind(200_000_i64)
            .fetch_all(&self.backend)
            .await?;
        Ok(rows.iter().map(consent).collect())
    }

    /// Erase the ledger of an account (an erasure request).
    pub async fn consent_delete_for_account(&self, account_id: &str) -> Result<u64, StoreError> {
        Ok(q("DELETE FROM consent_record WHERE account_id=?1")
            .bind(account_id)
            .execute(&self.backend)
            .await?)
    }

    /// Delete ledger rows older than `before` (retention).
    pub async fn consent_purge(&self, before: i64) -> Result<u64, StoreError> {
        Ok(q("DELETE FROM consent_record WHERE created_at<?1")
            .bind(before)
            .execute(&self.backend)
            .await?)
    }

    // ── footprint ───────────────────────────────────────────────────────────

    /// Counts and approximate sizes of the stored data of one account, by
    /// category. Each entry is `(rows, bytes)`; bytes is 0 where the size is not
    /// tracked per row.
    pub async fn privacy_footprint(&self, account_id: &str) -> Result<Footprint, StoreError> {
        const COUNTS: &[(&str, &str)] = &[
            ("mail", "SELECT COUNT(*) AS n, COALESCE(SUM(size),0) AS b FROM messages WHERE account_id=?1"),
            ("contacts", "SELECT COUNT(*) AS n, COALESCE(SUM(LENGTH(vcard_raw)),0) AS b FROM contacts WHERE address_book_id IN (SELECT id FROM address_books WHERE account_id=?1)"),
            ("calendars", "SELECT COUNT(*) AS n, COALESCE(SUM(LENGTH(ical_raw)),0) AS b FROM events WHERE calendar_id IN (SELECT id FROM calendars WHERE account_id=?1)"),
            ("notes", "SELECT COUNT(*) AS n, COALESCE(SUM(LENGTH(body_text_sealed)),0) AS b FROM notes WHERE account_id=?1"),
            ("sessions", "SELECT COUNT(*) AS n, 0 AS b FROM sessions WHERE account_id=?1"),
            ("devices", "SELECT COUNT(*) AS n, 0 AS b FROM remembered_device WHERE account_id=?1"),
            ("signInHistory", "SELECT COUNT(*) AS n, 0 AS b FROM signin_event WHERE account_id=?1"),
            ("push", "SELECT COUNT(*) AS n, 0 AS b FROM push_subscriptions WHERE account_id=?1"),
            ("apiKeys", "SELECT COUNT(*) AS n, 0 AS b FROM api_keys WHERE account_id=?1 AND revoked_at IS NULL"),
            ("rules", "SELECT COUNT(*) AS n, COALESCE(SUM(LENGTH(content)),0) AS b FROM sieve_script WHERE account_id=?1"),
            ("searchEmbeddings", "SELECT COUNT(*) AS n, COALESCE(SUM(LENGTH(vector_sealed)),0) AS b FROM message_embeddings WHERE account_id=?1"),
            ("tracking", "SELECT COUNT(*) AS n, 0 AS b FROM open_tracking WHERE account_id=?1"),
            ("consents", "SELECT COUNT(*) AS n, 0 AS b FROM consent_record WHERE account_id=?1"),
            ("requests", "SELECT COUNT(*) AS n, 0 AS b FROM dsr_request WHERE account_id=?1"),
        ];
        let mut items = BTreeMap::new();
        for (name, sql) in COUNTS {
            let (n, b) = match q(sql).bind(account_id).fetch_optional(&self.backend).await {
                Ok(Some(r)) => (r.get_i64("n"), r.get_i64("b")),
                // A table an older schema lacks counts as empty.
                _ => (0, 0),
            };
            items.insert((*name).to_string(), (n, b));
        }
        let target = format!("account:{account_id}");
        if let Ok(Some(r)) = q("SELECT COUNT(*) AS n, 0 AS b FROM audit_log WHERE actor=?1 OR target=?2")
            .bind(account_id)
            .bind(target)
            .fetch_optional(&self.backend)
            .await
        {
            items.insert("audit".into(), (r.get_i64("n"), 0));
        }
        Ok(Footprint { items })
    }
}
