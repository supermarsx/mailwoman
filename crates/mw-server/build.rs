//! Build-local product inventory: no network access or credentials.
use serde_json::json;
use sha2::{Digest, Sha256};
use std::{env, fs, path::Path, process::Command};
fn hash(bytes: &[u8]) -> String {
    Sha256::digest(bytes)
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect()
}
fn git(root: &Path, args: &[&str]) -> String {
    Command::new("git")
        .args(args)
        .current_dir(root)
        .output()
        .ok()
        .filter(|o| o.status.success())
        .map(|o| String::from_utf8_lossy(&o.stdout).trim().into())
        .unwrap_or_default()
}
fn encoded(text: &str) -> String {
    text.bytes()
        .map(|b| {
            if b.is_ascii_alphanumeric() || b".-_~".contains(&b) {
                (b as char).to_string()
            } else {
                format!("%{b:02X}")
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
fn main() {
    let manifest = env::var("CARGO_MANIFEST_DIR").unwrap();
    let root = Path::new(&manifest).parent().unwrap().parent().unwrap();
    let out = env::var("OUT_DIR").unwrap();
    for file in [
        "Cargo.lock",
        "apps/web/pnpm-lock.yaml",
        "SECURITY.md",
    ] {
        println!("cargo:rerun-if-changed={}", root.join(file).display());
    }
    for key in ["SOURCE_DATE_EPOCH", "MW_PRODUCT_SUPPORT_UNTIL"] {
        println!("cargo:rerun-if-env-changed={key}");
    }
    let git_head = git(root, &["rev-parse", "--git-path", "HEAD"]);
    if !git_head.is_empty() {
        println!("cargo:rerun-if-changed={}", root.join(git_head).display());
    }
    let branch = git(root, &["symbolic-ref", "-q", "HEAD"]);
    if !branch.is_empty() {
        let p = git(root, &["rev-parse", "--git-path", &branch]);
        println!("cargo:rerun-if-changed={}", root.join(p).display());
    }
    let lock =
        fs::read(root.join("Cargo.lock")).expect("Cargo.lock required for product inventory");
    let parsed: toml::Value = toml::from_str(std::str::from_utf8(&lock).unwrap()).unwrap();
    let packages = parsed["package"].as_array().unwrap();
    let mut components = Vec::new();
    let mut edges = Vec::new();
    // This is intentionally the workspace lock inventory, including development and
    // target-specific packages. Exact shipped membership is not inferred from a lock.
    let reference = |p: &toml::Value| {
        format!(
            "cargo:{}@{}:{}",
            p["name"].as_str().unwrap(),
            p["version"].as_str().unwrap(),
            hash(
                p.get("source")
                    .and_then(|v| v.as_str())
                    .unwrap_or("workspace")
                    .as_bytes()
            )[..12]
                .to_owned()
        )
    };
    for package in packages {
        let name = package["name"].as_str().unwrap();
        let version = package["version"].as_str().unwrap();
        let source = package
            .get("source")
            .and_then(|v| v.as_str())
            .unwrap_or("workspace");
        let mut item = json!({"type":"library","bom-ref":reference(package),"name":name,"version":version,"purl":format!("pkg:cargo/{}@{}",encoded(name),encoded(version)),"scope":"optional","properties":[{"name":"mailwoman:inventory-source","value":source},{"name":"mailwoman:artifact-membership","value":"unverified-workspace-lock"}]});
        if let Some(checksum) = package.get("checksum").and_then(|v| v.as_str()) {
            item["hashes"] = json!([{"alg":"SHA-256","content":checksum}]);
            item["properties"].as_array_mut().unwrap().push(json!({"name":"mailwoman:checksum-kind","value":"Cargo registry archive checksum; compiled or vendored contents are not verified"}));
        }
        let dependencies: Vec<String> = package
            .get("dependencies")
            .and_then(|d| d.as_array())
            .into_iter()
            .flatten()
            .filter_map(|d| d.as_str())
            .flat_map(|dep| {
                let parts: Vec<_> = dep.split_whitespace().collect();
                packages
                    .iter()
                    .filter(move |p| {
                        p["name"].as_str() == parts.first().copied()
                            && (parts.get(1).is_none()
                                || p["version"].as_str() == parts.get(1).copied())
                    })
                    .map(&reference)
            })
            .collect();
        edges.push(json!({"ref":reference(package),"dependsOn":dependencies}));
        components.push(item);
    }
    let version = env::var("CARGO_PKG_VERSION").unwrap();
    let lock_hash = hash(&lock);
    let bom = json!({"bomFormat":"CycloneDX","specVersion":"1.6","version":1,"metadata":{"component":{"type":"application","bom-ref":"mailwoman","name":"Mailwoman","version":version},"properties":[{"name":"mailwoman:coverage","value":"Cargo workspace lock inventory; includes development and platform-specific dependencies. Native system libraries, containers, plugins and frontend are separate."},{"name":"mailwoman:cargo-lock-sha256","value":lock_hash}]},"components":components,"dependencies":edges});
    let pnpm = fs::read(root.join("apps/web/pnpm-lock.yaml")).ok();
    let security = fs::read(root.join("SECURITY.md")).unwrap_or_default();
    let support = env::var("MW_PRODUCT_SUPPORT_UNTIL").unwrap_or_default();
    if !support.is_empty() {
        assert!(
            valid_date(&support),
            "MW_PRODUCT_SUPPORT_UNTIL must be YYYY-MM-DD"
        );
    }
    let bom_bytes = serde_json::to_vec(&bom).unwrap();
    let metadata = json!({"name":"Mailwoman","version":version,"revision":git(root,&["rev-parse","HEAD"]),"releaseTag":git(root,&["describe","--tags","--exact-match"]),"target":env::var("TARGET").unwrap_or_default(),"sourceDateEpoch":env::var("SOURCE_DATE_EPOCH").ok(),"supportUntil":if support.is_empty(){None}else{Some(support)},"supportPolicy":"Latest tagged rolling release; older releases are unsupported. A fixed support-end date has not been inferred.","vulnerabilityReportingUrl":"https://github.com/supermarsx/mailwoman/security/advisories/new","advisoriesUrl":"https://github.com/supermarsx/mailwoman/security/advisories","releasesUrl":"https://github.com/supermarsx/mailwoman/releases","securityPolicySha256":hash(&security),"cargoLockSha256":lock_hash,"pnpmLockSha256":pnpm.map(|b|hash(&b)),"componentCount":components.len(),"sbomFormat":"CycloneDX 1.6","sbomSha256":hash(&bom_bytes),"inventoryScope":"workspace-lock","conformityAssessed":false});
    fs::write(
        Path::new(&out).join("product-security.json"),
        serde_json::to_vec(&metadata).unwrap(),
    )
    .unwrap();
    fs::write(Path::new(&out).join("software-bom.cdx.json"), bom_bytes).unwrap();
}
