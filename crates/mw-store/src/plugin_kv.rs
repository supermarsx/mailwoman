//! 0013 persistent plugin key/value storage (26.15 t15).
//!
//! Replaces the former non-persistent `HostKv` stub behind the `store:kv-scoped`
//! capability with a real, sealed, quota-bounded, per-(plugin, account)-isolated KV
//! over the 0013 `plugin_kv` table.
//!
//! INVARIANTS (PQ1/PQ2/PQ6):
//!   * **Sealed at rest.** Each value's opaque bytes are sealed (XChaCha20-Poly1305
//!     under the store [`ServerKey`], the same zero-access posture as bodies/creds)
//!     before they hit the DB — `plugin_kv.sealed_value` is never plaintext. `key` is
//!     a non-secret lookup key.
//!   * **Isolation.** Every method is scoped by `(plugin_id, account_id)`. Those two
//!     coordinates are derived HOST-side by `mw-plugin` from the bound plugin instance
//!     (never from guest/wasm arguments) and passed in here, so one plugin can never
//!     read another plugin's — or another account's — keys. A deployment-wide plugin
//!     uses `account_id = ""` (the `plugin_grants` convention).
//!   * **Quotas.** [`Store::plugin_kv_set`] enforces the per-value and per-namespace
//!     ceilings in [`PluginKvLimits`] at put time; an over-quota / oversize put returns
//!     a distinct [`PluginKvError`] (never a silent drop). Limits are
//!     deployment-configurable (constructed by `mw-server` from env, defaulting here).
//!   * **No TTL.** Plugin KV is intentional state: keys persist until the plugin
//!     deletes them, or the whole namespace is purged on uninstall via
//!     [`Store::plugin_kv_purge`].

use chrono::Utc;

use crate::backend::q;
use crate::{Store, StoreError};

/// Per-(plugin, account) quota ceilings enforced at [`Store::plugin_kv_set`] (PQ1).
/// Deployment-configurable: `mw-server` builds these (optionally from env) and passes
/// them to each put; the defaults here are the advertised ceilings.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PluginKvLimits {
    /// Max key length in bytes (default 256).
    pub max_key_bytes: usize,
    /// Max single-value length in bytes (default 64 KiB).
    pub max_value_bytes: usize,
    /// Max total plaintext value bytes across one (plugin, account) namespace
    /// (default 5 MiB).
    pub max_total_bytes: i64,
    /// Max number of keys in one (plugin, account) namespace (default 1000).
    pub max_keys: i64,
}

impl Default for PluginKvLimits {
    fn default() -> Self {
        Self {
            max_key_bytes: 256,
            max_value_bytes: 64 * 1024,
            max_total_bytes: 5 * 1024 * 1024,
            max_keys: 1000,
        }
    }
}

/// A visible failure from a quota-bounded [`Store::plugin_kv_set`] — the guest's put
/// fails LOUDLY on any of these rather than silently appearing to succeed (PQ1).
#[derive(Debug, thiserror::Error)]
pub enum PluginKvError {
    /// The key exceeds [`PluginKvLimits::max_key_bytes`].
    #[error("plugin kv key exceeds the {limit}-byte limit ({actual} bytes)")]
    KeyTooLarge { limit: usize, actual: usize },
    /// The value exceeds [`PluginKvLimits::max_value_bytes`].
    #[error("plugin kv value exceeds the {limit}-byte limit ({actual} bytes)")]
    ValueTooLarge { limit: usize, actual: usize },
    /// Storing a new key would exceed [`PluginKvLimits::max_keys`] for the namespace.
    #[error("plugin kv namespace is at its {limit}-key limit")]
    TooManyKeys { limit: i64 },
    /// The put would push the namespace over [`PluginKvLimits::max_total_bytes`].
    #[error("plugin kv namespace would exceed its {limit}-byte quota (would be {would_be} bytes)")]
    QuotaExceeded { limit: i64, would_be: i64 },
    /// An underlying store/seal error.
    #[error(transparent)]
    Store(#[from] StoreError),
}

impl Store {
    /// Read one plugin KV value, unsealed. `None` if the `(plugin_id, account_id, key)`
    /// triple has no row. Isolation is total: a different `plugin_id` or `account_id`
    /// simply does not match, so a plugin can never read outside its own namespace.
    pub async fn plugin_kv_get(
        &self,
        plugin_id: &str,
        account_id: &str,
        key: &str,
    ) -> Result<Option<Vec<u8>>, StoreError> {
        let row = q("SELECT sealed_value FROM plugin_kv
                       WHERE plugin_id = ?1 AND account_id = ?2 AND key = ?3")
        .bind(plugin_id)
        .bind(account_id)
        .bind(key)
        .fetch_optional(&self.backend)
        .await?;
        match row {
            Some(r) => Ok(Some(self.key.open(&r.get_blob("sealed_value"))?)),
            None => Ok(None),
        }
    }

    /// Seal `value` and upsert it under `(plugin_id, account_id, key)`, enforcing the
    /// [`PluginKvLimits`] quotas FIRST. An oversize key/value, or a put that would push
    /// the namespace past its total-bytes or key-count ceiling, returns a distinct
    /// [`PluginKvError`] and writes nothing (PQ1 — visible, never silent).
    pub async fn plugin_kv_set(
        &self,
        plugin_id: &str,
        account_id: &str,
        key: &str,
        value: &[u8],
        limits: &PluginKvLimits,
    ) -> Result<(), PluginKvError> {
        // Cheap per-item bounds first (fail fast, no DB round-trip).
        if key.len() > limits.max_key_bytes {
            return Err(PluginKvError::KeyTooLarge {
                limit: limits.max_key_bytes,
                actual: key.len(),
            });
        }
        if value.len() > limits.max_value_bytes {
            return Err(PluginKvError::ValueTooLarge {
                limit: limits.max_value_bytes,
                actual: value.len(),
            });
        }

        // Namespace accounting: fetch every (key, size) once (bounded by max_keys) and
        // sum in Rust — dialect-safe (SQLite SUM vs Postgres SUM→numeric would diverge)
        // and it yields the current total, the count, and this key's prior size together.
        let rows = q("SELECT key, size FROM plugin_kv WHERE plugin_id = ?1 AND account_id = ?2")
            .bind(plugin_id)
            .bind(account_id)
            .fetch_all(&self.backend)
            .await
            .map_err(StoreError::from)?;
        let mut total: i64 = 0;
        let mut existed = false;
        let mut old_size: i64 = 0;
        for r in &rows {
            let sz = r.get_i64("size");
            total += sz;
            if r.get_string("key") == key {
                existed = true;
                old_size = sz;
            }
        }

        let value_len = value.len() as i64;
        // A new key adds one to the count; an upsert of an existing key does not.
        if !existed && (rows.len() as i64) + 1 > limits.max_keys {
            return Err(PluginKvError::TooManyKeys {
                limit: limits.max_keys,
            });
        }
        let new_total = total - old_size + value_len;
        if new_total > limits.max_total_bytes {
            return Err(PluginKvError::QuotaExceeded {
                limit: limits.max_total_bytes,
                would_be: new_total,
            });
        }

        let sealed = self
            .key
            .seal(value)
            .map_err(|e| PluginKvError::Store(e.into()))?;
        let now = Utc::now().to_rfc3339();
        q(
            "INSERT INTO plugin_kv (plugin_id, account_id, key, sealed_value, size, updated_at)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6)
             ON CONFLICT(plugin_id, account_id, key) DO UPDATE SET
                 sealed_value = excluded.sealed_value,
                 size = excluded.size,
                 updated_at = excluded.updated_at",
        )
        .bind(plugin_id)
        .bind(account_id)
        .bind(key)
        .bind(sealed)
        .bind(value_len)
        .bind(&now)
        .execute(&self.backend)
        .await
        .map_err(StoreError::from)?;
        Ok(())
    }

    /// Delete one key from a namespace. A no-op (not an error) if it is absent.
    pub async fn plugin_kv_delete(
        &self,
        plugin_id: &str,
        account_id: &str,
        key: &str,
    ) -> Result<(), StoreError> {
        q("DELETE FROM plugin_kv WHERE plugin_id = ?1 AND account_id = ?2 AND key = ?3")
            .bind(plugin_id)
            .bind(account_id)
            .bind(key)
            .execute(&self.backend)
            .await?;
        Ok(())
    }

    /// List the keys in one `(plugin_id, account_id)` namespace, ascending. Keys only —
    /// values stay sealed and are fetched individually via [`Store::plugin_kv_get`].
    pub async fn plugin_kv_list(
        &self,
        plugin_id: &str,
        account_id: &str,
    ) -> Result<Vec<String>, StoreError> {
        let rows = q("SELECT key FROM plugin_kv
                        WHERE plugin_id = ?1 AND account_id = ?2 ORDER BY key ASC")
        .bind(plugin_id)
        .bind(account_id)
        .fetch_all(&self.backend)
        .await?;
        Ok(rows.iter().map(|r| r.get_string("key")).collect())
    }

    /// Purge a plugin's ENTIRE KV state across every account (PQ6, uninstall/removal).
    /// Returns the number of rows removed. There is no TTL, so this is the only
    /// whole-namespace reclamation path besides the plugin deleting its own keys.
    pub async fn plugin_kv_purge(&self, plugin_id: &str) -> Result<u64, StoreError> {
        let n = q("DELETE FROM plugin_kv WHERE plugin_id = ?1")
            .bind(plugin_id)
            .execute(&self.backend)
            .await?;
        Ok(n)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ServerKey;

    async fn store() -> Store {
        Store::open_in_memory(ServerKey::generate()).await.unwrap()
    }

    /// Tiny ceilings so quota rejections are cheap to provoke.
    fn tiny() -> PluginKvLimits {
        PluginKvLimits {
            max_key_bytes: 256,
            max_value_bytes: 50,
            max_total_bytes: 100,
            max_keys: 3,
        }
    }

    #[tokio::test]
    async fn set_get_delete_list_round_trip() {
        let s = store().await;
        let lim = PluginKvLimits::default();
        s.plugin_kv_set("p", "acct", "k1", b"one", &lim)
            .await
            .unwrap();
        s.plugin_kv_set("p", "acct", "k2", b"two", &lim)
            .await
            .unwrap();

        assert_eq!(
            s.plugin_kv_get("p", "acct", "k1").await.unwrap().as_deref(),
            Some(&b"one"[..])
        );
        assert_eq!(
            s.plugin_kv_list("p", "acct").await.unwrap(),
            vec!["k1".to_string(), "k2".to_string()]
        );

        // Upsert replaces in place (no duplicate key, updated value).
        s.plugin_kv_set("p", "acct", "k1", b"ONE!", &lim)
            .await
            .unwrap();
        assert_eq!(
            s.plugin_kv_get("p", "acct", "k1").await.unwrap().as_deref(),
            Some(&b"ONE!"[..])
        );
        assert_eq!(s.plugin_kv_list("p", "acct").await.unwrap().len(), 2);

        s.plugin_kv_delete("p", "acct", "k1").await.unwrap();
        assert!(s.plugin_kv_get("p", "acct", "k1").await.unwrap().is_none());
        // Deleting an absent key is a no-op.
        s.plugin_kv_delete("p", "acct", "k1").await.unwrap();
        assert_eq!(
            s.plugin_kv_list("p", "acct").await.unwrap(),
            vec!["k2".to_string()]
        );
    }

    #[tokio::test]
    async fn value_is_sealed_at_rest_not_plaintext() {
        let s = store().await;
        let marker = b"top-secret-plugin-value-marker";
        s.plugin_kv_set("p", "acct", "k", marker, &PluginKvLimits::default())
            .await
            .unwrap();
        // The raw column must not carry the plaintext marker.
        let row = q("SELECT sealed_value FROM plugin_kv WHERE plugin_id = ?1")
            .bind("p")
            .fetch_one(s.backend())
            .await
            .unwrap();
        let raw = row.get_blob("sealed_value");
        assert!(
            !raw.windows(marker.len()).any(|w| w == marker),
            "value must be sealed, never plaintext at rest"
        );
        // And it still round-trips through the key.
        assert_eq!(
            s.plugin_kv_get("p", "acct", "k").await.unwrap().as_deref(),
            Some(&marker[..])
        );
    }

    #[tokio::test]
    async fn isolation_across_plugins_and_accounts() {
        let s = store().await;
        let lim = PluginKvLimits::default();
        s.plugin_kv_set("plugin-a", "acct-1", "k", b"a1", &lim)
            .await
            .unwrap();

        // A different plugin cannot see plugin-a's key.
        assert!(
            s.plugin_kv_get("plugin-b", "acct-1", "k")
                .await
                .unwrap()
                .is_none()
        );
        // Same plugin, a different account, cannot see it either.
        assert!(
            s.plugin_kv_get("plugin-a", "acct-2", "k")
                .await
                .unwrap()
                .is_none()
        );
        // list is namespaced too.
        assert!(
            s.plugin_kv_list("plugin-b", "acct-1")
                .await
                .unwrap()
                .is_empty()
        );
        assert_eq!(
            s.plugin_kv_get("plugin-a", "acct-1", "k")
                .await
                .unwrap()
                .as_deref(),
            Some(&b"a1"[..])
        );
    }

    #[tokio::test]
    async fn deployment_wide_uses_empty_account() {
        let s = store().await;
        s.plugin_kv_set("p", "", "k", b"wide", &PluginKvLimits::default())
            .await
            .unwrap();
        assert_eq!(
            s.plugin_kv_get("p", "", "k").await.unwrap().as_deref(),
            Some(&b"wide"[..])
        );
        // An account-scoped read does not see the deployment-wide key.
        assert!(s.plugin_kv_get("p", "acct", "k").await.unwrap().is_none());
    }

    #[tokio::test]
    async fn oversize_value_is_rejected() {
        let s = store().await;
        let err = s
            .plugin_kv_set("p", "acct", "k", &[0u8; 51], &tiny())
            .await
            .unwrap_err();
        assert!(matches!(err, PluginKvError::ValueTooLarge { .. }));
        // Nothing was written.
        assert!(s.plugin_kv_get("p", "acct", "k").await.unwrap().is_none());
    }

    #[tokio::test]
    async fn oversize_key_is_rejected() {
        let s = store().await;
        let big_key = "x".repeat(257);
        let err = s
            .plugin_kv_set("p", "acct", &big_key, b"v", &PluginKvLimits::default())
            .await
            .unwrap_err();
        assert!(matches!(err, PluginKvError::KeyTooLarge { .. }));
    }

    #[tokio::test]
    async fn key_count_quota_is_enforced() {
        let s = store().await;
        let lim = tiny(); // max_keys = 3
        for i in 0..3 {
            s.plugin_kv_set("p", "acct", &format!("k{i}"), b"x", &lim)
                .await
                .unwrap();
        }
        let err = s
            .plugin_kv_set("p", "acct", "k3", b"x", &lim)
            .await
            .unwrap_err();
        assert!(matches!(err, PluginKvError::TooManyKeys { .. }));
        // Upserting an EXISTING key at the limit is still allowed (count unchanged).
        s.plugin_kv_set("p", "acct", "k0", b"y", &lim)
            .await
            .unwrap();
    }

    #[tokio::test]
    async fn total_bytes_quota_is_enforced() {
        let s = store().await;
        let lim = tiny(); // max_total_bytes = 100, max_value_bytes = 50
        s.plugin_kv_set("p", "acct", "a", &[0u8; 50], &lim)
            .await
            .unwrap();
        s.plugin_kv_set("p", "acct", "b", &[0u8; 50], &lim)
            .await
            .unwrap();
        // Total is now 100; another 1 byte would exceed the 100-byte quota.
        let err = s
            .plugin_kv_set("p", "acct", "c", b"x", &lim)
            .await
            .unwrap_err();
        assert!(matches!(err, PluginKvError::QuotaExceeded { .. }));
        // Shrinking an existing key frees room (replace 50 with 1 → total 51).
        s.plugin_kv_set("p", "acct", "a", b"x", &lim).await.unwrap();
        s.plugin_kv_set("p", "acct", "c", b"x", &lim).await.unwrap();
    }

    #[tokio::test]
    async fn purge_clears_whole_namespace_across_accounts() {
        let s = store().await;
        let lim = PluginKvLimits::default();
        s.plugin_kv_set("p", "acct-1", "k", b"1", &lim)
            .await
            .unwrap();
        s.plugin_kv_set("p", "acct-2", "k", b"2", &lim)
            .await
            .unwrap();
        s.plugin_kv_set("p", "", "k", b"w", &lim).await.unwrap();
        // A different plugin's data must survive the purge.
        s.plugin_kv_set("other", "acct-1", "k", b"keep", &lim)
            .await
            .unwrap();

        let removed = s.plugin_kv_purge("p").await.unwrap();
        assert_eq!(removed, 3);
        assert!(s.plugin_kv_get("p", "acct-1", "k").await.unwrap().is_none());
        assert!(s.plugin_kv_get("p", "acct-2", "k").await.unwrap().is_none());
        assert!(s.plugin_kv_get("p", "", "k").await.unwrap().is_none());
        assert_eq!(
            s.plugin_kv_get("other", "acct-1", "k")
                .await
                .unwrap()
                .as_deref(),
            Some(&b"keep"[..])
        );
    }
}
