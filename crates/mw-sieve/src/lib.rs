#![forbid(unsafe_code)]
//! `mw-sieve` — rules/automation (plan §0.6, §3 e2, SPEC §15.5).
//!
//! A GUI [`Rule`] model compiles to Sieve (RFC 5228) for upload via ManageSieve
//! (RFC 5804) where the server advertises it; otherwise the engine-side
//! [`evaluate`] runs the rule at ingest (the always-green path). A raw-Sieve
//! lint rounds out the editor. GUI→Sieve generation is the gate; Sieve→GUI
//! parse-back is best-effort (plan §0).
//!
//! ## Scaffolder note (e0)
//! e0 authors ONLY the frozen [`Rule`]/[`Condition`]/[`Action`] model + the
//! [`evaluate`] signature. e2 owns the whole crate — Sieve generation, the
//! ManageSieve client, the evaluator, the linter, and the Sieve-text fuzz
//! target (plan §1.12). Bodies are `todo!()`.

use serde::{Deserialize, Serialize};

/// A GUI rule: when ALL (or ANY) conditions match, run the actions in order.
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct Rule {
    pub id: String,
    pub name: String,
    /// `true` = all conditions must match; `false` = any.
    pub match_all: bool,
    pub conditions: Vec<Condition>,
    pub actions: Vec<Action>,
    /// Disabled rules are kept but not evaluated/generated.
    pub enabled: bool,
}

/// A single match condition (plan §3 e2).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase", tag = "field", content = "test")]
pub enum Condition {
    From(StringTest),
    To(StringTest),
    Subject(StringTest),
    Body(StringTest),
    HasAttachment,
    /// Size in bytes; `over` = larger-than, else smaller-than.
    Size {
        over: bool,
        bytes: u64,
    },
    Keyword(StringTest),
}

/// How a string condition matches its operand.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct StringTest {
    pub op: MatchOp,
    pub value: String,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum MatchOp {
    Contains,
    Is,
    Matches,
}

/// An action to take when a rule fires (plan §3 e2).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase", tag = "kind")]
pub enum Action {
    Move { mailbox: String },
    Copy { mailbox: String },
    Tag { keyword: String },
    Mark { keyword: String },
    Forward { address: String },
    ReplyTemplate { template: String },
    Notify { message: String },
    Stop,
}

/// The minimal parsed-envelope view the engine-side evaluator matches against
/// (plan §3 e2). e2 refines this against `mw-mime` at integration.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ParsedEnvelope {
    pub from: String,
    pub to: String,
    pub subject: String,
    pub body: String,
    pub has_attachment: bool,
    pub size: u64,
    pub keywords: Vec<String>,
}

#[derive(Debug, thiserror::Error)]
pub enum SieveError {
    #[error("sieve generation error: {0}")]
    Generate(String),
    #[error("managesieve error: {0}")]
    ManageSieve(String),
    #[error("lint error: {0}")]
    Lint(String),
}

pub type Result<T> = std::result::Result<T, SieveError>;

/// Generate RFC 5228 Sieve source for a set of rules (the GUI→Sieve gate).
#[allow(unused_variables)]
pub fn generate(rules: &[Rule]) -> Result<String> {
    todo!("e2: RFC 5228 generation")
}

/// Engine-side evaluation (the no-ManageSieve path): the ordered actions a rule
/// yields for a message. e2 owns the semantics.
#[allow(unused_variables)]
pub fn evaluate(rule: &Rule, envelope: &ParsedEnvelope) -> Vec<Action> {
    todo!("e2: engine-side evaluator")
}

/// Lint raw Sieve text, returning diagnostics (empty = clean).
#[allow(unused_variables)]
pub fn lint(sieve: &str) -> Result<Vec<String>> {
    todo!("e2: raw-Sieve lint")
}
