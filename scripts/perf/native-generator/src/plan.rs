//! The plan the Node adapter writes (private file): principals exactly as the Node
//! driver holds them after its own discovery, the client simulation, the group
//! layout taken from `prepareFixtureRequests` (shape names and their serialise /
//! stateful flags, in slot order), and the request constants of
//! `fixture-properties.mjs` and the native read contract. The request templates are
//! rebuilt from these here; `mw-loadgen templates` prints them for the byte-for-byte
//! comparison against the Node templates.
use serde::Deserialize;
use serde_json::{Map, Value};
use std::collections::{BTreeMap, HashSet};
use std::net::Ipv4Addr;

#[derive(Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct GroupEntry {
    pub name: String,
    #[serde(default)]
    pub serialize: bool,
    #[serde(default)]
    pub stateful: bool,
}

#[derive(Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ReadContract {
    pub header: String,
    pub value: String,
    pub properties: Vec<String>,
    pub arguments: Vec<String>,
    pub max_ids: usize,
    pub max_id_bytes: usize,
    pub max_call_id_bytes: usize,
    pub max_body_bytes: usize,
}

#[derive(Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Constants {
    pub default_header_properties: Vec<String>,
    pub minimal_header_properties: Vec<String>,
    pub reader_properties: Vec<String>,
    /// ordered object, merged after `properties` like `...READER_FETCH_OPTIONS`
    pub reader_fetch_options: Map<String, Value>,
    pub read_contract: ReadContract,
}

#[derive(Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct RateLimits {
    pub client_requests: f64,
    pub account_requests: f64,
    pub rate_window_secs: f64,
    pub source: String,
}

#[derive(Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct PlanFile {
    pub schema: u32,
    pub engine_mode: String,
    pub representative: bool,
    pub principals: Vec<Value>,
    #[serde(default)]
    pub client_simulation: Option<Value>,
    #[serde(default = "one")]
    pub clients_per_user: usize,
    pub groups: BTreeMap<String, Vec<GroupEntry>>,
    pub constants: Constants,
    pub rate_limits: RateLimits,
}
fn one() -> usize {
    1
}

#[derive(Clone, Debug)]
pub struct Principal {
    pub index: usize,
    pub account_id: String,
    pub owner_account_id: Option<String>,
    pub message_ids: Vec<String>,
    pub inbox_id: Option<Value>,
    pub archive_id: Option<Value>,
    pub trash_id: Option<Value>,
    pub drafts_id: Option<Value>,
    pub attachment_blob_id: Option<Value>,
    pub body_sample_bytes: Option<f64>,
    pub attachment_sample_bytes: Option<f64>,
    pub planted_quarterly: Option<Value>,
    /// cookie, then x-csrf-token when present (the order `prepareFixtureClients` builds)
    pub headers: Vec<(String, String)>,
    /// forwarded client addresses; empty without client simulation
    pub addresses: Vec<String>,
    /// client index of each address (address 0 = principal index, as in Node)
    pub client_indexes: Vec<usize>,
}
impl Principal {
    pub fn owner(&self) -> &str {
        self.owner_account_id.as_deref().unwrap_or(&self.account_id)
    }
    pub fn str_field(v: &Option<Value>) -> Option<&str> {
        v.as_ref().and_then(Value::as_str)
    }
}

fn percent_decode(text: &str) -> Result<String, String> {
    let bytes = text.as_bytes();
    let mut out = Vec::with_capacity(bytes.len());
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'%' {
            let hex = text.get(i + 1..i + 3).ok_or("Invalid fixture CSRF token")?;
            out.push(u8::from_str_radix(hex, 16).map_err(|_| "Invalid fixture CSRF token")?);
            i += 3;
        } else {
            out.push(bytes[i]);
            i += 1;
        }
    }
    String::from_utf8(out).map_err(|_| "Invalid fixture CSRF token".into())
}

fn has_control(s: &str) -> bool {
    s.chars().any(|c| (c as u32) < 0x20 || c as u32 == 0x7f)
}

/// `prepareFixtureClients` plus the extra client addresses of plan N11.
pub fn principals(plan: &PlanFile) -> Result<Vec<Principal>, String> {
    let list = &plan.principals;
    if list.is_empty() || list.len() > 10000 {
        return Err("Invalid bounded principal handoff".into());
    }
    let simulated = plan.client_simulation.is_some();
    if let Some(sim) = &plan.client_simulation {
        if sim.get("mode").and_then(Value::as_str) != Some("fixture-loopback-trusted-proxy")
            || sim.get("clientCount").and_then(Value::as_u64) != Some(list.len() as u64)
        {
            return Err("Invalid fixture client simulation".into());
        }
    }
    let per_user = plan.clients_per_user;
    if per_user < 1 || per_user > 64 || (per_user > 1 && !simulated) {
        return Err("Several client addresses per user need the fixture client simulation and 1-64 addresses".into());
    }
    let (mut cookies, mut sessions, mut accounts, mut addresses) = (
        HashSet::new(),
        HashSet::new(),
        HashSet::new(),
        HashSet::new(),
    );
    let users = list.len();
    let mut out = Vec::with_capacity(users);
    for (index, p) in list.iter().enumerate() {
        let cookie = p
            .get("cookie")
            .and_then(Value::as_str)
            .ok_or("Invalid or duplicate fixture credentials")?;
        if cookie.is_empty()
            || cookie.len() > 8192
            || has_control(cookie)
            || !cookies.insert(cookie.to_owned())
        {
            return Err("Invalid or duplicate fixture credentials".into());
        }
        let tokens: Vec<&str> = cookie
            .split(';')
            .map(str::trim)
            .filter(|c| c.starts_with("mw_session="))
            .collect();
        if users > 1
            && (tokens.len() != 1 || tokens[0] == "mw_session=" || sessions.contains(tokens[0]))
        {
            return Err("Distinct fixture sessions are required".into());
        }
        if tokens.len() == 1 {
            sessions.insert(tokens[0].to_owned());
        }
        let account_id = p
            .get("accountId")
            .and_then(Value::as_str)
            .unwrap_or("")
            .to_owned();
        if users > 1 && (account_id.is_empty() || accounts.contains(&account_id)) {
            return Err("Distinct fixture accounts are required".into());
        }
        accounts.insert(account_id.clone());
        if !simulated && p.get("clientAddress").is_some() {
            return Err("Forwarded fixture clients require explicit proxy simulation".into());
        }
        let mut headers = vec![("cookie".to_owned(), cookie.to_owned())];
        let csrf = match p.get("csrfToken") {
            Some(Value::Null) | None => cookie
                .split(';')
                .map(str::trim)
                .find(|c| c.starts_with("mw_csrf="))
                .map(|c| c[8..].to_owned()),
            Some(Value::String(s)) => Some(s.clone()),
            Some(_) => return Err("Invalid fixture CSRF token".into()),
        };
        if let Some(csrf) = csrf.filter(|c| !c.is_empty()) {
            if csrf.len() > 8192 {
                return Err("Invalid fixture CSRF token".into());
            }
            let decoded = percent_decode(&csrf)?;
            if has_control(&decoded) {
                return Err("Invalid fixture CSRF token".into());
            }
            headers.push(("x-csrf-token".into(), decoded));
        }
        let mut own = Vec::new();
        let mut client_indexes = Vec::new();
        if simulated {
            let address = p.get("clientAddress").and_then(Value::as_str).unwrap_or("");
            let ok = address.parse::<Ipv4Addr>().is_ok_and(|a| {
                a.octets()[0] == 198 && (a.octets()[1] == 18 || a.octets()[1] == 19)
            });
            if !ok || !addresses.insert(address.to_owned()) {
                return Err("Distinct benchmark-range fixture clients are required".into());
            }
            own.push(address.to_owned());
            client_indexes.push(index);
            if per_user > 1 {
                let listed = p.get("clientAddresses").and_then(Value::as_array);
                for extra in 1..per_user {
                    let address = match listed {
                        Some(items) => items
                            .get(extra)
                            .and_then(Value::as_str)
                            .ok_or("clientAddresses shorter than the clients per user")?
                            .to_owned(),
                        // Derived: 198.19.0.0/16, disjoint from the fixture's 198.18.0.1 + index.
                        None => {
                            let n = index * (per_user - 1) + (extra - 1);
                            if n >= 65536 {
                                return Err(
                                    "Derived client addresses exhausted (198.19.0.0/16)".into()
                                );
                            }
                            Ipv4Addr::new(198, 19, (n >> 8) as u8, (n & 255) as u8).to_string()
                        }
                    };
                    let ok = address.parse::<Ipv4Addr>().is_ok_and(|a| {
                        a.octets()[0] == 198 && (a.octets()[1] == 18 || a.octets()[1] == 19)
                    });
                    if !ok || !addresses.insert(address.clone()) {
                        return Err("Distinct benchmark-range fixture clients are required".into());
                    }
                    own.push(address);
                    client_indexes.push(users + index * (per_user - 1) + (extra - 1));
                }
            }
        }
        let message_ids: Vec<String> = match p.get("messageIds").and_then(Value::as_array) {
            Some(ids) => ids
                .iter()
                .map(|id| {
                    id.as_str()
                        .map(str::to_owned)
                        .ok_or("Fixture requests require distinct seeded account/message bindings")
                })
                .collect::<Result<_, _>>()?,
            None => {
                return Err(
                    "Fixture requests require distinct seeded account/message bindings".into(),
                );
            }
        };
        if account_id.is_empty()
            || message_ids.is_empty()
            || message_ids.len() > 100
            || message_ids.iter().any(String::is_empty)
        {
            return Err("Fixture requests require distinct seeded account/message bindings".into());
        }
        let field = |key: &str| p.get(key).filter(|v| !v.is_null()).cloned();
        out.push(Principal {
            index,
            account_id,
            owner_account_id: p
                .get("ownerAccountId")
                .and_then(Value::as_str)
                .map(str::to_owned),
            message_ids,
            inbox_id: field("inboxId"),
            archive_id: field("archiveId"),
            trash_id: field("trashId"),
            drafts_id: field("draftsId"),
            attachment_blob_id: field("attachmentBlobId"),
            body_sample_bytes: p.get("bodySampleBytes").and_then(Value::as_f64),
            attachment_sample_bytes: p.get("attachmentSampleBytes").and_then(Value::as_f64),
            planted_quarterly: field("plantedQuarterly"),
            headers,
            addresses: own,
            client_indexes,
        });
    }
    Ok(out)
}
