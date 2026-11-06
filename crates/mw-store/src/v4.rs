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

use crate::{Row, Store, StoreError, q};

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
        let next = q(
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
        .fetch_scalar_i64(&self.backend)
        .await?;
        Ok(next as u64)
    }

    /// The current `(account, type)` crypto/security state counter (0 when none).
    pub async fn current_crypto_state(
        &self,
        account_id: &str,
        type_name: &str,
    ) -> Result<u64, StoreError> {
        let n = q(
            "SELECT COALESCE(MAX(state), 0) FROM crypto_changes WHERE account_id = ?1 AND type = ?2",
        )
        .bind(account_id)
        .bind(type_name)
        .fetch_scalar_i64(&self.backend)
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
        let rows = q("SELECT state, object_id, op FROM crypto_changes
             WHERE account_id = ?1 AND type = ?2 AND state > ?3 ORDER BY state ASC")
        .bind(account_id)
        .bind(type_name)
        .bind(since as i64)
        .fetch_all(&self.backend)
        .await?;
        Ok(rows
            .iter()
            .map(|r| CryptoChangeRow {
                state: r.get_i64("state") as u64,
                object_id: r.get_string("object_id"),
                op: r.get_string("op"),
            })
            .collect())
    }

    // ── crypto_keys (keyring: own + harvested/contact PUBLIC keys) ───────────

    /// Insert or replace a key/cert row. `encrypted_private_backup` is opaque —
    /// stored verbatim, NEVER decrypted (plan §1.2 / risk #4).
    pub async fn upsert_crypto_key(&self, row: &CryptoKeyRow) -> Result<(), StoreError> {
        q(
            "INSERT INTO crypto_keys
                 (id, account_id, kind, is_own, addresses_json, fingerprint, key_id, algorithm,
                  created_at, expires_at, public_key, cert_pem, trust, autocrypt, source,
                  encrypted_private_backup, verified_at, key_history_json)
             VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11,?12,?13,?14,?15,?16,?17,?18)
             ON CONFLICT(id) DO UPDATE SET
                 kind=excluded.kind, is_own=excluded.is_own, addresses_json=excluded.addresses_json,
                 fingerprint=excluded.fingerprint, key_id=excluded.key_id, algorithm=excluded.algorithm,
                 created_at=excluded.created_at, expires_at=excluded.expires_at,
                 public_key=excluded.public_key, cert_pem=excluded.cert_pem, trust=excluded.trust,
                 autocrypt=excluded.autocrypt, source=excluded.source,
                 encrypted_private_backup=excluded.encrypted_private_backup,
                 verified_at=excluded.verified_at, key_history_json=excluded.key_history_json",
        )
        .bind(&row.id)
        .bind(&row.account_id)
        .bind(&row.kind)
        .bind(i64::from(row.is_own))
        .bind(&row.addresses_json)
        .bind(&row.fingerprint)
        .bind(&row.key_id)
        .bind(&row.algorithm)
        .bind(&row.created_at)
        .bind(row.expires_at.as_deref())
        .bind(row.public_key.as_deref())
        .bind(row.cert_pem.as_deref())
        .bind(&row.trust)
        .bind(i64::from(row.autocrypt))
        .bind(&row.source)
        .bind(row.encrypted_private_backup.as_deref())
        .bind(row.verified_at.as_deref())
        .bind(&row.key_history_json)
        .execute(&self.backend)
        .await?;
        Ok(())
    }

    /// All of an account's keys (own + harvested/contact), newest first.
    pub async fn list_crypto_keys(
        &self,
        account_id: &str,
    ) -> Result<Vec<CryptoKeyRow>, StoreError> {
        let rows = q(
            "SELECT * FROM crypto_keys WHERE account_id = ?1 ORDER BY is_own DESC, created_at DESC",
        )
        .bind(account_id)
        .fetch_all(&self.backend)
        .await?;
        Ok(rows.iter().map(crypto_key_from_row).collect())
    }

    /// Fetch one key by id (scoped to the account).
    pub async fn get_crypto_key(
        &self,
        account_id: &str,
        id: &str,
    ) -> Result<Option<CryptoKeyRow>, StoreError> {
        let row = q("SELECT * FROM crypto_keys WHERE account_id = ?1 AND id = ?2")
            .bind(account_id)
            .bind(id)
            .fetch_optional(&self.backend)
            .await?;
        Ok(row.as_ref().map(crypto_key_from_row))
    }

    /// Delete one key by id (scoped to the account).
    pub async fn delete_crypto_key(&self, account_id: &str, id: &str) -> Result<(), StoreError> {
        q("DELETE FROM crypto_keys WHERE account_id = ?1 AND id = ?2")
            .bind(account_id)
            .bind(id)
            .execute(&self.backend)
            .await?;
        Ok(())
    }

    // ── key_associations (TOFU per-address history) ──────────────────────────

    /// Record a first-seen (address → key) TOFU association.
    pub async fn add_key_association(&self, row: &KeyAssociationRow) -> Result<(), StoreError> {
        q(
            "INSERT INTO key_associations (account_id, address, crypto_key_id, seen_at)
             VALUES (?1, ?2, ?3, ?4)",
        )
        .bind(&row.account_id)
        .bind(&row.address)
        .bind(&row.crypto_key_id)
        .bind(&row.seen_at)
        .execute(&self.backend)
        .await?;
        Ok(())
    }

    /// The key-id history for one address, oldest first (TOFU key-change detect).
    pub async fn key_associations_for(
        &self,
        account_id: &str,
        address: &str,
    ) -> Result<Vec<KeyAssociationRow>, StoreError> {
        let rows = q(
            "SELECT account_id, address, crypto_key_id, seen_at FROM key_associations
             WHERE account_id = ?1 AND address = ?2 ORDER BY seen_at ASC",
        )
        .bind(account_id)
        .bind(address)
        .fetch_all(&self.backend)
        .await?;
        Ok(rows
            .iter()
            .map(|r| KeyAssociationRow {
                account_id: r.get_string("account_id"),
                address: r.get_string("address"),
                crypto_key_id: r.get_string("crypto_key_id"),
                seen_at: r.get_string("seen_at"),
            })
            .collect())
    }

    // ── security_verdicts (lazy verdict cache, keyed by email + raw_hash) ─────

    /// The cached verdict for an email IF the raw hash still matches (else `None`
    /// so the caller recomputes). Returns the raw `verdict_json` blob.
    pub async fn get_security_verdict(
        &self,
        email_id: &str,
        raw_hash: &str,
    ) -> Result<Option<Vec<u8>>, StoreError> {
        let row =
            q("SELECT verdict_json FROM security_verdicts WHERE email_id = ?1 AND raw_hash = ?2")
                .bind(email_id)
                .bind(raw_hash)
                .fetch_optional(&self.backend)
                .await?;
        Ok(row.map(|r| r.get_blob("verdict_json")))
    }

    /// Cache a computed verdict (replaces any stale row for the email).
    pub async fn upsert_security_verdict(
        &self,
        row: &SecurityVerdictRow,
    ) -> Result<(), StoreError> {
        q(
            "INSERT INTO security_verdicts (email_id, account_id, raw_hash, verdict_json, computed_at)
             VALUES (?1, ?2, ?3, ?4, ?5)
             ON CONFLICT(email_id) DO UPDATE SET
                 raw_hash=excluded.raw_hash, verdict_json=excluded.verdict_json,
                 computed_at=excluded.computed_at",
        )
        .bind(&row.email_id)
        .bind(&row.account_id)
        .bind(&row.raw_hash)
        .bind(&row.verdict_json)
        .bind(&row.computed_at)
        .execute(&self.backend)
        .await?;
        Ok(())
    }

    // ── dlp_audit (redacted — matched detector/rule, NEVER content) ───────────

    /// Append one redacted DLP audit row (plan §1.8).
    pub async fn insert_dlp_audit(&self, row: &DlpAuditRow) -> Result<(), StoreError> {
        q("INSERT INTO dlp_audit
                 (id, account_id, at, rule_id, rule_name, action, matched_detectors_json, blocked)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)")
        .bind(&row.id)
        .bind(&row.account_id)
        .bind(&row.at)
        .bind(&row.rule_id)
        .bind(&row.rule_name)
        .bind(&row.action)
        .bind(&row.matched_detectors_json)
        .bind(i64::from(row.blocked))
        .execute(&self.backend)
        .await?;
        Ok(())
    }

    /// The account's DLP audit trail, newest first (admin review / test assert).
    pub async fn list_dlp_audit(&self, account_id: &str) -> Result<Vec<DlpAuditRow>, StoreError> {
        let rows = q(
            "SELECT id, account_id, at, rule_id, rule_name, action, matched_detectors_json, blocked
             FROM dlp_audit WHERE account_id = ?1 ORDER BY at DESC",
        )
        .bind(account_id)
        .fetch_all(&self.backend)
        .await?;
        Ok(rows
            .iter()
            .map(|r| DlpAuditRow {
                id: r.get_string("id"),
                account_id: r.get_string("account_id"),
                at: r.get_string("at"),
                rule_id: r.get_string("rule_id"),
                rule_name: r.get_string("rule_name"),
                action: r.get_string("action"),
                matched_detectors_json: r.get_string("matched_detectors_json"),
                blocked: r.get_i64("blocked") != 0,
            })
            .collect())
    }

    // ── sender_controls ──────────────────────────────────────────────────────

    /// Record a sender-control action (linked to the real MailRule it made, if any).
    pub async fn insert_sender_control(&self, row: &SenderControlRow) -> Result<(), StoreError> {
        q(
            "INSERT INTO sender_controls (account_id, address, thread_id, action, mail_rule_id, at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
        )
        .bind(&row.account_id)
        .bind(row.address.as_deref())
        .bind(row.thread_id.as_deref())
        .bind(&row.action)
        .bind(row.mail_rule_id.as_deref())
        .bind(&row.at)
        .execute(&self.backend)
        .await?;
        Ok(())
    }

    /// The account's sender controls, newest first.
    pub async fn list_sender_controls(
        &self,
        account_id: &str,
    ) -> Result<Vec<SenderControlRow>, StoreError> {
        let rows = q(
            "SELECT account_id, address, thread_id, action, mail_rule_id, at
             FROM sender_controls WHERE account_id = ?1 ORDER BY at DESC",
        )
        .bind(account_id)
        .fetch_all(&self.backend)
        .await?;
        Ok(rows
            .iter()
            .map(|r| SenderControlRow {
                account_id: r.get_string("account_id"),
                address: r.get_opt_string("address"),
                thread_id: r.get_opt_string("thread_id"),
                action: r.get_string("action"),
                mail_rule_id: r.get_opt_string("mail_rule_id"),
                at: r.get_string("at"),
            })
            .collect())
    }

    // ── store_key_material (PQC-hybrid-wrapped seal key at rest, §1.7) ─────────

    /// The current PQC-wrapped seal-key material (there is at most one row).
    pub async fn get_store_key_material(&self) -> Result<Option<StoreKeyMaterialRow>, StoreError> {
        let row = q(
            "SELECT id, wrapped_seal_key, suite, created_at FROM store_key_material
             ORDER BY created_at DESC LIMIT 1",
        )
        .fetch_optional(&self.backend)
        .await?;
        Ok(row.map(|r| StoreKeyMaterialRow {
            id: r.get_string("id"),
            wrapped_seal_key: r.get_blob("wrapped_seal_key"),
            suite: r.get_string("suite"),
            created_at: r.get_string("created_at"),
        }))
    }

    /// Store (or replace) the PQC-wrapped seal-key material + its suite tag.
    pub async fn upsert_store_key_material(
        &self,
        row: &StoreKeyMaterialRow,
    ) -> Result<(), StoreError> {
        q(
            "INSERT INTO store_key_material (id, wrapped_seal_key, suite, created_at)
             VALUES (?1, ?2, ?3, ?4)
             ON CONFLICT(id) DO UPDATE SET
                 wrapped_seal_key=excluded.wrapped_seal_key, suite=excluded.suite,
                 created_at=excluded.created_at",
        )
        .bind(&row.id)
        .bind(&row.wrapped_seal_key)
        .bind(&row.suite)
        .bind(&row.created_at)
        .execute(&self.backend)
        .await?;
        Ok(())
    }
}

/// Map a `crypto_keys` row (`SELECT *`) to a [`CryptoKeyRow`].
fn crypto_key_from_row(r: &Row) -> CryptoKeyRow {
    CryptoKeyRow {
        id: r.get_string("id"),
        account_id: r.get_string("account_id"),
        kind: r.get_string("kind"),
        is_own: r.get_i64("is_own") != 0,
        addresses_json: r.get_string("addresses_json"),
        fingerprint: r.get_string("fingerprint"),
        key_id: r.get_string("key_id"),
        algorithm: r.get_string("algorithm"),
        created_at: r.get_string("created_at"),
        expires_at: r.get_opt_string("expires_at"),
        public_key: r.get_opt_string("public_key"),
        cert_pem: r.get_opt_string("cert_pem"),
        trust: r.get_string("trust"),
        autocrypt: r.get_i64("autocrypt") != 0,
        source: r.get_string("source"),
        encrypted_private_backup: r.get_opt_blob("encrypted_private_backup"),
        verified_at: r.get_opt_string("verified_at"),
        key_history_json: r.get_string("key_history_json"),
    }
}
