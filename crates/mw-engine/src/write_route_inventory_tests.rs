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
