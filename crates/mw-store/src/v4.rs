//! V4 (Crypto & Security) repository methods (plan §2.4) layered over [`Store`]:
//! the keyring (`crypto_keys` / `key_associations`), the lazy verdict cache
//! (`security_verdicts`), the redacted DLP audit (`dlp_audit`), sender controls
//! (`sender_controls`), the PQC-wrapped seal-key home (`store_key_material`), and
//! the per-account `crypto_changes` log.
//!
//! Same seam discipline as [`crate::v3`]: every value crosses as an opaque
//! primitive; enum-like fields (`kind`/`trust`/`source`/`action`/change `op`
//! /`type`) are plain strings the engine owns. PRIVACY (plan §1.2/§1.8/risk #4):
//! `encrypted_private_backup` is an OPAQUE client-encrypted blob the server never
//! decrypts; `dlp_audit` is redacted (matched detector/rule, never content).
//!
//! e0 ships the Row types + the working `crypto_changes` triple (mirrors the V3
//! `pim_changes` log, so the `CryptoKey`/`MailRule` `*/changes` + push contract
//! has a real state source). e6 fills the keyring / verdict / DLP / sender-control
//! CRUD over these tables.

use sqlx::Row;

use crate::{Store, StoreError};

/// A key/cert row (`crypto_keys`, plan §2.4). Own keys carry an opaque
/// `encrypted_private_backup` (never decrypted server-side); others are public.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CryptoKeyRow {
    pub id: String,
    pub account_id: String,
    pub kind: String,
    pub is_own: bool,
    pub addresses_json: String,
    pub fingerprint: String,
    pub key_id: String,
    pub algorithm: String,
    pub created_at: String,
    pub expires_at: Option<String>,
    pub public_key: Option<String>,
    pub cert_pem: Option<String>,
    pub trust: String,
    pub autocrypt: bool,
    pub source: String,
    pub encrypted_private_backup: Option<Vec<u8>>,
    pub verified_at: Option<String>,
    pub key_history_json: String,
}

/// A per-address → key association (`key_associations`) with a TOFU first-seen.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KeyAssociationRow {
    pub account_id: String,
    pub address: String,
    pub crypto_key_id: String,
    pub seen_at: String,
}

/// A cached security verdict (`security_verdicts`), keyed by email + raw-hash.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SecurityVerdictRow {
    pub email_id: String,
    pub account_id: String,
    pub raw_hash: String,
    pub verdict_json: Vec<u8>,
    pub computed_at: String,
}

/// One redacted DLP audit row (`dlp_audit`) — matched detector/rule, never content.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DlpAuditRow {
    pub id: String,
    pub account_id: String,
    pub at: String,
    pub rule_id: String,
    pub rule_name: String,
    pub action: String,
    pub matched_detectors_json: String,
    pub blocked: bool,
}

/// A sender-control row (`sender_controls`), linked to the real MailRule it made.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SenderControlRow {
    pub account_id: String,
    pub address: Option<String>,
    pub thread_id: Option<String>,
    pub action: String,
    pub mail_rule_id: Option<String>,
    pub at: String,
}

/// The PQC-hybrid-wrapped seal master key at rest (`store_key_material`, §1.7),
/// tagged with its crypto-agility algorithm suite.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StoreKeyMaterialRow {
    pub id: String,
    pub wrapped_seal_key: Vec<u8>,
    pub suite: String,
    pub created_at: String,
}

/// One row of the per-account `crypto_changes` log (state token + `*/changes`
/// input for `CryptoKey`/`MailRule`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CryptoChangeRow {
    pub state: u64,
    pub object_id: String,
    pub op: String,
}

impl Store {
    // ── crypto_changes (state tokens + `*/changes`) ─────────────────────────

    /// Append one crypto/security change and return the new `(account, type)`
    /// state. A single atomic `INSERT … SELECT MAX+1 … RETURNING` (same WAL-safe
    /// discipline as [`Store::record_pim_change`]). `type_name` is a ChangeType
    /// name (`"CryptoKey"` | `"MailRule"`).
    pub async fn record_crypto_change(
        &self,
        account_id: &str,
        type_name: &str,
        object_id: &str,
        op: &str,
    ) -> Result<u64, StoreError> {
        let now = chrono::Utc::now().to_rfc3339();
        let next: i64 = sqlx::query_scalar(
            "INSERT INTO crypto_changes (account_id, type, state, object_id, op, at)
             VALUES (
                 ?1, ?2,
                 (SELECT COALESCE(MAX(state), 0) + 1 FROM crypto_changes WHERE account_id = ?1 AND type = ?2),
                 ?3, ?4, ?5
             )
             RETURNING state",
        )
        .bind(account_id)
        .bind(type_name)
        .bind(object_id)
        .bind(op)
        .bind(&now)
        .fetch_one(&self.pool)
        .await?;
        Ok(next as u64)
    }

    /// The current `(account, type)` crypto/security state counter (0 when none).
    pub async fn current_crypto_state(
        &self,
        account_id: &str,
        type_name: &str,
    ) -> Result<u64, StoreError> {
        let n: i64 = sqlx::query_scalar(
            "SELECT COALESCE(MAX(state), 0) FROM crypto_changes WHERE account_id = ?1 AND type = ?2",
        )
        .bind(account_id)
        .bind(type_name)
        .fetch_one(&self.pool)
        .await?;
        Ok(n as u64)
    }

    /// The crypto/security change rows for a datatype since `since_state`, oldest
    /// first (the `*/changes` diff input).
    pub async fn crypto_changes_since(
        &self,
        account_id: &str,
        type_name: &str,
        since: u64,
    ) -> Result<Vec<CryptoChangeRow>, StoreError> {
        let rows = sqlx::query(
            "SELECT state, object_id, op FROM crypto_changes
             WHERE account_id = ?1 AND type = ?2 AND state > ?3 ORDER BY state ASC",
        )
        .bind(account_id)
        .bind(type_name)
        .bind(since as i64)
        .fetch_all(&self.pool)
        .await?;
        Ok(rows
            .iter()
            .map(|r| CryptoChangeRow {
                state: r.get::<i64, _>("state") as u64,
                object_id: r.get("object_id"),
                op: r.get("op"),
            })
            .collect())
    }
}
