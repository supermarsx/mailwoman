//! t60 search scenarios: selectivity, field operators, scope, sort and paging,
//! search-as-you-type, freshness after a change, snippets, reload and rebuild.
//!
//! Every expected result comes from a brute-force oracle over the generated
//! corpus (`corpus::Fields`), never from the server. The queries are the web
//! client's: the typed text goes into `filter.text` with the folder scope the
//! client adds (apps/web/src/state/slices/mail.ts, search/defaultScope.ts).
use super::*;

const TO: &str = "bulk-search@native-load.invalid";
/// Planted terms tracked per message (bit index = position).
const TERMS: [&str; 18] = [
    "zephyrine", "quarterly", "\"amber lantern\"", "amber", "lantern", "zephyr*", "from:sender07", "cc:audit", "subject:urgent", "filename:report",
    "q", "qu", "qua", "quar", "quart", "quarte", "quarter", "quarterl",
];
pub(super) const EXECUTION_CONTEXT: &str =
    "inline in the async request task: Engine::search_ids_ordered calls Index::search_hits_ordered directly (crates/mw-engine/src/jmap.rs), no spawn_blocking";

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(super) enum Folder {
    Inbox,
    Archive,
}
#[derive(Clone, Debug)]
pub(super) struct Meta {
    seed: u32,
    folder: Folder,
    seen: bool,
    flagged: bool,
    received: i64,
    size: u64,
    attachment: bool,
    terms: u32,
}
impl Meta {
    fn of(seed: u32, folder: Folder, pattern: bool) -> Self {
        let f = corpus::Fields::of("search", seed, TO);
        let checks: [bool; 18] = [
            f.any_has("zephyrine"),
            f.any_has("quarterly"),
            f.body_phrase("amber", "lantern"),
            f.any_has("amber"),
            f.any_has("lantern"),
            f.any_prefix("zephyr"),
            f.from.iter().any(|t| t == "sender07"),
            f.cc.iter().any(|t| t == "audit"),
            f.subject.iter().any(|t| t == "urgent"),
            f.filename.iter().any(|t| t == "report"),
            f.any_has("q"),
            f.any_has("qu"),
            f.any_has("qua"),
            f.any_has("quar"),
            f.any_has("quart"),
            f.any_has("quarte"),
            f.any_has("quarter"),
            f.any_has("quarterl"),
        ];
        let (seen, flagged) = if pattern { corpus::pattern_flags(seed) } else { (false, false) };
        Self {
            seed,
            folder,
            seen,
            flagged,
            received: f.received,
            size: f.size,
            attachment: f.has_attachment,
            terms: checks.iter().enumerate().fold(0, |bits, (i, on)| bits | (u32::from(*on) << i)),
        }
    }
    fn has(&self, term: &str) -> bool {
        let index = TERMS.iter().position(|t| *t == term).expect("tracked term");
        self.terms & (1 << index) != 0
    }
}

pub(super) struct Oracle {
    messages: Mutex<Vec<Meta>>,
}
impl Oracle {
    pub(super) fn build(n: u32) -> Self {
        let archive = search_archive(n);
        let mut messages: Vec<Meta> = (1..=n).map(|seed| Meta::of(seed, Folder::Inbox, true)).collect();
        messages.extend((n + 1..=n + archive).map(|seed| Meta::of(seed, Folder::Archive, true)));
        Self { messages: Mutex::new(messages) }
    }
    /// Matches newest first (received time is strictly increasing in seed).
    fn matches(&self, pred: &dyn Fn(&Meta) -> bool) -> Vec<Meta> {
        let mut out: Vec<Meta> = self.messages.lock().unwrap().iter().filter(|m| pred(m)).cloned().collect();
        out.sort_by(|a, b| b.received.cmp(&a.received));
        out
    }
    fn max_seed(&self) -> u32 {
        self.messages.lock().unwrap().iter().map(|m| m.seed).max().unwrap_or(0)
    }
    fn update(&self, seed: u32, change: impl Fn(&mut Meta)) {
        if let Some(m) = self.messages.lock().unwrap().iter_mut().find(|m| m.seed == seed) {
            change(m);
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(super) enum Scope {
    Inbox,
    AllFolders,
    Saved,
}
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(super) enum SortBy {
    Date,
    From,
    Size,
}
pub(super) struct Spec {
    pub name: String,
    text: Option<String>,
    scope: Scope,
    sort: SortBy,
    position: u64,
    limit: u64,
    total: bool,
    /// `true`: the client's unread count (`notKeyword $seen`, limit 1) with no text.
    unread_count: bool,
    pred: Box<dyn Fn(&Meta) -> bool + Send + Sync>,
}
fn spec(name: &str, text: &str, pred: impl Fn(&Meta) -> bool + Send + Sync + 'static) -> Spec {
    Spec { name: name.into(), text: Some(text.into()), scope: Scope::Inbox, sort: SortBy::Date, position: 0, limit: PAGE, total: true, unread_count: false, pred: Box::new(pred) }
}
fn date(days: i64) -> i64 {
    corpus::BASE_EPOCH + days * 86_400
}

/// The query list. Planted-term frequencies make every expected count exact.
pub(super) fn specs(n: u32) -> Vec<Spec> {
    // A date window inside the corpus: calendar dates when the mailbox spans
    // several days (what the client's date chip writes), else Unix seconds.
    let (after, before, after_ts, before_ts) = if n >= 6_000 {
        ("2026-01-02".to_string(), "2026-01-04".to_string(), date(1), date(3))
    } else {
        let (a, b) = (corpus::BASE_EPOCH + i64::from(n / 4) * 60, corpus::BASE_EPOCH + i64::from(n / 2) * 60);
        (a.to_string(), b.to_string(), a, b)
    };
    let inbox = |m: &Meta| m.folder == Folder::Inbox;
    let common = move |m: &Meta| inbox(m) && m.has("quarterly");
    let common_count = (1..=n).filter(|s| s % 5 == 0).count() as u64;
    let mut list = vec![
        spec("rare-term", "zephyrine", move |m| inbox(m) && m.has("zephyrine")),
        spec("common-term", "quarterly", common),
        spec("phrase", "\"amber lantern\"", move |m| inbox(m) && m.has("\"amber lantern\"")),
        spec("and-terms", "amber lantern", move |m| inbox(m) && m.has("amber") && m.has("lantern")),
        spec("prefix", "zephyr*", move |m| inbox(m) && m.has("zephyr*")),
        spec("from", "from:sender07", move |m| inbox(m) && m.has("from:sender07")),
        spec("cc", "cc:audit", move |m| inbox(m) && m.has("cc:audit")),
        spec("subject", "subject:urgent", move |m| inbox(m) && m.has("subject:urgent")),
        spec("has-attachment", "has:attachment", move |m| inbox(m) && m.attachment),
        spec("filename", "filename:report", move |m| inbox(m) && m.has("filename:report")),
        spec("date-range", &format!("after:{after} before:{before}"), move |m| inbox(m) && m.received >= after_ts && m.received < before_ts),
        spec("larger", "larger:20k", move |m| inbox(m) && m.size > 20_000),
        spec("smaller", "smaller:1500", move |m| inbox(m) && m.size < 1_500),
        spec("is-unread", "is:unread", move |m| inbox(m) && !m.seen),
        spec("is-flagged", "is:flagged", move |m| inbox(m) && m.flagged),
        spec("unread-and-text", "is:unread quarterly", move |m| common(m) && !m.seen),
        spec("from-and-attachment", "from:sender07 has:attachment", move |m| inbox(m) && m.has("from:sender07") && m.attachment),
        spec("text-and-date", &format!("quarterly after:{after} before:{before}"), move |m| common(m) && m.received >= after_ts && m.received < before_ts),
        spec("subject-and-flagged", "subject:urgent is:flagged", move |m| inbox(m) && m.has("subject:urgent") && m.flagged),
        Spec { scope: Scope::AllFolders, ..spec("all-folders", "quarterly", |m| m.has("quarterly")) },
        Spec { scope: Scope::Saved, text: None, ..spec("saved-search-folder", "quarterly", |m| m.has("quarterly")) },
        Spec { position: PAGE, ..spec("page-2", "quarterly", common) },
        Spec { position: 4 * PAGE, ..spec("page-5", "quarterly", common) },
        Spec { position: common_count / 2, ..spec("deep-page", "quarterly", common) },
        Spec { position: PAGE, total: false, ..spec("page-2-no-total", "quarterly", common) },
        Spec { sort: SortBy::From, ..spec("sort-from", "quarterly", common) },
        Spec { sort: SortBy::Size, ..spec("sort-size", "quarterly", common) },
        Spec { sort: SortBy::Size, text: None, ..spec("size-sort-folder", "", inbox) },
        Spec { text: None, limit: 1, unread_count: true, ..spec("unread-count", "", move |m| inbox(m) && !m.seen) },
        Spec { text: Some("is:flagged".into()), ..spec("flagged-page", "", move |m| inbox(m) && m.flagged) },
    ];
    list.iter_mut().for_each(|s| s.name = format!("search-{}", s.name));
    list
}

pub(super) struct SearchState {
    pub oracle: Oracle,
    inbox: String,
    archive: String,
    trash: String,
    junk: String,
    saved: Option<String>,
    saved_error: Option<String>,
}

pub(super) async fn prepare(ctx: &Ctx) -> std::result::Result<SearchState, String> {
    let p = ctx.principal("search");
    let n = ctx.config.mailbox;
    let oracle = tokio::task::spawn_blocking(move || Oracle::build(n)).await.map_err(|e| e.to_string())?;
    let ids = mailbox_ids(ctx, p).await;
    let get = |name: &str| ids.get(name).cloned().ok_or(format!("missing {name}"));
    // A saved-search folder as the client creates one (Settings, PUT route,
    // fenced by the storage authority that /api/me reports).
    let me: Value = ctx
        .http
        .get(format!("{}/api/me", ctx.base))
        .header("cookie", &p.cookie)
        .header("x-forwarded-for", &p.client)
        .send()
        .await
        .map_err(|e| e.to_string())?
        .json()
        .await
        .map_err(|e| e.to_string())?;
    let authority = me["storageAuthority"].as_str().unwrap_or_default().to_string();
    let saved = ctx
        .http
        .put(format!("{}/api/account/saved-searches", ctx.base))
        .header("cookie", &p.cookie)
        .header("x-csrf-token", &p.csrf)
        .header("x-forwarded-for", &p.client)
        .header("origin", &ctx.base)
        .header("x-mailwoman-account", &p.account)
        .header("x-mailwoman-storage-authority", &authority)
        .json(&json!({"name":"Quarterly","queryJson":json!({"text":corpus::SEARCH_WORD}).to_string(),"asFolder":true}))
        .send()
        .await
        .map_err(|e| e.to_string())?;
    let status = saved.status().as_u16();
    let body = saved.text().await.unwrap_or_default();
    let saved_id = (status == 200).then(|| serde_json::from_str::<Value>(&body).ok().and_then(|v| v["id"].as_str().map(str::to_owned))).flatten();
    let saved_error = saved_id.is_none().then(|| format!("saved-search PUT returned {status}: {}", body.chars().take(300).collect::<String>()));
    Ok(SearchState { oracle, inbox: get("INBOX")?, archive: get("Archive")?, trash: get("Trash")?, junk: get("Junk")?, saved: saved_id, saved_error })
}

fn filter(state: &SearchState, spec: &Spec) -> std::result::Result<Value, Stop> {
    if spec.unread_count {
        // apps/web/src/api/workspaceMail.ts: the unread count query.
        return Ok(json!({"inMailbox":state.inbox,"notKeyword":"$seen"}));
    }
    let mut filter = match spec.scope {
        Scope::Inbox => json!({"inMailbox":state.inbox,"notKeyword":"$junk"}),
        Scope::AllFolders => json!({"inMailboxOtherThan":[state.trash,state.junk],"notKeyword":"$junk"}),
        Scope::Saved => json!({"inMailbox":state.saved.clone().ok_or_else(|| Stop::Failed(state.saved_error.clone().unwrap_or_else(|| "saved-search folder was not created".into())))?}),
    };
    if let Some(text) = &spec.text {
        filter["text"] = json!(text);
    }
    Ok(filter)
}
fn sort(spec: &Spec) -> Value {
    match spec.sort {
        SortBy::Date => newest_first(),
        SortBy::From => json!([{"property":"from","isAscending":true}]),
        SortBy::Size => json!([{"property":"size","isAscending":false}]),
    }
}
fn calls(ctx: &Ctx, p: &Principal, filter: Value, spec: &Spec) -> Value {
    let mut query = json!({"accountId":p.account,"filter":filter,"sort":sort(spec),"limit":spec.limit,"calculateTotal":spec.total});
    if spec.position > 0 {
        query["position"] = json!(spec.position);
    }
    json!([["Email/query", query, "q"], ["Email/get", {"accountId":p.account,"#ids":{"resultOf":"q","name":"Email/query","path":"/ids"},"properties":ctx.properties}, "g"]])
}
fn seed_of(row: &Value) -> Option<u32> {
    row["messageId"][0].as_str()?.strip_prefix("bulk-search-")?.split('@').next()?.parse().ok()
}

/// Per-request search counters (process-wide; read between requests only).
pub(super) struct SearchCounters {
    hits: u64,
    docs: u64,
    projection_reads: u64,
    sealed_bytes: u64,
    sealed_reads: u64,
    cpu: Option<f64>,
}
pub(super) fn search_counters(ctx: &Ctx) -> SearchCounters {
    let timing = serde_json::to_value(mw_search::timing::snapshot()).unwrap_or_default();
    SearchCounters {
        hits: mw_search::write_and_hit_counts().2,
        docs: ctx.engine.search().doc_fetch_count(),
        projection_reads: ctx.engine.search().list_projection_doc_read_count(),
        sealed_bytes: timing["phases"]["sealedRead"]["bytes"].as_u64().unwrap_or(0),
        sealed_reads: timing["phases"]["sealedRead"]["calls"].as_u64().unwrap_or(0),
        cpu: process_cpu_seconds(),
    }
}
pub(super) fn search_delta(a: &SearchCounters, b: &SearchCounters, requests: u64) -> Value {
    let per = |x: u64| x as f64 / requests.max(1) as f64;
    let hits = b.hits - a.hits;
    let decoded = hits + (b.docs - a.docs) + (b.projection_reads - a.projection_reads);
    json!({"requests":requests,"hitsMaterialisedPerRequest":per(hits),"storedDocumentsDecodedPerRequest":per(decoded),
        "documentFetchesPerRequest":per(b.docs-a.docs),"listProjectionDocumentReadsPerRequest":per(b.projection_reads-a.projection_reads),
        "sealedIndexBytesReadPerRequest":per(b.sealed_bytes-a.sealed_bytes),"sealedIndexFileReadsPerRequest":per(b.sealed_reads-a.sealed_reads),
        "processCpuMsPerRequest":a.cpu.zip(b.cpu).map(|(x,y)| (y-x)*1000.0/requests.max(1) as f64),
        "scope":"process-wide counters read between sequential requests; CPU includes every thread of the process"})
}

/// Check one response against the oracle.
fn verify(rec: &mut Record, spec: &Spec, value: &Value, expected: &[Meta]) {
    let query = result(value, 0);
    let rows = result(value, 1)["list"].as_array().cloned().unwrap_or_default();
    let count = expected.len() as u64;
    let want = spec.limit.min(count.saturating_sub(spec.position)) as usize;
    if spec.total {
        rec.check("total-matches-oracle", query["total"].as_u64() == Some(count), json!({"expected":count,"got":query["total"]}));
    } else {
        rec.check("no-total-when-not-asked", query["total"].is_null(), json!({"got":query["total"]}));
    }
    rec.check("page-length", rows.len() == want, json!({"expected":want,"got":rows.len()}));
    let seeds: Vec<Option<u32>> = rows.iter().map(seed_of).collect();
    let allowed: HashSet<u32> = expected.iter().map(|m| m.seed).collect();
    rec.check("every-row-matches-the-query", seeds.iter().all(|s| s.is_some_and(|s| allowed.contains(&s))), json!({"rows":rows.len()}));
    match spec.sort {
        SortBy::Date => {
            let window: Vec<Option<u32>> = expected.iter().skip(spec.position as usize).take(want).map(|m| Some(m.seed)).collect();
            rec.check("exact-page-in-date-order", seeds == window, json!({"first":seeds.first(),"expectedFirst":window.first()}));
        }
        SortBy::Size => {
            let sizes: Vec<u64> = rows.iter().map(|r| r["size"].as_u64().unwrap_or(0)).collect();
            let largest = expected.iter().map(|m| m.size).max();
            rec.check("size-order", sizes.windows(2).all(|w| w[0] >= w[1]) && (spec.position > 0 || want == 0 || sizes.first().copied() == largest),
                json!({"first":sizes.first(),"oracleLargest":largest}));
        }
        SortBy::From => {
            let key = |row: &Value, field: &str| row["from"][0][field].as_str().unwrap_or_default().to_lowercase();
            let ordered = |field: &str| rows.windows(2).all(|w| key(&w[0], field) <= key(&w[1], field));
            rec.check("from-order", ordered("email") || ordered("name"), json!({}));
        }
    }
}

pub(super) async fn s_query(ctx: &Ctx, rec: &mut Record, spec: &Spec, state: &SearchState) -> std::result::Result<(), Stop> {
    let p = ctx.principal("search");
    let expected = state.oracle.matches(&*spec.pred);
    let filter = filter(state, spec)?;
    let calls = calls(ctx, p, filter.clone(), spec);
    let c0 = search_counters(ctx);
    let mut cold = Calls::default();
    let first = call(ctx, p, calls.clone(), &mut cold).await.map_err(|r| refused("cold request", r))?;
    let c1 = search_counters(ctx);
    let mut warm = Calls::default();
    let mut last = first.clone();
    for _ in 0..5 {
        last = call(ctx, p, calls.clone(), &mut warm).await.map_err(|r| refused("warm request", r))?;
    }
    let c2 = search_counters(ctx);
    rec.put("request", json!({"filter":filter,"sort":sort(spec),"position":spec.position,"limit":spec.limit,"calculateTotal":spec.total}));
    rec.put("mailboxMessages", ctx.config.mailbox);
    rec.put("hitsMatched", expected.len());
    rec.put("returned", result(&first, 0)["ids"].as_array().map_or(0, Vec::len));
    rec.put("coldMs", cold.latencies[0]);
    rec.put("coldNote", "first request of this shape in this process; the OS page cache was not dropped");
    rec.put("warm", warm.report());
    rec.put("cold", search_delta(&c0, &c1, 1));
    rec.put("warmPerRequest", search_delta(&c1, &c2, 5));
    rec.put("executionContext", EXECUTION_CONTEXT);
    let extra: serde_json::Map<String, Value> = result(&first, 0).as_object().into_iter().flatten().filter(|(k, _)| !matches!(k.as_str(), "ids" | "accountId")).map(|(k, v)| (k.clone(), v.clone())).collect();
    rec.put("queryResponse", extra);
    verify(rec, spec, &first, &expected);
    rec.check("warm-equals-cold", result(&last, 0)["ids"] == result(&first, 0)["ids"], json!({}));
    Ok(())
}

/// What a client typing "quarterly" sends when it pauses for at least the
/// 250 ms debounce after each key (apps/web/src/components/search/SearchBox.tsx).
pub(super) async fn s_typing(ctx: &Ctx, rec: &mut Record, state: &SearchState) -> std::result::Result<(), Stop> {
    let p = ctx.principal("search");
    let word = corpus::SEARCH_WORD;
    let mut keys = Vec::new();
    let c0 = search_counters(ctx);
    let mut stats = Calls::default();
    for end in 1..=word.len() {
        let typed = &word[..end];
        let probe = spec("typing", typed, |_| true);
        let expected = state.oracle.matches(&move |m: &Meta| m.folder == Folder::Inbox && if end == word.len() { m.has("quarterly") } else { m.has(typed) });
        let value = call(ctx, p, calls(ctx, p, filter(state, &probe)?, &probe), &mut stats).await.map_err(|r| refused(&format!("keystroke {end}"), r))?;
        let total = result(&value, 0)["total"].as_u64();
        rec.check(&format!("keystroke-{typed}"), total == Some(expected.len() as u64), json!({"expected":expected.len(),"got":total}));
        keys.push(json!({"typed":typed,"latencyMs":stats.latencies.last(),"total":total}));
    }
    let c1 = search_counters(ctx);
    rec.put("keystrokes", keys);
    rec.put("requests", stats.report());
    rec.put("responsesSlowerThanDebounce", stats.latencies.iter().filter(|l| **l > 250.0).count());
    rec.put("debounceMs", 250);
    rec.put("counters", search_delta(&c0, &c1, stats.requests));
    rec.put("note", "worst case: one request per keystroke (a pause of at least 250 ms after each key); a user typing faster sends only the last query, measured as search-common-term");
    Ok(())
}

pub(super) async fn s_snippets(ctx: &Ctx, rec: &mut Record, state: &SearchState) -> std::result::Result<(), Stop> {
    let p = ctx.principal("search");
    let mut setup = Calls::default();
    let page = call(ctx, p, json!([["Email/query",{"accountId":p.account,"filter":{"inMailbox":state.inbox,"text":corpus::SEARCH_WORD},"sort":newest_first(),"limit":PAGE},"q"]]), &mut setup).await.map_err(|r| refused("setup", r))?;
    let ids = result(&page, 0)["ids"].clone();
    let n = ids.as_array().map_or(0, Vec::len);
    let calls = json!([["SearchSnippet/get",{"accountId":p.account,"filter":{"inMailbox":state.inbox,"text":corpus::SEARCH_WORD},"emailIds":ids},"s"]]);
    let c0 = search_counters(ctx);
    let mut cold = Calls::default();
    let first = call(ctx, p, calls.clone(), &mut cold).await.map_err(|r| refused("SearchSnippet/get", r))?;
    let c1 = search_counters(ctx);
    let mut warm = Calls::default();
    for _ in 0..5 {
        call(ctx, p, calls.clone(), &mut warm).await.map_err(|r| refused("SearchSnippet/get", r))?;
    }
    let list = result(&first, 0)["list"].as_array().cloned().unwrap_or_default();
    rec.put("snippets", list.len());
    rec.put("coldMs", cold.latencies[0]);
    rec.put("warm", warm.report());
    rec.put("cold", search_delta(&c0, &c1, 1));
    rec.put("note", "the web client never calls SearchSnippet/get; measured because the engine implements it");
    rec.check("one-snippet-per-id", list.len() == n && n > 0, json!({"ids":n,"snippets":list.len()}));
    let marked = list.iter().filter(|s| [&s["subject"], &s["preview"]].iter().any(|v| v.as_str().is_some_and(|t| t.contains("<mark>")))).count();
    rec.put("snippetsWithMark", marked);
    Ok(())
}

/// The first query after the index reader reloaded (one committed flag write).
pub(super) async fn s_after_reload(ctx: &Ctx, rec: &mut Record, state: &SearchState) -> std::result::Result<(), Stop> {
    let p = ctx.principal("search");
    let common = spec("after-reload", corpus::SEARCH_WORD, |m| m.folder == Folder::Inbox && m.has("quarterly"));
    let calls = calls(ctx, p, filter(state, &common)?, &common);
    let mut setup = Calls::default();
    let target = state.oracle.matches(&|m: &Meta| m.folder == Folder::Inbox && !m.has("quarterly") && !m.flagged).first().cloned().ok_or(Stop::Failed("no target".into()))?;
    let id = find_id(ctx, p, state, target.seed).await?;
    let flag = |on: bool| json!([["Email/set",{"accountId":p.account,"update":{&id:{"keywords/$flagged": if on { json!(true) } else { Value::Null }}}},"s"]]);
    call(ctx, p, flag(true), &mut setup).await.map_err(|r| refused("flag write", r))?;
    publication_idle(ctx).await;
    let c0 = search_counters(ctx);
    let mut after = Calls::default();
    let value = call(ctx, p, calls.clone(), &mut after).await.map_err(|r| refused("query after reload", r))?;
    let c1 = search_counters(ctx);
    let mut warm = Calls::default();
    for _ in 0..5 {
        call(ctx, p, calls.clone(), &mut warm).await.map_err(|r| refused("warm", r))?;
    }
    call(ctx, p, flag(false), &mut setup).await.map_err(|r| refused("restore write", r))?;
    publication_idle(ctx).await;
    let expected = state.oracle.matches(&*common.pred);
    rec.put("hitsMatched", expected.len());
    rec.put("returned", result(&value, 0)["ids"].as_array().map_or(0, Vec::len));
    rec.put("firstAfterReloadMs", after.latencies[0]);
    rec.put("warm", warm.report());
    rec.put("firstAfterReload", search_delta(&c0, &c1, 1));
    verify(rec, &common, &value, &expected);
    Ok(())
}

/// The id of a corpus message near the top of the Inbox (newest first).
async fn find_id(ctx: &Ctx, p: &Principal, state: &SearchState, seed: u32) -> std::result::Result<String, Stop> {
    let mut stats = Calls::default();
    for page in 0..50u64 {
        let value = call(ctx, p, json!([
            ["Email/query",{"accountId":p.account,"filter":{"inMailbox":state.inbox},"sort":newest_first(),"position":page*200,"limit":200},"q"],
            ["Email/get",{"accountId":p.account,"#ids":{"resultOf":"q","name":"Email/query","path":"/ids"},"properties":["id","messageId"]},"g"]
        ]), &mut stats).await.map_err(|r| refused("setup lookup", r))?;
        let rows = result(&value, 1)["list"].as_array().cloned().unwrap_or_default();
        if let Some(row) = rows.iter().find(|row| seed_of(row) == Some(seed)) {
            return Ok(row["id"].as_str().unwrap_or_default().to_string());
        }
        if rows.len() < 200 {
            break;
        }
    }
    Err(Stop::Failed(format!("message for seed {seed} not found")))
}

/// Freshness: how soon a search reflects a change, and what it returns first.
pub(super) async fn s_fresh(ctx: &Ctx, rec: &mut Record, state: &SearchState, change: &str) -> std::result::Result<(), Stop> {
    let p = ctx.principal("search");
    let (text, pred): (&str, Box<dyn Fn(&Meta) -> bool + Send + Sync>) = match change {
        "arrival" => ("zephyrine", Box::new(|m| m.folder == Folder::Inbox && m.has("zephyrine"))),
        "mark-read" => ("is:unread quarterly", Box::new(|m| m.folder == Folder::Inbox && m.has("quarterly") && !m.seen)),
        _ => ("quarterly", Box::new(|m| m.folder == Folder::Inbox && m.has("quarterly"))),
    };
    let probe = Spec { pred, ..spec(change, text, |_| true) };
    let calls_for = calls(ctx, p, filter(state, &probe)?, &probe);
    let before_count = state.oracle.matches(&*probe.pred).len() as u64;
    let c0 = snapshot(&ctx.engine, Some(p));
    let started = Instant::now();
    let mut op = Calls::default();
    let changed: u64;
    match change {
        "arrival" => {
            let mut seed = state.oracle.max_seed() + 1;
            while seed % 2000 != 7 {
                seed += 1;
            }
            p.backend.deliver("INBOX", seed);
            ctx.engine.refresh_mailbox(&p.account, Some(state.inbox.as_str())).await.map_err(|e| Stop::Failed(format!("refresh: {e}")))?;
            state.oracle.messages.lock().unwrap().push(Meta::of(seed, Folder::Inbox, false));
            changed = 1;
            rec.put("change", "one upstream arrival carrying the rare term, ingested by a folder refresh");
        }
        "mark-read" => {
            let target = state.oracle.matches(&*probe.pred).first().cloned().ok_or(Stop::Failed("no unread target".into()))?;
            let id = find_id(ctx, p, state, target.seed).await?;
            call(ctx, p, json!([["Email/set",{"accountId":p.account,"update":{&id:{"keywords/$seen":true}}},"s"]]), &mut op).await.map_err(|r| refused("mark read", r))?;
            state.oracle.update(target.seed, |m| m.seen = true);
            changed = 1;
            rec.put("change", "Email/set keywords/$seen on the newest unread match");
        }
        _ => {
            let m = u64::from(ctx.config.operation).min(1000).min(before_count);
            let ids = query_all(ctx, p, json!({"inMailbox":state.inbox,"text":corpus::SEARCH_WORD}), m, 1000, &mut Calls::default()).await.map_err(|r| refused("setup listing", r))?;
            let moved: Vec<u32> = state.oracle.matches(&*probe.pred).iter().take(m as usize).map(|x| x.seed).collect();
            let mut run = SetRun::default();
            bulk_email_set(ctx, p, &ids, &json!({"mailboxIds":{&state.archive:true}}), false, &mut run).await?;
            op = run.calls;
            for seed in moved {
                state.oracle.update(seed, |x| x.folder = Folder::Archive);
            }
            changed = m;
            rec.put("change", format!("move of the {m} newest matches to Archive (Email/set mailboxIds)"));
        }
    }
    let change_ms = ms(started.elapsed());
    let expected = state.oracle.matches(&*probe.pred).len() as u64;
    let after_change = Instant::now();
    let mut stats = Calls::default();
    let first = call(ctx, p, calls_for.clone(), &mut stats).await.map_err(|r| refused("first query", r))?;
    let first_total = result(&first, 0)["total"].as_u64();
    let visible = wait_for(VISIBILITY_LIMIT, || {
        let calls_for = calls_for.clone();
        async move { call(ctx, p, calls_for, &mut Calls::default()).await.ok().and_then(|v| result(&v, 0)["total"].as_u64()) == Some(expected) }
    })
    .await;
    let c1 = snapshot(&ctx.engine, Some(p));
    rec.put("messagesChanged", changed);
    rec.put("changeMs", change_ms);
    rec.put("changeRequests", op.report());
    rec.put("totalBefore", before_count);
    rec.put("expectedTotal", expected);
    rec.put("firstQueryTotal", first_total);
    rec.put("firstQueryMs", stats.latencies[0]);
    rec.put("firstQueryWasCurrent", first_total == Some(expected));
    rec.put("correctAfterChangeMs", visible.map(|_| ms(after_change.elapsed())));
    rec.put("counters", delta(&c0, &c1, changed));
    rec.check("search-reflects-change", visible.is_some(), json!({"expected":expected,"firstQueryTotal":first_total,"limitSeconds":VISIBILITY_LIMIT.as_secs()}));
    Ok(())
}

pub(super) async fn s_rebuild(ctx: &Ctx, rec: &mut Record, state: &SearchState) -> std::result::Result<(), Stop> {
    let mut stored = 0u64;
    for p in &ctx.principals {
        stored += ctx.store.count_account_messages(&p.account).await.unwrap_or(0);
    }
    let disk_before = disk(ctx).await;
    let started = Instant::now();
    let summary = ctx.engine.rebuild_search_index_all().await.map_err(|e| Stop::Refused { stage: "rebuild".into(), refusal: json!({"error":e.to_string()}) })?;
    let elapsed = started.elapsed();
    rec.put("rebuildMs", ms(elapsed));
    rec.put("messagesIndexed", summary.indexed);
    rec.put("accounts", summary.accounts);
    rec.put("documentsPerSecond", summary.indexed as f64 / elapsed.as_secs_f64().max(1e-9));
    rec.put("diskBefore", disk_before);
    rec.put("disk", disk(ctx).await);
    rec.check("rebuild-indexed-every-stored-message", summary.indexed as u64 == stored && ctx.engine.search().num_docs() == stored, json!({"stored":stored,"indexed":summary.indexed,"documents":ctx.engine.search().num_docs()}));
    let common = spec("after-rebuild", corpus::SEARCH_WORD, |m| m.folder == Folder::Inbox && m.has("quarterly"));
    let p = ctx.principal("search");
    let mut stats = Calls::default();
    let value = call(ctx, p, calls(ctx, p, filter(state, &common)?, &common), &mut stats).await.map_err(|r| refused("query after rebuild", r))?;
    rec.put("firstQueryAfterRebuildMs", stats.latencies[0]);
    verify(rec, &common, &value, &state.oracle.matches(&*common.pred));
    Ok(())
}

#[test]
fn search_specs_have_exact_oracle_counts_on_a_small_corpus() {
    let oracle = Oracle::build(2_000);
    let specs = specs(2_000);
    let count = |name: &str| oracle.matches(&*specs.iter().find(|s| s.name == format!("search-{name}")).unwrap().pred).len();
    assert_eq!(count("rare-term"), 1);
    assert_eq!(count("common-term"), 400);
    assert_eq!(count("phrase"), 40);
    assert_eq!(count("and-terms"), 40 + (1..=2000u32).filter(|s| s % 10 == 6 && s % 7 == 2).count());
    assert_eq!(count("prefix"), 3);
    assert_eq!(count("cc"), 80);
    assert_eq!(count("subject"), 50);
    assert_eq!(count("has-attachment"), 100);
    assert_eq!(count("filename"), 100);
    assert_eq!(count("is-unread"), 600);
    assert_eq!(count("is-flagged"), 40);
    assert_eq!(count("all-folders"), 440);
    assert!(count("date-range") > 0 && count("larger") > 0 && count("smaller") > 0);
    assert_eq!(oracle.matches(&|m: &Meta| m.has("qua")).len(), 0);
}
