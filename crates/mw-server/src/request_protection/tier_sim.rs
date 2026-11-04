//! Trust-tier effects for the Adaptive protection simulator.
//!
//! The tester picks a trust tier, a claimed user-agent class, a country (or
//! "not found" / "no database" / local network) and whether the request is an
//! admin-discovery probe. Every number below comes from the function the live
//! request path calls, not from a copy of its arithmetic:
//!
//! * request budget: `trust_tier::Policy::scale_limit`;
//! * sign-in cooldown: `Policy::scale_wait`, then
//!   `signin_cooldown::heightened_wait` under heightened mode;
//! * challenge difficulty: `signin_challenge::scaled_difficulty`;
//! * user-agent action: `Policy::ua_action`, then `request_protection::class_outcome`;
//! * unknown-country admin block: `admin_probe::geo_decision`;
//! * heightened mode: `signin_spike::pressure` (read only).
//!
//! Nothing here writes, counts or blocks anything.
use crate::trust_tier::{GeoRules, Policy, Tier, UA_CLASSES, UaAction};
use mw_engine::security::CountryLookup;
use serde::Deserialize;
use serde_json::{Value, json};

/// The extra inputs of a simulation (`tier` in the request body).
#[derive(Clone, Debug, Default, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(super) struct TierInput {
    /// `authenticated`, `known` or `unknown` (default).
    #[serde(default)]
    pub tier: Option<String>,
    /// A claimed user-agent class from `UA_CLASSES`, or `normal`.
    #[serde(default)]
    pub ua_class: Option<String>,
    /// ISO country code the lookup finds.
    #[serde(default)]
    pub country: Option<String>,
    /// `found` (default when a country is given), `notFound`, `noDatabase`, `local`.
    #[serde(default)]
    pub country_state: Option<String>,
    /// The request is an admin-discovery probe.
    #[serde(default)]
    pub admin_probe: bool,
}

pub(super) struct Inputs<'a> {
    pub trust: &'a Policy,
    pub shared: crate::geo_lookup::Shared,
    /// Budget of the chosen subject kind before the tier percentage (requests per window).
    pub subject_budget: Option<(u32, u32)>,
    /// Budget of the user-agent class dimension, when it applies.
    pub ua_budget: Option<(u32, u32)>,
    pub subject_kind: &'a str,
    pub cooldown_secs: u64,
    pub difficulty: u32,
    pub heightened: HeightenedInput,
}

/// A snapshot of heightened mode for one tier, so the maths can be tested
/// without the process-wide detector.
#[derive(Clone, Copy, Debug, Default)]
pub(super) struct HeightenedInput {
    pub on: bool,
    pub force_challenge: bool,
    pub pace_percent: Option<u32>,
}

pub(super) fn tier_of(raw: Option<&str>) -> Result<Tier, ()> {
    match raw.unwrap_or("unknown") {
        "authenticated" => Ok(Tier::Authenticated),
        "known" => Ok(Tier::Known),
        "unknown" => Ok(Tier::Unknown),
        _ => Err(()),
    }
}
fn tier_name(tier: Tier) -> &'static str {
    match tier {
        Tier::Authenticated => "authenticated",
        Tier::Known => "known",
        Tier::Unknown => "unknown",
    }
}
fn action_name(a: UaAction) -> &'static str {
    match a {
        UaAction::Allow => "allow",
        UaAction::Limit => "limit",
        UaAction::Block => "block",
    }
}

/// The lookup the tester describes.
pub(super) fn lookup_of(input: &TierInput) -> Result<(Option<std::net::IpAddr>, CountryLookup), ()> {
    let state = input
        .country_state
        .as_deref()
        .unwrap_or(if input.country.is_some() { "found" } else { "notFound" });
    // Documentation addresses stand in for the source: public (not local) and
    // never in a real database, so only the described lookup matters.
    let public: Option<std::net::IpAddr> = "203.0.113.50".parse().ok();
    match state {
        "found" => {
            let cc = input.country.as_deref().ok_or(())?;
            if !crate::account_signin_geo::valid_country(cc) {
                return Err(());
            }
            Ok((public, CountryLookup::Known(cc.to_owned())))
        }
        "notFound" => Ok((public, CountryLookup::Unknown)),
        "noDatabase" => Ok((public, CountryLookup::DatabaseUnavailable)),
        "local" => Ok(("192.168.1.10".parse().ok(), CountryLookup::Unknown)),
        _ => Err(()),
    }
}

/// The plain-language result. `seen` says whether a country has had a recent
/// successful sign-in (the live known-country table in production).
pub(super) fn effects(
    input: &TierInput,
    i: &Inputs<'_>,
    seen: impl Fn(&str) -> bool,
) -> Result<Value, ()> {
    let tier = tier_of(input.tier.as_deref())?;
    let t = i.trust;
    let percent = t.percent(tier);

    // Request budget (only an anonymous client and the user-agent dimension scale).
    let budget = |b: Option<(u32, u32)>| {
        b.map(|(base, window)| {
            json!({"base": base, "effective": t.scale_limit(base, tier), "windowSecs": window})
        })
    };
    let scales = i.subject_kind == "client";
    let client_budget = if scales { budget(i.subject_budget) } else { None };

    // Sign-in cooldown and challenge.
    let cooldown = t.scale_wait(i.cooldown_secs, tier);
    let heightened = i.heightened;
    let heightened_cooldown = heightened
        .pace_percent
        .map(|p| crate::signin_cooldown::heightened_wait(cooldown, p));
    let difficulty = crate::signin_challenge::scaled_difficulty(t, i.difficulty, tier);

    // User-agent class.
    let signed_in = tier == Tier::Authenticated;
    let ua = match input.ua_class.as_deref() {
        None | Some("normal") => None,
        Some(class) if UA_CLASSES.contains(&class) => {
            let configured = t.ua_action(class);
            Some((class, configured, crate::request_protection::class_outcome(configured, signed_in)))
        }
        Some(_) => return Err(()),
    };

    // Unknown-country admin-discovery block.
    let admin = if input.admin_probe {
        let (ip, lookup) = lookup_of(input)?;
        Some(admin_probe_effect(tier, ip, &lookup, &t.geo, &i.shared, &seen))
    } else {
        None
    };

    Ok(json!({
        "tier": tier_name(tier),
        "percent": percent,
        "requestBudget": client_budget,
        "userAgentBudget": ua.as_ref().filter(|(_, _, effective)| *effective != UaAction::Allow).and_then(|_| budget(i.ua_budget)),
        "signinCooldown": {
            "baseSecs": i.cooldown_secs,
            "effectiveSecs": cooldown,
            "heightenedSecs": heightened_cooldown,
        },
        "challenge": {
            "baseDifficulty": i.difficulty,
            "effectiveDifficulty": difficulty,
            "forcedByHeightened": heightened.force_challenge,
        },
        "userAgentAction": ua.map(|(class, configured, effective)| json!({
            "class": class,
            "configured": action_name(configured),
            "effective": action_name(effective),
            "signedInNeverBlocked": signed_in && configured == UaAction::Block,
        })),
        "heightened": {
            "on": heightened.on,
            "forcesChallenge": heightened.force_challenge,
            "pacePercent": heightened.pace_percent,
        },
        "adminProbe": admin,
    }))
}

fn admin_probe_effect(
    tier: Tier,
    ip: Option<std::net::IpAddr>,
    lookup: &CountryLookup,
    rules: &GeoRules,
    shared: &crate::geo_lookup::Shared,
    seen: &impl Fn(&str) -> bool,
) -> Value {
    use crate::trust_tier::CountryVerdict;
    let common = json!({
        "enabled": rules.enabled,
        "threshold": if rules.block_on_first { 1 } else { rules.threshold },
        "windowSecs": rules.window_secs,
        "blockSecs": rules.block_secs[0],
    });
    let mut out = common.as_object().cloned().unwrap_or_default();
    let (outcome, verdict) = crate::admin_probe::geo_decision(ip, lookup, rules, shared, seen);
    out.insert("cause".into(), json!(outcome.cause()));
    // The live gate returns before the country for a known source; an
    // authenticated administrator is not discovery.
    let result = if !rules.enabled {
        "disabled"
    } else if tier != Tier::Unknown {
        "notApplicable"
    } else {
        match verdict {
            None => "localNetwork",
            Some(CountryVerdict::Known) => "knownCountry",
            Some(CountryVerdict::Unresolved) => "unresolvedNotBlocked",
            Some(CountryVerdict::Unknown) => "counted",
        }
    };
    out.insert("result".into(), json!(result));
    Value::Object(out)
}

/// Read the process-wide heightened mode for `tier` (no writes).
pub(super) fn heightened_now(tier: Tier) -> HeightenedInput {
    let pressure = crate::signin_spike::pressure(tier);
    HeightenedInput {
        on: crate::signin_spike::is_heightened(),
        force_challenge: pressure.challenge,
        pace_percent: pressure.pace_percent,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::trust_tier::Policy;

    fn inputs(trust: &Policy) -> Inputs<'_> {
        Inputs {
            trust,
            shared: crate::geo_lookup::Shared::default(),
            subject_budget: Some((120, 60)),
            ua_budget: Some((30, 60)),
            subject_kind: "client",
            cooldown_secs: 3,
            difficulty: 2000,
            heightened: HeightenedInput::default(),
        }
    }
    fn input(tier: &str) -> TierInput {
        TierInput { tier: Some(tier.into()), ..Default::default() }
    }

    /// The simulator's numbers are the live functions' numbers, for every tier.
    #[test]
    fn budget_cooldown_and_difficulty_equal_the_live_decisions() {
        let mut trust = Policy::default();
        trust.unknown_percent = 40;
        trust.known_percent = 250;
        for (name, tier) in [
            ("authenticated", Tier::Authenticated),
            ("known", Tier::Known),
            ("unknown", Tier::Unknown),
        ] {
            let out = effects(&input(name), &inputs(&trust), |_| false).unwrap();
            assert_eq!(out["requestBudget"]["effective"], json!(trust.scale_limit(120, tier)), "{name}");
            assert_eq!(out["signinCooldown"]["effectiveSecs"], json!(trust.scale_wait(3, tier)), "{name}");
            assert_eq!(
                out["challenge"]["effectiveDifficulty"],
                json!(crate::signin_challenge::scaled_difficulty(&trust, 2000, tier)),
                "{name}"
            );
            assert_eq!(out["percent"], json!(trust.percent(tier)), "{name}");
        }
        // The documented shapes: a known source doubles the budget and halves waits.
        let trust = Policy::default();
        let known = effects(&input("known"), &inputs(&trust), |_| false).unwrap();
        assert_eq!(known["requestBudget"]["effective"], json!(240));
        assert_eq!(known["signinCooldown"]["effectiveSecs"], json!(2));
        assert_eq!(known["challenge"]["effectiveDifficulty"], json!(1000));
        let low = Policy { unknown_percent: 50, ..Policy::default() };
        let unknown = effects(&input("unknown"), &inputs(&low), |_| false).unwrap();
        assert_eq!(unknown["requestBudget"]["effective"], json!(60));
        assert_eq!(unknown["signinCooldown"]["effectiveSecs"], json!(6));
    }

    #[test]
    fn user_agent_actions_follow_the_trust_policy_and_signed_in_is_never_blocked() {
        let trust = Policy::default();
        for class in UA_CLASSES {
            for (name, tier) in [("unknown", Tier::Unknown), ("authenticated", Tier::Authenticated)] {
                let mut i = input(name);
                i.ua_class = Some(class.into());
                let out = effects(&i, &inputs(&trust), |_| false).unwrap();
                let configured = trust.ua_action(class);
                let live = crate::request_protection::class_outcome(configured, tier == Tier::Authenticated);
                assert_eq!(out["userAgentAction"]["configured"], json!(action_name(configured)));
                assert_eq!(out["userAgentAction"]["effective"], json!(action_name(live)), "{class} {name}");
            }
        }
        let mut i = input("authenticated");
        i.ua_class = Some("ai-training".into());
        let out = effects(&i, &inputs(&trust), |_| false).unwrap();
        assert_eq!(out["userAgentAction"]["configured"], "block");
        assert_eq!(out["userAgentAction"]["effective"], "limit");
        assert_eq!(out["userAgentAction"]["signedInNeverBlocked"], true);
        let mut i = input("unknown");
        i.ua_class = Some("nonsense".into());
        assert!(effects(&i, &inputs(&trust), |_| false).is_err());
        assert!(effects(&input("martian"), &inputs(&trust), |_| false).is_err());
    }

    #[test]
    fn heightened_mode_effects_come_from_the_shared_helpers() {
        let trust = Policy::default();
        let mut i = inputs(&trust);
        i.heightened = HeightenedInput { on: true, force_challenge: true, pace_percent: Some(300) };
        let out = effects(&input("unknown"), &i, |_| false).unwrap();
        assert_eq!(
            out["signinCooldown"]["heightenedSecs"],
            json!(crate::signin_cooldown::heightened_wait(trust.scale_wait(3, Tier::Unknown), 300))
        );
        assert_eq!(out["signinCooldown"]["heightenedSecs"], 9);
        assert_eq!(out["challenge"]["forcedByHeightened"], true);
        assert!(effects(&input("unknown"), &inputs(&trust), |_| false).unwrap()["signinCooldown"]["heightenedSecs"].is_null());
    }

    #[test]
    fn the_admin_probe_block_matches_the_live_country_decision() {
        let trust = Policy::default();
        let probe = |tier: &str, country: Option<&str>, state: &str, seen: bool, rules_fail_unknown: bool| {
            let mut t = trust.clone();
            t.geo.lookup_failure_unknown = rules_fail_unknown;
            let mut i = input(tier);
            i.admin_probe = true;
            i.country = country.map(str::to_owned);
            i.country_state = Some(state.into());
            let out = effects(&i, &inputs(&t), |_| seen).unwrap();
            // The same inputs through the live decision.
            let (ip, lookup) = lookup_of(&i).unwrap();
            let (_, verdict) = crate::admin_probe::geo_decision(ip, &lookup, &t.geo, &crate::geo_lookup::Shared::default(), |_| seen);
            (out["adminProbe"].clone(), verdict)
        };
        use crate::trust_tier::CountryVerdict::*;
        let (v, live) = probe("unknown", Some("RU"), "found", false, false);
        assert_eq!((v["result"].as_str(), live), (Some("counted"), Some(Unknown)));
        assert_eq!((v["threshold"].clone(), v["blockSecs"].clone(), v["windowSecs"].clone()), (json!(3), json!(3600), json!(600)));
        let (v, live) = probe("unknown", Some("PT"), "found", true, false);
        assert_eq!((v["result"].as_str(), live), (Some("knownCountry"), Some(Known)));
        let (v, live) = probe("unknown", None, "notFound", false, false);
        assert_eq!((v["result"].as_str(), live), (Some("unresolvedNotBlocked"), Some(Unresolved)));
        assert_eq!(v["cause"], "not-found");
        let (v, live) = probe("unknown", None, "notFound", false, true);
        assert_eq!((v["result"].as_str(), live), (Some("counted"), Some(Unknown)));
        let (v, live) = probe("unknown", None, "noDatabase", false, false);
        assert_eq!((v["result"].as_str(), live, v["cause"].as_str()), (Some("unresolvedNotBlocked"), Some(Unresolved), Some("no-database")));
        let (v, live) = probe("unknown", None, "local", false, true);
        assert_eq!((v["result"].as_str(), live), (Some("localNetwork"), None), "a private source is never a stranger");
        let (v, _) = probe("known", Some("RU"), "found", false, false);
        assert_eq!(v["result"], "notApplicable");
        let (v, _) = probe("authenticated", Some("RU"), "found", false, false);
        assert_eq!(v["result"], "notApplicable");
        // block-on-first reports a threshold of one.
        let mut t = Policy::default();
        t.geo.block_on_first = true;
        let mut i = input("unknown");
        i.admin_probe = true;
        i.country = Some("RU".into());
        assert_eq!(effects(&i, &inputs(&t), |_| false).unwrap()["adminProbe"]["threshold"], 1);
    }

    #[test]
    fn invalid_country_inputs_are_refused() {
        let mut i = input("unknown");
        i.admin_probe = true;
        i.country = Some("XX".into());
        i.country_state = Some("found".into());
        assert!(effects(&i, &inputs(&Policy::default()), |_| false).is_err());
        i.country = Some("PT".into());
        i.country_state = Some("sideways".into());
        assert!(effects(&i, &inputs(&Policy::default()), |_| false).is_err());
    }
}
