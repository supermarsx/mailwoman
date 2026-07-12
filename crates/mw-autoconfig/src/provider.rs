//! Offline provider database (plan §0 rung 4) — the bundled fallback when no
//! network discovery method resolves. Big providers (Gmail, Outlook, Yahoo,
//! Fastmail, iCloud) are stable enough to ship as data.

use std::collections::HashMap;
use std::sync::OnceLock;

use serde::Deserialize;

use crate::{AccountCandidate, DiscoverySource, ServerSpec};

/// One provider entry. `ServerSpec`/`AuthMethod` deserialize straight from the
/// bundled JSON (tls: `implicit|start-tls|none`, auth: `password|oauth2`).
#[derive(Debug, Clone, Deserialize)]
struct Entry {
    imap: ServerSpec,
    #[serde(default)]
    pop3: Option<ServerSpec>,
    smtp: ServerSpec,
    auth: crate::AuthMethod,
}

const DB_JSON: &str = include_str!("../data/provider-db.json");

fn db() -> &'static HashMap<String, Entry> {
    static DB: OnceLock<HashMap<String, Entry>> = OnceLock::new();
    DB.get_or_init(|| {
        let raw: HashMap<String, serde_json::Value> =
            serde_json::from_str(DB_JSON).expect("bundled provider-db.json is valid");
        raw.into_iter()
            .filter(|(k, _)| !k.starts_with('_'))
            .filter_map(|(k, v)| serde_json::from_value(v).ok().map(|e| (k, e)))
            .collect()
    })
}

/// Look up a domain (case-insensitive) in the offline provider DB.
pub fn lookup(domain: &str) -> Option<AccountCandidate> {
    let entry = db().get(&domain.to_ascii_lowercase())?;
    Some(AccountCandidate {
        imap: entry.imap.clone(),
        pop3: entry.pop3.clone(),
        smtp: entry.smtp.clone(),
        auth: entry.auth.clone(),
        source: DiscoverySource::ProviderDb,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::TlsMode;

    #[test]
    fn known_provider_resolves() {
        let c = lookup("Gmail.COM").expect("gmail is in the db");
        assert_eq!(c.imap.host, "imap.gmail.com");
        assert_eq!(c.imap.port, 993);
        assert_eq!(c.imap.tls, TlsMode::Implicit);
        assert_eq!(c.auth, crate::AuthMethod::OAuth2);
        assert_eq!(c.source, DiscoverySource::ProviderDb);
    }

    #[test]
    fn start_tls_and_missing_pop3_parse() {
        let c = lookup("icloud.com").unwrap();
        assert_eq!(c.smtp.tls, TlsMode::StartTls);
        assert!(c.pop3.is_none());
    }

    #[test]
    fn unknown_domain_is_none() {
        assert!(lookup("no-such-domain.invalid").is_none());
    }
}
