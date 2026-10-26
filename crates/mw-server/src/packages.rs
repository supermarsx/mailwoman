//! Explicitly opted-in package tracking. No mailbox scanning or background export.
use crate::{AppState, admin_ctx::{AdminApiError,AdminCtx,AdminJson,check_if_match,etag_of,with_etag}, admin_router::AdminRegistrar, capabilities::Capability};
use crate::packages_model::{AccountPackages,Package,apply_provider,valid_tracking};
use axum::{Json,Router,extract::{Path,State},http::{HeaderMap,Method,StatusCode},response::{IntoResponse,Response},routing::{get,post,put}};
use mw_store::{Session,package_tracking::PackageProvider};
use serde::Deserialize;
use serde_json::{Value,json};

pub(crate) fn router()->Router<AppState> {
    Router::new().route("/api/account/packages",get(view).put(preferences).post(add))
        .route("/api/account/packages/{id}",put(edit).delete(remove))
        .route("/api/account/packages/{id}/refresh",post(refresh))
}
pub(crate) fn register(r:&mut impl AdminRegistrar) {
    r.admin_endpoint(Method::GET,"/package-tracking",Capability::IntegrationsRead,admin_get);
    r.admin_endpoint(Method::PUT,"/package-tracking",Capability::IntegrationsWrite,admin_put);
}
fn error(status:StatusCode,code:&str)->Response { (status,Json(json!({"code":code}))).into_response() }
fn unavailable()->Response { error(StatusCode::SERVICE_UNAVAILABLE,"packageUnavailable") }
fn stale()->Response { error(StatusCode::CONFLICT,"packageChanged") }
async fn admin_get(State(state):State<AppState>,ctx:AdminCtx)->Result<Response,AdminApiError> {
    ctx.require_unscoped(Capability::IntegrationsRead)?;
    let config=state.store.package_provider().await.map_err(AdminApiError::internal)?;
    Ok(with_etag(Json(json!({"provider":config})),&etag_of(&json!(config.as_ref().map(|c|&c.revision)))))
}
#[derive(Deserialize)]
#[serde(rename_all="camelCase",deny_unknown_fields)]
struct ProviderWrite { enabled:bool, refresh_secs:i64, api_key:Option<String> }
async fn admin_put(State(state):State<AppState>,ctx:AdminCtx,headers:HeaderMap,AdminJson(body):AdminJson<ProviderWrite>)->Result<Response,AdminApiError> {
    ctx.require_unscoped(Capability::IntegrationsWrite)?;
    crate::step_up::require_admin(&state,&ctx,crate::step_up::StepUpAction::IntegrationManage).await?;
    if !(60..=86400).contains(&body.refresh_secs) || body.api_key.as_ref().is_some_and(|key|key.is_empty()||key.len()>512||!key.bytes().all(|byte|byte.is_ascii_graphic())) {
        return Err(AdminApiError::invalid("invalidPackageProvider","Use a valid key and a refresh interval between 60 and 86400 seconds.","/apiKey"));
    }
    let old=state.store.package_provider().await.map_err(AdminApiError::internal)?;
    if !headers.contains_key("if-match") { return Err(AdminApiError::new(StatusCode::PRECONDITION_REQUIRED,"ifMatchRequired","Reload provider settings before saving.")); }
    check_if_match(&headers,&etag_of(&json!(old.as_ref().map(|c|&c.revision))),None,None)?;
    let saved=state.store.put_package_provider(body.enabled,body.refresh_secs,body.api_key.as_deref(),old.as_ref().map(|c|c.revision.as_str())).await.map_err(AdminApiError::internal)?
        .ok_or_else(||AdminApiError::invalid("packageProviderChanged","Reload settings and provide an API key for first setup.","/apiKey"))?;
    crate::admin_accounts::audit(&state,&ctx,"package-provider-configured","package-provider:aftership",None,Some(json!({"enabled":saved.enabled,"refreshSecs":saved.refresh_secs}))).await;
    Ok(with_etag(Json(json!({"provider":saved})),&etag_of(&json!(Some(&saved.revision)))))
}
async fn session(state:&AppState,headers:&HeaderMap,account:Option<&str>)->Result<Session,Response> {
    let session=crate::authed(state,headers).await?;
    if account.is_some_and(|account|account!=session.account_id) {return Err(stale());}
    Ok(session)
}
async fn load(state:&AppState,account:&str)->Result<(AccountPackages,Option<String>),Response> {
    match state.store.package_account(account).await.map_err(|_|unavailable())? {
        Some((raw,revision))=>Ok((serde_json::from_str(&raw).map_err(|_|unavailable())?,Some(revision))),
        None=>Ok((AccountPackages::default(),None)),
    }
}
async fn save(state:&AppState,account:&str,data:&AccountPackages,revision:Option<&str>)->Result<String,Response> {
    state.store.put_package_account(account,&serde_json::to_string(data).map_err(|_|unavailable())?,revision).await.map_err(|_|unavailable())?.ok_or_else(stale)
}
async fn permitted(state:&AppState,session:&Session)->Result<bool,Response> {
    let subject=crate::settings_routes::UserSubject::resolved(&state.store,session).await.map_err(|_|unavailable())?;
    Ok(crate::settings_registry::effective_for(&state.store,"packages.allowed",&subject.as_subject()).await.map_err(|_|unavailable())?.is_some_and(|entry|entry.value==json!(true)))
}
async fn view(State(state):State<AppState>,headers:HeaderMap)->Response {
    let result=async {
        let session=session(&state,&headers,None).await?;
        let (data,revision)=load(&state,&session.account_id).await?;
        let provider=state.store.package_provider().await.map_err(|_|unavailable())?;
        Ok::<_,Response>(Json(json!({"accountId":session.account_id,"state":data,"revision":revision,"available":permitted(&state,&session).await?&&provider.as_ref().is_some_and(|p|p.enabled),"refreshSecs":provider.map_or(900,|p|p.refresh_secs)})).into_response())
    }.await; result.unwrap_or_else(|e|e)
}
#[derive(Deserialize)]
#[serde(rename_all="camelCase",deny_unknown_fields)]
struct PreferenceWrite { account_id:String, revision:Option<String>, enabled:bool }
async fn preferences(State(state):State<AppState>,headers:HeaderMap,Json(body):Json<PreferenceWrite>)->Response {
    let result=async {
        let session=session(&state,&headers,Some(&body.account_id)).await?;
        let (mut data,revision)=load(&state,&session.account_id).await?;
        if revision!=body.revision {return Err(stale());}
        if body.enabled && !permitted(&state,&session).await? {return Err(error(StatusCode::FORBIDDEN,"packageDisabled"));}
        data.enabled=body.enabled;
        let revision=save(&state,&session.account_id,&data,revision.as_deref()).await?;
        Ok::<_,Response>(Json(json!({"revision":revision})).into_response())
    }.await;result.unwrap_or_else(|e|e)
}
#[derive(Deserialize)]
#[serde(rename_all="camelCase",deny_unknown_fields)]
struct PackageWrite { account_id:String, revision:Option<String>, label:String, tracking_number:String, carrier:String }
async fn modify(state:AppState,headers:HeaderMap,id:Option<String>,body:PackageWrite)->Response {
    let result=async {
        let session=session(&state,&headers,Some(&body.account_id)).await?;
        if !valid_tracking(&body.tracking_number,&body.carrier,&body.label) {return Err(error(StatusCode::UNPROCESSABLE_ENTITY,"invalidPackage"));}
        let (mut data,revision)=load(&state,&session.account_id).await?;
        if revision!=body.revision {return Err(stale());}
        if !data.enabled || !permitted(&state,&session).await? {return Err(error(StatusCode::FORBIDDEN,"packageDisabled"));}
        if let Some(id)=id {
            let package=data.packages.iter_mut().find(|row|row.id==id).ok_or_else(||error(StatusCode::NOT_FOUND,"packageNotFound"))?;
            if package.tracking_number!=body.tracking_number || package.carrier!=body.carrier {
                package.provider_id=None;package.provider_revision=None;package.status="Pending".into();package.checkpoints.clear();package.estimated_delivery=None;package.updated_at=None;
                // Preserve the last attempt to prevent edit/refresh rate-limit bypass.
            }
            package.label=body.label;package.tracking_number=body.tracking_number;package.carrier=body.carrier;
        } else {
            if data.packages.len()>=100 {return Err(error(StatusCode::UNPROCESSABLE_ENTITY,"packageLimit"));}
            if data.packages.iter().any(|row|row.tracking_number==body.tracking_number&&row.carrier==body.carrier) {return Err(error(StatusCode::CONFLICT,"packageDuplicate"));}
            data.packages.push(Package{id:uuid::Uuid::new_v4().to_string(),label:body.label,tracking_number:body.tracking_number,carrier:body.carrier,provider_id:None,provider_revision:None,status:"Pending".into(),estimated_delivery:None,checkpoints:vec![],attempted_at:None,updated_at:None});
        }
        let revision=save(&state,&session.account_id,&data,revision.as_deref()).await?;
        Ok::<_,Response>(Json(json!({"revision":revision})).into_response())
    }.await;result.unwrap_or_else(|e|e)
}
async fn add(State(state):State<AppState>,headers:HeaderMap,Json(body):Json<PackageWrite>)->Response {modify(state,headers,None,body).await}
async fn edit(State(state):State<AppState>,Path(id):Path<String>,headers:HeaderMap,Json(body):Json<PackageWrite>)->Response {modify(state,headers,Some(id),body).await}
#[derive(Deserialize)]
#[serde(rename_all="camelCase",deny_unknown_fields)]
struct Action { account_id:String,revision:String }
async fn remove(State(state):State<AppState>,Path(id):Path<String>,headers:HeaderMap,Json(body):Json<Action>)->Response {
    let result=async {
        let session=session(&state,&headers,Some(&body.account_id)).await?;
        let (mut data,revision)=load(&state,&session.account_id).await?;
        if revision.as_deref()!=Some(&body.revision) {return Err(stale());}
        data.packages.retain(|row|row.id!=id);
        let revision=save(&state,&session.account_id,&data,revision.as_deref()).await?;
        Ok::<_,Response>(Json(json!({"revision":revision})).into_response())
    }.await;result.unwrap_or_else(|e|e)
}
async fn provider_call(state:&AppState,session:&Session,provider:&PackageProvider,url:reqwest::Url,body:Option<&Value>)->Result<Value,Response> {
    // Every attempt, including a refused provider response, consumes pacing.
    // The outer operation has a single non-queued permit, so no unbounded waiters.
    tokio::time::sleep(std::time::Duration::from_millis(300)).await;
    if !permitted(state,session).await? {return Err(error(StatusCode::FORBIDDEN,"packageDisabled"));}
    if !load(state,&session.account_id).await?.0.enabled {return Err(error(StatusCode::FORBIDDEN,"packageDisabled"));}
    let key=state.store.package_provider_key(&provider.revision).await.map_err(|_|unavailable())?.ok_or_else(||error(StatusCode::FORBIDDEN,"packageDisabled"))?;
    let plan=crate::image_proxy::load_plan(&state.store).await.map_err(|_|unavailable())?;
    let bytes=body.map(serde_json::to_vec).transpose().map_err(|_|unavailable())?;
    let method=if body.is_some(){mw_egress::ApiMethod::Post}else{mw_egress::ApiMethod::Get};
    let request=mw_egress::plan::Request::package_api(method,&key,bytes.as_ref().map(|bytes|mw_egress::OutBody{content_type:"application/json",bytes})).map_err(|_|unavailable())?.response_limit(1024*1024);
    let subject=crate::settings_routes::UserSubject::resolved(&state.store,session).await.map_err(|_|unavailable())?;
    let request=request.for_domain(&subject.domain);
    let result=tokio::time::timeout(std::time::Duration::from_secs(15),mw_egress::plan::send_with(&plan,url,&request)).await.map_err(|_|error(StatusCode::BAD_GATEWAY,"packageProviderUnavailable"))?;
    let bytes=result.outcome.map_err(|_|error(StatusCode::BAD_GATEWAY,"packageProviderUnavailable"))?;
    serde_json::from_slice(&bytes).map_err(|_|error(StatusCode::BAD_GATEWAY,"packageProviderResponse"))
}
async fn refresh(State(state):State<AppState>,Path(id):Path<String>,headers:HeaderMap,Json(body):Json<Action>)->Response {
    let result=async {
        static SLOTS:std::sync::OnceLock<tokio::sync::Semaphore>=std::sync::OnceLock::new();
        let _permit=SLOTS.get_or_init(||tokio::sync::Semaphore::new(1)).try_acquire().map_err(|_|error(StatusCode::TOO_MANY_REQUESTS,"packageBusy"))?;
        let session=session(&state,&headers,Some(&body.account_id)).await?;
        let (mut data,revision)=load(&state,&session.account_id).await?;
        if revision.as_deref()!=Some(&body.revision) {return Err(stale());}
        if !data.enabled || !permitted(&state,&session).await? {return Err(error(StatusCode::FORBIDDEN,"packageDisabled"));}
        let provider=state.store.package_provider().await.map_err(|_|unavailable())?.filter(|p|p.enabled).ok_or_else(||error(StatusCode::FORBIDDEN,"packageDisabled"))?;
        let package=data.packages.iter_mut().find(|row|row.id==id).ok_or_else(||error(StatusCode::NOT_FOUND,"packageNotFound"))?;
        let now=chrono::Utc::now().timestamp();
        if package.attempted_at.is_some_and(|last|now.saturating_sub(last)<provider.refresh_secs) {return Err(error(StatusCode::TOO_MANY_REQUESTS,"packageCooldown"));}
        package.attempted_at=Some(now);
        let mut package=package.clone();
        let reserved=save(&state,&session.account_id,&data,revision.as_deref()).await?;
        let base="https://api.aftership.com/tracking/2026-07/trackings";
        let response=if package.provider_revision.as_deref()==Some(&provider.revision) && package.provider_id.is_some() {
            provider_call(&state,&session,&provider,reqwest::Url::parse(&format!("{base}/{}",package.provider_id.as_deref().unwrap())).map_err(|_|unavailable())?,None).await?["data"].clone()
        }else {
            // Lookup before create also recovers a registration whose response
            // was lost, without blindly retrying a non-idempotent POST.
            let mut url=reqwest::Url::parse(base).map_err(|_|unavailable())?;
            url.query_pairs_mut().append_pair("tracking_numbers",&package.tracking_number).append_pair("slug",&package.carrier).append_pair("limit","10");
            let response=provider_call(&state,&session,&provider,url,None).await?;
            let rows=response["data"]["trackings"].as_array().ok_or_else(||error(StatusCode::BAD_GATEWAY,"packageProviderResponse"))?;
            if let Some(row)=rows.iter().find(|row|row["tracking_number"]==package.tracking_number&&row["slug"]==package.carrier) {row.clone()} else {
                provider_call(&state,&session,&provider,reqwest::Url::parse(base).map_err(|_|unavailable())?,Some(&json!({"tracking_number":package.tracking_number,"slug":package.carrier}))).await?["data"].clone()
            }
        };
        apply_provider(&mut package,&response,&provider.revision,chrono::Utc::now().timestamp()).map_err(|_|error(StatusCode::BAD_GATEWAY,"packageProviderResponse"))?;
        if state.store.package_provider().await.map_err(|_|unavailable())?.is_none_or(|p|!p.enabled||p.revision!=provider.revision) {return Err(stale());}
        let target=data.packages.iter_mut().find(|row|row.id==id).ok_or_else(stale)?;*target=package;
        let revision=save(&state,&session.account_id,&data,Some(&reserved)).await?;
        Ok::<_,Response>(Json(json!({"revision":revision})).into_response())
    }.await;result.unwrap_or_else(|e|e)
}
