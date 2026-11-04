//! Per-account sign-in geography (off by default), built on the instance policy
//! in `signin_geography` and the same country database.
//!
//! Modes (`security.signin.accountGeoMode`, global + domain + user; an administrator
//! sets the default and may lock or force it with the usual setting policy):
//! * `off` (default);
//! * `block-unusual-countries` - the account's own successful sign-ins establish
//!   its countries; a sign-in from another country is refused. The first sign-in
//!   (no earlier success with a known country) is always allowed. History is
//!   the account's own durable list of countries (`account_geo_history`, kept
//!   `security.signin.accountGeoHistoryDays`, 365 by default), independent of
//!   the authentication log's retention; an account without a list is seeded
//!   once from the log's retained successful rows;
//! * `block-foreign-countries` - refuse any country outside the allowed countries.
//!
//! Allowlists always pass this policy (never the password, second factor, lockout
//! or other protections): countries and IP networks, an instance-wide pair set by
//! the administrator (applies to everyone) plus the account's own pair (the user,
//! or the administrator for that user). A bare IPv6 address means its /64.
//!
//! Coverage: applied once a sign-in has proven its credentials and before the
//! session is kept, in `security_center::record_login`: password, passkey, SSO
//! and second-factor completions, for the webmail account. Not applied: sessions
//! that already exist, IMAP/SMTP/app-password clients (no country policy exists
//! on those paths), the administrator console (it has `security.signin.geo*`).
//! A source with no country: private, loopback and link-local addresses are a
//! local network and always allowed. Otherwise the two causes (no database
//! loaded, address not in the database) follow `security.geo.noDatabase` /
//! `security.geo.notFound` and the older `security.signin.accountGeoLookupFailure`
//! (`deny` = block); the stricter of the two applies. Default: allowed and logged.
use crate::{AppState, scope_mw::proxy::Cidr};
use axum::{Json, http::StatusCode, response::IntoResponse, response::Response};
use crate::geo_lookup::{self, Failure, Outcome, Shared};
use mw_engine::security::{CountryLookup, country_lookup};
use mw_store::settings_policy::SettingSubject;
use serde_json::{Value, json};
use std::{collections::BTreeSet, net::IpAddr};

pub(crate) const KEYS: [&str; 7] = [
    "security.signin.accountGeoMode",
    "security.signin.accountGeoAllowCountries",
    "security.signin.accountGeoAllowIps",
    "security.signin.accountGeoGlobalAllowCountries",
    "security.signin.accountGeoGlobalAllowIps",
    "security.signin.accountGeoLookupFailure",
    "security.signin.accountGeoHistoryDays",
];
/// Every key `load` reads: the policy keys and the shared lookup-failure defaults.
fn wanted(key: &str) -> bool {
    KEYS.contains(&key) || key == geo_lookup::NO_DATABASE_KEY || key == geo_lookup::NOT_FOUND_KEY
}
/// The keys a user (or the administrator for a user) edits for one account.
pub(crate) const USER_KEYS: [&str; 3] = [KEYS[0], KEYS[1], KEYS[2]];
/// Shown wherever the policy is enabled or edited; the clients carry translations.
pub(crate) const ACCURACY_NOTICE: &str = "Country detection uses an IP geolocation database that can be wrong or out of date. VPNs, mobile carriers, proxies and travel can block a legitimate sign-in or let a foreign one through. Add trusted IP addresses or countries to the allowlist.";

const ISO_3166: &str = "AD AE AF AG AI AL AM AO AQ AR AS AT AU AW AX AZ BA BB BD BE BF BG BH BI BJ BL BM BN BO BQ BR BS BT BV BW BY BZ CA CC CD CF CG CH CI CK CL CM CN CO CR CU CV CW CX CY CZ DE DJ DK DM DO DZ EC EE EG EH ER ES ET FI FJ FK FM FO FR GA GB GD GE GF GG GH GI GL GM GN GP GQ GR GS GT GU GW GY HK HM HN HR HT HU ID IE IL IM IN IO IQ IR IS IT JE JM JO JP KE KG KH KI KM KN KP KR KW KY KZ LA LB LC LI LK LR LS LT LU LV LY MA MC MD ME MF MG MH MK ML MM MN MO MP MQ MR MS MT MU MV MW MX MY MZ NA NC NE NF NG NI NL NO NP NR NU NZ OM PA PE PF PG PH PK PL PM PN PR PS PT PW PY QA RE RO RS RU RW SA SB SC SD SE SG SH SI SJ SK SL SM SN SO SR SS ST SV SX SY SZ TC TD TF TG TH TJ TK TL TM TN TO TR TT TV TW TZ UA UG UM US UY UZ VA VC VE VG VI VN VU WF WS YE YT ZA ZM ZW";

/// Whether `code` is an assigned ISO 3166-1 alpha-2 code (uppercase).
pub(crate) fn valid_country(code: &str) -> bool {
    code.len() == 2 && ISO_3166.split(' ').any(|c| c == code)
}
/// An allowlist entry: a CIDR network or a single address (an IPv6 single means
/// its /64). Private ranges are valid entries (an office); they have no country.
pub(crate) fn parse_ip_entry(raw: &str) -> Option<Cidr> {
    let raw = raw.trim();
    if raw.is_empty() || raw.len() > 64 {
        return None;
    }
    if !raw.contains('/')
        && let Ok(IpAddr::V6(v6)) = raw.trim_matches(['[', ']']).parse::<IpAddr>()
        && v6.to_ipv4_mapped().is_none()
    {
        return Cidr::parse(&format!("{v6}/64"));
    }
    Cidr::parse(raw)
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Mode {
    Off,
    Unusual,
    Foreign,
}
impl Mode {
    fn parse(raw: &str) -> Option<Self> {
        match raw {
            "off" => Some(Mode::Off),
            "block-unusual-countries" => Some(Mode::Unusual),
            "block-foreign-countries" => Some(Mode::Foreign),
            _ => None,
        }
    }
}

#[derive(Clone, Debug)]
pub(crate) struct AccountPolicy {
    pub mode: Mode,
    pub countries: BTreeSet<String>,
    pub ips: Vec<Cidr>,
    /// `security.signin.accountGeoLookupFailure` (applies to both causes).
    pub lookup_failure: Failure,
    pub shared: Shared,
}
impl AccountPolicy {
    /// The effective setting for a failure cause: the stricter of the feature's
    /// own key and the shared default.
    fn failure_for(&self, outcome: &Outcome) -> Failure {
        self.lookup_failure.max(self.shared.for_outcome(outcome))
    }
}
/// The account's own earlier successful sign-ins, as countries.
#[derive(Clone, Debug, Default)]
pub(crate) struct Prior {
    pub countries: BTreeSet<String>,
}
#[derive(Debug, PartialEq, Eq)]
pub(crate) enum Decision {
    Allow(&'static str),
    /// No country for the source and the policy allows: the caller logs it.
    AllowUnresolved(&'static str),
    Deny(&'static str),
}

pub(crate) fn evaluate(
    policy: &AccountPolicy,
    ip: Option<IpAddr>,
    lookup: &CountryLookup,
    prior: &Prior,
) -> Decision {
    if policy.mode == Mode::Off {
        return Decision::Allow("off");
    }
    if ip.is_some_and(|ip| policy.ips.iter().any(|net| net.contains(ip))) {
        return Decision::Allow("ip-allowlist");
    }
    let outcome = geo_lookup::classify_with(ip, lookup.clone());
    let country = match &outcome {
        Outcome::Known(country) => Some(country.as_str()),
        // A local network has no country and is never blocked for that.
        Outcome::Local => return Decision::Allow("local-network"),
        Outcome::NoDatabase | Outcome::NotFound => None,
    };
    if let Some(country) = country
        && policy.countries.contains(country)
    {
        return Decision::Allow("country-allowlist");
    }
    let cause = outcome.cause();
    if country.is_none() {
        match policy.failure_for(&outcome) {
            Failure::Allow => return Decision::AllowUnresolved(cause),
            Failure::Block => {
                return Decision::Deny(if outcome == Outcome::NoDatabase {
                    "database-unavailable"
                } else {
                    "country-not-found"
                });
            }
            // Handled as a country that is on no list, below.
            Failure::Unknown => {}
        }
    }
    match policy.mode {
        Mode::Off => Decision::Allow("off"),
        Mode::Foreign => Decision::Deny(if country.is_some() { "foreign-country" } else { "unknown-country" }),
        Mode::Unusual => {
            if prior.countries.is_empty() {
                Decision::Allow("first-sign-in")
            } else if country.is_some_and(|c| prior.countries.contains(c)) {
                Decision::Allow("usual-country")
            } else {
                Decision::Deny(if country.is_some() { "unusual-country" } else { "unknown-country" })
            }
        }
    }
}

/// Build a policy from effective rows; `candidate` replaces one key's value (the
/// self-lockout check evaluates the policy a write would produce).
fn from_rows(
    rows: Vec<crate::settings_registry::Effective>,
    candidate: Option<(&str, &Value)>,
) -> Result<AccountPolicy, ()> {
    let mut get: std::collections::HashMap<String, Value> =
        rows.into_iter().map(|r| (r.key, r.value)).collect();
    if let Some((key, value)) = candidate {
        get.insert(key.to_owned(), value.clone());
    }
    let list = |key: &str| -> Vec<String> {
        get.get(key)
            .and_then(Value::as_array)
            .map(|a| {
                a.iter()
                    .filter_map(|v| v.as_str().map(str::to_owned))
                    .collect()
            })
            .unwrap_or_default()
    };
    let mode = get
        .get(KEYS[0])
        .and_then(Value::as_str)
        .and_then(Mode::parse)
        .ok_or(())?;
    Ok(AccountPolicy {
        mode,
        countries: list(KEYS[1])
            .into_iter()
            .chain(list(KEYS[3]))
            .filter(|c| valid_country(c))
            .collect(),
        ips: list(KEYS[2])
            .into_iter()
            .chain(list(KEYS[4]))
            .filter_map(|e| parse_ip_entry(&e))
            .collect(),
        lookup_failure: get
            .get(KEYS[5])
            .and_then(Value::as_str)
            .map_or(Failure::Allow, Failure::parse),
        shared: Shared {
            no_database: get
                .get(geo_lookup::NO_DATABASE_KEY)
                .and_then(Value::as_str)
                .map_or(Failure::Allow, Failure::parse),
            not_found: get
                .get(geo_lookup::NOT_FOUND_KEY)
                .and_then(Value::as_str)
                .map_or(Failure::Allow, Failure::parse),
        },
    })
}

async fn load(
    state: &AppState,
    subject: &SettingSubject<'_>,
    candidate: Option<(&str, &Value)>,
) -> Result<AccountPolicy, ()> {
    let rows =
        crate::settings_registry::effective_all(&state.store, subject, |d| wanted(d.key))
            .await
            .map_err(|_| ())?;
    from_rows(rows, candidate)
}

/// The countries the account has signed in from within
/// `security.signin.accountGeoHistoryDays` (see `account_geo_history`).
async fn prior(state: &AppState, account: &str) -> Prior {
    Prior {
        countries: crate::account_geo_history::usual_countries(&state.store, account).await,
    }
}

/// Called after credentials are proven and before the session is kept.
/// `Err(response)` refuses the sign-in (the caller deletes the session).
pub(crate) async fn check_signin(
    state: &AppState,
    account: &str,
    address: &str,
    device: &mw_store::SessionDevice,
    method: &str,
) -> Result<(), Response> {
    let subject = SettingSubject {
        personal_owner: None,
        account_id: account,
        domain: address.rsplit_once('@').map_or("", |(_, d)| d),
        address,
    };
    let policy = match load(state, &subject, None).await {
        Ok(p) => p,
        // Unreadable settings never lock an account out of its own sign-in.
        Err(()) => return Ok(()),
    };
    if policy.mode == Mode::Off {
        return Ok(());
    }
    let ip = device.ip.parse::<IpAddr>().ok();
    let lookup = country_lookup(ip);
    let prior = if policy.mode == Mode::Unusual {
        prior(state, account).await
    } else {
        Prior::default()
    };
    match evaluate(&policy, ip, &lookup, &prior) {
        Decision::Allow(_) => Ok(()),
        Decision::AllowUnresolved(cause) => {
            tracing::info!(target: "signin_geography", cause, "account sign-in policy: no country for the source; allowed by policy");
            Ok(())
        }
        Decision::Deny(rule) => {
            let event = mw_store::SigninEvent {
                id: uuid::Uuid::new_v4().to_string(),
                account_id: Some(account.to_owned()),
                username_attempted: address.to_owned(),
                ts: chrono::Utc::now().timestamp(),
                result: "failure".into(),
                method: method.into(),
                ip: device.ip.clone(),
                user_agent: String::new(),
                geo_country: match &lookup {
                    CountryLookup::Known(c) => Some(c.clone()),
                    _ => None,
                },
                reason: format!("account-geo-{rule}"),
            };
            crate::auth_logs::record(state, &event, mw_store::AuthSurface::Webmail, None).await;
            let mut response = (
                StatusCode::FORBIDDEN,
                Json(json!({
                    "code": "accountSigninGeographyDenied",
                    "error": "Sign-in is not permitted from this location.",
                    "notice": ACCURACY_NOTICE,
                })),
            )
                .into_response();
            response.headers_mut().insert(
                axum::http::header::CACHE_CONTROL,
                "no-store".parse().unwrap(),
            );
            Err(response)
        }
    }
}

/// A sign-in completed (the policy allowed it and the session was kept): note
/// its country in the account's durable history. Only while the account's
/// policy is `block-unusual-countries`, the one mode that reads the history.
pub(crate) async fn record_success(
    state: &AppState,
    account: &str,
    address: &str,
    device: &mw_store::SessionDevice,
) {
    let subject = SettingSubject {
        personal_owner: None,
        account_id: account,
        domain: address.rsplit_once('@').map_or("", |(_, d)| d),
        address,
    };
    let Ok(policy) = load(state, &subject, None).await else {
        return;
    };
    if policy.mode != Mode::Unusual {
        return;
    }
    if let Outcome::Known(country) = geo_lookup::classify(device.ip.parse::<IpAddr>().ok()) {
        crate::account_geo_history::record_success(&state.store, account, &country).await;
    }
}

/// `GET`/`DELETE /api/security/signin-geography/history`: the countries seen in
/// this account's successful sign-ins, and forgetting them.
pub(crate) async fn history_view(
    axum::extract::State(state): axum::extract::State<AppState>,
    headers: axum::http::HeaderMap,
) -> Response {
    let session = match crate::authed(&state, &headers).await {
        Ok(s) => s,
        Err(r) => return r,
    };
    private(Json(crate::account_geo_history::view(&state.store, &session.account_id).await).into_response())
}
pub(crate) async fn history_forget(
    axum::extract::State(state): axum::extract::State<AppState>,
    headers: axum::http::HeaderMap,
) -> Response {
    let session = match crate::authed(&state, &headers).await {
        Ok(s) => s,
        Err(r) => return r,
    };
    if !crate::account_geo_history::forget(&state.store, &session.account_id).await {
        return StatusCode::INTERNAL_SERVER_ERROR.into_response();
    }
    crate::audit::security_event(
        &state,
        crate::audit::SecurityEvent::user(&session.account_id, "signin-geography-history-forgotten"),
    )
    .await;
    private(Json(crate::account_geo_history::view(&state.store, &session.account_id).await).into_response())
}
fn private(mut r: Response) -> Response {
    r.headers_mut()
        .insert(axum::http::header::CACHE_CONTROL, "private, no-store".parse().unwrap());
    r
}

/// Self-lockout guard for a write to one of the user keys: the policy the write
/// would produce must still admit the current request. `Err(detail)` is the 409.
pub(crate) async fn guard_write(
    state: &AppState,
    subject: &SettingSubject<'_>,
    key: &str,
    value: &Value,
    current_ip: Option<IpAddr>,
) -> Result<(), String> {
    if !USER_KEYS.contains(&key) {
        return Ok(());
    }
    let Ok(policy) = load(state, subject, Some((key, value))).await else {
        return Ok(());
    };
    if policy.mode == Mode::Off {
        return Ok(());
    }
    let lookup = country_lookup(current_ip);
    let prior = if policy.mode == Mode::Unusual {
        prior(state, subject.account_id).await
    } else {
        Prior::default()
    };
    match evaluate(&policy, current_ip, &lookup, &prior) {
        Decision::Deny(_) => Err("This setting would block sign-in from your current address or country. Add your current IP address or country to the allowlist first.".into()),
        _ => Ok(()),
    }
}

/// The caller's own address and detected country, for the "add my current
/// IP/country" buttons.
pub(crate) fn context(ip: Option<IpAddr>) -> Value {
    let lookup = country_lookup(ip);
    json!({
        "ip": ip.map(source_label),
        "country": match &lookup { CountryLookup::Known(c) => Some(c.clone()), _ => None },
        "database": if matches!(lookup, CountryLookup::DatabaseUnavailable) { "unavailable" } else { "ready" },
        "notice": ACCURACY_NOTICE,
    })
}
/// The address as an allowlist entry: IPv4 as is, IPv6 as its /64.
fn source_label(ip: IpAddr) -> String {
    match ip.to_canonical() {
        IpAddr::V4(v4) => v4.to_string(),
        IpAddr::V6(v6) => {
            let s = v6.segments();
            format!("{:x}:{:x}:{:x}:{:x}::/64", s[0], s[1], s[2], s[3])
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn policy(mode: Mode) -> AccountPolicy {
        AccountPolicy {
            mode,
            countries: BTreeSet::new(),
            ips: Vec::new(),
            lookup_failure: Failure::Allow,
            shared: Shared::default(),
        }
    }
    fn ip(s: &str) -> Option<IpAddr> {
        Some(s.parse().unwrap())
    }
    fn known(c: &str) -> CountryLookup {
        CountryLookup::Known(c.into())
    }
    fn prior(cs: &[&str]) -> Prior {
        Prior {
            countries: cs.iter().map(|c| c.to_string()).collect(),
        }
    }

    #[test]
    fn off_allows_everything() {
        assert_eq!(
            evaluate(
                &policy(Mode::Off),
                ip("203.0.113.1"),
                &known("RU"),
                &prior(&["PT"])
            ),
            Decision::Allow("off")
        );
    }

    #[test]
    fn the_registry_defaults_are_off_and_empty() {
        let get = |k: &str| crate::settings_registry::lookup(k).unwrap().default_value();
        assert_eq!(get(KEYS[0]), json!("off"));
        for key in &KEYS[1..5] {
            assert_eq!(get(key), json!([]), "{key}");
        }
        assert_eq!(get(KEYS[5]), json!("allow"));
        assert_eq!(get(KEYS[6]), json!(365));
    }

    #[test]
    fn unusual_mode_allows_the_first_sign_in_and_known_countries_only() {
        let p = policy(Mode::Unusual);
        let d = |c: &str, pr: &Prior| evaluate(&p, ip("203.0.113.1"), &known(c), pr);
        assert_eq!(d("RU", &prior(&[])), Decision::Allow("first-sign-in"));
        assert_eq!(d("PT", &prior(&["PT"])), Decision::Allow("usual-country"));
        assert_eq!(d("RU", &prior(&["PT"])), Decision::Deny("unusual-country"));
    }

    #[test]
    fn foreign_mode_denies_outside_the_allowed_countries() {
        let mut p = policy(Mode::Foreign);
        assert_eq!(
            evaluate(&p, ip("203.0.113.1"), &known("PT"), &prior(&[])),
            Decision::Deny("foreign-country")
        );
        p.countries.insert("PT".into());
        assert_eq!(
            evaluate(&p, ip("203.0.113.1"), &known("PT"), &prior(&[])),
            Decision::Allow("country-allowlist")
        );
        assert_eq!(
            evaluate(&p, ip("203.0.113.1"), &known("DE"), &prior(&["DE"])),
            Decision::Deny("foreign-country"),
            "history does not matter in foreign mode"
        );
    }

    #[test]
    fn allowlists_win_over_both_modes() {
        for mode in [Mode::Unusual, Mode::Foreign] {
            let mut p = policy(mode);
            p.ips.push(parse_ip_entry("198.51.100.0/24").unwrap());
            p.countries.insert("DE".into());
            let pr = prior(&["PT"]);
            assert_eq!(
                evaluate(&p, ip("198.51.100.77"), &known("RU"), &pr),
                Decision::Allow("ip-allowlist")
            );
            assert_eq!(
                evaluate(&p, ip("203.0.113.1"), &known("DE"), &pr),
                Decision::Allow("country-allowlist")
            );
            assert_eq!(
                evaluate(
                    &p,
                    ip("198.51.100.77"),
                    &CountryLookup::DatabaseUnavailable,
                    &pr
                ),
                Decision::Allow("ip-allowlist"),
                "an allowlisted IP passes even with no country"
            );
        }
    }

    #[test]
    fn a_missing_country_is_allowed_and_flagged_unless_the_admin_says_deny() {
        let mut p = policy(Mode::Foreign);
        assert_eq!(
            evaluate(&p, ip("203.0.113.1"), &CountryLookup::Unknown, &prior(&[])),
            Decision::AllowUnresolved("not-found")
        );
        assert_eq!(
            evaluate(&p, ip("203.0.113.1"), &CountryLookup::DatabaseUnavailable, &prior(&[])),
            Decision::AllowUnresolved("no-database")
        );
        p.lookup_failure = Failure::Block;
        assert_eq!(
            evaluate(&p, ip("203.0.113.1"), &CountryLookup::Unknown, &prior(&[])),
            Decision::Deny("country-not-found")
        );
    }

    /// The lookup-failure matrix: cause x setting x mode, plus local networks.
    #[test]
    fn lookup_failure_matrix_per_cause_mode_and_setting() {
        use CountryLookup::{DatabaseUnavailable as NoDb, Unknown as NotFound};
        let src = ip("203.0.113.7");
        for mode in [Mode::Foreign, Mode::Unusual] {
            for (lookup, cause, deny_rule) in
                [(NoDb, "no-database", "database-unavailable"), (NotFound, "not-found", "country-not-found")]
            {
                let mut p = policy(mode);
                // Defaults allow.
                assert_eq!(evaluate(&p, src, &lookup, &prior(&["PT"])), Decision::AllowUnresolved(cause));
                // The shared setting applies to its own cause only.
                p.shared = if cause == "no-database" {
                    Shared { no_database: Failure::Block, not_found: Failure::Allow }
                } else {
                    Shared { no_database: Failure::Allow, not_found: Failure::Block }
                };
                assert_eq!(evaluate(&p, src, &lookup, &prior(&["PT"])), Decision::Deny(deny_rule));
                let other = if cause == "no-database" { NotFound } else { NoDb };
                assert!(matches!(evaluate(&p, src, &other, &prior(&["PT"])), Decision::AllowUnresolved(_)));
                // The feature's own key and the shared key combine as the stricter.
                p.shared = Shared::default();
                p.lookup_failure = Failure::Block;
                assert_eq!(evaluate(&p, src, &lookup, &prior(&["PT"])), Decision::Deny(deny_rule));
                // treat-as-unknown: handled as a country on no list.
                p.lookup_failure = Failure::Unknown;
                let expected = match mode {
                    Mode::Foreign => Decision::Deny("unknown-country"),
                    _ => Decision::Deny("unknown-country"),
                };
                assert_eq!(evaluate(&p, src, &lookup, &prior(&["PT"])), expected);
            }
        }
        // treat-as-unknown never blocks a first sign-in in unusual mode.
        let mut p = policy(Mode::Unusual);
        p.lookup_failure = Failure::Unknown;
        assert_eq!(evaluate(&p, src, &NotFound, &prior(&[])), Decision::Allow("first-sign-in"));
        // Allowlisted IPs still pass, whatever the failure setting.
        p.lookup_failure = Failure::Block;
        p.ips.push(parse_ip_entry("203.0.113.0/24").unwrap());
        assert_eq!(evaluate(&p, src, &NoDb, &prior(&["PT"])), Decision::Allow("ip-allowlist"));
    }

    #[test]
    fn private_and_loopback_addresses_are_a_local_network_and_never_blocked() {
        for mode in [Mode::Foreign, Mode::Unusual] {
            let mut p = policy(mode);
            p.lookup_failure = Failure::Block;
            p.shared = Shared { no_database: Failure::Block, not_found: Failure::Block };
            for a in ["10.0.0.5", "192.168.1.20", "127.0.0.1", "fd00::1", "169.254.1.1"] {
                for l in [CountryLookup::Unknown, CountryLookup::DatabaseUnavailable] {
                    assert_eq!(
                        evaluate(&p, ip(a), &l, &prior(&["PT"])),
                        Decision::Allow("local-network"),
                        "{a}"
                    );
                }
            }
        }
    }

    #[test]
    fn ip_entries_match_v4_v6_cidrs_and_single_v6_means_its_64() {
        let v4 = parse_ip_entry("192.0.2.0/24").unwrap();
        assert!(v4.contains("192.0.2.200".parse().unwrap()));
        assert!(!v4.contains("192.0.3.1".parse().unwrap()));
        let single = parse_ip_entry("203.0.113.5").unwrap();
        assert!(single.contains("203.0.113.5".parse().unwrap()));
        assert!(!single.contains("203.0.113.6".parse().unwrap()));
        let v6 = parse_ip_entry("2001:db8:1:2::7").unwrap();
        assert!(v6.contains("2001:db8:1:2:ffff::1".parse().unwrap()));
        assert!(!v6.contains("2001:db8:1:3::7".parse().unwrap()));
        let net = parse_ip_entry("2001:db8::/32").unwrap();
        assert!(net.contains("2001:db8:ffff::1".parse().unwrap()));
        assert!(!net.contains("2001:db9::1".parse().unwrap()));
        for bad in ["", "office", "300.1.1.1", "1.2.3.4/33", "::1/129", "1.2.3.4/x"] {
            assert!(parse_ip_entry(bad).is_none(), "{bad}");
        }
    }

    #[test]
    fn countries_must_be_assigned_iso_codes() {
        for ok in ["PT", "US", "GB", "DE"] {
            assert!(valid_country(ok), "{ok}");
        }
        for bad in ["pt", "XX", "ZZ", "USA", "P", "", "10", "AA"] {
            assert!(!valid_country(bad), "{bad}");
        }
        assert_eq!(ISO_3166.split(' ').count(), 249);
    }

    #[test]
    fn the_source_label_is_a_usable_allowlist_entry() {
        assert_eq!(source_label("203.0.113.9".parse().unwrap()), "203.0.113.9");
        let label = source_label("2001:db8:1:2:aaaa::9".parse().unwrap());
        assert_eq!(label, "2001:db8:1:2::/64");
        assert!(
            parse_ip_entry(&label)
                .unwrap()
                .contains("2001:db8:1:2::5".parse().unwrap())
        );
    }

    #[tokio::test]
    async fn precedence_user_value_admin_lock_and_the_self_lockout_policy() {
        use mw_store::settings_policy::{SettingPolicyRow, SettingScope, SettingValueRow};
        let store = mw_store::Store::open_in_memory(mw_store::ServerKey::generate())
            .await
            .unwrap();
        let load = |account: &'static str| {
            let store = store.clone();
            async move {
                let subject = SettingSubject {
                    personal_owner: None,
                    account_id: account,
                    domain: "example.org",
                    address: "u@example.org",
                };
                let rows = crate::settings_registry::effective_all(&store, &subject, |d| {
                    KEYS.contains(&d.key)
                })
                .await
                .unwrap();
                from_rows(rows, None).unwrap()
            }
        };
        let put = |account: &'static str, value: Value| {
            let store = store.clone();
            async move {
                store
                    .put_setting_value(&SettingValueRow {
                        scope: SettingScope::User,
                        scope_id: account.into(),
                        key: KEYS[0].into(),
                        value: value.to_string(),
                        locked: false,
                        updated_by: account.into(),
                        updated_at: 1,
                    })
                    .await
                    .unwrap();
            }
        };
        assert_eq!(load("acc-0").await.mode, Mode::Off, "off by default");
        put("acc-1", json!("block-foreign-countries")).await;
        assert_eq!(load("acc-1").await.mode, Mode::Foreign, "a user value applies");
        // A write that would lock the writer out is visible to the guard's evaluate.
        let mut candidate = load("acc-1").await;
        candidate.countries.insert("PT".into());
        assert_eq!(
            evaluate(&candidate, ip("203.0.113.1"), &known("DE"), &prior(&[])),
            Decision::Deny("foreign-country")
        );
        // The administrator forces the instance value and locks it.
        let row = SettingPolicyRow {
            key: KEYS[0].into(),
            global_value: Some(json!("block-unusual-countries").to_string()),
            allow_user_override: false,
            global_locked: true,
            updated_by: "admin".into(),
            updated_at: 2,
        };
        assert!(
            store
                .compare_setting_policy(KEYS[0], None, Some(&row))
                .await
                .unwrap()
        );
        put("acc-2", json!("off")).await;
        assert_eq!(
            load("acc-2").await.mode,
            Mode::Unusual,
            "a locked admin value beats the user's"
        );
    }
}
