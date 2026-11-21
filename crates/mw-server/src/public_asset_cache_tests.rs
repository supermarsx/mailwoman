//! Real filesystem loads and deterministic blocking-worker ownership, not timing races.
use super::*;
use axum::http::{HeaderMap, Method};
use std::sync::{
    Condvar,
    atomic::{AtomicUsize, Ordering},
};
struct Gate {
    open: Mutex<bool>,
    changed: Condvar,
    entered: Semaphore,
    calls: AtomicUsize,
}
impl Gate {
    fn new() -> Arc<Self> {
        Arc::new(Self {
            open: Mutex::new(false),
            changed: Condvar::new(),
            entered: Semaphore::new(0),
            calls: AtomicUsize::new(0),
        })
    }
    fn hook(self: &Arc<Self>) -> Arc<dyn Fn(&str) + Send + Sync> {
        let gate = self.clone();
        Arc::new(move |_| {
            gate.calls.fetch_add(1, Ordering::SeqCst);
            gate.entered.add_permits(1);
            let (open, timed) = gate
                .changed
                .wait_timeout_while(gate.open.lock().unwrap(), Duration::from_secs(4), |open| {
                    !*open
                })
                .unwrap();
            assert!(
                *open && !timed.timed_out(),
                "owned test gate must be released"
            );
        })
    }
    async fn entered(&self, n: u32) {
        tokio::time::timeout(Duration::from_secs(2), self.entered.acquire_many(n))
            .await
            .unwrap()
            .unwrap()
            .forget();
    }
    fn release(&self) {
        *self.open.lock().unwrap() = true;
        self.changed.notify_all();
    }
}
struct Release(Arc<Gate>);
impl Drop for Release {
    fn drop(&mut self) {
        self.0.release();
    }
}
fn fixture() -> (tempfile::TempDir, Arc<Cache>) {
    let directory = tempfile::tempdir().unwrap();
    let cache = Cache::new(Some(directory.path().into()), Arc::from("/mail"), None);
    cache.apply(Config {
        preload: Preload::Off,
        ..Config::default()
    });
    (directory, cache)
}
async fn until(cache: &Cache, predicate: impl Fn(&serde_json::Value) -> bool) {
    tokio::time::timeout(Duration::from_secs(3), async {
        loop {
            if predicate(&cache.status()) {
                break;
            }
            tokio::task::yield_now().await;
        }
    })
    .await
    .unwrap();
}
async fn get(cache: &Arc<Cache>, path: &str) -> Prepared {
    cache
        .get(Source::Web, Variant::Public, path)
        .await
        .unwrap()
        .unwrap()
}

#[tokio::test]
async fn same_key_singleflight_survives_waiter_cancellation_and_shares_bytes() {
    let (directory, cache) = fixture();
    std::fs::write(directory.path().join("entry.js"), b"one shared payload").unwrap();
    let gate = Gate::new();
    let _release = Release(gate.clone());
    *cache.before_load.lock().unwrap() = Some(gate.hook());
    let mut waits = Vec::new();
    for _ in 0..10 {
        let cache = cache.clone();
        waits.push(tokio::spawn(async move { get(&cache, "entry.js").await }));
    }
    gate.entered(1).await;
    until(&cache, |s| s["counters"]["coalesced"] == 9).await;
    waits.remove(0).abort();
    gate.release();
    let first = waits.remove(0).await.unwrap();
    for wait in waits {
        let next = wait.await.unwrap();
        assert_eq!(first.bytes.as_ptr(), next.bytes.as_ptr());
        assert_eq!(first.etag, next.etag);
    }
    assert_eq!(gate.calls.load(Ordering::SeqCst), 1);
    assert_eq!(cache.status()["counters"]["loads"], 1);
    until(&cache, |_| {
        cache.workers.available_permits() == WORKERS && cache.queue.available_permits() == QUEUED
    })
    .await;
}
#[tokio::test]
async fn clear_epoch_prevents_inflight_repopulation_and_disable_still_serves() {
    let (directory, cache) = fixture();
    std::fs::write(directory.path().join("a.js"), b"first").unwrap();
    let gate = Gate::new();
    let _release = Release(gate.clone());
    *cache.before_load.lock().unwrap() = Some(gate.hook());
    let waiting = {
        let cache = cache.clone();
        tokio::spawn(async move { get(&cache, "a.js").await })
    };
    gate.entered(1).await;
    cache.clear();
    gate.release();
    assert_eq!(&waiting.await.unwrap().bytes[..], b"first");
    assert_eq!(cache.status()["entries"], 0);
    assert_eq!(cache.status()["counters"]["staleInserts"], 1);
    *cache.before_load.lock().unwrap() = None;
    let first = get(&cache, "a.js").await;
    cache.apply(Config {
        enabled: false,
        preload: Preload::Core,
        ..Config::default()
    });
    assert_eq!(cache.status()["effectivePreload"], "off");
    std::fs::write(directory.path().join("a.js"), b"second").unwrap();
    let second = get(&cache, "a.js").await;
    assert_ne!(first.etag, second.etag);
    assert_eq!(cache.status()["entries"], 0);
}
#[tokio::test]
async fn queue_is_bounded_and_aborted_waiters_keep_real_worker_permits() {
    let (directory, cache) = fixture();
    let gate = Gate::new();
    let _release = Release(gate.clone());
    *cache.before_load.lock().unwrap() = Some(gate.hook());
    let mut tasks = Vec::new();
    for n in 0..WORKERS + QUEUED {
        let path = format!("{n}.js");
        std::fs::write(directory.path().join(&path), b"x").unwrap();
        let cache = cache.clone();
        tasks.push(tokio::spawn(async move { get(&cache, &path).await }));
        if n == 1 {
            gate.entered(2).await;
        }
    }
    until(&cache, |s| s["pendingUnique"] == WORKERS + QUEUED).await;
    assert!(
        cache
            .get(Source::Web, Variant::Public, "overflow.js")
            .await
            .is_err()
    );
    for task in tasks {
        task.abort();
    }
    assert_eq!(cache.workers.available_permits(), 0);
    assert_eq!(cache.status()["runningLoads"], 2);
    gate.release();
    until(&cache, |s| s["pendingUnique"] == 0).await;
    until(&cache, |_| {
        cache.workers.available_permits() == WORKERS && cache.queue.available_permits() == QUEUED
    })
    .await;
    assert_eq!(cache.status()["counters"]["loads"], WORKERS + QUEUED);
}
#[tokio::test]
async fn worker_failure_is_busy_not_spa_miss_and_releases_admission() {
    let (directory, cache) = fixture();
    std::fs::write(directory.path().join("a.js"), b"ok").unwrap();
    *cache.before_load.lock().unwrap() = Some(Arc::new(|_| panic!("owned loader failure")));
    assert!(
        cache
            .get(Source::Web, Variant::Public, "a.js")
            .await
            .is_err()
    );
    *cache.before_load.lock().unwrap() = None;
    assert_eq!(&get(&cache, "a.js").await.bytes[..], b"ok");
    until(&cache, |_| cache.workers.available_permits() == WORKERS).await;
    assert_eq!(cache.status()["counters"]["loadFailures"], 1);
}
#[tokio::test]
async fn mutable_file_expiry_reloads_bytes_etag_and_service_worker() {
    let (directory, cache) = fixture();
    std::fs::write(directory.path().join("sw.js"), b"old worker").unwrap();
    let old = get(&cache, "sw.js").await;
    assert_eq!(&get(&cache, "sw.js").await.bytes[..], b"old worker");
    {
        let mut state = cache.lock();
        for entry in state.entries.values_mut() {
            assert!(entry.expires.unwrap() <= Instant::now() + Duration::from_secs(1));
            entry.expires = Some(Instant::now());
        }
    }
    std::fs::write(directory.path().join("sw.js"), b"new worker").unwrap();
    let new = get(&cache, "sw.js").await;
    assert_ne!(old.etag, new.etag);
    assert_eq!(&new.bytes[..], b"new worker");
    assert_eq!(cache.status()["counters"]["expired"], 1);
}
#[tokio::test]
async fn owned_budget_entries_and_oversized_bypass_are_enforced() {
    let (directory, cache) = fixture();
    cache.apply(Config {
        budget_mib: 8,
        max_file_mib: 1,
        max_entries: 16,
        preload: Preload::Off,
        ..Config::default()
    });
    let payload = vec![b'x'; 1024 * 1024];
    for n in 0..9 {
        let path = format!("{n}.js");
        std::fs::write(directory.path().join(&path), &payload).unwrap();
        get(&cache, &path).await;
    }
    assert_eq!(cache.status()["residentOwnedBytes"], 8 * 1024 * 1024);
    assert_eq!(cache.status()["entries"], 8);
    assert_eq!(cache.status()["counters"]["evictions"], 1);
    std::fs::write(directory.path().join("big.js"), vec![b'x'; 1024 * 1024 + 1]).unwrap();
    get(&cache, "big.js").await;
    get(&cache, "big.js").await;
    assert_eq!(cache.status()["counters"]["oversize"], 2);
    cache.clear();
    for n in 0..17 {
        let path = format!("small{n}.js");
        std::fs::write(directory.path().join(&path), b"x").unwrap();
        get(&cache, &path).await;
    }
    assert_eq!(cache.status()["entries"], 16);
}
#[tokio::test]
async fn console_public_raw_and_prefix_namespaces_keep_headers_and_guards() {
    let (directory, cache) = fixture();
    let raw = b"<!doctype html><html><head><script src=\"./assets/a.js\"></script></head></html>";
    std::fs::write(directory.path().join("index.html"), raw).unwrap();
    let public = get(&cache, "index.html").await;
    let console = cache
        .get(Source::Web, Variant::Console, "index.html")
        .await
        .unwrap()
        .unwrap();
    let pristine = cache
        .get(Source::Web, Variant::Raw, "index.html")
        .await
        .unwrap()
        .unwrap();
    assert!(!String::from_utf8_lossy(&public.bytes).contains(crate::CONSOLE_MARKER));
    assert!(String::from_utf8_lossy(&console.bytes).contains(crate::CONSOLE_MARKER));
    assert_eq!(&pristine.bytes[..], raw);
    assert!(String::from_utf8_lossy(&console.bytes).contains("/mail/assets/a.js"));
    assert_ne!(public.etag, console.etag);
    let mut headers = HeaderMap::new();
    headers.insert(header::IF_NONE_MATCH, public.etag.clone());
    let response = crate::asset_response_prepared("index.html", "text/html", &public, &headers);
    assert_eq!(response.status(), StatusCode::NOT_MODIFIED);
    assert_eq!(response.headers()[header::CACHE_CONTROL], "no-cache");
    for path in [
        "../index.html",
        "..\\index.html",
        "x.map",
        "/index.html",
        "x/../index.html",
    ] {
        assert!(
            cache
                .get(Source::Web, Variant::Public, path)
                .await
                .unwrap()
                .is_none()
        );
    }
    assert!(
        cache
            .get(Source::Web, Variant::Console, "entry.js")
            .await
            .unwrap()
            .is_none()
    );
    let other = Cache::new(Some(directory.path().into()), Arc::from("/other"), None);
    let elsewhere = other
        .get(Source::Web, Variant::Console, "index.html")
        .await
        .unwrap()
        .unwrap();
    assert!(String::from_utf8_lossy(&elsewhere.bytes).contains("/other/assets/a.js"));
}
#[test]
fn borrowed_embedded_payload_has_no_owned_copy_and_clones_share_memory() {
    static VALUE: &[u8] = b"embedded raw bytes";
    let value = Prepared::new(Cow::Borrowed(VALUE));
    let clone = value.clone();
    assert_eq!(value.charge, 0);
    assert!(value.immutable);
    assert_eq!(value.bytes.as_ptr(), VALUE.as_ptr());
    assert_eq!(clone.bytes.as_ptr(), VALUE.as_ptr());
    let pdf = crate::pdf_assets::response(None, &Method::HEAD, "pdf.worker.mjs", &HeaderMap::new());
    assert_eq!(pdf.status(), StatusCode::OK);
    assert!(pdf.headers().contains_key(header::CONTENT_LENGTH));
    assert_eq!(pdf.headers()[header::ACCESS_CONTROL_ALLOW_ORIGIN], "*");
}

#[tokio::test]
async fn preload_does_not_queue_behind_foreground_or_run_when_disabled() {
    let (directory, cache) = fixture();
    std::fs::write(directory.path().join("a.js"), b"foreground").unwrap();
    let gate = Gate::new();
    let _release = Release(gate.clone());
    *cache.before_load.lock().unwrap() = Some(gate.hook());
    let foreground = {
        let cache = cache.clone();
        tokio::spawn(async move { get(&cache, "a.js").await })
    };
    gate.entered(1).await;
    assert!(
        cache
            .get_inner(Source::Web, Variant::Public, "index.html", true)
            .await
            .is_err()
    );
    assert_eq!(cache.status()["pendingUnique"], 1);
    assert_eq!(cache.status()["counters"]["preloadSkipped"], 1);
    gate.release();
    foreground.await.unwrap();
    cache.apply(Config {
        enabled: false,
        preload: Preload::Core,
        ..Config::default()
    });
    assert_eq!(cache.status()["effectivePreload"], "off");
}
#[test]
fn actual_static_router_keeps_head_etag_csp_console_and_admin_auth() {
    // Build the large production router future inside an owned stack, as the
    // existing authentication-boundary fixtures do; no listeners or global env.
    let worker = std::thread::Builder::new()
        .name("mw-public-asset-test".into())
        .stack_size(16 * 1024 * 1024)
        .spawn(|| {
            tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()
                .unwrap()
                .block_on(async {
                    use axum::{body::to_bytes, http::Request};
                    use tower::ServiceExt;
                    let directory = tempfile::tempdir().unwrap();
                    let web = directory.path().join("web");
                    std::fs::create_dir(&web).unwrap();
                    std::fs::write(
                        web.join("index.html"),
                        b"<!doctype html><html><head></head><body>owned shell</body></html>",
                    )
                    .unwrap();
                    std::fs::create_dir(web.join("assets")).unwrap();
                    let public_js=b"export const publicStaticFixture='content';\n".repeat(512);
                    std::fs::write(web.join("assets/public-ABC12345.js"),&public_js).unwrap();
                    let (router, _) = crate::build_app_full(
                        crate::AppConfig {
                            db_path: directory
                                .path()
                                .join("mail.db")
                                .to_string_lossy()
                                .into_owned(),
                            server_key_hex: Some(mw_store::ServerKey::generate().to_hex()),
                            web_dir: Some(web),
                            cookie_secure: false,
                            mode: crate::ServerMode::Engine,
                            hardening: Default::default(),
                            security: Default::default(),
                        },
                        crate::V6Config {
                            admin_enabled: true,
                            admin_username: Some("fixture-admin".into()),
                            admin_password: Some("fixture-only-password".into()),
                            redis_url: None,
                        },
                    )
                    .await
                    .unwrap();
                    let request = |method: Method, path: &str| {
                        Request::builder()
                            .method(method)
                            .uri(path)
                            .body(Body::empty())
                            .unwrap()
                    };
                    let public = router
                        .clone()
                        .oneshot(request(Method::GET, "/"))
                        .await
                        .unwrap();
                    assert_eq!(public.status(), StatusCode::OK);
                    let etag = public.headers()[header::ETAG].clone();
                    let csp = public.headers().get("content-security-policy").cloned();
                    assert!(csp.is_some());
                    let bytes = to_bytes(public.into_body(), 64 * 1024).await.unwrap();
                    assert!(!String::from_utf8_lossy(&bytes).contains(crate::CONSOLE_MARKER));
                    let head = router
                        .clone()
                        .oneshot(request(Method::HEAD, "/"))
                        .await
                        .unwrap();
                    assert_eq!(head.status(), StatusCode::OK);
                    assert_eq!(head.headers()[header::ETAG], etag);
                    assert_eq!(head.headers().get("content-security-policy"), csp.as_ref());
                    assert_eq!(
                        head.headers()[header::CONTENT_LENGTH],
                        bytes.len().to_string()
                    );
                    assert!(to_bytes(head.into_body(), 1).await.unwrap().is_empty());
                    let conditional = Request::builder()
                        .uri("/")
                        .header(header::IF_NONE_MATCH, etag)
                        .body(Body::empty())
                        .unwrap();
                    let unchanged = router.clone().oneshot(conditional).await.unwrap();
                    assert_eq!(unchanged.status(), StatusCode::NOT_MODIFIED);
                    assert!(to_bytes(unchanged.into_body(), 1).await.unwrap().is_empty());
                    // The actual production router must select the SAME encoding
                    // for GET/HEAD/304, not a raw-length HEAD over encoded bytes.
                    let encoded_request=|method:Method|Request::builder().method(method)
                        .uri("/assets/public-ABC12345.js").header(header::ACCEPT_ENCODING,"gzip")
                        .body(Body::empty()).unwrap();
                    let encoded=router.clone().oneshot(encoded_request(Method::GET)).await.unwrap();
                    assert_eq!(encoded.status(),StatusCode::OK);
                    assert_eq!(encoded.headers()[header::CONTENT_ENCODING],"gzip");
                    assert!(encoded.headers().get_all(header::VARY).iter().any(|v|v.to_str().unwrap().contains("Accept-Encoding")));
                    let encoded_tag=encoded.headers()[header::ETAG].clone();
                    let encoded_length=encoded.headers()[header::CONTENT_LENGTH].clone();
                    let encoded_body=to_bytes(encoded.into_body(),public_js.len()).await.unwrap();
                    let mut decoded=Vec::new();std::io::Read::read_to_end(&mut flate2::read::MultiGzDecoder::new(encoded_body.as_ref()),&mut decoded).unwrap();
                    assert_eq!(decoded,public_js);
                    let encoded_head=router.clone().oneshot(encoded_request(Method::HEAD)).await.unwrap();
                    assert_eq!(encoded_head.status(),StatusCode::OK);assert_eq!(encoded_head.headers()[header::CONTENT_LENGTH],encoded_length);
                    assert_eq!(encoded_head.headers()[header::ETAG],encoded_tag);
                    assert!(to_bytes(encoded_head.into_body(),1).await.unwrap().is_empty());
                    let mut revalidate=encoded_request(Method::GET);revalidate.headers_mut().insert(header::IF_NONE_MATCH,encoded_tag.clone());
                    let encoded_304=router.clone().oneshot(revalidate).await.unwrap();assert_eq!(encoded_304.status(),StatusCode::NOT_MODIFIED);
                    assert_eq!(encoded_304.headers()[header::CONTENT_ENCODING],"gzip");assert!(to_bytes(encoded_304.into_body(),1).await.unwrap().is_empty());
                    let console_path = format!("/{}", crate::console_page());
                    // An extensionless console page is a real navigation only
                    // with browser navigation metadata (or an HTML Accept).
                    // Bare API/subresource-like requests must stay fail-closed.
                    let bare_console = router
                        .clone()
                        .oneshot(request(Method::GET, &console_path))
                        .await
                        .unwrap();
                    assert_eq!(bare_console.status(), StatusCode::NOT_FOUND);
                    let navigation = Request::builder()
                        .method(Method::GET)
                        .uri(&console_path)
                        .header(header::ACCEPT, "text/html")
                        .header("sec-fetch-mode", "navigate")
                        .header("sec-fetch-dest", "document")
                        .body(Body::empty())
                        .unwrap();
                    let console = router.clone().oneshot(navigation).await.unwrap();
                    assert_eq!(console.status(), StatusCode::OK);
                    assert_eq!(
                        console.headers()[header::CACHE_CONTROL],
                        "private, no-cache"
                    );
                    assert!(
                        console
                            .headers()
                            .get_all("content-security-policy")
                            .iter()
                            .any(|v| v.to_str().unwrap().contains("frame-ancestors 'none'"))
                    );
                    assert!(
                        String::from_utf8_lossy(
                            &to_bytes(console.into_body(), 64 * 1024).await.unwrap()
                        )
                        .contains(crate::CONSOLE_MARKER)
                    );
                    let map = router
                        .clone()
                        .oneshot(request(Method::GET, "/entry.js.map"))
                        .await
                        .unwrap();
                    assert_eq!(map.status(), StatusCode::NOT_FOUND);
                    let cross_origin = router.clone().oneshot(request(
                        Method::POST,
                        "/admin/maintenance/web-assets/clear",
                    )).await.unwrap();
                    assert_eq!(cross_origin.status(), StatusCode::FORBIDDEN);
                    for (method, path) in [
                        (Method::GET, "/admin/maintenance/web-assets"),
                        (Method::POST, "/admin/maintenance/web-assets/clear"),
                    ] {
                        // Pass the existing origin guard so this request actually
                        // reaches the independent administrator authentication gate.
                        let mut incoming = request(method.clone(), path);
                        incoming.headers_mut().insert(header::HOST, "localhost".parse().unwrap());
                        incoming.headers_mut().insert(header::ORIGIN, "http://localhost".parse().unwrap());
                        let denied = router.clone().oneshot(incoming).await.unwrap();
                        assert_eq!(denied.status(), StatusCode::UNAUTHORIZED, "{method} {path}");
                    }
                });
        })
        .unwrap();
    if let Err(panic) = worker.join() {
        std::panic::resume_unwind(panic);
    }
}

#[test]
fn configuration_wire_names_match_registry_and_browser_contract() {
    let value = serde_json::to_value(Config::default()).unwrap();
    assert_eq!(value.get("budgetMiB"), Some(&serde_json::json!(64)));
    assert_eq!(value.get("maxFileMiB"), Some(&serde_json::json!(4)));
    assert!(value.get("budgetMib").is_none());
    assert!(value.get("maxFileMib").is_none());
}
#[test]
fn out_of_order_refresh_completion_cannot_revert_newer_admin_success_or_staleness() {
    let (_directory, cache) = fixture();
    let old = cache.begin_refresh();
    let latest = cache.begin_refresh();
    let newest = Config {
        enabled: false,
        budget_mib: 8,
        preload: Preload::Off,
        ..Config::default()
    };
    assert!(cache.complete_refresh(latest, Ok(newest)));
    let accepted = cache.status();
    assert!(!cache.complete_refresh(old, Ok(Config::default())));
    assert_eq!(cache.status(), accepted);
    assert!(!cache.complete_refresh(old, Err(())));
    assert_eq!(cache.status(), accepted);
    let failed = cache.begin_refresh();
    assert!(cache.complete_refresh(failed, Err(())));
    assert_eq!(cache.status()["configurationStale"], true);
    assert_eq!(cache.status()["config"]["enabled"], false);
    assert!(!cache.complete_refresh(latest, Ok(Config::default())));
    assert_eq!(cache.status()["configurationStale"], true);
}
#[test]
fn core_discovery_is_size_count_and_path_length_bounded() {
    let mut oversized = vec![b' '; MAX_DISCOVERY_BYTES + 1];
    let tag = b"<script src=\"./assets/not-discovered.js\"></script>";
    oversized[..tag.len()].copy_from_slice(tag);
    assert_eq!(
        core_paths(&oversized, "/mail"),
        vec!["sw.js", "manifest.webmanifest"]
    );
    let html = (0..30)
        .map(|n| format!("<script src=\"/mail/assets/{n}.js\"></script>"))
        .collect::<String>();
    let paths = core_paths(html.as_bytes(), "/mail");
    assert_eq!(paths.len(), 18);
    assert_eq!(paths.last().unwrap(), "assets/15.js");
    let giant = format!("<script src=\"./assets/{}.js\"></script>", "a".repeat(4096));
    assert_eq!(core_paths(giant.as_bytes(), "/mail").len(), 2);
}
#[tokio::test]
async fn pdf_embedded_fallback_revalidates_into_matching_deployed_override() {
    let (directory, cache) = fixture();
    std::fs::create_dir_all(directory.path().join("pdfjs/build")).unwrap();
    std::fs::write(
        directory.path().join("pdfjs/manifest.json"),
        b"{\"version\":\"stale\"}",
    )
    .unwrap();
    std::fs::write(
        directory.path().join("pdfjs/build/pdf.mjs"),
        b"fixture permitted override",
    )
    .unwrap();
    let path = "pdfjs/build/pdf.mjs";
    let embedded = mw_web_assets::pdf(path).unwrap();
    let fallback = cache
        .get(Source::Pdf, Variant::Public, path)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(&fallback.bytes[..], embedded.as_ref());
    assert!(!fallback.immutable);
    {
        let mut state = cache.lock();
        for entry in state.entries.values_mut() {
            assert!(entry.expires.is_some());
            entry.expires = Some(Instant::now());
        }
    }
    std::fs::write(
        directory.path().join("pdfjs/manifest.json"),
        mw_web_assets::pdf("pdfjs/manifest.json").unwrap().as_ref(),
    )
    .unwrap();
    let replaced = cache
        .get(Source::Pdf, Variant::Public, path)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(&replaced.bytes[..], b"fixture permitted override");
    assert_ne!(replaced.etag, fallback.etag);
    assert_eq!(cache.status()["counters"]["expired"], 1);
}

#[tokio::test]
async fn adapted_prefix_css_is_encoded_once_on_miss_and_policy_changes_clear_variants() {
    use crate::public_asset_compression::Coding;
    let (directory,cache)=fixture();
    let raw=b".fixture{background:url('/fonts/fixture.woff2')}\n".repeat(512);
    let expected=crate::adapt_asset_to_base("fixture.css",raw.clone(),"/mail");
    assert_ne!(expected,raw,"fixture must actually exercise prefix adaptation");
    std::fs::write(directory.path().join("fixture.css"),&raw).unwrap();
    let sidecar=crate::public_asset_compression::prepare(&raw,true,1024*1024,|_|None);
    std::fs::write(directory.path().join("fixture.css.gz"),&sidecar.gzip.unwrap().bytes).unwrap();
    let first=get(&cache,"fixture.css").await;
    let gzip=first.encodings.gzip.as_ref().unwrap();
    assert!(gzip.generated,"the old unadapted sidecar cannot certify prefix CSS");
    let mut decoder=flate2::read::MultiGzDecoder::new(gzip.bytes.as_ref());
    let mut decoded=Vec::new();std::io::Read::read_to_end(&mut decoder,&mut decoded).unwrap();
    assert_eq!(decoded,expected);
    let before=cache.status();let second=get(&cache,"fixture.css").await;let after=cache.status();
    assert_eq!(before["counters"]["generatedEncodings"],after["counters"]["generatedEncodings"]);
    assert_eq!(first.encodings.get(Coding::Gzip).unwrap().bytes.as_ptr(),second.encodings.get(Coding::Gzip).unwrap().bytes.as_ptr());
    assert!(after["residentOwnedBytes"].as_u64().unwrap()<=after["config"]["budgetMiB"].as_u64().unwrap()*1024*1024);
    cache.apply(Config {preload:Preload::Off,compression_enabled:false,..Default::default()});
    let disabled=get(&cache,"fixture.css").await;
    assert!(disabled.encodings.gzip.is_none()&&disabled.encodings.brotli.is_none());
    cache.apply(Config {enabled:false,preload:Preload::Off,..Default::default()});
    let generated=cache.status()["counters"]["generatedEncodings"].clone();
    for _ in 0..2 {let uncached=get(&cache,"fixture.css").await;assert!(uncached.encodings.gzip.is_none()&&uncached.encodings.brotli.is_none());}
    assert_eq!(cache.status()["compressionEffectiveEnabled"],false);
    assert_eq!(cache.status()["counters"]["generatedEncodings"],generated,"cache-off cannot generate on every request");
}
