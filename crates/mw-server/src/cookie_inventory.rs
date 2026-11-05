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
    pub duration: &'static str,
    /// `first` or `third`.
    pub party: &'static str,
    pub provider: &'static str,
}

const fn e(
    id: &'static str,
    kind: &'static str,
    pattern: &'static str,
    category: &'static str,
    purpose: &'static str,
    duration: &'static str,
) -> Entry {
    Entry {
        id,
        kind,
        pattern,
        category,
        purpose,
        duration,
        party: "first",
        provider: "This server",
    }
}

pub(crate) static INVENTORY: &[Entry] = &[
    e("session", "cookie", "mw_session", "necessary", "Keeps you signed in to the mail app.", "Session"),
    e("csrf", "cookie", "mw_csrf", "necessary", "Protects requests made while you are signed in against forgery.", "Session"),
    e("admin-session", "cookie", "mw_admin_session", "necessary", "Keeps an administrator signed in to the admin console.", "Session"),
    e("admin-browser", "cookie", "mw_admin_browser", "necessary", "Marks a browser used for the admin console so the analytics tag is not loaded in it.", "About 400 days"),
    e("client-access", "cookie", "mw_client_access*", "necessary", "Holds the proof of a client-access check when the administrator requires one.", "Until the check expires"),
    e("consent", "localStorage", "mw.consent*", "necessary", "Remembers your cookie choices and when the browser last scanned its storage.", "Until the consent lifetime ends"),
    e("secure-store", "localStorage", "mw.secure.*", "necessary", "Browser fallback store for sign-in material the app must keep between visits.", "Until you sign out"),
    e("boot-session", "localStorage", "mw.boot.session", "necessary", "Tells the start-up screen whether a session existed, to avoid a flash of the wrong screen.", "Until you sign out"),
    e("bundle-reload", "sessionStorage", "mw.bundleReload", "necessary", "Prevents a reload loop when a new version of the app is loaded.", "Browser session"),
    e("appearance", "localStorage", "mw.theme.*", "functional", "Remembers theme, density and accent, and the state of syncing them to your account.", "Until cleared"),
    e("boot-appearance", "localStorage", "mw.boot.*", "functional", "Start-up screen theme and window layout, so the app opens as you left it.", "Until cleared"),
    e("language", "localStorage", "mw.locale", "functional", "Remembers the language you chose.", "Until cleared"),
    e("layout", "localStorage", "mw.mail.*", "functional", "Remembers the reading-pane layout.", "Until cleared"),
    e("workspace", "localStorage", "mw.workspace.*", "functional", "Remembers open tabs and workspace layout.", "Until cleared"),
    e("session-restore", "localStorage", "mw.session.v1", "functional", "Reopens the folder and message you had open.", "Until cleared"),
    e("compose-drafts", "localStorage", "mw.compose.*", "functional", "Keeps unsent drafts and composer layout in this browser.", "Until cleared"),
    e("prefs", "localStorage", "mw.settings.*", "functional", "Remembers settings that apply to this browser only.", "Until cleared"),
    e("account-prefs", "localStorage", "mw.account-prefs:*", "functional", "Holds account preference changes that have not reached the server yet.", "Until sent"),
    e("sign-ins", "localStorage", "mw.successful-signins.*", "functional", "Remembers addresses that signed in on this browser, never passwords, when the administrator allows it.", "Until cleared"),
    e("local-lists", "localStorage", "mw.tags.*", "functional", "Remembers tags, blocked senders, focused senders and similar lists.", "Until cleared"),
    e("local-lists-2", "localStorage", "mw.blocked.*", "functional", "Remembers blocked senders.", "Until cleared"),
    e("local-lists-3", "localStorage", "mw.focused.*", "functional", "Remembers focused senders.", "Until cleared"),
    e("viewer", "localStorage", "mw.maxsec.*", "functional", "Remembers the maximum-security message viewer choice.", "Until cleared"),
    e("assist", "localStorage", "mw.assist.*", "functional", "Remembers assistant suggestions you turned on or off.", "Until cleared"),
    e("colors", "localStorage", "mw.color.*", "functional", "Remembers recently used colours.", "Until cleared"),
    e("notices", "localStorage", "mw.newVersion.*", "functional", "Remembers that you dismissed a new-version notice.", "Until cleared"),
    e("migrate", "localStorage", "mw.migrate.*", "functional", "Remembers a collapsed migration panel.", "Until cleared"),
    e("admin-prefs", "localStorage", "mw.admin.*", "functional", "Admin console appearance, language, hidden checklist items and table layout.", "Until cleared"),
    e("admin-misc", "localStorage", "mailwoman.*", "functional", "Admin console and feature-profile memory.", "Until cleared"),
    e("notification-prompt", "sessionStorage", "mw.notification-permission:*", "functional", "Remembers that you deferred the notification prompt.", "Browser session"),
    e("admin-session-mark", "sessionStorage", "mailwoman.admin-session:*", "necessary", "Tells the admin console why a session ended.", "Browser session"),
    e("offline-db", "indexedDB", "mailwoman", "functional", "Offline copy of mail and search data on this device, when you use offline mode.", "Until cleared"),
];

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
    pick(
        INVENTORY
            .iter()
            .filter(|e| e.kind == kind)
            .map(|e| (e.pattern, e.category))
            .collect(),
    )
    .map(|c| (c, Source::Inventory))
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
    fn glob_matching() {
        assert!(glob_match("a*c", "abc") && glob_match("a*c", "ac") && !glob_match("a*c", "ab"));
        assert!(glob_match("*", "x") && glob_match("a*b*c", "a--b--c") && !glob_match("a*b*c", "a--c"));
        assert!(glob_match("mw.*", "mw.") && !glob_match("mw.*", "mw"));
        assert!(!valid_pattern("*") && !valid_pattern("") && valid_pattern("_ga*"));
        assert!(!valid_name("a\nb") && !valid_name(&"x".repeat(129)));
    }
}
