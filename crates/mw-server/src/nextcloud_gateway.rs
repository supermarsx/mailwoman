//! Resolve a gateway from the authenticated account for each operation.
use crate::{
    AppState,
    nextcloud::{NextcloudEntry, NextcloudError, NextcloudGateway, ShareLink, ShareLinkReq},
    nextcloud_transport::Operation,
};
use async_trait::async_trait;
use axum::{
    Json,
    http::{HeaderMap, StatusCode},
    response::{IntoResponse, Response},
};
use serde_json::json;
use std::sync::Arc;

struct BoundGateway {
    state: AppState,
    headers: HeaderMap,
    account_id: String,
    instance_id: String,
}

pub(crate) async fn resolve(
    state: &AppState,
    headers: &HeaderMap,
    id: Option<&str>,
) -> Result<Arc<dyn NextcloudGateway>, Response> {
    let session =
        crate::user_router::feature_session(state, headers, "integrations.nextcloud").await?;
    let links = state
        .store
        .nextcloud_links(&session.account_id)
        .await
        .map_err(|_| StatusCode::SERVICE_UNAVAILABLE.into_response())?;
    let chosen = match id {
        Some(id) => links.iter().find(|row| row.instance_id == id),
        None if links.len() == 1 => links.first(),
        _ => None,
    };
    let Some(chosen) = chosen else {
        return Err((
            StatusCode::CONFLICT,
            Json(json!({"error":"Link a Nextcloud account and select an instance."})),
        )
            .into_response());
    };
    Ok(Arc::new(BoundGateway {
        state: state.clone(),
        headers: headers.clone(),
        account_id: session.account_id,
        instance_id: chosen.instance_id.clone(),
    }))
}
fn error(reason: &str) -> NextcloudError {
    NextcloudError::Api(reason.into())
}
impl BoundGateway {
    async fn authority(
        &self,
    ) -> Result<
        (
            mw_store::Session,
            mw_store::nextcloud_links::NextcloudInstance,
            mw_store::nextcloud_links::NextcloudLink,
            String,
        ),
        NextcloudError,
    > {
        let session = crate::user_router::feature_session(
            &self.state,
            &self.headers,
            "integrations.nextcloud",
        )
        .await
        .map_err(|_| error("account or integration unavailable"))?;
        if session.account_id != self.account_id {
            return Err(error("account changed"));
        }
        let (instance, link, password) = self
            .state
            .store
            .nextcloud_authority(&self.account_id, &self.instance_id)
            .await
            .map_err(|_| error("link unavailable"))?
            .ok_or_else(|| error("link again after instance or credential changes"))?;
        Ok((session, instance, link, password))
    }
    async fn file_call(
        &self,
        path: &str,
        method: mw_egress::DavMethod,
        body: Option<(&str, &[u8])>,
    ) -> Result<Vec<u8>, NextcloudError> {
        let (session, instance, link, password) = self.authority().await?;
        let url = crate::nextcloud_config::file_url(&instance.base_url, &link.username, path)
            .map_err(error)?;
        let result = crate::nextcloud_transport::call(
            &self.state,
            &session,
            &instance,
            &link.username,
            &password,
            Operation {
                url,
                method,
                body,
                depth: None,
            },
        )
        .await;
        self.finish(result, &link).await
    }
    async fn finish(
        &self,
        result: Result<Vec<u8>, &'static str>,
        link: &mw_store::nextcloud_links::NextcloudLink,
    ) -> Result<Vec<u8>, NextcloudError> {
        if matches!(result, Err("credentials rejected; link again")) {
            let _ = self
                .state
                .store
                .delete_nextcloud_link(&self.account_id, &self.instance_id, &link.revision)
                .await;
        }
        result.map_err(error)
    }
}
#[async_trait]
impl NextcloudGateway for BoundGateway {
    async fn fetch(&self, path: &str) -> Result<Vec<u8>, NextcloudError> {
        self.file_call(path, mw_egress::DavMethod::Get, None).await
    }
    async fn save(&self, path: &str, bytes: &[u8]) -> Result<(), NextcloudError> {
        self.file_call(
            path,
            mw_egress::DavMethod::Put,
            Some(("application/octet-stream", bytes)),
        )
        .await
        .map(|_| ())
    }
    async fn list(&self, path: &str) -> Result<Vec<NextcloudEntry>, NextcloudError> {
        let (session, instance, link, password) = self.authority().await?;
        let root = crate::nextcloud_config::file_url(&instance.base_url, &link.username, "/")
            .map_err(error)?;
        let url = crate::nextcloud_config::file_url(&instance.base_url, &link.username, path)
            .map_err(error)?;
        let result = crate::nextcloud_transport::call(
            &self.state,
            &session,
            &instance,
            &link.username,
            &password,
            Operation {
                url: url.clone(),
                method: mw_egress::DavMethod::Propfind,
                body: Some(("application/xml", crate::nextcloud_transport::PROPERTIES)),
                depth: Some(1),
            },
        )
        .await;
        let bytes = self.finish(result, &link).await?;
        Ok(crate::nextcloud_xml::entries(&bytes, &root, &url)
            .map_err(error)?
            .into_iter()
            .map(|entry| NextcloudEntry {
                path: entry.path,
                name: entry.name,
                is_dir: entry.is_dir,
                size: entry.size,
            })
            .collect())
    }
    async fn create_share_link(&self, req: &ShareLinkReq) -> Result<ShareLink, NextcloudError> {
        let (session, instance, link, password) = self.authority().await?;
        crate::nextcloud_config::file_url(&instance.base_url, &link.username, &req.path)
            .map_err(error)?;
        if req.password.as_ref().is_some_and(|value| value.len() > 256) {
            return Err(error("share password too long"));
        }
        if let Some(date) = &req.expire_date {
            chrono::NaiveDate::parse_from_str(date, "%Y-%m-%d")
                .map_err(|_| error("use a YYYY-MM-DD expiry"))?;
        }
        let base = crate::nextcloud_config::base_url(&instance.base_url).map_err(error)?;
        let url = base
            .join("ocs/v2.php/apps/files_sharing/api/v1/shares?format=json")
            .map_err(|_| error("invalid instance"))?;
        let mut encoded = reqwest::Url::parse("https://form.invalid/").expect("constant URL");
        {
            let mut form = encoded.query_pairs_mut();
            form.append_pair("path", &req.path)
                .append_pair("shareType", "3")
                .append_pair("permissions", "1");
            if let Some(password) = &req.password {
                form.append_pair("password", password);
            }
            if let Some(date) = &req.expire_date {
                form.append_pair("expireDate", date);
            }
        }
        let result = crate::nextcloud_transport::call(
            &self.state,
            &session,
            &instance,
            &link.username,
            &password,
            Operation {
                url,
                method: mw_egress::DavMethod::Post,
                body: Some((
                    "application/x-www-form-urlencoded",
                    encoded.query().unwrap_or_default().as_bytes(),
                )),
                depth: None,
            },
        )
        .await;
        let bytes = self.finish(result, &link).await?;
        let response: serde_json::Value =
            serde_json::from_slice(&bytes).map_err(|_| error("invalid share response"))?;
        if !matches!(
            response["ocs"]["meta"]["statuscode"].as_u64(),
            Some(100 | 200)
        ) {
            return Err(error("share refused"));
        }
        let url = response["ocs"]["data"]["url"]
            .as_str()
            .and_then(|url| reqwest::Url::parse(url).ok())
            .ok_or_else(|| error("invalid share URL"))?;
        if url.origin() != base.origin() || !url.username().is_empty() || url.password().is_some() {
            return Err(error("share URL escaped instance"));
        }
        Ok(ShareLink { url: url.into() })
    }
}
