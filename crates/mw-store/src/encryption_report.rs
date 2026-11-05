//! Encryption observations for the Postgres backend, and the column-level sealing
//! coverage report.
//!
//! The application cannot encrypt a Postgres server's disk and does not claim to.
//! This module reports what can be queried from a connection: whether this
//! connection uses TLS, whether the server has TLS on, `data_checksums`, `pgcrypto`,
//! any transparent-data-encryption setting or extension, and the tablespaces. Disk or
//! volume encryption is the operator's responsibility (see docs/admin/encryption.md).

use crate::backend::Backend;
use crate::{Store, StoreError, q};
use serde_json::{Value, json};

/// `sslmode` of a Postgres DSN (`PGSSLMODE` when the DSN has none; libpq default `prefer`).
pub fn dsn_ssl_mode(dsn: &str) -> String {
    let from_dsn = dsn.split_once('?').and_then(|(_, query)| {
        query.split('&').find_map(|kv| {
            let (k, v) = kv.split_once('=')?;
            (k == "sslmode").then(|| v.to_ascii_lowercase())
        })
    });
    from_dsn
        .or_else(|| std::env::var("PGSSLMODE").ok().map(|v| v.to_ascii_lowercase()))
        .unwrap_or_else(|| "prefer".into())
}

/// Whether a DSN's `sslmode` forces TLS.
pub fn ssl_mode_requires_tls(mode: &str) -> bool {
    matches!(mode, "require" | "verify-ca" | "verify-full")
}

/// Refuse a Postgres DSN that does not force TLS when `MW_DB_REQUIRE_TLS` is set.
pub fn enforce_tls_policy(dsn: &str) -> Result<(), StoreError> {
    let on = std::env::var("MW_DB_REQUIRE_TLS")
        .map(|v| matches!(v.trim().to_ascii_lowercase().as_str(), "1" | "true" | "yes" | "on"))
        .unwrap_or(false);
    if on && !ssl_mode_requires_tls(&dsn_ssl_mode(dsn)) {
        return Err(StoreError::Encryption(
            crate::encryption::EncryptionError::Config(
                "MW_DB_REQUIRE_TLS is set but the Postgres DSN does not force TLS; add sslmode=require (or verify-ca / verify-full)".into(),
            ),
        ));
    }
    Ok(())
}

/// One sensitive column and whether the application seals it before it reaches the database.
#[derive(Debug, Clone, Copy, serde::Serialize)]
pub struct ColumnCoverage {
    pub table: &'static str,
    pub column: &'static str,
    pub what: &'static str,
    /// `true`: XChaCha20-Poly1305 under the server key before the row is written.
    pub sealed: bool,
}

const fn c(table: &'static str, column: &'static str, what: &'static str, sealed: bool) -> ColumnCoverage {
    ColumnCoverage { table, column, what, sealed }
}

/// Sensitive columns, from a search of the `seal(` call sites and the table definitions.
/// A column absent from this list is not a claim that it is plaintext or sealed.
pub const SEALED_COLUMN_COVERAGE: &[ColumnCoverage] = &[
    c("sessions", "sealed_creds", "mailbox credentials of a session", true),
    c("accounts", "sealed_creds", "stored mailbox credentials", true),
    c("bodies", "sealed_bytes", "cached message bodies", true),
    c("mailboxes", "sealed_env", "cached message envelopes", true),
    c("notes", "title_sealed, body_*_sealed, tags_json_sealed", "note content and metadata", true),
    c("message_embeddings", "vector_sealed", "search embeddings", true),
    c("totp_secrets", "sealed_secret", "mailbox second-factor secrets", true),
    c("admin_totp", "sealed_secret", "admin second-factor secrets", true),
    c("sso_config", "secret_sealed", "identity provider client secret", true),
    c("ssh_tunnel", "sealed_private_key", "SSH tunnel private keys", true),
    c("push_config", "vapid_private_sealed", "push signing key", true),
    c("webhooks", "secret_sealed", "webhook signing secrets", true),
    c("inbound_webhooks", "secret_sealed", "inbound webhook secrets", true),
    c("plugin_kv", "sealed_value", "plugin key-value data", true),
    c("hosting_connection", "sealed_credential", "hosting provider credentials", true),
    c("directory_config", "sealed_bind_password", "directory bind password", true),
    c("egress_proxy", "sealed_refresh_token", "egress proxy tokens", true),
    c("bridge_oauth_tokens", "sealed_access, sealed_refresh", "bridge OAuth tokens", true),
    c("dav_account", "sealed_password", "DAV account passwords", true),
    c("open_tracking", "subject_sealed, recipients_sealed", "tracked-message subject and recipients", true),
    c("settings", "value", "configuration values (secrets are sealed by their own tables)", false),
    c("audit_log", "detail", "administrator action details", false),
    c("auth_logs", "ip, username", "sign-in attempts with source address", false),
    c("sessions", "username, ip", "who is signed in and from where", false),
    c("contacts / events", "all columns", "address book and calendar entries", false),
    c("data_requests", "subject, notes", "privacy request records", false),
];

impl Store {
    /// What a Postgres connection reveals about encryption. Errors on SQLite.
    pub async fn postgres_encryption_report(&self) -> Result<Value, StoreError> {
        if !matches!(&self.backend, Backend::Postgres(_)) {
            return Err(StoreError::Invalid("not a Postgres store"));
        }
        let b = &self.backend;
        let conn = q("SELECT CAST(CASE WHEN ssl THEN 1 ELSE 0 END AS BIGINT), COALESCE(version, ''), COALESCE(cipher, '') FROM pg_stat_ssl WHERE pid = pg_backend_pid()")
            .fetch_optional(b)
            .await
            .ok()
            .flatten();
        let (conn_ssl, tls_version, tls_cipher) = match &conn {
            Some(r) => (
                Some(r.get_i64_idx(0) != 0),
                Some(r.get_string_idx(1)),
                Some(r.get_string_idx(2)),
            ),
            None => (None, None, None),
        };
        let setting = |sql: &'static str| async move {
            q(sql)
                .fetch_one(b)
                .await
                .ok()
                .map(|r| r.get_string_idx(0))
        };
        let server_ssl = setting("SHOW ssl").await;
        let checksums = setting("SHOW data_checksums").await;
        let data_directory = setting("SHOW data_directory").await;
        let version = setting("SHOW server_version").await;
        let pgcrypto_installed = q("SELECT count(*) FROM pg_extension WHERE extname = 'pgcrypto'")
            .fetch_one(b)
            .await
            .map(|r| r.get_i64_idx(0) > 0)
            .ok();
        let pgcrypto_available = q("SELECT count(*) FROM pg_available_extensions WHERE name = 'pgcrypto'")
            .fetch_one(b)
            .await
            .map(|r| r.get_i64_idx(0) > 0)
            .ok();
        let tde_settings: Vec<String> = q("SELECT name FROM pg_settings WHERE name ILIKE '%tde%' OR name ILIKE '%encrypt%' ORDER BY name LIMIT 20")
            .fetch_all(b)
            .await
            .map(|rows| rows.iter().map(|r| r.get_string_idx(0)).collect())
            .unwrap_or_default();
        let tde_extensions: Vec<String> = q("SELECT CAST(extname AS TEXT) FROM pg_extension WHERE extname ILIKE '%tde%' OR extname ILIKE '%encrypt%' ORDER BY extname")
            .fetch_all(b)
            .await
            .map(|rows| rows.iter().map(|r| r.get_string_idx(0)).collect())
            .unwrap_or_default();
        let tablespaces: Vec<Value> = q("SELECT CAST(spcname AS TEXT), COALESCE(pg_tablespace_location(oid), '') FROM pg_tablespace ORDER BY spcname")
            .fetch_all(b)
            .await
            .map(|rows| {
                rows.iter()
                    .map(|r| json!({"name": r.get_string_idx(0), "location": r.get_string_idx(1)}))
                    .collect()
            })
            .unwrap_or_default();
        Ok(json!({
            "serverVersion": version,
            "connectionSsl": conn_ssl,
            "tlsVersion": tls_version,
            "tlsCipher": tls_cipher,
            "serverSsl": server_ssl,
            "dataChecksums": checksums,
            "dataDirectory": data_directory,
            "pgcryptoInstalled": pgcrypto_installed,
            "pgcryptoAvailable": pgcrypto_available,
            "tdeIndicators": { "settings": tde_settings, "extensions": tde_extensions },
            "tablespaces": tablespaces,
            "columnCoverage": SEALED_COLUMN_COVERAGE,
            "note": "Disk and volume encryption of the Postgres server cannot be observed from a connection and is the operator's responsibility. data_checksums is corruption detection, not encryption.",
        }))
    }

    /// The sealed / plaintext column list (both backends).
    pub fn sealed_column_coverage(&self) -> &'static [ColumnCoverage] {
        SEALED_COLUMN_COVERAGE
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ssl_mode_is_read_from_the_dsn() {
        assert_eq!(dsn_ssl_mode("postgres://u@h/db?sslmode=Require"), "require");
        assert_eq!(dsn_ssl_mode("postgres://u@h/db?application_name=x&sslmode=verify-full"), "verify-full");
        assert!(ssl_mode_requires_tls("verify-ca"));
        assert!(!ssl_mode_requires_tls("prefer"));
        assert!(!ssl_mode_requires_tls("disable"));
    }
}
