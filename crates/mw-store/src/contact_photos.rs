//! Bounded local contact evidence. Remote PHOTO URLs are never fetched here.
use crate::{
    Store, StoreError,
    backend::{Dialect, q},
};

impl Store {
    pub async fn contact_photo_cards(
        &self,
        account: &str,
        address: &str,
    ) -> Result<Vec<String>, StoreError> {
        let address = crate::receipts::normalize_sender(address)?;
        let pattern = format!(
            "%{}%",
            address
                .replace('\\', "\\\\")
                .replace('%', "\\%")
                .replace('_', "\\_")
        );
        let sql = match self.backend.dialect() {
            Dialect::Sqlite => {
                "SELECT c.vcard_raw FROM contacts c JOIN address_books b ON b.id=c.address_book_id WHERE b.account_id=?1 AND length(CAST(c.vcard_raw AS BLOB))<=262144 AND c.vcard_raw LIKE ?2 ESCAPE '\\' ORDER BY c.id LIMIT 16"
            }
            Dialect::Postgres => {
                "SELECT c.vcard_raw FROM contacts c JOIN address_books b ON b.id=c.address_book_id WHERE b.account_id=?1 AND octet_length(c.vcard_raw)<=262144 AND c.vcard_raw ILIKE ?2 ESCAPE '\\' ORDER BY c.id LIMIT 16"
            }
        };
        Ok(q(sql)
            .bind(account)
            .bind(pattern)
            .fetch_all(&self.backend)
            .await?
            .into_iter()
            .map(|row| row.get_string("vcard_raw"))
            .collect())
    }
}
