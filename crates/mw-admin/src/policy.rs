//! The deployment security policy (§19 security-policy section): its typed
//! record, how a stored record is read back, how a write is checked, and the
//! environment variables that pin a field.
//!
//! **Who applies what.** This crate stores and validates; `mw-server`'s
//! `admin_apply` module applies, at start and after every change:
//!
//! * `min_tls` → the HTTPS listener and the outbound IMAP, SMTP, POP3,
//!   ManageSieve and proxy-tunnel connectors (`tls::apply_min_tls`), for
//!   connections opened afterwards;
//! * `argon2_*` → the cost of new Argon2id hashes of locally stored account
//!   passwords (`mw_passwd::local::set_cost`) and of recovery codes
//!   (`mw_mfa::recovery::set_cost`);
//! * `dlp_rules_json` → read by the engine's outbound DLP check beside
//!   `MW_DLP_RULES`, from the settings key [`SECURITY_POLICY_SETTINGS_KEY`];
//! * `max_security_floor`, `capture_policy` → served to signed-in clients at
//!   `GET /api/policy`; the Mailwoman clients enforce them.
//!
//! `require_2fa` is not part of the stored record. It is the global row of the
//! `twofa_policy` table (the one the "Require two-factor" screen writes), read and
//! written through [`crate::AdminBackend`], so there is one source of truth.
//!
//! Callers read the policy through one accessor,
//! [`crate::Admin::effective_security_policy`], which returns the stored record
//! with the environment overlay applied and says which fields the overlay pins.

use serde::{Deserialize, Deserializer, Serialize};
use serde_json::Value;

/// The `settings` row the security policy is stored under. The engine reads the
/// stored DLP rules from this key without depending on this crate; a test in
/// `mw-server` pins its copy of the string equal to this one.
pub const SECURITY_POLICY_SETTINGS_KEY: &str = "v6:admin:security_policy";

/// The lowest TLS version a connection may negotiate. rustls speaks nothing older
/// than 1.2, so there is nothing lower to offer.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum MinTls {
    #[default]
    #[serde(rename = "1.2")]
    V12,
    #[serde(rename = "1.3")]
    V13,
}

impl MinTls {
    /// `"1.2"` or `"1.3"`; anything else is `None`.
    pub fn parse(s: &str) -> Option<Self> {
        match s {
            "1.2" => Some(Self::V12),
            "1.3" => Some(Self::V13),
            _ => None,
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Self::V12 => "1.2",
            Self::V13 => "1.3",
        }
    }
}

/// Whether the desktop and Android apps ask the operating system to keep the
/// window out of screenshots and screen recordings. A browser has no such switch.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum CapturePolicy {
    #[default]
    Off,
    Protect,
}

impl CapturePolicy {
    pub fn parse(s: &str) -> Option<Self> {
        match s {
            "off" => Some(Self::Off),
            "protect" => Some(Self::Protect),
            _ => None,
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Off => "off",
            Self::Protect => "protect",
        }
    }
}

/// The least restrictive message-rendering mode a Mailwoman client may switch to.
/// The names are the client's own mode names (`apps/web/src/viewers/max-security.ts`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum SecurityFloor {
    FullSanitized,
    SanitizedNoMedia,
    PlainText,
}

impl SecurityFloor {
    pub fn parse(s: &str) -> Option<Self> {
        match s {
            "full-sanitized" => Some(Self::FullSanitized),
            "sanitized-no-media" => Some(Self::SanitizedNoMedia),
            "plain-text" => Some(Self::PlainText),
            _ => None,
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Self::FullSanitized => "full-sanitized",
            Self::SanitizedNoMedia => "sanitized-no-media",
            Self::PlainText => "plain-text",
        }
    }
}

/// The Argon2id bounds a cost setting must fall within. They are the bounds of
/// `mw_passwd::local::ArgonCost::new` and `mw_mfa::recovery::ArgonCost::new`; this
/// crate depends on neither, and a test in `mw-server` pins the three copies equal.
pub mod argon_bounds {
    /// 19 MiB, OWASP's minimum configuration.
    pub const MIN_M_KIB: u32 = 19_456;
    /// 1 GiB.
    pub const MAX_M_KIB: u32 = 1_048_576;
    /// With a single pass OWASP lists 46 MiB as the minimum.
    pub const MIN_M_KIB_SINGLE_PASS: u32 = 47_104;
    pub const MAX_T: u32 = 10;
    pub const MAX_P: u32 = 16;

    /// `None` when the triple is inside the bounds, otherwise what is wrong.
    pub fn check(m_kib: u32, t: u32, p: u32) -> Option<String> {
        if !(MIN_M_KIB..=MAX_M_KIB).contains(&m_kib) {
            return Some(format!(
                "argon2 memory must be {MIN_M_KIB}..={MAX_M_KIB} KiB, got {m_kib}"
            ));
        }
        if !(1..=MAX_T).contains(&t) {
            return Some(format!("argon2 passes must be 1..={MAX_T}, got {t}"));
        }
        if t == 1 && m_kib < MIN_M_KIB_SINGLE_PASS {
            return Some(format!(
                "argon2 with a single pass needs at least {MIN_M_KIB_SINGLE_PASS} KiB of memory, got {m_kib}"
            ));
        }
        if !(1..=MAX_P).contains(&p) {
            return Some(format!("argon2 parallelism must be 1..={MAX_P}, got {p}"));
        }
        None
    }
}

/// The stored security-policy record. See the module docs for what applies each
/// field.
///
/// Reading is lenient (a record written by an older release must not stop the
/// server): a stored value outside a field's set reads as that field's default,
/// and a legacy boolean `max_security_floor` reads as `None`. [`Self::from_stored`]
/// reports each such substitution so the caller can log it. Writing is strict:
/// [`Self::validate`].
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct SecurityPolicy {
    pub min_tls: MinTls,
    pub argon2_m_cost: u32,
    pub argon2_t_cost: u32,
    pub argon2_p_cost: u32,
    /// A JSON array of DLP rules (the engine's `DlpRule` shape).
    pub dlp_rules_json: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_security_floor: Option<SecurityFloor>,
    pub capture_policy: CapturePolicy,
}

impl Default for SecurityPolicy {
    /// What a deployment that never stored a policy runs with: TLS 1.2 and up, the
    /// Argon2id parameters hashes were made with before the cost was a setting, no
    /// stored DLP rules, no floor, no capture protection.
    fn default() -> Self {
        Self {
            min_tls: MinTls::V12,
            argon2_m_cost: argon_bounds::MIN_M_KIB,
            argon2_t_cost: 2,
            argon2_p_cost: 1,
            dlp_rules_json: "[]".to_string(),
            max_security_floor: None,
            capture_policy: CapturePolicy::Off,
        }
    }
}

impl SecurityPolicy {
    /// Read a stored record, returning the record and one note for each stored
    /// value that was replaced by its default. An absent field is not noted; a
    /// legacy boolean floor is not noted (it never did anything).
    pub fn from_stored(json: &str) -> Result<(Self, Vec<String>), serde_json::Error> {
        let raw: Value = serde_json::from_str(json)?;
        Ok(Self::from_value(&raw))
    }

    fn from_value(raw: &Value) -> (Self, Vec<String>) {
        let mut p = Self::default();
        let mut notes = Vec::new();
        let get = |k: &str| raw.get(k).filter(|v| !v.is_null());

        if let Some(v) = get("min_tls") {
            match v.as_str().and_then(MinTls::parse) {
                Some(m) => p.min_tls = m,
                None => notes.push(format!(
                    "stored min_tls {v} is not \"1.2\" or \"1.3\"; using \"1.2\""
                )),
            }
        }
        if let Some(v) = get("capture_policy") {
            match v.as_str().and_then(CapturePolicy::parse) {
                Some(c) => p.capture_policy = c,
                None => notes.push(format!(
                    "stored capture_policy {v} is not \"off\" or \"protect\"; using \"off\""
                )),
            }
        }
        // A legacy boolean floor reads as none without a note: it never did anything.
        if let Some(v) = get("max_security_floor").filter(|v| !v.is_boolean()) {
            match v.as_str().and_then(SecurityFloor::parse) {
                Some(f) => p.max_security_floor = Some(f),
                None => notes.push(format!(
                    "stored max_security_floor {v} is not a rendering mode; using none"
                )),
            }
        }
        let num = |k: &str| get(k).map(|v| v.as_u64().and_then(|n| u32::try_from(n).ok()));
        let (m, t, pp) = (
            num("argon2_m_cost"),
            num("argon2_t_cost"),
            num("argon2_p_cost"),
        );
        if m.is_some() || t.is_some() || pp.is_some() {
            let m = m.unwrap_or(Some(p.argon2_m_cost));
            let t = t.unwrap_or(Some(p.argon2_t_cost));
            let pp = pp.unwrap_or(Some(p.argon2_p_cost));
            match (m, t, pp) {
                (Some(m), Some(t), Some(pp)) if argon_bounds::check(m, t, pp).is_none() => {
                    p.argon2_m_cost = m;
                    p.argon2_t_cost = t;
                    p.argon2_p_cost = pp;
                }
                _ => notes.push(format!(
                    "stored argon2 cost {}/{}/{} is out of range; using {}/{}/{}",
                    get("argon2_m_cost").unwrap_or(&Value::Null),
                    get("argon2_t_cost").unwrap_or(&Value::Null),
                    get("argon2_p_cost").unwrap_or(&Value::Null),
                    p.argon2_m_cost,
                    p.argon2_t_cost,
                    p.argon2_p_cost
                )),
            }
        }
        if let Some(v) = get("dlp_rules_json") {
            match v.as_str() {
                Some(s) => p.dlp_rules_json = s.to_string(),
                None => notes.push("stored dlp_rules_json is not a string; using []".into()),
            }
        }
        (p, notes)
    }

    /// The checks a write must pass that the types do not already make: the Argon2
    /// cost within [`argon_bounds`], and `dlp_rules_json` a JSON array. Whether each
    /// array element is a well-formed rule is checked by the validator `mw-server`
    /// installs ([`crate::Admin::with_dlp_validator`]), which knows the rule type.
    pub fn validate(&self) -> Result<(), String> {
        if let Some(e) =
            argon_bounds::check(self.argon2_m_cost, self.argon2_t_cost, self.argon2_p_cost)
        {
            return Err(e);
        }
        match serde_json::from_str::<Value>(&self.dlp_rules_json) {
            Ok(Value::Array(_)) => Ok(()),
            Ok(_) => Err("dlpRulesJson must be a JSON array of rules".into()),
            Err(e) => Err(format!("dlpRulesJson is not valid JSON: {e}")),
        }
    }
}

impl<'de> Deserialize<'de> for SecurityPolicy {
    /// Lenient, as [`SecurityPolicy::from_stored`] (without the notes): used by the
    /// TOML model.
    fn deserialize<D: Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        let raw = Value::deserialize(d)?;
        Ok(Self::from_value(&raw).0)
    }
}

/// One field of the policy that an environment variable pins: `field` is its wire
/// name, `var` the variable.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct PinnedField {
    pub field: &'static str,
    pub var: &'static str,
}

/// The policy as it is in force: the stored record with the environment overlay
/// applied, the global require-two-factor rule, and which fields the environment
/// pins. This is what [`crate::Admin::effective_security_policy`] returns and what
/// `GET /admin/security-policy` serialises.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EffectiveSecurityPolicy {
    pub policy: SecurityPolicy,
    pub require_2fa: bool,
    pub pinned_by_env: Vec<PinnedField>,
}

impl EffectiveSecurityPolicy {
    /// Whether `field` (wire name) is pinned, and by which variable.
    pub fn pinned(&self, field: &str) -> Option<&'static str> {
        self.pinned_by_env
            .iter()
            .find(|p| p.field == field)
            .map(|p| p.var)
    }
}

/// A full write of the policy: every stored field plus the global
/// require-two-factor rule (`PUT /admin/security-policy`, `security-policy set`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SecurityPolicyWrite {
    pub policy: SecurityPolicy,
    pub require_2fa: bool,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_record_from_before_26_20_reads_with_a_null_floor_and_without_notes() {
        let legacy = r#"{"min_tls":"1.2","require_2fa":true,"argon2_m_cost":19456,
            "argon2_t_cost":2,"argon2_p_cost":1,"dlp_rules_json":"[]",
            "max_security_floor":true,"capture_policy":"off"}"#;
        let (p, notes) = SecurityPolicy::from_stored(legacy).unwrap();
        assert_eq!(p, SecurityPolicy::default());
        assert!(notes.is_empty(), "{notes:?}");
    }

    #[test]
    fn stored_values_outside_their_sets_read_as_defaults_and_are_noted() {
        let odd = r#"{"min_tls":"1.1","argon2_m_cost":1,"argon2_t_cost":2,
            "argon2_p_cost":1,"max_security_floor":"paranoid","capture_policy":"block",
            "dlp_rules_json":"[{\"id\":\"x\"}]"}"#;
        let (p, notes) = SecurityPolicy::from_stored(odd).unwrap();
        assert_eq!(p.min_tls, MinTls::V12);
        assert_eq!(p.capture_policy, CapturePolicy::Off);
        assert_eq!(p.max_security_floor, None);
        assert_eq!(p.argon2_m_cost, argon_bounds::MIN_M_KIB);
        assert_eq!(p.dlp_rules_json, "[{\"id\":\"x\"}]");
        assert_eq!(notes.len(), 4, "{notes:?}");
    }

    #[test]
    fn a_typed_record_round_trips() {
        let p = SecurityPolicy {
            min_tls: MinTls::V13,
            argon2_m_cost: 32_768,
            argon2_t_cost: 3,
            argon2_p_cost: 1,
            dlp_rules_json: "[]".into(),
            max_security_floor: Some(SecurityFloor::PlainText),
            capture_policy: CapturePolicy::Protect,
        };
        let json = serde_json::to_string(&p).unwrap();
        assert!(json.contains(r#""min_tls":"1.3""#), "{json}");
        assert!(
            json.contains(r#""max_security_floor":"plain-text""#),
            "{json}"
        );
        let (back, notes) = SecurityPolicy::from_stored(&json).unwrap();
        assert_eq!(back, p);
        assert!(notes.is_empty());
    }

    #[test]
    fn validate_refuses_an_out_of_range_cost_and_non_array_rules() {
        let ok = SecurityPolicy::default();
        assert!(ok.validate().is_ok());
        let low = SecurityPolicy {
            argon2_m_cost: 1024,
            ..ok.clone()
        };
        assert!(low.validate().unwrap_err().contains("memory"));
        let single = SecurityPolicy {
            argon2_t_cost: 1,
            ..ok.clone()
        };
        assert!(single.validate().unwrap_err().contains("single pass"));
        let obj = SecurityPolicy {
            dlp_rules_json: "{}".into(),
            ..ok.clone()
        };
        assert!(obj.validate().is_err());
        let junk = SecurityPolicy {
            dlp_rules_json: "[".into(),
            ..ok
        };
        assert!(junk.validate().is_err());
    }
}
