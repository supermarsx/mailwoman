//! PQC-hybrid wrap of the `mw-store` seal master key at rest (plan §1.7 / §2.4).
//! Crypto-agility groundwork toward the V6 zero-access hierarchy — NOT a
//! user-facing security claim (ml-kem is unaudited, plan §6#8). The seal key is
//! wrapped with the hybrid **X25519 + ML-KEM-768** primitive from `mw-crypto`
//! (`native::wrap_store_key`) and persisted in `store_key_material` with the
//! algorithm-suite tag [`mw_crypto::STORE_KEY_WRAP_SUITE`]; the recipient secret
//! is returned to the caller to safeguard (an HSM/env master in a real deploy).

use crate::engine::Engine;

impl Engine {
    /// Wrap `seal_key` under a fresh hybrid recipient and persist the wrapped blob
    /// with its suite tag in `store_key_material`. Returns the recipient key pair —
    /// the deployment safeguards `secret` (needed to unwrap); only the wrapped key
    /// is stored at rest.
    pub async fn pqc_wrap_store_seal(
        &self,
        seal_key: &[u8],
    ) -> Result<mw_crypto::pqc::HybridKeyPair, String> {
        let recipient = mw_crypto::native::generate_store_recipient();
        let wrapped = mw_crypto::native::wrap_store_key(seal_key, &recipient.public)
            .map_err(|e| e.to_string())?;
        self.store()
            .upsert_store_key_material(&mw_store::StoreKeyMaterialRow {
                id: "seal".into(),
                wrapped_seal_key: wrapped,
                suite: mw_crypto::STORE_KEY_WRAP_SUITE.to_string(),
                created_at: chrono::Utc::now().to_rfc3339(),
            })
            .await
            .map_err(|e| e.to_string())?;
        Ok(recipient)
    }

    /// Unwrap the persisted PQC-wrapped seal key using `recipient_secret`.
    pub async fn pqc_unwrap_store_seal(&self, recipient_secret: &[u8]) -> Result<Vec<u8>, String> {
        let row = self
            .store()
            .get_store_key_material()
            .await
            .map_err(|e| e.to_string())?
            .ok_or_else(|| "no wrapped seal key persisted".to_string())?;
        mw_crypto::native::unwrap_store_key(&row.wrapped_seal_key, recipient_secret)
            .map_err(|e| e.to_string())
    }
}
