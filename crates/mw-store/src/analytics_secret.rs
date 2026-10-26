//! Purpose-bound credentials for the optional GA4 collector.
use crate::{Store, StoreError};
impl Store {
    pub fn seal_analytics_secret(
        &self,
        secret: &[u8],
        endpoint: &str,
        measurement_id: &str,
    ) -> Result<Vec<u8>, StoreError> {
        Ok(self
            .key
            .seal_bound(secret, "analytics-ga4-v1", &[endpoint, measurement_id])?)
    }
    pub fn open_analytics_secret(
        &self,
        sealed: &[u8],
        endpoint: &str,
        measurement_id: &str,
    ) -> Result<Vec<u8>, StoreError> {
        Ok(self
            .key
            .open_bound(sealed, "analytics-ga4-v1", &[endpoint, measurement_id])?)
    }
}
