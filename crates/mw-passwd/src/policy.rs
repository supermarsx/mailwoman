//! [`PasswordPolicy`] (display + validation) and [`PasswdConfig`] (forced-change state).

use serde::{Deserialize, Serialize};

use crate::{PasswordError, Result, Secret};

/// Password policy the UI displays before a change, and this crate enforces on the
/// *new* password before contacting a backend (plan §2.3).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct PasswordPolicy {
    pub min_length: u32,
    pub require_upper: bool,
    pub require_lower: bool,
    pub require_digit: bool,
    pub require_symbol: bool,
    /// Human-readable description shown to the user.
    pub description: String,
}

impl Default for PasswordPolicy {
    fn default() -> Self {
        Self {
            min_length: 12,
            require_upper: false,
            require_lower: false,
            require_digit: false,
            require_symbol: false,
            description: "At least 12 characters.".into(),
        }
    }
}

impl PasswordPolicy {
    /// Validate a new password against this policy. Returns
    /// [`PasswordError::PolicyViolation`] describing the first unmet rule.
    ///
    /// Length is counted in Unicode scalar values (`chars`), not bytes, so multibyte
    /// passphrases are not penalized.
    pub fn validate(&self, new: &Secret) -> Result<()> {
        let pw = new.expose();
        let len = pw.chars().count() as u32;
        if len < self.min_length {
            return Err(PasswordError::PolicyViolation(format!(
                "must be at least {} characters",
                self.min_length
            )));
        }
        if self.require_upper && !pw.chars().any(char::is_uppercase) {
            return Err(PasswordError::PolicyViolation(
                "must contain an uppercase letter".into(),
            ));
        }
        if self.require_lower && !pw.chars().any(char::is_lowercase) {
            return Err(PasswordError::PolicyViolation(
                "must contain a lowercase letter".into(),
            ));
        }
        if self.require_digit && !pw.chars().any(|c| c.is_ascii_digit()) {
            return Err(PasswordError::PolicyViolation(
                "must contain a digit".into(),
            ));
        }
        if self.require_symbol
            && !pw
                .chars()
                .any(|c| !c.is_alphanumeric() && !c.is_whitespace())
        {
            return Err(PasswordError::PolicyViolation(
                "must contain a symbol".into(),
            ));
        }
        Ok(())
    }
}

/// Human-readable rendering shown before a change (plan §2.3 "policy display").
impl std::fmt::Display for PasswordPolicy {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "At least {} characters", self.min_length)?;
        let classes = [
            (self.require_upper, "an uppercase letter"),
            (self.require_lower, "a lowercase letter"),
            (self.require_digit, "a digit"),
            (self.require_symbol, "a symbol"),
        ];
        let required: Vec<&str> = classes
            .iter()
            .filter(|(on, _)| *on)
            .map(|(_, label)| *label)
            .collect();
        if !required.is_empty() {
            write!(f, ", including {}", required.join(", "))?;
        }
        write!(f, ".")
    }
}

/// Per-account password-change configuration (maps to a 0008 `passwd_config` row).
///
/// Carries the displayed [`PasswordPolicy`] and the forced-change-on-next-login flag.
/// Persistence (the `passwd_config` table + a store read/write method) is `mw-store`/
/// `mw-server`'s concern (e9/e14); this crate owns only the shape + round-trip.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct PasswdConfig {
    pub policy: PasswordPolicy,
    /// When set, the account must change its password at next login.
    pub force_change_on_next_login: bool,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn display_lists_required_classes() {
        let p = PasswordPolicy {
            min_length: 14,
            require_upper: true,
            require_digit: true,
            ..PasswordPolicy::default()
        };
        assert_eq!(
            p.to_string(),
            "At least 14 characters, including an uppercase letter, a digit."
        );
        assert_eq!(
            PasswordPolicy::default().to_string(),
            "At least 12 characters."
        );
    }

    #[test]
    fn validate_enforces_length_and_classes() {
        let p = PasswordPolicy {
            min_length: 8,
            require_upper: true,
            require_digit: true,
            require_symbol: true,
            ..PasswordPolicy::default()
        };
        assert!(p.validate(&Secret::new("Ab1!short")).is_ok());
        assert!(matches!(
            p.validate(&Secret::new("Ab1!")),
            Err(PasswordError::PolicyViolation(_))
        ));
        assert!(matches!(
            p.validate(&Secret::new("alllower1!")),
            Err(PasswordError::PolicyViolation(_))
        ));
        assert!(matches!(
            p.validate(&Secret::new("NoSymbol123")),
            Err(PasswordError::PolicyViolation(_))
        ));
    }

    #[test]
    fn config_round_trips_with_forced_change() {
        let cfg = PasswdConfig {
            policy: PasswordPolicy {
                min_length: 16,
                require_symbol: true,
                ..PasswordPolicy::default()
            },
            force_change_on_next_login: true,
        };
        let back: PasswdConfig =
            serde_json::from_str(&serde_json::to_string(&cfg).unwrap()).unwrap();
        assert_eq!(cfg, back);
        assert!(back.force_change_on_next_login);
    }
}
