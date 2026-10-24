//! Applying the admin security policy (`mw_admin::policy`) to the running process.
//!
//! * [`env_overlay`] runs first at start: it refuses the `MW_ADMIN_*` variables
//!   nothing applies (`mw_admin::config::reject_unsupported_env`), parses the three
//!   that are applied, and sets the admin cookie name from
//!   `MW_ADMIN_SESSION_COOKIE`. An invalid value stops the start with an error
//!   naming the variable.
//! * [`boot`] builds the [`Admin`] over the store, reads the effective policy and
//!   applies it: the TLS floor ([`crate::tls::apply_min_tls`]), the Argon2id cost
//!   for new password hashes (`mw_passwd::set_argon_cost`) and for new recovery
//!   codes (`mw_mfa::recovery::set_cost`). `MW_ADMIN_REQUIRE_2FA` is written to the
//!   global `twofa_policy` row, which is what the sign-in path reads.
//! * [`on_policy_change`] applies a policy written through this process
//!   (`PUT /admin/security-policy`) at once.
//! * [`refresh_loop`] re-reads the stored policy every 30 seconds and applies it
//!   when it changed, so a write made by another server process or by the
//!   `mailwoman admin` CLI takes effect here too.
//!
//! The stored DLP rules are not applied here: the engine reads them from the store
//! on every evaluation (`mw_engine::security::dlp::load_rules`). The floor and the
//! capture policy are served to clients by `crate::client_policy`.

use std::sync::Arc;
use std::time::Duration;

use anyhow::anyhow;
use mw_admin::{Admin, AdminConfig, EffectiveSecurityPolicy, EnvOverlay, MinTls};
use mw_store::Store;

use crate::stores_v6::AdminBackendAdapter;
use crate::tls;

/// How often [`refresh_loop`] re-reads the stored policy.
pub(crate) const REFRESH_EVERY: Duration = Duration::from_secs(30);

/// Read and check the `MW_ADMIN_*` environment, and set the admin cookie name.
///
/// # Errors
/// When an unsupported `MW_ADMIN_*` variable is set, or an applied one has an
/// invalid value; the message names the variable.
pub(crate) fn env_overlay() -> anyhow::Result<EnvOverlay> {
    mw_admin::config::reject_unsupported_env().map_err(|e| anyhow!("{e}"))?;
    let overlay = EnvOverlay::from_env().map_err(|e| anyhow!("{e}"))?;
    mw_admin::config::set_session_cookie_name(overlay.session_cookie.as_deref());
    Ok(overlay)
}

/// The check every policy write runs on `dlpRulesJson`: the engine's own parser.
fn dlp_validator() -> mw_admin::DlpValidator {
    Arc::new(|json: &str| mw_engine::security::dlp::validate_rules(json).map(|_| ()))
}

/// Build the admin façade over `store` with the overlay attached (and the DLP
/// check installed), without applying anything. The `mailwoman admin` CLI uses
/// this, so its writes are checked and pinned exactly as the API's are.
pub(crate) fn admin_for(
    store: Store,
    engine: Option<Arc<mw_engine::Engine>>,
    overlay: EnvOverlay,
) -> Admin {
    Admin::new(
        Arc::new(AdminBackendAdapter::new(store).with_engine(engine)),
        AdminConfig::default(),
    )
    .with_overlay(overlay)
    .with_dlp_validator(dlp_validator())
}

/// Build the [`Admin`] and apply the effective policy to this process. Returns the
/// admin and the policy that was applied (the starting point for
/// [`refresh_loop`]).
///
/// # Errors
/// When the stored policy cannot be read, or the global require-two-factor row
/// cannot be written for `MW_ADMIN_REQUIRE_2FA`.
pub(crate) async fn boot(
    store: Store,
    engine: Option<Arc<mw_engine::Engine>>,
    overlay: EnvOverlay,
) -> anyhow::Result<(Admin, EffectiveSecurityPolicy)> {
    let admin = admin_for(store, engine, overlay);
    sync_require_2fa(&admin).await?;
    let policy = admin
        .effective_security_policy()
        .await
        .map_err(|e| anyhow!("reading the stored security policy: {e}"))?;
    apply(&policy);
    tracing::info!(
        "security policy applied: minimum TLS {}, argon2 m={} t={} p={}{}",
        policy.policy.min_tls.as_str(),
        policy.policy.argon2_m_cost,
        policy.policy.argon2_t_cost,
        policy.policy.argon2_p_cost,
        pinned_note(&policy)
    );
    Ok((admin, policy))
}

fn pinned_note(p: &EffectiveSecurityPolicy) -> String {
    if p.pinned_by_env.is_empty() {
        String::new()
    } else {
        let vars: Vec<&str> = p.pinned_by_env.iter().map(|f| f.var).collect();
        format!(" (pinned by {})", vars.join(", "))
    }
}

/// `MW_ADMIN_REQUIRE_2FA` pins the global require-two-factor rule. The sign-in
/// path reads that rule from the `twofa_policy` table, so the pinned value is
/// written there whenever the row differs from it.
async fn sync_require_2fa(admin: &Admin) -> anyhow::Result<()> {
    let Some(want) = admin.overlay().require_2fa else {
        return Ok(());
    };
    let backend = admin.backend();
    let have = backend
        .get_global_require_2fa()
        .await
        .map_err(|e| anyhow!("reading the global two-factor rule: {e}"))?;
    if have != want {
        backend
            .set_global_require_2fa(want, EnvOverlay::REQUIRE_2FA_VAR)
            .await
            .map_err(|e| anyhow!("writing MW_ADMIN_REQUIRE_2FA to the two-factor rule: {e}"))?;
        tracing::info!("MW_ADMIN_REQUIRE_2FA: global two-factor requirement set to {want}");
    }
    Ok(())
}

/// Put the process-wide settings the policy controls into force. New connections
/// and new hashes use them; nothing already established changes.
pub(crate) fn apply(p: &EffectiveSecurityPolicy) {
    tls::apply_min_tls(match p.policy.min_tls {
        MinTls::V12 => tls::MinTls::V12,
        MinTls::V13 => tls::MinTls::V13,
    });
    let (m, t, l) = (
        p.policy.argon2_m_cost,
        p.policy.argon2_t_cost,
        p.policy.argon2_p_cost,
    );
    // The stored record is read through `SecurityPolicy::from_stored`, which
    // replaces an out-of-range cost with the default, and writes are validated
    // against the same bounds; an error here means the two copies of the bounds
    // have drifted, which `tests::the_argon_bounds_match_the_hashers` catches.
    match mw_passwd::ArgonCost::new(m, t, l) {
        Ok(c) => mw_passwd::set_argon_cost(c),
        Err(e) => tracing::error!("argon2 cost for passwords not applied: {e}"),
    }
    match mw_mfa::recovery::ArgonCost::new(m, t, l) {
        Ok(c) => mw_mfa::recovery::set_cost(c),
        Err(e) => tracing::error!("argon2 cost for recovery codes not applied: {e}"),
    }
}

/// Apply a policy written through this process. Called by
/// `PUT /admin/security-policy` with the effective policy before and after.
pub(crate) fn on_policy_change(old: &EffectiveSecurityPolicy, new: &EffectiveSecurityPolicy) {
    if old.policy.min_tls != new.policy.min_tls {
        tracing::info!(
            "minimum TLS version changed from {} to {}; applies to new connections",
            old.policy.min_tls.as_str(),
            new.policy.min_tls.as_str()
        );
    }
    apply(new);
}

/// Re-read the policy every [`REFRESH_EVERY`] and apply it when it differs from
/// the last one applied. Runs for the life of the server.
pub(crate) async fn refresh_loop(admin: Admin, mut last: EffectiveSecurityPolicy) {
    let mut tick = tokio::time::interval(REFRESH_EVERY);
    tick.tick().await;
    loop {
        tick.tick().await;
        if let Err(e) = sync_require_2fa(&admin).await {
            tracing::warn!("security policy refresh: {e}");
        }
        match admin.effective_security_policy().await {
            Ok(now) if now != last => {
                on_policy_change(&last, &now);
                last = now;
            }
            Ok(_) => {}
            Err(e) => tracing::warn!("security policy refresh: {e}"),
        }
    }
}

#[cfg(test)]
mod tests {
    use mw_admin::policy::argon_bounds;

    /// `mw-admin` validates the cost against its own copy of the bounds; the two
    /// hashers check theirs. All three must agree, or a value the panel accepts
    /// would be refused when applied.
    #[test]
    fn the_argon_bounds_match_the_hashers() {
        use mw_mfa::recovery::ArgonCost as Rc;
        use mw_passwd::ArgonCost as Pc;
        assert_eq!(argon_bounds::MIN_M_KIB, Pc::MIN_M_KIB);
        assert_eq!(argon_bounds::MAX_M_KIB, Pc::MAX_M_KIB);
        assert_eq!(
            argon_bounds::MIN_M_KIB_SINGLE_PASS,
            Pc::MIN_M_KIB_SINGLE_PASS
        );
        assert_eq!(argon_bounds::MAX_T, Pc::MAX_T);
        assert_eq!(argon_bounds::MAX_P, Pc::MAX_P);
        assert_eq!(argon_bounds::MIN_M_KIB, Rc::MIN_M_KIB);
        assert_eq!(argon_bounds::MAX_M_KIB, Rc::MAX_M_KIB);
        assert_eq!(
            argon_bounds::MIN_M_KIB_SINGLE_PASS,
            Rc::MIN_M_KIB_SINGLE_PASS
        );
        assert_eq!(argon_bounds::MAX_T, Rc::MAX_T);
        assert_eq!(argon_bounds::MAX_P, Rc::MAX_P);
    }

    /// The engine reads the stored DLP rules by its own copy of the settings key.
    #[test]
    fn the_engine_reads_the_policy_from_the_key_the_admin_writes() {
        assert_eq!(
            mw_engine::security::dlp::SECURITY_POLICY_SETTINGS_KEY,
            mw_admin::policy::SECURITY_POLICY_SETTINGS_KEY
        );
    }
}
