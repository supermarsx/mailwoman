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
//! # 0033 route policy (t32 e31b)
//! Migration 0033 added the route kind (`http` | `socks5` | `socks5h` | `ssh`),
//! TLS to the proxy, remote DNS and its recorded acceptance, the SSH tunnel and
//! the chained second hop, the scope (`global` | `domain:<name>`) and the traffic
//! classes the route carries. [`Store::put_egress_proxy`] writes all of them except
//! the remote-DNS acceptance, which only [`Store::accept_egress_remote_dns_risk`]
//! records, and which an edit that changes what was accepted clears. The internal
//! destination allowlist (`egress_internal_allow`) lives here too.
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
    /// Route kind (0033): `"http"`, `"socks5"`, `"socks5h"` or `"ssh"`. Validated
    /// at the admin boundary and by 0033's CHECK. The 0026 `scheme` column is
    /// written alongside it (see `legacy_scheme`) until that column is dropped.
    pub kind: String,
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
    /// Whether this is **the** live route. At most one row may have this set, by a
    /// partial unique index in 0027 — see [`Store::active_egress_proxy`] for why that
    /// is a security property rather than a simplification.
    pub active: bool,
    /// `http` kind only: speak TLS to the proxy itself, so the proxy credential is
    /// not sent in the clear.
    pub tls_to_proxy: bool,
    /// The proxy resolves origin names (`socks5h`, or SSH by host name). Has no
    /// effect until an admin has accepted the risk
    /// ([`Self::remote_dns_risk_accepted_by`]).
    pub remote_dns: bool,
    /// Who accepted that hosts reachable from the proxy's network can be fetched.
    /// Written only by [`Store::accept_egress_remote_dns_risk`]; cleared by an edit
    /// that changes the kind, host, port or remote-DNS setting.
    pub remote_dns_risk_accepted_by: Option<String>,
    /// When, in unix seconds.
    pub remote_dns_risk_accepted_at: Option<i64>,
    /// `ssh` kind: the `ssh_tunnel` row whose channels carry the traffic.
    pub tunnel_id: Option<String>,
    /// The route used as the second hop through this one (`ssh` then `http` or
    /// `socks5`).
    pub chain_next_id: Option<String>,
    /// `global` or `domain:<name>`.
    pub scope: String,
    /// The traffic classes this route carries, by their storage spelling
    /// (`remote_content`, `ics_feeds`, ...). A class no live route lists goes direct.
    pub traffic_classes: Vec<String>,
    pub created_at: String,
    pub updated_at: String,
}

/// The value written to the 0026 `scheme` column for a route kind. `scheme` was
/// `http` | `socks5`; a `socks5h` route is a SOCKS5 proxy, and an `ssh` route has no
/// proxy scheme, so it is written as itself.
fn legacy_scheme(kind: &str) -> &str {
    match kind {
        "socks5h" => "socks5",
        other => other,
    }
}

/// One internal-destination allowlist entry (0033 `egress_internal_allow`).
/// `traffic_class` is `webhooks` or `siem` (0033's CHECK); `destination` is the
/// canonical spelling of `mw_egress::AllowDestination`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EgressAllowRow {
    pub id: String,
    pub traffic_class: String,
    pub destination: String,
    pub note: String,
    pub created_by: String,
    /// Unix seconds.
    pub created_at: i64,
}

/// Hand-written so the password can never reach a log line, a panic message or an
/// error body. See the module docs — a derived `Debug` is the leak.
impl fmt::Debug for EgressProxyRow {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("EgressProxyRow")
            .field("id", &self.id)
            .field("kind", &self.kind)
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
            .field("active", &self.active)
            .field("tls_to_proxy", &self.tls_to_proxy)
            .field("remote_dns", &self.remote_dns)
            .field(
                "remote_dns_risk_accepted_by",
                &self.remote_dns_risk_accepted_by,
            )
            .field("tunnel_id", &self.tunnel_id)
            .field("chain_next_id", &self.chain_next_id)
            .field("scope", &self.scope)
            .field("traffic_classes", &self.traffic_classes)
            .finish()
    }
}

/// `SELECT <every route column> FROM egress_proxy <tail>`, so the reads cannot
/// drift from `Store::decode_egress_row`. A macro because `q` takes a
/// `&'static str`.
macro_rules! select_routes {
    ($tail:literal) => {
        concat!(
            "SELECT id, kind, host, port, username, sealed_password, allow_plaintext, active, ",
            "tls_to_proxy, remote_dns, remote_dns_risk_accepted_by, remote_dns_risk_accepted_at, ",
            "tunnel_id, chain_next_id, scope, traffic_classes_json, created_at, updated_at ",
            "FROM egress_proxy ",
            $tail
        )
    };
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
    ///
    /// **`row.active` is ignored.** Activation is [`Store::set_active_egress_proxy`]
    /// and nothing else, so editing a route's host or credentials can never make it
    /// live as a side effect — switching which proxy all egress flows through is a
    /// decision an operator takes deliberately, not one an edit form can make for
    /// them.
    pub async fn put_egress_proxy(&self, row: &EgressProxyRow) -> Result<(), StoreError> {
        // `write_password = 0` leaves the stored value untouched on conflict. On a
        // fresh insert there is nothing to carry forward, so the sealed empty string
        // is what lands — no credentials, the same empty-means-absent convention as
        // 0018's `sealed_refresh_token`.
        let write_password = i64::from(row.password.is_some());
        let sealed = self
            .key
            .seal(row.password.as_deref().unwrap_or("").as_bytes())?;
        let classes = serde_json::to_string(&row.traffic_classes)
            .map_err(|e| StoreError::Corrupt(format!("egress traffic classes encode: {e}")))?;
        let now = Utc::now().to_rfc3339();
        // The remote-DNS acceptance is carried forward only across an edit that
        // leaves what was accepted unchanged: the same kind, host and port with
        // remote DNS still on. Anything else clears it, so accepting the risk for
        // one proxy can never silently cover another.
        q("INSERT INTO egress_proxy
                 (id, scheme, kind, host, port, username, sealed_password, allow_plaintext,
                  tls_to_proxy, remote_dns, tunnel_id, chain_next_id, scope, traffic_classes_json,
                  created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16)
             ON CONFLICT(id) DO UPDATE SET
                 remote_dns_risk_accepted_by = CASE
                     WHEN excluded.remote_dns = 1 AND egress_proxy.remote_dns = 1
                      AND excluded.kind = egress_proxy.kind AND excluded.host = egress_proxy.host
                      AND excluded.port = egress_proxy.port
                     THEN egress_proxy.remote_dns_risk_accepted_by ELSE NULL END,
                 remote_dns_risk_accepted_at = CASE
                     WHEN excluded.remote_dns = 1 AND egress_proxy.remote_dns = 1
                      AND excluded.kind = egress_proxy.kind AND excluded.host = egress_proxy.host
                      AND excluded.port = egress_proxy.port
                     THEN egress_proxy.remote_dns_risk_accepted_at ELSE NULL END,
                 scheme = excluded.scheme, kind = excluded.kind,
                 host = excluded.host, port = excluded.port,
                 username = excluded.username,
                 sealed_password = CASE WHEN ?17 = 1
                     THEN excluded.sealed_password
                     ELSE egress_proxy.sealed_password END,
                 allow_plaintext = excluded.allow_plaintext,
                 tls_to_proxy = excluded.tls_to_proxy, remote_dns = excluded.remote_dns,
                 tunnel_id = excluded.tunnel_id, chain_next_id = excluded.chain_next_id,
                 scope = excluded.scope, traffic_classes_json = excluded.traffic_classes_json,
                 updated_at = excluded.updated_at")
        .bind(&row.id)
        .bind(legacy_scheme(&row.kind))
        .bind(&row.kind)
        .bind(&row.host)
        .bind(i64::from(row.port))
        .bind(&row.username)
        .bind(sealed)
        .bind(i64::from(row.allow_plaintext))
        .bind(i64::from(row.tls_to_proxy))
        .bind(i64::from(row.remote_dns))
        .bind(row.tunnel_id.clone())
        .bind(row.chain_next_id.clone())
        .bind(&row.scope)
        .bind(classes)
        .bind(&now)
        .bind(&now)
        .bind(write_password)
        .execute(&self.backend)
        .await?;
        Ok(())
    }

    /// Record that `admin` accepted the remote-DNS risk for route `id` at `at`
    /// (unix seconds). Writes nothing and returns `false` unless the route exists
    /// and has remote DNS switched on, so an acceptance is always about a route
    /// that actually asks its proxy to resolve names.
    pub async fn accept_egress_remote_dns_risk(
        &self,
        id: &str,
        admin: &str,
        at: i64,
    ) -> Result<bool, StoreError> {
        let affected = q("UPDATE egress_proxy
                 SET remote_dns_risk_accepted_by = ?2, remote_dns_risk_accepted_at = ?3
                 WHERE id = ?1 AND remote_dns = 1")
        .bind(id)
        .bind(admin)
        .bind(at)
        .execute(&self.backend)
        .await?;
        Ok(affected > 0)
    }

    /// Every live route (`active = 1`), password unsealed into memory. 0027's index
    /// keeps this at one row today; the egress plan is built from this list, so a
    /// later per-scope index needs no change here.
    pub async fn live_egress_routes(&self) -> Result<Vec<EgressProxyRow>, StoreError> {
        let rows = q(select_routes!("WHERE active = 1 ORDER BY id"))
            .fetch_all(&self.backend)
            .await?;
        rows.iter().map(|r| self.decode_egress_row(r)).collect()
    }

    /// The internal-destination allowlist, ordered by class, then destination.
    pub async fn list_egress_internal_allow(&self) -> Result<Vec<EgressAllowRow>, StoreError> {
        let rows = q(
            "SELECT id, traffic_class, destination, note, created_by, created_at
                      FROM egress_internal_allow ORDER BY traffic_class, destination, id",
        )
        .fetch_all(&self.backend)
        .await?;
        Ok(rows
            .iter()
            .map(|r| EgressAllowRow {
                id: r.get_string("id"),
                traffic_class: r.get_string("traffic_class"),
                destination: r.get_string("destination"),
                note: r.get_string("note"),
                created_by: r.get_string("created_by"),
                created_at: r.get_i64("created_at"),
            })
            .collect())
    }

    /// Add an allowlist entry. The caller validates the class and the destination;
    /// 0033's CHECK refuses a class other than `webhooks` or `siem` regardless.
    pub async fn add_egress_internal_allow(&self, row: &EgressAllowRow) -> Result<(), StoreError> {
        q("INSERT INTO egress_internal_allow
                 (id, traffic_class, destination, note, created_by, created_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)")
        .bind(&row.id)
        .bind(&row.traffic_class)
        .bind(&row.destination)
        .bind(&row.note)
        .bind(&row.created_by)
        .bind(row.created_at)
        .execute(&self.backend)
        .await?;
        Ok(())
    }

    /// Remove an allowlist entry. `true` when a row was actually deleted.
    pub async fn delete_egress_internal_allow(&self, id: &str) -> Result<bool, StoreError> {
        let affected = q("DELETE FROM egress_internal_allow WHERE id = ?1")
            .bind(id)
            .execute(&self.backend)
            .await?;
        Ok(affected > 0)
    }

    /// Every configured route, password unsealed into memory.
    pub async fn list_egress_proxies(&self) -> Result<Vec<EgressProxyRow>, StoreError> {
        let rows = q(select_routes!("ORDER BY id"))
            .fetch_all(&self.backend)
            .await?;
        rows.iter().map(|r| self.decode_egress_row(r)).collect()
    }

    /// One route by id, password unsealed into memory.
    pub async fn get_egress_proxy(&self, id: &str) -> Result<Option<EgressProxyRow>, StoreError> {
        let row = q(select_routes!("WHERE id = ?1"))
            .bind(id)
            .fetch_optional(&self.backend)
            .await?;
        row.as_ref().map(|r| self.decode_egress_row(r)).transpose()
    }

    /// **The** live egress route, or `None` when egress is direct.
    ///
    /// # This signature is the security boundary
    /// It takes **`&self` and nothing else**, and that is deliberate: `ProxyRoute`'s
    /// host is exempt from the SSRF address policy (an egress proxy on RFC1918 is the
    /// normal deployment), and that carve-out is safe only while a route is
    /// deployment-wide operator configuration **that nothing request-derived can
    /// select**. Give this function a destination, an account or a header to choose
    /// by, and choosing a route becomes choosing a destination whose host bypasses
    /// the gate.
    ///
    /// **A function with no request-shaped parameter cannot be steered** — which is
    /// checkable by reading one line, holds under every future edit to the body, and
    /// does not depend on the next maintainer having heard of this reasoning. If a
    /// caller ever needs a parameter here, that is the moment to escalate rather than
    /// to add one.
    ///
    /// 0027's partial unique index is what makes the answer well-defined; without it
    /// "no arguments" would have to pick arbitrarily, which is a latent surprise.
    pub async fn active_egress_proxy(&self) -> Result<Option<EgressProxyRow>, StoreError> {
        let row = q(select_routes!("WHERE active = 1"))
            .fetch_optional(&self.backend)
            .await?;
        row.as_ref().map(|r| self.decode_egress_row(r)).transpose()
    }

    /// Make `id` the live route, or deactivate every route when `id` is `None`.
    ///
    /// Deactivating first is not tidiness: 0027's partial unique index REFUSES a
    /// second active row, so activating without clearing the previous one fails
    /// rather than silently switching. Both statements run so the pair is atomic from
    /// the caller's view; a partial failure leaves no route active, which is the
    /// fail-safe direction — egress falls back to direct rather than to a route the
    /// operator did not choose.
    pub async fn set_active_egress_proxy(&self, id: Option<&str>) -> Result<bool, StoreError> {
        q("UPDATE egress_proxy SET active = 0 WHERE active = 1")
            .execute(&self.backend)
            .await?;
        let Some(id) = id else { return Ok(true) };
        let affected = q("UPDATE egress_proxy SET active = 1 WHERE id = ?1")
            .bind(id)
            .execute(&self.backend)
            .await?;
        Ok(affected > 0)
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
        let classes = r.get_string("traffic_classes_json");
        let traffic_classes: Vec<String> = serde_json::from_str(&classes)
            .map_err(|_| StoreError::Corrupt("egress traffic classes decode".into()))?;
        Ok(EgressProxyRow {
            id: r.get_string("id"),
            kind: r.get_string("kind"),
            host: r.get_string("host"),
            // Stored as an integer in both dialects; a value outside the port range
            // means a corrupt row, not a port to guess at.
            port: u16::try_from(r.get_i64("port"))
                .map_err(|_| StoreError::Corrupt("egress proxy port out of range".into()))?,
            username: r.get_string("username"),
            // Empty ⇒ absent, mirroring how it was written.
            password: (!plain.is_empty()).then_some(plain),
            allow_plaintext: r.get_i64("allow_plaintext") != 0,
            active: r.get_i64("active") != 0,
            tls_to_proxy: r.get_i64("tls_to_proxy") != 0,
            remote_dns: r.get_i64("remote_dns") != 0,
            remote_dns_risk_accepted_by: r.get_opt_string("remote_dns_risk_accepted_by"),
            remote_dns_risk_accepted_at: r.get_opt_i64("remote_dns_risk_accepted_at"),
            tunnel_id: r.get_opt_string("tunnel_id"),
            chain_next_id: r.get_opt_string("chain_next_id"),
            scope: r.get_string("scope"),
            traffic_classes,
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
            kind: "http".into(),
            host: "proxy.corp.example".into(),
            port: 3128,
            username: "svc-mail".into(),
            password: Some(PASSWORD.into()),
            allow_plaintext: false,
            active: false,
            tls_to_proxy: false,
            remote_dns: false,
            remote_dns_risk_accepted_by: None,
            remote_dns_risk_accepted_at: None,
            tunnel_id: None,
            chain_next_id: None,
            scope: "global".into(),
            traffic_classes: vec!["remote_content".into(), "ics_feeds".into()],
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

    /// A second route, so "exactly one active" has something to be exactly one of.
    fn other_row() -> EgressProxyRow {
        EgressProxyRow {
            id: "backup".into(),
            host: "proxy-backup.corp.example".into(),
            ..row()
        }
    }

    #[tokio::test]
    async fn no_route_is_active_until_one_is_made_active() {
        let s = store().await;
        s.put_egress_proxy(&row()).await.unwrap();
        s.put_egress_proxy(&other_row()).await.unwrap();
        assert!(
            s.active_egress_proxy().await.unwrap().is_none(),
            "configuring a route must not make it live — egress stays direct until an \
             operator chooses, or adding a route silently reroutes all traffic"
        );
    }

    #[tokio::test]
    async fn activating_a_second_route_leaves_exactly_one_live() {
        let s = store().await;
        s.put_egress_proxy(&row()).await.unwrap();
        s.put_egress_proxy(&other_row()).await.unwrap();

        assert!(s.set_active_egress_proxy(Some("corp")).await.unwrap());
        assert_eq!(s.active_egress_proxy().await.unwrap().unwrap().id, "corp");

        // The switch. Without the deactivate-first step 0027's partial unique index
        // refuses this outright, which is the constraint doing its job.
        assert!(s.set_active_egress_proxy(Some("backup")).await.unwrap());
        assert_eq!(s.active_egress_proxy().await.unwrap().unwrap().id, "backup");

        let live: Vec<_> = s
            .list_egress_proxies()
            .await
            .unwrap()
            .into_iter()
            .filter(|r| r.active)
            .map(|r| r.id)
            .collect();
        assert_eq!(
            live,
            vec!["backup".to_string()],
            "exactly one route may be live — two would mean selection has to choose, \
             and a selection key is the SSRF primitive 0027 exists to prevent"
        );
    }

    #[tokio::test]
    async fn deactivating_everything_returns_egress_to_direct() {
        let s = store().await;
        s.put_egress_proxy(&row()).await.unwrap();
        s.set_active_egress_proxy(Some("corp")).await.unwrap();
        s.set_active_egress_proxy(None).await.unwrap();
        assert!(s.active_egress_proxy().await.unwrap().is_none());
    }

    #[tokio::test]
    async fn activating_an_unknown_id_reports_that_nothing_was_activated() {
        let s = store().await;
        s.put_egress_proxy(&row()).await.unwrap();
        s.set_active_egress_proxy(Some("corp")).await.unwrap();
        assert!(
            !s.set_active_egress_proxy(Some("nope")).await.unwrap(),
            "an unknown id activated nothing and must say so, or the audit row \
             records a switch that did not happen"
        );
        assert!(
            s.active_egress_proxy().await.unwrap().is_none(),
            "and it must leave NOTHING live rather than the previous route — falling \
             back to direct egress is the fail-safe direction; silently keeping the \
             old route would mean the operator believes they switched and did not"
        );
    }

    #[tokio::test]
    async fn editing_a_route_cannot_make_it_live() {
        let s = store().await;
        s.put_egress_proxy(&row()).await.unwrap();
        let mut edit = row();
        edit.active = true; // ignored by design
        edit.host = "proxy2.corp.example".into();
        s.put_egress_proxy(&edit).await.unwrap();
        assert!(
            s.active_egress_proxy().await.unwrap().is_none(),
            "an edit must not activate a route as a side effect — switching the proxy \
             all egress flows through is a deliberate act, not a form field"
        );
        assert_eq!(
            s.get_egress_proxy("corp").await.unwrap().unwrap().host,
            "proxy2.corp.example",
            "…while the edit itself still applied"
        );
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

    // ── 0033 route policy (t32 e31b) ──────────────────────────────────────────

    #[tokio::test]
    async fn the_0033_route_fields_round_trip() {
        let s = store().await;
        let mut r = row();
        r.kind = "socks5h".into();
        r.tls_to_proxy = true;
        r.remote_dns = true;
        r.tunnel_id = Some("tun-1".into());
        r.chain_next_id = Some("corp-squid".into());
        r.scope = "domain:corp.example".into();
        r.traffic_classes = vec!["keyservers".into(), "pki".into()];
        s.put_egress_proxy(&r).await.unwrap();

        let got = s.get_egress_proxy("corp").await.unwrap().unwrap();
        assert_eq!(got.kind, "socks5h");
        assert!(got.tls_to_proxy && got.remote_dns);
        assert_eq!(got.tunnel_id.as_deref(), Some("tun-1"));
        assert_eq!(got.chain_next_id.as_deref(), Some("corp-squid"));
        assert_eq!(got.scope, "domain:corp.example");
        assert_eq!(got.traffic_classes, ["keyservers", "pki"]);
        assert_eq!(
            got.remote_dns_risk_accepted_by, None,
            "an insert never records an acceptance"
        );

        // The 0026 column keeps a value the older code path understands.
        let scheme = q("SELECT scheme FROM egress_proxy WHERE id = ?1")
            .bind("corp")
            .fetch_optional(&s.backend)
            .await
            .unwrap()
            .unwrap()
            .get_string("scheme");
        assert_eq!(scheme, "socks5");
    }

    #[tokio::test]
    async fn a_row_written_without_0033_values_reads_with_the_migration_defaults() {
        // What an upgraded install holds: a 0026-shaped row with no 0033 values.
        let s = store().await;
        let sealed = s.key.seal(b"").unwrap();
        q("INSERT INTO egress_proxy (id, scheme, host, port, sealed_password, created_at, updated_at)
           VALUES ('old', 'http', 'squid.local', 3128, ?1, 't0', 't0')")
        .bind(sealed)
        .execute(&s.backend)
        .await
        .unwrap();
        let got = s.get_egress_proxy("old").await.unwrap().unwrap();
        assert_eq!(got.kind, "http");
        assert_eq!(got.scope, "global");
        assert_eq!(got.traffic_classes, ["remote_content", "ics_feeds"]);
        assert!(!got.tls_to_proxy && !got.remote_dns);
        assert_eq!(got.tunnel_id, None);
    }

    #[tokio::test]
    async fn the_remote_dns_acceptance_is_only_recorded_for_a_route_that_asks_for_it() {
        let s = store().await;
        s.put_egress_proxy(&row()).await.unwrap();
        assert!(
            !s.accept_egress_remote_dns_risk("corp", "root", 100)
                .await
                .unwrap(),
            "remote DNS is off on this route, so there is nothing to accept"
        );
        assert!(
            !s.accept_egress_remote_dns_risk("nope", "root", 100)
                .await
                .unwrap()
        );
        let got = s.get_egress_proxy("corp").await.unwrap().unwrap();
        assert_eq!(got.remote_dns_risk_accepted_by, None);

        let mut r = row();
        r.kind = "socks5h".into();
        r.remote_dns = true;
        s.put_egress_proxy(&r).await.unwrap();
        assert!(
            s.accept_egress_remote_dns_risk("corp", "root", 100)
                .await
                .unwrap()
        );
        let got = s.get_egress_proxy("corp").await.unwrap().unwrap();
        assert_eq!(got.remote_dns_risk_accepted_by.as_deref(), Some("root"));
        assert_eq!(got.remote_dns_risk_accepted_at, Some(100));
    }

    async fn acceptance(s: &Store) -> Option<String> {
        s.get_egress_proxy("corp")
            .await
            .unwrap()
            .unwrap()
            .remote_dns_risk_accepted_by
    }

    #[tokio::test]
    async fn an_edit_that_changes_what_was_accepted_clears_the_acceptance() {
        let s = store().await;
        let mut accepted = row();
        accepted.kind = "socks5h".into();
        accepted.remote_dns = true;

        // An unrelated edit (username, classes) keeps it.
        s.put_egress_proxy(&accepted).await.unwrap();
        assert!(
            s.accept_egress_remote_dns_risk("corp", "root", 100)
                .await
                .unwrap()
        );
        let mut edit = accepted.clone();
        edit.username = "svc-2".into();
        edit.traffic_classes = vec!["remote_content".into()];
        s.put_egress_proxy(&edit).await.unwrap();
        assert_eq!(acceptance(&s).await.as_deref(), Some("root"));

        // Each of these changes what the admin accepted, and clears it.
        let changes: [fn(&mut EgressProxyRow); 4] = [
            |r| r.host = "other-proxy.example".into(),
            |r| r.port = 1081,
            |r| r.kind = "ssh".into(),
            |r| r.remote_dns = false,
        ];
        for change in changes {
            s.put_egress_proxy(&accepted).await.unwrap();
            assert!(
                s.accept_egress_remote_dns_risk("corp", "root", 100)
                    .await
                    .unwrap()
            );
            let mut r = accepted.clone();
            change(&mut r);
            s.put_egress_proxy(&r).await.unwrap();
            assert_eq!(acceptance(&s).await, None, "{r:?}");
            s.put_egress_proxy(&accepted).await.unwrap();
            assert_eq!(
                acceptance(&s).await,
                None,
                "putting the old values back must not resurrect an acceptance"
            );
        }
    }

    #[tokio::test]
    async fn live_routes_are_the_active_rows_only() {
        let s = store().await;
        s.put_egress_proxy(&row()).await.unwrap();
        s.put_egress_proxy(&other_row()).await.unwrap();
        assert!(s.live_egress_routes().await.unwrap().is_empty());
        s.set_active_egress_proxy(Some("backup")).await.unwrap();
        let live: Vec<_> = s
            .live_egress_routes()
            .await
            .unwrap()
            .into_iter()
            .map(|r| r.id)
            .collect();
        assert_eq!(live, ["backup"]);
    }

    fn allow_entry(id: &str, class: &str, dest: &str) -> EgressAllowRow {
        EgressAllowRow {
            id: id.into(),
            traffic_class: class.into(),
            destination: dest.into(),
            note: "on-prem collector".into(),
            created_by: "root".into(),
            created_at: 1_700_000_000,
        }
    }

    #[tokio::test]
    async fn the_internal_allowlist_round_trips_and_refuses_other_classes() {
        let s = store().await;
        s.add_egress_internal_allow(&allow_entry("a2", "webhooks", "hooks.corp.example"))
            .await
            .unwrap();
        s.add_egress_internal_allow(&allow_entry("a1", "siem", "10.20.0.0/16"))
            .await
            .unwrap();
        let got = s.list_egress_internal_allow().await.unwrap();
        assert_eq!(
            got,
            [
                allow_entry("a1", "siem", "10.20.0.0/16"),
                allow_entry("a2", "webhooks", "hooks.corp.example")
            ]
        );
        assert!(
            s.add_egress_internal_allow(&allow_entry("a3", "remote_content", "10.0.0.0/8"))
                .await
                .is_err(),
            "0033's CHECK keeps every other class out, whatever the caller validated"
        );
        assert!(s.delete_egress_internal_allow("a1").await.unwrap());
        assert!(!s.delete_egress_internal_allow("a1").await.unwrap());
        assert_eq!(s.list_egress_internal_allow().await.unwrap().len(), 1);
    }

    #[test]
    fn the_legacy_scheme_column_gets_a_value_the_0026_reader_knows() {
        assert_eq!(legacy_scheme("http"), "http");
        assert_eq!(legacy_scheme("socks5"), "socks5");
        assert_eq!(legacy_scheme("socks5h"), "socks5");
        assert_eq!(legacy_scheme("ssh"), "ssh");
    }

    #[test]
    fn debug_shows_the_new_fields_and_still_no_password() {
        let rendered = format!("{:?}", row());
        assert!(!rendered.contains(PASSWORD), "{rendered}");
        assert!(rendered.contains("traffic_classes"), "{rendered}");
    }
}
