//! t39-e6 (R2): the administrator passkey policy for mailbox accounts.
//!
//! Five registry settings, each global/domain-scoped with administrator locks
//! (`settings_registry.rs`, read through `security_center::policy`):
//!
//! * `twofa.passkeyAttachment` — `any` | `platform` | `cross-platform`
//! * `twofa.passkeyUserVerification` — `required` | `preferred` | `discouraged`
//! * `twofa.passkeyResidentKey` — `required` | `preferred` | `discouraged`
//! * `twofa.passkeyLegacy` — `allow` | `refuse`: what happens to a passkey that
//!   was registered before a stricter policy and does not meet it
//! * `twofa.lastFactorRemoval` — `allow` | `block`: whether the last passkey or
//!   TOTP of an account can be removed (t77: `block` refuses until another
//!   factor exists; recovery codes do not count). The t39-e6 value
//!   `requireRecoveryCodes` is read as `block` (migration 0074 rewrites it).
//!
//! What the server can check, and how:
//!
//! * User verification is a signed flag in `authenticatorData`. With
//!   `required`, a registration or an assertion without the UV flag is refused
//!   after its signature verified.
//! * Attachment is what the browser reports (`authenticatorAttachment`) and the
//!   transports it lists. Attestation is `none`, so neither is signed by the
//!   authenticator: the check refuses a browser that reports the wrong kind, or
//!   none at all, but it cannot prove the hardware.
//! * Resident key: the browser is asked for it in `authenticatorSelection`; a
//!   registration whose `credProps.rk` is reported `false` while the policy says
//!   `required` is refused. A browser that does not report `credProps` is
//!   trusted to have honoured `residentKey: "required"`.
//!
//! In engine mode a passkey is also a passwordless sign-in credential, so the
//! registration floor stays a discoverable, user-verified credential: the
//! policy can make UV and resident-key stricter there, never weaker.
//!
//! Existing passkeys that do not meet a stricter policy are GRANDFATHERED by
//! default (`twofa.passkeyLegacy = allow`): they keep signing in under the rule
//! they were registered with, Settings marks them and asks the user to add a new
//! passkey. With `refuse` they are refused after their signature verified, with
//! a message that names the reason; the user can still sign in with TOTP or a
//! recovery code. A credential that has already shown user verification is
//! held to `required` (no downgrade to a UV-less assertion).

use mw_mfa::webauthn::UserVerification;
use serde_json::{Value, json};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Attachment {
    Any,
    Platform,
    CrossPlatform,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) enum Requirement {
    Discouraged,
    Preferred,
    Required,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Legacy {
    Allow,
    Refuse,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum LastFactor {
    Allow,
    Block,
}

/// Why a passkey ceremony or a factor removal was refused. `code()` is the
/// stable machine code the web app maps to a localised message.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Refusal {
    UserVerification,
    AttachmentPlatform,
    AttachmentCrossPlatform,
    AttachmentUnknown,
    ResidentKey,
    LegacyCredential,
    LastFactorBlocked,
    /// Two-factor sign-in is required for the account (`twofa.required`).
    LastFactorRequired,
}

impl Refusal {
    pub(crate) fn code(self) -> &'static str {
        match self {
            Refusal::UserVerification => "passkeyPolicyUserVerification",
            Refusal::AttachmentPlatform => "passkeyPolicyPlatform",
            Refusal::AttachmentCrossPlatform => "passkeyPolicyCrossPlatform",
            Refusal::AttachmentUnknown => "passkeyPolicyAttachmentUnknown",
            Refusal::ResidentKey => "passkeyPolicyResidentKey",
            Refusal::LegacyCredential => "passkeyPolicyLegacy",
            Refusal::LastFactorBlocked => "lastFactorBlocked",
            Refusal::LastFactorRequired => "lastFactorRequired",
        }
    }

    /// The English fallback text (the web app shows its own localised text).
    pub(crate) fn message(self) -> &'static str {
        match self {
            Refusal::UserVerification => {
                "Your organization requires a passkey that checks your PIN, fingerprint or face. This authenticator did not verify you."
            }
            Refusal::AttachmentPlatform => {
                "Your organization requires a passkey stored on this device (for example Windows Hello, Touch ID or Android). Security keys and phones are not accepted."
            }
            Refusal::AttachmentCrossPlatform => {
                "Your organization requires a security key or phone passkey. Passkeys stored on this device are not accepted."
            }
            Refusal::AttachmentUnknown => {
                "Your browser did not say what kind of authenticator this is, so it cannot be checked against your organization's passkey policy."
            }
            Refusal::ResidentKey => {
                "Your organization requires a passkey that is stored on the authenticator (a discoverable credential). This authenticator did not store one."
            }
            Refusal::LegacyCredential => {
                "This passkey was added before your organization's current passkey policy and no longer meets it. Sign in another way, then add a new passkey."
            }
            Refusal::LastFactorBlocked => {
                "Your organization does not allow removing your last second factor. Add another sign-in method first."
            }
            Refusal::LastFactorRequired => {
                "Your organization requires two-factor authentication. Add another sign-in method before removing this one."
            }
        }
    }

    /// The JSON body of a refusal answer.
    pub(crate) fn body(self) -> Value {
        json!({ "code": self.code(), "error": self.message(), "detail": self.message() })
    }
}

/// The effective passkey policy for one account.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct PasskeyPolicy {
    pub attachment: Attachment,
    pub user_verification: Requirement,
    pub resident_key: Requirement,
    pub legacy: Legacy,
    pub last_factor: LastFactor,
}

impl Default for PasskeyPolicy {
    fn default() -> Self {
        PasskeyPolicy {
            attachment: Attachment::Any,
            user_verification: Requirement::Preferred,
            resident_key: Requirement::Discouraged,
            legacy: Legacy::Allow,
            last_factor: LastFactor::Allow,
        }
    }
}

pub(crate) fn parse_attachment(v: &str) -> Attachment {
    match v {
        "platform" => Attachment::Platform,
        "cross-platform" => Attachment::CrossPlatform,
        _ => Attachment::Any,
    }
}

pub(crate) fn parse_requirement(v: &str, fallback: Requirement) -> Requirement {
    match v {
        "required" => Requirement::Required,
        "preferred" => Requirement::Preferred,
        "discouraged" => Requirement::Discouraged,
        _ => fallback,
    }
}

pub(crate) fn parse_legacy(v: &str) -> Legacy {
    if v == "refuse" {
        Legacy::Refuse
    } else {
        Legacy::Allow
    }
}

/// `requireRecoveryCodes` is the t39-e6 name of the rule that became `block`
/// (t77): a config written before t77 keeps refusing the last removal.
pub(crate) const LEGACY_BLOCK_ALIAS: &str = "requireRecoveryCodes";

pub(crate) fn parse_last_factor(v: &str) -> LastFactor {
    match v {
        "block" | LEGACY_BLOCK_ALIAS => LastFactor::Block,
        _ => LastFactor::Allow,
    }
}

fn requirement_str(r: Requirement) -> &'static str {
    match r {
        Requirement::Required => "required",
        Requirement::Preferred => "preferred",
        Requirement::Discouraged => "discouraged",
    }
}

/// The attachment a credential's stored transports imply. `internal` is a
/// platform authenticator (a synced platform passkey also lists `hybrid`);
/// `usb`/`nfc`/`ble`/`smart-card`/`hybrid` alone are roaming. No transports:
/// unknown.
pub(crate) fn attachment_from_transports(transports: &str) -> Option<Attachment> {
    let list: Vec<&str> = transports
        .split(',')
        .map(str::trim)
        .filter(|t| !t.is_empty())
        .collect();
    if list.contains(&"internal") {
        Some(Attachment::Platform)
    } else if list
        .iter()
        .any(|t| matches!(*t, "usb" | "nfc" | "ble" | "smart-card" | "hybrid"))
    {
        Some(Attachment::CrossPlatform)
    } else {
        None
    }
}

/// What the browser reported alongside a registration.
#[derive(Debug, Clone, Default)]
pub(crate) struct ClientReport<'a> {
    /// `PublicKeyCredential.authenticatorAttachment`.
    pub attachment: Option<&'a str>,
    /// `getTransports()`, comma-joined.
    pub transports: &'a str,
    /// `getClientExtensionResults().credProps.rk`.
    pub resident_key: Option<bool>,
}

/// A stored credential's properties, for the grandfathering check.
#[derive(Debug, Clone, Copy)]
pub(crate) struct StoredCredential<'a> {
    pub transports: &'a str,
    pub discoverable: bool,
    /// The last ceremony of this credential carried the UV flag.
    pub uv_seen: bool,
}

impl PasskeyPolicy {
    /// The policy with the mode's registration floor applied: in engine mode a
    /// passkey is also a passwordless credential, so UV and resident key are
    /// `required` whatever the setting says.
    pub(crate) fn with_engine_floor(mut self, engine: bool) -> Self {
        if engine {
            self.user_verification = self.user_verification.max(Requirement::Required);
            self.resident_key = self.resident_key.max(Requirement::Required);
        }
        self
    }

    /// The `authenticatorSelection` part of the registration options, plus
    /// the flat fields older clients read.
    pub(crate) fn registration_options(&self) -> Value {
        let mut selection = json!({
            "userVerification": requirement_str(self.user_verification),
            "residentKey": requirement_str(self.resident_key),
            "requireResidentKey": self.resident_key == Requirement::Required,
        });
        match self.attachment {
            Attachment::Platform => selection["authenticatorAttachment"] = json!("platform"),
            Attachment::CrossPlatform => {
                selection["authenticatorAttachment"] = json!("cross-platform")
            }
            Attachment::Any => {}
        }
        selection
    }

    /// The policy as `GET /api/account/2fa` reports it to the user.
    pub(crate) fn summary(&self) -> Value {
        json!({
            "attachment": match self.attachment {
                Attachment::Any => "any",
                Attachment::Platform => "platform",
                Attachment::CrossPlatform => "cross-platform",
            },
            "userVerification": requirement_str(self.user_verification),
            "residentKey": requirement_str(self.resident_key),
            "legacy": if self.legacy == Legacy::Refuse { "refuse" } else { "allow" },
            "lastFactorRemoval": match self.last_factor {
                LastFactor::Allow => "allow",
                LastFactor::Block => "block",
            },
        })
    }

    /// The UV the authentication options ask the browser for.
    pub(crate) fn assertion_uv_hint(&self) -> &'static str {
        requirement_str(self.user_verification)
    }

    /// Whether a registered credential would be stored discoverable.
    pub(crate) fn discoverable(&self) -> bool {
        self.resident_key == Requirement::Required
    }

    /// Check a verified registration against the policy. `user_verified` is
    /// the signed UV flag of the attestation's authenticatorData.
    pub(crate) fn check_registration(
        &self,
        user_verified: bool,
        report: &ClientReport<'_>,
    ) -> Result<(), Refusal> {
        if self.user_verification == Requirement::Required && !user_verified {
            return Err(Refusal::UserVerification);
        }
        if self.attachment != Attachment::Any {
            let reported = match report.attachment {
                Some("platform") => Some(Attachment::Platform),
                Some("cross-platform") => Some(Attachment::CrossPlatform),
                _ => None,
            };
            let from_transports = attachment_from_transports(report.transports);
            let Some(kind) = reported.or(from_transports) else {
                return Err(Refusal::AttachmentUnknown);
            };
            // The two client signals must not disagree with the policy.
            for k in [Some(kind), from_transports].into_iter().flatten() {
                if k != self.attachment {
                    return Err(self.attachment_refusal());
                }
            }
        }
        if self.resident_key == Requirement::Required && report.resident_key == Some(false) {
            return Err(Refusal::ResidentKey);
        }
        Ok(())
    }

    fn attachment_refusal(&self) -> Refusal {
        match self.attachment {
            Attachment::Platform => Refusal::AttachmentPlatform,
            _ => Refusal::AttachmentCrossPlatform,
        }
    }

    /// Why a stored credential does not meet the current policy (empty: it
    /// does). Used to mark it in Settings and, with `legacy = refuse`, to
    /// refuse it.
    pub(crate) fn credential_gaps(&self, cred: StoredCredential<'_>) -> Vec<Refusal> {
        let mut gaps = Vec::new();
        if self.user_verification == Requirement::Required && !cred.uv_seen {
            gaps.push(Refusal::UserVerification);
        }
        if self.attachment != Attachment::Any {
            match attachment_from_transports(cred.transports) {
                Some(kind) if kind == self.attachment => {}
                Some(_) => gaps.push(self.attachment_refusal()),
                None => gaps.push(Refusal::AttachmentUnknown),
            }
        }
        if self.resident_key == Requirement::Required && !cred.discoverable {
            gaps.push(Refusal::ResidentKey);
        }
        gaps
    }

    /// Decide an assertion: `Ok(uv)` is the user-verification level the
    /// signature check must enforce, `Err` a refusal of a non-compliant
    /// credential (only with `legacy = refuse`).
    ///
    /// A UV-required policy asks for UV. A grandfathered credential that has
    /// never shown UV is verified with `Preferred`; one that has shown UV is
    /// held to `Required`.
    pub(crate) fn assertion_rule(
        &self,
        cred: StoredCredential<'_>,
    ) -> Result<UserVerification, Refusal> {
        // UV is decided per ceremony from the signed flag, not as a stored
        // property: with `refuse` a passkey that never showed UV is asked for
        // it (and refused without it) rather than refused outright.
        if self.legacy == Legacy::Refuse
            && self
                .credential_gaps(cred)
                .iter()
                .any(|g| *g != Refusal::UserVerification)
        {
            return Err(Refusal::LegacyCredential);
        }
        if self.user_verification == Requirement::Required
            && (cred.uv_seen || self.legacy == Legacy::Refuse)
        {
            Ok(UserVerification::Required)
        } else {
            Ok(UserVerification::Preferred)
        }
    }

    /// The rule for a removal that would leave no TOTP and no passkey.
    /// `required` = the account's `twofa.required` policy. Recovery codes are
    /// not a factor here: they only replace a lost one.
    pub(crate) fn check_last_factor_removal(&self, required: bool) -> Result<(), Refusal> {
        if required {
            return Err(Refusal::LastFactorRequired);
        }
        match self.last_factor {
            LastFactor::Allow => Ok(()),
            LastFactor::Block => Err(Refusal::LastFactorBlocked),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn strict() -> PasskeyPolicy {
        PasskeyPolicy {
            attachment: Attachment::Platform,
            user_verification: Requirement::Required,
            resident_key: Requirement::Required,
            legacy: Legacy::Allow,
            last_factor: LastFactor::Allow,
        }
    }

    #[test]
    fn options_reflect_each_policy() {
        let d = PasskeyPolicy::default().registration_options();
        assert_eq!(d["userVerification"], "preferred");
        assert_eq!(d["residentKey"], "discouraged");
        assert!(d.get("authenticatorAttachment").is_none());
        let s = strict().registration_options();
        assert_eq!(s["authenticatorAttachment"], "platform");
        assert_eq!(s["userVerification"], "required");
        assert_eq!(s["residentKey"], "required");
        assert_eq!(s["requireResidentKey"], true);
        let roaming = PasskeyPolicy {
            attachment: Attachment::CrossPlatform,
            user_verification: Requirement::Discouraged,
            resident_key: Requirement::Preferred,
            ..PasskeyPolicy::default()
        };
        let r = roaming.registration_options();
        assert_eq!(r["authenticatorAttachment"], "cross-platform");
        assert_eq!(r["userVerification"], "discouraged");
        assert_eq!(r["residentKey"], "preferred");
        assert_eq!(roaming.assertion_uv_hint(), "discouraged");
    }

    #[test]
    fn engine_floor_only_tightens() {
        let p = PasskeyPolicy {
            user_verification: Requirement::Discouraged,
            resident_key: Requirement::Discouraged,
            ..PasskeyPolicy::default()
        }
        .with_engine_floor(true);
        assert_eq!(p.user_verification, Requirement::Required);
        assert_eq!(p.resident_key, Requirement::Required);
        assert!(p.discoverable());
        let proxy = PasskeyPolicy::default().with_engine_floor(false);
        assert_eq!(proxy, PasskeyPolicy::default());
    }

    #[test]
    fn registration_checks() {
        let p = strict();
        let ok = ClientReport {
            attachment: Some("platform"),
            transports: "internal",
            resident_key: Some(true),
        };
        assert_eq!(p.check_registration(true, &ok), Ok(()));
        assert_eq!(
            p.check_registration(false, &ok),
            Err(Refusal::UserVerification)
        );
        let usb = ClientReport {
            attachment: Some("cross-platform"),
            transports: "usb",
            resident_key: Some(true),
        };
        assert_eq!(
            p.check_registration(true, &usb),
            Err(Refusal::AttachmentPlatform)
        );
        // Reported platform but listing only usb: the signals disagree.
        let mixed = ClientReport {
            attachment: Some("platform"),
            transports: "usb",
            resident_key: None,
        };
        assert_eq!(
            p.check_registration(true, &mixed),
            Err(Refusal::AttachmentPlatform)
        );
        let silent = ClientReport::default();
        assert_eq!(
            p.check_registration(true, &silent),
            Err(Refusal::AttachmentUnknown)
        );
        let no_rk = ClientReport {
            resident_key: Some(false),
            ..ok.clone()
        };
        assert_eq!(
            p.check_registration(true, &no_rk),
            Err(Refusal::ResidentKey)
        );
        // Absent credProps is trusted (the browser was asked for residentKey).
        let rk_absent = ClientReport {
            resident_key: None,
            ..ok
        };
        assert_eq!(p.check_registration(true, &rk_absent), Ok(()));
        let roaming = PasskeyPolicy {
            attachment: Attachment::CrossPlatform,
            ..PasskeyPolicy::default()
        };
        assert_eq!(roaming.check_registration(false, &usb), Ok(()));
        assert_eq!(
            roaming.check_registration(
                false,
                &ClientReport {
                    attachment: Some("platform"),
                    transports: "internal,hybrid",
                    resident_key: None
                }
            ),
            Err(Refusal::AttachmentCrossPlatform)
        );
        // The default policy accepts what it always accepted.
        assert_eq!(
            PasskeyPolicy::default().check_registration(false, &ClientReport::default()),
            Ok(())
        );
    }

    #[test]
    fn transports_map_to_attachment() {
        assert_eq!(
            attachment_from_transports("internal"),
            Some(Attachment::Platform)
        );
        assert_eq!(
            attachment_from_transports("hybrid,internal"),
            Some(Attachment::Platform)
        );
        assert_eq!(
            attachment_from_transports("usb, nfc"),
            Some(Attachment::CrossPlatform)
        );
        assert_eq!(attachment_from_transports(""), None);
    }

    #[test]
    fn grandfathering_allows_by_default_and_refuses_when_asked() {
        let old = StoredCredential {
            transports: "usb",
            discoverable: false,
            uv_seen: false,
        };
        let p = strict();
        assert_eq!(
            p.credential_gaps(old),
            vec![
                Refusal::UserVerification,
                Refusal::AttachmentPlatform,
                Refusal::ResidentKey
            ]
        );
        // Default legacy = allow: still signs in, without a UV demand it never met.
        assert_eq!(p.assertion_rule(old), Ok(UserVerification::Preferred));
        // Once UV has been seen it may not be dropped.
        let seen = StoredCredential {
            uv_seen: true,
            ..old
        };
        assert_eq!(p.assertion_rule(seen), Ok(UserVerification::Required));
        let refuse = PasskeyPolicy {
            legacy: Legacy::Refuse,
            ..p
        };
        assert_eq!(refuse.assertion_rule(old), Err(Refusal::LegacyCredential));
        let compliant = StoredCredential {
            transports: "internal",
            discoverable: true,
            uv_seen: true,
        };
        assert!(refuse.credential_gaps(compliant).is_empty());
        assert_eq!(
            refuse.assertion_rule(compliant),
            Ok(UserVerification::Required)
        );
        // A credential that has not shown UV yet is asked for it under
        // `refuse` (the signed flag then decides), not refused outright.
        let fresh = StoredCredential {
            uv_seen: false,
            ..compliant
        };
        assert_eq!(refuse.assertion_rule(fresh), Ok(UserVerification::Required));
    }

    #[test]
    fn last_factor_rule() {
        let mut p = PasskeyPolicy::default();
        assert_eq!(p.check_last_factor_removal(false), Ok(()));
        assert_eq!(
            p.check_last_factor_removal(true),
            Err(Refusal::LastFactorRequired)
        );
        p.last_factor = LastFactor::Block;
        assert_eq!(
            p.check_last_factor_removal(false),
            Err(Refusal::LastFactorBlocked)
        );
        assert_eq!(
            p.check_last_factor_removal(true),
            Err(Refusal::LastFactorRequired)
        );
    }

    #[test]
    fn parsers_fall_back_to_the_defaults() {
        assert_eq!(parse_attachment("bogus"), Attachment::Any);
        assert_eq!(
            parse_requirement("bogus", Requirement::Preferred),
            Requirement::Preferred
        );
        assert_eq!(parse_legacy("bogus"), Legacy::Allow);
        assert_eq!(parse_last_factor("bogus"), LastFactor::Allow);
        // The t39-e6 value is read as `block`.
        assert_eq!(parse_last_factor("requireRecoveryCodes"), LastFactor::Block);
        assert_eq!(parse_last_factor("block"), LastFactor::Block);
    }
}
