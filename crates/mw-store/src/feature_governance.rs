//! Atomic administration of the existing feature catalogue. Session authority is
//! resolved separately; accepting a target here never authenticates that user.
use crate::{
    Store, StoreError,
    backend::{Tx, q},
    feature_profiles::{FeatureProfile, ProfileAssignment},
    groups::UserGroup,
};
use serde::Serialize;
use sha2::{Digest, Sha256};
use std::collections::BTreeMap;

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct NativeFeatureTarget {
    pub account_id: String,
    pub address: String,
    pub domain: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FeatureSnapshot {
    pub revision: String,
    pub profiles: Vec<FeatureProfile>,
    pub groups: Vec<UserGroup>,
    pub assignments: Vec<ProfileAssignment>,
    pub memberships: BTreeMap<String, Vec<String>>,
    pub native_targets: Vec<NativeFeatureTarget>,
    pub domains: Vec<String>,
    #[serde(skip)]
    aliases: BTreeMap<String, String>,
}

pub enum FeatureMutation {
    PutProfile(FeatureProfile),
    DeleteProfile(String),
    PutGroup(UserGroup),
    DeleteGroup(String),
    Members { group: String, members: Vec<String> },
    PutAssignment(ProfileAssignment),
    DeleteAssignment(ProfileAssignment),
}
#[derive(Debug)]
pub enum FeatureWriteError {
    Store(StoreError),
    Stale,
    Invalid(&'static str),
}
impl From<StoreError> for FeatureWriteError {
    fn from(e: StoreError) -> Self {
        Self::Store(e)
    }
}
impl From<sqlx::Error> for FeatureWriteError {
    fn from(e: sqlx::Error) -> Self {
        Self::Store(e.into())
    }
}

fn canonical_domain(value: &str, aliases: &BTreeMap<String, String>) -> Result<String, StoreError> {
    if value.is_empty() {
        return Ok(String::new());
    }
    let mut current = crate::settings_policy::normalize_policy_domain(value)?;
    let mut seen = Vec::new();
    while let Some(next) = aliases.get(&current) {
        if seen.contains(&current) || seen.len() >= 16 {
            return Err(StoreError::Corrupt("cyclic policy domain".into()));
        }
        seen.push(current);
        current = next.clone();
    }
    Ok(current)
}

async fn snapshot(tx: &mut Tx) -> Result<FeatureSnapshot, StoreError> {
    let profiles =
        q("SELECT id,name,values_json,builtin,updated_at FROM feature_profile ORDER BY id")
            .fetch_all_tx(tx)
            .await?
            .into_iter()
            .map(|r| {
                Ok(FeatureProfile {
                    id: r.get_string("id"),
                    name: r.get_string("name"),
                    values: serde_json::from_str(&r.get_string("values_json"))
                        .map_err(|_| StoreError::Corrupt("invalid feature profile".into()))?,
                    builtin: r.get_i64("builtin") != 0,
                    updated_at: r.get_i64("updated_at"),
                })
            })
            .collect::<Result<Vec<_>, StoreError>>()?;
    let groups: Vec<_> =
        q("SELECT id,name,domain,source,ldap_dn,created_at FROM user_group ORDER BY id")
            .fetch_all_tx(tx)
            .await?
            .into_iter()
            .map(|r| UserGroup {
                id: r.get_string("id"),
                name: r.get_string("name"),
                domain: r.get_string("domain"),
                source: r.get_string("source"),
                ldap_dn: r.get_opt_string("ldap_dn"),
                created_at: r.get_i64("created_at"),
            })
            .collect();
    let assignments = q("SELECT scope_kind,scope_id,profile_id,priority FROM feature_profile_assignment ORDER BY scope_kind,scope_id,profile_id").fetch_all_tx(tx).await?.into_iter().map(|r|ProfileAssignment {scope_kind:r.get_string("scope_kind"),scope_id:r.get_string("scope_id"),profile_id:r.get_string("profile_id"),priority:r.get_i64("priority")}).collect();
    let mut memberships: BTreeMap<String, Vec<String>> =
        groups.iter().map(|g| (g.id.clone(), Vec::new())).collect();
    for row in q("SELECT group_id,account_id FROM user_group_member ORDER BY group_id,account_id")
        .fetch_all_tx(tx)
        .await?
    {
        memberships
            .entry(row.get_string("group_id"))
            .or_default()
            .push(row.get_string("account_id"));
    }
    let aliases: BTreeMap<_, _> = q("SELECT alias,domain FROM domain_alias ORDER BY alias")
        .fetch_all_tx(tx)
        .await?
        .into_iter()
        .map(|r| (r.get_string("alias"), r.get_string("domain")))
        .collect();
    let domains = q("SELECT name FROM domains ORDER BY name")
        .fetch_all_tx(tx)
        .await?
        .into_iter()
        .map(|r| r.get_string("name"))
        .collect();
    let mut native_targets = Vec::new();
    for row in q("SELECT id,username FROM accounts ORDER BY id")
        .fetch_all_tx(tx)
        .await?
    {
        let address = row.get_string("username").trim().to_lowercase();
        let domain = canonical_domain(
            address.rsplit_once('@').map(|(_, d)| d).unwrap_or(""),
            &aliases,
        )?;
        native_targets.push(NativeFeatureTarget {
            account_id: row.get_string("id"),
            address,
            domain,
        });
    }
    let mut result = FeatureSnapshot {
        revision: String::new(),
        profiles,
        groups,
        assignments,
        memberships,
        native_targets,
        domains,
        aliases,
    };
    // Include alias routing too: a changed canonical domain invalidates a form's
    // previously displayed target membership even when profile rows are unchanged.
    let bytes = serde_json::to_vec(&(&result, &result.aliases))
        .map_err(|_| StoreError::Corrupt("invalid feature catalogue".into()))?;
    let digest: String = Sha256::digest(bytes)
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect();
    result.revision = format!("fg1_{digest}");
    Ok(result)
}

impl Store {
    pub async fn feature_governance_snapshot(&self) -> Result<FeatureSnapshot, StoreError> {
        let mut tx = self.backend.begin().await?;
        q("UPDATE identity_serialization SET id=id WHERE id=1")
            .execute_tx(&mut tx)
            .await?;
        let result = snapshot(&mut tx).await?;
        tx.commit().await?;
        Ok(result)
    }

    pub async fn mutate_feature_governance(
        &self,
        expected: &str,
        change: FeatureMutation,
        actor: &str,
    ) -> Result<FeatureSnapshot, FeatureWriteError> {
        let mut tx = self.backend.begin().await?;
        q("UPDATE identity_serialization SET id=id WHERE id=1")
            .execute_tx(&mut tx)
            .await?;
        let current = snapshot(&mut tx).await?;
        if current.revision != expected {
            return Err(FeatureWriteError::Stale);
        }
        match change {
            FeatureMutation::PutProfile(p) => {
                if p.id.is_empty()
                    || p.id.len() > 256
                    || p.name.trim().is_empty()
                    || p.name.len() > 128
                    || p.values.len() > 256
                {
                    return Err(FeatureWriteError::Invalid("invalidProfile"));
                }
                if current
                    .profiles
                    .iter()
                    .any(|old| old.id == p.id && old.builtin)
                {
                    return Err(FeatureWriteError::Invalid("builtinProfile"));
                }
                if current
                    .profiles
                    .iter()
                    .any(|old| old.id != p.id && old.name == p.name)
                {
                    return Err(FeatureWriteError::Invalid("duplicateProfileName"));
                }
                let json = serde_json::to_string(&p.values)
                    .map_err(|_| FeatureWriteError::Invalid("invalidProfile"))?;
                q("INSERT INTO feature_profile(id,name,values_json,builtin,updated_at) VALUES(?1,?2,?3,0,?4) ON CONFLICT(id) DO UPDATE SET name=excluded.name,values_json=excluded.values_json,updated_at=excluded.updated_at")
                    .bind(&p.id).bind(p.name).bind(json).bind(p.updated_at).execute_tx(&mut tx).await?;
                q("DELETE FROM setting_value WHERE scope_kind='profile' AND scope_id=?1")
                    .bind(&p.id)
                    .execute_tx(&mut tx)
                    .await?;
                for (key, value) in p.values {
                    q("INSERT INTO setting_value(scope_kind,scope_id,key,value,locked,updated_by,updated_at) VALUES('profile',?1,?2,?3,?4,?5,?6)").bind(&p.id).bind(key).bind(value.value.to_string()).bind(i64::from(value.locked)).bind(actor).bind(p.updated_at).execute_tx(&mut tx).await?;
                }
            }
            FeatureMutation::DeleteProfile(id) => {
                if current.profiles.iter().any(|p| p.id == id && p.builtin) {
                    return Err(FeatureWriteError::Invalid("builtinProfile"));
                }
                q("DELETE FROM setting_value WHERE scope_kind='profile' AND scope_id=?1")
                    .bind(&id)
                    .execute_tx(&mut tx)
                    .await?;
                q("DELETE FROM feature_profile WHERE id=?1")
                    .bind(id)
                    .execute_tx(&mut tx)
                    .await?;
            }
            FeatureMutation::PutGroup(g) => {
                if g.id.is_empty()
                    || g.id.len() > 256
                    || g.name.trim().is_empty()
                    || g.name.len() > 128
                    || g.source != "manual"
                {
                    return Err(FeatureWriteError::Invalid("invalidGroup"));
                }
                if current
                    .groups
                    .iter()
                    .any(|old| old.id == g.id && old.source != "manual")
                {
                    return Err(FeatureWriteError::Invalid("directoryManaged"));
                }
                let domain = canonical_domain(&g.domain, &current.aliases)?;
                if !domain.is_empty() && !current.domains.contains(&domain) {
                    return Err(FeatureWriteError::Invalid("unknownDomain"));
                }
                for member in current.memberships.get(&g.id).into_iter().flatten() {
                    validate_member(&current, member, &domain)?;
                }
                q("INSERT INTO user_group(id,name,domain,source,ldap_dn,created_at) VALUES(?1,?2,?3,'manual',NULL,?4) ON CONFLICT(id) DO UPDATE SET name=excluded.name,domain=excluded.domain")
                    .bind(g.id).bind(g.name).bind(domain).bind(g.created_at).execute_tx(&mut tx).await?;
            }
            FeatureMutation::DeleteGroup(id) => {
                if current
                    .groups
                    .iter()
                    .any(|g| g.id == id && g.source != "manual")
                {
                    return Err(FeatureWriteError::Invalid("directoryManaged"));
                }
                q("DELETE FROM feature_profile_assignment WHERE scope_kind='group' AND scope_id=?1").bind(&id).execute_tx(&mut tx).await?;
                q("DELETE FROM setting_value WHERE scope_kind='group' AND scope_id=?1")
                    .bind(&id)
                    .execute_tx(&mut tx)
                    .await?;
                q("DELETE FROM user_group WHERE id=?1")
                    .bind(id)
                    .execute_tx(&mut tx)
                    .await?;
            }
            FeatureMutation::Members { group, members } => {
                let g = current
                    .groups
                    .iter()
                    .find(|g| g.id == group)
                    .ok_or(FeatureWriteError::Invalid("unknownGroup"))?;
                if g.source != "manual" {
                    return Err(FeatureWriteError::Invalid("directoryManaged"));
                }
                if members.len() > 10000 {
                    return Err(FeatureWriteError::Invalid("invalidMembers"));
                }
                for member in &members {
                    validate_member(&current, member, &g.domain)?;
                }
                q("DELETE FROM user_group_member WHERE group_id=?1")
                    .bind(&group)
                    .execute_tx(&mut tx)
                    .await?;
                for member in members {
                    q("INSERT INTO user_group_member(group_id,account_id) VALUES(?1,?2) ON CONFLICT(group_id,account_id) DO NOTHING").bind(&group).bind(member).execute_tx(&mut tx).await?;
                }
            }
            FeatureMutation::PutAssignment(mut a) => {
                validate_assignment(&current, &mut a)?;
                q("INSERT INTO feature_profile_assignment(scope_kind,scope_id,profile_id,priority) VALUES(?1,?2,?3,?4) ON CONFLICT(scope_kind,scope_id,profile_id) DO UPDATE SET priority=excluded.priority")
                    .bind(a.scope_kind).bind(a.scope_id).bind(a.profile_id).bind(a.priority).execute_tx(&mut tx).await?;
            }
            FeatureMutation::DeleteAssignment(a) => {
                // Permit removal of an orphaned legacy target without granting it.
                let scope = if a.scope_kind == "domain" {
                    canonical_domain(&a.scope_id, &current.aliases)?
                } else {
                    a.scope_id
                };
                q("DELETE FROM feature_profile_assignment WHERE scope_kind=?1 AND scope_id=?2 AND profile_id=?3").bind(a.scope_kind).bind(scope).bind(a.profile_id).execute_tx(&mut tx).await?;
            }
        }
        let result = snapshot(&mut tx).await?;
        tx.commit().await?;
        Ok(result)
    }
}

fn validate_member(s: &FeatureSnapshot, id: &str, domain: &str) -> Result<(), FeatureWriteError> {
    let target = s
        .native_targets
        .iter()
        .find(|t| t.account_id == id)
        .ok_or(FeatureWriteError::Invalid("unknownNativeAccount"))?;
    if !domain.is_empty() && target.domain != domain {
        return Err(FeatureWriteError::Invalid("memberOutsideDomain"));
    }
    Ok(())
}
fn validate_assignment(
    s: &FeatureSnapshot,
    a: &mut ProfileAssignment,
) -> Result<(), FeatureWriteError> {
    if !s.profiles.iter().any(|p| p.id == a.profile_id) {
        return Err(FeatureWriteError::Invalid("unknownProfile"));
    }
    match a.scope_kind.as_str() {
        "user" => validate_member(s, &a.scope_id, "")?,
        "group" => {
            if !s.groups.iter().any(|g| g.id == a.scope_id) {
                return Err(FeatureWriteError::Invalid("unknownGroup"));
            }
        }
        "domain" => {
            a.scope_id = canonical_domain(&a.scope_id, &s.aliases)?;
            if !s.domains.contains(&a.scope_id) {
                return Err(FeatureWriteError::Invalid("unknownDomain"));
            }
        }
        _ => return Err(FeatureWriteError::Invalid("invalidAssignment")),
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ServerKey;

    fn profile(id: &str) -> FeatureMutation {
        FeatureMutation::PutProfile(FeatureProfile {
            id: id.into(),
            name: id.into(),
            values: BTreeMap::new(),
            builtin: false,
            updated_at: 1,
        })
    }

    async fn exercise(store: Store) {
        q("INSERT INTO domains(name,upstream_json) VALUES('one.test','{}'),('two.test','{}')")
            .execute(&store.backend)
            .await
            .unwrap();
        q("INSERT INTO accounts(id,kind,host,port,tls,username,sealed_creds) VALUES('native','imap','mail.test',993,'implicit','user@one.test',?1)").bind(vec![0u8]).execute(&store.backend).await.unwrap();
        let initial = store.feature_governance_snapshot().await.unwrap();
        let a = store.clone();
        let b = store.clone();
        let rev = initial.revision.clone();
        let (one, two) = tokio::join!(
            a.mutate_feature_governance(&rev, profile("one"), "admin"),
            b.mutate_feature_governance(&rev, profile("two"), "admin")
        );
        assert_eq!(usize::from(one.is_ok()) + usize::from(two.is_ok()), 1);
        assert!(
            matches!(one, Err(FeatureWriteError::Stale))
                || matches!(two, Err(FeatureWriteError::Stale))
        );
        let current = store.feature_governance_snapshot().await.unwrap();
        assert_eq!(current.profiles.len(), 1);
        let profile_id = current.profiles[0].id.clone();
        let invalid = store
            .mutate_feature_governance(
                &current.revision,
                FeatureMutation::PutAssignment(ProfileAssignment {
                    scope_kind: "user".into(),
                    scope_id: "proxy-raw-id".into(),
                    profile_id: profile_id.clone(),
                    priority: 1,
                }),
                "admin",
            )
            .await;
        assert!(matches!(
            invalid,
            Err(FeatureWriteError::Invalid("unknownNativeAccount"))
        ));
        let current = store
            .mutate_feature_governance(
                &current.revision,
                FeatureMutation::PutAssignment(ProfileAssignment {
                    scope_kind: "user".into(),
                    scope_id: "native".into(),
                    profile_id,
                    priority: 1,
                }),
                "admin",
            )
            .await
            .unwrap();
        let current = store
            .mutate_feature_governance(
                &current.revision,
                FeatureMutation::PutGroup(UserGroup {
                    id: "group".into(),
                    name: "Group".into(),
                    domain: "two.test".into(),
                    source: "manual".into(),
                    ldap_dn: None,
                    created_at: 1,
                }),
                "admin",
            )
            .await
            .unwrap();
        let invalid = store
            .mutate_feature_governance(
                &current.revision,
                FeatureMutation::Members {
                    group: "group".into(),
                    members: vec!["native".into()],
                },
                "admin",
            )
            .await;
        assert!(matches!(
            invalid,
            Err(FeatureWriteError::Invalid("memberOutsideDomain"))
        ));
        assert!(
            store
                .feature_governance_snapshot()
                .await
                .unwrap()
                .memberships["group"]
                .is_empty()
        );
        let current = store
            .mutate_feature_governance(
                &current.revision,
                FeatureMutation::PutGroup(UserGroup {
                    id: "group".into(),
                    name: "Group".into(),
                    domain: "one.test".into(),
                    source: "manual".into(),
                    ldap_dn: None,
                    created_at: 1,
                }),
                "admin",
            )
            .await
            .unwrap();
        let current = store
            .mutate_feature_governance(
                &current.revision,
                FeatureMutation::Members {
                    group: "group".into(),
                    members: vec!["native".into()],
                },
                "admin",
            )
            .await
            .unwrap();
        assert_eq!(current.memberships["group"], vec!["native"]);
        assert!(matches!(
            store
                .mutate_feature_governance(
                    &initial.revision,
                    FeatureMutation::DeleteGroup("group".into()),
                    "admin"
                )
                .await,
            Err(FeatureWriteError::Stale)
        ));
    }
    #[tokio::test]
    async fn catalogue_cas_sqlite() {
        exercise(Store::open_in_memory(ServerKey::generate()).await.unwrap()).await;
    }
    #[tokio::test]
    async fn catalogue_cas_postgres() {
        use sqlx::Connection;
        let Ok(dsn) = std::env::var("MW_TEST_PG") else {
            eprintln!("SKIP: MW_TEST_PG absent");
            return;
        };
        let schema = format!("feature_cas_{}", uuid::Uuid::new_v4().simple());
        let mut admin = sqlx::PgConnection::connect(&dsn).await.unwrap();
        sqlx::query(&format!("CREATE SCHEMA {schema}"))
            .execute(&mut admin)
            .await
            .unwrap();
        let mut scoped = url::Url::parse(&dsn).unwrap();
        scoped
            .query_pairs_mut()
            .append_pair("options", &format!("-csearch_path={schema}"));
        exercise(
            Store::open(scoped.as_str(), ServerKey::generate())
                .await
                .unwrap(),
        )
        .await;
        sqlx::query(&format!("DROP SCHEMA {schema} CASCADE"))
            .execute(&mut admin)
            .await
            .unwrap();
    }
}
