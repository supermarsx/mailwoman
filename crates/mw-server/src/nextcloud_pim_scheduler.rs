//! Periodic managed PIM refresh: bounded pages, no account login or credential replay.
use mw_engine::Engine;
use mw_store::Store;
use std::{
    sync::{Arc, Weak},
    time::Duration,
};

pub(crate) fn spawn(engine: &Arc<Engine>, store: Store) {
    let engine = Arc::downgrade(engine);
    tokio::spawn(run(engine, store));
}
async fn interval(store: &Store, account: &str) -> Option<Duration> {
    let address = store.security_account_address(account).await.ok()??;
    let address = store.canonical_policy_address(&address).await.ok()?;
    let subject = mw_store::settings_policy::SettingSubject {
        account_id: account,
        address: &address,
        domain: address.rsplit_once('@').map(|(_, d)| d).unwrap_or(""),
    };
    let value =
        crate::settings_registry::effective_for(store, "nextcloud.syncIntervalSecs", &subject)
            .await
            .ok()??;
    let seconds = value.value.as_u64()?;
    (seconds > 0).then(|| Duration::from_secs(seconds.max(60)))
}
async fn run(engine: Weak<Engine>, store: Store) {
    let mut tick = tokio::time::interval(Duration::from_secs(15));
    tick.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);
    let mut cursor = String::new();
    let mut jobs = tokio::task::JoinSet::new();
    loop {
        tick.tick().await;
        let Some(engine) = engine.upgrade() else {
            break;
        };
        while jobs.try_join_next().is_some() {}
        if jobs.len() >= 2 {
            continue;
        }
        let Ok(accounts) = store.active_nextcloud_pim_accounts(&cursor).await else {
            continue;
        };
        if accounts.is_empty() {
            cursor.clear();
            continue;
        }
        for account in accounts {
            if jobs.len() >= 2 {
                break;
            }
            cursor = account.clone();
            if !engine.is_registered(&account) {
                continue;
            }
            let Some(delay) = interval(&store, &account).await else {
                continue;
            };
            if !engine.pim_sync_due(&account, delay) {
                continue;
            }
            let engine = engine.clone();
            let store = store.clone();
            jobs.spawn(async move {
                crate::nextcloud_pim::install(&engine, &store, &account);
                // Every DAV operation rechecks capabilities, account, link and route.
                // Cancellation drops the shared sync guard and records failure backoff.
                let work = tokio::time::timeout(
                    Duration::from_secs(30),
                    engine.sync_current_pim(&account),
                );
                tokio::pin!(work);
                let mut admission = tokio::time::interval(Duration::from_secs(1));
                loop {
                    tokio::select! {
                        _ = &mut work => break,
                        _ = admission.tick() => {
                            if !engine.is_registered(&account) { break; }
                        }
                    }
                }
            });
        }
    }
    jobs.abort_all();
}
#[cfg(test)]
mod tests {
    use super::*;
    #[tokio::test]
    async fn an_unknown_account_never_gets_a_default_schedule() {
        let store = Store::open_in_memory(mw_store::ServerKey::from_bytes(&[31; 32]).unwrap())
            .await
            .unwrap();
        assert!(interval(&store, "unverified").await.is_none());
    }
    #[tokio::test]
    async fn zero_and_locked_domain_policy_stop_periodic_refresh() {
        use mw_store::settings_policy::{SettingScope, SettingValueRow};
        let store = Store::open_in_memory(mw_store::ServerKey::from_bytes(&[17; 32]).unwrap())
            .await
            .unwrap();
        store
            .record_auth_event(
                &mw_store::SigninEvent {
                    id: "verified".into(),
                    account_id: Some("account".into()),
                    username_attempted: "alice@example.org".into(),
                    ts: 1,
                    result: "success".into(),
                    method: "password".into(),
                    ip: String::new(),
                    user_agent: String::new(),
                    geo_country: None,
                    reason: "success".into(),
                },
                mw_store::AuthSurface::Webmail,
                None,
                mw_store::AuthLogLimits::default(),
            )
            .await
            .unwrap();
        assert_eq!(
            interval(&store, "account").await,
            Some(Duration::from_secs(300))
        );
        let row = SettingValueRow {
            scope: SettingScope::User,
            scope_id: "account".into(),
            key: "nextcloud.syncIntervalSecs".into(),
            value: "0".into(),
            locked: false,
            updated_by: "test".into(),
            updated_at: 1,
        };
        store.put_setting_value(&row).await.unwrap();
        assert_eq!(interval(&store, "account").await, None);
        store
            .put_setting_value(&SettingValueRow {
                scope: SettingScope::Domain,
                scope_id: "example.org".into(),
                value: "1".into(),
                locked: true,
                ..row
            })
            .await
            .unwrap();
        assert_eq!(
            interval(&store, "account").await,
            Some(Duration::from_secs(60))
        );
    }
}
