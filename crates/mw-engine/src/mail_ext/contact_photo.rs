//! Local embedded contact photos only; never an external image lookup.
use crate::{backend::Result, engine::Engine};
use mw_store::receipts::normalize_sender;

impl Engine {
    /// Returns an encoded PHOTO content line only after exact owned EMAIL matching.
    pub async fn contact_photo(&self, account: &str, address: &str) -> Result<Option<String>> {
        let address = normalize_sender(address)?;
        for raw in self.store().contact_photo_cards(account, &address).await? {
            let Ok(cards) = mw_ics::parse_vcard(raw.as_bytes()) else {
                continue;
            };
            for card in cards {
                let matched = card.json["emails"].as_array().is_some_and(|emails| {
                    emails
                        .iter()
                        .filter_map(|email| email["value"].as_str())
                        .any(|value| normalize_sender(value).is_ok_and(|value| value == address))
                });
                if !matched {
                    continue;
                }
                if let Some(lines) = card.json["vcardExtra"].as_array() {
                    for line in lines.iter().filter_map(|value| value.as_str()) {
                        let head = line
                            .split([';', ':'])
                            .next()
                            .unwrap_or_default()
                            .rsplit('.')
                            .next()
                            .unwrap_or_default();
                        if head.eq_ignore_ascii_case("PHOTO") {
                            return Ok(Some(line.to_owned()));
                        }
                    }
                }
            }
        }
        Ok(None)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    async fn exercise(store: mw_store::Store) {
        let mut accounts = Vec::new();
        for suffix in ["owner", "other"] {
            let username = format!(
                "photo-{}-{suffix}@example.org",
                std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_nanos()
            );
            accounts.push(
                store
                    .create_account(
                        &mw_store::NewAccount {
                            kind: mw_store::AccountKind::Imap,
                            host: "localhost",
                            port: 993,
                            tls: "implicit",
                            username: &username,
                            sync_policy_json: "{}",
                        },
                        &mw_store::Credentials {
                            username: username.clone(),
                            password: "test".into(),
                        },
                    )
                    .await
                    .unwrap(),
            );
        }
        let book = format!("photo-book-{}", accounts[0]);
        store
            .upsert_address_book(&mw_store::AddressBookRow {
                id: book.clone(),
                account_id: accounts[0].clone(),
                name: "Photos".into(),
                is_default: true,
                carddav_url: None,
                sync_token: None,
                ctag: None,
            })
            .await
            .unwrap();
        let mut card=mw_store::ContactRow{id:format!("photo-card-{}",accounts[0]),address_book_id:book,uid:"photo".into(),etag:None,vcard_raw:"BEGIN:VCARD\r\nVERSION:3.0\r\nFN:Ann\r\nEMAIL:ann@example.org\r\nNOTE:wrong@example.org\r\nPHOTO;ENCODING=b;TYPE=PNG:AAAA\r\nEND:VCARD\r\n".into(),json:None,full_name:"Ann".into(),is_favorite:false,photo_blob_id:None,pgp_key:None,smime_cert:None};
        store.upsert_contact(&card).await.unwrap();
        let engine = Engine::new(store.clone());
        assert!(
            engine
                .contact_photo(&accounts[0], "ann@EXAMPLE.org")
                .await
                .unwrap()
                .unwrap()
                .starts_with("PHOTO")
        );
        assert!(
            engine
                .contact_photo(&accounts[1], "ann@example.org")
                .await
                .unwrap()
                .is_none()
        );
        assert!(
            engine
                .contact_photo(&accounts[0], "wrong@example.org")
                .await
                .unwrap()
                .is_none()
        );
        card.vcard_raw.push_str(&"é".repeat(140000));
        store.upsert_contact(&card).await.unwrap();
        assert!(
            engine
                .contact_photo(&accounts[0], "ann@example.org")
                .await
                .unwrap()
                .is_none()
        );
    }
    #[tokio::test]
    async fn photos_require_exact_owned_contact_and_bounded_bytes() {
        exercise(
            mw_store::Store::open_in_memory(mw_store::ServerKey::generate())
                .await
                .unwrap(),
        )
        .await;
        if let Ok(url) = std::env::var("MW_TEST_PG") {
            exercise(
                mw_store::Store::open_postgres(&url, mw_store::ServerKey::generate())
                    .await
                    .unwrap(),
            )
            .await;
        }
    }
}
