//! Upstream JMAP implements the existing engine submission seam, including MDNs and queued sends.
use async_trait::async_trait;
use base64::Engine as _;
use mw_engine::{
    account::MailSubmitter,
    backend::{EngineError, Result},
};
use mw_jmap::submission::{self, EnvelopeOptions, HttpResponse, Transport};
use mw_smtp::{DsnNotify, DsnRet, Outgoing, SubmissionResult, SubmitOptions};
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};
struct ManagedTransport {
    store: mw_store::Store,
    account: String,
    bound: reqwest::Url,
    credential_revision: Vec<u8>,
}
struct JmapSubmitter {
    client: submission::Submitter,
    dsn: AtomicBool,
}
fn error(value: submission::Error) -> EngineError {
    match value {
        err @ submission::Error::Auth => EngineError::Auth(err.to_string()),
        err @ submission::Error::Transport => EngineError::Transport(err.to_string()),
        err @ submission::Error::Uncertain => EngineError::SubmissionUncertain(err.to_string()),
        submission::Error::Protocol(message) => EngineError::Protocol(message),
        submission::Error::Unsupported(message) => EngineError::Unsupported(message),
    }
}
pub(crate) async fn for_account(
    store: &mw_store::Store,
    account: &str,
) -> std::result::Result<Option<Arc<dyn MailSubmitter>>, String> {
    let row = store
        .get_account(account)
        .await
        .map_err(|_| "submission account unavailable")?;
    let config = crate::domain_config::upstream_for(store, &row.username).await?;
    let Some(endpoint) = config
        .as_ref()
        .and_then(|config| config.jmap_submission.as_ref())
    else {
        return Ok(None);
    };
    let bound =
        reqwest::Url::parse(&endpoint.session_url).map_err(|_| "invalid JMAP submission URL")?;
    let transport = Arc::new(ManagedTransport {
        store: store.clone(),
        account: account.into(),
        bound,
        credential_revision: store
            .account_credential_revision(account)
            .await
            .map_err(|_| "submission credentials unavailable")?,
    });
    Ok(Some(Arc::new(JmapSubmitter {
        client: submission::Submitter::new(transport, endpoint.session_url.clone(), row.username),
        dsn: AtomicBool::new(false),
    })))
}
#[async_trait]
impl Transport for ManagedTransport {
    async fn request(
        &self,
        method: &str,
        target: &str,
        content_type: Option<&str>,
        body: &[u8],
    ) -> std::result::Result<HttpResponse, submission::Error> {
        let refused = || submission::Error::Auth;
        let gate = crate::account_gate::for_account(&self.store, &self.account)
            .await
            .map_err(|_| refused())?;
        if gate.disabled || gate.password_change_required {
            return Err(refused());
        }
        let row = self
            .store
            .get_account(&self.account)
            .await
            .map_err(|_| refused())?;
        if self
            .store
            .account_credential_revision(&self.account)
            .await
            .map_err(|_| refused())?
            != self.credential_revision
        {
            return Err(refused());
        }
        let config = crate::domain_config::upstream_for(&self.store, &row.username)
            .await
            .map_err(|_| refused())?
            .ok_or_else(refused)?;
        let endpoint = config.jmap_submission.as_ref().ok_or_else(refused)?;
        if reqwest::Url::parse(&endpoint.session_url).ok().as_ref() != Some(&self.bound) {
            return Err(refused());
        }
        let policy = mw_engine::account::AccountPolicy::from_json(&row.sync_policy_json);
        if config.binding().map_err(|_| refused())? != policy.domain_binding {
            return Err(refused());
        }
        let url = reqwest::Url::parse(target)
            .map_err(|_| submission::Error::Protocol("invalid target URL".into()))?;
        if url.origin() != self.bound.origin()
            || !url.username().is_empty()
            || url.password().is_some()
            || url.fragment().is_some()
        {
            return Err(submission::Error::Protocol(
                "submission authority mismatch".into(),
            ));
        }
        let credentials = self
            .store
            .account_credentials(&self.account)
            .await
            .map_err(|_| refused())?;
        let wire = config
            .auth_username(
                crate::domain_selection::ServiceRole::Submission,
                &row.username,
            )
            .map_err(|_| refused())?;
        let authorization = format!(
            "Basic {}",
            base64::engine::general_purpose::STANDARD
                .encode(format!("{}:{}", wire, credentials.password))
        );
        let mut headers = vec![
            ("authorization".into(), authorization),
            ("accept".into(), "application/json".into()),
        ];
        if let Some(kind) = content_type {
            headers.push(("content-type".into(), kind.into()));
        }
        let metadata = mw_egress::ResponseMetadata::default();
        let plan = crate::image_proxy::load_plan(&self.store)
            .await
            .map_err(|_| submission::Error::Transport)?;
        let canonical = self
            .store
            .canonical_policy_address(&row.username)
            .await
            .map_err(|_| refused())?;
        let domain = canonical
            .rsplit_once('@')
            .map(|(_, domain)| domain)
            .unwrap_or("");
        let outgoing = mw_egress::plan::Request::mail_submission(
            mw_egress::TrafficClass::Plugins,
            mw_egress::PluginHttp {
                connection: None,
                method,
                headers: &headers,
                additional_roots: &[],
            },
            content_type.map(|kind| mw_egress::OutBody {
                content_type: kind,
                bytes: body,
            }),
        )
        .map_err(|_| submission::Error::Unsupported("invalid JMAP submission request".into()))?
        .for_domain(domain)
        .capture_response(&metadata);
        let response = mw_egress::plan::send_with(&plan, url, &outgoing).await;
        let bytes = response.outcome.map_err(|_| submission::Error::Transport)?;
        let head = metadata.snapshot().ok_or(submission::Error::Transport)?;
        Ok(HttpResponse {
            status: head.status,
            body: bytes,
            location: metadata.redirect_location(),
        })
    }
}
#[async_trait]
impl MailSubmitter for JmapSubmitter {
    fn cached_dsn_support(&self) -> bool {
        self.dsn.load(Ordering::Relaxed)
    }
    fn probe_timeout(&self) -> std::time::Duration {
        std::time::Duration::from_secs(15)
    }
    async fn probe_dsn(&self) -> Result<bool> {
        let supported = self.client.probe_dsn().await.map_err(error)?;
        self.dsn.store(supported, Ordering::Relaxed);
        Ok(supported)
    }
    async fn submit(&self, message: Outgoing) -> Result<SubmissionResult> {
        self.submit_with(message, SubmitOptions::default()).await
    }
    async fn submit_with(
        &self,
        message: Outgoing,
        options: SubmitOptions,
    ) -> Result<SubmissionResult> {
        message
            .validate()
            .map_err(|_| EngineError::Protocol("invalid submission envelope".into()))?;
        let envelope = envelope_options(&message, &options)?;
        self.client
            .submit(
                &message.mail_from,
                &message.rcpt_to,
                &message.raw,
                &envelope,
            )
            .await
            .map_err(error)?;
        Ok(SubmissionResult {
            accepted: message.rcpt_to,
            rejected: vec![],
        })
    }
}
fn envelope_options(message: &Outgoing, options: &SubmitOptions) -> Result<EnvelopeOptions> {
    let mut result = EnvelopeOptions {
        require_dsn: options.dsn.is_some(),
        require_tls: options.require_tls,
        ..Default::default()
    };
    if let Some(dsn) = &options.dsn {
        if dsn.notify.contains(&DsnNotify::Never) && dsn.notify.len() > 1 {
            return Err(EngineError::Unsupported(
                "DSN NEVER cannot be combined with other conditions".into(),
            ));
        }
        if let Some(ret) = dsn.ret {
            result.mail_from_parameters.insert(
                "RET".into(),
                serde_json::json!(match ret {
                    DsnRet::Full => "FULL",
                    DsnRet::Hdrs => "HDRS",
                }),
            );
        }
        if let Some(id) = &dsn.envid {
            if id.len() > 100 || id.chars().any(char::is_control) {
                return Err(EngineError::Unsupported("invalid DSN envelope ID".into()));
            }
            result
                .mail_from_parameters
                .insert("ENVID".into(), serde_json::json!(xtext(id)));
        }
        for recipient in &message.rcpt_to {
            let mut parameters = serde_json::Map::new();
            if !dsn.notify.is_empty() {
                parameters.insert(
                    "NOTIFY".into(),
                    serde_json::json!(dsn
                        .notify
                        .iter()
                        .map(|value| match value {
                            DsnNotify::Never => "NEVER",
                            DsnNotify::Success => "SUCCESS",
                            DsnNotify::Failure => "FAILURE",
                            DsnNotify::Delay => "DELAY",
                        })
                        .collect::<Vec<_>>()
                        .join(",")),
                );
            }
            if dsn.orcpt {
                parameters.insert(
                    "ORCPT".into(),
                    serde_json::json!(format!(
                        "{};{}",
                        if recipient.is_ascii() {
                            "rfc822"
                        } else {
                            "utf-8"
                        },
                        xtext(recipient)
                    )),
                );
            }
            result.recipient_parameters.push(parameters);
        }
    }
    Ok(result)
}
fn xtext(value: &str) -> String {
    value
        .bytes()
        .map(|byte| {
            if (33..=126).contains(&byte) && !matches!(byte, b'+' | b'=') {
                (byte as char).to_string()
            } else {
                format!("+{byte:02X}")
            }
        })
        .collect()
}

/// Compliance/system notifications use their own credentials and system_mail route.
struct SystemTransport {
    store: mw_store::Store,
    bound: reqwest::Url,
}
pub(crate) fn system(
    store: mw_store::Store,
    url: &str,
    identity: &str,
) -> std::result::Result<Arc<dyn MailSubmitter>, String> {
    let bound = reqwest::Url::parse(url).map_err(|_| "Configure a JMAP session URL")?;
    if bound.scheme() != "https"
        || !bound.username().is_empty()
        || bound.password().is_some()
        || bound.fragment().is_some()
    {
        return Err("JMAP system mail requires HTTPS without URL credentials or fragment".into());
    }
    let transport = Arc::new(SystemTransport { store, bound });
    Ok(Arc::new(JmapSubmitter {
        client: submission::Submitter::new(transport, url.into(), identity.into()),
        dsn: AtomicBool::new(false),
    }))
}
#[async_trait]
impl Transport for SystemTransport {
    async fn request(
        &self,
        method: &str,
        target: &str,
        content_type: Option<&str>,
        body: &[u8],
    ) -> std::result::Result<HttpResponse, submission::Error> {
        use mw_store::settings_policy::SettingSubject;
        let subject = SettingSubject {
            personal_owner: None,
            account_id: "",
            address: "",
            domain: "",
        };
        let settings = crate::settings_registry::effective_all(&self.store, &subject, |def| {
            def.key.starts_with("compliance.mail.")
        })
        .await
        .map_err(|_| submission::Error::Auth)?;
        let values: std::collections::HashMap<_, _> = settings
            .into_iter()
            .map(|entry| (entry.key, entry.value))
            .collect();
        let value = |key: &str| {
            values
                .get(key)
                .and_then(serde_json::Value::as_str)
                .unwrap_or("")
        };
        if values
            .get("compliance.mail.enabled")
            .and_then(serde_json::Value::as_bool)
            != Some(true)
            || value("compliance.mail.protocol") != "jmap"
            || reqwest::Url::parse(value("compliance.mail.jmapSessionUrl"))
                .ok()
                .as_ref()
                != Some(&self.bound)
        {
            return Err(submission::Error::Auth);
        }
        let url = reqwest::Url::parse(target)
            .map_err(|_| submission::Error::Protocol("invalid target".into()))?;
        if url.origin() != self.bound.origin()
            || !url.username().is_empty()
            || url.password().is_some()
            || url.fragment().is_some()
        {
            return Err(submission::Error::Protocol(
                "system mail authority mismatch".into(),
            ));
        }
        let username = value("compliance.mail.username");
        if username.is_empty() {
            return Err(submission::Error::Unsupported(
                "Configure a JMAP system-mail username".into(),
            ));
        }
        let password = system_password()?;
        let authorization = format!(
            "Basic {}",
            base64::engine::general_purpose::STANDARD.encode(format!("{username}:{password}"))
        );
        let mut headers = vec![
            ("authorization".into(), authorization),
            ("accept".into(), "application/json".into()),
        ];
        if let Some(kind) = content_type {
            headers.push(("content-type".into(), kind.into()));
        }
        let plan = crate::image_proxy::load_plan(&self.store)
            .await
            .map_err(|_| submission::Error::Transport)?;
        if values
            .get("compliance.mail.requireProxy")
            .and_then(serde_json::Value::as_bool)
            != Some(false)
            && plan
                .selected_route(mw_egress::TrafficClass::SystemMail, None)
                .is_none()
        {
            return Err(submission::Error::Unsupported(
                "Configure a system_mail proxy route before sending".into(),
            ));
        }
        let metadata = mw_egress::ResponseMetadata::default();
        let outgoing = mw_egress::plan::Request::mail_submission(
            mw_egress::TrafficClass::SystemMail,
            mw_egress::PluginHttp {
                connection: None,
                method,
                headers: &headers,
                additional_roots: &[],
            },
            content_type.map(|kind| mw_egress::OutBody {
                content_type: kind,
                bytes: body,
            }),
        )
        .map_err(|_| submission::Error::Protocol("invalid system mail request".into()))?
        .capture_response(&metadata);
        let response = mw_egress::plan::send_with(&plan, url, &outgoing).await;
        let body = response.outcome.map_err(|_| submission::Error::Transport)?;
        let head = metadata.snapshot().ok_or(submission::Error::Transport)?;
        Ok(HttpResponse {
            status: head.status,
            body,
            location: metadata.redirect_location(),
        })
    }
}
fn system_password() -> std::result::Result<String, submission::Error> {
    let refuse = || {
        submission::Error::Unsupported("JMAP system-mail password is unavailable; configure MW_COMPLIANCE_JMAP_PASSWORD or MW_COMPLIANCE_JMAP_PASSWORD_FILE".into())
    };
    let password = if let Some(file) = std::env::var_os("MW_COMPLIANCE_JMAP_PASSWORD_FILE") {
        use std::io::Read;
        let mut bytes = Vec::new();
        std::fs::File::open(file)
            .map_err(|_| refuse())?
            .take(4097)
            .read_to_end(&mut bytes)
            .map_err(|_| refuse())?;
        if bytes.len() > 4096 {
            return Err(refuse());
        }
        String::from_utf8(bytes)
            .map_err(|_| refuse())?
            .trim_end_matches(['\r', '\n'])
            .to_owned()
    } else {
        std::env::var("MW_COMPLIANCE_JMAP_PASSWORD").map_err(|_| refuse())?
    };
    if password.len() > 4096 || password.is_empty() {
        return Err(refuse());
    }
    Ok(password)
}
