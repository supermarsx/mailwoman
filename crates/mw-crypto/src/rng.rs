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

#[cfg(test)]
mod tests {
    use super::*;
    use rand_core::TryRng;

    /// The `Rc10` adapter is what feeds `rsa` 0.10-rc its randomness, so a bug
    /// here is a key-material bug. `docs/testing/mutation.md` recorded these
    /// exact mutants as surviving: `try_next_u32`/`try_next_u64` replaced with
    /// `Ok(0)`, and `try_fill_bytes` replaced with `Ok(())` — the last of which
    /// hangs a test rather than failing one, because nothing asserted that any
    /// bytes were written. These tests assert the outputs, not the `Ok`.
    #[test]
    fn rc10_next_u32_and_u64_are_not_constant() {
        let mut r = rc10();
        let a: Vec<u32> = (0..16).map(|_| r.try_next_u32().unwrap()).collect();
        assert!(
            a.iter().any(|&v| v != 0),
            "try_next_u32 only produced zeroes"
        );
        assert!(
            a.iter().any(|&v| v != a[0]),
            "try_next_u32 produced a constant: {a:?}"
        );

        let b: Vec<u64> = (0..16).map(|_| r.try_next_u64().unwrap()).collect();
        assert!(
            b.iter().any(|&v| v != 0),
            "try_next_u64 only produced zeroes"
        );
        assert!(
            b.iter().any(|&v| v != b[0]),
            "try_next_u64 produced a constant: {b:?}"
        );
    }

    /// `try_fill_bytes` must write the *whole* buffer. Starting from a sentinel
    /// catches both "wrote nothing" and "wrote only a prefix".
    #[test]
    fn rc10_fill_bytes_writes_the_whole_buffer() {
        let mut r = rc10();
        const N: usize = 512;
        let mut buf = [0xAAu8; N];
        r.try_fill_bytes(&mut buf).unwrap();
        assert!(buf.iter().any(|&b| b != 0xAA), "buffer was not written");

        // Every 32-byte window must have been touched: with 512 random bytes the
        // chance of a whole 32-byte window still reading 0xAA is negligible, so
        // a short write is caught wherever it stopped.
        for (i, window) in buf.chunks(32).enumerate() {
            assert!(
                window.iter().any(|&b| b != 0xAA),
                "window {i} was left untouched"
            );
        }

        // A zero-length fill is a no-op, not a panic.
        r.try_fill_bytes(&mut []).unwrap();
    }

    /// Two adapters are seeded independently from the OS, so they do not agree.
    /// A constant `os_seed` would make every S/MIME operation deterministic.
    #[test]
    fn rc10_instances_are_independently_seeded() {
        let mut a = [0u8; 64];
        let mut b = [0u8; 64];
        rc10().try_fill_bytes(&mut a).unwrap();
        rc10().try_fill_bytes(&mut b).unwrap();
        assert_ne!(a, b, "two rc10() instances produced the same stream");
        assert_ne!(os_seed(), os_seed(), "os_seed is not fresh per call");
    }

    /// `fill_random` is the salt/IV/CEK source: it must fill what it is given.
    #[test]
    fn fill_random_writes_the_whole_buffer() {
        let mut buf = [0xAAu8; 256];
        fill_random(&mut buf);
        for (i, window) in buf.chunks(32).enumerate() {
            assert!(
                window.iter().any(|&b| b != 0xAA),
                "window {i} was left untouched"
            );
        }
        let mut again = [0xAAu8; 256];
        fill_random(&mut again);
        assert_ne!(buf, again, "fill_random repeated itself");
        fill_random(&mut []);
    }
}
