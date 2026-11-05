//! t32 e31a — signed remote-content URLs (`/api/rc/*`), end to end.
//!
//! A user signs in (proxy mode, mock JMAP upstream), holds a grant for a message,
//! and asks `POST /api/rc/sign` for image and part URLs. The image URL is then
//! fetched **without a cookie**, as the sandboxed body frame does. Remote origins
//! are `http://1.2.3.4/…` through a stand-in proxy route that answers the
//! tunnelled request itself, so nothing leaves the machine; the stand-in records
//! what it was sent and how many connections it saw.
//!
//! The token format is checked from outside: this file derives the signing key
//! from the server key with its own HMAC code and re-mints the server's token
//! byte for byte, then forges expired and retargeted variants with it.
//!
//! Run:
//!   cargo test -p mw-server --test t32_remote_content -- --test-threads=1

use std::net::SocketAddr;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use base64::Engine as _;
use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use hmac::{Hmac, KeyInit, Mac};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};

use mw_server::{AppConfig, V6Config, build_app_full};
use mw_store::{ServerKey, Store};

mod common;
use common::test_db;

const KEY_HEX: &str = "2f2e3d4c5b6a79889796a5b4c3d2e1f00f1e2d3c4b5a69788796a5b4c3d2e1f2";
const INDEX_HTML: &str = "<!doctype html><title>Mailwoman</title><div id=app>MW</div>";

#[tokio::test]
async fn automatic_capacity_presets_are_live_and_preserve_numeric_ceilings() {
    let (proxy, _) = standin_proxy(vec![("image/png".into(), png(64, 64))]).await;
    let f = fixture(proxy, "mw-image-auto-presets").await;
    let read = || async {
        f.admin
            .get(format!("{}/admin/egress/privacy-stats", f.base))
            .send()
            .await
            .unwrap()
            .json::<Value>()
            .await
            .unwrap()
    };
    let value = read().await;
    let capacity = &value["imageConcurrency"];
    assert_eq!(capacity["mode"], "auto");
    assert_eq!(capacity["adaptive"], true);
    assert_eq!(capacity["ceilings"]["perMessage"], 30);
    assert_eq!(capacity["ceilings"]["global"], 128);
    assert_eq!(capacity["ceilings"]["decoders"], 10);
    assert!(capacity["cpuCores"].as_f64().unwrap() > 0.0);
    for key in ["perMessage", "perAccount", "global", "pipeline", "decoders"] {
        let effective = capacity["effective"][key].as_u64().unwrap();
        assert!(effective > 0 && effective <= capacity["ceilings"][key].as_u64().unwrap());
    }
    locked_policy(&f, "privacy.imageConcurrencyMode", json!("conservative")).await;
    let value = read().await;
    let capacity = &value["imageConcurrency"];
    assert_eq!(capacity["mode"], "conservative");
    assert!(capacity["effective"]["perMessage"].as_u64().unwrap() <= 8);
    assert!(capacity["effective"]["global"].as_u64().unwrap() <= 32);
    assert!(capacity["effective"]["decoders"].as_u64().unwrap() <= 2);
    locked_policy(&f, "privacy.imageConcurrencyMode", json!("manual")).await;
    let value = read().await;
    let capacity = &value["imageConcurrency"];
    assert_eq!(capacity["adaptive"], false);
    assert_eq!(capacity["effective"], capacity["ceilings"]);
    locked_policy(&f, "privacy.imageConcurrencyMode", json!("auto")).await;
    let value = read().await;
    assert_eq!(value["imageConcurrency"]["mode"], "auto");
    assert_eq!(value["imageConcurrency"]["ceilings"]["global"], 128);
}

#[tokio::test]
async fn an_old_request_waiting_for_a_flight_cannot_restore_pre_shrink_limits() {
    let (proxy, seen) = standin_status_proxy_with_delay(
        vec![(200, "image/png".into(), png(64, 64))],
        std::time::Duration::from_millis(500),
    )
    .await;
    let f = fixture(proxy, "mw-image-stale-capacity").await;
    f.grant().await;
    locked_policy(&f, "privacy.imageConcurrencyMode", json!("manual")).await;
    locked_policy(&f, "privacy.cacheTtlSecs", json!(0)).await;
    locked_policy(&f, "privacy.maxConcurrentGlobal", json!(16)).await;
    let urls: Vec<_> = (0..25)
        .map(|i| format!("http://1.2.3.4/stale-capacity/{i}.png"))
        .collect();
    let refs: Vec<_> = urls.iter().map(String::as_str).collect();
    let (_, signed) = f.sign(&refs, &[]).await;
    let first = format!("{}{}", f.base, signed["images"][0]["src"].as_str().unwrap());
    let start = |url: String| {
        let client = f.anon.clone();
        tokio::spawn(async move {
            let response = client.get(url).send().await.unwrap();
            assert_eq!(response.status(), 200);
            assert!(response.bytes().await.unwrap().starts_with(b"\x89PNG"));
        })
    };
    let mut jobs = vec![start(first.clone())];
    tokio::time::timeout(std::time::Duration::from_secs(2), async {
        while seen.lock().unwrap().connections == 0 {
            tokio::task::yield_now().await;
        }
    })
    .await
    .unwrap();
    jobs.push(start(first));
    tokio::time::timeout(std::time::Duration::from_secs(2), async {
        loop {
            let stats: Value = f
                .admin
                .get(format!("{}/admin/egress/privacy-stats", f.base))
                .send()
                .await
                .unwrap()
                .json()
                .await
                .unwrap();
            if stats["imageConcurrency"]["active"]["queued"]
                .as_u64()
                .unwrap()
                > 0
            {
                break;
            }
            tokio::time::sleep(std::time::Duration::from_millis(10)).await;
        }
    })
    .await
    .unwrap();
    locked_policy(&f, "privacy.maxConcurrentGlobal", json!(4)).await;
    for image in signed["images"].as_array().unwrap().iter().skip(1) {
        jobs.push(start(format!(
            "{}{}",
            f.base,
            image["src"].as_str().unwrap()
        )));
    }
    for job in jobs {
        job.await.unwrap();
    }
    assert!(
        seen.lock().unwrap().peak <= 4,
        "an older flight waiter must not reinstall sixteen network slots"
    );
}

#[tokio::test]
async fn image_cache_lifetimes_and_reader_budgets_follow_hot_configuration() {
    let (proxy, seen) = standin_status_proxy(vec![
        (200, "image/png".into(), png(64, 64)),
        (200, "image/png".into(), png(64, 64)),
        (
            503,
            "text/plain".into(),
            b"temporarily unavailable".to_vec(),
        ),
    ])
    .await;
    let f = fixture(proxy, "mw-rc-cache-controls").await;
    f.grant().await;
    // The fixed lifetime and size keys apply in manual cache mode; the default is adaptive.
    locked_policy(&f, "privacy.imageCacheMode", json!("manual")).await;
    let policy: Value = f
        .user
        .get(format!("{}/api/rc/policy", f.base))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    assert_eq!(policy["imageLoadBudgetMs"], 90000);
    assert_eq!(policy["imageRetryAttempts"], 3);
    assert_eq!(policy["imageRetryDelayMs"], 5000);
    let (_, signed) = f
        .sign(&["http://1.2.3.4/cache-controls/success.png"], &[])
        .await;
    let src = image_src(&signed);
    assert_eq!(f.load(&src).await.status(), 200);
    assert_eq!(f.load(&src).await.status(), 200);
    assert_eq!(
        seen.lock().unwrap().connections,
        1,
        "positive cache avoids a duplicate upstream fetch"
    );
    locked_policy(&f, "privacy.cacheTtlSecs", json!(0)).await;
    assert_eq!(f.load(&src).await.status(), 200);
    assert_eq!(
        seen.lock().unwrap().connections,
        2,
        "zero TTL must stop reuse immediately"
    );
    let (_, signed) = f
        .sign(&["http://1.2.3.4/cache-controls/failure.png"], &[])
        .await;
    let failed = image_src(&signed);
    assert_eq!(f.load(&failed).await.status(), 502);
    assert_eq!(f.load(&failed).await.status(), 502);
    assert_eq!(
        seen.lock().unwrap().connections,
        3,
        "negative cache avoids a repeated failing fetch"
    );
    locked_policy(&f, "privacy.negativeCacheTtlSecs", json!(0)).await;
    let response = f.load(&failed).await;
    assert_eq!(response.status(), 502);
    assert_eq!(response.headers()["cache-control"], "no-store");
    assert_eq!(
        seen.lock().unwrap().connections,
        4,
        "zero TTL removes a remembered failure"
    );
    assert_eq!(f.load(&failed).await.status(), 502);
    assert_eq!(seen.lock().unwrap().connections, 5);
    for (key, value) in [
        ("privacy.fetchQueueTimeoutMs", 5000),
        ("privacy.fetchTotalTimeoutMs", 2000),
        ("privacy.imageDecodeTimeoutMs", 1500),
        ("privacy.imageRetryAttempts", 0),
        ("privacy.imageRetryDelayMs", 500),
    ] {
        locked_policy(&f, key, json!(value)).await;
    }
    let policy: Value = f
        .user
        .get(format!("{}/api/rc/policy", f.base))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    assert_eq!(policy["imageLoadBudgetMs"], 8500);
    assert_eq!(policy["imageRetryAttempts"], 0);
    assert_eq!(policy["imageRetryDelayMs"], 500);
}

#[tokio::test]
async fn concurrent_signed_and_legacy_misses_each_fetch_once() {
    let (proxy, seen) = standin_proxy(vec![("image/png".into(), png(64, 64))]).await;
    let f = fixture(proxy, "mw-rc-single-flight").await;
    f.grant().await;
    let upstream = "http://1.2.3.4/single-flight.png";
    let (status, signed) = f.sign(&[upstream], &[]).await;
    assert_eq!(status, 200);
    for legacy in [false, true] {
        let mut jobs = Vec::new();
        let barrier = Arc::new(tokio::sync::Barrier::new(12));
        for _ in 0..12 {
            let client = if legacy {
                f.user.clone()
            } else {
                f.anon.clone()
            };
            let mut url = reqwest::Url::parse(&f.base).unwrap();
            if legacy {
                url.set_path("/api/image-proxy");
                url.query_pairs_mut()
                    .append_pair("url", upstream)
                    .append_pair("emailId", &f.email_id);
            } else {
                url.set_path(&image_src(&signed));
            }
            let barrier = barrier.clone();
            jobs.push(tokio::spawn(async move {
                barrier.wait().await;
                let response = client.get(url).send().await.unwrap();
                assert_eq!(response.status(), 200);
                assert!(response.bytes().await.unwrap().starts_with(b"\x89PNG"));
            }));
        }
        for job in jobs {
            job.await.unwrap();
        }
        assert_eq!(seen.lock().unwrap().connections, if legacy { 2 } else { 1 });
    }
}

#[tokio::test]
async fn distinct_image_burst_waits_for_bounded_capacity_instead_of_shedding_valid_images() {
    let (proxy, seen) = standin_status_proxy_with_delay(
        vec![(200, "image/png".into(), png(64, 64))],
        std::time::Duration::from_millis(200),
    )
    .await;
    let f = fixture(proxy, "mw-rc-distinct-burst").await;
    let anonymous = f
        .anon
        .get(format!("{}/api/image-proxy", f.base))
        .query(&[
            ("url", "http://1.2.3.4/anonymous.png"),
            ("emailId", f.email_id.as_str()),
        ])
        .send()
        .await
        .unwrap();
    assert_eq!(
        anonymous.status(),
        401,
        "legacy proxy requires an authenticated mailbox session"
    );
    let anonymous = f
        .anon
        .post(format!("{}/api/rc/sign", f.base))
        .json(&json!({"emailId":f.email_id,"urls":["http://1.2.3.4/anonymous.png"]}))
        .send()
        .await
        .unwrap();
    assert_eq!(
        anonymous.status(),
        401,
        "anonymous clients cannot mint image capabilities"
    );
    assert_eq!(seen.lock().unwrap().connections, 0);
    for key in [
        "privacy.maxConcurrentPerMessage",
        "privacy.maxConcurrentPerAccount",
    ] {
        let url = format!("{}/admin/settings/{key}", f.base);
        let current = f.admin.get(&url).send().await.unwrap();
        let tag = current.headers()["etag"].to_str().unwrap().to_owned();
        let response = f
            .admin
            .put(url)
            .header("if-match", tag)
            .json(&json!({"globalValue":32}))
            .send()
            .await
            .unwrap();
        assert_eq!(
            response.status(),
            200,
            "{key}: {}",
            response.text().await.unwrap()
        );
    }
    locked_policy(&f, "privacy.imageConcurrencyMode", json!("manual")).await;
    locked_policy(&f, "privacy.maxConcurrentGlobal", json!(24)).await;
    locked_policy(&f, "privacy.maxConcurrentPipeline", json!(48)).await;
    f.grant().await;
    let urls: Vec<_> = (0..96)
        .map(|index| format!("http://1.2.3.4/burst-{index}.png"))
        .collect();
    let refs: Vec<_> = urls.iter().map(String::as_str).collect();
    let (status, signed) = f.sign(&refs, &[]).await;
    assert_eq!(status, 200);
    let barrier = Arc::new(tokio::sync::Barrier::new(urls.len()));
    let mut jobs = Vec::new();
    for image in signed["images"].as_array().unwrap() {
        let url = format!("{}{}", f.base, image["src"].as_str().unwrap());
        let (client, barrier) = (f.anon.clone(), barrier.clone());
        jobs.push(tokio::spawn(async move {
            barrier.wait().await;
            let response = client.get(url).send().await.unwrap();
            let status = response.status().as_u16();
            let bytes = response.bytes().await.unwrap();
            (status, bytes.starts_with(b"\x89PNG"))
        }));
    }
    let mut results = Vec::new();
    for job in jobs {
        results.push(job.await.unwrap());
    }
    let failed: Vec<_> = results
        .iter()
        .filter(|(status, png)| *status != 200 || !png)
        .collect();
    assert!(
        failed.is_empty(),
        "valid distinct image burst failed: {failed:?}; completed={}",
        results.len()
    );
    let observed = seen.lock().unwrap();
    assert_eq!(observed.connections, urls.len());
    assert!(
        observed.peak > 16,
        "configured parallel fetches must exceed the old fixed sixteen-request ceiling: {}",
        observed.peak
    );
    assert!(
        observed.peak <= 24,
        "configured replica ceiling must hold: {}",
        observed.peak
    );
    let distinct: std::collections::HashSet<_> = observed
        .requests
        .iter()
        .map(|request| request.lines().next().unwrap())
        .collect();
    assert_eq!(
        distinct.len(),
        urls.len(),
        "every URL was independently fetched through the configured proxy"
    );
    drop(observed);
    let stats: Value = f
        .user
        .get(format!("{}/api/rc/stats?emailId={}", f.base, f.email_id))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    assert_eq!(stats["imagesLoaded"], 96);
    assert_eq!(stats["imagesFailed"], 0);

    // Hot reductions reuse the same limiter, on the legacy path too.
    locked_policy(&f, "privacy.maxConcurrentGlobal", json!(4)).await;
    tokio::time::timeout(std::time::Duration::from_secs(2), async {
        while seen.lock().unwrap().active != 0 {
            tokio::task::yield_now().await;
        }
    })
    .await
    .unwrap();
    seen.lock().unwrap().peak = 0;
    let mut jobs = Vec::new();
    for index in 0..12 {
        let (client, base, email) = (f.user.clone(), f.base.clone(), f.email_id.clone());
        jobs.push(tokio::spawn(async move {
            let response = client
                .get(format!("{base}/api/image-proxy"))
                .query(&[
                    ("url", format!("http://1.2.3.4/hot-capacity/{index}.png")),
                    ("emailId", email),
                ])
                .send()
                .await
                .unwrap();
            assert_eq!(response.status(), 200);
            assert!(response.bytes().await.unwrap().starts_with(b"\x89PNG"));
        }));
    }
    for job in jobs {
        job.await.unwrap();
    }
    assert!(seen.lock().unwrap().peak <= 4);
}

/// A 4×4 opaque PNG (larger than a tracking pixel).
fn png(w: u32, h: u32) -> Vec<u8> {
    // Built here (not a fixture) so the size is visible in the test.
    fn crc(data: &[u8]) -> u32 {
        let mut c = 0xffff_ffffu32;
        for &b in data {
            c ^= u32::from(b);
            for _ in 0..8 {
                c = if c & 1 != 0 {
                    0xedb8_8320 ^ (c >> 1)
                } else {
                    c >> 1
                };
            }
        }
        !c
    }
    fn adler(data: &[u8]) -> u32 {
        let (mut a, mut b) = (1u32, 0u32);
        for &d in data {
            a = (a + u32::from(d)) % 65521;
            b = (b + a) % 65521;
        }
        (b << 16) | a
    }
    fn chunk(out: &mut Vec<u8>, kind: &[u8], data: &[u8]) {
        out.extend_from_slice(&(data.len() as u32).to_be_bytes());
        let mut c = kind.to_vec();
        c.extend_from_slice(data);
        out.extend_from_slice(&c);
        out.extend_from_slice(&crc(&c).to_be_bytes());
    }
    let mut raw = Vec::new();
    for _ in 0..h {
        raw.extend_from_slice(&[0u8]);
        for _ in 0..w {
            raw.extend_from_slice(&[0x20, 0x40, 0x80]);
        }
    }
    // zlib "stored" block: no compression needed for a tiny image.
    let mut z = vec![0x78, 0x01, 0x01];
    z.extend_from_slice(&(raw.len() as u16).to_le_bytes());
    z.extend_from_slice(&(!(raw.len() as u16)).to_le_bytes());
    z.extend_from_slice(&raw);
    z.extend_from_slice(&adler(&raw).to_be_bytes());
    let mut ihdr = Vec::new();
    ihdr.extend_from_slice(&w.to_be_bytes());
    ihdr.extend_from_slice(&h.to_be_bytes());
    ihdr.extend_from_slice(&[8, 2, 0, 0, 0]);
    let mut out = b"\x89PNG\r\n\x1a\n".to_vec();
    chunk(&mut out, b"IHDR", &ihdr);
    chunk(&mut out, b"IDAT", &z);
    chunk(&mut out, b"IEND", &[]);
    out
}

fn png_size(png: &[u8]) -> (u32, u32) {
    (
        u32::from_be_bytes(png[16..20].try_into().unwrap()),
        u32::from_be_bytes(png[20..24].try_into().unwrap()),
    )
}

// ── the stand-in proxy ───────────────────────────────────────────────────────────

#[derive(Default)]
struct Seen {
    connections: usize,
    requests: Vec<String>,
    active: usize,
    peak: usize,
}

/// Read an HTTP head through `\r\n\r\n`.
async fn read_head(s: &mut TcpStream) -> String {
    let mut head = Vec::new();
    let mut byte = [0u8; 1];
    while s.read(&mut byte).await.unwrap_or(0) == 1 {
        head.push(byte[0]);
        if head.ends_with(b"\r\n\r\n") || head.len() > 16384 {
            break;
        }
    }
    String::from_utf8_lossy(&head).into_owned()
}

/// A `CONNECT` proxy that answers each tunnelled request with the next reply from
/// `replies` (the last one repeats), recording the request heads.
async fn standin_proxy(replies: Vec<(String, Vec<u8>)>) -> (SocketAddr, Arc<Mutex<Seen>>) {
    standin_status_proxy(
        replies
            .into_iter()
            .map(|(kind, body)| (200, kind, body))
            .collect(),
    )
    .await
}

async fn standin_status_proxy(
    replies: Vec<(u16, String, Vec<u8>)>,
) -> (SocketAddr, Arc<Mutex<Seen>>) {
    standin_status_proxy_with_delay(replies, std::time::Duration::ZERO).await
}

async fn standin_status_proxy_with_delay(
    replies: Vec<(u16, String, Vec<u8>)>,
    delay: std::time::Duration,
) -> (SocketAddr, Arc<Mutex<Seen>>) {
    standin_status_proxy_with_agent_gate(replies, delay, false).await
}

async fn standin_status_proxy_with_agent_gate(
    replies: Vec<(u16, String, Vec<u8>)>,
    delay: std::time::Duration,
    require_browser_compatible_agent: bool,
) -> (SocketAddr, Arc<Mutex<Seen>>) {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    let seen = Arc::new(Mutex::new(Seen::default()));
    let sink = Arc::clone(&seen);
    let replies = Arc::new(replies);
    tokio::spawn(async move {
        while let Ok((mut c, _)) = listener.accept().await {
            let sink = Arc::clone(&sink);
            let replies = Arc::clone(&replies);
            tokio::spawn(async move {
                let n = {
                    let mut s = sink.lock().unwrap();
                    s.connections += 1;
                    s.connections - 1
                };
                let _ = read_head(&mut c).await;
                let _ = c
                    .write_all(b"HTTP/1.1 200 Connection established\r\n\r\n")
                    .await;
                let req = read_head(&mut c).await;
                let compatible_agent = req.lines().any(|line| {
                    line.eq_ignore_ascii_case(
                        "user-agent: Mozilla/5.0 (compatible; Mailwoman-Image-Proxy)",
                    )
                });
                {
                    let mut seen = sink.lock().unwrap();
                    seen.requests.push(req);
                    seen.active += 1;
                    seen.peak = seen.peak.max(seen.active);
                }
                tokio::time::sleep(delay).await;
                let denied = (
                    410,
                    "text/plain".to_string(),
                    b"Access Denied!\r\n".to_vec(),
                );
                let (status, ctype, body) = if require_browser_compatible_agent && !compatible_agent
                {
                    &denied
                } else {
                    &replies[n.min(replies.len() - 1)]
                };
                let head = format!(
                    "HTTP/1.1 {status} Fixture\r\nContent-Type: {ctype}\r\nContent-Length: {}\r\n\r\n",
                    body.len()
                );
                let _ = c.write_all(head.as_bytes()).await;
                let _ = c.write_all(body).await;
                let _ = c.flush().await;
                sink.lock().unwrap().active -= 1;
            });
        }
    });
    (addr, seen)
}

// ── the server ───────────────────────────────────────────────────────────────────

static MOCK_UPSTREAMS: std::sync::Mutex<Vec<String>> = std::sync::Mutex::new(Vec::new());

async fn spawn_mock() -> String {
    spawn_mock_with_image_parts(false).await
}

async fn spawn_mock_with_image_parts(image_parts: bool) -> String {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    tokio::spawn(async move {
        let app = mw_mock_jmap::router().layer(axum::middleware::map_response(move |response: axum::response::Response| async move {
            let (parts, body) = response.into_parts();
            let bytes = axum::body::to_bytes(body, 1024 * 1024).await.unwrap();
            if image_parts && bytes.starts_with(b"BLOB:") {
                let mut response = axum::response::Response::from_parts(parts, axum::body::Body::from(png(4, 4)));
                response.headers_mut().remove("content-length");
                response.headers_mut().insert("content-type", "image/png".parse().unwrap());
                return response;
            }
            let mut value = serde_json::from_slice::<Value>(&bytes).ok();
            if let Some(calls) = value.as_mut().and_then(|v| v["methodResponses"].as_array_mut()) {
                for call in calls {
                    if call[0] == "Email/get" && let Some(list) = call[1]["list"].as_array_mut() {
                        for email in list {
                            if email["id"] == "e1" {
                                email["bodyStructure"] = json!({"type":"multipart/related","subParts":[{"blobId":"Mblob1","cid":"logo@example.test","type":"image/png"}]});
                                email["htmlBody"] = json!([{"partId":"html","type":"text/html"}]);
                                email["bodyValues"] = json!({"html":{"value":"<a href='http://1.2.3.4/dl/report.html'>Report</a><img src='http://1.2.3.4/download-image.png'><img src='http://1.2.3.4/download-svg.svg'><img src='http://1.2.3.4/declared-pixel.png' width='1' height='1'>","isTruncated":false,"isEncodingProblem":false}});
                            }
                        }
                    }
                }
            }
            // Each isolated test server owns a distinct account; exhausting its
            // rate budget must not consume another fixture's process-wide bucket.
            let body = value.map_or_else(|| bytes.to_vec(), |v| {
                serde_json::to_string(&v).unwrap().replace(
                    &format!("\"{}\"", mw_mock_jmap::ACCOUNT_ID),
                    &format!("\"rc-account-{}\"", addr.port()),
                ).into_bytes()
            });
            let mut result = axum::response::Response::from_parts(parts, axum::body::Body::from(body));
            result.headers_mut().remove("content-length");
            result
        }));
        let _ = axum::serve(listener, app).await;
    });
    let origin = format!("http://{addr}");
    MOCK_UPSTREAMS.lock().unwrap().push(origin.clone());
    origin
}

async fn spawn_server(db_path: &str, mock: &str) -> SocketAddr {
    let web = PathBuf::from(db_path)
        .parent()
        .unwrap()
        .join(format!("web-{}", test_db::unique_tag()));
    std::fs::create_dir_all(&web).unwrap();
    std::fs::write(web.join("index.html"), INDEX_HTML).unwrap();
    let config = AppConfig {
        db_path: db_path.to_string(),
        server_key_hex: Some(KEY_HEX.to_string()),
        web_dir: Some(web),
        cookie_secure: false,
        mode: mw_server::ServerMode::Proxy,
        hardening: mw_server::HardeningConfig::default(),
        security: mw_server::SecurityConfig {
            jmap_upstreams: Some(MOCK_UPSTREAMS.lock().unwrap().clone()),
            upstream_default: Some(mock.to_string()),
            ..mw_server::SecurityConfig::default()
        },
    };
    let (app, _) = build_app_full(
        config,
        V6Config {
            admin_enabled: true,
            admin_username: Some("root".into()),
            admin_password: Some("rc-fixture-password".into()),
            redis_url: None,
        },
    )
    .await
    .expect("build_app");
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    tokio::spawn(async move {
        let _ = axum::serve(
            listener,
            app.into_make_service_with_connect_info::<SocketAddr>(),
        )
        .await;
    });
    addr
}

async fn store_at(db_path: &str) -> Store {
    Store::open(db_path, ServerKey::from_hex(KEY_HEX).unwrap())
        .await
        .expect("open store")
}

// ── the token, from outside ──────────────────────────────────────────────────────

fn hmac(key: &[u8], parts: &[&[u8]]) -> Vec<u8> {
    let mut m = Hmac::<Sha256>::new_from_slice(key).unwrap();
    for p in parts {
        m.update(p);
    }
    m.finalize().into_bytes().to_vec()
}

/// The signing key, derived as the server derives it.
fn signing_key() -> Vec<u8> {
    let server_key: Vec<u8> = (0..32)
        .map(|i| u8::from_str_radix(&KEY_HEX[i * 2..i * 2 + 2], 16).unwrap())
        .collect();
    hmac(
        &server_key,
        &[b"mailwoman-subkey:", b"mw-remote-content-v1"],
    )
}

fn mint(claims: &Value) -> String {
    let payload = URL_SAFE_NO_PAD.encode(serde_json::to_vec(claims).unwrap());
    let tag = hmac(&signing_key(), &[b"mw-rc-token\0", payload.as_bytes()]);
    format!("{payload}.{}", URL_SAFE_NO_PAD.encode(tag))
}

/// The claims behind a link token: a short handle is read from the shared
/// store, a legacy token decodes from its own payload.
async fn claims_of(db: &str, token: &str) -> Value {
    if let Some((payload, _)) = token.split_once('.') {
        return serde_json::from_slice(&URL_SAFE_NO_PAD.decode(payload).unwrap()).unwrap();
    }
    let json = store_at(db)
        .await
        .rc_handle_get(token, 0)
        .await
        .unwrap()
        .expect("handle is stored");
    serde_json::from_str(&json).unwrap()
}

// ── fixture ──────────────────────────────────────────────────────────────────────

struct Fixture {
    base: String,
    user: reqwest::Client,
    admin: reqwest::Client,
    anon: reqwest::Client,
    email_id: String,
    account: String,
    db: String,
}

async fn fixture(proxy: SocketAddr, tag: &str) -> Fixture {
    fixture_with_protection(proxy, tag, &[], false).await
}

async fn fixture_with_protection(
    proxy: SocketAddr,
    tag: &str,
    values: &[(&str, Value)],
    image_parts: bool,
) -> Fixture {
    let db = test_db::unique_dir(tag)
        .join("mw.db")
        .to_string_lossy()
        .into_owned();
    if !values.is_empty() {
        let store = store_at(&db).await;
        for (key, value) in values {
            store
                .put_setting_policy(&mw_store::settings_policy::SettingPolicyRow {
                    key: (*key).into(),
                    global_value: Some(value.to_string()),
                    allow_user_override: false,
                    global_locked: true,
                    updated_by: "fixture".into(),
                    updated_at: 1,
                })
                .await
                .unwrap();
        }
    }
    let mock = if image_parts {
        spawn_mock_with_image_parts(true).await
    } else {
        spawn_mock().await
    };
    let addr = spawn_server(&db, &mock).await;
    let base = format!("http://{addr}");
    let user = reqwest::Client::builder()
        .cookie_store(true)
        .no_proxy()
        .build()
        .unwrap();
    let login: Value = user
        .post(format!("{base}/api/login"))
        .json(&json!({ "username": mw_mock_jmap::USER, "password": mw_mock_jmap::PASS }))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    assert_eq!(login["ok"], json!(true), "login failed: {login}");
    let account = login["accountId"].as_str().unwrap().to_string();
    let email_id = {
        let opts = sqlx::sqlite::SqliteConnectOptions::new().filename(&db);
        let raw = sqlx::SqlitePool::connect_with(opts).await.unwrap();
        sqlx::query(
            "INSERT INTO accounts (id, kind, host, port, tls, username, sealed_creds, sync_policy_json)
             VALUES (?1, 'imap', 'mail.example', 993, 1, ?1, X'00', '{}')",
        )
        .bind(&account)
        .execute(&raw)
        .await
        .unwrap();
        raw.close().await;
        let store = store_at(&db).await;
        let mailbox = store
            .upsert_mailbox(&mw_store::MailboxUpsert {
                account_id: &account,
                name: "INBOX",
                role: Some("inbox"),
                uidvalidity: 1,
                uidnext: 2,
                highestmodseq: 1,
                total: 1,
                unread: 0,
                parent_id: None,
            })
            .await
            .unwrap();
        store
            .upsert_message(&mw_store::MessageUpsert {
                account_id: &account,
                mailbox_id: &mailbox,
                uid: 1,
                uidvalidity: 1,
                message_id: Some("<rc@test>"),
                thread_id: None,
                internaldate: Some("2026-10-25T12:00:00Z"),
                size: 42,
                flags_json: "[]",
                envelope: None,
                blob_ref: None,
            })
            .await
            .unwrap()
    };
    // Console and mailbox sign-ins share the trusted-client cooldown.
    tokio::time::sleep(std::time::Duration::from_secs(3)).await;
    // The route that carries remote content: the stand-in proxy.
    let admin = reqwest::Client::builder()
        .no_proxy()
        .cookie_store(true)
        .build()
        .unwrap();
    let login = admin
        .post(format!("{base}/admin/login"))
        .json(&json!({"username":"root","password":"rc-fixture-password"}))
        .send()
        .await
        .unwrap();
    assert_eq!(login.status(), 200, "{}", login.text().await.unwrap());
    let proof = admin
        .post(format!("{base}/admin/step-up"))
        .json(&json!({"password":"rc-fixture-password"}))
        .send()
        .await
        .unwrap();
    assert_eq!(proof.status(), 200, "{}", proof.text().await.unwrap());
    for (path, body) in [
        (
            "/admin/egress/proxies".to_string(),
            Some(json!({
                "id": "rc",
                "scheme": "http",
                "host": proxy.ip().to_string(),
                "port": proxy.port(),
                "allowPlaintext": true,
            })),
        ),
        ("/admin/egress/proxies/rc/activate".to_string(), None),
    ] {
        let mut req = admin.post(format!("{base}{path}"));
        if let Some(b) = body {
            req = req.json(&b);
        }
        let r = req.send().await.unwrap();
        assert_eq!(r.status(), 200, "{path}: {}", r.text().await.unwrap());
    }
    Fixture {
        base,
        user,
        admin,
        anon: reqwest::Client::builder().no_proxy().build().unwrap(),
        email_id,
        account,
        db,
    }
}

impl Fixture {
    async fn sign(&self, urls: &[&str], parts: &[&str]) -> (u16, Value) {
        let r = self
            .user
            .post(format!("{}/api/rc/sign", self.base))
            .json(&json!({ "emailId": self.email_id, "urls": urls, "parts": parts }))
            .send()
            .await
            .unwrap();
        let status = r.status().as_u16();
        (status, r.json().await.unwrap_or(Value::Null))
    }

    async fn grant(&self) {
        store_at(&self.db)
            .await
            .grant_remote_image(&self.account, "all", "")
            .await
            .unwrap();
    }

    /// GET a signed path the way the sandboxed frame does: no cookie.
    async fn load(&self, src: &str) -> reqwest::Response {
        self.anon
            .get(format!("{}{src}", self.base))
            .send()
            .await
            .unwrap()
    }
}

fn image_src(signed: &Value) -> String {
    signed["images"][0]["src"].as_str().unwrap().to_string()
}

fn token_of(src: &str) -> String {
    src.rsplit('/').next().unwrap().to_string()
}

// ── the tests ────────────────────────────────────────────────────────────────────

#[tokio::test]
async fn image_burst_uses_a_fixed_browser_compatible_agent_for_signed_and_legacy_fetches() {
    let (proxy, seen) = standin_status_proxy_with_agent_gate(
        vec![(200, "image/png".into(), png(4, 4))],
        std::time::Duration::ZERO,
        true,
    )
    .await;
    // Reproduce the upstream's policy: a bare product UA is denied, independent
    // of load. This direct fixture control never contacts a public origin.
    let mut control = TcpStream::connect(proxy).await.unwrap();
    control
        .write_all(b"CONNECT 1.2.3.4:80 HTTP/1.1\r\nHost: 1.2.3.4:80\r\n\r\n")
        .await
        .unwrap();
    assert!(read_head(&mut control).await.starts_with("HTTP/1.1 200"));
    control.write_all(b"GET /old-agent.png HTTP/1.1\r\nHost: 1.2.3.4\r\nUser-Agent: Mailwoman-Image-Proxy\r\n\r\n").await.unwrap();
    assert!(read_head(&mut control).await.starts_with("HTTP/1.1 410"));
    drop(control);
    let f = fixture(proxy, "mw-rc-cdn-agent-burst").await;
    f.grant().await;
    let urls: Vec<_> = (0..4)
        .map(|i| format!("http://1.2.3.4/agent-signed-{i}.png"))
        .collect();
    let refs: Vec<_> = urls.iter().map(String::as_str).collect();
    let (status, signed) = f.sign(&refs, &[]).await;
    assert_eq!(status, 200, "{signed}");
    let mut jobs = Vec::new();
    for index in 0..8 {
        let (client, url) = if index < 4 {
            (
                f.anon.clone(),
                format!(
                    "{}{}",
                    f.base,
                    signed["images"][index]["src"].as_str().unwrap()
                ),
            )
        } else {
            let mut url = reqwest::Url::parse(&format!("{}/api/image-proxy", f.base)).unwrap();
            url.query_pairs_mut()
                .append_pair("url", &format!("http://1.2.3.4/agent-legacy-{index}.png"))
                .append_pair("emailId", &f.email_id);
            (f.user.clone(), url.to_string())
        };
        jobs.push(tokio::spawn(async move {
            let response = client
                .get(url)
                .header("user-agent", "Sensitive Reader Device/123")
                .header("referer", "https://private.example/reader")
                .send()
                .await
                .unwrap();
            assert_eq!(response.status(), 200, "{}", response.text().await.unwrap());
            assert!(response.bytes().await.unwrap().starts_with(b"\x89PNG"));
        }));
    }
    for job in jobs {
        job.await.unwrap();
    }
    let seen = seen.lock().unwrap();
    assert_eq!(seen.connections, 9);
    for request in seen.requests.iter().skip(1) {
        let head = request.to_ascii_lowercase();
        assert!(
            head.contains("user-agent: mozilla/5.0 (compatible; mailwoman-image-proxy)"),
            "{head}"
        );
        for private in ["sensitive reader", "private.example", "cookie:"] {
            assert!(!head.contains(private), "{head}");
        }
    }
}

#[tokio::test]
async fn signed_display_batch_pays_one_aggregate_request_for_remote_and_inline_images() {
    let (proxy, _) = standin_proxy(vec![("image/png".into(), png(4, 4))]).await;
    let mut f = fixture_with_protection(
        proxy,
        "mw-rc-batch-budget",
        &[
            ("security.protection.rateMode", json!("enforce")),
            ("security.protection.clientRequests", json!(3)),
            ("security.protection.accountRequests", json!(1)),
            ("security.protection.adminRequests", json!(0)),
            ("security.protection.rateWindowSecs", json!(3600)),
            ("security.protection.adaptiveMode", json!("enforce")),
        ],
        true,
    )
    .await;
    f.email_id = "e1".into();
    f.grant().await;
    let urls: Vec<_> = (0..8)
        .map(|i| format!("http://1.2.3.4/aggregate-batch-{i}.png"))
        .collect();
    let refs: Vec<_> = urls.iter().map(String::as_str).collect();
    let (status, signed) = f.sign(&refs, &["Mblob1"]).await;
    assert_eq!(status, 200, "{signed}");
    assert_eq!(signed["images"].as_array().unwrap().len(), 8);
    assert_eq!(signed["parts"].as_array().unwrap().len(), 1);
    let store = store_at(&f.db).await;
    let before_rows = store
        .request_protection_rows(chrono::Utc::now().timestamp(), None, 100)
        .await
        .unwrap();
    let before_clients: u32 = before_rows
        .iter()
        .filter(|row| row.subject_kind == "client")
        .map(|row| row.snapshot.rate_count)
        .sum();
    let before = before_rows
        .into_iter()
        .find(|row| row.subject_kind == "account")
        .unwrap();
    assert_eq!(before.snapshot.rate_count, 1);
    for resource in signed["images"]
        .as_array()
        .unwrap()
        .iter()
        .chain(signed["parts"].as_array().unwrap())
    {
        let response = f.load(resource["src"].as_str().unwrap()).await;
        assert_eq!(response.status(), 200, "{}", response.text().await.unwrap());
        assert!(response.bytes().await.unwrap().starts_with(b"\x89PNG"));
    }
    let after_rows = store
        .request_protection_rows(chrono::Utc::now().timestamp(), None, 100)
        .await
        .unwrap();
    let after_clients: u32 = after_rows
        .iter()
        .filter(|row| row.subject_kind == "client")
        .map(|row| row.snapshot.rate_count)
        .sum();
    assert!(
        before_clients > 0,
        "fixture establishes an actual aggregate source budget"
    );
    assert_eq!(
        after_clients, before_clients,
        "cookie-less display fanout must not spend the anonymous source budget"
    );
    let after = after_rows
        .into_iter()
        .find(|row| row.subject_kind == "account")
        .unwrap();
    assert_eq!(
        after.snapshot.rate_count, 1,
        "all display URLs share the sign admission"
    );
    assert_eq!(
        after.snapshot.short, before.snapshot.short,
        "image successes must not inflate adaptive normal samples"
    );
    let response = f
        .user
        .get(format!("{}/api/me", f.base))
        .send()
        .await
        .unwrap();
    assert_eq!(
        response.status(),
        429,
        "the next ordinary account request still spends its quota"
    );
    assert_eq!(
        response.json::<Value>().await.unwrap()["code"],
        "requestProtectionLimited"
    );
}

fn display_client(octet: u8) -> reqwest::Client {
    reqwest::Client::builder()
        .no_proxy()
        .local_address(std::net::IpAddr::V4(std::net::Ipv4Addr::new(
            127, 0, 0, octet,
        )))
        .build()
        .unwrap()
}

#[tokio::test]
async fn invalid_display_capabilities_keep_the_ordinary_anonymous_request_budget() {
    let (proxy, seen) = standin_proxy(vec![("image/png".into(), png(4, 4))]).await;
    let f = fixture_with_protection(
        proxy,
        "mw-rc-invalid-budget",
        &[
            ("security.protection.rateMode", json!("enforce")),
            ("security.protection.clientRequests", json!(100)),
            ("security.protection.accountRequests", json!(100)),
            ("security.protection.adminRequests", json!(0)),
            ("security.protection.rateWindowSecs", json!(3600)),
            ("security.protection.adaptiveMode", json!("off")),
        ],
        false,
    )
    .await;
    // Login, step-up and fixture administration complete before installing the
    // tiny quota. Fresh client addresses below start with an empty budget.
    let setting_url = format!(
        "{}/admin/settings/security.protection.clientRequests",
        f.base
    );
    let current = f.admin.get(&setting_url).send().await.unwrap();
    let tag = current.headers()["etag"].to_str().unwrap().to_owned();
    let changed = f
        .admin
        .put(setting_url)
        .header("if-match", tag)
        .json(&json!({"globalValue":2}))
        .send()
        .await
        .unwrap();
    assert_eq!(changed.status(), 200, "{}", changed.text().await.unwrap());
    f.grant().await;
    let (status, signed) = f.sign(&["http://1.2.3.4/invalid-budget.png"], &[]).await;
    assert_eq!(status, 200, "{signed}");
    let token = token_of(&image_src(&signed));
    let mut expired = claims_of(&f.db, &token).await;
    expired["x"] = json!(chrono::Utc::now().timestamp() - 1);
    let paths = [
        "/api/rc/i/abc.def".to_string(),
        format!("/api/rc/i/{}", mint(&expired)),
        format!("/api/rc/p/{token}"),
        image_src(&signed),
    ];
    for (index, path) in paths.iter().enumerate() {
        if index == 3 {
            store_at(&f.db)
                .await
                .delete_sessions_for_account(&f.account)
                .await
                .unwrap();
        }
        let client = display_client(70 + index as u8);
        for expected in [403, 403, 429] {
            let response = client
                .get(format!("{}{path}", f.base))
                .send()
                .await
                .unwrap();
            assert_eq!(response.status(), expected, "invalid case {index}: {path}");
        }
    }
    assert_eq!(
        seen.lock().unwrap().connections,
        0,
        "no invalid capability starts image egress"
    );
}

#[tokio::test]
async fn prepaid_display_keeps_active_source_probe_and_account_adaptive_actions() {
    let (proxy, seen) = standin_proxy(vec![("image/png".into(), png(4, 4))]).await;
    let f = fixture_with_protection(
        proxy,
        "mw-rc-active-actions",
        &[
            ("security.adminProbe.enabled", json!(false)),
            ("security.protection.rateMode", json!("off")),
            ("security.protection.adaptiveMode", json!("enforce")),
            ("security.protection.shortMinimumSamples", json!(1)),
            ("security.protection.shortAbsoluteAnomalies", json!(1)),
            ("security.protection.shortRatioPercent", json!(0)),
            ("security.protection.longAbsoluteAnomalies", json!(0)),
            ("security.protection.longRatioPercent", json!(0)),
            ("security.protection.signals", json!(["malformed"])),
        ],
        false,
    )
    .await;
    f.grant().await;
    let (status, signed) = f.sign(&["http://1.2.3.4/active-actions.png"], &[]).await;
    assert_eq!(status, 200, "{signed}");
    let src = image_src(&signed);
    assert_eq!(f.load(&src).await.status(), 200);
    let scanner = display_client(76);
    assert_eq!(
        scanner
            .get(format!("{}/cmd_sco", f.base))
            .send()
            .await
            .unwrap()
            .status(),
        429
    );
    assert_eq!(
        scanner
            .get(format!("{}{src}", f.base))
            .send()
            .await
            .unwrap()
            .status(),
        429,
        "valid capabilities do not bypass source blocks"
    );
    let malformed = f
        .user
        .post(format!("{}/jmap/api", f.base))
        .header("content-type", "application/json")
        .body("{not-json")
        .send()
        .await
        .unwrap();
    assert_eq!(malformed.status(), 400);
    let blocked = f.load(&src).await;
    assert_eq!(
        blocked.status(),
        429,
        "valid capabilities do not bypass existing account adaptive actions"
    );
    assert_eq!(
        blocked.json::<Value>().await.unwrap()["code"],
        "requestProtectionLimited"
    );
    assert_eq!(
        seen.lock().unwrap().connections,
        1,
        "all blocked loads stop before even cached image serving"
    );
}

#[tokio::test]
async fn a_signed_image_loads_without_a_cookie_and_forwards_nothing_of_the_reader() {
    let (proxy, seen) = standin_proxy(vec![("image/png".into(), png(4, 4))]).await;
    let f = fixture(proxy, "mw-t32-rc-load").await;

    // Without a grant nothing is signed.
    let (status, _) = f.sign(&["http://1.2.3.4/load.png"], &[]).await;
    assert_eq!(status, 403);
    f.grant().await;
    let (status, signed) = f.sign(&["http://1.2.3.4/load.png"], &[]).await;
    assert_eq!(status, 200, "{signed}");
    let src = image_src(&signed);
    assert!(src.starts_with("/api/rc/i/"), "{src}");

    let r = f
        .anon
        .get(format!("{}{src}", f.base))
        .header("cookie", "mw_session=not-forwarded-xyz")
        .header("referer", "https://reader.example/secret-path")
        .header("user-agent", "Mozilla/5.0 (Reader Browser)")
        .send()
        .await
        .unwrap();
    assert_eq!(r.status(), 200);
    assert_eq!(r.headers()["content-type"], "image/png");
    assert_eq!(r.headers()["cross-origin-resource-policy"], "cross-origin");
    assert_eq!(r.headers()["cache-control"], "private, max-age=600");
    assert!(r.headers().get("x-mw-remote-content").is_none());
    let body = r.bytes().await.unwrap();
    assert_eq!(png_size(&body), (4, 4), "re-encoded, same size");

    let seen = seen.lock().unwrap();
    assert_eq!(seen.requests.len(), 1);
    let head = seen.requests[0].to_ascii_lowercase();
    assert!(head.starts_with("get /load.png http/1.1"), "{head}");
    for leaked in [
        "cookie:",
        "referer:",
        "not-forwarded",
        "secret-path",
        "reader browser",
    ] {
        assert!(
            !head.contains(leaked),
            "{leaked} reached the origin: {head}"
        );
    }
    assert!(
        head.contains("user-agent: mozilla/5.0 (compatible; mailwoman-image-proxy)"),
        "{head}"
    );
}

#[tokio::test]
async fn signing_returns_short_handles_that_resolve_for_the_owner_only() {
    let (proxy, _seen) = standin_proxy(vec![("image/png".into(), png(4, 4))]).await;
    let f = fixture(proxy, "mw-t32-rc-handle").await;
    f.grant().await;
    let long = format!("http://1.2.3.4/{}.png", "a".repeat(3000));
    let (status, signed) = f.sign(&[&long], &[]).await;
    assert_eq!(status, 200, "{signed}");
    let src = image_src(&signed);
    let token = token_of(&src);
    assert_eq!(token.len(), 22, "short opaque handle: {src}");
    assert!(!token.contains('.'), "{token}");
    assert!(src.len() < 64, "{src}");
    // The stored claims carry the full URL and the documented lifetimes.
    let claims = claims_of(&f.db, &token).await;
    assert_eq!(claims["u"], json!(long));
    let ttl = claims["x"].as_i64().unwrap() - chrono::Utc::now().timestamp();
    assert!(ttl <= 3600 && ttl > 3590, "{ttl}");
    assert_eq!(f.load(&src).await.status(), 200);

    // A handle holding another account's or session's claims is refused, as is
    // an expired one and one used as another kind.
    let store = store_at(&f.db).await;
    let now = chrono::Utc::now().timestamp();
    let mut rows = Vec::new();
    for (name, edit) in [
        ("other account", ("a", json!("someone-else"))),
        ("other session", ("s", json!("00".repeat(16)))),
        ("expired", ("x", json!(now - 5))),
    ] {
        let mut c = claims.clone();
        c[edit.0] = edit.1;
        let handle = URL_SAFE_NO_PAD.encode(Sha256::digest(name.as_bytes()))[..22].to_string();
        rows.push((handle.clone(), now + 60, c.to_string()));
        let r = f.load(&format!("/api/rc/i/{handle}")).await;
        assert_eq!(r.status(), 403, "{name} before storing");
    }
    store.rc_handles_put(&rows, now, 1000).await.unwrap();
    for (handle, _, _) in &rows {
        let r = f.load(&format!("/api/rc/i/{handle}")).await;
        assert_eq!(r.status(), 403, "{handle}");
    }
    assert_eq!(f.load(&format!("/api/rc/p/{token}")).await.status(), 403);
    // A legacy self-contained token minted before the change still loads.
    assert_eq!(
        f.load(&format!("/api/rc/i/{}", mint(&claims))).await.status(),
        200
    );
}

#[tokio::test]
async fn a_tampered_retargeted_or_expired_token_is_refused_before_any_fetch() {
    let (proxy, seen) = standin_proxy(vec![("image/png".into(), png(4, 4))]).await;
    let f = fixture(proxy, "mw-t32-rc-forge").await;
    f.grant().await;
    let (_, signed) = f.sign(&["http://1.2.3.4/forge.png"], &[]).await;
    let token = token_of(&image_src(&signed));
    let claims = claims_of(&f.db, &token).await;

    // This file's own key derivation reproduces the server's tag for the server's
    // payload — so the forgeries below differ from a valid token only in what they
    // change. (The payload is compared as bytes: a re-serialized JSON object may
    // order its keys differently, which the server accepts but which is not the
    // same string.)
    let legacy = mint(&claims);
    let (payload, tag) = legacy.split_once('.').unwrap();
    assert_eq!(
        URL_SAFE_NO_PAD.encode(hmac(
            &signing_key(),
            &[b"mw-rc-token\0", payload.as_bytes()]
        )),
        tag,
        "key derivation and token format"
    );
    // And a token this file mints for the same claims is accepted (positive
    // control for the forged ones below).
    let remint = mint(&claims);

    let mut retarget = claims.clone();
    retarget["u"] = json!("http://169.254.169.254/latest/meta-data/");
    let retargeted = format!(
        "{}.{tag}",
        URL_SAFE_NO_PAD.encode(serde_json::to_vec(&retarget).unwrap())
    );
    let mut expired = claims.clone();
    expired["x"] = json!(chrono::Utc::now().timestamp() - 10);
    let mut wrong_session = claims.clone();
    wrong_session["s"] = json!("00".repeat(16));
    let mut as_part = claims.clone();
    as_part["k"] = json!("p");

    let mut flipped = legacy.clone().into_bytes();
    let last = flipped.len() - 1;
    flipped[last] = if flipped[last] == b'A' { b'B' } else { b'A' };
    for (what, t) in [
        ("tampered tag", String::from_utf8(flipped).unwrap()),
        ("retargeted", retargeted),
        ("expired", mint(&expired)),
        ("another session", mint(&wrong_session)),
        ("junk", "abc.def".to_string()),
        ("unknown handle", "A".repeat(22)),
    ] {
        let r = f.load(&format!("/api/rc/i/{t}")).await;
        assert_eq!(r.status(), 403, "{what}");
        // t56: a refusal must reach the opaque-origin frame as what it is. With
        // the global `same-origin` CORP, Firefox reports every failed image as a
        // CORP block and the status is lost.
        assert_eq!(
            r.headers()["cross-origin-resource-policy"],
            "cross-origin",
            "{what}"
        );
        assert_eq!(r.headers()["x-content-type-options"], "nosniff", "{what}");
        assert!(r.headers().get("set-cookie").is_none(), "{what}");
    }
    // A valid image token is not a part token.
    let r = f.load(&format!("/api/rc/p/{token}")).await;
    assert_eq!(r.status(), 403, "kind is bound");
    assert_eq!(r.headers()["cross-origin-resource-policy"], "cross-origin");
    for kind in ["f", "d"] {
        let r = f.load(&format!("/api/rc/{kind}/abc.def")).await;
        assert_eq!(r.status(), 403, "{kind}");
        assert_eq!(
            r.headers()["cross-origin-resource-policy"],
            "cross-origin",
            "{kind}"
        );
    }
    // Every other route keeps the strict default, the rc routes that are not
    // token-authorised included.
    for path in [
        "/api/rc/policy".to_string(),
        format!("/api/rc/stats?emailId={}", f.email_id),
        "/api/me".to_string(),
    ] {
        let r = f
            .user
            .get(format!("{}{path}", f.base))
            .send()
            .await
            .unwrap();
        assert_eq!(
            r.headers()["cross-origin-resource-policy"],
            "same-origin",
            "{path}"
        );
    }
    let r = f
        .user
        .post(format!("{}/api/rc/sign", f.base))
        .json(&json!({ "emailId": f.email_id, "urls": [] }))
        .send()
        .await
        .unwrap();
    assert_eq!(
        r.headers()["cross-origin-resource-policy"],
        "same-origin",
        "sign"
    );
    let r = f.load(&format!("/api/rc/i/{}", mint(&as_part))).await;
    assert_eq!(r.status(), 403, "kind is bound");

    assert_eq!(
        seen.lock().unwrap().connections,
        0,
        "a refused token must not reach the network"
    );
    // Positive control: the untouched token does fetch, and so does this file's
    // re-mint of the same claims (served from the cache the first load filled).
    assert_eq!(f.load(&image_src(&signed)).await.status(), 200);
    assert_eq!(seen.lock().unwrap().connections, 1);
    assert_eq!(f.load(&format!("/api/rc/i/{remint}")).await.status(), 200);
}

#[tokio::test]
async fn reader_bridge_has_one_independent_policy_while_shell_stays_restricted() {
    let (proxy, _) = standin_proxy(vec![]).await;
    let f = fixture(proxy, "mw-t32-rc-frame-policy").await;
    let compatible = f
        .anon
        .get(format!("{}/api/rc/frame?coep=require-corp", f.base))
        .send()
        .await
        .unwrap();
    assert_eq!(compatible.status(), 200);
    assert_eq!(
        compatible.headers()["cross-origin-embedder-policy"],
        "require-corp"
    );
    assert_eq!(
        compatible.headers()["cross-origin-resource-policy"],
        "cross-origin"
    );
    assert!(compatible.headers().get("x-frame-options").is_none());
    let compatible_csp = compatible.headers()["content-security-policy"]
        .to_str()
        .unwrap();
    assert!(compatible_csp.contains("img-src 'self' data: blob:;"));
    assert!(!compatible_csp.contains("https:"));
    assert!(compatible_csp.contains("frame-ancestors 'self'"));
    for origin in [
        "http://tauri.localhost",
        "https://tauri.localhost",
        "tauri://localhost",
    ] {
        let native = f
            .anon
            .get(format!("{}/api/rc/frame", f.base))
            .query(&[("parentOrigin", origin), ("coep", "require-corp")])
            .send()
            .await
            .unwrap();
        assert_eq!(native.status(), 200, "{origin}");
        let policy = native.headers()["content-security-policy"]
            .to_str()
            .unwrap();
        assert!(policy.contains(&format!("frame-ancestors {origin};")));
        assert!(policy.contains("img-src 'self' data: blob:;"));
        assert!(!policy.contains("'unsafe-inline' 'unsafe-eval'"));
        let html = native.text().await.unwrap();
        assert!(html.contains(&format!(
            "const expectedOrigin = {}",
            serde_json::to_string(origin).unwrap()
        )));
    }
    for origin in [
        "null",
        "*",
        "https://unknown.example",
        "https://tauri.localhost/",
        "https://tauri.localhost?x=1",
        "https://tauri.localhost;script-src *",
    ] {
        let refused = f
            .anon
            .get(format!("{}/api/rc/frame", f.base))
            .query(&[("parentOrigin", origin)])
            .send()
            .await
            .unwrap();
        assert_eq!(refused.status(), 403, "{origin}");
    }
    let response = f
        .anon
        .get(format!("{}/api/rc/frame", f.base))
        .send()
        .await
        .unwrap();
    assert_eq!(response.status(), 200);
    assert_eq!(
        response
            .headers()
            .get_all("content-security-policy")
            .iter()
            .count(),
        1
    );
    let csp = response.headers()["content-security-policy"]
        .to_str()
        .unwrap();
    assert!(csp.contains("script-src 'sha256-"));
    assert!(csp.contains("frame-ancestors 'self'"));
    assert!(!csp.contains("connect-src"));
    let csp = csp.to_owned();
    assert!(response.headers().get("x-frame-options").is_none());
    assert_eq!(
        response.headers()["cross-origin-embedder-policy"],
        "credentialless"
    );
    assert_eq!(
        response.headers()["cross-origin-resource-policy"],
        "cross-origin"
    );
    let html = response.text().await.unwrap();
    assert!(html.contains("mw-reader-ready"));
    let script = html
        .split_once("<script>")
        .unwrap()
        .1
        .split_once("</script>")
        .unwrap()
        .0;
    let normalized = script.replace("\r\n", "\n").replace('\r', "\n");
    let digest =
        base64::engine::general_purpose::STANDARD.encode(Sha256::digest(normalized.as_bytes()));
    assert!(
        csp.contains(&format!("'sha256-{digest}'")),
        "CSP hashes the HTML parser's normalized script bytes"
    );
    let shell = f.anon.get(format!("{}/", f.base)).send().await.unwrap();
    assert_eq!(shell.headers()["x-frame-options"], "DENY");
    assert_eq!(
        shell.headers()["cross-origin-embedder-policy"],
        "require-corp"
    );
    assert_eq!(
        shell.headers()["cross-origin-resource-policy"],
        "same-origin"
    );
    assert!(
        shell.headers()["content-security-policy"]
            .to_str()
            .unwrap()
            .contains("img-src 'self' blob: data:")
    );
}

#[tokio::test]
async fn proxy_message_without_an_engine_cache_can_load_signed_images() {
    let (proxy, seen) = standin_proxy(vec![("image/png".into(), png(4, 4))]).await;
    let mut f = fixture(proxy, "mw-t32-rc-cacheless-proxy").await;
    f.email_id = "e1".into();
    assert!(store_at(&f.db).await.get_message("e1").await.is_err());
    f.grant().await;
    let (status, signed) = f.sign(&["http://1.2.3.4/proxy.png"], &[]).await;
    assert_eq!(status, 200, "{signed}");
    assert_eq!(f.load(&image_src(&signed)).await.status(), 200);
    assert_eq!(seen.lock().unwrap().connections, 1);
}

#[tokio::test]
async fn privacy_modes_and_capabilities_are_hot_and_do_not_bypass_message_ownership() {
    let (proxy, seen) = standin_proxy(vec![("image/png".into(), png(4, 4))]).await;
    let f = fixture(proxy, "mw-t32-rc-policy").await;
    let store = store_at(&f.db).await;
    async fn setting(store: &Store, key: &str, value: Value) {
        store
            .put_setting_policy(&mw_store::settings_policy::SettingPolicyRow {
                key: key.into(),
                global_value: Some(value.to_string()),
                allow_user_override: true,
                global_locked: true,
                updated_by: "test".into(),
                updated_at: 1,
            })
            .await
            .unwrap();
    }
    let url = "http://1.2.3.4/policy.png";
    assert_eq!(f.sign(&[url], &[]).await.0, 403);
    setting(&store, "privacy.remoteContent", json!("proxy")).await;
    let (status, signed) = f.sign(&[url], &[]).await;
    assert_eq!(status, 200, "{signed}");
    let src = image_src(&signed);
    let absent = f
        .user
        .post(format!("{}/api/rc/sign", f.base))
        .json(&json!({"emailId":"not-owned", "urls":[url]}))
        .send()
        .await
        .unwrap();
    assert_eq!(absent.status(), 403);
    setting(&store, "privacy.remoteContent", json!("block")).await;
    assert_eq!(f.load(&src).await.status(), 403);
    assert_eq!(seen.lock().unwrap().connections, 0);
    setting(&store, "privacy.remoteContent", json!("direct")).await;
    setting(&store, "privacy.allowDirect", json!(true)).await;
    let policy = f
        .user
        .get(format!("{}/api/rc/policy", f.base))
        .send()
        .await
        .unwrap();
    assert_eq!(policy.headers()["cache-control"], "no-store");
    let policy: Value = policy.json().await.unwrap();
    assert_eq!(policy["mode"], "ask");
    assert_eq!(policy["allowDirect"], false);
    assert_eq!(policy["locked"]["mode"], true);
    setting(&store, "cap.remote_content.direct", json!(true)).await;
    let policy: Value = f
        .user
        .get(format!("{}/api/rc/policy", f.base))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    assert_eq!(policy["mode"], "direct");
    assert_eq!(policy["allowDirect"], true);
    assert_eq!(f.load(&src).await.status(), 200);
    setting(&store, "privacy.trackers", json!(false)).await;
    let (status, tracker) = f
        .sign(&["https://x.list-manage.com/track/open.php?u=1"], &[])
        .await;
    assert_eq!(status, 200);
    setting(&store, "privacy.trackers", json!(true)).await;
    assert_eq!(f.load(&image_src(&tracker)).await.status(), 403);
    assert_eq!(
        seen.lock().unwrap().connections,
        1,
        "new tracker policy must block before fetch"
    );
}

#[tokio::test]
async fn account_restrictions_revoke_already_signed_content_before_fetch() {
    let (proxy, seen) = standin_proxy(vec![("image/png".into(), png(4, 4))]).await;
    let f = fixture(proxy, "mw-t32-rc-account-policy").await;
    f.grant().await;
    let (status, signed) = f.sign(&["http://1.2.3.4/restricted.png"], &[]).await;
    assert_eq!(status, 200);
    let src = image_src(&signed);
    let store = store_at(&f.db).await;
    let key = format!("v6:admin:flags:{}", f.account.to_lowercase());
    for flags in [
        json!({"disabled":true}),
        json!({"force_password_change":true}),
    ] {
        store.set_setting(&key, &flags.to_string()).await.unwrap();
        assert_eq!(f.load(&src).await.status(), 403);
    }
    assert_eq!(seen.lock().unwrap().connections, 0);
    store.set_setting(&key, "{}").await.unwrap();
    assert_eq!(f.load(&src).await.status(), 200);
}

#[tokio::test]
async fn logging_out_or_revoking_the_grant_ends_the_tokens() {
    let (proxy, seen) = standin_proxy(vec![("image/png".into(), png(4, 4))]).await;
    let f = fixture(proxy, "mw-t32-rc-logout").await;
    f.grant().await;
    let (_, signed) = f
        .sign(
            &["http://1.2.3.4/out-a.png", "http://1.2.3.4/out-b.png"],
            &[],
        )
        .await;
    let a = signed["images"][0]["src"].as_str().unwrap().to_string();
    let b = signed["images"][1]["src"].as_str().unwrap().to_string();
    assert_eq!(f.load(&a).await.status(), 200);

    store_at(&f.db)
        .await
        .revoke_remote_image(&f.account, "all", "")
        .await
        .unwrap();
    assert_eq!(f.load(&b).await.status(), 403, "the grant was revoked");
    f.grant().await;
    assert_eq!(f.load(&b).await.status(), 200);

    let out = f
        .user
        .post(format!("{}/api/logout", f.base))
        .send()
        .await
        .unwrap();
    assert!(out.status().is_success(), "{}", out.status());
    // Served from the cache before logout; after logout even a cached URL is refused.
    assert_eq!(f.load(&a).await.status(), 403, "a token after logout");
    assert_eq!(seen.lock().unwrap().connections, 2);
}

#[tokio::test]
async fn only_allowed_image_types_are_served_and_a_tracking_pixel_is_replaced() {
    let (proxy, _) = standin_proxy(vec![
        (
            "image/svg+xml".into(),
            b"<svg xmlns='http://www.w3.org/2000/svg'/>".to_vec(),
        ),
        ("text/html".into(), png(4, 4)),
        ("image/gif".into(), png(4, 4)),
        ("image/png".into(), png(1, 1)),
    ])
    .await;
    let f = fixture(proxy, "mw-t32-rc-types").await;
    f.grant().await;
    let urls = [
        "http://1.2.3.4/types-a.svg",
        "http://1.2.3.4/types-b.png",
        "http://1.2.3.4/types-c.gif",
        "http://1.2.3.4/types-d.png",
    ];
    let (_, signed) = f.sign(&urls, &[]).await;
    let src = |i: usize| signed["images"][i]["src"].as_str().unwrap().to_string();

    let r = f.load(&src(0)).await;
    assert_eq!(r.status(), 415);
    assert_eq!(r.text().await.unwrap(), "SVG images are not loaded");
    let r = f.load(&src(1)).await;
    assert_eq!(r.status(), 415, "declared type not allowed");
    let r = f.load(&src(2)).await;
    assert_eq!(r.status(), 415);
    assert_eq!(
        r.text().await.unwrap(),
        "the image does not match its declared type"
    );

    let r = f.load(&src(3)).await;
    assert_eq!(r.status(), 200);
    assert_eq!(r.headers()["x-mw-remote-content"], "tracker");
    let body = r.bytes().await.unwrap();
    assert_eq!(png_size(&body), (1, 1));
    assert_eq!(
        body[25], 6,
        "the replacement is RGBA (transparent), not the origin's RGB pixel"
    );
    // Reloading the same decoded pixel from cache must not count it twice.
    assert_eq!(f.load(&src(3)).await.status(), 200);
    let stats: Value = f
        .user
        .get(format!("{}/api/rc/stats", f.base))
        .query(&[("emailId", &f.email_id)])
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    assert_eq!(stats["decodedTrackersReplaced"], 1);
    assert_eq!(stats["limited"], false);
    assert_eq!(stats["windowSecs"], 3600);
}

#[tokio::test]
async fn signing_refuses_trackers_and_non_http_urls_and_parts_load_without_a_cookie() {
    let (proxy, _) = standin_proxy(vec![("image/png".into(), png(4, 4))]).await;
    let mut f = fixture(proxy, "mw-t32-rc-sign").await;
    f.email_id = "e1".into();
    f.grant().await;
    let (status, signed) = f
        .sign(
            &[
                "https://x.list-manage.com/track/open.php?u=1",
                "javascript:alert(1)",
                "http://1.2.3.4/sign-ok.png",
            ],
            &["Mblob1", "../etc", "other-message-blob"],
        )
        .await;
    assert_eq!(status, 200, "{signed}");
    let reasons: Vec<&str> = signed["refused"]
        .as_array()
        .unwrap()
        .iter()
        .map(|r| r["reason"].as_str().unwrap())
        .collect();
    assert_eq!(
        reasons,
        ["tracker", "scheme", "blobId", "notMessagePart"],
        "{signed}"
    );
    assert_eq!(signed["images"].as_array().unwrap().len(), 1);

    // The mock upstream serves a part as application/octet-stream text: the signed
    // part URL reaches it without a cookie (proxy mode, the session's credentials)
    // and is refused as not an image.
    let part = signed["parts"][0]["src"].as_str().unwrap().to_string();
    assert!(part.starts_with("/api/rc/p/"), "{part}");
    let r = f.load(&part).await;
    assert_eq!(r.status(), 415, "{}", r.text().await.unwrap());
    let claims = claims_of(&f.db, &token_of(&part)).await;
    let ttl = claims["x"].as_i64().unwrap() - chrono::Utc::now().timestamp();
    assert!(
        ttl <= 300 && ttl > 290,
        "part tokens live five minutes: {ttl}"
    );
    let img = claims_of(&f.db, &token_of(signed["images"][0]["src"].as_str().unwrap())).await;
    let ttl = img["x"].as_i64().unwrap() - chrono::Utc::now().timestamp();
    assert!(
        ttl <= 3600 && ttl > 3590,
        "image tokens live an hour: {ttl}"
    );
}

#[tokio::test]
async fn a_linked_file_downloads_through_the_server_and_is_never_rendered() {
    let (proxy, seen) = standin_proxy(vec![(
        "text/html".into(),
        b"<script>window.__ran=1</script>".to_vec(),
    )])
    .await;
    let f = fixture(proxy, "mw-t32-rc-file").await;
    // No grant: a file download is a deliberate click on the reader's button.
    let (status, signed) = f
        .user
        .post(format!("{}/api/rc/sign", f.base))
        .json(&json!({ "emailId": "e1", "files": ["http://1.2.3.4/dl/report.html", "file:///etc/passwd"] }))
        .send()
        .await
        .map(|r| (r.status().as_u16(), r))
        .unwrap();
    assert_eq!(status, 200);
    let signed: Value = signed.json().await.unwrap();
    assert_eq!(signed["refused"][0]["reason"], "scheme", "{signed}");
    let src = signed["files"][0]["src"].as_str().unwrap().to_string();
    assert!(src.starts_with("/api/rc/f/"), "{src}");

    let r = f.load(&src).await;
    assert_eq!(r.status(), 200);
    let h = r.headers();
    assert_eq!(h["content-type"], "application/octet-stream");
    assert_eq!(
        h["content-disposition"],
        "attachment; filename=\"report.html\""
    );
    let csp: Vec<_> = h
        .get_all("content-security-policy")
        .iter()
        .map(|v| v.to_str().unwrap().to_string())
        .collect();
    assert!(
        csp.iter().any(|v| v == "sandbox; default-src 'none'"),
        "{csp:?}"
    );
    assert_eq!(
        r.bytes().await.unwrap(),
        &b"<script>window.__ran=1</script>"[..]
    );
    assert_eq!(seen.lock().unwrap().connections, 1);

    // Another account's message cannot be used to sign a file.
    let r = f
        .user
        .post(format!("{}/api/rc/sign", f.base))
        .json(&json!({ "emailId": "no-such-message", "files": ["http://1.2.3.4/x.zip"] }))
        .send()
        .await
        .unwrap();
    assert_eq!(r.status(), 403);
}

async fn locked_policy(f: &Fixture, key: &str, value: Value) {
    store_at(&f.db)
        .await
        .put_setting_policy(&mw_store::settings_policy::SettingPolicyRow {
            key: key.into(),
            global_value: Some(value.to_string()),
            allow_user_override: true,
            global_locked: true,
            updated_by: "test".into(),
            updated_at: 1,
        })
        .await
        .unwrap();
}

#[tokio::test]
async fn legacy_image_decoder_and_cache_follow_tracker_and_type_policy() {
    let (proxy, seen) = standin_proxy(vec![
        ("image/png".into(), png(2, 2)),
        ("image/png".into(), png(2, 2)),
        ("text/html".into(), png(4, 4)),
    ])
    .await;
    let f = fixture(proxy, "mw-t32-rc-legacy-policy").await;
    f.grant().await;
    locked_policy(&f, "privacy.trackers", json!(false)).await;
    let url = format!("{}/api/image-proxy", f.base);
    let query = [
        ("url", "http://1.2.3.4/legacy-policy-pixel.png"),
        ("emailId", f.email_id.as_str()),
    ];
    let response = f.user.get(&url).query(&query).send().await.unwrap();
    assert_eq!(response.status(), 200);
    assert_eq!(png_size(&response.bytes().await.unwrap()), (2, 2));
    locked_policy(&f, "privacy.trackers", json!(true)).await;
    let response = f.user.get(&url).query(&query).send().await.unwrap();
    assert_eq!(response.status(), 200);
    assert_eq!(response.headers()["x-mw-remote-content"], "tracker");
    assert_eq!(png_size(&response.bytes().await.unwrap()), (1, 1));
    assert_eq!(seen.lock().unwrap().connections, 2);
    let response = f
        .user
        .get(&url)
        .query(&[
            ("url", "http://1.2.3.4/legacy-policy-type.png"),
            ("emailId", f.email_id.as_str()),
        ])
        .send()
        .await
        .unwrap();
    assert_eq!(response.status(), 415);
}

#[tokio::test]
async fn part_redemption_rechecks_membership_and_charges_rate_budget() {
    let (proxy, _) = standin_proxy(vec![]).await;
    let mut f = fixture(proxy, "mw-t32-rc-part-limit").await;
    f.email_id = "e1".into();
    locked_policy(&f, "privacy.ratePerMin", json!(10)).await;
    locked_policy(&f, "privacy.rateBurst", json!(10)).await;
    let (status, signed) = f.sign(&[], &["Mblob1"]).await;
    assert_eq!(status, 200, "{signed}");
    let src = signed["parts"][0]["src"].as_str().unwrap();
    let mut claims = claims_of(&f.db, &token_of(src)).await;
    claims["u"] = json!("another-message-blob");
    assert_eq!(
        f.load(&format!("/api/rc/p/{}", mint(&claims)))
            .await
            .status(),
        403
    );
    claims["u"] = json!("Mblob1");
    claims["e"] = json!("deleted-message");
    assert_eq!(
        f.load(&format!("/api/rc/p/{}", mint(&claims)))
            .await
            .status(),
        403
    );
    let mut refused = false;
    for _ in 0..12 {
        let status = f.load(src).await.status();
        if status == 429 {
            refused = true;
            break;
        }
        assert_eq!(status, 415, "bound part is served by mock as non-image");
    }
    assert!(
        refused,
        "part decoding must consume the account rate budget"
    );
}

#[tokio::test]
async fn domain_alias_inherits_hot_locks_direct_capability_and_scoped_egress_without_changing_login()
 {
    use mw_store::settings_policy::{SettingScope, SettingValueRow};
    let (proxy, seen) = standin_proxy(vec![("image/png".into(), png(4, 4))]).await;
    let f = fixture(proxy, "mw-t32-rc-alias").await;
    let store = store_at(&f.db).await;
    let before = store
        .sessions_by_account(&f.account)
        .await
        .unwrap()
        .remove(0);
    assert_eq!(before.username, mw_mock_jmap::USER);
    store
        .delete_domain_config("example.org", true)
        .await
        .unwrap();
    store
        .put_domain_config(&mw_store::DomainConfigRow {
            name: "canonical.test".into(),
            upstream_json: json!({"jmap":{"sessionUrl":before.jmap_url}}).to_string(),
            allowlist_json: "[]".into(),
            blocklist_json: "[]".into(),
            enabled: true,
            preferred: false,
            settings_json: "{}".into(),
            updated_at: 1,
        })
        .await
        .unwrap();
    store
        .put_domain_alias("example.org", "canonical.test")
        .await
        .unwrap();
    let value = |key: &str, value: Value| SettingValueRow {
        scope: SettingScope::Domain,
        scope_id: "canonical.test".into(),
        key: key.into(),
        value: value.to_string(),
        locked: true,
        updated_by: "root".into(),
        updated_at: 1,
    };
    store
        .put_setting_value(&value("privacy.remoteContent", json!("block")))
        .await
        .unwrap();
    let response = f
        .user
        .get(format!("{}/api/rc/policy", f.base))
        .send()
        .await
        .unwrap();
    assert_eq!(response.status(), 200);
    let policy: Value = response.json().await.unwrap();
    assert_eq!(policy["mode"], "block");
    assert_eq!(policy["locked"]["mode"], true);
    // A personal settings write must name the session's storage authority (0a11b581,
    // 428 `storageAuthorityRequired` otherwise), so the refusal below is the lock's.
    let me: Value = f
        .user
        .get(format!("{}/api/me", f.base))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    let write = f
        .user
        .put(format!("{}/api/settings/privacy.remoteContent", f.base))
        .header(
            "x-mailwoman-storage-authority",
            me["storageAuthority"]
                .as_str()
                .expect("/api/me names a storageAuthority"),
        )
        .json(&json!({"value":"direct"}))
        .send()
        .await
        .unwrap();
    assert_eq!(write.status(), 403, "alias cannot override canonical lock");
    let refusal: Value = write.json().await.unwrap();
    assert_eq!(refusal["code"], "setting_locked", "{refusal}");
    locked_policy(&f, "privacy.allowDirect", json!(true)).await;
    store
        .put_setting_value(&value("privacy.remoteContent", json!("direct")))
        .await
        .unwrap();
    store
        .put_setting_value(&value("cap.remote_content.direct", json!(false)))
        .await
        .unwrap();
    let policy: Value = f
        .user
        .get(format!("{}/api/rc/policy", f.base))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    assert_eq!(policy["mode"], "ask");
    assert_eq!(policy["allowDirect"], false);
    store
        .put_setting_value(&value("privacy.remoteContent", json!("proxy")))
        .await
        .unwrap();
    let mut route = store.get_egress_proxy("rc").await.unwrap().unwrap();
    route.scope = "domain:canonical.test".into();
    store.put_egress_proxy(&route).await.unwrap();
    let (status, signed) = f.sign(&["http://1.2.3.4/alias-route.png"], &[]).await;
    assert_eq!(status, 200, "{signed}");
    assert_eq!(f.load(&image_src(&signed)).await.status(), 200);
    assert_eq!(
        seen.lock().unwrap().connections,
        1,
        "canonical scoped proxy must carry alias traffic"
    );
    // Configuration rows using the alias are resolved to the same scope as well.
    route.scope = "domain:example.org".into();
    store.put_egress_proxy(&route).await.unwrap();
    let (_, signed) = f
        .sign(&["http://1.2.3.4/alias-route-config.png"], &[])
        .await;
    assert_eq!(f.load(&image_src(&signed)).await.status(), 200);
    assert_eq!(seen.lock().unwrap().connections, 2);
    let after = store
        .sessions_by_account(&f.account)
        .await
        .unwrap()
        .remove(0);
    assert_eq!(after.username, before.username);
    assert_eq!(after.account_id, before.account_id);
    assert_eq!(after.credentials.username, before.credentials.username);
}

#[tokio::test]
async fn message_tracker_stats_require_session_and_owned_message_and_start_at_zero() {
    let (proxy, _) = standin_proxy(vec![]).await;
    let f = fixture(proxy, "mw-t32-rc-stats-auth").await;
    let store = store_at(&f.db).await;
    let other = store
        .create_account(
            &mw_store::NewAccount {
                kind: mw_store::AccountKind::Imap,
                host: "other.test",
                port: 993,
                tls: "implicit",
                username: "other@other.test",
                sync_policy_json: "{}",
            },
            &mw_store::Credentials {
                username: "other@other.test".into(),
                password: "test".into(),
            },
        )
        .await
        .unwrap();
    let mailbox = store
        .upsert_mailbox(&mw_store::MailboxUpsert {
            account_id: &other,
            name: "INBOX",
            role: Some("inbox"),
            uidvalidity: 1,
            uidnext: 2,
            highestmodseq: 0,
            total: 1,
            unread: 1,
            parent_id: None,
        })
        .await
        .unwrap();
    let foreign = store
        .upsert_message(&mw_store::MessageUpsert {
            account_id: &other,
            mailbox_id: &mailbox,
            uid: 1,
            uidvalidity: 1,
            message_id: None,
            thread_id: None,
            internaldate: None,
            size: 1,
            flags_json: "[]",
            envelope: None,
            blob_ref: None,
        })
        .await
        .unwrap();
    let url = format!("{}/api/rc/stats", f.base);
    assert_eq!(
        f.anon
            .get(&url)
            .query(&[("emailId", &f.email_id)])
            .send()
            .await
            .unwrap()
            .status(),
        401
    );
    assert_eq!(
        f.user
            .get(&url)
            .query(&[("emailId", foreign.as_str())])
            .send()
            .await
            .unwrap()
            .status(),
        403
    );
    let response = f
        .user
        .get(&url)
        .query(&[("emailId", &f.email_id)])
        .send()
        .await
        .unwrap();
    assert_eq!(response.status(), 200);
    assert!(
        response.headers()["cache-control"]
            .to_str()
            .unwrap()
            .contains("no-store")
    );
    let stats: Value = response.json().await.unwrap();
    assert_eq!(
        stats,
        // t56 adds the per-message load outcomes for the loading toast.
        json!({"decodedTrackersReplaced":0,"limited":false,"windowSecs":3600,"imagesLoaded":0,"imagesFailed":0,"failedImageRefs":[],"retryableImageRefs":[]})
    );
}

#[tokio::test]
async fn manual_image_downloads_require_policy_and_real_message_membership_at_every_use() {
    let (proxy, seen) = standin_proxy(vec![
        ("image/png".into(), png(4, 4)),
        ("image/svg+xml".into(), b"<svg/>".to_vec()),
    ])
    .await;
    let f = fixture(proxy, "mw-rc-image-download").await;
    let endpoint = format!("{}/api/rc/sign", f.base);
    let body = json!({"emailId":"e1","imageDownloads":["http://1.2.3.4/download-image.png","http://1.2.3.4/not-in-message.png","http://1.2.3.4/declared-pixel.png","http://1.2.3.4/download-svg.svg"]});
    assert_eq!(
        f.user
            .post(&endpoint)
            .header("x-mailwoman-account", &f.account)
            .json(&body)
            .send()
            .await
            .unwrap()
            .status(),
        403
    );
    assert_eq!(seen.lock().unwrap().connections, 0);
    async fn enable(f: &Fixture, value: bool) {
        let url = format!("{}/admin/settings/privacy.remoteImageDownloads", f.base);
        let current = f.admin.get(&url).send().await.unwrap();
        let tag = current.headers()["etag"].to_str().unwrap().to_owned();
        let response = f
            .admin
            .put(url)
            .header("if-match", tag)
            .json(&json!({"globalValue":value}))
            .send()
            .await
            .unwrap();
        assert_eq!(response.status(), 200, "{}", response.text().await.unwrap());
    }
    enable(&f, true).await;
    assert_eq!(
        f.user
            .post(&endpoint)
            .json(&body)
            .send()
            .await
            .unwrap()
            .status(),
        409
    );
    assert_eq!(
        f.user
            .post(&endpoint)
            .header("x-mailwoman-account", "foreign")
            .json(&body)
            .send()
            .await
            .unwrap()
            .status(),
        409
    );
    let response = f
        .user
        .post(&endpoint)
        .header("x-mailwoman-account", &f.account)
        .json(&body)
        .send()
        .await
        .unwrap();
    assert_eq!(response.status(), 200);
    let signed: Value = response.json().await.unwrap();
    assert_eq!(
        signed["imageDownloads"].as_array().unwrap().len(),
        2,
        "{signed}"
    );
    assert_eq!(signed["refused"][0]["reason"], "message");
    assert_eq!(signed["refused"][1]["reason"], "tracker");
    let image = signed["imageDownloads"][0]["src"].as_str().unwrap();
    assert_eq!(
        f.load(&image.replace("/api/rc/d/", "/api/rc/i/"))
            .await
            .status(),
        403
    );
    assert_eq!(
        f.load(image).await.status(),
        401,
        "manual download requires the current authenticated account"
    );
    assert_eq!(
        f.user
            .get(format!("{}{image}", f.base))
            .header("x-mailwoman-account", "foreign")
            .send()
            .await
            .unwrap()
            .status(),
        409
    );
    let loaded = f
        .user
        .get(format!("{}{image}", f.base))
        .send()
        .await
        .unwrap();
    assert_eq!(loaded.status(), 200);
    assert_eq!(
        loaded.headers()["content-disposition"],
        "attachment; filename=\"message-image.png\""
    );
    assert_eq!(loaded.headers()["content-type"], "image/png");
    assert!(
        loaded
            .bytes()
            .await
            .unwrap()
            .starts_with(b"\x89PNG\r\n\x1a\n")
    );
    assert_eq!(seen.lock().unwrap().connections, 1);
    // Even another valid session with the same provider-local account ID cannot
    // redeem a token minted for this browser's authority.
    tokio::time::sleep(std::time::Duration::from_secs(3)).await;
    let other = reqwest::Client::builder()
        .cookie_store(true)
        .no_proxy()
        .build()
        .unwrap();
    let login = other
        .post(format!("{}/api/login", f.base))
        .json(&json!({"username":mw_mock_jmap::USER,"password":mw_mock_jmap::PASS}))
        .send()
        .await
        .unwrap();
    assert_eq!(login.status(), 200, "{}", login.text().await.unwrap());
    assert_eq!(
        other
            .get(format!("{}{image}", f.base))
            .send()
            .await
            .unwrap()
            .status(),
        409
    );
    assert_eq!(seen.lock().unwrap().connections, 1);
    let file = f
        .user
        .post(&endpoint)
        .json(&json!({"emailId":"e1","files":["http://1.2.3.4/not-in-message.zip"]}))
        .send()
        .await
        .unwrap()
        .json::<Value>()
        .await
        .unwrap();
    assert_eq!(file["files"].as_array().unwrap().len(), 0);
    assert_eq!(file["refused"][0]["reason"], "message");
    assert_eq!(seen.lock().unwrap().connections, 1);
    let svg = signed["imageDownloads"][1]["src"].as_str().unwrap();
    assert_eq!(
        f.user
            .get(format!("{}{svg}", f.base))
            .send()
            .await
            .unwrap()
            .status(),
        415
    );
    assert_eq!(seen.lock().unwrap().connections, 2);
    enable(&f, false).await;
    assert_eq!(
        f.user
            .get(format!("{}{image}", f.base))
            .send()
            .await
            .unwrap()
            .status(),
        403,
        "cached image cannot bypass the current download policy"
    );
    assert_eq!(seen.lock().unwrap().connections, 2);
    enable(&f, true).await;
    // The signing secret is available only to this test: forge a valid token for
    // an absent URL, proving redemption rechecks membership rather than trusting it.
    let mut claims = claims_of(&f.db, &token_of(image)).await;
    claims["u"] = json!("http://1.2.3.4/retargeted.png");
    let forged = mint(&claims);
    assert_eq!(
        f.user
            .get(format!("{}/api/rc/d/{forged}", f.base))
            .send()
            .await
            .unwrap()
            .status(),
        403
    );
    assert_eq!(seen.lock().unwrap().connections, 2);
    claims["k"] = json!("f");
    let forged_file = mint(&claims);
    assert_eq!(
        f.load(&format!("/api/rc/f/{forged_file}")).await.status(),
        403,
        "linked files also recheck exact message membership at redemption"
    );
    assert_eq!(seen.lock().unwrap().connections, 2);
}

/// t56: a fetch that failed at the network is answered as a short-lived,
/// cacheable failure, remembered for the account so a repeat costs no
/// connection, counted for the reader's loading toast, and forgotten when the
/// reader asks to retry.
#[tokio::test]
async fn ordinary_resigning_preserves_outcomes_and_retry_clears_only_selected_targets() {
    let (proxy, seen) = standin_status_proxy(vec![
        (200, "image/png".into(), png(4, 4)),
        (500, "text/plain".into(), Vec::new()),
        (403, "text/plain".into(), Vec::new()),
        (200, "image/png".into(), png(4, 4)),
    ])
    .await;
    let f = fixture(proxy, "mw-rc-idempotent-outcomes").await;
    f.grant().await;
    let urls = [
        "http://1.2.3.4/good.png",
        "http://1.2.3.4/transient.png",
        "http://1.2.3.4/permanent.png",
    ];
    let (status, signed) = f.sign(&urls, &[]).await;
    assert_eq!(status, 200);
    for (index, image) in signed["images"].as_array().unwrap().iter().enumerate() {
        let response = f.load(image["src"].as_str().unwrap()).await;
        assert_eq!(response.status().is_success(), index == 0);
    }
    async fn stats(f: &Fixture) -> Value {
        f.user
            .get(format!("{}/api/rc/stats?emailId={}", f.base, f.email_id))
            .send()
            .await
            .unwrap()
            .json()
            .await
            .unwrap()
    }
    let before = stats(&f).await;
    assert_eq!(before["imagesLoaded"], 1);
    assert_eq!(before["imagesFailed"], 2);
    assert_eq!(
        before["retryableImageRefs"],
        json!([signed["images"][1]["ref"]])
    );
    // Do not redeem the repeated signatures: an unchanged browser image src
    // need not produce a second network response to reconstruct its outcome.
    assert_eq!(f.sign(&urls, &[]).await.0, 200);
    assert_eq!(stats(&f).await, before);
    assert_eq!(seen.lock().unwrap().connections, 3);
    let retry: Value = f
        .user
        .post(format!("{}/api/rc/sign", f.base))
        .json(&json!({"emailId":f.email_id,"urls":[urls[1],"data:image/png,refused"],"retry":true}))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    assert_eq!(retry["images"].as_array().unwrap().len(), 1);
    assert_eq!(retry["refused"].as_array().unwrap().len(), 1);
    let pending = stats(&f).await;
    assert_eq!(pending["imagesLoaded"], 1);
    assert_eq!(pending["imagesFailed"], 1);
    assert_eq!(
        pending["failedImageRefs"],
        json!([signed["images"][2]["ref"]])
    );
    assert_eq!(pending["retryableImageRefs"], json!([]));
    assert_eq!(f.load(&image_src(&retry)).await.status(), 200);
    let recovered = stats(&f).await;
    assert_eq!(recovered["imagesLoaded"], 2);
    assert_eq!(recovered["imagesFailed"], 1);
    assert_eq!(seen.lock().unwrap().connections, 4);
}

#[tokio::test]
async fn a_failed_fetch_is_remembered_briefly_counted_and_retried_on_request() {
    // Larger than any `privacy.maxBytes`: refused from Content-Length, after the
    // origin was reached, so it is a network failure worth remembering.
    let (proxy, seen) = standin_proxy(vec![("image/png".into(), vec![0u8; 9 * 1024 * 1024])]).await;
    let f = fixture(proxy, "mw-t56-rc-negative").await;
    f.grant().await;
    let url = "http://1.2.3.4/too-large.png";
    let (_, signed) = f.sign(&[url], &[]).await;
    let src = image_src(&signed);

    let first = f.load(&src).await;
    assert_eq!(first.status(), 502);
    assert_eq!(first.headers()["x-mw-remote-content"], "failed");
    assert_eq!(first.headers()["cache-control"], "private, max-age=10");
    assert_eq!(
        first.headers()["cross-origin-resource-policy"],
        "cross-origin"
    );
    assert_eq!(seen.lock().unwrap().connections, 1);

    // The same link, and a freshly signed one, answer from memory.
    for again in [src.clone(), image_src(&f.sign(&[url], &[]).await.1)] {
        let r = f.load(&again).await;
        assert_eq!(r.status(), 502);
        assert_eq!(r.headers()["x-mw-remote-content"], "failed");
    }
    assert_eq!(
        seen.lock().unwrap().connections,
        1,
        "no refetch inside the window"
    );

    let stats: Value = f
        .user
        .get(format!("{}/api/rc/stats?emailId={}", f.base, f.email_id))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    assert_eq!(stats["imagesFailed"], json!(1), "{stats}");
    assert_eq!(stats["imagesLoaded"], json!(0), "{stats}");
    assert_eq!(
        stats["failedImageRefs"],
        json!([signed["images"][0]["ref"]])
    );
    assert_eq!(
        stats["retryableImageRefs"],
        json!([]),
        "oversize is not a transient failure"
    );

    // Retry images: signing with retry forgets the failure, so it fetches again.
    let retried: Value = f
        .user
        .post(format!("{}/api/rc/sign", f.base))
        .json(&json!({ "emailId": f.email_id, "urls": [url], "retry": true }))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    assert_ne!(
        image_src(&retried),
        src,
        "even a same-second Retry must avoid the browser negative cache"
    );
    // t67: signing starts a new load, so its counters start from nothing
    // until that load's own fetch answers.
    let fresh: Value = f
        .user
        .get(format!("{}/api/rc/stats?emailId={}", f.base, f.email_id))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    assert_eq!(fresh["imagesFailed"], json!(0), "{fresh}");
    assert_eq!(f.load(&image_src(&retried)).await.status(), 502);
    assert_eq!(
        seen.lock().unwrap().connections,
        2,
        "retry reached the network"
    );
    let after: Value = f
        .user
        .get(format!("{}/api/rc/stats?emailId={}", f.base, f.email_id))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    assert_eq!(after["imagesFailed"], json!(1), "{after}");
}

#[tokio::test]
async fn tracker_consent_is_explicit_per_message_revocable_and_rechecks_decoded_cache() {
    let (proxy, _) = standin_proxy(vec![("image/png".into(), png(2, 2))]).await;
    let f = fixture(proxy, "mw-rc-tracker-consent").await;
    f.grant().await;
    // The re-encode cache intentionally shares public URLs across accounts on
    // the same route. Keep fixture URLs distinct while retaining the beacon
    // suffix: other fixtures serve different bytes behind the test route `rc`.
    let known = "http://1.2.3.4/tracker-consent/open.png";
    let tiny = "http://1.2.3.4/tracker-consent/ordinary-small.png";
    let (_, blocked) = f.sign(&[known, tiny], &[]).await;
    assert_eq!(blocked["refused"][0]["reason"], "tracker");
    let tiny_src = image_src(&blocked);
    let response = f.load(&tiny_src).await;
    assert_eq!(response.headers()["x-mw-remote-content"], "tracker");
    assert_eq!(png_size(&response.bytes().await.unwrap()), (1, 1));
    store_at(&f.db)
        .await
        .grant_remote_image("another-account", "trackers-single", &f.email_id)
        .await
        .unwrap();
    let (_, isolated) = f.sign(&[known], &[]).await;
    assert!(
        isolated["images"].as_array().unwrap().is_empty(),
        "another account's consent must not apply"
    );
    let mutation = json!({"scopeKind":"trackers-single","scopeValue":f.email_id});
    assert_eq!(
        f.user
            .post(format!("{}/api/remote-images/grant", f.base))
            .json(&json!({"scopeKind":"trackers-single","scopeValue":"missing-message"}))
            .send()
            .await
            .unwrap()
            .status(),
        403
    );
    assert_eq!(
        f.user
            .post(format!("{}/api/remote-images/grant", f.base))
            .json(&mutation)
            .send()
            .await
            .unwrap()
            .status(),
        200
    );
    let grants: Value = f
        .user
        .get(format!("{}/api/remote-images/grants", f.base))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    assert!(
        grants["list"]
            .as_array()
            .unwrap()
            .iter()
            .any(|grant| grant["scopeKind"] == "trackers-single"
                && grant["scopeValue"] == f.email_id)
    );
    let (_, allowed) = f.sign(&[known, tiny], &[]).await;
    assert_eq!(allowed["images"].as_array().unwrap().len(), 2, "{allowed}");
    for image in allowed["images"].as_array().unwrap() {
        let response = f.load(image["src"].as_str().unwrap()).await;
        assert_eq!(response.status(), 200);
        assert_eq!(
            png_size(&response.bytes().await.unwrap()),
            (2, 2),
            "consent must use a separate decoded-image cache variant"
        );
    }
    let legacy = f
        .user
        .get(format!("{}/api/image-proxy", f.base))
        .query(&[("url", known), ("emailId", f.email_id.as_str())])
        .send()
        .await
        .unwrap();
    assert_eq!(legacy.status(), 200);
    assert_eq!(png_size(&legacy.bytes().await.unwrap()), (2, 2));
    assert_eq!(
        f.user
            .post(format!("{}/api/remote-images/revoke", f.base))
            .json(&mutation)
            .send()
            .await
            .unwrap()
            .status(),
        200
    );
    assert_eq!(
        f.load(allowed["images"][0]["src"].as_str().unwrap())
            .await
            .status(),
        403,
        "old signed tracker URLs must stop working after consent is revoked"
    );
    let response = f.load(allowed["images"][1]["src"].as_str().unwrap()).await;
    assert_eq!(response.headers()["x-mw-remote-content"], "tracker");
    assert_eq!(png_size(&response.bytes().await.unwrap()), (1, 1));
}

#[tokio::test]
async fn tracker_consent_cannot_bypass_permission_denial_or_locked_tracker_blocking() {
    let (proxy, _) = standin_proxy(vec![("image/png".into(), png(4, 4))]).await;
    let f = fixture(proxy, "mw-rc-tracker-locks").await;
    f.grant().await;
    let mutation = json!({"scopeKind":"trackers-single","scopeValue":f.email_id});
    let known = "http://1.2.3.4/tracker-locks/open.png";
    assert_eq!(
        f.user
            .post(format!("{}/api/remote-images/grant", f.base))
            .json(&mutation)
            .send()
            .await
            .unwrap()
            .status(),
        200
    );
    let (_, signed) = f.sign(&[known], &[]).await;
    let src = image_src(&signed);
    assert_eq!(f.load(&src).await.status(), 200);
    for (key, value) in [
        ("privacy.trackerOverrideAllowed", json!(false)),
        ("privacy.trackers", json!(true)),
    ] {
        if key == "privacy.trackers" {
            locked_policy(&f, "privacy.trackerOverrideAllowed", json!(true)).await;
            store_at(&f.db)
                .await
                .grant_remote_image(&f.account, "trackers-single", &f.email_id)
                .await
                .unwrap();
        }
        locked_policy(&f, key, value).await;
        let policy: Value = f
            .user
            .get(format!("{}/api/rc/policy", f.base))
            .send()
            .await
            .unwrap()
            .json()
            .await
            .unwrap();
        assert_eq!(policy["trackerOverrideAllowed"], false);
        assert_eq!(
            f.load(&src).await.status(),
            403,
            "old cached capability after {key}"
        );
        let (_, refused) = f.sign(&[known], &[]).await;
        assert_eq!(refused["refused"][0]["reason"], "tracker");
        assert_eq!(
            f.user
                .get(format!("{}/api/image-proxy", f.base))
                .query(&[("url", known), ("emailId", f.email_id.as_str())])
                .send()
                .await
                .unwrap()
                .status(),
            403
        );
        assert_eq!(
            f.user
                .post(format!("{}/api/remote-images/grant", f.base))
                .json(&mutation)
                .send()
                .await
                .unwrap()
                .status(),
            403
        );
        assert_eq!(
            f.user
                .post(format!("{}/api/remote-images/revoke", f.base))
                .json(&mutation)
                .send()
                .await
                .unwrap()
                .status(),
            200,
            "revocation remains available under every policy"
        );
    }
}

// ── POST /api/rc/batch ───────────────────────────────────────────────────────────

impl Fixture {
    async fn batch(&self, tokens: &[String], bodies: bool) -> (u16, Value) {
        let r = self
            .user
            .post(format!("{}/api/rc/batch", self.base))
            .json(&json!({ "tokens": tokens, "bodies": bodies }))
            .send()
            .await
            .unwrap();
        let status = r.status().as_u16();
        (status, r.json().await.unwrap_or(Value::Null))
    }
}

fn batch_tokens(signed: &Value) -> Vec<String> {
    signed["images"]
        .as_array()
        .unwrap()
        .iter()
        .map(|image| token_of(image["src"].as_str().unwrap()))
        .collect()
}

#[tokio::test]
async fn batch_without_a_session_or_a_grant_never_fetches() {
    let (proxy, seen) = standin_proxy(vec![("image/png".into(), png(64, 64))]).await;
    let f = fixture(proxy, "mw-rc-batch-ungranted").await;
    // Signing needs a grant, so mint the tokens while granted, then revoke.
    f.grant().await;
    let (_, signed) = f
        .sign(&["http://1.2.3.4/batch-a.png", "http://1.2.3.4/batch-b.png"], &[])
        .await;
    let tokens = batch_tokens(&signed);
    store_at(&f.db)
        .await
        .revoke_remote_image(&f.account, "all", "")
        .await
        .unwrap();
    let anonymous = f
        .anon
        .post(format!("{}/api/rc/batch", f.base))
        .json(&json!({ "tokens": tokens }))
        .send()
        .await
        .unwrap();
    assert_eq!(anonymous.status(), 401);
    let (status, body) = f.batch(&tokens, true).await;
    assert_eq!(status, 200, "{body}");
    for result in body["results"].as_array().unwrap() {
        assert_eq!(result["status"], 403, "{body}");
    }
    assert_eq!(seen.lock().unwrap().connections, 0, "no fetch without consent");
}

#[tokio::test]
async fn batch_caps_the_token_count() {
    let (proxy, seen) = standin_proxy(vec![("image/png".into(), png(64, 64))]).await;
    let f = fixture(proxy, "mw-rc-batch-cap").await;
    f.grant().await;
    let many: Vec<String> = (0..101).map(|i| format!("tok{i}")).collect();
    assert_eq!(f.batch(&many, false).await.0, 400);
    assert_eq!(f.batch(&[], false).await.0, 400);
    assert_eq!(seen.lock().unwrap().connections, 0);
}

#[tokio::test]
async fn batch_isolates_a_refused_url_and_serves_the_second_batch_from_the_cache() {
    let (proxy, seen) = standin_proxy(vec![("image/png".into(), png(64, 64))]).await;
    let f = fixture(proxy, "mw-rc-batch-mixed").await;
    f.grant().await;
    let (status, signed) = f
        .sign(
            &[
                "http://1.2.3.4/batch-ok-1.png",
                "http://169.254.169.254/latest/meta-data/x.png",
                "http://1.2.3.4/batch-ok-2.png",
            ],
            &[],
        )
        .await;
    assert_eq!(status, 200, "{signed}");
    let mut tokens = batch_tokens(&signed);
    tokens.push("not-a-token".into());
    let (status, body) = f.batch(&tokens, true).await;
    assert_eq!(status, 200, "{body}");
    let results = body["results"].as_array().unwrap();
    assert_eq!(results.len(), 4);
    assert_eq!(results[0]["status"], 200, "{body}");
    assert_eq!(results[1]["status"], 403, "SSRF refusal stays per URL: {body}");
    assert_eq!(results[2]["status"], 200, "{body}");
    assert_eq!(results[3]["status"], 403, "{body}");
    for i in [0, 2] {
        let bytes = base64::engine::general_purpose::STANDARD
            .decode(results[i]["body"].as_str().unwrap())
            .unwrap();
        assert!(bytes.starts_with(b"\x89PNG"));
        assert!(results[i]["etag"].is_string());
    }
    let upstream = seen.lock().unwrap().requests.len();
    assert!(upstream >= 2);
    let (_, again) = f.batch(&tokens[..1], false).await;
    assert_eq!(again["results"][0]["status"], 200, "{again}");
    assert!(again["results"][0].get("body").is_none());
    assert_eq!(
        seen.lock().unwrap().requests.len(),
        upstream,
        "a cached image costs no second upstream request"
    );
    // The single-URL route now answers from the same cache.
    let single = f.load(signed["images"][0]["src"].as_str().unwrap()).await;
    assert_eq!(single.status(), 200);
    assert_eq!(seen.lock().unwrap().requests.len(), upstream);
}
