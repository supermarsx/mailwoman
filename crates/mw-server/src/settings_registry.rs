//! The settings registry (26.20 t32-e10b, worker f01; t32-admin §2.4, §12, §13,
//! t32-schedule §12.9, §13.4, §13.10, t32-q §8 A3/A4/A7/A15).
//!
//! One entry per setting key: its type and bounds, built-in default, the scopes
//! it may be set at, whether a user may override it, the environment variable that
//! pins it or stands in as its fallback, and the route that owns it when the
//! generic settings API must not write it. Values are JSON; every write through
//! any route is validated here first ([`validate`]).
//!
//! Resolution of one key for one person lives in
//! [`mw_store::settings_policy::resolve`]; [`effective_for`] and
//! [`effective_all`] feed it the stored rows, the default and the environment.
//!
//! # Environment precedence (t32-cfg DC2)
//! * **pin** — a set variable wins; a write of the stored value is refused
//!   `409 {code:"pinnedByEnv", var}`; the stored value applies once the variable is
//!   unset.
//! * **fallback** — the stored value wins; the variable stands in for the
//!   built-in default when nothing is stored.
//! * **boot** — read once at start; a stored value applies after a restart.
//!
//! # Limits (t32-q A3, A15)
//! `limit.*` and `quota.*` keys are per address: their per-user rows use the
//! canonical address as `scope_id`, not the account id. Every writer of a
//! `limit.*` key calls [`ceiling`] first, so a domain-scoped admin cannot raise a
//! limit above what an unscoped admin set.
//!
//! # Hot reload (R-CFG 6)
//! Every value is read from the database at the point of use (`hot: true` in the
//! API) unless the entry says `boot`; nothing caches a value for longer than 30 s.

use serde_json::{Value, json};

use mw_store::settings_policy::{
    ResolveInput, SettingPolicyRow, SettingScope, SettingSource, SettingSubject, SettingValueRow,
    resolve,
};
use mw_store::{Store, StoreError};

/// The scopes a key may be stored at, besides global.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Scopes {
    /// Global only.
    Global,
    /// Global and per domain.
    GlobalDomain,
    /// Global, per domain, and per user by an admin (no user override).
    GlobalDomainAdminUser,
    /// Global, per domain, per user by an admin, and the user's own value.
    GlobalDomainUser,
}

impl Scopes {
    /// Whether a value may be stored at `scope`. Profile and group values follow
    /// the domain rule (feature profiles and groups carry admin values).
    pub fn allows(self, scope: SettingScope) -> bool {
        match scope {
            SettingScope::Domain | SettingScope::Profile | SettingScope::Group => {
                self != Scopes::Global
            }
            SettingScope::AdminUser => matches!(
                self,
                Scopes::GlobalDomainAdminUser | Scopes::GlobalDomainUser
            ),
            SettingScope::User => self == Scopes::GlobalDomainUser,
        }
    }

    /// The wire name.
    pub fn as_str(self) -> &'static str {
        match self {
            Scopes::Global => "global",
            Scopes::GlobalDomain => "global+domain",
            Scopes::GlobalDomainAdminUser => "global+domain+admin_user",
            Scopes::GlobalDomainUser => "global+domain+user",
        }
    }
}

/// How an environment variable relates to the stored value (DC2).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Precedence {
    Pin,
    Fallback,
    Boot,
}

impl Precedence {
    pub fn as_str(self) -> &'static str {
        match self {
            Precedence::Pin => "pin",
            Precedence::Fallback => "fallback",
            Precedence::Boot => "boot",
        }
    }
}

/// Which id a key's per-user rows are stored under (t32-q A15).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScopeId {
    /// `accounts.id`.
    Account,
    /// The canonical address (`limit.*`, `quota.*`).
    Address,
}

impl ScopeId {
    pub fn as_str(self) -> &'static str {
        match self {
            ScopeId::Account => "account",
            ScopeId::Address => "address",
        }
    }
}

/// The shape of a text value.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Text {
    /// Any text without control characters.
    Plain,
    /// `[a-z0-9-]`, may be empty (theme and accent ids).
    Slug,
    /// An `https://` URL.
    HttpsUrl,
    /// An `http://` or `https://` URL.
    HttpUrl,
    /// HTTP(S) JMAP or IMAP/POP3 mail transport URL.
    UpstreamUrl,
    /// An e-mail address.
    Email,
    /// A domain name.
    Domain,
    /// `mailto:` + address, or an `https://` URL.
    Contact,
}

/// A value's type and bounds.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Kind {
    /// An exact owned mailbox ID or upstream folder path; null selects discovery.
    FolderRole,
    Bool,
    Int {
        min: i64,
        max: i64,
    },
    Float {
        min: f64,
        max: f64,
    },
    Enum(&'static [&'static str]),
    Text {
        shape: Text,
        max: usize,
    },
    /// `"browser"` or a shipped locale.
    Locale,
    /// `{ "<locale>": "<text>" }` with shipped locales as keys.
    LocaleMap {
        max: usize,
    },
    /// A strictly increasing list of integers.
    IntList {
        min: i64,
        max: i64,
        max_len: usize,
    },
    StringList {
        max_len: usize,
        max_item: usize,
    },
    /// Bounded attachment extension or MIME rules (MIME supports top-level /*).
    AttachmentList { mime: bool },
    /// `{ "start": 0..=23, "end": 0..=23 }` (UTC hours).
    QuietHours,
    /// `[{ "label": "...", "url": "https://..." }]`.
    Links {
        max_len: usize,
    },
}

impl Kind {
    /// The wire description the admin UI validates against (UX5).
    pub fn describe(&self) -> Value {
        match *self {
            Kind::FolderRole => json!({ "type": "folderRole", "maxLength": 512 }),
            Kind::Bool => json!({ "type": "bool" }),
            Kind::AttachmentList { .. } => json!({ "type": "stringList", "maxItems": 100, "maxLength": 127 }),
            Kind::StringList { max_len, max_item } => {
                json!({ "type": "stringList", "maxItems": max_len, "maxLength": max_item })
            }
            Kind::Int { min, max } => json!({ "type": "int", "min": min, "max": max }),
            Kind::Float { min, max } => json!({ "type": "float", "min": min, "max": max }),
            Kind::Enum(values) => json!({ "type": "enum", "values": values }),
            Kind::Text { shape, max } => json!({
                "type": "text",
                "format": match shape {
                    Text::Plain => "plain",
                    Text::Slug => "slug",
                    Text::HttpsUrl => "httpsUrl",
                    Text::HttpUrl => "httpUrl",
                    Text::UpstreamUrl => "upstreamUrl",
                    Text::Email => "email",
                    Text::Domain => "domain",
                    Text::Contact => "contact",
                },
                "maxLength": max,
            }),
            Kind::Locale => json!({ "type": "locale", "values": locale_values() }),
            Kind::LocaleMap { max } => {
                json!({ "type": "localeMap", "locales": LOCALES, "maxLength": max })
            }
            Kind::IntList { min, max, max_len } => {
                json!({ "type": "intList", "min": min, "max": max, "maxItems": max_len })
            }
            Kind::QuietHours => json!({ "type": "quietHours" }),
            Kind::Links { max_len } => json!({ "type": "links", "maxItems": max_len }),
        }
    }
}

/// The shipped web locales (`apps/web/src/i18n/locales.ts`).
pub const LOCALES: &[&str] = &[
    "en", "de", "fr", "es", "pt-BR", "nl", "it", "pl", "ru", "uk", "zh", "ja", "ar",
];

fn locale_values() -> Vec<&'static str> {
    std::iter::once("browser")
        .chain(LOCALES.iter().copied())
        .collect()
}

/// One registry entry.
#[derive(Debug, Clone, Copy)]
pub struct SettingDef {
    /// The key, or for a family the prefix followed by `*` (`serverText.*`).
    pub key: &'static str,
    pub kind: Kind,
    /// The built-in default, as JSON text.
    pub default: &'static str,
    /// Whether `null` is a valid value ("not set").
    pub nullable: bool,
    pub scopes: Scopes,
    /// Whether users may override the value when no admin says otherwise. Only
    /// meaningful for [`Scopes::GlobalDomainUser`].
    pub user_override: bool,
    pub category: &'static str,
    /// What reads the value (documentation for the admin UI and the catalogue).
    pub maps_to: &'static str,
    pub env: Option<(&'static str, Precedence)>,
    /// The admin route (a suffix of the console path, for
    /// `mw_admin::config::admin_route`) that owns writes of this key; the generic
    /// settings routes refuse it with `403 {code:"useDedicatedRoute", route}`
    /// (t32-q A4).
    pub dedicated_route: Option<&'static str>,
    pub scope_id: ScopeId,
}

impl SettingDef {
    /// Whether this entry is a family (`serverText.*`).
    pub fn is_family(&self) -> bool {
        self.key.ends_with(".*")
    }

    /// Whether the value is returned by `GET /api/settings` to the user.
    pub fn user_visible(&self) -> bool {
        self.scopes == Scopes::GlobalDomainUser || PUBLIC_KEYS.contains(&self.key) || self.key == "receipts.dsnAllowed"
    }

    /// Whether a value is re-read at the point of use (R-CFG 6).
    pub fn hot(&self) -> bool {
        !matches!(self.env, Some((_, Precedence::Boot)))
    }

    /// Whether [`ceiling`] guards writes of this key.
    pub fn has_ceiling(&self) -> bool {
        self.key.starts_with("limit.")
    }

    /// The default as JSON.
    pub fn default_value(&self) -> Value {
        serde_json::from_str(self.default).unwrap_or(Value::Null)
    }

    /// The registry entry as the admin API describes it.
    pub fn describe(&self) -> Value {
        json!({
            "key": self.key,
            "family": self.is_family(),
            "kind": self.kind.describe(),
            "default": self.default_value(),
            "nullable": self.nullable,
            "scopes": self.scopes.as_str(),
            "userOverride": self.user_override,
            "category": self.category,
            "mapsTo": self.maps_to,
            "env": self.env.map(|(var, p)| json!({ "var": var, "precedence": p.as_str() })),
            "dedicatedRoute": self.dedicated_route,
            "scopeId": self.scope_id.as_str(),
            "hot": self.hot(),
            "ceiling": self.has_ceiling(),
            "userVisible": self.user_visible(),
        })
    }
}

/// Global-only keys a signed-in user may still read (branding the client shows).
const PUBLIC_KEYS: &[&str] = &["appearance.brandName", "login.notice", "login.showDomainHint", "login.rememberSignins", "login.showRememberedHint"];

/// The admin route (console-path suffix) that owns `signin.newUsers`,
/// `signin.domainMode` and `signin.refusalDetail` (e66): those writes need
/// `signin.policy` and the impact confirmation, which the generic routes do not do.
pub const SIGNIN_POLICY_ROUTE: &str = "/signin-policy";

const BYTES_MAX: i64 = 1 << 50;

macro_rules! def {
    ($key:expr, $kind:expr, $default:expr, $scopes:expr, $cat:expr, $maps:expr) => {
        SettingDef {
            key: $key,
            kind: $kind,
            default: $default,
            nullable: false,
            scopes: $scopes,
            user_override: matches!($scopes, Scopes::GlobalDomainUser),
            category: $cat,
            maps_to: $maps,
            env: None,
            dedicated_route: None,
            scope_id: ScopeId::Account,
        }
    };
}

const fn int(min: i64, max: i64) -> Kind {
    Kind::Int { min, max }
}

const fn text(shape: Text, max: usize) -> Kind {
    Kind::Text { shape, max }
}

const fn pin(mut d: SettingDef, var: &'static str) -> SettingDef {
    d.env = Some((var, Precedence::Pin));
    d
}

const fn nullable(mut d: SettingDef) -> SettingDef {
    d.nullable = true;
    d
}

const fn per_address(mut d: SettingDef) -> SettingDef {
    d.scope_id = ScopeId::Address;
    d
}

const fn dedicated(mut d: SettingDef, route: &'static str) -> SettingDef {
    d.dedicated_route = Some(route);
    d
}

use Scopes::{
    Global as G, GlobalDomain as GD, GlobalDomainAdminUser as GDA, GlobalDomainUser as GDU,
};

/// Every registered key. Order is the admin screen's order within a category.
pub static REGISTRY: &[SettingDef] = &[
    def!("accounts.linkingEnabled", Kind::Bool, "false", GDA, "accounts", "Allow verified linked mailbox accounts; global false is a hard ceiling"),
    def!("accounts.useLinked", Kind::Bool, "true", GDU, "accounts", "Use linked mailboxes when permitted by the administrator"),
    def!("accounts.maxLinked", int(1,16), "5", G, "accounts", "Maximum verified linked mailboxes per primary identity"),
    def!("privacy.remoteImageDownloads", Kind::Bool, "false", GDU, "privacy", "allow deliberate message image downloads through the guarded privacy proxy"),
    def!("appearance.customCssEnabled", Kind::Bool, "false", G, "appearance", "Allow owner-scoped appearance CSS; disabled by default and paused under enforced mailbox appearance locks"),
    def!("appearance.customJsEnabled", Kind::Bool, "false", G, "appearance", "Allow owner-written scripts in an opaque appearance-only worker sandbox; disabled by default"),
    nullable(def!("mail.folderRole.inbox", Kind::FolderRole, "null", GDU, "mail", "inbox folder role override; null keeps automatic discovery, an exact path or owned ID selects the folder")),
    nullable(def!("mail.folderRole.drafts", Kind::FolderRole, "null", GDU, "mail", "drafts folder role override; null keeps automatic discovery, an exact path or owned ID selects the folder")),
    nullable(def!("mail.folderRole.sent", Kind::FolderRole, "null", GDU, "mail", "sent folder role override; null keeps automatic discovery, an exact path or owned ID selects the folder")),
    nullable(def!("mail.folderRole.trash", Kind::FolderRole, "null", GDU, "mail", "trash folder role override; null keeps automatic discovery, an exact path or owned ID selects the folder")),
    nullable(def!("mail.folderRole.junk", Kind::FolderRole, "null", GDU, "mail", "junk folder role override; null keeps automatic discovery, an exact path or owned ID selects the folder")),
    nullable(def!("mail.folderRole.archive", Kind::FolderRole, "null", GDU, "mail", "archive folder role override; null keeps automatic discovery, an exact path or owned ID selects the folder")),
    def!("nextcloud.syncIntervalSecs", int(0,86400), "300", GDU, "integrations", "Nextcloud collection refresh interval; zero is manual only, positive values have a sixty-second minimum"),
    def!("packages.allowed", Kind::Bool, "true", GDA, "integrations", "Allow package tracking; provider activation and separate account opt-in are still required"),
    def!("cap.integration.nextcloud", Kind::Bool, "true", GDA, "integrations", "Nextcloud integration"),
    def!("cap.integration.google", Kind::Bool, "true", GDA, "integrations", "Google OAuth bridge"),
    def!("cap.integration.microsoft", Kind::Bool, "true", GDA, "integrations", "Microsoft OAuth bridge"),
    def!("cap.integration.*", Kind::Bool, "true", GDA, "integrations", "Individual integration permission"),
    def!("cap.ui_plugin.*", Kind::Bool, "true", GDA, "integrations", "Individual UI plugin permission"),
    def!("cap.ai_assistant", Kind::Bool, "true", GDA, "features", "AI assistant"),
    def!("cap.plugins", Kind::Bool, "true", GDA, "features", "Plugins"),
    def!("cap.pim.calendar", Kind::Bool, "true", GDA, "features", "Calendar"),
    def!("cap.pim.contacts", Kind::Bool, "true", GDA, "features", "Contacts"),
    def!("cap.pim.tasks", Kind::Bool, "true", GDA, "features", "Tasks"),
    def!("cap.pim.notes", Kind::Bool, "true", GDA, "features", "Notes"),
    def!("cap.export", Kind::Bool, "true", GDA, "features", "Export"),
    def!("cap.import", Kind::Bool, "true", GDA, "features", "Import"),
    def!("cap.api_keys", Kind::Bool, "true", GDA, "features", "API keys"),
    def!("cap.oauth_apps", Kind::Bool, "true", GDA, "features", "OAuth applications"),
    def!("cap.rules", Kind::Bool, "true", GDA, "features", "Mail rules"),
    def!("cap.rules.forward", Kind::Bool, "true", GDA, "features", "Rule forwarding"),
    def!("cap.remote_images", Kind::Bool, "true", GDA, "features", "Remote images"),
    def!("cap.user_background", Kind::Bool, "false", GDA, "features", "Custom background"),
    def!("cap.user_css", Kind::Bool, "false", GDA, "features", "Custom CSS"),
    def!(
        "privacy.remoteContent",
        Kind::Enum(&["block", "ask", "proxy", "direct"]),
        "\"ask\"",
        GDU,
        "privacy",
        "Reader remote-content mode"
    ),
    def!(
        "privacy.trackers",
        Kind::Bool,
        "true",
        GDU,
        "privacy",
        "Block email trackers"
    ),
    def!(
        "privacy.cleanLinks",
        Kind::Bool,
        "true",
        GDU,
        "privacy",
        "Remove tracking parameters from links"
    ),
    def!(
        "privacy.maxBytes",
        int(65536, 8388608),
        "8388608",
        G,
        "privacy",
        "Largest fetched remote image in bytes"
    ),
    def!(
        "privacy.cacheBytes",
        int(0, 1073741824),
        "33554432",
        G,
        "privacy",
        "Remote image cache size in bytes"
    ),
    def!(
        "privacy.ratePerMin",
        int(10, 6000),
        "120",
        G,
        "privacy",
        "Remote fetches per account per minute"
    ),
    def!(
        "privacy.allowDirect",
        Kind::Bool,
        "false",
        G,
        "privacy",
        "Permit direct remote-content mode with the direct-content capability"
    ),
    def!(
        "cap.remote_content.direct",
        Kind::Bool,
        "false",
        GDA,
        "privacy",
        "Permit this account to load remote images directly"
    ),
    def!(
        "session.maxConcurrent",
        int(1, 100),
        "10",
        GDA,
        "signin",
        "Maximum concurrent mailbox sessions"
    ),
    def!(
        "session.idleSecs",
        int(60, 86400),
        "1800",
        GDA,
        "signin",
        "Mailbox session idle timeout, bounded by the deployment cap"
    ),
    def!(
        "session.absoluteSecs",
        int(300, 1296000),
        "1296000",
        GDA,
        "signin",
        "Mailbox session absolute timeout, bounded by the deployment cap"
    ),
    def!(
        "twofa.rememberDays",
        int(0, 90),
        "0",
        GDA,
        "signin",
        "Remembered-device lifetime; zero disables remembered devices"
    ),
    def!(
        "twofa.passkeysEnabled",
        Kind::Bool,
        "true",
        GDA,
        "signin",
        "Allow passkey authentication"
    ),
    def!(
        "twofa.required",
        Kind::Bool,
        "false",
        GDA,
        "signin",
        "Require a second authentication factor"
    ),
    def!(
        "sieve.maxScriptBytes",
        int(1, 1048576),
        "65536",
        GDA,
        "sieve",
        "Sieve script byte limit"
    ),
    def!(
        "sieve.maxScripts",
        int(1, 100),
        "5",
        GDA,
        "sieve",
        "Sieve script count limit"
    ),
    def!(
        "sieve.maxRedirects",
        int(0, 100),
        "4",
        GDA,
        "sieve",
        "Sieve redirect count limit"
    ),
    def!(
        "sieve.vacationAllowed",
        Kind::Bool,
        "false",
        GDA,
        "sieve",
        "Sieve vacation actions"
    ),
    nullable(def!(
        "sieve.allowedExtensions",
        Kind::StringList {
            max_len: 128,
            max_item: 64
        },
        "null",
        GDA,
        "sieve",
        "Allowed Sieve extensions"
    )),
    def!(
        "cap.sieve.raw_edit",
        Kind::Bool,
        "false",
        GDA,
        "sieve",
        "Raw Sieve editing permission"
    ),
    def!(
        "cap.sieve.forward_external",
        Kind::Bool,
        "false",
        GDA,
        "sieve",
        "Sieve forwarding outside the account domain"
    ),
    def!("mail.externalRecipientWarning", Kind::Enum(&["off", "warn", "confirm"]), "\"warn\"", GDU, "mail", "external recipient composer warning"),
    def!("mail.trustedRecipientDomains", Kind::StringList { max_len: 100, max_item: 253 }, "[]", GDU, "mail", "exact trusted recipient domains for composer warnings"),
    def!("time.displayZone", text(Text::Plain, 64), "\"local\"", GDU, "time", "preferred message display time zone"),
    def!("time.comparisonZone", text(Text::Plain, 64), "\"\"", GDU, "time", "optional second time zone for message and event helpers"),
    def!("time.showHelpers", Kind::Bool, "true", GDU, "time", "show message and event time zone comparisons"),
    def!("shortcuts.enabled", Kind::Bool, "true", GDU, "shortcuts", "enable application keyboard shortcuts"),
    def!("shortcuts.compose", text(Text::Plain, 64), "\"inherit\"", GDU, "shortcuts", "keyboard binding for compose"),
    def!("shortcuts.archive", text(Text::Plain, 64), "\"inherit\"", GDU, "shortcuts", "keyboard binding for archive"),
    def!("shortcuts.reply", text(Text::Plain, 64), "\"inherit\"", GDU, "shortcuts", "keyboard binding for reply"),
    def!("shortcuts.replyAll", text(Text::Plain, 64), "\"inherit\"", GDU, "shortcuts", "keyboard binding for replyAll"),
    def!("shortcuts.forward", text(Text::Plain, 64), "\"inherit\"", GDU, "shortcuts", "keyboard binding for forward"),
    def!("shortcuts.next", text(Text::Plain, 64), "\"inherit\"", GDU, "shortcuts", "keyboard binding for next"),
    def!("shortcuts.previous", text(Text::Plain, 64), "\"inherit\"", GDU, "shortcuts", "keyboard binding for previous"),
    def!("shortcuts.search", text(Text::Plain, 64), "\"inherit\"", GDU, "shortcuts", "keyboard binding for search"),
    def!("shortcuts.refresh", text(Text::Plain, 64), "\"inherit\"", GDU, "shortcuts", "keyboard binding for refresh"),
    def!("shortcuts.settings", text(Text::Plain, 64), "\"inherit\"", GDU, "shortcuts", "keyboard binding for settings"),
    def!("shortcuts.trash", text(Text::Plain, 64), "\"inherit\"", GDU, "shortcuts", "keyboard binding for trash"),
    def!("shortcuts.markRead", text(Text::Plain, 64), "\"inherit\"", GDU, "shortcuts", "keyboard binding for markRead"),
    def!("shortcuts.markUnread", text(Text::Plain, 64), "\"inherit\"", GDU, "shortcuts", "keyboard binding for markUnread"),
    def!("shortcuts.spam", text(Text::Plain, 64), "\"inherit\"", GDU, "shortcuts", "keyboard binding for spam"),
    def!("mail.quoteStyle", Kind::Enum(&["classic", "outlook"]), "\"classic\"", GDU, "mail", "reply and forward header style"),
    // ── Interface (user-facing) ──────────────────────────────────────────────
    def!("mail.quickReply", Kind::Bool, "false", GDU, "mail", "enable the compact message reply composer"),
    def!("mail.requestReadReceipt", Kind::Bool, "false", GDU, "mail", "request a read receipt by default when composing supported mail"),
    def!("mail.requestDeliveryReceipt", Kind::Bool, "false", GDU, "mail", "request delivery status notifications by default when the account supports DSN"),
    def!("mail.showFrom", Kind::Bool, "false", GDU, "mail", "show the From field by default when composing; sending authority is unchanged"),
    def!("mail.modifierSelection", Kind::Bool, "true", GDU, "mail", "select messages with Ctrl, Command, Shift and arrow keys"),
    def!("mail.draftSaveMode", Kind::Enum(&["realtime", "periodic", "manual"]), "\"realtime\"", GDU, "mail", "account draft save scheduling; manual disables automatic upstream writes"),
    def!("mail.draftDebounceMs", int(500,10000), "1000", GDU, "mail", "idle milliseconds before saving an edited account draft"),
    def!("mail.draftIntervalSecs", int(5,3600), "30", GDU, "mail", "periodic account draft save interval in seconds"),
    def!("mail.resumeTabs", Kind::Bool, "true", GDU, "mail", "restore account-owned workspace tab references after reload"),
    def!("mail.dragToFolder", Kind::Bool, "true", GDU, "mail", "move selected messages by dragging them to a folder"),
    def!("mail.folderCounts", Kind::Enum(&["none", "unread", "total", "both"]), "\"both\"", GDU, "mail", "counts displayed beside mail folders"),
    def!("mail.compactFolderBadges", Kind::Bool, "true", GDU, "mail", "smaller count badges for folders other than Inbox"),
    def!("contacts.avatarMode", Kind::Enum(&["off", "initials", "contacts"]), "\"initials\"", GDU, "contacts", "local generated avatars or embedded photos from this account contacts; never remote photo requests"),
    def!("contacts.senderIcons", Kind::Bool, "true", GDU, "contacts", "bundled sender-domain icons with generated initials fallback; domain hints are not authentication evidence"),
    def!("reader.actionStyle", Kind::Enum(&["icons", "labels"]), "\"icons\"", GDU, "reader", "compact reader actions or icons with visible labels"),
    def!("reader.colorMode", Kind::Enum(&["auto", "original", "adaptive", "invert"]), "\"auto\"", GDU, "reader", "message color adaptation inside the isolated reader"),
    def!("reader.fontSize", int(12, 24), "14", GDU, "reader", "message reading font size in pixels"),
    def!("reader.lineHeight", Kind::Float { min: 1.2, max: 2.2 }, "1.6", GDU, "reader", "message reading line height"),
    def!("reader.contentWidth", Kind::Enum(&["comfortable", "wide", "full"]), "\"wide\"", GDU, "reader", "message content width"),
    def!("reader.loading", Kind::Enum(&["skeleton", "indicator", "none"]), "\"skeleton\"", GDU, "reader", "message loading appearance"),
    def!("reader.loadingDelayMs", int(0, 1000), "120", GDU, "reader", "delay before showing message loading feedback"),
    def!(
        "ui.language",
        Kind::Locale,
        "\"browser\"",
        GDU,
        "interface",
        "web client locale (cached in localStorage mw.locale)"
    ),
    def!(
        "appearance.theme",
        text(Text::Slug, 64),
        "\"dark\"",
        GDU,
        "appearance",
        "appearance blob theme/lightTheme/darkTheme; deployment default"
    ),
    def!(
        "appearance.accent",
        text(Text::Slug, 32),
        "\"\"",
        GDU,
        "appearance",
        "appearance blob accentId (empty = the theme's own)"
    ),
    def!(
        "appearance.density",
        Kind::Enum(&["compact", "cozy", "relaxed"]),
        "\"cozy\"",
        GDU,
        "appearance",
        "appearance blob density"
    ),
    def!(
        "appearance.font",
        Kind::Enum(&["default", "system", "serif", "mono", "inter", "roboto", "open-sans", "jost"]),
        "\"default\"",
        GDU,
        "appearance",
        "self-hosted interface font; account choice or administrator default/lock"
    ),
    def!(
        "appearance.brandName",
        text(Text::Plain, 64),
        "\"Mailwoman\"",
        G,
        "appearance",
        "public config brand name"
    ),
    def!(
        "login.showDomainHint",
        Kind::Bool,
        "false",
        G,
        "appearance",
        "public config preferred-domain hint visibility"
    ),
    def!("login.rememberSignins", Kind::Bool, "true", G, "appearance", "allow this browser to remember successful sign-in addresses; never credentials"),
    def!("login.showRememberedHint", Kind::Bool, "false", G, "appearance", "show explanatory text for remembered sign-in addresses"),
    nullable(def!(
        "login.notice",
        text(Text::Plain, 500),
        "null",
        G,
        "appearance",
        "public config sign-in notice"
    )),
    def!(
        "receipts.dsnAllowed",
        Kind::Bool,
        "true",
        GDA,
        "mail",
        "Allow delivery status notification requests at SMTP submission"
    ),
    def!(
        "mail.readReceipts",
        Kind::Enum(&["ask", "never", "always", "contactsOnly", "orgOnly", "orgAlwaysAskOthers"]),
        "\"ask\"",
        GDU,
        "mail",
        "mdn_policy:<account> (MDN/send)"
    ),
    def!(
        "notifications.pushAllowed",
        Kind::Bool,
        "true",
        GD,
        "notifications",
        "account-authorized push registration and delivery"
    ),
    def!(
        "offline.persistentStorageAllowed",
        Kind::Bool,
        "false",
        GD,
        "offline",
        "explicit browser persistent-storage permission request"
    ),
    def!(
        "notifications.enabled",
        Kind::Bool,
        "false",
        GDU,
        "notifications",
        "notification rules enabled flag"
    ),
    def!(
        "offline.retentionDays",
        int(0, 3650),
        "30",
        GDU,
        "offline",
        "web device prefs offlineRetentionDays (client clamps)"
    ),
    def!(
        "offline.budgetMb",
        int(0, 10240),
        "250",
        GDU,
        "offline",
        "web device prefs offlineBudgetMb (client clamps)"
    ),
    def!(
        "assistant.enabled",
        Kind::Bool,
        "true",
        GDU,
        "assistant",
        "below the assist_config deployment ceiling and kill switch"
    ),
    // ── Upstream ─────────────────────────────────────────────────────────────
    SettingDef {
        env: Some(("MW_UPSTREAM_DEFAULT", Precedence::Fallback)),
        ..nullable(def!(
            "upstream.default",
            text(Text::UpstreamUrl, 2048),
            "null",
            G,
            "upstream",
            "deployment default upstream (domain config wins)"
        ))
    },
    def!(
        "upstream.proxyTimeoutSecs",
        int(30, 900),
        "300",
        G,
        "upstream",
        "proxy per-request timeout (lib.rs)"
    ),
    def!("signin.antiBotMode", Kind::Enum(&["off", "mailbox", "admin", "all"]), "\"off\"", G, "signin", "initial sign-in proof-of-work policy"),
    def!("signin.antiBotDifficulty", int(100, 20000), "2000", G, "signin", "maximum proof-of-work solver counter"),
    def!("signin.antiBotExpirySecs", int(60, 600), "180", G, "signin", "single-use challenge lifetime"),
    def!("security.protection.rateMode", Kind::Enum(&["off", "observe", "enforce"]), "\"off\"", G, "security", "Off by default. Observe records budget exhaustion without refusal; enforce returns a temporary 429. Existing protocol concurrency limits remain independent."),
    def!("security.protection.adaptiveMode", Kind::Enum(&["off", "observe", "enforce"]), "\"off\"", G, "security", "Off by default. Observe records explainable typed anomalies and proposed actions; enforce applies bounded temporary throttles or blocks. This is a deterministic counter policy, not an AI classifier."),
    def!("security.protection.clientRequests", int(0, 1000000), "600", G, "security", "Aggregate request allowance across replicas for this verified subject dimension. Zero disables this dimension; source clients use only the trusted proxy address resolver."),
    def!("security.protection.accountRequests", int(0, 1000000), "300", G, "security", "Aggregate request allowance across replicas for this verified subject dimension. Zero disables this dimension; source clients use only the trusted proxy address resolver."),
    def!("security.protection.adminRequests", int(0, 1000000), "120", G, "security", "Aggregate request allowance across replicas for this verified subject dimension. Zero disables this dimension; source clients use only the trusted proxy address resolver."),
    def!("security.protection.rateWindowSecs", int(1, 3600), "60", G, "security", "Fixed request-budget interval. Capacity load shedding and ordinary not-found responses are not adaptive evidence."),
    def!("security.protection.adaptiveClients", Kind::Bool, "true", G, "security", "Enable this subject dimension when adaptive mode is active. Accounts and administrators are counted only after their identity is verified."),
    def!("security.protection.adaptiveAccounts", Kind::Bool, "true", G, "security", "Enable this subject dimension when adaptive mode is active. Accounts and administrators are counted only after their identity is verified."),
    def!("security.protection.adaptiveAdmins", Kind::Bool, "true", G, "security", "Enable this subject dimension when adaptive mode is active. Accounts and administrators are counted only after their identity is verified."),
    def!("security.protection.shortWindowSecs", int(60, 3600), "300", G, "security", "Rolling observation window. Long windows support hours through thirty days; bucket rounding can retain at most one extra sixty-fourth of a window."),
    def!("security.protection.shortMinimumSamples", int(1, 1000000), "30", G, "security", "Minimum observed normal plus anomalous requests before the ratio trigger applies. The absolute anomaly threshold is independent of this minimum."),
    def!("security.protection.shortAbsoluteAnomalies", int(0, 1000000), "20", G, "security", "Trigger when this many selected anomalies accumulate, even when normal requests dilute the ratio. Zero disables this trigger."),
    def!("security.protection.shortRatioPercent", int(0, 100), "50", G, "security", "Trigger when selected anomalies reach this percentage after the minimum sample count. Either the absolute or ratio trigger suffices. Zero disables this trigger."),
    def!("security.protection.longWindowSecs", int(3600, 2592000), "604800", G, "security", "Rolling observation window. Long windows support hours through thirty days; bucket rounding can retain at most one extra sixty-fourth of a window."),
    def!("security.protection.longMinimumSamples", int(1, 1000000), "100", G, "security", "Minimum observed normal plus anomalous requests before the ratio trigger applies. The absolute anomaly threshold is independent of this minimum."),
    def!("security.protection.longAbsoluteAnomalies", int(0, 1000000), "100", G, "security", "Trigger when this many selected anomalies accumulate, even when normal requests dilute the ratio. Zero disables this trigger."),
    def!("security.protection.longRatioPercent", int(0, 100), "25", G, "security", "Trigger when selected anomalies reach this percentage after the minimum sample count. Either the absolute or ratio trigger suffices. Zero disables this trigger."),
    def!("security.protection.normalCapPerBucket", int(1, 1000000), "1000", G, "security", "Bound the normal baseline learned in each rolling-window bucket. Independent absolute anomaly budgets prevent unlimited valid-request padding from hiding abuse."),
    def!("security.protection.signals", Kind::StringList { max_len: 6, max_item: 32 }, "[\"malformed\",\"oversized\",\"credential-rejected\"]", G, "security", "Select authoritative malformed, oversized, credential-rejected, permission-denied, body-deadline or opt-in admin-route-miss observations. Admin route probing excludes entity-not-found responses and static assets; generic status codes, capacity refusals and user-agent strings do not establish abuse."),
    def!("security.protection.throttleSecs", int(1, 300), "3", G, "security", "First threshold crossing starts a fixed throttle deadline. Requests during that deadline do not extend it."),
    def!("security.protection.blockSecs", int(1, 86400), "300", G, "security", "A later threshold crossing before recovery may escalate to this bounded block. No permanent account lock is created."),
    def!("security.protection.decaySecs", int(3600, 2592000), "86400", G, "security", "After this anomaly-free interval and expiry of the current action, escalation returns to its initial stage. Rolling evidence expires according to its configured windows."),
    def!("security.protection.exemptClients", Kind::StringList { max_len: 100, max_item: 64 }, "[]", G, "security", "Exact IP addresses or CIDR networks exempt only the client dimension. Verified account limits still apply. Use deliberately for shared NAT networks; forwarded addresses are trusted only through configured proxies."),
    def!("security.protection.exemptAccounts", Kind::StringList { max_len: 100, max_item: 512 }, "[]", G, "security", "Exact server-verified mailbox account IDs exempt only their account dimension, never the source-client dimension. Email addresses and caller-supplied account parameters are not identity evidence."),
    def!("security.protection.exemptAdmins", Kind::StringList { max_len: 100, max_item: 512 }, "[]", G, "security", "Exact authenticated administrator IDs exempt only their administrator dimension. Existing sign-in failure locks and strict administrator session deadlines remain unchanged."),
    def!("security.protection.storageFailure", Kind::Enum(&["closed", "open"]), "\"closed\"", G, "security", "Closed refuses protected work when an enabled enforcement budget cannot be checked. Open temporarily permits it with a bounded operational warning. Observe-only failures never impose an enforcement action."),
    def!("security.signin.geoEnforceSessions", Kind::Bool, "false", G, "security", "also check current source country on authenticated requests"),
    def!("security.signin.geoScope", Kind::Enum(&["off", "mailbox", "admin", "all"]), "\"off\"", G, "security", "country policy applies only to new sign-ins"),
    def!("security.signin.geoMode", Kind::Enum(&["allow", "deny"]), "\"allow\"", G, "security", "allow only listed countries or deny listed countries"),
    def!("security.signin.geoCountries", Kind::StringList { max_len: 249, max_item: 2 }, "[]", G, "security", "uppercase two-letter country codes"),
    def!("security.signin.geoUnknown", Kind::Enum(&["allow", "deny"]), "\"deny\"", G, "security", "decision for unmapped addresses in a readable local database"),
    def!("security.signin.mailboxFailureLimit", int(1, 100), "10", G, "security", "rejected mailbox credentials per source before a temporary lock"),
    def!("security.signin.adminFailureLimit", int(1, 100), "5", G, "security", "rejected admin credentials per source before a temporary lock"),
    def!("security.signin.failureWindowSecs", int(30, 86400), "900", G, "security", "rolling failed-credential counting window"),
    def!("security.signin.lockSecs", int(30, 86400), "900", G, "security", "fixed temporary source lock duration; rejected retries never extend it"),
    def!("signin.retryCooldownSecs", int(0, 300), "3", G, "signin", "initial sign-in retry cooldown"),
    // ── Sign-in (e66, e70b) ──────────────────────────────────────────────────
    dedicated(
        def!(
            "signin.newUsers",
            Kind::Enum(&["open", "known", "provisioned", "closed"]),
            "\"open\"",
            GD,
            "signin",
            "signin_policy::admit new-user rule"
        ),
        SIGNIN_POLICY_ROUTE,
    ),
    dedicated(
        def!(
            "signin.domainMode",
            Kind::Enum(&["any", "listed"]),
            "\"any\"",
            G,
            "signin",
            "domain gate mode (e12)"
        ),
        SIGNIN_POLICY_ROUTE,
    ),
    dedicated(
        def!(
            "signin.refusalDetail",
            Kind::Enum(&["explain", "uniform"]),
            "\"explain\"",
            G,
            "signin",
            "refusal answers: explain (403 codes) or uniform (401)"
        ),
        SIGNIN_POLICY_ROUTE,
    ),
    dedicated(
        pin(
            def!(
                "signin.ldapBindPrecheck",
                Kind::Bool,
                "false",
                G,
                "signin",
                "LDAP bind pre-check before the mail credential"
            ),
            "MW_LDAP_BIND_AUTH",
        ),
        "/integrations/directory",
    ),
    def!(
        "twofa.pendingTtlSecs",
        int(60, 900),
        "300",
        G,
        "signin",
        "pending 2FA token life (twofa_routes.rs)"
    ),
    def!(
        "twofa.maxAttempts",
        int(3, 10),
        "5",
        G,
        "signin",
        "pending 2FA attempts (twofa_routes.rs)"
    ),
    def!("admin.allowLastSuperadminLoss", Kind::Bool, "false", G, "admin", "Explicitly allow removing the final Super admin access; requires privileged confirmation"),
    def!(
        "admin.session.idleSecs",
        int(300, 86_400),
        "1800",
        G,
        "admin",
        "admin session idle timeout"
    ),
    def!(
        "admin.session.absoluteSecs",
        int(3600, 604_800),
        "43200",
        G,
        "admin",
        "admin session absolute cap"
    ),
    def!(
        "passwd.policy.minLength",
        int(8, 128),
        "12",
        GD,
        "passwords",
        "password policy minimum length (every backend)"
    ),
    def!(
        "passwd.policy.requireUpper",
        Kind::Bool,
        "false",
        GD,
        "passwords",
        "password policy: an upper-case letter"
    ),
    def!(
        "passwd.policy.requireLower",
        Kind::Bool,
        "false",
        GD,
        "passwords",
        "password policy: a lower-case letter"
    ),
    def!(
        "passwd.policy.requireDigit",
        Kind::Bool,
        "false",
        GD,
        "passwords",
        "password policy: a digit"
    ),
    def!(
        "passwd.policy.requireSymbol",
        Kind::Bool,
        "false",
        GD,
        "passwords",
        "password policy: a symbol"
    ),
    // Attachment delivery policy: administrative global/domain scope only.
    def!("attachments.previewAllowed", Kind::Bool, "true", GD, "attachments", "allow attachment previews"),
    def!("attachments.downloadAllowed", Kind::Bool, "true", GD, "attachments", "allow original attachment delivery; disabling also prevents raw previews"),
    def!("attachments.maxBytes", int(0, BYTES_MAX), "0", GD, "attachments", "maximum delivered attachment bytes; zero is unlimited"),
    def!("attachments.allowedExtensions", Kind::AttachmentList { mime: false }, "[]", GD, "attachments", "attachment allowed Extensions; blocked rules take precedence"),
    def!("attachments.allowedMimeTypes", Kind::AttachmentList { mime: true }, "[]", GD, "attachments", "attachment allowed MimeTypes; blocked rules take precedence"),
    def!("attachments.blockedExtensions", Kind::AttachmentList { mime: false }, "[]", GD, "attachments", "attachment blocked Extensions; blocked rules take precedence"),
    def!("attachments.blockedMimeTypes", Kind::AttachmentList { mime: true }, "[]", GD, "attachments", "attachment blocked MimeTypes; blocked rules take precedence"),
    def!("attachments.warnExtensions", Kind::AttachmentList { mime: false }, "[]", GD, "attachments", "attachment warn Extensions; blocked rules take precedence"),
    def!("attachments.warnMimeTypes", Kind::AttachmentList { mime: true }, "[]", GD, "attachments", "attachment warn MimeTypes; blocked rules take precedence"),
    // ── Limits, quotas, usage (e67) ─────────────────────────────────────────
    per_address(def!("limit.attachment_bytes", int(0, BYTES_MAX), "50000000", GDA, "limits", "Attachment bytes; 0 = no limit")),
    per_address(def!("limit.send_per_hour", int(0, 1_000_000), "0", GDA, "limits", "Submissions per hour; 0 = no limit")),
    per_address(def!("limit.recipients_per_message", int(0, 1_000_000), "0", GDA, "limits", "Recipients per message; 0 = no limit")),
    per_address(def!(
        "limit.storage_bytes",
        int(0, BYTES_MAX),
        "0",
        GDA,
        "limits",
        "blob reservations; 0 = no limit"
    )),
    per_address(def!(
        "limit.send_per_day",
        int(0, 1_000_000),
        "0",
        GDA,
        "limits",
        "EmailSubmission/set creates per day; 0 = no limit"
    )),
    per_address(def!(
        "limit.ai_calls_per_day",
        int(0, 1_000_000),
        "0",
        GDA,
        "limits",
        "assistant adapter calls per day; 0 = no limit"
    )),
    per_address(def!(
        "quota.warnThresholds",
        Kind::IntList {
            min: 1,
            max: 100,
            max_len: 5
        },
        "[80,95]",
        GD,
        "limits",
        "usage warning thresholds (percent)"
    )),
    per_address(def!(
        "quota.warnEnabled",
        Kind::Bool,
        "true",
        GD,
        "limits",
        "usage warnings on"
    )),
    def!(
        "usage.retentionDays",
        int(31, 3650),
        "400",
        G,
        "limits",
        "usage counter pruning"
    ),
    def!(
        "usage.meterApi",
        Kind::Bool,
        "true",
        G,
        "limits",
        "API usage counters"
    ),
    def!(
        "usage.measureIntervalHours",
        int(1, 168),
        "6",
        G,
        "limits",
        "storage measurer interval"
    ),
    // ── Tunables (e70c) ──────────────────────────────────────────────────────
    pin(
        def!(
            "assistant.rateLimitPerMin",
            int(0, 10_000),
            "0",
            G,
            "assistant",
            "assistant rate limit; 0 = none (v7_mount.rs)"
        ),
        "MW_ASSIST_RATE_LIMIT_PER_MIN",
    ),
    pin(
        def!(
            "plugins.kv.maxKeyBytes",
            int(1, 1024),
            "256",
            G,
            "plugins",
            "plugin KV key size (v7_mount.rs)"
        ),
        "MW_PLUGIN_KV_MAX_KEY_BYTES",
    ),
    pin(
        def!(
            "plugins.kv.maxValueBytes",
            int(1, 262_144),
            "65536",
            G,
            "plugins",
            "plugin KV value size"
        ),
        "MW_PLUGIN_KV_MAX_VALUE_BYTES",
    ),
    pin(
        def!(
            "plugins.kv.maxTotalBytes",
            int(1, 20_971_520),
            "5242880",
            G,
            "plugins",
            "plugin KV total size"
        ),
        "MW_PLUGIN_KV_MAX_TOTAL_BYTES",
    ),
    pin(
        def!(
            "plugins.kv.maxKeys",
            int(1, 4000),
            "1000",
            G,
            "plugins",
            "plugin KV key count"
        ),
        "MW_PLUGIN_KV_MAX_KEYS",
    ),
    def!(
        "import.maxMessages",
        int(1, 80_000),
        "20000",
        GD,
        "import",
        "import message count (import_routes.rs)"
    ),
    def!(
        "import.maxMessageBytes",
        int(1, 268_435_456),
        "67108864",
        GD,
        "import",
        "import message size (import_routes.rs)"
    ),
    def!(
        "webhooks.maxAttempts",
        int(1, 10),
        "3",
        G,
        "webhooks",
        "webhook delivery attempts"
    ),
    def!(
        "webhooks.timeoutSecs",
        int(1, 60),
        "10",
        G,
        "webhooks",
        "webhook delivery timeout"
    ),
    def!(
        "outbox.maxAttempts",
        int(1, 50),
        "8",
        G,
        "mail",
        "submission attempts (engine dispatcher)"
    ),
    pin(
        def!(
            "realtime.recheckSecs",
            int(5, 300),
            "30",
            G,
            "realtime",
            "push stream re-check period (push.rs)"
        ),
        "MW_REALTIME_RECHECK_SECS",
    ),
    pin(
        def!(
            "privacy.watermark.enabled",
            Kind::Bool,
            "false",
            GD,
            "privacy",
            "reader watermark"
        ),
        "MW_WATERMARK",
    ),
    pin(
        def!(
            "privacy.watermark.opacity",
            Kind::Float {
                min: 0.02,
                max: 0.5
            },
            "0.08",
            GD,
            "privacy",
            "reader watermark opacity"
        ),
        "MW_WATERMARK_OPACITY",
    ),
    def!(
        "privacy.openTracking.retentionDays",
        int(1, 400),
        "90",
        G,
        "privacy",
        "open-tracking token retention"
    ),
    def!(
        "privacy.openTracking.unboundTtlHours",
        int(1, 168),
        "24",
        G,
        "privacy",
        "open-tracking unbound token life"
    ),
    pin(
        nullable(def!(
            "maskedEmail.domain",
            text(Text::Domain, 253),
            "null",
            G,
            "mail",
            "masked-address domain (masked.rs)"
        )),
        "MW_MASKED_EMAIL_DOMAIN",
    ),
    pin(
        nullable(def!(
            "abuse.reportAddress",
            text(Text::Email, 254),
            "null",
            GD,
            "mail",
            "ARF report address"
        )),
        "MW_ABUSE_ADDRESS",
    ),
    pin(
        nullable(def!(
            "dlp.adminAddress",
            text(Text::Email, 254),
            "null",
            G,
            "mail",
            "DLP notice address"
        )),
        "MW_DLP_ADMIN",
    ),
    pin(
        nullable(def!(
            "observability.errorForwardUrl",
            text(Text::HttpsUrl, 2048),
            "null",
            G,
            "observability",
            "client error forwarding (errors.rs)"
        )),
        "MW_ERROR_FORWARD_URL",
    ),
    pin(
        def!(
            "egress.probeUrl",
            text(Text::HttpsUrl, 2048),
            "\"https://example.com/\"",
            G,
            "egress",
            "egress proxy test target"
        ),
        "MW_EGRESS_PROBE_URL",
    ),
    pin(
        nullable(def!(
            "push.vapidContact",
            text(Text::Contact, 320),
            "null",
            G,
            "notifications",
            "VAPID contact (push_relay.rs)"
        )),
        "MW_VAPID_CONTACT",
    ),
    nullable(def!(
        "push.quietHours",
        Kind::QuietHours,
        "null",
        G,
        "notifications",
        "push quiet hours, UTC"
    )),
    // ── Operations (e72, e78, e79, e73a, e83, e70a) ──────────────────────────
    def!(
        "updates.check",
        Kind::Enum(&["off", "notify"]),
        "\"off\"",
        G,
        "updates",
        "daily update check"
    ),
    def!(
        "updates.feedUrl",
        text(Text::HttpsUrl, 2048),
        "\"https://api.github.com/repos/supermarsx/mailwoman/releases\"",
        G,
        "updates",
        "release feed"
    ),
    def!(
        "updates.apply",
        Kind::Enum(&["disabled", "agent"]),
        "\"disabled\"",
        G,
        "updates",
        "update apply through the host agent"
    ),
    def!(
        "backup.retentionCount",
        int(1, 365),
        "7",
        G,
        "backups",
        "SQLite backups kept"
    ),
    nullable(def!(
        "about.securityContact",
        text(Text::Contact, 320),
        "null",
        G,
        "about",
        "security.txt Contact"
    )),
    nullable(def!(
        "about.securityPolicyUrl",
        text(Text::HttpsUrl, 2048),
        "null",
        G,
        "about",
        "security.txt Policy"
    )),
    def!(
        "about.projectLinks",
        Kind::Links { max_len: 10 },
        "[]",
        G,
        "about",
        "About screen links"
    ),
    def!(
        "config.snapshotsKept",
        int(1, 500),
        "20",
        G,
        "config",
        "configuration snapshots kept"
    ),
    def!(
        "jobs.historyDays",
        int(1, 365),
        "30",
        G,
        "jobs",
        "job run history"
    ),
    def!(
        "maintenance.announceMinutesDefault",
        int(0, 10_080),
        "15",
        G,
        "maintenance",
        "maintenance announcement lead time"
    ),
    def!(
        "maintenance.defaultMessage",
        Kind::LocaleMap { max: 500 },
        "{}",
        G,
        "maintenance",
        "maintenance banner text per locale"
    ),
    def!(
        "health.diskFreeMinPercent",
        int(1, 50),
        "5",
        G,
        "health",
        "/readyz disk free floor"
    ),
    def!(
        "observability.otlpMetrics",
        Kind::Bool,
        "false",
        G,
        "observability",
        "OTLP metrics export"
    ),
    def!(
        "metrics.drilldownTopN",
        int(1, 1000),
        "200",
        G,
        "observability",
        "metrics drill-down rows"
    ),
    def!("metrics.defaultRangeSeconds", int(60, 34560000), "3600", G, "observability", "default request history range"),
    def!("metrics.maxRangeSeconds", int(3600, 34560000), "2592000", G, "observability", "maximum request history range"),
    def!(
        "metrics.rollupDays",
        int(1, 400),
        "30",
        G,
        "observability",
        "hourly metric rollups kept"
    ),
    def!(
        "diagnostics.retentionDays",
        int(1, 365),
        "30",
        G,
        "diagnostics",
        "diagnostic runs kept"
    ),
    def!(
        "dataExport.linkHours",
        int(1, 168),
        "24",
        G,
        "users",
        "data export download window"
    ),
    def!("logs.auth.retentionDays", int(1,90), "7", G, "observability", "authentication history days kept"),
    def!("logs.auth.maxEntries", int(1000,100000), "50000", G, "observability", "maximum authentication history entries"),
    // ── Server text (e70c seam) ──────────────────────────────────────────────
    def!(
        "serverText.*",
        Kind::LocaleMap { max: 4000 },
        "{}",
        GD,
        "serverText",
        "server_text(id, locale, domain) overrides"
    ),
];

/// The entry for `key`: an exact entry, or the family whose prefix it has
/// (`serverText.<id>`, id `[a-z0-9-]{1,64}`).
pub fn lookup(key: &str) -> Option<&'static SettingDef> {
    if let Some(d) = REGISTRY.iter().find(|d| !d.is_family() && d.key == key) {
        return Some(d);
    }
    REGISTRY.iter().find(|d| {
        d.is_family()
            && key
                .strip_prefix(&d.key[..d.key.len() - 1])
                .is_some_and(|id| {
                    !id.is_empty()
                        && id.len() <= 64
                        && id
                            .bytes()
                            .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'-')
                })
    })
}

// ─── refusals ─────────────────────────────────────────────────────────────────

/// A refused write: the HTTP status and the `{code, detail, field}` body (R-CFG 1).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Refusal {
    pub status: u16,
    pub code: &'static str,
    pub detail: String,
    /// A JSON pointer into the request body.
    pub field: Option<String>,
    /// Extra members for the body (`var`, `route`, …).
    pub extra: Option<(&'static str, String)>,
}

impl Refusal {
    pub fn invalid(field: &str, detail: impl Into<String>) -> Self {
        Refusal {
            status: 422,
            code: "invalidValue",
            detail: detail.into(),
            field: Some(field.to_string()),
            extra: None,
        }
    }

    pub fn locked(detail: impl Into<String>) -> Self {
        Refusal {
            status: 403,
            code: "setting_locked",
            detail: detail.into(),
            field: None,
            extra: None,
        }
    }

    /// The JSON body.
    pub fn body(&self) -> Value {
        let mut v = json!({ "code": self.code, "detail": self.detail, "field": self.field });
        if let Some((k, val)) = &self.extra {
            v[*k] = Value::String(val.clone());
        }
        v
    }
}

// ─── validation ───────────────────────────────────────────────────────────────

fn no_controls(s: &str) -> bool {
    !s.chars().any(|c| c.is_control() && c != '\n')
}

fn is_email(s: &str) -> bool {
    match s.rsplit_once('@') {
        Some((local, domain)) => {
            !local.is_empty()
                && local.len() <= 64
                && !local.chars().any(|c| c.is_whitespace() || c == '@')
                && is_domain(domain)
        }
        None => false,
    }
}

fn is_domain(s: &str) -> bool {
    let s = s.strip_suffix('.').unwrap_or(s);
    !s.is_empty()
        && s.len() <= 253
        && s.contains('.')
        && s.split('.').all(|label| {
            !label.is_empty()
                && label.len() <= 63
                && !label.starts_with('-')
                && !label.ends_with('-')
                && label
                    .bytes()
                    .all(|b| b.is_ascii_alphanumeric() || b == b'-')
        })
}

fn is_url(s: &str, schemes: &[&str]) -> bool {
    match reqwest::Url::parse(s) {
        Ok(u) => schemes.contains(&u.scheme()) && u.host_str().is_some_and(|h| !h.is_empty()),
        Err(_) => false,
    }
}

fn check_text(shape: Text, s: &str) -> Result<(), &'static str> {
    let ok = match shape {
        Text::Plain => no_controls(s),
        Text::Slug => s
            .bytes()
            .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'-'),
        Text::HttpsUrl => is_url(s, &["https"]),
        Text::HttpUrl => is_url(s, &["https", "http"]),
        Text::UpstreamUrl => is_url(s, &["https", "http", "imap", "imaps", "pop3", "pop3s"]),
        Text::Email => is_email(s),
        Text::Domain => is_domain(s),
        Text::Contact => match s.strip_prefix("mailto:") {
            Some(addr) => is_email(addr),
            None => is_url(s, &["https"]),
        },
    };
    if ok {
        Ok(())
    } else {
        Err(match shape {
            Text::Plain => "must not contain control characters",
            Text::Slug => "may contain only a-z, 0-9 and -",
            Text::HttpsUrl => "must be an https:// URL",
            Text::HttpUrl => "must be an http:// or https:// URL",
            Text::UpstreamUrl => "must be an HTTP(S), IMAP(S), or POP3(S) URL",
            Text::Email => "must be an e-mail address",
            Text::Domain => "must be a domain name",
            Text::Contact => "must be mailto:<address> or an https:// URL",
        })
    }
}

/// Validate `value` for `def`. `field` is the JSON pointer of the value in the
/// request body, used in the refusal. Returns the value to store (numbers are
/// normalised: an integral float is accepted for an int key).
fn valid_shortcut(value: &str) -> bool {
    let value = value.trim();
    if matches!(value, "inherit" | "disabled") { return true; }
    if value.len() > 64 || value.is_empty() { return false; }
    let (prefix, key) = if value == "+" { ("", "+") }
        else if let Some(prefix) = value.strip_suffix("++") { (prefix, "+") }
        else { value.rsplit_once('+').unwrap_or(("", value)) };
    if !prefix.is_empty() && !prefix.split('+').all(|part| matches!(part.trim().to_ascii_lowercase().as_str(), "ctrl" | "cmd" | "meta" | "mod" | "alt" | "option" | "shift")) { return false; }
    (key.chars().count() == 1 && !key.chars().any(char::is_control))
        || matches!(key, "ArrowUp" | "ArrowDown" | "ArrowLeft" | "ArrowRight" | "Home" | "End" | "PageUp" | "PageDown" | "Delete" | "Del" | "Backspace" | "Insert" | "Escape" | "Esc" | "Tab" | "Enter" | "Space" | "F1" | "F2" | "F3" | "F4" | "F5" | "F6" | "F7" | "F8" | "F9" | "F10" | "F11" | "F12")
}

pub fn validate(def: &SettingDef, value: &Value, field: &str) -> Result<Value, Refusal> {
    if value.is_null() {
        return if def.nullable {
            Ok(Value::Null)
        } else {
            Err(Refusal::invalid(field, "a value is required"))
        };
    }
    let bad = |detail: String| Err(Refusal::invalid(field, detail));
    if matches!(def.key, "time.displayZone" | "time.comparisonZone") && !value.as_str().is_some_and(|zone|
        (def.key == "time.displayZone" && zone == "local") || (def.key == "time.comparisonZone" && zone.is_empty()) || zone.parse::<chrono_tz::Tz>().is_ok()) {
        return bad("must be a valid IANA time zone; displayZone also accepts local, comparisonZone also accepts an empty value".into());
    }
    if def.key.starts_with("shortcuts.") && def.key != "shortcuts.enabled"
        && !value.as_str().is_some_and(valid_shortcut) {
        return bad("must be inherit, disabled, or one key with Ctrl, Alt and Shift modifiers".into());
    }
    if def.key == "security.signin.geoCountries" && !value.as_array().is_some_and(|items| items.iter().all(|item| item.as_str().is_some_and(|code| code.len()==2 && code.bytes().all(|b|b.is_ascii_uppercase())))) {
        return bad("must contain uppercase two-letter country codes".into());
    }
    match def.kind {
        Kind::FolderRole => mw_engine::folder_roles::parse_target(value)
            .map(|_| value.clone()).map_err(|detail| Refusal::invalid(field, detail)),
        Kind::Bool => match value {
            Value::Bool(_) => Ok(value.clone()),
            _ => bad("must be true or false".into()),
        },
        Kind::Int { min, max } => {
            let n = value.as_i64().or_else(|| {
                value
                    .as_f64()
                    .filter(|f| f.fract() == 0.0)
                    .map(|f| f as i64)
            });
            match n {
                Some(n) if (min..=max).contains(&n) => Ok(json!(n)),
                Some(_) => bad(format!("must be between {min} and {max}")),
                None => bad("must be a whole number".into()),
            }
        }
        Kind::Float { min, max } => match value.as_f64() {
            Some(f) if f.is_finite() && f >= min && f <= max => Ok(json!(f)),
            Some(_) => bad(format!("must be between {min} and {max}")),
            None => bad("must be a number".into()),
        },
        Kind::Enum(values) => match value.as_str() {
            Some(s) if values.contains(&s) => Ok(value.clone()),
            _ => bad(format!("must be one of: {}", values.join(", "))),
        },
        Kind::Text { shape, max } => match value.as_str() {
            Some(s) if s.chars().count() > max => bad(format!("must be at most {max} characters")),
            Some(s) => match check_text(shape, s) {
                Ok(()) => Ok(value.clone()),
                Err(why) => bad(why.into()),
            },
            None => bad("must be text".into()),
        },
        Kind::Locale => match value.as_str() {
            Some(s) if s == "browser" || LOCALES.contains(&s) => Ok(value.clone()),
            _ => bad(format!("must be browser or one of: {}", LOCALES.join(", "))),
        },
        Kind::LocaleMap { max } => {
            let Some(map) = value.as_object() else {
                return bad("must be an object of locale to text".into());
            };
            for (locale, text) in map {
                let at = format!("{field}/{locale}");
                if !LOCALES.contains(&locale.as_str()) {
                    return Err(Refusal::invalid(
                        &at,
                        format!("{locale} is not a shipped locale"),
                    ));
                }
                match text.as_str() {
                    Some(s) if s.chars().count() <= max && no_controls(s) => {}
                    Some(_) => {
                        return Err(Refusal::invalid(
                            &at,
                            format!("must be at most {max} characters without control characters"),
                        ));
                    }
                    None => return Err(Refusal::invalid(&at, "must be text")),
                }
            }
            Ok(value.clone())
        }
        Kind::IntList { min, max, max_len } => {
            let Some(items) = value.as_array() else {
                return bad("must be a list of whole numbers".into());
            };
            if items.len() > max_len {
                return bad(format!("must have at most {max_len} items"));
            }
            let mut prev = i64::MIN;
            for (i, item) in items.iter().enumerate() {
                let at = format!("{field}/{i}");
                match item.as_i64() {
                    Some(n) if (min..=max).contains(&n) && n > prev => prev = n,
                    Some(n) if n <= prev => {
                        return Err(Refusal::invalid(
                            &at,
                            "must be larger than the item before it",
                        ));
                    }
                    _ => {
                        return Err(Refusal::invalid(
                            &at,
                            format!("must be a whole number between {min} and {max}"),
                        ));
                    }
                }
            }
            Ok(value.clone())
        }
        Kind::AttachmentList { mime } => {
            let Some(items) = value.as_array() else { return bad("must be a list of attachment rules".into()); };
            if items.len() > 100 { return bad("at most 100 attachment rules are allowed".into()); }
            let mut seen = std::collections::BTreeSet::new();
            for item in items {
                let Some(rule) = item.as_str() else { return bad("attachment rules must be text".into()); };
                if !crate::attachment_policy::valid_rule(rule, mime) || !seen.insert(rule) {
                    return bad("invalid or duplicate attachment rule; use lowercase extensions or MIME types".into());
                }
            }
            Ok(value.clone())
        }
        Kind::StringList { max_len, max_item } => {
            let Some(items) = value.as_array() else {
                return bad("must be a list of text values".into());
            };
            if items.len() > max_len {
                return bad(format!("must have at most {max_len} items"));
            }
            let mut seen = std::collections::BTreeSet::new();
            for (index, item) in items.iter().enumerate() {
                let Some(text) = item.as_str() else {
                    return Err(Refusal::invalid(
                        &format!("{field}/{index}"),
                        "must be text",
                    ));
                };
                if text.is_empty()
                    || text.len() > max_item
                    || !text.bytes().all(|b| {
                        if def.key == "security.signin.geoCountries" { b.is_ascii_uppercase() }
                        else if matches!(def.key, "security.protection.exemptClients" | "security.protection.exemptAccounts" | "security.protection.exemptAdmins") { !b.is_ascii_control() }
                        else { b.is_ascii_lowercase() || b.is_ascii_digit() || matches!(b, b'-' | b'.') }
                    })
                    || (def.key == "security.protection.exemptClients" && crate::scope_mw::proxy::Cidr::parse(text).is_none())
                    || (def.key == "security.protection.signals" && !matches!(text, "malformed" | "oversized" | "credential-rejected" | "permission-denied" | "body-deadline" | "admin-route-miss"))
                    || !seen.insert(text)
                {
                    return Err(Refusal::invalid(
                        &format!("{field}/{index}"),
                        "must be a unique value using the permitted characters",
                    ));
                }
            }
            Ok(value.clone())
        }
        Kind::QuietHours => {
            let Some(obj) = value.as_object() else {
                return bad("must be {start, end}".into());
            };
            for part in ["start", "end"] {
                match obj.get(part).and_then(Value::as_i64) {
                    Some(h) if (0..=23).contains(&h) => {}
                    _ => {
                        return Err(Refusal::invalid(
                            &format!("{field}/{part}"),
                            "must be an hour from 0 to 23",
                        ));
                    }
                }
            }
            if obj.len() != 2 {
                return bad("must have only start and end".into());
            }
            Ok(value.clone())
        }
        Kind::Links { max_len } => {
            let Some(items) = value.as_array() else {
                return bad("must be a list of {label, url}".into());
            };
            if items.len() > max_len {
                return bad(format!("must have at most {max_len} items"));
            }
            for (i, item) in items.iter().enumerate() {
                let label = item.get("label").and_then(Value::as_str);
                let url = item.get("url").and_then(Value::as_str);
                if !label.is_some_and(|l| {
                    !l.trim().is_empty() && l.chars().count() <= 80 && no_controls(l)
                }) {
                    return Err(Refusal::invalid(
                        &format!("{field}/{i}/label"),
                        "must be text of 1 to 80 characters",
                    ));
                }
                if !url.is_some_and(|u| is_url(u, &["https"])) {
                    return Err(Refusal::invalid(
                        &format!("{field}/{i}/url"),
                        "must be an https:// URL",
                    ));
                }
            }
            Ok(value.clone())
        }
    }
}

// ─── environment ──────────────────────────────────────────────────────────────

/// The value an environment variable gives `def`, parsed per its kind. A value
/// that does not parse is carried as text so the pin is still visible; the read
/// site that owns the variable decides what it does with it.
fn parse_env(def: &SettingDef, raw: &str) -> Value {
    let raw = raw.trim();
    match def.kind {
        Kind::Bool => match raw.to_ascii_lowercase().as_str() {
            "1" | "true" | "yes" | "on" => Value::Bool(true),
            "0" | "false" | "no" | "off" | "" => Value::Bool(false),
            _ => Value::String(raw.to_string()),
        },
        Kind::Int { .. } => raw
            .parse::<i64>()
            .map(|n| json!(n))
            .unwrap_or_else(|_| Value::String(raw.to_string())),
        Kind::Float { .. } => raw
            .parse::<f64>()
            .ok()
            .filter(|f| f.is_finite())
            .map(|f| json!(f))
            .unwrap_or_else(|| Value::String(raw.to_string())),
        _ => Value::String(raw.to_string()),
    }
}

/// Where a key's environment variable stands right now.
#[derive(Debug, Clone, PartialEq)]
pub struct EnvState {
    pub var: &'static str,
    pub precedence: Precedence,
    /// The parsed value, when the variable is set.
    pub value: Option<Value>,
}

/// The environment state of `def`, read through `lookup_env` (tests pass their
/// own; the server passes [`process_env`]). A variable set to the empty string
/// counts as set, as for `UNSUPPORTED_ENV`.
pub fn env_state(
    def: &SettingDef,
    lookup_env: &dyn Fn(&str) -> Option<String>,
) -> Option<EnvState> {
    def.env.map(|(var, precedence)| EnvState {
        var,
        precedence,
        value: lookup_env(var).map(|raw| parse_env(def, &raw)),
    })
}

/// The process environment.
pub fn process_env(var: &str) -> Option<String> {
    std::env::var(var).ok()
}

/// `409 {code:"pinnedByEnv", var}` when an environment variable pins `def`.
pub fn refuse_if_pinned(
    def: &SettingDef,
    lookup_env: &dyn Fn(&str) -> Option<String>,
) -> Result<(), Refusal> {
    match env_state(def, lookup_env) {
        Some(EnvState {
            var,
            precedence: Precedence::Pin,
            value: Some(_),
        }) => Err(Refusal {
            status: 409,
            code: "pinnedByEnv",
            detail: format!("{var} is set in the environment and wins over a stored value"),
            field: None,
            extra: Some(("var", var.to_string())),
        }),
        _ => Ok(()),
    }
}

/// `403 {code:"useDedicatedRoute", route}` for a key the generic routes must not
/// write (t32-q A4).
pub fn refuse_if_dedicated(def: &SettingDef) -> Result<(), Refusal> {
    match def.dedicated_route {
        Some(route) => Err(Refusal {
            status: 403,
            code: "useDedicatedRoute",
            detail: format!("{} is changed through {route}", def.key),
            field: None,
            extra: Some(("route", mw_admin::config::admin_route(route))),
        }),
        None => Ok(()),
    }
}

// ─── effective values ─────────────────────────────────────────────────────────

/// One key's effective value for one subject.
#[derive(Debug, Clone, PartialEq)]
pub struct Effective {
    pub key: String,
    pub value: Value,
    pub source: SettingSource,
    pub locked_by: Option<SettingSource>,
    /// Whether the user may store a value of their own.
    pub user_may_override: bool,
    /// The user's own stored value, even when it does not apply.
    pub user_value: Option<Value>,
}

impl Effective {
    /// Whether the user is held to the value (`locked` in `GET /api/settings`).
    pub fn locked(&self) -> bool {
        !self.user_may_override
    }

    /// The `GET /api/settings` entry.
    pub fn to_user_json(&self) -> Value {
        let mut v = json!({
            "value": self.value,
            "source": self.source.as_str(),
            "locked": self.locked(),
        });
        if let Some(by) = self.locked_by {
            v["lockedBy"] = Value::String(by.as_str().to_string());
        }
        if let Some(u) = &self.user_value {
            v["userValue"] = u.clone();
        }
        v
    }
}

fn parse_json(s: &str) -> Value {
    serde_json::from_str(s).unwrap_or(Value::Null)
}

/// Resolve `key` from rows the caller already loaded. `values` are all of the
/// subject's rows (any key); the ones of `key` under the right per-user id are
/// picked here.
pub fn resolve_loaded(
    def: &SettingDef,
    key: &str,
    subject: &SettingSubject<'_>,
    policy: Option<&SettingPolicyRow>,
    values: &[SettingValueRow],
    lookup_env: &dyn Fn(&str) -> Option<String>,
) -> Effective {
    let user_id = subject.personal_owner.unwrap_or(match def.scope_id {
        ScopeId::Account => subject.account_id,
        ScopeId::Address => subject.address,
    });
    let mine: Vec<&SettingValueRow> = values
        .iter()
        .filter(|v| v.key == key && def.scopes.allows(v.scope))
        .filter(|v| match v.scope {
            SettingScope::AdminUser | SettingScope::User => {
                !user_id.is_empty() && v.scope_id == user_id
            }
            SettingScope::Domain => !subject.domain.is_empty() && v.scope_id == subject.domain,
            SettingScope::Profile | SettingScope::Group => true,
        })
        .collect();
    let env = env_state(def, lookup_env);
    let pin_text = match &env {
        Some(EnvState {
            precedence: Precedence::Pin,
            value: Some(v),
            ..
        }) => Some(v.to_string()),
        _ => None,
    };
    let (floor, floor_is_env) = match &env {
        Some(EnvState {
            precedence: Precedence::Fallback | Precedence::Boot,
            value: Some(v),
            ..
        }) => (v.to_string(), true),
        _ => (def.default.to_string(), false),
    };
    let mut policy = policy.cloned();
    if let Some(p) = policy.as_mut()
        && def.scopes != Scopes::GlobalDomainUser
    {
        // Admin-only keys: a user value can never apply, whatever the row says.
        p.allow_user_override = false;
    }
    let default_policy;
    let policy_ref = match policy.as_ref() {
        Some(p) => Some(p),
        None if !def.user_override => {
            default_policy = SettingPolicyRow {
                key: key.to_string(),
                global_value: None,
                allow_user_override: false,
                global_locked: false,
                updated_by: String::new(),
                updated_at: 0,
            };
            Some(&default_policy)
        }
        None => None,
    };
    let r = resolve(&ResolveInput {
        floor: &floor,
        floor_is_env,
        pin: pin_text.as_deref(),
        policy: policy_ref,
        values: &mine,
    });
    Effective {
        key: key.to_string(),
        value: parse_json(&r.value),
        source: r.source,
        locked_by: r.locked_by,
        user_may_override: r.user_may_override,
        user_value: r.user_value.as_deref().map(parse_json),
    }
}

/// The effective value of one key for one subject, read from the store now.
pub async fn effective_for(
    store: &Store,
    key: &str,
    subject: &SettingSubject<'_>,
) -> Result<Option<Effective>, StoreError> {
    effective_for_with(store, key, subject, &[]).await
}

/// [`effective_for`] with more value rows than the store holds: the per-user
/// values a key keeps elsewhere (the appearance blob, `mdn_policy:<account>`,
/// the notification rules row), passed in as `user` rows by the caller.
pub async fn effective_for_with(
    store: &Store,
    key: &str,
    subject: &SettingSubject<'_>,
    extra: &[SettingValueRow],
) -> Result<Option<Effective>, StoreError> {
    let canonical = store.canonical_setting_subject(subject).await?;
    let subject = &canonical.as_subject();
    let Some(def) = lookup(key) else {
        return Ok(None);
    };
    let policy = store.get_setting_policy(key).await?;
    let mut values = store.setting_values_for_subject(subject).await?;
    values.extend_from_slice(extra);
    Ok(Some(resolve_loaded(
        def,
        key,
        subject,
        policy.as_ref(),
        &values,
        &process_env,
    )))
}

/// The effective values of every registered (non-family) key that `filter`
/// accepts, plus every stored family key, for one subject. Two store reads.
pub async fn effective_all(
    store: &Store,
    subject: &SettingSubject<'_>,
    filter: impl Fn(&SettingDef) -> bool,
) -> Result<Vec<Effective>, StoreError> {
    effective_all_with(store, subject, filter, &[]).await
}

/// [`effective_all`] with extra value rows (see [`effective_for_with`]).
pub async fn effective_all_with(
    store: &Store,
    subject: &SettingSubject<'_>,
    filter: impl Fn(&SettingDef) -> bool,
    extra: &[SettingValueRow],
) -> Result<Vec<Effective>, StoreError> {
    let canonical = store.canonical_setting_subject(subject).await?;
    let subject = &canonical.as_subject();
    let policies = store.list_setting_policies().await?;
    let mut values = store.setting_values_for_subject(subject).await?;
    values.extend_from_slice(extra);
    let mut out = Vec::new();
    for def in REGISTRY.iter().filter(|d| !d.is_family() && filter(d)) {
        let policy = policies.iter().find(|p| p.key == def.key);
        out.push(resolve_loaded(
            def,
            def.key,
            subject,
            policy,
            &values,
            &process_env,
        ));
    }
    let mut family_keys: Vec<&str> = policies
        .iter()
        .map(|p| p.key.as_str())
        .chain(values.iter().map(|v| v.key.as_str()))
        .filter(|k| lookup(k).is_some_and(|d| d.is_family() && filter(d)))
        .collect();
    family_keys.sort_unstable();
    family_keys.dedup();
    for key in family_keys {
        if let Some(def) = lookup(key) {
            let policy = policies.iter().find(|p| p.key == key);
            out.push(resolve_loaded(
                def,
                key,
                subject,
                policy,
                &values,
                &process_env,
            ));
        }
    }
    Ok(out)
}

// ─── ceiling (t32-q A3) ───────────────────────────────────────────────────────

/// Who is writing, as far as the ceiling is concerned. f02's `AdminCtx` maps
/// onto this: an admin without a domain restriction is [`Actor::Unscoped`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Actor {
    Unscoped,
    /// A domain-scoped admin and the domains it may act on.
    Domains(Vec<String>),
}

/// What a `limit.*` write would change.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CeilingTarget<'a> {
    /// The global value.
    Global,
    /// A domain row.
    Domain(&'a str),
    /// An admin per-user row of a user in this domain.
    User { domain: &'a str },
    /// A profile or group value, or assigning a profile or group, within this
    /// domain.
    Assignment { domain: &'a str },
}

/// `0` means "no limit": compare as +∞.
fn limit_of(v: &Value) -> Option<u64> {
    match v.as_i64() {
        Some(0) => Some(u64::MAX),
        Some(n) if n > 0 => Some(n as u64),
        _ => None,
    }
}

/// The ceiling check every writer of a `limit.*` key calls before writing
/// (generic settings routes, profile and group values and assignments, e66's
/// `POST /admin/users` limits, e67's routes). Unscoped admins pass. For a
/// domain-scoped admin:
/// * a global write, or a domain outside its scope → 403 `forbidden`;
/// * a write to a **locked** domain row → 403 `setting_locked`;
/// * a value above the ceiling → 422 `aboveGlobalLimit`, where the ceiling is
///   the lower of the global value and a locked domain value, and `0` (no
///   limit) counts as above every finite ceiling.
///
/// Keys that are not `limit.*` pass.
pub async fn ceiling(
    store: &Store,
    actor: &Actor,
    key: &str,
    target: CeilingTarget<'_>,
    value: &Value,
) -> Result<(), CeilingError> {
    if !key.starts_with("limit.") {
        return Ok(());
    }
    let Actor::Domains(domains) = actor else {
        return Ok(());
    };
    let domain = match target {
        CeilingTarget::Global => {
            return Err(CeilingError::Refused(Refusal {
                status: 403,
                code: "forbidden",
                detail: "a domain administrator cannot change a global limit".into(),
                field: None,
                extra: None,
            }));
        }
        CeilingTarget::Domain(d)
        | CeilingTarget::User { domain: d }
        | CeilingTarget::Assignment { domain: d } => d,
    };
    let canonical_domain = store.canonical_policy_domain(domain).await?;
    let domain = canonical_domain.as_str();
    if !domains.iter().any(|d| d.eq_ignore_ascii_case(domain)) {
        return Err(CeilingError::Refused(Refusal {
            status: 403,
            code: "forbidden",
            detail: format!("{domain} is outside your domains"),
            field: None,
            extra: None,
        }));
    }
    let domain_row = store
        .get_setting_value(SettingScope::Domain, &domain.to_ascii_lowercase(), key)
        .await?;
    if matches!(target, CeilingTarget::Domain(_)) && domain_row.as_ref().is_some_and(|r| r.locked) {
        return Err(CeilingError::Refused(Refusal::locked(format!(
            "{key} is locked for {domain} by an administrator for all domains"
        ))));
    }
    let def = lookup(key);
    let global = match store
        .get_setting_policy(key)
        .await?
        .and_then(|p| p.global_value)
    {
        Some(v) => parse_json(&v),
        None => def.map(SettingDef::default_value).unwrap_or(Value::Null),
    };
    let mut cap = limit_of(&global).unwrap_or(u64::MAX);
    if let Some(row) = domain_row.filter(|r| r.locked) {
        cap = cap.min(limit_of(&parse_json(&row.value)).unwrap_or(u64::MAX));
    }
    let wanted = limit_of(value).unwrap_or(u64::MAX);
    if wanted > cap {
        return Err(CeilingError::Refused(Refusal {
            status: 422,
            code: "aboveGlobalLimit",
            detail: format!("{key} may not be above {cap} for a domain administrator"),
            field: Some("/value".into()),
            extra: Some(("ceiling", cap.to_string())),
        }));
    }
    Ok(())
}

/// [`ceiling`]'s failure: a refusal, or the store failing.
#[derive(Debug)]
pub enum CeilingError {
    Refused(Refusal),
    Store(StoreError),
}

impl From<StoreError> for CeilingError {
    fn from(e: StoreError) -> Self {
        CeilingError::Store(e)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use mw_store::ServerKey;

    fn no_env(_: &str) -> Option<String> {
        None
    }

    #[test]
    fn composer_from_visibility_is_hidden_by_default_and_user_overridable() {
        let def = lookup("mail.showFrom").unwrap();
        assert_eq!(def.default, "false");
        assert_eq!(def.scopes, Scopes::GlobalDomainUser);
        assert!(def.user_visible());
        assert!(def.user_override);
        assert!(validate(def, &serde_json::json!(true), "/value").is_ok());
        assert!(validate(def, &serde_json::json!("true"), "/value").is_err());
    }

    #[test]
    fn remembered_signin_policies_are_public_global_booleans() {
        for (key, default) in [("login.rememberSignins", "true"), ("login.showRememberedHint", "false")] {
            let def = lookup(key).unwrap();
            assert_eq!(def.default, default);
            assert_eq!(def.scopes, Scopes::Global);
            assert!(def.user_visible());
            assert!(!def.user_override);
            assert!(validate(def, &serde_json::json!(true), "/value").is_ok());
            assert!(validate(def, &serde_json::json!("true"), "/value").is_err());
        }
    }

    #[test]
    fn every_default_validates_and_keys_are_unique() {
        let mut seen = std::collections::HashSet::new();
        for d in REGISTRY {
            assert!(seen.insert(d.key), "duplicate key {}", d.key);
            validate(d, &d.default_value(), "/value")
                .unwrap_or_else(|r| panic!("{} default refused: {r:?}", d.key));
            assert!(
                serde_json::from_str::<Value>(d.default).is_ok(),
                "{} default is JSON",
                d.key
            );
        }
    }

    #[test]
    fn planned_keys_are_registered() {
        for key in [
            "ui.language",
            "appearance.theme",
            "appearance.accent",
            "appearance.density",
            "appearance.brandName",
            "login.notice",
            "mail.readReceipts",
            "notifications.enabled",
            "offline.retentionDays",
            "offline.budgetMb",
            "assistant.enabled",
            "upstream.default",
            "signin.newUsers",
            "signin.domainMode",
            "signin.refusalDetail",
            "signin.ldapBindPrecheck",
            "limit.storage_bytes",
            "limit.send_per_day",
            "limit.ai_calls_per_day",
            "quota.warnThresholds",
            "quota.warnEnabled",
            "usage.retentionDays",
            "usage.meterApi",
            "usage.measureIntervalHours",
            "assistant.rateLimitPerMin",
            "plugins.kv.maxKeyBytes",
            "import.maxMessages",
            "webhooks.maxAttempts",
            "outbox.maxAttempts",
            "realtime.recheckSecs",
            "privacy.watermark.enabled",
            "maskedEmail.domain",
            "abuse.reportAddress",
            "dlp.adminAddress",
            "observability.errorForwardUrl",
            "egress.probeUrl",
            "push.vapidContact",
            "push.quietHours",
            "twofa.pendingTtlSecs",
            "twofa.maxAttempts",
            "admin.session.idleSecs",
            "admin.session.absoluteSecs",
            "updates.check",
            "updates.feedUrl",
            "updates.apply",
            "backup.retentionCount",
            "about.securityContact",
            "about.securityPolicyUrl",
            "about.projectLinks",
            "config.snapshotsKept",
            "jobs.historyDays",
            "maintenance.announceMinutesDefault",
            "maintenance.defaultMessage",
            "health.diskFreeMinPercent",
            "observability.otlpMetrics",
            "metrics.drilldownTopN",
            "metrics.rollupDays",
            "diagnostics.retentionDays",
            "dataExport.linkHours",
            "passwd.policy.minLength",
            "passwd.policy.requireSymbol",
            "privacy.openTracking.retentionDays",
            "privacy.openTracking.unboundTtlHours",
            "upstream.proxyTimeoutSecs",
            "serverText.mdn-body",
        ] {
            assert!(lookup(key).is_some(), "{key} is registered");
        }
        // Dropped by the t32-q amendment.
        assert!(lookup("quota.defaultBytes").is_none());
        assert!(lookup("quota.defaultMessages").is_none());
        // A family id must be a slug.
        assert!(lookup("serverText.").is_none());
        assert!(lookup("serverText.Bad Id").is_none());
        assert!(lookup("serverText.*").is_none());
    }

    #[test]
    fn signin_keys_name_a_dedicated_route_and_limits_are_per_address() {
        for d in REGISTRY.iter().filter(|d| d.key.starts_with("signin.") && !matches!(d.key, "signin.retryCooldownSecs" | "signin.antiBotMode" | "signin.antiBotDifficulty" | "signin.antiBotExpirySecs")) {
            let r = refuse_if_dedicated(d).unwrap_err();
            assert_eq!((r.status, r.code), (403, "useDedicatedRoute"), "{}", d.key);
            assert!(r.body()["route"].as_str().unwrap().starts_with('/'));
        }
        for d in REGISTRY
            .iter()
            .filter(|d| d.key.starts_with("limit.") || d.key.starts_with("quota."))
        {
            assert_eq!(d.scope_id, ScopeId::Address, "{}", d.key);
            assert!(!d.user_override, "{}", d.key);
        }
        assert!(refuse_if_dedicated(lookup("ui.language").unwrap()).is_ok());
        assert_eq!(
            lookup("signin.refusalDetail").unwrap().default_value(),
            json!("explain")
        );
    }

    /// Every key has its documentation anchor (UX4) and a label in the console
    /// (`apps/web/src/screens/Admin/Settings/keys.ts`); a key added here without
    /// them fails.
    #[test]
    fn every_key_is_documented_and_labelled() {
        let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
        let docs = std::fs::read_to_string(root.join("docs/admin/settings.md")).unwrap();
        let labels =
            std::fs::read_to_string(root.join("apps/web/src/screens/Admin/Settings/keys.ts"))
                .unwrap();
        let mut missing = Vec::new();
        for d in REGISTRY {
            if !docs.contains(&format!("<a id=\"{}\"></a>", d.key)) {
                missing.push(format!("docs/admin/settings.md#{}", d.key));
            }
            if !labels.contains(&format!("'{}'", d.key)) {
                missing.push(format!("Settings/keys.ts {}", d.key));
            }
        }
        assert!(missing.is_empty(), "missing: {missing:?}");
    }

    #[test]
    fn appearance_default_is_dark() {
        assert_eq!(
            lookup("appearance.theme").unwrap().default_value(),
            json!("dark")
        );
    }

    #[test]
    fn validation_refuses_with_a_field_pointer() {
        let d = lookup("offline.budgetMb").unwrap();
        assert_eq!(validate(d, &json!(10), "/value").unwrap(), json!(10));
        let r = validate(d, &json!(99_999), "/value").unwrap_err();
        assert_eq!(
            (r.status, r.code, r.field.as_deref()),
            (422, "invalidValue", Some("/value"))
        );
        assert!(validate(d, &json!("10"), "/value").is_err());
        assert!(validate(d, &Value::Null, "/value").is_err());

        let d = lookup("quota.warnThresholds").unwrap();
        assert!(validate(d, &json!([50, 90]), "/v").is_ok());
        assert_eq!(
            validate(d, &json!([90, 50]), "/v")
                .unwrap_err()
                .field
                .as_deref(),
            Some("/v/1")
        );
        let d = lookup("maintenance.defaultMessage").unwrap();
        assert!(
            validate(
                d,
                &json!({ "en": "Back soon", "de": "Gleich zurück" }),
                "/v"
            )
            .is_ok()
        );
        assert_eq!(
            validate(d, &json!({ "xx": "?" }), "/v")
                .unwrap_err()
                .field
                .as_deref(),
            Some("/v/xx")
        );
        let d = lookup("login.notice").unwrap();
        assert_eq!(validate(d, &Value::Null, "/v").unwrap(), Value::Null);
        let d = lookup("egress.probeUrl").unwrap();
        assert!(validate(d, &json!("http://example.com/"), "/v").is_err());
        assert!(validate(d, &json!("https://example.com/"), "/v").is_ok());
        let d = lookup("push.vapidContact").unwrap();
        assert!(validate(d, &json!("mailto:ops@example.org"), "/v").is_ok());
        assert!(validate(d, &json!("ops@example.org"), "/v").is_err());
        let d = lookup("ui.language").unwrap();
        assert!(validate(d, &json!("pt-BR"), "/v").is_ok());
        assert!(validate(d, &json!("xx"), "/v").is_err());
        let d = lookup("push.quietHours").unwrap();
        assert!(validate(d, &json!({ "start": 22, "end": 6 }), "/v").is_ok());
        assert_eq!(
            validate(d, &json!({ "start": 24, "end": 6 }), "/v")
                .unwrap_err()
                .field
                .as_deref(),
            Some("/v/start")
        );
        let d = lookup("about.projectLinks").unwrap();
        assert!(
            validate(
                d,
                &json!([{ "label": "Code", "url": "https://example.org" }]),
                "/v"
            )
            .is_ok()
        );
        assert!(
            validate(
                d,
                &json!([{ "label": "Code", "url": "javascript:alert(1)" }]),
                "/v"
            )
            .is_err()
        );
    }

    #[test]
    fn env_pin_and_fallback() {
        let d = lookup("realtime.recheckSecs").unwrap();
        let env = |v: &str| (v == "MW_REALTIME_RECHECK_SECS").then(|| "45".to_string());
        let r = refuse_if_pinned(d, &env).unwrap_err();
        assert_eq!((r.status, r.code), (409, "pinnedByEnv"));
        assert_eq!(r.body()["var"], "MW_REALTIME_RECHECK_SECS");
        assert!(refuse_if_pinned(d, &no_env).is_ok());

        let subject = SettingSubject::default();
        let e = resolve_loaded(d, d.key, &subject, None, &[], &env);
        assert_eq!(
            (e.value.clone(), e.source),
            (json!(45), SettingSource::EnvPin)
        );
        assert!(e.locked());

        let d = lookup("upstream.default").unwrap();
        let env =
            |v: &str| (v == "MW_UPSTREAM_DEFAULT").then(|| "https://mail.example.org".to_string());
        assert!(
            refuse_if_pinned(d, &env).is_ok(),
            "a fallback never refuses"
        );
        let e = resolve_loaded(d, d.key, &subject, None, &[], &env);
        assert_eq!(e.source, SettingSource::EnvFallback);
        assert_eq!(e.value, json!("https://mail.example.org"));
        let stored = SettingPolicyRow {
            key: d.key.into(),
            global_value: Some("\"https://stored.example.org\"".into()),
            allow_user_override: false,
            global_locked: false,
            updated_by: "root".into(),
            updated_at: 1,
        };
        let e = resolve_loaded(d, d.key, &subject, Some(&stored), &[], &env);
        assert_eq!(e.source, SettingSource::Global);
    }

    #[test]
    fn admin_only_keys_ignore_user_rows_and_limits_use_the_address() {
        let subject = SettingSubject { personal_owner: None,
            domain: "example.org",
            account_id: "acct-1",
            address: "alice@example.org",
        };
        let row = |scope, id: &str, key: &str, v: &str| SettingValueRow {
            scope,
            scope_id: id.into(),
            key: key.into(),
            value: v.into(),
            locked: false,
            updated_by: "root".into(),
            updated_at: 1,
        };
        let d = lookup("limit.storage_bytes").unwrap();
        let rows = [
            row(SettingScope::AdminUser, "acct-1", d.key, "111"),
            row(SettingScope::AdminUser, "alice@example.org", d.key, "222"),
            row(SettingScope::User, "alice@example.org", d.key, "333"),
        ];
        let e = resolve_loaded(d, d.key, &subject, None, &rows, &no_env);
        assert_eq!(
            e.value,
            json!(222),
            "the address row, never the account row or a user row"
        );
        assert!(e.locked());

        let d = lookup("appearance.theme").unwrap();
        let rows = [row(SettingScope::User, "acct-1", d.key, "\"light\"")];
        let e = resolve_loaded(d, d.key, &subject, None, &rows, &no_env);
        assert_eq!(
            (e.value.clone(), e.source),
            (json!("light"), SettingSource::User)
        );
        assert!(!e.locked());
    }

    async fn store() -> Store {
        Store::open_in_memory(ServerKey::generate()).await.unwrap()
    }

    fn refused(r: Result<(), CeilingError>) -> Refusal {
        match r {
            Err(CeilingError::Refused(r)) => r,
            other => panic!("expected a refusal, got {other:?}"),
        }
    }

    #[tokio::test]
    async fn ceiling_rules_for_domain_admins() {
        let s = store().await;
        let key = "limit.storage_bytes";
        let scoped = Actor::Domains(vec!["example.org".into()]);
        let one_gb = json!(1_000_000_000);

        // No global limit (0): a domain admin may set anything, including 0.
        ceiling(
            &s,
            &scoped,
            key,
            CeilingTarget::User {
                domain: "example.org",
            },
            &json!(0),
        )
        .await
        .unwrap();

        s.put_setting_policy(&SettingPolicyRow {
            key: key.into(),
            global_value: Some(one_gb.to_string()),
            allow_user_override: false,
            global_locked: false,
            updated_by: "root".into(),
            updated_at: 1,
        })
        .await
        .unwrap();

        // Under the global value passes; 0 (no limit) and above are refused.
        ceiling(
            &s,
            &scoped,
            key,
            CeilingTarget::User {
                domain: "example.org",
            },
            &json!(500),
        )
        .await
        .unwrap();
        for v in [json!(0), json!(2_000_000_000)] {
            let r = refused(
                ceiling(
                    &s,
                    &scoped,
                    key,
                    CeilingTarget::User {
                        domain: "example.org",
                    },
                    &v,
                )
                .await,
            );
            assert_eq!((r.status, r.code), (422, "aboveGlobalLimit"), "{v}");
        }
        let r = refused(
            ceiling(
                &s,
                &scoped,
                key,
                CeilingTarget::Assignment {
                    domain: "example.org",
                },
                &json!(0),
            )
            .await,
        );
        assert_eq!(
            r.code, "aboveGlobalLimit",
            "a profile with 0 assigned by a domain admin"
        );

        // A locked domain value of 500 MB: the domain row cannot be written, and
        // a user value is capped by it.
        s.put_setting_value(&SettingValueRow {
            scope: SettingScope::Domain,
            scope_id: "example.org".into(),
            key: key.into(),
            value: "500000000".into(),
            locked: true,
            updated_by: "root".into(),
            updated_at: 1,
        })
        .await
        .unwrap();
        let r = refused(
            ceiling(
                &s,
                &scoped,
                key,
                CeilingTarget::Domain("example.org"),
                &json!(1),
            )
            .await,
        );
        assert_eq!((r.status, r.code), (403, "setting_locked"));
        let r = refused(
            ceiling(
                &s,
                &scoped,
                key,
                CeilingTarget::User {
                    domain: "example.org",
                },
                &json!(600_000_000),
            )
            .await,
        );
        assert_eq!(r.code, "aboveGlobalLimit");

        // Scope: global writes and other domains are refused; unscoped admins
        // and non-limit keys pass.
        let r = refused(ceiling(&s, &scoped, key, CeilingTarget::Global, &json!(1)).await);
        assert_eq!((r.status, r.code), (403, "forbidden"));
        let r = refused(
            ceiling(
                &s,
                &scoped,
                key,
                CeilingTarget::Domain("other.org"),
                &json!(1),
            )
            .await,
        );
        assert_eq!(r.code, "forbidden");
        ceiling(
            &s,
            &Actor::Unscoped,
            key,
            CeilingTarget::Domain("example.org"),
            &json!(0),
        )
        .await
        .unwrap();
        ceiling(
            &s,
            &scoped,
            "quota.warnEnabled",
            CeilingTarget::Global,
            &json!(false),
        )
        .await
        .unwrap();
    }
    #[tokio::test]
    async fn alias_subject_is_canonical_before_registry_filters_and_ceiling() {
        let s = store().await;
        s.put_domain_config(&mw_store::DomainConfigRow {
            name: "canonical.test".into(),
            upstream_json: "{}".into(),
            allowlist_json: "[]".into(),
            blocklist_json: "[]".into(),
            enabled: true,
            preferred: false,
            settings_json: "{}".into(),
            updated_at: 1,
        })
        .await
        .unwrap();
        s.put_domain_alias("alias.test", "canonical.test")
            .await
            .unwrap();
        let subject = SettingSubject { personal_owner: None,
            domain: "alias.test",
            account_id: "unchanged-account",
            address: "USER@alias.test",
        };
        for (key, scope, id, value) in [
            (
                "privacy.remoteContent",
                SettingScope::Domain,
                "canonical.test",
                "\"block\"",
            ),
            (
                "cap.remote_content.direct",
                SettingScope::Domain,
                "canonical.test",
                "false",
            ),
            (
                "limit.storage_bytes",
                SettingScope::AdminUser,
                "user@canonical.test",
                "123",
            ),
        ] {
            s.put_setting_value(&SettingValueRow {
                scope,
                scope_id: id.into(),
                key: key.into(),
                value: value.into(),
                locked: true,
                updated_by: "root".into(),
                updated_at: 1,
            })
            .await
            .unwrap();
        }
        let remote = effective_for(&s, "privacy.remoteContent", &subject)
            .await
            .unwrap()
            .unwrap();
        assert_eq!(remote.value, json!("block"));
        assert!(remote.locked());
        assert_eq!(
            effective_for(&s, "cap.remote_content.direct", &subject)
                .await
                .unwrap()
                .unwrap()
                .value,
            json!(false)
        );
        assert_eq!(
            effective_for(&s, "limit.storage_bytes", &subject)
                .await
                .unwrap()
                .unwrap()
                .value,
            json!(123)
        );
        let all = effective_all(&s, &subject, |d| d.key == "privacy.remoteContent")
            .await
            .unwrap();
        assert_eq!(all[0].value, json!("block"));
        assert!(all[0].locked());
        ceiling(
            &s,
            &Actor::Domains(vec!["canonical.test".into()]),
            "limit.storage_bytes",
            CeilingTarget::User {
                domain: "alias.test",
            },
            &json!(1),
        )
        .await
        .unwrap();
    }
}

#[cfg(test)]
mod shortcut_validation_tests {
    use super::*;
    #[test]
    fn shortcut_chords_are_bounded_and_match_supported_browser_syntax() {
        for value in ["inherit", "disabled", "Ctrl+Shift+K", "Cmd+ArrowDown", "Space", "Ctrl++", "+", "↓"] { assert!(valid_shortcut(value), "{value}"); }
        for value in ["", "Hyper+x", "UnknownKey", "Ctrl+", "Ctrl+Shift+", "\n"] { assert!(!valid_shortcut(value), "{value:?}"); }
        assert!(!valid_shortcut(&"x".repeat(65)));
    }
}

#[cfg(test)]
mod time_zone_validation_tests {
    use super::*;
    #[test]
    fn accepts_only_supported_zones_and_explicit_sentinels() {
        let display = REGISTRY.iter().find(|def| def.key == "time.displayZone").unwrap();
        let comparison = REGISTRY.iter().find(|def| def.key == "time.comparisonZone").unwrap();
        for zone in ["Europe/Lisbon", "America/New_York", "UTC", "local"] {
            assert!(validate(display, &serde_json::json!(zone), "/value").is_ok());
        }
        assert!(validate(comparison, &serde_json::json!(""), "/value").is_ok());
        for zone in ["", "someone@example.org", "+01:00", "Mars/Olympus"] {
            assert!(validate(display, &serde_json::json!(zone), "/value").is_err());
        }
        assert!(validate(comparison, &serde_json::json!("local"), "/value").is_err());
    }
}

#[cfg(test)]
mod geography_validation_tests {
    use super::*;
    #[test]
    fn country_lists_are_bounded_uppercase_codes_without_wildcards() {
        let def=REGISTRY.iter().find(|def|def.key=="security.signin.geoCountries").unwrap();
        assert!(validate(def,&serde_json::json!(["PT","GB"]),"/value").is_ok());
        for value in [serde_json::json!(["pt"]),serde_json::json!(["*"]),serde_json::json!(["USA"]),serde_json::json!(["1A"]),serde_json::json!("PT"),serde_json::json!([null]),serde_json::json!(["PT","PT"])] {
            assert!(validate(def,&value,"/value").is_err(),"{value}");
        }
        assert!(validate(def,&serde_json::json!(vec!["PT";250]),"/value").is_err());
    }
}

#[cfg(test)]
mod folder_role_validation_tests {
    use super::*;
    #[test]
    fn only_automatic_or_one_bounded_owned_target_shape_is_accepted() {
        for role in mw_engine::folder_roles::ROLES {
            let key = format!("mail.folderRole.{role}");
            let def = REGISTRY.iter().find(|def| def.key == key).unwrap();
            for value in [json!(null), json!({"id":"owned"}), json!({"path":"INBOX/Deleted Items"})] {
                assert!(validate(def, &value, "/value").is_ok(), "{key}: {value}");
            }
            for value in [json!("Trash"), json!({}), json!({"id":"a","path":"Trash"}), json!({"path":"Trash\r\nINBOX"})] {
                assert!(validate(def, &value, "/value").is_err(), "{key}: {value}");
            }
        }
    }
}
