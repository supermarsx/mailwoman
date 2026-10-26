//! Bounded host-approved plugin HTTP operations. No ambient headers or redirects.

use crate::Refusal;

/// The server must verify the plugin/account and destination authority before construction.
#[derive(Clone, Copy)]
pub struct PluginHttp<'a> {
    pub method: &'a str,
    pub headers: &'a [(String, String)],
}

impl std::fmt::Debug for PluginHttp<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PluginHttp")
            .field("method", &self.method)
            .finish_non_exhaustive()
    }
}

impl PluginHttp<'_> {
    pub fn validate(&self) -> Result<(), Refusal> {
        if !matches!(
            self.method,
            "GET" | "HEAD" | "POST" | "PUT" | "PATCH" | "DELETE" | "OPTIONS"
        ) {
            return Err(Refusal::BadRequest("unsupported plugin HTTP method"));
        }
        if self.headers.len() > 16
            || self
                .headers
                .iter()
                .map(|(k, v)| k.len() + v.len())
                .sum::<usize>()
                > 32768
        {
            return Err(Refusal::BadRequest("plugin HTTP headers too large"));
        }
        let mut seen = std::collections::BTreeSet::new();
        for (name, value) in self.headers {
            let name = name.to_ascii_lowercase();
            if !matches!(
                name.as_str(),
                "authorization"
                    | "content-type"
                    | "accept"
                    | "soapaction"
                    | "prefer"
                    | "consistencylevel"
                    | "if-match"
                    | "if-none-match"
                    | "range"
            ) || !seen.insert(name)
                || value.len() > 16384
                || reqwest::header::HeaderValue::from_str(value).is_err()
            {
                return Err(Refusal::BadRequest("invalid plugin HTTP header"));
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn credentials_are_redacted_and_authority_headers_cannot_be_overridden() {
        let headers = vec![("Authorization".into(), "Bearer secret-token".into())];
        let request = PluginHttp {
            method: "POST",
            headers: &headers,
        };
        request.validate().unwrap();
        assert!(!format!("{request:?}").contains("secret-token"));
        for name in [
            "Host",
            "Cookie",
            "Proxy-Authorization",
            "Content-Length",
            "Connection",
            "Transfer-Encoding",
            "X-Forwarded-For",
            "Referer",
        ] {
            let headers = vec![(name.into(), "value".into())];
            assert!(
                PluginHttp {
                    method: "GET",
                    headers: &headers
                }
                .validate()
                .is_err()
            );
        }
        let duplicate = vec![
            ("Authorization".into(), "one".into()),
            ("authorization".into(), "two".into()),
        ];
        assert!(
            PluginHttp {
                method: "GET",
                headers: &duplicate
            }
            .validate()
            .is_err()
        );
        let injected = vec![("Accept".into(), "text/plain\r\nHost: foreign".into())];
        assert!(
            PluginHttp {
                method: "GET",
                headers: &injected
            }
            .validate()
            .is_err()
        );
    }
}
