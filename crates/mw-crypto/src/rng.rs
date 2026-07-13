//! Portable CSPRNG helpers. THREE `rand_core` generations coexist in the V4 crypto
//! tree (plan §1.11 note): rPGP rides `rand` 0.8 / `rand_core` 0.6; the S/MIME stack
//! (`rsa` 0.10-rc) rides `rand_core` 0.10; the PQC stack rides its own. All are
//! seeded from ONE OS entropy source — `rand::rngs::OsRng` (whose `getrandom` 0.2
//! `js` backend covers `wasm32-unknown-unknown` with just a feature, no build-time
//! `--cfg`). The [`Rc10`] adapter bridges a ChaCha20 core to the `rand_core` 0.10
//! `CryptoRng` surface `rsa` wants, so nothing on the wasm path pulls `getrandom`
//! 0.3+ (which would need the `--cfg getrandom_backend` flag). See plan §1.13/§6#2.

use core::convert::Infallible;

/// 32 bytes of OS entropy via `getrandom` (0.2 `js` backend on wasm).
fn os_seed() -> [u8; 32] {
    use rand::RngCore;
    let mut seed = [0u8; 32];
    rand::rngs::OsRng.fill_bytes(&mut seed);
    seed
}

/// Fill `dst` with OS entropy (salts, IVs, content-encryption keys).
pub(crate) fn fill_random(dst: &mut [u8]) {
    use rand::RngCore;
    rand::rngs::OsRng.fill_bytes(dst);
}

/// The `rand` 0.8 CSPRNG for rPGP (`Rng + CryptoRng`); OS-backed on every target.
pub(crate) fn pgp_rng() -> rand::rngs::OsRng {
    rand::rngs::OsRng
}

/// A `rand_core` 0.10 CSPRNG for the S/MIME stack (`rsa` 0.10-rc), a ChaCha20 core
/// seeded from the OS. Deterministic PRNG core — no direct `getrandom` 0.3 pull.
pub(crate) fn rc10() -> Rc10 {
    use rand_chacha::rand_core::SeedableRng;
    Rc10(rand_chacha::ChaCha20Rng::from_seed(os_seed()))
}

/// Adapter exposing a `rand_chacha` 0.9 (`rand_core` 0.9) core through the
/// `rand_core` 0.10 `CryptoRng` traits that `rsa` 0.10-rc requires.
pub(crate) struct Rc10(rand_chacha::ChaCha20Rng);

// `rand_core` 0.10 blanket-impls `Rng`/`CryptoRng` for any `TryRng`/`TryCryptoRng`
// with `Error = Infallible`, so we implement only the fallible traits here.
impl rand_core::TryRng for Rc10 {
    type Error = Infallible;
    fn try_next_u32(&mut self) -> Result<u32, Infallible> {
        use rand_chacha::rand_core::RngCore;
        Ok(self.0.next_u32())
    }
    fn try_next_u64(&mut self) -> Result<u64, Infallible> {
        use rand_chacha::rand_core::RngCore;
        Ok(self.0.next_u64())
    }
    fn try_fill_bytes(&mut self, dst: &mut [u8]) -> Result<(), Infallible> {
        use rand_chacha::rand_core::RngCore;
        self.0.fill_bytes(dst);
        Ok(())
    }
}

impl rand_core::TryCryptoRng for Rc10 {}
