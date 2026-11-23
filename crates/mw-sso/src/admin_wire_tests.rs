//! Tests of the `/admin/sso` wire contract. The `tests/fixtures/admin-sso/*.json`
//! files are shared with the web client's test (`apps/web/src/screens/Admin/Sso/
//! Sso.test.tsx`), which asserts the console builds exactly `console-*.json` and
//! reads `list-*.json` back into the same form: the two sides cannot drift apart
//! without one of the two tests failing.

use super::*;
use crate::{OidcProvider, SamlProvider, SsoBackend, SsoLogin, SsoScope};

const CONSOLE_OIDC: &str = include_str!("../tests/fixtures/admin-sso/console-oidc.json");
const CONSOLE_SAML: &str = include_str!("../tests/fixtures/admin-sso/console-saml.json");
const LIST_OIDC: &str = include_str!("../tests/fixtures/admin-sso/list-oidc.json");
const LIST_SAML: &str = include_str!("../tests/fixtures/admin-sso/list-saml.json");
const TS: &str = "2026-11-23T00:00:00Z";

fn value(s: &str) -> Value {
    serde_json::from_str(s).unwrap()
}

/// What the server stores and then lists for a validated upsert (the
/// `sso_config` columns), mirroring `mw-server` `admin_sso::upsert` + `list`.
fn stored_then_listed(v: &ValidUpsert, has_secret: bool) -> (String, Value) {
    let config_json = serde_json::to_string(&v.config).unwrap();
    let claim_map_json = serde_json::to_string(&v.claim_map).unwrap();
    let listed = backend_view(&BackendView {
        id: &v.id,
        kind: v.config.kind().as_db(),
        display_name: &v.display_name,
        scope: &v.scope,
        enabled: v.enabled,
        config_json: &config_json,
        claim_map_json: &claim_map_json,
        has_secret,
        created_at: TS,
        updated_at: TS,
    });
    (config_json, listed)
}

fn backend(v: &ValidUpsert, config_json: &str) -> SsoBackend {
    // Sign-in parses the stored column exactly like this (mw-server sso.rs
    // `build_entry_from_row`).
    let config: SsoConfig = serde_json::from_str(config_json).unwrap();
    SsoBackend {
        id: v.id.clone(),
        kind: config.kind(),
        display_name: v.display_name.clone(),
        scope: SsoScope::parse(&v.scope),
        enabled: v.enabled,
        config,
        claim_map: v.claim_map.clone(),
        secret: v.secret.clone().map(String::into_bytes),
    }
}

fn invalid_fields(body: Value) -> Vec<String> {
    match parse_upsert(body) {
        Err(UpsertError::Invalid(f)) => f.into_iter().map(|f| f.field).collect(),
        other => panic!("expected Invalid, got {other:?}"),
    }
}

#[test]
fn console_oidc_payload_round_trips_to_a_usable_provider() {
    let v = parse_upsert(value(CONSOLE_OIDC)).expect("console payload is valid");
    let SsoConfig::Oidc(c) = &v.config else {
        panic!("oidc")
    };
    assert_eq!(c.issuer_url, "https://idp.example.org/realms/staff");
    assert_eq!(c.client_id, "mailwoman");
    assert_eq!(
        c.redirect_url,
        "https://mail.example.org/api/sso/corp-oidc/callback"
    );
    assert_eq!(c.scopes, vec!["openid", "email", "profile"]);
    assert_eq!(c.first_login_policy, FirstLoginPolicy::AutoCreate);
    assert_eq!(v.claim_map.groups, None);
    assert_eq!(v.secret.as_deref(), Some("fixture-client-secret"));

    let (config_json, listed) = stored_then_listed(&v, true);
    // The stored column keeps the storage (snake_case) names sign-in reads.
    assert!(config_json.contains(r#""issuer_url":"https://idp.example.org/realms/staff""#));
    assert!(config_json.contains(r#""first_login_policy":"auto_create""#));
    assert_eq!(listed, value(LIST_OIDC));
    // The read side never carries the secret.
    assert!(!listed.to_string().contains("fixture-client-secret"));
    // The stored row constructs a sign-in provider.
    OidcProvider::new(backend(&v, &config_json)).expect("provider builds");
}

#[tokio::test]
async fn console_saml_payload_round_trips_to_a_usable_provider() {
    let v = parse_upsert(value(CONSOLE_SAML)).expect("console payload is valid");
    let SsoConfig::Saml(c) = &v.config else {
        panic!("saml")
    };
    assert_eq!(c.idp_sso_url, "https://idp.example.org/saml/sso");
    assert_eq!(
        c.idp_slo_url.as_deref(),
        Some("https://idp.example.org/saml/slo")
    );
    assert_eq!(
        c.sp_entity_id,
        "https://mail.example.org/api/sso/corp-saml/metadata"
    );
    assert_eq!(c.acs_url, "https://mail.example.org/api/sso/corp-saml/acs");
    assert_eq!(c.idp_signing_certs_pem.len(), 1);
    assert!(c.want_assertions_signed);
    assert_eq!(v.scope, "domain:example.org");
    assert_eq!(v.claim_map.groups.as_deref(), Some("memberOf"));

    let (config_json, listed) = stored_then_listed(&v, false);
    assert!(config_json.contains(r#""idp_sso_url":"https://idp.example.org/saml/sso""#));
    assert_eq!(listed, value(LIST_SAML));
    let p = SamlProvider::new(backend(&v, &config_json)).expect("provider builds");
    let begin = p.begin(None).await.expect("AuthnRequest builds");
    assert!(begin.url.starts_with("https://idp.example.org/saml/sso?"));
}

#[test]
fn a_listed_row_posted_back_unchanged_is_accepted() {
    // The console's enable/disable toggle and API scripts post a GET row back.
    for list in [LIST_OIDC, LIST_SAML] {
        let v = parse_upsert(value(list)).expect("listed row is a valid upsert");
        assert_eq!(v.secret, None, "no secret: the stored one is kept");
    }
}

#[test]
fn snake_case_names_stay_accepted() {
    // A stored config (snake_case) posted as-is, e.g. by an older script.
    let body = json!({
        "id": "corp-oidc", "display_name": "Acme", "enabled": true,
        "config": { "kind": "oidc", "issuer_url": "https://idp.example.org",
            "client_id": "c", "redirect_url": "https://mail.example.org/cb",
            "scopes": ["openid"], "first_login_policy": "auto_create" },
        "claim_map": { "email": "email" }
    });
    let v = parse_upsert(body).unwrap();
    let SsoConfig::Oidc(c) = v.config else {
        panic!()
    };
    assert_eq!(c.issuer_url, "https://idp.example.org");
    assert_eq!(c.first_login_policy, FirstLoginPolicy::AutoCreate);
}

#[test]
fn empty_issuer_is_rejected_with_the_field_named() {
    let mut body = value(CONSOLE_OIDC);
    body["config"]["issuerUrl"] = json!("");
    assert_eq!(invalid_fields(body), vec!["config.issuerUrl"]);

    let mut body = value(CONSOLE_OIDC);
    body["config"]["issuerUrl"] = json!("not a url");
    assert_eq!(invalid_fields(body), vec!["config.issuerUrl"]);

    let mut body = value(CONSOLE_OIDC);
    body["config"]["issuerUrl"] = json!("https://idp.example.org/?x=1");
    assert_eq!(invalid_fields(body), vec!["config.issuerUrl"]);
}

#[test]
fn every_problem_is_listed() {
    let body = json!({
        "id": "Bad Id", "displayName": " ", "scope": "domain:",
        "config": { "kind": "oidc", "scopes": ["email"] }
    });
    assert_eq!(
        invalid_fields(body),
        vec![
            "id",
            "displayName",
            "scope",
            "config.issuerUrl",
            "config.clientId",
            "config.redirectUrl",
            "config.scopes"
        ]
    );
    let body = json!({ "id": "s", "displayName": "S", "config": { "kind": "saml" } });
    assert_eq!(
        invalid_fields(body),
        vec![
            "config.idpSsoUrl",
            "config.idpSigningCertsPem",
            "config.spEntityId",
            "config.acsUrl"
        ]
    );
}

#[test]
fn unknown_fields_are_refused_not_dropped() {
    let mut body = value(CONSOLE_OIDC);
    body["config"]["issuer"] = json!("https://idp.example.org");
    let Err(UpsertError::Malformed(m)) = parse_upsert(body) else {
        panic!()
    };
    assert!(m.contains("issuer"), "{m}");

    let mut body = value(CONSOLE_OIDC);
    body["clientSecret"] = json!("x");
    assert!(matches!(parse_upsert(body), Err(UpsertError::Malformed(_))));

    let mut body = value(CONSOLE_OIDC);
    body["claimMap"]["mail"] = json!("x");
    assert!(matches!(parse_upsert(body), Err(UpsertError::Malformed(_))));

    let mut body = value(CONSOLE_SAML);
    body["config"]["wantSigned"] = json!(false);
    assert!(matches!(parse_upsert(body), Err(UpsertError::Malformed(_))));
}

#[test]
fn both_spellings_of_one_field_is_refused() {
    let mut body = value(CONSOLE_OIDC);
    body["config"]["issuer_url"] = json!("https://other.example.org");
    assert!(matches!(parse_upsert(body), Err(UpsertError::Malformed(_))));
}

#[test]
fn top_level_kind_must_match_config() {
    let mut body = value(LIST_OIDC);
    body["kind"] = json!("saml");
    assert_eq!(invalid_fields(body), vec!["kind"]);
}

#[test]
fn stored_rows_with_an_empty_issuer_are_flagged_on_the_read_side() {
    // A row written by the console before this fix: every protocol field empty.
    let listed = backend_view(&BackendView {
        id: "corp-oidc",
        kind: "oidc",
        display_name: "Acme",
        scope: "deployment",
        enabled: true,
        config_json: r#"{"kind":"oidc","issuer_url":"","client_id":"","redirect_url":"","scopes":["openid"],"first_login_policy":"allowlist"}"#,
        claim_map_json: "{}",
        has_secret: true,
        created_at: TS,
        updated_at: TS,
    });
    let fields: Vec<&str> = listed["problems"]
        .as_array()
        .unwrap()
        .iter()
        .map(|p| p["field"].as_str().unwrap())
        .collect();
    assert_eq!(
        fields,
        vec!["config.issuerUrl", "config.clientId", "config.redirectUrl"]
    );
    assert_eq!(listed["hasSecret"], json!(true));
    assert!(listed.get("secret").is_none());
    // Posting that row back is refused rather than stored again.
    assert!(matches!(parse_upsert(listed), Err(UpsertError::Invalid(_))));
}

#[test]
fn unreadable_stored_config_lists_as_null() {
    let listed = backend_view(&BackendView {
        id: "x",
        kind: "oidc",
        display_name: "X",
        scope: "deployment",
        enabled: false,
        config_json: "not json",
        claim_map_json: "{}",
        has_secret: false,
        created_at: TS,
        updated_at: TS,
    });
    assert_eq!(listed["config"], Value::Null);
}

#[test]
fn saml_security_fields_map_both_ways() {
    // Every security-relevant SAML value survives console -> store -> list.
    let v = parse_upsert(value(CONSOLE_SAML)).unwrap();
    let (config_json, listed) = stored_then_listed(&v, false);
    let stored: SsoConfig = serde_json::from_str(&config_json).unwrap();
    let SsoConfig::Saml(c) = stored else { panic!() };
    assert!(c.want_assertions_signed);
    assert!(!c.want_encrypted);
    assert_eq!(
        c.sp_entity_id,
        "https://mail.example.org/api/sso/corp-saml/metadata"
    );
    assert_eq!(c.acs_url, "https://mail.example.org/api/sso/corp-saml/acs");
    assert_eq!(c.idp_sso_url, "https://idp.example.org/saml/sso");
    assert_eq!(
        c.idp_signing_certs_pem,
        vec!["-----BEGIN CERTIFICATE-----\nMIIBfixture\n-----END CERTIFICATE-----"]
    );
    let console = value(CONSOLE_SAML);
    for k in [
        "wantAssertionsSigned",
        "wantEncrypted",
        "spEntityId",
        "acsUrl",
        "idpSsoUrl",
        "idpSigningCertsPem",
    ] {
        assert_eq!(listed["config"][k], console["config"][k], "{k}");
    }
    assert_eq!(listed["scope"], console["scope"]);
}

#[test]
fn omitted_want_assertions_signed_means_signed() {
    let mut body = value(CONSOLE_SAML);
    body["config"]
        .as_object_mut()
        .unwrap()
        .remove("wantAssertionsSigned");
    let v = parse_upsert(body).expect("omission is valid and means signed");
    let SsoConfig::Saml(c) = &v.config else {
        panic!()
    };
    assert!(c.want_assertions_signed);
    let (config_json, _) = stored_then_listed(&v, false);
    assert!(config_json.contains(r#""want_assertions_signed":true"#));
}

#[test]
fn a_saml_provider_that_accepts_unsigned_assertions_is_not_saved() {
    for name in ["wantAssertionsSigned", "want_assertions_signed"] {
        let mut body = value(CONSOLE_SAML);
        let cfg = body["config"].as_object_mut().unwrap();
        cfg.remove("wantAssertionsSigned");
        cfg.insert(name.into(), json!(false));
        let Err(UpsertError::Invalid(f)) = parse_upsert(body) else {
            panic!("{name}")
        };
        assert_eq!(f.len(), 1);
        assert_eq!(f[0].field, "config.wantAssertionsSigned");
        assert!(
            f[0].message
                .contains("unsigned SAML assertions are not accepted")
        );
    }
}

#[test]
fn stored_saml_rows_that_accept_unsigned_assertions_are_flagged() {
    // Explicitly false is flagged. (A row missing the key reads as true since the
    // storage default was fixed; see `missing_want_assertions_signed_reads_as_signed`.)
    {
        let config_json = r#"{"kind":"saml","sp_entity_id":"https://m.example/sp","acs_url":"https://m.example/acs","idp_sso_url":"https://idp.example/sso","idp_signing_certs_pem":["PEM"],"want_assertions_signed":false,"nameid_format":"x"}"#;
        let listed = backend_view(&BackendView {
            id: "s",
            kind: "saml",
            display_name: "S",
            scope: "deployment",
            enabled: true,
            config_json,
            claim_map_json: "{}",
            has_secret: false,
            created_at: TS,
            updated_at: TS,
        });
        assert_eq!(listed["config"]["wantAssertionsSigned"], json!(false));
        let fields: Vec<&str> = listed["problems"]
            .as_array()
            .unwrap()
            .iter()
            .map(|p| p["field"].as_str().unwrap())
            .collect();
        assert_eq!(fields, vec!["config.wantAssertionsSigned"], "{config_json}");
    }
}

#[test]
fn missing_want_assertions_signed_reads_as_signed() {
    let listed = backend_view(&BackendView {
        id: "s",
        kind: "saml",
        display_name: "S",
        scope: "deployment",
        enabled: true,
        config_json: r#"{"kind":"saml","sp_entity_id":"https://m.example/sp","acs_url":"https://m.example/acs","idp_sso_url":"https://idp.example/sso","idp_signing_certs_pem":["PEM"],"nameid_format":"x"}"#,
        claim_map_json: "{}",
        has_secret: false,
        created_at: TS,
        updated_at: TS,
    });
    assert_eq!(listed["config"]["wantAssertionsSigned"], json!(true));
    assert_eq!(listed["problems"], json!([]));
}

#[test]
fn require_verified_email_defaults_on_and_round_trips() {
    // The console sends `true` unless an opt-out was stored; omitted also means on.
    let v = parse_upsert(value(CONSOLE_OIDC)).unwrap();
    let SsoConfig::Oidc(c) = &v.config else {
        panic!()
    };
    assert!(c.require_verified_email);
    // An explicit opt-out (either spelling) is stored and listed back.
    for name in ["requireVerifiedEmail", "require_verified_email"] {
        let mut body = value(CONSOLE_OIDC);
        let cfg = body["config"].as_object_mut().unwrap();
        cfg.remove("requireVerifiedEmail");
        cfg.insert(name.into(), json!(false));
        let v = parse_upsert(body).unwrap();
        let (config_json, listed) = stored_then_listed(&v, true);
        assert!(
            config_json.contains(r#""require_verified_email":false"#),
            "{name}"
        );
        assert_eq!(listed["config"]["requireVerifiedEmail"], json!(false));
        assert!(parse_upsert(listed).is_ok(), "a listed opt-out posts back");
    }
    // A stored row from before the field existed reads as on.
    let old: SsoConfig = serde_json::from_str(
        r#"{"kind":"oidc","issuer_url":"https://idp.example.org","client_id":"c","redirect_url":"https://m.example/cb"}"#,
    )
    .unwrap();
    let SsoConfig::Oidc(c) = old else { panic!() };
    assert!(c.require_verified_email);
}

#[test]
fn problems_carry_a_stable_code() {
    let mut body = value(CONSOLE_OIDC);
    body["config"]["issuerUrl"] = json!("");
    body["config"]["redirectUrl"] = json!("not a url");
    let Err(e) = parse_upsert(body) else { panic!() };
    let codes: Vec<(String, String)> = match &e {
        UpsertError::Invalid(f) => f
            .iter()
            .map(|f| (f.field.clone(), f.code.clone()))
            .collect(),
        _ => panic!(),
    };
    assert_eq!(
        codes,
        vec![
            ("config.issuerUrl".to_string(), "required".to_string()),
            ("config.redirectUrl".to_string(), "invalid_url".to_string())
        ]
    );
    assert_eq!(e.body()["fields"][0]["code"], json!("required"));
    // A GET row with the server-added signInReady posts back.
    let mut row = value(LIST_OIDC);
    row["signInReady"] = json!(true);
    assert!(parse_upsert(row).is_ok());
}

#[test]
fn console_opt_out_of_verified_email_round_trips() {
    // Shared with the web editor test: the console's body with the switch off.
    let body = value(include_str!(
        "../tests/fixtures/admin-sso/console-oidc-unverified-email.json"
    ));
    let v = parse_upsert(body).expect("valid");
    let SsoConfig::Oidc(c) = &v.config else {
        panic!()
    };
    assert!(!c.require_verified_email);
    let (_, listed) = stored_then_listed(&v, true);
    assert_eq!(listed["config"]["requireVerifiedEmail"], json!(false));
    assert_eq!(listed["problems"], json!([]));
}
