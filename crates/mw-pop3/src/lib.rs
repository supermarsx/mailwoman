#![forbid(unsafe_code)]
//! `mw-pop3` — RFC 1939 client with CAPA (2449), STLS (2595) / implicit-TLS on
//! :995, UIDL pull, SASL, and leave-on-server policies (keep /
//! delete-after-N-days / delete-on-retrieval) (plan §0, SPEC §6.3).
//!
//! POP3 exposes a single INBOX and no server-side flags, so its
//! [`AccountBackend`](mw_engine::backend::AccountBackend) sync is a UIDL-set
//! diff and flags are engine-local. The crate is layered:
//!
//! - [`proto`] — total, I/O-free wire parsing (the `cargo-fuzz` surface).
//! - [`sasl`] — SASL initial-response encoders.
//! - [`policy`] — the leave-on-server DELE decision function.
//! - [`conn`] — tokio + tokio-rustls transport and the command set.
//! - [`backend`] — [`Pop3Backend`], the `AccountBackend` implementation.

pub mod backend;
pub mod conn;
pub mod policy;
pub mod proto;
pub mod sasl;

pub use backend::{Pop3Auth, Pop3Backend, Pop3Config, TlsMode};
pub use policy::{DeleteContext, LeavePolicy};
pub use proto::fuzz_response_lines;
