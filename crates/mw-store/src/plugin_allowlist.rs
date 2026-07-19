//! 0014 admin-managed third-party component load allowlist (26.15 t15, TQ1/TQ2/TQ6).
//!
//! The trust store behind the ONLY security-core loosening in 26.15: it lets an admin
//! pin the exact SHA-256 of a specific NON-first-party component so `resolve_component`
//! (in `mw-server`) will load those byte-exact bytes — and nothing else.
//!
//! TRUST MODEL — read before touching:
//!   * This table is a SEPARATE, admin-managed fallback that the compiled-in first-party
//!     digest pin ALWAYS takes precedence over. `resolve_component` checks the frozen
//!     first-party table FIRST and terminally; it consults this allowlist ONLY for a
//!     non-first-party `plugin_id` (TQ2). A row whose `plugin_id` collides with a
//!     first-party id is therefore unreachable at load time regardless — but
//!     [`Store::put_plugin_allowlist`] ALSO rejects such a row at approve time
//!     (defense-in-depth anti-spoof), given the authoritative first-party id list which
//!     `mw-store` cannot know on its own (it lives in `mw-server`).
//!   * NO component bytes are stored here — only the pinned 64-hex identity + the admin
//!     approval provenance (`approved_by`/`approved_at`). Loading a component is
//!     authorized purely by a byte-exact SHA-256 match to a NON-REVOKED row.
//!   * `digest_hex` is canonical lowercase 64-hex. A non-canonical argument (upper-case,
//!     truncated, empty) can NEVER match — it is rejected at approve time and treated as
//!     "not approved" at verify time, so a malformed/empty digest is never a wildcard.

use chrono::Utc;

use crate::backend::q;
use crate::{Store, StoreError};

/// A row of the 0014 `plugin_allowlist` table: one admin-pinned (`plugin_id`,
/// `digest_hex`) identity plus its approval provenance and optional descriptive
/// metadata. `revoked` is the BIGINT-as-bool flag (never native BOOLEAN — the V6 lesson).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PluginAllowlistRow {
    pub plugin_id: String,
    /// The admin-pinned lowercase 64-hex SHA-256 of the exact reviewed component bytes.
    pub digest_hex: String,
    pub name: Option<String>,
    pub version: Option<String>,
    pub source: Option<String>,
    pub note: Option<String>,
    pub approved_by: String,
    pub approved_at: String,
    pub revoked: bool,
}

/// A failure from [`Store::put_plugin_allowlist`] — the two admit-time refusals plus the
/// underlying store error. Both refusals fail LOUDLY (never a silent partial write).
#[derive(Debug, thiserror::Error)]
pub enum PluginAllowlistError {
    /// The `plugin_id` collides with a first-party component id. A third-party allowlist
    /// entry can never shadow or spoof a first-party identity (TQ2 anti-spoof).
    #[error(
        "plugin id '{0}' collides with a first-party component id; a third-party allowlist \
         entry can never shadow or spoof a first-party identity"
    )]
    FirstPartyCollision(String),
    /// The digest is not canonical lowercase 64-hex (guards against a truncated /
    /// upper-case / empty digest being treated as a wildcard).
    #[error("digest must be exactly 64 lowercase hex characters (got {0:?})")]
    MalformedDigest(String),
    #[error(transparent)]
    Store(#[from] StoreError),
}

/// Whether `s` is exactly 64 lowercase hex characters (the only shape a real SHA-256
/// pin can take). Rejects upper-case, non-hex, short/long, and empty inputs — none of
/// which may ever be treated as an approved (or wildcard) digest.
pub(crate) fn is_canonical_digest(s: &str) -> bool {
    s.len() == 64
        && s.bytes()
            .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
}

fn row_to_allowlist(r: &crate::backend::Row) -> PluginAllowlistRow {
    PluginAllowlistRow {
        plugin_id: r.get_string("plugin_id"),
        digest_hex: r.get_string("digest_hex"),
        name: r.get_opt_string("name"),
        version: r.get_opt_string("version"),
        source: r.get_opt_string("source"),
        note: r.get_opt_string("note"),
        approved_by: r.get_string("approved_by"),
        approved_at: r.get_string("approved_at"),
        revoked: r.get_i64("revoked") != 0,
    }
}

impl Store {
    /// Every allowlist row (admin list/oversight), ordered by `(plugin_id, digest_hex)`.
    /// Includes revoked rows so the admin UI can show the full history.
    pub async fn list_plugin_allowlist(&self) -> Result<Vec<PluginAllowlistRow>, StoreError> {
        let rows = q(
            "SELECT plugin_id, digest_hex, name, version, source, note, \
                             approved_by, approved_at, revoked
                        FROM plugin_allowlist ORDER BY plugin_id ASC, digest_hex ASC",
        )
        .fetch_all(&self.backend)
        .await?;
        Ok(rows.iter().map(row_to_allowlist).collect())
    }

    /// The VERIFY-PATH check (TQ2/TQ4): is `digest_hex` an ACTIVE (non-revoked) admin pin
    /// for exactly this `plugin_id`? `resolve_component` calls this with the SHA-256 it
    /// just computed over the in-memory component bytes it is about to load — a match
    /// means "an admin approved these exact bytes for this exact id". A non-canonical
    /// digest can never match (so an empty/short/upper-case argument is never a wildcard),
    /// and a revoked or absent pin returns `false` (hard refuse at the caller).
    pub async fn is_third_party_digest_approved(
        &self,
        plugin_id: &str,
        digest_hex: &str,
    ) -> Result<bool, StoreError> {
        // A non-canonical digest can never equal a stored canonical pin; short-circuit
        // so a malformed/empty argument is never even compared.
        if !is_canonical_digest(digest_hex) {
            return Ok(false);
        }
        let row = q("SELECT 1 AS ok FROM plugin_allowlist
                       WHERE plugin_id = ?1 AND digest_hex = ?2 AND revoked = 0")
        .bind(plugin_id)
        .bind(digest_hex)
        .fetch_optional(&self.backend)
        .await?;
        Ok(row.is_some())
    }

    /// Approve (pin) an exact `(plugin_id, digest_hex)`. Rejects — loudly, writing
    /// nothing — a `plugin_id` that collides with any id in `reserved_first_party_ids`
    /// (TQ2 anti-spoof) or a non-canonical `digest_hex`. `mw-server` passes its
    /// authoritative first-party id list here; `mw-store` cannot know it on its own. An
    /// existing pin for the same key is re-approved (metadata refreshed, `revoked`
    /// cleared) so re-approving after a revoke is a deliberate, audited un-revoke.
    pub async fn put_plugin_allowlist(
        &self,
        row: &PluginAllowlistRow,
        reserved_first_party_ids: &[&str],
    ) -> Result<(), PluginAllowlistError> {
        if reserved_first_party_ids
            .iter()
            .any(|id| id.eq_ignore_ascii_case(row.plugin_id.trim()))
        {
            return Err(PluginAllowlistError::FirstPartyCollision(
                row.plugin_id.clone(),
            ));
        }
        if !is_canonical_digest(&row.digest_hex) {
            return Err(PluginAllowlistError::MalformedDigest(
                row.digest_hex.clone(),
            ));
        }
        q("INSERT INTO plugin_allowlist
               (plugin_id, digest_hex, name, version, source, note, approved_by, approved_at, revoked)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)
           ON CONFLICT(plugin_id, digest_hex) DO UPDATE SET
               name = excluded.name, version = excluded.version, source = excluded.source,
               note = excluded.note, approved_by = excluded.approved_by,
               approved_at = excluded.approved_at, revoked = 0")
        .bind(&row.plugin_id)
        .bind(&row.digest_hex)
        .bind(row.name.clone())
        .bind(row.version.clone())
        .bind(row.source.clone())
        .bind(row.note.clone())
        .bind(&row.approved_by)
        .bind(&row.approved_at)
        .bind(i64::from(row.revoked))
        .execute(&self.backend)
        .await
        .map_err(StoreError::from)?;
        Ok(())
    }

    /// Revoke a pin (TQ6): set `revoked = 1` for `(plugin_id, digest_hex)`. Effective on
    /// the NEXT load — `resolve_component` reads the allowlist fresh each load, so a
    /// revoked pin refuses immediately thereafter (hot-unloading an already-running
    /// instance is out of scope, matching existing enable/disable semantics). Returns
    /// whether a previously-active row was revoked.
    pub async fn revoke_plugin_allowlist(
        &self,
        plugin_id: &str,
        digest_hex: &str,
    ) -> Result<bool, StoreError> {
        let n = q("UPDATE plugin_allowlist SET revoked = 1
                     WHERE plugin_id = ?1 AND digest_hex = ?2 AND revoked = 0")
        .bind(plugin_id)
        .bind(digest_hex)
        .execute(&self.backend)
        .await?;
        Ok(n > 0)
    }

    /// Remove EVERY allowlist row for a plugin (uninstall/removal companion to
    /// [`Store::plugin_kv_purge`], PQ6). Returns the number of rows removed. After this
    /// the plugin is not loadable again until an admin re-approves a digest.
    pub async fn delete_plugin_allowlist(&self, plugin_id: &str) -> Result<u64, StoreError> {
        let n = q("DELETE FROM plugin_allowlist WHERE plugin_id = ?1")
            .bind(plugin_id)
            .execute(&self.backend)
            .await?;
        Ok(n)
    }
}

/// Build an allowlist row with the approval timestamp set to now (RFC 3339). A small
/// convenience for the admin API / CLI so callers do not re-implement the clock.
#[must_use]
pub fn new_allowlist_pin(
    plugin_id: impl Into<String>,
    digest_hex: impl Into<String>,
    approved_by: impl Into<String>,
    name: Option<String>,
    version: Option<String>,
    source: Option<String>,
    note: Option<String>,
) -> PluginAllowlistRow {
    PluginAllowlistRow {
        plugin_id: plugin_id.into(),
        digest_hex: digest_hex.into(),
        name,
        version,
        source,
        note,
        approved_by: approved_by.into(),
        approved_at: Utc::now().to_rfc3339(),
        revoked: false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ServerKey;

    async fn store() -> Store {
        Store::open_in_memory(ServerKey::generate()).await.unwrap()
    }

    const A_DIGEST: &str = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
    const B_DIGEST: &str = "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb";

    fn pin(plugin: &str, digest: &str) -> PluginAllowlistRow {
        new_allowlist_pin(plugin, digest, "admin@x", None, None, None, None)
    }

    #[test]
    fn canonical_digest_shape_is_strict() {
        assert!(is_canonical_digest(A_DIGEST));
        assert!(!is_canonical_digest(""), "empty is never a digest");
        assert!(
            !is_canonical_digest(&A_DIGEST[..63]),
            "63 chars is too short"
        );
        assert!(
            !is_canonical_digest(&A_DIGEST.to_uppercase()),
            "upper-case is not canonical"
        );
        assert!(
            !is_canonical_digest(&format!("{}g", &A_DIGEST[..63])),
            "non-hex char rejected"
        );
    }

    #[tokio::test]
    async fn approve_then_verify_exact_digest() {
        let s = store().await;
        s.put_plugin_allowlist(&pin("third-party-x", A_DIGEST), &["nextcloud"])
            .await
            .unwrap();
        // The exact pin verifies.
        assert!(
            s.is_third_party_digest_approved("third-party-x", A_DIGEST)
                .await
                .unwrap()
        );
        // A different (non-approved) digest for the same id does NOT verify.
        assert!(
            !s.is_third_party_digest_approved("third-party-x", B_DIGEST)
                .await
                .unwrap()
        );
        // The same digest under a DIFFERENT plugin id does not verify (identity is the
        // (plugin_id, digest) pair, never the digest alone).
        assert!(
            !s.is_third_party_digest_approved("other-plugin", A_DIGEST)
                .await
                .unwrap()
        );
    }

    #[tokio::test]
    async fn approve_rejects_first_party_collision() {
        let s = store().await;
        // A row whose plugin_id equals a first-party id is refused at approve time
        // (anti-spoof) — case-insensitively, and writing nothing.
        let err = s
            .put_plugin_allowlist(&pin("nextcloud", A_DIGEST), &["nextcloud", "bridge-graph"])
            .await
            .unwrap_err();
        assert!(matches!(err, PluginAllowlistError::FirstPartyCollision(_)));
        let err = s
            .put_plugin_allowlist(
                &pin("Bridge-Graph", A_DIGEST),
                &["nextcloud", "bridge-graph"],
            )
            .await
            .unwrap_err();
        assert!(matches!(err, PluginAllowlistError::FirstPartyCollision(_)));
        // Nothing was written for the colliding id.
        assert!(
            !s.is_third_party_digest_approved("nextcloud", A_DIGEST)
                .await
                .unwrap()
        );
    }

    #[tokio::test]
    async fn approve_rejects_malformed_digest() {
        let s = store().await;
        for bad in ["", "abc", &A_DIGEST.to_uppercase()] {
            let err = s
                .put_plugin_allowlist(&pin("third-party-x", bad), &[])
                .await
                .unwrap_err();
            assert!(matches!(err, PluginAllowlistError::MalformedDigest(_)));
        }
    }

    #[tokio::test]
    async fn revoke_makes_a_pin_stop_verifying() {
        let s = store().await;
        s.put_plugin_allowlist(&pin("third-party-x", A_DIGEST), &[])
            .await
            .unwrap();
        assert!(
            s.is_third_party_digest_approved("third-party-x", A_DIGEST)
                .await
                .unwrap()
        );
        assert!(
            s.revoke_plugin_allowlist("third-party-x", A_DIGEST)
                .await
                .unwrap()
        );
        // Revoked ⇒ no longer verifies (hard refuse on the next load).
        assert!(
            !s.is_third_party_digest_approved("third-party-x", A_DIGEST)
                .await
                .unwrap()
        );
        // Revoking again reports no active row changed.
        assert!(
            !s.revoke_plugin_allowlist("third-party-x", A_DIGEST)
                .await
                .unwrap()
        );
        // The revoked row is still visible for the admin list (history preserved).
        let rows = s.list_plugin_allowlist().await.unwrap();
        assert_eq!(rows.len(), 1);
        assert!(rows[0].revoked);
    }

    #[tokio::test]
    async fn re_approve_after_revoke_un_revokes() {
        let s = store().await;
        s.put_plugin_allowlist(&pin("third-party-x", A_DIGEST), &[])
            .await
            .unwrap();
        s.revoke_plugin_allowlist("third-party-x", A_DIGEST)
            .await
            .unwrap();
        // A deliberate re-approve clears the revoked flag.
        s.put_plugin_allowlist(&pin("third-party-x", A_DIGEST), &[])
            .await
            .unwrap();
        assert!(
            s.is_third_party_digest_approved("third-party-x", A_DIGEST)
                .await
                .unwrap()
        );
    }

    #[tokio::test]
    async fn delete_removes_all_rows_for_a_plugin() {
        let s = store().await;
        s.put_plugin_allowlist(&pin("third-party-x", A_DIGEST), &[])
            .await
            .unwrap();
        s.put_plugin_allowlist(&pin("third-party-x", B_DIGEST), &[])
            .await
            .unwrap();
        s.put_plugin_allowlist(&pin("keep-me", A_DIGEST), &[])
            .await
            .unwrap();
        let removed = s.delete_plugin_allowlist("third-party-x").await.unwrap();
        assert_eq!(removed, 2);
        assert!(
            !s.is_third_party_digest_approved("third-party-x", A_DIGEST)
                .await
                .unwrap()
        );
        // A different plugin's pin survives.
        assert!(
            s.is_third_party_digest_approved("keep-me", A_DIGEST)
                .await
                .unwrap()
        );
    }
}
