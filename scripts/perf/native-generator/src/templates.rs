//! Request templates, rebuilt from the same inputs as `fixture-requests.mjs`,
//! `representative-workload.mjs` and `fixture-clients.mjs`. Bodies are built with
//! insertion-ordered JSON objects so that the bytes equal `JSON.stringify` of the
//! Node object literals; `mw-loadgen templates` prints them for comparison.
use crate::plan::{Constants, Principal};
use crate::validate::Validator;
use serde_json::{Map, Value, json};

pub const USING: [&str; 2] = ["urn:ietf:params:jmap:core", "urn:ietf:params:jmap:mail"];

#[derive(Clone, Debug)]
pub struct Spec {
    #[allow(dead_code)]
    pub name: String,
    pub method: &'static str,
    pub path: String,
    /// spec headers after the client headers (`content-type`, read intent)
    pub headers: Vec<(String, String)>,
    pub body: Option<String>,
    pub validator: Validator,
}

fn enc(component: &str) -> String {
    // encodeURIComponent
    let mut out = String::with_capacity(component.len());
    for byte in component.bytes() {
        match byte {
            b'A'..=b'Z'
            | b'a'..=b'z'
            | b'0'..=b'9'
            | b'-'
            | b'_'
            | b'.'
            | b'!'
            | b'~'
            | b'*'
            | b'\''
            | b'('
            | b')' => out.push(byte as char),
            _ => out.push_str(&format!("%{byte:02X}")),
        }
    }
    out
}

fn obj(pairs: Vec<(&str, Value)>) -> Value {
    let mut map = Map::new();
    for (key, value) in pairs {
        map.insert(key.to_owned(), value);
    }
    Value::Object(map)
}
fn obj_owned(pairs: Vec<(String, Value)>) -> Value {
    let mut map = Map::new();
    for (key, value) in pairs {
        map.insert(key, value);
    }
    Value::Object(map)
}
/// `[method, {accountId, ...args}, tag]`
fn call(p: &Principal, method: &str, args: Vec<(&str, Value)>, tag: &str) -> Value {
    let mut map = Map::new();
    map.insert("accountId".into(), Value::String(p.account_id.clone()));
    for (key, value) in args {
        map.insert(key.to_owned(), value);
    }
    json!([method, Value::Object(map), tag])
}
fn batch_body(calls: Vec<Value>) -> String {
    serde_json::to_string(&json!({"using": USING, "methodCalls": calls})).unwrap()
}
fn strings(values: &[String]) -> Value {
    Value::Array(values.iter().cloned().map(Value::String).collect())
}
fn json_header() -> Vec<(String, String)> {
    vec![("content-type".into(), "application/json".into())]
}
fn result_ref(of: &str, path: &str) -> Value {
    json!({"resultOf": of, "name": "Email/query", "path": path})
}
fn tags(calls: &[Value]) -> Vec<(String, String)> {
    calls
        .iter()
        .map(|c| {
            (
                c[0].as_str().unwrap().to_owned(),
                c[2].as_str().unwrap().to_owned(),
            )
        })
        .collect()
}

/// `isNativeMetadataReadIntent` of native-read-intent.mjs.
fn native_read_intent(body: &Value, constants: &Constants) -> bool {
    let c = &constants.read_contract;
    let Some(v) = body.as_object() else {
        return false;
    };
    if v.keys().any(|k| k != "using" && k != "methodCalls") {
        return false;
    }
    let Some(using) = v.get("using").and_then(Value::as_array) else {
        return false;
    };
    if using.len() > 32
        || using
            .iter()
            .any(|k| !k.as_str().is_some_and(|s| s.encode_utf16().count() <= 256))
    {
        return false;
    }
    let Some(calls) = v.get("methodCalls").and_then(Value::as_array) else {
        return false;
    };
    if calls.len() != 1 {
        return false;
    }
    let Some(call) = calls[0].as_array() else {
        return false;
    };
    let bounded = |v: &Value, n: usize| {
        v.as_str()
            .is_some_and(|s| !s.is_empty() && s.encode_utf16().count() <= n && s.len() <= n)
    };
    if call.len() != 3
        || call[0].as_str() != Some("Email/get")
        || !bounded(&call[2], c.max_call_id_bytes)
    {
        return false;
    }
    let Some(a) = call[1].as_object() else {
        return false;
    };
    if a.keys().any(|k| !c.arguments.contains(k))
        || !a
            .get("accountId")
            .is_some_and(|v| bounded(v, c.max_id_bytes))
    {
        return false;
    }
    for k in [
        "mailwomanBodyUse",
        "fetchHTMLBodyValues",
        "fetchTextBodyValues",
        "fetchAllBodyValues",
    ] {
        if a.get(k).is_some_and(|v| v != &Value::Bool(false)) {
            return false;
        }
    }
    let Some(ids) = a.get("ids").and_then(Value::as_array) else {
        return false;
    };
    if ids.is_empty() || ids.len() > c.max_ids || ids.iter().any(|id| !bounded(id, c.max_id_bytes))
    {
        return false;
    }
    let Some(props) = a.get("properties").and_then(Value::as_array) else {
        return false;
    };
    let wanted: std::collections::HashSet<&str> = c.properties.iter().map(String::as_str).collect();
    if props.len() != wanted.len() {
        return false;
    }
    let mut actual = std::collections::HashSet::new();
    for p in props {
        let Some(s) = p.as_str() else { return false };
        actual.insert(s);
    }
    actual.len() == wanted.len()
        && actual.iter().all(|p| wanted.contains(p))
        && serde_json::to_string(body).unwrap().len() <= c.max_body_bytes
}

/// `jmapFixtureRequest`
fn jmap_single(
    p: &Principal,
    name: &str,
    args: Vec<(&str, Value)>,
    native: bool,
    constants: &Constants,
    validator: Validator,
) -> Spec {
    let body = json!({"using": USING, "methodCalls": [call(p, name, args, "load")]});
    let mut headers = json_header();
    if native && native_read_intent(&body, constants) {
        headers.push((
            constants.read_contract.header.clone(),
            constants.read_contract.value.clone(),
        ));
    }
    Spec {
        name: name.into(),
        method: "POST",
        path: "/jmap/api".into(),
        headers,
        body: Some(serde_json::to_string(&body).unwrap()),
        validator,
    }
}

fn post_batch(
    name: &str,
    calls: Vec<Value>,
    validator: impl FnOnce(Vec<(String, String)>) -> Validator,
) -> Spec {
    let t = tags(&calls);
    Spec {
        name: name.into(),
        method: "POST",
        path: "/jmap/api".into(),
        headers: json_header(),
        body: Some(batch_body(calls)),
        validator: validator(t),
    }
}

fn get(name: &str, path: String, validator: Validator) -> Spec {
    Spec {
        name: name.into(),
        method: "GET",
        path,
        headers: Vec::new(),
        body: None,
        validator,
    }
}

fn sv(v: &Option<Value>) -> Value {
    v.clone().unwrap_or(Value::Null)
}
fn key_of(v: &Option<Value>) -> String {
    // A computed key `[`mailboxIds/${x}`]` stringifies x; ids are strings in every fixture.
    match v {
        Some(Value::String(s)) => s.clone(),
        Some(other) => other.to_string(),
        None => "undefined".into(),
    }
}

/// Every template of one principal, by Node shape name. `representative` and
/// `native` follow `prepareFixtureRequests`. Unknown names return None.
pub fn build(
    name: &str,
    p: &Principal,
    constants: &Constants,
    representative: bool,
    native: bool,
) -> Result<Option<Spec>, String> {
    let owner = p.owner().to_owned();
    let ids = &p.message_ids;
    let spec = match name {
        "authenticated-identity" => get(name, "/api/me".into(), Validator::Identity { owner }),
        "Mailbox/get" => jmap_single(
            p,
            name,
            vec![],
            native,
            constants,
            Validator::JmapSingle {
                name: name.into(),
                account: p.account_id.clone(),
            },
        ),
        "Email/query" => jmap_single(
            p,
            name,
            vec![("limit", json!(25))],
            native,
            constants,
            Validator::JmapSingle {
                name: name.into(),
                account: p.account_id.clone(),
            },
        ),
        "Email/get" => {
            let first: Vec<String> = ids.iter().take(2).cloned().collect();
            let props = if representative {
                &constants.default_header_properties
            } else {
                &constants.minimal_header_properties
            };
            jmap_single(
                p,
                name,
                vec![("ids", strings(&first)), ("properties", strings(props))],
                native,
                constants,
                Validator::EmailGet {
                    account: p.account_id.clone(),
                    ids: first,
                },
            )
        }
        "concurrent-refresh-handler" => Spec {
            name: name.into(),
            method: "POST",
            path: "/api/mail/refresh".into(),
            headers: json_header(),
            body: Some(serde_json::to_string(&json!({"accountId": owner})).unwrap()),
            validator: Validator::Refresh { owner, native },
        },
        _ if !representative => return Ok(client_hot(name, p, constants)),
        _ => match representative_spec(name, p, constants)? {
            Some(spec) => spec,
            None => match client_hot(name, p, constants) {
                Some(spec) => spec,
                None => match goal_spec(name, p)? {
                    Some(spec) => spec,
                    None => return Ok(None),
                },
            },
        },
    };
    Ok(Some(spec))
}

fn require_representative(p: &Principal) -> Result<(), String> {
    let s = |v: &Option<Value>| {
        Principal::str_field(v)
            .filter(|s| !s.is_empty())
            .map(str::to_owned)
    };
    let unique: std::collections::HashSet<&String> = p.message_ids.iter().collect();
    if p.message_ids.len() < 2
        || unique.len() != p.message_ids.len()
        || s(&p.inbox_id).is_none()
        || s(&p.archive_id).is_none()
        || s(&p.archive_id) == s(&p.inbox_id)
        || s(&p.attachment_blob_id).is_none()
    {
        return Err(
            "Representative fixture requires seeded folders, messages and attachment".into(),
        );
    }
    Ok(())
}

fn representative_spec(
    name: &str,
    p: &Principal,
    constants: &Constants,
) -> Result<Option<Spec>, String> {
    if !matches!(
        name,
        "paged-query-and-headers"
            | "body-and-attachment-metadata"
            | "attachment-download"
            | "mark-read-and-restore"
            | "move-and-restore"
    ) {
        return Ok(None);
    }
    require_representative(p)?;
    let id = p.message_ids[0].clone();
    let account = p.account_id.clone();
    let inbox = key_of(&p.inbox_id);
    let archive = key_of(&p.archive_id);
    let headers = strings(&constants.default_header_properties);
    let page = p.message_ids.len().min(25);
    let next = 25usize.min(p.message_ids.len().saturating_sub(page));
    let getp = |props: &[&str], tag: &str| {
        call(
            p,
            "Email/get",
            vec![("ids", json!([id])), ("properties", json!(props))],
            tag,
        )
    };
    Ok(Some(match name {
        "paged-query-and-headers" => post_batch(
            name,
            vec![
                call(
                    p,
                    "Email/query",
                    vec![
                        ("limit", json!(page)),
                        ("position", json!(0)),
                        ("calculateTotal", json!(true)),
                    ],
                    "page1",
                ),
                call(
                    p,
                    "Email/get",
                    vec![
                        ("#ids", result_ref("page1", "/ids")),
                        ("properties", headers.clone()),
                    ],
                    "headers1",
                ),
                call(
                    p,
                    "Email/query",
                    vec![
                        ("limit", json!(25)),
                        ("position", json!(page)),
                        ("calculateTotal", json!(true)),
                    ],
                    "page2",
                ),
                call(
                    p,
                    "Email/get",
                    vec![
                        ("#ids", result_ref("page2", "/ids")),
                        ("properties", headers),
                    ],
                    "headers2",
                ),
            ],
            |calls| Validator::Paged {
                calls,
                account,
                page,
                next,
                known: p.message_ids.clone(),
            },
        ),
        "body-and-attachment-metadata" => {
            let mut args = vec![
                ("ids", json!([id])),
                ("properties", strings(&constants.reader_properties)),
            ];
            for (k, v) in &constants.reader_fetch_options {
                args.push((k.as_str(), v.clone()));
            }
            let bytes = p.body_sample_bytes.unwrap_or(27.0);
            if bytes < 27.0 {
                return Err("Invalid count value: representative body sample".into());
            }
            let expected = format!(
                "Representative body sample.{}",
                "x".repeat((bytes - 27.0).floor() as usize)
            );
            post_batch(name, vec![call(p, "Email/get", args, "body")], |calls| {
                Validator::Body {
                    calls,
                    account,
                    id,
                    expected,
                    bytes,
                    blob: key_of(&p.attachment_blob_id),
                }
            })
        }
        "attachment-download" => {
            let size = p.attachment_sample_bytes.unwrap_or(4.0);
            if size.fract() != 0.0 || !(4.0..=262144.0).contains(&size) {
                return Err("Invalid bounded attachment sample size".into());
            }
            get(
                name,
                format!(
                    "/jmap/download/{}/{}/sample.txt",
                    enc(&p.account_id),
                    enc(&key_of(&p.attachment_blob_id))
                ),
                Validator::Download {
                    size: size as usize,
                },
            )
        }
        "mark-read-and-restore" => post_batch(
            name,
            vec![
                call(
                    p,
                    "Email/set",
                    vec![(
                        "update",
                        obj_owned(vec![(
                            id.clone(),
                            obj(vec![("keywords/$seen", json!(true))]),
                        )]),
                    )],
                    "read",
                ),
                getp(&["id", "keywords"], "readback"),
                call(
                    p,
                    "Email/set",
                    vec![(
                        "update",
                        obj_owned(vec![(
                            id.clone(),
                            obj(vec![("keywords/$seen", Value::Null)]),
                        )]),
                    )],
                    "restore",
                ),
                getp(&["id", "keywords"], "restored"),
            ],
            |calls| Validator::MarkRead {
                calls,
                account,
                id: id.clone(),
            },
        ),
        "move-and-restore" => post_batch(
            name,
            vec![
                call(
                    p,
                    "Email/set",
                    vec![(
                        "update",
                        obj_owned(vec![(
                            id.clone(),
                            obj_owned(vec![
                                (format!("mailboxIds/{archive}"), json!(true)),
                                (format!("mailboxIds/{inbox}"), Value::Null),
                            ]),
                        )]),
                    )],
                    "move",
                ),
                getp(&["id", "mailboxIds"], "moved"),
                call(
                    p,
                    "Email/set",
                    vec![(
                        "update",
                        obj_owned(vec![(
                            id.clone(),
                            obj_owned(vec![
                                (format!("mailboxIds/{inbox}"), json!(true)),
                                (format!("mailboxIds/{archive}"), Value::Null),
                            ]),
                        )]),
                    )],
                    "restore",
                ),
                getp(&["id", "mailboxIds"], "restored"),
            ],
            |calls| Validator::Move {
                calls,
                account,
                id: id.clone(),
                from: inbox.clone(),
                to: archive.clone(),
            },
        ),
        _ => unreachable!(),
    }))
}

/// `clientHotSpecs`
fn client_hot(name: &str, p: &Principal, constants: &Constants) -> Option<Spec> {
    let owner = p.owner().to_owned();
    let account = p.account_id.clone();
    let inbox = Principal::str_field(&p.inbox_id)
        .filter(|s| !s.is_empty())
        .map(str::to_owned);
    let filter = || match &inbox {
        Some(id) => json!({"inMailbox": id}),
        None => json!({}),
    };
    let sort = || json!([{"property": "receivedAt", "isAscending": false}]);
    let page = p.message_ids.len().min(25);
    Some(match name {
        "jmap-session" => get(name, "/jmap/session".into(), Validator::Session { account }),
        "list-batch" => {
            let mut unread = filter();
            unread
                .as_object_mut()
                .unwrap()
                .insert("notKeyword".into(), json!("$seen"));
            post_batch(
                name,
                vec![
                    call(
                        p,
                        "Email/query",
                        vec![
                            ("filter", filter()),
                            ("sort", sort()),
                            ("limit", json!(page)),
                            ("calculateTotal", json!(true)),
                        ],
                        "q",
                    ),
                    call(
                        p,
                        "Email/get",
                        vec![
                            ("#ids", result_ref("q", "/ids")),
                            ("properties", strings(&constants.default_header_properties)),
                        ],
                        "g",
                    ),
                    call(
                        p,
                        "Email/query",
                        vec![
                            ("filter", unread),
                            ("limit", json!(0)),
                            ("calculateTotal", json!(true)),
                        ],
                        "unread",
                    ),
                ],
                |calls| Validator::ListBatch {
                    calls,
                    account,
                    page,
                },
            )
        }
        "changes" => {
            let since = json!({"resultOf": "base", "name": "Email/query", "path": "/queryState"});
            post_batch(
                name,
                vec![
                    call(
                        p,
                        "Email/query",
                        vec![("filter", filter()), ("sort", sort()), ("limit", json!(1))],
                        "base",
                    ),
                    call(
                        p,
                        "Email/queryChanges",
                        vec![
                            ("filter", filter()),
                            ("sort", sort()),
                            ("#sinceQueryState", since.clone()),
                            ("maxChanges", json!(1000)),
                            ("calculateTotal", json!(true)),
                        ],
                        "delta-query",
                    ),
                    call(
                        p,
                        "Email/changes",
                        vec![("#sinceState", since), ("maxChanges", json!(1000))],
                        "delta-emails",
                    ),
                ],
                |calls| Validator::Changes { calls, account },
            )
        }
        "account-preferences" => get(
            name,
            "/api/account/preferences".into(),
            Validator::Preferences,
        ),
        "appearance-customization" => get(
            name,
            format!(
                "/api/account/appearance/customization?accountId={}",
                enc(&owner)
            ),
            Validator::Appearance { owner },
        ),
        "session-activity" => Spec {
            name: name.into(),
            method: "POST",
            path: "/api/account/session/activity".into(),
            headers: Vec::new(),
            body: None,
            validator: Validator::Activity,
        },
        _ => return None,
    })
}

/// `goalMixSeeded`
pub fn goal_seeded(p: &Principal) -> bool {
    let s = |v: &Option<Value>| matches!(v, Some(Value::String(_)));
    let nonempty = |v: &Option<Value>| Principal::str_field(v).is_some_and(|s| !s.is_empty());
    p.message_ids.len() >= 5
        && s(&p.inbox_id)
        && nonempty(&p.trash_id)
        && nonempty(&p.drafts_id)
        && p.planted_quarterly
            .as_ref()
            .and_then(Value::as_f64)
            .is_some_and(|n| n >= 0.0 && n.fract() == 0.0 && n.is_finite())
}

fn goal_spec(name: &str, p: &Principal) -> Result<Option<Spec>, String> {
    let account = p.account_id.clone();
    let toggle = |name: &str, id: &String, keyword: &str, first: bool, second: bool| {
        let val = |b: bool| if b { json!(true) } else { Value::Null };
        let getk = |tag: &str| {
            call(
                p,
                "Email/get",
                vec![
                    ("ids", json!([id])),
                    ("properties", json!(["id", "keywords"])),
                ],
                tag,
            )
        };
        post_batch(
            name,
            vec![
                call(
                    p,
                    "Email/set",
                    vec![(
                        "update",
                        obj_owned(vec![(
                            id.clone(),
                            obj_owned(vec![(format!("keywords/{keyword}"), val(first))]),
                        )]),
                    )],
                    "first",
                ),
                getk("firstback"),
                call(
                    p,
                    "Email/set",
                    vec![(
                        "update",
                        obj_owned(vec![(
                            id.clone(),
                            obj_owned(vec![(format!("keywords/{keyword}"), val(second))]),
                        )]),
                    )],
                    "second",
                ),
                getk("secondback"),
            ],
            |calls| Validator::Toggle {
                calls,
                account: account.clone(),
                id: id.clone(),
                keyword: keyword.into(),
                first,
                second,
            },
        )
    };
    let need4 = || {
        if p.message_ids.len() < 4 {
            Err("The goal mix needs at least four seeded messages per user".to_owned())
        } else {
            Ok(())
        }
    };
    let seeded = || {
        if goal_seeded(p) {
            Ok(())
        } else {
            Err("The goal mix search needs the planted search term seeding".to_owned())
        }
    };
    Ok(Some(match name {
        "mark-unread-and-restore" => {
            need4()?;
            toggle(name, &p.message_ids[2], "$seen", false, true)
        }
        "flag-and-restore" => {
            need4()?;
            toggle(name, &p.message_ids[1], "$flagged", true, false)
        }
        "unflag-and-restore" => {
            need4()?;
            toggle(name, &p.message_ids[3], "$flagged", false, true)
        }
        "delete-to-trash-and-restore" => {
            need4()?;
            if !goal_seeded(p) {
                return Ok(None);
            }
            let id = p.message_ids[4].clone();
            let (trash, inbox) = (key_of(&p.trash_id), key_of(&p.inbox_id));
            let getm = |tag: &str| {
                call(
                    p,
                    "Email/get",
                    vec![
                        ("ids", json!([id])),
                        ("properties", json!(["id", "mailboxIds"])),
                    ],
                    tag,
                )
            };
            post_batch(
                name,
                vec![
                    call(
                        p,
                        "Email/set",
                        vec![(
                            "update",
                            obj_owned(vec![(
                                id.clone(),
                                obj_owned(vec![
                                    (format!("mailboxIds/{trash}"), json!(true)),
                                    (format!("mailboxIds/{inbox}"), Value::Null),
                                ]),
                            )]),
                        )],
                        "trash",
                    ),
                    getm("trashed"),
                    call(
                        p,
                        "Email/set",
                        vec![(
                            "update",
                            obj_owned(vec![(
                                id.clone(),
                                obj_owned(vec![
                                    (format!("mailboxIds/{inbox}"), json!(true)),
                                    (format!("mailboxIds/{trash}"), Value::Null),
                                ]),
                            )]),
                        )],
                        "restore",
                    ),
                    getm("restored"),
                ],
                |calls| Validator::Move {
                    calls,
                    account,
                    id: id.clone(),
                    from: inbox.clone(),
                    to: trash.clone(),
                },
            )
        }
        "search-planted-term" => {
            seeded()?;
            post_batch(
                name,
                vec![
                    call(
                        p,
                        "Email/query",
                        vec![
                            (
                                "filter",
                                json!({"inMailbox": sv(&p.inbox_id), "text": "quarterly"}),
                            ),
                            (
                                "sort",
                                json!([{"property": "receivedAt", "isAscending": false}]),
                            ),
                            ("limit", json!(50)),
                            ("calculateTotal", json!(true)),
                        ],
                        "q",
                    ),
                    call(
                        p,
                        "Email/get",
                        vec![
                            ("#ids", result_ref("q", "/ids")),
                            ("properties", json!(["id", "subject", "receivedAt"])),
                        ],
                        "g",
                    ),
                ],
                |calls| Validator::Search {
                    calls,
                    account,
                    planted: p
                        .planted_quarterly
                        .as_ref()
                        .and_then(Value::as_f64)
                        .unwrap_or(f64::NAN),
                },
            )
        }
        "draft-save" => {
            seeded().map_err(|_| "Draft save needs the drafts folder seeding".to_owned())?;
            draft_save(name, p)
        }
        // The discard slot before any draft exists: the save template under the discard label.
        "draft-discard" => {
            seeded().map_err(|_| "Draft save needs the drafts folder seeding".to_owned())?;
            draft_save(name, p)
        }
        _ => return Ok(None),
    }))
}

fn draft_save(name: &str, p: &Principal) -> Spec {
    let drafts = key_of(&p.drafts_id);
    let create = obj(vec![(
        "d1",
        obj(vec![
            ("mailboxIds", obj_owned(vec![(drafts, json!(true))])),
            ("keywords", json!({"$draft": true, "$seen": true})),
            ("to", json!([{"email": "someone@native-load.invalid"}])),
            ("subject", json!("Goal mix draft")),
            ("bodyValues", json!({"b1": {"value": "Draft body."}})),
            ("textBody", json!([{"partId": "b1", "type": "text/plain"}])),
        ]),
    )]);
    post_batch(
        name,
        vec![call(p, "Email/set", vec![("create", create)], "save")],
        |calls| Validator::DraftSave {
            calls,
            account: p.account_id.clone(),
        },
    )
}

/// `goalMixDraftSpecs(p).discard(id)`
pub fn draft_discard(p: &Principal, id: &str) -> Spec {
    post_batch(
        "draft-discard",
        vec![
            call(
                p,
                "Email/get",
                vec![
                    ("ids", json!([id])),
                    ("properties", json!(["id", "mailboxIds", "keywords"])),
                ],
                "saved",
            ),
            call(p, "Email/set", vec![("destroy", json!([id]))], "discard"),
        ],
        |calls| Validator::DraftDiscard {
            calls,
            account: p.account_id.clone(),
            id: id.to_owned(),
            drafts: key_of(&p.drafts_id),
        },
    )
}

/// `fixturePrincipalIndex`
pub fn principal_index(sequence: u64, request_count: u64, principals: u64) -> u64 {
    (sequence / request_count
        + (sequence % request_count) * request_count_div(principals, request_count))
        % principals
}
fn request_count_div(principals: u64, request_count: u64) -> u64 {
    principals.div_ceil(request_count)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn rotation_matches_node_formula() {
        // (floor(s/r) + (s%r)*ceil(n/r)) % n
        assert_eq!(principal_index(0, 100, 1000), 0);
        assert_eq!(principal_index(1, 100, 1000), 10);
        assert_eq!(principal_index(101, 100, 1000), 11);
        assert_eq!(principal_index(7, 1, 3), 1);
        assert_eq!(enc("a b/é"), "a%20b%2F%C3%A9");
    }
}
