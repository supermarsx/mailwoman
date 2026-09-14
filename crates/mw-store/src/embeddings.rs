//! 0022 message-embedding repository (26.19 t19, SPEC gap A8): additive,
//! dual-backend `Store` methods over `message_embeddings` (0022, both dialects)
//! backing the **opt-in** semantic search re-rank.
//!
//! An embedding is a content-DERIVED projection of a message's subject/body, so it
//! carries the same at-rest posture as every other content-derived blob: the
//! vector is sealed under the store ServerKey (XChaCha20-Poly1305) and only ever
//! held decrypted in memory, exactly like `notes.body_html_sealed`.
//!
//! The plaintext wire form is deliberately boring — `dim` little-endian `f32`s,
//! `dim * 4` bytes — so there is no serialization dependency and no version byte
//! to get wrong. `dim` and `model` are stored ALONGSIDE the vector because they
//! are the guard that lets [`Store::get_message_embedding`]'s caller notice a row
//! written under a *different* embedding model and skip it, degrading that hit to
//! its lexical rank rather than cosine-comparing incompatible vectors.
//!
//! Nothing here runs in a default deployment: rows are written only when an Assist
//! embedding provider is configured AND the `search-semantic` capability is
//! granted. Authored in the SQLite `?n` style so it runs identically on SQLite or
//! Postgres through [`crate::backend`].

use chrono::Utc;

use crate::backend::q;
use crate::{Store, StoreError};

/// Upper bound on a stored vector's dimensionality. Well above every embedding
/// model in circulation (OpenAI `text-embedding-3-large` is 3072); its job is to
/// stop a malformed or hostile provider response from being persisted at all.
pub const MAX_EMBEDDING_DIM: usize = 8192;

/// One message's embedding vector (0022), decrypted.
#[derive(Debug, Clone, PartialEq)]
pub struct MessageEmbedding {
    /// The store's opaque message stable id — the primary key.
    pub stable_id: String,
    pub account_id: String,
    /// The embedding model id the vector was produced by. Empty when the provider
    /// does not report one; a caller comparing models must treat empty as
    /// "unknown", not as a match.
    pub model: String,
    /// The vector itself. `vector.len()` is the authoritative dimension — the
    /// `dim` column is validated against it on read, so a truncated row is
    /// rejected rather than silently re-ranked against a short vector.
    pub vector: Vec<f32>,
}

impl Store {
    /// Upsert one message's embedding, sealing the vector. Re-embedding a message
    /// REPLACES its row (the stable id is the primary key), so a model change
    /// cannot accumulate stale vectors alongside fresh ones.
    ///
    /// Refuses an empty vector or one longer than [`MAX_EMBEDDING_DIM`]: both mean
    /// the provider returned something unusable, and persisting it would only
    /// produce a row that every reader has to skip.
    pub async fn put_message_embedding(
        &self,
        stable_id: &str,
        account_id: &str,
        model: &str,
        vector: &[f32],
    ) -> Result<(), StoreError> {
        if vector.is_empty() || vector.len() > MAX_EMBEDDING_DIM {
            return Err(StoreError::Corrupt(format!(
                "embedding dimension {} out of range (1..={MAX_EMBEDDING_DIM})",
                vector.len()
            )));
        }
        let sealed = self.key.seal(&encode_vector(vector))?;
        let now = Utc::now().to_rfc3339();
        q("INSERT INTO message_embeddings
                 (stable_id, account_id, model, dim, vector_sealed, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)
             ON CONFLICT(stable_id) DO UPDATE SET
                 account_id = excluded.account_id, model = excluded.model,
                 dim = excluded.dim, vector_sealed = excluded.vector_sealed,
                 updated_at = excluded.updated_at")
        .bind(stable_id)
        .bind(account_id)
        .bind(model)
        .bind(vector.len() as i64)
        .bind(sealed)
        .bind(&now)
        .execute(&self.backend)
        .await?;
        Ok(())
    }

    /// Read + unseal one message's embedding, if present.
    ///
    /// A row whose sealed payload does not decode to exactly `dim` `f32`s is
    /// treated as **absent** (`Ok(None)`), not as an error: the embedding cache is
    /// regenerable, and a corrupt row must degrade that hit to its lexical rank
    /// rather than fail the user's search. An unseal failure (wrong ServerKey)
    /// propagates, because that is a deployment fault, not a cache fault.
    pub async fn get_message_embedding(
        &self,
        stable_id: &str,
    ) -> Result<Option<MessageEmbedding>, StoreError> {
        let row = q("SELECT stable_id, account_id, model, dim, vector_sealed
                       FROM message_embeddings WHERE stable_id = ?1")
        .bind(stable_id)
        .fetch_optional(&self.backend)
        .await?;
        let Some(r) = row else { return Ok(None) };
        let plain = self.key.open(&r.get_blob("vector_sealed"))?;
        let dim = r.get_i64("dim");
        let Some(vector) = decode_vector(&plain, dim) else {
            return Ok(None);
        };
        Ok(Some(MessageEmbedding {
            stable_id: r.get_string("stable_id"),
            account_id: r.get_string("account_id"),
            model: r.get_string("model"),
            vector,
        }))
    }

    /// Drop one message's embedding (called when a message is expunged, so the
    /// cache does not outlive the message it describes).
    pub async fn delete_message_embedding(&self, stable_id: &str) -> Result<(), StoreError> {
        q("DELETE FROM message_embeddings WHERE stable_id = ?1")
            .bind(stable_id)
            .execute(&self.backend)
            .await?;
        Ok(())
    }

    /// Drop every embedding for an account. The operator escape hatch after an
    /// embedding-model change and the disconnect-account cleanup path; returns the
    /// number of rows removed.
    pub async fn delete_account_message_embeddings(
        &self,
        account_id: &str,
    ) -> Result<u64, StoreError> {
        Ok(q("DELETE FROM message_embeddings WHERE account_id = ?1")
            .bind(account_id)
            .execute(&self.backend)
            .await?)
    }

    /// How many embeddings an account has cached (observability + tests).
    pub async fn count_message_embeddings(&self, account_id: &str) -> Result<i64, StoreError> {
        Ok(
            q("SELECT COUNT(*) FROM message_embeddings WHERE account_id = ?1")
                .bind(account_id)
                .fetch_scalar_i64(&self.backend)
                .await?,
        )
    }
}

/// `f32`s → little-endian bytes. Fixed-width and endian-explicit so a store file
/// written on one host reads identically on another.
fn encode_vector(v: &[f32]) -> Vec<u8> {
    let mut out = Vec::with_capacity(v.len() * 4);
    for f in v {
        out.extend_from_slice(&f.to_le_bytes());
    }
    out
}

/// Little-endian bytes → `f32`s, cross-checked against the row's `dim`. Returns
/// `None` (caller treats the row as absent) when the payload length disagrees with
/// `dim`, when `dim` is not a plausible dimension, or when any component is not
/// finite — a NaN would poison a cosine comparison silently.
fn decode_vector(bytes: &[u8], dim: i64) -> Option<Vec<f32>> {
    let dim = usize::try_from(dim).ok()?;
    if dim == 0 || dim > MAX_EMBEDDING_DIM || bytes.len() != dim * 4 {
        return None;
    }
    let mut out = Vec::with_capacity(dim);
    for chunk in bytes.as_chunks::<4>().0 {
        let f = f32::from_le_bytes(*chunk);
        if !f.is_finite() {
            return None;
        }
        out.push(f);
    }
    Some(out)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ServerKey;

    async fn store() -> Store {
        Store::open_in_memory(ServerKey::generate()).await.unwrap()
    }

    #[tokio::test]
    async fn round_trip_and_replace() {
        let s = store().await;
        assert!(s.get_message_embedding("m1").await.unwrap().is_none());
        assert_eq!(s.count_message_embeddings("a1").await.unwrap(), 0);

        let v = vec![0.5_f32, -0.25, 0.125];
        s.put_message_embedding("m1", "a1", "mock-embed", &v)
            .await
            .unwrap();
        let got = s.get_message_embedding("m1").await.unwrap().unwrap();
        assert_eq!(got.vector, v);
        assert_eq!(got.model, "mock-embed");
        assert_eq!(got.account_id, "a1");
        assert_eq!(s.count_message_embeddings("a1").await.unwrap(), 1);

        // Re-embedding REPLACES rather than accumulating, dimension included.
        let v2 = vec![1.0_f32, 2.0, 3.0, 4.0];
        s.put_message_embedding("m1", "a1", "other-embed", &v2)
            .await
            .unwrap();
        let got = s.get_message_embedding("m1").await.unwrap().unwrap();
        assert_eq!(got.vector, v2);
        assert_eq!(got.model, "other-embed");
        assert_eq!(s.count_message_embeddings("a1").await.unwrap(), 1);
    }

    #[tokio::test]
    async fn vector_is_sealed_at_rest_not_plaintext() {
        let s = store().await;
        // A component with an unmistakable byte pattern: if the column held
        // plaintext, these 4 bytes would appear verbatim in the stored blob.
        let v = vec![f32::from_le_bytes([0xDE, 0xAD, 0xBE, 0x4F]); 8];
        s.put_message_embedding("m1", "a1", "m", &v).await.unwrap();

        let row = q("SELECT vector_sealed FROM message_embeddings WHERE stable_id = ?1")
            .bind("m1")
            .fetch_one(s.backend())
            .await
            .unwrap();
        let stored = row.get_blob("vector_sealed");
        let plain = encode_vector(&v);
        assert_ne!(stored, plain, "vector must not be stored in the clear");
        assert!(
            !stored
                .windows(4)
                .any(|w| w == [0xDE, 0xAD, 0xBE, 0x4F].as_slice()),
            "no plaintext component survives in the sealed blob"
        );
        // ...and it still opens back to exactly what went in.
        assert_eq!(
            s.get_message_embedding("m1").await.unwrap().unwrap().vector,
            v
        );
    }

    #[tokio::test]
    async fn refuses_unusable_vectors() {
        let s = store().await;
        assert!(s.put_message_embedding("m1", "a1", "m", &[]).await.is_err());
        let too_long = vec![0.0_f32; MAX_EMBEDDING_DIM + 1];
        assert!(
            s.put_message_embedding("m1", "a1", "m", &too_long)
                .await
                .is_err()
        );
        assert_eq!(s.count_message_embeddings("a1").await.unwrap(), 0);
    }

    #[tokio::test]
    async fn delete_paths() {
        let s = store().await;
        s.put_message_embedding("m1", "a1", "m", &[1.0])
            .await
            .unwrap();
        s.put_message_embedding("m2", "a1", "m", &[1.0])
            .await
            .unwrap();
        s.put_message_embedding("m3", "a2", "m", &[1.0])
            .await
            .unwrap();

        s.delete_message_embedding("m1").await.unwrap();
        assert!(s.get_message_embedding("m1").await.unwrap().is_none());
        assert_eq!(s.count_message_embeddings("a1").await.unwrap(), 1);

        assert_eq!(s.delete_account_message_embeddings("a1").await.unwrap(), 1);
        assert_eq!(s.count_message_embeddings("a1").await.unwrap(), 0);
        // Another account's cache is untouched.
        assert_eq!(s.count_message_embeddings("a2").await.unwrap(), 1);
        // Deleting an absent row is a no-op, not an error.
        s.delete_message_embedding("nope").await.unwrap();
    }

    #[test]
    fn vector_codec_round_trips_and_rejects_bad_payloads() {
        let v = vec![0.0_f32, 1.5, -2.25, f32::MIN_POSITIVE];
        let bytes = encode_vector(&v);
        assert_eq!(bytes.len(), 16);
        assert_eq!(decode_vector(&bytes, 4), Some(v));

        // Length disagreeing with `dim` → absent, never a short/garbage vector.
        assert_eq!(decode_vector(&bytes, 3), None);
        assert_eq!(decode_vector(&bytes, 5), None);
        assert_eq!(decode_vector(&bytes, 0), None);
        assert_eq!(decode_vector(&bytes, -1), None);
        assert_eq!(decode_vector(&[], 0), None);
        // A non-finite component would poison cosine — rejected.
        assert_eq!(decode_vector(&encode_vector(&[f32::NAN]), 1), None);
        assert_eq!(decode_vector(&encode_vector(&[f32::INFINITY]), 1), None);
    }

    #[tokio::test]
    async fn corrupt_row_reads_as_absent_not_as_error() {
        let s = store().await;
        s.put_message_embedding("m1", "a1", "m", &[1.0, 2.0, 3.0])
            .await
            .unwrap();
        // Simulate a row whose `dim` no longer matches its payload (the shape a
        // half-written or externally-tampered cache row would take).
        q("UPDATE message_embeddings SET dim = ?1 WHERE stable_id = ?2")
            .bind(7_i64)
            .bind("m1")
            .execute(s.backend())
            .await
            .unwrap();
        assert!(s.get_message_embedding("m1").await.unwrap().is_none());
    }
}
