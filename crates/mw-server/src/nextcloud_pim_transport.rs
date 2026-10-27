//! Per-operation Nextcloud PIM authority; no retained credentials or AppState cycle.
use mw_dav::{DavError, DavTransport, TransportRequest, TransportResponse};
use mw_store::Store;
use reqwest::Url;
use std::sync::Arc;

pub(crate) struct ManagedTransport {
    store: Store,
    account: String,
    discovery: Option<(String, String, String)>,
}
fn refused(message: &'static str) -> DavError {
    DavError::Transport(message.into())
}
impl ManagedTransport {
    pub(crate) fn runtime(store: Store, account: String) -> Arc<dyn DavTransport> {
        Arc::new(Self {
            store,
            account,
            discovery: None,
        })
    }
    pub(crate) fn discovery(
        store: Store,
        account: String,
        instance: String,
        revision: String,
        kind: String,
    ) -> Arc<dyn DavTransport> {
        Arc::new(Self {
            store,
            account,
            discovery: Some((instance, revision, kind)),
        })
    }
    async fn execute(&self, request: TransportRequest) -> mw_dav::Result<TransportResponse> {
        use base64::Engine as _;
        request.validate()?;
        let gate = crate::account_gate::for_account(&self.store, &self.account)
            .await
            .map_err(|_| refused("account policy unavailable"))?;
        if gate.disabled || gate.password_change_required {
            return Err(refused("account unavailable"));
        }
        let address = self
            .store
            .security_account_address(&self.account)
            .await
            .map_err(|_| refused("account unavailable"))?
            .ok_or_else(|| refused("account identity unavailable"))?;
        let canonical = self
            .store
            .canonical_policy_address(&address)
            .await
            .map_err(|_| refused("account identity unavailable"))?;
        let subject = mw_store::settings_policy::SettingSubject {
            personal_owner: None,
            account_id: &self.account,
            address: &canonical,
            domain: canonical.rsplit_once('@').map(|(_, d)| d).unwrap_or(""),
        };
        for key in ["cap.integration.nextcloud"] {
            let value = crate::settings_registry::effective_for(&self.store, key, &subject)
                .await
                .map_err(|_| refused("PIM policy unavailable"))?;
            if !value.is_some_and(|v| v.value.as_bool() == Some(true)) {
                return Err(refused("PIM integration disabled"));
            }
        }
        let url = Url::parse(&request.url).map_err(|_| refused("invalid DAV URL"))?;
        let (instance_id, revision, kind) = if let Some(binding) = &self.discovery {
            if request.method != "PROPFIND" {
                return Err(refused("discovery is read only"));
            }
            binding.clone()
        } else {
            let bindings = self
                .store
                .active_nextcloud_pim_bindings(&self.account)
                .await
                .map_err(|_| refused("bindings unavailable"))?;
            let mut matching = bindings.into_iter().filter(|binding| {
                binding.enabled
                    && request.kind == Some(binding.kind.as_str())
                    && within(&url, &binding.href)
            });
            let first = matching
                .next()
                .ok_or_else(|| refused("collection is disconnected"))?;
            if matching.any(|binding| {
                binding.instance_id != first.instance_id
                    || binding.link_revision != first.link_revision
            }) {
                return Err(refused("ambiguous collection authority"));
            }
            (first.instance_id, first.link_revision, first.kind)
        };
        let capability = match kind.as_str() {
            "calendar" => "cap.pim.calendar",
            "tasks" => "cap.pim.tasks",
            "contacts" => "cap.pim.contacts",
            _ => return Err(refused("invalid PIM kind")),
        };
        if !crate::settings_registry::effective_for(&self.store, capability, &subject)
            .await
            .map_err(|_| refused("PIM policy unavailable"))?
            .is_some_and(|v| v.value.as_bool() == Some(true))
        {
            return Err(refused("PIM feature disabled"));
        }
        let (instance, link, password) = self
            .store
            .nextcloud_authority(&self.account, &instance_id)
            .await
            .map_err(|_| refused("cloud authority unavailable"))?
            .ok_or_else(|| refused("cloud link revoked"))?;
        if link.revision != revision {
            return Err(refused("cloud link changed; apply discovery again"));
        }
        if !account_target(
            &url,
            &instance.base_url,
            &link.username,
            self.discovery.is_some(),
        )? {
            return Err(refused("DAV target escaped account"));
        }
        let method = match request.method.as_str() {
            "GET" => mw_egress::DavMethod::Get,
            "PROPFIND" => mw_egress::DavMethod::Propfind,
            "REPORT" => mw_egress::DavMethod::Report,
            "PUT" => mw_egress::DavMethod::Put,
            "DELETE" => mw_egress::DavMethod::Delete,
            "MKCALENDAR" => mw_egress::DavMethod::MkCalendar,
            // Scheduling writes to principals/outboxes require a separate explicit permission.
            _ => return Err(refused("unsupported managed DAV operation")),
        };
        let authorization = format!(
            "Basic {}",
            base64::engine::general_purpose::STANDARD
                .encode(format!("{}:{password}", link.username))
        );
        let roots = crate::nextcloud_config::ca_roots(&instance.ca_pem).map_err(refused)?;
        let metadata = mw_egress::ResponseMetadata::default();
        let plan = crate::image_proxy::load_plan(&self.store)
            .await
            .map_err(|_| refused("egress unavailable"))?;
        let mut outgoing = mw_egress::plan::Request::dav(
            mw_egress::AuthorizedDav {
                additional_roots: &roots,
                method,
                authorization: &authorization,
                depth: request
                    .depth
                    .as_deref()
                    .map(|d| if d == "1" { 1 } else { 0 }),
                ocs: false,
                if_match: request.if_match.as_deref(),
                if_none_match: request.if_none_match.as_deref(),
            },
            request.body.as_ref().map(|body| mw_egress::OutBody {
                content_type: request.content_type.as_deref().unwrap_or("application/xml"),
                bytes: body.as_bytes(),
            }),
        )
        .map_err(|_| refused("invalid DAV request"))?
        .capture_response(&metadata)
        .response_limit(mw_dav::transport::MAX_DOCUMENT_BYTES);
        if !subject.domain.is_empty() {
            outgoing = outgoing.for_domain(subject.domain);
        }
        let result = mw_egress::plan::send_with(&plan, url.clone(), &outgoing)
            .await
            .outcome;
        let status = match &result {
            Err(mw_egress::Refusal::Status(status)) => *status,
            _ => metadata.snapshot().map(|head| head.status).unwrap_or(0),
        };
        if status == 401 {
            let _ = self
                .store
                .delete_nextcloud_link(&self.account, &instance_id, &revision)
                .await;
        }
        let bytes = match result {
            Ok(bytes) => bytes,
            Err(mw_egress::Refusal::Status(_)) => Vec::new(),
            Err(_) => return Err(refused("DAV transport refused")),
        };
        let body = String::from_utf8(bytes).map_err(|_| refused("DAV response is not UTF-8"))?;
        if body.trim_start().starts_with('<') {
            validate_xml(&body)?;
            if self.discovery.is_none() && matches!(request.method.as_str(), "REPORT" | "PROPFIND")
            {
                for resource in mw_dav::response::parse_resource_list(&body)? {
                    let href = url
                        .join(&resource.href)
                        .map_err(|_| refused("invalid DAV resource href"))?;
                    if href != url && !within(&href, url.as_str()) {
                        return Err(refused("DAV response escaped collection"));
                    }
                    if !account_target(&href, &instance.base_url, &link.username, false)? {
                        return Err(refused("DAV response escaped account"));
                    }
                }
            }
        }
        Ok(TransportResponse {
            status,
            etag: metadata.snapshot().and_then(|head| head.etag),
            body,
        })
    }
}
impl DavTransport for ManagedTransport {
    fn send(&self, request: TransportRequest) -> mw_dav::transport::TransportFuture<'_> {
        Box::pin(async move {
            static SLOTS: std::sync::OnceLock<tokio::sync::Semaphore> = std::sync::OnceLock::new();
            let _slot = SLOTS
                .get_or_init(|| tokio::sync::Semaphore::new(4))
                .try_acquire()
                .map_err(|_| refused("PIM busy"))?;
            tokio::time::timeout(std::time::Duration::from_secs(15), self.execute(request))
                .await
                .map_err(|_| refused("DAV timeout"))?
        })
    }
}

pub(crate) fn collection_url(base: &str, user: &str, kind: &str) -> mw_dav::Result<Url> {
    let mut url = crate::nextcloud_config::base_url(base).map_err(refused)?;
    crate::nextcloud_config::username(user).map_err(refused)?;
    {
        let mut path = url
            .path_segments_mut()
            .map_err(|_| refused("invalid instance"))?;
        path.pop_if_empty().extend(["remote.php", "dav"]);
        match kind {
            "calendar" | "tasks" => {
                path.extend(["calendars", user, ""]);
            }
            "contacts" => {
                path.extend(["addressbooks", "users", user, ""]);
            }
            "principal" => {
                path.extend(["principals", "users", user, ""]);
            }
            _ => return Err(refused("invalid collection kind")),
        }
    }
    Ok(url)
}
fn within(url: &Url, base: &str) -> bool {
    Url::parse(base).is_ok_and(|base| {
        url.origin() == base.origin()
            && url
                .path()
                .starts_with(&format!("{}/", base.path().trim_end_matches('/')))
    })
}
pub(crate) fn account_target(
    url: &Url,
    base: &str,
    user: &str,
    discovery: bool,
) -> mw_dav::Result<bool> {
    if url.query().is_some()
        || url.fragment().is_some()
        || !url.username().is_empty()
        || url.password().is_some()
    {
        return Ok(false);
    }
    let lower = url.path().to_ascii_lowercase();
    if ["%2f", "%5c", "%2e", "%25"]
        .iter()
        .any(|token| lower.contains(token))
    {
        return Ok(false);
    }
    for kind in ["calendar", "contacts"] {
        if within(url, collection_url(base, user, kind)?.as_str()) {
            return Ok(true);
        }
    }
    if discovery && *url == collection_url(base, user, "principal")? {
        return Ok(true);
    }
    Ok(false)
}
fn validate_xml(body: &str) -> mw_dav::Result<()> {
    let mut reader = quick_xml::Reader::from_str(body);
    let mut depth = 0usize;
    let mut events = 0usize;
    loop {
        events += 1;
        if events > 100_000 {
            return Err(refused("DAV XML event limit"));
        }
        match reader
            .read_event()
            .map_err(|_| refused("invalid DAV XML"))?
        {
            quick_xml::events::Event::DocType(_) => {
                return Err(refused("DAV XML document types refused"));
            }
            quick_xml::events::Event::Start(_) => {
                depth += 1;
                if depth > 32 {
                    return Err(refused("DAV XML depth limit"));
                }
            }
            quick_xml::events::Event::End(_) => {
                depth = depth
                    .checked_sub(1)
                    .ok_or_else(|| refused("invalid DAV XML nesting"))?;
            }
            quick_xml::events::Event::Eof => {
                return if depth == 0 {
                    Ok(())
                } else {
                    Err(refused("invalid DAV XML nesting"))
                };
            }
            _ => {}
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn collection_targets_cannot_escape_account_origin_or_installation() {
        let base = "https://cloud.test/nextcloud/";
        let own = collection_url(base, "Jóse", "calendar").unwrap();
        assert!(account_target(&own, base, "Jóse", false).unwrap());
        for target in [
            "https://elsewhere.test/nextcloud/remote.php/dav/calendars/J%C3%B3se/",
            "https://cloud.test/nextcloud/remote.php/dav/calendars/other/",
            "https://cloud.test/remote.php/dav/calendars/J%C3%B3se/",
            "https://cloud.test/nextcloud/remote.php/dav/calendars/J%C3%B3se/%252e%252e/other/",
            "https://cloud.test/nextcloud/remote.php/dav/calendars/J%C3%B3se/work/?redirect=other",
        ] {
            assert!(
                !account_target(&Url::parse(target).unwrap(), base, "Jóse", false).unwrap(),
                "{target}"
            );
        }
        let principal = collection_url(base, "Jóse", "principal").unwrap();
        assert!(!account_target(&principal, base, "Jóse", false).unwrap());
        assert!(account_target(&principal, base, "Jóse", true).unwrap());
    }
    #[test]
    fn xml_external_entities_and_deep_documents_are_refused() {
        assert!(
            validate_xml("<!DOCTYPE d [<!ENTITY e SYSTEM 'file:///secret'>]><d>&e;</d>").is_err()
        );
        assert!(validate_xml(&format!("{}{}", "<d>".repeat(33), "</d>".repeat(33))).is_err());
        assert!(validate_xml("<d><x/></d>").is_ok());
    }
}
