//! Opt-in bounded response metadata for conditional DAV operations and download progress.

use std::sync::Mutex;

/// Captures bounded concurrency metadata and accepted-body byte counters. Cookies,
/// authentication challenges and arbitrary upstream headers are never retained.
#[derive(Debug, Default)]
pub struct ResponseMetadata {
    value: Mutex<Option<ResponseHead>>,
    progress: Mutex<Option<ResponseProgress>>,
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
