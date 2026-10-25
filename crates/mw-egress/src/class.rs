//! Traffic classes: what an outbound request is *for* (t32 e31b, plan R2 + M11 + DC23).
//!
//! Every outbound request the server makes on its own behalf belongs to exactly
//! one class. The class decides two things and nothing else:
//!   * **which egress route carries it** — an operator route lists the classes it
//!     carries (`egress_proxy.traffic_classes_json`), and a class no live route
//!     lists goes direct ([`crate::plan::EgressPlan::selected_route`]);
//!   * **whether the internal-destination allowlist applies** — only
//!     [`TrafficClass::Webhooks`] and [`TrafficClass::Siem`] may reach an
//!     admin-listed private destination (an on-prem collector); every other class
//!     is held to [`crate::ip_allowed`] without exception.
//!
//! The class is a constant at the call site. It never comes from a request, a
//! header or a stored row that a user can write, which is what keeps "choose a
//! class" from becoming "choose a route".
//!
//! The full set is declared here up front (M11) so lanes that add a caller later
//! (`pki`, `dns_auth`, `unsubscribe`, `siem`, `update_check`, …) only pick a
//! variant; nobody adds one in passing. Mail upstream connections (IMAP, SMTP,
//! ManageSieve, JMAP) are **not** egress classes: they use the per-domain SSH
//! tunnels of plan J.

use std::fmt;

/// What an outbound request is for. The wire and storage spelling is
/// [`TrafficClass::as_str`] (snake_case), matching
/// `egress_proxy.traffic_classes_json` and `egress_internal_allow.traffic_class`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum TrafficClass {
    /// Remote images and remote linked files in mail, fetched by the privacy proxy.
    RemoteContent,
    /// Calendar subscriptions (`webcal://` / ICS URLs).
    IcsFeeds,
    /// OpenPGP key lookups: WKD and VKS keyservers.
    Keyservers,
    /// Outgoing webhooks. May use the internal-destination allowlist.
    Webhooks,
    /// Domain autoconfig/autodiscover detection run from the admin console.
    AdminDetect,
    /// S/MIME revocation: OCSP responders and CRL distribution points.
    Pki,
    /// DNS-over-HTTPS lookups for local authentication re-verification.
    DnsAuth,
    /// One-click List-Unsubscribe requests (RFC 8058).
    Unsubscribe,
    /// Audit log forwarding to a SIEM. May use the internal-destination allowlist.
    Siem,
    /// OAuth token and device-code endpoints of mail providers.
    Oauth,
    /// The release feed polled by the optional update check.
    UpdateCheck,
}

/// The `User-Agent` sent by every class without a more specific one. It names the
/// product and nothing about the deployment.
pub const CLASS_UA: &str = "Mailwoman";

impl TrafficClass {
    /// Every class, in declaration order.
    pub const ALL: [TrafficClass; 11] = [
        TrafficClass::RemoteContent,
        TrafficClass::IcsFeeds,
        TrafficClass::Keyservers,
        TrafficClass::Webhooks,
        TrafficClass::AdminDetect,
        TrafficClass::Pki,
        TrafficClass::DnsAuth,
        TrafficClass::Unsubscribe,
        TrafficClass::Siem,
        TrafficClass::Oauth,
        TrafficClass::UpdateCheck,
    ];

    /// The classes a route carries when its row predates per-class selection:
    /// migration 0033's default for `traffic_classes_json`, which is what the one
    /// route of 26.19 carried (remote images and ICS imports).
    pub const ROUTED_BY_DEFAULT: [TrafficClass; 2] =
        [TrafficClass::RemoteContent, TrafficClass::IcsFeeds];

    /// The storage and wire spelling.
    pub const fn as_str(self) -> &'static str {
        match self {
            TrafficClass::RemoteContent => "remote_content",
            TrafficClass::IcsFeeds => "ics_feeds",
            TrafficClass::Keyservers => "keyservers",
            TrafficClass::Webhooks => "webhooks",
            TrafficClass::AdminDetect => "admin_detect",
            TrafficClass::Pki => "pki",
            TrafficClass::DnsAuth => "dns_auth",
            TrafficClass::Unsubscribe => "unsubscribe",
            TrafficClass::Siem => "siem",
            TrafficClass::Oauth => "oauth",
            TrafficClass::UpdateCheck => "update_check",
        }
    }

    /// Parse the storage spelling. Exact and case-sensitive: a stored class that
    /// does not parse is a corrupt row, not something to guess at.
    pub fn parse(s: &str) -> Option<TrafficClass> {
        TrafficClass::ALL.into_iter().find(|c| c.as_str() == s)
    }

    /// Whether the admin's internal-destination allowlist may widen this class's
    /// address policy. Exactly the two classes migration 0033's CHECK constraint on
    /// `egress_internal_allow.traffic_class` names; keep the two in step.
    pub const fn internal_allow_eligible(self) -> bool {
        matches!(self, TrafficClass::Webhooks | TrafficClass::Siem)
    }

    /// The `User-Agent` this class sends unless the caller supplies its own.
    ///
    /// Remote content and ICS feeds keep [`crate::PROXY_UA`], the value they sent
    /// before classes existed; the others send [`CLASS_UA`]. A caller with a more
    /// specific constant (the update check's `Mailwoman/<version>`) passes it on
    /// the request. Never request-derived either way.
    pub const fn default_user_agent(self) -> &'static str {
        match self {
            TrafficClass::RemoteContent | TrafficClass::IcsFeeds => crate::PROXY_UA,
            _ => CLASS_UA,
        }
    }
}

impl fmt::Display for TrafficClass {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_class_round_trips_through_its_spelling() {
        for c in TrafficClass::ALL {
            assert_eq!(TrafficClass::parse(c.as_str()), Some(c), "{c}");
        }
        let mut names: Vec<_> = TrafficClass::ALL.iter().map(|c| c.as_str()).collect();
        names.sort_unstable();
        names.dedup();
        assert_eq!(
            names.len(),
            TrafficClass::ALL.len(),
            "spellings must be unique"
        );
    }

    #[test]
    fn the_spellings_are_the_ones_the_plan_and_the_migration_use() {
        // M11 + DC23. These strings are stored in `traffic_classes_json` and
        // `egress_internal_allow.traffic_class`, so changing one orphans rows.
        let names: Vec<_> = TrafficClass::ALL.iter().map(|c| c.as_str()).collect();
        assert_eq!(
            names,
            [
                "remote_content",
                "ics_feeds",
                "keyservers",
                "webhooks",
                "admin_detect",
                "pki",
                "dns_auth",
                "unsubscribe",
                "siem",
                "oauth",
                "update_check",
            ]
        );
    }

    #[test]
    fn parse_refuses_near_misses() {
        for s in [
            "",
            "Remote_Content",
            "dns-auth",
            "remote_content ",
            "all",
            "direct",
        ] {
            assert_eq!(TrafficClass::parse(s), None, "{s:?}");
        }
    }

    #[test]
    fn only_webhooks_and_siem_may_use_the_internal_allowlist() {
        let eligible: Vec<_> = TrafficClass::ALL
            .into_iter()
            .filter(|c| c.internal_allow_eligible())
            .collect();
        assert_eq!(eligible, [TrafficClass::Webhooks, TrafficClass::Siem]);
    }

    #[test]
    fn the_default_routed_classes_are_the_migration_default() {
        let names: Vec<_> = TrafficClass::ROUTED_BY_DEFAULT
            .iter()
            .map(|c| c.as_str())
            .collect();
        assert_eq!(names, ["remote_content", "ics_feeds"]);
    }
}
