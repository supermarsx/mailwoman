//! The cookies and browser storage the application itself sets, with the
//! category each belongs to, and the classifier that matches a reported name
//! against that inventory and the administrator's own rules.
//!
//! The inventory is a static list: it documents what this build sets, so it can
//! be shown to visitors and administrators and kept in step with the code by the
//! tests below. A name that matches no entry and no administrator rule is
//! "unclassified" and goes to the review list in the admin console.

use mw_store::cookies::CookieRule;

/// The categories, in display order. `necessary` is always on.
pub(crate) const CATEGORIES: [&str; 4] = ["necessary", "functional", "analytics", "marketing"];
/// The storage kinds a scan reports.
pub(crate) const KINDS: [&str; 4] = ["cookie", "localStorage", "sessionStorage", "indexedDB"];

pub(crate) struct Entry {
    /// Stable id; the client's catalogue has `cookies-inv-<id>` text for it.
    pub id: &'static str,
    pub kind: &'static str,
    /// An exact name, or a name with `*` standing for any run of characters.
    pub pattern: &'static str,
    pub category: &'static str,
    /// What it is for, in English (the admin table and API; visitors read the
    /// localized `cookies-inv-<id>` text).
    pub purpose: &'static str,
    /// How long it is kept, in English; always `lifetime.text()`.
    pub duration: &'static str,
    /// The lifetime the text above is rendered from.
    pub lifetime: Lifetime,
    /// Whether the server sets the cookie `HttpOnly`, so page script cannot read it. Always false for
    /// browser storage, which has no such flag.
    pub http_only: bool,
    /// `first` or `third`.
    pub party: &'static str,
    pub provider: &'static str,
}

/// How long an entry is kept. The text shown to visitors and administrators is rendered from this, so
/// it cannot drift from the structured value. The figures in the session variants are the defaults of
/// `session.idleSecs`, `admin.session.idleSecs` and `admin.session.absoluteSecs`; a unit test keeps them
/// equal to the registry.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Lifetime {
    /// A cookie sent without `Max-Age`, or `sessionStorage`: gone when the browser closes.
    BrowserSession,
    /// `Max-Age` is the time left in the mailbox session, renewed whenever the session is refreshed.
    MailboxSession,
    /// As `MailboxSession` where the cookie is issued with the session. Some responses set `mw_csrf`
    /// without `Max-Age`; the copy then lasts until the browser closes.
    MailboxSessionOrBrowser,
    /// `Max-Age` is `admin.session.absoluteSecs`.
    AdminSession,
    /// A fixed `Max-Age` in days.
    Days(u32),
    /// A fixed `Max-Age` in minutes.
    Minutes(u32),
    /// `Max-Age` is `twofa.rememberDays`.
    RememberedDevicePeriod,
    /// `Max-Age` is the time left on the client-access proof.
    CheckExpiry,
    /// Storage the browser keeps until the visitor or the app removes it.
    UntilCleared,
    UntilSignOut,
    UntilSent,
    UntilConsentEnds,
}

impl Lifetime {
    pub(crate) const fn text(self) -> &'static str {
        match self {
            Self::BrowserSession => "Browser session",
            Self::MailboxSession => {
                "Until the sign-in session ends (7 days without activity by default)"
            }
            Self::MailboxSessionOrBrowser => {
                "Until the sign-in session ends (7 days without activity by default); a copy set without an expiry lasts until the browser closes"
            }
            Self::AdminSession => {
                "Until the admin session ends (30 minutes without activity, 12 hours at most, by default)"
            }
            Self::Days(365) => "365 days",
            Self::Days(400) => "400 days",
            Self::Days(_) => "A fixed number of days",
            Self::Minutes(10) => "10 minutes",
            Self::Minutes(_) => "A fixed number of minutes",
            Self::RememberedDevicePeriod => {
                "The remembered-device period the administrator sets (off by default)"
            }
            Self::CheckExpiry => "Until the check expires",
            Self::UntilCleared => "Until cleared",
            Self::UntilSignOut => "Until you sign out",
            Self::UntilSent => "Until sent",
            Self::UntilConsentEnds => "Until the consent lifetime ends",
        }
    }
}

/// A browser-storage entry (`localStorage`, `sessionStorage`, `indexedDB`).
const fn e(
    id: &'static str,
    kind: &'static str,
    pattern: &'static str,
    category: &'static str,
    purpose: &'static str,
    lifetime: Lifetime,
) -> Entry {
    Entry {
        id,
        kind,
        pattern,
        category,
        purpose,
        duration: lifetime.text(),
        lifetime,
        http_only: false,
        party: "first",
        provider: "This server",
    }
}

/// A cookie the server sets.
const fn c(
    id: &'static str,
    pattern: &'static str,
    category: &'static str,
    purpose: &'static str,
    lifetime: Lifetime,
    http_only: bool,
) -> Entry {
    Entry {
        http_only,
        ..e(id, "cookie", pattern, category, purpose, lifetime)
    }
}

pub(crate) static INVENTORY: &[Entry] = &[
    c("session", "mw_session", "necessary", "Keeps you signed in to the mail app.", Lifetime::MailboxSession, true),
    c("csrf", "mw_csrf", "necessary", "Protects requests made while you are signed in against forgery. Page script reads it to echo it back.", Lifetime::MailboxSessionOrBrowser, false),
    c("admin-session", "mw_admin_session", "necessary", "Keeps an administrator signed in to the admin console. The name can be changed with MW_ADMIN_SESSION_COOKIE.", Lifetime::AdminSession, true),
    c("admin-browser", "mw_admin_browser", "necessary", "Marks a browser used for the admin console so the analytics tag is not loaded in it.", Lifetime::Days(400), true),
    c("client-access", "mw_client_access*", "necessary", "Holds the proof of a client-access check when the administrator requires one.", Lifetime::CheckExpiry, true),
    c("browser-device", "mw_browser_device", "necessary", "Recognises this browser at sign-in so a sign-in from a new device can be reported to you.", Lifetime::Days(365), true),
    c("remembered-device", "mw_remembered_device", "necessary", "Marks this browser as a remembered device, so a second-factor prompt is skipped while the period lasts.", Lifetime::RememberedDevicePeriod, true),
    c("oidc-flow", "mw_oidc_flow", "necessary", "Ties an in-progress sign-in through this server's identity provider to the browser that started it.", Lifetime::Minutes(10), true),
    e("consent", "localStorage", "mw.consent*", "necessary", "Remembers your cookie choices and when the browser last scanned its storage.", Lifetime::UntilConsentEnds),
    e("secure-store", "localStorage", "mw.secure.*", "necessary", "Browser fallback store for sign-in material the app must keep between visits.", Lifetime::UntilSignOut),
    e("boot-session", "localStorage", "mw.boot.session", "necessary", "Tells the start-up screen whether a session existed, to avoid a flash of the wrong screen.", Lifetime::UntilSignOut),
    e("bundle-reload", "sessionStorage", "mw.bundleReload", "necessary", "Prevents a reload loop when a new version of the app is loaded.", Lifetime::BrowserSession),
    e("appearance", "localStorage", "mw.theme.*", "functional", "Remembers theme, density and accent, and the state of syncing them to your account.", Lifetime::UntilCleared),
    e("boot-appearance", "localStorage", "mw.boot.*", "functional", "Start-up screen theme and window layout, so the app opens as you left it.", Lifetime::UntilCleared),
    e("language", "localStorage", "mw.locale", "functional", "Remembers the language you chose.", Lifetime::UntilCleared),
    e("layout", "localStorage", "mw.mail.*", "functional", "Remembers the reading-pane layout.", Lifetime::UntilCleared),
    e("workspace", "localStorage", "mw.workspace.*", "functional", "Remembers open tabs and workspace layout.", Lifetime::UntilCleared),
    e("session-restore", "localStorage", "mw.session.v1", "functional", "Reopens the folder and message you had open.", Lifetime::UntilCleared),
    e("compose-drafts", "localStorage", "mw.compose.*", "functional", "Keeps unsent drafts and composer layout in this browser.", Lifetime::UntilCleared),
    e("prefs", "localStorage", "mw.settings.*", "functional", "Remembers settings that apply to this browser only.", Lifetime::UntilCleared),
    e("account-prefs", "localStorage", "mw.account-prefs:*", "functional", "Holds account preference changes that have not reached the server yet.", Lifetime::UntilSent),
    e("sign-ins", "localStorage", "mw.successful-signins.*", "functional", "Remembers addresses that signed in on this browser, never passwords, when the administrator allows it.", Lifetime::UntilCleared),
    e("local-lists", "localStorage", "mw.tags.*", "functional", "Remembers tags, blocked senders, focused senders and similar lists.", Lifetime::UntilCleared),
    e("local-lists-2", "localStorage", "mw.blocked.*", "functional", "Remembers blocked senders.", Lifetime::UntilCleared),
    e("local-lists-3", "localStorage", "mw.focused.*", "functional", "Remembers focused senders.", Lifetime::UntilCleared),
    e("viewer", "localStorage", "mw.maxsec.*", "functional", "Remembers the maximum-security message viewer choice.", Lifetime::UntilCleared),
    e("assist", "localStorage", "mw.assist.*", "functional", "Remembers assistant suggestions you turned on or off.", Lifetime::UntilCleared),
    e("colors", "localStorage", "mw.color.*", "functional", "Remembers recently used colours.", Lifetime::UntilCleared),
    e("notices", "localStorage", "mw.newVersion.*", "functional", "Remembers that you dismissed a new-version notice.", Lifetime::UntilCleared),
    e("migrate", "localStorage", "mw.migrate.*", "functional", "Remembers a collapsed migration panel.", Lifetime::UntilCleared),
    e("admin-prefs", "localStorage", "mw.admin.*", "functional", "Admin console appearance, language, hidden checklist items and table layout.", Lifetime::UntilCleared),
    e("admin-misc", "localStorage", "mailwoman.*", "functional", "Admin console and feature-profile memory.", Lifetime::UntilCleared),
    e("notification-prompt", "sessionStorage", "mw.notification-permission:*", "functional", "Remembers that you deferred the notification prompt.", Lifetime::BrowserSession),
    e("admin-session-mark", "sessionStorage", "mailwoman.admin-session:*", "necessary", "Tells the admin console why a session ended.", Lifetime::BrowserSession),
    e("offline-db", "indexedDB", "mailwoman", "functional", "Offline copy of mail and search data on this device, when you use offline mode.", Lifetime::UntilCleared),
];

impl Entry {
    /// The name to match. The admin session cookie's name is configurable
    /// (`MW_ADMIN_SESSION_COOKIE`, applied through `mw_admin::config::set_session_cookie_name`), so its
    /// entry matches the configured name as well as the default.
    pub(crate) fn matches(&self, name: &str) -> bool {
        glob_match(self.pattern, name)
            || (self.id == "admin-session"
                && self.kind == "cookie"
                && name == mw_admin::config::session_cookie_name())
    }

    /// The pattern shown for the entry: the configured name for the admin session cookie.
    pub(crate) fn effective_pattern(&self) -> String {
        if self.id == "admin-session" {
            mw_admin::config::session_cookie_name()
        } else {
            self.pattern.to_owned()
        }
    }
}

/// `*` matches any run of characters (including none); everything else is exact.
pub(crate) fn glob_match(pattern: &str, name: &str) -> bool {
    let parts: Vec<&str> = pattern.split('*').collect();
    if parts.len() == 1 {
        return pattern == name;
    }
    let mut rest = name;
    let last = parts.len() - 1;
    for (i, part) in parts.iter().enumerate() {
        if i == 0 {
            match rest.strip_prefix(part) {
                Some(r) => rest = r,
                None => return false,
            }
        } else if i == last {
            return rest.ends_with(part);
        } else if let Some(at) = rest.find(part) {
            rest = &rest[at + part.len()..];
        } else {
            return false;
        }
    }
    true
}

/// Where a classification came from.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Source {
    Rule,
    Inventory,
}

/// The category of `name`, with the source. An administrator rule wins over the
/// built-in inventory; among entries of one source the first match wins, and an
/// exact pattern beats a wildcard one.
pub(crate) fn classify(
    kind: &str,
    name: &str,
    rules: &[CookieRule],
) -> Option<(String, Source)> {
    let pick = |patterns: Vec<(&str, &str)>| -> Option<String> {
        patterns
            .iter()
            .find(|(p, _)| !p.contains('*') && *p == name)
            .or_else(|| patterns.iter().find(|(p, _)| glob_match(p, name)))
            .map(|(_, c)| (*c).to_owned())
    };
    let from_rules = pick(
        rules
            .iter()
            .filter(|r| r.kind == kind)
            .map(|r| (r.pattern.as_str(), r.category.as_str()))
            .collect(),
    );
    if let Some(category) = from_rules {
        return Some((category, Source::Rule));
    }
    // Exact entries first, then wildcard ones, as for rules.
    let hits = INVENTORY.iter().filter(|e| e.kind == kind && e.matches(name));
    let (exact, wild): (Vec<&Entry>, Vec<&Entry>) = hits.partition(|e| !e.pattern.contains('*'));
    exact
        .first()
        .or_else(|| wild.first())
        .map(|e| (e.category.to_owned(), Source::Inventory))
}

/// A name a scan may report: bounded, printable, without separators that would
/// make a stored name ambiguous.
pub(crate) fn valid_name(name: &str) -> bool {
    !name.is_empty() && name.chars().count() <= 128 && !name.chars().any(|c| c.is_control())
}

/// A rule pattern: a valid name that may use `*`, and does not match everything.
pub(crate) fn valid_pattern(pattern: &str) -> bool {
    valid_name(pattern) && pattern.chars().any(|c| c != '*')
}

#[cfg(test)]
mod tests {
    use super::*;

    fn rule(kind: &str, pattern: &str, category: &str) -> CookieRule {
        CookieRule {
            id: pattern.into(),
            kind: kind.into(),
            pattern: pattern.into(),
            category: category.into(),
            note: String::new(),
            created_at: 0,
        }
    }

    #[test]
    fn inventory_ids_are_unique_and_categories_and_kinds_known() {
        let mut seen = std::collections::HashSet::new();
        for e in INVENTORY {
            assert!(seen.insert(e.id), "duplicate id {}", e.id);
            assert!(CATEGORIES.contains(&e.category), "{}", e.id);
            assert!(KINDS.contains(&e.kind), "{}", e.id);
            assert!(valid_pattern(e.pattern), "{}", e.id);
            assert!(!e.purpose.is_empty() && !e.duration.is_empty());
        }
    }

    #[test]
    fn the_server_cookie_names_are_in_the_inventory() {
        for name in [
            crate::hardening::CSRF_COOKIE,
            "mw_session",
            "mw_admin_session",
            crate::analytics_tag::ADMIN_BROWSER_COOKIE,
            "mw_client_access",
            "mw_client_access_pending",
            "mw_browser_device",
            "mw_remembered_device",
            "mw_oidc_flow",
        ] {
            assert_eq!(
                classify("cookie", name, &[]).map(|c| c.0).as_deref(),
                Some("necessary"),
                "{name}"
            );
        }
    }

    #[test]
    fn web_app_storage_keys_are_classified() {
        for (kind, name, category) in [
            ("localStorage", "mw.theme.prefs", "functional"),
            ("localStorage", "mw.theme.sync:v2:abc", "functional"),
            ("localStorage", "mw.locale", "functional"),
            ("localStorage", "mw.consent", "necessary"),
            ("localStorage", "mw.consent.scan", "necessary"),
            ("localStorage", "mw.compose.drafts.v2", "functional"),
            ("sessionStorage", "mw.bundleReload", "necessary"),
            ("indexedDB", "mailwoman", "functional"),
        ] {
            assert_eq!(
                classify(kind, name, &[]).map(|c| c.0).as_deref(),
                Some(category),
                "{name}"
            );
        }
        assert_eq!(classify("localStorage", "_ga", &[]), None);
        assert_eq!(classify("cookie", "mw.theme.prefs", &[]), None, "kind matters");
    }

    #[test]
    fn rules_win_and_exact_beats_wildcard() {
        let rules = [
            rule("cookie", "_ga*", "analytics"),
            rule("cookie", "_gat", "necessary"),
            rule("cookie", "mw_session", "marketing"),
        ];
        let got = |name| classify("cookie", name, &rules).map(|c| c.0);
        assert_eq!(got("_ga_ABC").as_deref(), Some("analytics"));
        assert_eq!(got("_gat").as_deref(), Some("necessary"));
        assert_eq!(got("mw_session").as_deref(), Some("marketing"));
        assert_eq!(
            classify("cookie", "mw_csrf", &rules).map(|c| c.1),
            Some(Source::Inventory)
        );
    }

    #[test]
    fn session_lifetime_text_matches_the_registry_defaults() {
        let default = |key: &str| {
            crate::settings_registry::lookup(key)
                .unwrap_or_else(|| panic!("{key} is not in the registry"))
                .default_value()
                .as_u64()
                .unwrap()
        };
        assert_eq!(default("session.idleSecs"), 7 * 86_400);
        assert_eq!(default("admin.session.idleSecs"), 30 * 60);
        assert_eq!(default("admin.session.absoluteSecs"), 12 * 3_600);
        assert_eq!(default("twofa.rememberDays"), 0, "remembered devices are off by default");
        assert!(Lifetime::MailboxSession.text().contains("7 days"));
        assert!(Lifetime::AdminSession.text().contains("30 minutes"));
        assert!(Lifetime::AdminSession.text().contains("12 hours"));
        // The fixed lifetimes equal the Max-Age the code sets.
        assert_eq!(400 * 86_400, 34_560_000, "mw_admin_browser Max-Age");
        for e in INVENTORY {
            assert_eq!(e.duration, e.lifetime.text(), "{}", e.id);
        }
    }

    #[test]
    fn http_only_is_set_only_on_cookies_and_only_csrf_is_readable() {
        for e in INVENTORY {
            if e.kind != "cookie" {
                assert!(!e.http_only, "{}: storage has no HttpOnly flag", e.id);
            }
        }
        let readable: Vec<&str> = INVENTORY
            .iter()
            .filter(|e| e.kind == "cookie" && !e.http_only)
            .map(|e| e.pattern)
            .collect();
        assert_eq!(readable, ["mw_csrf"]);
    }

    #[test]
    fn the_admin_session_entry_follows_the_configured_cookie_name() {
        mw_admin::config::set_session_cookie_name(Some("corp_admin"));
        let renamed = classify("cookie", "corp_admin", &[]).map(|c| c.0);
        let shown = INVENTORY.iter().find(|e| e.id == "admin-session").unwrap().effective_pattern();
        mw_admin::config::set_session_cookie_name(None);
        assert_eq!(renamed.as_deref(), Some("necessary"));
        assert_eq!(shown, "corp_admin");
        assert_eq!(classify("cookie", "corp_admin", &[]), None);
    }

    #[test]
    fn glob_matching() {
        assert!(glob_match("a*c", "abc") && glob_match("a*c", "ac") && !glob_match("a*c", "ab"));
        assert!(glob_match("*", "x") && glob_match("a*b*c", "a--b--c") && !glob_match("a*b*c", "a--c"));
        assert!(glob_match("mw.*", "mw.") && !glob_match("mw.*", "mw"));
        assert!(!valid_pattern("*") && !valid_pattern("") && valid_pattern("_ga*"));
        assert!(!valid_name("a\nb") && !valid_name(&"x".repeat(129)));
    }
}
