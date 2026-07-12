//! Engine-side rule evaluation — the always-green path for servers without
//! ManageSieve (plan §0.6, §3 e2).
//!
//! [`evaluate`] runs a single rule against a [`ParsedEnvelope`] and returns the
//! ordered actions it yields (empty when the rule is disabled or does not
//! match). [`evaluate_all`] runs a rule list in order with Sieve `stop`
//! semantics: once a fired rule contains [`Action::Stop`], evaluation halts and
//! no later rule contributes.
//!
//! String matching follows Sieve's default `i;ascii-casemap` comparator
//! (case-insensitive) for `:contains`/`:is`; `:matches` uses Sieve glob
//! wildcards (`*` any run, `?` any one character).

use crate::{Action, Condition, MatchOp, ParsedEnvelope, Rule, StringTest};

/// Evaluate one rule: the ordered actions it fires for `envelope`, or empty if
/// it is disabled or its conditions do not match.
pub fn evaluate(rule: &Rule, envelope: &ParsedEnvelope) -> Vec<Action> {
    if !rule.enabled || !rule_matches(rule, envelope) {
        return Vec::new();
    }
    rule.actions.clone()
}

/// Evaluate a rule list in order, honouring `stop`: actions accumulate across
/// rules until a fired rule's [`Action::Stop`] halts the chain (the `stop`
/// itself is included, mirroring Sieve).
pub fn evaluate_all(rules: &[Rule], envelope: &ParsedEnvelope) -> Vec<Action> {
    let mut out = Vec::new();
    for rule in rules {
        if !rule.enabled || !rule_matches(rule, envelope) {
            continue;
        }
        for action in &rule.actions {
            out.push(action.clone());
            if matches!(action, Action::Stop) {
                return out;
            }
        }
    }
    out
}

fn rule_matches(rule: &Rule, envelope: &ParsedEnvelope) -> bool {
    if rule.conditions.is_empty() {
        return true; // no conditions ⇒ always fires (mirrors generation)
    }
    if rule.match_all {
        rule.conditions
            .iter()
            .all(|c| condition_matches(c, envelope))
    } else {
        rule.conditions
            .iter()
            .any(|c| condition_matches(c, envelope))
    }
}

fn condition_matches(cond: &Condition, env: &ParsedEnvelope) -> bool {
    match cond {
        Condition::From(t) => string_matches(t, &env.from),
        Condition::To(t) => string_matches(t, &env.to),
        Condition::Subject(t) => string_matches(t, &env.subject),
        Condition::Body(t) => string_matches(t, &env.body),
        Condition::HasAttachment => env.has_attachment,
        Condition::Size { over, bytes } => {
            if *over {
                env.size > *bytes
            } else {
                env.size < *bytes
            }
        }
        Condition::Keyword(t) => env.keywords.iter().any(|k| string_matches(t, k)),
    }
}

fn string_matches(test: &StringTest, haystack: &str) -> bool {
    match test.op {
        MatchOp::Contains => haystack
            .to_ascii_lowercase()
            .contains(&test.value.to_ascii_lowercase()),
        MatchOp::Is => haystack.eq_ignore_ascii_case(&test.value),
        MatchOp::Matches => glob_matches(&test.value, haystack),
    }
}

/// Sieve `:matches` glob: `*` matches any run (incl. empty), `?` matches exactly
/// one character. Case-insensitive (ascii-casemap). Iterative backtracking so an
/// adversarial pattern cannot blow the stack.
fn glob_matches(pattern: &str, text: &str) -> bool {
    let p: Vec<char> = pattern.to_ascii_lowercase().chars().collect();
    let t: Vec<char> = text.to_ascii_lowercase().chars().collect();
    let (mut pi, mut ti) = (0usize, 0usize);
    let (mut star, mut mark) = (None, 0usize);

    while ti < t.len() {
        if pi < p.len() && (p[pi] == '?' || p[pi] == t[ti]) {
            pi += 1;
            ti += 1;
        } else if pi < p.len() && p[pi] == '*' {
            star = Some(pi);
            mark = ti;
            pi += 1;
        } else if let Some(s) = star {
            pi = s + 1;
            mark += 1;
            ti = mark;
        } else {
            return false;
        }
    }
    while pi < p.len() && p[pi] == '*' {
        pi += 1;
    }
    pi == p.len()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn st(op: MatchOp, v: &str) -> StringTest {
        StringTest {
            op,
            value: v.into(),
        }
    }

    fn rule(conditions: Vec<Condition>, actions: Vec<Action>, match_all: bool) -> Rule {
        Rule {
            id: "r".into(),
            name: "r".into(),
            match_all,
            conditions,
            actions,
            enabled: true,
        }
    }

    fn env() -> ParsedEnvelope {
        ParsedEnvelope {
            from: "Alice <alice@example.com>".into(),
            to: "team@example.com".into(),
            subject: "Weekly Report".into(),
            body: "Please review the attached invoice.".into(),
            has_attachment: true,
            size: 2_000_000,
            keywords: vec!["\\Seen".into(), "work".into()],
        }
    }

    #[test]
    fn contains_is_case_insensitive() {
        let r = rule(
            vec![Condition::Subject(st(MatchOp::Contains, "weekly"))],
            vec![Action::Stop],
            true,
        );
        assert_eq!(evaluate(&r, &env()), vec![Action::Stop]);
    }

    #[test]
    fn is_requires_full_equality() {
        let r = rule(
            vec![Condition::To(st(MatchOp::Is, "team@example.com"))],
            vec![Action::Tag {
                keyword: "t".into(),
            }],
            true,
        );
        assert_eq!(
            evaluate(&r, &env()),
            vec![Action::Tag {
                keyword: "t".into()
            }]
        );

        let no = rule(
            vec![Condition::To(st(MatchOp::Is, "team"))],
            vec![Action::Stop],
            true,
        );
        assert!(evaluate(&no, &env()).is_empty());
    }

    #[test]
    fn matches_glob() {
        assert!(glob_matches("a*c", "abbbc"));
        assert!(glob_matches("a?c", "aXc"));
        assert!(glob_matches("*", ""));
        assert!(glob_matches("*report", "weekly REPORT"));
        assert!(!glob_matches("a?c", "ac"));
        assert!(!glob_matches("a*c", "abbb"));
        // Adversarial: many stars must not hang.
        assert!(glob_matches("*a*a*a*a*b", "aaaaaaaaaaaaaaaab"));
    }

    #[test]
    fn all_vs_any() {
        let both = vec![
            Condition::Subject(st(MatchOp::Contains, "weekly")),
            Condition::From(st(MatchOp::Contains, "nobody")),
        ];
        let all = rule(both.clone(), vec![Action::Stop], true);
        assert!(evaluate(&all, &env()).is_empty(), "allof needs both");
        let any = rule(both, vec![Action::Stop], false);
        assert_eq!(
            evaluate(&any, &env()),
            vec![Action::Stop],
            "anyof needs one"
        );
    }

    #[test]
    fn size_and_attachment_and_keyword() {
        let r = rule(
            vec![
                Condition::Size {
                    over: true,
                    bytes: 1_000_000,
                },
                Condition::HasAttachment,
                Condition::Keyword(st(MatchOp::Is, "work")),
            ],
            vec![Action::Move {
                mailbox: "Big".into(),
            }],
            true,
        );
        assert_eq!(
            evaluate(&r, &env()),
            vec![Action::Move {
                mailbox: "Big".into()
            }]
        );
    }

    #[test]
    fn disabled_rule_yields_nothing() {
        let mut r = rule(vec![], vec![Action::Stop], true);
        r.enabled = false;
        assert!(evaluate(&r, &env()).is_empty());
    }

    #[test]
    fn empty_conditions_always_fire() {
        let r = rule(
            vec![],
            vec![Action::Tag {
                keyword: "x".into(),
            }],
            true,
        );
        assert_eq!(evaluate(&r, &env()).len(), 1);
    }

    #[test]
    fn evaluate_all_honours_stop() {
        let r1 = rule(
            vec![Condition::Subject(st(MatchOp::Contains, "weekly"))],
            vec![
                Action::Tag {
                    keyword: "a".into(),
                },
                Action::Stop,
            ],
            true,
        );
        let r2 = rule(
            vec![],
            vec![Action::Tag {
                keyword: "b".into(),
            }],
            true,
        );
        let actions = evaluate_all(&[r1, r2], &env());
        assert_eq!(
            actions,
            vec![
                Action::Tag {
                    keyword: "a".into()
                },
                Action::Stop
            ]
        );
    }

    #[test]
    fn evaluate_all_accumulates_without_stop() {
        let r1 = rule(
            vec![],
            vec![Action::Tag {
                keyword: "a".into(),
            }],
            true,
        );
        let r2 = rule(
            vec![],
            vec![Action::Tag {
                keyword: "b".into(),
            }],
            true,
        );
        assert_eq!(evaluate_all(&[r1, r2], &env()).len(), 2);
    }
}
