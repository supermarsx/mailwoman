use super::*;

fn ip(s: &str) -> IpAddr {
    s.parse().unwrap()
}
const DAY: i64 = 86_400;
const SALT: [u8; 32] = [7; 32];
const KEY: [u8; 32] = [9; 32];

fn table(rows: &[(&str, i64)]) -> KnownStore {
    let mut k = KnownStore::from_parts(SALT, HashMap::new(), HashMap::new(), 0);
    for (a, t) in rows {
        k.record_success(ip(a), Some("PT"), "user", *t, 100 * DAY, 1000);
    }
    k
}
fn sealed(k: &KnownStore, now: i64, ttl: i64, cap: usize) -> String {
    seal(&k.snap(now, ttl, cap), &KEY)
}
fn reload(raw: &str, now: i64, ttl: i64, cap: usize) -> KnownStore {
    let mut fresh = KnownStore::from_parts(SALT, HashMap::new(), HashMap::new(), 0);
    let r = open(raw, &KEY).expect("valid snapshot");
    fresh.merge_restored(r, now, ttl, cap);
    fresh
}

#[test]
fn a_snapshot_round_trips_to_the_same_tiers() {
    let now = 1_000_000;
    let k = table(&[("203.0.113.5", now - 10), ("2001:db8:1:2::1", now - 20)]);
    let again = reload(&sealed(&k, now, 30 * DAY, 100), now, 30 * DAY, 100);
    assert!(again.is_known(ip("203.0.113.5"), now, 30 * DAY));
    // IPv6 is still per /64.
    assert!(again.is_known(ip("2001:db8:1:2:ffff::9"), now, 30 * DAY));
    assert!(!again.is_known(ip("2001:db8:1:3::1"), now, 30 * DAY));
    assert!(!again.is_known(ip("203.0.113.6"), now, 30 * DAY));
    assert!(again.country_known("PT", now, 30 * DAY));
    assert_eq!(again.account_count(ip("203.0.113.5")), 1);
}

#[test]
fn the_stored_text_holds_no_address_or_account_name() {
    let k = table(&[("203.0.113.5", 1000), ("2001:db8:1:2::1", 1000)]);
    let raw = sealed(&k, 1000, DAY, 10);
    for needle in ["203.0.113", "2001:db8", "user"] {
        assert!(!raw.contains(needle), "{needle}");
    }
}

#[test]
fn a_tampered_truncated_or_foreign_snapshot_is_rejected() {
    let k = table(&[("203.0.113.5", 1000)]);
    let raw = sealed(&k, 1000, DAY, 10);
    // Edit the payload: the country.
    let edited = raw.replace("\"PT\"", "\"RU\"");
    assert_ne!(edited, raw);
    assert_eq!(open(&edited, &KEY).err(), Some(Reject::Mac));
    // Edit the MAC.
    let mut bad_mac = raw.clone();
    bad_mac.replace_range(PREFIX_V1.len()..PREFIX_V1.len() + 1, if &raw[PREFIX_V1.len()..PREFIX_V1.len() + 1] == "0" { "1" } else { "0" });
    assert_eq!(open(&bad_mac, &KEY).err(), Some(Reject::Mac));
    // Cut off.
    assert!(open(&raw[..raw.len() - 5], &KEY).is_err());
    // Another server key.
    assert_eq!(open(&raw, &[1; 32]).err(), Some(Reject::Mac));
    // Not ours.
    assert_eq!(open("{\"entries\":[]}", &KEY).err(), Some(Reject::Corrupt));
    assert_eq!(open("   ", &KEY).err(), Some(Reject::Empty));
    assert_eq!(open("mw-tt1.zz.{}", &KEY).err(), Some(Reject::Corrupt));
}

#[test]
fn an_unknown_version_is_ignored_even_with_a_valid_mac() {
    let mut snap = table(&[("203.0.113.5", 1000)]).snap(1000, DAY, 10);
    snap.v = 2;
    assert_eq!(open(&seal(&snap, &KEY), &KEY).err(), Some(Reject::Version));
    let future = format!("mw-tt2.{}.{{}}", "0".repeat(64));
    assert_eq!(open(&future, &KEY).err(), Some(Reject::Version));
}

#[test]
fn an_oversized_snapshot_is_rejected() {
    let mut snap = table(&[("203.0.113.5", 1000)]).snap(1000, DAY, 10);
    let one = snap.entries.remove(0);
    for i in 0..=MAX_ENTRIES {
        snap.entries.push(SnapEntry {
            k: format!("{i:032x}"),
            c: one.c.clone(),
            t: one.t,
            a: vec![],
        });
    }
    assert_eq!(open(&seal(&snap, &KEY), &KEY).err(), Some(Reject::TooLarge));
}

#[test]
fn entries_past_the_window_are_dropped_at_save_and_at_load() {
    let now = 100 * DAY;
    let k = table(&[("192.0.2.1", now - 40 * DAY), ("192.0.2.2", now - DAY)]);
    // At save.
    let saved = k.snap(now, 30 * DAY, 100);
    assert_eq!(saved.entries.len(), 1);
    // At load: a snapshot written with a wide window, loaded with a narrow one.
    let wide = sealed(&k, now, 100 * DAY, 100);
    let narrow = reload(&wide, now, 30 * DAY, 100);
    assert!(!narrow.is_known(ip("192.0.2.1"), now, 100 * DAY));
    assert!(narrow.is_known(ip("192.0.2.2"), now, 30 * DAY));
    // And later, once the window has passed for everything.
    let later = reload(&wide, now + 60 * DAY, 30 * DAY, 100);
    assert_eq!(later.len(), 0);
    assert!(!later.country_known("PT", now + 60 * DAY, 30 * DAY));
}

#[test]
fn the_capacity_keeps_the_newest_entries() {
    let now = 5000;
    let rows: Vec<(String, i64)> = (0..20).map(|i| (format!("192.0.2.{i}"), now - 100 + i)).collect();
    let refs: Vec<(&str, i64)> = rows.iter().map(|(a, t)| (a.as_str(), *t)).collect();
    let k = table(&refs);
    let raw = sealed(&k, now, DAY, 5);
    let again = reload(&raw, now, DAY, 5);
    assert_eq!(again.len(), 5);
    assert!(again.is_known(ip("192.0.2.19"), now, DAY));
    assert!(!again.is_known(ip("192.0.2.0"), now, DAY));
    // A larger snapshot loaded under a smaller capacity is cut the same way.
    let wide = reload(&sealed(&k, now, DAY, 20), now, DAY, 3);
    assert_eq!(wide.len(), 3);
    assert!(wide.is_known(ip("192.0.2.19"), now, DAY));
}

#[test]
fn loading_merges_with_the_auth_log_rebuild_by_taking_the_latest_time() {
    let now = 10_000;
    let saved = table(&[("192.0.2.1", now - 500), ("192.0.2.2", now - 50)]);
    let raw = sealed(&saved, now, DAY, 100);
    // The rebuild already holds a newer .1 and an account the snapshot lacks.
    let mut rebuilt = KnownStore::from_parts(SALT, HashMap::new(), HashMap::new(), 0);
    rebuilt.record_success(ip("192.0.2.1"), Some("DE"), "other", now - 5, DAY, 100);
    rebuilt.merge_restored(open(&raw, &KEY).unwrap(), now, DAY, 100);
    let key = rebuilt.key(ip("192.0.2.1"));
    assert_eq!(rebuilt.entries[&key].last_seen, now - 5);
    assert_eq!(rebuilt.entries[&key].country.as_deref(), Some("DE"));
    assert_eq!(rebuilt.account_count(ip("192.0.2.1")), 2);
    assert!(rebuilt.is_known(ip("192.0.2.2"), now, DAY));
    // The other direction: an older rebuilt row takes the snapshot's time.
    let mut older = KnownStore::from_parts(SALT, HashMap::new(), HashMap::new(), 0);
    older.record_success(ip("192.0.2.2"), None, "user", now - 900, DAY, 100);
    older.merge_restored(open(&raw, &KEY).unwrap(), now, DAY, 100);
    let key = older.key(ip("192.0.2.2"));
    assert_eq!(older.entries[&key].last_seen, now - 50);
    assert_eq!(older.entries[&key].country.as_deref(), Some("PT"));
    assert_eq!(older.entries[&key].accounts.len(), 1);
}

#[test]
fn a_forgotten_time_discards_older_entries_everywhere() {
    let now = 10_000;
    let raw = sealed(&table(&[("192.0.2.1", now - 500)]), now, DAY, 100);
    let mut k = KnownStore::from_parts(SALT, HashMap::new(), HashMap::new(), now - 100);
    assert_eq!(k.merge_restored(open(&raw, &KEY).unwrap(), now, DAY, 100), 0);
    assert_eq!(k.len(), 0);
    // A snapshot that carries the cut-off passes it on.
    let cleared = KnownStore::from_parts(SALT, HashMap::new(), HashMap::new(), now - 100);
    let mut other = table(&[("192.0.2.1", now - 500), ("192.0.2.9", now - 10)]);
    other.merge_restored(
        open(&sealed(&cleared, now, DAY, 10), &KEY).unwrap(),
        now,
        DAY,
        100,
    );
    assert_eq!(other.len(), 1);
    assert!(other.is_known(ip("192.0.2.9"), now, DAY));
}

#[test]
fn the_save_interval_is_clamped() {
    assert_eq!(clamp_interval(0), 30);
    assert_eq!(clamp_interval(29), 30);
    assert_eq!(clamp_interval(300), 300);
    assert_eq!(clamp_interval(1_000_000), 3600);
}

#[test]
fn hashes_are_stable_for_one_server_key_and_differ_across_keys() {
    let a = KnownStore::from_parts(SALT, HashMap::new(), HashMap::new(), 0);
    let b = KnownStore::from_parts(SALT, HashMap::new(), HashMap::new(), 0);
    let c = KnownStore::from_parts([8; 32], HashMap::new(), HashMap::new(), 0);
    assert_eq!(a.key(ip("203.0.113.5")), b.key(ip("203.0.113.5")));
    assert_ne!(a.key(ip("203.0.113.5")), c.key(ip("203.0.113.5")));
    // The process-random salt of `new()` is not what a store-bound table uses.
    assert_ne!(a.key(ip("203.0.113.5")), KnownStore::new().key(ip("203.0.113.5")));
}

async fn fresh_store(key: &mw_store::ServerKey) -> mw_store::Store {
    mw_store::Store::open_in_memory(key.clone()).await.unwrap()
}
/// Drop the in-memory table the way a restart does, then adopt the store again.
fn restart(store: &mw_store::Store) {
    *lock() = KnownStore::new();
    lock().adopt(store.instance_id(), salt(store));
}

#[tokio::test]
async fn a_store_backed_save_survives_a_restart_and_only_writes_when_changed() {
    let key = mw_store::ServerKey::generate();
    let store = fresh_store(&key).await;
    let policy = policy(&store).await;
    assert!(policy.persist);
    // Hashes come from the server key, so a second store with the same key agrees.
    let twin = fresh_store(&key).await;
    assert_eq!(salt(&store), salt(&twin));
    assert_ne!(salt(&store), salt(&fresh_store(&mw_store::ServerKey::generate()).await));

    // Nothing changed yet: no write.
    assert_eq!(save(&store, false).await, Ok(Saved::Skipped));
    assert_eq!(store.get_setting(SETTING_KEY).await.unwrap(), None);

    record_success(ip("203.0.113.77"), "someone");
    record_success(ip("2001:db8:aa:bb::5"), "someone");
    assert_eq!(save(&store, false).await, Ok(Saved::Written(2)));
    let first = store.get_setting(SETTING_KEY).await.unwrap().unwrap();
    assert!(first.starts_with(PREFIX_V1));
    assert!(status().last_saved.is_some());
    // Unchanged again: skipped, row untouched.
    assert_eq!(save(&store, false).await, Ok(Saved::Skipped));
    assert_eq!(store.get_setting(SETTING_KEY).await.unwrap().unwrap(), first);

    // "Restart": a cold table, then the saved copy is loaded.
    restart(&store);
    assert!(!is_known(ip("203.0.113.77")));
    restore(&store, &policy).await;
    assert!(is_known(ip("203.0.113.77")));
    assert!(is_known(ip("2001:db8:aa:bb:1::9")));
    assert!(!is_known(ip("203.0.113.78")));
    assert_eq!(status().restored_at_startup, 2);

    // A saved copy from another server key is ignored; startup is unaffected.
    let other = fresh_store(&mw_store::ServerKey::generate()).await;
    other.set_setting(SETTING_KEY, &first).await.unwrap();
    restart(&other);
    restore(&other, &policy).await;
    assert_eq!(status().entries, 0);
    // Garbage in the row is ignored too.
    other.set_setting(SETTING_KEY, "not a snapshot").await.unwrap();
    restore(&other, &policy).await;
    assert_eq!(status().entries, 0);
}

#[tokio::test]
async fn forgetting_clears_memory_and_the_saved_copy() {
    let key = mw_store::ServerKey::generate();
    let store = fresh_store(&key).await;
    let policy = policy(&store).await;
    record_success(ip("198.51.100.30"), "someone");
    assert_eq!(save(&store, false).await, Ok(Saved::Written(1)));
    forget_all(&store).await.unwrap();
    assert!(!is_known(ip("198.51.100.30")));
    restart(&store);
    restore(&store, &policy).await;
    assert!(!is_known(ip("198.51.100.30")));
    assert_eq!(status().entries, 0);
}

#[tokio::test]
async fn persistence_off_writes_nothing_and_loads_nothing() {
    let key = mw_store::ServerKey::generate();
    let store = fresh_store(&key).await;
    let on = policy(&store).await;
    record_success(ip("198.51.100.40"), "someone");
    assert_eq!(save(&store, false).await, Ok(Saved::Written(1)));
    // Turn it off the way a settings change does: the cached policy.
    let off = Arc::new(Policy {
        persist: false,
        ..(*on).clone()
    });
    *cache().write().unwrap() = Some(Cached {
        at: Instant::now(),
        store: store.instance_id(),
        value: off.clone(),
    });
    record_success(ip("198.51.100.41"), "someone");
    let before = store.get_setting(SETTING_KEY).await.unwrap();
    assert_eq!(save(&store, true).await, Ok(Saved::Skipped));
    assert_eq!(store.get_setting(SETTING_KEY).await.unwrap(), before);
    restart(&store);
    restore(&store, &off).await;
    assert_eq!(status().entries, 0);
    assert_eq!(status().restored_at_startup, 0);
    // Put the policy back for the other tests.
    *cache().write().unwrap() = Some(Cached {
        at: Instant::now(),
        store: store.instance_id(),
        value: on,
    });
}

#[tokio::test]
async fn a_table_that_belongs_to_another_store_is_never_written() {
    let store = fresh_store(&mw_store::ServerKey::generate()).await;
    let stranger = fresh_store(&mw_store::ServerKey::generate()).await;
    let _ = policy(&store).await;
    record_success(ip("198.51.100.50"), "someone");
    assert_eq!(save(&stranger, true).await, Ok(Saved::Skipped));
    assert_eq!(stranger.get_setting(SETTING_KEY).await.unwrap(), None);
    // The failure path (the store refusing the write) re-marks the table dirty so
    // the next interval retries; lookups never depend on it.
    assert!(is_known(ip("198.51.100.50")));
    assert!(!IN_FLIGHT.load(Ordering::Acquire));
}
