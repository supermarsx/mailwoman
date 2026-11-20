//! Unit-fixture compatibility only. Complete current source inventory is produced
//! at TEST EXECUTION, without a revision-bearing production dependency edge.
use super::*;
use std::{fs, path::Path, process::Command, sync::OnceLock};

fn git(root: &Path, args: &[&str]) -> String {
    Command::new("git")
        .args(args)
        .current_dir(root)
        .output()
        .ok()
        .filter(|output| output.status.success())
        .map(|output| String::from_utf8_lossy(&output.stdout).trim().into())
        .unwrap_or_default()
}

fn target_matches_executable(target: &str) -> bool {
    let arch = std::env::consts::ARCH;
    let target_arch = target.split('-').next().unwrap_or_default();
    let architecture = match arch {
        "x86" => matches!(target_arch, "i386" | "i486" | "i586" | "i686"),
        "arm" => target_arch.starts_with("arm") || target_arch.starts_with("thumb"),
        "riscv32" => target_arch.starts_with("riscv32"),
        "riscv64" => target_arch.starts_with("riscv64"),
        other => target_arch == other,
    };
    let os = match std::env::consts::OS {
        "macos" => "darwin",
        other => other,
    };
    architecture
        && target.split('-').any(|part| part == os)
        && (!cfg!(target_env = "msvc") || target.ends_with("msvc"))
        && (!cfg!(target_env = "musl") || target.ends_with("musl"))
        && (!cfg!(target_env = "gnu") || target.ends_with("gnu") || target.ends_with("gnueabihf"))
}

fn actual_test_target() -> anyhow::Result<String> {
    let target = match std::env::var("MW_TEST_BUILD_TARGET") {
        Ok(target) => target,
        Err(_) => {
            let output = Command::new(std::env::var_os("RUSTC").unwrap_or_else(|| "rustc".into()))
                .arg("-vV")
                .output()?;
            anyhow::ensure!(
                output.status.success(),
                "Cannot determine unit fixture compiler target"
            );
            String::from_utf8(output.stdout)?
                .lines()
                .find_map(|line| line.strip_prefix("host: "))
                .ok_or_else(|| anyhow::anyhow!("Compiler host target missing"))?
                .to_owned()
        }
    };
    anyhow::ensure!(
        target_matches_executable(&target),
        "Unit fixture target differs from executable; supply exact MW_TEST_BUILD_TARGET for a cross-target runner"
    );
    Ok(target)
}

fn produce() -> anyhow::Result<ProductBuildInfo> {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let explicit = std::env::var("MW_BUILD_REVISION").ok();
    let legacy = std::env::var("MW_BUILD_SHA").ok();
    let revision = mw_build_metadata::resolve_revision(
        explicit.as_deref(),
        legacy.as_deref(),
        &git(&root, &["rev-parse", "HEAD"]),
    )
    .map_err(anyhow::Error::msg)?;
    let git_tag = git(&root, &["describe", "--tags", "--exact-match"]);
    let tag = std::env::var("MW_BUILD_RELEASE_TAG")
        .ok()
        .filter(|tag| !tag.is_empty())
        .unwrap_or_else(|| git_tag.clone());
    anyhow::ensure!(
        git_tag.is_empty() || tag == git_tag,
        "Unit fixture release tag differs from source"
    );
    let lock = fs::read(root.join("Cargo.lock"))?;
    let pnpm = fs::read(root.join("apps/web/pnpm-lock.yaml")).ok();
    let policy = fs::read(root.join("SECURITY.md"))?;
    let target = actual_test_target()?;
    let epoch = std::env::var("SOURCE_DATE_EPOCH").ok();
    let support = std::env::var("MW_PRODUCT_SUPPORT_UNTIL").ok();
    let artifacts = mw_build_metadata::generate(mw_build_metadata::Inputs {
        version: env!("CARGO_PKG_VERSION"),
        target: &target,
        revision: &revision,
        release_tag: &tag,
        source_date_epoch: epoch.as_deref(),
        support_until: support.as_deref(),
        cargo_lock: &lock,
        pnpm_lock: pnpm.as_deref(),
        security_policy: &policy,
    })
    .map_err(anyhow::Error::msg)?;
    ProductBuildInfo::new(
        artifacts.metadata.into(),
        artifacts.sbom.into(),
        artifacts.security_policy.into(),
    )
}

pub(crate) fn info() -> anyhow::Result<ProductBuildInfo> {
    static INFO: OnceLock<Result<ProductBuildInfo, String>> = OnceLock::new();
    INFO.get_or_init(|| produce().map_err(|error| error.to_string()))
        .as_ref()
        .cloned()
        .map_err(|error| anyhow::anyhow!("Cannot produce complete core unit inventory: {error}"))
}

pub async fn build_app(config: AppConfig) -> anyhow::Result<Router> {
    super::build_app_with_info(config, info()?).await
}
pub async fn build_app_with_push(config: AppConfig) -> anyhow::Result<(Router, PushHandle)> {
    super::build_app_with_push_and_info(config, info()?).await
}
pub async fn build_app_full(
    config: AppConfig,
    v6config: V6Config,
) -> anyhow::Result<(Router, PushHandle)> {
    super::build_app_full_with_info(config, v6config, info()?).await
}
#[cfg(feature = "load-test-fixtures")]
pub async fn build_app_with_fixture_engine(
    config: AppConfig,
    v6config: V6Config,
    factory: impl FnOnce(&Store) -> Arc<mw_engine::Engine> + Send + 'static,
) -> anyhow::Result<(Router, PushHandle)> {
    super::build_app_with_fixture_engine_and_info(config, v6config, factory, info()?).await
}
pub async fn build_app_with_sso_mock(
    config: AppConfig,
    v6config: V6Config,
    providers: Vec<(String, sso::SsoEntry)>,
) -> anyhow::Result<(Router, PushHandle)> {
    super::build_app_with_sso_mock_and_info(config, v6config, providers, info()?).await
}
