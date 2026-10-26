//! First-login proof uses transient credentials and never writes a provisional account.
use super::SelectedEws;
use crate::login_failure::{LoginFailure, LoginFailureReason};
use async_trait::async_trait;
use mw_plugin::{
    BasicCredentialProvider, BasicCredentials, HostServices, HttpFetcher, HttpReq, HttpResp,
    InvocationPolicy,
};
use std::sync::{Arc, OnceLock};
use std::time::Duration;

struct Context {
    store: mw_store::Store,
    selected: SelectedEws,
    wire_username: String,
    password: String,
    policy_account: Option<String>,
    scope: String,
}

impl Context {
    async fn admit(&self, plugin: &str, account: Option<&str>) -> Result<(), String> {
        if plugin != "bridge-ews" || account != Some(self.scope.as_str()) {
            return Err("provider probe scope refused".into());
        }
        self.selected
            .admit(&self.store, self.policy_account.as_deref())
            .await?;
        let row = self
            .store
            .get_plugin("bridge-ews")
            .await
            .map_err(|_| "provider registry unavailable")?
            .ok_or("provider unavailable")?;
        crate::v7_mount::plan_load(&self.store, &row, None)
            .await
            .map_err(|_| "provider grant unavailable")?;
        let grants = self
            .store
            .plugin_grants_scoped("bridge-ews", None)
            .await
            .map_err(|_| "provider grants unavailable")?;
        if !["net", "account-backend"]
            .iter()
            .all(|required| grants.iter().any(|grant| grant == required))
        {
            return Err("provider grant unavailable".into());
        }
        Ok(())
    }
}

#[async_trait]
impl InvocationPolicy for Context {
    async fn admit(&self, plugin: &str, account: Option<&str>) -> Result<(), String> {
        Context::admit(self, plugin, account).await
    }
}

#[async_trait]
impl BasicCredentialProvider for Context {
    async fn credentials(&self, account: &str) -> Result<BasicCredentials, String> {
        self.admit("bridge-ews", Some(account)).await?;
        Ok(BasicCredentials {
            user: self.wire_username.clone(),
            password: self.password.clone(),
            domain: String::new(),
            workstation: String::new(),
            endpoint: self.selected.config.endpoint()?.to_string(),
        })
    }
}

#[async_trait]
impl HttpFetcher for Context {
    async fn fetch(&self, _request: HttpReq) -> Result<HttpResp, String> {
        Err("unbound provider probe refused".into())
    }
    async fn fetch_for(
        &self,
        plugin: &str,
        account: Option<&str>,
        request: HttpReq,
    ) -> Result<HttpResp, String> {
        self.admit(plugin, account).await?;
        let endpoint = self.selected.config.endpoint()?;
        let target = reqwest::Url::parse(&request.url).map_err(|_| "invalid provider probe URL")?;
        if target != endpoint
            || request.method != "POST"
            || request
                .body
                .as_ref()
                .is_some_and(|body| body.len() > 1024 * 1024)
        {
            return Err("provider probe destination refused".into());
        }
        let plan = crate::image_proxy::load_plan(&self.store)
            .await
            .map_err(|_| "provider egress unavailable")?;
        let roots = self.selected.config.roots()?;
        let metadata = mw_egress::ResponseMetadata::default();
        let body = request.body.as_deref().map(|bytes| mw_egress::OutBody {
            content_type: "text/xml",
            bytes,
        });
        let operation = mw_egress::plan::Request::plugin(
            mw_egress::TrafficClass::Plugins,
            mw_egress::PluginHttp {
                method: "POST",
                headers: &request.headers,
                additional_roots: &roots,
            },
            body,
        )
        .map_err(|_| "invalid provider probe request")?
        .for_domain(&self.selected.policy_domain)
        .capture_response(&metadata);
        let response = mw_egress::plan::send_with(&plan, endpoint, &operation).await;
        let body = response
            .outcome
            .map_err(|_| "provider egress refused or failed")?;
        let head = metadata.snapshot().ok_or("provider response unavailable")?;
        let headers = response
            .content_type
            .map(|value| vec![("content-type".into(), value)])
            .unwrap_or_default();
        Ok(HttpResp {
            status: head.status,
            headers,
            body,
        })
    }
}

pub(super) async fn authenticate(
    store: &mw_store::Store,
    selected: &SelectedEws,
    username: &str,
    password: &str,
    policy_account: Option<&str>,
) -> Result<Vec<mw_engine::RawMailbox>, LoginFailure> {
    static SLOTS: OnceLock<Arc<tokio::sync::Semaphore>> = OnceLock::new();
    let permit = SLOTS
        .get_or_init(|| Arc::new(tokio::sync::Semaphore::new(4)))
        .clone()
        .try_acquire_owned()
        .map_err(|_| LoginFailure::new(LoginFailureReason::Policy))?;
    let operation = async {
        let context = Arc::new(Context {
            store: store.clone(),
            selected: selected.clone(),
            wire_username: username.into(),
            password: password.into(),
            policy_account: policy_account.map(str::to_owned),
            scope: uuid::Uuid::new_v4().to_string(),
        });
        context
            .admit("bridge-ews", Some(&context.scope))
            .await
            .map_err(|_| LoginFailure::new(LoginFailureReason::Policy))?;
        let row = store
            .get_plugin("bridge-ews")
            .await
            .map_err(|_| LoginFailure::new(LoginFailureReason::Storage))?
            .ok_or_else(|| LoginFailure::new(LoginFailureReason::NotConfigured))?;
        let mut plan = crate::v7_mount::plan_load(store, &row, None)
            .await
            .map_err(|_| LoginFailure::new(LoginFailureReason::Policy))?;
        let endpoint = selected
            .config
            .endpoint()
            .map_err(|_| LoginFailure::new(LoginFailureReason::NotConfigured))?;
        let host = endpoint
            .host_str()
            .ok_or_else(|| LoginFailure::new(LoginFailureReason::NotConfigured))?
            .to_string();
        if !plan
            .manifest
            .net_allowlist
            .iter()
            .any(|existing| existing.eq_ignore_ascii_case(&host))
        {
            plan.manifest.net_allowlist.push(host);
        }
        let bytes = crate::v7_mount::resolve_component("bridge-ews", store)
            .await
            .ok_or_else(|| LoginFailure::new(LoginFailureReason::NotConfigured))?;
        let services = HostServices {
            invocation_policy: Some(context.clone()),
            http: context.clone(),
            basic_creds: context.clone(),
            ..HostServices::default()
        };
        let scope = context.scope.clone();
        let handle = tokio::task::spawn_blocking(move || {
            // Compilation cannot be cancelled by dropping its JoinHandle. Keep
            // the admission slot inside this task until compilation finishes,
            // including when the surrounding login deadline has elapsed.
            let permit = permit;
            let host = mw_plugin::PluginHost::try_new(services, mw_plugin::TrustRoot::empty())?;
            let handle = host.load_for_account(&bytes, &plan.manifest, &plan.grant, &scope)?;
            Ok::<_, mw_plugin::PluginError>((host, handle, permit))
        })
        .await
        .map_err(|_| LoginFailure::new(LoginFailureReason::Storage))?
        .map_err(|_| LoginFailure::new(LoginFailureReason::Policy))?;
        let (_host, handle, _permit) = handle;
        let backend = handle
            .as_account_backend()
            .ok_or_else(|| LoginFailure::new(LoginFailureReason::Policy))?;
        let mailboxes = backend
            .list_mailboxes()
            .await
            .map_err(|error| LoginFailure::from_backend(error, None))?;
        if mailboxes.len() > 4096 {
            return Err(LoginFailure::new(LoginFailureReason::Protocol));
        }
        context
            .admit("bridge-ews", Some(&context.scope))
            .await
            .map_err(|_| LoginFailure::new(LoginFailureReason::Policy))?;
        Ok(mailboxes)
    };
    tokio::time::timeout(Duration::from_secs(20), operation)
        .await
        .map_err(|_| LoginFailure::new(LoginFailureReason::Timeout))?
}
