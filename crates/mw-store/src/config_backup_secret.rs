//! The app password of the administrator's Nextcloud configuration-backup
//! destination, sealed with the server key and bound to the server URL and
//! username it was entered for, so a blob copied into another configuration
//! does not open.
use crate::{Store, StoreError};

const PURPOSE: &str = "config-backup-nextcloud-v1";

impl Store {
    pub fn seal_config_backup_secret(
        &self,
        secret: &[u8],
        base_url: &str,
        username: &str,
    ) -> Result<Vec<u8>, StoreError> {
        Ok(self
            .key
            .seal_bound(secret, PURPOSE, &[base_url, username])?)
    }
    pub fn open_config_backup_secret(
        &self,
        sealed: &[u8],
        base_url: &str,
        username: &str,
    ) -> Result<Vec<u8>, StoreError> {
        Ok(self
            .key
            .open_bound(sealed, PURPOSE, &[base_url, username])?)
    }
}

#[cfg(test)]
mod tests {
    #[tokio::test]
    async fn sealed_secret_opens_only_for_its_url_and_user() {
        let store = crate::Store::open_in_memory(crate::ServerKey::generate())
            .await
            .unwrap();
        let sealed = store
            .seal_config_backup_secret(b"app-pass", "https://cloud.test/", "backup")
            .unwrap();
        assert!(!sealed.windows(8).any(|w| w == b"app-pass"));
        assert_eq!(
            store
                .open_config_backup_secret(&sealed, "https://cloud.test/", "backup")
                .unwrap(),
            b"app-pass"
        );
        assert!(
            store
                .open_config_backup_secret(&sealed, "https://other.test/", "backup")
                .is_err()
        );
        assert!(
            store
                .open_config_backup_secret(&sealed, "https://cloud.test/", "admin")
                .is_err()
        );
    }
}
