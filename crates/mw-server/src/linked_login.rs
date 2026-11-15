//! Direct Mailwoman sign-in policy for verified linked mailboxes.
use crate::{AppState,settings_routes::effective_for_session};
use mw_store::{Session,StoreError};
pub(crate) const KEY:&str="accounts.linked.directLogin";
pub(crate) const USER_CONTROL:&str="accounts.linked.allowUserLoginPolicy";
pub(crate) async fn blocked(state:&AppState,session:&Session)->Result<bool,StoreError>{
    if state.store.linked_session_kind(&session.id).await?==Some(1) || !state.store.is_linked_mailbox(&session.account_id).await? {return Ok(false);}
    Ok(!effective_for_session(state,session,KEY).await?.and_then(|entry|entry.value.as_bool()).unwrap_or(true))
}
pub(crate) async fn user_control(state:&AppState,session:&Session)->Result<bool,StoreError>{
    Ok(effective_for_session(state,session,USER_CONTROL).await?.and_then(|entry|entry.value.as_bool()).unwrap_or(false))
}
