//! Pure build-inventory generation. No filesystem, Git, environment or clock reads.
//! Source discovery belongs to the thin executable package's build script.
#![forbid(unsafe_code)]

use serde_json::json;
use sha2::{Digest, Sha256};

pub struct Inputs<'a> {
    pub version: &'a str,
    pub target: &'a str,
    pub revision: &'a str,
    pub release_tag: &'a str,
    pub source_date_epoch: Option<&'a str>,
    pub support_until: Option<&'a str>,
    pub cargo_lock: &'a [u8],
    pub pnpm_lock: Option<&'a [u8]>,
    pub security_policy: &'a [u8],
}

pub struct Artifacts {
    pub metadata: Vec<u8>,
    pub sbom: Vec<u8>,
    pub security_policy: Vec<u8>,
}

pub fn hash(bytes: &[u8]) -> String {
    Sha256::digest(bytes)
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect()
}

pub fn valid_revision(value: &str) -> bool {
    value.len() == 40 && value.bytes().all(|byte| byte.is_ascii_hexdigit())
}

/// Resolve immutable provenance without guessing a revision for source archives.
pub fn resolve_revision(
    explicit: Option<&str>,
    legacy: Option<&str>,
    git: &str,
) -> Result<String, String> {
    let mut selected: Option<String> = None;
    for value in [explicit, legacy, (!git.is_empty()).then_some(git)]
        .into_iter()
        .flatten()
    {
        if !valid_revision(value) {
            return Err("Build revision must be a full 40-character hexadecimal commit".into());
        }
        let value = value.to_ascii_lowercase();
        if selected.as_ref().is_some_and(|previous| previous != &value) {
            return Err("Explicit build revision, legacy build SHA and Git HEAD disagree".into());
        }
        selected = Some(value);
    }
    selected.ok_or_else(|| "Source archive has no Git provenance; set MW_BUILD_REVISION to its immutable source commit".into())
}

fn encoded(text: &str) -> String {
    text.bytes()
        .map(|byte| {
            if byte.is_ascii_alphanumeric() || b".-_~".contains(&byte) {
                (byte as char).to_string()
            } else {
                format!("%{byte:02X}")
            }
        })
        .collect()
}

fn valid_date(text: &str) -> bool {
    if text.len() != 10
        || text.as_bytes()[4] != b'-'
        || text.as_bytes()[7] != b'-'
        || !text
            .chars()
            .enumerate()
            .all(|(i, c)| i == 4 || i == 7 || c.is_ascii_digit())
    {
        return false;
    }
    let (Ok(year), Ok(month), Ok(day)) = (
        text[..4].parse::<u32>(),
        text[5..7].parse::<usize>(),
        text[8..].parse::<u32>(),
    ) else {
        return false;
    };
    let leap = year % 4 == 0 && (year % 100 != 0 || year % 400 == 0);
    let days = [
        31,
        if leap { 29 } else { 28 },
        31,
        30,
        31,
        30,
        31,
        31,
        30,
        31,
        30,
        31,
    ];
    year >= 1970 && (1..=12).contains(&month) && day >= 1 && day <= days[month - 1]
}

pub fn generate(input: Inputs<'_>) -> Result<Artifacts, String> {
    if input.version.is_empty() || input.target.is_empty() || !valid_revision(input.revision) {
        return Err(
            "Product inventory requires version, target and an immutable 40-character revision"
                .into(),
        );
    }
    if input
        .source_date_epoch
        .is_some_and(|value| value.parse::<u64>().is_err())
    {
        return Err("SOURCE_DATE_EPOCH must be an unsigned Unix timestamp".into());
    }
    let support = input.support_until.unwrap_or_default();
    if !support.is_empty() && !valid_date(support) {
        return Err("MW_PRODUCT_SUPPORT_UNTIL must be YYYY-MM-DD".into());
    }
    std::str::from_utf8(input.security_policy).map_err(|_| "SECURITY.md must be UTF-8")?;
    let lock = std::str::from_utf8(input.cargo_lock).map_err(|_| "Cargo.lock must be UTF-8")?;
    let parsed: toml::Value =
        toml::from_str(lock).map_err(|error| format!("Invalid Cargo.lock: {error}"))?;
    let packages = parsed
        .get("package")
        .and_then(toml::Value::as_array)
        .ok_or("Cargo.lock package inventory missing")?;
    if packages.is_empty() || input.security_policy.is_empty() {
        return Err("Complete package inventory and SECURITY.md are required".into());
    }
    if packages.iter().any(|package| {
        package.get("name").and_then(toml::Value::as_str).is_none()
            || package
                .get("version")
                .and_then(toml::Value::as_str)
                .is_none()
    }) {
        return Err("Cargo.lock package name/version missing".into());
    }
    let reference = |package: &toml::Value| {
        format!(
            "cargo:{}@{}:{}",
            package["name"].as_str().unwrap(),
            package["version"].as_str().unwrap(),
            &hash(
                package
                    .get("source")
                    .and_then(|value| value.as_str())
                    .unwrap_or("workspace")
                    .as_bytes()
            )[..12]
        )
    };
    let mut components = Vec::new();
    let mut edges = Vec::new();
    for package in packages {
        let name = package["name"].as_str().unwrap();
        let version = package["version"].as_str().unwrap();
        let source = package
            .get("source")
            .and_then(|value| value.as_str())
            .unwrap_or("workspace");
        let mut item = json!({"type":"library","bom-ref":reference(package),"name":name,"version":version,"purl":format!("pkg:cargo/{}@{}",encoded(name),encoded(version)),"scope":"optional","properties":[{"name":"mailwoman:inventory-source","value":source},{"name":"mailwoman:artifact-membership","value":"unverified-workspace-lock"}]});
        if let Some(checksum) = package.get("checksum").and_then(|value| value.as_str()) {
            item["hashes"] = json!([{"alg":"SHA-256","content":checksum}]);
            item["properties"].as_array_mut().unwrap().push(json!({"name":"mailwoman:checksum-kind","value":"Cargo registry archive checksum; compiled or vendored contents are not verified"}));
        }
        let dependencies: Vec<String> = package
            .get("dependencies")
            .and_then(|value| value.as_array())
            .into_iter()
            .flatten()
            .filter_map(|dependency| dependency.as_str())
            .flat_map(|dependency| {
                let parts: Vec<_> = dependency.split_whitespace().collect();
                packages
                    .iter()
                    .filter(move |candidate| {
                        candidate["name"].as_str() == parts.first().copied()
                            && (parts.get(1).is_none()
                                || candidate["version"].as_str() == parts.get(1).copied())
                    })
                    .map(&reference)
            })
            .collect();
        edges.push(json!({"ref":reference(package),"dependsOn":dependencies}));
        components.push(item);
    }
    let lock_hash = hash(input.cargo_lock);
    let bom = json!({"bomFormat":"CycloneDX","specVersion":"1.6","version":1,"metadata":{"component":{"type":"application","bom-ref":"mailwoman","name":"Mailwoman","version":input.version},"properties":[{"name":"mailwoman:coverage","value":"Cargo workspace lock inventory; includes development and platform-specific dependencies. Native system libraries, containers, plugins and frontend are separate."},{"name":"mailwoman:cargo-lock-sha256","value":lock_hash}]},"components":components,"dependencies":edges});
    let sbom = serde_json::to_vec(&bom).map_err(|error| error.to_string())?;
    let metadata = json!({"name":"Mailwoman","version":input.version,"revision":input.revision,"releaseTag":input.release_tag,"target":input.target,"sourceDateEpoch":input.source_date_epoch,"supportUntil":if support.is_empty(){None}else{Some(support)},"supportPolicy":"Latest tagged rolling release; older releases are unsupported. A fixed support-end date has not been inferred.","vulnerabilityReportingUrl":"https://github.com/supermarsx/mailwoman/security/advisories/new","advisoriesUrl":"https://github.com/supermarsx/mailwoman/security/advisories","releasesUrl":"https://github.com/supermarsx/mailwoman/releases","securityPolicySha256":hash(input.security_policy),"cargoLockSha256":lock_hash,"pnpmLockSha256":input.pnpm_lock.map(hash),"componentCount":packages.len(),"sbomFormat":"CycloneDX 1.6","sbomSha256":hash(&sbom),"inventoryScope":"workspace-lock","conformityAssessed":false});
    Ok(Artifacts {
        metadata: serde_json::to_vec(&metadata).map_err(|error| error.to_string())?,
        sbom,
        security_policy: input.security_policy.to_vec(),
    })
}

#[cfg(test)]
mod tests;
