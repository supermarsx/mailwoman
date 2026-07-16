//! Sieve source → GUI [`Rule`] model — the round-trip inverse of
//! [`codegen::generate`](crate::codegen) (audit #1, SPEC §6.1/§10.5).
//!
//! [`codegen`](crate::codegen) is the GATE direction (GUI → Sieve). This module
//! is its reader: a hand-rolled recursive-descent parser over the existing
//! [`lint::tokenize_with_comments`](crate::lint::tokenize_with_comments) token
//! stream — **no new dependency**. It targets exactly the constrained subset the
//! generator emits, so `parse(generate(rules))` reconstructs the rule set (the
//! round-trip property test below covers the full Condition/Action matrix).
//!
//! ## What the generator does *not* encode (and how we reconstruct it)
//! A few `Rule` fields have no Sieve representation, so the parser normalizes
//! them the way codegen collapses them (documented so the round-trip is exact):
//! - **`id`** — never written. The parser assigns the 1-based rule index as a
//!   decimal string (`"1"`, `"2"`, …).
//! - **`match_all`** — only encoded as `allof`/`anyof` for **two or more**
//!   conditions. With 0 or 1 condition the generator omits the combinator, so the
//!   parser defaults `match_all = true`.
//! - **`enabled`** — disabled rules generate nothing, so every parsed rule is
//!   `enabled = true`.
//! - **`Action::Mark`** — codegen emits it as `addflag`, identical to
//!   `Action::Tag`; the parser therefore reads any `addflag` back as `Tag`
//!   (`Mark` is a UI-only distinction with no Sieve surface).

use crate::lint::{Token, tokenize_with_comments};
use crate::{Action, Condition, MatchOp, Result, Rule, SieveError, StringTest};

/// Parse Sieve source produced by [`codegen::generate`](crate::codegen) back into
/// the [`Rule`] model. Returns the rules in source order; the leading `require`
/// header (and any other preamble before the first `# rule:` marker) is ignored,
/// since the required extensions are re-derived by codegen.
pub fn parse(input: &str) -> Result<Vec<Rule>> {
    let tokens = tokenize_with_comments(input);
    let mut p = Parser {
        toks: &tokens,
        pos: 0,
    };
    let mut rules = Vec::new();

    // Advance to each `# rule:` marker; everything before the first one (the
    // `require [...]` header) is preamble and skipped.
    while let Some(tok) = p.peek() {
        if let Token::Comment(text) = tok
            && let Some(name) = rule_name(text)
        {
            p.pos += 1;
            let id = (rules.len() + 1).to_string();
            rules.push(p.parse_rule_body(id, name)?);
            continue;
        }
        p.pos += 1;
    }
    Ok(rules)
}

/// `# rule: <name>` marker → the `<name>` (codegen writes `"# rule: {name}"`).
fn rule_name(comment: &str) -> Option<String> {
    comment.strip_prefix(" rule: ").map(str::to_string)
}

/// `# notify: <msg>` marker → the `<msg>` (codegen's [`Action::Notify`] rendering).
fn notify_message(comment: &str) -> Option<String> {
    comment.strip_prefix(" notify: ").map(str::to_string)
}

struct Parser<'a> {
    toks: &'a [Token],
    pos: usize,
}

impl<'a> Parser<'a> {
    fn peek(&self) -> Option<&'a Token> {
        self.toks.get(self.pos)
    }

    fn bump(&mut self) -> Option<&'a Token> {
        let t = self.toks.get(self.pos);
        if t.is_some() {
            self.pos += 1;
        }
        t
    }

    fn err<T>(&self, what: &str) -> Result<T> {
        Err(SieveError::Parse(format!(
            "{what} at token {}: {:?}",
            self.pos,
            self.peek()
        )))
    }

    /// Consume the body of one rule (its optional `if` guard + action block, or —
    /// for a conditionless rule — the bare action statements) up to the next
    /// `# rule:` marker or end-of-input.
    fn parse_rule_body(&mut self, id: String, name: String) -> Result<Rule> {
        if matches!(self.peek(), Some(Token::Word(w)) if w == "if") {
            self.pos += 1;
            let (match_all, conditions) = self.parse_guard()?;
            self.expect_punct('{')?;
            let actions = self.parse_block_actions()?;
            return Ok(Rule {
                id,
                name,
                match_all,
                conditions,
                actions,
                enabled: true,
            });
        }
        // Conditionless rule: unguarded statements until the next rule / EOF.
        let actions = self.parse_unguarded_actions()?;
        Ok(Rule {
            id,
            name,
            match_all: true,
            conditions: vec![],
            actions,
            enabled: true,
        })
    }

    /// Parse the test expression after `if`: either `allof(...)`/`anyof(...)` with
    /// two or more comma-separated tests, or a single bare test.
    fn parse_guard(&mut self) -> Result<(bool, Vec<Condition>)> {
        match self.peek() {
            Some(Token::Word(w)) if w == "allof" || w == "anyof" => {
                let match_all = w == "allof";
                self.pos += 1;
                self.expect_punct('(')?;
                let mut conditions = Vec::new();
                loop {
                    conditions.push(self.parse_condition()?);
                    match self.bump() {
                        Some(Token::Punct(',')) => continue,
                        Some(Token::Punct(')')) => break,
                        _ => return self.err("expected `,` or `)` in test list"),
                    }
                }
                Ok((match_all, conditions))
            }
            _ => Ok((true, vec![self.parse_condition()?])),
        }
    }

    /// Parse one Sieve test emitted by [`codegen::render_condition`]. Each test has
    /// a fixed token shape keyed by its leading command word.
    fn parse_condition(&mut self) -> Result<Condition> {
        let cmd = self.expect_word()?;
        match cmd.as_str() {
            "address" => {
                let op = self.expect_matchop()?;
                let field = self.expect_str()?;
                let value = self.expect_str()?;
                let test = StringTest { op, value };
                match field.as_str() {
                    "from" => Ok(Condition::From(test)),
                    "to" => Ok(Condition::To(test)),
                    _ => self.err("unknown address field"),
                }
            }
            "header" => {
                let op = self.expect_matchop()?;
                let field = self.expect_str()?;
                let value = self.expect_str()?;
                // The attachment heuristic codegen emits for `HasAttachment`.
                if field == "content-type" && op == MatchOp::Contains && value == "multipart/mixed"
                {
                    return Ok(Condition::HasAttachment);
                }
                match field.as_str() {
                    "subject" => Ok(Condition::Subject(StringTest { op, value })),
                    _ => self.err("unknown header field"),
                }
            }
            "body" => {
                self.expect_exact_word(":text")?;
                let op = self.expect_matchop()?;
                let value = self.expect_str()?;
                Ok(Condition::Body(StringTest { op, value }))
            }
            "size" => {
                let tag = self.expect_word()?;
                let over = match tag.as_str() {
                    ":over" => true,
                    ":under" => false,
                    _ => return self.err("expected `:over`/`:under`"),
                };
                let num = self.expect_word()?;
                let bytes = num
                    .parse::<u64>()
                    .map_err(|_| SieveError::Parse(format!("bad size literal `{num}`")))?;
                Ok(Condition::Size { over, bytes })
            }
            "hasflag" => {
                let op = self.expect_matchop()?;
                let value = self.expect_str()?;
                Ok(Condition::Keyword(StringTest { op, value }))
            }
            _ => self.err("unknown test command"),
        }
    }

    /// Statements inside a `{ ... }` block, up to the closing brace.
    fn parse_block_actions(&mut self) -> Result<Vec<Action>> {
        let mut actions = Vec::new();
        loop {
            match self.peek() {
                Some(Token::Punct('}')) => {
                    self.pos += 1;
                    return Ok(actions);
                }
                None => return self.err("unterminated action block (missing `}`)"),
                _ => {
                    if let Some(a) = self.parse_action()? {
                        actions.push(a);
                    }
                }
            }
        }
    }

    /// Bare statements for a conditionless rule, up to the next `# rule:` marker
    /// or end-of-input.
    fn parse_unguarded_actions(&mut self) -> Result<Vec<Action>> {
        let mut actions = Vec::new();
        loop {
            match self.peek() {
                None => return Ok(actions),
                Some(Token::Comment(text)) if rule_name(text).is_some() => return Ok(actions),
                _ => {
                    if let Some(a) = self.parse_action()? {
                        actions.push(a);
                    }
                }
            }
        }
    }

    /// Parse one action statement. Returns `Ok(None)` for a comment that is not a
    /// `# notify:` marker (a stray comment is skipped, not an error).
    fn parse_action(&mut self) -> Result<Option<Action>> {
        match self.bump() {
            Some(Token::Comment(text)) => {
                Ok(notify_message(text).map(|m| Action::Notify { message: m }))
            }
            Some(Token::Word(w)) => {
                let action = match w.as_str() {
                    "fileinto" => {
                        // `fileinto :copy "mbox";` → Copy, else `fileinto "mbox";` → Move.
                        if matches!(self.peek(), Some(Token::Word(t)) if t == ":copy") {
                            self.pos += 1;
                            Action::Copy {
                                mailbox: self.expect_str()?,
                            }
                        } else {
                            Action::Move {
                                mailbox: self.expect_str()?,
                            }
                        }
                    }
                    "addflag" => Action::Tag {
                        keyword: self.expect_str()?,
                    },
                    "redirect" => Action::Forward {
                        address: self.expect_str()?,
                    },
                    "vacation" => Action::ReplyTemplate {
                        template: self.expect_str()?,
                    },
                    "stop" => Action::Stop,
                    _ => return self.err("unknown action command"),
                };
                self.expect_punct(';')?;
                Ok(Some(action))
            }
            _ => self.err("expected an action"),
        }
    }

    // ── token expectations ───────────────────────────────────────────────────

    fn expect_word(&mut self) -> Result<String> {
        match self.bump() {
            Some(Token::Word(w)) => Ok(w.clone()),
            _ => self.err("expected a word"),
        }
    }

    fn expect_exact_word(&mut self, want: &str) -> Result<()> {
        match self.bump() {
            Some(Token::Word(w)) if w == want => Ok(()),
            _ => self.err(&format!("expected `{want}`")),
        }
    }

    fn expect_str(&mut self) -> Result<String> {
        match self.bump() {
            Some(Token::Str(s)) => Ok(s.clone()),
            _ => self.err("expected a string literal"),
        }
    }

    fn expect_punct(&mut self, want: char) -> Result<()> {
        match self.bump() {
            Some(Token::Punct(c)) if *c == want => Ok(()),
            _ => self.err(&format!("expected `{want}`")),
        }
    }

    fn expect_matchop(&mut self) -> Result<MatchOp> {
        let w = self.expect_word()?;
        match w.as_str() {
            ":contains" => Ok(MatchOp::Contains),
            ":is" => Ok(MatchOp::Is),
            ":matches" => Ok(MatchOp::Matches),
            _ => self.err("expected a match tag (`:contains`/`:is`/`:matches`)"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::generate;

    fn st(op: MatchOp, v: &str) -> StringTest {
        StringTest {
            op,
            value: v.into(),
        }
    }

    /// Build a canonical rule set covering every Condition and Action variant. The
    /// rules are already in the shape codegen preserves (ids `"1"`.., `enabled`,
    /// `match_all = true` for <2 conditions, `Tag` not `Mark`) so equality is exact.
    fn matrix() -> Vec<Rule> {
        vec![
            // Single condition of each string kind + each match op.
            Rule {
                id: "1".into(),
                name: "from-contains".into(),
                match_all: true,
                conditions: vec![Condition::From(st(MatchOp::Contains, "news@example.com"))],
                actions: vec![Action::Move {
                    mailbox: "INBOX/News".into(),
                }],
                enabled: true,
            },
            Rule {
                id: "2".into(),
                name: "to-is".into(),
                match_all: true,
                conditions: vec![Condition::To(st(MatchOp::Is, "me@example.com"))],
                actions: vec![Action::Copy {
                    mailbox: "Sent".into(),
                }],
                enabled: true,
            },
            Rule {
                id: "3".into(),
                name: "subject-matches".into(),
                match_all: true,
                conditions: vec![Condition::Subject(st(MatchOp::Matches, "*sale*"))],
                actions: vec![Action::Tag {
                    keyword: "promo".into(),
                }],
                enabled: true,
            },
            Rule {
                id: "4".into(),
                name: "body".into(),
                match_all: true,
                conditions: vec![Condition::Body(st(MatchOp::Contains, "invoice"))],
                actions: vec![Action::Forward {
                    address: "acct@example.com".into(),
                }],
                enabled: true,
            },
            // Multi-condition allof + every remaining Condition/Action.
            Rule {
                id: "5".into(),
                name: "everything allof".into(),
                match_all: true,
                conditions: vec![
                    Condition::HasAttachment,
                    Condition::Size {
                        over: true,
                        bytes: 1_000_000,
                    },
                    Condition::Keyword(st(MatchOp::Contains, "urgent")),
                ],
                actions: vec![
                    Action::ReplyTemplate {
                        template: "Out of office".into(),
                    },
                    Action::Notify {
                        message: "you got mail".into(),
                    },
                    Action::Stop,
                ],
                enabled: true,
            },
            // Multi-condition anyof (match_all = false round-trips via `anyof`).
            Rule {
                id: "6".into(),
                name: "anyof".into(),
                match_all: false,
                conditions: vec![
                    Condition::Size {
                        over: false,
                        bytes: 42,
                    },
                    Condition::From(st(MatchOp::Is, "spammer@x.test")),
                ],
                actions: vec![Action::Move {
                    mailbox: "Junk".into(),
                }],
                enabled: true,
            },
            // Conditionless rule (unguarded actions).
            Rule {
                id: "7".into(),
                name: "always".into(),
                match_all: true,
                conditions: vec![],
                actions: vec![
                    Action::Forward {
                        address: "a@b.com".into(),
                    },
                    Action::Stop,
                ],
                enabled: true,
            },
            // Escaping: values carrying `"` and `\`.
            Rule {
                id: "8".into(),
                name: r#"quote " and \ back"#.into(),
                match_all: true,
                conditions: vec![Condition::Subject(st(MatchOp::Is, r#"a "b" \c"#))],
                actions: vec![Action::Tag {
                    keyword: r"\Flagged".into(),
                }],
                enabled: true,
            },
        ]
    }

    #[test]
    fn round_trip_full_matrix() {
        let rules = matrix();
        let sieve = generate(&rules).unwrap();
        let parsed = parse(&sieve).unwrap();
        assert_eq!(parsed, rules, "generated Sieve:\n{sieve}");
    }

    #[test]
    fn round_trip_each_rule_independently() {
        // Each rule alone round-trips to id "1" (index-based id assignment).
        for rule in matrix() {
            let one = Rule {
                id: "1".into(),
                ..rule.clone()
            };
            let sieve = generate(std::slice::from_ref(&one)).unwrap();
            let parsed = parse(&sieve).unwrap();
            assert_eq!(parsed, vec![one], "generated Sieve:\n{sieve}");
        }
    }

    #[test]
    fn empty_input_parses_to_no_rules() {
        assert_eq!(parse(&generate(&[]).unwrap()).unwrap(), Vec::<Rule>::new());
        assert_eq!(parse("").unwrap(), Vec::<Rule>::new());
    }

    #[test]
    fn require_header_is_ignored() {
        let sieve = "require [\"fileinto\", \"imap4flags\"];\n\n# rule: r\nif address :is \"from\" \"x@y\" {\n    fileinto \"A\";\n}\n";
        let parsed = parse(sieve).unwrap();
        assert_eq!(parsed.len(), 1);
        assert_eq!(parsed[0].name, "r");
    }

    #[test]
    fn mark_reads_back_as_tag() {
        // Mark has no distinct Sieve surface (both emit `addflag`) → parses as Tag.
        let rule = Rule {
            id: "1".into(),
            name: "m".into(),
            match_all: true,
            conditions: vec![Condition::Subject(st(MatchOp::Contains, "x"))],
            actions: vec![Action::Mark {
                keyword: "seen".into(),
            }],
            enabled: true,
        };
        let parsed = parse(&generate(&[rule]).unwrap()).unwrap();
        assert_eq!(
            parsed[0].actions,
            vec![Action::Tag {
                keyword: "seen".into()
            }]
        );
    }

    #[test]
    fn notify_recovered_from_comment() {
        let rule = Rule {
            id: "1".into(),
            name: "n".into(),
            match_all: true,
            conditions: vec![],
            actions: vec![Action::Notify {
                message: "ping: done".into(),
            }],
            enabled: true,
        };
        let parsed = parse(&generate(std::slice::from_ref(&rule)).unwrap()).unwrap();
        assert_eq!(parsed, vec![rule]);
    }

    #[test]
    fn malformed_input_never_panics() {
        // parse must survive arbitrary/hand-broken input (returns Ok or Err, no panic).
        for s in [
            "# rule: broken\nif allof (",
            "# rule: x\nif address :is \"from\" {",
            "# rule: y\nfileinto;",
            "garbage tokens ][ ;",
            "# rule: z\nif size :over notanumber { stop; }",
        ] {
            let _ = parse(s);
        }
    }
}
