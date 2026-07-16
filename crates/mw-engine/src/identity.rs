//! Sending identities (plan §0.7, §2.1) — multiple from-addresses, server-pulled
//! allowed-froms, and signature templates.
//!
//! Beyond the single `configured` identity seeded from the account's own address,
//! the deployment advertises additional allowed-froms through `MW_ALLOWED_FROMS`
//! (source `"server"`). A JMAP server advertises the set of identities a user may
//! send as; the standards backends (IMAP/POP3) carry no such advertisement, so the
//! deployment supplies the allowed-from set via this config. `Identity/get`/`query`
//! merge both.

use serde::{Deserialize, Serialize};

/// A sending identity (§2.1).
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Identity {
    pub id: String,
    pub name: String,
    pub email: String,
    pub reply_to: Option<String>,
    pub signature_html: Option<String>,
    pub signature_text: Option<String>,
    pub sent_mailbox_id: Option<String>,
}

/// One deployment/server-advertised allowed-from (source `"server"`), parsed from
/// `MW_ALLOWED_FROMS`.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ServerIdentity {
    pub email: String,
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub reply_to: Option<String>,
    #[serde(default)]
    pub signature_html: Option<String>,
    #[serde(default)]
    pub signature_text: Option<String>,
}

/// Load the deployment's server-advertised allowed-froms from `MW_ALLOWED_FROMS`.
/// Unset ⇒ none (only the configured seed remains). See [`parse_server_identities`]
/// for the accepted value forms.
pub fn load_server_identities() -> Vec<ServerIdentity> {
    std::env::var("MW_ALLOWED_FROMS")
        .ok()
        .map(|v| parse_server_identities(&v))
        .unwrap_or_default()
}

/// Parse a `MW_ALLOWED_FROMS` value. It is EITHER inline JSON (a `[ServerIdentity]`
/// array) OR a plain list of addresses separated by commas / whitespace / newlines.
/// Empty/unparseable ⇒ no server identities.
pub fn parse_server_identities(value: &str) -> Vec<ServerIdentity> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return Vec::new();
    }
    if trimmed.starts_with('[') {
        return serde_json::from_str(trimmed).unwrap_or_default();
    }
    trimmed
        .split(|c: char| c == ',' || c.is_whitespace())
        .filter(|s| !s.is_empty())
        .map(|addr| ServerIdentity {
            email: addr.to_string(),
            ..Default::default()
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_a_plain_address_list() {
        let ids =
            parse_server_identities("sales@example.com, ceo@example.com\nnoreply@example.com");
        let emails: Vec<&str> = ids.iter().map(|i| i.email.as_str()).collect();
        assert_eq!(
            emails,
            [
                "sales@example.com",
                "ceo@example.com",
                "noreply@example.com"
            ]
        );
        assert!(ids.iter().all(|i| i.name.is_empty()));
    }

    #[test]
    fn parses_inline_json_with_display_names() {
        let ids = parse_server_identities(
            r#"[{"email":"team@example.com","name":"Team","replyTo":"help@example.com"}]"#,
        );
        assert_eq!(ids.len(), 1);
        assert_eq!(ids[0].email, "team@example.com");
        assert_eq!(ids[0].name, "Team");
        assert_eq!(ids[0].reply_to.as_deref(), Some("help@example.com"));
    }

    #[test]
    fn blank_or_garbage_yields_no_identities() {
        assert!(parse_server_identities("   ").is_empty());
        assert!(parse_server_identities("[not json").is_empty());
    }
}
