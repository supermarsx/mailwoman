//! Keyring family (`CryptoKey/*`, frozen §2.2) — server-side, PUBLIC keys +
//! opaque client-encrypted backup ONLY. `set` uploads the PUBLIC key/cert plus an
//! opaque `encryptedPrivateBackup` (the server NEVER decrypts it, plan §1.2 /
//! risk #4). `lookup` orchestrates WKD/VKS (with consent) + cert-harvest from
//! stored signed mail; `setTrust` is the TOFU verify/revoke.
//!
//! e0 skeleton — the frozen arms with `todo!()` bodies. e6 fills them over the
//! `crypto_keys` / `key_associations` store tables + `mw-crypto` native harvest.

use serde_json::Value;

use crate::account::AccountRuntime;
use crate::engine::Engine;

impl Engine {
    /// `CryptoKey/get {ids?}` → `{accountId,state,list:[CryptoKey],notFound}`.
    pub(crate) async fn crypto_key_get(&self, _account_id: &str, _args: &Value) -> Value {
        todo!("e6: read own + harvested/contact keys from crypto_keys")
    }

    /// `CryptoKey/set` — upload the PUBLIC key/cert + opaque `encryptedPrivateBackup`
    /// (never plaintext private material, plan §1.2).
    pub(crate) async fn crypto_key_set(&self, _account_id: &str, _args: &Value) -> Value {
        todo!("e6: persist public key + opaque backup; record crypto_changes")
    }

    /// `CryptoKey/query` → the account's keys (own + contact), filtered/sorted.
    pub(crate) async fn crypto_key_query(&self, _account_id: &str, _args: &Value) -> Value {
        todo!("e6: query crypto_keys")
    }

    /// `CryptoKey/lookup {address, sources:["wkd","vks","autocrypt","harvested"]}`
    /// → `{list:[CryptoKey], notFound}` (WKD/VKS with consent + cert harvest).
    pub(crate) async fn crypto_key_lookup(
        &self,
        _account_id: &str,
        _rt: &AccountRuntime,
        _args: &Value,
    ) -> Value {
        todo!("e6: WKD/VKS lookup (consent) + harvest from stored mail")
    }

    /// `CryptoKey/setTrust {id, trust}` — TOFU verify/revoke.
    pub(crate) async fn crypto_key_set_trust(&self, _account_id: &str, _args: &Value) -> Value {
        todo!("e6: update crypto_keys.trust + key_associations TOFU history")
    }

    /// `MailRule/get {ids?}` → the block/silence/ignore surface over `rules.rs`
    /// (frozen §2.2). Also exposes the Sieve round-trip for power users.
    pub(crate) async fn mail_rule_get(&self, _account_id: &str, _args: &Value) -> Value {
        todo!("e6: project rules.rs get_rules() → [MailRule]")
    }

    /// `MailRule/set` — create/update/destroy mail rules (materializes as Sieve
    /// where advertised, engine-applied otherwise, plan §1.9).
    pub(crate) async fn mail_rule_set(
        &self,
        _account_id: &str,
        _rt: &AccountRuntime,
        _args: &Value,
    ) -> Value {
        todo!("e6: rules.rs set_rules() + Sieve codegen; record crypto_changes")
    }
}
