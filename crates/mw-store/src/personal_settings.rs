//! Verified proxy personal overrides. Never infer ownership of legacy rows.
use crate::{
    Store, StoreError,
    backend::q,
    settings_policy::{SettingScope, SettingValueRow},
};
impl Store {
    pub async fn personal_setting_values(
        &self,
        owner: &str,
    ) -> Result<Vec<SettingValueRow>, StoreError> {
        Ok(q("SELECT key,value,updated_at FROM personal_setting_value WHERE owner_subject=?1 ORDER BY key").bind(owner).fetch_all(&self.backend).await?.into_iter().map(|r|SettingValueRow{
            scope:SettingScope::User,scope_id:owner.into(),key:r.get_string("key"),value:r.get_string("value"),locked:false,updated_by:owner.into(),updated_at:r.get_i64("updated_at"),
        }).collect())
    }
    pub async fn put_personal_setting(
        &self,
        owner: &str,
        key: &str,
        value: &str,
    ) -> Result<(), StoreError> {
        q("INSERT INTO personal_setting_value(owner_subject,key,value,updated_at) VALUES(?1,?2,?3,?4) ON CONFLICT(owner_subject,key) DO UPDATE SET value=excluded.value,updated_at=excluded.updated_at")
            .bind(owner).bind(key).bind(value).bind(chrono::Utc::now().timestamp()).execute(&self.backend).await?;
        Ok(())
    }
    pub async fn delete_personal_setting(&self, owner: &str, key: &str) -> Result<(), StoreError> {
        q("DELETE FROM personal_setting_value WHERE owner_subject=?1 AND key=?2")
            .bind(owner)
            .bind(key)
            .execute(&self.backend)
            .await?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        Credentials, ServerKey, identity_principal::VerifiedPrimary,
        settings_policy::SettingSubject,
    };

    async fn exercise(store: Store) {
        q("INSERT INTO accounts(id,kind,host,port,tls,username,sealed_creds) VALUES('1','imap','native.test',993,'implicit','same@example.test',?1)").bind(vec![0u8]).execute(&store.backend).await.unwrap();
        q("INSERT INTO feature_profile(id,name,values_json,builtin,updated_at) VALUES('p','Native','{}',0,1),('d','Domain','{}',0,1)").execute(&store.backend).await.unwrap();
        q("INSERT INTO user_group(id,name,domain,source,created_at) VALUES('g','Native group','','manual',1)").execute(&store.backend).await.unwrap();
        q("INSERT INTO user_group_member(group_id,account_id) VALUES('g','1')")
            .execute(&store.backend)
            .await
            .unwrap();
        q("INSERT INTO feature_profile_assignment(scope_kind,scope_id,profile_id,priority) VALUES('user','1','p',1),('domain','example.test','d',1)").execute(&store.backend).await.unwrap();
        for (scope, id, key) in [
            (SettingScope::AdminUser, "1", "cap.contacts"),
            (
                SettingScope::AdminUser,
                "same@example.test",
                "limit.messages",
            ),
            (SettingScope::User, "1", "mail.showFrom"),
            (SettingScope::Group, "g", "cap.tasks"),
            (SettingScope::Profile, "p", "cap.notes"),
            (SettingScope::Profile, "d", "cap.calendar"),
        ] {
            store
                .put_setting_value(&SettingValueRow {
                    scope,
                    scope_id: id.into(),
                    key: key.into(),
                    value: "false".into(),
                    locked: false,
                    updated_by: "test".into(),
                    updated_at: 1,
                })
                .await
                .unwrap();
        }
        let mut owners = Vec::new();
        for service in [
            "https://proxy.test/a/session",
            "https://proxy.test/b/session",
        ] {
            let id = store
                .create_session(
                    "1",
                    "same@example.test",
                    service,
                    "https://proxy.test/api",
                    &Credentials {
                        username: "same@example.test".into(),
                        password: "synthetic".into(),
                    },
                )
                .await
                .unwrap();
            store
                .set_session_authentication(&id, "header-auth")
                .await
                .unwrap();
            store
                .bind_verified_primary(
                    &id,
                    VerifiedPrimary {
                        account_id: "1",
                        kind: "header",
                        authority: "trusted-header",
                        subject: "verified",
                    },
                    100,
                )
                .await
                .unwrap()
                .unwrap();
            let verified = store.session_policy_subject(&id).await.unwrap().unwrap();
            assert!(verified.native_account.is_none());
            owners.push(verified.personal_subject);
        }
        assert_ne!(owners[0], owners[1]);
        // Even an old attacker-chosen raw ID equal to po1 cannot enter the typed table.
        store
            .put_setting_value(&SettingValueRow {
                scope: SettingScope::User,
                scope_id: owners[0].clone(),
                key: "mail.showFrom".into(),
                value: "false".into(),
                locked: false,
                updated_by: "legacy".into(),
                updated_at: 1,
            })
            .await
            .unwrap();
        store
            .put_personal_setting(&owners[0], "mail.showFrom", "true")
            .await
            .unwrap();
        for (index, owner) in owners.iter().enumerate() {
            let rows = store
                .setting_values_for_subject(&SettingSubject {
                    personal_owner: Some(owner),
                    account_id: "1",
                    address: "same@example.test",
                    domain: "example.test",
                })
                .await
                .unwrap();
            assert!(
                rows.iter().any(|r| r.key == "cap.calendar"),
                "domain profile remains effective"
            );
            assert!(
                !rows.iter().any(
                    |r| ["cap.contacts", "limit.messages", "cap.tasks", "cap.notes"]
                        .contains(&r.key.as_str())
                ),
                "native rows must not leak into a proxy realm"
            );
            let own: Vec<_> = rows.iter().filter(|r| r.key == "mail.showFrom").collect();
            if index == 0 {
                assert_eq!(own.len(), 1);
                assert_eq!(own[0].value, "true");
            } else {
                assert!(own.is_empty());
            }
        }
        let native = store
            .setting_values_for_subject(&SettingSubject {
                personal_owner: None,
                account_id: "1",
                address: "same@example.test",
                domain: "example.test",
            })
            .await
            .unwrap();
        for key in [
            "cap.contacts",
            "limit.messages",
            "cap.tasks",
            "cap.notes",
            "cap.calendar",
        ] {
            assert!(
                native.iter().any(|r| r.key == key),
                "native compatibility {key}"
            );
        }
        store
            .delete_personal_setting(&owners[1], "mail.showFrom")
            .await
            .unwrap();
        assert_eq!(
            store
                .personal_setting_values(&owners[0])
                .await
                .unwrap()
                .len(),
            1
        );
        assert!(
            store
                .get_setting_value(SettingScope::User, &owners[0], "mail.showFrom")
                .await
                .unwrap()
                .is_some(),
            "legacy rows retained"
        );
    }
    #[tokio::test]
    async fn verified_policy_realms_sqlite() {
        exercise(Store::open_in_memory(ServerKey::generate()).await.unwrap()).await;
    }
    #[tokio::test]
    async fn verified_policy_realms_postgres() {
        use sqlx::Connection;
        let Ok(dsn) = std::env::var("MW_TEST_PG") else {
            eprintln!("SKIP: MW_TEST_PG absent");
            return;
        };
        let schema = format!("personal_settings_{}", uuid::Uuid::new_v4().simple());
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
