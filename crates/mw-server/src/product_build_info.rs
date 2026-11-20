//! Immutable, self-consistent public build facts supplied by the thin host.
//! No runtime environment, Git, source-tree or deployment-configuration reads.
//! The ordinary package version remains a compiled compatibility contract.
use bytes::Bytes;
use serde_json::Value;
use sha2::{Digest, Sha256};
use std::{collections::HashSet, sync::Arc};

#[derive(Clone)]
pub struct ProductBuildInfo(Arc<Artifacts>);

struct Artifacts {
    metadata: Bytes,
    sbom: Bytes,
    policy: Bytes,
    revision: String,
}

impl std::fmt::Debug for ProductBuildInfo {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("ProductBuildInfo")
            .field("metadata_bytes", &self.0.metadata.len())
            .field("sbom_bytes", &self.0.sbom.len())
            .field("policy_bytes", &self.0.policy.len())
            .finish()
    }
}

fn hash(bytes: &[u8]) -> String {
    Sha256::digest(bytes)
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect()
}

impl ProductBuildInfo {
    /// Validate a complete immutable build set once, before application startup.
    /// Static executable data may use `Bytes::from_static` without copying.
    pub fn new(metadata: Bytes, sbom: Bytes, policy: Bytes) -> anyhow::Result<Self> {
        let facts: Value = serde_json::from_slice(&metadata)?;
        let inventory: Value = serde_json::from_slice(&sbom)?;
        std::str::from_utf8(&policy)?;
        anyhow::ensure!(!policy.is_empty(), "Product security policy missing");
        anyhow::ensure!(facts["name"] == "Mailwoman", "Unexpected product metadata");
        anyhow::ensure!(
            facts["version"] == env!("CARGO_PKG_VERSION"),
            "Product version does not match server"
        );
        let revision = facts["revision"].as_str().unwrap_or_default();
        anyhow::ensure!(
            revision.len() == 40 && revision.bytes().all(|byte| byte.is_ascii_hexdigit()),
            "Immutable build revision missing"
        );
        anyhow::ensure!(
            facts["target"]
                .as_str()
                .is_some_and(|target| !target.is_empty()),
            "Product target missing"
        );
        anyhow::ensure!(
            facts["sbomSha256"] == hash(&sbom),
            "Product inventory digest mismatch"
        );
        anyhow::ensure!(
            facts["securityPolicySha256"] == hash(&policy),
            "Product security policy digest mismatch"
        );
        anyhow::ensure!(
            facts["inventoryScope"] == "workspace-lock" && facts["conformityAssessed"] == false,
            "Unexpected product inventory claims"
        );
        anyhow::ensure!(
            inventory["bomFormat"] == "CycloneDX" && inventory["specVersion"] == "1.6",
            "Unexpected inventory format"
        );
        anyhow::ensure!(
            inventory["metadata"]["component"]["version"] == facts["version"],
            "Inventory product version mismatch"
        );
        let lock_hash = facts["cargoLockSha256"].as_str().unwrap_or_default();
        anyhow::ensure!(
            lock_hash.len() == 64 && lock_hash.bytes().all(|byte| byte.is_ascii_hexdigit()),
            "Inventory source digest missing"
        );
        let source_hash = inventory["metadata"]["properties"]
            .as_array()
            .and_then(|properties| {
                properties
                    .iter()
                    .find(|property| property["name"] == "mailwoman:cargo-lock-sha256")
            })
            .and_then(|property| property["value"].as_str());
        anyhow::ensure!(
            source_hash == Some(lock_hash),
            "Inventory source digest mismatch"
        );
        let components = inventory["components"]
            .as_array()
            .ok_or_else(|| anyhow::anyhow!("Inventory components missing"))?;
        anyhow::ensure!(!components.is_empty(), "Product inventory is empty");
        let refs: HashSet<&str> = components
            .iter()
            .filter_map(|component| component["bom-ref"].as_str())
            .collect();
        anyhow::ensure!(
            refs.len() == components.len()
                && facts["componentCount"].as_u64() == Some(refs.len() as u64),
            "Inventory component references mismatch"
        );
        let dependencies = inventory["dependencies"]
            .as_array()
            .ok_or_else(|| anyhow::anyhow!("Inventory dependency graph missing"))?;
        for dependency in dependencies {
            anyhow::ensure!(
                dependency["ref"]
                    .as_str()
                    .is_some_and(|reference| refs.contains(reference)),
                "Unresolved inventory component"
            );
            let targets = dependency["dependsOn"]
                .as_array()
                .ok_or_else(|| anyhow::anyhow!("Inventory dependency edges missing"))?;
            anyhow::ensure!(
                targets.iter().all(|target| target
                    .as_str()
                    .is_some_and(|reference| refs.contains(reference))),
                "Unresolved inventory dependency"
            );
        }
        let revision = revision.to_owned();
        Ok(Self(Arc::new(Artifacts {
            metadata,
            sbom,
            policy,
            revision,
        })))
    }

    pub fn revision(&self) -> &str {
        &self.0.revision
    }
    pub(crate) fn metadata(&self) -> Bytes {
        self.0.metadata.clone()
    }
    pub(crate) fn sbom(&self) -> Bytes {
        self.0.sbom.clone()
    }
    pub(crate) fn policy(&self) -> Bytes {
        self.0.policy.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn complete_source_fixture_preserves_shared_immutable_payloads_and_legacy_log_prefix() {
        let info = crate::core_test_build_info::info().unwrap();
        let clone = info.clone();
        assert_eq!(info.metadata().as_ptr(), clone.metadata().as_ptr());
        assert_eq!(info.sbom().as_ptr(), clone.sbom().as_ptr());
        let facts: Value = serde_json::from_slice(&info.metadata()).unwrap();
        assert!(facts["componentCount"].as_u64().unwrap() > 100);
        assert_eq!(facts["revision"], info.revision());
        assert_eq!(module_path!().split("::").next(), Some("mw_server"));
        assert!(!format!("{info:?}").contains("CycloneDX"));
    }

    #[test]
    fn mismatched_policy_or_inventory_bytes_cannot_be_installed() {
        let info = crate::core_test_build_info::info().unwrap();
        assert!(
            ProductBuildInfo::new(
                info.metadata(),
                info.sbom(),
                Bytes::from_static(b"wrong policy")
            )
            .is_err()
        );
        assert!(
            ProductBuildInfo::new(info.metadata(), Bytes::from_static(b"{}"), info.policy())
                .is_err()
        );
    }

    #[test]
    fn hash_consistent_inventory_still_rejects_unknown_dependency_references() {
        let info = crate::core_test_build_info::info().unwrap();
        let mut facts: Value = serde_json::from_slice(&info.metadata()).unwrap();
        let mut bom: Value = serde_json::from_slice(&info.sbom()).unwrap();
        bom["dependencies"][0]["dependsOn"]
            .as_array_mut()
            .unwrap()
            .push(Value::String("unresolved-component".into()));
        let sbom = serde_json::to_vec(&bom).unwrap();
        facts["sbomSha256"] = Value::String(hash(&sbom));
        assert!(
            ProductBuildInfo::new(
                serde_json::to_vec(&facts).unwrap().into(),
                sbom.into(),
                info.policy()
            )
            .is_err()
        );
    }
}
