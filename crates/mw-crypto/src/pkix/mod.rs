//! X.509 for S/MIME (t32-e41), pure Rust on the crate's existing RustCrypto
//! primitives so it runs the same in the server and in the browser worker:
//! signature checks ([`sig`]), certificate addresses and sender matching
//! ([`name`]), path building and validation ([`path`]), and revocation queries and
//! checks ([`revocation`]). `rustls-webpki` is not used: it targets TLS and its
//! `ring` dependency complicates the wasm build (t32-ext §10.6).

pub mod name;
pub mod oid;
pub mod path;
pub mod revocation;
pub mod sig;
