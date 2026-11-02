use mw_store::{ServerKey, SignatureRow, Store};
fn row(name: &str, format: Option<&str>, html: Option<&str>) -> SignatureRow {
    SignatureRow {
        account_id: "owner".into(),
        name: name.into(),
        body: "Independent plain signature".into(),
        format: format.map(str::to_string),
        html: html.map(str::to_string),
        text: Some("Plain alternative".into()),
        text_edited: true,
        is_default: false,
        rule_json: "{}".into(),
        updated_at: String::new(),
    }
}
#[tokio::test]
async fn explicit_formats_round_trip_through_legacy_and_personal_repositories() {
    let store = Store::open_in_memory(ServerKey::generate()).await.unwrap();
    let html = "<table style=\"color: #123456; font-size: 15px\"><tbody><tr><td>Native source</td></tr></tbody></table>";
    for format in ["plain", "rich", "html"] {
        let signature = row(
            format,
            Some(format),
            if format == "plain" { None } else { Some(html) },
        );
        store.upsert_signature(&signature).await.unwrap();
        store.upsert_personal_signature(&signature).await.unwrap();
    }
    for rows in [
        store.list_signatures("owner").await.unwrap(),
        store.list_personal_signatures("owner").await.unwrap(),
    ] {
        assert_eq!(rows.len(), 3);
        for signature in rows {
            assert_eq!(signature.format.as_deref(), Some(signature.name.as_str()));
            assert_eq!(signature.effective_format(), signature.name);
            if signature.name != "plain" {
                assert_eq!(signature.html.as_deref(), Some(html));
            }
        }
    }
    let mut changed = row("rich", Some("html"), Some(html));
    changed.is_default = true;
    store.upsert_personal_signature(&changed).await.unwrap();
    assert_eq!(
        store
            .list_personal_signatures("owner")
            .await
            .unwrap()
            .iter()
            .find(|row| row.name == "rich")
            .unwrap()
            .format
            .as_deref(),
        Some("html")
    );
    let mut invalid = row("invalid", Some("markdown"), Some(html));
    invalid.is_default = true;
    assert!(store.upsert_signature(&invalid).await.is_err());
    assert!(store.upsert_personal_signature(&invalid).await.is_err());
    assert!(
        store
            .list_personal_signatures("owner")
            .await
            .unwrap()
            .iter()
            .any(|row| row.name == "rich" && row.is_default)
    );
}
#[tokio::test]
async fn legacy_html_infers_html_mode_while_plain_remains_plain() {
    let store = Store::open_in_memory(ServerKey::generate()).await.unwrap();
    store
        .upsert_personal_signature(&row(
            "legacy-html",
            None,
            Some("<div style=\"color: red\">Authored HTML</div>"),
        ))
        .await
        .unwrap();
    store
        .upsert_personal_signature(&row("legacy-plain", None, None))
        .await
        .unwrap();
    let rows = store.list_personal_signatures("owner").await.unwrap();
    assert_eq!(rows[0].format, None);
    assert_eq!(rows[0].effective_format(), "html");
    assert_eq!(rows[1].format, None);
    assert_eq!(rows[1].effective_format(), "plain");
}
