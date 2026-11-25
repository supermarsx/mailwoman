//! t50hp-e31 (design part 7 b-e, C6): the planner end to end, through
//! `Email/query`, against the brute-force oracle of the differential test.
//!
//! * stale index: flags, labels, pins and follow-ups changed in SQL only (the
//!   index keeps the old values); today's path follows the index, the planner
//!   follows SQL. Proves the index's mutable fields are not read any more.
//! * fail closed: projection triggers missing or disabled, an SQL error, and a
//!   refusing index: an error, never rows.
//! * a lowered id-set cap: the filter strategy, the complement and
//!   `unsupportedFilter`.
//! * S3-a: the unread count's limit-1 id from SQL against the index's.
use super::query_differential_tests::{
    Account, Row, SortSpec, WINDOWS, World, check_path, singles, sorts, text_filter, text_forms,
};
use super::*;
use crate::query_plan::{CAP, COMBINE_CAP, LEGACY, TAKEN, Taken, UNCACHED_PROJECTION_CHECK};
use mw_store::{ServerKey, Store};
use std::collections::BTreeSet;

fn plain_sort() -> SortSpec {
    sorts().swap_remove(0)
}

async fn query(w: &World, acct: &Account, filter: Value, legacy: bool) -> Value {
    LEGACY.with(|l| l.set(legacy));
    let r = w
        .send(&acct.id, json!([["Email/query", {"accountId": acct.id, "filter": filter, "calculateTotal": true}, "q"]]))
        .await;
    LEGACY.with(|l| l.set(false));
    r["methodResponses"][0].clone()
}

fn id_set(resp: &Value) -> BTreeSet<String> {
    resp[1]["ids"]
        .as_array()
        .unwrap_or_else(|| panic!("{resp}"))
        .iter()
        .map(|v| v.as_str().unwrap().to_owned())
        .collect()
}

fn oracle_set(rows: &[Row], pred: impl Fn(&Row) -> bool) -> BTreeSet<String> {
    rows.iter()
        .filter(|r| pred(r))
        .map(|r| r.id.clone())
        .collect()
}

/// Every filter of the differential matrix on the planner, against the oracle.
async fn planner_matches_oracle(w: &World, which: char, context: &str) -> usize {
    let acct = w.account(which);
    let rows = w.rows(acct).await;
    let mut filters: Vec<String> = singles().into_iter().map(str::to_owned).collect();
    filters.extend(text_forms().into_iter().map(text_filter));
    let mut n = 0;
    for f in &filters {
        for win in [WINDOWS[0], WINDOWS[1], WINDOWS[5]] {
            check_path(w, acct, &rows, f, &plain_sort(), win, context, false, false)
                .await
                .unwrap();
            n += 1;
        }
        for s in &sorts()[1..] {
            check_path(w, acct, &rows, f, s, WINDOWS[0], context, false, false)
                .await
                .unwrap();
            n += 1;
        }
    }
    n
}

async fn flags_of(w: &World, id: &str) -> Vec<Flag> {
    let m = w
        .engine
        .store()
        .get_messages(&[id.to_owned()])
        .await
        .unwrap()
        .remove(0)
        .unwrap();
    crate::mapping::flags_from_json(&m.flags_json)
}

async fn set_flags_sql_only(w: &World, id: &str, flags: Vec<Flag>) {
    w.engine
        .store()
        .set_flags(id, &serde_json::to_string(&flags).unwrap())
        .await
        .unwrap();
}

/// D: the index's keyword, pin and follow-up fields go stale (SQL writes with
/// no index publication); queries follow SQL, today's path follows the index.
async fn stale_index_is_not_read(store: Store) {
    let w = World::new(store, 30).await;
    let acct = &w.a;
    let rows = w.rows(acct).await;
    let mut used = BTreeSet::new();
    let mut pick = |pred: &dyn Fn(&Row) -> bool| {
        let r = rows
            .iter()
            .find(|r| !used.contains(&r.id) && pred(r))
            .expect("corpus has one")
            .clone();
        used.insert(r.id.clone());
        r
    };
    let read = pick(&|r| r.keywords.contains("$seen"));
    let unread = pick(&|r| !r.keywords.contains("$seen"));
    let unlabelled = pick(&|r| !r.keywords.contains("work"));
    let flagged = pick(&|r| r.keywords.contains("$flagged"));
    let unpinned = pick(&|r| !r.pinned);
    let no_follow_up = pick(&|r| !r.keywords.contains(mw_search::KW_FOLLOWUP_ACTIVE));
    let store = w.engine.store();
    // Flags: read -> unread, unread -> read, + label, - $flagged.
    let mut f = flags_of(&w, &read.id).await;
    f.retain(|x| *x != Flag::Seen);
    set_flags_sql_only(&w, &read.id, f).await;
    let mut f = flags_of(&w, &unread.id).await;
    f.push(Flag::Seen);
    set_flags_sql_only(&w, &unread.id, f).await;
    let mut f = flags_of(&w, &unlabelled.id).await;
    f.push(Flag::Keyword("work".into()));
    set_flags_sql_only(&w, &unlabelled.id, f).await;
    let mut f = flags_of(&w, &flagged.id).await;
    f.retain(|x| *x != Flag::Flagged);
    set_flags_sql_only(&w, &flagged.id, f).await;
    // message_meta: a pin and a follow-up, no publication.
    let mut meta = store
        .get_message_meta(&unpinned.id)
        .await
        .unwrap()
        .unwrap_or_default();
    meta.pinned = true;
    store
        .upsert_message_meta(&unpinned.id, &meta)
        .await
        .unwrap();
    let mut meta = store
        .get_message_meta(&no_follow_up.id)
        .await
        .unwrap()
        .unwrap_or_default();
    meta.follow_up_at = Some("2026-09-01T00:00:00Z".into());
    store
        .upsert_message_meta(&no_follow_up.id, &meta)
        .await
        .unwrap();
    // The index really is stale: today's path answers the old values.
    let now = w.rows(acct).await;
    let checks: Vec<(Value, BTreeSet<String>)> = vec![
        (
            json!({"notKeyword": "$seen"}),
            oracle_set(&now, |r| !r.keywords.contains("$seen")),
        ),
        (
            json!({"hasKeyword": "work"}),
            oracle_set(&now, |r| r.keywords.contains("work")),
        ),
        (
            json!({"hasKeyword": "$flagged"}),
            oracle_set(&now, |r| r.keywords.contains("$flagged")),
        ),
        (
            json!({"text": "pinned:true"}),
            oracle_set(&now, |r| r.pinned),
        ),
        (
            json!({"hasKeyword": "$mw-followup-active"}),
            oracle_set(&now, |r| r.keywords.contains(mw_search::KW_FOLLOWUP_ACTIVE)),
        ),
        (
            json!({"text": "is:unread alpha OR is:unread bravo OR is:unread charlie OR is:unread delta OR is:unread echo OR is:unread foxtrot"}),
            oracle_set(&now, |r| !r.keywords.contains("$seen")),
        ),
    ];
    for (filter, want) in &checks {
        let old = id_set(&query(&w, acct, filter.clone(), true).await);
        let new = id_set(&query(&w, acct, filter.clone(), false).await);
        assert_ne!(
            &old, want,
            "{filter}: the index is not stale, the test proves nothing"
        );
        assert_eq!(&new, want, "{filter}: the planner must follow SQL");
    }
    // And every matrix query equals the oracle (which reads SQL).
    let n = planner_matches_oracle(&w, 'a', "stale index").await;
    eprintln!(
        "t50hp-e31 stale-index: {n} planner compares, {} old-path differences shown",
        checks.len()
    );
    // The unread count's id too (S3-a, now from SQL).
    for (_, mb) in &acct.folders {
        let r = w
            .send(&acct.id, json!([["Email/query", {"accountId": acct.id, "filter": {"inMailbox": mb, "notKeyword": "$seen"}, "limit": 1, "calculateTotal": true}, "u"]]))
            .await;
        let body = &r["methodResponses"][0][1];
        let unread_here: Vec<&Row> = now
            .iter()
            .filter(|r| &r.mailbox == mb && !r.keywords.contains("$seen"))
            .collect();
        assert_eq!(body["total"], json!(unread_here.len()), "{r}");
        if let Some(id) = body["ids"][0].as_str() {
            let row = unread_here
                .iter()
                .find(|r| r.id == id)
                .unwrap_or_else(|| panic!("S3-a id {id} is not unread in SQL: {r}"));
            assert_eq!(row.date, unread_here.iter().map(|r| r.date).max().unwrap());
        }
    }
}

#[tokio::test]
async fn stale_index_fields_are_not_read_sqlite() {
    stale_index_is_not_read(Store::open_in_memory(ServerKey::generate()).await.unwrap()).await;
}

#[tokio::test]
async fn stale_index_fields_are_not_read_postgres() {
    let Some((store, pg)) = private_pg("stale").await else {
        return;
    };
    stale_index_is_not_read(store).await;
    pg.drop().await;
}

// ---- fail closed ------------------------------------------------------------------

/// A PostgreSQL store in a private schema of MW_E14_PG_DSN, and a raw handle.
struct PgSchema {
    admin: sqlx::PgPool,
    schema: String,
}
impl PgSchema {
    async fn exec(&self, sql: &str) {
        sqlx::query(&format!("SET search_path TO {}", self.schema))
            .execute(&self.admin)
            .await
            .unwrap();
        sqlx::query(sql).execute(&self.admin).await.unwrap();
    }
    async fn drop(self) {
        sqlx::query(&format!("DROP SCHEMA {} CASCADE", self.schema))
            .execute(&self.admin)
            .await
            .unwrap();
    }
}

async fn private_pg(test: &str) -> Option<(Store, PgSchema)> {
    let Some(dsn) = mw_test_gate::pg_dsn() else {
        mw_test_gate::skip(format!(
            "DATABASE_URL_PG and MW_E14_PG_DSN unset: {test} on PostgreSQL"
        ));
        return None;
    };
    let admin = sqlx::postgres::PgPoolOptions::new()
        .max_connections(1)
        .connect(&dsn)
        .await
        .unwrap();
    let schema = format!(
        "e31_{test}_{}_{}",
        std::process::id(),
        chrono::Utc::now().timestamp_micros()
    );
    sqlx::query(&format!("CREATE SCHEMA {schema}"))
        .execute(&admin)
        .await
        .unwrap();
    let sep = if dsn.contains('?') { '&' } else { '?' };
    let scoped = format!("{dsn}{sep}options=-csearch_path%3D{schema}");
    let store = Store::open(&scoped, ServerKey::generate()).await.unwrap();
    Some((store, PgSchema { admin, schema }))
}

/// A plain (unencrypted) SQLite file store and a raw pool on the same file.
async fn sqlite_file(test: &str) -> (Store, sqlx::SqlitePool) {
    let dir = std::env::temp_dir().join(format!(
        "e31-{test}-{}-{}",
        std::process::id(),
        chrono::Utc::now().timestamp_micros()
    ));
    std::fs::create_dir_all(&dir).unwrap();
    let path = dir.join("mail.db");
    let store = Store::open_with_encryption(
        path.to_str().unwrap(),
        ServerKey::generate(),
        &mw_store::encryption::EncryptionConfig::off(),
    )
    .await
    .unwrap();
    let pool = sqlx::sqlite::SqlitePoolOptions::new()
        .max_connections(1)
        .connect_with(sqlx::sqlite::SqliteConnectOptions::new().filename(&path))
        .await
        .unwrap();
    (store, pool)
}

fn is_error(resp: &Value, kind: &str) -> bool {
    resp[0] == json!("error") && resp[1]["type"] == json!(kind) && resp[1].get("ids").is_none()
}

/// The metadata routes (R1, R3 inject, R3 nested, the unread count's id)
/// against a broken SQL side: each must be `serverFail`; R2 still answers.
/// `unread_id_reads_it`: whether the broken object is one the unread count's
/// limit-1 id reads (message_state via the folder read, or the trigger check);
/// when it is not (message_keyword), that id must still be right.
async fn metadata_routes_refuse(w: &World, context: &str, unread_id_reads_it: bool) {
    let acct = &w.a;
    let inbox = acct.folder("INBOX").to_owned();
    for filter in [
        json!({"notKeyword": "$seen"}),
        json!({"hasKeyword": "work", "inMailbox": inbox}),
        json!({"notKeyword": "$seen", "text": "alpha"}),
        json!({"text": "is:unread OR from:s3"}),
        json!({"text": "list:x"}),
        json!({"inMailbox": acct.saved}),
    ] {
        let resp = query(w, acct, filter.clone(), false).await;
        assert!(
            is_error(&resp, "serverFail"),
            "{context}: {filter} must refuse: {resp}"
        );
    }
    let r = w
        .send(&acct.id, json!([["Email/query", {"accountId": acct.id, "filter": {"inMailbox": inbox, "notKeyword": "$seen"}, "limit": 1, "calculateTotal": true}, "u"]]))
        .await;
    let resp = &r["methodResponses"][0];
    let unread_in_inbox = w
        .rows(acct)
        .await
        .iter()
        .filter(|r| r.mailbox == inbox && !r.keywords.contains("$seen"))
        .count();
    assert!(unread_in_inbox > 0);
    if unread_id_reads_it {
        assert!(
            is_error(resp, "serverFail"),
            "{context}: unread count id must refuse: {resp}"
        );
    } else {
        assert_eq!(resp[1]["total"], json!(unread_in_inbox), "{context}: {resp}");
        let id = resp[1]["ids"][0].as_str().unwrap_or_else(|| panic!("{context}: {resp}"));
        assert!(w.rows(acct).await.iter().any(|r| r.id == id && r.mailbox == inbox && !r.keywords.contains("$seen")), "{context}: {resp}");
    }
    // No metadata condition: the index answers as today.
    let resp = query(w, acct, json!({"text": "alpha"}), false).await;
    assert_eq!(resp[0], json!("Email/query"), "{context}: {resp}");
}

#[tokio::test]
async fn missing_or_broken_projection_refuses_sqlite() {
    UNCACHED_PROJECTION_CHECK.with(|c| c.set(true));
    // Triggers missing: the projection is not maintained.
    let (store, pool) = sqlite_file("trigger").await;
    let w = World::new(store, 12).await;
    sqlx::query("DROP TRIGGER message_projection_update")
        .execute(&pool)
        .await
        .unwrap();
    metadata_routes_refuse(&w, "sqlite trigger dropped", true).await;
    // An SQL error: the table is gone (triggers intact).
    let (store, pool) = sqlite_file("table").await;
    let w = World::new(store, 12).await;
    sqlx::query("DROP TABLE message_keyword")
        .execute(&pool)
        .await
        .unwrap();
    metadata_routes_refuse(&w, "sqlite keyword table dropped", false).await;
    let (store, pool) = sqlite_file("state").await;
    let w = World::new(store, 12).await;
    sqlx::query("DROP TABLE message_state")
        .execute(&pool)
        .await
        .unwrap();
    metadata_routes_refuse(&w, "sqlite state table dropped", true).await;
    UNCACHED_PROJECTION_CHECK.with(|c| c.set(false));
}

#[tokio::test]
async fn missing_or_broken_projection_refuses_postgres() {
    UNCACHED_PROJECTION_CHECK.with(|c| c.set(true));
    let Some((store, pg)) = private_pg("disabled").await else {
        return;
    };
    let w = World::new(store, 12).await;
    pg.exec("ALTER TABLE messages DISABLE TRIGGER message_projection_insert")
        .await;
    metadata_routes_refuse(&w, "postgres trigger disabled", true).await;
    pg.drop().await;
    let Some((store, pg)) = private_pg("renamed").await else {
        return;
    };
    let w = World::new(store, 12).await;
    pg.exec("ALTER TABLE message_keyword RENAME TO message_keyword_gone")
        .await;
    metadata_routes_refuse(&w, "postgres keyword table renamed", false).await;
    pg.drop().await;
    let Some((store, pg)) = private_pg("staterenamed").await else {
        return;
    };
    let w = World::new(store, 12).await;
    pg.exec("ALTER TABLE message_state RENAME TO message_state_gone")
        .await;
    metadata_routes_refuse(&w, "postgres state table renamed", true).await;
    pg.drop().await;
    UNCACHED_PROJECTION_CHECK.with(|c| c.set(false));
}

/// The index side refusing (journal mode, owner poisoned for a rebuild): every
/// route that needs the index refuses as today; the SQL-only route and the
/// unread count answer from SQL, equal to the oracle.
#[tokio::test]
async fn refusing_index_fails_closed_only_where_it_is_needed() {
    let store = Store::open_in_memory(ServerKey::generate()).await.unwrap();
    let search = std::sync::Arc::new(mw_search::Index::open_in_ram().unwrap());
    let options = crate::PublicationOptions {
        ack: crate::PublicationAck::Journal,
        collection_window: std::time::Duration::ZERO,
        ..Default::default()
    };
    let w = World::with_engine(
        store,
        12,
        |s| Engine::with_search_and_publication_options(s, search.clone(), options).unwrap(),
        true,
    )
    .await;
    let acct = &w.a;
    let rows = w.rows(acct).await;
    // Before: both sides answer.
    assert_eq!(
        query(
            &w,
            acct,
            json!({"notKeyword": "$seen", "text": "alpha"}),
            false
        )
        .await[0],
        json!("Email/query")
    );
    w.engine.index_publisher.begin_full_rebuild();
    for filter in [
        json!({"text": "alpha"}),
        json!({"notKeyword": "$seen", "text": "alpha"}),
        json!({"text": "is:unread OR from:s3"}),
        json!({"hasKeyword": "$list:x"}),
    ] {
        let resp = query(&w, acct, filter.clone(), false).await;
        assert!(
            resp[0] == json!("error") && resp[1].get("ids").is_none(),
            "{filter} must refuse: {resp}"
        );
        let old = query(&w, acct, filter.clone(), true).await;
        assert_eq!(
            old[1]["type"], resp[1]["type"],
            "{filter}: the same refusal as today: {old}"
        );
    }
    // SQL-only: answered, equal to the oracle.
    for template in [
        r#"{"notKeyword":"$seen"}"#,
        r#"{"hasKeyword":"work","inMailbox":"{INBOX}"}"#,
        r#"{"text":"tag:work -tag:home"}"#,
    ] {
        check_path(
            &w,
            acct,
            &rows,
            template,
            &plain_sort(),
            WINDOWS[0],
            "index refusing",
            false,
            false,
        )
        .await
        .unwrap();
    }
    let inbox = acct.folder("INBOX");
    let r = w
        .send(&acct.id, json!([["Email/query", {"accountId": acct.id, "filter": {"inMailbox": inbox, "notKeyword": "$seen"}, "limit": 1, "calculateTotal": true}, "u"]]))
        .await;
    assert_eq!(r["methodResponses"][0][0], json!("Email/query"), "{r}");
}

// ---- the id-set cap ---------------------------------------------------------------

/// A lowered cap drives the filter strategy, the complement and
/// `unsupportedFilter`; every answer that is not `unsupportedFilter` equals
/// the oracle, and the old path (no cap) is untouched.
#[tokio::test]
async fn id_set_cap_strategies_equal_the_oracle() {
    let w = World::new(
        Store::open_in_memory(ServerKey::generate()).await.unwrap(),
        40,
    )
    .await;
    let acct = &w.a;
    let rows = w.rows(acct).await;
    let unread = rows
        .iter()
        .filter(|r| !r.keywords.contains("$seen"))
        .count();
    let read = rows.len() - unread;
    assert!(
        unread > 4 && read > unread,
        "corpus: {unread} unread of {}",
        rows.len()
    );
    let set_cap = |n: usize| CAP.with(|c| c.set(n));
    TAKEN.with(|t| t.borrow_mut().clear());

    // Filter strategy: the top-level set is above the cap.
    set_cap(4);
    for template in [
        r#"{"notKeyword":"$seen","text":"alpha"}"#,
        r#"{"hasKeyword":"$seen","subject":"bravo"}"#,
        r#"{"notKeyword":"$seen","hasAttachment":false}"#,
    ] {
        for sort in [plain_sort(), sorts().swap_remove(9)] {
            check_path(
                &w,
                acct,
                &rows,
                template,
                &sort,
                WINDOWS[0],
                "cap 4 filter",
                false,
                false,
            )
            .await
            .unwrap();
        }
    }
    assert!(
        TAKEN.with(|t| t.borrow().contains(&Taken::Filter)),
        "filter strategy taken"
    );
    // Nested, both the set and its complement above the cap: unsupportedFilter.
    let resp = query(&w, acct, json!({"text": "is:unread OR from:s3"}), false).await;
    assert!(is_error(&resp, "unsupportedFilter"), "{resp}");
    assert_eq!(resp[1]["description"], json!(crate::query_plan::TOO_BROAD));
    assert!(TAKEN.with(|t| t.borrow().contains(&Taken::TooBroad)));
    assert_eq!(
        query(&w, acct, json!({"text": "is:unread OR from:s3"}), true).await[0],
        json!("Email/query"),
        "old path has no cap"
    );
    // Complement: the read set is above the cap, the unread set within it.
    set_cap(unread);
    for template in [
        r#"{"text":"is:read OR from:s3"}"#,
        r#"{"text":"NOT is:read OR subject:bravo"}"#,
    ] {
        check_path(
            &w,
            acct,
            &rows,
            template,
            &plain_sort(),
            WINDOWS[0],
            "complement",
            false,
            false,
        )
        .await
        .unwrap();
    }
    // The whole matrix under a small cap: unsupportedFilter or the oracle.
    let mut unsupported = 0;
    let mut answered = 0;
    let mut filters: Vec<String> = singles().into_iter().map(str::to_owned).collect();
    filters.extend(text_forms().into_iter().map(text_filter));
    for f in &filters {
        match check_path(
            &w,
            acct,
            &rows,
            f,
            &plain_sort(),
            WINDOWS[0],
            "cap matrix",
            false,
            true,
        )
        .await
        {
            Some(_) => answered += 1,
            None => unsupported += 1,
        }
    }
    set_cap(COMBINE_CAP);
    eprintln!(
        "t50hp-e31 cap {unread}: {answered} answered = oracle, {unsupported} unsupportedFilter"
    );
    assert!(answered > unsupported);
}

// ---- S3-a (C6) ----------------------------------------------------------------------

/// The unread count's limit-1 id from SQL against the old answer (the index's
/// top-1): both unread, both of the newest unread date (ties may pick another
/// member: the index breaks them by document address, SQL by uid), same total.
#[tokio::test]
async fn unread_count_id_from_sql_matches_the_index_answer() {
    let w = World::new(
        Store::open_in_memory(ServerKey::generate()).await.unwrap(),
        36,
    )
    .await;
    let mut rng = super::query_differential_tests::Rng::new(7);
    for round in 0..4 {
        for acct in [&w.a, &w.b] {
            let rows = w.rows(acct).await;
            for (_, mb) in &acct.folders {
                let call = json!([["Email/query", {"accountId": acct.id, "filter": {"inMailbox": mb, "notKeyword": "$seen"}, "limit": 1, "calculateTotal": true}, "u"]]);
                LEGACY.with(|l| l.set(true));
                let old = w.send(&acct.id, call.clone()).await["methodResponses"][0].clone();
                LEGACY.with(|l| l.set(false));
                let new = w.send(&acct.id, call).await["methodResponses"][0].clone();
                assert_eq!(
                    old[1]["total"], new[1]["total"],
                    "round {round}: {old} / {new}"
                );
                let unread: Vec<&Row> = rows
                    .iter()
                    .filter(|r| &r.mailbox == mb && !r.keywords.contains("$seen"))
                    .collect();
                let newest = unread.iter().map(|r| r.date).max();
                for resp in [&old, &new] {
                    match resp[1]["ids"][0].as_str() {
                        None => assert!(unread.is_empty(), "{resp}"),
                        Some(id) => {
                            let row = unread
                                .iter()
                                .find(|r| r.id == id)
                                .unwrap_or_else(|| panic!("{id} not unread: {resp}"));
                            assert_eq!(Some(row.date), newest, "round {round}: {resp}");
                        }
                    }
                }
            }
        }
        for _ in 0..6 {
            w.mutate(&w.a, &mut rng).await;
        }
    }
}

// ---- decision P1: zero-access accounts ---------------------------------------------

/// Zero-access for one account id; standard for every other.
struct ZeroAccessFor(String);
impl crate::v6::AccountPostureSource for ZeroAccessFor {
    fn posture(&self, account_id: &str) -> crate::v6::AccountPosture {
        if account_id == self.0 {
            crate::v6::AccountPosture::ZeroAccess
        } else {
            crate::v6::AccountPosture::Standard
        }
    }
}

/// P1 (designer's default): a zero-access account has no index document, so
/// every index route answers nothing, as before; a metadata-only query (R1)
/// is answered from SQL. The other (standard) account is unaffected.
#[tokio::test]
async fn zero_access_metadata_queries_are_answered_from_sql() {
    let store = Store::open_in_memory(ServerKey::generate()).await.unwrap();
    // Account ids are created inside World; the posture source is attached
    // before any message is ingested, keyed by the first account created.
    let w = World::new(store, 0).await;
    w.engine.attach_v6(
        crate::v6::V6Hooks::new()
            .with_posture_source(std::sync::Arc::new(ZeroAccessFor(w.a.id.clone()))),
    );
    for _ in 0..16 {
        w.ingest_pair_for_tests().await;
    }
    let acct = &w.a;
    assert_eq!(
        w.engine.search().account_doc_count(&acct.id).unwrap(),
        0,
        "zero-access: nothing indexed"
    );
    assert!(w.engine.search().account_doc_count(&w.b.id).unwrap() > 0);
    let rows = w.rows(acct).await;
    assert!(!rows.is_empty());
    for template in [
        r#"{"notKeyword":"$seen"}"#,
        r#"{"hasKeyword":"work","inMailbox":"{INBOX}"}"#,
        r#"{"text":"tag:work -tag:home"}"#,
        r#"{"text":"pinned:false"}"#,
    ] {
        check_path(
            &w,
            acct,
            &rows,
            template,
            &plain_sort(),
            WINDOWS[0],
            "zero-access R1",
            false,
            false,
        )
        .await
        .unwrap();
    }
    for filter in [
        json!({"text": "alpha"}),
        json!({"notKeyword": "$seen", "text": "alpha"}),
        json!({"text": "is:unread OR from:s3"}),
    ] {
        let resp = query(&w, acct, filter.clone(), false).await;
        assert_eq!(resp[0], json!("Email/query"), "{filter}: {resp}");
        assert!(
            id_set(&resp).is_empty(),
            "{filter}: no index document, no hit: {resp}"
        );
    }
    // The standard account answers every route.
    let b_rows = w.rows(&w.b).await;
    for template in [
        r#"{"notKeyword":"$seen"}"#,
        r#"{"notKeyword":"$seen","text":"alpha"}"#,
    ] {
        check_path(
            &w,
            &w.b,
            &b_rows,
            template,
            &plain_sort(),
            WINDOWS[0],
            "standard account",
            false,
            false,
        )
        .await
        .unwrap();
    }
}

// ---- pre-existing bug, recorded, not fixed here --------------------------------------

/// Pre-existing (found by the differential test's PostgreSQL leg, t50hp-e31):
/// a keyword the user sets whose name has a derived prefix (`$list:`,
/// `$supersedes:`, `$mw-mention:`, `$priority-high/low`) can never be removed
/// from the search index. `index_publication::patch_current` rebuilds the
/// keywords from `flags_json` and then re-adds every derived-prefix value of
/// the STORED document, so the user's value is carried forever (until a full
/// rebuild from source). Decision P2 (union of index and SQL) keeps the stale
/// value visible to `hasKeyword` / `list:`.
///
/// Fix sketch (index_publication.rs, not this lane): carry over only the
/// values the content produced. Either recompute them from the source at
/// publication (mentions, List-Id, Supersedes, priority are all content), or
/// keep them in a separate `IndexDoc` field (e.g. `derived_keywords`) written
/// at build time and re-added by patch_current instead of filtering
/// `doc.keywords` by prefix; the user's keywords then come only from
/// flags_json.
#[tokio::test]
#[ignore = "pre-existing bug: patch_current keeps a user keyword with a derived-prefix name (see doc comment)"]
async fn user_keyword_with_a_derived_prefix_leaves_the_index_when_removed() {
    let w = World::new(Store::open_in_memory(ServerKey::generate()).await.unwrap(), 0).await;
    w.ingest_pair_for_tests().await;
    let acct = &w.a;
    let id = w.rows(acct).await.remove(0).id;
    for set in [json!(true), Value::Null] {
        w.update(&acct.id, &id, json!({"keywords/$list:x": set})).await;
    }
    let keywords = w.engine.search().fetch_doc(&id).unwrap().unwrap().keywords;
    assert!(!keywords.contains(&"$list:x".to_owned()), "index still holds the removed keyword: {keywords:?}");
    let resp = query(&w, acct, json!({"hasKeyword": "$list:x"}), false).await;
    assert!(id_set(&resp).is_empty(), "{resp}");
}

/// t49pg-e5b F1: a nested metadata subtree with its own folder (`in:M`) in an
/// account-wide query, under OR / NOT / double NOT, with the cap set so that
/// the subtree's set is above it and its complement within it: the
/// complement must keep the folder condition. Every answer equals the
/// oracle (or is unsupportedFilter).
#[tokio::test]
async fn nested_folder_complement_keeps_the_folder() {
    let w = World::new(Store::open_in_memory(ServerKey::generate()).await.unwrap(), 48).await;
    let acct = &w.a;
    let rows = w.rows(acct).await;
    let archive = acct.folder("Archive").to_owned();
    let in_archive = |seen: bool| rows.iter().filter(|r| r.mailbox == archive && r.keywords.contains("$seen") == seen).count();
    let (read, unread) = (in_archive(true), in_archive(false));
    assert!(read > unread && unread > 0, "corpus: Archive read {read} unread {unread}");
    CAP.with(|c| c.set(unread));
    TAKEN.with(|t| t.borrow_mut().clear());
    let mut answered = 0;
    for form in [
        "(in:{Archive} is:read) OR from:s3",
        "NOT (in:{Archive} is:read) OR subject:bravo",
        "-(-(in:{Archive} is:read)) OR from:s3",
        "(in:{Archive} is:read) OR (in:{Work} is:unread) OR from:s3",
        "NOT (in:{Archive} is:read)",
        "(in:{Archive} is:read) OR (in:{Archive} is:read) OR from:s3",
    ] {
        if check_path(&w, acct, &rows, &text_filter(form), &plain_sort(), WINDOWS[0], "F1 complement", false, true)
            .await
            .is_some()
        {
            answered += 1;
        }
    }
    CAP.with(|c| c.set(COMBINE_CAP));
    assert!(answered >= 4, "the complement branch must answer, not refuse: {answered}");
}

/// t49pg-e5b F2: one request must not materialise unbounded id sets. More
/// metadata subtrees than the per-query limit is unsupportedFilter before any
/// SQL; identical subtrees are fetched once; all sets of one query share one
/// id budget. `ids_materialised` counts every id row the planner reads.
#[tokio::test]
async fn metadata_subtrees_are_bounded_per_query() {
    let w = World::new(Store::open_in_memory(ServerKey::generate()).await.unwrap(), 48).await;
    let acct = &w.a;
    let rows = w.rows(acct).await;
    let read = rows.iter().filter(|r| r.keywords.contains("$seen")).count();
    let unread = rows.len() - read;
    let materialised = crate::query_plan::ids_materialised;
    // 64 copies of a subtree that matches most of the account.
    let adversarial = "(pinned:false OR zz) ".repeat(64);
    let before = materialised();
    let started = std::time::Instant::now();
    let resp = query(&w, acct, json!({"text": adversarial}), false).await;
    assert!(is_error(&resp, "unsupportedFilter"), "{resp}");
    assert_eq!(materialised() - before, 0, "refused before any id set was read");
    assert!(started.elapsed() < std::time::Duration::from_secs(5), "{:?}", started.elapsed());
    // Up to the limit, distinct subtrees answer (= oracle).
    let eight = (0..crate::query_plan::MAX_META_NODES).map(|i| format!("(pinned:false OR zz{i})")).collect::<Vec<_>>().join(" OR ");
    check_path(&w, acct, &rows, &text_filter(&eight), &plain_sort(), WINDOWS[0], "eight subtrees", false, false).await.unwrap();
    // Identical subtrees are read once.
    let before = materialised();
    check_path(&w, acct, &rows, &text_filter("(is:unread OR zz) (is:unread OR zz) (is:unread OR zz) alpha"), &plain_sort(), WINDOWS[0], "duplicates", false, false)
        .await
        .unwrap();
    assert!(materialised() - before <= unread as u64 + 1, "one fetch for three equal subtrees: {}", materialised() - before);
    // One id budget per query: two sets that each fit the cap but not together.
    CAP.with(|c| c.set(read.max(unread) + 2));
    let before = materialised();
    let r = check_path(&w, acct, &rows, &text_filter("(is:read OR zz) (is:unread OR zz2) OR from:s3"), &plain_sort(), WINDOWS[0], "budget", false, true).await;
    let used = materialised() - before;
    CAP.with(|c| c.set(COMBINE_CAP));
    // The budget, plus one look-ahead row per refused attempt (a set and its
    // complement per subtree) and one for the prefetch.
    let bound = (read.max(unread) + 2 + 2 * 2 + 1) as u64;
    assert!(used <= bound, "budget exceeded: {used} > {bound}");
    eprintln!("t49pg-e5b F2 budget case: answered {}, {used} ids read", r.is_some());
}
