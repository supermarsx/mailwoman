#![forbid(unsafe_code)]
//! `mw-sieve` — rules/automation (plan §0.6, §3 e2, SPEC §15.5).
//!
//! A GUI [`Rule`] model compiles to Sieve (RFC 5228) for upload via ManageSieve
//! (RFC 5804) where the server advertises it; otherwise the engine-side
//! [`evaluate`] runs the rule at ingest (the always-green path). A raw-Sieve
//! [`lint`] rounds out the editor. GUI→Sieve generation is the gate; Sieve→GUI
//! parse-back is best-effort (plan §0).
//!
//! ## Layers
//! - the [`Rule`]/[`Condition`]/[`Action`] model (this module, frozen by e0);
//! - [`codegen`] — [`Rule`]`[]` → Sieve source (the GATE direction);
//! - [`parse`] — Sieve source → [`Rule`], the round-trip inverse of [`codegen`]
//!   over the constrained subset it emits (`parse(generate(rules)) == rules`);
//! - [`eval`] — the engine-side [`evaluate`]/[`evaluate_all`] (no-ManageSieve path);
//! - [`lint`] — basic syntactic validation of raw Sieve text;
//! - [`managesieve`] — the RFC 5804 client (CAPABILITY, AUTHENTICATE, PUTSCRIPT,
//!   LISTSCRIPTS, SETACTIVE, GETSCRIPT) for servers that advertise it.
//!
//! The Sieve-text [`fuzz_sieve_text`] entry point drives the `cargo-fuzz` target
//! and the in-crate corpus smoke test — no input may panic the linter/parser.

use serde::{Deserialize, Serialize};

pub mod client;
pub mod codegen;
pub mod eval;
pub mod lint;
pub mod managesieve;
pub mod parse;
mod tls;
pub mod transport;

pub use client::ManageSieveClient;
pub use codegen::generate;
pub use eval::{evaluate, evaluate_all};
pub use lint::lint;
pub use managesieve::{Capabilities, Connection, Credentials, ScriptInfo};
pub use parse::parse;
pub use transport::{SieveStream, TlsMode};

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
    #[error("parse error: {0}")]
    Parse(String),
    #[error("i/o error: {0}")]
    Io(#[from] std::io::Error),
}

pub type Result<T> = std::result::Result<T, SieveError>;

/// Fuzz/robustness entry point: run the raw-Sieve linter and the best-effort
/// tokenizer over arbitrary bytes. The invariant (§1.12): no input, however
/// malformed or truncated, may panic or hang.
///
/// Drives the `cargo-fuzz` target (`fuzz/fuzz_targets/sieve_text.rs`) and the
/// in-crate corpus smoke test.
pub fn fuzz_sieve_text(data: &[u8]) {
    let text = String::from_utf8_lossy(data);
    let _ = lint(&text);
    // The best-effort tokenizer must also survive arbitrary input.
    let _ = lint::tokenize(&text);
    // The round-trip parser must never panic either — it returns `Err` on
    // anything outside the codegen subset, but never unwinds.
    let _ = parse(&text);
}

#[cfg(test)]
mod smoke {
    use super::*;

    /// A corpus of realistic and adversarial Sieve fragments must lint without
    /// panicking (the fuzz invariant, runnable under plain `cargo test`).
    #[test]
    fn corpus_never_panics() {
        let corpus: &[&[u8]] = &[
            b"require [\"fileinto\"];\r\nif header :contains \"subject\" \"x\" { fileinto \"A\"; }\r\n",
            b"# just a comment\n",
            b"/* block comment */ keep;",
            b"{{{{{{{{",
            b"require [\"",
            b"if allof (",
            b"\xff\xfe\x00\x01 garbage",
            b"",
            b"fileinto \"A\" \"B\" \"C\";",
            b"\"unterminated string",
            b"{999999999999999999999}",
            b"if header :contains \"subject\" \"\\\"quoted\\\"\" { stop; }",
        ];
        for bytes in corpus {
            fuzz_sieve_text(bytes);
        }
    }
}
