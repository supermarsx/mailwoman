//! No-I/O prerequisite for composite admission. These are policy DATA, not an
//! authenticated identity or allow decision. Missing entries force fresh fallback.
use super::*;
impl Store {
    /// Copy the existing resolver's complete inputs only at this exact fresh
    /// Store-bound revision. Never hydrates an entry or consults SQL. The caller
    /// must validate the current authoritative subject before using this data.
    /// Each memo entry is already bounded to2MiB; copies are not an RSS ceiling.
    pub fn try_settings_resolution_inputs_at(
        &self,
        subject: &SettingSubject<'_>,
        revision: &SettingsReadRevision,
    ) -> Result<Option<SettingsResolutionInputs>, StoreError> {
        if revision.scope != self.cache_scope() {
            return Err(StoreError::Corrupt(
                "settings revision belongs to another store".into(),
            ));
        }
        let Some(number) = revision.number().filter(|number| *number >= 0) else {
            return Ok(None);
        };
        let input = [
            subject.personal_owner.unwrap_or(""),
            subject.domain,
            subject.account_id,
            subject.address,
        ];
        if input.iter().any(|s| s.len() > 1024) {
            return Ok(None);
        };
        let Some(canonical) = CANONICAL_MEMO.get(self.cache_scope(), &subject_key(subject), number)
        else {
            return Ok(None);
        };
        if [&canonical.domain, &canonical.account_id, &canonical.address]
            .iter()
            .any(|s| s.len() > 1024)
            || canonical
                .personal_owner
                .as_ref()
                .is_some_and(|s| s.len() > 1024)
        {
            return Ok(None);
        };
        // UserSubject::resolved and effective_all currently canonicalize twice.
        // Require both exact memo entries and idempotence, not just C1 presence.
        let key = subject_key(&canonical.as_subject());
        let Some(second) = CANONICAL_MEMO.get(self.cache_scope(), &key, number) else {
            return Ok(None);
        };
        if canonical != second {
            return Ok(None);
        };
        let Some(policies) = ALL_POLICIES_MEMO.get(self.cache_scope(), "all", number) else {
            return Ok(None);
        };
        let Some(values) = VALUES_MEMO.get(self.cache_scope(), &key, number) else {
            return Ok(None);
        };
        Ok(Some(SettingsResolutionInputs {
            canonical,
            policies,
            values,
        }))
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    fn subject() -> SettingSubject<'static> {
        SettingSubject {
            personal_owner: None,
            domain: "",
            account_id: "",
            address: "",
        }
    }
    #[tokio::test]
    async fn warm_inputs_equal_reader_cold_dirty_and_foreign_proofs_never_reuse() {
        let s = Store::open_in_memory(crate::ServerKey::generate())
            .await
            .unwrap();
        let rev = s.settings_read_revision().await.unwrap();
        assert!(
            s.try_settings_resolution_inputs_at(&subject(), &rev)
                .unwrap()
                .is_none()
        );
        let old = s
            .settings_resolution_inputs_at(&subject(), &rev)
            .await
            .unwrap();
        let (hit, count) = crate::pool_telemetry::measure_queries(async {
            s.try_settings_resolution_inputs_at(&subject(), &rev)
        })
        .await;
        let hit = hit.unwrap().unwrap();
        assert_eq!(count.read.calls, 0);
        assert_eq!(hit.canonical, old.canonical);
        assert_eq!(hit.policies, old.policies);
        assert_eq!(hit.values, old.values);
        crate::q("UPDATE settings_revision SET n=n+1 WHERE id=1")
            .execute(&s.backend)
            .await
            .unwrap();
        let dirty = s.settings_read_revision().await.unwrap();
        assert!(
            s.try_settings_resolution_inputs_at(&subject(), &dirty)
                .unwrap()
                .is_none()
        );
        let other = Store::open_in_memory(crate::ServerKey::generate())
            .await
            .unwrap();
        assert!(
            other
                .try_settings_resolution_inputs_at(&subject(), &rev)
                .is_err()
        );
        s.close().await;
        other.close().await;
    }
    #[tokio::test]
    async fn zero_is_valid_missing_revision_disables_and_nonidempotent_memo_falls_back() {
        let s = Store::open_in_memory(crate::ServerKey::generate())
            .await
            .unwrap();
        crate::q("UPDATE settings_revision SET n=0 WHERE id=1")
            .execute(&s.backend)
            .await
            .unwrap();
        let rev = s.settings_read_revision().await.unwrap();
        s.settings_resolution_inputs_at(&subject(), &rev)
            .await
            .unwrap();
        assert!(
            s.try_settings_resolution_inputs_at(&subject(), &rev)
                .unwrap()
                .is_some()
        );
        CANONICAL_MEMO.put(
            s.cache_scope(),
            subject_key(&subject()),
            0,
            CanonicalSettingSubject {
                personal_owner: None,
                domain: "changed.test".into(),
                account_id: "".into(),
                address: "".into(),
            },
        );
        assert!(
            s.try_settings_resolution_inputs_at(&subject(), &rev)
                .unwrap()
                .is_none()
        );
        crate::q("DELETE FROM settings_revision WHERE id=1")
            .execute(&s.backend)
            .await
            .unwrap();
        let missing = s.settings_read_revision().await.unwrap();
        assert!(
            s.try_settings_resolution_inputs_at(&subject(), &missing)
                .unwrap()
                .is_none()
        );
        s.close().await;
    }
}
