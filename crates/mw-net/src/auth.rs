//! Password authentication selection before credentials are sent.
//!
//! Selection consumes capabilities from the actual, secured connection. It is
//! deliberately separate from authentication: an authentication rejection never
//! causes this policy to choose another mechanism.

/// Explicit wire mechanisms; protocol commands are distinct from SASL LOGIN.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AuthMechanism {
    Plain,
    Login,
    ScramSha256,
    ScramSha256Plus,
    ImapLogin,
    Pop3UserPass,
}

impl AuthMechanism {
    pub const ALL: [Self; 6] = [
        Self::Plain,
        Self::Login,
        Self::ScramSha256,
        Self::ScramSha256Plus,
        Self::ImapLogin,
        Self::Pop3UserPass,
    ];
    /// Stable configuration and diagnostic name, containing no credentials.
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Plain => "plain",
            Self::Login => "login",
            Self::ScramSha256 => "scram-sha-256",
            Self::ScramSha256Plus => "scram-sha-256-plus",
            Self::ImapLogin => "imap-login",
            Self::Pop3UserPass => "pop3-user-pass",
        }
    }

    /// SASL capability token, or None for a protocol's password command.
    pub fn sasl_name(self) -> Option<&'static str> {
        match self {
            Self::Plain => Some("PLAIN"),
            Self::Login => Some("LOGIN"),
            Self::ScramSha256 => Some("SCRAM-SHA-256"),
            Self::ScramSha256Plus => Some("SCRAM-SHA-256-PLUS"),
            Self::ImapLogin | Self::Pop3UserPass => None,
        }
    }
}

impl std::str::FromStr for AuthMechanism {
    type Err = &'static str;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        Self::ALL
            .into_iter()
            .find(|mechanism| mechanism.as_str() == value)
            .ok_or("unsupported password authentication mechanism")
    }
}

/// Protocol matrix for the mechanisms actually implemented by these clients.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AuthProtocol {
    Imap,
    Pop3,
    Smtp,
    Sieve,
}

impl AuthProtocol {
    pub fn supports(self, mechanism: AuthMechanism) -> bool {
        match mechanism {
            AuthMechanism::ImapLogin => self == Self::Imap,
            AuthMechanism::Pop3UserPass => self == Self::Pop3,
            AuthMechanism::ScramSha256Plus => self != Self::Sieve,
            _ => true,
        }
    }
}

/// An ordered allow-list. A fixed selection is represented by one mechanism.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PasswordAuthPolicy {
    mechanisms: Vec<AuthMechanism>,
}

impl PasswordAuthPolicy {
    pub fn new(mechanisms: Vec<AuthMechanism>) -> Result<Self, &'static str> {
        if mechanisms.is_empty() || mechanisms.len() > 6 {
            return Err("authentication requires one to six mechanisms");
        }
        for (index, mechanism) in mechanisms.iter().enumerate() {
            if mechanisms[..index].contains(mechanism) {
                return Err("authentication mechanisms must be unique");
            }
        }
        Ok(Self { mechanisms })
    }

    pub fn mechanisms(&self) -> &[AuthMechanism] {
        &self.mechanisms
    }

    /// Returns one mechanism without transmitting credentials or attempting auth.
    /// `command_available` means IMAP LOGIN is permitted or POP3 USER/PASS is
    /// supported; SASL mechanisms always require explicit advertisement.
    pub fn select(
        &self,
        protocol: AuthProtocol,
        advertised: impl Fn(&str) -> bool,
        channel_binding: bool,
        command_available: bool,
    ) -> Option<AuthMechanism> {
        self.mechanisms.iter().copied().find(|mechanism| {
            protocol.supports(*mechanism)
                && (*mechanism != AuthMechanism::ScramSha256Plus || channel_binding)
                && mechanism.sasl_name().map_or(command_available, &advertised)
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn exact_order_and_fixed_selection_do_not_invent_a_fallback() {
        let policy =
            PasswordAuthPolicy::new(vec![AuthMechanism::Login, AuthMechanism::Plain]).unwrap();
        assert_eq!(
            policy.select(AuthProtocol::Smtp, |_| true, false, false),
            Some(AuthMechanism::Login)
        );
        let fixed = PasswordAuthPolicy::new(vec![AuthMechanism::ScramSha256]).unwrap();
        assert_eq!(
            fixed.select(AuthProtocol::Smtp, |name| name == "PLAIN", false, false),
            None
        );
    }

    #[test]
    fn plus_requires_real_channel_binding_and_a_supported_protocol() {
        let policy = PasswordAuthPolicy::new(vec![
            AuthMechanism::ScramSha256Plus,
            AuthMechanism::ScramSha256,
        ])
        .unwrap();
        assert_eq!(
            policy.select(AuthProtocol::Imap, |_| true, true, false),
            Some(AuthMechanism::ScramSha256Plus)
        );
        assert_eq!(
            policy.select(AuthProtocol::Imap, |_| true, false, false),
            Some(AuthMechanism::ScramSha256)
        );
        assert_eq!(
            policy.select(AuthProtocol::Sieve, |_| true, true, false),
            Some(AuthMechanism::ScramSha256)
        );
    }

    #[test]
    fn command_auth_is_distinct_and_invalid_lists_are_rejected() {
        assert!(PasswordAuthPolicy::new(vec![]).is_err());
        assert!(PasswordAuthPolicy::new(vec![AuthMechanism::Plain; 2]).is_err());
        let policy = PasswordAuthPolicy::new(vec![AuthMechanism::ImapLogin]).unwrap();
        assert_eq!(
            policy.select(AuthProtocol::Imap, |_| false, false, true),
            Some(AuthMechanism::ImapLogin)
        );
        assert_eq!(
            policy.select(AuthProtocol::Imap, |_| true, true, false),
            None
        );
        assert_eq!(
            policy.select(AuthProtocol::Pop3, |_| true, true, true),
            None
        );
    }
}
