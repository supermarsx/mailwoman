//! One credential-bearing operation through the current guarded egress plan.
use crate::AppState;
use base64::Engine as _;
use mw_egress::{AuthorizedDav, DavMethod, OutBody};
use mw_store::{Session, nextcloud_links::NextcloudInstance};

pub(crate) struct Operation<'a> {
    pub url: reqwest::Url,
    pub method: DavMethod,
    pub body: Option<(&'a str, &'a [u8])>,
    pub depth: Option<u8>,
}
pub(crate) async fn call(
    state: &AppState,
    session: &Session,
    instance: &NextcloudInstance,
    username: &str,
    password: &str,
    op: Operation<'_>,
) -> Result<Vec<u8>, &'static str> {
    let Operation {
        url,
        method,
        body,
        depth,
    } = op;
    static SLOTS: std::sync::OnceLock<tokio::sync::Semaphore> = std::sync::OnceLock::new();
    let operation = async {
        let _permit = SLOTS
            .get_or_init(|| tokio::sync::Semaphore::new(4))
            .try_acquire()
            .map_err(|_| "cloud busy")?;
        let subject = crate::settings_routes::UserSubject::resolved(&state.store, session)
            .await
            .map_err(|_| "account policy")?;
        crate::user_router::require_feature(state, &subject, "integrations.nextcloud")
            .await
            .map_err(|_| "integration disabled")?;
        let current = state
            .store
            .nextcloud_instance(&instance.id)
            .await
            .map_err(|_| "storage")?
            .ok_or("instance unavailable")?;
        if !current.enabled || current.revision != instance.revision {
            return Err("instance changed; link again");
        }
        let canonical = match state
            .store
            .security_account_address(&session.account_id)
            .await
            .map_err(|_| "storage")?
        {
            Some(address) => state
                .store
                .canonical_policy_address(&address)
                .await
                .map_err(|_| "account policy")?,
            None => return Err("account policy"),
        };
        let domain = canonical.rsplit_once('@').map(|(_, domain)| domain);
        let base = crate::nextcloud_config::base_url(&instance.base_url)?;
        if url.origin() != base.origin() || !url.path().starts_with(base.path()) {
            return Err("invalid instance target");
        }
        let plan = crate::image_proxy::load_plan(&state.store)
            .await
            .map_err(|_| "egress unavailable")?;
        let authorization = format!(
            "Basic {}",
            base64::engine::general_purpose::STANDARD.encode(format!("{username}:{password}"))
        );
        let additional_roots = crate::nextcloud_config::ca_roots(&instance.ca_pem)?;
        let mut request = mw_egress::plan::Request::dav(
            AuthorizedDav {
                additional_roots: &additional_roots,
                method,
                authorization: &authorization,
                depth,
                ocs: true,
                if_none_match: None,
                if_match: None,
            },
            body.map(|(content_type, bytes)| OutBody {
                content_type,
                bytes,
            }),
        )
        .map_err(|_| "invalid cloud request")?
        .response_limit(mw_egress::MAX_IMAGE_BYTES);
        if let Some(domain) = domain {
            request = request.for_domain(domain);
        }
        let response = mw_egress::plan::send_with(&plan, url, &request).await;
        response.outcome.map_err(|error| match error {
            mw_egress::Refusal::Status(401) => "credentials rejected; link again",
            mw_egress::Refusal::Status(403) => "cloud permission denied",
            mw_egress::Refusal::TooLarge => "file or response exceeds 8 MiB limit",
            mw_egress::Refusal::Timeout => "cloud timeout",
            _ => "cloud transport refused",
        })
    };
    tokio::time::timeout(std::time::Duration::from_secs(15), operation)
        .await
        .map_err(|_| "cloud timeout")?
}

pub(crate) const PROPERTIES:&[u8]=br#"<?xml version="1.0"?><d:propfind xmlns:d="DAV:"><d:prop><d:resourcetype/><d:getcontentlength/><d:getlastmodified/><d:getcontenttype/></d:prop></d:propfind>"#;

pub(crate) async fn verify(
    state: &AppState,
    session: &Session,
    instance: &NextcloudInstance,
    user: &str,
    password: &str,
) -> Result<(), &'static str> {
    let url = crate::nextcloud_config::file_url(&instance.base_url, user, "/")?;
    let bytes = call(
        state,
        session,
        instance,
        user,
        password,
        Operation {
            url: url.clone(),
            method: DavMethod::Propfind,
            body: Some(("application/xml", PROPERTIES)),
            depth: Some(0),
        },
    )
    .await?;
    crate::nextcloud_xml::entries(&bytes, &url, &url)?;
    Ok(())
}
