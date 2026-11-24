//! Minimal, uncached configuration exposed before mailbox sign-in.
use crate::{AppState, domain_config, settings_registry};
use axum::{
    Json,
    extract::State,
    http::{HeaderMap, header},
    response::{IntoResponse, Response},
};
use serde_json::json;

pub(crate) async fn public_config(
    State(state): State<AppState>,
    headers: HeaderMap,
    ext: axum::http::Extensions,
) -> Response {
    let host = state.external.host(&headers, &ext);
    let identity = match domain_config::canonical(&state.store, "hint", host.as_deref()).await {
        Ok(identity) => identity,
        Err(_) => return axum::http::StatusCode::SERVICE_UNAVAILABLE.into_response(),
    };
    let subject = mw_store::settings_policy::SettingSubject {
        personal_owner: None,
        domain: &identity.domain,
        address: "",
        account_id: "",
    };
    let values = match settings_registry::effective_all(&state.store, &subject, |def| {
        matches!(
            def.key,
            "appearance.brandName"
                | "appearance.theme"
                | "appearance.accent"
                | "login.notice"
                | "login.showDomainHint"
                | "login.rememberSignins"
                | "login.showRememberedHint"
                | "login.showAccountHelpNotice"
                | "signin.retryCooldownSecs"
                | "web.i18n.retainedLocales"
                | "web.i18n.concurrentLoads"
                | "web.prefetch.modules"
                | "web.prefetch.enabled"
                | "web.prefetch.idleDelayMs"
                | "web.offline.precacheMode"
                | "web.offline.precacheConcurrency"
                | "web.offline.respectSaveData"
        )
    })
    .await
    {
        Ok(values) => values,
        Err(_) => return axum::http::StatusCode::SERVICE_UNAVAILABLE.into_response(),
    };
    let value = |key: &str| {
        values
            .iter()
            .find(|v| v.key == key)
            .map(|v| v.value.clone())
            .unwrap_or(serde_json::Value::Null)
    };
    let hint = if value("login.showDomainHint") != json!(true) || identity.domain.is_empty() {
        None
    } else {
        match state.store.domain_signin_state(&identity.domain).await {
            Ok(mw_store::DomainSigninState::Enabled) => Some(identity.domain),
            Ok(_) => None,
            Err(_) => return axum::http::StatusCode::SERVICE_UNAVAILABLE.into_response(),
        }
    };
    let branding = match state.store.branding_snapshot().await {
        Ok(branding) => crate::branding_routes::view(&branding),
        Err(_) => return axum::http::StatusCode::SERVICE_UNAVAILABLE.into_response(),
    };
    let anti_bot = match crate::signin_challenge::config(&state).await {
        Ok(config) => {
            // A source guessing many usernames, or any unknown source during an
            // instance-wide spike, is shown the challenge too.
            let forced = match crate::scope_mw::proxy::client_ip(&headers, &ext) {
                Some(ip) => crate::signin_challenge::forced_for_source(&state, ip).await,
                None => false,
            };
            json!({"mailbox":forced || config.enabled(crate::signin_challenge::Surface::Mailbox),"admin":forced || config.enabled(crate::signin_challenge::Surface::Admin)})
        }
        Err(_) => return axum::http::StatusCode::SERVICE_UNAVAILABLE.into_response(),
    };
    let maintenance = match crate::maintenance_mode::config(&state).await {
        Ok(config) => config,
        Err(_) => return axum::http::StatusCode::SERVICE_UNAVAILABLE.into_response(),
    };
    let mut response = Json(json!({
        // Public performance hints only: no authentication/security authority.
        "webPerformance": {
            "retainedLocales":value("web.i18n.retainedLocales"),
            "concurrentLoads":value("web.i18n.concurrentLoads"),
            "prefetchEnabled":value("web.prefetch.enabled"),
            "prefetchModules":value("web.prefetch.modules"),
            "idleDelayMs":value("web.prefetch.idleDelayMs"),
            "precacheMode":value("web.offline.precacheMode"),
            "precacheConcurrency":value("web.offline.precacheConcurrency"),
            "respectSaveData":value("web.offline.respectSaveData"),
        },
        "maintenance":maintenance,
        "antiBot":anti_bot,
        "retryCooldownSecs":value("signin.retryCooldownSecs"),
        "brandName":branding["brandName"], "theme":value("appearance.theme"),
        "accent":value("appearance.accent"), "loginNotice":branding["loginNotice"],
        "usernameDomainHint":hint, "showDomainHint":value("login.showDomainHint"),
        "rememberSignins":value("login.rememberSignins"),
        "showRememberedHint":value("login.showRememberedHint"),
        "showAccountHelpNotice":value("login.showAccountHelpNotice"),
        "logoUrl":branding["assets"]["logoUrl"], "faviconUrl":branding["assets"]["faviconUrl"],
        "loginBackgroundUrl":branding["assets"]["loginBackgroundUrl"],
    }))
    .into_response();
    response.headers_mut().insert(
        header::CACHE_CONTROL,
        axum::http::HeaderValue::from_static("no-store"),
    );
    response
}

#[cfg(test)]
mod web_performance_tests {
    use super::*;
    use axum::{body::{Body,to_bytes},http::Request};
    use mw_store::{Store,ServerKey,settings_policy::SettingPolicyRow};
    use tower::ServiceExt;
    #[test]
    fn public_performance_controls_follow_committed_admin_values_without_exposing_private_settings() {
        // Construct and poll build_app_full's debug future inside the bounded
        // fixture thread, rather than overflowing libtest's default stack.
        let thread=std::thread::Builder::new().stack_size(16*1024*1024).spawn(||{
            tokio::runtime::Builder::new_current_thread().enable_all().build().unwrap().block_on(async{
                let path=std::env::temp_dir().join(format!("mw-web-performance-{}.sqlite",uuid::Uuid::new_v4()));
                let key=ServerKey::generate();let store=Store::open(&path.to_string_lossy(),key.clone()).await.unwrap();
                let (router,_)=crate::build_app_full(crate::AppConfig{db_path:path.to_string_lossy().into_owned(),server_key_hex:Some(key.to_hex()),web_dir:None,cookie_secure:false,mode:crate::ServerMode::Proxy,hardening:crate::HardeningConfig::default(),security:crate::SecurityConfig::default()},crate::V6Config{admin_enabled:false,..Default::default()}).await.unwrap();
                let read=|router:axum::Router|async move{
                    let response=router.oneshot(Request::builder().uri("/api/public-config").header("host","localhost").body(Body::empty()).unwrap()).await.unwrap();
                    assert_eq!(response.status(),200);assert_eq!(response.headers()[header::CACHE_CONTROL],"no-store");
                    serde_json::from_slice::<serde_json::Value>(&to_bytes(response.into_body(),1024*1024).await.unwrap()).unwrap()
                };
                let before=read(router.clone()).await;assert_eq!(before["webPerformance"],json!({"retainedLocales":3,"concurrentLoads":4,"prefetchEnabled":true,"prefetchModules":false,"idleDelayMs":2500,"precacheMode":"app","precacheConcurrency":4,"respectSaveData":true}));
                for(key,value)in [("web.i18n.retainedLocales",json!(2)),("web.i18n.concurrentLoads",json!(1)),("web.prefetch.enabled",json!(false)),("web.prefetch.idleDelayMs",json!(800)),("web.offline.precacheMode",json!("core")),("web.offline.precacheConcurrency",json!(1)),("web.offline.respectSaveData",json!(false))]{
                    store.put_setting_policy(&SettingPolicyRow{key:key.into(),global_value:Some(value.to_string()),allow_user_override:false,global_locked:true,updated_by:"web-performance-fixture".into(),updated_at:chrono::Utc::now().timestamp()}).await.unwrap();
                }
                store.set_setting("web.private.not-public","must-not-leak").await.unwrap();
                let after=read(router).await;assert_eq!(after["webPerformance"],json!({"retainedLocales":2,"concurrentLoads":1,"prefetchEnabled":false,"prefetchModules":false,"idleDelayMs":800,"precacheMode":"core","precacheConcurrency":1,"respectSaveData":false}));
                assert!(!after.to_string().contains("must-not-leak"));
            });
        }).unwrap();
        if let Err(panic)=thread.join(){std::panic::resume_unwind(panic);}
    }
}
