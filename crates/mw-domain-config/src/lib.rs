//! Validated operator-owned domain authorities and deterministic backend selection.
//!
//! This crate has no HTTP routing, application state or credentialed connection
//! orchestration. Network failover and protocol negotiation belong to the host.

pub mod domain_auth;
pub mod domain_config;
pub mod domain_security;
pub mod domain_selection;
pub mod ews;
pub mod username_format;

/// Maximum mail authorities one pre-authentication failover may consider.
pub const MAX_BACKENDS: usize = 3;
