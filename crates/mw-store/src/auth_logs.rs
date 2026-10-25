//! Bounded authentication history and durable last-authenticated policy identity.
use crate::backend::{Row, q};
use crate::{SigninEvent, Store, StoreError};
use serde::Serialize;
use sha2::{Digest, Sha256};

#[derive(Clone, Copy, Debug)]
pub enum AuthSurface {
    Webmail,
    Admin,
}
impl AuthSurface {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Webmail => "webmail",
            Self::Admin => "admin",
        }
    }
}
#[derive(Clone, Copy, Debug)]
pub struct AuthLogLimits {
    pub retention_days: u32,
    pub max_entries: u32,
}
impl Default for AuthLogLimits {
    fn default() -> Self {
        Self {
            retention_days: 7,
            max_entries: 50_000,
        }
    }
}
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AuthLogEvent {
    #[serde(flatten)]
    pub event: SigninEvent,
    pub surface: String,
    pub backend: Option<String>,
    pub last_seen: i64,
    pub repeat_count: i64,
}
#[derive(Default)]
pub struct AuthLogQuery {
    pub q: Option<String>,
    pub result: Option<String>,
    pub reason: Option<String>,
    pub backend: Option<String>,
    pub surface: Option<String>,
    pub before: Option<(i64, String)>,
    pub limit: u32,
}
fn clean(value: &str, limit: usize) -> String {
    value
        .chars()
        .filter(|c| !c.is_control())
        .take(limit)
        .collect()
}
pub(crate) fn signin_event_row(r: &Row) -> SigninEvent {
    SigninEvent {
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
    }
}
impl Store {
    /// Metadata is bounded before persistence; only successful webmail authentication
    /// updates the identity used by policy resolution. Retention cannot erase it.
    pub async fn record_auth_event(
        &self,
        event: &SigninEvent,
        surface: AuthSurface,
        backend: Option<&str>,
        limits: AuthLogLimits,
    ) -> Result<(), StoreError> {
        let username = clean(&event.username_attempted, 320);
        let ip = clean(&event.ip, 128);
        let agent = clean(&event.user_agent, 512);
        let method = clean(&event.method, 32);
        let reason = clean(&event.reason, 64);
        let backend = backend.map(|s| clean(s, 128));
        let key = if event.result == "success" {
            None
        } else {
            let data = serde_json::json!([
                surface.as_str(),
                event.account_id,
                username,
                ip,
                agent,
                method,
                reason,
                backend,
                event.result,
                event.ts.div_euclid(60)
            ]);
            Some(
                Sha256::digest(data.to_string().as_bytes())
                    .iter()
                    .map(|byte| format!("{byte:02x}"))
                    .collect::<String>(),
            )
        };
        let mut tx = self.backend.begin().await?;
        q("INSERT INTO settings(key,value) VALUES('auth-log:write-lock','0') ON CONFLICT(key) DO NOTHING").execute_tx(&mut tx).await?;
        q("UPDATE settings SET value=value WHERE key='auth-log:write-lock'")
            .execute_tx(&mut tx)
            .await?;
        q("INSERT INTO signin_event(id,account_id,username_attempted,ts,result,method,ip,user_agent,geo_country,reason,surface,backend,last_seen,repeat_count,aggregate_key) VALUES(?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11,?12,?4,1,?13) ON CONFLICT(aggregate_key) DO UPDATE SET last_seen=CASE WHEN signin_event.last_seen>excluded.last_seen THEN signin_event.last_seen ELSE excluded.last_seen END,ts=CASE WHEN signin_event.ts<excluded.ts THEN signin_event.ts ELSE excluded.ts END,repeat_count=CASE WHEN signin_event.repeat_count<1000000 THEN signin_event.repeat_count+1 ELSE signin_event.repeat_count END")
            .bind(&event.id).bind(event.account_id.as_deref()).bind(&username).bind(event.ts).bind(&event.result).bind(method).bind(ip).bind(agent).bind(event.geo_country.as_deref()).bind(reason).bind(surface.as_str()).bind(backend).bind(key).execute_tx(&mut tx).await?;
        if matches!(surface, AuthSurface::Webmail)
            && event.result == "success"
            && let Some(account) = &event.account_id
        {
            q("INSERT INTO signin_identity(account_id,address,seen_at,event_id) VALUES(?1,?2,?3,?4) ON CONFLICT(account_id) DO UPDATE SET address=excluded.address,seen_at=excluded.seen_at,event_id=excluded.event_id WHERE excluded.seen_at>signin_identity.seen_at OR (excluded.seen_at=signin_identity.seen_at AND excluded.event_id>signin_identity.event_id)")
                .bind(account).bind(&username).bind(event.ts).bind(&event.id).execute_tx(&mut tx).await?;
        }
        let cutoff = event
            .ts
            .saturating_sub(i64::from(limits.retention_days.clamp(1, 90)) * 86_400);
        q("DELETE FROM signin_event WHERE last_seen<?1")
            .bind(cutoff)
            .execute_tx(&mut tx)
            .await?;
        q("DELETE FROM signin_event WHERE id IN (SELECT id FROM signin_event ORDER BY last_seen DESC,id DESC LIMIT 2147483647 OFFSET ?1)")
            .bind(i64::from(limits.max_entries.clamp(1,100_000))).execute_tx(&mut tx).await?;
        tx.commit().await?;
        Ok(())
    }
    /// Apply configured retention even when no authentication writes arrive.
    pub async fn prune_auth_events(
        &self,
        now: i64,
        limits: AuthLogLimits,
    ) -> Result<(), StoreError> {
        let mut tx = self.backend.begin().await?;
        q("INSERT INTO settings(key,value) VALUES('auth-log:write-lock','0') ON CONFLICT(key) DO NOTHING").execute_tx(&mut tx).await?;
        q("UPDATE settings SET value=value WHERE key='auth-log:write-lock'")
            .execute_tx(&mut tx)
            .await?;
        q("DELETE FROM signin_event WHERE last_seen<?1")
            .bind(now.saturating_sub(i64::from(limits.retention_days.clamp(1, 90)) * 86400))
            .execute_tx(&mut tx)
            .await?;
        q("DELETE FROM signin_event WHERE id IN (SELECT id FROM signin_event ORDER BY last_seen DESC,id DESC LIMIT 2147483647 OFFSET ?1)").bind(i64::from(limits.max_entries.clamp(1,100000))).execute_tx(&mut tx).await?;
        tx.commit().await?;
        Ok(())
    }
    /// Global privileged view, including attempts with no known account.
    pub async fn auth_log_events(
        &self,
        filter: &AuthLogQuery,
    ) -> Result<Vec<AuthLogEvent>, StoreError> {
        let search = filter.q.as_ref().map(|s| {
            format!(
                "%{}%",
                clean(s, 128)
                    .to_lowercase()
                    .replace('!', "!!")
                    .replace('%', "!%")
                    .replace('_', "!_")
            )
        });
        let before_ts = filter.before.as_ref().map(|x| x.0);
        let before_id = filter.before.as_ref().map(|x| x.1.as_str());
        let rows=q("SELECT * FROM signin_event WHERE (?1 IS NULL OR lower(username_attempted) LIKE ?1 ESCAPE '!' OR lower(COALESCE(account_id,'')) LIKE ?1 ESCAPE '!' OR lower(ip) LIKE ?1 ESCAPE '!') AND (?2 IS NULL OR result=?2) AND (?3 IS NULL OR reason=?3) AND (?4 IS NULL OR backend=?4) AND (?5 IS NULL OR surface=?5) AND (?6 IS NULL OR last_seen<?6 OR (last_seen=?6 AND id<?7)) ORDER BY last_seen DESC,id DESC LIMIT ?8")
            .bind(search).bind(filter.result.as_deref()).bind(filter.reason.as_deref()).bind(filter.backend.as_deref()).bind(filter.surface.as_deref()).bind(before_ts).bind(before_id).bind(i64::from(filter.limit.clamp(1,101))).fetch_all(&self.backend).await?;
        Ok(rows
            .iter()
            .map(|r| AuthLogEvent {
                event: signin_event_row(r),
                surface: r.get_string("surface"),
                backend: r.get_opt_string("backend"),
                last_seen: r.get_i64("last_seen"),
                repeat_count: r.get_i64("repeat_count"),
            })
            .collect())
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    fn event(id: &str, ts: i64, result: &str) -> SigninEvent {
        SigninEvent {
            id: id.into(),
            account_id: Some("account".into()),
            username_attempted: "Person@alias.example".into(),
            ts,
            result: result.into(),
            method: "password".into(),
            ip: "192.0.2.1".into(),
            user_agent: "browser".into(),
            geo_country: None,
            reason: if result == "success" {
                ""
            } else {
                "auth-rejected"
            }
            .into(),
        }
    }
    async fn exercise(store: Store) {
        let limits = AuthLogLimits {
            retention_days: 1,
            max_entries: 3,
        };
        for (id, ts) in [("a", 120), ("b", 121)] {
            store
                .record_auth_event(
                    &event(id, ts, "failure"),
                    AuthSurface::Webmail,
                    Some("imap"),
                    limits,
                )
                .await
                .unwrap();
        }
        let rows = store
            .auth_log_events(&AuthLogQuery {
                limit: 100,
                ..Default::default()
            })
            .await
            .unwrap();
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].repeat_count, 2);
        assert_eq!(rows[0].event.ts, 120);
        assert_eq!(rows[0].last_seen, 121);
        store
            .record_auth_event(
                &event("c", 122, "success"),
                AuthSurface::Webmail,
                Some("imap"),
                limits,
            )
            .await
            .unwrap();
        let mut older = event("d", 121, "success");
        older.username_attempted = "wrong@example.org".into();
        store
            .record_auth_event(&older, AuthSurface::Webmail, None, limits)
            .await
            .unwrap();
        let mut admin = event("e", 200000, "success");
        admin.username_attempted = "admin@example.org".into();
        store
            .record_auth_event(&admin, AuthSurface::Admin, Some("local"), limits)
            .await
            .unwrap();
        assert!(
            store
                .signin_history("account", i64::MAX, None, 100)
                .await
                .unwrap()
                .is_empty()
        );
        assert_eq!(
            store
                .security_account_address("account")
                .await
                .unwrap()
                .as_deref(),
            Some("Person@alias.example")
        );
        assert_eq!(
            store
                .security_account_for_address("person@ALIAS.example")
                .await
                .unwrap()
                .as_deref(),
            Some("account")
        );
        let mut unknown = event("f", 200001, "failure");
        unknown.account_id = None;
        unknown.username_attempted = "unknown@example.org".into();
        store
            .record_auth_event(&unknown, AuthSurface::Webmail, Some("jmap"), limits)
            .await
            .unwrap();
        let found = store
            .auth_log_events(&AuthLogQuery {
                q: Some("UNKNOWN".into()),
                surface: Some("webmail".into()),
                backend: Some("jmap".into()),
                limit: 50,
                ..Default::default()
            })
            .await
            .unwrap();
        assert_eq!(found.len(), 1);
        assert!(found[0].event.account_id.is_none());
        for (id, ts) in [("g", 200002), ("h", 200003), ("i", 200004)] {
            store
                .record_auth_event(
                    &event(id, ts, "success"),
                    AuthSurface::Webmail,
                    None,
                    limits,
                )
                .await
                .unwrap();
        }
        let rows = store
            .auth_log_events(&AuthLogQuery {
                limit: 100,
                ..Default::default()
            })
            .await
            .unwrap();
        assert_eq!(rows.len(), 3);
        let next = store
            .auth_log_events(&AuthLogQuery {
                before: Some((rows[0].last_seen, rows[0].event.id.clone())),
                limit: 100,
                ..Default::default()
            })
            .await
            .unwrap();
        assert_eq!(next.len(), 2);
        assert_eq!(next[0].event.id, "h");
        let literal = store
            .auth_log_events(&AuthLogQuery {
                q: Some("%".into()),
                limit: 100,
                ..Default::default()
            })
            .await
            .unwrap();
        assert!(literal.is_empty());
        store.prune_auth_events(400000, limits).await.unwrap();
        assert!(
            store
                .auth_log_events(&AuthLogQuery {
                    limit: 100,
                    ..Default::default()
                })
                .await
                .unwrap()
                .is_empty()
        );
        assert_eq!(
            store
                .security_account_address("account")
                .await
                .unwrap()
                .as_deref(),
            Some("Person@alias.example")
        );
    }
    #[tokio::test]
    async fn bounded_history_preserves_identity_sqlite() {
        exercise(
            Store::open_in_memory(crate::ServerKey::generate())
                .await
                .unwrap(),
        )
        .await;
    }
    #[tokio::test]
    async fn bounded_history_preserves_identity_postgres() {
        let Ok(dsn) = std::env::var("MW_AUTH_LOG_TEST_PG_DSN") else {
            return;
        };
        let pool = sqlx::postgres::PgPoolOptions::new()
            .max_connections(1)
            .connect(&dsn)
            .await
            .unwrap();
        let schema = format!("auth_logs_{}", rand::random::<u64>());
        sqlx::query(&format!("CREATE SCHEMA {schema}"))
            .execute(&pool)
            .await
            .unwrap();
        let sep = if dsn.contains('?') { '&' } else { '?' };
        let scoped = format!("{dsn}{sep}options=-c%20search_path%3D{schema}");
        let result = tokio::spawn(async move {
            exercise(
                Store::open_postgres(&scoped, crate::ServerKey::generate())
                    .await
                    .unwrap(),
            )
            .await;
        })
        .await;
        sqlx::query(&format!("DROP SCHEMA {schema} CASCADE"))
            .execute(&pool)
            .await
            .unwrap();
        result.unwrap();
    }
}
