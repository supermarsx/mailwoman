//! Feature profiles project atomically into the existing settings authority.
use crate::backend::q;
use crate::{Store, StoreError};
use std::collections::BTreeMap;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, PartialEq)]
pub struct ProfileValue {
    pub value: serde_json::Value,
    #[serde(default)]
    pub locked: bool,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::groups::UserGroup;
    use crate::settings_policy::{SettingScope, SettingSubject};

    fn profile(id: &str, value: bool) -> FeatureProfile {
        FeatureProfile {
            id: id.into(),
            name: id.into(),
            values: BTreeMap::from([(
                "cap.export".into(),
                ProfileValue {
                    value: value.into(),
                    locked: true,
                },
            )]),
            builtin: false,
            updated_at: 1,
        }
    }

    #[tokio::test]
    async fn profiles_project_replace_and_resolve_membership_hot_with_stable_priority() {
        let s = Store::open_in_memory(crate::ServerKey::generate())
            .await
            .unwrap();
        s.put_domain_config(&crate::DomainConfigRow {
            name: "example.test".into(),
            upstream_json: "{}".into(),
            allowlist_json: "[]".into(),
            blocklist_json: "[]".into(),
            enabled: true,
            preferred: false,
            settings_json: "{}".into(),
            updated_at: 1,
        })
        .await
        .unwrap();
        s.put_domain_alias("alias.test", "example.test")
            .await
            .unwrap();
        for (id, value) in [("domain", true), ("group", false), ("user", true)] {
            s.put_feature_profile(&profile(id, value), "admin")
                .await
                .unwrap();
        }
        let g = UserGroup {
            id: "g1".into(),
            name: "Group".into(),
            domain: "alias.test".into(),
            source: "manual".into(),
            ldap_dn: None,
            created_at: 1,
        };
        s.put_user_group(&g).await.unwrap();
        s.replace_user_group_members("g1", &["account".into()])
            .await
            .unwrap();
        for (kind, id, p) in [
            ("domain", "alias.test", "domain"),
            ("group", "g1", "group"),
            ("user", "account", "user"),
        ] {
            s.put_feature_profile_assignment(&ProfileAssignment {
                scope_kind: kind.into(),
                scope_id: id.into(),
                profile_id: p.into(),
                priority: 1,
            })
            .await
            .unwrap();
        }
        assert_eq!(
            s.setting_profile_and_group_ids("account", "alias.test")
                .await
                .unwrap(),
            (
                vec!["user".into(), "group".into(), "domain".into()],
                vec!["g1".into()]
            )
        );
        let subject = SettingSubject {
            personal_owner: None,
            domain: "alias.test",
            account_id: "account",
            address: "a@alias.test",
        };
        assert_eq!(
            s.held_setting("cap.export", "false", &subject)
                .await
                .unwrap()
                .as_deref(),
            Some("true")
        );
        s.put_feature_profile_assignment(&ProfileAssignment {
            scope_kind: "group".into(),
            scope_id: "g1".into(),
            profile_id: "group".into(),
            priority: 2,
        })
        .await
        .unwrap();
        assert_eq!(
            s.held_setting("cap.export", "true", &subject)
                .await
                .unwrap()
                .as_deref(),
            Some("false")
        );
        s.replace_user_group_members("g1", &[]).await.unwrap();
        assert_eq!(
            s.held_setting("cap.export", "false", &subject)
                .await
                .unwrap()
                .as_deref(),
            Some("true")
        );
        let mut replacement = profile("user", false);
        replacement.values.clear();
        s.put_feature_profile(&replacement, "admin").await.unwrap();
        assert!(
            s.setting_values_at(SettingScope::Profile, "user")
                .await
                .unwrap()
                .is_empty()
        );
        assert!(
            s.feature_profiles()
                .await
                .unwrap()
                .iter()
                .find(|p| p.id == "user")
                .unwrap()
                .values
                .is_empty()
        );
        s.delete_feature_profile("domain").await.unwrap();
        assert!(
            s.setting_values_at(SettingScope::Profile, "domain")
                .await
                .unwrap()
                .is_empty()
        );
        assert!(
            s.feature_profile_assignments()
                .await
                .unwrap()
                .iter()
                .all(|a| a.profile_id != "domain")
        );
    }

    #[tokio::test]
    async fn domain_group_cannot_leak_into_other_or_bare_identity() {
        let s = Store::open_in_memory(crate::ServerKey::generate())
            .await
            .unwrap();
        s.put_feature_profile(&profile("p", false), "admin")
            .await
            .unwrap();
        s.put_user_group(&UserGroup {
            id: "g".into(),
            name: "g".into(),
            domain: "a.test".into(),
            source: "manual".into(),
            ldap_dn: None,
            created_at: 1,
        })
        .await
        .unwrap();
        s.replace_user_group_members("g", &["a".into()])
            .await
            .unwrap();
        s.put_feature_profile_assignment(&ProfileAssignment {
            scope_kind: "group".into(),
            scope_id: "g".into(),
            profile_id: "p".into(),
            priority: 1,
        })
        .await
        .unwrap();
        for domain in ["b.test", ""] {
            assert_eq!(
                s.setting_profile_and_group_ids("a", domain).await.unwrap(),
                (vec![], vec![])
            );
        }
        s.delete_user_group("g").await.unwrap();
        assert!(s.feature_profile_assignments().await.unwrap().is_empty());
    }
}

#[derive(Debug, Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FeatureProfile {
    pub id: String,
    pub name: String,
    pub values: BTreeMap<String, ProfileValue>,
    pub builtin: bool,
    pub updated_at: i64,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProfileAssignment {
    pub scope_kind: String,
    pub scope_id: String,
    pub profile_id: String,
    pub priority: i64,
}

impl Store {
    pub async fn feature_profiles(&self) -> Result<Vec<FeatureProfile>, StoreError> {
        q("SELECT id,name,values_json,builtin,updated_at FROM feature_profile ORDER BY name,id")
            .fetch_all(&self.backend)
            .await?
            .into_iter()
            .map(|r| {
                Ok(FeatureProfile {
                    id: r.get_string("id"),
                    name: r.get_string("name"),
                    values: serde_json::from_str(&r.get_string("values_json")).map_err(|_| {
                        StoreError::Corrupt("invalid feature profile values".into())
                    })?,
                    builtin: r.get_i64("builtin") != 0,
                    updated_at: r.get_i64("updated_at"),
                })
            })
            .collect()
    }

    /// Callers validate every key/value against the server registry before this
    /// transaction. JSON and runtime rows cannot become independent authorities.
    pub async fn put_feature_profile(
        &self,
        profile: &FeatureProfile,
        actor: &str,
    ) -> Result<(), StoreError> {
        if profile.id.is_empty() || profile.name.trim().is_empty() {
            return Err(StoreError::Corrupt("invalid feature profile".into()));
        }
        let json = serde_json::to_string(&profile.values)
            .map_err(|_| StoreError::Corrupt("invalid feature profile values".into()))?;
        let mut tx = self.backend.begin().await?;
        q("INSERT INTO feature_profile(id,name,values_json,builtin,updated_at) VALUES(?1,?2,?3,?4,?5) ON CONFLICT(id) DO UPDATE SET name=excluded.name,values_json=excluded.values_json,updated_at=excluded.updated_at")
            .bind(&profile.id).bind(&profile.name).bind(json).bind(i64::from(profile.builtin)).bind(profile.updated_at).execute_tx(&mut tx).await?;
        q("DELETE FROM setting_value WHERE scope_kind='profile' AND scope_id=?1")
            .bind(&profile.id)
            .execute_tx(&mut tx)
            .await?;
        for (key, entry) in &profile.values {
            q("INSERT INTO setting_value(scope_kind,scope_id,key,value,locked,updated_by,updated_at) VALUES('profile',?1,?2,?3,?4,?5,?6)")
                .bind(&profile.id).bind(key).bind(entry.value.to_string()).bind(i64::from(entry.locked)).bind(actor).bind(profile.updated_at).execute_tx(&mut tx).await?;
        }
        tx.commit().await?;
        Ok(())
    }

    pub async fn delete_feature_profile(&self, id: &str) -> Result<(), StoreError> {
        let mut tx = self.backend.begin().await?;
        q("DELETE FROM setting_value WHERE scope_kind='profile' AND scope_id=?1")
            .bind(id)
            .execute_tx(&mut tx)
            .await?;
        q("DELETE FROM feature_profile WHERE id=?1")
            .bind(id)
            .execute_tx(&mut tx)
            .await?;
        tx.commit().await?;
        Ok(())
    }

    pub async fn feature_profile_assignments(&self) -> Result<Vec<ProfileAssignment>, StoreError> {
        Ok(q("SELECT scope_kind,scope_id,profile_id,priority FROM feature_profile_assignment ORDER BY priority DESC,scope_kind,scope_id,profile_id").fetch_all(&self.backend).await?.into_iter().map(|r|ProfileAssignment {scope_kind:r.get_string("scope_kind"),scope_id:r.get_string("scope_id"),profile_id:r.get_string("profile_id"),priority:r.get_i64("priority")}).collect())
    }

    pub async fn put_feature_profile_assignment(
        &self,
        value: &ProfileAssignment,
    ) -> Result<(), StoreError> {
        if !matches!(value.scope_kind.as_str(), "domain" | "group" | "user")
            || value.scope_id.is_empty()
        {
            return Err(StoreError::Corrupt("invalid profile assignment".into()));
        }
        let scope_id = if value.scope_kind == "domain" {
            self.canonical_policy_domain(&value.scope_id).await?
        } else {
            value.scope_id.clone()
        };
        q("INSERT INTO feature_profile_assignment(scope_kind,scope_id,profile_id,priority) VALUES(?1,?2,?3,?4) ON CONFLICT(scope_kind,scope_id,profile_id) DO UPDATE SET priority=excluded.priority")
            .bind(&value.scope_kind).bind(scope_id).bind(&value.profile_id).bind(value.priority).execute(&self.backend).await?;
        Ok(())
    }

    pub async fn delete_feature_profile_assignment(
        &self,
        value: &ProfileAssignment,
    ) -> Result<(), StoreError> {
        let scope_id = if value.scope_kind == "domain" {
            self.canonical_policy_domain(&value.scope_id).await?
        } else {
            value.scope_id.clone()
        };
        q("DELETE FROM feature_profile_assignment WHERE scope_kind=?1 AND scope_id=?2 AND profile_id=?3").bind(&value.scope_kind).bind(scope_id).bind(&value.profile_id).execute(&self.backend).await?;
        Ok(())
    }
}
