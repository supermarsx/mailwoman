//! t39 N8: AI provider profiles and the egress grants for local endpoints
//! (migration 0073, both dialects).
//!
//! A profile's API key is sealed with the server key, bound by AAD to the profile
//! id, its dialect and its endpoint ([`KEY_PURPOSE`]): a sealed key copied onto
//! another row, or left on a row whose endpoint was changed, does not open. The
//! key is never read back by [`Store::list_assist_profiles`] or
//! [`Store::get_assist_profile`]; only [`Store::assist_profile_key`] opens it, for
//! the request that uses it.
//!
//! Which profile applies to whom is not stored here: it is the settings registry
//! key `assist.profile` (deployment, domain and per-user values with locks).

use crate::backend::q;
use crate::{Store, StoreError};

/// The seal purpose of a profile's API key.
const KEY_PURPOSE: &str = "assist-profile-key-v1";

/// One `assist_profile` row, without its key.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AssistProfileRow {
    pub id: String,
    pub name: String,
    /// The preset it was made from (`mw_assist::profiles::PRESETS`).
    pub preset: String,
    /// `openai` | `azure-openai` | `anthropic`.
    pub dialect: String,
    pub endpoint: String,
    pub model: String,
    /// `mw_assist::profiles::ProfileParams` as JSON.
    pub params_json: String,
    /// Whether a sealed key is stored.
    pub has_key: bool,
    pub created_by: String,
    pub created_at: i64,
    pub updated_by: String,
    pub updated_at: i64,
}

/// What a write does with the stored key.
#[derive(Clone, PartialEq, Eq)]
pub enum ProfileKey {
    /// Keep the stored key. Only valid while dialect and endpoint are unchanged
    /// (the AAD binds them); [`Store::put_assist_profile`] refuses otherwise.
    Keep,
    /// Seal and store this key.
    Set(String),
    /// Remove the stored key.
    Clear,
}

impl std::fmt::Debug for ProfileKey {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ProfileKey::Keep => f.write_str("Keep"),
            ProfileKey::Set(_) => f.write_str("Set(<redacted>)"),
            ProfileKey::Clear => f.write_str("Clear"),
        }
    }
}

/// One `assist_egress_grant` row: a `host:port` a profile may reach although it
/// resolves to a loopback or private address.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AssistEgressGrantRow {
    pub id: String,
    pub destination: String,
    pub note: String,
    pub created_by: String,
    pub created_at: i64,
}

/// Every profile read selects these columns (the key itself never).
macro_rules! profile_select {
    ($tail:literal) => {
        concat!(
            "SELECT id, name, preset, dialect, endpoint, model, params, ",
            "CAST(CASE WHEN api_key_sealed IS NULL THEN 0 ELSE 1 END AS BIGINT) AS has_key, ",
            "created_by, created_at, updated_by, updated_at FROM assist_profile ",
            $tail
        )
    };
}

fn profile_row(r: &crate::backend::Row) -> AssistProfileRow {
    AssistProfileRow {
        id: r.get_string("id"),
        name: r.get_string("name"),
        preset: r.get_string("preset"),
        dialect: r.get_string("dialect"),
        endpoint: r.get_string("endpoint"),
        model: r.get_string("model"),
        params_json: r.get_string("params"),
        has_key: r.get_i64("has_key") != 0,
        created_by: r.get_string("created_by"),
        created_at: r.get_i64("created_at"),
        updated_by: r.get_string("updated_by"),
        updated_at: r.get_i64("updated_at"),
    }
}

/// The error a [`ProfileKey::Keep`] write gets when the endpoint moved.
pub const KEY_ENDPOINT_MOVED: &str = "the stored key is kept only while the dialect and \
     endpoint stay the same; send the key for the new endpoint, or clear it";

impl Store {
    /// Every profile, by name.
    pub async fn list_assist_profiles(&self) -> Result<Vec<AssistProfileRow>, StoreError> {
        let rows = q(profile_select!("ORDER BY name, id"))
            .fetch_all(&self.backend)
            .await?;
        Ok(rows.iter().map(profile_row).collect())
    }

    /// One profile, without its key.
    pub async fn get_assist_profile(
        &self,
        id: &str,
    ) -> Result<Option<AssistProfileRow>, StoreError> {
        let row = q(profile_select!("WHERE id = ?1"))
            .bind(id)
            .fetch_optional(&self.backend)
            .await?;
        Ok(row.as_ref().map(profile_row))
    }

    /// The profile's API key, opened (`""` when none is stored). A sealed key that
    /// does not open under the row's id, dialect and endpoint is
    /// [`StoreError::Seal`].
    pub async fn assist_profile_key(&self, id: &str) -> Result<Option<String>, StoreError> {
        let Some(row) =
            q("SELECT dialect, endpoint, api_key_sealed FROM assist_profile WHERE id = ?1")
                .bind(id)
                .fetch_optional(&self.backend)
                .await?
        else {
            return Ok(None);
        };
        let Some(sealed) = row.get_opt_blob("api_key_sealed") else {
            return Ok(Some(String::new()));
        };
        let dialect = row.get_string("dialect");
        let endpoint = row.get_string("endpoint");
        let plain = self
            .key
            .open_bound(&sealed, KEY_PURPOSE, &[id, &dialect, &endpoint])?;
        Ok(Some(String::from_utf8_lossy(&plain).into_owned()))
    }

    /// Insert or replace a profile. `created_*` are kept on an update. With
    /// [`ProfileKey::Keep`] on an existing row whose dialect or endpoint differs,
    /// nothing is written and the answer is `Err(KEY_ENDPOINT_MOVED)` inside
    /// `Ok`, so the caller can answer `400`.
    pub async fn put_assist_profile(
        &self,
        row: &AssistProfileRow,
        key: &ProfileKey,
    ) -> Result<Result<(), &'static str>, StoreError> {
        let existing = q("SELECT dialect, endpoint, \
                          CAST(CASE WHEN api_key_sealed IS NULL THEN 0 ELSE 1 END AS BIGINT) AS has_key \
                          FROM assist_profile WHERE id = ?1")
        .bind(&row.id)
        .fetch_optional(&self.backend)
        .await?;
        let sealed: Option<Vec<u8>> = match key {
            ProfileKey::Set(k) if !k.is_empty() => Some(self.key.seal_bound(
                k.as_bytes(),
                KEY_PURPOSE,
                &[&row.id, &row.dialect, &row.endpoint],
            )?),
            ProfileKey::Set(_) | ProfileKey::Clear => None,
            ProfileKey::Keep => {
                if let Some(e) = &existing
                    && e.get_i64("has_key") != 0
                    && (e.get_string("dialect") != row.dialect
                        || e.get_string("endpoint") != row.endpoint)
                {
                    return Ok(Err(KEY_ENDPOINT_MOVED));
                }
                None
            }
        };
        let keep = matches!(key, ProfileKey::Keep);
        if existing.is_some() {
            let sql = if keep {
                "UPDATE assist_profile SET name=?2, preset=?3, dialect=?4, endpoint=?5, \
                 model=?6, params=?7, updated_by=?8, updated_at=?9 WHERE id=?1"
            } else {
                "UPDATE assist_profile SET name=?2, preset=?3, dialect=?4, endpoint=?5, \
                 model=?6, params=?7, updated_by=?8, updated_at=?9, api_key_sealed=?10 \
                 WHERE id=?1"
            };
            let mut stmt = q(sql)
                .bind(&row.id)
                .bind(&row.name)
                .bind(&row.preset)
                .bind(&row.dialect)
                .bind(&row.endpoint)
                .bind(&row.model)
                .bind(&row.params_json)
                .bind(&row.updated_by)
                .bind(row.updated_at);
            if !keep {
                stmt = stmt.bind(sealed);
            }
            stmt.execute(&self.backend).await?;
        } else {
            q(
                "INSERT INTO assist_profile (id, name, preset, dialect, endpoint, model, params, \
               api_key_sealed, created_by, created_at, updated_by, updated_at) \
               VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12)",
            )
            .bind(&row.id)
            .bind(&row.name)
            .bind(&row.preset)
            .bind(&row.dialect)
            .bind(&row.endpoint)
            .bind(&row.model)
            .bind(&row.params_json)
            .bind(sealed)
            .bind(&row.created_by)
            .bind(row.created_at)
            .bind(&row.updated_by)
            .bind(row.updated_at)
            .execute(&self.backend)
            .await?;
        }
        Ok(Ok(()))
    }

    /// Delete a profile. Returns whether a row was removed.
    pub async fn delete_assist_profile(&self, id: &str) -> Result<bool, StoreError> {
        Ok(q("DELETE FROM assist_profile WHERE id = ?1")
            .bind(id)
            .execute(&self.backend)
            .await?
            > 0)
    }

    /// Every egress grant, by destination.
    pub async fn list_assist_egress_grants(&self) -> Result<Vec<AssistEgressGrantRow>, StoreError> {
        let rows = q("SELECT id, destination, note, created_by, created_at \
                      FROM assist_egress_grant ORDER BY destination")
        .fetch_all(&self.backend)
        .await?;
        Ok(rows
            .iter()
            .map(|r| AssistEgressGrantRow {
                id: r.get_string("id"),
                destination: r.get_string("destination"),
                note: r.get_string("note"),
                created_by: r.get_string("created_by"),
                created_at: r.get_i64("created_at"),
            })
            .collect())
    }

    /// Add a grant. Returns `false` when the destination is already granted.
    pub async fn add_assist_egress_grant(
        &self,
        row: &AssistEgressGrantRow,
    ) -> Result<bool, StoreError> {
        Ok(q(
            "INSERT INTO assist_egress_grant (id, destination, note, created_by, created_at) \
              VALUES (?1, ?2, ?3, ?4, ?5) ON CONFLICT(destination) DO NOTHING",
        )
        .bind(&row.id)
        .bind(&row.destination)
        .bind(&row.note)
        .bind(&row.created_by)
        .bind(row.created_at)
        .execute(&self.backend)
        .await?
            > 0)
    }

    /// Remove a grant by id. Returns the removed row.
    pub async fn delete_assist_egress_grant(
        &self,
        id: &str,
    ) -> Result<Option<AssistEgressGrantRow>, StoreError> {
        let found = self
            .list_assist_egress_grants()
            .await?
            .into_iter()
            .find(|g| g.id == id);
        if found.is_some() {
            q("DELETE FROM assist_egress_grant WHERE id = ?1")
                .bind(id)
                .execute(&self.backend)
                .await?;
        }
        Ok(found)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const KEY: &str = "sk-t39e11-sealed-profile-key-0123456789";

    async fn store() -> Store {
        Store::open_in_memory(crate::seal::ServerKey::generate())
            .await
            .unwrap()
    }

    fn row(id: &str, endpoint: &str) -> AssistProfileRow {
        AssistProfileRow {
            id: id.into(),
            name: "Mock".into(),
            preset: "openai-compatible".into(),
            dialect: "openai".into(),
            endpoint: endpoint.into(),
            model: "m".into(),
            params_json: "{}".into(),
            has_key: false,
            created_by: "root".into(),
            created_at: 1,
            updated_by: "root".into(),
            updated_at: 1,
        }
    }

    /// The Postgres store under `MW_TEST_PG`, emptied of this module's rows, or
    /// `None` (the Postgres leg is then not run, and says so).
    async fn pg() -> Option<Store> {
        let dsn = std::env::var("MW_TEST_PG").ok().filter(|s| !s.is_empty());
        let Some(dsn) = dsn else {
            eprintln!("[assist_profiles] MW_TEST_PG unset: Postgres leg not run");
            return None;
        };
        let s = Store::open_postgres(&dsn, crate::seal::ServerKey::generate())
            .await
            .unwrap();
        for t in [
            "DELETE FROM assist_profile",
            "DELETE FROM assist_egress_grant",
        ] {
            q(t).execute(&s.backend).await.unwrap();
        }
        Some(s)
    }

    #[tokio::test]
    async fn the_profile_key_is_sealed_bound_and_never_listed() {
        profile_ops(store().await).await;
        if let Some(s) = pg().await {
            profile_ops(s).await;
            eprintln!("[assist_profiles] Postgres leg RAN");
        }
    }

    #[tokio::test]
    async fn grants_are_unique_per_destination() {
        grant_ops(store().await).await;
        if let Some(s) = pg().await {
            grant_ops(s).await;
        }
    }

    async fn profile_ops(s: Store) {
        let r = row("p1", "https://a.test/v1");
        s.put_assist_profile(&r, &ProfileKey::Set(KEY.into()))
            .await
            .unwrap()
            .unwrap();
        let raw = q("SELECT api_key_sealed FROM assist_profile WHERE id='p1'")
            .fetch_optional(&s.backend)
            .await
            .unwrap()
            .unwrap()
            .get_blob("api_key_sealed");
        assert!(
            !String::from_utf8_lossy(&raw).contains(KEY),
            "sealed at rest"
        );
        let listed = s.list_assist_profiles().await.unwrap();
        assert!(listed[0].has_key);
        assert!(!format!("{listed:?}").contains(KEY));
        assert_eq!(s.assist_profile_key("p1").await.unwrap().unwrap(), KEY);

        // Moving the endpoint without a new key is refused, and nothing changes.
        let moved = row("p1", "https://b.test/v1");
        assert_eq!(
            s.put_assist_profile(&moved, &ProfileKey::Keep)
                .await
                .unwrap(),
            Err(KEY_ENDPOINT_MOVED)
        );
        assert_eq!(
            s.get_assist_profile("p1").await.unwrap().unwrap().endpoint,
            "https://a.test/v1"
        );
        // Keep on the same endpoint keeps the key.
        let mut renamed = row("p1", "https://a.test/v1");
        renamed.name = "Renamed".into();
        s.put_assist_profile(&renamed, &ProfileKey::Keep)
            .await
            .unwrap()
            .unwrap();
        assert_eq!(s.assist_profile_key("p1").await.unwrap().unwrap(), KEY);

        // A sealed blob copied onto another row does not open there.
        s.put_assist_profile(&row("p2", "https://a.test/v1"), &ProfileKey::Clear)
            .await
            .unwrap()
            .unwrap();
        assert_eq!(s.assist_profile_key("p2").await.unwrap().unwrap(), "");
        q("UPDATE assist_profile SET api_key_sealed=?1 WHERE id='p2'")
            .bind(raw)
            .execute(&s.backend)
            .await
            .unwrap();
        assert!(matches!(
            s.assist_profile_key("p2").await,
            Err(StoreError::Seal(_))
        ));

        assert!(s.delete_assist_profile("p1").await.unwrap());
        assert!(s.get_assist_profile("p1").await.unwrap().is_none());
    }

    async fn grant_ops(s: Store) {
        let g = AssistEgressGrantRow {
            id: "g1".into(),
            destination: "127.0.0.1:11434".into(),
            note: "ollama".into(),
            created_by: "root".into(),
            created_at: 1,
        };
        assert!(s.add_assist_egress_grant(&g).await.unwrap());
        let dup = AssistEgressGrantRow {
            id: "g2".into(),
            ..g.clone()
        };
        assert!(!s.add_assist_egress_grant(&dup).await.unwrap());
        assert_eq!(
            s.list_assist_egress_grants().await.unwrap(),
            vec![g.clone()]
        );
        assert_eq!(s.delete_assist_egress_grant("g1").await.unwrap(), Some(g));
        assert!(s.list_assist_egress_grants().await.unwrap().is_empty());
    }
}
