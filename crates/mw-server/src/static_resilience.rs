//! Static answers that keep clients of an older build working (t102).
//!
//! Two kinds of request reach the server from a client that is running an
//! older build, and both used to end in a 404:
//!
//! * **A lazily-loaded chunk of the previous build.** A tab opened before a
//!   deploy still runs the old entry script, and it imports chunks by the
//!   content-hashed names that build emitted (`Dashboard-CdQvJhSt.js`). The new
//!   build has other names, so the old tab's next lazy route failed. [`Retained`]
//!   keeps the hashed files of the last few builds in a directory outside the
//!   web build, beside the database (`/data/retained-assets` in the Docker
//!   image), so they outlive the image that shipped them. Each build's files are
//!   kept while it is one of the [`MAX_BUILDS`] newest and for [`GRACE_SECS`]
//!   after a newer build replaced it. Only content-hashed names under `assets/`
//!   are kept or served; they are immutable by construction, so a retained file
//!   is the exact file its build served.
//! * **A service-worker script the server does not have.** The browser re-fetches
//!   a registered worker's script URL to check for an update: on navigation, on
//!   `register()`, and about once a day. If the URL answers 404 the registration
//!   stays, the check fails, and it is repeated on the next navigation, so one
//!   stale registration produces a steady stream of 404s. [`worker_fallback`]
//!   answers such a fetch (only a fetch the browser marks as a worker-script
//!   fetch) with [`CLEANUP_WORKER`]: a worker with no fetch handler that
//!   unregisters itself on activation. The registration then removes itself, and
//!   its update checks stop. The app registers its real worker again
//!   (`apps/web/src/sw/register.ts`) at the URL the server does serve.
//!
//! Neither changes what the current build serves: both run only after the web
//! build had no file at the requested path.
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

use axum::http::{HeaderMap, HeaderValue, header};
use axum::response::Response;
use serde::{Deserialize, Serialize};

/// The previous builds whose hashed files are kept, the current one included.
pub(crate) const MAX_BUILDS: usize = 5;
/// How long a build's files are kept after a newer build replaced it.
pub(crate) const GRACE_SECS: u64 = 14 * 24 * 60 * 60;
/// Upper bound on what the retained directory may hold. The oldest replaced
/// builds are dropped first once it is passed.
pub(crate) const MAX_BYTES: u64 = 512 * 1024 * 1024;
const MANIFEST: &str = "builds.json";

/// Served in place of a service-worker script the server does not have.
///
/// No `fetch` listener, so every request of a page it still controls goes to the
/// network as if there were no worker (the admin console included). On
/// activation it unregisters its own registration and deletes the caches the
/// app's worker creates (`mw-` names, see `apps/web/src/sw/purge.ts`); pages it
/// controls keep running and are not reloaded, so nothing unsaved is lost.
pub(crate) const CLEANUP_WORKER: &str = "\
// Mailwoman: this service-worker address is no longer served. This worker\n\
// removes its own registration and the caches the old worker stored.\n\
self.addEventListener('install', () => { self.skipWaiting(); });\n\
self.addEventListener('activate', (event) => {\n\
  event.waitUntil((async () => {\n\
    try {\n\
      const names = await caches.keys();\n\
      await Promise.all(names.filter((name) => name.startsWith('mw-')).map((name) => caches.delete(name)));\n\
    } catch (_) { /* Storage is optional. */ }\n\
    await self.registration.unregister();\n\
  })());\n\
});\n";

/// Marks the cleanup worker's response, for operators and tests.
pub(crate) const CLEANUP_HEADER: &str = "x-mailwoman-worker";

/// Is this request the browser fetching a service-worker script? Browsers send
/// `Service-Worker: script` on every such fetch (registration and update
/// checks) and `Sec-Fetch-Dest: serviceworker`; nothing else sends either.
fn is_worker_script_fetch(headers: &HeaderMap) -> bool {
    let header = |name: &str| headers.get(name).and_then(|v| v.to_str().ok());
    header("service-worker").is_some_and(|v| v.eq_ignore_ascii_case("script"))
        || header("sec-fetch-dest").is_some_and(|v| v.eq_ignore_ascii_case("serviceworker"))
}

/// The cleanup worker for a worker-script fetch of a `sw.js` the web build does
/// not have (`path` is relative to the deploy prefix, without its leading `/`):
/// a registration made under an earlier deploy prefix (`mail/sw.js` after the
/// prefix was removed), or `sw.js` itself when the web directory lacks it. Any
/// other request, and any other file name, is left to the 404.
pub(crate) fn worker_fallback(path: &str, headers: &HeaderMap) -> Option<Response> {
    let segments: Vec<&str> = path.split('/').collect();
    if segments.last() != Some(&"sw.js") || segments.len() > 5 || !is_worker_script_fetch(headers) {
        return None;
    }
    let plain = |s: &&str| {
        !s.is_empty()
            && *s != "."
            && *s != ".."
            && s.bytes()
                .all(|b| b.is_ascii_alphanumeric() || matches!(b, b'-' | b'_' | b'.' | b'~'))
    };
    if !segments.iter().all(plain) {
        return None;
    }
    if path == "sw.js" {
        warn_missing_worker();
    }
    let mut resp = Response::new(axum::body::Body::from(CLEANUP_WORKER));
    let h = resp.headers_mut();
    h.insert(
        header::CONTENT_TYPE,
        HeaderValue::from_static("text/javascript; charset=utf-8"),
    );
    // A worker script must be re-checked on every update check, never kept.
    h.insert(header::CACHE_CONTROL, HeaderValue::from_static("no-cache"));
    h.insert(CLEANUP_HEADER, HeaderValue::from_static("cleanup"));
    Some(resp)
}

fn warn_missing_worker() {
    static ONCE: std::sync::Once = std::sync::Once::new();
    ONCE.call_once(|| {
        tracing::warn!(
            "the web build has no sw.js: browsers that registered the offline worker are sent a \
             worker that removes itself; rebuild the web client (pnpm build) to restore offline use"
        );
    });
}

#[derive(Debug, Default, Serialize, Deserialize)]
struct Manifest {
    builds: Vec<Build>,
}

#[derive(Debug, Serialize, Deserialize)]
struct Build {
    /// SHA-256 over the build's sorted hashed file names.
    id: String,
    /// When this server first served the build (Unix seconds).
    first_seen: u64,
    /// When a newer build replaced it; `None` for the build being served.
    replaced: Option<u64>,
    files: Vec<String>,
}

/// The retained hashed files of earlier builds.
#[derive(Debug)]
pub(crate) struct Retained {
    dir: PathBuf,
}

/// Where retained files are kept: `MW_RETAINED_ASSETS_DIR` (`off` disables), or
/// `retained-assets` beside a SQLite database file. `None` for an in-memory or
/// PostgreSQL database without the variable.
pub(crate) fn location(db_path: &str) -> Option<PathBuf> {
    match std::env::var("MW_RETAINED_ASSETS_DIR") {
        Ok(v) if v.trim().eq_ignore_ascii_case("off") => return None,
        Ok(v) if !v.trim().is_empty() => return Some(PathBuf::from(v.trim())),
        _ => {}
    }
    let lower = db_path.to_ascii_lowercase();
    if db_path.is_empty()
        || lower.contains(":memory:")
        || lower.contains("://")
        || lower.starts_with("sqlite:")
    {
        return None;
    }
    let parent = Path::new(db_path).parent()?;
    let parent = if parent.as_os_str().is_empty() {
        Path::new(".")
    } else {
        parent
    };
    Some(parent.join("retained-assets"))
}

impl Retained {
    /// Record the build being served and copy its hashed files into `dir`, then
    /// drop the files of builds outside the retention policy. `current` lists
    /// the build's files as `(name under assets/, bytes)`. Errors are logged and
    /// leave retention off; serving the current build never depends on it.
    pub(crate) fn snapshot(dir: PathBuf, current: Vec<(String, Vec<u8>)>) -> Option<Arc<Self>> {
        match Self::snapshot_at(&dir, current, now()) {
            Ok(()) => Some(Arc::new(Self { dir })),
            Err(error) => {
                tracing::warn!(dir = %dir.display(), %error, "retained web assets are off");
                None
            }
        }
    }

    fn snapshot_at(dir: &Path, current: Vec<(String, Vec<u8>)>, at: u64) -> std::io::Result<()> {
        let files_dir = dir.join("assets");
        std::fs::create_dir_all(&files_dir)?;
        let mut names: Vec<String> = current
            .iter()
            .map(|(name, _)| name.clone())
            .filter(|name| retainable(name))
            .collect();
        names.sort();
        names.dedup();
        if names.is_empty() {
            return Ok(());
        }
        let id = {
            use sha2::{Digest, Sha256};
            let mut h = Sha256::new();
            for name in &names {
                h.update(name.as_bytes());
                h.update([0]);
            }
            h.finalize()
                .iter()
                .map(|b| format!("{b:02x}"))
                .collect::<String>()
        };
        let mut manifest: Manifest = std::fs::read(dir.join(MANIFEST))
            .ok()
            .and_then(|bytes| serde_json::from_slice(&bytes).ok())
            .unwrap_or_default();
        for (name, bytes) in &current {
            if !retainable(name) {
                continue;
            }
            let target = files_dir.join(name);
            if !target.exists() {
                // Write then rename, so a reader never sees a partial file.
                let partial = files_dir.join(format!(".{name}.partial"));
                std::fs::write(&partial, bytes)?;
                std::fs::rename(&partial, &target)?;
            }
        }
        if manifest.builds.last().map(|b| b.id.as_str()) != Some(id.as_str()) {
            // A build served again after another (a rollback) moves to the end.
            manifest.builds.retain(|b| b.id != id);
            for build in &mut manifest.builds {
                build.replaced.get_or_insert(at);
            }
            manifest.builds.push(Build {
                id,
                first_seen: at,
                replaced: None,
                files: names,
            });
        }
        // Policy: the newest MAX_BUILDS, each replaced build for GRACE_SECS.
        let keep_from = manifest.builds.len().saturating_sub(MAX_BUILDS);
        let mut kept: Vec<Build> = manifest
            .builds
            .drain(..)
            .enumerate()
            .filter(|(i, b)| {
                *i >= keep_from && b.replaced.is_none_or(|r| at.saturating_sub(r) < GRACE_SECS)
            })
            .map(|(_, b)| b)
            .collect();
        // Size bound: drop the oldest replaced builds while the set is too large.
        let size = |builds: &[Build]| -> u64 {
            let mut names: Vec<&String> = builds.iter().flat_map(|b| &b.files).collect();
            names.sort();
            names.dedup();
            names
                .iter()
                .filter_map(|n| std::fs::metadata(files_dir.join(n)).ok())
                .map(|m| m.len())
                .sum()
        };
        while kept.len() > 1 && size(&kept) > MAX_BYTES {
            kept.remove(0);
        }
        let live: std::collections::HashSet<&String> = kept.iter().flat_map(|b| &b.files).collect();
        for entry in std::fs::read_dir(&files_dir)?.flatten() {
            let name = entry.file_name().to_string_lossy().into_owned();
            if !live.contains(&name) {
                let _ = std::fs::remove_file(entry.path());
            }
        }
        manifest.builds = kept;
        let partial = dir.join(format!(".{MANIFEST}.partial"));
        std::fs::write(&partial, serde_json::to_vec_pretty(&manifest)?)?;
        std::fs::rename(&partial, dir.join(MANIFEST))?;
        Ok(())
    }

    /// The bytes of a retained file, for a `path` (relative to the deploy
    /// prefix) of the form `assets/<content-hashed name>`.
    pub(crate) fn read(&self, path: &str) -> Option<Vec<u8>> {
        let name = path.strip_prefix("assets/")?;
        if !retainable(name) {
            return None;
        }
        std::fs::read(self.dir.join("assets").join(name)).ok()
    }
}

/// A single plain file name carrying a bundler content hash. Only these are
/// kept: anything else could change between builds under the same name.
fn retainable(name: &str) -> bool {
    !name.is_empty()
        && !name.starts_with('.')
        && name
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || matches!(b, b'-' | b'_' | b'.'))
        && crate::is_content_hashed(name)
}

/// The hashed files of the build being served: `assets/` of `web_dir`, or of the
/// embedded bundle. The analytics loader directory is generated, never kept.
pub(crate) fn current_build(web_dir: Option<&Path>) -> Vec<(String, Vec<u8>)> {
    match web_dir {
        Some(dir) => std::fs::read_dir(dir.join("assets"))
            .map(|entries| {
                entries
                    .flatten()
                    .filter(|e| e.file_type().is_ok_and(|t| t.is_file()))
                    .filter_map(|e| {
                        let name = e.file_name().to_str()?.to_string();
                        retainable(&name)
                            .then(|| std::fs::read(e.path()).ok().map(|bytes| (name, bytes)))
                            .flatten()
                    })
                    .collect()
            })
            .unwrap_or_default(),
        None => crate::WebAssets::iter()
            .filter_map(|path| {
                let name = path.strip_prefix("assets/")?;
                if name.contains('/') || !retainable(name) {
                    return None;
                }
                Some((
                    name.to_string(),
                    crate::WebAssets::get(&path)?.data.into_owned(),
                ))
            })
            .collect(),
    }
}

fn now() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::*;

    struct Scratch(PathBuf);
    impl Scratch {
        fn new() -> Self {
            static N: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);
            let n = N.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
            let dir =
                std::env::temp_dir().join(format!("mw-t102-retained-{}-{n}", std::process::id()));
            let _ = std::fs::remove_dir_all(&dir);
            Self(dir)
        }
        fn path(&self) -> &Path {
            &self.0
        }
    }
    impl Drop for Scratch {
        fn drop(&mut self) {
            let _ = std::fs::remove_dir_all(&self.0);
        }
    }

    fn build(names: &[&str]) -> Vec<(String, Vec<u8>)> {
        names
            .iter()
            .map(|n| (n.to_string(), n.as_bytes().to_vec()))
            .collect()
    }

    #[test]
    fn replaced_builds_are_kept_for_the_grace_period_and_the_newest_count() {
        let dir = Scratch::new();
        let root = dir.path().to_path_buf();
        let day = 24 * 60 * 60;
        Retained::snapshot_at(&root, build(&["index-AAAA1111.js", "Dash-BBBB2222.js"]), 0).unwrap();
        Retained::snapshot_at(
            &root,
            build(&["index-CCCC3333.js", "Dash-BBBB2222.js"]),
            day,
        )
        .unwrap();
        let r = Retained { dir: root.clone() };
        assert!(
            r.read("assets/index-AAAA1111.js").is_some(),
            "previous build kept"
        );
        assert!(r.read("assets/index-CCCC3333.js").is_some());
        // Same build again: nothing changes.
        Retained::snapshot_at(
            &root,
            build(&["index-CCCC3333.js", "Dash-BBBB2222.js"]),
            2 * day,
        )
        .unwrap();
        assert!(r.read("assets/index-AAAA1111.js").is_some());
        // Past the grace period of the replaced build: dropped; shared file stays.
        Retained::snapshot_at(
            &root,
            build(&["index-CCCC3333.js", "Dash-BBBB2222.js"]),
            day + GRACE_SECS,
        )
        .unwrap();
        assert!(r.read("assets/index-AAAA1111.js").is_none());
        assert!(r.read("assets/Dash-BBBB2222.js").is_some());
        // Count bound: only the newest MAX_BUILDS survive, even inside the grace.
        for i in 0..(MAX_BUILDS + 2) {
            let name = format!("index-N{i:07}x.js");
            Retained::snapshot_at(&root, vec![(name, vec![1])], day + GRACE_SECS + i as u64)
                .unwrap();
        }
        assert!(r.read("assets/index-N0000000x.js").is_none());
        assert!(r.read("assets/index-N0000002x.js").is_some());
        let manifest: Manifest =
            serde_json::from_slice(&std::fs::read(root.join(MANIFEST)).unwrap()).unwrap();
        assert_eq!(manifest.builds.len(), MAX_BUILDS);
    }

    #[test]
    fn only_hashed_plain_names_are_kept_or_read() {
        let dir = Scratch::new();
        let root = dir.path().to_path_buf();
        Retained::snapshot_at(
            &root,
            build(&["index-AAAA1111.js", "logo.svg", "readme.txt"]),
            0,
        )
        .unwrap();
        let r = Retained { dir: root.clone() };
        assert!(r.read("assets/index-AAAA1111.js").is_some());
        assert!(r.read("assets/logo.svg").is_none());
        assert!(r.read("assets/../builds.json").is_none());
        assert!(r.read("builds.json").is_none());
        assert!(r.read("assets/.index-AAAA1111.js.partial").is_none());
    }

    #[test]
    fn location_follows_the_database_unless_set() {
        assert_eq!(location(":memory:"), None);
        assert_eq!(location("postgres://u@h/db"), None);
        assert_eq!(
            location("/data/mailwoman.db"),
            Some(PathBuf::from("/data/retained-assets"))
        );
    }

    #[test]
    fn the_cleanup_worker_answers_only_worker_script_fetches_of_sw_js() {
        let mut sw = HeaderMap::new();
        sw.insert("service-worker", HeaderValue::from_static("script"));
        let resp = worker_fallback("mail/sw.js", &sw).unwrap();
        assert_eq!(resp.headers()[CLEANUP_HEADER], "cleanup");
        assert_eq!(resp.headers()[header::CACHE_CONTROL], "no-cache");
        assert!(worker_fallback("sw.js", &sw).is_some());
        assert!(worker_fallback("mail/sw.js", &HeaderMap::new()).is_none());
        assert!(worker_fallback("mail/other.js", &sw).is_none());
        assert!(worker_fallback("a/b/c/d/e/sw.js", &sw).is_none());
        assert!(worker_fallback("../sw.js", &sw).is_none());
        let mut dest = HeaderMap::new();
        dest.insert("sec-fetch-dest", HeaderValue::from_static("serviceworker"));
        assert!(worker_fallback("sw.js", &dest).is_some());
        assert!(!CLEANUP_WORKER.contains("'fetch'"), "no fetch handler");
    }
}
