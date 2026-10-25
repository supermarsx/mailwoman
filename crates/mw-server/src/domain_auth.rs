//! Explicit password mechanism selection. OAuth tokens are a separate credential
//! type and cannot be supplied through this password configuration.
use mw_net::{AuthMechanism, AuthProtocol, PasswordAuthPolicy};
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub(crate) enum AuthenticationMode {
    Fixed,
    Auto,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(crate) struct PasswordAuthentication {
    pub mode: AuthenticationMode,
    pub mechanisms: Vec<String>,
}

impl PasswordAuthentication {
    pub fn policy(&self, protocol: AuthProtocol) -> Result<PasswordAuthPolicy, String> {
        if self.mode == AuthenticationMode::Fixed && self.mechanisms.len() != 1 {
            return Err("Fixed authentication requires exactly one mechanism.".into());
        }
        let mechanisms = self
            .mechanisms
            .iter()
            .map(|name| name.parse::<AuthMechanism>())
            .collect::<Result<Vec<_>, _>>()?;
        if mechanisms
            .iter()
            .any(|mechanism| !protocol.supports(*mechanism))
        {
            return Err(
                "The selected authentication mechanism is not supported by this protocol.".into(),
            );
        }
        PasswordAuthPolicy::new(mechanisms).map_err(str::to_string)
    }
}

pub(crate) fn protocol(name: &str) -> Result<AuthProtocol, String> {
    match name {
        "imap" => Ok(AuthProtocol::Imap),
        "pop3" => Ok(AuthProtocol::Pop3),
        "smtp" => Ok(AuthProtocol::Smtp),
        "sieve" => Ok(AuthProtocol::Sieve),
        _ => Err("Password mechanism selection is unavailable for this protocol.".into()),
    }
}

pub(crate) fn capabilities() -> serde_json::Value {
    // Stronger password mechanisms first in the editor's initial selection.
    let preferred = [
        AuthMechanism::ScramSha256Plus,
        AuthMechanism::ScramSha256,
        AuthMechanism::Plain,
        AuthMechanism::Login,
        AuthMechanism::ImapLogin,
        AuthMechanism::Pop3UserPass,
    ];
    let mut protocols = serde_json::Map::new();
    for name in ["imap", "pop3", "smtp", "sieve"] {
        let protocol = protocol(name).expect("known protocol");
        protocols.insert(
            name.into(),
            serde_json::json!(
                preferred
                    .iter()
                    .filter(|m| protocol.supports(**m))
                    .map(|m| m.as_str())
                    .collect::<Vec<_>>()
            ),
        );
    }
    serde_json::json!({"runtimeReady":true,"protocols":protocols})
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn validates_exact_protocol_matrix_and_no_oauth_password_reinterpretation() {
        let parse = |mechanisms: &[&str]| PasswordAuthentication {
            mode: AuthenticationMode::Auto,
            mechanisms: mechanisms.iter().map(|s| s.to_string()).collect(),
        };
        assert!(
            parse(&["scram-sha-256-plus", "plain"])
                .policy(AuthProtocol::Imap)
                .is_ok()
        );
        assert!(
            parse(&["scram-sha-256-plus"])
                .policy(AuthProtocol::Sieve)
                .is_err()
        );
        assert!(parse(&["imap-login"]).policy(AuthProtocol::Pop3).is_err());
        assert!(parse(&["oauthbearer"]).policy(AuthProtocol::Smtp).is_err());
        assert!(
            parse(&["plain", "plain"])
                .policy(AuthProtocol::Smtp)
                .is_err()
        );
        assert!(parse(&[]).policy(AuthProtocol::Smtp).is_err());
        let fixed = PasswordAuthentication {
            mode: AuthenticationMode::Fixed,
            mechanisms: vec!["plain".into(), "login".into()],
        };
        assert!(fixed.policy(AuthProtocol::Smtp).is_err());
    }
}
