//! Bounded host-approved plugin HTTP operations. No ambient headers or redirects.

use crate::Refusal;

/// The server must verify the plugin/account and destination authority before construction.
#[derive(Clone, Copy)]
pub struct PluginHttp<'a> {
    pub method: &'a str,
    pub headers: &'a [(String, String)],
    /// Administrator-owned roots for the exact origin, never for a proxy hop.
    pub additional_roots: &'a [Vec<u8>],
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
        if self.additional_roots.len() > 8
            || self.additional_roots.iter().map(Vec::len).sum::<usize>() > 65536
        {
            return Err(Refusal::BadRequest("plugin CA bundle too large"));
        }
        let mut roots = rustls::RootCertStore::empty();
        for certificate in self.additional_roots {
            roots
                .add(rustls_pki_types::CertificateDer::from(certificate.clone()))
                .map_err(|_| Refusal::BadRequest("invalid plugin CA"))?;
        }
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
    fn origin_ca_roots_are_bounded_and_validated() {
        let invalid = vec![vec![1, 2, 3]];
        assert!(
            PluginHttp {
                method: "POST",
                headers: &[],
                additional_roots: &invalid
            }
            .validate()
            .is_err()
        );
        let excessive = vec![Vec::new(); 9];
        assert!(
            PluginHttp {
                method: "POST",
                headers: &[],
                additional_roots: &excessive
            }
            .validate()
            .is_err()
        );
        let large = vec![vec![0; 65537]];
        assert!(
            PluginHttp {
                method: "POST",
                headers: &[],
                additional_roots: &large
            }
            .validate()
            .is_err()
        );
    }

    #[test]
    fn credentials_are_redacted_and_authority_headers_cannot_be_overridden() {
        let headers = vec![("Authorization".into(), "Bearer secret-token".into())];
        let request = PluginHttp {
            additional_roots: &[],
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
                    additional_roots: &[],
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
                additional_roots: &[],
                method: "GET",
                headers: &duplicate
            }
            .validate()
            .is_err()
        );
        let injected = vec![("Accept".into(), "text/plain\r\nHost: foreign".into())];
        assert!(
            PluginHttp {
                additional_roots: &[],
                method: "GET",
                headers: &injected
            }
            .validate()
            .is_err()
        );
    }
}
