//! Protection decisions and typed policies, independent of server state and routing.
pub mod geography;
pub mod probe_path;
pub mod proxy;
pub mod user_agent;

/// Shared settings namespace for request-protection policy.
pub const SETTINGS_PREFIX: &str = "security.protection.";

/// Built-in high-confidence probe patterns, shared by defaults and classification tests.
pub const DEFAULT_PROBE_PATHS_JSON: &str = include_str!("../data/admin_probe_paths.json");

/// Whole-segment scanner targets, excluding application resource namespaces.
pub const DEFAULT_SCAN_PATHS_JSON: &str = include_str!("../data/scan_probe_paths.json");
