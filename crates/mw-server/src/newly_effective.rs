//! Console settings that the server now applies (t48), and the readers for them.
//!
//! Before t48 a set of registry keys could be saved in the console but nothing
//! read them, or only an environment variable was read. This module holds:
//!
//! - small readers ([`int`], [`text`], [`flag`]) that resolve a key through the
//!   settings registry, so an environment variable with `Pin` precedence still
//!   wins and a console value applies when it is unset. Numbers are clamped to
//!   the registry's range, so a stored `0` or an out-of-range value cannot turn
//!   a limit off.
//! - [`report_once`]: on the first start after the upgrade it writes ONE audit
//!   entry (`settings.now-applied`) listing the saved non-default values of the
//!   keys that now take effect, so an administrator can see what changed. A
//!   marker row makes it run once per database; a server with nothing saved
//!   sets the marker and writes no entry.
//! - the cached proxy timeout (`upstream.proxyTimeoutSecs`), read on the JMAP
//!   proxy hot path without a database query.

use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Duration;

use mw_store::settings_policy::{SettingPolicyRow, SettingScope, SettingSubject};
use mw_store::{Store, StoreError};
use serde_json::{Value, json};

use crate::settings_registry::{self, Kind};

/// The audit action of the first-start entry.
pub(crate) const AUDIT_ACTION: &str = "settings.now-applied";
/// The `setting_policy` row that records the entry was considered. It is not a
/// registry key, so no settings route lists or edits it.
pub(crate) const MARKER_KEY: &str = "upgrade.t48.settingsNowApplied";

/// Keys that a console value now changes (the WIRE keys of the t48 plan).
pub(crate) const NOW_APPLIED: &[&str] = &[
    "passwd.policy.minLength",
    "passwd.policy.requireUpper",
    "passwd.policy.requireLower",
    "passwd.policy.requireDigit",
    "passwd.policy.requireSymbol",
    "limit.send_per_hour",
    "limit.send_per_day",
    "limit.storage_bytes",
    "limit.ai_calls_per_day",
    "assistant.enabled",
    "assistant.rateLimitPerMin",
    "usage.retentionDays",
    "import.maxMessages",
    "import.maxMessageBytes",
    "webhooks.maxAttempts",
    "webhooks.timeoutSecs",
    "outbox.maxAttempts",
    "upstream.proxyTimeoutSecs",
    "jobs.historyDays",
    "health.diskFreeMinPercent",
    "about.securityContact",
    "about.securityPolicyUrl",
    "views.attachments.name",
    "views.outbox.name",
    "views.attachments.showCount",
    "views.outbox.showCount",
    "views.templates.showCount",
    "signin.ldapBindPrecheck",
    "plugins.kv.maxKeyBytes",
    "plugins.kv.maxValueBytes",
    "plugins.kv.maxTotalBytes",
    "plugins.kv.maxKeys",
    "maskedEmail.domain",
    "abuse.reportAddress",
    "dlp.adminAddress",
];
/// Family members that now apply (`serverText.alert-<kind>-…` domain overrides).
const NOW_APPLIED_PREFIX: &str = "serverText.alert-";
/// At most this many listed values; the entry says when more were left out.
const MAX_LISTED: usize = 200;
/// Text longer than this is listed by its length only.
const MAX_TEXT: usize = 120;

// ─── readers ──────────────────────────────────────────────────────────────────

/// The subject of a server-wide read.
pub(crate) fn global() -> SettingSubject<'static> {
    SettingSubject::default()
}

/// The subject of a domain-scoped read (no user layer).
pub(crate) fn domain(domain: &str) -> SettingSubject<'_> {
    SettingSubject {
        personal_owner: None,
        domain,
        account_id: "",
        address: "",
    }
}

/// The effective value of `key` for `subject`; `None` when it cannot be read.
async fn value(store: &Store, key: &str, subject: &SettingSubject<'_>) -> Option<Value> {
    match settings_registry::effective_for(store, key, subject).await {
        Ok(Some(e)) => Some(e.value),
        Ok(None) => None,
        Err(e) => {
            tracing::warn!("setting {key} not read, using its default: {e}");
            None
        }
    }
}

/// `value` as an integer clamped to the registry range of `key`; the registry
/// default when it is missing or not a number.
pub(crate) fn clamp_int(key: &str, value: Option<&Value>) -> i64 {
    let def = settings_registry::lookup(key);
    let (min, max) = match def.map(|d| &d.kind) {
        Some(Kind::Int { min, max }) => (*min, *max),
        _ => (i64::MIN, i64::MAX),
    };
    let default = def.and_then(|d| d.default_value().as_i64()).unwrap_or(min.max(0));
    value
        .and_then(Value::as_i64)
        .unwrap_or(default)
        .clamp(min, max)
}

/// An integer key, clamped to its registry range.
pub(crate) async fn int(store: &Store, key: &str, subject: &SettingSubject<'_>) -> i64 {
    clamp_int(key, value(store, key, subject).await.as_ref())
}

/// A text key, trimmed; `None` when unset or empty.
pub(crate) async fn text(store: &Store, key: &str, subject: &SettingSubject<'_>) -> Option<String> {
    value(store, key, subject)
        .await
        .and_then(|v| v.as_str().map(str::trim).map(str::to_string))
        .filter(|s| !s.is_empty())
}

/// A boolean key; `false` when it is not a boolean.
pub(crate) async fn flag(store: &Store, key: &str, subject: &SettingSubject<'_>) -> bool {
    value(store, key, subject).await.and_then(|v| v.as_bool()) == Some(true)
}

// ─── upstream.proxyTimeoutSecs ────────────────────────────────────────────────

const PROXY_TIMEOUT_KEY: &str = "upstream.proxyTimeoutSecs";
/// The registry default, used until the first read.
const PROXY_TIMEOUT_DEFAULT: u64 = 300;
/// How often the cached proxy timeout is re-read.
const PROXY_TIMEOUT_REFRESH: Duration = Duration::from_secs(30);

static PROXY_TIMEOUT_SECS: AtomicU64 = AtomicU64::new(PROXY_TIMEOUT_DEFAULT);

/// Total time allowed for one JMAP upstream request (`upstream.proxyTimeoutSecs`,
/// 30 to 900 seconds). Read from memory; [`refresh_proxy_timeout`] updates it.
pub(crate) fn proxy_timeout() -> Duration {
    Duration::from_secs(PROXY_TIMEOUT_SECS.load(Ordering::Relaxed))
}

/// Re-read `upstream.proxyTimeoutSecs` into the cache and return it.
pub(crate) async fn refresh_proxy_timeout(store: &Store) -> Duration {
    let secs = int(store, PROXY_TIMEOUT_KEY, &global()).await;
    // `int` clamps to 30..=900, so this never stores 0.
    let secs = u64::try_from(secs).unwrap_or(PROXY_TIMEOUT_DEFAULT);
    PROXY_TIMEOUT_SECS.store(secs, Ordering::Relaxed);
    Duration::from_secs(secs)
}

/// Read the proxy timeout now and every 30 seconds for the life of the process.
pub(crate) fn start(store: Store) {
    tokio::spawn(async move {
        loop {
            refresh_proxy_timeout(&store).await;
            tokio::time::sleep(PROXY_TIMEOUT_REFRESH).await;
        }
    });
}

// ─── first-start report ───────────────────────────────────────────────────────

/// What [`report_once`] did.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum Reported {
    /// The marker was already set (an earlier start, or another instance).
    AlreadyDone,
    /// Nothing saved differs from the defaults; the marker is set, no entry.
    NothingSaved,
    /// One audit entry listing this many values.
    Entry(usize),
}

fn shown(value: Value) -> Value {
    match value {
        Value::String(s) if s.chars().count() > MAX_TEXT => json!({ "chars": s.chars().count() }),
        other => other,
    }
}

fn parse(text: &str) -> Value {
    serde_json::from_str(text).unwrap_or(Value::Null)
}

/// Whether an environment variable pins `key`, so a stored value still does not
/// apply.
fn env_pinned(def: &settings_registry::SettingDef) -> bool {
    matches!(
        settings_registry::env_state(def, &settings_registry::process_env),
        Some(settings_registry::EnvState {
            precedence: settings_registry::Precedence::Pin,
            value: Some(_),
            ..
        })
    )
}

/// The saved values of the keys in [`NOW_APPLIED`] (and the `serverText.alert-`
/// family) that differ from the defaults. Global and domain values are listed
/// with the value; per-user, profile and group values are counted.
pub(crate) async fn saved_values(store: &Store) -> Result<Vec<Value>, StoreError> {
    let policies = store.list_setting_policies().await?;
    let counts = store.count_setting_values().await?;
    let mut keys: Vec<String> = NOW_APPLIED.iter().map(|k| k.to_string()).collect();
    for k in policies
        .iter()
        .map(|p| p.key.as_str())
        .chain(counts.iter().map(|c| c.key.as_str()))
    {
        if k.starts_with(NOW_APPLIED_PREFIX) && !keys.iter().any(|x| x == k) {
            keys.push(k.to_string());
        }
    }
    let mut out = Vec::new();
    for key in &keys {
        let Some(def) = settings_registry::lookup(key) else {
            continue;
        };
        if env_pinned(def) {
            continue;
        }
        let default = def.default_value();
        if let Some(v) = policies
            .iter()
            .find(|p| p.key == *key)
            .and_then(|p| p.global_value.as_deref())
            .map(parse)
            .filter(|v| *v != default)
        {
            out.push(json!({ "key": key, "scope": "global", "value": shown(v) }));
        }
        let mut after: Option<String> = None;
        loop {
            let page = store
                .list_setting_values(key, SettingScope::Domain, after.as_deref(), 500)
                .await?;
            for row in &page {
                let v = parse(&row.value);
                if v != default {
                    out.push(json!({
                        "key": key,
                        "scope": "domain",
                        "domain": row.scope_id,
                        "value": shown(v),
                    }));
                }
            }
            if page.len() < 500 {
                break;
            }
            after = page.last().map(|r| r.scope_id.clone());
        }
        for c in counts
            .iter()
            .filter(|c| c.key == *key && c.scope != SettingScope::Domain)
        {
            out.push(json!({ "key": key, "scope": c.scope.as_str(), "count": c.count }));
        }
    }
    Ok(out)
}

/// Write the first-start audit entry unless the marker says it was done. Errors
/// are returned; the caller logs them and starts anyway.
pub(crate) async fn report_once(store: &Store) -> Result<Reported, StoreError> {
    if store.get_setting_policy(MARKER_KEY).await?.is_some() {
        return Ok(Reported::AlreadyDone);
    }
    let mut values = saved_values(store).await?;
    let now = chrono::Utc::now();
    let marker = SettingPolicyRow {
        key: MARKER_KEY.to_string(),
        global_value: Some(json!(now.timestamp()).to_string()),
        allow_user_override: false,
        global_locked: false,
        updated_by: "system".to_string(),
        updated_at: now.timestamp(),
    };
    // Claim the marker first: of two instances starting together, one writes.
    if !store
        .compare_setting_policy(MARKER_KEY, None, Some(&marker))
        .await?
    {
        return Ok(Reported::AlreadyDone);
    }
    if values.is_empty() {
        return Ok(Reported::NothingSaved);
    }
    let total = values.len();
    values.truncate(MAX_LISTED);
    let detail = json!({
        "summary": "Settings saved in the console before this release that the server now applies",
        "values": values,
        "total": total,
        "truncated": total > MAX_LISTED,
        "upgradeNote": "docs/deploy/upgrade-notes/console-settings-now-applied.md",
    });
    store
        .append_audit(&mw_store::AuditRow {
            id: uuid::Uuid::new_v4().to_string(),
            ts: now.to_rfc3339(),
            actor: "system".into(),
            actor_kind: "system".into(),
            action: AUDIT_ACTION.into(),
            target: None,
            detail_json: detail.to_string(),
            ip: None,
        })
        .await?;
    Ok(Reported::Entry(total))
}

/// [`report_once`], logging instead of failing start-up.
pub(crate) async fn report_at_start(store: &Store) {
    match report_once(store).await {
        Ok(Reported::Entry(n)) => tracing::info!(
            "{n} saved console setting value(s) now take effect; see the audit entry {AUDIT_ACTION}"
        ),
        Ok(_) => {}
        Err(e) => tracing::warn!("could not check for console settings that now apply: {e}"),
    }
}

/// Test helpers: store a global or domain value as the console would.
#[cfg(test)]
pub(crate) mod test_support {
    use mw_store::Store;
    use mw_store::settings_policy::{SettingPolicyRow, SettingScope, SettingValueRow};

    pub(crate) async fn store() -> Store {
        Store::open_in_memory(mw_store::ServerKey::generate()).await.unwrap()
    }

    pub(crate) async fn put_global(store: &Store, key: &str, json: &str) {
        store
            .put_setting_policy(&SettingPolicyRow {
                key: key.into(),
                global_value: Some(json.into()),
                allow_user_override: false,
                global_locked: false,
                updated_by: "test".into(),
                updated_at: 1,
            })
            .await
            .unwrap();
    }

    pub(crate) async fn put_domain(store: &Store, domain: &str, key: &str, json: &str) {
        store
            .put_setting_value(&SettingValueRow {
                scope: SettingScope::Domain,
                scope_id: domain.into(),
                key: key.into(),
                value: json.into(),
                locked: false,
                updated_by: "test".into(),
                updated_at: 1,
            })
            .await
            .unwrap();
    }
}

#[cfg(test)]
mod tests {
    use super::test_support::*;
    use super::*;

    #[test]
    fn every_listed_key_is_registered_and_never_secret() {
        for key in NOW_APPLIED {
            let def = settings_registry::lookup(key)
                .unwrap_or_else(|| panic!("{key} is not a registry key"));
            let described = def.kind.describe().to_string().to_ascii_lowercase();
            assert!(!described.contains("secret"), "{key} is a secret kind");
        }
    }

    #[test]
    fn clamp_keeps_values_in_the_registry_range() {
        assert_eq!(clamp_int("webhooks.maxAttempts", Some(&json!(0))), 1);
        assert_eq!(clamp_int("webhooks.maxAttempts", Some(&json!(99))), 10);
        assert_eq!(clamp_int("webhooks.maxAttempts", Some(&json!("x"))), 3);
        assert_eq!(clamp_int("webhooks.timeoutSecs", Some(&json!(86_400))), 60);
        assert_eq!(clamp_int("upstream.proxyTimeoutSecs", Some(&json!(0))), 30);
        assert_eq!(clamp_int("upstream.proxyTimeoutSecs", None), 300);
        assert_eq!(clamp_int("import.maxMessages", Some(&json!(-5))), 1);
    }

    #[test]
    fn long_text_is_listed_by_length() {
        assert_eq!(shown(json!("short")), json!("short"));
        assert_eq!(shown(json!("x".repeat(500))), json!({ "chars": 500 }));
    }

    async fn audited(store: &Store) -> Vec<mw_store::AuditRow> {
        let all = store.list_audit(1000).await.unwrap();
        all.into_iter().filter(|r| r.action == AUDIT_ACTION).collect()
    }

    /// The environment is process-wide: these tests restore what they change.
    /// The gate runs with `--test-threads=1`.
    struct EnvGuard(&'static str, Option<String>);
    impl EnvGuard {
        fn set(var: &'static str, value: Option<&str>) -> Self {
            let old = std::env::var(var).ok();
            unsafe {
                match value {
                    Some(v) => std::env::set_var(var, v),
                    None => std::env::remove_var(var),
                }
            }
            Self(var, old)
        }
    }
    impl Drop for EnvGuard {
        fn drop(&mut self) {
            unsafe {
                match &self.1 {
                    Some(v) => std::env::set_var(self.0, v),
                    None => std::env::remove_var(self.0),
                }
            }
        }
    }

    #[tokio::test]
    async fn ldap_precheck_console_value_applies_and_the_environment_wins() {
        let store = store().await;
        let key = "signin.ldapBindPrecheck";
        {
            let _env = EnvGuard::set("MW_LDAP_BIND_AUTH", None);
            assert!(!flag(&store, key, &global()).await, "default off");
            put_global(&store, key, "true").await;
            assert!(flag(&store, key, &global()).await, "console value applies");
        }
        {
            let _env = EnvGuard::set("MW_LDAP_BIND_AUTH", Some("0"));
            assert!(!flag(&store, key, &global()).await, "environment wins");
        }
        put_global(&store, key, "false").await;
        let _env = EnvGuard::set("MW_LDAP_BIND_AUTH", Some("1"));
        assert!(flag(&store, key, &global()).await, "environment wins");
    }

    #[tokio::test]
    async fn abuse_address_is_domain_scoped_and_the_environment_wins() {
        let store = store().await;
        let key = "abuse.reportAddress";
        let _env = EnvGuard::set("MW_ABUSE_ADDRESS", None);
        assert_eq!(text(&store, key, &domain("a.example")).await, None);
        put_domain(&store, "a.example", key, "\"abuse@a.example\"").await;
        assert_eq!(
            text(&store, key, &domain("a.example")).await.as_deref(),
            Some("abuse@a.example")
        );
        assert_eq!(text(&store, key, &domain("b.example")).await, None);
        let _env = EnvGuard::set("MW_ABUSE_ADDRESS", Some("ops@host.example"));
        assert_eq!(
            text(&store, key, &domain("a.example")).await.as_deref(),
            Some("ops@host.example")
        );
    }

    #[tokio::test]
    async fn proxy_timeout_follows_the_setting_within_its_range() {
        let store = store().await;
        assert_eq!(refresh_proxy_timeout(&store).await, Duration::from_secs(300));
        put_global(&store, PROXY_TIMEOUT_KEY, "45").await;
        assert_eq!(refresh_proxy_timeout(&store).await, Duration::from_secs(45));
        assert_eq!(proxy_timeout(), Duration::from_secs(45));
        // A raw row outside the range (the console refuses one) is clamped.
        put_global(&store, PROXY_TIMEOUT_KEY, "0").await;
        assert_eq!(refresh_proxy_timeout(&store).await, Duration::from_secs(30));
        put_global(&store, PROXY_TIMEOUT_KEY, "999999").await;
        assert_eq!(refresh_proxy_timeout(&store).await, Duration::from_secs(900));
        put_global(&store, PROXY_TIMEOUT_KEY, "300").await;
        refresh_proxy_timeout(&store).await;
    }

    #[tokio::test]
    async fn the_report_lists_saved_values_once() {
        let _env = EnvGuard::set("MW_MASKED_EMAIL_DOMAIN", None);
        let store = store().await;
        put_global(&store, "webhooks.maxAttempts", "1").await;
        // Equal to the default: not listed.
        put_global(&store, "webhooks.timeoutSecs", "10").await;
        put_domain(&store, "a.example", "import.maxMessages", "2").await;
        put_global(&store, "maskedEmail.domain", "\"m.example\"").await;
        // Not a key that now applies: not listed.
        put_global(&store, "signin.refusalDetail", "\"uniform\"").await;

        assert_eq!(report_once(&store).await.unwrap(), Reported::Entry(3));
        assert_eq!(report_once(&store).await.unwrap(), Reported::AlreadyDone);

        let entries = audited(&store).await;
        assert_eq!(entries.len(), 1, "one entry");
        let detail: Value = serde_json::from_str(&entries[0].detail_json).unwrap();
        let listed: Vec<(String, String)> = detail["values"]
            .as_array()
            .unwrap()
            .iter()
            .map(|v| (v["key"].as_str().unwrap().to_string(), v["scope"].as_str().unwrap().to_string()))
            .collect();
        assert_eq!(
            listed,
            vec![
                ("import.maxMessages".to_string(), "domain".to_string()),
                ("webhooks.maxAttempts".to_string(), "global".to_string()),
                ("maskedEmail.domain".to_string(), "global".to_string()),
            ]
        );
        assert_eq!(detail["values"][0]["domain"], "a.example");
        assert_eq!(detail["values"][0]["value"], 2);
    }

    #[tokio::test]
    async fn nothing_saved_sets_the_marker_without_an_entry() {
        let store = store().await;
        assert_eq!(report_once(&store).await.unwrap(), Reported::NothingSaved);
        assert_eq!(report_once(&store).await.unwrap(), Reported::AlreadyDone);
        assert!(audited(&store).await.is_empty());
    }

    #[tokio::test]
    async fn a_value_the_environment_overrides_is_not_listed() {
        let store = store().await;
        put_global(&store, "maskedEmail.domain", "\"m.example\"").await;
        let _env = EnvGuard::set("MW_MASKED_EMAIL_DOMAIN", Some("env.example"));
        assert_eq!(report_once(&store).await.unwrap(), Reported::NothingSaved);
    }
}
