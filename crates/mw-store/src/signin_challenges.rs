//! Short-lived, deployment-wide single-use sign-in challenges. No account identifiers.
use crate::{Store, StoreError, backend::q};
use sha2::{Digest, Sha256};

impl Store {
    /// Purpose-separated signing material; never log or expose through configuration APIs.
    pub fn signin_challenge_secret(&self, purpose: &str) -> String {
        let mut hash = Sha256::new();
        hash.update(b"mailwoman:signin-challenge:v1\0");
        hash.update(self.key.to_hex().as_bytes());
        hash.update(b"\0");
        hash.update(purpose.as_bytes());
        hash.finalize()
            .iter()
            .map(|byte| format!("{byte:02x}"))
            .collect()
    }

    /// Admit an issued nonce under a global transaction lock. Expired rows are the
    /// only rows evicted: a full table refuses new work, never makes a used proof valid.
    pub async fn issue_signin_challenge(
        &self,
        nonce: &str,
        surface: &str,
        peer: &str,
        now: i64,
        expires: i64,
    ) -> Result<bool, StoreError> {
        if nonce.len() > 128
            || peer.len() != 64
            || !matches!(surface, "mailbox" | "admin")
            || expires <= now
            || expires > now.saturating_add(600)
        {
            return Ok(false);
        }
        let mut tx = self.backend.begin().await?;
        q("INSERT INTO settings(key,value) VALUES('signin-challenge:lock','0') ON CONFLICT(key) DO NOTHING").execute_tx(&mut tx).await?;
        q("UPDATE settings SET value=value WHERE key='signin-challenge:lock'")
            .execute_tx(&mut tx)
            .await?;
        q("DELETE FROM signin_challenge WHERE expires_at<=?1")
            .bind(now)
            .execute_tx(&mut tx)
            .await?;
        let count = q("SELECT COUNT(*) AS total FROM signin_challenge")
            .fetch_all_tx(&mut tx)
            .await?[0]
            .get_i64("total");
        let peer_count = q("SELECT COUNT(*) AS total FROM signin_challenge WHERE peer_key=?1")
            .bind(peer)
            .fetch_all_tx(&mut tx)
            .await?[0]
            .get_i64("total");
        if count >= 10_000 || peer_count >= 8 {
            tx.commit().await?;
            return Ok(false);
        }
        let inserted=q("INSERT INTO signin_challenge(nonce,surface,peer_key,expires_at) VALUES(?1,?2,?3,?4) ON CONFLICT(nonce) DO NOTHING").bind(nonce).bind(surface).bind(peer).bind(expires).execute_tx(&mut tx).await?;
        tx.commit().await?;
        Ok(inserted == 1)
    }

    /// Atomic across connections and replicas; absence and exact-expiry both refuse.
    pub async fn consume_signin_challenge(
        &self,
        nonce: &str,
        surface: &str,
        peer: &str,
        now: i64,
    ) -> Result<bool, StoreError> {
        Ok(q("DELETE FROM signin_challenge WHERE nonce=?1 AND surface=?2 AND peer_key=?3 AND expires_at>?4").bind(nonce).bind(surface).bind(peer).bind(now).execute(&self.backend).await? == 1)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[tokio::test]
    async fn global_capacity_refuses_instead_of_evicting_live_proofs() {
        let store = Store::open_in_memory(crate::ServerKey::generate())
            .await
            .unwrap();
        q("WITH RECURSIVE seq(n) AS (SELECT 1 UNION ALL SELECT n+1 FROM seq WHERE n<10000) INSERT INTO signin_challenge(nonce,surface,peer_key,expires_at) SELECT CAST(n AS TEXT),'mailbox',?1,200 FROM seq")
            .bind("a".repeat(64)).execute(&store.backend).await.unwrap();
        assert!(
            !store
                .issue_signin_challenge("full", "admin", &"b".repeat(64), 100, 160)
                .await
                .unwrap()
        );
        assert!(
            store
                .consume_signin_challenge("1", "mailbox", &"a".repeat(64), 101)
                .await
                .unwrap()
        );
        assert!(
            store
                .issue_signin_challenge("slot", "admin", &"b".repeat(64), 101, 161)
                .await
                .unwrap()
        );
        assert!(
            store
                .issue_signin_challenge("expired-pruned", "admin", &"b".repeat(64), 200, 260)
                .await
                .unwrap()
        );
    }
}
