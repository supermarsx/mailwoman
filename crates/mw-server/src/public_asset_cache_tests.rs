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
async fn full_admission_waits_for_a_slot_and_aborted_waiters_keep_real_worker_permits() {
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
    // A further unique miss is not refused: it waits for a dispatcher slot.
    std::fs::write(directory.path().join("overflow.js"), b"late").unwrap();
    let overflow = {
        let cache = cache.clone();
        tokio::spawn(async move { get(&cache, "overflow.js").await })
    };
    until(&cache, |s| s["admissionWaiting"] == 1).await;
    assert_eq!(cache.status()["pendingUnique"], WORKERS + QUEUED);
    for task in tasks {
        task.abort();
    }
    assert_eq!(cache.workers.available_permits(), 0);
    assert_eq!(cache.status()["runningLoads"], 2);
    gate.release();
    assert_eq!(&overflow.await.unwrap().bytes[..], b"late");
    until(&cache, |s| s["pendingUnique"] == 0 && s["admissionWaiting"] == 0).await;
    until(&cache, |_| {
        cache.workers.available_permits() == WORKERS && cache.queue.available_permits() == QUEUED
    })
    .await;
    assert_eq!(cache.status()["counters"]["loads"], WORKERS + QUEUED + 1);
    assert_eq!(cache.status()["counters"]["busy"], 0);
    assert_eq!(cache.status()["counters"]["admissionWaits"], 1);
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
    // A different length, so the stamp differs even within one mtime tick.
    std::fs::write(directory.path().join("sw.js"), b"new worker v2").unwrap();
    let new = get(&cache, "sw.js").await;
    assert_ne!(old.etag, new.etag);
    assert_eq!(&new.bytes[..], b"new worker v2");
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
    let mut entry_bytes=0;
    for n in 0..9 {
        let path = format!("{n}.js");
        std::fs::write(directory.path().join(&path), &payload).unwrap();
        let prepared=get(&cache, &path).await;
        let gzip=prepared.encodings.gzip.as_ref().expect("compressible fixture must retain gzip");
        let brotli=prepared.encodings.brotli.as_ref().expect("compressible fixture must retain Brotli");
        let actual=payload.len()+gzip.bytes.len()+brotli.bytes.len();
        assert_eq!(prepared.charge,actual,"all three owned representations count toward the budget");
        if n==0 {entry_bytes=actual;} else {assert_eq!(entry_bytes,actual);}
    }
    let capacity=8*1024*1024/entry_bytes;
    assert_eq!(capacity,7,"eight raw1MiB files leave no room for their encodings");
    assert_eq!(cache.status()["residentOwnedBytes"],capacity*entry_bytes);
    assert!(capacity*entry_bytes<=8*1024*1024);
    assert_eq!(cache.status()["entries"],capacity);
    assert_eq!(cache.status()["counters"]["evictions"],9-capacity);
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
    assert_eq!(value.get("budgetMiB"), Some(&serde_json::json!(128)));
    assert_eq!(value.get("maxFileMiB"), Some(&serde_json::json!(8)));
    assert_eq!(value.get("maxEntries"), Some(&serde_json::json!(4096)));
    assert_eq!(value.get("preload"), Some(&serde_json::json!("app")));
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

fn write(root: &std::path::Path, path: &str, bytes: &[u8]) {
    let file = root.join(path);
    std::fs::create_dir_all(file.parent().unwrap()).unwrap();
    std::fs::write(file, bytes).unwrap();
}
fn resident(cache: &Cache) -> std::collections::BTreeSet<String> {
    let s = cache.lock();
    s.entries
        .keys()
        .filter(|key| key.generation == s.generation)
        .map(|key| key.path.clone())
        .collect()
}
fn expire_mutable(cache: &Cache) {
    for entry in cache.lock().entries.values_mut() {
        if entry.expires.is_some() {
            entry.expires = Some(Instant::now());
        }
    }
}

#[tokio::test]
async fn deployed_hashed_entry_outlives_revalidate_seconds_and_unhashed_does_not() {
    let (directory, cache) = fixture();
    write(directory.path(), "assets/app-AbCd1234.js", b"hashed build bytes");
    write(directory.path(), "sw.js", b"unhashed worker");
    let first = get(&cache, "assets/app-AbCd1234.js").await;
    get(&cache, "sw.js").await;
    {
        let s = cache.lock();
        for (key, entry) in &s.entries {
            match key.path.as_str() {
                "assets/app-AbCd1234.js" => {
                    assert!(entry.expires.is_none() && entry.stamp.is_none())
                }
                "sw.js" => assert!(entry.expires.is_some() && entry.stamp.is_some()),
                other => panic!("unexpected entry {other}"),
            }
        }
    }
    // Disk changes under a hashed name are not re-read; a new build has new names.
    write(directory.path(), "assets/app-AbCd1234.js", b"rewritten in place");
    tokio::time::sleep(Duration::from_millis(1100)).await;
    let later = get(&cache, "assets/app-AbCd1234.js").await;
    assert_eq!(later.bytes.as_ptr(), first.bytes.as_ptr());
    assert_eq!(&later.bytes[..], b"hashed build bytes");
    let status = cache.status();
    assert_eq!(status["counters"]["loads"], 2);
    assert_eq!(status["counters"]["expired"], 0);
}

#[tokio::test]
async fn expired_unhashed_entry_with_unchanged_stamp_is_kept_without_reread() {
    let (directory, cache) = fixture();
    write(directory.path(), "sw.js", &b"// worker\n".repeat(400));
    let old = get(&cache, "sw.js").await;
    assert!(old.encodings.gzip.is_some(), "fixture must exercise encoding work");
    let generated = cache.status()["counters"]["generatedEncodings"].clone();
    expire_mutable(&cache);
    let kept = get(&cache, "sw.js").await;
    assert_eq!(kept.bytes.as_ptr(), old.bytes.as_ptr());
    assert_eq!(kept.etag, old.etag);
    let status = cache.status();
    assert_eq!(status["counters"]["expired"], 1);
    assert_eq!(status["counters"]["revalidatedUnchanged"], 1);
    assert_eq!(status["counters"]["generatedEncodings"], generated);
    assert_eq!(status["entries"], 1);
    assert_eq!(status["residentOwnedBytes"].as_u64().unwrap() as usize, old.charge);
}

#[tokio::test]
async fn shell_change_bumps_generation_and_drops_the_previous_build() {
    let (directory, cache) = fixture();
    write(directory.path(), "index.html", b"<script src=\"./assets/a-AbCd1234.js\"></script>");
    write(directory.path(), "assets/a-AbCd1234.js", b"build one");
    get(&cache, "index.html").await;
    get(&cache, "assets/a-AbCd1234.js").await;
    let before = cache.status()["generation"].as_u64().unwrap();
    // Unchanged shell: expiry keeps the generation and the set.
    expire_mutable(&cache);
    get(&cache, "index.html").await;
    assert_eq!(cache.status()["generation"], before);
    assert_eq!(resident(&cache).len(), 2);
    // A new build: new shell (different length), new hashed names.
    write(directory.path(), "index.html", b"<script src=\"./assets/b-EfGh5678.js\"></script> ");
    write(directory.path(), "assets/b-EfGh5678.js", b"build two");
    expire_mutable(&cache);
    let shell = get(&cache, "index.html").await;
    assert!(String::from_utf8_lossy(&shell.bytes).contains("b-EfGh5678.js"));
    let status = cache.status();
    assert_eq!(status["generation"], before + 1);
    assert_eq!(status["counters"]["shellChanges"], 1);
    assert_eq!(status["counters"]["clears"], 0);
    assert_eq!(resident(&cache), ["index.html".to_string()].into());
    assert_eq!(*cache.preload.borrow(), before + 1, "the preloader is told to run again");
    // The old hashed file is no longer resident; a request reads disk again.
    let misses = status["counters"]["misses"].as_u64().unwrap();
    get(&cache, "assets/a-AbCd1234.js").await;
    assert_eq!(cache.status()["counters"]["misses"], misses + 1);
}

#[tokio::test]
async fn shell_load_from_an_older_generation_cannot_bump_or_insert() {
    let (directory, cache) = fixture();
    write(directory.path(), "index.html", b"shell one");
    get(&cache, "index.html").await;
    let gate = Gate::new();
    let _release = Release(gate.clone());
    *cache.before_load.lock().unwrap() = Some(gate.hook());
    write(directory.path(), "index.html", b"shell two, longer");
    let raw = {
        let cache = cache.clone();
        tokio::spawn(async move {
            cache.get(Source::Web, Variant::Raw, "index.html").await.unwrap().unwrap()
        })
    };
    gate.entered(1).await;
    cache.clear();
    let generation = cache.status()["generation"].clone();
    gate.release();
    assert_eq!(&raw.await.unwrap().bytes[..], b"shell two, longer");
    let status = cache.status();
    assert_eq!(status["generation"], generation);
    assert_eq!(status["counters"]["shellChanges"], 0);
    assert_eq!(status["counters"]["staleInserts"], 1);
}

fn app_build(root: &std::path::Path) {
    write(
        root,
        "index.html",
        b"<link rel=\"stylesheet\" href=\"./assets/index-Css12345.css\"><script type=\"module\" src=\"./assets/index-AbCd1234.js\"></script>",
    );
    write(root, "sw.js", b"self.addEventListener('fetch',()=>{});");
    write(root, "manifest.webmanifest", b"{}");
    write(root, "favicon.svg", b"<svg/>");
    write(root, "assets/index-AbCd1234.js", b"entry");
    write(root, "assets/index-AbCd1234.js.br", b"not a key");
    write(root, "assets/index-AbCd1234.js.map", b"{}");
    write(root, "assets/index-Css12345.css", b"body{}");
    write(root, "assets/Mailbox-Ef567890.css", b".m{}");
    write(root, "assets/networkWorker-Byfrw9g2.js", b"worker");
    write(root, "assets/mw_crypto_bg-DbOVROPL.wasm", b"\0asm");
    write(root, "assets/mw_sanitize_bg-De1o9umO.wasm", b"\0asm");
    write(root, "assets/mail-settings.css", b"hand written, unhashed");
    write(root, "assets/catalog.en.app-DZnSKoWf.json", b"{}");
    write(root, "assets/catalog.en.admin-Q1w2E3r4.json", b"{}");
    write(root, "assets/catalog.nl.app-Bao7Q7Kh.json", b"{}");
    write(root, "assets/sender-Pq9Rs8Tu.png", b"png");
    write(root, "fonts/jost-latin-Jk1Lm2No.woff2", b"font");
    write(root, "pdfjs/build/pdf-Zz9Yy8Xx.mjs", b"pdf");
}
fn app_set() -> std::collections::BTreeSet<String> {
    [
        "index.html",
        "sw.js",
        "manifest.webmanifest",
        "assets/index-AbCd1234.js",
        "assets/index-Css12345.css",
        "assets/Mailbox-Ef567890.css",
        "assets/networkWorker-Byfrw9g2.js",
        "assets/mw_crypto_bg-DbOVROPL.wasm",
        "assets/mw_sanitize_bg-De1o9umO.wasm",
        "assets/catalog.en.app-DZnSKoWf.json",
    ]
    .into_iter()
    .map(String::from)
    .collect()
}

#[tokio::test]
async fn app_preload_loads_the_listed_set_and_reports_progress() {
    let (directory, cache) = fixture();
    app_build(directory.path());
    cache.apply(Config::default());
    assert_eq!(cache.status()["effectivePreload"], "app");
    cache.start_preloader();
    until(&cache, |s| s["preloadProgress"]["state"] == "done").await;
    let expected = app_set();
    assert_eq!(resident(&cache), expected);
    let status = cache.status();
    let progress = &status["preloadProgress"];
    assert_eq!(progress["planned"], expected.len());
    assert_eq!(progress["ready"], expected.len());
    assert_eq!(progress["absent"], 0);
    assert_eq!(progress["skipped"], 0);
    assert_eq!(progress["catalogs"], 1);
    assert_eq!(status["counters"]["preloadLoaded"], expected.len());
    // Every preloaded hashed file is a permanent entry.
    for (key, entry) in &cache.lock().entries {
        assert_eq!(entry.expires.is_none(), crate::is_content_hashed(&key.path), "{}", key.path);
    }
    // A clear starts a new pass that fills the same set again.
    cache.clear();
    until(&cache, |s| {
        s["preloadProgress"]["state"] == "done" && s["preloadProgress"]["generation"] == s["generation"]
    })
    .await;
    assert_eq!(resident(&cache), expected);
}

#[tokio::test]
async fn app_preload_yields_to_foreground_misses() {
    let (directory, cache) = fixture();
    app_build(directory.path());
    write(directory.path(), "late.js", b"foreground");
    cache.apply(Config::default());
    let gate = Gate::new();
    let _release = Release(gate.clone());
    *cache.before_load.lock().unwrap() = Some(gate.hook());
    let foreground = {
        let cache = cache.clone();
        tokio::spawn(async move { get(&cache, "late.js").await })
    };
    gate.entered(1).await;
    cache.start_preloader();
    // With a foreground miss pending, the preloader starts nothing.
    tokio::time::sleep(Duration::from_millis(100)).await;
    assert_eq!(gate.calls.load(Ordering::SeqCst), 1);
    assert_eq!(cache.status()["preloadProgress"]["state"], "running");
    gate.release();
    foreground.await.unwrap();
    until(&cache, |s| s["preloadProgress"]["state"] == "done").await;
    assert_eq!(cache.status()["preloadProgress"]["skipped"], 0);
    assert_eq!(resident(&cache).len(), app_set().len() + 1);
}

#[tokio::test]
async fn app_preload_stops_at_three_quarters_of_the_entry_limit() {
    let (directory, cache) = fixture();
    app_build(directory.path());
    for n in 0..40 {
        write(directory.path(), &format!("assets/chunk{n}-AbCd{n:04}.js"), b"x");
    }
    cache.apply(Config {
        max_entries: 16,
        ..Config::default()
    });
    cache.start_preloader();
    until(&cache, |s| s["preloadProgress"]["state"] == "limited").await;
    assert_eq!(cache.status()["entries"], 12);
    assert_eq!(cache.status()["counters"]["evictions"], 0);
}

#[test]
fn build_walk_skips_links_junctions_and_deep_or_unsafe_names() {
    let outside = tempfile::tempdir().unwrap();
    write(outside.path(), "secret-AbCd1234.js", b"outside");
    let root = tempfile::tempdir().unwrap();
    write(root.path(), "assets/real-AbCd1234.js", b"inside");
    let mut deep = String::from("assets");
    for n in 0..MAX_WALK_DEPTH + 1 {
        deep.push_str(&format!("/d{n}"));
    }
    write(root.path(), &format!("{deep}/deep-AbCd1234.js"), b"deep");
    let mut links = 0;
    #[cfg(unix)]
    {
        std::os::unix::fs::symlink(
            outside.path().join("secret-AbCd1234.js"),
            root.path().join("assets/file-AbCd1234.js"),
        )
        .unwrap();
        std::os::unix::fs::symlink(outside.path(), root.path().join("assets/dir")).unwrap();
        links += 2;
    }
    #[cfg(windows)]
    {
        // Symbolic links need a privilege a test host may lack; junctions do not.
        if std::os::windows::fs::symlink_file(
            outside.path().join("secret-AbCd1234.js"),
            root.path().join("assets/file-AbCd1234.js"),
        )
        .is_ok()
        {
            links += 1;
        }
        if std::os::windows::fs::symlink_dir(outside.path(), root.path().join("assets/dir")).is_ok() {
            links += 1;
        }
        let made = std::process::Command::new("cmd")
            .args(["/C", "mklink", "/J"])
            .arg(root.path().join("assets").join("junction"))
            .arg(outside.path())
            .output()
            .unwrap();
        assert!(made.status.success(), "junction fixture: {made:?}");
        assert!(root.path().join("assets/junction/secret-AbCd1234.js").exists());
        links += 1;
    }
    assert!(links >= 1, "at least one link fixture must exist");
    let listed = walk(root.path());
    assert_eq!(listed, ["assets/real-AbCd1234.js".to_string()], "links, junctions and over-deep files are not listed");
    #[cfg(windows)]
    std::fs::remove_dir(root.path().join("assets").join("junction")).unwrap();
    // The plan never contains anything the request guard would refuse.
    let plan = app_paths(
        &[
            "assets/ok-AbCd1234.js".into(),
            "assets/..%2fx-AbCd1234.js".into(),
            "assets/x-AbCd1234.js.map".into(),
            "assets/x-AbCd1234.js.gz".into(),
            "fonts/f-AbCd1234.woff2".into(),
            "assets/catalog.en.app-AbCd1234.json".into(),
            "assets/catalog.en.app-AbCd1234.json.br".into(),
            "assets/catalog.en.admin-AbCd1234.json".into(),
            "assets/catalog.de.app-AbCd1234.json".into(),
            "assets/catalog.en.app-x/y-AbCd1234.json".into(),
        ],
        None,
        "",
    );
    assert_eq!(plan, ["assets/ok-AbCd1234.js", "assets/catalog.en.app-AbCd1234.json"]);
}

#[tokio::test]
async fn large_wasm_is_retained_under_default_limits_and_brotli_negotiated() {
    use crate::public_asset_compression::Coding;
    let (directory, cache) = fixture();
    // 6.6 MB, the size of the current mw_crypto build; compressible but not trivial.
    let mut raw = Vec::with_capacity(6_610_000);
    let mut state = 0x2545_f491_u32;
    while raw.len() < 6_610_000 {
        state ^= state << 13;
        state ^= state >> 17;
        state ^= state << 5;
        raw.extend_from_slice(&[b'a' + (state % 16) as u8, b' ', b'(', b')']);
    }
    let path = "assets/mw_crypto_bg-DbOVROPL.wasm";
    write(directory.path(), path, &raw);
    let sidecar = crate::public_asset_compression::prepare(&raw, true, 64 * 1024 * 1024, |_| None);
    let brotli = sidecar.brotli.expect("fixture compresses").bytes;
    write(directory.path(), &format!("{path}.br"), &brotli);
    let prepared = get(&cache, path).await;
    let status = cache.status();
    assert_eq!(status["config"]["maxFileMiB"], 8);
    assert_eq!(status["entries"], 1, "{status}");
    assert_eq!(status["counters"]["oversize"], 0);
    assert!(status["counters"]["validatedSidecars"].as_u64().unwrap() >= 1);
    assert!(!prepared.encodings.brotli.as_ref().unwrap().generated);
    let mut headers = axum::http::HeaderMap::new();
    headers.insert(header::ACCEPT_ENCODING, HeaderValue::from_static("gzip, deflate, br"));
    let response = crate::asset_response_prepared(path, "application/wasm", &prepared, &headers);
    assert_eq!(response.status(), StatusCode::OK);
    assert_eq!(response.headers()[header::CONTENT_ENCODING], "br");
    assert_eq!(response.headers()[header::CONTENT_LENGTH], brotli.len().to_string());
    assert_eq!(response.headers()[header::CACHE_CONTROL], "public, max-age=31536000, immutable");
    assert_eq!(
        prepared.encodings.get(Coding::Brotli).unwrap().bytes.as_ref(),
        &brotli[..]
    );
}

/// 100 distinct cold files requested at the same instant (a barrier releases
/// all requests together), each load holding its worker briefly.
async fn hundred_simultaneous_distinct_misses(cache: Arc<Cache>, source: Source, paths: Vec<String>) {
    assert_eq!(paths.len(), 100);
    *cache.before_load.lock().unwrap() =
        Some(Arc::new(|_| std::thread::sleep(Duration::from_millis(5))));
    let start = Arc::new(tokio::sync::Barrier::new(paths.len()));
    let mut tasks = Vec::new();
    for path in paths.clone() {
        let (cache, start) = (cache.clone(), start.clone());
        tasks.push(tokio::spawn(async move {
            start.wait().await;
            cache.get(source, Variant::Public, &path).await
        }));
    }
    for (task, path) in tasks.into_iter().zip(&paths) {
        let prepared = task.await.unwrap().unwrap_or_else(|_| panic!("503 for {path}"));
        assert!(prepared.is_some(), "{path} exists");
    }
    let status = cache.status();
    assert_eq!(status["counters"]["busy"], 0, "{status}");
    assert_eq!(status["counters"]["loads"], 100);
    assert!(status["counters"]["admissionWaits"].as_u64().unwrap() > 0, "{status}");
    assert_eq!(status["admissionWaiting"], 0);
    assert_eq!(status["pendingUnique"], 0);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn hundred_simultaneous_cold_requests_get_no_busy_answer_from_a_deployed_directory() {
    let (directory, cache) = fixture();
    let paths: Vec<String> = (0..100).map(|n| format!("assets/c{n}-AbCd{n:04}.js")).collect();
    for (n, path) in paths.iter().enumerate() {
        write(directory.path(), path, format!("chunk {n}").repeat(200).as_bytes());
    }
    hundred_simultaneous_distinct_misses(cache, Source::Web, paths).await;
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn hundred_simultaneous_cold_requests_get_no_busy_answer_from_the_embedded_build() {
    // The embedded pdf.js table is present in every checkout, unlike apps/web/dist.
    let cache = Cache::new(None, Arc::from(""), None);
    cache.apply(Config {
        preload: Preload::Off,
        ..Config::default()
    });
    let manifest: serde_json::Value =
        serde_json::from_slice(&mw_web_assets::pdf("pdfjs/manifest.json").unwrap()).unwrap();
    let paths: Vec<String> = manifest["files"]
        .as_object()
        .unwrap()
        .keys()
        .map(|name| format!("pdfjs/{name}"))
        .filter(|path| mw_web_assets::pdf(path).is_some())
        .take(100)
        .collect();
    hundred_simultaneous_distinct_misses(cache, Source::Pdf, paths).await;
}

#[tokio::test]
async fn embedded_file_above_the_owned_file_limit_still_gets_encodings() {
    // Before, a borrowed embedded file over maxFileMiB (the 6.6 MB wasm in a
    // release build) was cached but always served identity. Borrowed bytes cost
    // no owned copy, so the limit does not apply to them; owned (deployed) files
    // above it are still served identity and not retained.
    static BIG: [u8; 2 * 1024 * 1024] = [b'a'; 2 * 1024 * 1024];
    let config = Config {
        max_file_mib: 1,
        ..Config::default()
    };
    assert!(config.encodes(&Prepared::new(Cow::Borrowed(&BIG))));
    assert!(!config.encodes(&Prepared::new(Cow::Owned(BIG.to_vec()))));
    assert!(config.encodes(&Prepared::new(Cow::Owned(vec![b'a'; 1024 * 1024]))));
    let (directory, owned) = fixture();
    owned.apply(Config {
        preload: Preload::Off,
        ..config
    });
    write(directory.path(), "assets/big-AbCd1234.js", &BIG);
    let deployed = get(&owned, "assets/big-AbCd1234.js").await;
    assert!(deployed.encodings.gzip.is_none() && deployed.encodings.brotli.is_none());
    assert_eq!(owned.status()["counters"]["oversize"], 1);
    assert_eq!(owned.status()["entries"], 0);
}
