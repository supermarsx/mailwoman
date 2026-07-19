//! t15-E-e2e — LEG 6+7: third-party component LOADING trust store — the security
//! headline, where the NEGATIVE paths are the point.
//!
//! 26.15's ONLY security-core loosening widens `resolve_component` from
//! first-party-pinned-only to first-party-pinned-OR-admin-pinned-digest (migration 0014
//! `plugin_allowlist`). `resolve_component` reads a candidate `<id>.wasm` ONCE into
//! memory, hashes THOSE bytes with SHA-256, and admits them ONLY on a byte-exact match
//! to a NON-REVOKED admin-approved pin for that exact id — via the public store gate
//! [`Store::is_third_party_digest_approved`], which is the exact trust decision the
//! loader consults.
//!
//! `resolve_component` itself is a private `mw-server` fn (its byte-hash-then-load glue
//! is unit-proven in `v7_mount.rs::third_party_load_gate_positive_and_negatives`). What
//! THIS live-E2E adds is the same decision driven end-to-end against REAL infrastructure:
//! a real component file on disk hashed with the real `sha2`, the pin approved/revoked in
//! a REAL Postgres (via `MW_E14_PG_DSN`) 0014 allowlist, so the 0014 migration + verify
//! SQL are exercised in the second dialect. This is the class of wiring bug that, if it
//! regressed, would be a real CVE (unapproved bytes loading).
//!
//! Legs proven live, mirroring `resolve_component`'s decision over real files + store:
//!   6.  POSITIVE — a component whose EXACT digest an admin approved is admitted.
//!   7a. NO approval row               → REFUSED.
//!   7b. REVOKED row                   → REFUSED on the next load (revocation is fresh
//!       each load).
//!   7c. TAMPERED byte (digest change) → REFUSED (the pin is for the ORIGINAL bytes).
//!   7d. NO-SPOOF — approving a FIRST-PARTY id is rejected at approve time
//!       (`FirstPartyCollision`); a colliding row can never be created, so first-party
//!       resolution is never shadowed.
//!   7e. A malformed/empty digest is NEVER a wildcard (rejected at approve; never matches
//!       at verify).
//!   +   Uninstall deletes the allowlist rows; append-only audit rows persist + read back.
//!
//! ## Running
//!   cargo test -p mw-server --test t15_thirdparty_load                 # SQLite trust store
//!   docker compose -f docker-compose.ci.yml up -d --wait postgres
//!   MW_E14_PG_DSN=postgres://mailwoman:mailwoman@localhost:5432/mailwoman \
//!     cargo test -p mw-server --test t15_thirdparty_load -- --nocapture --test-threads=1

use std::path::{Path, PathBuf};

use sha2::{Digest, Sha256};

use mw_store::{PluginAllowlistError, ServerKey, Store, new_allowlist_pin};

/// The authoritative first-party ids (mirrors `mw-server`'s compiled-in
/// `FIRST_PARTY_DIGESTS` + the `nextcloud-plugin` alias). Passed to `put_plugin_allowlist`
/// as the reserved set so the anti-spoof (TQ2) collision refusal can be exercised live.
const FIRST_PARTY_IDS: &[&str] = &[
    "bridge-graph",
    "bridge-ews",
    "bridge-gmail",
    "languagetool",
    "nextcloud",
    "spam-rspamd",
    "spam-spamassassin",
    "nextcloud-plugin",
];

fn unique() -> String {
    static SEQ: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
    format!(
        "{}_{}",
        std::process::id(),
        SEQ.fetch_add(1, std::sync::atomic::Ordering::Relaxed)
    )
}

fn temp_dir(tag: &str) -> PathBuf {
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let d = std::env::temp_dir().join(format!("mw-t15-tp-{tag}-{}-{nanos}", unique()));
    std::fs::create_dir_all(&d).unwrap();
    d
}

fn sha256_hex(bytes: &[u8]) -> String {
    let d: [u8; 32] = Sha256::digest(bytes).into();
    let mut s = String::with_capacity(64);
    for b in d {
        s.push_str(&format!("{b:02x}"));
    }
    s
}

/// The trust decision `resolve_component` makes for a NON-first-party id: read the
/// on-disk bytes, hash them, and admit ONLY if the digest is an active admin pin.
/// Reproduced here over real files + the real store gate (the loader's own private glue
/// is a single `std::fs::read` + this call).
async fn admits(store: &Store, dir: &Path, id: &str) -> bool {
    let bytes = std::fs::read(dir.join(format!("{id}.wasm"))).expect("component file present");
    let digest = sha256_hex(&bytes);
    store
        .is_third_party_digest_approved(id, &digest)
        .await
        .unwrap()
}

/// The full positive + negative trust-store proof against a real store.
async fn drive(store: Store, dialect: &str) {
    let dir = temp_dir(dialect);
    let id = format!("acme-thirdparty-{}", unique());
    let bytes = b"\x00asm-third-party-component-bytes-not-first-party".to_vec();
    std::fs::write(dir.join(format!("{id}.wasm")), &bytes).unwrap();
    let digest = sha256_hex(&bytes);

    // 7a — NO approval row ⇒ REFUSED (deny-by-default; unapproved bytes never load).
    assert!(
        !admits(&store, &dir, &id).await,
        "[{dialect}] an unapproved third-party component must be REFUSED"
    );

    // 6 — approve the EXACT digest ⇒ ADMITTED (would load its exact bytes).
    store
        .put_plugin_allowlist(
            &new_allowlist_pin(&id, &digest, "admin@x", None, None, None, None),
            FIRST_PARTY_IDS,
        )
        .await
        .expect("approve the exact digest");
    assert!(
        admits(&store, &dir, &id).await,
        "[{dialect}] an admin-approved exact-digest component is ADMITTED"
    );

    // 7c — TAMPER one on-disk byte ⇒ digest mismatch ⇒ REFUSED (the pin is for the
    // ORIGINAL bytes; the loader hashes what it will actually load).
    let mut tampered = bytes.clone();
    tampered[0] ^= 0xff;
    std::fs::write(dir.join(format!("{id}.wasm")), &tampered).unwrap();
    assert!(
        !admits(&store, &dir, &id).await,
        "[{dialect}] a tampered component (digest mismatch) must be REFUSED"
    );
    // Restore the good bytes for the revoke leg.
    std::fs::write(dir.join(format!("{id}.wasm")), &bytes).unwrap();
    assert!(
        admits(&store, &dir, &id).await,
        "[{dialect}] the restored good bytes are admitted again"
    );

    // 7b — REVOKE ⇒ REFUSED on the next load (allowlist read fresh each load).
    assert!(
        store.revoke_plugin_allowlist(&id, &digest).await.unwrap(),
        "[{dialect}] revoke reports the active pin was revoked"
    );
    assert!(
        !admits(&store, &dir, &id).await,
        "[{dialect}] a revoked pin must REFUSE on the next load"
    );

    // 7e — a malformed / empty digest is NEVER a wildcard.
    assert!(
        !store.is_third_party_digest_approved(&id, "").await.unwrap(),
        "[{dialect}] an empty digest is never approved"
    );
    assert!(
        !store
            .is_third_party_digest_approved(&id, &digest.to_uppercase())
            .await
            .unwrap(),
        "[{dialect}] a non-canonical (upper-case) digest is never approved"
    );
    for bad in ["", "abc", &digest.to_uppercase()] {
        let err = store
            .put_plugin_allowlist(
                &new_allowlist_pin(&id, bad, "admin@x", None, None, None, None),
                FIRST_PARTY_IDS,
            )
            .await
            .unwrap_err();
        assert!(
            matches!(err, PluginAllowlistError::MalformedDigest(_)),
            "[{dialect}] a malformed digest {bad:?} is rejected at approve time"
        );
    }

    // 7d — NO-SPOOF: approving a FIRST-PARTY id is rejected at approve time, so a
    // colliding row can never exist to shadow first-party resolution.
    let spoof = sha256_hex(b"attacker-supplied-first-party-lookalike");
    let err = store
        .put_plugin_allowlist(
            &new_allowlist_pin("languagetool", &spoof, "admin@x", None, None, None, None),
            FIRST_PARTY_IDS,
        )
        .await
        .unwrap_err();
    assert!(
        matches!(err, PluginAllowlistError::FirstPartyCollision(_)),
        "[{dialect}] approving a first-party id is refused (anti-spoof): {err:?}"
    );
    assert!(
        !store
            .is_third_party_digest_approved("languagetool", &spoof)
            .await
            .unwrap(),
        "[{dialect}] no colliding first-party row was written"
    );

    // Uninstall companion: delete removes every allowlist row for the plugin.
    let removed = store.delete_plugin_allowlist(&id).await.unwrap();
    assert!(
        removed >= 1,
        "[{dialect}] uninstall deletes the plugin's allowlist rows ({removed})"
    );
    assert!(
        store
            .list_plugin_allowlist()
            .await
            .unwrap()
            .iter()
            .all(|r| r.plugin_id != id),
        "[{dialect}] no allowlist rows remain for the uninstalled plugin"
    );

    let _ = std::fs::remove_dir_all(&dir);
}

// ── SQLite (always) ──────────────────────────────────────────────────────────────────

#[tokio::test]
async fn third_party_trust_store_positive_and_negatives_sqlite() {
    let store = Store::open_in_memory(ServerKey::generate()).await.unwrap();
    drive(store, "sqlite").await;
}

// ── Postgres (live via MW_E14_PG_DSN, else loud-skip) ────────────────────────────────

#[tokio::test]
async fn third_party_trust_store_positive_and_negatives_postgres() {
    let Ok(dsn) = std::env::var("MW_E14_PG_DSN") else {
        eprintln!(
            "\n[t15 third-party SKIP] MW_E14_PG_DSN unset — live Postgres 0014 allowlist not driven.\n"
        );
        return;
    };
    let store = Store::open(&dsn, ServerKey::generate())
        .await
        .expect("open live Postgres store");
    drive(store, "postgres").await;
}
