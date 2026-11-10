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

pub mod link_trust;

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
    /// Bounded plain-text response templates; never executable HTML.
    MessageTemplates,
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
    AttachmentList {
        mime: bool,
    },
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
            Kind::MessageTemplates => {
                json!({"type":"messageTemplates","maxItems":20,"maxTitleLength":80,"maxBodyLength":8000,"maxBytes":65536})
            }
            Kind::FolderRole => json!({ "type": "folderRole", "maxLength": 512 }),
            Kind::Bool => json!({ "type": "bool" }),
            Kind::AttachmentList { .. } => {
                json!({ "type": "stringList", "maxItems": 100, "maxLength": 127 })
            }
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
    "en", "de", "fr", "es", "pt-BR", "pt-PT", "nl", "it", "pl", "ru", "uk", "zh", "ja", "ar",
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
        self.scopes == Scopes::GlobalDomainUser
            || PUBLIC_KEYS.contains(&self.key)
            || self.key == "receipts.dsnAllowed"
            || matches!(
                self.key,
                "mail.templates.shared"
                    | "mail.conversations.allowed"
                    | "mail.templates.allowPersonal"
                    | "mail.send.recallDelayMaxSeconds"
                    | "mail.send.allowRecall"
                    | "mail.send.maxDelayedSeconds"
                    | "compose.replyTo.allowOverride"
                    | "compose.headers.allowAutoSubmitted"
                    | "compose.headers.allowThreadingEdit"
                    | "compose.headers.allowCustom"
                    | "compose.headers.allowReturnReceiptTo"
            )
            // t39-e7: the storage meter warns at the admin's thresholds; the
            // user reads them and cannot change them (scope is global/domain).
            || matches!(self.key, "quota.warnEnabled" | "quota.warnThresholds")
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
const PUBLIC_KEYS: &[&str] = &[
    "appearance.brandName",
    "login.notice",
    "login.showDomainHint",
    "login.rememberSignins",
    "login.showRememberedHint",
];

/// The admin route (console-path suffix) that owns `signin.newUsers`,
/// `signin.domainMode` and `signin.refusalDetail` (e66): those writes need
/// `signin.policy` and the impact confirmation, which the generic routes do not do.
pub const SIGNIN_POLICY_ROUTE: &str = "/signin-policy";

const BYTES_MAX: i64 = 1 << 50;

/// t98: the values `access.protocols` may hold (the native account kinds).
pub const ACCESS_PROTOCOLS: &[&str] = &["imap", "pop3", "ews"];

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

/// The admin-probe policy route (t100, `admin_probe::ROUTE`).
const ADMIN_PROBE_ROUTE: &str = "/security/admin-probe";

const fn dedicated(mut d: SettingDef, route: &'static str) -> SettingDef {
    d.dedicated_route = Some(route);
    d
}

use Scopes::{
    Global as G, GlobalDomain as GD, GlobalDomainAdminUser as GDA, GlobalDomainUser as GDU,
};

/// Every registered key. Order is the admin screen's order within a category.
pub static REGISTRY: &[SettingDef] = &[
    def!(
        "mail.reader.unsafeHtmlMode",
        Kind::Enum(&["ask", "strict", "sanitized"]),
        "\"sanitized\"",
        GDU,
        "mail",
        "Handling of message HTML detected as unsafe"
    ),
    def!(
        "reader.attachmentsDensity",
        Kind::Enum(&["compact", "comfortable"]),
        "\"compact\"",
        GDU,
        "reader",
        "Reader attachment list density"
    ),
    def!(
        "mail.composeWindow",
        Kind::Enum(&["tab", "window"]),
        "\"tab\"",
        GDU,
        "mail",
        "Preferred composer location"
    ),
    def!(
        "mail.replyCopyToOriginal",
        Kind::Bool,
        "false",
        GDU,
        "mail",
        "Offer original folder for reply copies"
    ),
    def!(
        "mail.forwardMode",
        Kind::Enum(&["inline", "attachment"]),
        "\"inline\"",
        GDU,
        "mail",
        "Preferred forwarding format"
    ),
    def!(
        "mail.allowSend",
        Kind::Bool,
        "true",
        GDU,
        "mail",
        "Allow sending messages; drafts can still be saved"
    ),
    def!(
        "mail.allowReply",
        Kind::Bool,
        "true",
        GDU,
        "mail",
        "Allow reply and reply all"
    ),
    def!(
        "mail.allowForward",
        Kind::Bool,
        "true",
        GDU,
        "mail",
        "Allow forwarding messages"
    ),
    def!(
        "mail.allowDownload",
        Kind::Bool,
        "true",
        GDU,
        "mail",
        "Allow downloading messages and attachments"
    ),
    def!(
        "mail.allowPrint",
        Kind::Bool,
        "true",
        GDU,
        "mail",
        "Allow printing messages"
    ),
    def!(
        "mail.showSpam",
        Kind::Bool,
        "true",
        GDU,
        "mail",
        "Show the Spam folder and messages marked as spam"
    ),
    def!(
        "mail.showAuthStatus",
        Kind::Bool,
        "true",
        GDU,
        "mail",
        "Show SPF, DKIM and DMARC results in the reader"
    ),
    def!(
        "mail.closeOnDelete",
        Kind::Bool,
        "true",
        GDU,
        "mail",
        "Close the open message when it is deleted"
    ),
    def!(
        "mail.rowActionsMode",
        Kind::Enum(&["open", "hover", "click", "off"]),
        "\"open\"",
        GDU,
        "mail",
        "How the quick actions on message list rows are shown"
    ),
    def!(
        "mail.showGoToTop",
        Kind::Bool,
        "false",
        GDU,
        "mail",
        "Show a Go to top button in the message list"
    ),
    def!(
        "mail.showCreateFolderButton",
        Kind::Bool,
        "false",
        GDU,
        "mail",
        "Show a Create folder button next to Compose"
    ),
    def!(
        "mail.cache.refreshOnOpen",
        Kind::Bool,
        "true",
        GDU,
        "mail",
        "Refresh cached mail in the background on opening"
    ),
    def!(
        "mail.cache.enabled",
        Kind::Bool,
        r#"true"#,
        GDU,
        "mail",
        "Browser session mail cache: enabled"
    ),
    def!(
        "mail.cache.mailboxes",
        Kind::Bool,
        r#"true"#,
        GDU,
        "mail",
        "Browser session mail cache: mailboxes"
    ),
    def!(
        "mail.cache.messages",
        Kind::Bool,
        r#"true"#,
        GDU,
        "mail",
        "Browser session mail cache: messages"
    ),
    def!(
        "mail.cache.keepIndefinitely",
        Kind::Bool,
        "false",
        GDU,
        "mail",
        "Opt in to encrypted browser mail storage across reloads without age expiry; capacity limits and manual clearing still apply"
    ),
    def!(
        "mail.cache.freshSeconds",
        int(0, 3600),
        r#"30"#,
        GDU,
        "mail",
        "Browser session mail cache: freshSeconds"
    ),
    def!(
        "mail.cache.maxAgeSeconds",
        int(1, 86400),
        r#"300"#,
        GDU,
        "mail",
        "Browser session mail cache: maxAgeSeconds"
    ),
    def!(
        "mail.cache.maxEntries",
        int(1, 512),
        r#"32"#,
        GDU,
        "mail",
        "Browser session mail cache: maxEntries"
    ),
    def!(
        "mail.cache.maxBytes",
        int(65536, 268435456),
        r#"16777216"#,
        GDU,
        "mail",
        "Browser session mail cache: maxBytes"
    ),
    def!(
        "mail.cache.prefetchEnabled",
        Kind::Bool,
        r#"false"#,
        GDU,
        "mail",
        "Browser session mail cache: prefetchEnabled"
    ),
    def!(
        "mail.cache.hoverEnabled",
        Kind::Bool,
        r#"true"#,
        GDU,
        "mail",
        "Browser session mail cache: hoverEnabled"
    ),
    def!(
        "mail.cache.focusEnabled",
        Kind::Bool,
        r#"true"#,
        GDU,
        "mail",
        "Browser session mail cache: focusEnabled"
    ),
    def!(
        "mail.cache.dwellMs",
        int(0, 10000),
        r#"200"#,
        GDU,
        "mail",
        "Browser session mail cache: dwellMs"
    ),
    def!(
        "mail.cache.idleEnabled",
        Kind::Bool,
        r#"false"#,
        GDU,
        "mail",
        "Browser session mail cache: idleEnabled"
    ),
    def!(
        "mail.cache.idleDelayMs",
        int(100, 60000),
        r#"1500"#,
        GDU,
        "mail",
        "Browser session mail cache: idleDelayMs"
    ),
    def!(
        "mail.cache.maxConcurrency",
        int(1, 4),
        r#"1"#,
        GDU,
        "mail",
        "Browser session mail cache: maxConcurrency"
    ),
    def!(
        "mail.cache.maxQueued",
        int(1, 64),
        r#"8"#,
        GDU,
        "mail",
        "Browser session mail cache: maxQueued"
    ),
    def!(
        "mail.cache.preloadMessages",
        Kind::Bool,
        r#"false"#,
        GDU,
        "mail",
        "Browser session mail cache: preloadMessages"
    ),
    def!(
        "mail.cache.excludedFolders",
        Kind::StringList {
            max_len: 100,
            max_item: 256
        },
        r#"["role:trash","role:junk"]"#,
        GDU,
        "mail",
        "Browser session mail cache: excludedFolders"
    ),
    def!(
        "mail.templates.enabled",
        Kind::Bool,
        "true",
        GDU,
        "mail",
        "Plain-text response templates: enabled"
    ),
    def!(
        "mail.templates.allowPersonal",
        Kind::Bool,
        "true",
        GDA,
        "mail",
        "Plain-text response templates: allowPersonal"
    ),
    def!(
        "mail.templates.shared",
        Kind::MessageTemplates,
        "[]",
        GDA,
        "mail",
        "Plain-text response templates: shared"
    ),
    def!(
        "mail.templates.personal",
        Kind::MessageTemplates,
        "[]",
        GDU,
        "mail",
        "Plain-text response templates: personal"
    ),
    def!(
        "accounts.linkingEnabled",
        Kind::Bool,
        "false",
        GDA,
        "accounts",
        "Allow verified linked mailbox accounts; global false is a hard ceiling"
    ),
    def!(
        "accounts.useLinked",
        Kind::Bool,
        "true",
        GDU,
        "accounts",
        "Use linked mailboxes when permitted by the administrator"
    ),
    def!(
        "accounts.maxLinked",
        int(1, 16),
        "5",
        G,
        "accounts",
        "Maximum verified linked mailboxes per primary identity"
    ),
    def!(
        "accounts.storeLinkedCredentials",
        Kind::Bool,
        "true",
        GDA,
        "accounts",
        "Keep a sealed copy of each linked mailbox's password so its access renews without asking; off keeps sign-in-only links"
    ),
    def!(
        "privacy.stripMailerHeaders",
        Kind::Bool,
        "true",
        GDU,
        "privacy",
        "remove client fingerprint headers from native outgoing email; preserve message bodies and attached messages"
    ),
    def!(
        "privacy.remoteImageDownloads",
        Kind::Bool,
        "false",
        GDU,
        "privacy",
        "allow deliberate message image downloads through the guarded privacy proxy"
    ),
    def!(
        "privacy.remoteImagesToast",
        Kind::Bool,
        "true",
        GDU,
        "privacy",
        "show one notification while a message's remote images load through the privacy proxy; presentation only, the image policy is unchanged"
    ),
    def!(
        "privacy.sessions.showLinkedOrigin",
        Kind::Enum(&["full", "masked-local-part", "hidden"]),
        "\"full\"",
        GDU,
        "privacy",
        "How Settings > Security > Sessions names the account a linked-mailbox session originates from: full address, address with the local part masked, or hidden"
    ),
    def!(
        "appearance.customCssEnabled",
        Kind::Bool,
        "false",
        G,
        "appearance",
        "Allow owner-scoped appearance CSS; disabled by default and paused under enforced mailbox appearance locks"
    ),
    def!(
        "appearance.customJsEnabled",
        Kind::Bool,
        "false",
        G,
        "appearance",
        "Allow owner-written scripts in an opaque appearance-only worker sandbox; disabled by default"
    ),
    nullable(def!(
        "mail.folderRole.inbox",
        Kind::FolderRole,
        "null",
        GDU,
        "mail",
        "inbox folder role override; null keeps automatic discovery, an exact path or owned ID selects the folder"
    )),
    nullable(def!(
        "mail.folderRole.drafts",
        Kind::FolderRole,
        "null",
        GDU,
        "mail",
        "drafts folder role override; null keeps automatic discovery, an exact path or owned ID selects the folder"
    )),
    nullable(def!(
        "mail.folderRole.sent",
        Kind::FolderRole,
        "null",
        GDU,
        "mail",
        "sent folder role override; null keeps automatic discovery, an exact path or owned ID selects the folder"
    )),
    nullable(def!(
        "mail.folderRole.trash",
        Kind::FolderRole,
        "null",
        GDU,
        "mail",
        "trash folder role override; null keeps automatic discovery, an exact path or owned ID selects the folder"
    )),
    nullable(def!(
        "mail.folderRole.junk",
        Kind::FolderRole,
        "null",
        GDU,
        "mail",
        "junk folder role override; null keeps automatic discovery, an exact path or owned ID selects the folder"
    )),
    nullable(def!(
        "mail.folderRole.archive",
        Kind::FolderRole,
        "null",
        GDU,
        "mail",
        "archive folder role override; null keeps automatic discovery, an exact path or owned ID selects the folder"
    )),
    def!(
        "nextcloud.syncIntervalSecs",
        int(0, 86400),
        "300",
        GDU,
        "integrations",
        "Nextcloud collection refresh interval; zero is manual only, positive values have a sixty-second minimum"
    ),
    def!(
        "packages.allowed",
        Kind::Bool,
        "true",
        GDA,
        "integrations",
        "Allow package tracking; provider activation and separate account opt-in are still required"
    ),
    def!(
        "cap.integration.nextcloud",
        Kind::Bool,
        "true",
        GDA,
        "integrations",
        "Nextcloud integration"
    ),
    def!(
        "cap.integration.google",
        Kind::Bool,
        "true",
        GDA,
        "integrations",
        "Google OAuth bridge"
    ),
    def!(
        "cap.integration.microsoft",
        Kind::Bool,
        "true",
        GDA,
        "integrations",
        "Microsoft OAuth bridge"
    ),
    def!(
        "cap.integration.*",
        Kind::Bool,
        "true",
        GDA,
        "integrations",
        "Individual integration permission"
    ),
    def!(
        "cap.ui_plugin.*",
        Kind::Bool,
        "true",
        GDA,
        "integrations",
        "Individual UI plugin permission"
    ),
    def!(
        "cap.ai_assistant",
        Kind::Bool,
        "true",
        GDA,
        "features",
        "AI assistant"
    ),
    def!(
        "cap.plugins",
        Kind::Bool,
        "true",
        GDA,
        "features",
        "Plugins"
    ),
    def!(
        "cap.pim.calendar",
        Kind::Bool,
        "true",
        GDA,
        "features",
        "Calendar"
    ),
    def!(
        "cap.pim.contacts",
        Kind::Bool,
        "true",
        GDA,
        "features",
        "Contacts"
    ),
    def!(
        "cap.pim.tasks",
        Kind::Bool,
        "true",
        GDA,
        "features",
        "Tasks"
    ),
    def!(
        "cap.pim.notes",
        Kind::Bool,
        "true",
        GDA,
        "features",
        "Notes"
    ),
    def!("cap.export", Kind::Bool, "true", GDA, "features", "Export"),
    def!("cap.import", Kind::Bool, "true", GDA, "features", "Import"),
    def!(
        "cap.api_keys",
        Kind::Bool,
        "true",
        GDA,
        "features",
        "API keys"
    ),
    def!(
        "cap.oauth_apps",
        Kind::Bool,
        "true",
        GDA,
        "features",
        "OAuth applications"
    ),
    def!(
        "cap.rules",
        Kind::Bool,
        "true",
        GDA,
        "features",
        "Mail rules"
    ),
    def!(
        "cap.rules.forward",
        Kind::Bool,
        "true",
        GDA,
        "features",
        "Rule forwarding"
    ),
    def!(
        "cap.remote_images",
        Kind::Bool,
        "true",
        GDA,
        "features",
        "Remote images"
    ),
    def!(
        "cap.user_background",
        Kind::Bool,
        "false",
        GDA,
        "features",
        "Custom background"
    ),
    def!(
        "cap.user_css",
        Kind::Bool,
        "false",
        GDA,
        "features",
        "Custom CSS"
    ),
    def!(
        "cap.compose.rich_text",
        Kind::Bool,
        "true",
        GDA,
        "features",
        "JMAP Email/set create with an HTML body (feature_gate.rs); composer plain-text only"
    ),
    def!(
        "cap.compose.attachments",
        Kind::Bool,
        "true",
        GDA,
        "features",
        "JMAP Email/set create with attachment parts (feature_gate.rs); composer attach button"
    ),
    def!(
        "cap.compose.share_links",
        Kind::Bool,
        "true",
        GDA,
        "features",
        "POST /api/nextcloud/share-link (feature_gate.rs)"
    ),
    def!(
        "cap.compose.scheduled_send",
        Kind::Bool,
        "true",
        GDA,
        "features",
        "JMAP EmailSubmission/set with sendAt (feature_gate.rs)"
    ),
    def!(
        "cap.compose.read_receipts",
        Kind::Bool,
        "true",
        GDA,
        "features",
        "JMAP Email/set create with mailwomanRequestReadReceipt (feature_gate.rs)"
    ),
    def!(
        "cap.compose.open_tracking",
        Kind::Bool,
        "true",
        GDA,
        "features",
        "Allow outgoing tracking pixels; enforced on mint, bind and image recording using the sender's current policy"
    ),
    def!(
        "cap.reading.original_html",
        Kind::Bool,
        "true",
        GDA,
        "features",
        "Allow the isolated original HTML reader option; does not authorize scripts or unrestricted remote resources"
    ),
    def!(
        "cap.reading.delete_sender", Kind::Bool, "false", Scopes::GlobalDomain,
        "features", "Allow the optional reader action that moves all messages from a sender to Trash after confirmation"
    ),
    def!(
        "mail.reader.allowOriginalHtml",
        Kind::Bool,
        "false",
        GDU,
        "privacy",
        "Offer an explicit per-message original HTML formatting override; off by default and never selected automatically"
    ),
    def!(
        "cap.signatures",
        Kind::Bool,
        "true",
        GDA,
        "features",
        "writes under /api/account/signatures (feature_gate.rs)"
    ),
    def!(
        "cap.identities",
        Kind::Bool,
        "true",
        GDA,
        "features",
        "writes under /api/account/identities (feature_gate.rs)"
    ),
    def!(
        "cap.keys",
        Kind::Bool,
        "true",
        GDA,
        "features",
        "JMAP CryptoKey/* (feature_gate.rs); Keys module, compose sign and encrypt"
    ),
    def!(
        "cap.links.trusted",
        Kind::Bool,
        "true",
        GDA,
        "features",
        "PUT /api/settings/links.* (feature_gate.rs)"
    ),
    def!(
        "cap.zoom",
        Kind::Bool,
        "true",
        GDA,
        "features",
        "PUT /api/settings/reader.zoom (feature_gate.rs); reader zoom controls"
    ),
    def!(
        "cap.mail.focused_inbox",
        Kind::Bool,
        "true",
        GDA,
        "features",
        "PUT /api/settings/mail.focusedInbox (feature_gate.rs); Focused and Other tabs"
    ),
    def!(
        "cap.views.events",
        Kind::Bool,
        "true",
        GDA,
        "features",
        "JMAP Email/query on mwViewEvents (feature_gate.rs); Events view"
    ),
    def!(
        "cap.views.attachments",
        Kind::Bool,
        "true",
        GDA,
        "features",
        "Attachments view in the sidebar (web only: the same messages stay reachable by search)"
    ),
    def!(
        "cap.search.operators",
        Kind::Bool,
        "true",
        GDA,
        "features",
        "JMAP Email/query text with field operators (feature_gate.rs); search filter chips"
    ),
    def!(
        "cap.saved_searches",
        Kind::Bool,
        "true",
        GDA,
        "features",
        "writes under /api/account/saved-searches (feature_gate.rs)"
    ),
    def!(
        "cap.vacation",
        Kind::Bool,
        "true",
        GDA,
        "features",
        "JMAP VacationResponse/* (feature_gate.rs, besides cap.rules)"
    ),
    def!(
        "cap.labels",
        Kind::Bool,
        "true",
        GDA,
        "features",
        "JMAP Email/set adding a non-system keyword (feature_gate.rs)"
    ),
    def!(
        "cap.snooze",
        Kind::Bool,
        "true",
        GDA,
        "features",
        "JMAP Email/set snoozedUntil (feature_gate.rs)"
    ),
    def!(
        "cap.follow_up",
        Kind::Bool,
        "true",
        GDA,
        "features",
        "JMAP Email/set followUpAt (feature_gate.rs)"
    ),
    def!(
        "cap.masked",
        Kind::Bool,
        "true",
        GDA,
        "features",
        "/api/masked (feature_gate.rs)"
    ),
    def!(
        "cap.directory",
        Kind::Bool,
        "true",
        GDA,
        "features",
        "/api/directory (feature_gate.rs)"
    ),
    def!(
        "cap.mcp",
        Kind::Bool,
        "true",
        GDA,
        "features",
        "/mcp with a key or token (scope_mw.rs)"
    ),
    def!(
        "cap.appearance.custom",
        Kind::Bool,
        "true",
        GDA,
        "features",
        "PUT /api/account/appearance (feature_gate.rs); theme, accent, custom and badge colours, density, font"
    ),
    def!(
        "cap.appearance.custom_code",
        Kind::Bool,
        "true",
        GDA,
        "features",
        "PUT /api/account/appearance/customization (feature_gate.rs), below appearance.customCssEnabled/customJsEnabled"
    ),
    def!(
        "cap.avatars",
        Kind::Bool,
        "true",
        GDA,
        "features",
        "PUT /api/accounts/layout with avatars (feature_gate.rs)"
    ),
    def!(
        "cap.shortcuts",
        Kind::Bool,
        "true",
        GDA,
        "features",
        "PUT /api/settings/shortcuts.* (feature_gate.rs); shortcut editor"
    ),
    def!(
        "cap.analytics",
        Kind::Bool,
        "true",
        GDA,
        "features",
        "POST /api/analytics/events (feature_gate.rs); consent prompt"
    ),
    def!(
        "cap.apps.native",
        Kind::Bool,
        "true",
        GDA,
        "features",
        "native bearer sessions (feature_gate.rs)"
    ),
    def!(
        "access.protocols",
        Kind::StringList {
            max_len: 3,
            max_item: 8
        },
        "[\"imap\",\"pop3\",\"ews\"]",
        GDA,
        "features",
        "mail protocols a native account may use (imap, pop3, ews); /jmap and /api/v1 refuse the others (feature_gate.rs)"
    ),
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
        "privacy.trackerOverrideAllowed",
        Kind::Bool,
        "true",
        GDU,
        "privacy",
        "Offer explicit per-message tracker consent; a locked privacy.trackers=true still forbids this override"
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
        "images.trustedHosts",
        Kind::StringList {
            max_len: 100,
            max_item: 253
        },
        "[]",
        GDU,
        "privacy",
        "image hosts (and their subdomains) whose remote images load without asking; the reader sets one state per host"
    ),
    def!(
        "images.blockedHosts",
        Kind::StringList {
            max_len: 100,
            max_item: 253
        },
        "[]",
        GDU,
        "privacy",
        "image hosts (and their subdomains) whose remote images never load; wins over a trusted host and over any grant"
    ),
    def!(
        "links.trustedDomains",
        Kind::StringList {
            max_len: 100,
            max_item: 253
        },
        "[]",
        GDU,
        "privacy",
        "exact trusted link domains that open from Review links without the destination step"
    ),
    def!(
        "links.trustedSenders",
        Kind::StringList {
            max_len: 100,
            max_item: 254
        },
        "[]",
        GDU,
        "privacy",
        "exact sender mailboxes whose reviewed message links may open without per-message consent"
    ),
    def!(
        "links.trustedSubdomains",
        Kind::Bool,
        "false",
        GDU,
        "privacy",
        "whether trusted link domains also trust their subdomains"
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
        "imageProxy",
        "Remote image cache size in bytes"
    ),
    def!(
        "privacy.ratePerMin",
        int(10, 60000),
        "1200",
        G,
        "imageProxy",
        "Remote fetches per account per minute (the fixed refill in manual rate mode)"
    ),
    def!(
        "privacy.imageConcurrencyMode",
        Kind::Enum(&["auto", "conservative", "balanced", "throughput", "manual"]),
        "\"auto\"",
        G,
        "privacy",
        "Image concurrency preset; automatic sizing uses instance CPU, container memory and authorized message image counts within explicit numeric ceilings"
    ),
    def!(
        "privacy.maxConcurrentGlobal",
        int(1, 256),
        "128",
        G,
        "privacy",
        "Maximum active remote-image network fetches per replica"
    ),
    def!(
        "privacy.maxConcurrentPipeline",
        int(1, 256),
        "128",
        G,
        "privacy",
        "Maximum in-flight remote images including fetched buffers and decode work per replica"
    ),
    def!(
        "privacy.maxConcurrentDecodes",
        int(1, 16),
        "10",
        G,
        "privacy",
        "Maximum simultaneous sandboxed image decoders per replica"
    ),
    def!(
        "privacy.maxQueuedRequests",
        int(16, 16384),
        "2048",
        G,
        "privacy",
        "Maximum waiters and distinct remote-image flights per replica"
    ),
    def!(
        "privacy.rateBurst",
        int(1, 12000),
        "1024",
        G,
        "imageProxy",
        "Remote-image burst allowance per account (the fixed burst in manual rate mode), separate from sustained refill"
    ),
    def!(
        "privacy.imageRateMode",
        Kind::Enum(&["auto", "manual"]),
        "\"auto\"",
        G,
        "imageProxy",
        "Per-account image fetch rate sizing: auto scales burst and refill from replica capacity, queue load and message image count within the floors and ceilings; manual uses privacy.rateBurst and privacy.ratePerMin exactly"
    ),
    def!(
        "privacy.imageRateBurstFloor",
        int(1, 12000),
        "64",
        G,
        "imageProxy",
        "Lowest per-account burst allowance in automatic rate mode"
    ),
    def!(
        "privacy.imageRateBurstCeiling",
        int(1, 12000),
        "2048",
        G,
        "imageProxy",
        "Highest per-account burst allowance in automatic rate mode"
    ),
    def!(
        "privacy.imageRateRefillFloor",
        int(10, 60000),
        "300",
        G,
        "imageProxy",
        "Lowest per-account refill, tokens per minute, in automatic rate mode"
    ),
    def!(
        "privacy.imageRateRefillCeiling",
        int(10, 60000),
        "6000",
        G,
        "imageProxy",
        "Highest per-account refill, tokens per minute, in automatic rate mode"
    ),
    def!(
        "privacy.imageCacheMode",
        Kind::Enum(&["auto", "manual"]),
        "\"auto\"",
        G,
        "imageProxy",
        "Remote image cache sizing: auto derives bytes, entries and lifetime from the memory left after decoder and pipeline reservations and the observed hit rate; manual uses privacy.cacheBytes, privacy.cacheEntries and privacy.cacheTtlSecs exactly"
    ),
    def!(
        "privacy.imageCacheBytesFloor",
        int(0, 1073741824),
        "16777216",
        G,
        "imageProxy",
        "Smallest cache size in bytes in automatic cache mode"
    ),
    def!(
        "privacy.imageCacheBytesCeiling",
        int(0, 1073741824),
        "268435456",
        G,
        "imageProxy",
        "Largest cache size in bytes in automatic cache mode"
    ),
    def!(
        "privacy.cacheEntries",
        int(16, 65536),
        "4096",
        G,
        "imageProxy",
        "Most cached remote images; the fixed count in manual cache mode, the ceiling in automatic mode"
    ),
    def!(
        "privacy.cacheEntryBytes",
        int(65536, 16777216),
        "4194304",
        G,
        "imageProxy",
        "Largest single re-encoded image kept in the cache, in bytes"
    ),
    def!(
        "privacy.imageCacheTtlFloorSecs",
        int(0, 86400),
        "300",
        G,
        "imageProxy",
        "Shortest cache lifetime in seconds in automatic cache mode, used on a low hit rate or under memory pressure"
    ),
    def!(
        "privacy.imageCacheTtlCeilingSecs",
        int(0, 86400),
        "21600",
        G,
        "imageProxy",
        "Longest cache lifetime in seconds in automatic cache mode, used on a high hit rate with free memory"
    ),
    def!(
        "privacy.cacheTtlSecs",
        int(0, 86400),
        "3600",
        G,
        "imageProxy",
        "Maximum age of cached re-encoded remote images in seconds; zero disables reuse"
    ),
    def!(
        "privacy.imageRamMaxMiB",
        int(0, 1048576),
        "0",
        G,
        "imageProxy",
        "Extra cap in MiB on the RAM tier of the image cache; zero leaves the automatic or manual cache size as the only limit"
    ),
    def!(
        "privacy.imageDiskEnabled",
        Kind::Bool,
        "false",
        G,
        "imageProxy",
        "Keep re-encoded remote images on disk as a second cache tier behind the RAM tier; off by default"
    ),
    def!(
        "privacy.imageDiskPath",
        text(Text::Plain, 512),
        "\"\"",
        G,
        "imageProxy",
        "Absolute directory for the image disk tier; empty uses an image-cache folder next to the database file"
    ),
    def!(
        "privacy.imageDiskMode",
        Kind::Enum(&["auto", "manual"]),
        "\"auto\"",
        G,
        "imageProxy",
        "Disk tier size: auto takes a tenth of the free space between the floor and ceiling; manual uses privacy.imageDiskMaxMiB exactly"
    ),
    def!(
        "privacy.imageDiskMaxMiB",
        int(1, 1048576),
        "512",
        G,
        "imageProxy",
        "Disk tier size in MiB in manual disk mode"
    ),
    def!(
        "privacy.imageDiskFloorMiB",
        int(0, 1048576),
        "64",
        G,
        "imageProxy",
        "Smallest disk tier size in MiB in automatic disk mode"
    ),
    def!(
        "privacy.imageDiskCeilingMiB",
        int(0, 1048576),
        "4096",
        G,
        "imageProxy",
        "Largest disk tier size in MiB in automatic disk mode"
    ),
    def!(
        "privacy.imageDiskEntries",
        int(16, 1048576),
        "16384",
        G,
        "imageProxy",
        "Most images kept on disk; also bounds the in-memory index"
    ),
    def!(
        "privacy.imageDiskEntryBytes",
        int(65536, 67108864),
        "8388608",
        G,
        "imageProxy",
        "Largest single re-encoded image written to disk, in bytes"
    ),
    def!(
        "privacy.imageDiskMinHits",
        int(0, 100),
        "1",
        G,
        "imageProxy",
        "Hits an entry needs before an eviction from RAM writes it to disk instead of dropping it"
    ),
    def!(
        "privacy.imageDiskWriteQueueMiB",
        int(1, 256),
        "8",
        G,
        "imageProxy",
        "Bytes in MiB that may wait for the background disk writer; past it, evictions are dropped"
    ),
    def!(
        "privacy.imageTtlStepSecs",
        int(0, 86400),
        "600",
        G,
        "imageProxy",
        "Base seconds added to an entry's remaining lifetime on each hit, scaled up with its decayed hit count; zero keeps every entry at the base lifetime"
    ),
    def!(
        "privacy.imageTtlMaxSecs",
        int(60, 2592000),
        "86400",
        G,
        "imageProxy",
        "Longest remaining lifetime in seconds an entry can accumulate through hits"
    ),
    def!(
        "privacy.imageMaxAgeSecs",
        int(3600, 7776000),
        "604800",
        G,
        "imageProxy",
        "Absolute age in seconds after which an entry is refetched no matter how often it is hit"
    ),
    def!(
        "privacy.imageScoreHalfLifeSecs",
        int(60, 604800),
        "3600",
        G,
        "imageProxy",
        "Half-life in seconds of the hit count and recency used in the priority score"
    ),
    def!(
        "privacy.imageScoreDemoteMilli",
        int(0, 10000),
        "300",
        G,
        "imageProxy",
        "Priority score in thousandths below which a RAM entry may be demoted to disk"
    ),
    def!(
        "privacy.imageScoreHysteresisPercent",
        int(0, 500),
        "50",
        G,
        "imageProxy",
        "Percentage the promote threshold sits above the demote threshold, so entries do not flap between tiers"
    ),
    def!(
        "privacy.imageContextOpenPercent",
        int(100, 1000),
        "200",
        G,
        "imageProxy",
        "Score weight in percent for images requested for a message that is open"
    ),
    def!(
        "privacy.imageContextRecentPercent",
        int(100, 1000),
        "150",
        G,
        "imageProxy",
        "Score weight in percent for recently used images; background loads weigh 100"
    ),
    def!(
        "privacy.imageAccountSharePercent",
        int(1, 100),
        "50",
        G,
        "imageProxy",
        "Largest share in percent of the RAM tier one account may fill while other accounts are present"
    ),
    def!(
        "privacy.imageSweepSecs",
        int(5, 3600),
        "30",
        G,
        "imageProxy",
        "Seconds between background maintenance sweeps of the image cache"
    ),
    def!(
        "privacy.imageSnapshotSecs",
        int(0, 86400),
        "300",
        G,
        "imageProxy",
        "Seconds between atomic snapshots of the disk index; zero turns snapshots off and restarts read each file header instead"
    ),
    def!(
        "privacy.imageSweepBatch",
        int(16, 8192),
        "256",
        G,
        "imageProxy",
        "Entries one maintenance sweep may expire, rescore or evict"
    ),
    def!(
        "privacy.imageCacheStaleSecs",
        int(0, 86400),
        "3600",
        G,
        "imageProxy",
        "Seconds past its freshness a cached image is still served at once while it is revalidated in the background (If-None-Match / If-Modified-Since, one revalidation per image); zero turns stale-while-revalidate off"
    ),
    def!(
        "privacy.imageCacheMaxStaleSecs",
        int(0, 604800),
        "86400",
        G,
        "imageProxy",
        "Seconds past its freshness a cached image is kept to answer when the origin cannot be reached; never below the stale-while-revalidate window"
    ),
    def!(
        "privacy.imageResizeEnabled",
        Kind::Bool,
        "false",
        G,
        "imageProxy",
        "Scale oversized remote images down to the width the reader's frame asks for (320, 640, 960, 1280 or 1600 pixels) after the sandboxed decode; animated images pass through unchanged"
    ),
    def!(
        "privacy.imageResizeMaxWidth",
        int(320, 1600),
        "1600",
        G,
        "imageProxy",
        "Widest image the resize will produce, in pixels; a frame asking for more gets this width"
    ),
    def!(
        "privacy.imageResizeMaxPixels",
        int(100000, 64000000),
        "4000000",
        G,
        "imageProxy",
        "Largest decoded image, in pixels, the resize will touch; a larger one is served unscaled (checked from the image header before any pixel is decoded)"
    ),
    def!(
        "privacy.imageResizeFormat",
        Kind::Enum(&["auto", "png"]),
        "\"auto\"",
        G,
        "imageProxy",
        "Format of a resized image; auto picks the best encoder in the build, which is PNG (there is no lossy WebP encoder)"
    ),
    def!(
        "privacy.negativeCacheTtlSecs",
        int(0, 300),
        "10",
        G,
        "privacy",
        "Maximum age of remembered remote-image failures in seconds; zero disables negative caching"
    ),
    def!(
        "privacy.imageLatencyTracking",
        Kind::Bool,
        "true",
        G,
        "imageProxy",
        "Record image proxy speed (median, p95, p99 and phase timings) as aggregate histogram buckets in memory; no URLs, hosts or accounts are stored"
    ),
    def!(
        "privacy.imageRetryAttempts",
        int(0, 8),
        "3",
        GDU,
        "privacy",
        "Automatic retry rounds for transient image failures; zero disables automatic retries"
    ),
    def!(
        "privacy.imageRetryDelayMs",
        int(500, 60000),
        "5000",
        GDU,
        "privacy",
        "Delay between automatic transient image retry rounds in milliseconds"
    ),
    def!(
        "privacy.fetchConnectTimeoutMs",
        int(500, 30000),
        "5000",
        G,
        "privacy",
        "Remote image connect and TLS handshake timeout in milliseconds"
    ),
    def!(
        "privacy.fetchHeadersTimeoutMs",
        int(500, 60000),
        "10000",
        G,
        "privacy",
        "Remote image time to response headers in milliseconds"
    ),
    def!(
        "privacy.fetchIdleTimeoutMs",
        int(500, 60000),
        "10000",
        G,
        "privacy",
        "Remote image longest pause between body chunks in milliseconds"
    ),
    def!(
        "privacy.fetchTotalTimeoutMs",
        int(1000, 120000),
        "20000",
        G,
        "privacy",
        "Remote image fetch work budget in milliseconds, separate from bounded queue waiting"
    ),
    def!(
        "privacy.imageDecodeTimeoutMs",
        int(1000, 60000),
        "10000",
        G,
        "privacy",
        "Maximum sandboxed image decode time in milliseconds, separate from network and queue budgets"
    ),
    def!(
        "privacy.fetchQueueTimeoutMs",
        int(1000, 180000),
        "60000",
        G,
        "privacy",
        "Maximum queue wait budget for remote images in milliseconds; actual fetches retain their own total timeout"
    ),
    def!(
        "privacy.maxRedirects",
        int(0, 10),
        "3",
        G,
        "privacy",
        "Redirects followed per remote image"
    ),
    def!(
        "privacy.maxConcurrentPerMessage",
        int(1, 128),
        "30",
        G,
        "privacy",
        "Remote image fetches at a time per message"
    ),
    def!(
        "privacy.maxConcurrentPerAccount",
        int(1, 256),
        "128",
        G,
        "privacy",
        "Remote image fetches at a time per account"
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
        "session.persistAcrossRestarts",
        Kind::Bool,
        "true",
        G,
        "signin",
        "Keep mailbox, native and console sessions across a server restart; off ends them all at start-up"
    ),
    def!(
        "session.lastActivityFlushSeconds",
        int(0, 300),
        "15",
        G,
        "signin",
        "Write a session's last-seen time at most this often; zero writes on every request"
    ),
    def!(
        "server.shutdownGraceSeconds",
        int(1, 120),
        "30",
        G,
        "maintenance",
        "How long a stopping server lets in-flight requests finish before it closes connections"
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
        "twofa.passkeyAttachment",
        Kind::Enum(&["any", "platform", "cross-platform"]),
        "\"any\"",
        GDA,
        "signin",
        "Which authenticators can hold a mailbox passkey"
    ),
    def!(
        "twofa.passkeyUserVerification",
        Kind::Enum(&["required", "preferred", "discouraged"]),
        "\"preferred\"",
        GDA,
        "signin",
        "Whether a passkey must verify the user (PIN or biometric)"
    ),
    def!(
        "twofa.passkeyResidentKey",
        Kind::Enum(&["required", "preferred", "discouraged"]),
        "\"discouraged\"",
        GDA,
        "signin",
        "Whether a passkey must be a discoverable credential"
    ),
    def!(
        "twofa.passkeyLegacy",
        Kind::Enum(&["allow", "refuse"]),
        "\"allow\"",
        GDA,
        "signin",
        "Passkeys registered before a stricter passkey policy"
    ),
    def!(
        "twofa.lastFactorRemoval",
        // t77: `block` keeps the last passkey or TOTP until another factor is
        // added. The t39-e6 value `requireRecoveryCodes` is accepted on write
        // as `block` (see `validate`) and rewritten by migration 0074.
        Kind::Enum(&["allow", "block"]),
        "\"allow\"",
        GDA,
        "signin",
        "Removing the last second factor of an account"
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
    def!(
        "mail.externalRecipientWarning",
        Kind::Enum(&["off", "warn", "confirm", "warn-remember", "deny"]),
        "\"warn\"",
        GDU,
        "mail",
        "external recipient composer warning"
    ),
    def!(
        "mail.replyImages",
        Kind::Enum(&["ask", "allow", "trusted", "deny"]),
        "\"ask\"",
        GDU,
        "mail",
        "whether a reply or forward includes the original's images"
    ),
    def!(
        "mail.replyImageTrustedDomains",
        Kind::StringList {
            max_len: 200,
            max_item: 253
        },
        "[]",
        GDU,
        "mail",
        "sender domains whose images are included in replies and forwards without asking"
    ),
    def!(
        "mail.trustedRecipientDomains",
        Kind::StringList {
            max_len: 100,
            max_item: 253
        },
        "[]",
        GDU,
        "mail",
        "exact trusted recipient domains for composer warnings"
    ),
    def!(
        "mail.compose.checkInvalidRecipients",
        Kind::Enum(&["allow", "warn", "warn-remember", "deny"]),
        "\"warn\"",
        GDU,
        "mail",
        "pre-send check for invalid recipient addresses"
    ),
    def!(
        "mail.compose.checkMissingAttachment",
        Kind::Enum(&["allow", "warn", "warn-remember", "deny"]),
        "\"warn\"",
        GDU,
        "mail",
        "pre-send check for a message that mentions an attachment but has none"
    ),
    def!(
        "mail.compose.checkEmptySubject",
        Kind::Enum(&["allow", "warn", "warn-remember", "deny"]),
        "\"warn\"",
        GDU,
        "mail",
        "pre-send check for an empty subject"
    ),
    def!(
        "mail.compose.headerCollapse",
        Kind::Enum(&["expanded", "collapsed", "auto"]),
        "\"expanded\"",
        GDU,
        "mail",
        "composer header (To, Cc, Bcc, Subject): expanded, folded to a summary line, or folded while writing the body and open on focus"
    ),
    def!(
        "mail.compose.headerAutoCollapseSeconds",
        int(0, 60),
        "0",
        GDU,
        "mail",
        "seconds the composer body must hold focus before a complete header (recipient and subject) folds to its summary line; 0 never folds it on its own"
    ),
    def!(
        "mail.compose.autoChipRecipients",
        Kind::Bool,
        "true",
        GDU,
        "mail",
        "finish a typed recipient when a space, comma or semicolon follows it"
    ),
    def!(
        "mail.compose.recipientSeparator",
        Kind::Enum(&["comma", "semicolon"]),
        "\"comma\"",
        GDU,
        "mail",
        "separator inserted after a finished recipient"
    ),
    def!(
        "time.displayZone",
        text(Text::Plain, 64),
        "\"local\"",
        GDU,
        "time",
        "preferred message display time zone"
    ),
    def!(
        "time.comparisonZone",
        text(Text::Plain, 64),
        "\"\"",
        GDU,
        "time",
        "optional second time zone for message and event helpers"
    ),
    def!(
        "time.showHelpers",
        Kind::Bool,
        "true",
        GDU,
        "time",
        "show message and event time zone comparisons"
    ),
    def!(
        "shortcuts.enabled",
        Kind::Bool,
        "true",
        GDU,
        "shortcuts",
        "enable application keyboard shortcuts"
    ),
    def!(
        "shortcuts.compose",
        text(Text::Plain, 64),
        "\"inherit\"",
        GDU,
        "shortcuts",
        "keyboard binding for compose"
    ),
    def!(
        "shortcuts.archive",
        text(Text::Plain, 64),
        "\"inherit\"",
        GDU,
        "shortcuts",
        "keyboard binding for archive"
    ),
    def!(
        "shortcuts.reply",
        text(Text::Plain, 64),
        "\"inherit\"",
        GDU,
        "shortcuts",
        "keyboard binding for reply"
    ),
    def!(
        "shortcuts.replyAll",
        text(Text::Plain, 64),
        "\"inherit\"",
        GDU,
        "shortcuts",
        "keyboard binding for replyAll"
    ),
    def!(
        "shortcuts.forward",
        text(Text::Plain, 64),
        "\"inherit\"",
        GDU,
        "shortcuts",
        "keyboard binding for forward"
    ),
    def!(
        "shortcuts.next",
        text(Text::Plain, 64),
        "\"inherit\"",
        GDU,
        "shortcuts",
        "keyboard binding for next"
    ),
    def!(
        "shortcuts.previous",
        text(Text::Plain, 64),
        "\"inherit\"",
        GDU,
        "shortcuts",
        "keyboard binding for previous"
    ),
    def!(
        "shortcuts.search",
        text(Text::Plain, 64),
        "\"inherit\"",
        GDU,
        "shortcuts",
        "keyboard binding for search"
    ),
    def!(
        "shortcuts.refresh",
        text(Text::Plain, 64),
        "\"inherit\"",
        GDU,
        "shortcuts",
        "keyboard binding for refresh"
    ),
    def!(
        "shortcuts.settings",
        text(Text::Plain, 64),
        "\"inherit\"",
        GDU,
        "shortcuts",
        "keyboard binding for settings"
    ),
    def!(
        "shortcuts.trash",
        text(Text::Plain, 64),
        "\"inherit\"",
        GDU,
        "shortcuts",
        "keyboard binding for trash"
    ),
    def!(
        "shortcuts.markRead",
        text(Text::Plain, 64),
        "\"inherit\"",
        GDU,
        "shortcuts",
        "keyboard binding for markRead"
    ),
    def!(
        "shortcuts.markUnread",
        text(Text::Plain, 64),
        "\"inherit\"",
        GDU,
        "shortcuts",
        "keyboard binding for markUnread"
    ),
    def!(
        "shortcuts.spam",
        text(Text::Plain, 64),
        "\"inherit\"",
        GDU,
        "shortcuts",
        "keyboard binding for spam"
    ),
    def!(
        "shortcuts.zoomIn",
        text(Text::Plain, 64),
        "\"inherit\"",
        GDU,
        "shortcuts",
        "keyboard binding for zoomIn"
    ),
    def!(
        "shortcuts.zoomOut",
        text(Text::Plain, 64),
        "\"inherit\"",
        GDU,
        "shortcuts",
        "keyboard binding for zoomOut"
    ),
    def!(
        "shortcuts.zoomReset",
        text(Text::Plain, 64),
        "\"inherit\"",
        GDU,
        "shortcuts",
        "keyboard binding for zoomReset"
    ),
    def!(
        "shortcuts.openInTab",
        text(Text::Plain, 64),
        "\"inherit\"",
        GDU,
        "shortcuts",
        "keyboard binding for openInTab"
    ),
    def!(
        "mail.quoteStyle",
        Kind::Enum(&["classic", "outlook"]),
        "\"classic\"",
        GDU,
        "mail",
        "reply and forward header style"
    ),
    // ── Interface (user-facing) ──────────────────────────────────────────────
    def!(
        "mail.quickReply",
        Kind::Bool,
        "false",
        GDU,
        "mail",
        "enable the compact message reply composer"
    ),
    def!(
        "mail.requestReadReceipt",
        Kind::Bool,
        "false",
        GDU,
        "mail",
        "request a read receipt by default when composing supported mail"
    ),
    def!(
        "mail.requestDeliveryReceipt",
        Kind::Bool,
        "false",
        GDU,
        "mail",
        "request delivery status notifications by default when the account supports DSN"
    ),
    def!(
        "mail.showFrom",
        Kind::Bool,
        "false",
        GDU,
        "mail",
        "show the From field by default when composing; sending authority is unchanged"
    ),
    def!(
        "mail.replyFromMode",
        Kind::Enum(&["match", "default", "ask", "keep"]),
        "\"match\"",
        GDU,
        "mail",
        "which identity replies and forwards are sent from: the one the original was addressed to, the default, a chooser, or the current one"
    ),
    def!(
        "mail.replyFrom.headerPriority",
        Kind::StringList {
            max_len: 8,
            max_item: 32
        },
        "[\"Delivered-To\",\"X-Original-To\",\"To\",\"Cc\"]",
        GDU,
        "mail",
        "which message headers name the address a reply is sent from, in order: Delivered-To, X-Original-To, Envelope-To, X-Forwarded-To, Resent-To, To, Cc"
    ),
    def!(
        "mail.modifierSelection",
        Kind::Bool,
        "true",
        GDU,
        "mail",
        "select messages with Ctrl, Command, Shift and arrow keys"
    ),
    def!(
        "mail.draftSaveMode",
        Kind::Enum(&["realtime", "periodic", "manual"]),
        "\"realtime\"",
        GDU,
        "mail",
        "account draft save scheduling; manual disables automatic upstream writes"
    ),
    def!(
        "mail.draftDebounceMs",
        int(500, 10000),
        "1000",
        GDU,
        "mail",
        "idle milliseconds before saving an edited account draft"
    ),
    def!(
        "mail.draftIntervalSecs",
        int(5, 3600),
        "30",
        GDU,
        "mail",
        "periodic account draft save interval in seconds"
    ),
    def!(
        "mail.send.recallDelaySeconds",
        int(0, 1800),
        "10",
        GDU,
        "mail",
        "Undo-send window in seconds: how long a sent message waits in the Outbox and can be recalled; 0 sends at once, capped by mail.send.recallDelayMaxSeconds"
    ),
    def!(
        "mail.send.recallDelayMaxSeconds",
        int(0, 1800),
        "1800",
        GD,
        "mail",
        "Longest undo-send window a user may choose, in seconds (enforced by the engine)"
    ),
    def!(
        "mail.send.allowRecall",
        Kind::Bool,
        "true",
        GD,
        "mail",
        "Allow recalling a message during the undo-send window; off sends immediately (scheduled sends are unaffected)"
    ),
    def!(
        "mail.send.maxDelayedSeconds",
        int(0, 31536000),
        "7776000",
        GD,
        "mail",
        "How far ahead a message may be scheduled, in seconds (default 90 days, maximum 1 year; 0 turns scheduling off)"
    ),
    def!(
        "mail.send.maxQueued",
        int(1, 5000),
        "200",
        GD,
        "mail",
        "Queued or scheduled messages allowed per mailbox"
    ),
    def!(
        "mail.resumeTabs",
        Kind::Bool,
        "true",
        GDU,
        "mail",
        "restore account-owned workspace tab references after reload"
    ),
    def!(
        "mail.dragToFolder",
        Kind::Bool,
        "true",
        GDU,
        "mail",
        "move selected messages by dragging them to a folder"
    ),
    def!(
        "mail.folderCounts",
        Kind::Enum(&["none", "unread", "unreadIfAny", "total", "both"]),
        "\"both\"",
        GDU,
        "mail",
        "counts displayed beside mail folders (unreadIfAny: unread count, hidden at 0); account wide across linked mailboxes"
    ),
    def!(
        "mail.folderCountScope",
        Kind::Enum(&["all", "main"]),
        "\"all\"",
        GDU,
        "mail",
        "which folders show counts: all, or only folders with a role (Inbox, Drafts, Sent, Archive, Junk, Trash, Outbox, other special-use); account wide"
    ),
    def!(
        "mail.compactFolderBadges",
        Kind::Bool,
        "true",
        GDU,
        "mail",
        "smaller count badges for folders other than Inbox"
    ),
    def!(
        "mail.focusedInbox",
        Kind::Bool,
        "false",
        GDU,
        "mail",
        "split Inbox into Focused and Other tabs (newsletters and automated senders go to Other); off by default"
    ),
    def!(
        "mail.priorityMarks",
        Kind::Enum(&["both", "high", "off"]),
        r#""both""#,
        GDU,
        "mail",
        "Marks on high and low priority messages: both, high only, or off"
    ),
    def!(
        "mail.priorityMarks.style",
        Kind::Enum(&["icon", "iconText", "bar"]),
        r#""icon""#,
        GDU,
        "mail",
        "Priority mark style: icon, icon with text, or bar"
    ),
    def!(
        "mail.badge.auth",
        Kind::Enum(&["always", "notable", "never"]),
        r#""always""#,
        GDU,
        "mail",
        "Reader header: when the DMARC authentication badge shows (always, only when notable, never)"
    ),
    def!(
        "mail.badge.spam",
        Kind::Enum(&["always", "notable", "never"]),
        r#""always""#,
        GDU,
        "mail",
        "Reader header: when the spam score badge shows (always, only when notable, never)"
    ),
    def!(
        "mail.badge.sanitized",
        Kind::Enum(&["always", "notable", "never"]),
        r#""notable""#,
        GDU,
        "mail",
        "Reader header: when the sanitized HTML badge shows (always, only when notable, never)"
    ),
    def!(
        "mail.badge.folder",
        Kind::Enum(&["always", "notable", "never"]),
        r#""always""#,
        GDU,
        "mail",
        "Reader header: when the folder badge shows (always, only when notable, never)"
    ),
    def!(
        "mail.badge.list",
        Kind::Enum(&["always", "notable", "never"]),
        r#""always""#,
        GDU,
        "mail",
        "Reader header: when the mailing list badge shows (always, only when notable, never); it needs a List-Id or List-Unsubscribe header"
    ),
    def!(
        "mail.list.unsubscribeMode",
        Kind::Enum(&["ask", "one-click-direct", "open-link"]),
        r#""ask""#,
        GDU,
        "mail",
        "Mailing list Unsubscribe: ask first and send the one-click request, send it at once, or only open the list's link or draft"
    ),
    def!(
        "mail.list.keep",
        Kind::StringList {
            max_len: 500,
            max_item: 300
        },
        "[]",
        GDU,
        "mail",
        "Senders and mailing lists the user always keeps: sender:address or list:List-Id. Clean up this sender, unsubscribe suggestions and retention rules skip them"
    ),
    def!(
        "mail.badge.density",
        Kind::Enum(&["compact", "comfortable"]),
        r#""compact""#,
        GDU,
        "mail",
        "Reader header badge size: compact or comfortable"
    ),
    def!(
        "mail.badge.labels",
        Kind::Enum(&["text", "icon"]),
        r#""text""#,
        GDU,
        "mail",
        "Reader header badges: icon with text, or icon only"
    ),
    def!(
        "mail.compose.priority.enabled",
        Kind::Bool,
        r#"true"#,
        GDU,
        "mail",
        "Priority selector (Normal, High, Low) in the composer"
    ),
    def!(
        "mail.compose.priority.default",
        Kind::Enum(&["normal", "high", "low"]),
        r#""normal""#,
        GDU,
        "mail",
        "Priority a new message starts with"
    ),
    def!(
        "mail.priority.sendHeaders",
        Kind::StringList {
            max_len: 3,
            max_item: 32
        },
        r#"["Importance","X-Priority","X-MSMail-Priority"]"#,
        GD,
        "mail",
        "Headers written for a non-normal priority: Importance, X-Priority, X-MSMail-Priority"
    ),
    def!(
        "mail.priority.trustSenderHeaders",
        Kind::Enum(&["always", "trusted", "never"]),
        r#""always""#,
        GD,
        "mail",
        "Whose priority headers are honoured: always, only when DMARC passed aligned, or never"
    ),
    def!(
        "properties.folder.enabled",
        Kind::Bool,
        r#"true"#,
        GDU,
        "mail",
        "Folder Properties dialog"
    ),
    def!(
        "properties.message.enabled",
        Kind::Bool,
        r#"true"#,
        GDU,
        "mail",
        "Message Properties dialog"
    ),
    def!(
        "properties.folder.sections",
        Kind::StringList {
            max_len: 10,
            max_item: 24
        },
        r#"["general","remote","size","offline","overrides","notes","actions"]"#,
        GD,
        "mail",
        "Sections of the folder dialog: general, remote, size, offline, overrides, notes, actions"
    ),
    def!(
        "properties.message.sections",
        Kind::StringList {
            max_len: 10,
            max_item: 24
        },
        r#"["summary","scoring","headers","attachments","actions"]"#,
        GD,
        "mail",
        "Sections of the message dialog: summary, scoring, headers, attachments, actions"
    ),
    def!(
        "properties.remoteLocation.visibility",
        Kind::Enum(&["full", "masked-host", "path-only", "hidden"]),
        r#""masked-host""#,
        GD,
        "mail",
        "Server location shown to users: full, masked host, path only, or hidden"
    ),
    def!(
        "properties.headers.raw",
        Kind::Bool,
        r#"true"#,
        GD,
        "mail",
        "Raw message headers in the message dialog"
    ),
    def!(
        "properties.scoring.show",
        Kind::Bool,
        r#"true"#,
        GD,
        "mail",
        "Spam score and authentication details in the message dialog"
    ),
    def!(
        "properties.folder.overrides.allowed",
        Kind::StringList {
            max_len: 12,
            max_item: 24
        },
        r#"["counters","badgeColour","compactBadge","notifications","sort","threading","previewLines","markReadDelay","replyIdentity","unifiedInbox","hideWhenEmpty"]"#,
        GD,
        "mail",
        "Per-folder overrides users may set: counters, badgeColour, compactBadge, notifications, sort, threading, previewLines, markReadDelay, replyIdentity, unifiedInbox, hideWhenEmpty"
    ),
    def!(
        "properties.folder.notes.enabled",
        Kind::Bool,
        r#"true"#,
        GDU,
        "mail",
        "Folder notes"
    ),
    def!(
        "properties.folder.notes.maxLength",
        int(100, 2000),
        r#"2000"#,
        GD,
        "mail",
        "Longest folder note, in characters"
    ),
    def!(
        "properties.folder.sizeCompute",
        Kind::Enum(&["off", "on-open", "background"]),
        r#""on-open""#,
        GD,
        "mail",
        "When a folder's size on the server is computed: off, when the dialog opens, or in the background"
    ),
    def!(
        "properties.offlineClear.enabled",
        Kind::Bool,
        r#"true"#,
        GD,
        "mail",
        "Clear the offline cache of one folder from its dialog"
    ),
    def!(
        "export.folder.enabled",
        Kind::Bool,
        r#"true"#,
        GD,
        "mail",
        "Download a folder as a ZIP"
    ),
    def!(
        "export.folder.format",
        Kind::Enum(&["zip-eml", "zip-mbox", "both"]),
        r#""both""#,
        GD,
        "mail",
        "Folder export format: ZIP of .eml files, ZIP with an mbox, or both"
    ),
    def!(
        "export.folder.maxBytes",
        int(1048576, 107374182400),
        r#"2147483648"#,
        GD,
        "mail",
        "Largest folder export, in bytes"
    ),
    def!(
        "export.folder.maxMessages",
        int(1, 1000000),
        r#"100000"#,
        GD,
        "mail",
        "Most messages in one folder export"
    ),
    def!(
        "export.folder.includeSubfolders",
        Kind::Bool,
        r#"true"#,
        GDU,
        "mail",
        "Folder export includes subfolders by default"
    ),
    def!(
        "export.message.download.enabled",
        Kind::Bool,
        r#"true"#,
        GD,
        "mail",
        "Download a message as .eml, headers only or mbox"
    ),
    def!(
        "export.dialog.enabled",
        Kind::Bool,
        r#"true"#,
        GDU,
        "mail",
        "Export dialog in the message view and the message menus"
    ),
    def!(
        "export.formats",
        Kind::StringList {
            max_len: 12,
            max_item: 8
        },
        r#"["eml","mbox","txt","md","msg","docx","csv","json","pdf"]"#,
        GD,
        "mail",
        "Formats the export dialog offers: eml, mbox, txt, md, msg, docx, csv, json, pdf"
    ),
    def!(
        "export.defaultFormat",
        Kind::Enum(&["eml", "mbox", "txt", "md", "msg", "docx", "csv", "json", "pdf"]),
        r#""eml""#,
        GDU,
        "mail",
        "Format the export dialog starts with"
    ),
    def!(
        "export.defaultScope",
        Kind::Enum(&["message", "conversation", "selection", "folder"]),
        r#""message""#,
        GDU,
        "mail",
        "Scope the export dialog starts with, when the context offers it"
    ),
    def!(
        "properties.message.permanentDelete",
        Kind::Enum(&["allow", "confirm-twice", "deny"]),
        r#""allow""#,
        GD,
        "mail",
        "Permanent delete from the message dialog: allow, confirm twice, or deny (a legal hold always denies)"
    ),
    def!(
        "properties.shortcut.enabled",
        Kind::Bool,
        r#"true"#,
        GDU,
        "mail",
        "Keyboard shortcut that opens Properties"
    ),
    def!(
        "properties.shortcut.key",
        text(Text::Plain, 32),
        r#""Alt+Enter""#,
        GDU,
        "mail",
        "Key combination that opens Properties"
    ),
    def!(
        "mail.accountUnreadBadge",
        Kind::Bool,
        "false",
        GDU,
        "mail",
        "Inbox unread count beside the mailbox name in the sidebar head; the mailbox menu always shows per-mailbox counts; account wide"
    ),
    def!(
        "contacts.avatarMode",
        Kind::Enum(&["off", "initials", "contacts"]),
        "\"initials\"",
        GDU,
        "contacts",
        "local generated avatars or embedded photos from this account contacts; never remote photo requests"
    ),
    def!(
        "contacts.senderIcons",
        Kind::Bool,
        "true",
        GDU,
        "contacts",
        "bundled sender-domain icons with generated initials fallback; domain hints are not authentication evidence"
    ),
    def!(
        "mail.brandAvatars",
        Kind::Bool,
        "true",
        GDU,
        "mail",
        "show a bundled brand logo beside the sender (reader and message list) when the From domain or any subdomain of it belongs to a known brand; mail.brandAvatars.requireAuth decides what authentication changes"
    ),
    def!(
        "mail.brandAvatars.requireAuth",
        Kind::Enum(&["never", "known-failure", "always"]),
        "\"known-failure\"",
        GDU,
        "mail",
        "when a sender brand logo needs authentication: never = domain match only; known-failure = hidden only when the cached DKIM or DMARC result is a failure (unchecked mail keeps it); always = needs an aligned DMARC pass with DKIM for the brand's own domain"
    ),
    def!(
        "contacts.favoritesFirst",
        Kind::Bool,
        "true",
        GDU,
        "contacts",
        "recipient suggestions list favourite contacts ahead of other matches; a sort preference only, it never hides a suggestion"
    ),
    def!(
        "contacts.saveFromMessage.enabled",
        Kind::Bool,
        "true",
        GDU,
        "contacts",
        "offer Save contacts from this message in message menus; when off the item is hidden"
    ),
    def!(
        "contacts.saveFromMessage.defaultScope",
        Kind::Enum(&["sender", "sender+replyto", "all"]),
        "\"sender\"",
        GDU,
        "contacts",
        "which participants of a message are ticked first in the Save contacts dialog: the sender, the sender and Reply-To, or everyone"
    ),
    def!(
        "contacts.saveFromMessage.skipAutomated",
        Kind::Bool,
        "true",
        GDU,
        "contacts",
        "leave no-reply, notification and bounce addresses out of Save contacts unless the user ticks them"
    ),
    def!(
        "contacts.saveFromMessage.addNote",
        Kind::Bool,
        "true",
        GDU,
        "contacts",
        "add a note recording the date to contacts saved from a message"
    ),
    def!(
        "contacts.saveFromMessage.defaultBook",
        text(Text::Plain, 128),
        "\"default\"",
        GDU,
        "contacts",
        "address book that receives contacts saved from a message: an address book id, or default for the default book"
    ),
    def!(
        "contacts.saveSender.enabled",
        Kind::Bool,
        "true",
        GDU,
        "contacts",
        "offer the one-click Save sender as contact item in message menus; when off the item is hidden"
    ),
    def!(
        "contacts.saveSender.useReplyTo",
        Kind::Bool,
        "false",
        GDU,
        "contacts",
        "save the Reply-To address instead of From for mailing-list and notification senders"
    ),
    def!(
        "reader.actions.reply",
        Kind::Enum(&["toolbar", "menu", "hidden"]),
        "\"toolbar\"",
        GDU,
        "reader",
        "where the reply reader action appears: toolbar, More menu, or hidden; permissions still apply"
    ),
    def!(
        "reader.actions.reply-all",
        Kind::Enum(&["toolbar", "menu", "hidden"]),
        "\"toolbar\"",
        GDU,
        "reader",
        "where the reply all reader action appears: toolbar, More menu, or hidden; permissions still apply"
    ),
    def!(
        "reader.actions.forward",
        Kind::Enum(&["toolbar", "menu", "hidden"]),
        "\"toolbar\"",
        GDU,
        "reader",
        "where the forward reader action appears: toolbar, More menu, or hidden; permissions still apply"
    ),
    def!(
        "reader.actions.edit-draft",
        Kind::Enum(&["toolbar", "menu", "hidden"]),
        "\"toolbar\"",
        GDU,
        "reader",
        "where the edit draft reader action appears: toolbar, More menu, or hidden; permissions still apply"
    ),
    def!(
        "reader.actions.open-draft",
        Kind::Enum(&["toolbar", "menu", "hidden"]),
        "\"toolbar\"",
        GDU,
        "reader",
        "where the open reply draft reader action appears: toolbar, More menu, or hidden; permissions still apply"
    ),
    def!(
        "reader.actions.delete",
        Kind::Enum(&["toolbar", "menu", "hidden"]),
        "\"toolbar\"",
        GDU,
        "reader",
        "where the delete reader action appears: toolbar, More menu, or hidden; permissions still apply"
    ),
    def!(
        "reader.actions.archive",
        Kind::Enum(&["toolbar", "menu", "hidden"]),
        "\"toolbar\"",
        GDU,
        "reader",
        "where the archive reader action appears: toolbar, More menu, or hidden; permissions still apply"
    ),
    def!(
        "reader.actions.move",
        Kind::Enum(&["toolbar", "menu", "hidden"]),
        "\"toolbar\"",
        GDU,
        "reader",
        "where the move reader action appears: toolbar, More menu, or hidden; permissions still apply"
    ),
    def!(
        "reader.actions.conversation",
        Kind::Enum(&["toolbar", "menu", "hidden"]),
        "\"toolbar\"",
        GDU,
        "reader",
        "where the view conversation reader action appears: toolbar, More menu, or hidden; permissions still apply"
    ),
    def!(
        "reader.actions.links",
        Kind::Enum(&["toolbar", "menu", "hidden"]),
        "\"toolbar\"",
        GDU,
        "reader",
        "where the review links reader action appears: toolbar, More menu, or hidden; permissions still apply"
    ),
    def!(
        "reader.actions.read",
        Kind::Enum(&["toolbar", "menu", "hidden"]),
        "\"toolbar\"",
        GDU,
        "reader",
        "where the read / unread reader action appears: toolbar, More menu, or hidden; permissions still apply"
    ),
    def!(
        "reader.actions.flag",
        Kind::Enum(&["toolbar", "menu", "hidden"]),
        "\"toolbar\"",
        GDU,
        "reader",
        "where the flag / unflag reader action appears: toolbar, More menu, or hidden; permissions still apply"
    ),
    def!(
        "reader.actions.pin",
        Kind::Enum(&["toolbar", "menu", "hidden"]),
        "\"toolbar\"",
        GDU,
        "reader",
        "where the pin / unpin reader action appears: toolbar, More menu, or hidden; permissions still apply"
    ),
    def!(
        "reader.actions.spam",
        Kind::Enum(&["toolbar", "menu", "hidden"]),
        "\"toolbar\"",
        GDU,
        "reader",
        "where the mark as spam reader action appears: toolbar, More menu, or hidden; permissions still apply"
    ),
    def!(
        "reader.actions.security",
        Kind::Enum(&["toolbar", "menu", "hidden"]),
        "\"toolbar\"",
        GDU,
        "reader",
        "where the security mode reader action appears: toolbar, More menu, or hidden; permissions still apply"
    ),
    def!(
        "reader.actions.headers",
        Kind::Enum(&["toolbar", "menu", "hidden"]),
        "\"toolbar\"",
        GDU,
        "reader",
        "where the raw headers reader action appears: toolbar, More menu, or hidden; permissions still apply"
    ),
    def!(
        "reader.actions.delete-sender",
        Kind::Enum(&["toolbar", "menu", "hidden"]), "\"hidden\"", GDU, "reader",
        "where the delete sender reader action appears; hidden by default and requires the administrator capability"
    ),
    def!(
        "reader.actions.finish-followup",
        Kind::Enum(&["toolbar", "menu", "hidden"]), "\"menu\"", GDU, "reader",
        "where the finish follow-up reader action appears: toolbar, More menu, or hidden"
    ),
    def!(
        "reader.actions.sweep",
        Kind::Enum(&["toolbar", "menu", "hidden"]),
        "\"toolbar\"",
        GDU,
        "reader",
        "where the sweep sender reader action appears: toolbar, More menu, or hidden; permissions still apply"
    ),
    def!(
        "reader.actions.print",
        Kind::Enum(&["toolbar", "menu", "hidden"]),
        "\"toolbar\"",
        GDU,
        "reader",
        "where the print reader action appears: toolbar, More menu, or hidden; permissions still apply"
    ),
    def!(
        "reader.actions.export",
        Kind::Enum(&["toolbar", "menu", "hidden"]),
        "\"toolbar\"",
        GDU,
        "reader",
        "where the export reader action appears: toolbar, More menu, or hidden; permissions still apply"
    ),
    def!(
        "reader.actions.properties",
        Kind::Enum(&["toolbar", "menu", "hidden"]),
        "\"toolbar\"",
        GDU,
        "reader",
        "where the message properties reader action appears: toolbar, More menu, or hidden; permissions still apply"
    ),
    def!(
        "reader.actions.zoom",
        Kind::Enum(&["toolbar", "menu", "hidden"]),
        "\"toolbar\"",
        GDU,
        "reader",
        "where the zoom reader action appears: toolbar, More menu, or hidden; permissions still apply"
    ),
    def!(
        "reader.actions.search",
        Kind::Enum(&["toolbar", "menu", "hidden"]),
        "\"toolbar\"",
        GDU,
        "reader",
        "where the related search reader action appears: toolbar, More menu, or hidden; permissions still apply"
    ),
    def!(
        "reader.actions.colors",
        Kind::Enum(&["toolbar", "menu", "hidden"]),
        "\"toolbar\"",
        GDU,
        "reader",
        "where the message colors reader action appears: toolbar, More menu, or hidden; permissions still apply"
    ),
    def!(
        "reader.actions.save-sender",
        Kind::Enum(&["toolbar", "menu", "hidden"]),
        "\"menu\"",
        GDU,
        "reader",
        "where the save sender reader action appears: toolbar, More menu, or hidden; permissions still apply"
    ),
    def!(
        "reader.actions.save-contacts",
        Kind::Enum(&["toolbar", "menu", "hidden"]),
        "\"menu\"",
        GDU,
        "reader",
        "where the save message contacts reader action appears: toolbar, More menu, or hidden; permissions still apply"
    ),
    def!(
        "reader.actions.focus-focused",
        Kind::Enum(&["toolbar", "menu", "hidden"]),
        "\"menu\"",
        GDU,
        "reader",
        "where the move to focused reader action appears: toolbar, More menu, or hidden; permissions still apply"
    ),
    def!(
        "reader.actions.focus-other",
        Kind::Enum(&["toolbar", "menu", "hidden"]),
        "\"menu\"",
        GDU,
        "reader",
        "where the move to other reader action appears: toolbar, More menu, or hidden; permissions still apply"
    ),
    def!(
        "reader.actions.not-spam",
        Kind::Enum(&["toolbar", "menu", "hidden"]),
        "\"menu\"",
        GDU,
        "reader",
        "where the not spam reader action appears: toolbar, More menu, or hidden; permissions still apply"
    ),
    def!(
        "reader.actions.view-source",
        Kind::Enum(&["toolbar", "menu", "hidden"]),
        "\"menu\"",
        GDU,
        "reader",
        "where the view message source reader action appears: toolbar, More menu, or hidden; permissions still apply"
    ),
    def!(
        "reader.actions.create-filter",
        Kind::Enum(&["toolbar", "menu", "hidden"]),
        "\"menu\"",
        GDU,
        "reader",
        "where the create filter reader action appears: toolbar, More menu, or hidden; permissions still apply"
    ),
    def!(
        "reader.actions.block-sender",
        Kind::Enum(&["toolbar", "menu", "hidden"]),
        "\"menu\"",
        GDU,
        "reader",
        "where the block sender reader action appears: toolbar, More menu, or hidden; permissions still apply"
    ),
    def!(
        "reader.actionStyle",
        Kind::Enum(&["icons", "labels"]),
        "\"icons\"",
        GDU,
        "reader",
        "compact reader actions or icons with visible labels"
    ),
    def!(
        "reader.colorMode",
        Kind::Enum(&["auto", "original", "adaptive", "invert"]),
        "\"auto\"",
        GDU,
        "reader",
        "message color adaptation inside the isolated reader"
    ),
    def!(
        "reader.fontSize",
        int(12, 24),
        "14",
        GDU,
        "reader",
        "message reading font size in pixels"
    ),
    def!(
        "reader.lineHeight",
        Kind::Float { min: 1.2, max: 2.2 },
        "1.6",
        GDU,
        "reader",
        "message reading line height"
    ),
    def!(
        "reader.contentWidth",
        Kind::Enum(&["comfortable", "wide", "full"]),
        "\"full\"",
        GDU,
        "reader",
        "message content width"
    ),
    def!(
        "reader.loading",
        Kind::Enum(&["skeleton", "indicator", "none"]),
        "\"skeleton\"",
        GDU,
        "reader",
        "message loading appearance"
    ),
    def!(
        "reader.loadingDelayMs",
        int(0, 1000),
        "120",
        GDU,
        "reader",
        "delay before showing message loading feedback"
    ),
    def!(
        "reader.zoom",
        int(50, 200),
        "100",
        GDU,
        "reader",
        "message body zoom in percent; the reader snaps it to its zoom steps"
    ),
    def!(
        "reader.attachmentsCollapsed",
        Kind::Bool,
        "true",
        GDU,
        "reader",
        "start the reader's attachments bar collapsed (t110)"
    ),
    def!(
        "attachments.bar.clickToToggle",
        Kind::Bool,
        "true",
        GDU,
        "reader",
        "clicking anywhere on the reader's attachments bar header expands or collapses it"
    ),
    def!(
        "attachments.bar.showDownloadButtons",
        Kind::Enum(&["always", "hover", "never"]),
        "\"hover\"",
        GDU,
        "reader",
        "when each attachment shows its download button"
    ),
    def!(
        "attachments.bar.downloadAll",
        Kind::Bool,
        "true",
        GDU,
        "reader",
        "offer a Download all button on the reader's attachments bar"
    ),
    def!(
        "attachments.bar.zipInlineImages",
        Kind::Bool,
        "false",
        GDU,
        "reader",
        "include inline (cid) images in the attachments bar's Download all ZIP"
    ),
    def!(
        "ui.language",
        Kind::Locale,
        "\"browser\"",
        GDU,
        "interface",
        "web client locale (cached in localStorage mw.locale)"
    ),
    def!(
        "ui.notifyNewVersion",
        Kind::Bool,
        "true",
        GDU,
        "interface",
        "show a toast when a newer version of the web app has been deployed"
    ),
    def!(
        "ui.undoHistorySize",
        int(0, 50),
        "50",
        GDU,
        "interface",
        "how many recent mail actions the Undo and Redo buttons remember (0 turns them off)"
    ),
    def!(
        "ui.undoHistoryMax",
        int(0, 50),
        "50",
        G,
        "interface",
        "most recent mail actions any user may keep in the Undo and Redo history"
    ),
    def!(
        "folders.hideEmpty",
        Kind::Bool,
        "false",
        GDU,
        "folders",
        "hide empty folders in the sidebar; per-folder overrides apply unless the setting is locked"
    ),
    def!(
        "folders.reorder.enabled",
        Kind::Bool,
        "true",
        GDU,
        "folders",
        "let users rearrange folders in the sidebar and in Settings by drag and drop"
    ),
    def!(
        "folders.reorder.allowNesting",
        Kind::Bool,
        "true",
        GD,
        "folders",
        "let users move a folder under another folder or back to the top level"
    ),
    def!(
        "folders.reorder.allowServerMove",
        Kind::Bool,
        "true",
        GD,
        "folders",
        "let users change a folders real parent on the mail server when they nest it"
    ),
    def!(
        "folders.reorder.allowSystemFolders",
        Kind::Bool,
        "false",
        GD,
        "folders",
        "let users drag system folders (Inbox, Drafts, Sent, Junk, Trash, Archive) to reorder them"
    ),
    def!(
        "folders.reorder.maxDepth",
        int(1, 16),
        "8",
        GD,
        "folders",
        "deepest level of folder nesting the sidebar allows when users drag folders"
    ),
    def!(
        "folders.order.locked",
        Kind::Bool,
        "false",
        GD,
        "folders",
        "fix the folder order for everyone: drag and drop and the Move commands are off"
    ),
    def!(
        "folders.pinned.admin",
        Kind::StringList {
            max_len: 50,
            max_item: 256
        },
        "[]",
        GD,
        "folders",
        "folders pinned at the top of every users sidebar (role:<role> or a folder path); users cannot unpin or move them"
    ),
    def!(
        "folders.pinned.hideFromTree",
        Kind::Bool,
        "false",
        GD,
        "folders",
        "hide a pinned folder that has no subfolders from the folder tree, showing it only in the Pinned group"
    ),
    def!(
        "folders.order.default",
        Kind::StringList {
            max_len: 100,
            max_item: 256
        },
        "[]",
        GD,
        "folders",
        "the order folders start in for users with no order of their own (role:<role> or a folder path, first to last)"
    ),
    def!(
        "folders.newFolderPlacement",
        Kind::Enum(&["end", "alphabetical"]),
        "\"end\"",
        GDU,
        "folders",
        "where a folder that appears on the server is placed among its siblings"
    ),
    def!(
        "ui.bar.mailTopBar",
        Kind::Enum(&["pinned", "auto"]),
        "\"pinned\"",
        GDU,
        "interface",
        "mail top bar: always shown, or auto-hidden behind a slim handle"
    ),
    def!(
        "ui.bar.workspaceTabs",
        Kind::Enum(&["pinned", "auto"]),
        "\"pinned\"",
        GDU,
        "interface",
        "workspace tab strip: always shown, or auto-hidden behind a slim handle"
    ),
    def!(
        "ui.bar.mailRibbon",
        Kind::Enum(&["pinned", "auto"]),
        "\"pinned\"",
        GDU,
        "interface",
        "mail ribbon (ribbon layout): always shown, or only its tab names"
    ),
    def!(
        "ui.bar.composerRibbon",
        Kind::Enum(&["pinned", "auto"]),
        "\"pinned\"",
        GDU,
        "interface",
        "composer ribbon: always shown, or only its tab names"
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
        "appearance.badgeColor",
        Kind::Enum(&[
            "default",
            "accent",
            "accent-lighter",
            "accent-darker",
            "red",
            "orange",
            "yellow",
            "green",
            "teal",
            "blue",
            "indigo",
            "violet",
            "magenta",
            "graphite",
            "gold",
            "brown",
            "beige",
            "taupe",
            "silver",
            "stone"
        ]),
        "\"default\"",
        GDU,
        "appearance",
        "appearance blob badgeColor: unread-count badge colour (default = the theme's signal colour; accent-lighter/accent-darker = the accent 0.2 OKLCH L lighter/darker)"
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
        Kind::Enum(&[
            "default",
            "system",
            "serif",
            "mono",
            "inter",
            "roboto",
            "open-sans",
            "jost",
            "manrope"
        ]),
        "\"default\"",
        GDU,
        "appearance",
        "self-hosted interface font; account choice or administrator default/lock"
    ),
    def!(
        "appearance.fontSize",
        int(12, 24),
        "14",
        GDU,
        "appearance",
        "interface font size in pixels"
    ),
    def!(
        "appearance.fontWeight",
        Kind::Enum(&["300", "400", "500", "600"]),
        "\"400\"",
        GDU,
        "appearance",
        "interface base font weight; emphasized text keeps its hierarchy"
    ),
    def!(
        "appearance.fontColor",
        text(Text::Plain, 7),
        "\"\"",
        GDU,
        "appearance",
        "interface text color as #RRGGBB; empty follows theme"
    ),
    def!(
        "appearance.lowContrast",
        Kind::Bool,
        "false",
        GDU,
        "appearance",
        "soften interface text contrast"
    ),
    def!(
        "mail.messageListStyle",
        Kind::Enum(&["cards", "table"]),
        "\"cards\"",
        GDU,
        "mail",
        "message list cards or Gmail-style rows with column headings"
    ),
    def!(
        "mail.messageListOrder",
        Kind::Enum(&["newest", "oldest", "sender", "sender-desc", "subject", "subject-desc", "sent-newest", "sent-oldest", "importance", "importance-asc", "size", "size-asc"]),
        "\"newest\"",
        GDU,
        "mail",
        "message list ordering"
    ),
    def!(
        "mail.conversations.allowed",
        Kind::Bool,
        "true",
        GD,
        "mail",
        "Make conversation threading available; users still opt in separately"
    ),
    def!(
        "mail.conversations.enabled",
        Kind::Bool,
        "false",
        GDU,
        "mail",
        "Group related replies into expandable conversations independently of category grouping"
    ),
    def!(
        "mail.messageListGroup",
        Kind::Enum(&["none", "received", "sent", "sender", "subject", "importance", "size"]),
        "\"none\"",
        GDU,
        "mail",
        "message list grouping"
    ),
    def!(
        "mail.list.pageSize",
        int(20, 200),
        "50",
        GDU,
        "mail",
        "messages fetched per page when the message list is scrolled"
    ),
    def!(
        "mail.search.defaultScope",
        Kind::Enum(&["current", "all"]),
        r#""current""#,
        GDU,
        "mail",
        "Default scope for a new normal mail search: current folder or all folders; explicit search scope choices remain available"
    ),
    def!(
        "mail.messageListPreview",
        Kind::Bool,
        "true",
        GDU,
        "mail",
        "show body snippets in message list"
    ),
    def!(
        "mail.afterDelete",
        Kind::Enum(&["next", "previous", "close"]),
        "\"next\"",
        GDU,
        "mail",
        "message shown after deleting the open message"
    ),
    def!(
        "reader.attachmentsAtEnd",
        Kind::Bool,
        "false",
        GDU,
        "reader",
        "show attachments after the message body"
    ),
    def!(
        "appearance.leetspeak",
        Kind::Bool,
        "false",
        GDU,
        "appearance",
        "optional leetspeak presentation for interface and message reading; stored text and submitted values stay unchanged"
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
    def!(
        "login.rememberSignins",
        Kind::Bool,
        "true",
        G,
        "appearance",
        "allow this browser to remember successful sign-in addresses; never credentials"
    ),
    def!(
        "login.showRememberedHint",
        Kind::Bool,
        "false",
        G,
        "appearance",
        "show explanatory text for remembered sign-in addresses"
    ),
    def!(
        "login.showAccountHelpNotice",
        Kind::Bool,
        "false",
        GD,
        "appearance",
        "show optional account-help notice after a refused sign-in"
    ),
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
        Kind::Enum(&[
            "ask",
            "never",
            "always",
            "contactsOnly",
            "orgOnly",
            "orgAlwaysAskOthers"
        ]),
        "\"ask\"",
        GDU,
        "mail",
        "mdn_policy:<account> (MDN/send)"
    ),
    def!(
        "mail.openTracking",
        Kind::Bool,
        "false",
        GDU,
        "mail",
        "Default outgoing open tracking for new messages; a locked false value also forbids token minting and recording"
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
        "notifications.askPermission",
        Kind::Bool,
        "true",
        GDU,
        "notifications",
        "offer the browser notification permission request to the user"
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
    // t39-e11 (N8): which AI provider profile serves a user. Empty = the
    // deployment endpoint of PUT /admin/assist. Per-user values are the admin's.
    def!(
        "assist.profile",
        text(Text::Slug, 64),
        "\"\"",
        GDA,
        "assistant",
        "AI provider profile id; empty = the deployment endpoint (assist_profiles.rs)"
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
    def!(
        "signin.antiBotMode",
        Kind::Enum(&["off", "mailbox", "admin", "all"]),
        "\"off\"",
        G,
        "signin",
        "initial sign-in proof-of-work policy"
    ),
    def!(
        "signin.antiBotDifficulty",
        int(100, 20000),
        "2000",
        G,
        "signin",
        "maximum proof-of-work solver counter"
    ),
    def!(
        "signin.antiBotExpirySecs",
        int(60, 600),
        "180",
        G,
        "signin",
        "single-use challenge lifetime"
    ),
    def!(
        "security.spray.enabled",
        Kind::Bool,
        "true",
        G,
        "security",
        "Count the distinct usernames one source fails with (signin_spray.rs). Only a keyed hash of each username is kept, in memory."
    ),
    def!(
        "security.spray.windowSecs",
        int(60, 86400),
        "900",
        G,
        "security",
        "Window in which one source's failed sign-ins are counted by distinct username."
    ),
    def!(
        "security.spray.challengeAt",
        int(2, 1000),
        "10",
        G,
        "security",
        "Distinct failing usernames from one source inside the window that force the proof-of-work challenge for that source. A shared office or university address is not affected by successful sign-ins; only failures count."
    ),
    def!(
        "security.spray.paceAt",
        int(2, 1000),
        "15",
        G,
        "security",
        "Distinct failing usernames that add extra pacing between attempts from the source."
    ),
    def!(
        "security.spray.lockAt",
        int(2, 1000),
        "25",
        G,
        "security",
        "Distinct failing usernames that lock the source for security.spray.lockSecs through the existing sign-in lock."
    ),
    def!(
        "security.spray.paceSecs",
        int(1, 300),
        "10",
        G,
        "security",
        "Seconds between attempts from a source at or above the pacing step."
    ),
    def!(
        "security.spray.lockSecs",
        int(30, 86400),
        "900",
        G,
        "security",
        "Length of the temporary lock."
    ),
    def!(
        "security.spray.knownMultiplier",
        int(1, 20),
        "3",
        G,
        "security",
        "Multiplier on the three thresholds for a known source (a recent successful sign-in from the same address)."
    ),
    def!(
        "security.spray.capacity",
        int(100, 200000),
        "20000",
        G,
        "security",
        "Most sources tracked in memory; the least recently active are dropped first."
    ),
    def!(
        "security.spike.enabled",
        Kind::Bool,
        "true",
        G,
        "security",
        "Learn a baseline of failed sign-ins and enter heightened mode when the whole instance is far above it (signin_spike.rs). Never locks the instance."
    ),
    def!(
        "security.spike.multiplierPercent",
        int(150, 2000),
        "400",
        G,
        "security",
        "A signal counts when it is at least this percentage of its learned baseline (400 = four times)."
    ),
    def!(
        "security.spike.minFailuresPerMin",
        int(1, 10000),
        "20",
        G,
        "security",
        "Floor: failed sign-ins per minute below which the failure-rate signal never counts."
    ),
    def!(
        "security.spike.minSources",
        int(2, 10000),
        "10",
        G,
        "security",
        "Floor: distinct failing sources in a five-minute bucket below which that signal never counts."
    ),
    def!(
        "security.spike.minUsernames",
        int(2, 10000),
        "30",
        G,
        "security",
        "Floor: distinct failing usernames in a five-minute bucket below which that signal never counts."
    ),
    def!(
        "security.spike.signalsRequired",
        int(1, 4),
        "2",
        G,
        "security",
        "Signals that must be over their baseline and floor together. The distributed-spray signature counts as two."
    ),
    def!(
        "security.spike.warmupHours",
        int(1, 720),
        "48",
        G,
        "security",
        "Hours of observed traffic before heightened mode can start. Until then the baseline only learns."
    ),
    def!(
        "security.spike.adaptHours",
        int(6, 2160),
        "168",
        G,
        "security",
        "Time constant of the baseline. Longer absorbs growth more slowly. A spike never updates the baseline."
    ),
    def!(
        "security.spike.durationSecs",
        int(60, 86400),
        "1800",
        G,
        "security",
        "How long heightened mode lasts after the last evidence of a spike."
    ),
    def!(
        "security.spike.forceChallenge",
        Kind::Bool,
        "true",
        G,
        "security",
        "In heightened mode, require the proof-of-work challenge for anonymous sign-ins from unknown sources."
    ),
    def!(
        "security.spike.paceFactorPercent",
        int(100, 1000),
        "300",
        G,
        "security",
        "In heightened mode, the sign-in wait for unknown sources as a percentage of the usual wait."
    ),
    def!(
        "security.sessionBinding.enabled",
        Kind::Bool,
        "true",
        G,
        "security",
        "Bind each browser session to a keyed hash of its client attributes (session_binding.rs). Native clients with a bearer token, API keys and protocol clients are exempt."
    ),
    def!(
        "security.sessionBinding.attributes",
        Kind::StringList {
            max_len: 8,
            max_item: 16
        },
        "[\"browser\",\"os\",\"hints\",\"country\"]",
        G,
        "security",
        "Attributes bound: browser, os, hints, language, country, network (IPv4 /16, IPv6 /48). Unknown names are ignored."
    ),
    def!(
        "security.sessionBinding.strictness",
        Kind::Enum(&["tolerant", "strict"]),
        "\"tolerant\"",
        G,
        "security",
        "Tolerant accepts browser updates and one soft attribute change; strict needs every bound attribute to match."
    ),
    def!(
        "security.sessionBinding.webmailAction",
        Kind::Enum(&["off", "detect", "step-up", "revoke"]),
        "\"step-up\"",
        G,
        "security",
        "What a webmail session mismatch does: nothing, record only, require re-authentication, or revoke the session."
    ),
    def!(
        "security.sessionBinding.adminAction",
        Kind::Enum(&["off", "detect", "step-up", "revoke"]),
        "\"revoke\"",
        G,
        "security",
        "What an admin console session mismatch does."
    ),
    def!(
        "security.theft.enabled",
        Kind::Bool,
        "true",
        G,
        "security",
        "Session theft detection: rotated-session reuse, impossible travel and concurrent use from different countries."
    ),
    def!(
        "security.theft.rotationGraceSecs",
        int(0, 600),
        "30",
        G,
        "security",
        "After a session rotates, the old id may be presented again for this long (an in-flight request) before it counts as theft."
    ),
    def!(
        "security.theft.minTravelSecs",
        int(60, 172800),
        "3600",
        G,
        "security",
        "A country change sooner than this after the previous request is impossible travel."
    ),
    def!(
        "security.theft.concurrentUse",
        Kind::Enum(&["off", "country", "network"]),
        "\"country\"",
        G,
        "security",
        "Flag one session used alternately from different countries (or networks) within five minutes."
    ),
    def!(
        "security.theft.webmailAction",
        Kind::Enum(&["alert", "step-up", "revoke"]),
        "\"step-up\"",
        G,
        "security",
        "Response to impossible travel or concurrent use on a webmail session. Rotated-session reuse always revokes."
    ),
    def!(
        "security.theft.adminAction",
        Kind::Enum(&["alert", "step-up", "revoke"]),
        "\"revoke\"",
        G,
        "security",
        "Response to impossible travel or concurrent use on an admin console session."
    ),
    def!(
        "security.theft.retentionDays",
        int(1, 60),
        "15",
        G,
        "security",
        "How long rotated-session markers and per-session security state are kept; at least the longest session lifetime."
    ),
    def!(
        "security.alerts.countryChange",
        Kind::Bool,
        "true",
        GDU,
        "security",
        "Warn when a signed-in session is used from a country it was not started in. Country detection may be inaccurate; VPNs and mobile networks can cause false changes."
    ),
    def!(
        "security.alerts.countryChangeEnabled",
        Kind::Bool,
        "true",
        G,
        "security",
        "Global switch for the country-change alert. Off disables it for everyone."
    ),
    def!(
        "security.alerts.countryChangeCooldownSecs",
        int(0, 86400),
        "300",
        G,
        "security",
        "Minimum time between country-change alerts of one session. Each new country alerts once per session."
    ),
    def!(
        "security.alerts.countryChangeUserMute",
        Kind::Bool,
        "false",
        G,
        "security",
        "Whether a user may turn their own country-change alert off."
    ),
    def!(
        "security.alerts.channels",
        Kind::Enum(&["app", "email", "both"]),
        "\"both\"",
        GDU,
        "security",
        "How session security warnings reach the user: in the app, as a message in their own mailbox, or both."
    ),
    def!(
        "security.alerts.emailEnabled",
        Kind::Bool,
        "true",
        G,
        "security",
        "Allow session security warnings to be delivered to the user's own mailbox."
    ),
    def!(
        "security.alerts.sensitivity",
        Kind::Enum(&["low", "normal", "high"]),
        "\"normal\"",
        GDU,
        "security",
        "Low only warns on travel and mismatch; normal follows the administrator's actions; high doubles the travel interval and also checks concurrent networks."
    ),
    def!(
        "security.alerts.noticeRetentionDays",
        int(1, 365),
        "30",
        G,
        "security",
        "How long in-app security notices are kept per account (at most 20 per account)."
    ),
    def!(
        "security.protection.rateMode",
        Kind::Enum(&["off", "observe", "enforce"]),
        "\"off\"",
        G,
        "security",
        "Off by default. Observe records budget exhaustion without refusal; enforce returns a temporary 429. Existing protocol concurrency limits remain independent."
    ),
    def!(
        "security.protection.adaptiveMode",
        Kind::Enum(&["off", "observe", "enforce"]),
        "\"off\"",
        G,
        "security",
        "Off by default. Observe records explainable typed anomalies and proposed actions; enforce applies bounded temporary throttles or blocks. This is a deterministic counter policy, not an AI classifier."
    ),
    def!(
        "security.protection.clientRequests",
        int(0, 1000000),
        "600",
        G,
        "security",
        "Requests per window from one source address that is not signed in, and every sign-in or identity-confirmation request, across replicas. A request with a valid mailbox or admin session uses its own allowance instead. Zero disables this dimension; the source address comes only from the trusted proxy resolver."
    ),
    def!(
        "security.protection.accountRequests",
        int(0, 1000000),
        "1800",
        G,
        "security",
        "Requests per window for one signed-in mailbox, across its sessions, devices and addresses and across replicas. Kept above the anonymous allowance so signed-in use has more room than an unknown address. Zero disables this dimension."
    ),
    def!(
        "security.protection.adminRequests",
        int(0, 1000000),
        "1200",
        G,
        "security",
        "Requests per window for one signed-in administrator, across replicas. Kept above the anonymous allowance. Zero disables this dimension."
    ),
    def!(
        "security.protection.rateWindowSecs",
        int(1, 3600),
        "60",
        G,
        "security",
        "Fixed request-budget interval. Capacity load shedding and ordinary not-found responses are not adaptive evidence."
    ),
    def!(
        "security.protection.adaptiveClients",
        Kind::Bool,
        "true",
        G,
        "security",
        "Enable this subject dimension when adaptive mode is active. Accounts and administrators are counted only after their identity is verified."
    ),
    def!(
        "security.protection.adaptiveAccounts",
        Kind::Bool,
        "true",
        G,
        "security",
        "Enable this subject dimension when adaptive mode is active. Accounts and administrators are counted only after their identity is verified."
    ),
    def!(
        "security.protection.adaptiveAdmins",
        Kind::Bool,
        "true",
        G,
        "security",
        "Enable this subject dimension when adaptive mode is active. Accounts and administrators are counted only after their identity is verified."
    ),
    def!(
        "security.protection.shortWindowSecs",
        int(60, 3600),
        "300",
        G,
        "security",
        "Rolling observation window. Long windows support hours through thirty days; bucket rounding can retain at most one extra sixty-fourth of a window."
    ),
    def!(
        "security.protection.shortMinimumSamples",
        int(1, 1000000),
        "30",
        G,
        "security",
        "Minimum observed normal plus anomalous requests before the ratio trigger applies. The absolute anomaly threshold is independent of this minimum."
    ),
    def!(
        "security.protection.shortAbsoluteAnomalies",
        int(0, 1000000),
        "20",
        G,
        "security",
        "Trigger when this many selected anomalies accumulate, even when normal requests dilute the ratio. Zero disables this trigger."
    ),
    def!(
        "security.protection.shortRatioPercent",
        int(0, 100),
        "50",
        G,
        "security",
        "Trigger when selected anomalies reach this percentage after the minimum sample count. Either the absolute or ratio trigger suffices. Zero disables this trigger."
    ),
    def!(
        "security.protection.longWindowSecs",
        int(3600, 2592000),
        "604800",
        G,
        "security",
        "Rolling observation window. Long windows support hours through thirty days; bucket rounding can retain at most one extra sixty-fourth of a window."
    ),
    def!(
        "security.protection.longMinimumSamples",
        int(1, 1000000),
        "100",
        G,
        "security",
        "Minimum observed normal plus anomalous requests before the ratio trigger applies. The absolute anomaly threshold is independent of this minimum."
    ),
    def!(
        "security.protection.longAbsoluteAnomalies",
        int(0, 1000000),
        "100",
        G,
        "security",
        "Trigger when this many selected anomalies accumulate, even when normal requests dilute the ratio. Zero disables this trigger."
    ),
    def!(
        "security.protection.longRatioPercent",
        int(0, 100),
        "25",
        G,
        "security",
        "Trigger when selected anomalies reach this percentage after the minimum sample count. Either the absolute or ratio trigger suffices. Zero disables this trigger."
    ),
    def!(
        "security.protection.normalCapPerBucket",
        int(1, 1000000),
        "1000",
        G,
        "security",
        "Bound the normal baseline learned in each rolling-window bucket. Independent absolute anomaly budgets prevent unlimited valid-request padding from hiding abuse."
    ),
    def!(
        "security.protection.signals",
        Kind::StringList {
            max_len: 6,
            max_item: 32
        },
        "[\"malformed\",\"oversized\",\"credential-rejected\"]",
        G,
        "security",
        "Select authoritative malformed, oversized, credential-rejected, permission-denied, body-deadline or opt-in admin-route-miss observations. Admin route probing excludes entity-not-found responses and static assets; generic status codes, capacity refusals and user-agent strings do not establish abuse."
    ),
    def!(
        "security.protection.throttleSecs",
        int(1, 300),
        "3",
        G,
        "security",
        "First threshold crossing starts a fixed throttle deadline. Requests during that deadline do not extend it."
    ),
    def!(
        "security.protection.blockSecs",
        int(1, 86400),
        "300",
        G,
        "security",
        "A later threshold crossing before recovery may escalate to this bounded block. No permanent account lock is created."
    ),
    def!(
        "security.protection.decaySecs",
        int(3600, 2592000),
        "86400",
        G,
        "security",
        "After this anomaly-free interval and expiry of the current action, escalation returns to its initial stage. Rolling evidence expires according to its configured windows."
    ),
    def!(
        "security.protection.exemptClients",
        Kind::StringList {
            max_len: 100,
            max_item: 64
        },
        "[]",
        G,
        "security",
        "Exact IP addresses or CIDR networks exempt only the client dimension. Verified account limits still apply. Use deliberately for shared NAT networks; forwarded addresses are trusted only through configured proxies."
    ),
    def!(
        "security.protection.exemptAccounts",
        Kind::StringList {
            max_len: 100,
            max_item: 512
        },
        "[]",
        G,
        "security",
        "Exact server-verified mailbox account IDs exempt only their account dimension, never the source-client dimension. Email addresses and caller-supplied account parameters are not identity evidence."
    ),
    def!(
        "security.protection.exemptAdmins",
        Kind::StringList {
            max_len: 100,
            max_item: 512
        },
        "[]",
        G,
        "security",
        "Exact authenticated administrator IDs exempt only their administrator dimension. Existing sign-in failure locks and strict administrator session deadlines remain unchanged."
    ),
    def!(
        "security.protection.storageFailure",
        Kind::Enum(&["closed", "open"]),
        "\"closed\"",
        G,
        "security",
        "Closed refuses protected work when an enabled enforcement budget cannot be checked. Open temporarily permits it with a bounded operational warning. Observe-only failures never impose an enforcement action."
    ),
    def!(
        "security.protection.userAgentMode",
        Kind::Enum(&["off", "observe", "enforce"]),
        "\"enforce\"",
        G,
        "security",
        "Independent protection for self-claimed client classes, enabled by default. Selected bot classes are blocked immediately when userAgentBlockBots is enabled. Missing and uncommon headers use request budgets; claims never authenticate a crawler."
    ),
    def!(
        "security.protection.userAgentBlockBots",
        Kind::Bool,
        "true",
        G,
        "security",
        "Block selected bot, crawler, automation and custom-pattern user-agent claims on their first request. Enabled by default; observe mode records the proposed block, off disables it, and configured scope and exceptions still apply. Missing and uncommon headers retain their request budgets."
    ),
    def!(
        "security.protection.userAgentCategories",
        Kind::StringList {
            max_len: 7,
            max_item: 32
        },
        "[\"missing\",\"uncommon\",\"automation\",\"search\",\"ai-training\",\"ai-search\",\"ai-user\"]",
        G,
        "security",
        "Enabled header-claim categories. Known browser and mail-client families have no extra user-agent budget; these are not verified crawler identities."
    ),
    def!(
        "security.protection.userAgentWeakRequests",
        int(1, 1000000),
        "600",
        G,
        "security",
        "Per-source request allowance for missing and uncommon headers, and for automation/custom-pattern claims when immediate bot blocking is disabled."
    ),
    def!(
        "security.protection.userAgentCrawlerRequests",
        int(1, 1000000),
        "300",
        G,
        "security",
        "Per-source request allowance for claimed search, AI training, AI search and user-triggered AI fetchers when immediate bot blocking is disabled."
    ),
    def!(
        "security.protection.userAgentWindowSecs",
        int(1, 3600),
        "60",
        G,
        "security",
        "User-agent request-budget interval; independent of the legacy rate and adaptive mode switches."
    ),
    def!(
        "security.protection.userAgentMaxAction",
        Kind::Enum(&["throttle", "block"]),
        "\"throttle\"",
        G,
        "security",
        "Maximum escalation for repeated weak user-agent anomalies. Immediate bot blocking is independent of this cap. Request-rate exhaustion always uses a temporary 429."
    ),
    def!(
        "security.protection.userAgentScope",
        Kind::Enum(&["anonymous", "all"]),
        "\"anonymous\"",
        G,
        "security",
        "By default only anonymous and sign-in requests receive the extra budget. All includes verified sessions; operator recovery remains available."
    ),
    def!(
        "security.protection.userAgentAdaptive",
        Kind::Bool,
        "true",
        G,
        "security",
        "Combine the extra budget with selected authoritative signals and rolling-window thresholds. Immediate bot blocking is controlled independently by userAgentBlockBots."
    ),
    def!(
        "security.protection.userAgentPatterns",
        Kind::StringList {
            max_len: 32,
            max_item: 256
        },
        "[]",
        G,
        "security",
        "Additional case-insensitive Rust regular expressions for custom claims, including immediate blocking when userAgentBlockBots is enabled. Compiled with bounded size; no backreferences or lookaround."
    ),
    def!(
        "security.protection.userAgentExceptions",
        Kind::StringList {
            max_len: 32,
            max_item: 256
        },
        "[]",
        G,
        "security",
        "Case-insensitive regular expressions exempting only the new user-agent dimension, never existing rate, adaptive or sign-in controls."
    ),
    // ── HTTP protocol anomaly guard (protocol_guard.rs) ──────────────────────
    def!(
        "security.protocol.framingMode",
        Kind::Enum(&["off", "detect", "enforce"]),
        "\"enforce\"",
        G,
        "security",
        "Request-smuggling class: Content-Length together with Transfer-Encoding, Transfer-Encoding other than exactly chunked (or on HTTP/1.0 and HTTP/2), non-numeric or conflicting Content-Length. Enforce answers 400 and closes the connection; detect only counts. A route-prefix exemption never applies to this class."
    ),
    def!(
        "security.protocol.hostMode",
        Kind::Enum(&["off", "detect", "enforce"]),
        "\"enforce\"",
        G,
        "security",
        "Host header syntax: missing on HTTP/1.1, empty, repeated, or holding userinfo, whitespace or path characters. IPv6 literals, ports and a trailing dot are accepted. Enforce answers 400 and closes the connection."
    ),
    def!(
        "security.protocol.hostPinningMode",
        Kind::Enum(&["off", "detect", "enforce"]),
        "\"detect\"",
        G,
        "security",
        "Host not matching the host of MW_PUBLIC_URL or one of allowedHosts. Does nothing while neither names a host. Detect by default because proxies and health checks may send an internal name."
    ),
    def!(
        "security.protocol.pathControlMode",
        Kind::Enum(&["off", "detect", "enforce"]),
        "\"enforce\"",
        G,
        "security",
        "NUL, control characters or encoded CR/LF (%00, %0d, %0a) in the request path. Enforce answers 400. Never exempted by a route prefix."
    ),
    def!(
        "security.protocol.targetMode",
        Kind::Enum(&["off", "detect", "enforce"]),
        "\"detect\"",
        G,
        "security",
        "Double or overlong percent-encoding of dots and slashes, encoded dots, dot segments, empty path segments (//), backslashes, raw non-ASCII bytes, and an absolute-form target naming another authority than Host."
    ),
    def!(
        "security.protocol.pathLengthMode",
        Kind::Enum(&["off", "detect", "enforce"]),
        "\"detect\"",
        G,
        "security",
        "Path or query longer than maxPathBytes or maxQueryBytes. Enforce answers 414."
    ),
    def!(
        "security.protocol.methodMode",
        Kind::Enum(&["off", "detect", "enforce"]),
        "\"detect\"",
        G,
        "security",
        "Method outside allowedMethods (TRACE, TRACK, CONNECT and unknown or lower-case methods by default). Enforce answers 405."
    ),
    def!(
        "security.protocol.duplicateHeadersMode",
        Kind::Enum(&["off", "detect", "enforce"]),
        "\"detect\"",
        G,
        "security",
        "A header that should appear once appearing more than once (Content-Type, Authorization, Origin, Referer, User-Agent, Sec-Fetch-*, conditional and range headers), a repeated Content-Length with equal values, or repeated Cookie headers on HTTP/1.x."
    ),
    def!(
        "security.protocol.forwardedMode",
        Kind::Enum(&["off", "detect", "enforce"]),
        "\"detect\"",
        G,
        "security",
        "Forwarded, X-Forwarded-* or X-Real-IP sent by a peer outside MW_TRUSTED_PROXIES. The headers are never trusted either way; this records the attempt. It counts toward adaptive protection only when trusted proxies are configured."
    ),
    def!(
        "security.protocol.headerHygieneMode",
        Kind::Enum(&["off", "detect", "enforce"]),
        "\"detect\"",
        G,
        "security",
        "Header count, a single value, or the Cookie header over the configured caps; non-ASCII header bytes; Upgrade or Connection: upgrade outside the WebSocket route; Connection naming headers such as Content-Length or Host. Enforce answers 431 for size and count findings and 400 for the rest."
    ),
    def!(
        "security.protocol.bodyMode",
        Kind::Enum(&["off", "detect", "enforce"]),
        "\"detect\"",
        G,
        "security",
        "GET or HEAD carrying a body, Content-Length above maxBodyBytes (413), Expect other than 100-continue (417) or without a body."
    ),
    def!(
        "security.protocol.maxPathBytes",
        int(256, 65536),
        "4096",
        G,
        "security",
        "Longest request path before the path-length class reports it."
    ),
    def!(
        "security.protocol.maxQueryBytes",
        int(256, 262144),
        "8192",
        G,
        "security",
        "Longest query string before the path-length class reports it."
    ),
    def!(
        "security.protocol.maxHeaderCount",
        int(16, 256),
        "64",
        G,
        "security",
        "Most header fields in one request before the header-hygiene class reports it. The listener's own hard limit (MW_HTTP_MAX_HEADERS) still applies."
    ),
    def!(
        "security.protocol.maxHeaderValueBytes",
        int(256, 65536),
        "8192",
        G,
        "security",
        "Longest single header value before the header-hygiene class reports it."
    ),
    def!(
        "security.protocol.maxCookieBytes",
        int(512, 65536),
        "16384",
        G,
        "security",
        "Combined size of Cookie headers before the header-hygiene class reports it."
    ),
    def!(
        "security.protocol.maxBodyBytes",
        int(1048576, 1099511627776),
        "134217728",
        G,
        "security",
        "Largest Content-Length the body class accepts without reporting it. Keep it above the largest upload or import a route accepts; a route-prefix exemption can raise it for one route."
    ),
    def!(
        "security.protocol.allowedMethods",
        Kind::StringList {
            max_len: 32,
            max_item: 16
        },
        "[\"GET\",\"HEAD\",\"POST\",\"PUT\",\"PATCH\",\"DELETE\",\"OPTIONS\",\"PROPFIND\",\"PROPPATCH\",\"REPORT\",\"MKCOL\",\"MKCALENDAR\",\"COPY\",\"MOVE\",\"LOCK\",\"UNLOCK\"]",
        G,
        "security",
        "Methods the method class accepts. The default lists the ordinary HTTP verbs and the WebDAV, CalDAV and CardDAV verbs; the server mounts no inbound DAV routes today, so those reach a 405 at the router."
    ),
    def!(
        "security.protocol.allowedHosts",
        Kind::StringList {
            max_len: 50,
            max_item: 255
        },
        "[]",
        G,
        "security",
        "Host names accepted by the host-pinning class in addition to the host of MW_PUBLIC_URL, for example an internal name a proxy or health check uses. Ports and a trailing dot are ignored."
    ),
    def!(
        "security.protocol.exemptPrefixes",
        Kind::StringList {
            max_len: 20,
            max_item: 200
        },
        "[]",
        G,
        "security",
        "Route prefixes (each starting with / and longer than one character) for which the heuristic classes do not run, for a legitimate client that needs it, for example a large upload route. Matches whole path segments. Framing and path-control findings are never exempted, and / is ignored."
    ),
    // ── Health endpoint access (health_access.rs) ────────────────────────────
    def!(
        "security.health.mode",
        Kind::Enum(&["internal-only", "allowlist", "open"]),
        "\"internal-only\"",
        G,
        "security",
        "Who may reach /healthz and /readyz. internal-only: loopback, private, link-local and unique-local client addresses. allowlist: those plus the networks below. open: anyone. Anyone else gets the same 404 as an unknown path. A request with the MW_HEALTH_TOKEN token is always admitted. The client address is the trusted-proxy-resolved one: behind a reverse proxy without MW_FORWARDED_MODE and MW_TRUSTED_PROXIES every request looks like the proxy's private address, so the whole internet is admitted."
    ),
    def!(
        "security.health.networks",
        Kind::StringList {
            max_len: 50,
            max_item: 64
        },
        "[]",
        G,
        "security",
        "IP addresses or CIDR networks that may reach /healthz and /readyz in allowlist mode, in addition to the internal ranges, for example an external uptime monitor."
    ),
    // ── Admin-probe protection (t100, admin_probe.rs) ────────────────────────
    dedicated(
        pin(
            def!(
                "security.adminProbe.enabled",
                Kind::Bool,
                "true",
                G,
                "security",
                "Count requests that look for the admin console per client network (IPv4 address, IPv6 /64) and slow, then block, a source that keeps going. The grace allowance absorbs typos and stale bookmarks."
            ),
            "MW_ADMIN_PROBE_ENABLED",
        ),
        ADMIN_PROBE_ROUTE,
    ),
    dedicated(
        def!(
            "security.adminProbe.graceAllowance",
            int(0, 1000),
            "10",
            G,
            "security",
            "Misses per window that are ignored. The score drains continuously at this many points per window."
        ),
        ADMIN_PROBE_ROUTE,
    ),
    dedicated(
        def!(
            "security.adminProbe.windowSecs",
            int(60, 86400),
            "600",
            G,
            "security",
            "The window the grace allowance applies to."
        ),
        ADMIN_PROBE_ROUTE,
    ),
    dedicated(
        def!(
            "security.adminProbe.blockAfter",
            int(1, 1000),
            "10",
            G,
            "security",
            "Misses beyond the grace allowance that start a block. Misses in between are answered after the tarpit delay."
        ),
        ADMIN_PROBE_ROUTE,
    ),
    dedicated(
        def!(
            "security.adminProbe.tarpitDelayMs",
            int(0, 10000),
            "1500",
            G,
            "security",
            "Delay added to each counted miss above the grace allowance. Zero turns the tarpit off. Requests that are not misses are never delayed."
        ),
        ADMIN_PROBE_ROUTE,
    ),
    dedicated(
        def!(
            "security.adminProbe.blockSecs",
            int(60, 604800),
            "900",
            G,
            "security",
            "Length of the first block. Every request from a blocked source gets the same answer, whatever its path."
        ),
        ADMIN_PROBE_ROUTE,
    ),
    dedicated(
        def!(
            "security.adminProbe.maxBlockSecs",
            int(60, 2592000),
            "86400",
            G,
            "security",
            "A repeat block doubles the previous length up to this cap."
        ),
        ADMIN_PROBE_ROUTE,
    ),
    dedicated(
        def!(
            "security.adminProbe.repeatWindowSecs",
            int(0, 2592000),
            "86400",
            G,
            "security",
            "A block that starts within this long after the previous one ended counts as a repeat. After a quiet period this long, the next block is a first block again."
        ),
        ADMIN_PROBE_ROUTE,
    ),
    dedicated(
        def!(
            "security.adminProbe.blockStatus",
            Kind::Enum(&["429", "403"]),
            "\"429\"",
            G,
            "security",
            "HTTP status of the blocked answer, sent with Retry-After."
        ),
        ADMIN_PROBE_ROUTE,
    ),
    dedicated(
        def!(
            "security.adminProbe.signals",
            Kind::StringList {
                max_len: 4,
                max_item: 32
            },
            "[\"probe-path\",\"slug-guess\",\"admin-signin-failed\",\"admin-unauthenticated\"]",
            G,
            "security",
            "Which misses count: probe-path (a path on the probe list), slug-guess (a 404 at the console's path depth), admin-signin-failed (a refused admin sign-in) and admin-unauthenticated (an admin API call without a valid admin session)."
        ),
        ADMIN_PROBE_ROUTE,
    ),
    dedicated(
        def!(
            "security.adminProbe.probePaths",
            Kind::StringList {
                max_len: 100,
                max_item: 128
            },
            "[\"/admin\",\"/administrator\",\"/wp-admin\",\"/wp-login.php\",\"/xmlrpc.php\",\"/phpmyadmin\",\"/pma\",\"/myadmin\",\"/adminer.php\",\"/.env\",\"/.git\",\"/manager/html\",\"/admin.php\",\"/cpanel\",\"/webadmin\",\"/server-status\",\"/actuator\",\"/vendor/phpunit\",\"/cgi-bin\",\"/boaform\"]",
            G,
            "security",
            "Paths scanners ask for. A request whose path contains one of these as whole segments (any case) counts as a miss; the console's own path never does."
        ),
        ADMIN_PROBE_ROUTE,
    ),
    dedicated(
        def!(
            "security.adminProbe.allowlist",
            Kind::StringList {
                max_len: 100,
                max_item: 64
            },
            "[]",
            G,
            "security",
            "IP addresses or CIDR networks (office, VPN) that are never counted or blocked. Forwarded addresses are trusted only through configured proxies."
        ),
        ADMIN_PROBE_ROUTE,
    ),
    dedicated(
        def!(
            "security.adminProbe.auditIp",
            Kind::Enum(&["masked", "full"]),
            "\"masked\"",
            G,
            "security",
            "How a blocked source appears in the audit log and the Security centre: masked (IPv4 /24, IPv6 /48) or full. The fail2ban line always carries the full address."
        ),
        ADMIN_PROBE_ROUTE,
    ),
    dedicated(
        def!(
            "security.adminProbe.fail2banLog",
            Kind::Bool,
            "true",
            G,
            "security",
            "Write one fail2ban-readable line (mailwoman[admin-probe]: source blocked; ... rhost=<address>) when a source is blocked."
        ),
        ADMIN_PROBE_ROUTE,
    ),
    def!(
        "security.trust.enabled",
        Kind::Bool,
        "true",
        G,
        "security",
        "Derive a trust tier for each request: authenticated, known source (an address, IPv6 per /64, with a successful sign-in inside the window) or unknown source. When off, every anonymous request is treated as unknown with the baseline limits."
    ),
    def!(
        "security.trust.knownTtlDays",
        int(1, 365),
        "30",
        G,
        "security",
        "How long a successful sign-in keeps its source address known. Only successful sign-ins count; failed attempts and unauthenticated requests never do."
    ),
    def!(
        "security.trust.knownCapacity",
        int(100, 100000),
        "10000",
        G,
        "security",
        "Most known sources kept in memory. The oldest is dropped first. Only a salted hash of the address, its country and a small account count are kept."
    ),
    def!(
        "security.trust.persist",
        Kind::Bool,
        "true",
        G,
        "security",
        "Save the known-source table to the settings store and load it at startup, so a restart does not begin with every source unknown. Only keyed hashes of the address, the country, the last-seen time and a small account count are saved; entries older than the window are dropped when saving and loading."
    ),
    def!(
        "security.trust.persistIntervalSecs",
        int(30, 3600),
        "300",
        G,
        "security",
        "Seconds between saves of the known-source table. A save happens only when the table changed since the last one, and once more at graceful shutdown."
    ),
    def!(
        "security.trust.unknownPercent",
        int(10, 100),
        "100",
        G,
        "security",
        "Anonymous request allowance for an unknown source as a percentage of the configured limits. 100 keeps the baseline; lower is stricter. Applies to the per-address request budget, /api/discover, the sign-in cooldown and the challenge difficulty, never to credential checks, second factors, lockouts or abuse signals."
    ),
    def!(
        "security.trust.knownPercent",
        int(100, 1000),
        "200",
        G,
        "security",
        "Anonymous request allowance for a known source as a percentage of the configured limits. 200 doubles the request budgets and halves the sign-in cooldown and challenge difficulty. It never bypasses credentials, second factors, sign-in lockouts, the SSRF gate or an active block."
    ),
    def!(
        "security.trust.uaMissing",
        Kind::Enum(&["allow", "limit", "block"]),
        "\"limit\"",
        G,
        "security",
        "Action for anonymous requests whose user-agent claims an empty user-agent: allow skips the user-agent budget, limit keeps it, block refuses the request. Signed-in sessions are never affected. A claim is a header, not verified identity; security.protection.userAgentExceptions keeps a needed client allowed."
    ),
    def!(
        "security.trust.uaUncommon",
        Kind::Enum(&["allow", "limit", "block"]),
        "\"limit\"",
        G,
        "security",
        "Action for anonymous requests whose user-agent claims an unrecognised user-agent: allow skips the user-agent budget, limit keeps it, block refuses the request. Signed-in sessions are never affected. A claim is a header, not verified identity; security.protection.userAgentExceptions keeps a needed client allowed."
    ),
    def!(
        "security.trust.uaAutomation",
        Kind::Enum(&["allow", "limit", "block"]),
        "\"limit\"",
        G,
        "security",
        "Action for anonymous requests whose user-agent claims a script, HTTP library, headless browser or generic bot: allow skips the user-agent budget, limit keeps it, block refuses the request. Signed-in sessions are never affected. A claim is a header, not verified identity; security.protection.userAgentExceptions keeps a needed client allowed."
    ),
    def!(
        "security.trust.uaSearch",
        Kind::Enum(&["allow", "limit", "block"]),
        "\"limit\"",
        G,
        "security",
        "Action for anonymous requests whose user-agent claims a declared search-engine crawler: allow skips the user-agent budget, limit keeps it, block refuses the request. Signed-in sessions are never affected. A claim is a header, not verified identity; security.protection.userAgentExceptions keeps a needed client allowed."
    ),
    def!(
        "security.trust.uaAiTraining",
        Kind::Enum(&["allow", "limit", "block"]),
        "\"block\"",
        G,
        "security",
        "Action for anonymous requests whose user-agent claims a declared AI training crawler (GPTBot, ClaudeBot, CCBot, Amazonbot, Bytespider, Google-Extended, Applebot-Extended): allow skips the user-agent budget, limit keeps it, block refuses the request. Signed-in sessions are never affected. A claim is a header, not verified identity; security.protection.userAgentExceptions keeps a needed client allowed."
    ),
    def!(
        "security.trust.uaAiSearch",
        Kind::Enum(&["allow", "limit", "block"]),
        "\"block\"",
        G,
        "security",
        "Action for anonymous requests whose user-agent claims a declared AI search crawler (OAI-SearchBot, PerplexityBot, Claude-SearchBot): allow skips the user-agent budget, limit keeps it, block refuses the request. Signed-in sessions are never affected. A claim is a header, not verified identity; security.protection.userAgentExceptions keeps a needed client allowed."
    ),
    def!(
        "security.trust.uaAiUser",
        Kind::Enum(&["allow", "limit", "block"]),
        "\"block\"",
        G,
        "security",
        "Action for anonymous requests whose user-agent claims a declared user-triggered AI fetcher (ChatGPT-User, Claude-User): allow skips the user-agent budget, limit keeps it, block refuses the request. Signed-in sessions are never affected. A claim is a header, not verified identity; security.protection.userAgentExceptions keeps a needed client allowed."
    ),
    def!(
        "security.trust.uaCustom",
        Kind::Enum(&["allow", "limit", "block"]),
        "\"limit\"",
        G,
        "security",
        "Action for anonymous requests whose user-agent claims a user-agent matching security.protection.userAgentPatterns: allow skips the user-agent budget, limit keeps it, block refuses the request. Signed-in sessions are never affected. A claim is a header, not verified identity; security.protection.userAgentExceptions keeps a needed client allowed."
    ),
    pin(
        def!(
            "security.trust.adminGeoEnabled",
            Kind::Bool,
            "true",
            G,
            "security",
            "Block a source that probes for the admin console from a country in which no recent sign-in succeeded. Needs a readable country database; without one nothing is blocked. security.adminProbe.enabled must also be on; MW_ADMIN_GEO_ENABLED pins this switch for recovery."
        ),
        "MW_ADMIN_GEO_ENABLED",
    ),
    def!(
        "security.trust.adminGeoCountries",
        Kind::StringList {
            max_len: 249,
            max_item: 2
        },
        "[]",
        G,
        "security",
        "Uppercase two-letter countries always treated as known for the admin-console country block, besides the countries of recent successful sign-ins."
    ),
    def!(
        "security.trust.adminGeoLookupFailure",
        Kind::Enum(&["allow", "unknown"]),
        "\"allow\"",
        G,
        "security",
        "When the source has no country (no database, or an unmapped address): allow does not block and logs the case, unknown treats the source as an unknown country."
    ),
    def!(
        "security.trust.adminGeoThreshold",
        int(1, 100),
        "3",
        G,
        "security",
        "Admin-discovery probes from an unknown country inside the window that start a block."
    ),
    def!(
        "security.trust.adminGeoWindowSecs",
        int(60, 86400),
        "600",
        G,
        "security",
        "Window for security.trust.adminGeoThreshold."
    ),
    def!(
        "security.trust.adminGeoBlockOnFirst",
        Kind::Bool,
        "false",
        G,
        "security",
        "Block on the first admin-discovery probe from an unknown country instead of waiting for the threshold. Raises the chance of locking out a travelling administrator."
    ),
    def!(
        "security.trust.adminGeoBlockSecs",
        int(60, 2592000),
        "3600",
        G,
        "security",
        "Length of the first block."
    ),
    def!(
        "security.trust.adminGeoSecondBlockSecs",
        int(60, 2592000),
        "86400",
        G,
        "security",
        "Length of the second block of the same source inside the repeat period."
    ),
    def!(
        "security.trust.adminGeoMaxBlockSecs",
        int(60, 2592000),
        "604800",
        G,
        "security",
        "Length of the third and every later block inside the repeat period."
    ),
    def!(
        "security.trust.adminGeoRepeatDays",
        int(1, 365),
        "30",
        G,
        "security",
        "A block that follows the previous one within this many days counts as a repeat and uses the next, longer length."
    ),
    def!(
        "security.signin.accountGeoMode",
        Kind::Enum(&["off", "block-unusual-countries", "block-foreign-countries"]),
        "\"off\"",
        GDU,
        "security",
        "Per-account sign-in country policy, off by default. block-unusual-countries refuses a sign-in from a country the account has not signed in from before (its first sign-in is always allowed); block-foreign-countries refuses any country outside the allowed countries. Allowlisted IPs and countries always pass. Applies to new webmail sign-ins (password, passkey, SSO) only, never to existing sessions or protocol clients. The administrator sets the instance default and may lock it. Country detection uses an IP geolocation database that can be wrong or out of date; VPNs, mobile carriers, proxies and travel can block a legitimate sign-in or let a foreign one through. Add trusted IPs or countries to the allowlist."
    ),
    def!(
        "security.signin.accountGeoAllowCountries",
        Kind::StringList {
            max_len: 50,
            max_item: 2
        },
        "[]",
        GDU,
        "security",
        "Countries (uppercase ISO 3166 codes) always allowed for this account by the per-account sign-in policy. Country detection uses an IP geolocation database that can be wrong or out of date; VPNs, mobile carriers, proxies and travel can block a legitimate sign-in or let a foreign one through. Add trusted IPs or countries to the allowlist."
    ),
    def!(
        "security.signin.accountGeoAllowIps",
        Kind::StringList {
            max_len: 50,
            max_item: 64
        },
        "[]",
        GDU,
        "security",
        "IP addresses or CIDR networks (IPv4 or IPv6) always allowed for this account by the per-account sign-in policy. A single IPv6 address means its /64."
    ),
    def!(
        "security.signin.accountGeoGlobalAllowCountries",
        Kind::StringList {
            max_len: 100,
            max_item: 2
        },
        "[]",
        G,
        "security",
        "Countries (uppercase ISO 3166 codes) allowed for every account by the per-account sign-in policy. Added to each account's own list."
    ),
    def!(
        "security.signin.accountGeoGlobalAllowIps",
        Kind::StringList {
            max_len: 100,
            max_item: 64
        },
        "[]",
        G,
        "security",
        "IP addresses or CIDR networks allowed for every account by the per-account sign-in policy (an office or VPN). Added to each account's own list."
    ),
    def!(
        "security.signin.accountGeoLookupFailure",
        Kind::Enum(&["allow", "deny"]),
        "\"allow\"",
        G,
        "security",
        "When the source has no country (no database, or an unmapped address) and the per-account policy is on: allow the sign-in and log it, or deny it."
    ),
    def!(
        "security.signin.accountGeoHistoryDays",
        int(30, 3650),
        "365",
        G,
        "security",
        "How long a country seen in a successful sign-in stays in the account's own history for the unusual-countries policy. Older countries are ignored and removed."
    ),
    def!(
        "security.geo.noDatabase",
        Kind::Enum(&["allow", "treat-as-unknown", "block"]),
        "\"allow\"",
        G,
        "security",
        "Shared default when no country database is loaded: allow (carry on, logged), treat-as-unknown (handle the source as a country not on the feature's list) or block (refuse a sign-in; other features treat block as treat-as-unknown). A feature's own key is applied too and the stricter value wins."
    ),
    def!(
        "security.geo.notFound",
        Kind::Enum(&["allow", "treat-as-unknown", "block"]),
        "\"allow\"",
        G,
        "security",
        "Shared default when a country database is loaded but has no entry for the address (unallocated or reserved ranges). Same values and precedence as the no-database key. Private, loopback and link-local addresses are local networks and are never affected."
    ),
    def!(
        "security.signin.geoNoDatabase",
        Kind::Enum(&["allow", "block"]),
        "\"block\"",
        G,
        "security",
        "Instance sign-in country policy (`geoScope`) when no database is loaded: block refuses sign-ins in scope (the original fail-closed behaviour), allow lets them through unchecked. Local networks are always allowed."
    ),
    def!(
        "security.geo.sessionLookupFailure",
        Kind::Enum(&["ignore", "alert"]),
        "\"ignore\"",
        G,
        "security",
        "Session country-change alerts and impossible-travel checks when a request's country cannot be determined: ignore it, or raise a session alert once per session. Never revokes or blocks a session."
    ),
    def!(
        "security.signin.geoEnforceSessions",
        Kind::Bool,
        "false",
        G,
        "security",
        "also check current source country on authenticated requests"
    ),
    def!(
        "security.signin.geoScope",
        Kind::Enum(&["off", "mailbox", "admin", "all"]),
        "\"off\"",
        G,
        "security",
        "country policy applies only to new sign-ins"
    ),
    def!(
        "security.signin.geoMode",
        Kind::Enum(&["allow", "deny"]),
        "\"allow\"",
        G,
        "security",
        "allow only listed countries or deny listed countries"
    ),
    def!(
        "security.signin.geoCountries",
        Kind::StringList {
            max_len: 249,
            max_item: 2
        },
        "[]",
        G,
        "security",
        "uppercase two-letter country codes"
    ),
    def!(
        "security.signin.geoUnknown",
        Kind::Enum(&["allow", "deny"]),
        "\"deny\"",
        G,
        "security",
        "decision for unmapped addresses in a readable local database"
    ),
    def!(
        "security.signin.mailboxFailureLimit",
        int(1, 100),
        "10",
        G,
        "security",
        "rejected mailbox credentials per source before a temporary lock"
    ),
    def!(
        "security.signin.adminFailureLimit",
        int(1, 100),
        "5",
        G,
        "security",
        "rejected admin credentials per source before a temporary lock"
    ),
    def!(
        "security.signin.failureWindowSecs",
        int(30, 86400),
        "900",
        G,
        "security",
        "rolling failed-credential counting window"
    ),
    def!(
        "security.signin.lockSecs",
        int(30, 86400),
        "900",
        G,
        "security",
        "fixed temporary source lock duration; rejected retries never extend it"
    ),
    def!(
        "signin.retryCooldownSecs",
        int(0, 300),
        "3",
        G,
        "signin",
        "wait after a sign-in attempt that does not end in a session (per source network)"
    ),
    // ── Diagnostics budget per admin session (t104) ─────────────────────────────
    dedicated(
        def!(
            "rateLimit.diagnosticsBurst",
            int(1, 100),
            "10",
            G,
            "security",
            "diagnostic runs and backend checks an admin session may start in a row (keyed by session); repeating a job the session has running returns it uncharged"
        ),
        crate::request_budget::ROUTE,
    ),
    dedicated(
        def!(
            "rateLimit.diagnosticsPerMinute",
            int(1, 600),
            "10",
            G,
            "security",
            "diagnostic runs and jobs per minute an admin session regains"
        ),
        crate::request_budget::ROUTE,
    ),
    // ── Leeway for signed-in clients (t106) ─────────────────────────────────────
    def!(
        "ratelimit.queueMs",
        int(0, 30000),
        "3000",
        GD,
        "security",
        "Longest a request waits for a free slot (JMAP concurrency, image decoders, preview workers) before it is refused with 429 and Retry-After. Zero refuses at once."
    ),
    def!(
        "ratelimit.jmap.perAccountConcurrent",
        int(4, 64),
        "8",
        GD,
        "security",
        "JMAP requests one signed-in account may run at once, across its tabs and devices. It never shares this with another account or another user behind the same address. Not below 4, the value the session resource advertises."
    ),
    def!(
        "ratelimit.jmap.globalConcurrent",
        int(4, 4096),
        "256",
        G,
        "security",
        "JMAP requests the whole process runs at once, all accounts together. A request over it waits for ratelimit.queueMs."
    ),
    def!(
        "ratelimit.session.burst",
        int(1, 1000),
        "60",
        GD,
        "security",
        "Client timing reports one session may send in a row. The report endpoint is keyed by session, never by address."
    ),
    def!(
        "ratelimit.session.perMinute",
        int(1, 600),
        "30",
        GD,
        "security",
        "Client timing reports per minute a session regains after its burst."
    ),
    def!(
        "ratelimit.anon.burst",
        int(1, 1000),
        "60",
        G,
        "security",
        "Requests one address may make to the anonymous cookie-consent routes in a row. Sized for an office behind one address; sign-in and the request-protection address budget are not affected."
    ),
    def!(
        "ratelimit.anon.perMinute",
        int(1, 600),
        "30",
        G,
        "security",
        "Requests per minute an address regains on the anonymous cookie-consent routes."
    ),
    def!(
        "ratelimit.trustedMultiplier",
        int(100, 1000),
        "150",
        G,
        "security",
        "Percentage of security.protection.accountRequests a signed-in mailbox gets when the request comes from a source with a successful sign-in on record (security.trust.knownTtlDays). 100 keeps the base allowance. Applies only while request protection enforces the account budget; it never relaxes the anonymous, administrator or user-agent budgets or an active block."
    ),
    // ── Sign-in (e66, e70b) ──────────────────────────────────────────────────
    dedicated(
        def!(
            "signin.newUsers",
            Kind::Enum(&["open", "preprovisioned", "provisioned", "closed"]),
            "\"open\"",
            GD,
            "signin",
            "signin_policy::admit new-user rule"
        ),
        SIGNIN_POLICY_ROUTE,
    ),
    dedicated(
        def!(
            "signin.newUsers.mail",
            Kind::Enum(&["inherit", "open", "preprovisioned", "provisioned", "closed"]),
            "\"inherit\"",
            GD,
            "signin",
            "signin_policy::admit rule for password mailbox sign-in"
        ),
        SIGNIN_POLICY_ROUTE,
    ),
    dedicated(
        def!(
            "signin.newUsers.sso",
            Kind::Enum(&["inherit", "open", "preprovisioned", "provisioned", "closed"]),
            "\"inherit\"",
            GD,
            "signin",
            "signin_policy::admit rule for SSO and header-auth sign-in"
        ),
        SIGNIN_POLICY_ROUTE,
    ),
    dedicated(
        def!(
            "signin.newUsers.linked",
            Kind::Enum(&["inherit", "open", "preprovisioned", "provisioned", "closed"]),
            "\"inherit\"",
            GD,
            "signin",
            "signin_policy::admit rule for linked mailboxes (all kinds)"
        ),
        SIGNIN_POLICY_ROUTE,
    ),
    dedicated(
        def!(
            "signin.newUsers.linked.imap",
            Kind::Enum(&["inherit", "open", "preprovisioned", "provisioned", "closed"]),
            "\"inherit\"",
            GD,
            "signin",
            "signin_policy::admit rule for linked IMAP mailboxes"
        ),
        SIGNIN_POLICY_ROUTE,
    ),
    dedicated(
        def!(
            "signin.newUsers.linked.pop3",
            Kind::Enum(&["inherit", "open", "preprovisioned", "provisioned", "closed"]),
            "\"inherit\"",
            GD,
            "signin",
            "signin_policy::admit rule for linked POP3 mailboxes"
        ),
        SIGNIN_POLICY_ROUTE,
    ),
    dedicated(
        def!(
            "signin.newUsers.linked.ews",
            Kind::Enum(&["inherit", "open", "preprovisioned", "provisioned", "closed"]),
            "\"inherit\"",
            GD,
            "signin",
            "signin_policy::admit rule for linked Exchange (EWS) mailboxes"
        ),
        SIGNIN_POLICY_ROUTE,
    ),
    dedicated(
        def!(
            "signin.preprovision.ttlDays",
            int(1, 3650),
            "30",
            G,
            "signin",
            "default lifetime of a pre-provisioned entry"
        ),
        SIGNIN_POLICY_ROUTE,
    ),
    dedicated(
        def!(
            "signin.preprovision.maxTtlDays",
            int(1, 3650),
            "90",
            G,
            "signin",
            "longest lifetime an admin may give a pre-provisioned entry"
        ),
        SIGNIN_POLICY_ROUTE,
    ),
    dedicated(
        def!(
            "signin.preprovision.domainWide",
            Kind::Bool,
            "false",
            G,
            "signin",
            "allow capped domain-wide (*@domain) pre-provisioned entries"
        ),
        SIGNIN_POLICY_ROUTE,
    ),
    dedicated(
        def!(
            "signin.preprovision.retentionDays",
            int(0, 365),
            "30",
            G,
            "signin",
            "days an expired pending entry is kept before cleanup removes it"
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
    def!(
        "admin.allowLastSuperadminLoss",
        Kind::Bool,
        "false",
        G,
        "admin",
        "Explicitly allow removing the final Super admin access; requires privileged confirmation"
    ),
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
        "admin.session.rotateSecs",
        int(0, 86_400),
        "900",
        G,
        "admin",
        "seconds between admin session id rotations while the console is in use (0 = only on step-up); idle and absolute clocks are unchanged"
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
    def!(
        "attachments.previewAllowed",
        Kind::Bool,
        "true",
        GD,
        "attachments",
        "allow attachment previews"
    ),
    def!(
        "attachments.downloadAllowed",
        Kind::Bool,
        "true",
        GD,
        "attachments",
        "allow original attachment delivery; disabling also prevents raw previews"
    ),
    def!(
        "attachments.maxBytes",
        int(0, BYTES_MAX),
        "0",
        GD,
        "attachments",
        "maximum delivered attachment bytes; zero is unlimited"
    ),
    def!(
        "attachments.allowedExtensions",
        Kind::AttachmentList { mime: false },
        "[]",
        GD,
        "attachments",
        "attachment allowed Extensions; blocked rules take precedence"
    ),
    def!(
        "attachments.allowedMimeTypes",
        Kind::AttachmentList { mime: true },
        "[]",
        GD,
        "attachments",
        "attachment allowed MimeTypes; blocked rules take precedence"
    ),
    def!(
        "attachments.blockedExtensions",
        Kind::AttachmentList { mime: false },
        "[]",
        GD,
        "attachments",
        "attachment blocked Extensions; blocked rules take precedence"
    ),
    def!(
        "attachments.blockedMimeTypes",
        Kind::AttachmentList { mime: true },
        "[]",
        GD,
        "attachments",
        "attachment blocked MimeTypes; blocked rules take precedence"
    ),
    def!(
        "attachments.warnExtensions",
        Kind::AttachmentList { mime: false },
        "[]",
        GD,
        "attachments",
        "attachment warn Extensions; blocked rules take precedence"
    ),
    def!(
        "attachments.warnMimeTypes",
        Kind::AttachmentList { mime: true },
        "[]",
        GD,
        "attachments",
        "attachment warn MimeTypes; blocked rules take precedence"
    ),
    // ── Limits, quotas, usage (e67) ─────────────────────────────────────────
    per_address(def!(
        "limit.attachment_bytes",
        int(0, BYTES_MAX),
        "50000000",
        GDA,
        "limits",
        "Attachment bytes; 0 = no limit"
    )),
    per_address(def!(
        "limit.send_per_hour",
        int(0, 1_000_000),
        "0",
        GDA,
        "limits",
        "Submissions per hour; 0 = no limit"
    )),
    per_address(def!(
        "limit.recipients_per_message",
        int(0, 1_000_000),
        "0",
        GDA,
        "limits",
        "Recipients per message; 0 = no limit"
    )),
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
        "limit.linked_mailboxes",
        int(0, 16),
        "0",
        GDA,
        "limits",
        "linked mailboxes per user, below accounts.maxLinked; 0 = no limit beyond it (feature_gate.rs)"
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
    def!(
        "metrics.defaultRangeSeconds",
        int(60, 34560000),
        "3600",
        G,
        "observability",
        "default request history range"
    ),
    def!(
        "metrics.maxRangeSeconds",
        int(3600, 34560000),
        "2592000",
        G,
        "observability",
        "maximum request history range"
    ),
    def!(
        "metrics.rollupDays",
        int(1, 400),
        "30",
        G,
        "observability",
        "hourly metric rollups kept"
    ),
    def!(
        "metrics.persist.enabled",
        Kind::Bool,
        "true",
        G,
        "observability",
        "keep performance statistics across restarts"
    ),
    def!(
        "metrics.persist.flushSeconds",
        int(5, 3600),
        "60",
        G,
        "observability",
        "seconds between saves of performance statistics"
    ),
    def!(
        "metrics.dimensions.enabled",
        Kind::Bool,
        "true",
        G,
        "observability",
        "split saved statistics by client kind and device class"
    ),
    def!(
        "retention.metrics.minuteBucketsHours",
        int(1, 8760),
        "48",
        G,
        "privacy",
        "Hours of one-minute statistics kept"
    ),
    def!(
        "retention.metrics.hourBucketsDays",
        int(1, 3650),
        "60",
        G,
        "privacy",
        "Days of one-hour statistics kept"
    ),
    def!(
        "retention.metrics.dayBucketsDays",
        int(0, 3650),
        "400",
        G,
        "privacy",
        "Days of one-day statistics kept (0 keeps them without limit)"
    ),
    def!(
        "observability.clientTiming.enabled",
        Kind::Bool,
        "true",
        G,
        "observability",
        "browser load time collection"
    ),
    def!(
        "observability.clientTiming.sampleRatePercent",
        int(1, 100),
        "100",
        G,
        "observability",
        "browser load time sample rate"
    ),
    def!(
        "observability.clientTiming.minSamples",
        int(1, 100000),
        "20",
        G,
        "observability",
        "samples needed to show load time percentiles"
    ),
    def!(
        "observability.clientTiming.folderLoads",
        Kind::Bool,
        "true",
        GDU,
        "observability",
        "folder load time measurement"
    ),
    def!(
        "observability.clientTiming.messageLoads",
        Kind::Bool,
        "true",
        GDU,
        "observability",
        "message load time measurement"
    ),
    def!(
        "observability.client.enabled",
        Kind::Bool,
        "true",
        G,
        "observability",
        "collect browser error reports (stack traces, scrubbed) for Observability > Client errors"
    ),
    def!(
        "observability.client.sampleRate",
        Kind::Float { min: 0.0, max: 1.0 },
        "1",
        G,
        "observability",
        "fraction of browser error reports sent"
    ),
    def!(
        "observability.client.maxPerSession",
        int(1, 1000),
        "20",
        G,
        "observability",
        "browser error reports one page load may send"
    ),
    def!(
        "observability.client.includeAccountHash",
        Kind::Bool,
        "false",
        G,
        "observability",
        "add a keyed one-way hash of the account to signed-in error reports"
    ),
    def!(
        "observability.client.captureConsole",
        Kind::Bool,
        "false",
        G,
        "observability",
        "also report console.error calls, at a throttled rate"
    ),
    def!(
        "observability.client.captureCsp",
        Kind::Bool,
        "true",
        G,
        "observability",
        "report Content-Security-Policy violations seen by the browser"
    ),
    def!(
        "observability.client.sourceMaps",
        Kind::Bool,
        "true",
        G,
        "observability",
        "resolve browser stack frames with the build's hidden source maps"
    ),
    def!(
        "observability.client.forward",
        Kind::Bool,
        "false",
        G,
        "observability",
        "also send scrubbed browser error reports to the configured error sink (MW_ERROR_FORWARD_URL)"
    ),
    def!(
        "retention.clientErrors.eventDays",
        int(1, 3650),
        "30",
        G,
        "privacy",
        "Days of individual browser error occurrences kept"
    ),
    def!(
        "retention.clientErrors.groupDays",
        int(1, 3650),
        "180",
        G,
        "privacy",
        "Days a browser error group is kept after it was last seen"
    ),
    def!(
        "privacy.errorReports",
        Kind::Enum(&["on", "off"]),
        "\"on\"",
        GDU,
        "privacy",
        "send an error report from this browser when a section fails to load"
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
    def!(
        "dsr.enabled",
        Kind::Bool,
        "true",
        GD,
        "privacy",
        "data subject requests accepted"
    ),
    def!(
        "dsr.type.access",
        Kind::Bool,
        "true",
        GD,
        "privacy",
        "access requests accepted"
    ),
    def!(
        "dsr.type.portability",
        Kind::Bool,
        "true",
        GD,
        "privacy",
        "portability requests accepted"
    ),
    def!(
        "dsr.type.rectification",
        Kind::Bool,
        "true",
        GD,
        "privacy",
        "rectification requests accepted"
    ),
    def!(
        "dsr.type.erasure",
        Kind::Bool,
        "true",
        GD,
        "privacy",
        "erasure requests accepted"
    ),
    def!(
        "dsr.type.restriction",
        Kind::Bool,
        "true",
        GD,
        "privacy",
        "restriction requests accepted"
    ),
    def!(
        "dsr.type.objection",
        Kind::Bool,
        "true",
        GD,
        "privacy",
        "objection requests accepted"
    ),
    def!(
        "dsr.type.other",
        Kind::Bool,
        "true",
        GD,
        "privacy",
        "other requests accepted"
    ),
    def!(
        "dsr.approval.export",
        Kind::Enum(&["auto", "admin"]),
        "\"auto\"",
        GD,
        "privacy",
        "who approves access and portability requests"
    ),
    def!(
        "dsr.approval.erasure",
        Kind::Enum(&["auto", "admin"]),
        "\"admin\"",
        GD,
        "privacy",
        "who approves erasure requests"
    ),
    def!(
        "dsr.approval.other",
        Kind::Enum(&["auto", "admin"]),
        "\"admin\"",
        GD,
        "privacy",
        "who handles other request types"
    ),
    def!(
        "dsr.reauth.required",
        Kind::Bool,
        "true",
        GD,
        "privacy",
        "fresh password or second factor to submit or download"
    ),
    def!(
        "dsr.confirmation.required",
        Kind::Bool,
        "true",
        GD,
        "privacy",
        "confirmation message to the requester's inbox"
    ),
    def!(
        "dsr.admin.stepUp",
        Kind::Bool,
        "true",
        GD,
        "privacy",
        "administrators confirm it is them again before approving a data request or running an erasure"
    ),
    def!(
        "dsr.identityVerification.required",
        Kind::Bool,
        "false",
        GD,
        "privacy",
        "admin identity verification step"
    ),
    def!(
        "dsr.erasure.graceDays",
        int(0, 365),
        "14",
        GD,
        "privacy",
        "days between approval and erasure"
    ),
    def!(
        "dsr.sla.days",
        int(1, 90),
        "30",
        GD,
        "privacy",
        "days allowed to answer a request"
    ),
    def!(
        "dsr.sla.maxExtensionDays",
        int(0, 90),
        "60",
        G,
        "privacy",
        "longest extension an admin can record"
    ),
    def!(
        "dsr.sla.reminderDays",
        Kind::IntList { min: 1, max: 90, max_len: 5 },
        "[1,3,7]",
        G,
        "privacy",
        "days before the due date that flag a request"
    ),
    def!(
        "dsr.export.mailAllowed",
        Kind::Bool,
        "false",
        GD,
        "privacy",
        "mail may be included in an export"
    ),
    def!(
        "dsr.export.mailFormat",
        Kind::Enum(&["mbox", "eml"]),
        "\"mbox\"",
        GD,
        "privacy",
        "format of mail in an export"
    ),
    def!(
        "dsr.export.include.settings",
        Kind::Bool,
        "true",
        GD,
        "privacy",
        "settings in an export"
    ),
    def!(
        "dsr.export.include.identities",
        Kind::Bool,
        "true",
        GD,
        "privacy",
        "identities in an export"
    ),
    def!(
        "dsr.export.include.contacts",
        Kind::Bool,
        "true",
        GD,
        "privacy",
        "contacts in an export"
    ),
    def!(
        "dsr.export.include.calendars",
        Kind::Bool,
        "true",
        GD,
        "privacy",
        "calendars in an export"
    ),
    def!(
        "dsr.export.include.notes",
        Kind::Bool,
        "true",
        GD,
        "privacy",
        "notes in an export"
    ),
    def!(
        "dsr.export.include.rules",
        Kind::Bool,
        "true",
        GD,
        "privacy",
        "rules in an export"
    ),
    def!(
        "dsr.export.include.sessions",
        Kind::Bool,
        "true",
        GD,
        "privacy",
        "sessions in an export"
    ),
    def!(
        "dsr.export.include.consent",
        Kind::Bool,
        "true",
        GD,
        "privacy",
        "consent in an export"
    ),
    def!(
        "dsr.export.include.audit",
        Kind::Bool,
        "true",
        GD,
        "privacy",
        "audit in an export"
    ),
    def!(
        "dsr.export.maxMegabytes",
        int(1, 2000),
        "500",
        G,
        "privacy",
        "size cap of one export"
    ),
    def!(
        "dsr.export.format",
        Kind::Enum(&["json", "jsonCsv"]),
        "\"json\"",
        GD,
        "privacy",
        "machine-readable formats in an export"
    ),
    def!(
        "dsr.export.encrypt",
        Kind::Bool,
        "false",
        GD,
        "privacy",
        "exports sealed at rest"
    ),
    def!(
        "dsr.download.linkMinutes",
        int(5, 10080),
        "1440",
        GD,
        "privacy",
        "minutes a download link works"
    ),
    def!(
        "dsr.download.singleUse",
        Kind::Bool,
        "true",
        GD,
        "privacy",
        "download link works once"
    ),
    def!(
        "dsr.retention.completedDays",
        int(30, 3650),
        "365",
        G,
        "privacy",
        "days a finished request is kept"
    ),
    def!(
        "dsr.notify.admins",
        Kind::Bool,
        "true",
        G,
        "privacy",
        "new requests recorded as security events for admins"
    ),
    def!(
        "dsr.notify.requester",
        Kind::Bool,
        "true",
        GDU,
        "privacy",
        "progress messages to the requester's inbox"
    ),
    nullable(def!(
        "dsr.contact.name",
        text(Text::Plain, 200),
        "null",
        GD,
        "privacy",
        "data protection contact name shown to users"
    )),
    nullable(def!(
        "dsr.contact.email",
        text(Text::Email, 254),
        "null",
        GD,
        "privacy",
        "data protection contact address shown to users"
    )),
    nullable(def!(
        "dsr.contact.url",
        text(Text::HttpsUrl, 2048),
        "null",
        GD,
        "privacy",
        "data protection contact page shown to users"
    )),
    def!(
        "policy.publicPages",
        Kind::Bool,
        "true",
        GD,
        "privacy",
        "serve the generated privacy and cookie policy pages at /privacy and /cookies-policy"
    ),
    def!(
        "policy.autoVersion",
        Kind::Bool,
        "true",
        GD,
        "privacy",
        "record a new policy version automatically when a policy-relevant setting changes"
    ),
    def!(
        "policy.reconsentOnChange",
        Kind::Bool,
        "false",
        GD,
        "privacy",
        "raise cookies.policyVersion when a new policy version is recorded, so visitors are asked again"
    ),
    nullable(def!(
        "policy.controller.name",
        text(Text::Plain, 300),
        "null",
        GD,
        "privacy",
        "name of the data controller shown in the privacy policy"
    )),
    nullable(def!(
        "policy.controller.address",
        text(Text::Plain, 500),
        "null",
        GD,
        "privacy",
        "postal address of the data controller shown in the privacy policy"
    )),
    nullable(def!(
        "policy.legalBases",
        text(Text::Plain, 1000),
        "null",
        GD,
        "privacy",
        "legal bases the operator relies on, as written in the privacy policy"
    )),
    nullable(def!(
        "policy.hosting.provider",
        text(Text::Plain, 500),
        "null",
        GD,
        "privacy",
        "hosting provider and location of the data, shown in the privacy policy"
    )),
    nullable(def!(
        "policy.hosting.transfers",
        text(Text::Plain, 1000),
        "null",
        GD,
        "privacy",
        "transfers of data outside the hosting country and their safeguards, shown in the privacy policy"
    )),
    nullable(def!(
        "policy.supervisoryAuthority",
        text(Text::Plain, 500),
        "null",
        GD,
        "privacy",
        "supervisory authority users may complain to, shown in the privacy policy"
    )),
    def!(
        "dpia.suggest",
        Kind::Bool,
        "true",
        G,
        "privacy",
        "suggest an impact assessment when the assistant, open tracking or analytics is enabled"
    ),
    def!(
        "dpia.reviewMonths",
        int(1, 60),
        "12",
        G,
        "privacy",
        "months between impact assessment reviews when a new assessment is approved"
    ),
    def!(
        "dpia.reminderDays",
        int(0, 365),
        "30",
        G,
        "privacy",
        "days before an impact assessment review date it is listed as due"
    ),
    def!(
        "dsr.onBehalf.allowed",
        Kind::Bool,
        "true",
        GD,
        "privacy",
        "admins may file a request for a user"
    ),
    def!(
        "dsr.rateLimit.perDay",
        int(1, 100),
        "5",
        GD,
        "privacy",
        "requests one person may file per day"
    ),
    def!(
        "dsr.legalHold.subjects",
        Kind::StringList { max_len: 500, max_item: 254 },
        "[]",
        G,
        "privacy",
        "addresses and @domains under legal hold"
    ),
    def!(
        "scrub.enabled",
        Kind::Bool,
        "true",
        G,
        "privacy",
        "the Scrub person action and the erasure-time scrub may run"
    ),
    def!(
        "scrub.pseudonymAlgorithm",
        Kind::Enum(&["hmac-sha256-16", "hmac-sha256-32"]),
        "\"hmac-sha256-16\"",
        G,
        "privacy",
        "keyed pseudonym used to replace identifiers: HMAC-SHA256 truncated to 16 or 32 hex characters"
    ),
    def!(
        "scrub.includeLogs",
        Kind::Bool,
        "true",
        G,
        "privacy",
        "a scrub also rewrites the rotated log files (mailwoman.log and its .gz rotations)"
    ),
    def!(
        "scrub.autoRunOnErasure",
        Kind::Bool,
        "false",
        G,
        "privacy",
        "a completed erasure also scrubs the person's identifiers from audit history and logs"
    ),
    def!(
        "scrub.keyRetentionDays",
        int(0, 3650),
        "0",
        G,
        "privacy",
        "days a retired pseudonym key is kept after rotation; 0 keeps it, after deletion old pseudonyms can no longer be recomputed"
    ),
    def!(
        "backup.propagateErasure",
        Kind::Bool,
        "true",
        G,
        "privacy",
        "a restore replays the erasure tombstones newer than the backup, so erased people are removed again"
    ),
    def!(
        "backup.tombstoneRetentionDays",
        int(0, 36500),
        "3650",
        G,
        "privacy",
        "days an erasure tombstone is kept; 0 keeps them forever"
    ),
    def!(
        "dsr.text.intro",
        Kind::LocaleMap { max: 2000 },
        "{}",
        GD,
        "privacy",
        "intro text"
    ),
    def!(
        "dsr.text.confirmation",
        Kind::LocaleMap { max: 2000 },
        "{}",
        GD,
        "privacy",
        "confirmation text"
    ),
    def!(
        "dsr.text.completion",
        Kind::LocaleMap { max: 2000 },
        "{}",
        GD,
        "privacy",
        "completion text"
    ),
    def!(
        "breach.enabled",
        Kind::Bool,
        "true",
        GD,
        "privacy",
        "breach register and notice clock switched on"
    ),
    def!(
        "breach.noticeHours",
        int(1, 720),
        "72",
        GD,
        "privacy",
        "hours from awareness to the supervisory authority notice"
    ),
    def!(
        "breach.reminders.hours",
        Kind::IntList { min: 1, max: 720, max_len: 6 },
        "[6,24,48]",
        GD,
        "privacy",
        "hours before the notice deadline that raise a reminder"
    ),
    def!(
        "breach.reminders.overdueRepeatHours",
        int(0, 168),
        "24",
        GD,
        "privacy",
        "hours between reminders once overdue; 0 reminds once"
    ),
    nullable(def!(
        "breach.authority.name",
        text(Text::Plain, 200),
        "null",
        GD,
        "privacy",
        "supervisory authority name used in notices"
    )),
    nullable(def!(
        "breach.authority.url",
        text(Text::HttpsUrl, 2048),
        "null",
        GD,
        "privacy",
        "supervisory authority notification page"
    )),
    nullable(def!(
        "breach.authority.contact",
        text(Text::Plain, 500),
        "null",
        GD,
        "privacy",
        "supervisory authority contact details"
    )),
    nullable(def!(
        "breach.dpo.name",
        text(Text::Plain, 200),
        "null",
        GD,
        "privacy",
        "data protection officer name used in notices"
    )),
    nullable(def!(
        "breach.dpo.email",
        text(Text::Email, 254),
        "null",
        GD,
        "privacy",
        "data protection officer address used in notices"
    )),
    nullable(def!(
        "breach.dpo.phone",
        text(Text::Plain, 60),
        "null",
        GD,
        "privacy",
        "data protection officer phone used in notices"
    )),
    def!(
        "breach.template.subject",
        Kind::LocaleMap { max: 300 },
        "{}",
        GD,
        "privacy",
        "data subject notice subject"
    ),
    def!(
        "breach.template.body",
        Kind::LocaleMap { max: 8000 },
        "{}",
        GD,
        "privacy",
        "data subject notice text"
    ),
    def!(
        "breach.notify.audit",
        Kind::Bool,
        "true",
        GD,
        "privacy",
        "breach reminders and changes recorded as security events"
    ),
    def!(
        "breach.notify.webhookUrls",
        Kind::StringList { max_len: 10, max_item: 2048 },
        "[]",
        G,
        "privacy",
        "webhook URLs told about breach events"
    ),
    def!(
        "breach.notify.operatorAddresses",
        Kind::StringList { max_len: 20, max_item: 254 },
        "[]",
        G,
        "privacy",
        "local mailboxes that receive breach reminders"
    ),
    def!(
        "breach.subjects.inApp",
        Kind::Bool,
        "true",
        GD,
        "privacy",
        "data subject notices shown in the person's account"
    ),
    def!(
        "breach.subjects.inbox",
        Kind::Bool,
        "true",
        GD,
        "privacy",
        "data subject notices delivered to the person's own mailbox"
    ),
    def!(
        "breach.subjects.webhook",
        Kind::Bool,
        "false",
        GD,
        "privacy",
        "data subject notices also sent to the webhook URLs"
    ),
    def!(
        "breach.retention.years",
        int(1, 50),
        "10",
        G,
        "privacy",
        "years a closed incident stays in the register"
    ),
    def!(
        "breach.auto.enabled",
        Kind::Bool,
        "false",
        G,
        "privacy",
        "draft incidents opened from detections"
    ),
    def!(
        "breach.auto.startClock",
        Kind::Enum(&["onConfirm", "onDetect"]),
        "\"onConfirm\"",
        G,
        "privacy",
        "when a draft incident starts the notice clock"
    ),
    nullable(def!(
        "breach.auto.assignee",
        text(Text::Plain, 120),
        "null",
        G,
        "privacy",
        "who a draft incident is assigned to"
    )),
    def!(
        "breach.auto.escalateAfterHours",
        int(0, 168),
        "4",
        G,
        "privacy",
        "hours an unassigned or unconfirmed draft waits before escalation; 0 never"
    ),
    nullable(def!(
        "breach.auto.escalateTo",
        text(Text::Plain, 120),
        "null",
        G,
        "privacy",
        "who an escalated draft is assigned to"
    )),
    def!(
        "breach.auto.windowMinutes",
        int(1, 1440),
        "60",
        G,
        "privacy",
        "minutes a rule counts events over"
    ),
    def!(
        "breach.auto.theft.threshold",
        int(0, 1000),
        "0",
        G,
        "privacy",
        "session theft alerts in the window that open a draft; 0 is off"
    ),
    def!(
        "breach.auto.credentialStuffing.threshold",
        int(0, 1000),
        "0",
        G,
        "privacy",
        "sign-in spray or spike events in the window that open a draft; 0 is off"
    ),
    def!(
        "breach.auto.massForward.threshold",
        int(0, 1000),
        "0",
        G,
        "privacy",
        "forwarding rule events in the window that open a draft; 0 is off"
    ),
    def!(
        "breach.auto.roleEscalation.threshold",
        int(0, 1000),
        "0",
        G,
        "privacy",
        "admin role changes in the window that open a draft; 0 is off"
    ),
    def!(
        "breach.auto.exportVolume.threshold",
        int(0, 1000),
        "0",
        G,
        "privacy",
        "audit and consent exports in the window that open a draft; 0 is off"
    ),
    def!(
        "breach.auto.auditChain.threshold",
        int(0, 1000),
        "0",
        G,
        "privacy",
        "audit chain verification failures that open a draft; 0 is off"
    ),
    def!(
        "breach.auto.backupAnomaly.threshold",
        int(0, 1000),
        "0",
        G,
        "privacy",
        "backup or restore failures in the window that open a draft; 0 is off"
    ),
    def!(
        "breach.auto.dsrExport.threshold",
        int(0, 1000),
        "0",
        G,
        "privacy",
        "data request exports in the window that open a draft; 0 is off"
    ),
    def!(
        "restriction.enabled",
        Kind::Bool,
        "true",
        GD,
        "privacy",
        "restrictions of processing allowed"
    ),
    def!(
        "restriction.autoApplyOnApproval",
        Kind::Bool,
        "true",
        GD,
        "privacy",
        "restriction applied when a restriction request is approved"
    ),
    def!(
        "restriction.mailDelivery",
        Kind::Enum(&["keep", "queue", "reject"]),
        "\"keep\"",
        GD,
        "privacy",
        "new mail while restricted"
    ),
    def!(
        "restriction.defaultExpiryDays",
        int(0, 3650),
        "0",
        GD,
        "privacy",
        "days until a restriction ends (0 = until lifted)"
    ),
    def!(
        "restriction.notifyUser",
        Kind::Bool,
        "true",
        GD,
        "privacy",
        "notice to the person on apply and lift"
    ),
    def!(
        "restriction.scope.indexing",
        Kind::Bool,
        "true",
        GD,
        "privacy",
        "indexing paused by default"
    ),
    def!(
        "restriction.scope.assistant",
        Kind::Bool,
        "true",
        GD,
        "privacy",
        "assistant paused by default"
    ),
    def!(
        "restriction.scope.analytics",
        Kind::Bool,
        "true",
        GD,
        "privacy",
        "analytics paused by default"
    ),
    def!(
        "restriction.scope.classification",
        Kind::Bool,
        "true",
        GD,
        "privacy",
        "classification paused by default"
    ),
    def!(
        "restriction.scope.push",
        Kind::Bool,
        "true",
        GD,
        "privacy",
        "push paused by default"
    ),
    def!(
        "restriction.scope.thirdParty",
        Kind::Bool,
        "true",
        GD,
        "privacy",
        "thirdParty paused by default"
    ),
    def!(
        "consent.enabled",
        Kind::Bool,
        "true",
        GD,
        "privacy",
        "purpose consent ledger and Your data section"
    ),
    def!(
        "consent.receipts.retentionDays",
        int(0, 3650),
        "1825",
        G,
        "privacy",
        "days consent receipts are kept (0 = keep)"
    ),
    def!(
        "consent.art22.notice",
        Kind::Bool,
        "true",
        GD,
        "privacy",
        "automated processing notice shown to users"
    ),
    def!(
        "consent.analytics.basis",
        Kind::Enum(&["consent", "legitimateInterest", "contract", "legalObligation"]),
        "\"consent\"",
        GD,
        "privacy",
        "legal basis shown for analytics"
    ),
    def!(
        "consent.analytics.version",
        int(1, 1000000),
        "1",
        GD,
        "privacy",
        "consent text version of analytics; raising it asks again"
    ),
    def!(
        "consent.analytics.text",
        Kind::LocaleMap { max: 2000 },
        "{}",
        GD,
        "privacy",
        "consent text of analytics"
    ),
    def!(
        "consent.analyticsTag.basis",
        Kind::Enum(&["consent", "legitimateInterest", "contract", "legalObligation"]),
        "\"consent\"",
        GD,
        "privacy",
        "legal basis shown for analyticsTag"
    ),
    def!(
        "consent.analyticsTag.version",
        int(1, 1000000),
        "1",
        GD,
        "privacy",
        "consent text version of analyticsTag; raising it asks again"
    ),
    def!(
        "consent.analyticsTag.text",
        Kind::LocaleMap { max: 2000 },
        "{}",
        GD,
        "privacy",
        "consent text of analyticsTag"
    ),
    def!(
        "consent.assistant.basis",
        Kind::Enum(&["consent", "legitimateInterest", "contract", "legalObligation"]),
        "\"contract\"",
        GD,
        "privacy",
        "legal basis shown for assistant"
    ),
    def!(
        "consent.assistant.version",
        int(1, 1000000),
        "1",
        GD,
        "privacy",
        "consent text version of assistant; raising it asks again"
    ),
    def!(
        "consent.assistant.text",
        Kind::LocaleMap { max: 2000 },
        "{}",
        GD,
        "privacy",
        "consent text of assistant"
    ),
    def!(
        "consent.push.basis",
        Kind::Enum(&["consent", "legitimateInterest", "contract", "legalObligation"]),
        "\"contract\"",
        GD,
        "privacy",
        "legal basis shown for push"
    ),
    def!(
        "consent.push.version",
        int(1, 1000000),
        "1",
        GD,
        "privacy",
        "consent text version of push; raising it asks again"
    ),
    def!(
        "consent.push.text",
        Kind::LocaleMap { max: 2000 },
        "{}",
        GD,
        "privacy",
        "consent text of push"
    ),
    def!(
        "consent.openTracking.basis",
        Kind::Enum(&["consent", "legitimateInterest", "contract", "legalObligation"]),
        "\"contract\"",
        GD,
        "privacy",
        "legal basis shown for openTracking"
    ),
    def!(
        "consent.openTracking.version",
        int(1, 1000000),
        "1",
        GD,
        "privacy",
        "consent text version of openTracking; raising it asks again"
    ),
    def!(
        "consent.openTracking.text",
        Kind::LocaleMap { max: 2000 },
        "{}",
        GD,
        "privacy",
        "consent text of openTracking"
    ),
    def!(
        "consent.remoteContent.basis",
        Kind::Enum(&["consent", "legitimateInterest", "contract", "legalObligation"]),
        "\"consent\"",
        GD,
        "privacy",
        "legal basis shown for remoteContent"
    ),
    def!(
        "consent.remoteContent.version",
        int(1, 1000000),
        "1",
        GD,
        "privacy",
        "consent text version of remoteContent; raising it asks again"
    ),
    def!(
        "consent.remoteContent.text",
        Kind::LocaleMap { max: 2000 },
        "{}",
        GD,
        "privacy",
        "consent text of remoteContent"
    ),
    def!(
        "consent.integrations.basis",
        Kind::Enum(&["consent", "legitimateInterest", "contract", "legalObligation"]),
        "\"contract\"",
        GD,
        "privacy",
        "legal basis shown for integrations"
    ),
    def!(
        "consent.integrations.version",
        int(1, 1000000),
        "1",
        GD,
        "privacy",
        "consent text version of integrations; raising it asks again"
    ),
    def!(
        "consent.integrations.text",
        Kind::LocaleMap { max: 2000 },
        "{}",
        GD,
        "privacy",
        "consent text of integrations"
    ),
    def!(
        "consent.telemetry.basis",
        Kind::Enum(&["consent", "legitimateInterest", "contract", "legalObligation"]),
        "\"legitimateInterest\"",
        GD,
        "privacy",
        "legal basis shown for telemetry"
    ),
    def!(
        "consent.telemetry.version",
        int(1, 1000000),
        "1",
        GD,
        "privacy",
        "consent text version of telemetry; raising it asks again"
    ),
    def!(
        "consent.telemetry.text",
        Kind::LocaleMap { max: 2000 },
        "{}",
        GD,
        "privacy",
        "consent text of telemetry"
    ),
    def!(
        "logs.auth.retentionDays",
        int(1, 90),
        "7",
        G,
        "observability",
        "authentication history days kept"
    ),
    def!(
        "logs.auth.maxEntries",
        int(1000, 100000),
        "50000",
        G,
        "observability",
        "maximum authentication history entries"
    ),
    def!(
        "retention.sessions.idleDays",
        int(0, 3650),
        "90",
        G,
        "privacy",
        "Days a browser or native-client session row may sit unused before it is deleted; 0 keeps them"
    ),
    def!(
        "retention.sweeper.enabled",
        Kind::Bool,
        "true",
        G,
        "privacy",
        "The retention sweeper runs daily and deletes rows older than the limits below"
    ),
    def!(
        "retention.sweeper.batchRows",
        int(50, 10000),
        "500",
        G,
        "privacy",
        "Rows deleted per statement by the sweeper"
    ),
    def!(
        "retention.sweeper.maxBatches",
        int(1, 1000),
        "20",
        G,
        "privacy",
        "Delete statements per store in one run"
    ),
    def!(
        "retention.sweeper.jitterMinutes",
        int(0, 240),
        "30",
        G,
        "privacy",
        "Spread of the daily start time"
    ),
    def!(
        "retention.sweeper.vacuumAfterRows",
        int(0, 100000000),
        "100000",
        G,
        "privacy",
        "Reclaim database space after a run that deleted this many rows"
    ),
    def!(
        "retention.passwordChangeAudit.days",
        int(0, 3650),
        "365",
        G,
        "privacy",
        "Days password-change records are kept; 0 keeps them forever"
    ),
    def!(
        "retention.ssoLoginAudit.days",
        int(0, 3650),
        "90",
        G,
        "privacy",
        "Days single sign-on login records are kept; 0 keeps them forever"
    ),
    def!(
        "retention.dlpAudit.days",
        int(0, 3650),
        "365",
        G,
        "privacy",
        "Days data-loss-prevention match records are kept; 0 keeps them forever"
    ),
    def!(
        "retention.assistAudit.days",
        int(0, 3650),
        "365",
        G,
        "privacy",
        "Days assistant access records are kept; 0 keeps them forever"
    ),
    def!(
        "retention.alertEvents.days",
        int(0, 3650),
        "90",
        G,
        "privacy",
        "Days fired-alert records are kept; 0 keeps them forever"
    ),
    def!(
        "retention.jobRuns.days",
        int(0, 3650),
        "90",
        G,
        "privacy",
        "Days finished background-job runs are kept; running jobs are never removed"
    ),
    def!(
        "retention.outbox.days",
        int(0, 3650),
        "30",
        G,
        "privacy",
        "Days delivered or cancelled send records are kept; unsent, retrying and failed ones are never removed"
    ),
    def!(
        "retention.expiredCredentials.graceDays",
        int(0, 365),
        "1",
        G,
        "privacy",
        "Days an expired token, remembered device, sign-in challenge or image link is kept after it expired; 0 removes it at once"
    ),
    def!(
        "retention.ipAddresses.days",
        int(0, 3650),
        "90",
        G,
        "privacy",
        "Days before stored IP addresses in authentication and audit history are truncated or hashed; 0 keeps them as recorded"
    ),
    def!(
        "retention.ipAddresses.mode",
        Kind::Enum(&["truncate", "hash"]),
        "\"truncate\"",
        G,
        "privacy",
        "Whether old IP addresses are cut to a network prefix or replaced by a short keyed hash"
    ),
    def!(
        "retention.user.enabled",
        Kind::Bool,
        "true",
        GD,
        "privacy",
        "Users may set their own retention rules for folders and mail; the rules run on the server as part of the retention sweep family"
    ),
    def!(
        "retention.user.minDays",
        int(1, 3650),
        "7",
        GD,
        "privacy",
        "Fewest days a user rule may wait before it moves or deletes mail; a smaller value is refused when the rule is saved and raised when it runs"
    ),
    def!(
        "retention.user.maxDays",
        int(0, 3650),
        "0",
        GD,
        "privacy",
        "Most days a user rule may wait; 0 sets no ceiling. A larger value is refused when the rule is saved and lowered when it runs"
    ),
    def!(
        "retention.user.maxRules",
        int(1, 500),
        "50",
        GD,
        "privacy",
        "Most retention rules one user may keep"
    ),
    def!(
        "retention.user.allowPermanentDelete",
        Kind::Bool,
        "false",
        GD,
        "privacy",
        "Whether a user rule may delete mail for good. When off, a delete rule moves mail to Trash and the undo window applies before it is purged"
    ),
    def!(
        "retention.user.undoDays",
        int(0, 365),
        "7",
        GD,
        "privacy",
        "Days mail moved to Trash by a retention rule stays there before it is purged; 0 purges at the next run"
    ),
    def!(
        "retention.user.mandatory",
        Kind::StringList {
            max_len: 20,
            max_item: 64
        },
        "[]",
        GD,
        "privacy",
        "Rules every account gets, as folder-role:days entries (for example trash:30 or junk:30). Mail in that folder older than the days is purged; the user cannot change or pause these"
    ),
    def!(
        "retention.user.protectedFolders",
        Kind::StringList {
            max_len: 30,
            max_item: 128
        },
        "[\"drafts\"]",
        GD,
        "privacy",
        "Folders retention rules never touch, by role (drafts, sent, inbox, archive) or by folder name"
    ),
    def!(
        "retention.user.templates",
        Kind::StringList {
            max_len: 30,
            max_item: 64
        },
        "[\"newsletters30\",\"receipts365\",\"notifications14\",\"trash30\",\"junk30\",\"read-archive180\"]",
        GD,
        "privacy",
        "Templates users can adopt in Settings, by id"
    ),
    def!(
        "retention.user.runForRestricted",
        Kind::Bool,
        "false",
        G,
        "privacy",
        "Whether user rules also run for accounts under a restriction of processing"
    ),
    def!(
        "retention.user.intervalHours",
        int(1, 168),
        "24",
        G,
        "privacy",
        "Hours between runs of the user retention job for one account"
    ),
    def!(
        "retention.user.batchMessages",
        int(10, 1000),
        "200",
        G,
        "privacy",
        "Messages one move or delete call handles"
    ),
    def!(
        "retention.user.maxPerRun",
        int(10, 100000),
        "5000",
        G,
        "privacy",
        "Most messages the user retention job changes for one account in one run; the rest waits for the next run"
    ),
    def!(
        "retention.user.pauseMs",
        int(0, 5000),
        "100",
        G,
        "privacy",
        "Pause between batches, in milliseconds, so a large cleanup does not load the mail server"
    ),
    def!(
        "retention.user.allowStripAttachments",
        Kind::Bool,
        "false",
        GD,
        "privacy",
        "Whether users may create rules that remove large attachments from old mail. Off by default; while off the action is hidden, a rule using it is refused and stored ones are skipped"
    ),
    def!(
        "retention.user.strip.minAgeDays",
        int(7, 3650),
        "90",
        GD,
        "privacy",
        "Fewest days a message must be old before a rule may remove its attachments; a smaller value is refused when the rule is saved and raised when it runs"
    ),
    def!(
        "retention.user.strip.minAttachmentKb",
        int(1, 102400),
        "100",
        GD,
        "privacy",
        "Only attachments at least this many KB are removed; smaller ones stay with the message"
    ),
    def!(
        "retention.user.strip.keepInlineImages",
        Kind::Bool,
        "true",
        GD,
        "privacy",
        "Whether images shown inside the message body are kept when attachments are removed"
    ),
    def!(
        "retention.user.strip.undoDays",
        int(-1, 365),
        "-1",
        GD,
        "privacy",
        "Days the original message stays in Trash after its attachments were removed; -1 follows retention.user.undoDays, 0 deletes the original at once"
    ),
    def!(
        "retention.user.calendar.enabled",
        Kind::Bool,
        "false",
        GD,
        "privacy",
        "Whether users may opt calendars in to deleting events that ended a set number of days ago. Off by default; while off the option is hidden and saved choices are not run"
    ),
    def!(
        "retention.user.calendar.minDays",
        int(1, 3650),
        "30",
        GD,
        "privacy",
        "Fewest days after an event ended before a user rule may delete it; a smaller value is refused when saved and raised when it runs"
    ),
    def!(
        "retention.user.calendar.maxDays",
        int(1, 3650),
        "3650",
        GD,
        "privacy",
        "Most days a user may wait before ended events are deleted; a larger value is refused when saved and lowered when it runs"
    ),
    def!(
        "retention.user.notes.enabled",
        Kind::Bool,
        "false",
        GD,
        "privacy",
        "Whether users may opt in to removing notes not changed for a set number of days. Off by default; removed notes stay restorable for retention.user.undoDays"
    ),
    def!(
        "retention.user.notes.minDays",
        int(1, 3650),
        "30",
        GD,
        "privacy",
        "Fewest days a note must be untouched before a user rule may remove it; a smaller value is refused when saved and raised when it runs"
    ),
    def!(
        "retention.user.notes.maxDays",
        int(1, 3650),
        "3650",
        GD,
        "privacy",
        "Most days a user may wait before untouched notes are removed; a larger value is refused when saved and lowered when it runs"
    ),
    def!(
        "retention.class.short.days",
        int(1, 3650),
        "365",
        G,
        "privacy",
        "Ceiling in days for the short class (sessions, tokens, rate limits, jobs, metrics); applies when retention.class.enforce is on"
    ),
    def!(
        "retention.class.medium.days",
        int(1, 3650),
        "1095",
        G,
        "privacy",
        "Ceiling in days for the medium class (sign-in history, request protection, security notices); must not be below the short class"
    ),
    def!(
        "retention.class.long.days",
        int(0, 3650),
        "2555",
        G,
        "privacy",
        "Days the long class (security audit, administrator actions, requests, consent, tombstones) is kept, archive included; 0 keeps it forever and the console warns. Must not be below the medium class"
    ),
    def!(
        "retention.class.enforce",
        Kind::Bool,
        "false",
        G,
        "privacy",
        "Clamp each store's own limit to its class ceiling, and treat 0 (keep) as the ceiling. Off leaves the per-store limits as set"
    ),
    def!(
        "retention.archive.enabled",
        Kind::Bool,
        "false",
        G,
        "privacy",
        "Move aged rows of the long class into sealed archive files instead of deleting them"
    ),
    def!(
        "retention.archive.directory",
        text(Text::Plain, 500),
        "\"archive\"",
        G,
        "privacy",
        "Directory for archive files, absolute or relative to the server working directory; no parent-directory segments"
    ),
    def!(
        "retention.archive.batchRows",
        int(50, 10000),
        "1000",
        G,
        "privacy",
        "Rows per archive file"
    ),
    def!(
        "retention.archive.maxSegments",
        int(1, 1000),
        "10",
        G,
        "privacy",
        "Archive files one run may write per store"
    ),
    def!(
        "retention.archive.keyGeneration",
        int(1, 1000000),
        "1",
        G,
        "privacy",
        "Generation of the key that seals new archive files. Raise it to rotate; earlier files still verify with their own generation"
    ),
    def!(
        "retention.archive.verifyDays",
        int(1, 365),
        "7",
        G,
        "privacy",
        "Days between integrity checks of the archive"
    ),
    def!(
        "retention.archive.legalHold",
        Kind::Bool,
        "false",
        G,
        "privacy",
        "Keep every archive file, whatever the long class limit says"
    ),
    def!(
        "logging.file.enabled",
        Kind::Bool,
        "false",
        G,
        "observability",
        "Also write the server log to rotating files in logging.file.directory"
    ),
    def!(
        "logging.file.directory",
        text(Text::Plain, 500),
        "\"logs\"",
        G,
        "observability",
        "Directory for rotating log files, absolute or relative to the server working directory; no parent-directory segments"
    ),
    def!(
        "logging.file.maxSizeMb",
        int(1, 4096),
        "50",
        G,
        "observability",
        "Size at which the current log file is rotated, in megabytes"
    ),
    def!(
        "logging.file.rotateHours",
        int(0, 8760),
        "24",
        G,
        "observability",
        "Age at which the current log file is rotated, in hours; 0 rotates by size only"
    ),
    def!(
        "logging.file.maxAgeDays",
        int(1, 3650),
        "14",
        G,
        "observability",
        "Days rotated log files are kept"
    ),
    def!(
        "logging.file.maxFiles",
        int(1, 1000),
        "14",
        G,
        "observability",
        "Most rotated log files kept"
    ),
    def!(
        "logging.file.maxTotalMb",
        int(1, 102400),
        "500",
        G,
        "observability",
        "Most disk space all rotated log files may use, in megabytes"
    ),
    def!(
        "logging.file.compress",
        Kind::Bool,
        "true",
        G,
        "observability",
        "Compress rotated log files with gzip"
    ),
    def!(
        "logging.level",
        Kind::Enum(&["error", "warn", "info", "debug", "trace"]),
        "\"info\"",
        G,
        "observability",
        "Log level for every component without its own level"
    ),
    def!(
        "logging.componentLevels",
        Kind::StringList { max_len: 50, max_item: 80 },
        "[]",
        G,
        "observability",
        "Per-component log levels as component=level, for example mw_imap=debug"
    ),
    def!(
        "logging.redaction.addresses",
        Kind::Enum(&["full", "truncated", "hashed", "none"]),
        "\"full\"",
        G,
        "observability",
        "How email addresses appear in log lines: full, truncated (first letter and domain), hashed or none (removed)"
    ),
    def!(
        "logging.redaction.ips",
        Kind::Enum(&["full", "truncated", "hashed", "none"]),
        "\"full\"",
        G,
        "observability",
        "How IP addresses appear in log lines: full, truncated (IPv4 /24, IPv6 /48), hashed or none (removed)"
    ),
    def!(
        "logging.redaction.tokens",
        Kind::Bool,
        "true",
        G,
        "observability",
        "Cut bearer tokens, API keys and similar secrets out of log lines"
    ),
    // Cookie consent (cookies.*)
    def!(
        "cookies.enabled",
        Kind::Bool,
        "false",
        GD,
        "privacy",
        "Show the cookie consent control; off leaves behaviour as before"
    ),
    def!(
        "cookies.mode",
        Kind::Enum(&["optIn", "optOut", "noticeOnly"]),
        "\"optIn\"",
        GD,
        "privacy",
        "Consent model outside the strict regions: optIn (nothing optional until accepted), optOut (defaults apply until rejected), noticeOnly (information, no choice)"
    ),
    def!(
        "cookies.regionMode",
        Kind::Enum(&["optIn", "optOut", "noticeOnly", "same"]),
        "\"optIn\"",
        GD,
        "privacy",
        "Consent model in the strict regions; same uses cookies.mode there too"
    ),
    def!(
        "cookies.strictRegions",
        Kind::StringList { max_len: 80, max_item: 8 },
        "[\"AT\",\"BE\",\"BG\",\"HR\",\"CY\",\"CZ\",\"DK\",\"EE\",\"FI\",\"FR\",\"DE\",\"GR\",\"HU\",\"IE\",\"IT\",\"LV\",\"LT\",\"LU\",\"MT\",\"NL\",\"PL\",\"PT\",\"RO\",\"SK\",\"SI\",\"ES\",\"SE\",\"IS\",\"LI\",\"NO\",\"GB\",\"CH\"]",
        GD,
        "privacy",
        "ISO country codes that use cookies.regionMode"
    ),
    def!(
        "cookies.regionHeader",
        text(Text::Plain, 64),
        "\"X-Country-Code\"",
        GD,
        "privacy",
        "Request header (set by a proxy or CDN) holding the visitor country code; the Accept-Language region is the fallback"
    ),
    def!(
        "cookies.unknownRegion",
        Kind::Enum(&["strict", "normal"]),
        "\"strict\"",
        GD,
        "privacy",
        "Which model applies when no country can be determined"
    ),
    def!(
        "cookies.functional.enabled",
        Kind::Bool,
        "true",
        GD,
        "privacy",
        "Offer the functional category in the consent control"
    ),
    def!(
        "cookies.functional.default",
        Kind::Bool,
        "true",
        GD,
        "privacy",
        "Pre-selected state of the functional category when the model allows defaults"
    ),
    def!(
        "cookies.analytics.enabled",
        Kind::Bool,
        "false",
        GD,
        "privacy",
        "Offer the analytics category in the consent control"
    ),
    def!(
        "cookies.analytics.default",
        Kind::Bool,
        "false",
        GD,
        "privacy",
        "Pre-selected state of the analytics category when the model allows defaults"
    ),
    def!(
        "cookies.marketing.enabled",
        Kind::Bool,
        "false",
        GD,
        "privacy",
        "Offer the marketing category in the consent control"
    ),
    def!(
        "cookies.marketing.default",
        Kind::Bool,
        "false",
        GD,
        "privacy",
        "Pre-selected state of the marketing category when the model allows defaults"
    ),
    def!(
        "cookies.layout",
        Kind::Enum(&["banner", "card", "modal"]),
        "\"banner\"",
        GD,
        "privacy",
        "Consent control layout"
    ),
    def!(
        "cookies.position",
        Kind::Enum(&["bottom", "top", "start", "end", "center"]),
        "\"bottom\"",
        GD,
        "privacy",
        "Consent control position"
    ),
    def!(
        "cookies.rejectProminent",
        Kind::Bool,
        "true",
        GD,
        "privacy",
        "Show Reject all with the same prominence as Accept all"
    ),
    nullable(def!(
        "cookies.policyUrl",
        text(Text::HttpsUrl, 500),
        "null",
        GD,
        "privacy",
        "Cookie policy link"
    )),
    nullable(def!(
        "cookies.privacyUrl",
        text(Text::HttpsUrl, 500),
        "null",
        GD,
        "privacy",
        "Privacy policy link"
    )),
    def!(
        "cookies.policyVersion",
        int(1, 1000000),
        "1",
        GD,
        "privacy",
        "Policy version; raising it re-prompts when cookies.repromptOnVersion is on"
    ),
    def!(
        "cookies.lifetimeDays",
        int(1, 730),
        "180",
        GD,
        "privacy",
        "Days a consent decision stays valid"
    ),
    def!(
        "cookies.repromptOnVersion",
        Kind::Bool,
        "true",
        GD,
        "privacy",
        "Ask again when cookies.policyVersion changed since the stored decision"
    ),
    def!(
        "cookies.honourGpc",
        Kind::Bool,
        "true",
        GD,
        "privacy",
        "Treat a Global Privacy Control signal as rejecting analytics and marketing"
    ),
    def!(
        "cookies.honourDnt",
        Kind::Bool,
        "false",
        GD,
        "privacy",
        "Treat a Do Not Track signal as rejecting analytics and marketing"
    ),
    def!(
        "cookies.logConsent",
        Kind::Bool,
        "true",
        GD,
        "privacy",
        "Keep a server-side log of consent decisions"
    ),
    def!(
        "cookies.logRetentionDays",
        int(1, 3650),
        "365",
        GD,
        "privacy",
        "Days consent log rows are kept"
    ),
    def!(
        "cookies.scanIntervalHours",
        int(0, 720),
        "24",
        GD,
        "privacy",
        "Hours between automatic browser storage scans; 0 scans only on demand"
    ),
    def!(
        "cookies.unclassified",
        Kind::Enum(&["report", "functional", "analytics", "marketing"]),
        "\"report\"",
        GD,
        "privacy",
        "Handling of storage no inventory entry or rule classifies: report only, or treat as a category"
    ),
    def!(
        "cookies.text.title",
        Kind::LocaleMap { max: 1000 },
        "{}",
        GD,
        "privacy",
        "Consent control title per locale; empty uses the built-in text"
    ),
    def!(
        "cookies.text.body",
        Kind::LocaleMap { max: 1000 },
        "{}",
        GD,
        "privacy",
        "Consent control body per locale; empty uses the built-in text"
    ),
    def!(
        "cookies.label.necessary",
        Kind::LocaleMap { max: 120 },
        "{}",
        GD,
        "privacy",
        "Name of the necessary category per locale"
    ),
    def!(
        "cookies.description.necessary",
        Kind::LocaleMap { max: 600 },
        "{}",
        GD,
        "privacy",
        "Description of the necessary category per locale"
    ),
    def!(
        "cookies.label.functional",
        Kind::LocaleMap { max: 120 },
        "{}",
        GD,
        "privacy",
        "Name of the functional category per locale"
    ),
    def!(
        "cookies.description.functional",
        Kind::LocaleMap { max: 600 },
        "{}",
        GD,
        "privacy",
        "Description of the functional category per locale"
    ),
    def!(
        "cookies.label.analytics",
        Kind::LocaleMap { max: 120 },
        "{}",
        GD,
        "privacy",
        "Name of the analytics category per locale"
    ),
    def!(
        "cookies.description.analytics",
        Kind::LocaleMap { max: 600 },
        "{}",
        GD,
        "privacy",
        "Description of the analytics category per locale"
    ),
    def!(
        "cookies.label.marketing",
        Kind::LocaleMap { max: 120 },
        "{}",
        GD,
        "privacy",
        "Name of the marketing category per locale"
    ),
    def!(
        "cookies.description.marketing",
        Kind::LocaleMap { max: 600 },
        "{}",
        GD,
        "privacy",
        "Description of the marketing category per locale"
    ),
    // ── Encryption at rest ──────────────────────────────────────────────────────
    def!(
        "encryption.rotationDays",
        int(0, 3650),
        "365",
        G,
        "privacy",
        "Days after which the database key is reported as due for rotation; 0 sets no due date"
    ),
    def!(
        "encryption.warnings.enabled",
        Kind::Bool,
        "true",
        G,
        "privacy",
        "Report plaintext stores and overdue key rotation in the security assessment"
    ),

    // ── Composer features (layout agent): one Bool per composer control ───────
    def!(
        "compose.feature.*",
        Kind::Bool,
        "true",
        GDU,
        "interface",
        "Whether a composer feature (ribbon control) is available; id is the COMPOSE_FEATURES id"
    ),
    def!(
        "compose.styles.maxCustom",
        int(0, 100),
        "20",
        GDU,
        "interface",
        "Most custom paragraph and text styles a user can keep in the composer's Styles gallery; 0 turns custom styles off"
    ),
    def!(
        "compose.styles.defaultSet",
        Kind::Enum(&["default", "modern", "classic", "compact"]),
        r#""default""#,
        GDU,
        "interface",
        "Style set the composer's Styles gallery starts with"
    ),
    def!(
        "compose.styles.custom",
        text(Text::Plain, 20000),
        "\"\"",
        GDU,
        "interface",
        "A user's custom composer styles, as JSON written by the Styles gallery"
    ),
    def!(
        "mail.badge.headers",
        Kind::Bool,
        "true",
        GDU,
        "mail",
        "Show badges for Expires, Reply-By, Sensitivity and Supersedes in the reader"
    ),
    def!(
        "compose.layout.defaultFont",
        text(Text::Plain, 200),
        "\"\"",
        GDU,
        "mail",
        "Default message font: family|size px|#colour"
    ),
    def!(
        "compose.replyTo.show",
        Kind::Enum(&["auto", "always", "never"]),
        r#""auto""#,
        GDU,
        "mail",
        "Reply-To row in the composer"
    ),
    def!(
        "compose.replyTo.allowOverride",
        Kind::Bool,
        "true",
        GD,
        "mail",
        "Users may change the Reply-To header"
    ),
    def!(
        "compose.sentFolder.mode",
        Kind::Enum(&["default", "fixed", "byRecipient", "byIdentity", "bySubjectRule"]),
        r#""default""#,
        GDU,
        "mail",
        "Where the copy of a sent message is filed"
    ),
    def!(
        "compose.sentFolder.fixedId",
        text(Text::Plain, 200),
        "\"\"",
        GDU,
        "mail",
        "Folder role or id for the fixed sent folder"
    ),
    def!(
        "compose.sentFolder.perIdentity",
        Kind::StringList {
            max_len: 50,
            max_item: 300
        },
        "[]",
        GDU,
        "mail",
        "Sent folder per identity: identity=folder"
    ),
    def!(
        "compose.sentFolder.rules",
        Kind::StringList {
            max_len: 50,
            max_item: 300
        },
        "[]",
        GDU,
        "mail",
        "Sent folder rules: domain:example.com=folder or subject:text=folder"
    ),
    def!(
        "compose.sentFolder.skipCopy",
        Kind::Bool,
        "false",
        GDU,
        "mail",
        "Never save a copy of sent messages"

    ),
    def!(
        "compose.headers.allowAutoSubmitted",
        Kind::Bool,
        "false",
        GD,
        "mail",
        "Allow setting the Auto-Submitted header"
    ),
    def!(
        "compose.headers.allowThreadingEdit",
        Kind::Bool,
        "false",
        GD,
        "mail",
        "Allow editing In-Reply-To and References"
    ),
    def!(
        "compose.headers.allowCustom",
        Kind::Bool,
        "false",
        GD,
        "mail",
        "Allow custom message headers"
    ),
    def!(
        "compose.headers.allowReturnReceiptTo",
        Kind::Bool,
        "false",
        GD,
        "mail",
        "Allow the legacy Return-Receipt-To header"
    ),
    def!(
        "compose.headers.forced",
        Kind::StringList {
            max_len: 20,
            max_item: 300
        },
        "[]",
        GD,
        "mail",
        "Headers added to every sent message: Name=value"
    ),
    def!(
        "mail.expiry.hideExpired",
        Kind::Enum(&["off", "dim", "archive-suggestion", "hide"]),
        r#""off""#,
        GDU,
        "mail",
        "How expired messages (Expires header) are shown"
    ),

    def!(
        "compose.insert.maxInlineImageMB",
        int(1, 50),
        "10",
        GDU,
        "interface",
        "Largest picture, in MB, the composer places in the message body (larger ones can still be attached)"
    ),
    def!(
        "compose.insert.imageResizeDefault",
        Kind::Enum(&["original", "large", "medium"]),
        "\"large\"",
        GDU,
        "interface",
        "Size preset the composer's Picture from file dialog starts with"
    ),

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
                    if d.key == "compose.feature.*" {
                        // Feature ids: `[a-z][A-Za-z0-9.]*`, e.g. `layout.columns`.
                        return id.len() <= 64
                            && id.bytes().next().is_some_and(|b| b.is_ascii_lowercase())
                            && id.bytes().all(|b| b.is_ascii_alphanumeric() || b == b'.');
                    }
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
    if matches!(value, "inherit" | "disabled") {
        return true;
    }
    if value.len() > 64 || value.is_empty() {
        return false;
    }
    let (prefix, key) = if value == "+" {
        ("", "+")
    } else if let Some(prefix) = value.strip_suffix("++") {
        (prefix, "+")
    } else {
        value.rsplit_once('+').unwrap_or(("", value))
    };
    if !prefix.is_empty()
        && !prefix.split('+').all(|part| {
            matches!(
                part.trim().to_ascii_lowercase().as_str(),
                "ctrl" | "cmd" | "meta" | "mod" | "alt" | "option" | "shift"
            )
        })
    {
        return false;
    }
    (key.chars().count() == 1 && !key.chars().any(char::is_control))
        || matches!(
            key,
            "ArrowUp"
                | "ArrowDown"
                | "ArrowLeft"
                | "ArrowRight"
                | "Home"
                | "End"
                | "PageUp"
                | "PageDown"
                | "Delete"
                | "Del"
                | "Backspace"
                | "Insert"
                | "Escape"
                | "Esc"
                | "Tab"
                | "Enter"
                | "Space"
                | "F1"
                | "F2"
                | "F3"
                | "F4"
                | "F5"
                | "F6"
                | "F7"
                | "F8"
                | "F9"
                | "F10"
                | "F11"
                | "F12"
        )
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
    if def.key == "appearance.fontColor" {
        let Some(color) = value.as_str() else {
            return bad("expected an empty value or #RRGGBB color".into());
        };
        if !color.is_empty()
            && !(color.len() == 7
                && color.starts_with('#')
                && color[1..].bytes().all(|b| b.is_ascii_hexdigit()))
        {
            return bad("expected an empty value or #RRGGBB color".into());
        }
    }

    // t77: a config written before `requireRecoveryCodes` became `block` (an
    // import, a backup, a script) keeps working and is stored as `block`.
    if def.key == "twofa.lastFactorRemoval"
        && value.as_str() == Some(crate::twofa_routes::passkey_policy::LEGACY_BLOCK_ALIAS)
    {
        return Ok(json!("block"));
    }
    // t98: the allowed-values list of mail protocols a native account may use.
    if def.key == "access.protocols"
        && let Some(items) = value.as_array()
        && let Some(index) = items
            .iter()
            .position(|item| !item.as_str().is_some_and(|p| ACCESS_PROTOCOLS.contains(&p)))
    {
        return Err(Refusal::invalid(
            &format!("{field}/{index}"),
            format!("must be one of {}", ACCESS_PROTOCOLS.join(", ")),
        ));
    }
    if matches!(def.key, "time.displayZone" | "time.comparisonZone")
        && !value.as_str().is_some_and(|zone| {
            (def.key == "time.displayZone" && zone == "local")
                || (def.key == "time.comparisonZone" && zone.is_empty())
                || zone.parse::<chrono_tz::Tz>().is_ok()
        })
    {
        return bad("must be a valid IANA time zone; displayZone also accepts local, comparisonZone also accepts an empty value".into());
    }
    if def.key.starts_with("shortcuts.")
        && def.key != "shortcuts.enabled"
        && !value.as_str().is_some_and(valid_shortcut)
    {
        return bad(
            "must be inherit, disabled, or one key with Ctrl, Alt and Shift modifiers".into(),
        );
    }
    if matches!(
        def.key,
        "security.signin.geoCountries"
            | "security.trust.adminGeoCountries"
            | "security.signin.accountGeoAllowCountries"
            | "security.signin.accountGeoGlobalAllowCountries"
    ) && !value.as_array().is_some_and(|items| {
            items.iter().all(|item| {
                item.as_str().is_some_and(|code| {
                    code.len() == 2 && code.bytes().all(|b| b.is_ascii_uppercase())
                })
            })
        })
    {
        return bad("must contain uppercase two-letter country codes".into());
    }
    if def.key == "mail.cache.excludedFolders"
        && !value.as_array().is_some_and(|items| {
            items.iter().all(|item| {
                item.as_str().is_some_and(|v| {
                    v.strip_prefix("id:").is_some_and(|id| !id.is_empty())
                        || v.strip_prefix("role:").is_some_and(|role| {
                            matches!(
                                role,
                                "inbox" | "drafts" | "sent" | "trash" | "junk" | "archive"
                            )
                        })
                })
            })
        })
    {
        return bad("use id:<folder ID> or role:inbox/drafts/sent/trash/junk/archive".into());
    }
    match def.kind {
        Kind::MessageTemplates => {
            let Some(items) = value.as_array() else {
                return bad("must be a template list".into());
            };
            if items.len() > 20 || value.to_string().len() > 65536 {
                return bad("template list exceeds its size limit".into());
            }
            let mut ids = std::collections::HashSet::new();
            for (index, item) in items.iter().enumerate() {
                let invalid = || {
                    Refusal::invalid(
                        &format!("{field}/{index}"),
                        "expected unique id, plain title and plain text body within their limits",
                    )
                };
                let object = item.as_object().ok_or_else(invalid)?;
                if object.len() != 3
                    || !object.contains_key("id")
                    || !object.contains_key("title")
                    || !object.contains_key("body")
                {
                    return Err(invalid());
                }
                let id = object["id"].as_str().ok_or_else(invalid)?;
                let title = object["title"].as_str().ok_or_else(invalid)?;
                let body = object["body"].as_str().ok_or_else(invalid)?;
                if id.is_empty()
                    || id.len() > 64
                    || !id
                        .bytes()
                        .all(|b| b.is_ascii_alphanumeric() || b == b'_' || b == b'-')
                    || !ids.insert(id)
                    || title.trim().is_empty()
                    || title.chars().count() > 80
                    || title.chars().any(char::is_control)
                    || body.trim().is_empty()
                    || body.chars().count() > 8000
                    || body
                        .chars()
                        .any(|c| c.is_control() && !matches!(c, '\n' | '\r' | '\t'))
                {
                    return Err(invalid());
                }
            }
            Ok(value.clone())
        }
        Kind::FolderRole => mw_engine::folder_roles::parse_target(value)
            .map(|_| value.clone())
            .map_err(|detail| Refusal::invalid(field, detail)),
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
            let Some(items) = value.as_array() else {
                return bad("must be a list of attachment rules".into());
            };
            if items.len() > 100 {
                return bad("at most 100 attachment rules are allowed".into());
            }
            let mut seen = std::collections::BTreeSet::new();
            for item in items {
                let Some(rule) = item.as_str() else {
                    return bad("attachment rules must be text".into());
                };
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
                        if matches!(
                            def.key,
                            "security.signin.geoCountries"
                                | "security.trust.adminGeoCountries"
                                | "security.signin.accountGeoAllowCountries"
                                | "security.signin.accountGeoGlobalAllowCountries"
                                | "cookies.strictRegions"
                        ) {
                            b.is_ascii_uppercase()
                        } else if matches!(
                            def.key,
                            "security.signin.accountGeoAllowIps"
                                | "security.signin.accountGeoGlobalAllowIps"
                        ) {
                            !b.is_ascii_control()
                        } else if def.key == "links.trustedSenders" {
                            !b.is_ascii_control()
                        } else if matches!(
                            def.key,
                            "mail.cache.excludedFolders"
                                | "retention.user.mandatory"
                                | "retention.user.protectedFolders"
                                | "security.protection.exemptClients"
                                | "security.protection.exemptAccounts"
                                | "security.protection.exemptAdmins"
                                | "security.protection.userAgentPatterns"
                                | "security.protection.userAgentExceptions"
                                | "security.adminProbe.allowlist"
                                | "security.health.networks"
                                | "security.protocol.allowedMethods"
                                | "security.protocol.allowedHosts"
                                | "security.protocol.exemptPrefixes"
                        ) {
                            !b.is_ascii_control()
                        } else if matches!(def.key, "mail.replyFrom.headerPriority" | "mail.priority.sendHeaders" | "properties.folder.overrides.allowed") {
                            // Header names: Delivered-To, X-Original-To, X-MSMail-Priority, ...
                            b.is_ascii_alphanumeric() || b == b'-'
                        } else if def.key == "security.adminProbe.probePaths" {
                            b.is_ascii_alphanumeric() || b"/._~-%+@:".contains(&b)
                        } else {
                            b.is_ascii_lowercase() || b.is_ascii_digit() || matches!(b, b'-' | b'.')
                        }
                    })
                    || (matches!(
                        def.key,
                        "security.protection.exemptClients"
                            | "security.adminProbe.allowlist"
                            | "security.health.networks"
                    ) && crate::scope_mw::proxy::Cidr::parse(text).is_none())
                    || (def.key == "retention.user.mandatory"
                        && crate::retention_rules::parse_mandatory(&[text.to_string()]).is_empty())
                    || (def.key == "retention.user.templates"
                        && crate::retention_rules::template(text).is_none())
                    || (def.key == "security.adminProbe.probePaths"
                        && (!text.starts_with('/') || text == "/" || text.contains("//")))
                    || (def.key == "security.protocol.exemptPrefixes"
                        && (!text.starts_with('/') || text == "/" || text.contains(' ')))
                    || (def.key == "security.protocol.allowedMethods"
                        && !text.bytes().all(|b| b.is_ascii_uppercase() || b == b'-'))
                    || (def.key == "security.protocol.allowedHosts" && text.contains(' '))
                    || (def.key == "security.adminProbe.signals"
                        && !matches!(
                            text,
                            "probe-path"
                                | "slug-guess"
                                | "admin-signin-failed"
                                | "admin-unauthenticated"
                        ))
                    || (matches!(
                        def.key,
                        "security.signin.accountGeoAllowIps"
                            | "security.signin.accountGeoGlobalAllowIps"
                    ) && crate::account_signin_geo::parse_ip_entry(text).is_none())
                    || (matches!(
                        def.key,
                        "security.signin.accountGeoAllowCountries"
                            | "security.signin.accountGeoGlobalAllowCountries"
                            | "security.trust.adminGeoCountries"
                    ) && !crate::account_signin_geo::valid_country(text))
                    || (matches!(
                        def.key,
                        "links.trustedDomains" | "images.trustedHosts" | "images.blockedHosts"
                    ) && !link_trust::is_canonical(text))
                    || (def.key == "links.trustedSenders" && !link_trust::is_canonical_sender(text))
                    || (def.key == "security.protection.userAgentCategories"
                        && !crate::request_protection::user_agent::CATEGORIES.contains(&text))
                    || (def.key == "security.protection.signals"
                        && !matches!(
                            text,
                            "malformed"
                                | "oversized"
                                | "credential-rejected"
                                | "permission-denied"
                                | "body-deadline"
                                | "admin-route-miss"
                        ))
                    || !seen.insert(text)
                {
                    return Err(Refusal::invalid(
                        &format!("{field}/{index}"),
                        "must be a unique value using the permitted characters",
                    ));
                }
            }
            if matches!(
                def.key,
                "security.protection.userAgentPatterns" | "security.protection.userAgentExceptions"
            ) && !crate::request_protection::user_agent::valid_patterns(value)
            {
                return bad("must contain bounded valid Rust regular expressions".into());
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

    #[test]
    fn mail_cache_policy_is_bounded_and_speculation_opt_in() {
        for key in [
            "keepIndefinitely",
            "enabled",
            "mailboxes",
            "messages",
            "freshSeconds",
            "maxAgeSeconds",
            "maxEntries",
            "maxBytes",
            "prefetchEnabled",
            "hoverEnabled",
            "focusEnabled",
            "dwellMs",
            "idleEnabled",
            "idleDelayMs",
            "maxConcurrency",
            "maxQueued",
            "preloadMessages",
            "excludedFolders",
        ] {
            let def = lookup(&format!("mail.cache.{key}")).unwrap();
            assert_eq!(def.scopes, Scopes::GlobalDomainUser);
            assert!(validate(def, &def.default_value(), "/value").is_ok());
        }
        assert_eq!(lookup("mail.cache.keepIndefinitely").unwrap().default_value(), json!(false));
        assert_eq!(lookup("mail.conversations.enabled").unwrap().default_value(), json!(false));
        assert_eq!(lookup("mail.conversations.enabled").unwrap().scopes, Scopes::GlobalDomainUser);
        assert_eq!(lookup("mail.conversations.allowed").unwrap().default_value(), json!(true));
        assert_eq!(lookup("mail.conversations.allowed").unwrap().scopes, Scopes::GlobalDomain);
        assert!(lookup("mail.conversations.allowed").unwrap().user_visible());
        assert_eq!(
            lookup("mail.cache.prefetchEnabled")
                .unwrap()
                .default_value(),
            json!(false)
        );
        assert_eq!(
            lookup("mail.cache.preloadMessages")
                .unwrap()
                .default_value(),
            json!(false)
        );
        assert!(
            validate(
                lookup("mail.cache.maxConcurrency").unwrap(),
                &json!(5),
                "/value"
            )
            .is_err()
        );
        assert!(
            validate(
                lookup("mail.cache.excludedFolders").unwrap(),
                &json!(["trash"]),
                "/value"
            )
            .is_err()
        );
        assert!(
            validate(
                lookup("mail.cache.excludedFolders").unwrap(),
                &json!(["id:abc", "role:trash"]),
                "/value"
            )
            .is_ok()
        );
    }
    #[test]
    fn message_templates_preserve_plaintext_and_reject_unbounded_or_ambiguous_input() {
        let def = lookup("mail.templates.personal").unwrap();
        let value = json!([{"id":"reply_1","title":"Resposta","body":"  Hello\r\n\t<literal>  "}]);
        assert_eq!(validate(def, &value, "/value").unwrap(), value);
        for bad in [
            json!([{"id":"a","title":"t","body":"b","html":true}]),
            json!([{"id":"a","title":"t","body":"b"},{"id":"a","title":"t","body":"b"}]),
            json!([{"id":"a","title":"t","body":"\u{0000}"}]),
            json!([{"id":"a","title":"t","body":"x".repeat(8001)}]),
            json!([{"id":"a","title":" ","body":"b"}]),
        ] {
            assert!(validate(def, &bad, "/value").is_err());
        }
        let oversized: Vec<_> = (0..20)
            .map(|n| json!({"id":format!("a{n}"),"title":"t","body":"é".repeat(4000)}))
            .collect();
        assert!(validate(def, &json!(oversized), "/value").is_err());
        assert!(
            !lookup("mail.templates.shared")
                .unwrap()
                .scopes
                .allows(SettingScope::User)
        );
        assert!(
            !lookup("mail.templates.allowPersonal")
                .unwrap()
                .scopes
                .allows(SettingScope::User)
        );
    }

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
        for (key, default) in [
            ("login.rememberSignins", "true"),
            ("login.showRememberedHint", "false"),
        ] {
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
    fn account_help_notice_is_admin_scoped_and_hidden_by_default() {
        let def = lookup("login.showAccountHelpNotice").unwrap();
        assert_eq!(def.default_value(), json!(false));
        assert_eq!(def.scopes, Scopes::GlobalDomain);
        assert!(!def.user_override);
        assert!(validate(def, &json!(true), "/value").is_ok());
        assert!(validate(def, &json!("true"), "/value").is_err());
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
            "ui.notifyNewVersion",
            "ui.bar.mailTopBar",
            "ui.bar.workspaceTabs",
            "ui.bar.mailRibbon",
            "ui.bar.composerRibbon",
            "appearance.theme",
            "appearance.accent",
            "appearance.badgeColor",
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
        assert!(lookup("compose.feature.layout.columns").is_some());
        assert!(lookup("compose.feature.options.replyTo").is_some());
        assert!(lookup("compose.styles.maxCustom").is_some());
        assert!(lookup("compose.styles.defaultSet").is_some());
        assert!(lookup("compose.styles.custom").is_some());
        assert!(lookup("compose.feature.Layout").is_none());
        assert!(lookup("compose.feature.").is_none());
        assert!(lookup("compose.feature.a-b").is_none());
        assert!(lookup("compose.feature.*").is_none());
    }

    #[test]
    fn signin_keys_name_a_dedicated_route_and_limits_are_per_address() {
        for d in REGISTRY.iter().filter(|d| {
            d.key.starts_with("signin.")
                && !matches!(
                    d.key,
                    "signin.retryCooldownSecs"
                        | "signin.antiBotMode"
                        | "signin.antiBotDifficulty"
                        | "signin.antiBotExpirySecs"
                )
        }) {
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

    /// Every image-cache tunable is a registered, global, bounded key in the one
    /// `imageProxy` section, documented, labelled in the console and in the en and
    /// pt-PT admin catalogs, and read into the policy; a new cache key that is not
    /// listed in `IMAGE_CACHE_KEYS` fails the sweep at the bottom.
    #[test]
    fn every_image_cache_tunable_is_keyed_documented_and_in_one_section() {
        use crate::remote_policy::{IMAGE_CACHE_KEYS, POLICY_KEYS};
        let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
        let read = |p: &str| std::fs::read_to_string(root.join(p)).unwrap();
        let docs = read("docs/admin/settings.md");
        let keys_ts = read("apps/web/src/screens/Admin/Settings/keys.ts");
        let ftl = [
            read("apps/web/locales/en/admin-settings.ftl"),
            read("apps/web/locales/pt-PT/admin-settings.ftl"),
        ];
        let slug = |key: &str| {
            let mut out = String::new();
            let mut prev = ' ';
            for c in key.chars() {
                if c.is_ascii_uppercase() && (prev.is_ascii_lowercase() || prev.is_ascii_digit()) {
                    out.push('-');
                }
                out.push(if c == '.' || c == '_' { '-' } else { c.to_ascii_lowercase() });
                prev = c;
            }
            out
        };
        let mut problems = Vec::new();
        for key in IMAGE_CACHE_KEYS {
            let Some(d) = lookup(key) else {
                problems.push(format!("{key}: not registered"));
                continue;
            };
            if d.category != "imageProxy" {
                problems.push(format!("{key}: category {}", d.category));
            }
            if d.scopes != Scopes::Global || d.user_override {
                problems.push(format!("{key}: must be global only"));
            }
            if !docs.contains(&format!("<a id=\"{key}\"></a>")) {
                problems.push(format!("{key}: no docs anchor"));
            }
            if !keys_ts.contains(&format!("'{key}'")) {
                problems.push(format!("{key}: not in keys.ts"));
            }
            if !POLICY_KEYS.contains(key) {
                problems.push(format!("{key}: not read into the policy"));
            }
            for (i, catalog) in ftl.iter().enumerate() {
                for prefix in ["admin-settings-key-", "admin-settings-help-"] {
                    if !catalog.contains(&format!("\n{prefix}{} = ", slug(key))) {
                        problems.push(format!("{key}: no {prefix} message in catalog {i}"));
                    }
                }
            }
            // Bounds are real and the default sits inside them.
            if let Kind::Int { min, max } = d.kind {
                let default = d.default_value();
                if min >= max || validate(d, &default, "/value").is_err() {
                    problems.push(format!("{key}: default {default} outside {min}..={max}"));
                }
                if validate(d, &json!(min - 1), "/value").is_ok()
                    || validate(d, &json!(max + 1), "/value").is_ok()
                {
                    problems.push(format!("{key}: bounds not enforced"));
                }
            }
        }
        // Sweep: nothing cache-shaped is registered without being listed.
        for d in REGISTRY {
            let cacheish = d.key.starts_with("privacy.cache")
                || d.key.starts_with("privacy.imageCache")
                || d.key.starts_with("privacy.imageDisk")
                || d.key.starts_with("privacy.imageRam")
                || d.key.starts_with("privacy.imageTtl")
                || d.key.starts_with("privacy.imageScore")
                || d.key.starts_with("privacy.imageContext")
                || d.key.starts_with("privacy.imageSweep")
                || d.key.starts_with("privacy.imageSnapshot")
                || d.key.starts_with("privacy.imageAccount")
                || d.key == "privacy.imageMaxAgeSecs";
            if cacheish && !IMAGE_CACHE_KEYS.contains(&d.key) {
                problems.push(format!("{}: cache key missing from IMAGE_CACHE_KEYS", d.key));
            }
            if d.category == "imageProxy"
                && !IMAGE_CACHE_KEYS.contains(&d.key)
                && !d.key.starts_with("privacy.imageRate")
                && !d.key.starts_with("privacy.imageResize")
                && !matches!(
                    d.key,
                    "privacy.ratePerMin" | "privacy.rateBurst" | "privacy.imageLatencyTracking"
                )
            {
                problems.push(format!("{}: in the Image proxy section but unlisted", d.key));
            }
        }
        // The defaults the registry declares are the defaults the tier uses.
        let budget = crate::image_proxy::adaptive::CacheBudget {
            bytes: 32 << 20,
            entries: 1024,
            entry_bytes: 4 << 20,
            ttl_secs: 3600,
        };
        let from_registry = crate::remote_policy::tier_config(
            &|key| lookup(key).map(|d| d.default_value()),
            &budget,
        );
        let mut expected = crate::image_proxy::tier::TierConfig::default();
        expected.disk.path = from_registry.disk.path.clone();
        assert_eq!(from_registry, expected, "registry defaults match the code defaults");
        assert!(problems.is_empty(), "{problems:#?}");
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
    fn message_order_and_group_choices_are_user_and_admin_configurable() {
        for (key, values) in [
            ("mail.messageListOrder", &["newest", "oldest", "sender", "sender-desc", "subject", "subject-desc", "sent-newest", "sent-oldest", "importance", "importance-asc", "size", "size-asc"][..]),
            ("mail.messageListGroup", &["none", "received", "sent", "sender", "subject", "importance", "size"][..]),
        ] {
            let definition = REGISTRY.iter().find(|definition| definition.key == key).unwrap();
            assert!(matches!(definition.scopes, Scopes::GlobalDomainUser));
            assert!(definition.user_visible());
            for value in values { assert!(validate(definition, &serde_json::json!(value), "/value").is_ok()); }
            assert!(validate(definition, &serde_json::json!("arbitrary"), "/value").is_err());
        }
    }

    /// The cookie consent keys are admin-only (global and per domain, with lock and
    /// default through the generic settings routes), off by default, and their
    /// defaults describe a conservative configuration.
    #[test]
    fn cookie_consent_keys_are_admin_scoped_and_default_conservative() {
        let keys: Vec<_> = REGISTRY
            .iter()
            .filter(|d| d.key.starts_with("cookies."))
            .collect();
        assert!(keys.len() >= 30, "{}", keys.len());
        for d in &keys {
            assert_eq!(d.category, "privacy", "{}", d.key);
            assert!(d.scopes.allows(SettingScope::Domain), "{}", d.key);
            assert!(!d.scopes.allows(SettingScope::User), "{}", d.key);
            assert!(!d.user_override, "{}", d.key);
            assert!(d.dedicated_route.is_none(), "{}", d.key);
        }
        let default = |k: &str| lookup(k).unwrap().default_value();
        assert_eq!(default("cookies.enabled"), json!(false));
        assert_eq!(default("cookies.mode"), json!("optIn"));
        assert_eq!(default("cookies.regionMode"), json!("optIn"));
        assert_eq!(default("cookies.unknownRegion"), json!("strict"));
        for c in ["analytics", "marketing"] {
            assert_eq!(default(&format!("cookies.{c}.enabled")), json!(false), "{c}");
            assert_eq!(default(&format!("cookies.{c}.default")), json!(false), "{c}");
        }
        assert_eq!(default("cookies.rejectProminent"), json!(true));
        assert_eq!(default("cookies.honourGpc"), json!(true));
        assert_eq!(default("cookies.policyUrl"), Value::Null);
        let regions = default("cookies.strictRegions");
        let regions: Vec<&str> = regions.as_array().unwrap().iter().filter_map(Value::as_str).collect();
        for code in ["DE", "PT", "GB", "CH", "IS"] {
            assert!(regions.contains(&code), "{code}");
        }
        assert!(regions.iter().all(|c| c.len() == 2 && c.chars().all(|x| x.is_ascii_uppercase())));
        // Bounds: a lifetime of zero or a version below one is refused.
        let bad = |k: &str, v: Value| validate(lookup(k).unwrap(), &v, "/value").is_err();
        assert!(bad("cookies.lifetimeDays", json!(0)));
        assert!(bad("cookies.lifetimeDays", json!(731)));
        assert!(bad("cookies.policyVersion", json!(0)));
        assert!(bad("cookies.mode", json!("always")));
        assert!(bad("cookies.policyUrl", json!("http://example.org/p")));
        assert!(!bad("cookies.policyUrl", json!("https://example.org/p")));
        assert!(!bad("cookies.text.title", json!({"en": "Cookies", "pt-PT": "Cookies"})));
        assert!(bad("cookies.text.title", json!({"xx": "Cookies"})));
    }

    /// t39-e6: the passkey policy keys are global/domain enums with locks, not
    /// user-settable, and their defaults keep the behaviour before them.
    #[test]
    fn passkey_policy_keys_are_scoped_enums_with_safe_defaults() {
        for (key, default, good, bad) in [
            ("twofa.passkeyAttachment", "any", "platform", "roaming"),
            (
                "twofa.passkeyUserVerification",
                "preferred",
                "required",
                "always",
            ),
            ("twofa.passkeyResidentKey", "discouraged", "required", "yes"),
            ("twofa.passkeyLegacy", "allow", "refuse", "lockout"),
            ("twofa.lastFactorRemoval", "allow", "block", "never"),
        ] {
            let d = lookup(key).unwrap_or_else(|| panic!("{key} is registered"));
            assert_eq!(d.default_value(), json!(default), "{key}");
            assert_eq!(d.category, "signin", "{key}");
            assert!(d.scopes.allows(SettingScope::Domain), "{key}");
            assert!(!d.scopes.allows(SettingScope::User), "{key}");
            assert!(!d.user_override, "{key}");
            assert_eq!(validate(d, &json!(good), "/value").unwrap(), json!(good));
            assert!(validate(d, &json!(bad), "/value").is_err(), "{key}");
        }
        // t77: the t39-e6 value is accepted and stored as `block`.
        let d = lookup("twofa.lastFactorRemoval").unwrap();
        assert_eq!(
            validate(d, &json!("requireRecoveryCodes"), "/value").unwrap(),
            json!("block")
        );
        assert_eq!(d.kind.describe()["values"], json!(["allow", "block"]));
    }

    /// t39-e7: the quota warning policy reaches the user's settings map, read
    /// only: no user scope, so a user write is refused.
    #[test]
    fn quota_warning_policy_is_visible_to_users_but_not_theirs_to_set() {
        for key in ["quota.warnEnabled", "quota.warnThresholds"] {
            let d = lookup(key).unwrap_or_else(|| panic!("{key} is registered"));
            assert!(d.user_visible(), "{key}");
            assert!(!d.scopes.allows(SettingScope::User), "{key}");
        }
    }

    /// t56: the remote-image fetch limits are global, admin-only integers with
    /// the documented defaults and bounds; out-of-range values are refused, and
    /// the defaults are exactly what the fetch uses when nothing is set.
    #[test]
    fn remote_image_fetch_limits_are_bounded_global_integers() {
        let expected: [(&str, i64, i64, i64); 13] = [
            ("privacy.fetchConnectTimeoutMs", 500, 30000, 5000),
            ("privacy.fetchHeadersTimeoutMs", 500, 60000, 10000),
            ("privacy.fetchIdleTimeoutMs", 500, 60000, 10000),
            ("privacy.fetchTotalTimeoutMs", 1000, 120000, 20000),
            ("privacy.fetchQueueTimeoutMs", 1000, 180000, 60000),
            ("privacy.imageDecodeTimeoutMs", 1000, 60000, 10000),
            ("privacy.maxRedirects", 0, 10, 3),
            ("privacy.maxConcurrentGlobal", 1, 256, 128),
            ("privacy.maxConcurrentPipeline", 1, 256, 128),
            ("privacy.maxConcurrentDecodes", 1, 16, 10),
            ("privacy.maxQueuedRequests", 16, 16384, 2048),
            ("privacy.maxConcurrentPerMessage", 1, 128, 30),
            ("privacy.maxConcurrentPerAccount", 1, 256, 128),
        ];
        assert_eq!(
            expected.map(|(k, ..)| k),
            crate::remote_policy::FETCH_KEYS,
            "the fetch reads exactly these keys"
        );
        for (key, min, max, default) in expected {
            let d = lookup(key).unwrap_or_else(|| panic!("{key} registered"));
            assert_eq!(d.category, "privacy", "{key}");
            assert_eq!(d.default_value(), json!(default), "{key}");
            // Global only, like privacy.maxBytes / cacheBytes / ratePerMin.
            assert!(d.scopes == Scopes::Global, "{key}");
            for scope in SettingScope::ALL {
                assert!(!d.scopes.allows(scope), "{key} {scope:?}");
            }
            assert!(!d.user_override, "{key}");
            assert!(!d.user_visible(), "{key}");
            for ok in [min, max, default] {
                assert_eq!(
                    validate(d, &json!(ok), "/value").unwrap(),
                    json!(ok),
                    "{key} {ok}"
                );
            }
            for bad in [
                json!(min - 1),
                json!(max + 1),
                json!("5000"),
                json!(1.5),
                Value::Null,
            ] {
                let r = validate(d, &bad, "/value").unwrap_err();
                assert_eq!((r.status, r.code), (422, "invalidValue"), "{key} {bad}");
            }
        }
        let defaults = crate::remote_policy::fetch_limits(|key| {
            lookup(key).and_then(|d| d.default_value().as_u64())
        });
        assert_eq!(
            defaults,
            crate::image_proxy::remote_content::FetchLimits::default()
        );
    }

    /// t55: the loading-images notification is an on/off presentation key,
    /// on by default, settable per domain and per user, lockable by admins.
    #[test]
    fn remote_images_toast_is_a_user_overridable_bool_defaulting_on() {
        let d = lookup("privacy.remoteImagesToast").expect("registered");
        assert_eq!(d.default_value(), json!(true));
        assert_eq!(d.category, "privacy");
        assert!(d.user_override);
        assert!(d.user_visible());
        for scope in [SettingScope::Domain, SettingScope::User] {
            assert!(d.scopes.allows(scope), "{scope:?}");
        }
        assert!(refuse_if_dedicated(d).is_ok());
        assert_eq!(validate(d, &json!(false), "/value").unwrap(), json!(false));
        for bad in [json!("off"), json!(1), Value::Null] {
            let r = validate(d, &bad, "/value").unwrap_err();
            assert_eq!((r.status, r.code), (422, "invalidValue"), "{bad}");
        }
    }

    /// t38: the four collapsible bars start pinned, take global, domain and user
    /// values (so an administrator can change or lock the default) and accept
    /// only `pinned` or `auto`.
    #[test]
    fn bar_modes_default_to_pinned_and_accept_only_pinned_or_auto() {
        for key in [
            "ui.bar.mailTopBar",
            "ui.bar.workspaceTabs",
            "ui.bar.mailRibbon",
            "ui.bar.composerRibbon",
        ] {
            let d = lookup(key).unwrap_or_else(|| panic!("{key} is registered"));
            assert_eq!(d.default_value(), json!("pinned"), "{key}");
            assert_eq!(d.category, "interface", "{key}");
            assert!(d.user_override, "{key}");
            assert!(d.user_visible(), "{key}");
            for scope in [SettingScope::Domain, SettingScope::User] {
                assert!(d.scopes.allows(scope), "{key} {scope:?}");
            }
            assert!(refuse_if_dedicated(d).is_ok(), "{key}");
            assert_eq!(
                validate(d, &json!("auto"), "/value").unwrap(),
                json!("auto")
            );
            assert_eq!(
                validate(d, &json!("pinned"), "/value").unwrap(),
                json!("pinned")
            );
            for bad in [json!("hidden"), json!(true), Value::Null, json!("")] {
                let r = validate(d, &bad, "/value").unwrap_err();
                assert_eq!((r.status, r.code), (422, "invalidValue"), "{key} {bad}");
            }
        }
    }

    #[test]
    fn appearance_default_is_dark() {
        assert_eq!(
            lookup("appearance.theme").unwrap().default_value(),
            json!("dark")
        );
    }

    #[test]
    fn appearance_font_accepts_manrope() {
        let font = lookup("appearance.font").unwrap();
        assert_eq!(validate(font, &json!("manrope"), "/value").unwrap(), json!("manrope"));
        assert!(validate(font, &json!("unsupported-font"), "/value").is_err());
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
        assert!(validate(d, &json!("pt-PT"), "/v").is_ok());
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
        let subject = SettingSubject {
            personal_owner: None,
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
        let subject = SettingSubject {
            personal_owner: None,
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
mod focused_inbox_tests {
    use super::*;

    /// t73: the Focused inbox is an off-by-default boolean the user can change,
    /// with a deployment and a domain default and lock, in the mail category.
    #[test]
    fn focused_inbox_is_an_off_by_default_lockable_user_boolean() {
        let d = lookup("mail.focusedInbox").expect("registered");
        assert!(matches!(d.kind, Kind::Bool));
        assert_eq!(d.default, "false");
        assert_eq!(d.default_value(), json!(false));
        assert_eq!(d.category, "mail");
        assert!(d.user_visible(), "reaches GET /api/settings");
        assert!(d.user_override, "a user value is allowed unless locked");
        assert!(
            d.scopes.allows(SettingScope::Domain),
            "domain default and lock"
        );
        assert!(d.scopes.allows(SettingScope::User), "user value");
        assert!(d.dedicated_route.is_none());
        assert!(validate(d, &json!(true), "/value").is_ok());
        assert!(validate(d, &json!("true"), "/value").is_err());
        assert!(validate(d, &json!(1), "/value").is_err());
    }
}

#[cfg(test)]
mod shortcut_validation_tests {
    use super::*;
    #[test]
    fn shortcut_chords_are_bounded_and_match_supported_browser_syntax() {
        for value in [
            "inherit",
            "disabled",
            "Ctrl+Shift+K",
            "Cmd+ArrowDown",
            "Space",
            "Ctrl++",
            "+",
            "↓",
        ] {
            assert!(valid_shortcut(value), "{value}");
        }
        for value in ["", "Hyper+x", "UnknownKey", "Ctrl+", "Ctrl+Shift+", "\n"] {
            assert!(!valid_shortcut(value), "{value:?}");
        }
        assert!(!valid_shortcut(&"x".repeat(65)));
    }
}

#[cfg(test)]
mod time_zone_validation_tests {
    use super::*;
    #[test]
    fn accepts_only_supported_zones_and_explicit_sentinels() {
        let display = REGISTRY
            .iter()
            .find(|def| def.key == "time.displayZone")
            .unwrap();
        let comparison = REGISTRY
            .iter()
            .find(|def| def.key == "time.comparisonZone")
            .unwrap();
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
        let def = REGISTRY
            .iter()
            .find(|def| def.key == "security.signin.geoCountries")
            .unwrap();
        assert!(validate(def, &serde_json::json!(["PT", "GB"]), "/value").is_ok());
        for value in [
            serde_json::json!(["pt"]),
            serde_json::json!(["*"]),
            serde_json::json!(["USA"]),
            serde_json::json!(["1A"]),
            serde_json::json!("PT"),
            serde_json::json!([null]),
            serde_json::json!(["PT", "PT"]),
        ] {
            assert!(validate(def, &value, "/value").is_err(), "{value}");
        }
        assert!(validate(def, &serde_json::json!(vec!["PT"; 250]), "/value").is_err());
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
            for value in [
                json!(null),
                json!({"id":"owned"}),
                json!({"path":"INBOX/Deleted Items"}),
            ] {
                assert!(validate(def, &value, "/value").is_ok(), "{key}: {value}");
            }
            for value in [
                json!("Trash"),
                json!({}),
                json!({"id":"a","path":"Trash"}),
                json!({"path":"Trash\r\nINBOX"}),
            ] {
                assert!(validate(def, &value, "/value").is_err(), "{key}: {value}");
            }
        }
    }
}
