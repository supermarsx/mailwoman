//! Opt-in bounded response metadata for conditional DAV operations and download progress.

use std::sync::Mutex;

/// Captures bounded concurrency metadata and accepted-body byte counters. Cookies,
/// arbitrary upstream headers are never retained. Plugin HTTP can separately
/// capture bounded authentication challenges for a host-owned handshake.
#[derive(Default)]
pub struct ResponseMetadata {
    value: Mutex<Option<ResponseHead>>,
    progress: Mutex<Option<ResponseProgress>>,
    cache: Mutex<Option<CacheHeaders>>,
    authentication: Mutex<Vec<String>>,
}
impl std::fmt::Debug for ResponseMetadata {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ResponseMetadata").finish_non_exhaustive()
    }
}

/// The two caching headers a revalidating cache needs, retained alongside the
/// `ETag`: `Last-Modified` and `Cache-Control`. Each is bounded and printable
/// ASCII; nothing else from the response is kept.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct CacheHeaders {
    pub last_modified: Option<String>,
    pub cache_control: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResponseHead {
    pub status: u16,
    pub etag: Option<String>,
}

/// Bytes accepted by the bounded body reader, never request headers or URLs.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ResponseProgress {
    pub received_bytes: u64,
    pub total_bytes: Option<u64>,
}

impl ResponseMetadata {
    pub fn authentication_challenges(&self) -> Vec<String> {
        self.authentication
            .lock()
            .map(|value| value.clone())
            .unwrap_or_default()
    }
    pub(crate) fn record_authentication(&self, headers: &reqwest::header::HeaderMap) {
        let mut total = 0;
        let mut values = headers
            .get_all(reqwest::header::WWW_AUTHENTICATE)
            .iter()
            .filter_map(|value| value.to_str().ok())
            .filter(|value| {
                total += value.len();
                value.len() <= 16384
                    && total <= 32768
                    && value.bytes().all(|b| (0x20..0x7f).contains(&b))
            })
            .take(8)
            .map(str::to_owned)
            .collect();
        if let Ok(mut held) = self.authentication.lock() {
            std::mem::swap(&mut *held, &mut values);
        }
    }
    pub fn progress(&self) -> Option<ResponseProgress> {
        *self.progress.lock().ok()?
    }

    pub(crate) fn record_progress(&self, received_bytes: u64, total_bytes: Option<u64>) {
        if let Ok(mut progress) = self.progress.lock() {
            *progress = Some(ResponseProgress {
                received_bytes,
                total_bytes,
            });
        }
    }

    /// The caching headers of the last response, if any were captured.
    pub fn cache_headers(&self) -> Option<CacheHeaders> {
        self.cache.lock().ok()?.clone()
    }

    pub(crate) fn record_cache_headers(
        &self,
        last_modified: Option<&str>,
        cache_control: Option<&str>,
    ) {
        let keep = |v: &str| v.len() <= 512 && v.bytes().all(|b| (0x20..0x7f).contains(&b));
        if let Ok(mut value) = self.cache.lock() {
            *value = Some(CacheHeaders {
                last_modified: last_modified.filter(|v| keep(v)).map(str::to_owned),
                cache_control: cache_control.filter(|v| keep(v)).map(str::to_owned),
            });
        }
    }

    pub fn snapshot(&self) -> Option<ResponseHead> {
        self.value.lock().ok()?.clone()
    }

    pub(crate) fn record(&self, status: u16, etag: Option<&str>) {
        if let Ok(mut value) = self.value.lock() {
            *value = Some(ResponseHead {
                status,
                etag: etag
                    .filter(|tag| tag.len() <= 4096 && !tag.bytes().any(|b| b < 0x20 || b == 0x7f))
                    .map(str::to_owned),
            });
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn authentication_metadata_is_bounded_separate_and_redacted() {
        let metadata = ResponseMetadata::default();
        let mut headers = reqwest::header::HeaderMap::new();
        headers.append(
            reqwest::header::WWW_AUTHENTICATE,
            "Negotiate".parse().unwrap(),
        );
        headers.append(
            reqwest::header::WWW_AUTHENTICATE,
            "NTLM challenge-secret".parse().unwrap(),
        );
        headers.append(
            reqwest::header::SET_COOKIE,
            "do-not-forward=secret".parse().unwrap(),
        );
        metadata.record_authentication(&headers);
        assert_eq!(
            metadata.authentication_challenges(),
            ["Negotiate", "NTLM challenge-secret"]
        );
        assert!(!format!("{metadata:?}").contains("challenge-secret"));
        headers.clear();
        headers.append(
            reqwest::header::WWW_AUTHENTICATE,
            "x".repeat(16385).parse().unwrap(),
        );
        metadata.record_authentication(&headers);
        assert!(metadata.authentication_challenges().is_empty());
    }

    #[test]
    fn captures_conflict_status_without_accepting_unbounded_or_injected_headers() {
        let metadata = ResponseMetadata::default();
        assert!(metadata.snapshot().is_none());
        metadata.record(412, Some("\"remote-revision\""));
        assert_eq!(
            metadata.snapshot(),
            Some(ResponseHead {
                status: 412,
                etag: Some("\"remote-revision\"".into()),
            })
        );
        metadata.record(200, Some(&"x".repeat(4097)));
        assert_eq!(metadata.snapshot().unwrap().etag, None);
        metadata.record(200, Some("revision\r\nCookie: secret"));
        assert_eq!(metadata.snapshot().unwrap().etag, None);
    }
}
