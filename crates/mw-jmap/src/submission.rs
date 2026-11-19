//! RFC 8620/8621 raw-MIME submission. All I/O is supplied by the owning host.
//! A lost acknowledgement AFTER EmailSubmission/set is always uncertain.
use crate::{Invocation, Request, Response, Session};
use async_trait::async_trait;
use serde_json::{json, Value};
use std::sync::Arc;
pub const CORE: &str = "urn:ietf:params:jmap:core";
pub const MAIL: &str = "urn:ietf:params:jmap:mail";
pub const SUBMISSION: &str = "urn:ietf:params:jmap:submission";
#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("JMAP submission authentication rejected")]
    Auth,
    #[error("JMAP submission transport unavailable")]
    Transport,
    #[error("JMAP submission protocol error: {0}")]
    Protocol(String),
    #[error("JMAP submission unavailable: {0}")]
    Unsupported(String),
    #[error("JMAP submission outcome is unknown; check Sent before retrying")]
    Uncertain,
}
#[derive(Clone, Debug)]
pub struct HttpResponse {
    pub status: u16,
    pub body: Vec<u8>,
    pub location: Option<String>,
}
#[async_trait]
pub trait Transport: Send + Sync {
    async fn request(
        &self,
        method: &str,
        url: &str,
        content_type: Option<&str>,
        body: &[u8],
    ) -> Result<HttpResponse, Error>;
}
#[derive(Clone, Debug, Default)]
pub struct EnvelopeOptions {
    pub mail_from_parameters: serde_json::Map<String, Value>,
    pub recipient_parameters: Vec<serde_json::Map<String, Value>>,
    pub require_dsn: bool,
    pub require_tls: bool,
}
pub struct Submitter {
    transport: Arc<dyn Transport>,
    session_url: String,
    identity: String,
}
impl Submitter {
    pub fn new(transport: Arc<dyn Transport>, session_url: String, identity: String) -> Self {
        Self {
            transport,
            session_url,
            identity,
        }
    }
    async fn discover(&self) -> Result<(Session, String), Error> {
        let start = validated_url(&self.session_url)?;
        let mut url = start.clone();
        if url.path() == "/" {
            url.set_path("/.well-known/jmap");
        }
        let mut document = None;
        for _ in 0..=5 {
            let response = self
                .transport
                .request("GET", url.as_str(), None, &[])
                .await?;
            if (300..400).contains(&response.status) {
                url = bound_url(
                    &start,
                    &url.join(
                        response
                            .location
                            .as_deref()
                            .ok_or_else(|| protocol("missing discovery redirect"))?,
                    )
                    .map_err(|_| protocol("invalid discovery redirect"))?
                    .to_string(),
                )?;
                continue;
            }
            success(response.status)?;
            document = Some(
                serde_json::from_slice::<Session>(&response.body)
                    .map_err(|_| protocol("invalid session document"))?,
            );
            break;
        }
        let session = document.ok_or_else(|| protocol("too many discovery redirects"))?;
        for capability in [CORE, MAIL, SUBMISSION] {
            if !session.capabilities.contains_key(capability) {
                return Err(Error::Unsupported(format!(
                    "missing capability {capability}"
                )));
            }
        }
        bound_url(&start, &session.api_url)?;
        // Validate the upload authority before any credentials or MIME bytes are sent.
        let upload = session.upload_url.replace("{accountId}", "account");
        bound_url(&start, &upload)?;
        let eligible = |id: &str| {
            session.accounts.get(id).is_some_and(|account| {
                !account.is_read_only
                    && account.account_capabilities.contains_key(MAIL)
                    && account.account_capabilities.contains_key(SUBMISSION)
            })
        };
        let account = if let Some(id) = session.primary_accounts.get(SUBMISSION) {
            if !eligible(id) {
                return Err(Error::Unsupported(
                    "primary submission account is not writable".into(),
                ));
            }
            id.clone()
        } else if let Some(id) = session.primary_accounts.get(MAIL).filter(|id| eligible(id)) {
            id.clone()
        } else {
            let ids: Vec<_> = session
                .accounts
                .keys()
                .filter(|id| eligible(id))
                .cloned()
                .collect();
            if ids.len() != 1 {
                return Err(Error::Unsupported(
                    "select an unambiguous writable submission account".into(),
                ));
            }
            ids[0].clone()
        };
        Ok((session, account))
    }
    pub async fn probe_dsn(&self) -> Result<bool, Error> {
        let (session, account) = self.discover().await?;
        Ok(extension(&session, &account, "DSN"))
    }
    async fn call(
        &self,
        session: &Session,
        account: &str,
        name: &str,
        mut args: Value,
        commit: bool,
    ) -> Result<Value, Error> {
        args["accountId"] = json!(account);
        let request = Request {
            using: vec![CORE.into(), MAIL.into(), SUBMISSION.into()],
            method_calls: vec![Invocation::new(name, args, "request")],
            created_ids: None,
        };
        let bytes = serde_json::to_vec(&request).map_err(|_| protocol("invalid request"))?;
        let response = match self
            .transport
            .request("POST", &session.api_url, Some("application/json"), &bytes)
            .await
        {
            Err(Error::Transport) if commit => return Err(Error::Uncertain),
            other => other?,
        };
        if commit && !(200..300).contains(&response.status) {
            return Err(if matches!(response.status, 401 | 403) {
                Error::Auth
            } else {
                Error::Uncertain
            });
        }
        success(response.status)?;
        let response = serde_json::from_slice::<Response>(&response.body).map_err(|_| {
            if commit {
                Error::Uncertain
            } else {
                protocol("invalid API response")
            }
        })?;
        let matching: Vec<_> = response
            .method_responses
            .iter()
            .filter(|call| {
                call.call_id() == "request" && (call.name() == name || call.name() == "error")
            })
            .collect();
        if matching.len() != 1 {
            return Err(if commit {
                Error::Uncertain
            } else {
                protocol("missing method acknowledgement")
            });
        }
        let call = matching[0];
        if call.name() == "error" {
            return Err(
                if commit
                    && !matches!(
                        call.args()["type"].as_str(),
                        Some(
                            "invalidArguments"
                                | "forbidden"
                                | "accountNotFound"
                                | "accountNotSupportedByMethod"
                                | "accountReadOnly"
                                | "unknownMethod"
                                | "stateMismatch"
                        )
                    )
                {
                    Error::Uncertain
                } else {
                    Error::Unsupported(safe_error(&call.args()["type"]))
                },
            );
        }
        if call.args().get("accountId").and_then(Value::as_str) != Some(account) {
            return Err(if commit {
                Error::Uncertain
            } else {
                protocol("account response mismatch")
            });
        }
        Ok(call.args().clone())
    }
    pub async fn submit(
        &self,
        mail_from: &str,
        recipients: &[String],
        raw: &[u8],
        options: &EnvelopeOptions,
    ) -> Result<(), Error> {
        if recipients.is_empty() {
            return Err(protocol("no recipients"));
        }
        let (session, account) = self.discover().await?;
        if options.require_dsn && !extension(&session, &account, "DSN") {
            return Err(Error::Unsupported("server does not advertise DSN".into()));
        }
        if options.require_tls && !extension(&session, &account, "REQUIRETLS") {
            return Err(Error::Unsupported(
                "server does not advertise REQUIRETLS".into(),
            ));
        }
        let max = session.capabilities[CORE]["maxSizeUpload"].as_u64();
        if max.is_some_and(|max| raw.len() as u64 > max) {
            return Err(Error::Unsupported(
                "message exceeds upstream size limit".into(),
            ));
        }
        let identities = self
            .call(
                &session,
                &account,
                "Identity/get",
                json!({"ids":null}),
                false,
            )
            .await?;
        let sender = if mail_from.is_empty() {
            &self.identity
        } else {
            mail_from
        };
        let identity = identities["list"]
            .as_array()
            .and_then(|rows| {
                rows.iter().find(|row| {
                    row["email"]
                        .as_str()
                        .is_some_and(|email| email.eq_ignore_ascii_case(sender))
                })
            })
            .and_then(|row| row["id"].as_str())
            .ok_or_else(|| Error::Unsupported("sender has no upstream JMAP identity".into()))?
            .to_string();
        let boxes = self
            .call(
                &session,
                &account,
                "Mailbox/get",
                json!({"ids":null,"properties":["id","role","myRights"]}),
                false,
            )
            .await?;
        let mailbox = if let Some(id) = boxes["list"]
            .as_array()
            .and_then(|rows| {
                rows.iter()
                    .find(|row| row["role"] == "drafts" && row["myRights"]["mayAddItems"] == true)
            })
            .and_then(|row| row["id"].as_str())
        {
            id.to_string()
        } else if session.accounts[&account].account_capabilities[MAIL]["mayCreateTopLevelMailbox"]
            == true
        {
            let created=self.call(&session,&account,"Mailbox/set",json!({"create":{"stageMailbox":{"name":"Drafts","parentId":null,"role":"drafts","isSubscribed":false}}}),false).await?;
            created["created"]["stageMailbox"]["id"]
                .as_str()
                .ok_or_else(|| {
                    Error::Unsupported("could not create a writable Drafts mailbox".into())
                })?
                .to_string()
        } else {
            return Err(Error::Unsupported(
                "a writable Drafts mailbox is required for submission staging".into(),
            ));
        };
        let escaped = encode_segment(&account);
        let upload = session.upload_url.replace("{accountId}", &escaped);
        if upload.contains(['{', '}']) {
            return Err(protocol("unsupported upload URL template"));
        }
        bound_url(&validated_url(&self.session_url)?, &upload)?;
        let upload = self
            .transport
            .request("POST", &upload, Some("message/rfc822"), raw)
            .await?;
        success(upload.status)?;
        let uploaded: Value = serde_json::from_slice(&upload.body)
            .map_err(|_| protocol("invalid upload response"))?;
        if uploaded["accountId"] != account || uploaded["size"].as_u64() != Some(raw.len() as u64) {
            return Err(protocol("upload response mismatch"));
        }
        let blob = uploaded["blobId"]
            .as_str()
            .filter(|id| !id.is_empty())
            .ok_or_else(|| protocol("missing uploaded blob"))?;
        let mut mailbox_ids = serde_json::Map::new();
        mailbox_ids.insert(mailbox, json!(true));
        let imported=self.call(&session,&account,"Email/import",json!({"emails":{"stage":{"blobId":blob,"mailboxIds":mailbox_ids,"keywords":{"$draft":true,"$seen":true}}}}),false).await?;
        let email = imported["created"]["stage"]["id"]
            .as_str()
            .ok_or_else(|| {
                Error::Unsupported(safe_error(&imported["notCreated"]["stage"]["type"]))
            })?
            .to_string();
        let mut mail_parameters = options.mail_from_parameters.clone();
        if options.require_tls {
            mail_parameters.insert("REQUIRETLS".into(), Value::Null);
        }
        let to:Vec<_>=recipients.iter().enumerate().map(|(index,email)|json!({"email":email,"parameters":options.recipient_parameters.get(index).cloned().unwrap_or_default()})).collect();
        let result=self.call(&session,&account,"EmailSubmission/set",json!({"create":{"send":{"emailId":email,"identityId":identity,"envelope":{"mailFrom":{"email":mail_from,"parameters":mail_parameters},"rcptTo":to}}},"onSuccessDestroyEmail":["#send"]}),true).await;
        match result {
            Ok(result)
                if result["created"]["send"]["id"]
                    .as_str()
                    .is_some_and(|id| !id.is_empty()) =>
            {
                Ok(())
            }
            Ok(result) => {
                // A specific notCreated acknowledges that nothing was sent. Unknown or
                // malformed responses must preserve the staging email for reconciliation.
                if result["notCreated"]["send"]["type"].is_string() {
                    let _ = self
                        .call(
                            &session,
                            &account,
                            "Email/set",
                            json!({"destroy":[email]}),
                            false,
                        )
                        .await;
                    Err(Error::Unsupported(safe_error(
                        &result["notCreated"]["send"]["type"],
                    )))
                } else {
                    Err(Error::Uncertain)
                }
            }
            Err(error) => Err(error),
        }
    }
}
fn success(status: u16) -> Result<(), Error> {
    match status {
        200..=299 => Ok(()),
        401 | 403 => Err(Error::Auth),
        400..=499 => Err(Error::Unsupported(format!("HTTP {status}"))),
        _ => Err(Error::Transport),
    }
}
fn protocol(message: &str) -> Error {
    Error::Protocol(message.into())
}
fn safe_error(value: &Value) -> String {
    let code = value.as_str().unwrap_or("invalidResponse");
    if code.len() <= 64 && code.bytes().all(|c| c.is_ascii_alphanumeric()) {
        format!("upstream refused submission ({code})")
    } else {
        "upstream refused submission".into()
    }
}
fn validated_url(value: &str) -> Result<reqwest::Url, Error> {
    let url = reqwest::Url::parse(value).map_err(|_| protocol("invalid URL"))?;
    if !matches!(url.scheme(), "https" | "http")
        || !url.username().is_empty()
        || url.password().is_some()
        || url.fragment().is_some()
    {
        return Err(protocol("invalid URL authority"));
    }
    Ok(url)
}
fn bound_url(bound: &reqwest::Url, value: &str) -> Result<reqwest::Url, Error> {
    let url = validated_url(value)?;
    if url.origin() != bound.origin() {
        return Err(protocol("cross-origin upstream URL refused"));
    }
    Ok(url)
}
fn extension(session: &Session, account: &str, name: &str) -> bool {
    session.accounts[account].account_capabilities[SUBMISSION]["submissionExtensions"]
        .as_object()
        .is_some_and(|values| values.keys().any(|key| key.eq_ignore_ascii_case(name)))
}
fn encode_segment(value: &str) -> String {
    value
        .bytes()
        .map(|b| {
            if b.is_ascii_alphanumeric() || matches!(b, b'-' | b'_' | b'.' | b'~') {
                (b as char).to_string()
            } else {
                format!("%{b:02X}")
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Mutex;
    #[derive(Default)]
    struct Fixture {
        session: Mutex<Value>,
        calls: Mutex<Vec<(String, String, Vec<u8>)>>,
        lost: bool,
        refused: bool,
        bad_ack: bool,
        method_error: Option<String>,
        no_drafts: bool,
        redirect: Option<String>,
    }
    fn session() -> Value {
        json!({"capabilities":{(CORE):{"maxSizeUpload":1000000},(MAIL):{},(SUBMISSION):{}},"accounts":{"a/b":{"isReadOnly":false,"accountCapabilities":{(MAIL):{},(SUBMISSION):{"submissionExtensions":{"DSN":[],"REQUIRETLS":[]}}}}},"primaryAccounts":{(SUBMISSION):"a/b"},"apiUrl":"https://jmap.example/api","uploadUrl":"https://jmap.example/upload/{accountId}"})
    }
    fn reply(value: Value) -> HttpResponse {
        HttpResponse {
            status: 200,
            body: serde_json::to_vec(&value).unwrap(),
            location: None,
        }
    }
    impl Fixture {
        fn new() -> Self {
            Self {
                session: Mutex::new(session()),
                ..Default::default()
            }
        }
    }
    #[async_trait]
    impl Transport for Fixture {
        async fn request(
            &self,
            method: &str,
            url: &str,
            _content_type: Option<&str>,
            body: &[u8],
        ) -> Result<HttpResponse, Error> {
            self.calls
                .lock()
                .unwrap()
                .push((method.into(), url.into(), body.to_vec()));
            if method == "GET" {
                if let Some(location) = &self.redirect {
                    return Ok(HttpResponse {
                        status: 302,
                        body: vec![],
                        location: Some(location.clone()),
                    });
                }
                return Ok(reply(self.session.lock().unwrap().clone()));
            }
            if url.contains("/upload/") {
                return Ok(reply(
                    json!({"accountId":"a/b","blobId":"uploaded","size":body.len()}),
                ));
            }
            let request: Request = serde_json::from_slice(body).unwrap();
            let call = &request.method_calls[0];
            let account = call.args()["accountId"].clone();
            let mut args = match call.name() {
                "Identity/get" => json!({"list":[{"id":"identity","email":"me@example.test"}]}),
                "Mailbox/get" if self.no_drafts => json!({"list":[]}),
                "Mailbox/set" => json!({"created":{"stageMailbox":{"id":"drafts"}}}),
                "Mailbox/get" => {
                    json!({"list":[{"id":"drafts","role":"drafts","myRights":{"mayAddItems":true}}]})
                }
                "Email/import" => json!({"created":{"stage":{"id":"staged"}}}),
                "EmailSubmission/set" if self.method_error.is_some() => {
                    return Ok(reply(
                        json!({"methodResponses":[["error",{"type":self.method_error},"request"]],"sessionState":"s"}),
                    ))
                }
                "EmailSubmission/set" if self.lost => return Err(Error::Transport),
                "EmailSubmission/set" if self.bad_ack => {
                    return Ok(reply(json!({"methodResponses":[]})))
                }
                "EmailSubmission/set" if self.refused => {
                    json!({"notCreated":{"send":{"type":"forbiddenFrom"}}})
                }
                "EmailSubmission/set" => json!({"created":{"send":{"id":"submitted"}}}),
                "Email/set" => json!({"destroyed":["staged"]}),
                _ => panic!("unexpected method"),
            };
            args["accountId"] = account;
            Ok(reply(
                json!({"methodResponses":[[call.name(),args,"request"]],"sessionState":"s"}),
            ))
        }
    }
    fn client(fixture: Arc<Fixture>) -> Submitter {
        Submitter::new(
            fixture,
            "https://jmap.example/.well-known/jmap".into(),
            "me@example.test".into(),
        )
    }
    fn recipients() -> Vec<String> {
        vec!["to@example.test".into(), "hidden@example.test".into()]
    }
    #[tokio::test]
    async fn raw_mime_envelope_and_staging_cleanup_are_preserved() {
        let fixture = Arc::new(Fixture::new());
        let client = client(fixture.clone());
        let raw =
            b"From: me@example.test\r\nBcc: hidden@example.test\r\n\r\nRaw MIME with attachments";
        let mut options = EnvelopeOptions {
            require_dsn: true,
            require_tls: true,
            ..Default::default()
        };
        options
            .mail_from_parameters
            .insert("RET".into(), json!("HDRS"));
        options.recipient_parameters =
            vec![serde_json::from_value(json!({"NOTIFY":"FAILURE,DELAY"})).unwrap(); 2];
        client
            .submit("me@example.test", &recipients(), raw, &options)
            .await
            .unwrap();
        let calls = fixture.calls.lock().unwrap();
        assert_eq!(
            calls
                .iter()
                .find(|(_, url, _)| url.contains("/upload/"))
                .unwrap()
                .1,
            "https://jmap.example/upload/a%2Fb"
        );
        assert_eq!(
            &calls
                .iter()
                .find(|(_, url, _)| url.contains("/upload/"))
                .unwrap()
                .2,
            raw
        );
        let request: Request = serde_json::from_slice(&calls.last().unwrap().2).unwrap();
        let args = request.method_calls[0].args();
        assert_eq!(args["onSuccessDestroyEmail"], json!(["#send"]));
        assert_eq!(
            args["create"]["send"]["envelope"]["rcptTo"]
                .as_array()
                .unwrap()
                .len(),
            2
        );
        assert_eq!(
            args["create"]["send"]["envelope"]["mailFrom"]["parameters"]["RET"],
            "HDRS"
        );
    }
    #[tokio::test]
    async fn lost_or_malformed_submission_acknowledgements_are_uncertain_and_never_cleaned_up() {
        for bad_ack in [false, true] {
            let mut fixture = Fixture::new();
            fixture.lost = !bad_ack;
            fixture.bad_ack = bad_ack;
            let fixture = Arc::new(fixture);
            assert!(matches!(
                client(fixture.clone())
                    .submit(
                        "me@example.test",
                        &recipients(),
                        b"raw",
                        &EnvelopeOptions::default()
                    )
                    .await,
                Err(Error::Uncertain)
            ));
            let calls = fixture.calls.lock().unwrap();
            assert!(!calls
                .iter()
                .any(|(_, _, body)| String::from_utf8_lossy(body).contains("destroy\":[")));
        }
    }
    #[tokio::test]
    async fn explicit_refusals_clean_the_unsent_staging_message() {
        let mut fixture = Fixture::new();
        fixture.refused = true;
        let fixture = Arc::new(fixture);
        assert!(matches!(
            client(fixture.clone())
                .submit(
                    "me@example.test",
                    &recipients(),
                    b"raw",
                    &EnvelopeOptions::default()
                )
                .await,
            Err(Error::Unsupported(_))
        ));
        let calls = fixture.calls.lock().unwrap();
        let request: Request = serde_json::from_slice(&calls.last().unwrap().2).unwrap();
        assert_eq!(request.method_calls[0].name(), "Email/set");
    }
    #[tokio::test]
    async fn dsn_is_probed_and_never_silently_ignored() {
        let fixture = Arc::new(Fixture::new());
        let client = client(fixture.clone());
        assert!(client.probe_dsn().await.unwrap());
        fixture.session.lock().unwrap()["accounts"]["a/b"]["accountCapabilities"][SUBMISSION]
            ["submissionExtensions"] = json!({});
        assert!(matches!(
            client
                .submit(
                    "me@example.test",
                    &recipients(),
                    b"raw",
                    &EnvelopeOptions {
                        require_dsn: true,
                        ..Default::default()
                    }
                )
                .await,
            Err(Error::Unsupported(_))
        ));
        assert!(fixture
            .calls
            .lock()
            .unwrap()
            .iter()
            .all(|(method, _, _)| method == "GET"));
    }
    #[tokio::test]
    async fn cross_origin_session_urls_are_refused_before_upload_or_api_calls() {
        for field in ["apiUrl", "uploadUrl"] {
            let fixture = Arc::new(Fixture::new());
            fixture.session.lock().unwrap()[field] = json!("https://evil.example/steal");
            assert!(matches!(
                client(fixture.clone())
                    .submit(
                        "me@example.test",
                        &recipients(),
                        b"raw",
                        &EnvelopeOptions::default()
                    )
                    .await,
                Err(Error::Protocol(_))
            ));
            assert_eq!(fixture.calls.lock().unwrap().len(), 1);
        }
    }
    #[tokio::test]
    async fn read_only_accounts_missing_capabilities_and_unowned_senders_are_refused() {
        for case in ["read-only", "capability", "sender"] {
            let fixture = Arc::new(Fixture::new());
            if case == "read-only" {
                fixture.session.lock().unwrap()["accounts"]["a/b"]["isReadOnly"] = json!(true);
            }
            if case == "capability" {
                fixture.session.lock().unwrap()["capabilities"]
                    .as_object_mut()
                    .unwrap()
                    .remove(SUBMISSION);
            }
            let from = if case == "sender" {
                "other@example.test"
            } else {
                "me@example.test"
            };
            assert!(matches!(
                client(fixture.clone())
                    .submit(from, &recipients(), b"raw", &EnvelopeOptions::default())
                    .await,
                Err(Error::Unsupported(_))
            ));
            assert!(!fixture
                .calls
                .lock()
                .unwrap()
                .iter()
                .any(|(_, url, _)| url.contains("/upload/")));
        }
    }
    #[tokio::test]
    async fn missing_drafts_is_created_only_when_account_permits_it() {
        for allowed in [false, true] {
            let mut fixture = Fixture::new();
            fixture.no_drafts = true;
            fixture.session.lock().unwrap()["accounts"]["a/b"]["accountCapabilities"][MAIL]
                ["mayCreateTopLevelMailbox"] = json!(allowed);
            let fixture = Arc::new(fixture);
            let result = client(fixture.clone())
                .submit(
                    "me@example.test",
                    &recipients(),
                    b"raw",
                    &EnvelopeOptions::default(),
                )
                .await;
            assert_eq!(result.is_ok(), allowed);
            let calls = fixture.calls.lock().unwrap();
            assert_eq!(
                calls.iter().any(|(_, url, _)| url.contains("/upload/")),
                allowed
            );
        }
    }
    #[tokio::test]
    async fn discovery_redirects_are_bounded_and_never_send_credentials_cross_origin() {
        for location in ["/loop", "https://evil.example/session"] {
            let mut fixture = Fixture::new();
            fixture.redirect = Some(location.into());
            let fixture = Arc::new(fixture);
            assert!(client(fixture.clone()).probe_dsn().await.is_err());
            let calls = fixture.calls.lock().unwrap();
            assert!(calls.len() <= 6);
            assert!(calls.iter().all(
                |(method, url, _)| method == "GET" && url.starts_with("https://jmap.example/")
            ));
        }
    }
    #[tokio::test]
    async fn ambiguous_accounts_are_refused_before_mutating_anything() {
        let fixture = Arc::new(Fixture::new());
        {
            let mut session = fixture.session.lock().unwrap();
            session["primaryAccounts"] = json!({});
            session["accounts"]["other"] = session["accounts"]["a/b"].clone();
        }
        assert!(matches!(
            client(fixture.clone()).probe_dsn().await,
            Err(Error::Unsupported(_))
        ));
        assert_eq!(fixture.calls.lock().unwrap().len(), 1);
    }
    #[tokio::test]
    async fn server_method_failures_preserve_uncertain_submission_outcomes() {
        for code in ["serverFail", "serverPartialFail", "unexpectedFailure"] {
            let mut fixture = Fixture::new();
            fixture.method_error = Some(code.into());
            let fixture = Arc::new(fixture);
            assert!(matches!(
                client(fixture)
                    .submit(
                        "me@example.test",
                        &recipients(),
                        b"raw",
                        &EnvelopeOptions::default()
                    )
                    .await,
                Err(Error::Uncertain)
            ));
        }
    }
}
