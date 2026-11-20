//! Request-local Focus contact candidates, preserving the legacy top20 rule.
//! SQL conservatively bounds encoded JSON output before materialization (6x
//! text bytes plus fixed card/token/edge overhead). The
//! unused contact JSON/photo bytes are never selected. This does not make LIKE
//! scans or database sort work independent of the address-book size.
use crate::{
    Store, StoreError,
    backend::{Dialect, q},
};
use serde::Deserialize;
use std::{
    collections::{BTreeMap, BTreeSet},
    sync::LazyLock,
};

pub const MAX_FOCUS_SENDERS: usize = 128;
const MAX_INPUT_BYTES: usize = 64 * 1024;
const MAX_CARD_BYTES: usize = 64 * 1024;
const MAX_TEXT_BYTES: usize = 256 * 1024;
const MAX_OUTPUT_BYTES: usize = 8 * 1024 * 1024;

/// A data-only result. No contact or Focus admission decision is cached.
pub enum FocusContactLookup {
    Batched {
        cards: Vec<FocusContactCard>,
        by_sender: BTreeMap<String, Vec<usize>>,
    },
    /// Unsupported input/storage shape or output budget: use the original
    /// per-sender reader, whose ranking/decoding semantics remain authoritative.
    LegacyFallback,
}
pub struct FocusContactCard {
    pub id: String,
    pub vcard_raw: String,
}
impl std::fmt::Debug for FocusContactLookup {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::LegacyFallback => f.write_str("LegacyFallback"),
            Self::Batched { cards, by_sender } => f
                .debug_struct("Batched")
                .field("cards", &cards.len())
                .field("senders", &by_sender.len())
                .finish(),
        }
    }
}
#[derive(Deserialize)]
#[allow(dead_code)] // Decode every text column touched by the old ContactRow reader.
struct CardPayload {
    token: String,
    id: String,
    address_book_id: String,
    uid: String,
    etag: Option<String>,
    vcard_raw: String,
    full_name: String,
    is_favorite: i64,
    photo_blob_id: Option<String>,
    pgp_key: Option<String>,
    smime_cert: Option<String>,
}
#[derive(Deserialize)]
struct Edge {
    token: String,
}
impl Store {
    /// One fresh owner-filtered statement for up to128 distinct senders. Each
    /// sender keeps the old favorite/name/id order and independent20-row cutoff.
    /// Repeated sender inputs are deduplicated; no cached decision is returned.
    pub async fn focus_contact_candidates(
        &self,
        account: &str,
        senders: &[String],
    ) -> Result<FocusContactLookup, StoreError> {
        if account.is_empty() || account.len() > 1024 || senders.len() > MAX_FOCUS_SENDERS {
            return Ok(FocusContactLookup::LegacyFallback);
        }
        let mut unique = BTreeSet::new();
        let mut bytes = account.len();
        for sender in senders {
            bytes = bytes.saturating_add(sender.len());
            if sender.len() > 1024 || bytes > MAX_INPUT_BYTES {
                return Ok(FocusContactLookup::LegacyFallback);
            }
            unique.insert(sender.as_str());
        }
        let senders = unique.into_iter().collect::<Vec<_>>();
        if senders.is_empty() {
            return Ok(FocusContactLookup::Batched {
                cards: Vec::new(),
                by_sender: BTreeMap::new(),
            });
        }
        let input=serde_json::to_string(&senders.iter().map(|sender|serde_json::json!({"sender":sender,"pattern":crate::v3::contact_autocomplete_pattern(sender)})).collect::<Vec<_>>()).map_err(|_|StoreError::Corrupt("Focus contact input encoding failed".into()))?;
        if input.len() > MAX_INPUT_BYTES {
            return Ok(FocusContactLookup::LegacyFallback);
        }
        let rows = q(batch_sql(self.backend.dialect()))
            .bind(account)
            .bind(input)
            .fetch_all(&self.backend)
            .await?;
        let mut cards = Vec::new();
        let mut tokens = BTreeMap::new();
        let mut edges = Vec::new();
        for row in rows {
            let tag = row
                .try_get_opt_string("tag")?
                .ok_or_else(|| StoreError::Corrupt("Focus contact result tag missing".into()))?;
            if tag == "overflow" {
                return Ok(FocusContactLookup::LegacyFallback);
            }
            let payload = row
                .try_get_opt_string("payload")?
                .ok_or_else(|| StoreError::Corrupt("Focus contact result missing".into()))?;
            if tag == "card" {
                let value: CardPayload = serde_json::from_str(&payload)
                    .map_err(|_| StoreError::Corrupt("Focus contact candidate malformed".into()))?;
                let index = cards.len();
                tokens.insert(value.token, index);
                cards.push(FocusContactCard {
                    id: value.id,
                    vcard_raw: value.vcard_raw,
                });
            } else if tag == "edge" {
                let sender = row.get_i64("sender_index");
                let value: Edge = serde_json::from_str(&payload)
                    .map_err(|_| StoreError::Corrupt("Focus contact edge malformed".into()))?;
                edges.push((sender, value.token));
            } else {
                return Err(StoreError::Corrupt(
                    "Focus contact result tag invalid".into(),
                ));
            }
        }
        let mut by_sender = senders
            .iter()
            .map(|sender| ((*sender).to_owned(), Vec::new()))
            .collect::<BTreeMap<_, _>>();
        for (index, token) in edges {
            let sender = usize::try_from(index)
                .ok()
                .and_then(|index| senders.get(index))
                .ok_or_else(|| StoreError::Corrupt("Focus contact sender index invalid".into()))?;
            let card = *tokens
                .get(&token)
                .ok_or_else(|| StoreError::Corrupt("Focus contact edge binding invalid".into()))?;
            by_sender
                .get_mut(*sender)
                .expect("validated sender")
                .push(card);
        }
        Ok(FocusContactLookup::Batched { cards, by_sender })
    }
}
fn batch_sql(dialect: Dialect) -> &'static str {
    static SQLITE: LazyLock<String> = LazyLock::new(|| build_sql(Dialect::Sqlite));
    static POSTGRES: LazyLock<String> = LazyLock::new(|| build_sql(Dialect::Postgres));
    match dialect {
        Dialect::Sqlite => SQLITE.as_str(),
        Dialect::Postgres => POSTGRES.as_str(),
    }
}
fn build_sql(dialect: Dialect) -> String {
    let (input, locator, json, cast, like, shape) = match dialect {
        Dialect::Sqlite => (
            "SELECT CAST(key AS BIGINT) AS sender_index,json_extract(value,'$.pattern') AS pattern FROM json_each(?2)",
            "c.rowid",
            "json_object",
            "",
            "LIKE",
            "typeof(c.id)!='text' OR typeof(c.address_book_id)!='text' OR typeof(c.uid)!='text' OR typeof(c.etag) NOT IN ('text','null') OR typeof(c.vcard_raw)!='text' OR typeof(c.full_name)!='text' OR typeof(c.is_favorite)!='integer' OR typeof(c.photo_blob_id) NOT IN ('text','null') OR typeof(c.pgp_key) NOT IN ('text','null') OR typeof(c.smime_cert) NOT IN ('text','null') OR typeof(c.json) NOT IN ('blob','null')",
        ),
        Dialect::Postgres => (
            "SELECT CAST(ordinality-1 AS BIGINT) AS sender_index,value->>'pattern' AS pattern FROM json_array_elements(?2::json) WITH ORDINALITY",
            "c.ctid",
            "json_build_object",
            "::text",
            "ILIKE",
            "FALSE",
        ),
    };
    let text = [
        "id",
        "address_book_id",
        "uid",
        "etag",
        "vcard_raw",
        "full_name",
        "photo_blob_id",
        "pgp_key",
        "smime_cert",
    ];
    let bytes = text
        .iter()
        .map(|key| format!("COALESCE(octet_length(c.{key}),0)"))
        .collect::<Vec<_>>()
        .join("+");
    let large = text
        .iter()
        .map(|key| {
            format!(
                "COALESCE(octet_length(c.{key}),0)>{}",
                if *key == "vcard_raw" {
                    MAX_CARD_BYTES
                } else {
                    MAX_TEXT_BYTES
                }
            )
        })
        .collect::<Vec<_>>()
        .join(" OR ");
    // JSON escapes can turn one control byte into six bytes. All copied text
    // columns are included above;512 bytes/card covers fixed keys, punctuation,
    // favorite integer and the bounded rowid/ctid token.128 bytes/edge covers its
    // token JSON and row metadata.1024 bytes covers fixed output framing. This
    // deliberately rejects some legal low-escape text rather than underbudget.
    let fields = text
        .iter()
        .map(|key| format!("'{key}',c.{key}"))
        .collect::<Vec<_>>()
        .join(",");
    format!(
        "WITH input AS ({input}),ranked AS (SELECT i.sender_index,{locator} AS locator,ROW_NUMBER() OVER(PARTITION BY i.sender_index ORDER BY c.is_favorite DESC,c.full_name,c.id) AS candidate_rank FROM input i JOIN contacts c ON(c.full_name {like} i.pattern ESCAPE '\\' OR c.vcard_raw {like} i.pattern ESCAPE '\\') JOIN address_books ab ON ab.id=c.address_book_id WHERE ab.account_id=?1),matched AS (SELECT sender_index,locator,candidate_rank FROM ranked WHERE candidate_rank<=20),selected AS (SELECT DISTINCT locator FROM matched),budget AS (SELECT CASE WHEN (6*COALESCE(SUM({bytes}),0)+512*COUNT(*)+128*(SELECT COUNT(*) FROM matched)+1024)<={MAX_OUTPUT_BYTES} AND COALESCE(MAX(CASE WHEN {large} OR ({shape}) THEN 1 ELSE 0 END),0)=0 THEN 1 ELSE 0 END AS ok FROM selected s JOIN contacts c ON {locator}=s.locator) SELECT tag,sender_index,payload FROM (SELECT 'overflow' AS tag,CAST(0 AS BIGINT) AS sender_index,CAST(0 AS BIGINT) AS candidate_rank,'{{}}' AS payload FROM budget WHERE ok=0 UNION ALL SELECT 'card',CAST(0 AS BIGINT),CAST(0 AS BIGINT),{json}('token',CAST({locator} AS TEXT),{fields},'is_favorite',c.is_favorite){cast} FROM budget CROSS JOIN selected s JOIN contacts c ON {locator}=s.locator WHERE budget.ok=1 UNION ALL SELECT 'edge',m.sender_index,m.candidate_rank,{json}('token',CAST(m.locator AS TEXT)){cast} FROM budget CROSS JOIN matched m WHERE budget.ok=1) output ORDER BY tag,sender_index,candidate_rank"
    )
}
#[cfg(test)]
#[path = "focus_contacts_tests.rs"]
mod tests;
