//! Compile embedded asset tables separately from HTTP handlers and policy code.
//!
//! The embedding inputs and rust-embed debug/release behavior are unchanged.
//! This crate does not read repository revisions or generate product metadata,
//! decide which route may serve a file, or implement runtime disk overrides.

#![forbid(unsafe_code)]

use std::borrow::Cow;

#[derive(rust_embed::RustEmbed)]
#[folder = "../../apps/web/dist"]
struct EmbeddedWebAssets;

#[derive(rust_embed::RustEmbed)]
#[folder = "../../apps/web/public"]
#[include = "pdfjs/**"]
#[include = "pdf.worker.mjs"]
struct EmbeddedPdfAssets;

/// Bytes of a web build file, if present in the build's asset set.
///
/// A checkout or Docker build may intentionally have only `dist/.gitkeep`.
/// Runtime `MW_WEB_DIR` selection and path validation belong to the server.
pub fn web(path: &str) -> Option<Cow<'static, [u8]>> {
    EmbeddedWebAssets::get(path).map(|asset| asset.data)
}

/// Filenames of the same web asset set, with rust-embed's existing ordering.
pub fn web_paths() -> impl Iterator<Item = Cow<'static, str>> {
    EmbeddedWebAssets::iter()
}

/// Bytes of the pinned PDF viewer library, worker, or coupled viewer document.
/// The server retains manifest validation and all deployed-file override rules.
pub fn pdf(path: &str) -> Option<Cow<'static, [u8]>> {
    EmbeddedPdfAssets::get(path).map(|asset| asset.data)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{fs, path::Path};

    fn public() -> std::path::PathBuf {
        Path::new(env!("CARGO_MANIFEST_DIR")).join("../../apps/web/public")
    }

    #[test]
    fn pinned_pdf_inventory_and_coupled_files_are_verbatim() {
        let root = public();
        let manifest = fs::read(root.join("pdfjs/manifest.json")).unwrap();
        assert_eq!(pdf("pdfjs/manifest.json").unwrap().as_ref(), manifest);
        let manifest: serde_json::Value = serde_json::from_slice(&manifest).unwrap();
        let files = manifest["files"].as_object().unwrap();
        assert!(
            files.len() > 300,
            "expected the complete pinned PDF inventory"
        );
        for name in files.keys() {
            let path = format!("pdfjs/{name}");
            assert_eq!(
                pdf(&path).unwrap().as_ref(),
                fs::read(root.join(&path)).unwrap()
            );
        }
        for path in [
            "pdf.worker.mjs",
            "pdfjs/bootstrap.mjs",
            "pdfjs/polyfills.mjs",
            "pdfjs/mailwoman.css",
            "pdfjs/web/mailwoman.html",
        ] {
            assert_eq!(
                pdf(path).unwrap().as_ref(),
                fs::read(root.join(path)).unwrap()
            );
        }
    }

    #[test]
    fn pdf_filter_excludes_unrelated_public_and_missing_files() {
        assert!(public().join("workspace-popup.html").is_file());
        assert!(pdf("workspace-popup.html").is_none());
        assert!(pdf("pdfjs/not-a-vendored-file.mjs").is_none());
    }

    #[test]
    fn web_lookup_keeps_the_optional_dist_contract() {
        let dist = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../apps/web/dist");
        match fs::read(dist.join("index.html")) {
            Ok(expected) => assert_eq!(web("index.html").unwrap().as_ref(), expected),
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
                assert!(web("index.html").is_none());
            }
            Err(error) => panic!("cannot inspect web fixture: {error}"),
        }
        for path in web_paths() {
            assert_eq!(
                web(&path).unwrap().as_ref(),
                fs::read(dist.join(path.as_ref())).unwrap()
            );
        }
        assert!(web("not-a-built-asset-7c1d9e.js").is_none());
    }
}
