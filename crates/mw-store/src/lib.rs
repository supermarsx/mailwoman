#![forbid(unsafe_code)]
//! Local store: opaque sessions + settings, with upstream credentials
//! sealed via XChaCha20-Poly1305 (SPEC §7.3, §9). SQLite via sqlx with
//! runtime queries (no compile-time `DATABASE_URL` needed).

mod cache;
mod redact;
mod seal;
mod v2;

pub use cache::{
    Account, AccountKind, Mailbox, MailboxUpsert, Message, MessageLocation, MessageUpsert,
    NewAccount,
};
pub use redact::Redacted;
pub use seal::{SealError, ServerKey};
pub use v2::{
    ChangeRow, IdentityRow, SavedSearchRow, SnoozeDue, StoredMeta, SubmissionRow, TagRow,
};

use chrono::Utc;
use sqlx::Row;
use sqlx::sqlite::{SqlitePool, SqlitePoolOptions};

#[derive(Debug, thiserror::Error)]
pub enum StoreError {
    #[error("database error: {0}")]
    Db(#[from] sqlx::Error),
    #[error("migration error: {0}")]
    Migrate(#[from] sqlx::migrate::MigrateError),
    #[error("seal error: {0}")]
    Seal(#[from] SealError),
    #[error("session not found")]
    NotFound,
    #[error("corrupt store data: {0}")]
    Corrupt(String),
}

/// Plaintext upstream credentials, only ever held decrypted in memory.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Credentials {
    pub username: String,
    pub password: String,
}

#[derive(Debug, Clone)]
pub struct Session {
    pub id: String,
    pub account_id: String,
    pub username: String,
    pub jmap_url: String,
    pub api_url: String,
    pub credentials: Credentials,
}

#[derive(Clone)]
pub struct Store {
    pool: SqlitePool,
    key: ServerKey,
}

impl Store {
    /// Open a file-backed store (creates the file if missing) and migrate.
    pub async fn open(path: &str, key: ServerKey) -> Result<Self, StoreError> {
        let url = format!("sqlite://{path}?mode=rwc");
        let pool = SqlitePoolOptions::new()
            .max_connections(5)
            .connect(&url)
            .await?;
        Self::init(pool, key).await
    }

    /// Open an in-memory store (tests).
    pub async fn open_in_memory(key: ServerKey) -> Result<Self, StoreError> {
        let pool = SqlitePoolOptions::new()
            .max_connections(1)
            .connect("sqlite::memory:")
            .await?;
        Self::init(pool, key).await
    }

    async fn init(pool: SqlitePool, key: ServerKey) -> Result<Self, StoreError> {
        sqlx::migrate!("./migrations").run(&pool).await?;
        Ok(Self { pool, key })
    }

    pub async fn create_session(
        &self,
        account_id: &str,
        username: &str,
        jmap_url: &str,
        api_url: &str,
        creds: &Credentials,
    ) -> Result<String, StoreError> {
        let id = seal::random_token();
        let sealed = self.key.seal(&encode_creds(creds))?;
        let now = Utc::now().to_rfc3339();
        sqlx::query(
            "INSERT INTO sessions (id, account_id, username, jmap_url, api_url, sealed_creds, created_at, last_seen)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?7)",
        )
        .bind(&id)
        .bind(account_id)
        .bind(username)
        .bind(jmap_url)
        .bind(api_url)
        .bind(sealed)
        .bind(now)
        .execute(&self.pool)
        .await?;
        Ok(id)
    }

    pub async fn get_session(&self, id: &str) -> Result<Session, StoreError> {
        let row = sqlx::query(
            "SELECT id, account_id, username, jmap_url, api_url, sealed_creds FROM sessions WHERE id = ?1",
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?
        .ok_or(StoreError::NotFound)?;

        let sealed: Vec<u8> = row.get("sealed_creds");
        let creds = decode_creds(&self.key.open(&sealed)?)?;
        Ok(Session {
            id: row.get("id"),
            account_id: row.get("account_id"),
            username: row.get("username"),
            jmap_url: row.get("jmap_url"),
            api_url: row.get("api_url"),
            credentials: creds,
        })
    }

    pub async fn touch_session(&self, id: &str) -> Result<(), StoreError> {
        sqlx::query("UPDATE sessions SET last_seen = ?2 WHERE id = ?1")
            .bind(id)
            .bind(Utc::now().to_rfc3339())
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    pub async fn delete_session(&self, id: &str) -> Result<(), StoreError> {
        sqlx::query("DELETE FROM sessions WHERE id = ?1")
            .bind(id)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    pub async fn set_setting(&self, key: &str, value: &str) -> Result<(), StoreError> {
        sqlx::query(
            "INSERT INTO settings (key, value) VALUES (?1, ?2)
             ON CONFLICT(key) DO UPDATE SET value = excluded.value",
        )
        .bind(key)
        .bind(value)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    pub async fn get_setting(&self, key: &str) -> Result<Option<String>, StoreError> {
        let row = sqlx::query("SELECT value FROM settings WHERE key = ?1")
            .bind(key)
            .fetch_optional(&self.pool)
            .await?;
        Ok(row.map(|r| r.get("value")))
    }
}

fn encode_creds(c: &Credentials) -> Vec<u8> {
    serde_json::to_vec(&(&c.username, &c.password)).expect("credential encode")
}

fn decode_creds(bytes: &[u8]) -> Result<Credentials, StoreError> {
    let (username, password): (String, String) =
        serde_json::from_slice(bytes).map_err(|_| StoreError::NotFound)?;
    Ok(Credentials { username, password })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn creds() -> Credentials {
        Credentials {
            username: "test@example.org".into(),
            password: "s3cr3t".into(),
        }
    }

    #[tokio::test]
    async fn session_crud_round_trip() {
        let store = Store::open_in_memory(ServerKey::generate()).await.unwrap();
        let id = store
            .create_session(
                "a1",
                "test@example.org",
                "http://mock/jmap",
                "http://mock/jmap",
                &creds(),
            )
            .await
            .unwrap();

        let got = store.get_session(&id).await.unwrap();
        assert_eq!(got.account_id, "a1");
        assert_eq!(got.credentials, creds());

        store.touch_session(&id).await.unwrap();
        store.delete_session(&id).await.unwrap();
        assert!(matches!(
            store.get_session(&id).await,
            Err(StoreError::NotFound)
        ));
    }

    #[tokio::test]
    async fn credentials_are_encrypted_at_rest() {
        let store = Store::open_in_memory(ServerKey::generate()).await.unwrap();
        let id = store
            .create_session("a1", "u", "http://mock", "http://mock", &creds())
            .await
            .unwrap();
        // Read the raw blob and ensure the password is not present in plaintext.
        let row = sqlx::query("SELECT sealed_creds FROM sessions WHERE id = ?1")
            .bind(&id)
            .fetch_one(&store.pool)
            .await
            .unwrap();
        let sealed: Vec<u8> = row.get("sealed_creds");
        assert!(!sealed.windows(6).any(|w| w == b"s3cr3t"));
    }

    #[tokio::test]
    async fn wrong_key_cannot_open() {
        let store = Store::open_in_memory(ServerKey::generate()).await.unwrap();
        let id = store
            .create_session("a1", "u", "http://mock", "http://mock", &creds())
            .await
            .unwrap();
        // Swap in a different key and confirm open fails.
        let other = Store {
            pool: store.pool.clone(),
            key: ServerKey::generate(),
        };
        assert!(other.get_session(&id).await.is_err());
    }

    #[tokio::test]
    async fn settings_get_set() {
        let store = Store::open_in_memory(ServerKey::generate()).await.unwrap();
        assert_eq!(store.get_setting("theme").await.unwrap(), None);
        store.set_setting("theme", "grove-dark").await.unwrap();
        store.set_setting("theme", "grove-light").await.unwrap();
        assert_eq!(
            store.get_setting("theme").await.unwrap().as_deref(),
            Some("grove-light")
        );
    }
}
