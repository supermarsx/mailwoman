use super::*;

const REVISION: &str = "0123456789abcdef0123456789abcdef01234567";
const LOCK: &[u8] = br#"version = 4
[[package]]
name = "app"
version = "1.0.0"
dependencies = ["dep 1.0.0"]
[[package]]
name = "dep"
version = "1.0.0"
[[package]]
name = "dep"
version = "2.0.0"
"#;
fn inputs() -> Inputs<'static> {
    Inputs {
        version: "26.19.0",
        target: "x86_64-unknown-linux-gnu",
        revision: REVISION,
        release_tag: "",
        source_date_epoch: None,
        support_until: None,
        cargo_lock: LOCK,
        pnpm_lock: Some(b"lockfileVersion: '9.0'\n"),
        security_policy: b"# Security\n",
    }
}

#[test]
fn complete_inventory_is_deterministic_and_preserves_explicit_dependency_versions() {
    let built = generate(inputs()).unwrap();
    let again = generate(inputs()).unwrap();
    assert_eq!(built.metadata, again.metadata);
    assert_eq!(built.sbom, again.sbom);
    let facts: serde_json::Value = serde_json::from_slice(&built.metadata).unwrap();
    let bom: serde_json::Value = serde_json::from_slice(&built.sbom).unwrap();
    assert_eq!(facts["componentCount"], 3);
    assert_eq!(facts["sbomSha256"], hash(&built.sbom));
    assert_eq!(facts["revision"], REVISION);
    assert_eq!(facts["sourceDateEpoch"], serde_json::Value::Null);
    assert_eq!(facts["supportUntil"], serde_json::Value::Null);
    assert_eq!(facts["conformityAssessed"], false);
    let components = bom["components"].as_array().unwrap();
    let dep_one = components
        .iter()
        .find(|item| item["name"] == "dep" && item["version"] == "1.0.0")
        .unwrap();
    let app_ref = &components
        .iter()
        .find(|item| item["name"] == "app")
        .unwrap()["bom-ref"];
    let app_edge = bom["dependencies"]
        .as_array()
        .unwrap()
        .iter()
        .find(|edge| &edge["ref"] == app_ref)
        .unwrap();
    assert_eq!(
        app_edge["dependsOn"],
        serde_json::json!([dep_one["bom-ref"]])
    );
    assert_eq!(
        components[0]["properties"][1]["value"],
        "unverified-workspace-lock"
    );
    assert!(
        !String::from_utf8(built.metadata)
            .unwrap()
            .contains(env!("CARGO_MANIFEST_DIR"))
    );
}

#[test]
fn source_only_policy_and_web_lock_changes_update_hashes_without_rewriting_backend_inventory() {
    let original = generate(inputs()).unwrap();
    let mut changed = inputs();
    changed.pnpm_lock = Some(b"updated frontend lock\n");
    changed.security_policy = b"# Updated security policy\n";
    let changed = generate(changed).unwrap();
    assert_eq!(original.sbom, changed.sbom);
    let old: serde_json::Value = serde_json::from_slice(&original.metadata).unwrap();
    let new: serde_json::Value = serde_json::from_slice(&changed.metadata).unwrap();
    assert_ne!(old["pnpmLockSha256"], new["pnpmLockSha256"]);
    assert_ne!(old["securityPolicySha256"], new["securityPolicySha256"]);
    assert_eq!(old["cargoLockSha256"], new["cargoLockSha256"]);
    assert_eq!(changed.security_policy, b"# Updated security policy\n");
    assert_eq!(
        hash(b"abc"),
        "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"
    );
}

#[test]
fn archive_revision_is_explicit_and_conflicting_or_partial_provenance_is_rejected() {
    assert_eq!(
        resolve_revision(Some(REVISION), None, "").unwrap(),
        REVISION
    );
    assert_eq!(resolve_revision(None, None, REVISION).unwrap(), REVISION);
    assert_eq!(
        resolve_revision(None, Some(REVISION), "").unwrap(),
        REVISION
    );
    assert!(resolve_revision(None, None, "").is_err());
    assert!(resolve_revision(Some("0123456"), None, "").is_err());
    assert!(
        resolve_revision(
            Some(REVISION),
            Some("ffffffffffffffffffffffffffffffffffffffff"),
            ""
        )
        .is_err()
    );
    assert!(
        resolve_revision(
            Some(REVISION),
            None,
            "ffffffffffffffffffffffffffffffffffffffff"
        )
        .is_err()
    );
}

#[test]
fn real_dates_and_epochs_are_validated_without_inferred_support_commitments() {
    let mut input = inputs();
    input.support_until = Some("2028-02-29");
    input.source_date_epoch = Some("1795219200");
    let built = generate(input).unwrap();
    let facts: serde_json::Value = serde_json::from_slice(&built.metadata).unwrap();
    assert_eq!(facts["supportUntil"], "2028-02-29");
    assert_eq!(facts["sourceDateEpoch"], "1795219200");
    for invalid in ["2027-02-29", "2028-13-01", "not-a-date"] {
        let mut input = inputs();
        input.support_until = Some(invalid);
        assert!(generate(input).is_err());
    }
    for invalid in ["-1", "tomorrow", "18446744073709551616"] {
        let mut input = inputs();
        input.source_date_epoch = Some(invalid);
        assert!(generate(input).is_err());
    }
}

#[test]
fn missing_malformed_and_empty_sources_are_not_successful_empty_inventories() {
    for lock in [
        b"not = [valid".as_slice(),
        b"version = 4\n".as_slice(),
        b"package = []\n".as_slice(),
    ] {
        let mut input = inputs();
        input.cargo_lock = lock;
        assert!(generate(input).is_err());
    }
    let mut input = inputs();
    input.security_policy = b"";
    assert!(generate(input).is_err());
    let mut input = inputs();
    input.security_policy = &[0xff];
    assert!(generate(input).is_err());
}

#[test]
fn inventory_has_optional_scope_and_never_infers_compiled_membership() {
    let built = generate(inputs()).unwrap();
    let bom: serde_json::Value = serde_json::from_slice(&built.sbom).unwrap();
    for component in bom["components"].as_array().unwrap() {
        assert_eq!(component["scope"], "optional");
        assert_eq!(
            component["properties"][1]["value"],
            "unverified-workspace-lock"
        );
    }
    assert!(
        bom["metadata"]["properties"][0]["value"]
            .as_str()
            .unwrap()
            .contains("development and platform-specific")
    );
}
