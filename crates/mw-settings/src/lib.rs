//! Settings catalog, validation and policy resolution, independent of HTTP application state.
pub mod settings_registry;
pub mod knobs;
pub mod attachment_rule;
/// Dedicated diagnostics-policy route, shared with its HTTP adapter.
pub const REQUEST_BUDGET_ROUTE: &str = "/rate-limits";
/// Legacy stored value normalized to the last-factor removal block policy.
pub const PASSKEY_LEGACY_BLOCK_ALIAS: &str = "requireRecoveryCodes";
