//! JSON actions over WebSocket, dispatched through the unchanged HTTP router.
//! The clone predates the dispatcher extension, so it cannot form an Arc cycle.
use std::{collections::HashMap,convert::Infallible};
use axum::{Router,body::Body,extract::{FromRequestParts,State,ws::{WebSocketUpgrade,Message}},http::{HeaderMap,Request,StatusCode,request::Parts},response::{Response,IntoResponse}};
use serde::Deserialize;
use serde_json::json;
use tower::ServiceExt;
use crate::AppState;
#[derive(Clone)]
pub(crate) struct Dispatcher(pub Router);
pub(crate) struct Context{headers:HeaderMap,extensions:axum::http::Extensions}
impl<S:Send+Sync> FromRequestParts<S> for Context{
 type Rejection=Infallible;
 async fn from_request_parts(parts:&mut Parts,_:&S)->Result<Self,Self::Rejection>{Ok(Self{headers:parts.headers.clone(),extensions:parts.extensions.clone()})}
}
#[derive(Deserialize)]
#[serde(rename_all="camelCase",deny_unknown_fields)]
struct Action{id:String,method:String,path:String,#[serde(default)]headers:HashMap<String,String>,#[serde(default)]body:Option<String>}
fn allowed_path(path:&str)->bool{
 if path.contains(['\\','\r','\n','#'])||path.contains("..")||path.contains('%')||path.starts_with("//"){return false;}
 let path=path.split('?').next().unwrap_or("");
 path=="/jmap/api"||path=="/api/version"||path.starts_with("/api/mail/")||path=="/api/settings"||path.starts_with("/api/settings/")||path.starts_with("/api/account/")||["/api/tasks/","/api/notes/","/api/contacts/","/api/calendar/","/api/events/"].iter().any(|prefix|path.starts_with(prefix))
}
async fn enabled(state:&AppState,headers:&HeaderMap)->bool{
 let Ok(session)=crate::authed(state,headers).await else{return false;};
 let Ok(subject)=crate::settings_routes::UserSubject::resolved(&state.store,&session).await else{return false;};
 crate::notification_policy::allowed(&state.store,&subject.as_subject(),"realtime.actions.enabled").await.unwrap_or(false)
}
pub(crate) async fn upgrade(State(state):State<AppState>,context:Context,ws:WebSocketUpgrade)->Response{
 if !context.headers.contains_key(axum::http::header::ORIGIN)||!crate::hardening::origin_ok(&context.headers,None){return StatusCode::FORBIDDEN.into_response();}
 if let Err(response)=crate::authed(&state,&context.headers).await{return response;}
 if !enabled(&state,&context.headers).await{return StatusCode::FORBIDDEN.into_response();}
 let Some(dispatcher)=context.extensions.get::<Dispatcher>().cloned() else{return StatusCode::SERVICE_UNAVAILABLE.into_response();};
 ws.max_message_size(2*1024*1024).on_upgrade(move|mut socket|async move{
  if socket.send(Message::Text(json!({"type":"ready","protocol":"mailwoman-actions-v1"}).to_string().into())).await.is_err(){return;}
  while let Some(Ok(frame))=socket.recv().await{
   let Message::Text(text)=frame else{if matches!(frame,Message::Close(_)){return;}continue;};
   let action:Action=match serde_json::from_str(&text){Ok(value)=>value,Err(_)=>{let _=socket.send(Message::Close(None)).await;return;}};
   let id=action.id.clone();
   let result=execute(&state,&context,&dispatcher,action).await;
   let payload=match result{Ok((status,headers,body))=>json!({"id":id,"status":status,"headers":headers,"body":body}),Err(status)=>json!({"id":id,"notDispatched":status==StatusCode::FORBIDDEN,"status":status.as_u16(),"headers":{"content-type":"application/json"},"body":json!({"code":"websocketActionRefused"}).to_string()})};
   if socket.send(Message::Text(payload.to_string().into())).await.is_err(){return;}
  }
 }).into_response()
}
async fn execute(state:&AppState,context:&Context,dispatcher:&Dispatcher,action:Action)->Result<(u16,HashMap<String,String>,String),StatusCode>{
 if action.id.is_empty()||action.id.len()>128||action.headers.len()>16||!allowed_path(&action.path)||!matches!(action.method.as_str(),"GET"|"POST"|"PUT"|"PATCH"|"DELETE"){return Err(StatusCode::BAD_REQUEST);}
 // Recheck the session and admin policy before every action, including after a
 // session revocation or policy change while the connection stays open.
 if !enabled(state,&context.headers).await{return Err(StatusCode::FORBIDDEN);}
 let mut headers=context.headers.clone();
 for key in ["connection","upgrade","sec-websocket-key","sec-websocket-version","sec-websocket-protocol","sec-websocket-extensions","content-length","accept-encoding"]{headers.remove(key);}
 for (name,value) in action.headers{
  let name=name.to_ascii_lowercase();
  if !matches!(name.as_str(),"content-type"|"x-csrf-token"|"x-mailwoman-link"|"x-mailwoman-account"|"x-mailwoman-storage-authority"|"x-mailwoman-client"|"idempotency-key"|"if-match"|"if-none-match"){return Err(StatusCode::BAD_REQUEST);}
  let key=axum::http::HeaderName::from_bytes(name.as_bytes()).map_err(|_|StatusCode::BAD_REQUEST)?;
  headers.insert(key,value.parse().map_err(|_|StatusCode::BAD_REQUEST)?);
 }
 if !enabled(state,&headers).await{return Err(StatusCode::FORBIDDEN);}
 let body=action.body.unwrap_or_default();
 if !body.is_empty()&&headers.get("content-type").and_then(|value|value.to_str().ok()).is_none_or(|value|!value.starts_with("application/json")){return Err(StatusCode::UNSUPPORTED_MEDIA_TYPE);}
 headers.insert(axum::http::header::CONTENT_LENGTH,body.len().to_string().parse().map_err(|_|StatusCode::BAD_REQUEST)?);
 let mut request=Request::builder().method(action.method.as_str()).uri(action.path).body(Body::from(body)).map_err(|_|StatusCode::BAD_REQUEST)?;
 *request.headers_mut()=headers;*request.extensions_mut()=context.extensions.clone();
 request.extensions_mut().remove::<Dispatcher>();
 request.extensions_mut().remove::<axum::extract::OriginalUri>();
 request.extensions_mut().remove::<axum::extract::MatchedPath>();
 // The original router applies CSRF, origin, account fences, scopes, limits,
 // feature policy, idempotency and auditing exactly as an HTTP request does.
 let response=dispatcher.0.clone().oneshot(request).await.unwrap_or_else(|never|match never{});
 let status=response.status().as_u16();let mut headers=HashMap::new();
 for name in ["content-type","cache-control","etag","retry-after"]{if let Some(value)=response.headers().get(name).and_then(|value|value.to_str().ok()){headers.insert(name.into(),value.into());}}
 let bytes=axum::body::to_bytes(response.into_body(),64*1024*1024).await.map_err(|_|StatusCode::BAD_GATEWAY)?;
 Ok((status,headers,String::from_utf8(bytes.to_vec()).map_err(|_|StatusCode::UNSUPPORTED_MEDIA_TYPE)?))
}
#[cfg(test)]mod tests{use super::*;#[test]fn paths_cannot_escape_to_admin_or_auth(){for path in ["/admin/settings","/api/login","//evil.test/api/settings","/api/account/../login","/api/account/%2e%2e/login","/jmap/upload/a"]{assert!(!allowed_path(path));}assert!(allowed_path("/jmap/api"));assert!(allowed_path("/api/account/notifications"));}}
