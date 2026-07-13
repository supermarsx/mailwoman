#![forbid(unsafe_code)]
//! `mw-crypto` — Mailwoman's crypto & security primitives (SPEC §8, plan §1.1):
//! OpenPGP (rPGP), S/MIME (RustCrypto `cms`/`x509-cert`/`rsa`/`p256`), and a PQC
//! hybrid X25519+ML-KEM-768 key-wrap, in ONE crate with TWO build targets split by
//! cargo feature:
//!
//! - **`native`** (default): the server/engine side — public-key *verify* of
//!   received mail, cert harvesting, verdict inputs, PQC store-key wrapping, WKD
//!   lookup. See [`native`].
//! - **`wasm`** (`wasm32-unknown-unknown` via wasm-bindgen/wasm-pack): the browser
//!   side — ALL private-key operations (keygen, decrypt, private sign, PKCS#12
//!   import, backup). The `#[wasm_bindgen]` surface (frozen §2.3) is gated on
//!   `cfg(target_arch = "wasm32")` so `cargo build -p mw-crypto --target
//!   wasm32-unknown-unknown` compiles it with no extra flags, and the native
//!   workspace build never links wasm-bindgen. See `wasm`.
//!
//! The frozen §2.1 DTOs in [`types`] are shared by both targets and re-exported so
//! `mw-engine` / the mock / the WASM boundary emit byte-identical shapes (parity
//! gate, plan §1.5). This is the e0 scaffold: types are frozen, operation bodies
//! are `todo!()` (e1 fills the crypto, e6 wires the engine, e8 builds+wires wasm).
//!
//! `#![forbid(unsafe_code)]` (plan §4.3 / DoD): NO hand-written unsafe in
//! Mailwoman crypto. The wasm-bindgen glue lives behind the `cfg(target_arch =
//! "wasm32")` boundary in `wasm.rs`; the native crate carries none.

pub mod types;
pub use types::*;

pub mod error;
pub use error::{CryptoError, Result};

mod rng;

pub mod pgp;
pub mod smime;

#[cfg(all(not(target_arch = "wasm32"), feature = "native"))]
pub mod pqc;

// The native engine facade wraps `pqc` (native-only), so it too is gated off wasm —
// even though the default `native` feature stays on, the wasm build excludes it.
#[cfg(all(not(target_arch = "wasm32"), feature = "native"))]
pub mod native;

#[cfg(target_arch = "wasm32")]
mod wasm;

/// The algorithm-suite tag recorded alongside the PQC-wrapped store seal key
/// (plan §2.4/§8.3 crypto-agility). Frozen so the store's `key_material`
/// suite column and any migration read the same identifier.
pub const STORE_KEY_WRAP_SUITE: &str = "x25519-ml-kem-768-v1";

/// The `mw-crypto` contract/scaffold version (bumped when the frozen §2.1/§2.3
/// surface changes and the coordinator re-broadcasts).
pub const CONTRACT_VERSION: &str = "v4-0";
