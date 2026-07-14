//! Session-scoped synthetic-UID map. EWS identifies items by opaque `ItemId`s, but
//! the engine (via the `mw-plugin` host adapter) only accepts `MessageRef::Imap`
//! coordinates (mailbox + uidvalidity + `uid`). The bridge therefore mints a stable,
//! monotonically-increasing `uid` per item at sync time and remembers the
//! `uid → (ItemId, ChangeKey)` mapping so `fetch_raw`/`store_flags`/`move` can turn
//! an engine ref back into the real EWS `ItemId`.
//!
//! The `mw-plugin` account-backend adapter keeps ONE instance per account and
//! reuses it across calls (`crates/mw-plugin/src/adapter.rs`), so this process-wide
//! map persists for the life of that session — `sync_mailbox` populates it and the
//! subsequent `fetch_raw` reads it. (A production bridge additionally mirrors the
//! map into the scoped host KV so it survives an instance recycle; that is a
//! `store:kv-scoped` grant concern wired at mount, out of scope for the fixtures.)

use std::collections::HashMap;
use std::sync::{LazyLock, Mutex};

#[derive(Default)]
struct ItemMap {
    next_uid: u32,
    by_uid: HashMap<u32, (String, String)>,
    by_item: HashMap<String, u32>,
}

static MAP: LazyLock<Mutex<ItemMap>> = LazyLock::new(|| Mutex::new(ItemMap::default()));

/// Assign (or return the existing) stable `uid` for an EWS `(item_id, change_key)`.
#[must_use]
pub fn assign_uid(item_id: &str, change_key: &str) -> u32 {
    let mut m = MAP.lock().expect("item-map poisoned");
    if let Some(&uid) = m.by_item.get(item_id) {
        // Refresh the change key (it advances as the item is modified).
        m.by_uid
            .insert(uid, (item_id.to_string(), change_key.to_string()));
        return uid;
    }
    m.next_uid += 1;
    let uid = m.next_uid;
    m.by_uid
        .insert(uid, (item_id.to_string(), change_key.to_string()));
    m.by_item.insert(item_id.to_string(), uid);
    uid
}

/// The `(item_id, change_key)` previously assigned to `uid`, if known this session.
#[must_use]
pub fn lookup_item(uid: u32) -> Option<(String, String)> {
    MAP.lock()
        .expect("item-map poisoned")
        .by_uid
        .get(&uid)
        .cloned()
}

/// The `uid` previously assigned to an EWS `item_id`, if known this session.
#[must_use]
pub fn uid_for(item_id: &str) -> Option<u32> {
    MAP.lock()
        .expect("item-map poisoned")
        .by_item
        .get(item_id)
        .copied()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn uids_are_stable_and_reversible() {
        let a = assign_uid("ITEM-1", "CK1");
        let b = assign_uid("ITEM-2", "CK2");
        assert_ne!(a, b);
        // Same item id ⇒ same uid (change key refreshed).
        assert_eq!(assign_uid("ITEM-1", "CK1b"), a);
        assert_eq!(lookup_item(a).unwrap().0, "ITEM-1");
        assert_eq!(lookup_item(a).unwrap().1, "CK1b");
        assert_eq!(uid_for("ITEM-2"), Some(b));
        assert_eq!(uid_for("nope"), None);
    }
}
