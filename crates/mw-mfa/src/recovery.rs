//! Break-glass recovery codes: CSPRNG-generated, Argon2id-hashed at rest, and
//! single-use. The plaintext codes are shown to the user exactly once at
//! generation; only their hashes are persisted (by t16-e3's store).

use argon2::Argon2;
use argon2::password_hash::{PasswordHash, PasswordHasher, PasswordVerifier, SaltString};
use rand::RngCore;
use rand::rngs::OsRng;

/// Default number of recovery codes issued per enrolment (DQ2).
pub const DEFAULT_RECOVERY_CODES: usize = 10;

// Unambiguous alphabet (Crockford-style: no I/L/O/U, no 0/1) so hand-typed codes
// are hard to mis-read.
const ALPHABET: &[u8] = b"23456789ABCDEFGHJKMNPQRSTVWXYZ";
// Characters per group and groups per code → 10 characters, printed as `xxxxx-xxxxx`.
const GROUP_LEN: usize = 5;
const GROUPS: usize = 2;

/// A stored recovery code: its Argon2id hash plus whether it has been consumed.
#[derive(Debug, Clone)]
pub struct RecoveryCode {
    /// Argon2id PHC hash string of the code.
    pub hash: String,
    /// Whether this code has already been used (single-use enforcement).
    pub used: bool,
}

/// Generate `n` fresh recovery codes as display strings (e.g. `A2C4E-9GHKM`).
/// The caller shows these once and persists only their [`hash_code`] outputs.
pub fn generate_codes(n: usize) -> Vec<String> {
    (0..n).map(|_| random_code()).collect()
}

fn random_code() -> String {
    let mut rng = OsRng;
    let mut out = String::with_capacity(GROUPS * GROUP_LEN + (GROUPS - 1));
    for g in 0..GROUPS {
        if g > 0 {
            out.push('-');
        }
        for _ in 0..GROUP_LEN {
            // Rejection-free uniform pick: the alphabet length (30) does not divide
            // 256 evenly, but the modulo bias is negligible for a display code; to
            // avoid it entirely we reject bytes in the biased tail.
            let idx = loop {
                let b = rng.next_u32() as u8;
                let limit = 256 - (256 % ALPHABET.len());
                if (b as usize) < limit {
                    break b as usize % ALPHABET.len();
                }
            };
            out.push(ALPHABET[idx] as char);
        }
    }
    out
}

/// Argon2id-hash a recovery code for storage. Input is normalized (uppercased,
/// dashes/whitespace stripped) so display formatting does not affect the hash.
pub fn hash_code(code: &str) -> String {
    let salt = SaltString::generate(&mut OsRng);
    Argon2::default()
        .hash_password(normalize(code).as_bytes(), &salt)
        .expect("argon2id hashing with default params cannot fail")
        .to_string()
}

/// Verify a presented code against a stored Argon2id hash (constant-time via
/// Argon2's own comparison). Does not enforce single-use — see [`consume`].
pub fn verify_code(presented: &str, stored_hash: &str) -> bool {
    let Ok(parsed) = PasswordHash::new(stored_hash) else {
        return false;
    };
    Argon2::default()
        .verify_password(normalize(presented).as_bytes(), &parsed)
        .is_ok()
}

/// Single-use consume: verify `presented` against the first unused code whose hash
/// matches, mark it used, and return `true`. A second attempt with the same code
/// finds it already used and returns `false`.
pub fn consume(codes: &mut [RecoveryCode], presented: &str) -> bool {
    for c in codes.iter_mut() {
        if !c.used && verify_code(presented, &c.hash) {
            c.used = true;
            return true;
        }
    }
    false
}

/// Normalize a code to its canonical hashed form: uppercase, dashes/whitespace
/// removed.
fn normalize(code: &str) -> String {
    code.chars()
        .filter(|c| c.is_ascii_alphanumeric())
        .map(|c| c.to_ascii_uppercase())
        .collect()
}
