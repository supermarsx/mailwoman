//! MCP server mount (SPEC §20.3, plan §3 e11). Backs `mw_mcp`'s abstract
//! [`McpBackend`] with the REAL engine (driving the JMAP surface, never raw
//! protocol) and nests the Streamable-HTTP transport at `/mcp`.
//!
//! Per-call scope enforcement uses [`mw_mcp::OAuthAuthorizer`] (verify + expiry +
//! `Scope::allows` + audit) — NOT `mw_oauth::require_scope`, whose future is
//! non-`Send` (it borrows a `&dyn AuditSink` across an await) and so cannot sit in
//! the axum `Send` path. Send is split (`enqueue_outbox` vs `send_now`) so
//! `mw_mcp::gate_send` owns the safety-critical decision.
//!
//! **Countersign resolver.** `unattended_send` may only transmit with an admin
//! countersignature (the `api_keys.unattended_send` flag). The resolver is a *sync*
//! `Fn(&str) -> bool`, so it reads a snapshot of countersigned key prefixes loaded
//! at mount time; a key minted after boot is treated as NOT countersigned (the safe
//! default — unattended send falls back to the Outbox / 403) until the next reload.

use std::collections::HashSet;
use std::sync::Arc;

use async_trait::async_trait;
use serde_json::{Value, json};

use mw_engine::Engine;
use mw_mcp::{
    BackendError, DraftInput, DraftRef, Folder, MailBody, McpBackend, McpServer, OAuthAuthorizer,
    SearchHit, mcp_router,
};

/// The sync countersign resolver signature `mw_mcp::OAuthAuthorizer::new` expects.
type CountersignResolver = Arc<dyn Fn(&str) -> bool + Send + Sync>;

use crate::stores_v6::{AdminOAuthAudit, OAuthStoreAdapter};

type Auth = mw_oauth::AuthServer<OAuthStoreAdapter>;

/// The real engine-backed MCP backend. Each tool call maps to the engine's JMAP
/// surface (`handle_jmap`), never a raw IMAP/SMTP command (§20.3 prompt-injection
/// posture). Mail content the tool returns is wrapped untrusted by the tool layer.
pub struct McpEngineBackend {
    engine: Arc<Engine>,
}

impl McpEngineBackend {
    pub fn new(engine: Arc<Engine>) -> Self {
        Self { engine }
    }

    async fn jmap(&self, account: &str, request: Value) -> Result<Value, BackendError> {
        Ok(self.engine.handle_jmap(account, &request).await)
    }
}

/// Extract a named method-call's arguments from a JMAP response envelope by call id.
fn method_args<'a>(resp: &'a Value, call_id: &str) -> Option<&'a Value> {
    resp.get("methodResponses")?
        .as_array()?
        .iter()
        .find_map(|e| {
            let arr = e.as_array()?;
            if arr.len() == 3
                && arr[2].as_str() == Some(call_id)
                && arr[0].as_str() != Some("error")
            {
                Some(&arr[1])
            } else {
                None
            }
        })
}

fn first_address(v: &Value, field: &str) -> String {
    v.get(field)
        .and_then(Value::as_array)
        .and_then(|a| a.first())
        .and_then(|e| e.get("email").or_else(|| e.get("name")))
        .and_then(Value::as_str)
        .unwrap_or_default()
        .to_string()
}

const EMAIL_PROPS: &[&str] = &[
    "id",
    "subject",
    "from",
    "to",
    "preview",
    "receivedAt",
    "mailboxIds",
    "bodyValues",
    "textBody",
];

#[async_trait]
impl McpBackend for McpEngineBackend {
    async fn mail_search(
        &self,
        account: &str,
        query: &str,
        limit: usize,
    ) -> Result<Vec<SearchHit>, BackendError> {
        let mut query_args = json!({
            "accountId": account,
            "sort": [{ "property": "receivedAt", "isAscending": false }],
            "limit": limit,
        });
        if !query.is_empty() {
            query_args["filter"] = json!({ "text": query });
        }
        let req = json!({
            "using": ["urn:ietf:params:jmap:core", "urn:ietf:params:jmap:mail"],
            "methodCalls": [
                ["Email/query", query_args, "q"],
                ["Email/get", {
                    "accountId": account,
                    "#ids": { "resultOf": "q", "name": "Email/query", "path": "/ids" },
                    "properties": ["id", "subject", "from", "preview", "receivedAt", "mailboxIds"],
                }, "g"],
            ],
        });
        let resp = self.jmap(account, req).await?;
        let list = method_args(&resp, "g")
            .and_then(|a| a.get("list"))
            .and_then(Value::as_array)
            .cloned()
            .unwrap_or_default();
        Ok(list
            .iter()
            .map(|m| SearchHit {
                message_id: m
                    .get("id")
                    .and_then(Value::as_str)
                    .unwrap_or_default()
                    .into(),
                folder: m
                    .get("mailboxIds")
                    .and_then(|v| v.as_object())
                    .and_then(|o| o.keys().next().cloned())
                    .unwrap_or_default(),
                from: first_address(m, "from"),
                subject: m
                    .get("subject")
                    .and_then(Value::as_str)
                    .unwrap_or_default()
                    .into(),
                snippet: m
                    .get("preview")
                    .and_then(Value::as_str)
                    .unwrap_or_default()
                    .into(),
                date: m
                    .get("receivedAt")
                    .and_then(Value::as_str)
                    .unwrap_or_default()
                    .into(),
            })
            .collect())
    }

    async fn mail_read(&self, account: &str, message_id: &str) -> Result<MailBody, BackendError> {
        let req = json!({
            "using": ["urn:ietf:params:jmap:core", "urn:ietf:params:jmap:mail"],
            "methodCalls": [
                ["Email/get", {
                    "accountId": account,
                    "ids": [message_id],
                    "properties": EMAIL_PROPS,
                    "fetchTextBodyValues": true,
                }, "g"],
            ],
        });
        let resp = self.jmap(account, req).await?;
        let msg = method_args(&resp, "g")
            .and_then(|a| a.get("list"))
            .and_then(Value::as_array)
            .and_then(|l| l.first())
            .cloned()
            .ok_or_else(|| BackendError::new("message not found"))?;
        let body_text = msg
            .get("bodyValues")
            .and_then(|v| v.as_object())
            .and_then(|o| o.values().next())
            .and_then(|b| b.get("value"))
            .and_then(Value::as_str)
            .unwrap_or_default()
            .to_string();
        let to = msg
            .get("to")
            .and_then(Value::as_array)
            .map(|a| {
                a.iter()
                    .filter_map(|e| e.get("email").and_then(Value::as_str).map(String::from))
                    .collect()
            })
            .unwrap_or_default();
        Ok(MailBody {
            message_id: message_id.to_string(),
            from: first_address(&msg, "from"),
            to,
            subject: msg
                .get("subject")
                .and_then(Value::as_str)
                .unwrap_or_default()
                .into(),
            date: msg
                .get("receivedAt")
                .and_then(Value::as_str)
                .unwrap_or_default()
                .into(),
            body_text,
            body_html: None,
        })
    }

    async fn folders_list(&self, account: &str) -> Result<Vec<Folder>, BackendError> {
        let req = json!({
            "using": ["urn:ietf:params:jmap:core", "urn:ietf:params:jmap:mail"],
            "methodCalls": [["Mailbox/get", { "accountId": account, "ids": null }, "m"]],
        });
        let resp = self.jmap(account, req).await?;
        let list = method_args(&resp, "m")
            .and_then(|a| a.get("list"))
            .and_then(Value::as_array)
            .cloned()
            .unwrap_or_default();
        Ok(list
            .iter()
            .map(|m| Folder {
                id: m
                    .get("id")
                    .and_then(Value::as_str)
                    .unwrap_or_default()
                    .into(),
                name: m
                    .get("name")
                    .and_then(Value::as_str)
                    .unwrap_or_default()
                    .into(),
                role: m.get("role").and_then(Value::as_str).map(String::from),
                unread: m.get("unreadEmails").and_then(Value::as_u64).unwrap_or(0) as u32,
            })
            .collect())
    }

    async fn drafts_create(
        &self,
        account: &str,
        draft: DraftInput,
    ) -> Result<DraftRef, BackendError> {
        let id = self.create_draft(account, &draft).await?;
        Ok(DraftRef { draft_id: id })
    }

    async fn enqueue_outbox(
        &self,
        account: &str,
        draft: DraftInput,
    ) -> Result<String, BackendError> {
        // Create the draft, then an EmailSubmission that lands in the V2 Outbox
        // (pending in-app confirmation) — the default, human-in-the-loop path.
        let draft_id = self.create_draft(account, &draft).await?;
        self.submit(account, &draft_id).await
    }

    async fn send_now(&self, account: &str, draft: DraftInput) -> Result<String, BackendError> {
        // Reached ONLY for an admin-countersigned unattended-send key. The engine's
        // submission path is the same; the human-confirmation gate is what the
        // countersign bypasses (enforced upstream by `gate_send`).
        let draft_id = self.create_draft(account, &draft).await?;
        self.submit(account, &draft_id).await
    }

    async fn calendar_read(&self, account: &str, range: &str) -> Result<Vec<Value>, BackendError> {
        // Real engine call: `CalendarEvent/query` (optionally windowed by a
        // `<after>/<before>` RFC3339 range) → `CalendarEvent/get` on the returned
        // ids. An empty/unparsed range falls back to all events (`ids: null`).
        let filter = parse_range(range);
        let ids = if filter.is_some() {
            let q = json!({
                "using": PIM_USING,
                "methodCalls": [["CalendarEvent/query", {
                    "accountId": account,
                    "filter": filter,
                }, "q"]],
            });
            let resp = self.jmap(account, q).await?;
            method_args(&resp, "q")
                .and_then(|a| a.get("ids"))
                .cloned()
                .unwrap_or(Value::Null)
        } else {
            Value::Null // all events
        };
        let req = json!({
            "using": PIM_USING,
            "methodCalls": [["CalendarEvent/get", { "accountId": account, "ids": ids }, "g"]],
        });
        let resp = self.jmap(account, req).await?;
        Ok(pim_list(&resp, "g"))
    }

    async fn calendar_propose(
        &self,
        account: &str,
        proposal: Value,
    ) -> Result<String, BackendError> {
        // Create a proposed (tentative) event from the JSCalendar-shaped proposal
        // via `CalendarEvent/set`. Does not auto-accept on anyone's behalf.
        let mut spec = proposal;
        if let Some(obj) = spec.as_object_mut() {
            obj.entry("status")
                .or_insert_with(|| Value::String("tentative".into()));
        }
        let req = json!({
            "using": PIM_USING,
            "methodCalls": [["CalendarEvent/set", {
                "accountId": account,
                "create": { "p0": spec },
            }, "s"]],
        });
        let resp = self.jmap(account, req).await?;
        pim_created_id(&resp, "s", "p0")
            .ok_or_else(|| BackendError::new("calendar proposal was not accepted by the engine"))
    }

    async fn tasks_read(&self, account: &str) -> Result<Vec<Value>, BackendError> {
        let req = json!({
            "using": PIM_USING,
            "methodCalls": [["Task/get", { "accountId": account, "ids": null }, "g"]],
        });
        let resp = self.jmap(account, req).await?;
        Ok(pim_list(&resp, "g"))
    }

    async fn tasks_write(&self, account: &str, task: Value) -> Result<String, BackendError> {
        // Update when the task carries an `id`; otherwise create. Returns the id.
        let existing = task.get("id").and_then(Value::as_str).map(String::from);
        let set_args = if let Some(id) = &existing {
            json!({ "accountId": account, "update": { id: task } })
        } else {
            json!({ "accountId": account, "create": { "t0": task } })
        };
        let req = json!({
            "using": PIM_USING,
            "methodCalls": [["Task/set", set_args, "s"]],
        });
        let resp = self.jmap(account, req).await?;
        if let Some(id) = existing {
            // Confirm the update was applied (id present in `updated`).
            method_args(&resp, "s")
                .and_then(|a| a.get("updated"))
                .and_then(|u| u.as_object())
                .filter(|u| u.contains_key(&id))
                .map(|_| id.clone())
                .ok_or_else(|| BackendError::new("task update was not accepted by the engine"))
        } else {
            pim_created_id(&resp, "s", "t0")
                .ok_or_else(|| BackendError::new("task creation was not accepted by the engine"))
        }
    }

    async fn contacts_read(&self, account: &str) -> Result<Vec<Value>, BackendError> {
        let req = json!({
            "using": PIM_USING,
            "methodCalls": [["ContactCard/get", { "accountId": account, "ids": null }, "g"]],
        });
        let resp = self.jmap(account, req).await?;
        Ok(pim_list(&resp, "g"))
    }
}

/// The Mailwoman-native PIM capability URNs (frozen §2.2) advertised in `using` for
/// the calendar/task/contact method families. Dispatch keys on the method name, but
/// the request declares the capabilities it exercises.
const PIM_USING: &[&str] = &[
    "urn:ietf:params:jmap:core",
    "urn:mailwoman:calendars",
    "urn:mailwoman:tasks",
    "urn:mailwoman:contacts",
];

/// A `<after>/<before>` RFC3339 window → a `CalendarEvent/query` filter, or `None`
/// when `range` is empty or not a two-part `start/end` string.
fn parse_range(range: &str) -> Option<Value> {
    let (after, before) = range.split_once('/')?;
    let (after, before) = (after.trim(), before.trim());
    if after.is_empty() || before.is_empty() {
        return None;
    }
    Some(json!({ "after": after, "before": before }))
}

/// Extract the `list` array from a PIM `*/get` response envelope by call id.
fn pim_list(resp: &Value, call_id: &str) -> Vec<Value> {
    method_args(resp, call_id)
        .and_then(|a| a.get("list"))
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default()
}

/// The server-assigned id of a `*/set` create, by call id + create id.
fn pim_created_id(resp: &Value, call_id: &str, create_id: &str) -> Option<String> {
    method_args(resp, call_id)?
        .get("created")?
        .get(create_id)?
        .get("id")?
        .as_str()
        .map(String::from)
}

impl McpEngineBackend {
    async fn create_draft(
        &self,
        account: &str,
        draft: &DraftInput,
    ) -> Result<String, BackendError> {
        let req = json!({
            "using": ["urn:ietf:params:jmap:core", "urn:ietf:params:jmap:mail"],
            "methodCalls": [["Email/set", {
                "accountId": account,
                "create": { "d0": {
                    "to": draft.to.iter().map(|a| json!({ "email": a })).collect::<Vec<_>>(),
                    "subject": draft.subject,
                    "keywords": { "$draft": true },
                    "bodyStructure": { "type": "text/plain", "partId": "b0" },
                    "bodyValues": { "b0": { "value": draft.body_text } },
                }},
            }, "s"]],
        });
        let resp = self.jmap(account, req).await?;
        method_args(&resp, "s")
            .and_then(|a| a.get("created"))
            .and_then(|c| c.get("d0"))
            .and_then(|e| e.get("id"))
            .and_then(Value::as_str)
            .map(String::from)
            .ok_or_else(|| BackendError::new("draft creation was not accepted by the engine"))
    }

    async fn submit(&self, account: &str, email_id: &str) -> Result<String, BackendError> {
        let req = json!({
            "using": ["urn:ietf:params:jmap:core", "urn:ietf:params:jmap:mail", "urn:ietf:params:jmap:submission"],
            "methodCalls": [["EmailSubmission/set", {
                "accountId": account,
                "create": { "sub0": { "emailId": email_id } },
            }, "sub"]],
        });
        let resp = self.jmap(account, req).await?;
        method_args(&resp, "sub")
            .and_then(|a| a.get("created"))
            .and_then(|c| c.get("sub0"))
            .and_then(|e| e.get("id"))
            .and_then(Value::as_str)
            .map(String::from)
            .ok_or_else(|| BackendError::new("submission was not accepted by the engine"))
    }
}

/// The concrete MCP server type mounted at `/mcp`.
type Server = McpServer<McpEngineBackend, OAuthAuthorizer<OAuthStoreAdapter, AdminOAuthAudit>>;

/// Build a countersign resolver from a snapshot of countersigned key prefixes.
fn countersign_resolver(prefixes: HashSet<String>) -> CountersignResolver {
    let set = Arc::new(prefixes);
    Arc::new(move |token: &str| {
        token
            .strip_prefix("mwk_")
            .and_then(|r| r.split('.').next())
            .map(|p| set.contains(p))
            .unwrap_or(false)
    })
}

/// Build the `/mcp` router: the real engine backend + an `OAuthAuthorizer` over the
/// mounted `AuthServer` + audit sink + countersign snapshot.
pub fn build_mcp_router(
    engine: Arc<Engine>,
    auth: Arc<Auth>,
    audit: Arc<AdminOAuthAudit>,
    countersigned_prefixes: HashSet<String>,
) -> axum::Router {
    let backend = Arc::new(McpEngineBackend::new(engine));
    let authorizer: Arc<OAuthAuthorizer<OAuthStoreAdapter, AdminOAuthAudit>> = Arc::new(
        OAuthAuthorizer::new(auth, audit, countersign_resolver(countersigned_prefixes)),
    );
    let server: Arc<Server> = Arc::new(McpServer::new(backend, authorizer));
    // RFC 8707 (A3): this endpoint's canonical resource identifier. When
    // `MW_MCP_RESOURCE` is set, a bearer token issued for a different resource is
    // rejected as a wrong-audience token before it can reach a tool. Unset → audience
    // enforcement is off (tokens are accepted regardless of their bound resource).
    let resource = std::env::var("MW_MCP_RESOURCE")
        .ok()
        .filter(|s| !s.is_empty());
    mcp_router(server, resource)
}

/// The `mailwoman mcp-stdio` bridge body (wired from `main.rs` by e11). Proxies
/// stdin/stdout JSON-RPC to a configured remote `/mcp` endpoint.
pub async fn run_stdio(url: &str, token: Option<String>) -> anyhow::Result<()> {
    mw_mcp::run_stdio_http(url, token)
        .await
        .map_err(|e| anyhow::anyhow!("mcp-stdio: {e}"))
}
