//! t50hp-e19 (e23 inventory): every SQL statement and commit of the JMAP write
//! routes, through `Engine::handle_jmap`, on SQLite and PostgreSQL. Report
//! only (prints `WRITE_INVENTORY` lines); the budgets come later.
use super::*;
use std::sync::atomic::{AtomicU32, Ordering};

/// A backend that accepts every write: flags, moves (UIDPLUS), appends,
/// expunges. No reads beyond what the routes ask for.
struct WriteBackend(AtomicU32);

#[async_trait::async_trait]
impl AccountBackend for WriteBackend {
    async fn capabilities(&self) -> crate::backend::Result<crate::backend::BackendCaps> {
        Err(crate::backend::EngineError::Unsupported("capabilities".into()))
    }
    async fn list_mailboxes(&self) -> crate::backend::Result<Vec<crate::backend::RawMailbox>> {
        Ok(Vec::new())
    }
    async fn sync_mailbox(
        &self,
        _: &crate::backend::RawMailboxRef,
        _: &crate::backend::SyncCursor,
    ) -> crate::backend::Result<crate::backend::MailboxDelta> {
        Err(crate::backend::EngineError::Unsupported("sync".into()))
    }
    async fn fetch_raw(&self, _: &[crate::backend::MessageRef]) -> crate::backend::Result<Vec<crate::backend::RawMessage>> {
        Err(crate::backend::EngineError::Unsupported("fetch".into()))
    }
    async fn store_flags(
        &self,
        _: &[crate::backend::MessageRef],
        _: &[crate::backend::Flag],
        _: &[crate::backend::Flag],
    ) -> crate::backend::Result<()> {
        Ok(())
    }
    async fn move_messages(
        &self,
        refs: &[crate::backend::MessageRef],
        to: &crate::backend::RawMailboxRef,
    ) -> crate::backend::Result<crate::backend::MoveOutcome> {
        let uids = refs.iter().map(|_| self.0.fetch_add(1, Ordering::Relaxed)).collect();
        Ok(crate::backend::MoveOutcome::Uidplus { uidvalidity: to.uidvalidity, uids })
    }
    async fn append(
        &self,
        mbox: &crate::backend::RawMailboxRef,
        _: &[u8],
        _: &[crate::backend::Flag],
    ) -> crate::backend::Result<crate::backend::MessageRef> {
        Ok(crate::backend::MessageRef::Imap {
            mailbox: mbox.clone(),
            uidvalidity: mbox.uidvalidity,
            uid: self.0.fetch_add(1, Ordering::Relaxed),
        })
    }
    async fn expunge_messages(&self, _: &[crate::backend::MessageRef]) -> crate::backend::Result<()> {
        Ok(())
    }
    async fn watch(&self, _: crate::backend::ChangeSink) -> crate::backend::Result<crate::backend::WatchHandle> {
        Err(crate::backend::EngineError::Unsupported("watch".into()))
    }
}

fn shape(sql: &str) -> String {
    sql.split_whitespace().collect::<Vec<_>>().join(" ").chars().take(160).collect()
}

async fn mailbox(hot: &Hot, role: &str) -> String {
    let response = hot
        .send(&hot.a, vec![json!(["Mailbox/get", {"accountId": hot.a, "properties": ["id", "role"]}, "m"])])
        .await;
    method_responses(&response)[0][1]["list"]
        .as_array()
        .unwrap()
        .iter()
        .find(|m| m["role"] == role)
        .unwrap()["id"]
        .as_str()
        .unwrap()
        .to_string()
}

async fn report(hot: &Hot, backend: &str, route: &str, calls: Vec<Value>) -> Value {
    let (response, stmts) = counted(&hot.threads, hot.send(&hot.a, calls)).await;
    let all: Vec<String> = stmts.0.iter().map(|s| shape(&s.sql)).collect();
    let commits = all.iter().filter(|s| s.eq_ignore_ascii_case("COMMIT")).count();
    let begins = all.iter().filter(|s| s.to_ascii_uppercase().starts_with("BEGIN")).count();
    println!(
        "WRITE_INVENTORY {}",
        json!({"backend": backend, "route": route, "statements": all.len(), "begins": begins, "commits": commits, "sql": all})
    );
    // Budgets (t50hp-e19, after slice 1 + W11): SQLite statements, PostgreSQL
    // round trips (statements + transactions; BEGIN is not recorded). Lower
    // them when a change saves more; never raise them silently.
    let budget = match (backend, route) {
        ("sqlite", "keywords") => Some(11),
        ("sqlite", "move") => Some(24),
        ("sqlite", "destroy") => Some(18),
        ("sqlite", "goal-mix-write (2 set + 2 get)") => Some(23),
        ("postgres", "keywords") => Some(11),
        ("postgres", "move") => Some(15),
        ("postgres", "destroy") => Some(18),
        ("postgres", "goal-mix-write (2 set + 2 get)") => Some(24),
        _ => None,
    };
    if let Some(budget) = budget {
        let cost = if backend == "postgres" { all.len() + commits } else { all.len() };
        assert!(cost <= budget, "{backend} {route}: {cost} > budget {budget}: {all:#?}");
    }
    response
}

async fn inventory(hot: Hot, backend: &str) {
    let a = hot.a.clone();
    for account in [&hot.a, &hot.b] {
        hot.engine.register_backend(
            account.clone(),
            crate::account::AccountRuntime::new(
                Arc::new(WriteBackend(AtomicU32::new(10_000))) as Arc<dyn AccountBackend>,
                Arc::new(NoSubmit) as Arc<dyn crate::MailSubmitter>,
                "owner@hot.invalid",
            ),
        );
    }
    let drafts = mailbox(&hot, "drafts").await;
    let sent = mailbox(&hot, "sent").await;
    let (k, m, d) = (hot.a_ids[0].clone(), hot.a_ids[1].clone(), hot.a_ids[2].clone());
    // Warm the caches the routes use (state, mailboxes) once.
    hot.send(&a, vec![json!(["Email/get", {"accountId": a, "ids": [k], "properties": ["keywords"]}, "w"])]).await;
    let r = report(&hot, backend, "keywords", vec![json!(["Email/set", {"accountId": a, "update": {k.clone(): {"keywords/$seen": true}}}, "s"])]).await;
    assert!(method_responses(&r)[0][1]["updated"].get(&k).is_some(), "{r}");
    let r = report(&hot, backend, "move", vec![json!(["Email/set", {"accountId": a, "update": {m.clone(): {"mailboxIds": {sent.clone(): true}}}}, "s"])]).await;
    assert!(method_responses(&r)[0][1]["updated"].get(&m).is_some(), "{r}");
    let r = report(&hot, backend, "destroy", vec![json!(["Email/set", {"accountId": a, "destroy": [d]}, "s"])]).await;
    assert!(method_responses(&r)[0][1]["destroyed"].as_array().is_some_and(|v| !v.is_empty()), "{r}");
    let r = report(&hot, backend, "draft-create", vec![json!(["Email/set", {"accountId": a, "create": {"c1": {
        "mailboxIds": {drafts.clone(): true}, "keywords": {"$draft": true},
        "from": [{"email": "owner@hot.invalid"}], "to": [{"email": "friend@hot.invalid"}],
        "subject": "inventory", "textBody": [{"partId": "t"}], "bodyValues": {"t": {"value": "hello"}}}}}, "s"])]).await;
    assert!(method_responses(&r)[0][1]["created"].get("c1").is_some(), "{r}");
    // The goal-mix write request: two Email/set with an Email/get after each.
    let (x, y) = (hot.a_ids[3].clone(), hot.a_ids[4].clone());
    let get = |id: &str, call: &str| json!(["Email/get", {"accountId": a, "ids": [id], "properties": ["id", "keywords", "mailboxIds"]}, call]);
    report(&hot, backend, "goal-mix-write (2 set + 2 get)", vec![
        json!(["Email/set", {"accountId": a, "update": {x.clone(): {"keywords/$flagged": true}}}, "s1"]),
        get(&x, "g1"),
        json!(["Email/set", {"accountId": a, "update": {y.clone(): {"keywords/$seen": true}}}, "s2"]),
        get(&y, "g2"),
    ])
    .await;
}

#[tokio::test]
async fn write_route_inventory_sqlite() {
    inventory(Hot::sqlite().await, "sqlite").await;
}

#[tokio::test]
async fn write_route_inventory_postgres() {
    let Some(hot) = Hot::pg().await else {
        mw_test_gate::skip("DATABASE_URL_PG and MW_E14_PG_DSN unset: write route inventory on PostgreSQL");
        return;
    };
    inventory(hot, "postgres").await;
}

/// Report-only cost run (ignored): `MW_E19_PG_COST=1`, PostgreSQL only.
/// 200 single-id keyword Email/set requests (toggling $seen over 20 ids) and
/// 100 single-id moves (alternating Sent/INBOX), each through handle_jmap.
/// Prints WAL bytes per request; database CPU is read outside (container
/// cgroup cpu.stat before/after).
#[tokio::test]
#[ignore = "explicit PostgreSQL cost run; set MW_E19_PG_COST=1"]
async fn write_route_pg_cost() {
    assert_eq!(std::env::var("MW_E19_PG_COST").as_deref(), Ok("1"));
    let Some(hot) = Hot::pg().await else { panic!("DATABASE_URL_PG required") };
    for account in [&hot.a, &hot.b] {
        hot.engine.register_backend(
            account.clone(),
            crate::account::AccountRuntime::new(
                Arc::new(WriteBackend(AtomicU32::new(10_000))) as Arc<dyn AccountBackend>,
                Arc::new(NoSubmit) as Arc<dyn crate::MailSubmitter>,
                "owner@hot.invalid",
            ),
        );
    }
    let a = hot.a.clone();
    let inbox = mailbox(&hot, "inbox").await;
    let sent = mailbox(&hot, "sent").await;
    let raw = sqlx::postgres::PgPoolOptions::new()
        .max_connections(1)
        .connect(&mw_test_gate::pg_dsn().unwrap())
        .await
        .unwrap();
    let lsn = || async {
        sqlx::query_scalar::<_, i64>("SELECT CAST(pg_current_wal_lsn() - '0/0'::pg_lsn AS BIGINT)")
            .fetch_one(&raw)
            .await
            .unwrap()
    };
    // Database CPU of the server container (MW_E19_PG_CONTAINER), from its cgroup.
    let cpu = || -> i64 {
        let Ok(name) = std::env::var("MW_E19_PG_CONTAINER") else { return 0 };
        let out = std::process::Command::new("docker").args(["exec", &name, "cat", "/sys/fs/cgroup/cpu.stat"]).env("MSYS_NO_PATHCONV", "1").output().unwrap();
        String::from_utf8_lossy(&out.stdout).lines().find_map(|l| l.strip_prefix("usage_usec ").map(|v| v.trim().parse().unwrap())).unwrap_or(0)
    };
    let ids: Vec<String> = hot.a_ids[..20].to_vec();
    let cpu0 = cpu();
    let start = lsn().await;
    let started = std::time::Instant::now();
    for n in 0..200usize {
        let id = &ids[n % ids.len()];
        let on = (n / ids.len()) % 2 == 0;
        let r = hot.send(&a, vec![json!(["Email/set", {"accountId": a, "update": {id.clone(): {"keywords/$seen": if on { json!(true) } else { Value::Null }}}}, "s"])]).await;
        assert!(method_responses(&r)[0][1]["updated"].get(id).is_some(), "{r}");
    }
    let keywords_elapsed = started.elapsed();
    let mid = lsn().await;
    let cpu1 = cpu();
    let started = std::time::Instant::now();
    for n in 0..100usize {
        let id = &ids[n % ids.len()];
        let to = if (n / ids.len()) % 2 == 0 { &sent } else { &inbox };
        let r = hot.send(&a, vec![json!(["Email/set", {"accountId": a, "update": {id.clone(): {"mailboxIds": {to.clone(): true}}}}, "s"])]).await;
        assert!(method_responses(&r)[0][1]["updated"].get(id).is_some(), "{r}");
    }
    let moves_elapsed = started.elapsed();
    let end = lsn().await;
    let cpu2 = cpu();
    println!(
        "WRITE_COST {}",
        json!({"keywordRequests": 200, "keywordWalBytesPerRequest": (mid - start) as f64 / 200.0,
               "keywordMicrosPerRequest": keywords_elapsed.as_micros() as f64 / 200.0,
               "moveRequests": 100, "moveWalBytesPerRequest": (end - mid) as f64 / 100.0,
               "moveMicrosPerRequest": moves_elapsed.as_micros() as f64 / 100.0,
               "keywordDbCpuMicrosPerRequest": (cpu1 - cpu0) as f64 / 200.0,
               "moveDbCpuMicrosPerRequest": (cpu2 - cpu1) as f64 / 100.0})
    );
}
