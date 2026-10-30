//! Versioned, account-bound UI preferences. Appearance retains its existing
//! policy-aware authority in the parent module; this document cannot override it.
use super::*;
use std::collections::{BTreeMap, HashSet};

const FOLDER_ICONS: &[&str] = &[
    "folder",
    "star",
    "flag",
    "tag",
    "clock",
    "calendar",
    "file",
    "image",
    "lock",
    "users",
    "archive",
    "mail",
    "attachment",
    "check",
    "alert",
    "inbox",
    "sent",
    "edit",
    "delete",
    "spam",
    "search",
    "pin",
    "link",
    "shield",
    "unlock",
    "fileAudio",
    "fileVideo",
    "filePdf",
    "fileCode",
    "print",
    "download",
    "preferences",
    "dashboard",
    "analytics",
    "nextcloud",
    "domains",
    "twofactor",
    "assistant",
    "integrations",
    "logs",
    "home",
    "briefcase",
    "heart",
    "shopping",
    "education",
    "travel",
    "coffee",
    "gift",
    "bank",
    "book",
];

const MAX_BYTES: usize = 65_536;

#[derive(Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Tag {
    name: String,
    color: String,
    icon: String,
}

#[derive(Default, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(super) struct Preferences {
    #[serde(skip_serializing_if = "Option::is_none")]
    analytics_consent: Option<bool>,
    /// t63: consent to one browser analytics tag, `<provider>:<loader sha256>`,
    /// or empty. Separate from `analytics_consent` (server-side counts only), so
    /// a provider change or a new loader asks again.
    #[serde(skip_serializing_if = "Option::is_none")]
    analytics_tag_consent: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    keyboard_preset: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    workspace_layout: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    folder_sidebar_mode: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    folder_sidebar_width: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    message_list_width: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    message_list_height: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    reading_pane: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    message_list_collapsed: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    reader_collapsed: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    workspace_split_ratio: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    direction: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    mark_read: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    mark_read_delay_ms: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    offline_budget_mb: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    offline_retention_days: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    eviction: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    folder_pinned: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    folder_hidden: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    user_folders_collapsed: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    folder_order: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    folder_icons: Option<BTreeMap<String, String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    tags: Option<BTreeMap<String, Tag>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    folder_labels: Option<BTreeMap<String, Vec<String>>>,
}

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct Document {
    version: u8,
    preferences: Preferences,
}

#[derive(Serialize, Deserialize)]
struct Stored {
    revision: String,
    document: Document,
}

fn key(account: &str) -> String {
    personal_storage::key("ui-preferences", account)
}
fn valid_id(id: &str) -> bool {
    !id.is_empty() && id.len() <= 256 && !id.chars().any(char::is_control)
}
fn choices(value: &Option<String>, allowed: &[&str]) -> bool {
    value.as_ref().is_none_or(|v| allowed.contains(&v.as_str()))
}
fn valid_ids(value: &Option<Vec<String>>) -> bool {
    value.as_ref().is_none_or(|ids| {
        ids.len() <= 500
            && ids.iter().all(|id| valid_id(id))
            && ids.iter().collect::<HashSet<_>>().len() == ids.len()
    })
}
fn valid(document: &Document) -> bool {
    let p = &document.preferences;
    document.version == 1
        && p.analytics_tag_consent
            .as_deref()
            .is_none_or(crate::analytics_tag::valid_consent_key)
        && choices(
            &p.keyboard_preset,
            &["default", "gmail", "outlook", "vim", "off"],
        )
        && choices(
            &p.workspace_layout,
            &[
                "tabs",
                "side-by-side",
                "stack",
                "masonry-side-by-side",
                "masonry-stack",
            ],
        )
        && choices(&p.folder_sidebar_mode, &["expanded", "compact", "hidden"])
        && p.folder_sidebar_width
            .is_none_or(|v| (160..=600).contains(&v))
        && choices(&p.reading_pane, &["right", "bottom", "off"])
        && p.message_list_height
            .is_none_or(|v| (160..=1200).contains(&v))
        && p.message_list_width
            .is_none_or(|v| (240..=1200).contains(&v))
        && p.workspace_split_ratio
            .is_none_or(|v| v.is_finite() && (0.2..=0.8).contains(&v))
        && choices(&p.direction, &["auto", "ltr", "rtl"])
        && choices(&p.mark_read, &["on-open", "never"])
        && choices(&p.eviction, &["lru", "oldest", "manual"])
        && p.mark_read_delay_ms.is_none_or(|v| v <= 60_000)
        && p.offline_budget_mb.is_none_or(|v| v <= 100_000)
        && p.offline_retention_days.is_none_or(|v| v <= 36_500)
        && valid_ids(&p.folder_pinned)
        && valid_ids(&p.folder_hidden)
        && valid_ids(&p.folder_order)
        && p.folder_labels.as_ref().is_none_or(|folders| {
            folders.len() <= 500
                && folders.iter().all(|(id, labels)| {
                    valid_id(id)
                        && labels.len() <= 32
                        && labels.iter().collect::<HashSet<_>>().len() == labels.len()
                        && labels.iter().all(|keyword| {
                            !keyword.is_empty()
                                && keyword.len() <= 255
                                && keyword.bytes().all(|byte| {
                                    byte.is_ascii_alphanumeric() || b"_-".contains(&byte)
                                })
                        })
                })
        })
        && p.tags.as_ref().is_none_or(|tags| {
            tags.len() <= 100
                && tags.iter().all(|(id, tag)| {
                    !id.is_empty()
                        && id.len() <= 255
                        && id
                            .bytes()
                            .all(|byte| byte.is_ascii_alphanumeric() || b"_-".contains(&byte))
                        && !tag.name.trim().is_empty()
                        && tag.name.chars().count() <= 128
                        && !tag.name.chars().any(char::is_control)
                        && tag.color.len() == 7
                        && tag.color.starts_with('#')
                        && tag.color[1..].bytes().all(|byte| byte.is_ascii_hexdigit())
                        && FOLDER_ICONS.contains(&tag.icon.as_str())
                })
        })
        && p.folder_icons.as_ref().is_none_or(|icons| {
            icons.len() <= 500
                && icons
                    .iter()
                    .all(|(id, icon)| valid_id(id) && FOLDER_ICONS.contains(&icon.as_str()))
        })
}

fn response(document: Document, revision: &str) -> Response {
    let mut response = Json(document).into_response();
    response
        .headers_mut()
        .insert("cache-control", "private, no-store".parse().unwrap());
    response
        .headers_mut()
        .insert("etag", format!("\"{revision}\"").parse().unwrap());
    response
}

pub(super) async fn load(State(state): State<AppState>, headers: HeaderMap) -> Response {
    let session = match session_of(&state, &headers).await {
        Ok(v) => v,
        Err(r) => return r,
    };
    let account = match personal_storage::owner(&state, &session, &headers, false).await {
        Ok(owner) => owner,
        Err(response) => return response,
    };
    match state.store.get_setting(&key(&account)).await {
        Ok(Some(raw)) => match serde_json::from_str::<Stored>(&raw) {
            Ok(stored) => response(stored.document, &stored.revision),
            Err(_) => StatusCode::INTERNAL_SERVER_ERROR.into_response(),
        },
        Ok(None) => response(
            Document {
                version: 1,
                preferences: Preferences::default(),
            },
            "0",
        ),
        Err(_) => StatusCode::INTERNAL_SERVER_ERROR.into_response(),
    }
}

pub(super) async fn save(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(document): Json<Document>,
) -> Response {
    let session = match session_of(&state, &headers).await {
        Ok(v) => v,
        Err(r) => return r,
    };
    let account = match personal_storage::owner(&state, &session, &headers, true).await {
        Ok(owner) => owner,
        Err(response) => return response,
    };
    if !valid(&document) {
        return bad_request("invalid UI preferences");
    }
    if serde_json::to_vec(&document).map_or(true, |v| v.len() > MAX_BYTES) {
        return payload_too_large("UI preferences are too large");
    }
    let Some(expected) = headers.get("if-match").and_then(|v| v.to_str().ok()) else {
        return StatusCode::PRECONDITION_REQUIRED.into_response();
    };
    let raw = match state.store.get_setting(&key(&account)).await {
        Ok(v) => v,
        Err(_) => return StatusCode::INTERNAL_SERVER_ERROR.into_response(),
    };
    let revision = match raw
        .as_ref()
        .map(|v| serde_json::from_str::<Stored>(v))
        .transpose()
    {
        Ok(Some(v)) => v.revision,
        Ok(None) => "0".into(),
        Err(_) => return StatusCode::INTERNAL_SERVER_ERROR.into_response(),
    };
    if expected != format!("\"{revision}\"") {
        return StatusCode::PRECONDITION_FAILED.into_response();
    }
    let stored = Stored {
        revision: format!("{:032x}", rand::random::<u128>()),
        document,
    };
    let serialized = serde_json::to_string(&stored).expect("preference document serializes");
    match state
        .store
        .compare_and_set_setting(&key(&account), raw.as_deref(), &serialized)
        .await
    {
        Ok(true) => response(stored.document, &stored.revision),
        Ok(false) => StatusCode::PRECONDITION_FAILED.into_response(),
        Err(_) => StatusCode::INTERNAL_SERVER_ERROR.into_response(),
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn five_workspace_modes_are_valid_but_unknown_layouts_are_rejected() {
        for layout in [
            "tabs",
            "side-by-side",
            "stack",
            "masonry-side-by-side",
            "masonry-stack",
        ] {
            let document: super::Document = serde_json::from_value(
                serde_json::json!({"version":1,"preferences":{"workspaceLayout":layout}}),
            )
            .unwrap();
            assert!(super::valid(&document), "{layout}");
        }
        let document: super::Document = serde_json::from_value(
            serde_json::json!({"version":1,"preferences":{"workspaceLayout":"unsupported"}}),
        )
        .unwrap();
        assert!(!super::valid(&document));
    }
    #[test]
    fn all_fifty_native_icons_are_valid_account_preferences() {
        assert_eq!(super::FOLDER_ICONS.len(), 50);
        for icon in super::FOLDER_ICONS {
            let document: super::Document = serde_json::from_value(serde_json::json!({"version":1,"preferences":{"folderIcons":{"inbox":icon},"tags":{"work":{"name":"Work","color":"#112233","icon":icon}}}})).unwrap();
            assert!(super::valid(&document), "{icon}");
        }
    }
    use super::*;
    #[test]
    fn refuses_unknown_authority_secret_and_invalid_fields() {
        for value in [
            json!({"accountId":"other"}),
            json!({"password":"secret"}),
            json!({"appearance":{}}),
        ] {
            assert!(
                serde_json::from_value::<Document>(json!({"version":1,"preferences":value}))
                    .is_err()
            );
        }
        for prefs in [
            json!({"markReadDelayMs":60001}),
            json!({"folderOrder":["a","a"]}),
            json!({"folderIcons":{"a":"https://avatar.invalid"}}),
            json!({"keyboardPreset":"arbitrary"}),
            json!({"tags":{"$seen":{"name":"System","color":"#112233","icon":"tag"}}}),
            json!({"tags":{"work":{"name":"Work","color":"url(https://invalid)","icon":"tag"}}}),
            json!({"tags":{"work":{"name":"Work","color":"#112233","icon":"https://invalid"}}}),
        ] {
            assert!(!valid(
                &serde_json::from_value(json!({"version":1,"preferences":prefs})).unwrap()
            ));
        }
        assert!(valid(
            &serde_json::from_value(
                json!({"version":1,"preferences":{"folderIcons":{"a":"star"},"markRead":"never"}})
            )
            .unwrap()
        ));
        assert!(valid(&serde_json::from_value(json!({"version":1,"preferences":{"tags":{"work":{"name":"Trabalho","color":"#112233","icon":"tag"}}}})).unwrap()));
    }
}
