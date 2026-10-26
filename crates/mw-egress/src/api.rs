//! Fixed-provider credential-bearing requests. No caller-controlled header names.
use crate::Refusal;
use std::fmt;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ApiMethod {
    Get,
    Post,
    Delete,
}
impl ApiMethod {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Get => "GET",
            Self::Post => "POST",
            Self::Delete => "DELETE",
        }
    }
}
#[derive(Clone, Copy)]
pub struct AuthorizedApi<'a> {
    pub method: ApiMethod,
    pub key: &'a str,
}
impl fmt::Debug for AuthorizedApi<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("AuthorizedApi")
            .field("method", &self.method)
            .finish_non_exhaustive()
    }
}
impl AuthorizedApi<'_> {
    pub fn validate(&self) -> Result<(), Refusal> {
        if self.key.is_empty()
            || self.key.len() > 4096
            || !self.key.bytes().all(|b| b.is_ascii_graphic())
        {
            return Err(Refusal::BadRequest("invalid package provider key"));
        }
        Ok(())
    }
    pub fn validate_target(&self, url: &reqwest::Url) -> Result<(), Refusal> {
        self.validate()?;
        if url.scheme() != "https"
            || url.host_str() != Some("api.aftership.com")
            || url.port_or_known_default() != Some(443)
            || !url.username().is_empty()
            || url.password().is_some()
            || url.fragment().is_some()
            || !(url.path() == "/tracking/2026-07" || url.path().starts_with("/tracking/2026-07/"))
        {
            return Err(Refusal::BadRequest("package provider authority refused"));
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        OutBody, TrafficClass,
        allow::InternalAllowList,
        plan::{EgressPlan, Request, send_with},
    };
    #[test]
    fn provider_credential_and_authority_are_narrow_and_redacted() {
        let auth = AuthorizedApi {
            method: ApiMethod::Get,
            key: "private-fixture-key",
        };
        assert!(!format!("{auth:?}").contains("private-fixture-key"));
        for bad in [
            "https://api.aftership.com.evil.test/tracking/2026-07/a",
            "http://api.aftership.com/tracking/2026-07/a",
            "https://api.aftership.com:444/tracking/2026-07/a",
            "https://u@api.aftership.com/tracking/2026-07/a",
            "https://api.aftership.com/other",
            "https://api.aftership.com/tracking/2026-07/a#x",
        ] {
            assert!(
                auth.validate_target(&reqwest::Url::parse(bad).unwrap())
                    .is_err()
            );
        }
        assert!(
            auth.validate_target(
                &reqwest::Url::parse("https://api.aftership.com/tracking/2026-07/trackings")
                    .unwrap()
            )
            .is_ok()
        );
        assert!(
            AuthorizedApi {
                method: ApiMethod::Get,
                key: "bad\r\nHeader: x"
            }
            .validate()
            .is_err()
        );
        assert!(!TrafficClass::PackageTracking.internal_allow_eligible());
        assert!(
            Request::package_api(
                ApiMethod::Get,
                "key",
                Some(OutBody {
                    content_type: "application/json",
                    bytes: b"{}"
                })
            )
            .is_err()
        );
    }
    #[tokio::test]
    async fn wrong_provider_is_refused_before_any_connection() {
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let plan = EgressPlan::new(vec![], InternalAllowList::default()).unwrap();
        let request = Request::package_api(ApiMethod::Delete, "key", None).unwrap();
        let result = send_with(
            &plan,
            reqwest::Url::parse(&format!(
                "https://{}/tracking/2026-07/a",
                listener.local_addr().unwrap()
            ))
            .unwrap(),
            &request,
        )
        .await;
        assert!(matches!(result.outcome, Err(Refusal::BadRequest(_))));
        assert!(
            tokio::time::timeout(std::time::Duration::from_millis(30), listener.accept())
                .await
                .is_err()
        );
    }
}
