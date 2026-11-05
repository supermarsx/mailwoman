//! Storage for user retention rules: the rule rows, a per-account settings row
//! and the run history. A run row holds counts only. A rule's match, action and
//! exclusions are in `body_json`, which the server validates.

use crate::{Store, StoreError, q};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuleRow {
    pub id: String,
    pub account_id: String,
    pub name: String,
    pub enabled: bool,
    pub sort_order: i64,
    pub template: Option<String>,
    pub body_json: String,
    pub created_at: i64,
    pub updated_at: i64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AccountRow {
    pub account_id: String,
    pub enabled: bool,
    pub trash_first: bool,
    /// Days in Trash before purge; negative uses the server's value.
    pub undo_days: i64,
    pub paused_until: i64,
    pub folder_defaults_json: String,
    pub last_run_at: i64,
    pub updated_at: i64,
}

impl AccountRow {
    /// The row an account without one has.
    pub fn fresh(account_id: &str) -> Self {
        Self {
            account_id: account_id.to_string(),
            enabled: true,
            trash_first: true,
            undo_days: -1,
            paused_until: 0,
            folder_defaults_json: "[]".into(),
            last_run_at: 0,
            updated_at: 0,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RunRow {
    pub id: String,
    pub account_id: String,
    pub started_at: i64,
    pub finished_at: i64,
    pub dry_run: bool,
    pub triggered_by: String,
    pub matched: i64,
    pub acted: i64,
    pub errors: i64,
    pub counts_json: String,
}

/// Counts for the administrator's overview.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct RuleStats {
    pub users_with_rules: i64,
    pub rules: i64,
    pub enabled_rules: i64,
    pub runs_since: i64,
    pub matched_since: i64,
    pub acted_since: i64,
}

fn rule(r: &crate::backend::Row) -> RuleRow {
    RuleRow {
        id: r.get_string("id"),
        account_id: r.get_string("account_id"),
        name: r.get_string("name"),
        enabled: r.get_i64("enabled") != 0,
        sort_order: r.get_i64("sort_order"),
        template: r.get_opt_string("template"),
        body_json: r.get_string("body_json"),
        created_at: r.get_i64("created_at"),
        updated_at: r.get_i64("updated_at"),
    }
}

fn account(r: &crate::backend::Row) -> AccountRow {
    AccountRow {
        account_id: r.get_string("account_id"),
        enabled: r.get_i64("enabled") != 0,
        trash_first: r.get_i64("trash_first") != 0,
        undo_days: r.get_i64("undo_days"),
        paused_until: r.get_i64("paused_until"),
        folder_defaults_json: r.get_string("folder_defaults_json"),
        last_run_at: r.get_i64("last_run_at"),
        updated_at: r.get_i64("updated_at"),
    }
}

fn run(r: &crate::backend::Row) -> RunRow {
    RunRow {
        id: r.get_string("id"),
        account_id: r.get_string("account_id"),
        started_at: r.get_i64("started_at"),
        finished_at: r.get_i64("finished_at"),
        dry_run: r.get_i64("dry_run") != 0,
        triggered_by: r.get_string("triggered_by"),
        matched: r.get_i64("matched"),
        acted: r.get_i64("acted"),
        errors: r.get_i64("errors"),
        counts_json: r.get_string("counts_json"),
    }
}

impl Store {
    /// An account's rules in evaluation order.
    pub async fn retention_rules(&self, account_id: &str) -> Result<Vec<RuleRow>, StoreError> {
        let rows = q("SELECT id,account_id,name,enabled,sort_order,template,body_json,created_at,updated_at FROM user_retention_rule WHERE account_id=?1 ORDER BY sort_order, created_at, id")
            .bind(account_id)
            .fetch_all(&self.backend)
            .await?;
        Ok(rows.iter().map(rule).collect())
    }

    pub async fn retention_rule_count(&self, account_id: &str) -> Result<i64, StoreError> {
        Ok(q("SELECT COUNT(*) AS n FROM user_retention_rule WHERE account_id=?1")
            .bind(account_id)
            .fetch_one(&self.backend)
            .await?
            .get_i64("n"))
    }

    /// Insert or replace one rule of the account named in the row. A row id that
    /// belongs to another account is left alone.
    pub async fn retention_rule_put(&self, r: &RuleRow) -> Result<(), StoreError> {
        q("INSERT INTO user_retention_rule(id,account_id,name,enabled,sort_order,template,body_json,created_at,updated_at) VALUES(?1,?2,?3,?4,?5,?6,?7,?8,?9) ON CONFLICT(id) DO UPDATE SET name=excluded.name,enabled=excluded.enabled,sort_order=excluded.sort_order,template=excluded.template,body_json=excluded.body_json,updated_at=excluded.updated_at WHERE user_retention_rule.account_id=excluded.account_id")
            .bind(r.id.as_str())
            .bind(r.account_id.as_str())
            .bind(r.name.as_str())
            .bind(r.enabled as i64)
            .bind(r.sort_order)
            .bind(r.template.clone())
            .bind(r.body_json.as_str())
            .bind(r.created_at)
            .bind(r.updated_at)
            .execute(&self.backend)
            .await?;
        Ok(())
    }

    /// Delete one rule; false when the account has no such rule.
    pub async fn retention_rule_delete(&self, account_id: &str, id: &str) -> Result<bool, StoreError> {
        let n = q("DELETE FROM user_retention_rule WHERE account_id=?1 AND id=?2")
            .bind(account_id)
            .bind(id)
            .execute(&self.backend)
            .await?;
        Ok(n > 0)
    }

    /// Remove every rule, setting and run of an account (account erasure).
    pub async fn retention_forget_account(&self, account_id: &str) -> Result<(), StoreError> {
        for sql in [
            "DELETE FROM user_retention_rule WHERE account_id=?1",
            "DELETE FROM user_retention_account WHERE account_id=?1",
            "DELETE FROM user_retention_run WHERE account_id=?1",
        ] {
            q(sql).bind(account_id).execute(&self.backend).await?;
        }
        Ok(())
    }

    pub async fn retention_account(&self, account_id: &str) -> Result<AccountRow, StoreError> {
        let r = q("SELECT account_id,enabled,trash_first,undo_days,paused_until,folder_defaults_json,last_run_at,updated_at FROM user_retention_account WHERE account_id=?1")
            .bind(account_id)
            .fetch_optional(&self.backend)
            .await?;
        Ok(r.as_ref().map(account).unwrap_or_else(|| AccountRow::fresh(account_id)))
    }

    pub async fn retention_account_put(&self, a: &AccountRow) -> Result<(), StoreError> {
        q("INSERT INTO user_retention_account(account_id,enabled,trash_first,undo_days,paused_until,folder_defaults_json,last_run_at,updated_at) VALUES(?1,?2,?3,?4,?5,?6,?7,?8) ON CONFLICT(account_id) DO UPDATE SET enabled=excluded.enabled,trash_first=excluded.trash_first,undo_days=excluded.undo_days,paused_until=excluded.paused_until,folder_defaults_json=excluded.folder_defaults_json,last_run_at=excluded.last_run_at,updated_at=excluded.updated_at")
            .bind(a.account_id.as_str())
            .bind(a.enabled as i64)
            .bind(a.trash_first as i64)
            .bind(a.undo_days)
            .bind(a.paused_until)
            .bind(a.folder_defaults_json.as_str())
            .bind(a.last_run_at)
            .bind(a.updated_at)
            .execute(&self.backend)
            .await?;
        Ok(())
    }

    /// Accounts with at least one enabled rule, for the scheduled run.
    pub async fn retention_accounts_with_rules(&self, limit: i64) -> Result<Vec<String>, StoreError> {
        let rows = q("SELECT DISTINCT account_id FROM user_retention_rule WHERE enabled=1 ORDER BY account_id LIMIT ?1")
            .bind(limit)
            .fetch_all(&self.backend)
            .await?;
        Ok(rows.iter().map(|r| r.get_string("account_id")).collect())
    }

    pub async fn retention_run_put(&self, r: &RunRow) -> Result<(), StoreError> {
        q("INSERT INTO user_retention_run(id,account_id,started_at,finished_at,dry_run,triggered_by,matched,acted,errors,counts_json) VALUES(?1,?2,?3,?4,?5,?6,?7,?8,?9,?10)")
            .bind(r.id.as_str())
            .bind(r.account_id.as_str())
            .bind(r.started_at)
            .bind(r.finished_at)
            .bind(r.dry_run as i64)
            .bind(r.triggered_by.as_str())
            .bind(r.matched)
            .bind(r.acted)
            .bind(r.errors)
            .bind(r.counts_json.as_str())
            .execute(&self.backend)
            .await?;
        Ok(())
    }

    /// An account's newest runs, newest first.
    pub async fn retention_runs(&self, account_id: &str, limit: i64) -> Result<Vec<RunRow>, StoreError> {
        let rows = q("SELECT id,account_id,started_at,finished_at,dry_run,triggered_by,matched,acted,errors,counts_json FROM user_retention_run WHERE account_id=?1 ORDER BY started_at DESC, id LIMIT ?2")
            .bind(account_id)
            .bind(limit.clamp(1, 200))
            .fetch_all(&self.backend)
            .await?;
        Ok(rows.iter().map(run).collect())
    }

    /// Delete run history older than `before` (unix seconds).
    pub async fn retention_runs_prune(&self, before: i64) -> Result<u64, StoreError> {
        Ok(q("DELETE FROM user_retention_run WHERE started_at < ?1")
            .bind(before)
            .execute(&self.backend)
            .await?)
    }

    /// Overview counts for the administrator; real (not dry) runs since `since`.
    pub async fn retention_rule_stats(&self, since: i64) -> Result<RuleStats, StoreError> {
        let r = q("SELECT COUNT(DISTINCT account_id) AS users, COUNT(*) AS rules, COALESCE(SUM(enabled),0) AS on_n FROM user_retention_rule")
            .fetch_one(&self.backend)
            .await?;
        let u = q("SELECT COUNT(*) AS runs, COALESCE(SUM(matched),0) AS m, COALESCE(SUM(acted),0) AS a FROM user_retention_run WHERE dry_run=0 AND started_at >= ?1")
            .bind(since)
            .fetch_one(&self.backend)
            .await?;
        Ok(RuleStats {
            users_with_rules: r.get_i64("users"),
            rules: r.get_i64("rules"),
            enabled_rules: r.get_i64("on_n"),
            runs_since: u.get_i64("runs"),
            matched_since: u.get_i64("m"),
            acted_since: u.get_i64("a"),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ServerKey;

    #[tokio::test]
    async fn rules_accounts_and_runs_round_trip() {
        let dir = std::env::temp_dir().join(format!("mw-ur-{}", uuid::Uuid::new_v4()));
        std::fs::create_dir_all(&dir).unwrap();
        let s = Store::open(
            dir.join("s.sqlite").to_str().unwrap(),
            ServerKey::from_bytes(&[6u8; 32]).unwrap(),
        )
        .await
        .unwrap();
        let mut r = RuleRow {
            id: "r1".into(),
            account_id: "a1".into(),
            name: "News".into(),
            enabled: true,
            sort_order: 1,
            template: None,
            body_json: "{}".into(),
            created_at: 1,
            updated_at: 1,
        };
        s.retention_rule_put(&r).await.unwrap();
        r.name = "News 2".into();
        r.enabled = false;
        s.retention_rule_put(&r).await.unwrap();
        // Another account cannot take the id over.
        let mut other = r.clone();
        other.account_id = "a2".into();
        other.name = "stolen".into();
        s.retention_rule_put(&other).await.unwrap();
        let got = s.retention_rules("a1").await.unwrap();
        assert_eq!(got.len(), 1);
        assert_eq!((got[0].name.as_str(), got[0].enabled), ("News 2", false));
        assert_eq!(s.retention_rule_count("a2").await.unwrap(), 0);
        assert!(s.retention_accounts_with_rules(10).await.unwrap().is_empty());
        r.enabled = true;
        s.retention_rule_put(&r).await.unwrap();
        assert_eq!(s.retention_accounts_with_rules(10).await.unwrap(), vec!["a1"]);

        assert_eq!(s.retention_account("a1").await.unwrap(), AccountRow::fresh("a1"));
        let mut a = AccountRow::fresh("a1");
        a.undo_days = 3;
        s.retention_account_put(&a).await.unwrap();
        assert_eq!(s.retention_account("a1").await.unwrap().undo_days, 3);

        for (i, dry) in [(1, false), (2, true)] {
            s.retention_run_put(&RunRow {
                id: format!("run{i}"),
                account_id: "a1".into(),
                started_at: 100 * i,
                finished_at: 100 * i + 1,
                dry_run: dry,
                triggered_by: "scheduler".into(),
                matched: 10,
                acted: 7,
                errors: 0,
                counts_json: "{}".into(),
            })
            .await
            .unwrap();
        }
        let st = s.retention_rule_stats(0).await.unwrap();
        assert_eq!((st.users_with_rules, st.rules, st.runs_since, st.acted_since), (1, 1, 1, 7));
        assert_eq!(s.retention_runs("a1", 10).await.unwrap()[0].id, "run2");
        assert_eq!(s.retention_runs_prune(150).await.unwrap(), 1);
        s.retention_forget_account("a1").await.unwrap();
        assert!(s.retention_rules("a1").await.unwrap().is_empty());
        std::fs::remove_dir_all(dir).ok();
    }
}
