//! 0026 egress-proxy route repository (t22-e12): additive, dual-backend `Store`
//! methods over `egress_proxy` (0026, both dialects).
//!
//! An egress route is deployment-wide OPERATOR configuration — the deliberate,
//! locally-resolved upstream proxy, as distinct from the ambient `HTTP_PROXY` that
//! 26.20 t22-e8 refuses everywhere. The password is a SECRET and lives sealed
//! (XChaCha20-Poly1305 under the store `ServerKey`), the same posture as
//! `bridge_oauth_tokens` and `ews_account_cred.sealed_cred`; the username is not a
//! secret and is stored in the clear.
//!
//! # The password must not leak through `Debug`
//! [`EgressProxyRow`]'s [`fmt::Debug`] is **hand-written to redact the password**. A
//! derived one would put the plaintext into every `tracing` event, panic message and
//! error body that formats a row — and this type is the one thing in the crate that
//! carries a proxy credential in memory. `mw_egress::proxy::ProxyAuth` redacts for
//! exactly this reason; the same care has to survive the trip through storage or the
//! redaction there is decorative.
//!
//! For contrast, `bridge_tokens::BridgeOauthTokenRow` DOES derive `Debug` over a
//! plaintext OAuth access token. That is pre-existing and outside this lane, but it
//! is the same hazard and worth closing separately.
//!
//! Authored in the SQLite `?n` style so it runs identically on SQLite or Postgres
//! through [`crate::backend`].

use std::fmt;

use chrono::Utc;

use crate::backend::q;
use crate::{Store, StoreError};

/// One configured egress route (0026). The password is held decrypted only in
/// memory; at rest it lives sealed. `None` means the route carries no credentials.
#[derive(Clone, PartialEq, Eq)]
pub struct EgressProxyRow {
    /// Stable identifier, used as the audit `target` and the route's cache key.
    pub id: String,
    /// `"http"` or `"socks5"`. Validated at the admin boundary, stored verbatim.
    pub scheme: String,
    /// Proxy hostname or IP literal. OPERATOR configuration — deliberately not
    /// subject to the SSRF address policy, since an egress proxy on RFC1918 is the
    /// normal deployment. Safe only while this can never become request-derived.
    pub host: String,
    pub port: u16,
    /// Proxy username. NOT a secret; stored and logged in the clear.
    pub username: String,
    /// Proxy password. `None` ⇒ the route is unauthenticated.
    pub password: Option<String>,
    /// Permit plaintext `http` origins through this route.
    pub allow_plaintext: bool,
    pub created_at: String,
    pub updated_at: String,
}

/// Hand-written so the password can never reach a log line, a panic message or an
/// error body. See the module docs — a derived `Debug` is the leak.
impl fmt::Debug for EgressProxyRow {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("EgressProxyRow")
            .field("id", &self.id)
            .field("scheme", &self.scheme)
            .field("host", &self.host)
            .field("port", &self.port)
            .field("username", &self.username)
            .field(
                "password",
                &self
                    .password
                    .as_ref()
                    .map(|_| "<redacted>")
                    .unwrap_or("None"),
            )
            .field("allow_plaintext", &self.allow_plaintext)
            .finish()
    }
}

impl Store {
    /// Insert or replace an egress route.
    ///
    /// # `password` means three different things, and that is the point
    /// This is the **write-only secret** contract, and it exists because **a UI that
    /// cannot read the secret cannot preserve it — only the server can.** The admin
    /// API never returns the password, so an edit form loads with the field blank and
    /// sends it only if the operator typed one. Writing whatever arrived would then
    /// mean *renaming a route wipes its credentials*, surfacing much later as an auth
    /// failure at fetch time with nothing connecting cause to effect.
    ///
    ///   * `None` ⇒ **keep** whatever is stored (the field was not submitted);
    ///   * `Some("")` ⇒ **clear** the credential (the operator emptied it deliberately);
    ///   * `Some(s)` ⇒ **set** it to `s`.
    ///
    /// `admin_sso.rs` states the same hazard in its own words — *"renaming/enabling a
    /// backend must not silently wipe its secret"*.
    ///
    /// Reading is total and round-trip safe against this: a route with no credential
    /// reads back as `None`, which writes back as keep-nothing; a route with one reads
    /// back as `Some(pw)`, which writes back the same value. A read never yields
    /// `Some("")`, so the clear arm is only ever reached deliberately.
    pub async fn put_egress_proxy(&self, row: &EgressProxyRow) -> Result<(), StoreError> {
        // `write_password = 0` leaves the stored value untouched on conflict. On a
        // fresh insert there is nothing to carry forward, so the sealed empty string
        // is what lands — no credentials, the same empty-means-absent convention as
        // 0018's `sealed_refresh_token`.
        let write_password = i64::from(row.password.is_some());
        let sealed = self
            .key
            .seal(row.password.as_deref().unwrap_or("").as_bytes())?;
        let now = Utc::now().to_rfc3339();
        q("INSERT INTO egress_proxy
                 (id, scheme, host, port, username, sealed_password, allow_plaintext, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)
             ON CONFLICT(id) DO UPDATE SET
                 scheme = excluded.scheme, host = excluded.host, port = excluded.port,
                 username = excluded.username,
                 sealed_password = CASE WHEN ?10 = 1
                     THEN excluded.sealed_password
                     ELSE egress_proxy.sealed_password END,
                 allow_plaintext = excluded.allow_plaintext, updated_at = excluded.updated_at")
            .bind(&row.id)
            .bind(&row.scheme)
            .bind(&row.host)
            .bind(i64::from(row.port))
            .bind(&row.username)
            .bind(sealed)
            .bind(i64::from(row.allow_plaintext))
            .bind(&now)
            .bind(&now)
            .bind(write_password)
            .execute(&self.backend)
            .await?;
        Ok(())
    }

    /// Every configured route, password unsealed into memory.
    pub async fn list_egress_proxies(&self) -> Result<Vec<EgressProxyRow>, StoreError> {
        let rows = q("SELECT id, scheme, host, port, username, sealed_password, allow_plaintext, created_at, updated_at
                      FROM egress_proxy ORDER BY id")
            .fetch_all(&self.backend)
            .await?;
        rows.iter().map(|r| self.decode_egress_row(r)).collect()
    }

    /// One route by id, password unsealed into memory.
    pub async fn get_egress_proxy(&self, id: &str) -> Result<Option<EgressProxyRow>, StoreError> {
        let row = q("SELECT id, scheme, host, port, username, sealed_password, allow_plaintext, created_at, updated_at
                     FROM egress_proxy WHERE id = ?1")
            .bind(id)
            .fetch_optional(&self.backend)
            .await?;
        row.as_ref().map(|r| self.decode_egress_row(r)).transpose()
    }

    /// Remove a route. Returns `true` when a row was actually deleted, so the caller
    /// can audit what happened rather than what it asked for.
    pub async fn delete_egress_proxy(&self, id: &str) -> Result<bool, StoreError> {
        let affected = q("DELETE FROM egress_proxy WHERE id = ?1")
            .bind(id)
            .execute(&self.backend)
            .await?;
        Ok(affected > 0)
    }

    fn decode_egress_row(&self, r: &crate::backend::Row) -> Result<EgressProxyRow, StoreError> {
        let plain = self.key.open(&r.get_blob("sealed_password"))?;
        let plain = String::from_utf8(plain)
            .map_err(|_| StoreError::Corrupt("egress proxy password decode".into()))?;
        Ok(EgressProxyRow {
            id: r.get_string("id"),
            scheme: r.get_string("scheme"),
            host: r.get_string("host"),
            // Stored as an integer in both dialects; a value outside the port range
            // means a corrupt row, not a port to guess at.
            port: u16::try_from(r.get_i64("port"))
                .map_err(|_| StoreError::Corrupt("egress proxy port out of range".into()))?,
            username: r.get_string("username"),
            // Empty ⇒ absent, mirroring how it was written.
            password: (!plain.is_empty()).then_some(plain),
            allow_plaintext: r.get_i64("allow_plaintext") != 0,
            created_at: r.get_string("created_at"),
            updated_at: r.get_string("updated_at"),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ServerKey;

    const PASSWORD: &str = "correct-horse-battery-staple";

    fn row() -> EgressProxyRow {
        EgressProxyRow {
            id: "corp".into(),
            scheme: "http".into(),
            host: "proxy.corp.example".into(),
            port: 3128,
            username: "svc-mail".into(),
            password: Some(PASSWORD.into()),
            allow_plaintext: false,
            created_at: String::new(),
            updated_at: String::new(),
        }
    }

    async fn store() -> Store {
        Store::open_in_memory(ServerKey::generate()).await.unwrap()
    }

    #[tokio::test]
    async fn round_trips_and_the_stored_column_is_not_plaintext() {
        let s = store().await;
        s.put_egress_proxy(&row()).await.unwrap();

        let got = s.get_egress_proxy("corp").await.unwrap().expect("present");
        assert_eq!(got.password.as_deref(), Some(PASSWORD));
        assert_eq!(got.username, "svc-mail");
        assert_eq!(got.port, 3128);
        assert!(!got.allow_plaintext);

        // The point of the column. Read the raw bytes back and assert the password
        // is not among them — asserting only that the round trip works would pass
        // just as happily against a table storing it in the clear.
        let raw = q("SELECT sealed_password FROM egress_proxy WHERE id = ?1")
            .bind("corp")
            .fetch_optional(&s.backend)
            .await
            .unwrap()
            .expect("row")
            .get_blob("sealed_password");
        assert!(
            !raw.is_empty(),
            "asserted before the negative so an empty column cannot pass as sealed"
        );
        assert!(
            !raw.windows(PASSWORD.len())
                .any(|w| w == PASSWORD.as_bytes()),
            "the plaintext password is present in the stored column"
        );
    }

    /// Read the raw stored bytes, so a test can tell a carried-forward value from a
    /// re-sealed one. Sealing is nonce-randomised, so re-sealing the SAME password
    /// produces DIFFERENT bytes — which is exactly what makes this able to prove
    /// carry-forward rather than merely "the column is still populated".
    async fn raw_sealed(s: &Store, id: &str) -> Vec<u8> {
        q("SELECT sealed_password FROM egress_proxy WHERE id = ?1")
            .bind(id)
            .fetch_optional(&s.backend)
            .await
            .unwrap()
            .expect("row")
            .get_blob("sealed_password")
    }

    #[tokio::test]
    async fn omitting_the_password_on_an_edit_preserves_the_stored_one() {
        // The write-only-secret contract. The admin API never returns the password,
        // so an edit form submits it only if the operator typed one. If an omitted
        // password overwrote the stored value, renaming a route would silently wipe
        // its credentials — an auth failure at fetch time, long after the edit, with
        // nothing connecting the two.
        let s = store().await;
        s.put_egress_proxy(&row()).await.unwrap();
        let before = raw_sealed(&s, "corp").await;

        // An unrelated edit: new host and username, password field absent.
        let mut edit = row();
        edit.password = None;
        edit.host = "proxy2.corp.example".into();
        edit.username = "svc-mail-2".into();
        s.put_egress_proxy(&edit).await.unwrap();

        let got = s.get_egress_proxy("corp").await.unwrap().expect("present");
        assert_eq!(
            got.host, "proxy2.corp.example",
            "the edit must have applied"
        );
        assert_eq!(got.username, "svc-mail-2");
        assert_eq!(
            got.password.as_deref(),
            Some(PASSWORD),
            "the credential was wiped by an edit that never mentioned it"
        );
        assert_eq!(
            raw_sealed(&s, "corp").await,
            before,
            "the stored bytes changed — the value was re-sealed rather than carried \
             forward, so this path is writing the password when it should not touch it"
        );
    }

    #[tokio::test]
    async fn an_explicitly_empty_password_clears_the_credential() {
        let s = store().await;
        s.put_egress_proxy(&row()).await.unwrap();
        let before = raw_sealed(&s, "corp").await;

        let mut edit = row();
        edit.password = Some(String::new());
        s.put_egress_proxy(&edit).await.unwrap();

        let got = s.get_egress_proxy("corp").await.unwrap().expect("present");
        assert_eq!(
            got.password, None,
            "an explicitly emptied password must clear the credential, not keep it — \
             otherwise a credential can be set but never removed"
        );
        assert_ne!(
            raw_sealed(&s, "corp").await,
            before,
            "the column must actually have been rewritten"
        );
    }

    #[tokio::test]
    async fn a_new_password_replaces_the_stored_one() {
        let s = store().await;
        s.put_egress_proxy(&row()).await.unwrap();
        let mut edit = row();
        edit.password = Some("a-different-secret".into());
        s.put_egress_proxy(&edit).await.unwrap();
        let got = s.get_egress_proxy("corp").await.unwrap().expect("present");
        assert_eq!(got.password.as_deref(), Some("a-different-secret"));
    }

    #[tokio::test]
    async fn a_route_without_credentials_decodes_to_none() {
        let s = store().await;
        let mut r = row();
        r.password = None;
        r.username = String::new();
        s.put_egress_proxy(&r).await.unwrap();
        let got = s.get_egress_proxy("corp").await.unwrap().expect("present");
        assert_eq!(got.password, None, "empty sealed value must mean absent");
    }

    #[tokio::test]
    async fn debug_never_renders_the_password() {
        // The hazard this type's hand-written `Debug` exists to close: one
        // `tracing::debug!("{row:?}")` anywhere would otherwise publish the
        // credential to logs, panic messages and error bodies alike.
        let rendered = format!("{:?}", row());
        assert!(
            !rendered.contains(PASSWORD),
            "Debug leaked the password: {rendered}"
        );
        assert!(
            rendered.contains("<redacted>"),
            "the password field must still be visible as redacted, not silently \
             omitted — an absent field reads as 'this type holds no secret': {rendered}"
        );
        assert!(
            rendered.contains("svc-mail"),
            "the username is not a secret and must stay legible for debugging"
        );
    }

    #[tokio::test]
    async fn delete_reports_whether_a_row_actually_went() {
        let s = store().await;
        s.put_egress_proxy(&row()).await.unwrap();
        assert!(s.delete_egress_proxy("corp").await.unwrap());
        assert!(
            !s.delete_egress_proxy("corp").await.unwrap(),
            "a second delete removed nothing and must say so, or the audit row \
             records what the caller asked for rather than what happened"
        );
        assert!(s.get_egress_proxy("corp").await.unwrap().is_none());
    }
}
