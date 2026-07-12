//! GUI [`Rule`] model → Sieve (RFC 5228) source — the GATE direction (plan §0.6).
//!
//! Generation is deterministic and self-contained: it collects exactly the
//! extensions the emitted commands use into a single leading `require`, then
//! renders each enabled rule as an `if allof/anyof (...) { ... }` block. The
//! output is what the golden tests snapshot and what ManageSieve `PUTSCRIPT`
//! uploads, so the shape is stable and every string is escaped.
//!
//! ## Action mapping (RFC 5228 + extensions)
//! - `Move`  → `fileinto "<mbox>";`            (`fileinto`)
//! - `Copy`  → `fileinto :copy "<mbox>";`      (`fileinto`, `copy`)
//! - `Tag`   → `addflag "<kw>";`               (`imap4flags`)
//! - `Mark`  → `addflag "<kw>";`               (`imap4flags`)
//! - `Forward` → `redirect "<addr>";`          (core)
//! - `ReplyTemplate` → `vacation "<tpl>";`     (`vacation`)
//! - `Notify` → `# notify: <msg>` comment      — the frozen `Notify{message}`
//!   carries no delivery method/target, so on the ManageSieve path it is emitted
//!   as an informational comment (keeping the script RFC-valid); the engine-side
//!   [`crate::evaluate`] executes the real notification.
//! - `Stop`  → `stop;`                         (core)

use std::collections::BTreeSet;

use crate::{Action, Condition, MatchOp, Result, Rule};

/// Generate RFC 5228 Sieve source for a set of rules (the GUI→Sieve gate).
///
/// Disabled rules are skipped entirely. Returns a `require [...]` header (only
/// when extensions are used) followed by one block per enabled rule.
pub fn generate(rules: &[Rule]) -> Result<String> {
    let mut extensions: BTreeSet<&'static str> = BTreeSet::new();
    let mut body = String::new();

    let enabled: Vec<&Rule> = rules.iter().filter(|r| r.enabled).collect();
    for (i, rule) in enabled.iter().enumerate() {
        if i > 0 {
            body.push('\n');
        }
        render_rule(rule, &mut body, &mut extensions);
    }

    let mut out = String::new();
    if !extensions.is_empty() {
        let quoted: Vec<String> = extensions.iter().map(|e| quote(e)).collect();
        out.push_str(&format!("require [{}];\n", quoted.join(", ")));
        if !body.is_empty() {
            out.push('\n');
        }
    }
    out.push_str(&body);
    Ok(out)
}

fn render_rule(rule: &Rule, out: &mut String, ext: &mut BTreeSet<&'static str>) {
    out.push_str(&format!("# rule: {}\n", sanitize_comment(&rule.name)));

    let tests: Vec<String> = rule
        .conditions
        .iter()
        .map(|c| render_condition(c, ext))
        .collect();

    // Build the action block once — reused whether or not there is a guard.
    let mut actions = String::new();
    for action in &rule.actions {
        let (line, requires) = render_action(action);
        for r in requires {
            ext.insert(r);
        }
        actions.push_str("    ");
        actions.push_str(&line);
        actions.push('\n');
    }

    match tests.len() {
        0 => {
            // No conditions ⇒ the rule always fires; emit actions unguarded.
            for action in &rule.actions {
                let (line, _) = render_action(action);
                out.push_str(&line);
                out.push('\n');
            }
        }
        1 => {
            out.push_str(&format!("if {} {{\n{actions}}}\n", tests[0]));
        }
        _ => {
            let combinator = if rule.match_all { "allof" } else { "anyof" };
            out.push_str(&format!(
                "if {combinator} ({}) {{\n{actions}}}\n",
                tests.join(", ")
            ));
        }
    }
}

/// Render one condition to a Sieve test, recording any required extension.
fn render_condition(cond: &Condition, ext: &mut BTreeSet<&'static str>) -> String {
    match cond {
        Condition::From(t) => format!("address {} \"from\" {}", match_type(t.op), quote(&t.value)),
        Condition::To(t) => format!("address {} \"to\" {}", match_type(t.op), quote(&t.value)),
        Condition::Subject(t) => {
            format!(
                "header {} \"subject\" {}",
                match_type(t.op),
                quote(&t.value)
            )
        }
        Condition::Body(t) => {
            ext.insert("body");
            format!("body :text {} {}", match_type(t.op), quote(&t.value))
        }
        // No portable core "has attachment" test; use the common multipart/mixed
        // Content-Type heuristic (documented — the evaluator is exact).
        Condition::HasAttachment => {
            "header :contains \"content-type\" \"multipart/mixed\"".to_string()
        }
        Condition::Size { over, bytes } => {
            let op = if *over { ":over" } else { ":under" };
            format!("size {op} {bytes}")
        }
        Condition::Keyword(t) => {
            ext.insert("imap4flags");
            format!("hasflag {} {}", match_type(t.op), quote(&t.value))
        }
    }
}

/// Render one action to a Sieve command plus the extensions it requires.
fn render_action(action: &Action) -> (String, &'static [&'static str]) {
    match action {
        Action::Move { mailbox } => (format!("fileinto {};", quote(mailbox)), &["fileinto"]),
        Action::Copy { mailbox } => (
            format!("fileinto :copy {};", quote(mailbox)),
            &["fileinto", "copy"],
        ),
        Action::Tag { keyword } => (format!("addflag {};", quote(keyword)), &["imap4flags"]),
        Action::Mark { keyword } => (format!("addflag {};", quote(keyword)), &["imap4flags"]),
        Action::Forward { address } => (format!("redirect {};", quote(address)), &[]),
        Action::ReplyTemplate { template } => {
            (format!("vacation {};", quote(template)), &["vacation"])
        }
        Action::Notify { message } => (format!("# notify: {}", sanitize_comment(message)), &[]),
        Action::Stop => ("stop;".to_string(), &[]),
    }
}

fn match_type(op: MatchOp) -> &'static str {
    match op {
        MatchOp::Contains => ":contains",
        MatchOp::Is => ":is",
        MatchOp::Matches => ":matches",
    }
}

/// Quote a Sieve string literal (RFC 5228 §2.4.2): escape `\` and `"`; fold any
/// newline to a space so a single quoted string stays on one line.
fn quote(s: &str) -> String {
    let mut out = String::with_capacity(s.len() + 2);
    out.push('"');
    for c in s.chars() {
        match c {
            '\\' => out.push_str("\\\\"),
            '"' => out.push_str("\\\""),
            '\r' | '\n' => out.push(' '),
            other => out.push(other),
        }
    }
    out.push('"');
    out
}

/// A `# comment` runs to end-of-line, so newlines are folded to spaces.
fn sanitize_comment(s: &str) -> String {
    s.replace(['\r', '\n'], " ")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::StringTest;

    fn st(op: MatchOp, v: &str) -> StringTest {
        StringTest {
            op,
            value: v.into(),
        }
    }

    #[test]
    fn empty_ruleset_is_empty() {
        assert_eq!(generate(&[]).unwrap(), "");
    }

    #[test]
    fn disabled_rules_are_skipped() {
        let rule = Rule {
            id: "1".into(),
            name: "off".into(),
            match_all: true,
            conditions: vec![Condition::Subject(st(MatchOp::Contains, "x"))],
            actions: vec![Action::Stop],
            enabled: false,
        };
        assert_eq!(generate(&[rule]).unwrap(), "");
    }

    #[test]
    fn single_condition_omits_allof() {
        let rule = Rule {
            id: "1".into(),
            name: "News".into(),
            match_all: true,
            conditions: vec![Condition::From(st(MatchOp::Contains, "news@example.com"))],
            actions: vec![Action::Move {
                mailbox: "INBOX/News".into(),
            }],
            enabled: true,
        };
        let out = generate(&[rule]).unwrap();
        assert_eq!(
            out,
            "require [\"fileinto\"];\n\n# rule: News\n\
             if address :contains \"from\" \"news@example.com\" {\n\
             \x20\x20\x20\x20fileinto \"INBOX/News\";\n}\n"
        );
    }

    #[test]
    fn multi_condition_uses_combinator_and_collects_extensions() {
        let rule = Rule {
            id: "1".into(),
            name: "Big".into(),
            match_all: false,
            conditions: vec![
                Condition::HasAttachment,
                Condition::Size {
                    over: true,
                    bytes: 1_000_000,
                },
                Condition::Body(st(MatchOp::Contains, "invoice")),
            ],
            actions: vec![
                Action::Copy {
                    mailbox: "Archive".into(),
                },
                Action::Tag {
                    keyword: "big".into(),
                },
                Action::Stop,
            ],
            enabled: true,
        };
        let out = generate(&[rule]).unwrap();
        assert!(out.starts_with("require [\"body\", \"copy\", \"fileinto\", \"imap4flags\"];\n"));
        assert!(out.contains(
            "if anyof (header :contains \"content-type\" \"multipart/mixed\", \
             size :over 1000000, body :text :contains \"invoice\") {"
        ));
        assert!(out.contains("    fileinto :copy \"Archive\";\n"));
        assert!(out.contains("    addflag \"big\";\n"));
        assert!(out.contains("    stop;\n"));
    }

    #[test]
    fn no_conditions_emit_unguarded_actions() {
        let rule = Rule {
            id: "1".into(),
            name: "always".into(),
            match_all: true,
            conditions: vec![],
            actions: vec![Action::Forward {
                address: "a@b.com".into(),
            }],
            enabled: true,
        };
        // redirect is core ⇒ no require header.
        assert_eq!(
            generate(&[rule]).unwrap(),
            "# rule: always\nredirect \"a@b.com\";\n"
        );
    }

    #[test]
    fn strings_are_escaped() {
        assert_eq!(quote(r#"a"b\c"#), r#""a\"b\\c""#);
    }

    #[test]
    fn notify_is_a_comment_and_needs_no_extension() {
        let rule = Rule {
            id: "1".into(),
            name: "n".into(),
            match_all: true,
            conditions: vec![],
            actions: vec![Action::Notify {
                message: "you got mail".into(),
            }],
            enabled: true,
        };
        assert_eq!(
            generate(&[rule]).unwrap(),
            "# rule: n\n# notify: you got mail\n"
        );
    }

    #[test]
    fn generated_sieve_lints_clean() {
        // A rule exercising every extension must pass our own linter.
        let rule = Rule {
            id: "1".into(),
            name: "everything".into(),
            match_all: true,
            conditions: vec![
                Condition::From(st(MatchOp::Is, "boss@example.com")),
                Condition::Keyword(st(MatchOp::Contains, "urgent")),
            ],
            actions: vec![
                Action::Copy {
                    mailbox: "Work".into(),
                },
                Action::Mark {
                    keyword: "\\Flagged".into(),
                },
                Action::ReplyTemplate {
                    template: "Out of office".into(),
                },
                Action::Forward {
                    address: "assistant@example.com".into(),
                },
                Action::Stop,
            ],
            enabled: true,
        };
        let sieve = generate(&[rule]).unwrap();
        let diagnostics = crate::lint(&sieve).unwrap();
        assert!(
            diagnostics.is_empty(),
            "unexpected lint output: {diagnostics:?}"
        );
    }
}
