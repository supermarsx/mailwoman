#![forbid(unsafe_code)]
//! JMAP (RFC 8620 core, RFC 8621 mail) types and a thin async client.
//! Only the surface Mailwoman V0 needs; grows with later milestones.

pub mod client;
pub mod types;

pub use client::{JmapClient, JmapError};
pub use types::*;
