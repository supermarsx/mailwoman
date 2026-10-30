//! `GET /admin/users/table` (t107): the admin Users table — search, filters,
//! sort and paging done on the server, with the columns the list needs.
//!
//! The directory is bounded (`Store::directory_accounts`, at most 5000 rows), so
//! the whole directory is read once with a fixed set of whole-table queries
//! (`Store::user_table_facts`) and filtered, sorted and paged here. Nothing is
//! looked up per user: the alias chain, the native login spelling, the flags, the
//! second factors, the feature profile and the linked-mailbox counts all come from
//! maps built by those queries.
//!
//! Role scoping is the same as `GET /admin/users`: an admin whose `users.read` is
//! limited to some domains sees only users whose canonical policy domain is one of
//! them. Every count (`total`, `unfilteredTotal`, the facets) is computed after
//! scoping, so a scoped admin learns nothing about other domains.
//!
//! Query (all optional): `q` (case-insensitive substring of the name, login
//! address, directory address, domain, or the address under an alias domain),
//! `domain`, `status` (`active|blocked|disabled|pending|deleted`), `twoFactor`
//! (`passkey|totp|none`), `role` (`admin|none|<role name>`), `profile`
//! (`<profile id>|none`) — each a comma-separated list of allowed values —
//! `signin` (`never`, `older:<days>`, `within:<days>`), `createdFrom` /
//! `createdTo` (epoch seconds, inclusive), `sort` (`name`, `address`, `domain`,
//! `status`, `twoFactor`, `role`, `profile`, `lastSeen`, `lastSignin`, `created`, `linked`,
//! `quota`), `dir` (`asc|desc`), `offset`, `limit` (1-200, default 50).
//! Default order is most recently seen first; absent activity sorts last.

use std::cmp::Ordering;
use std::collections::{BTreeSet, HashMap};

use axum::Json;
use axum::extract::{Query, State};
use axum::response::{IntoResponse, Response};
use serde::{Deserialize, Serialize};

use super::{FlagsDto, QuotaDto, err500};
use crate::AppState;
use crate::admin_ctx::AdminCtx;
use crate::capabilities::Capability;

const DEFAULT_LIMIT: usize = 50;
const MAX_LIMIT: usize = 200;
const DAY: i64 = 86_400;

#[derive(Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub(super) struct TableQuery {
    q: Option<String>,
    domain: Option<String>,
    status: Option<String>,
    two_factor: Option<String>,
    role: Option<String>,
    profile: Option<String>,
    signin: Option<String>,
    created_from: Option<i64>,
    created_to: Option<i64>,
    sort: Option<String>,
    dir: Option<String>,
    offset: Option<usize>,
    limit: Option<usize>,
}

#[derive(Serialize, Clone)]
#[serde(rename_all = "camelCase")]
struct ProfileRef {
    id: String,
    name: String,
    /// Where the profile comes from: `user`, `group` or `domain`.
    via: &'static str,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct UserRowDto {
    account_id: String,
    username: String,
    domain: String,
    login_address: String,
    directory_address: String,
    legacy_binding: bool,
    display_name: Option<String>,
    /// The lifecycle status as stored (`active|disabled|deleting|deleted`).
    status: String,
    /// What the table shows: `active|blocked|disabled|pending|deleted`.
    state: &'static str,
    deleted_at: Option<i64>,
    last_signin_at: Option<i64>,
    last_seen_at: Option<i64>,
    created_at: Option<i64>,
    /// The strongest second factor: `passkey`, `totp` or `none`.
    two_factor: &'static str,
    two_factor_methods: Vec<&'static str>,
    /// The console role of an admin account with this address, if there is one.
    admin_role: Option<String>,
    profile: Option<ProfileRef>,
    linked_mailboxes: i64,
    quota: Option<QuotaDto>,
    flags: FlagsDto,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct Facets {
    domains: Vec<String>,
    roles: Vec<String>,
    profiles: Vec<FacetProfile>,
}
#[derive(Serialize)]
struct FacetProfile {
    id: String,
    name: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct TablePage {
    rows: Vec<UserRowDto>,
    /// Rows matching the filters (before paging).
    total: usize,
    /// Rows the admin may see at all (before filters).
    unfiltered_total: usize,
    offset: usize,
    limit: usize,
    facets: Facets,
}

/// The alias chain resolved in memory, as `Store::canonical_policy_domain` does
/// it in the database: at most 16 hops, a cycle is an error.
fn canonical_domain(domain: &str, aliases: &HashMap<String, String>) -> Option<String> {
    if domain.trim().is_empty() {
        return Some(String::new());
    }
    let mut domain = mw_store::settings_policy::normalize_policy_domain(domain).ok()?;
    let mut seen = Vec::new();
    loop {
        if seen.len() >= 16 || seen.contains(&domain) {
            return None;
        }
        seen.push(domain.clone());
        match aliases.get(&domain) {
            Some(target) => {
                domain = mw_store::settings_policy::normalize_policy_domain(target).ok()?
            }
            None => return Some(domain),
        }
    }
}

fn canonical_address(address: &str, aliases: &HashMap<String, String>) -> Option<String> {
    let address = address.trim().to_lowercase();
    match address.rsplit_once('@') {
        Some((local, domain)) => Some(format!("{local}@{}", canonical_domain(domain, aliases)?)),
        None => Some(address),
    }
}

/// The presentation-only login spelling (`admin::directory_login_address`),
/// from the batch of verified native logins.
fn login_address(
    account: &str,
    address: &str,
    facts: &mw_store::user_directory::UserTableFacts,
) -> String {
    let Some(native) = facts
        .native_logins
        .get(&(account.to_string(), address.to_string()))
    else {
        return address.into();
    };
    let normalized = native.trim().to_lowercase();
    let Some((local, domain)) = normalized.rsplit_once('@') else {
        return address.into();
    };
    let Ok(domain) = crate::domain_config::normalize_domain(domain) else {
        return address.into();
    };
    let login = format!("{local}@{domain}");
    if login != address && canonical_address(&login, &facts.aliases).as_deref() == Some(address) {
        login
    } else {
        address.into()
    }
}

fn flags_of(account: &str, facts: &mw_store::user_directory::UserTableFacts) -> FlagsDto {
    let folded = crate::account_gate::flag_subject(account);
    facts
        .flags
        .get(&folded)
        .or_else(|| facts.flags.get(account))
        .and_then(|raw| serde_json::from_str::<mw_admin::UserFeatureFlags>(raw).ok())
        .map(FlagsDto::from)
        .unwrap_or_default()
}

fn list(value: &Option<String>) -> Option<Vec<String>> {
    let items: Vec<String> = value
        .as_deref()?
        .split(',')
        .map(|v| v.trim().to_lowercase())
        .filter(|v| !v.is_empty())
        .collect();
    (!items.is_empty()).then_some(items)
}

enum SigninFilter {
    Never,
    Older(i64),
    Within(i64),
}
fn signin_filter(value: &Option<String>) -> Option<SigninFilter> {
    let value = value.as_deref()?.trim();
    if value == "never" {
        return Some(SigninFilter::Never);
    }
    let (kind, days) = value.split_once(':')?;
    let days = days
        .parse::<i64>()
        .ok()
        .filter(|d| (0..=36_500).contains(d))?;
    match kind {
        "older" => Some(SigninFilter::Older(days)),
        "within" => Some(SigninFilter::Within(days)),
        _ => None,
    }
}

fn state_of(
    status: &str,
    flags: &FlagsDto,
    last: Option<i64>,
    origin: Option<&str>,
) -> &'static str {
    match status {
        "deleted" | "deleting" => "deleted",
        "disabled" => "blocked",
        _ if flags.disabled => "disabled",
        _ if last.is_none() && origin == Some("provisioned") => "pending",
        _ => "active",
    }
}

fn state_rank(state: &str) -> u8 {
    match state {
        "active" => 0,
        "pending" => 1,
        "blocked" => 2,
        "disabled" => 3,
        _ => 4,
    }
}
fn factor_rank(factor: &str) -> u8 {
    match factor {
        "passkey" => 0,
        "totp" => 1,
        _ => 2,
    }
}

/// `None` sorts after every value in either direction (a blank cell stays at the
/// bottom), except `lastSignin`, where `None` is "Never": the oldest.
fn opt_cmp<T: Ord>(a: &Option<T>, b: &Option<T>, desc: bool) -> Ordering {
    match (a, b) {
        (Some(a), Some(b)) => {
            if desc {
                b.cmp(a)
            } else {
                a.cmp(b)
            }
        }
        (Some(_), None) => Ordering::Less,
        (None, Some(_)) => Ordering::Greater,
        (None, None) => Ordering::Equal,
    }
}

fn name_key(row: &UserRowDto) -> String {
    row.display_name
        .clone()
        .unwrap_or_else(|| row.login_address.clone())
        .to_lowercase()
}

/// Match the priority and domain rules of `Store::setting_profile_and_group_ids`.
fn selected_profile(
    facts: &mw_store::user_directory::UserTableFacts,
    account: &str,
    domain: &str,
) -> Option<ProfileRef> {
    let mut candidates = Vec::new();
    let mut add = |kind: &'static str, scope: &str, rank: u8| {
        if let Some((id, priority)) = facts.profile_assignments.get(&(kind.into(), scope.into())) {
            candidates.push((*priority, rank, id.clone(), scope.to_string(), kind));
        }
    };
    if !account.is_empty() {
        add("user", account, 0);
        for group in facts.memberships.get(account).into_iter().flatten() {
            if facts
                .group_domains
                .get(group)
                .is_some_and(|d| d.is_empty() || d == domain)
            {
                add("group", group, 1);
            }
        }
    }
    if !domain.is_empty() {
        add("domain", domain, 2);
    }
    candidates.sort_by(|a, b| {
        b.0.cmp(&a.0)
            .then(a.1.cmp(&b.1))
            .then(a.2.cmp(&b.2))
            .then(a.3.cmp(&b.3))
    });
    candidates.into_iter().find_map(|(_, _, id, _, kind)| {
        let name = &facts.profile_names.iter().find(|(key, _)| key == &id)?.1;
        Some(ProfileRef {
            id: id.clone(),
            name: name.clone(),
            via: match kind {
                "user" => "user",
                "group" => "group",
                _ => "domain",
            },
        })
    })
}

pub(super) async fn users_table(
    State(state): State<AppState>,
    ctx: AdminCtx,
    Query(query): Query<TableQuery>,
) -> Response {
    let users = match state.store.directory_accounts().await {
        Ok(u) => u,
        Err(e) => return err500(e),
    };
    let facts = match state.store.user_table_facts().await {
        Ok(f) => f,
        Err(e) => return err500(e),
    };
    let scope: Option<&BTreeSet<String>> = ctx.domain_filter(Capability::UsersRead);
    // Domain alias → every alias of a canonical domain, for search.
    let mut alias_of: HashMap<String, Vec<String>> = HashMap::new();
    for alias in facts.aliases.keys() {
        if let Some(target) = canonical_domain(alias, &facts.aliases) {
            alias_of.entry(target).or_default().push(alias.clone());
        }
    }

    let mut rows = Vec::new();
    let mut searchable = Vec::new();
    for u in users {
        // An alias cycle: unscoped admins still see the row under its own spelling.
        let policy_address = match canonical_address(&u.address, &facts.aliases) {
            Some(address) => address,
            None if scope.is_some() => continue,
            None => u.address.to_lowercase(),
        };
        let policy_domain = crate::admin_ctx::domain_of(&policy_address);
        if let Some(scope) = scope
            && !scope.contains(&policy_domain)
        {
            continue;
        }
        let account_id = u.account_id;
        let directory_address = u.address;
        let login = login_address(&account_id, &directory_address, &facts);
        let legacy_binding = login != directory_address;
        let (username, domain) = match login.rsplit_once('@') {
            Some((a, b)) => (a.to_string(), b.to_string()),
            None => (login.clone(), String::new()),
        };
        let account = facts.accounts.get(&account_id).cloned().unwrap_or_default();
        let person = facts.people.get(&directory_address);
        let flags = flags_of(&account_id, &facts);
        let mut methods = Vec::new();
        if account.passkeys > 0 {
            methods.push("passkey");
        }
        if account.totp {
            methods.push("totp");
        }
        let profile = selected_profile(&facts, &account_id, &policy_domain);
        let provisioned = facts.provisioned.get(&account_id);
        let origin = person
            .map(|p| p.origin.as_str())
            .or(provisioned.map(|_| "provisioned"));
        let state_word = state_of(&u.status, &flags, u.last_signin_at, origin);
        let created_at = person
            .map(|p| p.created_at)
            .or(provisioned.copied().flatten())
            .or(account.first_seen);
        let mut haystack = vec![
            login.to_lowercase(),
            directory_address.to_lowercase(),
            domain.to_lowercase(),
            policy_domain.clone(),
        ];
        if let Some(name) = person.and_then(|p| p.display_name.as_ref()) {
            haystack.push(name.to_lowercase());
        }
        if let Some((local, _)) = policy_address.rsplit_once('@') {
            for alias in alias_of.get(&policy_domain).into_iter().flatten() {
                haystack.push(format!("{local}@{alias}"));
            }
        }
        searchable.push(haystack.join("\n"));
        rows.push(UserRowDto {
            account_id,
            username,
            domain,
            login_address: login,
            directory_address,
            legacy_binding,
            display_name: person.and_then(|p| p.display_name.clone()),
            status: u.status,
            state: state_word,
            deleted_at: u.deleted_at,
            last_signin_at: u.last_signin_at,
            last_seen_at: account.last_seen_at.max(u.last_signin_at),
            created_at,
            two_factor: methods.first().copied().unwrap_or("none"),
            two_factor_methods: methods,
            admin_role: facts.admin_roles.get(&policy_address).cloned(),
            profile,
            linked_mailboxes: account.linked,
            quota: account.quota.map(|(bytes_limit, msg_limit)| QuotaDto {
                bytes_limit,
                msg_limit,
            }),
            flags,
        });
    }

    let unfiltered_total = rows.len();
    let mut domains: Vec<String> = rows.iter().map(|r| r.domain.clone()).collect();
    domains.sort();
    domains.dedup();
    domains.retain(|d| !d.is_empty());
    let mut roles: Vec<String> = rows.iter().filter_map(|r| r.admin_role.clone()).collect();
    roles.sort();
    roles.dedup();
    let facets = Facets {
        domains,
        roles,
        profiles: facts
            .profile_names
            .iter()
            .filter(|(id, _)| {
                scope.is_none()
                    || rows.iter().any(|row| {
                        row.profile
                            .as_ref()
                            .is_some_and(|profile| profile.id == *id)
                    })
            })
            .map(|(id, name)| FacetProfile {
                id: id.clone(),
                name: name.clone(),
            })
            .collect(),
    };

    let needle = query
        .q
        .as_deref()
        .map(|q| q.trim().to_lowercase())
        .filter(|q| !q.is_empty());
    let domains = list(&query.domain);
    let statuses = list(&query.status);
    let factors = list(&query.two_factor);
    let role_filter = list(&query.role);
    let profiles = query
        .profile
        .as_deref()
        .map(|p| {
            p.split(',')
                .map(|v| v.trim().to_string())
                .filter(|v| !v.is_empty())
                .collect::<Vec<_>>()
        })
        .filter(|v| !v.is_empty());
    let signin = signin_filter(&query.signin);
    let now = chrono::Utc::now().timestamp();
    let keep = |row: &UserRowDto, text: &str| -> bool {
        if let Some(needle) = &needle
            && !text.contains(needle.as_str())
        {
            return false;
        }
        if let Some(d) = &domains
            && !d.contains(&row.domain.to_lowercase())
        {
            return false;
        }
        if let Some(s) = &statuses
            && !s.iter().any(|s| s == row.state)
        {
            return false;
        }
        if let Some(f) = &factors
            && !f.iter().any(|f| f == row.two_factor)
        {
            return false;
        }
        if let Some(r) = &role_filter {
            let role = row.admin_role.as_deref().map(str::to_lowercase);
            let ok = r.iter().any(|want| match want.as_str() {
                "admin" => role.is_some(),
                "none" => role.is_none(),
                name => role.as_deref() == Some(name),
            });
            if !ok {
                return false;
            }
        }
        if let Some(p) = &profiles {
            let id = row.profile.as_ref().map(|p| p.id.as_str());
            if !p.iter().any(|want| match want.as_str() {
                "none" => id.is_none(),
                want => id == Some(want),
            }) {
                return false;
            }
        }
        match (&signin, row.last_signin_at) {
            (Some(SigninFilter::Never), Some(_)) => return false,
            (Some(SigninFilter::Older(_) | SigninFilter::Within(_)), None) => return false,
            (Some(SigninFilter::Older(days)), Some(at)) if at > now - days * DAY => return false,
            (Some(SigninFilter::Within(days)), Some(at)) if at < now - days * DAY => return false,
            _ => {}
        }
        if let Some(from) = query.created_from
            && row.created_at.is_none_or(|c| c < from)
        {
            return false;
        }
        if let Some(to) = query.created_to
            && row.created_at.is_none_or(|c| c > to)
        {
            return false;
        }
        true
    };
    let mut matched: Vec<UserRowDto> = rows
        .into_iter()
        .zip(searchable.iter())
        .filter(|(row, text)| keep(row, text))
        .map(|(row, _)| row)
        .collect();

    let sort = query.sort.as_deref().unwrap_or("lastSeen");
    let desc = query
        .dir
        .as_deref()
        .map_or(query.sort.is_none(), |dir| dir == "desc");
    let flip = |o: Ordering| if desc { o.reverse() } else { o };
    matched.sort_by(|a, b| {
        let primary = match sort {
            "address" => flip(
                a.login_address
                    .to_lowercase()
                    .cmp(&b.login_address.to_lowercase()),
            ),
            "domain" => flip(a.domain.cmp(&b.domain)),
            "status" => flip(state_rank(a.state).cmp(&state_rank(b.state))),
            "twoFactor" => flip(factor_rank(a.two_factor).cmp(&factor_rank(b.two_factor))),
            "role" => opt_cmp(&a.admin_role, &b.admin_role, desc),
            "profile" => opt_cmp(
                &a.profile.as_ref().map(|p| p.name.to_lowercase()),
                &b.profile.as_ref().map(|p| p.name.to_lowercase()),
                desc,
            ),
            // Never is the oldest sign-in: first ascending, last descending.
            "lastSignin" => flip(a.last_signin_at.cmp(&b.last_signin_at)),
            "lastSeen" => opt_cmp(&a.last_seen_at, &b.last_seen_at, desc),
            "created" => opt_cmp(&a.created_at, &b.created_at, desc),
            "linked" => flip(a.linked_mailboxes.cmp(&b.linked_mailboxes)),
            "quota" => opt_cmp(
                &a.quota.map(|q| q.bytes_limit),
                &b.quota.map(|q| q.bytes_limit),
                desc,
            ),
            _ => flip(name_key(a).cmp(&name_key(b))),
        };
        primary
            .then_with(|| a.login_address.cmp(&b.login_address))
            .then_with(|| a.account_id.cmp(&b.account_id))
    });

    let total = matched.len();
    let limit = query.limit.unwrap_or(DEFAULT_LIMIT).clamp(1, MAX_LIMIT);
    let offset = query.offset.unwrap_or(0).min(total);
    let rows = matched.into_iter().skip(offset).take(limit).collect();
    Json(TablePage {
        rows,
        total,
        unfiltered_total,
        offset,
        limit,
        facets,
    })
    .into_response()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn displayed_profile_follows_priority_and_group_domain() {
        let mut facts = mw_store::user_directory::UserTableFacts::default();
        for id in ["personal", "domain", "foreign", "group"] {
            facts.profile_names.push((id.into(), id.into()));
        }
        for (kind, scope, id, priority) in [
            ("user", "alice", "personal", 1),
            ("domain", "example.test", "domain", 10),
            ("group", "other", "foreign", 100),
            ("group", "local", "group", 10),
        ] {
            facts
                .profile_assignments
                .insert((kind.into(), scope.into()), (id.into(), priority));
        }
        facts
            .memberships
            .insert("alice".into(), vec!["other".into(), "local".into()]);
        facts
            .group_domains
            .insert("other".into(), "elsewhere.test".into());
        facts
            .group_domains
            .insert("local".into(), "example.test".into());
        let selected = selected_profile(&facts, "alice", "example.test").unwrap();
        assert_eq!((selected.id.as_str(), selected.via), ("group", "group"));
        facts
            .group_domains
            .insert("local".into(), "elsewhere.test".into());
        assert_eq!(
            selected_profile(&facts, "alice", "example.test")
                .unwrap()
                .id,
            "domain"
        );
        facts
            .profile_assignments
            .insert(("user".into(), "alice".into()), ("personal".into(), 10));
        assert_eq!(
            selected_profile(&facts, "alice", "example.test")
                .unwrap()
                .id,
            "personal"
        );
    }

    #[test]
    fn alias_chain_resolves_and_cycles_fail() {
        let mut aliases = HashMap::new();
        aliases.insert("old.test".to_string(), "mid.test".to_string());
        aliases.insert("mid.test".to_string(), "new.test".to_string());
        assert_eq!(
            canonical_domain("Old.Test", &aliases).as_deref(),
            Some("new.test")
        );
        assert_eq!(
            canonical_address("A@old.test", &aliases).as_deref(),
            Some("a@new.test")
        );
        aliases.insert("new.test".to_string(), "old.test".to_string());
        assert_eq!(canonical_domain("old.test", &aliases), None);
        assert_eq!(canonical_address("bare", &aliases).as_deref(), Some("bare"));
    }

    #[test]
    fn signin_filter_parses_only_known_forms() {
        assert!(matches!(
            signin_filter(&Some("never".into())),
            Some(SigninFilter::Never)
        ));
        assert!(matches!(
            signin_filter(&Some("older:30".into())),
            Some(SigninFilter::Older(30))
        ));
        assert!(matches!(
            signin_filter(&Some("within:7".into())),
            Some(SigninFilter::Within(7))
        ));
        assert!(signin_filter(&Some("older:-1".into())).is_none());
        assert!(signin_filter(&Some("since:3".into())).is_none());
        assert!(signin_filter(&None).is_none());
    }

    #[test]
    fn state_words() {
        let clear = FlagsDto::default();
        let legacy = FlagsDto {
            disabled: true,
            ..FlagsDto::default()
        };
        assert_eq!(state_of("deleting", &clear, None, None), "deleted");
        assert_eq!(state_of("disabled", &legacy, Some(1), None), "blocked");
        assert_eq!(state_of("active", &legacy, Some(1), None), "disabled");
        assert_eq!(
            state_of("active", &clear, None, Some("provisioned")),
            "pending"
        );
        assert_eq!(state_of("active", &clear, None, Some("signin")), "active");
        assert_eq!(
            state_of("active", &clear, Some(5), Some("provisioned")),
            "active"
        );
    }
}
