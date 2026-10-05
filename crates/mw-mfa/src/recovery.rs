//! Break-glass recovery codes: CSPRNG-generated, Argon2id-hashed at rest, and
//! single-use. The plaintext codes are shown to the user exactly once at
//! generation; only their hashes are persisted (by t16-e3's store).
//!
//! New codes are hashed at an [`ArgonCost`]: the process-wide one ([`set_cost`] /
//! [`cost`], default [`ArgonCost::DEFAULT`]) or one passed to [`hash_code_with`]. A
//! stored code is always verified with the parameters written in its own hash, so
//! codes issued before a cost change keep working. They are not re-hashed: a code that
//! verifies is consumed.

use std::sync::{PoisonError, RwLock};

use argon2::password_hash::{PasswordHash, PasswordHasher, PasswordVerifier, SaltString};
use argon2::{Algorithm, Argon2, Params, Version};
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

/// An Argon2id cost setting that failed [`ArgonCost::new`]'s bounds.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum ArgonCostError {
    #[error("argon2 memory must be {min}..={max} KiB, got {got}")]
    Memory { got: u32, min: u32, max: u32 },
    #[error("argon2 passes must be 1..={max}, got {got}")]
    Passes { got: u32, max: u32 },
    #[error("argon2 with a single pass needs at least {min} KiB of memory, got {got}")]
    SinglePassMemory { got: u32, min: u32 },
    #[error("argon2 parallelism must be 1..={max}, got {got}")]
    Parallelism { got: u32, max: u32 },
}

/// Argon2id cost parameters: memory in KiB, passes, lanes. A value of this type is
/// always within the bounds [`ArgonCost::new`] checks. The bounds are the same as
/// `mw-passwd`'s type of the same name; neither crate depends on the other.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ArgonCost {
    m_kib: u32,
    t: u32,
    p: u32,
}

impl ArgonCost {
    /// OWASP's minimum Argon2id configuration (m = 19 MiB, t = 2, p = 1) — the
    /// `Argon2::default()` parameters codes were hashed with before the cost became a
    /// setting.
    pub const DEFAULT: Self = Self {
        m_kib: 19_456,
        t: 2,
        p: 1,
    };
    /// Lowest memory accepted: the 19 MiB of OWASP's minimum configuration.
    pub const MIN_M_KIB: u32 = 19_456;
    /// Highest memory accepted: 1 GiB. Every hash or verify in flight holds this much,
    /// so a larger value lets one mistyped setting exhaust the server.
    pub const MAX_M_KIB: u32 = 1_048_576;
    /// Lowest memory accepted with a single pass: OWASP lists t = 1 only at 46 MiB.
    pub const MIN_M_KIB_SINGLE_PASS: u32 = 47_104;
    /// Highest pass count accepted; time grows linearly with it.
    pub const MAX_T: u32 = 10;
    /// Highest lane count accepted.
    pub const MAX_P: u32 = 16;

    /// Check the three parameters against the bounds above. Out of range is an error;
    /// nothing is clamped.
    ///
    /// # Errors
    /// [`ArgonCostError`] naming the parameter that is out of range.
    pub fn new(m_kib: u32, t: u32, p: u32) -> Result<Self, ArgonCostError> {
        if !(Self::MIN_M_KIB..=Self::MAX_M_KIB).contains(&m_kib) {
            return Err(ArgonCostError::Memory {
                got: m_kib,
                min: Self::MIN_M_KIB,
                max: Self::MAX_M_KIB,
            });
        }
        if !(1..=Self::MAX_T).contains(&t) {
            return Err(ArgonCostError::Passes {
                got: t,
                max: Self::MAX_T,
            });
        }
        if t == 1 && m_kib < Self::MIN_M_KIB_SINGLE_PASS {
            return Err(ArgonCostError::SinglePassMemory {
                got: m_kib,
                min: Self::MIN_M_KIB_SINGLE_PASS,
            });
        }
        if !(1..=Self::MAX_P).contains(&p) {
            return Err(ArgonCostError::Parallelism {
                got: p,
                max: Self::MAX_P,
            });
        }
        Ok(Self { m_kib, t, p })
    }

    /// Memory in KiB.
    #[must_use]
    pub fn m_kib(self) -> u32 {
        self.m_kib
    }

    /// Passes.
    #[must_use]
    pub fn t(self) -> u32 {
        self.t
    }

    /// Lanes.
    #[must_use]
    pub fn p(self) -> u32 {
        self.p
    }
}

impl Default for ArgonCost {
    fn default() -> Self {
        Self::DEFAULT
    }
}

static COST: RwLock<ArgonCost> = RwLock::new(ArgonCost::DEFAULT);

/// Set the process-wide cost [`hash_code`] uses for new codes. Stored codes are
/// untouched and keep verifying.
pub fn set_cost(cost: ArgonCost) {
    *COST.write().unwrap_or_else(PoisonError::into_inner) = cost;
}

/// The process-wide cost ([`ArgonCost::DEFAULT`] until [`set_cost`] is called).
#[must_use]
pub fn cost() -> ArgonCost {
    *COST.read().unwrap_or_else(PoisonError::into_inner)
}

/// Argon2id-hash a recovery code for storage at the process-wide cost. Input is
/// normalized (uppercased, dashes/whitespace stripped) so display formatting does not
/// affect the hash.
pub fn hash_code(code: &str) -> String {
    hash_code_with(code, cost())
}

/// [`hash_code`] at an explicit cost.
pub fn hash_code_with(code: &str, cost: ArgonCost) -> String {
    let salt = SaltString::generate(&mut OsRng);
    let params = Params::new(cost.m_kib, cost.t, cost.p, None)
        .expect("ArgonCost bounds are inside argon2's own");
    Argon2::new(Algorithm::Argon2id, Version::V0x13, params)
        .hash_password(normalize(code).as_bytes(), &salt)
        .expect("argon2id hashing with in-range params and a generated salt cannot fail")
        .to_string()
}

/// The `(m_kib, t, p)` written in a stored hash, or `None` if it does not parse or
/// lacks one of the three.
#[must_use]
pub fn hash_cost(stored_hash: &str) -> Option<(u32, u32, u32)> {
    let parsed = PasswordHash::new(stored_hash).ok()?;
    Some((
        parsed.params.get_decimal("m")?,
        parsed.params.get_decimal("t")?,
        parsed.params.get_decimal("p")?,
    ))
}

/// Verify a presented code against a stored Argon2id hash (constant-time via
/// Argon2's own comparison), with the parameters written in that hash — not the
/// current cost. Does not enforce single-use — see [`consume`].
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

#[cfg(test)]
mod cost_tests {
    use super::*;

    /// `Argon2::default()`'s parameters: what codes were hashed with before the cost
    /// became a setting.
    const OLD: (u32, u32, u32) = (19_456, 2, 1);

    #[test]
    fn default_cost_is_the_previous_argon2_default() {
        let d = ArgonCost::DEFAULT;
        assert_eq!((d.m_kib(), d.t(), d.p()), OLD);
        let p = Params::default();
        assert_eq!((p.m_cost(), p.t_cost(), p.p_cost()), OLD);
        assert_eq!(hash_cost(&hash_code_with("A2C4E-9GHKM", d)), Some(OLD));
    }

    #[test]
    fn codes_issued_before_a_cost_change_still_verify_and_new_ones_carry_it() {
        let old = hash_code_with("A2C4E-9GHKM", ArgonCost::DEFAULT);
        // Precondition.
        assert!(verify_code("A2C4E-9GHKM", &old));

        let raised = ArgonCost::new(32_768, 3, 1).unwrap();
        let new = hash_code_with("A2C4E-9GHKM", raised);
        assert_eq!(hash_cost(&new), Some((32_768, 3, 1)));
        assert!(new.starts_with("$argon2id$v=19$m=32768,t=3,p=1$"));

        // Both verify, formatting-insensitively, and neither accepts another code.
        assert!(verify_code("A2C4E-9GHKM", &old));
        assert!(verify_code("a2c4e9ghkm", &new));
        assert!(!verify_code("WRONG-CODE0", &old));
        assert!(!verify_code("WRONG-CODE0", &new));

        let mut stored = vec![
            RecoveryCode {
                hash: old,
                used: false,
            },
            RecoveryCode {
                hash: hash_code_with("ZZZZZ-22222", raised),
                used: false,
            },
        ];
        assert!(consume(&mut stored, "A2C4E-9GHKM"));
        assert!(!consume(&mut stored, "A2C4E-9GHKM"));
        assert!(consume(&mut stored, "ZZZZZ-22222"));
    }

    #[test]
    fn out_of_bounds_cost_is_rejected() {
        // Precondition: the edges themselves are accepted.
        assert!(ArgonCost::new(19_456, 2, 1).is_ok());
        assert!(ArgonCost::new(1_048_576, 10, 16).is_ok());
        assert!(ArgonCost::new(47_104, 1, 1).is_ok());

        assert!(matches!(
            ArgonCost::new(19_455, 2, 1),
            Err(ArgonCostError::Memory { got: 19_455, .. })
        ));
        assert!(matches!(
            ArgonCost::new(1_048_577, 2, 1),
            Err(ArgonCostError::Memory { .. })
        ));
        assert!(matches!(
            ArgonCost::new(19_456, 0, 1),
            Err(ArgonCostError::Passes { got: 0, .. })
        ));
        assert!(matches!(
            ArgonCost::new(19_456, 11, 1),
            Err(ArgonCostError::Passes { got: 11, .. })
        ));
        assert!(matches!(
            ArgonCost::new(47_103, 1, 1),
            Err(ArgonCostError::SinglePassMemory { .. })
        ));
        assert!(matches!(
            ArgonCost::new(19_456, 2, 0),
            Err(ArgonCostError::Parallelism { got: 0, .. })
        ));
        assert!(matches!(
            ArgonCost::new(19_456, 2, 17),
            Err(ArgonCostError::Parallelism { got: 17, .. })
        ));
    }

    /// The only test that writes the process-wide cost; it puts the default back. The
    /// other tests in this crate only hash and verify, which works at any cost.
    #[test]
    fn process_wide_cost_drives_hash_code() {
        assert_eq!(cost(), ArgonCost::DEFAULT);
        let before = hash_code("A2C4E-9GHKM");
        set_cost(ArgonCost::new(32_768, 3, 1).unwrap());
        let after = hash_code("A2C4E-9GHKM");
        set_cost(ArgonCost::DEFAULT);

        assert_eq!(hash_cost(&before), Some(OLD));
        assert_eq!(hash_cost(&after), Some((32_768, 3, 1)));
        assert!(verify_code("A2C4E-9GHKM", &before));
        assert!(verify_code("A2C4E-9GHKM", &after));
    }
}
