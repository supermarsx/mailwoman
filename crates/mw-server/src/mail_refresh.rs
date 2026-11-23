use crate::AppState;
use axum::{Router,Json,extract::State,http::{HeaderMap,StatusCode},response::{Response,IntoResponse},routing::post};
use serde::Deserialize;
use serde_json::json;
#[derive(Deserialize)]
#[serde(rename_all="camelCase",deny_unknown_fields)]
struct Request {account_id:String,mailbox_id:Option<String>}
pub(crate) fn router()->Router<AppState>{Router::new().route("/api/mail/refresh",post(refresh))}
async fn refresh(State(state):State<AppState>,headers:HeaderMap,Json(request):Json<Request>)->Response {
 let session=match crate::authed(&state,&headers).await{Ok(session)=>session,Err(response)=>return response};
 if request.account_id!=session.account_id{return (StatusCode::CONFLICT,Json(json!({"code":"accountChanged"}))).into_response();}
 if let Some(engine)=&state.engine {
  match tokio::time::timeout(std::time::Duration::from_secs(90),crate::fixture_profile::measure(crate::fixture_profile::Phase::Handler,engine.refresh_mailbox(&session.account_id,request.mailbox_id.as_deref()))).await {
   Ok(Ok(()))=>{},_=>return (StatusCode::BAD_GATEWAY,Json(json!({"code":"mailRefreshFailed"}))).into_response(),
  }
 }
 ([("cache-control","private, no-store")],Json(json!({"accountId":session.account_id,"upstream":state.engine.is_some()}))).into_response()
}
