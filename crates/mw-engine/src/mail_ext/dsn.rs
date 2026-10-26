//! Delivery status notifications asked for on submit (t32-e46, D-21).
//!
//! RFC 8621 §7 lets an `EmailSubmission` envelope carry SMTP parameters: on
//! `mailFrom` (`RET`, `ENVID`) and on each `rcptTo` entry (`NOTIFY`, `ORCPT`).
//! The engine used to drop them. They are now read into a [`DsnRequest`], kept
//! with the submission (it may be held for the undo window or a send time) and
//! handed to `mw-smtp`'s DSN support (RFC 3461) when the message goes out.
//!
//! What is accepted, and why:
//! - `NOTIFY` is `NEVER` alone, or any of `SUCCESS`, `FAILURE`, `DELAY`. The
//!   submission client applies one `NOTIFY` to every recipient, so every
//!   recipient that names it must name the same set.
//! - `ORCPT` is accepted only as `rfc822;<the recipient's own address>`: the
//!   client writes exactly that, and an `ORCPT` naming another address would put
//!   a different address in the reports the recipients' servers send back.
//! - `RET` is `FULL` or `HDRS`; `ENVID` is 1-100 printable ASCII characters (it is
//!   xtext-encoded on the wire by `mw-smtp`).
//! - Other parameters are ignored, as before: they are not DSN, and refusing a
//!   submission over, say, a `SIZE` the client computed would help nobody.
//!
//! When the submission server does not offer DSN the message is sent without
//! it rather than not at all, and the account remembers that, so the session
//! stops advertising `DSN` in `submissionExtensions` and the client hides the
//! option (see [`Engine::record_dsn_support`]).

use std::collections::HashSet;
use std::sync::{Mutex, OnceLock};

use serde_json::{Map, Value, json};

use crate::backend::Result;
use crate::engine::Engine;

/// The DSN parameters of one submission.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct DsnRequest {
    /// `NOTIFY` keywords, upper case, in RFC 3461 order.
    pub notify: Vec<String>,
    /// `RET=FULL|HDRS`.
    pub ret: Option<String>,
    /// `ENVID`.
    pub envid: Option<String>,
    /// Emit `ORCPT=rfc822;<addr>` per recipient.
    pub orcpt: bool,
}

const NOTIFY_ORDER: [&str; 4] = ["NEVER", "SUCCESS", "FAILURE", "DELAY"];

impl DsnRequest {
    /// Read the DSN parameters of an RFC 8621 envelope. `Ok(None)` when it asks
    /// for none; `Err` names what is wrong (for `invalidProperties`).
    pub fn from_envelope(
        envelope: &Map<String, Value>,
    ) -> std::result::Result<Option<Self>, String> {
        let mut req = DsnRequest::default();
        if let Some(params) = parameters(envelope.get("mailFrom"))? {
            if let Some(ret) = param(params, "RET")? {
                let ret = ret.to_ascii_uppercase();
                if ret != "FULL" && ret != "HDRS" {
                    return Err("mailFrom.parameters.RET must be FULL or HDRS".into());
                }
                req.ret = Some(ret);
            }
            if let Some(envid) = param(params, "ENVID")? {
                if envid.is_empty()
                    || envid.len() > 100
                    || !envid.bytes().all(|b| (0x21..=0x7e).contains(&b))
                {
                    return Err(
                        "mailFrom.parameters.ENVID must be 1-100 printable ASCII characters".into(),
                    );
                }
                req.envid = Some(envid.to_string());
            }
        }
        let rcpts = envelope
            .get("rcptTo")
            .and_then(Value::as_array)
            .map(Vec::as_slice)
            .unwrap_or_default();
        let mut notifies: Vec<Option<Vec<String>>> = Vec::with_capacity(rcpts.len());
        let mut orcpt_seen = false;
        let mut orcpt_all = true;
        for rcpt in rcpts {
            let email = rcpt
                .get("email")
                .and_then(Value::as_str)
                .unwrap_or_default();
            let params = parameters(Some(rcpt))?;
            notifies.push(
                match params.map(|p| param(p, "NOTIFY")).transpose()?.flatten() {
                    Some(n) => Some(parse_notify(n)?),
                    None => None,
                },
            );
            match params.map(|p| param(p, "ORCPT")).transpose()?.flatten() {
                Some(o) => {
                    let addr = o
                        .split_once(';')
                        .filter(|(kind, _)| kind.eq_ignore_ascii_case("rfc822"))
                        .map(|(_, a)| a)
                        .ok_or_else(|| {
                            "rcptTo.parameters.ORCPT must be rfc822;<address>".to_string()
                        })?;
                    if !addr.eq_ignore_ascii_case(email) {
                        return Err(
                            "rcptTo.parameters.ORCPT must name the recipient's own address".into(),
                        );
                    }
                    orcpt_seen = true;
                }
                None => orcpt_all = false,
            }
        }
        if let Some(first) = notifies.first() {
            if notifies.iter().any(|n| n != first) {
                return Err("rcptTo: every recipient must ask for the same NOTIFY".into());
            }
            if let Some(n) = first {
                req.notify = n.clone();
            }
        }
        req.orcpt = orcpt_seen && orcpt_all;
        if req == DsnRequest::default() {
            return Ok(None);
        }
        Ok(Some(req))
    }

    /// The request as kept with a submission.
    pub fn to_json(&self) -> Value {
        json!({ "notify": self.notify, "ret": self.ret, "envid": self.envid, "orcpt": self.orcpt })
    }

    /// The request kept with a submission, if any.
    pub fn from_json(v: &Value) -> Option<Self> {
        let o = v.as_object()?;
        Some(DsnRequest {
            notify: o
                .get("notify")?
                .as_array()?
                .iter()
                .filter_map(|s| s.as_str().map(String::from))
                .collect(),
            ret: o.get("ret").and_then(Value::as_str).map(String::from),
            envid: o.get("envid").and_then(Value::as_str).map(String::from),
            orcpt: o.get("orcpt").and_then(Value::as_bool).unwrap_or(false),
        })
    }

    /// The `mw-smtp` form.
    pub fn to_smtp(&self) -> mw_smtp::Dsn {
        mw_smtp::Dsn {
            ret: self.ret.as_deref().map(|r| {
                if r == "HDRS" {
                    mw_smtp::DsnRet::Hdrs
                } else {
                    mw_smtp::DsnRet::Full
                }
            }),
            envid: self.envid.clone(),
            notify: self
                .notify
                .iter()
                .filter_map(|n| match n.as_str() {
                    "NEVER" => Some(mw_smtp::DsnNotify::Never),
                    "SUCCESS" => Some(mw_smtp::DsnNotify::Success),
                    "FAILURE" => Some(mw_smtp::DsnNotify::Failure),
                    "DELAY" => Some(mw_smtp::DsnNotify::Delay),
                    _ => None,
                })
                .collect(),
            orcpt: self.orcpt,
        }
    }
}

/// The `parameters` object of an envelope address, if it has one.
fn parameters(addr: Option<&Value>) -> std::result::Result<Option<&Map<String, Value>>, String> {
    match addr.and_then(|a| a.get("parameters")) {
        None | Some(Value::Null) => Ok(None),
        Some(Value::Object(p)) => Ok(Some(p)),
        Some(_) => Err("parameters must be an object".into()),
    }
}

/// One parameter by name (case-insensitive), as a string. A parameter given
/// without a value (`null`) reads as absent.
fn param<'a>(
    params: &'a Map<String, Value>,
    name: &str,
) -> std::result::Result<Option<&'a str>, String> {
    match params
        .iter()
        .find(|(k, _)| k.eq_ignore_ascii_case(name))
        .map(|(_, v)| v)
    {
        None | Some(Value::Null) => Ok(None),
        Some(Value::String(s)) => Ok(Some(s.as_str())),
        Some(_) => Err(format!("parameters.{name} must be a string")),
    }
}

fn parse_notify(raw: &str) -> std::result::Result<Vec<String>, String> {
    let words: Vec<String> = raw
        .split(',')
        .map(|w| w.trim().to_ascii_uppercase())
        .filter(|w| !w.is_empty())
        .collect();
    if words.is_empty() || words.iter().any(|w| !NOTIFY_ORDER.contains(&w.as_str())) {
        return Err(
            "rcptTo.parameters.NOTIFY must be NEVER, or any of SUCCESS, FAILURE, DELAY".into(),
        );
    }
    if words.iter().any(|w| w == "NEVER") && words.len() > 1 {
        return Err("rcptTo.parameters.NOTIFY: NEVER cannot be combined".into());
    }
    Ok(NOTIFY_ORDER
        .iter()
        .filter(|n| words.iter().any(|w| w == *n))
        .map(|n| n.to_string())
        .collect())
}

impl Engine {
    /// The `settings` key recording whether the account's submission server
    /// offered DSN on its last DSN send (`"true"` / `"false"`).
    pub fn dsn_support_key(account_id: &str) -> String {
        format!("smtp_dsn:{account_id}")
    }

    /// Record what the submission server said about DSN.
    pub(crate) async fn record_dsn_support(&self, account_id: &str, offered: bool) {
        if let Ok(mut refused) = dsn_refused().lock() {
            if offered {
                refused.remove(account_id);
            } else {
                refused.insert(account_id.to_string());
            }
        }
        if let Err(e) = self
            .store()
            .set_setting(
                &Self::dsn_support_key(account_id),
                if offered { "true" } else { "false" },
            )
            .await
        {
            tracing::warn!("account {account_id}: could not record DSN support: {e}");
        }
    }

    /// Whether DSN can be asked for: not known to be refused by the account's
    /// submission server. Reads the stored record (and refreshes the copy the
    /// session document reads).
    pub async fn dsn_offered(&self, account_id: &str) -> Result<bool> {
        let offered = self
            .store()
            .get_setting(&Self::dsn_support_key(account_id))
            .await?
            .as_deref()
            != Some("false");
        if let Ok(mut refused) = dsn_refused().lock() {
            if offered {
                refused.remove(account_id);
            } else {
                refused.insert(account_id.to_string());
            }
        }
        Ok(offered)
    }

    /// The session's `submissionExtensions` (RFC 8621 §1.3.2): `DSN` unless the
    /// account's submission server is known to refuse it. Synchronous, for the
    /// session document; it reads the copy [`Engine::record_dsn_support`] and
    /// [`Engine::dsn_offered`] keep.
    pub fn submission_extensions(&self, account_id: &str) -> Value {
        let refused = dsn_refused()
            .lock()
            .map(|r| r.contains(account_id))
            .unwrap_or(false);
        if refused {
            json!({})
        } else {
            json!({ "DSN": [] })
        }
    }

    /// Whether an administrator allows asking for delivery status
    /// notifications (`receipts.dsnAllowed`, default true).
    pub(crate) async fn dsn_allowed(&self, account_id: &str) -> bool {
        match self
            .held_setting_for(account_id, "receipts.dsnAllowed", "true")
            .await
        {
            Ok(Some(v)) => serde_json::from_str::<bool>(&v).unwrap_or(false),
            Ok(None) => true,
            Err(e) => {
                tracing::warn!("account {account_id}: could not read receipts.dsnAllowed: {e}");
                false
            }
        }
    }
}

/// Accounts whose submission server refused DSN, as last recorded.
fn dsn_refused() -> &'static Mutex<HashSet<String>> {
    static REFUSED: OnceLock<Mutex<HashSet<String>>> = OnceLock::new();
    REFUSED.get_or_init(|| Mutex::new(HashSet::new()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn malformed_held_dsn_permission_is_not_permission() {
        let store = mw_store::Store::open_in_memory(mw_store::ServerKey::generate())
            .await
            .unwrap();
        let account = store
            .create_account(
                &mw_store::NewAccount {
                    kind: mw_store::AccountKind::Imap,
                    host: "imap.example.org",
                    port: 993,
                    tls: "implicit",
                    username: "a@example.org",
                    sync_policy_json: "{}",
                },
                &mw_store::Credentials {
                    username: "a@example.org".into(),
                    password: "test".into(),
                },
            )
            .await
            .unwrap();
        let engine = crate::Engine::new(store.clone());
        for value in ["\"true\"", "null", "{}", "not-json", "false"] {
            store
                .put_setting_policy(&mw_store::settings_policy::SettingPolicyRow {
                    key: "receipts.dsnAllowed".into(),
                    global_value: Some(value.into()),
                    allow_user_override: false,
                    global_locked: true,
                    updated_by: "admin".into(),
                    updated_at: 1,
                })
                .await
                .unwrap();
            assert!(!engine.dsn_allowed(&account).await, "{value}");
        }
    }

    fn env(v: Value) -> Map<String, Value> {
        v.as_object().unwrap().clone()
    }

    #[test]
    fn no_parameters_is_no_request() {
        let e =
            env(json!({ "mailFrom": { "email": "a@x.org" }, "rcptTo": [{ "email": "b@y.org" }] }));
        assert_eq!(DsnRequest::from_envelope(&e), Ok(None));
    }

    #[test]
    fn reads_notify_orcpt_ret_envid() {
        let e = env(json!({
            "mailFrom": { "email": "a@x.org", "parameters": { "RET": "hdrs", "ENVID": "env-1" } },
            "rcptTo": [
                { "email": "b@y.org", "parameters": { "NOTIFY": "FAILURE,SUCCESS", "ORCPT": "rfc822;b@y.org" } },
                { "email": "c@y.org", "parameters": { "notify": "success, failure", "ORCPT": "RFC822;C@y.org" } }
            ]
        }));
        let r = DsnRequest::from_envelope(&e).unwrap().unwrap();
        assert_eq!(
            r,
            DsnRequest {
                notify: vec!["SUCCESS".into(), "FAILURE".into()],
                ret: Some("HDRS".into()),
                envid: Some("env-1".into()),
                orcpt: true,
            }
        );
        assert_eq!(DsnRequest::from_json(&r.to_json()), Some(r));
    }

    #[test]
    fn refuses_what_cannot_be_sent_as_asked() {
        for (e, why) in [
            (
                json!({ "rcptTo": [{ "email": "b@y.org", "parameters": { "NOTIFY": "SOMETIMES" } }] }),
                "NOTIFY",
            ),
            (
                json!({ "rcptTo": [{ "email": "b@y.org", "parameters": { "NOTIFY": "NEVER,SUCCESS" } }] }),
                "NEVER",
            ),
            (
                json!({ "rcptTo": [
                    { "email": "b@y.org", "parameters": { "NOTIFY": "SUCCESS" } },
                    { "email": "c@y.org", "parameters": { "NOTIFY": "FAILURE" } }
                ] }),
                "same NOTIFY",
            ),
            (
                json!({ "rcptTo": [
                    { "email": "b@y.org", "parameters": { "NOTIFY": "SUCCESS" } },
                    { "email": "c@y.org" }
                ] }),
                "same NOTIFY",
            ),
            (
                json!({ "rcptTo": [{ "email": "b@y.org", "parameters": { "ORCPT": "rfc822;other@y.org" } }] }),
                "own address",
            ),
            (
                json!({ "mailFrom": { "email": "a@x.org", "parameters": { "RET": "ALL" } }, "rcptTo": [] }),
                "RET",
            ),
            (
                json!({ "mailFrom": { "email": "a@x.org", "parameters": { "ENVID": "has space" } }, "rcptTo": [] }),
                "ENVID",
            ),
        ] {
            let err = DsnRequest::from_envelope(&env(e.clone())).unwrap_err();
            assert!(err.contains(why), "{e}: {err}");
        }
    }

    #[test]
    fn other_parameters_are_ignored() {
        let e = env(
            json!({ "mailFrom": { "email": "a@x.org", "parameters": { "SIZE": "1000" } },
                            "rcptTo": [{ "email": "b@y.org", "parameters": { "X-FOO": "bar" } }] }),
        );
        assert_eq!(DsnRequest::from_envelope(&e), Ok(None));
    }

    #[test]
    fn maps_onto_mw_smtp() {
        let d = DsnRequest {
            notify: vec!["SUCCESS".into(), "DELAY".into()],
            ret: Some("FULL".into()),
            envid: None,
            orcpt: false,
        }
        .to_smtp();
        assert_eq!(
            d.notify,
            vec![mw_smtp::DsnNotify::Success, mw_smtp::DsnNotify::Delay]
        );
        assert_eq!(d.ret, Some(mw_smtp::DsnRet::Full));
    }
}
