//! A8 (26.19): opt-in semantic re-rank of `Email/query` results (SPEC §10.4,
//! §14.3).
//!
//! ## What this is, and what it deliberately is not
//! Retrieval is unchanged. Tantivy's BM25 produces the candidate set exactly as it
//! did in 26.18, and this module only re-orders the head of that list. It runs
//! **only** when the request carries `filter.semantic == true` *and* the
//! deployment has attached an [`EmbeddingProvider`]; with either absent,
//! `Email/query` never calls in here at all and the default search path is
//! byte-identical to the previous release. There is no ANN index and no vector
//! crate: cosine over a few hundred candidates is microseconds, and an ANN
//! structure is not worth its dependency weight for a per-query opt-in.
//!
//! ## Data egress posture
//! A document embedding means sending that document's text to whatever endpoint
//! Assist is configured against, so nothing here happens implicitly. Embeddings
//! are **never** produced at ingest — that would proxy every message a deployment
//! receives to a third party as a side effect of installing a feature. They are
//! produced only for messages that a user's own opt-in semantic search actually
//! surfaced, through the same [`crate::v7`]-mounted gateway that enforces the
//! capability grant, the data-class ceiling, the rate limit and the content-free
//! audit row.
//!
//! The ceiling also bounds the *content* of what leaves, not just the audit row
//! that describes it: [`EmbeddingProvider::content_scope`] reports the classes the
//! gateway's effective clamp permits, and [`embed_input`] builds the payload from
//! that. A provider that says nothing forwards the least — decoded attachment text
//! is dropped unless it is explicitly permitted. An unenforced control that reports
//! itself enforced is worse than no control, because the audit row is the
//! compliance artifact.
//!
//! ## Degradation
//! Every failure mode degrades to the lexical ordering rather than to an error: a
//! provider timeout, a dimension change under a populated cache, a corrupt cache
//! row, a store read failure. A search must not fail because a re-rank could not
//! run. Each of those paths is recorded in the returned [`RerankReport`].

use std::collections::HashMap;
use std::sync::Arc;

use mw_search::{IndexDoc, rerank_by_cosine};
use mw_store::Store;

/// How many of the lexical hits are eligible for re-ranking (DQ-3). The tail keeps
/// its BM25 order untouched, which bounds both the store reads and the cosine
/// sweep regardless of how large the result set is.
pub const RERANK_TOP_N: usize = 200;

/// How many un-embedded hits a single semantic query will embed on demand. Each
/// one is a network round-trip to the configured endpoint, so this is the knob
/// that keeps an interactive search interactive; the cache converges over
/// successive searches rather than paying for the whole candidate set at once.
pub const LAZY_FILL_MAX: usize = 32;

/// Cap on the characters handed to the embedding endpoint for one document.
/// Embedding models truncate at their own context limit anyway; capping here keeps
/// a pathological message from dominating the request.
pub const MAX_EMBED_CHARS: usize = 6000;

/// The content classes an embedding provider is permitted to receive.
///
/// The engine has no `DataScope` — that type lives in `mw-assist`, which the engine
/// must not depend on — so a provider projects its *effective* clamp down to this
/// and the engine applies it to the payload before anything leaves. Every field
/// defaults to the excluding value, so a provider that implements nothing forwards
/// the least.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct EmbedScope {
    /// Whether decoded `text/*` attachment content may form part of a document's
    /// embed payload. **Default `false`.**
    ///
    /// `IndexDoc::attachment_text` is populated unconditionally at ingest because
    /// the lexical index needs it (W19), so this flag is the only thing standing
    /// between an attachment's decoded body and the configured endpoint.
    pub include_attachments: bool,
}

/// The seam between the engine and whatever produces embeddings.
///
/// `mw-engine` must not depend on `mw-assist` (and `mw-assist` is free to depend
/// on the engine's neighbours), so the provider is injected as a trait object at
/// mount time by `mw-server` — the same shape as every other V6/V7 hook. The
/// engine never learns what endpoint is behind it.
#[async_trait::async_trait]
pub trait EmbeddingProvider: Send + Sync + 'static {
    /// Embed one piece of text on behalf of `account_id`. The account travels with
    /// the call so the implementation can scope it and name it in the content-free
    /// audit row — an embedding request is real data egress, and an audit trail
    /// that cannot say whose mail was sent is not an audit trail. `Err` carries a
    /// human-readable reason and is always treated as "re-rank unavailable", never
    /// as a query failure.
    async fn embed(&self, account_id: &str, text: &str) -> std::result::Result<Vec<f32>, String>;

    /// The configured embedding model's id, used to detect vectors cached under a
    /// *different* model. An empty string means "unknown", which disables the
    /// model check and leaves dimensionality as the only guard.
    fn model_id(&self) -> String {
        String::new()
    }

    /// The content classes this provider's clamp actually permits, applied to the
    /// payload by [`embed_input`].
    ///
    /// This is what makes the deployment's data-class ceiling a control rather than
    /// a label: without it the clamp is computed, named in the audit row, and never
    /// consulted again. The default excludes everything optional, so failing to
    /// implement it can only forward less.
    fn content_scope(&self) -> EmbedScope {
        EmbedScope::default()
    }
}

/// What a re-rank pass actually did. Returned rather than logged-and-forgotten so
/// tests can pin the degradation paths by observation instead of by absence.
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct RerankReport {
    /// Hits in the eligible window (`min(hits, RERANK_TOP_N)`).
    pub considered: usize,
    /// Hits that carried a usable vector and therefore took part in the ordering.
    pub scored: usize,
    /// Hits embedded on demand during this query.
    pub filled: usize,
    /// Cached rows skipped because their model or dimension did not match the
    /// query embedding — the "model changed under a populated cache" case.
    pub skipped_mismatch: usize,
    /// Set when the pass returned the lexical order unchanged, naming why.
    pub degraded: Option<&'static str>,
}

impl RerankReport {
    fn degraded(reason: &'static str) -> Self {
        Self {
            degraded: Some(reason),
            ..Self::default()
        }
    }

    /// Whether this pass left the ordering exactly as the lexical search produced
    /// it.
    #[must_use]
    pub fn is_lexical(&self) -> bool {
        self.scored < 2
    }
}

/// The text a message is embedded from: subject first (it carries the most signal
/// per token), then body, then — **only when `scope.include_attachments`** — the
/// indexed attachment text, truncated on a `char` boundary to [`MAX_EMBED_CHARS`].
///
/// The attachment branch is the enforcement point for the data-class ceiling on this
/// path. `doc.attachment_text` is the decoded body of every `text/*` attachment,
/// populated at ingest whether or not Assist is configured, so appending it
/// unconditionally would put attachment content in the payload of any message whose
/// subject and body fit under the cap — while the audit row recorded `attach=false`.
#[must_use]
pub fn embed_input(doc: &IndexDoc, scope: EmbedScope) -> String {
    let attachments = if scope.include_attachments {
        doc.attachment_text.as_str()
    } else {
        ""
    };
    let mut out = String::new();
    for part in [doc.subject.as_str(), doc.body.as_str(), attachments] {
        let part = part.trim();
        if part.is_empty() {
            continue;
        }
        if !out.is_empty() {
            out.push('\n');
        }
        out.push_str(part);
        if out.chars().count() >= MAX_EMBED_CHARS {
            break;
        }
    }
    truncate_chars(&out, MAX_EMBED_CHARS)
}

fn truncate_chars(s: &str, max: usize) -> String {
    match s.char_indices().nth(max) {
        Some((byte, _)) => s[..byte].to_string(),
        None => s.to_string(),
    }
}

/// Re-order the head of `ids` by cosine similarity to `query_text`'s embedding.
///
/// Never returns an error: every failure path leaves `ids` untouched and is named
/// in the returned [`RerankReport`]. Callers therefore need no fallback branch —
/// the degraded result *is* the lexical result.
pub async fn rerank_hits(
    provider: &Arc<dyn EmbeddingProvider>,
    store: &Store,
    index: &Arc<mw_search::Index>,
    account_id: &str,
    query_text: &str,
    ids: &mut [String],
) -> RerankReport {
    let query_text = query_text.trim();
    if ids.len() < 2 || query_text.is_empty() {
        return RerankReport::degraded("nothing to re-rank");
    }

    let query_vec = match provider.embed(account_id, query_text).await {
        Ok(v) if !v.is_empty() => v,
        Ok(_) => return RerankReport::degraded("provider returned an empty query embedding"),
        Err(e) => {
            tracing::debug!("semantic re-rank unavailable: query embed failed: {e}");
            return RerankReport::degraded("query embedding failed");
        }
    };
    let model = provider.model_id();

    let window = ids.len().min(RERANK_TOP_N);
    let mut report = RerankReport {
        considered: window,
        ..RerankReport::default()
    };

    // 1. Whatever is already cached.
    let mut vectors: HashMap<String, Vec<f32>> = HashMap::with_capacity(window);
    let mut missing: Vec<String> = Vec::new();
    for id in &ids[..window] {
        match store.get_message_embedding(id).await {
            Ok(Some(row)) => {
                let model_mismatch =
                    !model.is_empty() && !row.model.is_empty() && row.model != model;
                if model_mismatch
                    || row.vector.len() != query_vec.len()
                    || row.account_id != account_id
                {
                    report.skipped_mismatch += 1;
                } else {
                    vectors.insert(id.clone(), row.vector);
                }
            }
            Ok(None) => missing.push(id.clone()),
            Err(e) => {
                // A cache read failure costs this hit its ranking, nothing more.
                tracing::debug!("semantic re-rank: embedding read failed for {id}: {e}");
            }
        }
    }

    // 2. Fill a bounded number of gaps on demand, concurrently — sequential
    //    round-trips would put seconds of endpoint latency on an interactive
    //    search. Documents are read from the search index (which already holds the
    //    indexed text) rather than re-parsed out of the store.
    //
    //    The payload is built against the provider's own effective content scope, so
    //    a class the deployment ceiling excludes is absent from the request rather
    //    than merely absent from the audit row.
    let content_scope = provider.content_scope();
    let mut pending = Vec::new();
    for id in missing.into_iter().take(LAZY_FILL_MAX) {
        let Ok(Some(doc)) = index.fetch_doc(&id) else {
            continue; // nothing indexed under this id — leave it lexical
        };
        let text = embed_input(&doc, content_scope);
        if text.is_empty() {
            continue;
        }
        let provider = Arc::clone(provider);
        let account = account_id.to_string();
        pending.push(tokio::spawn(async move {
            let v = provider.embed(&account, &text).await;
            (id, v)
        }));
    }
    for handle in pending {
        let Ok((id, result)) = handle.await else {
            continue; // the embed task panicked or was cancelled — stay lexical
        };
        match result {
            Ok(v) if v.len() == query_vec.len() => {
                if let Err(e) = store
                    .put_message_embedding(&id, account_id, &model, &v)
                    .await
                {
                    // Persisting is a cache optimisation; the vector is still good
                    // for THIS query.
                    tracing::debug!("semantic re-rank: embedding write failed for {id}: {e}");
                }
                vectors.insert(id, v);
                report.filled += 1;
            }
            Ok(_) => report.skipped_mismatch += 1,
            Err(e) => tracing::debug!("semantic re-rank: document embed failed for {id}: {e}"),
        }
    }

    // 3. Permute. With fewer than two usable vectors this is the identity.
    report.scored = rerank_by_cosine(&mut ids[..window], &query_vec, &vectors);
    if report.scored < 2 {
        report.degraded = Some("no usable embeddings for this result set");
    }
    report
}

#[cfg(test)]
mod tests {
    use super::*;
    use mw_store::ServerKey;
    use std::sync::atomic::{AtomicUsize, Ordering};

    /// A deterministic, content-DEPENDENT embedder: each token contributes to one
    /// bucket of a fixed-width vector. Two texts sharing vocabulary land near each
    /// other, so a test can assert an ordering that is genuinely justified by the
    /// embeddings rather than by a hardcoded permutation.
    ///
    /// It also RECORDS every text it was handed. An egress test has to assert on the
    /// payload that actually reached the provider; asserting on what `embed_input`
    /// returns in isolation would only prove the helper, not the call path.
    struct HashEmbedder {
        dim: usize,
        model: String,
        calls: AtomicUsize,
        fail: bool,
        scope: EmbedScope,
        seen: std::sync::Mutex<Vec<String>>,
    }

    impl HashEmbedder {
        fn new(dim: usize) -> Arc<Self> {
            Arc::new(Self {
                dim,
                model: "test-hash-embed".to_string(),
                calls: AtomicUsize::new(0),
                fail: false,
                scope: EmbedScope::default(),
                seen: std::sync::Mutex::new(Vec::new()),
            })
        }
        /// An embedder whose clamp permits attachment content — the deliberate
        /// opt-in, used to prove the flag is read rather than that the branch is dead.
        fn permitting_attachments(dim: usize) -> Arc<Self> {
            Arc::new(Self {
                dim,
                model: "test-hash-embed".to_string(),
                calls: AtomicUsize::new(0),
                fail: false,
                scope: EmbedScope {
                    include_attachments: true,
                },
                seen: std::sync::Mutex::new(Vec::new()),
            })
        }
        fn failing() -> Arc<Self> {
            Arc::new(Self {
                dim: 8,
                model: "test-hash-embed".to_string(),
                calls: AtomicUsize::new(0),
                fail: true,
                scope: EmbedScope::default(),
                seen: std::sync::Mutex::new(Vec::new()),
            })
        }
        /// Everything this provider was asked to embed, in call order.
        fn payloads(&self) -> Vec<String> {
            self.seen.lock().expect("seen lock").clone()
        }
    }

    #[async_trait::async_trait]
    impl EmbeddingProvider for HashEmbedder {
        async fn embed(
            &self,
            _account_id: &str,
            text: &str,
        ) -> std::result::Result<Vec<f32>, String> {
            self.calls.fetch_add(1, Ordering::SeqCst);
            self.seen.lock().expect("seen lock").push(text.to_string());
            if self.fail {
                return Err("endpoint unavailable".to_string());
            }
            let mut v = vec![0.0_f32; self.dim];
            for token in text.split(|c: char| !c.is_alphanumeric()) {
                if token.is_empty() {
                    continue;
                }
                let mut h: u64 = 1469598103934665603;
                for b in token.to_lowercase().bytes() {
                    h ^= u64::from(b);
                    h = h.wrapping_mul(1099511628211);
                }
                v[(h % self.dim as u64) as usize] += 1.0;
            }
            if v.iter().all(|x| *x == 0.0) {
                v[0] = 1.0; // never return a zero vector
            }
            Ok(v)
        }
        fn model_id(&self) -> String {
            self.model.clone()
        }
        fn content_scope(&self) -> EmbedScope {
            self.scope
        }
    }

    fn doc(id: &str, subject: &str, body: &str) -> IndexDoc {
        IndexDoc {
            stable_id: id.to_string(),
            account_id: "acct".to_string(),
            mailbox_id: "INBOX".to_string(),
            subject: subject.to_string(),
            body: body.to_string(),
            ..IndexDoc::default()
        }
    }

    /// Three indexed messages plus an in-memory store.
    async fn fixture() -> (Store, Arc<mw_search::Index>) {
        let store = Store::open_in_memory(ServerKey::generate()).await.unwrap();
        let index = Arc::new(mw_search::Index::open_in_ram().unwrap());
        index
            .upsert_batch(&[
                doc("m1", "Weekly status", "the sprint board and standup notes"),
                doc(
                    "m2",
                    "Invoice 4471",
                    "payment terms invoice billing amount due",
                ),
                doc("m3", "Lunch", "sandwiches and coffee on friday"),
            ])
            .unwrap();
        (store, index)
    }

    fn ids(v: &[&str]) -> Vec<String> {
        v.iter().map(|s| (*s).to_string()).collect()
    }

    #[tokio::test]
    async fn reranks_by_embedding_and_caches_what_it_computed() {
        let (store, index) = fixture().await;
        let p = HashEmbedder::new(64);
        let provider: Arc<dyn EmbeddingProvider> = p.clone();

        // Lexical order puts the invoice last; the query is about billing.
        let mut hits = ids(&["m1", "m3", "m2"]);
        let report = rerank_hits(
            &provider,
            &store,
            &index,
            "acct",
            "invoice billing payment",
            &mut hits,
        )
        .await;

        assert_eq!(report.degraded, None);
        assert_eq!(report.considered, 3);
        assert_eq!(report.filled, 3, "cold cache embeds every candidate");
        assert_eq!(report.scored, 3);
        assert_eq!(hits.first().map(String::as_str), Some("m2"));
        assert_eq!(hits.len(), 3, "re-rank is a permutation, not a filter");

        // Everything it computed is now cached under this account...
        assert_eq!(store.count_message_embeddings("acct").await.unwrap(), 3);
        // ...so a second identical query embeds only the QUERY, not the documents.
        let before = p.calls.load(Ordering::SeqCst);
        let mut again = ids(&["m1", "m3", "m2"]);
        let report2 = rerank_hits(
            &provider,
            &store,
            &index,
            "acct",
            "invoice billing payment",
            &mut again,
        )
        .await;
        assert_eq!(report2.filled, 0);
        assert_eq!(p.calls.load(Ordering::SeqCst) - before, 1);
        assert_eq!(again, hits, "the cached pass reproduces the same ordering");
    }

    #[tokio::test]
    async fn provider_failure_degrades_to_lexical() {
        let (store, index) = fixture().await;
        let provider: Arc<dyn EmbeddingProvider> = HashEmbedder::failing();
        let original = ids(&["m1", "m3", "m2"]);
        let mut hits = original.clone();
        let report = rerank_hits(&provider, &store, &index, "acct", "invoice", &mut hits).await;
        assert_eq!(report.degraded, Some("query embedding failed"));
        assert_eq!(hits, original);
        assert_eq!(store.count_message_embeddings("acct").await.unwrap(), 0);
    }

    #[tokio::test]
    async fn dimension_change_under_a_populated_cache_degrades_to_lexical() {
        let (store, index) = fixture().await;
        // Cache was populated by a 16-wide model...
        for id in ["m1", "m2", "m3"] {
            store
                .put_message_embedding(id, "acct", "test-hash-embed", &[0.5_f32; 16])
                .await
                .unwrap();
        }
        // ...and the deployment now runs an 8-wide one.
        let p = HashEmbedder::new(8);
        let provider: Arc<dyn EmbeddingProvider> = p.clone();
        let original = ids(&["m1", "m3", "m2"]);
        let mut hits = original.clone();
        let report = rerank_hits(&provider, &store, &index, "acct", "invoice", &mut hits).await;

        assert_eq!(report.skipped_mismatch, 3, "every stale row is skipped");
        assert_eq!(
            report.filled, 0,
            "a mismatched row is not silently re-embedded"
        );
        assert_eq!(report.scored, 0);
        assert_eq!(
            report.degraded,
            Some("no usable embeddings for this result set")
        );
        assert_eq!(hits, original, "results degrade to lexical, never corrupt");
    }

    #[tokio::test]
    async fn model_change_at_the_same_dimension_is_also_skipped() {
        let (store, index) = fixture().await;
        // Same width as the live provider, different model — the case a dimension
        // check alone would miss and silently mis-rank.
        for id in ["m1", "m2", "m3"] {
            store
                .put_message_embedding(id, "acct", "some-other-model", &[0.5_f32; 8])
                .await
                .unwrap();
        }
        let provider: Arc<dyn EmbeddingProvider> = HashEmbedder::new(8);
        let original = ids(&["m1", "m3", "m2"]);
        let mut hits = original.clone();
        let report = rerank_hits(&provider, &store, &index, "acct", "invoice", &mut hits).await;
        assert_eq!(report.skipped_mismatch, 3);
        assert_eq!(hits, original);
    }

    #[tokio::test]
    async fn another_accounts_cached_vector_is_never_used() {
        let (store, index) = fixture().await;
        for id in ["m1", "m2", "m3"] {
            store
                .put_message_embedding(id, "other-acct", "test-hash-embed", &[0.5_f32; 8])
                .await
                .unwrap();
        }
        let provider: Arc<dyn EmbeddingProvider> = HashEmbedder::new(8);
        let mut hits = ids(&["m1", "m3", "m2"]);
        let report = rerank_hits(&provider, &store, &index, "acct", "invoice", &mut hits).await;
        assert_eq!(report.skipped_mismatch, 3);
    }

    #[tokio::test]
    async fn empty_query_and_single_hit_are_no_ops() {
        let (store, index) = fixture().await;
        let p = HashEmbedder::new(8);
        let provider: Arc<dyn EmbeddingProvider> = p.clone();

        let mut one = ids(&["m1"]);
        assert!(
            rerank_hits(&provider, &store, &index, "acct", "invoice", &mut one)
                .await
                .degraded
                .is_some()
        );
        let mut hits = ids(&["m1", "m2"]);
        assert!(
            rerank_hits(&provider, &store, &index, "acct", "   ", &mut hits)
                .await
                .degraded
                .is_some()
        );
        // Neither reached the provider at all.
        assert_eq!(p.calls.load(Ordering::SeqCst), 0);
    }

    #[tokio::test]
    async fn only_the_top_n_window_is_touched() {
        let store = Store::open_in_memory(ServerKey::generate()).await.unwrap();
        let index = Arc::new(mw_search::Index::open_in_ram().unwrap());
        let n = RERANK_TOP_N + 5;
        let docs: Vec<IndexDoc> = (0..n)
            .map(|i| doc(&format!("m{i}"), "subject", "body text"))
            .collect();
        index.upsert_batch(&docs).unwrap();
        // Pre-cache the whole set so the LAZY_FILL_MAX bound isn't what limits us.
        let provider: Arc<dyn EmbeddingProvider> = HashEmbedder::new(8);
        for (i, d) in docs.iter().enumerate() {
            let v = provider
                .embed("acct", &embed_input(d, provider.content_scope()))
                .await
                .unwrap();
            store
                .put_message_embedding(&format!("m{i}"), "acct", "test-hash-embed", &v)
                .await
                .unwrap();
        }

        let mut hits: Vec<String> = (0..n).map(|i| format!("m{i}")).collect();
        let tail_before = hits[RERANK_TOP_N..].to_vec();
        let report = rerank_hits(&provider, &store, &index, "acct", "body text", &mut hits).await;

        assert_eq!(report.considered, RERANK_TOP_N);
        assert_eq!(
            hits[RERANK_TOP_N..],
            tail_before[..],
            "hits past the window keep their lexical order"
        );
        assert_eq!(hits.len(), n);
    }

    #[tokio::test]
    async fn lazy_fill_is_bounded_per_query() {
        let store = Store::open_in_memory(ServerKey::generate()).await.unwrap();
        let index = Arc::new(mw_search::Index::open_in_ram().unwrap());
        let n = LAZY_FILL_MAX + 10;
        let docs: Vec<IndexDoc> = (0..n)
            .map(|i| {
                doc(
                    &format!("m{i}"),
                    &format!("subject {i}"),
                    "shared body text",
                )
            })
            .collect();
        index.upsert_batch(&docs).unwrap();

        let provider: Arc<dyn EmbeddingProvider> = HashEmbedder::new(16);
        let mut hits: Vec<String> = (0..n).map(|i| format!("m{i}")).collect();
        let report = rerank_hits(&provider, &store, &index, "acct", "shared body", &mut hits).await;

        assert_eq!(
            report.filled, LAZY_FILL_MAX,
            "one query does not embed everything"
        );
        assert_eq!(
            store.count_message_embeddings("acct").await.unwrap() as usize,
            LAZY_FILL_MAX
        );
        // ...and the cache converges: a second pass fills the remainder.
        let mut again: Vec<String> = (0..n).map(|i| format!("m{i}")).collect();
        let report2 =
            rerank_hits(&provider, &store, &index, "acct", "shared body", &mut again).await;
        assert_eq!(report2.filled, n - LAZY_FILL_MAX);
        assert_eq!(
            store.count_message_embeddings("acct").await.unwrap() as usize,
            n
        );
    }

    // ── End-to-end at the `Email/query` seam ─────────────────────────────────
    //
    // The unit tests above pin the re-rank in isolation; these drive the real
    // engine path — ingest → Tantivy → `Email/query` → JSON — because the claim
    // that matters ("the default path is unchanged") is a claim about that path,
    // not about this module.
    mod e2e {
        use super::*;
        use crate::backend::{MessageRef, RawMailboxRef, RawMessage};
        use crate::engine::Engine;
        use mw_store::{AccountKind, Credentials, MailboxUpsert, NewAccount};
        use serde_json::{Value, json};

        async fn engine() -> Engine {
            Engine::new(Store::open_in_memory(ServerKey::generate()).await.unwrap())
        }

        async fn account(e: &Engine) -> String {
            e.store()
                .create_account(
                    &NewAccount {
                        kind: AccountKind::Imap,
                        host: "imap.example.org",
                        port: 993,
                        tls: "implicit",
                        username: "me@example.org",
                        sync_policy_json: "{}",
                    },
                    &Credentials {
                        username: "me@example.org".into(),
                        password: "pw".into(),
                    },
                )
                .await
                .unwrap()
        }

        async fn mailbox(e: &Engine, account: &str) -> String {
            e.store()
                .upsert_mailbox(&MailboxUpsert {
                    account_id: account,
                    name: "INBOX",
                    role: Some("inbox"),
                    uidvalidity: 1,
                    uidnext: 0,
                    highestmodseq: 0,
                    total: 0,
                    unread: 0,
                    parent_id: None,
                })
                .await
                .unwrap()
        }

        async fn ingest(
            e: &Engine,
            account: &str,
            mb: &str,
            uid: u32,
            when: &str,
            subject: &str,
            body: &str,
        ) -> String {
            let raw = format!(
                "Message-ID: <m{uid}@x>\r\nFrom: s@example.com\r\nTo: r@example.com\r\n\
                 Subject: {subject}\r\nDate: Mon, 01 Jan 2024 00:00:00 +0000\r\n\r\n{body}"
            )
            .into_bytes();
            e.ingest(
                account,
                mb,
                &RawMessage {
                    message_ref: MessageRef::Imap {
                        mailbox: RawMailboxRef {
                            name: "INBOX".into(),
                            uidvalidity: 1,
                        },
                        uidvalidity: 1,
                        uid,
                    },
                    raw,
                    flags: Vec::new(),
                    internaldate: Some(when.to_string()),
                },
            )
            .await
            .unwrap()
        }

        /// Three messages that all match the word "report", with `receivedAt`
        /// increasing — so the default (newest-first) lexical order is the reverse
        /// of ingest order. Returns `(account, [oldest, middle, newest])`.
        async fn seeded(e: &Engine) -> (String, Vec<String>) {
            let acct = account(e).await;
            let mb = mailbox(e, &acct).await;
            // `a` is purely about the query term; `b` and `c` merely mention it
            // among a lot of unrelated vocabulary. A "report" query embedding is
            // therefore closest to `a` — and `a` is the OLDEST, so lexical
            // (newest-first) order buries it last. That gap is what the re-rank
            // has to close, and it is justified by the vectors rather than by a
            // hardcoded permutation.
            let a = ingest(e, &acct, &mb, 1, "2024-01-01T00:00:00Z", "Report", "report").await;
            let b = ingest(
                e,
                &acct,
                &mb,
                2,
                "2024-02-01T00:00:00Z",
                "Standup",
                "sprint board standup notes retro planning velocity report",
            )
            .await;
            let c = ingest(
                e,
                &acct,
                &mb,
                3,
                "2024-03-01T00:00:00Z",
                "Lunch",
                "sandwiches coffee friday cafeteria menu queue report",
            )
            .await;
            (acct, vec![a, b, c])
        }

        /// Run an `Email/query` exactly as the JMAP surface does — same args, same
        /// resolver — and return the ordered ids.
        async fn query(e: &Engine, acct: &str, text: &str, semantic: Option<bool>) -> Vec<String> {
            let mut filter = json!({ "text": text });
            if let Some(s) = semantic {
                filter["semantic"] = Value::Bool(s);
            }
            e.query_ids(acct, &json!({ "accountId": acct, "filter": filter }))
                .await
                .expect("Email/query")
        }

        #[tokio::test]
        async fn default_path_is_untouched_by_the_feature() {
            // Baseline: an engine with NO provider — literally the 26.18 code path.
            let base = engine().await;
            let (base_acct, _) = seeded(&base).await;
            let baseline = query(&base, &base_acct, "report", None).await;
            assert_eq!(baseline.len(), 3);

            // The same corpus on an engine WITH a provider attached.
            let e = engine().await;
            let (acct, ids) = seeded(&e).await;
            e.attach_embeddings(Some(HashEmbedder::new(256)));

            // A query with no `semantic` key, and one that sets it to false, must
            // both produce exactly the newest-first lexical order.
            let expect: Vec<String> = ids.iter().rev().cloned().collect();
            assert_eq!(query(&e, &acct, "report", None).await, expect);
            assert_eq!(query(&e, &acct, "report", Some(false)).await, expect);
            // ...and not one embedding was computed or stored along the way.
            assert_eq!(e.store().count_message_embeddings(&acct).await.unwrap(), 0);
        }

        #[tokio::test]
        async fn semantic_true_reorders_by_embedding() {
            let e = engine().await;
            let (acct, ids) = seeded(&e).await;
            e.attach_embeddings(Some(HashEmbedder::new(256)));

            let lexical = query(&e, &acct, "report", None).await;
            assert_eq!(
                lexical,
                ids.iter().rev().cloned().collect::<Vec<_>>(),
                "newest-first, so the on-topic message is LAST"
            );

            let semantic = query(&e, &acct, "report", Some(true)).await;
            assert_ne!(semantic, lexical, "the flag changed the ordering");
            assert_eq!(
                semantic.first(),
                ids.first(),
                "the message that is actually about the query wins on cosine                  despite being the oldest"
            );
            // Same set, different order — a re-rank, not a filter.
            let (mut a, mut b) = (semantic.clone(), lexical.clone());
            a.sort();
            b.sort();
            assert_eq!(a, b);
            // The ordering is justified by vectors that are now cached.
            assert_eq!(e.store().count_message_embeddings(&acct).await.unwrap(), 3);
        }

        #[tokio::test]
        async fn semantic_true_without_a_provider_stays_lexical() {
            let e = engine().await;
            let (acct, ids) = seeded(&e).await;
            // No `attach_embeddings` — the default deployment.
            let expect: Vec<String> = ids.iter().rev().cloned().collect();
            assert_eq!(query(&e, &acct, "report", Some(true)).await, expect);
            assert_eq!(e.store().count_message_embeddings(&acct).await.unwrap(), 0);
        }

        #[tokio::test]
        async fn detaching_the_provider_restores_the_lexical_path() {
            let e = engine().await;
            let (acct, ids) = seeded(&e).await;
            e.attach_embeddings(Some(HashEmbedder::new(256)));
            let reranked = query(&e, &acct, "report", Some(true)).await;

            e.attach_embeddings(None);
            let after = query(&e, &acct, "report", Some(true)).await;
            assert_ne!(after, reranked);
            assert_eq!(after, ids.iter().rev().cloned().collect::<Vec<_>>());
        }
    }

    #[test]
    fn embed_input_concatenates_and_truncates() {
        let permitted = EmbedScope {
            include_attachments: true,
        };
        let mut d = doc("m1", "Subject line", "Body text");
        d.attachment_text = "attachment words".to_string();
        assert_eq!(
            embed_input(&d, permitted),
            "Subject line\nBody text\nattachment words"
        );

        // Empty parts are skipped rather than leaving blank lines.
        let d = doc("m2", "", "only body");
        assert_eq!(embed_input(&d, permitted), "only body");
        assert!(embed_input(&doc("m3", "", ""), permitted).is_empty());

        // Truncation is char-boundary safe on multi-byte text.
        let mut d = doc("m4", "", "");
        d.body = "é".repeat(MAX_EMBED_CHARS + 100);
        let out = embed_input(&d, permitted);
        assert_eq!(out.chars().count(), MAX_EMBED_CHARS);
    }

    // ── S1(a): the data-class ceiling bounds the PAYLOAD, not just the audit row ──

    /// Attachment marker chosen to be absent from every other fixture string, so a
    /// substring assertion cannot pass or fail by accident.
    const ATTACH_MARKER: &str = "ZZATTACHSECRETZZ";

    /// A store + index holding one message whose subject and body are short enough
    /// that `MAX_EMBED_CHARS` truncation can never be what excludes the attachment —
    /// the exclusion has to come from the scope or not at all.
    async fn attachment_fixture() -> (Store, Arc<mw_search::Index>) {
        let store = Store::open_in_memory(ServerKey::generate()).await.unwrap();
        let index = Arc::new(mw_search::Index::open_in_ram().unwrap());
        let mut with_attachment = doc("m1", "Invoice 4471", "payment terms billing amount due");
        with_attachment.attachment_text = format!("{ATTACH_MARKER} decoded attachment body text");
        index
            .upsert_batch(&[
                with_attachment,
                doc("m2", "Lunch", "sandwiches and coffee on friday"),
            ])
            .unwrap();
        (store, index)
    }

    /// `embed_input` alone proves the helper; this drives `rerank_hits` and asserts on
    /// what the PROVIDER was handed, which is the thing that leaves the deployment.
    ///
    /// Fails against the pre-fix code: `embed_input` appended `doc.attachment_text`
    /// unconditionally, so the marker was in the payload of the lazy-fill request
    /// while the audit row for that same call read `attach=false`.
    #[tokio::test]
    async fn attachment_text_never_reaches_the_provider_by_default() {
        let (store, index) = attachment_fixture().await;
        let p = HashEmbedder::new(32);
        let provider: Arc<dyn EmbeddingProvider> = p.clone();

        let mut hits = ids(&["m1", "m2"]);
        let report = rerank_hits(&provider, &store, &index, "acct", "invoice", &mut hits).await;
        assert_eq!(report.filled, 2, "both documents were embedded on demand");

        let payloads = p.payloads();
        assert!(
            !payloads.iter().any(|t| t.contains(ATTACH_MARKER)),
            "decoded attachment content must not leave when the effective scope \
             excludes it; payloads were {payloads:?}"
        );
        // Positive control: the non-attachment content DID leave. Without this, a
        // provider that was never called would satisfy the assertion above and the
        // test would pass for the wrong reason.
        assert!(
            payloads.iter().any(|t| t.contains("payment terms")),
            "the message body is still embedded — this is a content clamp, not an \
             unwired provider; payloads were {payloads:?}"
        );

        // Second control, opposite direction: a provider whose clamp DOES permit
        // attachments still gets them. Without this the assertion above would also be
        // satisfied by an attachment branch that is simply dead.
        //
        // A FRESH fixture, because the pass above cached both vectors — against a warm
        // cache nothing is embedded, the payload list is empty, and the assertion below
        // would fail for a reason that has nothing to do with the scope.
        let (store, index) = attachment_fixture().await;
        let p = HashEmbedder::permitting_attachments(32);
        let provider: Arc<dyn EmbeddingProvider> = p.clone();
        let mut hits = ids(&["m1", "m2"]);
        rerank_hits(&provider, &store, &index, "acct", "invoice", &mut hits).await;
        assert!(
            p.payloads().iter().any(|t| t.contains(ATTACH_MARKER)),
            "an explicit opt-in still forwards attachment text"
        );
    }

    /// The default is the excluding one, so a provider that implements nothing
    /// forwards the least. Fails against the pre-fix code: `EmbedScope` did not exist.
    #[test]
    fn the_default_content_scope_excludes_attachments() {
        assert!(!EmbedScope::default().include_attachments);

        let mut d = doc("m1", "Subject line", "Body text");
        d.attachment_text = ATTACH_MARKER.to_string();
        let out = embed_input(&d, EmbedScope::default());
        assert_eq!(out, "Subject line\nBody text");
        assert!(!out.contains(ATTACH_MARKER));
    }
}
