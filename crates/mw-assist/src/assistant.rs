//! The `Assistant` capability's tool surface — an **unprivileged CLIENT** of the
//! frozen V6 `mw-mcp` tool registry (plan §2.4 / SPEC §14.3).
//!
//! The assistant chat routes every tool call through [`mw_mcp::McpServer`] with the
//! caller's `mw_oauth::Scope`, so it **inherits** the exact per-tool scope
//! enforcement and the `mail.send`→Outbox gating that the MCP surface already
//! provides. There is **no privileged path**: Assist adds no new send/delete/accept
//! capability, and this wrapper cannot mint one — `mail.send` is reachable only if
//! the caller's own scope grants it, and even then it is gated to the Outbox unless
//! the key carries an admin countersignature (resolved by the `mw-mcp`
//! `Authorizer`; the real countersign resolver is wired by e14 at mount — this
//! crate only consumes the seam).
//!
//! The second half of this module is the **proposal** path (§14.3): the assistant may
//! name actions it would like taken, and [`ProposalFilter`] lifts them out of the
//! streamed reply as [`ProposedAction`]s for the UI to show. Reporting a proposal is
//! not taking one — nothing here calls a tool, and every proposal still ends at a human
//! confirmation.

use std::sync::Arc;

use mw_mcp::{Authorizer, Credential, McpBackend, McpServer};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

use crate::AssistCapability;

/// A thin client over an [`McpServer`] for the `Assistant` capability. Every call
/// is dispatched as a JSON-RPC `tools/call` through the same handler the MCP
/// transport uses, so scope + provenance + send-gating are identical.
pub struct AssistantTools<B: McpBackend, A: Authorizer> {
    server: Arc<McpServer<B, A>>,
}

impl<B: McpBackend, A: Authorizer> AssistantTools<B, A> {
    /// Wrap an MCP server (built at mount with the real engine backend +
    /// `OAuthAuthorizer`).
    #[must_use]
    pub fn new(server: Arc<McpServer<B, A>>) -> Self {
        Self { server }
    }

    /// Enumerate the tools available to the assistant (`tools/list`).
    pub async fn list_tools(&self, cred: &Credential<'_>) -> Value {
        let req = json!({ "jsonrpc": "2.0", "id": 1, "method": "tools/list" });
        self.server
            .handle_rpc(cred, req)
            .await
            .unwrap_or(Value::Null)
    }

    /// Invoke a tool by wire name (e.g. `mail.search`, `mail.send`). The caller's
    /// [`Credential`] carries the `mw-oauth` scope; the MCP server authorizes it
    /// per call and applies send-gating — the assistant never bypasses either.
    pub async fn call_tool(&self, cred: &Credential<'_>, name: &str, arguments: Value) -> Value {
        let req = json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "tools/call",
            "params": { "name": name, "arguments": arguments },
        });
        self.server
            .handle_rpc(cred, req)
            .await
            .unwrap_or(Value::Null)
    }
}

// ---------------------------------------------------------------------------
// Proposed actions (SPEC §14.3) — parsed OUT of the reply, never executed
// ---------------------------------------------------------------------------

/// The sentinel that separates the assistant's prose from its proposal block. The
/// `Assistant` system prompt (see [`AssistCapability::system_prompt`]) asks the model
/// to end its reply with this line followed by a JSON array.
///
/// [`AssistCapability::system_prompt`]: crate::AssistCapability::system_prompt
pub const ACTION_MARKER: &str = "<<<MW_ACTIONS";

/// Upper bounds on a parsed proposal block, so a model reply can never inflate the
/// response the browser has to render.
const MAX_ACTIONS: usize = 8;
const MAX_FIELD_CHARS: usize = 240;

/// A tool action the assistant **proposes**. Nothing here executes it: the gateway
/// only reports proposals, the web renders them for human review, and anything that
/// would transmit is routed to the Outbox for the in-app confirmation (§14, R4).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProposedAction {
    /// Stable-within-a-reply id (`act-1`, `act-2`, …) the UI keys on.
    pub id: String,
    /// The tool wire name the model named (e.g. `mail.search`, `mail.send`).
    pub tool: String,
    /// The model's own one-line summary of what the tool would do.
    pub summary: String,
    /// True when confirming the proposal could eventually enqueue a send. Deliberately
    /// **over**-inclusive: a proposal wrongly marked `would_send` gets the stricter
    /// Outbox review copy, which is the safe direction.
    pub would_send: bool,
}

/// Whether a named tool could end in a transmission. Conservative by design.
fn would_send(tool: &str) -> bool {
    let t = tool.to_ascii_lowercase();
    t.contains("send") || t.contains("reply") || t.contains("forward")
}

fn clip(s: &str, max: usize) -> String {
    s.chars().take(max).collect()
}

/// The model's raw proposal entry. `summary` is optional so a terse reply still parses.
#[derive(Debug, Deserialize)]
struct RawAction {
    tool: String,
    #[serde(default)]
    summary: Option<String>,
}

/// Parse a proposal block into [`ProposedAction`]s. A block that is not a JSON array
/// of `{tool, summary}` yields **no** actions — the gateway never invents a proposal
/// out of prose.
#[must_use]
pub fn parse_actions(block: &str) -> Vec<ProposedAction> {
    let mut body = block.trim();
    // Tolerate a fenced block (```json … ```), which models emit habitually.
    if let Some(rest) = body.strip_prefix("```") {
        let rest = rest.strip_prefix("json").unwrap_or(rest);
        body = rest.trim_start().trim_end().trim_end_matches("```").trim();
    }
    let Ok(raw) = serde_json::from_str::<Vec<RawAction>>(body) else {
        return Vec::new();
    };
    raw.into_iter()
        .filter(|a| !a.tool.trim().is_empty())
        .take(MAX_ACTIONS)
        .enumerate()
        .map(|(i, a)| {
            let tool = clip(a.tool.trim(), MAX_FIELD_CHARS);
            let summary = a
                .summary
                .as_deref()
                .map(str::trim)
                .filter(|s| !s.is_empty())
                .map_or_else(|| tool.clone(), |s| clip(s, MAX_FIELD_CHARS));
            ProposedAction {
                id: format!("act-{}", i + 1),
                would_send: would_send(&tool),
                tool,
                summary,
            }
        })
        .collect()
}

/// Splits a streamed reply into the text the user sees and the trailing proposal
/// block, holding back only as many bytes as could still turn out to be the start of
/// [`ACTION_MARKER`] — so a marker split across transport chunks is never leaked as
/// visible text.
///
/// Only the `Assistant` capability proposes actions; for every other capability
/// [`ProposalFilter::for_capability`] returns a pass-through filter.
#[derive(Debug, Default)]
pub struct ProposalFilter {
    active: bool,
    /// Text seen but not yet released (may be a partial marker).
    pending: String,
    /// Everything after the marker, once seen.
    block: Option<String>,
}

impl ProposalFilter {
    /// A filter that scans for proposals (the `Assistant` capability).
    #[must_use]
    pub fn scanning() -> Self {
        Self {
            active: true,
            ..Self::default()
        }
    }

    /// A filter that passes every delta through untouched.
    #[must_use]
    pub fn passthrough() -> Self {
        Self::default()
    }

    /// Scan only for the capability that can propose actions (§14.3).
    #[must_use]
    pub fn for_capability(cap: AssistCapability) -> Self {
        match cap {
            AssistCapability::Assistant => Self::scanning(),
            _ => Self::passthrough(),
        }
    }

    /// Feed one model delta; returns the text that is safe to show the user now.
    pub fn push(&mut self, delta: &str) -> String {
        if !self.active {
            return delta.to_string();
        }
        if let Some(block) = self.block.as_mut() {
            block.push_str(delta);
            return String::new();
        }
        self.pending.push_str(delta);
        if let Some(idx) = self.pending.find(ACTION_MARKER) {
            let visible = self.pending[..idx].to_string();
            self.block = Some(self.pending[idx + ACTION_MARKER.len()..].to_string());
            self.pending.clear();
            return visible;
        }
        let keep = self.holdback();
        let split = self.pending.len() - keep;
        let visible = self.pending[..split].to_string();
        self.pending.drain(..split);
        visible
    }

    /// Longest suffix of `pending` that is a prefix of [`ACTION_MARKER`] — the bytes
    /// that must stay held back because the marker may still complete.
    fn holdback(&self) -> usize {
        let max = (ACTION_MARKER.len() - 1).min(self.pending.len());
        (1..=max)
            .rev()
            .find(|&k| {
                let at = self.pending.len() - k;
                self.pending.is_char_boundary(at) && self.pending[at..] == ACTION_MARKER[..k]
            })
            .unwrap_or(0)
    }

    /// Finish the reply: any still-held text turned out to be ordinary text, and the
    /// proposal block (if the marker ever arrived) is parsed.
    #[must_use]
    pub fn finish(self) -> (String, Vec<ProposedAction>) {
        let actions = self.block.as_deref().map(parse_actions).unwrap_or_default();
        (self.pending, actions)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The prompt has to spell the marker out as a literal, so pin the two together:
    /// a drifted marker would silently stop the filter from ever matching.
    #[test]
    fn the_assistant_prompt_teaches_the_marker_it_is_parsed_with() {
        let prompt = AssistCapability::Assistant.system_prompt();
        assert!(prompt.contains(ACTION_MARKER), "prompt names the marker");
        assert!(
            prompt.contains("cannot send"),
            "the prompt states the no-transmit rule"
        );
        for cap in AssistCapability::ALL {
            if cap != AssistCapability::Assistant {
                assert!(
                    !cap.system_prompt().contains(ACTION_MARKER),
                    "{cap:?} does not invite proposals"
                );
            }
        }
    }

    #[test]
    fn passthrough_filter_never_withholds() {
        let mut f = ProposalFilter::for_capability(AssistCapability::Summarize);
        assert_eq!(f.push("hello "), "hello ");
        assert_eq!(f.push(ACTION_MARKER), ACTION_MARKER);
        let (trailing, actions) = f.finish();
        assert!(trailing.is_empty());
        assert!(actions.is_empty(), "only the assistant proposes actions");
    }

    #[test]
    fn marker_split_across_deltas_is_never_shown() {
        let mut f = ProposalFilter::for_capability(AssistCapability::Assistant);
        let mut seen = String::new();
        // The marker arrives one character at a time, interleaved with real text.
        seen.push_str(&f.push("I can look that up."));
        for ch in ACTION_MARKER.chars() {
            seen.push_str(&f.push(&ch.to_string()));
        }
        seen.push_str(&f.push(r#"[{"tool":"mail.search","summary":"Search for Bob"}]"#));
        assert_eq!(
            seen, "I can look that up.",
            "no marker byte reaches the user"
        );
        let (trailing, actions) = f.finish();
        assert!(trailing.is_empty());
        assert_eq!(actions.len(), 1);
        assert_eq!(actions[0].id, "act-1");
        assert_eq!(actions[0].tool, "mail.search");
        assert!(!actions[0].would_send, "a search cannot transmit");
    }

    #[test]
    fn held_back_partial_marker_is_released_as_text_when_it_never_completes() {
        let mut f = ProposalFilter::scanning();
        let visible = f.push("done <<<M");
        assert_eq!(visible, "done ", "the partial marker is held back");
        let (trailing, actions) = f.finish();
        assert_eq!(trailing, "<<<M", "it was ordinary text after all");
        assert!(actions.is_empty());
    }

    #[test]
    fn multibyte_text_is_not_split_mid_character() {
        let mut f = ProposalFilter::scanning();
        assert_eq!(f.push("café ☕"), "café ☕");
        let (trailing, actions) = f.finish();
        assert!(trailing.is_empty());
        assert!(actions.is_empty());
    }

    #[test]
    fn a_send_shaped_proposal_is_flagged_for_outbox_review() {
        let actions = parse_actions(r#"[{"tool":"mail.send","summary":"Reply to Bob"}]"#);
        assert_eq!(actions.len(), 1);
        assert!(actions[0].would_send, "a send proposal is Outbox-gated");
    }

    #[test]
    fn fenced_and_malformed_blocks() {
        let fenced = parse_actions("```json\n[{\"tool\":\"mail.search\"}]\n```");
        assert_eq!(fenced.len(), 1);
        assert_eq!(
            fenced[0].summary, "mail.search",
            "summary falls back to the tool name"
        );
        assert!(
            parse_actions("sorry, I cannot do that").is_empty(),
            "prose never becomes a proposal"
        );
        assert!(parse_actions(r#"[{"summary":"no tool"}]"#).is_empty());
        assert!(parse_actions(r#"[{"tool":"  "}]"#).is_empty());
    }

    #[test]
    fn action_count_and_field_length_are_bounded() {
        let many: String = format!(
            "[{}]",
            (0..40)
                .map(|i| format!(r#"{{"tool":"t{i}","summary":"{}"}}"#, "x".repeat(1000)))
                .collect::<Vec<_>>()
                .join(",")
        );
        let actions = parse_actions(&many);
        assert_eq!(actions.len(), MAX_ACTIONS);
        assert!(
            actions
                .iter()
                .all(|a| a.summary.chars().count() <= MAX_FIELD_CHARS)
        );
    }
}
