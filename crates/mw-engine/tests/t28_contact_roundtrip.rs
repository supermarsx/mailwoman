//! t28-e9: a contact keeps what it was imported with.
//!
//! Every test drives `Engine::handle_jmap` — `ContactCard/import`, `/set`,
//! `/get`, `/query`, `/merge` — and then reads the **stored row**
//! (`Store::get_contact`): `vcard_raw` is what a CardDAV push and an export
//! send, and `photo_blob_id` is a column no response used to show. Asserting on
//! the emitter alone would not see a save path that drops a field before it
//! gets there.

use std::sync::Arc;

use async_trait::async_trait;
use serde_json::{Value, json};

use mw_engine::account::AccountRuntime;
use mw_engine::backend::{
    AccountBackend, BackendCaps, ChangeSink, EngineError, Flag, MailboxDelta, MessageRef,
    MoveOutcome, RawMailbox, RawMailboxRef, RawMessage, Result, SyncCursor, WatchHandle,
};
use mw_engine::{Engine, MailSubmitter};
use mw_smtp::{Outgoing, SubmissionResult};
use mw_store::{AccountKind, ContactRow, Credentials, NewAccount, ServerKey, Store};

// ── harness (the `tests/pim.rs` one: no mail, no DAV) ─────────────────────────

struct NoopBackend;

#[async_trait]
impl AccountBackend for NoopBackend {
    async fn capabilities(&self) -> Result<BackendCaps> {
        Ok(BackendCaps::default())
    }
    async fn list_mailboxes(&self) -> Result<Vec<RawMailbox>> {
        Ok(Vec::new())
    }
    async fn sync_mailbox(&self, _m: &RawMailboxRef, c: &SyncCursor) -> Result<MailboxDelta> {
        Ok(MailboxDelta {
            added: Vec::new(),
            flag_changes: Vec::new(),
            removed: Vec::new(),
            next_cursor: c.clone(),
        })
    }
    async fn fetch_raw(&self, _refs: &[MessageRef]) -> Result<Vec<RawMessage>> {
        Ok(Vec::new())
    }
    async fn store_flags(&self, _r: &[MessageRef], _a: &[Flag], _d: &[Flag]) -> Result<()> {
        Ok(())
    }
    async fn move_messages(&self, _r: &[MessageRef], _to: &RawMailboxRef) -> Result<MoveOutcome> {
        Err(EngineError::Unsupported("noop".into()))
    }
    async fn append(&self, _m: &RawMailboxRef, _raw: &[u8], _f: &[Flag]) -> Result<MessageRef> {
        Err(EngineError::Unsupported("noop".into()))
    }
    async fn watch(&self, _sink: ChangeSink) -> Result<WatchHandle> {
        Err(EngineError::Unsupported("noop".into()))
    }
}

struct NoopSubmitter;

#[async_trait]
impl MailSubmitter for NoopSubmitter {
    async fn submit(&self, msg: Outgoing) -> Result<SubmissionResult> {
        Ok(SubmissionResult {
            accepted: msg.rcpt_to,
            rejected: Vec::new(),
        })
    }
}

struct Harness {
    engine: Arc<Engine>,
    account_id: String,
}

async fn setup() -> Harness {
    let store = Store::open_in_memory(ServerKey::generate()).await.unwrap();
    let account_id = store
        .create_account(
            &NewAccount {
                kind: AccountKind::Imap,
                host: "imap.example.org",
                port: 993,
                tls: "implicit",
                username: "me@example.org",
                sync_policy_json: "{}",
            },
            &Credentials {
                username: "me@example.org".into(),
                password: "pw".into(),
            },
        )
        .await
        .unwrap();
    let engine = Arc::new(Engine::new(store));
    engine.register_backend(
        account_id.clone(),
        AccountRuntime::new(
            Arc::new(NoopBackend) as Arc<dyn AccountBackend>,
            Arc::new(NoopSubmitter) as Arc<dyn MailSubmitter>,
            "me@example.org",
        ),
    );
    Harness { engine, account_id }
}

impl Harness {
    async fn call(&self, method: &str, args: Value) -> Value {
        let req = json!({ "methodCalls": [[method, args, "c0"]] });
        let resp = self.engine.handle_jmap(&self.account_id, &req).await;
        resp["methodResponses"][0][1].clone()
    }

    /// Import one card and return its id.
    async fn import(&self, vcf: &str) -> String {
        let imp = self
            .call("ContactCard/import", json!({ "blob": vcf }))
            .await;
        assert_eq!(imp["count"], 1, "import refused: {imp}");
        imp["imported"][0].as_str().unwrap().to_string()
    }

    async fn update(&self, id: &str, patch: Value) {
        let set = self
            .call("ContactCard/set", json!({ "update": { id: patch } }))
            .await;
        assert!(
            set["updated"].as_object().unwrap().contains_key(id),
            "update refused: {set}"
        );
    }

    async fn card(&self, id: &str) -> Value {
        let got = self.call("ContactCard/get", json!({ "ids": [id] })).await;
        assert_eq!(got["list"].as_array().unwrap().len(), 1, "{got}");
        got["list"][0].clone()
    }

    async fn row(&self, id: &str) -> ContactRow {
        self.engine
            .store()
            .get_contact(id)
            .await
            .unwrap()
            .unwrap_or_else(|| panic!("no stored contact {id}"))
    }
}

/// The content lines of a stored card.
fn lines(vcard_raw: &str) -> Vec<&str> {
    vcard_raw.lines().filter(|l| !l.is_empty()).collect()
}

const PHOTO_LINE: &str = "PHOTO:data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAYAAAAfFcSJAAAADUlEQVR42mNkYPhfDwAChwGA60e6kgAAAABJRU5ErkJggg==";

fn ada_vcf() -> String {
    format!(
        "BEGIN:VCARD\r\n\
         VERSION:4.0\r\n\
         UID:ada-1\r\n\
         FN:Ada Lovelace\r\n\
         N:Lovelace;Ada;Augusta;;\r\n\
         EMAIL;TYPE=work:ada@example.org\r\n\
         BDAY:18151210\r\n\
         ADR;TYPE=home:;;12 St James's Square;London;;SW1Y 4JH;United Kingdom\r\n\
         {PHOTO_LINE}\r\n\
         X-MAILWOMAN-TEST:kept as written\r\n\
         END:VCARD\r\n"
    )
}

const ADA_ADR: &str = "ADR;TYPE=home:;;12 St James's Square;London;;SW1Y 4JH;United Kingdom";

// ── import → update → read back ──────────────────────────────────────────────

#[tokio::test]
async fn an_update_keeps_the_birthday_the_address_the_photo_and_the_photo_column() {
    let h = setup().await;
    let id = h.import(&ada_vcf()).await;

    // Precondition: the import itself stored all of it. Without this the
    // "still there after the update" assertions below would pass on a card
    // that never had them.
    let stored = h.row(&id).await;
    let before = lines(&stored.vcard_raw);
    for want in [
        "BDAY:18151210",
        ADA_ADR,
        PHOTO_LINE,
        "X-MAILWOMAN-TEST:kept as written",
        "N:Lovelace;Ada;Augusta;;",
    ] {
        assert!(
            before.contains(&want),
            "import lost {want:?}\n{}",
            stored.vcard_raw
        );
    }
    assert_eq!(stored.photo_blob_id, None);

    // The client attaches a photo blob, then edits an unrelated field.
    h.update(&id, json!({ "photoBlobId": "blob-photo-1" }))
        .await;
    assert_eq!(
        h.row(&id).await.photo_blob_id.as_deref(),
        Some("blob-photo-1")
    );
    assert_eq!(h.card(&id).await["photoBlobId"], "blob-photo-1");

    h.update(&id, json!({ "nicknames": ["Countess"] })).await;

    let stored = h.row(&id).await;
    assert_eq!(
        stored.photo_blob_id.as_deref(),
        Some("blob-photo-1"),
        "an update that did not name photoBlobId erased the column"
    );
    let after = lines(&stored.vcard_raw);
    for want in [
        "NICKNAME:Countess",
        "BDAY:18151210",
        ADA_ADR,
        PHOTO_LINE,
        "X-MAILWOMAN-TEST:kept as written",
        "N:Lovelace;Ada;Augusta;;",
    ] {
        assert!(
            after.contains(&want),
            "update lost {want:?}\n{}",
            stored.vcard_raw
        );
    }

    let card = h.card(&id).await;
    assert_eq!(card["nicknames"], json!(["Countess"]));
    assert_eq!(card["photoBlobId"], "blob-photo-1");
    assert_eq!(
        card["anniversaries"],
        json!([{ "kind": "birthday", "date": "1815-12-10" }])
    );
    assert_eq!(
        card["addresses"],
        json!([{
            "context": "home",
            "pobox": "",
            "ext": "",
            "street": "12 St James's Square",
            "locality": "London",
            "region": "",
            "postcode": "SW1Y 4JH",
            "country": "United Kingdom",
        }])
    );

    // The export is the stored card.
    let exported = h.call("ContactCard/export", json!({ "ids": [id] })).await;
    assert!(
        exported["blob"].as_str().unwrap().contains(PHOTO_LINE),
        "{exported}"
    );

    // And the column can still be cleared on purpose.
    h.update(&id, json!({ "photoBlobId": null })).await;
    assert_eq!(h.row(&id).await.photo_blob_id, None);
    assert_eq!(h.card(&id).await["photoBlobId"], Value::Null);
}

#[tokio::test]
async fn a_group_card_is_stored_as_a_group() {
    let h = setup().await;
    let id = h
        .import(
            "BEGIN:VCARD\r\nVERSION:4.0\r\nUID:club-1\r\nKIND:group\r\nFN:The Engine Club\r\n\
             MEMBER:urn:uuid:03a0e51f-d1aa-4385-8a53-e29025acd8af\r\nEND:VCARD\r\n",
        )
        .await;
    h.update(&id, json!({ "notes": "meets on Tuesdays" })).await;
    let stored = h.row(&id).await;
    let after = lines(&stored.vcard_raw);
    assert!(after.contains(&"KIND:group"), "{}", stored.vcard_raw);
    assert!(
        after.contains(&"MEMBER:urn:uuid:03a0e51f-d1aa-4385-8a53-e29025acd8af"),
        "{}",
        stored.vcard_raw
    );
    assert_eq!(h.card(&id).await["kind"], "group");
}

// ── the carried-over lines are the engine's, not the client's ────────────────

#[tokio::test]
async fn the_carried_lines_are_not_sent_to_a_client_nor_taken_from_one() {
    let h = setup().await;
    let id = h.import(&ada_vcf()).await;

    // Not in a response: an inline photo would ride along with every list.
    let card = h.card(&id).await;
    let text = card.to_string();
    assert!(!text.contains("iVBORw0KGgo"), "{text}");
    assert!(!text.contains("X-MAILWOMAN-TEST"), "{text}");

    // Not from an update: neither an empty list (which would erase the photo)
    // nor a list of the client's own lines.
    h.update(
        &id,
        json!({ "vcardExtra": [], "titles": ["Mathematician"] }),
    )
    .await;
    h.update(&id, json!({ "vcardExtra": ["X-SMUGGLED:1"] }))
        .await;
    let stored = h.row(&id).await;
    assert!(
        lines(&stored.vcard_raw).contains(&PHOTO_LINE),
        "{}",
        stored.vcard_raw
    );
    assert!(
        stored.vcard_raw.contains("TITLE:Mathematician"),
        "{}",
        stored.vcard_raw
    );
    assert!(
        !stored.vcard_raw.contains("X-SMUGGLED"),
        "{}",
        stored.vcard_raw
    );

    // Not from a create.
    let set = h
        .call(
            "ContactCard/set",
            json!({ "create": { "c": {
                "name": { "full": "Mallory" },
                "vcardExtra": ["X-SMUGGLED:1"],
            }}}),
        )
        .await;
    let created = set["created"]["c"]["id"].as_str().unwrap().to_string();
    let stored = h.row(&created).await;
    assert!(
        stored.vcard_raw.contains("FN:Mallory"),
        "{}",
        stored.vcard_raw
    );
    assert!(
        !stored.vcard_raw.contains("X-SMUGGLED"),
        "{}",
        stored.vcard_raw
    );
}

// ── a hostile .vcf ───────────────────────────────────────────────────────────

/// Each of these values aborted the request inside the vCard reader's URI
/// parser: on import, and again when the card just written was read back.
#[tokio::test]
async fn a_card_whose_values_look_like_broken_uris_is_imported() {
    let h = setup().await;
    let id = h
        .import(
            "BEGIN:VCARD\r\nVERSION:4.0\r\nUID:a@b:c\r\nFN:Mallory\r\nTEL:+1:x\r\nKEY:a@b:c\r\n\
             IMPP:a@b:c\r\nEND:VCARD\r\n",
        )
        .await;
    let card = h.card(&id).await;
    assert_eq!(card["name"]["full"], "Mallory");
    assert_eq!(card["uid"], "a@b:c");
    assert_eq!(card["phones"][0]["value"], "+1:x");

    // The same values arriving through `ContactCard/set` reach the same reader.
    h.update(
        &id,
        json!({
            "phones": [{ "context": "work:x;y", "value": "+2:y" }],
            "pgpKey": "b@c:d",
        }),
    )
    .await;
    let stored = h.row(&id).await;
    let card = h.card(&id).await;
    assert_eq!(card["phones"].as_array().unwrap().len(), 1, "{card}");
    assert_eq!(card["phones"][0]["value"], "+2:y");
    assert_eq!(card["phones"][0]["context"], "work:x;y");
    assert_eq!(
        lines(&stored.vcard_raw)
            .iter()
            .filter(|l| l.starts_with("TEL"))
            .count(),
        1,
        "{}",
        stored.vcard_raw
    );
}

// ── merge and query see the new fields ───────────────────────────────────────

#[tokio::test]
async fn a_merge_keeps_the_addresses_and_the_photo_of_both_cards() {
    let h = setup().await;
    let keep = h.import(&ada_vcf()).await;
    let other = h
        .import(
            "BEGIN:VCARD\r\nVERSION:4.0\r\nUID:ada-2\r\nFN:A. Lovelace\r\n\
             EMAIL:countess@example.org\r\n\
             ADR;TYPE=work:;;1 Engine Way;Cambridge;;CB1 1AA;United Kingdom\r\n\
             ANNIVERSARY:18350708\r\n\
             X-OTHER:from the second card\r\n\
             END:VCARD\r\n",
        )
        .await;
    let merged = h
        .call(
            "ContactCard/merge",
            json!({ "keepId": keep, "mergeIds": [other] }),
        )
        .await;
    assert_eq!(merged["keptId"], keep, "{merged}");
    assert!(
        !merged.to_string().contains("iVBORw0KGgo"),
        "the merge response carries the photo"
    );

    let stored = h.row(&keep).await;
    let after = lines(&stored.vcard_raw);
    for want in [
        ADA_ADR,
        "ADR;TYPE=work:;;1 Engine Way;Cambridge;;CB1 1AA;United Kingdom",
        "BDAY:18151210",
        "ANNIVERSARY:18350708",
        PHOTO_LINE,
        "X-MAILWOMAN-TEST:kept as written",
        "X-OTHER:from the second card",
    ] {
        assert!(
            after.contains(&want),
            "merge lost {want:?}\n{}",
            stored.vcard_raw
        );
    }
}

#[tokio::test]
async fn a_text_query_does_not_match_inside_the_photo() {
    let h = setup().await;
    let id = h.import(&ada_vcf()).await;
    // Precondition: the photo is stored, so there is something to match.
    assert!(h.row(&id).await.vcard_raw.contains("iVBORw0KGgo"));

    let hit = h
        .call(
            "ContactCard/query",
            json!({ "filter": { "text": "james" } }),
        )
        .await;
    assert_eq!(hit["ids"], json!([id]), "{hit}");
    let miss = h
        .call(
            "ContactCard/query",
            json!({ "filter": { "text": "ivborw0kggo" } }),
        )
        .await;
    assert_eq!(miss["ids"], json!([]), "{miss}");

    // Compose recipient completion reads the same stored card.
    let hit = h
        .call("ContactCard/autocomplete", json!({ "prefix": "ada" }))
        .await;
    assert_eq!(hit["list"].as_array().unwrap().len(), 1, "{hit}");
    let miss = h
        .call("ContactCard/autocomplete", json!({ "prefix": "ivborw" }))
        .await;
    assert_eq!(miss["list"], json!([]), "{miss}");
}
