//! Real original-router cases. Futures are created inside an owned 16MiB
//! thread; these are source-authored until the coordinator grants execution.
use super::*;
use axum::http::HeaderValue;
use std::{cell::RefCell, future::Future};
fn run<F, Fut>(factory: F)
where
    F: FnOnce() -> Fut + Send + 'static,
    Fut: Future<Output = ()>,
{
    let thread = std::thread::Builder::new()
        .stack_size(16 * 1024 * 1024)
        .spawn(move || {
            tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()
                .unwrap()
                .block_on(factory())
        })
        .unwrap();
    if let Err(panic) = thread.join() {
        std::panic::resume_unwind(panic);
    }
}
struct Fixture {
    state: AppState,
    router: Router,
    token: String,
    headers: HeaderMap,
}
impl Fixture {
    async fn new() -> Self {
        let path = std::env::temp_dir().join(format!("mw-ws-action-{}.db", uuid::Uuid::new_v4()));
        let (router, state) = test_control::STATE
            .scope(RefCell::new(None), async {
                let (router, _push) = crate::build_app_full(
                    crate::AppConfig {
                        db_path: path.to_string_lossy().into_owned(),
                        server_key_hex: Some(mw_store::ServerKey::generate().to_hex()),
                        web_dir: None,
                        cookie_secure: false,
                        mode: crate::ServerMode::Engine,
                        hardening: Default::default(),
                        security: Default::default(),
                    },
                    crate::V6Config {
                        admin_enabled: true,
                        ..Default::default()
                    },
                )
                .await
                .unwrap();
                let state = test_control::STATE
                    .with(|slot| slot.borrow_mut().take().expect("actual built state"));
                (router, state)
            })
            .await;
        crate::admin_roles::ensure_builtin_roles(&state)
            .await
            .unwrap();
        let now = chrono::Utc::now().timestamp();
        state
            .store
            .insert_admin_account_with_roles(
                &mw_store::AdminAccountRow {
                    id: uuid::Uuid::new_v4().to_string(),
                    username: "ws-admin".into(),
                    display_name: "Fixture".into(),
                    password_hash: None,
                    disabled: false,
                    source: "db".into(),
                    created_at: now,
                    updated_at: now,
                    last_login_at: None,
                },
                &[mw_store::AdminRoleGrant {
                    role_id: crate::capabilities::SUPER_ADMIN.into(),
                    domains: None,
                }],
                false,
            )
            .await
            .unwrap();
        let token = mw_store::ServerKey::generate().to_hex();
        state
            .store
            .put_admin_session_with_timeouts(
                &crate::push_relay::hash_token(&token),
                "ws-admin",
                now,
                7200,
                86400,
            )
            .await
            .unwrap();
        for (key, value) in [
            ("realtime.actions.enabled", "true"),
            ("admin.session.rotateSecs", "0"),
        ] {
            policy(&state, key, value).await;
        }
        let mut headers = HeaderMap::new();
        headers.insert("host", "localhost".parse().unwrap());
        headers.insert("origin", "http://localhost".parse().unwrap());
        let csrf = state.store.session_csrf_token(&token);
        headers.insert(
            "cookie",
            format!(
                "{}={token}; mw_csrf={csrf}",
                mw_admin::config::session_cookie_name()
            )
            .parse()
            .unwrap(),
        );
        headers.insert("x-csrf-token", csrf.parse().unwrap());
        Self {
            state,
            router,
            token,
            headers,
        }
    }
    async fn actor(&self) -> Actor {
        Actor::Admin(cookie_delivery::AdminActor::new(
            &AdminCtx::resolve(&self.state, &self.headers).await.unwrap(),
        ))
    }
    fn context(&self) -> Context {
        Context {
            headers: self.headers.clone(),
            transport: None,
            dispatcher: Some(Dispatcher(self.router.clone())),
        }
    }
    async fn action(&self, method: &str, suffix: &str) -> Value {
        let slot = self.state.websocket_actions.dispatch_slot().unwrap();
        FRAME
            .scope(
                self.actor().await,
                execute(
                    self.state.clone(),
                    self.context(),
                    Dispatcher(self.router.clone()),
                    Action {
                        id: "fixed".into(),
                        method: method.into(),
                        path: format!("{}{suffix}", mw_admin::config::admin_path()),
                        headers: HashMap::new(),
                        body: None,
                    },
                    Arc::new(AtomicBool::new(false)),
                    Some(slot),
                ),
            )
            .await
    }
    async fn delivery(&self, token: &str, headers: HeaderMap, query: bool) -> Response {
        let mut request = Request::builder()
            .method("POST")
            .uri(format!(
                "{}{}{}",
                mw_admin::config::admin_path(),
                contract::get().admin.cookie_delivery_suffix,
                if query {
                    "?next=https://invalid.test"
                } else {
                    ""
                }
            ))
            .body(Body::from(json!({"token":token}).to_string()))
            .unwrap();
        *request.headers_mut() = headers;
        request
            .headers_mut()
            .insert(header::CONTENT_TYPE, "application/json".parse().unwrap());
        self.router.clone().oneshot(request).await.unwrap()
    }
}
async fn policy(state: &AppState, key: &str, value: &str) {
    state
        .store
        .put_setting_policy(&mw_store::settings_policy::SettingPolicyRow {
            key: key.into(),
            global_value: Some(value.into()),
            allow_user_override: false,
            global_locked: true,
            updated_by: "ws-test".into(),
            updated_at: chrono::Utc::now().timestamp(),
        })
        .await
        .unwrap();
}
#[test]
fn contract_routes_percent_queries_headers_and_actor_boundaries() {
    let base = mw_admin::config::admin_path();
    assert!(contract::target("/api/settings?keys=mail%2Eheaders%2Cworkspace%2Etabs").is_some());
    assert!(route_allowed(
        &format!("{base}/settings?x=1").split('?').next().unwrap(),
        "GET",
        true
    ));
    for value in [
        "//outside.test/api/settings",
        "/api/%2e%2e/admin",
        "/api/%252e%252e/admin",
        "/api/settings%2flogin",
        "/api/settings?q=%0a",
        "/api/settings?bad=%ff",
        "https://invalid.test/api/settings",
    ] {
        assert!(contract::target(value).is_none(), "invalid target");
    }
    assert!(!contract::webmail_path(&format!("{base}/settings"), &base));
    assert!(!route_allowed(&format!("{base}/login"), "POST", true));
    assert!(!route_allowed(
        &format!("{base}/actions/cookies"),
        "POST",
        true
    ));
    assert!(!route_allowed("/api/me", "GET", false));
    assert!(route_allowed("/api/settings/reset", "POST", false));
    assert!(route_allowed("/jmap/session", "GET", false));
}
#[test]
fn actual_admin_json_needs_no_webmail_cookie_and_revocation_is_fresh() {
    run(|| async {
        let fixture = Fixture::new().await;
        let reply = fixture.action("GET", "/capabilities").await;
        assert_eq!(reply["status"], 200);
        assert_eq!(reply["actionOutcome"], "completed");
        assert!(
            serde_json::from_str::<Value>(reply["body"].as_str().unwrap()).unwrap()["capabilities"]
                .is_array()
        );
        let actor = fixture.actor().await;
        fixture
            .state
            .store
            .delete_admin_sessions_of("ws-admin")
            .await
            .unwrap();
        let slot = fixture.state.websocket_actions.dispatch_slot().unwrap();
        let refused = FRAME
            .scope(
                actor,
                execute(
                    fixture.state.clone(),
                    fixture.context(),
                    Dispatcher(fixture.router.clone()),
                    Action {
                        id: "revoked".into(),
                        method: "GET".into(),
                        path: format!("{}/capabilities", mw_admin::config::admin_path()),
                        headers: HashMap::new(),
                        body: None,
                    },
                    Arc::new(AtomicBool::new(false)),
                    Some(slot),
                ),
            )
            .await;
        assert_eq!(refused["actionOutcome"], "notDispatched");
        assert_ne!(refused["status"], 200);
    });
}
#[test]
fn rotation_crosses_actual_handler_await_delivers_after_disabling_ws_and_never_replays() {
    run(|| async {
        let fixture = Fixture::new().await;
        policy(&fixture.state, "admin.session.rotateSecs", "60").await;
        let pause = Arc::new(test_control::Pause {
            entered: tokio::sync::Notify::new(),
            release: tokio::sync::Notify::new(),
            disable: true,
            calls: std::sync::atomic::AtomicUsize::new(0),
        });
        let future =
            test_control::PAUSE.scope(pause.clone(), fixture.action("POST", "/actions-test/pause"));
        tokio::pin!(future);
        tokio::select! {_=pause.entered.notified()=>{},_=&mut future=>panic!("handler must actually pause")}
        crate::admin_rotation::force_due_for_test(&crate::push_relay::hash_token(&fixture.token));
        pause.release.notify_one();
        let reply = future.await;
        assert_eq!(reply["status"], 200);
        assert_eq!(reply["actionOutcome"], "completed");
        assert!(serde_json::from_str::<Value>(reply["body"].as_str().unwrap()).unwrap()["actionApplied"].as_bool().unwrap());
        let token = reply["cookieDelivery"]["token"]
            .as_str()
            .expect("typed rotation handoff");
        assert!(reply["cookieDelivery"]["expiresInMs"].as_u64().unwrap() <= 30_000);
        assert_eq!(pause.calls.load(Ordering::SeqCst), 1);
        assert!(
            fixture
                .state
                .store
                .peek_admin_session(
                    &crate::push_relay::hash_token(&fixture.token),
                    chrono::Utc::now().timestamp()
                )
                .await
                .unwrap()
                .is_none()
        );
        // Disabled action policy is NOT a veto on completing already-issued cookies.
        let first = fixture
            .delivery(token, fixture.headers.clone(), false)
            .await;
        assert_eq!(first.status(), StatusCode::OK);
        assert_eq!(
            first.headers().get_all(header::SET_COOKIE).iter().count(),
            2
        );
        let cookies = first
            .headers()
            .get_all(header::SET_COOKIE)
            .iter()
            .map(|h| h.to_str().unwrap().split(';').next().unwrap())
            .collect::<Vec<_>>()
            .join("; ");
        let csrf = cookies
            .split(';')
            .find_map(|s| s.trim().strip_prefix("mw_csrf="))
            .unwrap();
        let mut issued = fixture.headers.clone();
        issued.insert(header::COOKIE, cookies.parse().unwrap());
        issued.insert("x-csrf-token", csrf.parse().unwrap());
        let again = fixture.delivery(token, issued.clone(), false).await;
        assert_eq!(again.status(), StatusCode::OK);
        assert_eq!(pause.calls.load(Ordering::SeqCst), 1);
        assert_ne!(
            fixture.delivery(token, issued.clone(), true).await.status(),
            StatusCode::OK
        );
        let mut foreign = issued.clone();
        foreign.insert("origin", "http://outside.test".parse().unwrap());
        assert_ne!(
            fixture.delivery(token, foreign, false).await.status(),
            StatusCode::OK
        );
        let mut badcsrf = issued.clone();
        badcsrf.insert("x-csrf-token", "wrong".parse().unwrap());
        assert_ne!(
            fixture.delivery(token, badcsrf, false).await.status(),
            StatusCode::OK
        );
        fixture
            .state
            .store
            .delete_admin_sessions_of("ws-admin")
            .await
            .unwrap();
        assert_ne!(
            fixture.delivery(token, issued, false).await.status(),
            StatusCode::OK
        );
        assert_eq!(pause.calls.load(Ordering::SeqCst), 1);
    });
}
#[test]
fn bounded_reservations_actor_parallelism_and_unknown_cookie_markers_fail_closed() {
    run(|| async {
        let fixture = Fixture::new().await;
        let ctx = AdminCtx::resolve(&fixture.state, &fixture.headers)
            .await
            .unwrap();
        let retired = Arc::new(AtomicBool::new(false));
        let ticket = fixture
            .state
            .websocket_actions
            .reserve(&ctx, "http://localhost", retired.clone())
            .unwrap();
        assert!(
            fixture
                .state
                .websocket_actions
                .reserve(&ctx, "http://localhost", retired)
                .is_none()
        );
        let mut slots = Vec::new();
        for _ in 0..contract::get().cookie_handoff.capacity {
            slots.push(fixture.state.websocket_actions.dispatch_slot().unwrap());
        }
        assert!(fixture.state.websocket_actions.dispatch_slot().is_none());
        drop(slots);
        drop(ticket);
        let mut response = StatusCode::OK.into_response();
        response.headers_mut().append(
            header::SET_COOKIE,
            HeaderValue::from_static("arbitrary=secret; Path=/"),
        );
        assert!(!cookie_delivery::issued_cookies_match(&response));
        let failure = completed_failure("fixed", true);
        assert_eq!(failure["actionOutcome"], "completed");
        assert!(failure.get("httpRequired").is_none());
        assert!(fixture.state.websocket_actions.shutdown().await);
        assert!(fixture.state.websocket_actions.dispatch_slot().is_none());
    });
}
#[test]
fn http_cookie_issuer_inventory_is_explicit_and_cannot_be_adopted_as_a_frame() {
    // Every known normal HttpOnly issuer must be rejected BEFORE dispatch.
    for (method, path) in [
        ("GET", "/api/me"),
        ("POST", "/api/login"),
        ("POST", "/api/logout"),
        ("POST", "/api/session/rotate"),
        ("POST", "/api/account/session/activity"),
        ("POST", "/api/account/security/remembered-devices"),
        ("POST", "/api/sso/complete"),
    ] {
        assert!(
            !route_allowed(path, method, false),
            "issuer must remain an explicit HTTP control"
        );
    }
    let base = mw_admin::config::admin_path();
    for path in [
        "/login",
        "/login/totp",
        "/logout",
        "/session",
        "/step-up",
        "/actions/cookies",
    ] {
        assert!(!route_allowed(&format!("{base}{path}"), "POST", true));
    }
}

#[test]
fn receiver_cancellation_keeps_bounded_owned_action_and_rotation_receipt() {
    run(|| async {
        let fixture = Fixture::new().await;
        policy(&fixture.state, "admin.session.rotateSecs", "60").await;
        crate::admin_rotation::force_due_for_test(&crate::push_relay::hash_token(&fixture.token));
        let pause = Arc::new(test_control::Pause {
            entered: tokio::sync::Notify::new(),
            release: tokio::sync::Notify::new(),
            disable: false,
            calls: std::sync::atomic::AtomicUsize::new(0),
        });
        let actor = fixture.actor().await;
        let slot = fixture.state.websocket_actions.dispatch_slot().unwrap();
        let actual = execute(
            fixture.state.clone(),
            fixture.context(),
            Dispatcher(fixture.router.clone()),
            Action {
                id: "cancelled-receiver".into(),
                method: "POST".into(),
                path: format!("{}/actions-test/pause", mw_admin::config::admin_path()),
                headers: HashMap::new(),
                body: None,
            },
            Arc::new(AtomicBool::new(false)),
            Some(slot),
        );
        let wrapper = tokio::spawn(owned_admin(
            actor,
            "cancelled-receiver".into(),
            test_control::PAUSE.scope(pause.clone(), actual),
        ));
        tokio::time::timeout(std::time::Duration::from_secs(5), pause.entered.notified())
            .await
            .unwrap();
        wrapper.abort();
        assert!(wrapper.await.unwrap_err().is_cancelled());
        pause.release.notify_one();
        // Shutdown is a real bounded wait for the detached owned handler, not a
        // claim that dropping its receiver cancelled the SQL mutation/rotation.
        let until = tokio::time::Instant::now() + std::time::Duration::from_secs(5);
        loop {
            let old = fixture
                .state
                .store
                .peek_admin_session(
                    &crate::push_relay::hash_token(&fixture.token),
                    chrono::Utc::now().timestamp(),
                )
                .await
                .unwrap();
            if old.is_none() {
                break;
            }
            assert!(tokio::time::Instant::now() < until);
            tokio::task::yield_now().await;
        }
        assert!(fixture.state.websocket_actions.shutdown().await);
        assert_eq!(pause.calls.load(Ordering::SeqCst), 1);
        assert!(
            fixture
                .state
                .store
                .peek_admin_session(
                    &crate::push_relay::hash_token(&fixture.token),
                    chrono::Utc::now().timestamp()
                )
                .await
                .unwrap()
                .is_none()
        );
    });
}
#[test]
fn real_admin_websocket_get_uses_only_admin_cookie() {
    run(|| async {
        use futures_util::{SinkExt, StreamExt};
        use tokio_tungstenite::tungstenite::client::IntoClientRequest;
        let fixture = Fixture::new().await;
        let listener = tokio::net::TcpListener::bind((std::net::Ipv4Addr::LOCALHOST, 0))
            .await
            .unwrap();
        let address = listener.local_addr().unwrap();
        let app = fixture.router.clone();
        let server = tokio::spawn(async move {
            axum::serve(
                listener,
                app.into_make_service_with_connect_info::<std::net::SocketAddr>(),
            )
            .await
            .unwrap();
        });
        struct Stop(tokio::task::JoinHandle<()>);
        impl Drop for Stop {
            fn drop(&mut self) {
                self.0.abort();
            }
        }
        let _stop = Stop(server);
        let mut request = format!("ws://{address}{}/actions", mw_admin::config::admin_path())
            .into_client_request()
            .unwrap();
        request
            .headers_mut()
            .insert(header::ORIGIN, format!("http://{address}").parse().unwrap());
        request
            .headers_mut()
            .insert(header::COOKIE, fixture.headers[header::COOKIE].clone());
        let (mut socket, _) = tokio_tungstenite::connect_async(request).await.unwrap();
        let ready = socket.next().await.unwrap().unwrap().into_text().unwrap();
        assert_eq!(
            serde_json::from_str::<Value>(&ready).unwrap()["type"],
            "ready"
        );
        socket.send(tokio_tungstenite::tungstenite::Message::Text(json!({"id":"actual-ws","method":"GET","path":format!("{}/capabilities",mw_admin::config::admin_path())}).to_string().into())).await.unwrap();
        let response = tokio::time::timeout(std::time::Duration::from_secs(5), socket.next())
            .await
            .unwrap()
            .unwrap()
            .unwrap()
            .into_text()
            .unwrap();
        let response: Value = serde_json::from_str(&response).unwrap();
        assert_eq!(response["actionOutcome"], "completed");
        assert_eq!(response["status"], 200);
        assert!(response.get("cookieDelivery").is_none());
        socket.close(None).await.unwrap();
    });
}

#[test]
fn cookie_receipt_expiry_changed_grants_and_cross_session_frames_are_not_authority() {
    run(|| async {
        let fixture = Fixture::new().await;
        let actor = fixture.actor().await;
        let other = mw_store::ServerKey::generate().to_hex();
        fixture
            .state
            .store
            .put_admin_session_with_timeouts(
                &crate::push_relay::hash_token(&other),
                "ws-admin",
                chrono::Utc::now().timestamp(),
                7200,
                86400,
            )
            .await
            .unwrap();
        let mut context = fixture.context();
        let csrf = fixture.state.store.session_csrf_token(&other);
        context.headers.insert(
            header::COOKIE,
            format!(
                "{}={other}; mw_csrf={csrf}",
                mw_admin::config::session_cookie_name()
            )
            .parse()
            .unwrap(),
        );
        context
            .headers
            .insert("x-csrf-token", csrf.parse().unwrap());
        let slot = fixture.state.websocket_actions.dispatch_slot().unwrap();
        let refused = FRAME
            .scope(
                actor,
                execute(
                    fixture.state.clone(),
                    context,
                    Dispatcher(fixture.router.clone()),
                    Action {
                        id: "different-session".into(),
                        method: "GET".into(),
                        path: format!("{}/capabilities", mw_admin::config::admin_path()),
                        headers: HashMap::new(),
                        body: None,
                    },
                    Arc::new(AtomicBool::new(false)),
                    Some(slot),
                ),
            )
            .await;
        assert_eq!(refused["actionOutcome"], "notDispatched");
        let ctx = AdminCtx::resolve(&fixture.state, &fixture.headers)
            .await
            .unwrap();
        let ticket = fixture
            .state
            .websocket_actions
            .reserve(&ctx, "http://localhost", Arc::new(AtomicBool::new(false)))
            .unwrap();
        let capture = ticket.capture();
        let (new, absolute) = crate::admin_rotation::rotate(&fixture.state, &fixture.token)
            .await
            .unwrap();
        assert!(
            capture
                .issued(&fixture.state, &fixture.headers, &new, absolute)
                .await
                .is_some()
        );
        let receipt = ticket.finish().unwrap();
        let token = receipt["token"].as_str().unwrap();
        let account = ctx.account_id.as_deref().unwrap();
        fixture
            .state
            .store
            .set_admin_account_roles(account, &[])
            .await
            .unwrap();
        let denied = fixture
            .delivery(token, fixture.headers.clone(), false)
            .await;
        assert_eq!(denied.status(), StatusCode::UNAUTHORIZED);
        assert!(!denied.headers().contains_key(header::SET_COOKIE));
        fixture
            .state
            .store
            .set_admin_account_roles(
                account,
                &[mw_store::AdminRoleGrant {
                    role_id: crate::capabilities::SUPER_ADMIN.into(),
                    domains: None,
                }],
            )
            .await
            .unwrap();
        fixture.state.websocket_actions.expire_for_test(token);
        let expired = fixture
            .delivery(token, fixture.headers.clone(), false)
            .await;
        assert_eq!(expired.status(), StatusCode::UNAUTHORIZED);
        assert!(!expired.headers().contains_key(header::SET_COOKIE));
    });
}

#[test]
fn header_deserializer_rejects_exact_and_case_folded_duplicates_before_dispatch() {
    for headers in [
        r#"{"x-csrf-token":"first","x-csrf-token":"second"}"#,
        r#"{"X-CSRF-Token":"first","x-csrf-token":"second"}"#,
    ] {
        let text = format!(
            r#"{{"id":"fixed","method":"GET","path":"/api/settings","headers":{headers}}}"#
        );
        assert!(serde_json::from_str::<Action>(&text).is_err());
    }
    let action: Action = serde_json::from_str(
        r#"{"id":"fixed","method":"GET","path":"/api/settings","headers":{"X-CSRF-Token":"only"}}"#,
    )
    .unwrap();
    assert_eq!(
        action.headers.get("x-csrf-token").map(String::as_str),
        Some("only")
    );
}

#[test]
fn shutdown_refuses_new_frames_even_when_waiting_drain_future_is_cancelled() {
    run(|| async {
        let service = cookie_delivery::Service::new();
        let held = service.dispatch_slot().unwrap();
        assert!(
            tokio::time::timeout(std::time::Duration::from_millis(25), service.shutdown())
                .await
                .is_err()
        );
        assert!(
            service.dispatch_slot().is_none(),
            "cancelled shutdown cannot reopen admission"
        );
        drop(held);
        assert!(service.shutdown().await);
    });
}

#[test]
fn real_webmail_duplicate_inflight_id_closes_without_false_predispatch_receipt() {
    run(|| async {
        use futures_util::{SinkExt, StreamExt};
        use mw_store::settings_policy::SettingScope;
        use tokio_tungstenite::tungstenite::{Message as WireMessage, client::IntoClientRequest};
        let fixture = Fixture::new().await;
        // The one-frame admin channel cannot expose this race: its receiver is
        // backpressured until the first task completes. Use a legitimate primary
        // webmail actor and its eight-frame receive window instead.
        let credentials = mw_store::Credentials {
            username: "wire-owner@socket.test".into(),
            password: "owned-fixture-not-an-upstream-secret".into(),
        };
        // A native session still traverses the ordinary domain routing guard.
        // Seed the same explicit operator-owned endpoint as its real account;
        // a verified binding alone is not an authorization/routing fixture.
        fixture.state.store.put_domain_config(&mw_store::DomainConfigRow {
            name:"socket.test".into(),
            upstream_json:json!({"imap":{"host":"127.0.0.1","port":9,"tls":"starttls","auth":"password","allowPrivate":true}}).to_string(),
            allowlist_json:"[]".into(),blocklist_json:"[]".into(),
            enabled:true,preferred:false,settings_json:"{}".into(),
            updated_at:chrono::Utc::now().timestamp(),
        }).await.unwrap();
        let account = fixture.state.store.create_account(
            &mw_store::NewAccount {
                kind: mw_store::AccountKind::Imap,
                host: "127.0.0.1", port: 9, tls: "starttls",
                username: &credentials.username, sync_policy_json: "{}",
            }, &credentials,
        ).await.unwrap();
        let session = fixture.state.store.create_session(
            &account, &credentials.username, "engine", "engine", &credentials,
        ).await.unwrap();
        fixture.state.store.set_session_authentication(&session, "password").await.unwrap();
        fixture.state.store.bind_verified_primary(
            &session,
            mw_store::identity_principal::VerifiedPrimary {
                account_id: &account, kind: "mailbox", authority: "wire-fixture", subject: &account,
            }, chrono::Utc::now().timestamp(),
        ).await.unwrap().unwrap();
        policy(&fixture.state, "access.protocols", r#"["imap"]"#).await;
        const KEY: &str = "mail.list.pageSize"; // User-scoped integer20..200, default50.
        assert!(fixture.state.store.get_setting_value(SettingScope::User, &account, KEY).await.unwrap().is_none());

        let listener = tokio::net::TcpListener::bind((std::net::Ipv4Addr::LOCALHOST, 0)).await.unwrap();
        let address = listener.local_addr().unwrap();
        let app = fixture.router.clone();
        struct Stop(tokio::task::JoinHandle<()>);
        impl Drop for Stop { fn drop(&mut self) { self.0.abort(); } }
        let _stop = Stop(tokio::spawn(async move {
            axum::serve(listener, app.into_make_service_with_connect_info::<std::net::SocketAddr>()).await.unwrap();
        }));
        let mut request = format!("ws://{address}{}/jmap/actions", fixture.state.base_path).into_client_request().unwrap();
        request.headers_mut().insert(header::ORIGIN, format!("http://{address}").parse().unwrap());
        let csrf = fixture.state.store.session_csrf_token(&session);
        request.headers_mut().insert(header::COOKIE, format!("mw_session={session}; mw_csrf={csrf}").parse().unwrap());
        // Diagnose the actual pre-upgrade requirements independently of open's
        // intentionally uniform403. Never print the cookie or subject source.
        assert!(crate::hardening::origin_ok(request.headers(),None),"wire fixture must carry matching actual Host/Origin");
        let authenticated=crate::authed(&fixture.state,request.headers()).await.unwrap_or_else(|response|
            panic!("wire fixture ordinary authentication preflight HTTP{}",response.status()));
        assert!(authenticated.account_id==account,"wire fixture must resolve its actual owned account");
        let subject=crate::settings_routes::UserSubject::resolved(&fixture.state.store,&authenticated).await
            .unwrap_or_else(|_|panic!("wire fixture requires a verified policy/storage subject"));
        assert!(subject.account_id==account && subject.domain=="socket.test","wire fixture subject must match configured native domain");
        assert!(crate::notification_policy::allowed(&fixture.state.store,&subject.as_subject(),"realtime.actions.enabled").await.unwrap_or(false),"wire fixture subject must have current realtime action policy");
        assert!(fixture.state.store.websocket_primary_actor(&session).await.unwrap().is_some(),"wire fixture must have a current primary socket actor");
        let (mut socket, _) = tokio::time::timeout(std::time::Duration::from_secs(5), tokio_tungstenite::connect_async(request)).await.unwrap().unwrap();
        let ready = tokio::time::timeout(std::time::Duration::from_secs(5), socket.next()).await.unwrap().unwrap().unwrap().into_text().unwrap();
        assert_eq!(serde_json::from_str::<Value>(&ready).unwrap()["type"], "ready");
        const ID: &str = "owned-duplicate-id";
        let lease = test_control::pause_wire(&fixture.state, ID);
        let frame = json!({"id":ID,"method":"PUT","path":format!("/api/settings/{KEY}"),"headers":{"content-type":"application/json","x-csrf-token":csrf},"body":"{\"value\":75}"});
        tokio::time::timeout(std::time::Duration::from_secs(5), socket.send(WireMessage::Text(frame.to_string().into()))).await.unwrap().unwrap();
        tokio::time::timeout(std::time::Duration::from_secs(5), async {
            tokio::select! {
                _ = lease.pause.entered.notified() => {},
                _ = socket.next() => panic!("real mutation must pause after successful dispatch, before any ACK"),
            }
        }).await.unwrap();
        let saved = fixture.state.store.get_setting_value(SettingScope::User, &account, KEY).await.unwrap().expect("real user setting persisted before duplicate");
        assert_eq!(serde_json::from_str::<Value>(&saved.value).unwrap(), json!(75));
        assert_eq!(lease.pause.calls.load(Ordering::SeqCst), 1);

        // Without the active-ID fence this control frame yields a false
        // notDispatched/httpRequired receipt for the already-applied mutation.
        let duplicate = json!({"id":ID,"method":"GET","path":"/api/me"});
        tokio::time::timeout(std::time::Duration::from_secs(5), socket.send(WireMessage::Text(duplicate.to_string().into()))).await.unwrap().unwrap();
        let next = tokio::time::timeout(std::time::Duration::from_secs(5), socket.next()).await.unwrap();
        assert!(matches!(next, Some(Ok(WireMessage::Close(_))) | None), "duplicate in-flight ID must close, never return a misleading Text ACK");
        assert_eq!(lease.pause.calls.load(Ordering::SeqCst), 1);
        let saved = fixture.state.store.get_setting_value(SettingScope::User, &account, KEY).await.unwrap().unwrap();
        assert_eq!(serde_json::from_str::<Value>(&saved.value).unwrap(), json!(75));
        // Close makes the first ACK unknown to the wire client; there is no
        // application replay here. Client no-replay handling is a separate gate.
        lease.pause.release.close();
        assert!(fixture.state.websocket_actions.shutdown().await, "closed receiver must release its real global dispatch slot");
        drop(lease); // RAII also releases/removes the bounded test-only gate.
    });
}
