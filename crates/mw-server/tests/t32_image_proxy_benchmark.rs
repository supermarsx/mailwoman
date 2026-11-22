//! Explicit, private fixture only. The measured clients run in OTHER processes.
//! No benchmark handler is added to the application router.
use super::*;
use axum::{body::Body, extract::State, http::{Request, StatusCode}, middleware::Next, response::IntoResponse, routing::{get, post}, Router};
use std::{sync::atomic::{AtomicU64, Ordering}, time::Duration};
use tokio::task::JoinHandle;

#[path = "common/native_enforced_policy.rs"]
mod enforced;
const BROWSER: &str = "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/140.0.0.0 Safari/537.36";
const PEOPLE: usize = 16;
const URLS: usize = 2048;
const DEADLINE: u64 = 900;

struct Tasks(Vec<JoinHandle<()>>);
impl Drop for Tasks { fn drop(&mut self) { for task in &self.0 { task.abort(); } } }
struct Person { client: reqwest::Client, account: String, message: String }
struct Bench {
    base: String, db: String, admin: reqwest::Client, people: Vec<Person>,
    fetches: Arc<AtomicU64>, token: String, sequence: AtomicU64,
}
fn strict_token() -> String {
    let value=std::env::var("MW_IMAGE_BENCH_TOKEN").expect("private parent token required");
    assert!(value.len()==64 && value.bytes().all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b)), "strict private token"); value
}
fn authorized(headers: &axum::http::HeaderMap, state: &Bench) -> bool {
    headers.get("x-mw-image-bench-token").and_then(|v| v.to_str().ok())==Some(state.token.as_str())
}
async fn policy(db: &str, key: &str, value: Value) {
    let def=mw_server::settings_registry::lookup(key).expect("known cache key");
    let value=mw_server::settings_registry::validate(def,&value,"globalValue").unwrap();
    store_at(db).await.put_setting_policy(&mw_store::settings_policy::SettingPolicyRow {
        key:key.into(),global_value:Some(value.to_string()),allow_user_override:false,
        global_locked:true,updated_by:"private-image-benchmark".into(),updated_at:chrono::Utc::now().timestamp(),
    }).await.unwrap();
}

// Only this MOCK upstream translates its sixteen explicit credentials to the
// existing mock's identity. The real application authenticates and seals each
// distinct username/session, and observes a distinct upstream account.
async fn mock_identity(mut request: Request<Body>, next: Next) -> axum::response::Response {
    let identity=request.headers().get("authorization").and_then(|v|v.to_str().ok())
        .and_then(|v|v.strip_prefix("Basic "))
        .and_then(|v|base64::engine::general_purpose::STANDARD.decode(v).ok())
        .and_then(|v|String::from_utf8(v).ok())
        .and_then(|v|(0..PEOPLE).find(|i|v==format!("image{i}@example.org:image-benchmark-password")));
    let Some(i)=identity else { return StatusCode::UNAUTHORIZED.into_response(); };
    let basic=base64::engine::general_purpose::STANDARD.encode(format!("{}:{}",mw_mock_jmap::USER,mw_mock_jmap::PASS));
    request.headers_mut().insert("authorization",format!("Basic {basic}").parse().unwrap());
    let response=next.run(request).await;
    let (mut parts,body)=response.into_parts();
    let bytes=axum::body::to_bytes(body,1<<20).await.unwrap();
    let text=String::from_utf8(bytes.to_vec()).unwrap()
        .replace(mw_mock_jmap::ACCOUNT_ID,&format!("image-bench-account-{i}"))
        .replace(mw_mock_jmap::USER,&format!("image{i}@example.org"));
    parts.headers.remove("content-length");
    axum::response::Response::from_parts(parts,Body::from(text))
}

async fn head(stream:&mut TcpStream)->Option<String> {
    let mut bytes=Vec::new();
    while bytes.len()<16384 {
        let byte=stream.read_u8().await.ok()?; bytes.push(byte);
        if bytes.ends_with(b"\r\n\r\n") { return String::from_utf8(bytes).ok(); }
    } None
}
async fn origin(listener:TcpListener, fetches:Arc<AtomicU64>) {
    let mut children=tokio::task::JoinSet::new();
    loop {
        tokio::select! {
            result=listener.accept()=>{
                let (mut stream,_)=result.unwrap();let fetches=fetches.clone();
                children.spawn(async move {
                    let _=tokio::time::timeout(Duration::from_secs(10),async {
                        let connect=head(&mut stream).await.unwrap_or_default();
                        if !connect.starts_with("CONNECT 1.2.3.4:80 ") { return; }
                        stream.write_all(b"HTTP/1.1 200 Connection established\r\n\r\n").await.unwrap();
                        let request=head(&mut stream).await.unwrap_or_default();
                        if !request.starts_with("GET ") { return; }
                        fetches.fetch_add(1,Ordering::Relaxed);
                        // A bounded real network wait lets identical cold requests
                        // overlap. It is declared origin delay, not server CPU.
                        tokio::time::sleep(Duration::from_millis(20)).await;
                        let bytes=png(if request.contains("/128/"){128}else{64},if request.contains("/128/"){128}else{64});
                        stream.write_all(format!("HTTP/1.1 200 OK\r\nContent-Type: image/png\r\nContent-Length: {}\r\nCache-Control: public, max-age=3600\r\nConnection: close\r\n\r\n",bytes.len()).as_bytes()).await.unwrap();
                        stream.write_all(&bytes).await.unwrap();
                    }).await;
                });
            },
            _=children.join_next(), if !children.is_empty()=>{},
        }
    }
}

async fn stats(state:&Bench)->Value {
    let response=state.admin.get(format!("{}/admin/egress/privacy-stats",state.base)).send().await.unwrap();
    assert_eq!(response.status(),200);
    let value=response.json::<Value>().await.unwrap();
    // Numeric extraction happens at the consumer; NEVER export raw paths,
    // lastError strings, cookies, grants or upstream request headers.
    value
}
async fn counters(State(state):State<Arc<Bench>>,headers:axum::http::HeaderMap)->axum::response::Response {
    if !authorized(&headers,&state) { return StatusCode::UNAUTHORIZED.into_response(); }
    let raw=stats(&state).await;
    let tier=&raw["imageCache"]["tiers"];
    let paths=[("ramHits","/ram/hits"),("diskHits","/disk/hits"),("misses","/misses"),
        ("ramUsedBytes","/ram/usedBytes"),("ramBudgetBytes","/ram/budgetBytes"),
        ("diskUsedBytes","/disk/bytes"),("diskBudgetBytes","/disk/budgetBytes"),
        ("diskPendingBytes","/disk/pendingBytes"),("diskErrors","/disk/errors"),
        ("ramEntries","/ram/entries"),("diskEntries","/disk/entries")];
    let mut clean=serde_json::Map::new();
    for (name,path) in paths {clean.insert(name.into(),json!(tier.pointer(path).and_then(Value::as_u64).expect("actual numeric cache counter")));}
    clean.insert("originFetches".into(),json!(state.fetches.load(Ordering::Relaxed)));
    axum::Json(Value::Object(clean)).into_response()
}
#[derive(serde::Deserialize)]
struct Prepare { case:String, size:u32 }
async fn prepare(State(state):State<Arc<Bench>>,headers:axum::http::HeaderMap,axum::Json(args):axum::Json<Prepare>)->axum::response::Response {
    if !authorized(&headers,&state) { return StatusCode::UNAUTHORIZED.into_response(); }
    if !["cold","coalesced","ram","disk"].contains(&args.case.as_str()) || ![64,128].contains(&args.size) { return StatusCode::BAD_REQUEST.into_response(); }
    policy(&state.db,"privacy.cacheBytes",json!(if args.case=="disk"{0}else{8<<20})).await;
    let sequence=state.sequence.fetch_add(1,Ordering::Relaxed);
    let mut paths=Vec::new();
    for (index,person) in state.people.iter().enumerate() {
        let count=if args.case=="cold"{URLS/PEOPLE}else{1};
        // One genuine batched signing request/account;128 is below the
        // production MAX_SIGN256. Each URL still gets its own real signature.
        let urls=(0..count).map(|n|format!("http://1.2.3.4/{}/{sequence}/{index}/{n}.png",args.size)).collect::<Vec<_>>();
        let signed=person.client.post(format!("{}/api/rc/sign",state.base))
            .json(&json!({"emailId":person.message,"urls":urls,"parts":[]}))
            .send().await.unwrap();assert_eq!(signed.status(),200);
        let signed=signed.json::<Value>().await.unwrap();
        let images=signed["images"].as_array().unwrap();assert_eq!(images.len(),count);
        for image in images {paths.push(json!({"path":image["src"].as_str().unwrap(),"principalIndex":index}));}
    }
    // Warm-up also learns the actual re-encoded bytes without presuming a
    // compression ratio or treating source bytes as delivered bytes.
    let reference_url=format!("http://1.2.3.4/{}/reference.png",args.size);
    let person=&state.people[0];
    let signed=person.client.post(format!("{}/api/rc/sign",state.base)).json(&json!({"emailId":person.message,"urls":[reference_url],"parts":[]})).send().await.unwrap().json::<Value>().await.unwrap();
    let response=reqwest::Client::new().get(format!("{}{}",state.base,image_src(&signed))).header("user-agent",BROWSER).send().await.unwrap();
    assert_eq!(response.status(),200);let bytes=response.bytes().await.unwrap();
    let expected=Sha256::digest(&bytes).iter().map(|b|format!("{b:02x}")).collect::<String>();
    if ["ram","disk"].contains(&args.case.as_str()) {
        for path in &paths {
            let r=reqwest::Client::new().get(format!("{}{}",state.base,path["path"].as_str().unwrap())).header("user-agent",BROWSER).send().await.unwrap();
            assert_eq!(r.status(),200);assert_eq!(r.bytes().await.unwrap(),bytes);
        }
    }
    axum::Json(json!({"schema":1,"case":args.case,"size":args.size,"paths":paths,"expectedSha256":expected,"expectedBytes":bytes.len(),"sourceAccounts":PEOPLE,"sourceSessions":PEOPLE,"originDelayMs":20})).into_response()
}

#[test]
#[ignore = "private real-image throughput fixture; requires explicit opt-in and external driver"]
fn image_proxy_external_benchmark() {
    assert_eq!(std::env::var("MW_RUN_IMAGE_PROXY_BENCHMARK").as_deref(),Ok("1"),"explicit fixture opt-in required");
    let token=strict_token();
    let handoff=PathBuf::from(std::env::var_os("MW_IMAGE_BENCH_HANDOFF").expect("private handoff path"));
    assert!(handoff.is_absolute());
    std::thread::Builder::new().stack_size(32<<20).spawn(move||{
        tokio::runtime::Builder::new_multi_thread().worker_threads(4).thread_stack_size(32<<20).enable_all().build().unwrap().block_on(run(token,handoff));
    }).unwrap().join().unwrap();
}
async fn run(token:String,handoff:PathBuf) {
    let encryption=mw_store::encryption::global().expect("database encryption configuration");
    assert_eq!(encryption.mode,mw_store::encryption::EncryptionMode::Required,"private image benchmark requires encrypted Store");
    let directory=test_db::unique_dir("mw-image-benchmark");let db=directory.join("mw.db").to_string_lossy().into_owned();
    let store=store_at(&db).await;enforced::install(&store).await;
    for (key,value) in [
        ("privacy.imageCacheMode",json!("manual")),("privacy.cacheBytes",json!(8<<20)),
        ("privacy.cacheEntries",json!(1024)),("privacy.imageDiskEnabled",json!(true)),
        ("privacy.imageDiskMode",json!("manual")),("privacy.imageDiskMaxMiB",json!(32)),
        ("privacy.imageDiskPath",json!(directory.join("image-cache").to_string_lossy())),
        ("privacy.imageDiskEntries",json!(1024)),("privacy.imageDiskWriteQueueMiB",json!(1)),
    ] { policy(&db,key,value).await; }
    // POSIX process record locks can be released by closing ANY descriptor for
    // this inode. Probe the header ONLY after draining the entire initial pool,
    // before app/background/listener ownership exists, then reopen normally.
    store.close().await;
    drop(store);
    assert_eq!(mw_store::encryption::probe_file(std::path::Path::new(&db)).unwrap(),mw_store::encryption::FileState::NotPlaintext);
    let store=store_at(&db).await;
    let mut tasks=Tasks(Vec::new());
    let mock_listener=TcpListener::bind("127.0.0.1:0").await.unwrap();let mock=format!("http://{}",mock_listener.local_addr().unwrap());
    MOCK_UPSTREAMS.lock().unwrap().push(mock.clone());
    tasks.0.push(tokio::spawn(async move {axum::serve(mock_listener,mw_mock_jmap::router().layer(axum::middleware::from_fn(mock_identity))).await.unwrap();}));
    let origin_listener=TcpListener::bind("127.0.0.1:0").await.unwrap();let proxy=origin_listener.local_addr().unwrap();let fetches=Arc::new(AtomicU64::new(0));
    tasks.0.push(tokio::spawn(origin(origin_listener,fetches.clone())));
    let web=directory.join("web");std::fs::create_dir_all(&web).unwrap();std::fs::write(web.join("index.html"),INDEX_HTML).unwrap();
    let (app,_)=build_app_full(AppConfig{db_path:db.clone(),server_key_hex:Some(KEY_HEX.into()),web_dir:Some(web),cookie_secure:false,mode:mw_server::ServerMode::Proxy,hardening:Default::default(),security:mw_server::SecurityConfig{jmap_upstreams:Some(vec![mock.clone()]),upstream_default:Some(mock),..Default::default()}},V6Config{admin_enabled:true,admin_username:Some("root".into()),admin_password:Some("rc-fixture-password".into()),redis_url:None}).await.unwrap();
    let app_listener=TcpListener::bind("127.0.0.1:0").await.unwrap();let base=format!("http://{}",app_listener.local_addr().unwrap());
    tasks.0.push(tokio::spawn(async move {axum::serve(app_listener,app.into_make_service_with_connect_info::<SocketAddr>()).await.unwrap();}));
    let mut people=Vec::new();
    for i in 0..PEOPLE {
        let client=reqwest::Client::builder().cookie_store(true).no_proxy().local_address(std::net::IpAddr::V4(std::net::Ipv4Addr::new(127,0,0,(i+2) as u8))).user_agent(BROWSER).timeout(Duration::from_secs(10)).build().unwrap();
        let response=client.post(format!("{base}/api/login")).json(&json!({"username":format!("image{i}@example.org"),"password":"image-benchmark-password"})).send().await.unwrap();
        assert_eq!(response.status(),200);let login=response.json::<Value>().await.unwrap();assert_eq!(login["ok"],true);
        let account=login["accountId"].as_str().unwrap().to_string();assert!(!people.iter().any(|p:&Person|p.account==account));
        // create_account/upsert_account_by_identity mint their OWN random ID;
        // this proxy fixture must retain the account ID actually accepted by
        // upstream login. Use the SAME global SQLCipher configuration as Store,
        // with its key pragma installed FIRST on every raw connection.
        let path=std::path::Path::new(&db);
        // Required configuration was already validated by Store::open. Do NOT
        // probe/open/close an unrelated file descriptor while SQLite is live.
        let key=Some(encryption.key.as_ref().expect("required database key"));
        let raw=sqlx::SqlitePool::connect_with(mw_store::encryption::connect_options(path,key,false)).await.unwrap();
        sqlx::query("INSERT INTO accounts (id,kind,host,port,tls,username,sealed_creds,sync_policy_json) VALUES (?1,'imap','mail.example',993,1,?1,X'00','{}')").bind(&account).execute(&raw).await.unwrap();raw.close().await;
        let mailbox=store.upsert_mailbox(&mw_store::MailboxUpsert{account_id:&account,name:"INBOX",role:Some("inbox"),uidvalidity:1,uidnext:2,highestmodseq:1,total:1,unread:0,parent_id:None}).await.unwrap();
        let message=store.upsert_message(&mw_store::MessageUpsert{account_id:&account,mailbox_id:&mailbox,uid:1,uidvalidity:1,message_id:Some("<image-benchmark@example.org>"),thread_id:None,internaldate:Some("2026-11-22T12:00:00Z"),size:42,flags_json:"[]",envelope:None,blob_ref:None}).await.unwrap();
        store.grant_remote_image(&account,"all","").await.unwrap();people.push(Person{client,account,message});
    }
    tokio::time::sleep(Duration::from_secs(3)).await;
    let admin=reqwest::Client::builder().cookie_store(true).no_proxy().user_agent(BROWSER).timeout(Duration::from_secs(10)).build().unwrap();
    for path in ["/admin/login","/admin/step-up"] {
        let r=admin.post(format!("{base}{path}")).json(&json!({"username":"root","password":"rc-fixture-password"})).send().await.unwrap();assert_eq!(r.status(),200);let _=r.bytes().await.unwrap();
    }
    for (path,body) in [("/admin/egress/proxies",Some(json!({"id":"image-benchmark","scheme":"http","host":proxy.ip().to_string(),"port":proxy.port(),"allowPlaintext":true}))),("/admin/egress/proxies/image-benchmark/activate",None)] {
        let mut r=admin.post(format!("{base}{path}"));if let Some(body)=body {r=r.json(&body);}assert_eq!(r.send().await.unwrap().status(),200);
    }
    assert_eq!(mw_server::fixture_profile::protection_policy_witness(&store).await.unwrap()["defaultsExceptModes"],true);
    // All negative controls precede measurement and must not reach the origin.
    let anonymous=reqwest::Client::builder().no_proxy().user_agent(BROWSER).build().unwrap();
    assert_eq!(anonymous.get(format!("{base}/api/me")).send().await.unwrap().status(),401);
    let control_sign=people[0].client.post(format!("{base}/api/rc/sign")).json(&json!({"emailId":people[0].message,"urls":["http://1.2.3.4/control.png"],"parts":[]})).send().await.unwrap();
    assert_eq!(control_sign.status(),200);
    let control_sign=control_sign.json::<Value>().await.unwrap();
    let signed_path=image_src(&control_sign);
    assert!(mw_store::rc_handles::is_handle(&token_of(&signed_path)),"actual issued opaque capability");
    let mut tampered=signed_path.into_bytes();let start=tampered.iter().rposition(|b|*b==b'/').expect("capability path")+1;
    tampered[start]=if tampered[start]==b'A'{b'B'}else{b'A'};
    assert_eq!(anonymous.get(format!("{base}{}",String::from_utf8(tampered).unwrap())).send().await.unwrap().status(),403);
    let refused=people[0].client.get(format!("{base}/api/image-proxy")).query(&[("url","http://127.0.0.1/private.png"),("emailId",people[0].message.as_str())]).send().await.unwrap();
    assert_eq!(refused.status(),403);assert_eq!(fetches.load(Ordering::Relaxed),0);
    // Positive proof for EVERY independently authenticated principal and its
    // actual persisted grant, outside all measured case windows.
    for (i,person) in people.iter().enumerate() {
        let signed=person.client.post(format!("{base}/api/rc/sign")).json(&json!({"emailId":person.message,"urls":[format!("http://1.2.3.4/control/{i}.png")],"parts":[]})).send().await.unwrap();
        assert_eq!(signed.status(),200);let signed=signed.json::<Value>().await.unwrap();
        let response=anonymous.get(format!("{base}{}",image_src(&signed))).send().await.unwrap();assert_eq!(response.status(),200);
        assert!(response.bytes().await.unwrap().starts_with(b"\x89PNG\r\n\x1a\n"));
    }
    assert_eq!(fetches.load(Ordering::Relaxed),PEOPLE as u64);
    let state=Arc::new(Bench{base:base.clone(),db,admin,people,fetches,token,sequence:AtomicU64::new(0)});
    let listener=TcpListener::bind("127.0.0.1:0").await.unwrap();let control=format!("http://{}",listener.local_addr().unwrap());
    let stop=Arc::new(tokio::sync::Notify::new());let notify=stop.clone();let control_token=state.token.clone();
    let router=Router::new().route("/prepare",post(prepare)).route("/counters",get(counters)).route("/stop",post(move|headers:axum::http::HeaderMap|{let notify=notify.clone();let token=control_token.clone();async move {if headers.get("x-mw-image-bench-token").and_then(|v|v.to_str().ok())!=Some(token.as_str()){return StatusCode::UNAUTHORIZED;}notify.notify_one();StatusCode::NO_CONTENT}})).with_state(state);
    tasks.0.push(tokio::spawn(async move {axum::serve(listener,router).await.unwrap();}));
    let mut options=std::fs::OpenOptions::new();options.write(true).create_new(true);
    #[cfg(unix)] {use std::os::unix::fs::OpenOptionsExt;options.mode(0o600);}
    let mut file=options.open(&handoff).unwrap();use std::io::Write;
    file.write_all(serde_json::to_string(&json!({"schema":1,"kind":"real-signed-image-proxy","base":base,"control":control,"sourceAccounts":PEOPLE,"sourceSessions":PEOPLE,"lifetimeSecs":DEADLINE,"security":"enforced-default-limits","storeEncryption":"required","storeFileNotPlaintext":true,"origin":"local-configured-connect-standin","originDelayMs":20,"controls":{"noSession":401,"tamperedToken":403,"privateTarget":403,"originFetches":0,"positiveSignedAccounts":PEOPLE,"positiveSignedStatus":200},"sourceDiversity":"sixteen-real-loopback-peer-addresses; no-external-source-diversity"})).unwrap().as_bytes()).unwrap();file.sync_all().unwrap();
    println!("MW_IMAGE_BENCH_READY");
    tokio::select!{_=stop.notified()=>{},_=tokio::time::sleep(Duration::from_secs(DEADLINE))=>{panic!("image benchmark deadline expired")}}
    drop(tasks); // Original owned handles only. No unrelated process control.
}
