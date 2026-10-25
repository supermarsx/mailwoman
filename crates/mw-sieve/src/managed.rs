//! The rule builder's managed blocks inside a hand-editable script.
//!
//! The builder owns only the parts of a script it generated. Each rule is a block
//! bracketed by a `# mailwoman:rule <id>` comment pair, and the extensions those
//! blocks need sit in one `# mailwoman:require` pair at the top:
//!
//! ```text
//! # mailwoman:require begin
//! require ["fileinto"];
//! # mailwoman:require end
//! ...anything the user wrote...
//! # mailwoman:rule 7 begin
//! # rule: News
//! if address :contains "from" "news@example.com" {
//!     fileinto "News";
//! }
//! # mailwoman:rule 7 end
//! ```
//!
//! [`merge`] rewrites only those blocks; every byte outside them is kept as it
//! was. A disabled rule stays in the script as a commented-out block
//! (`# mailwoman:rule <id> begin disabled`, each line prefixed `# `) so the
//! builder's view of the script is not lossy.
//!
//! Marker lines are recognised at the start of a line (leading spaces and tabs
//! allowed). They are plain comments to every Sieve implementation.

use std::collections::{BTreeSet, HashMap};

use crate::codegen::{generate_rule, require_line};
use crate::{Result, Rule, SieveError};

const RULE_MARK: &str = "# mailwoman:rule ";
const REQUIRE_BEGIN: &str = "# mailwoman:require begin";
const REQUIRE_END: &str = "# mailwoman:require end";

/// One piece of a script split at the managed markers. Concatenating every
/// segment's `text` gives back the script byte for byte.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Segment<'a> {
    /// Text the user owns.
    Text(&'a str),
    /// The managed `require` block, markers included.
    Require(&'a str),
    /// One managed rule block, markers included; `body` is the text between the
    /// markers (still commented out when `disabled`).
    Rule {
        id: String,
        disabled: bool,
        text: &'a str,
        body: &'a str,
    },
}

impl Segment<'_> {
    /// The segment's exact source text.
    pub fn text(&self) -> &str {
        match self {
            Segment::Text(t) | Segment::Require(t) => t,
            Segment::Rule { text, .. } => text,
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
enum Marker {
    RuleBegin { id: String, disabled: bool },
    RuleEnd { id: String },
    RequireBegin,
    RequireEnd,
}

fn marker(line: &str) -> Option<Marker> {
    let line = line
        .trim_end_matches(['\r', '\n'])
        .trim_start_matches([' ', '\t']);
    if line.trim_end() == REQUIRE_BEGIN {
        return Some(Marker::RequireBegin);
    }
    if line.trim_end() == REQUIRE_END {
        return Some(Marker::RequireEnd);
    }
    let rest = line.strip_prefix(RULE_MARK)?;
    let mut words = rest.split_whitespace();
    let id = words.next()?.to_string();
    let m = match (words.next(), words.next(), words.next()) {
        (Some("begin"), None, None) => Marker::RuleBegin {
            id,
            disabled: false,
        },
        (Some("begin"), Some("disabled"), None) => Marker::RuleBegin { id, disabled: true },
        (Some("end"), None, None) => Marker::RuleEnd { id },
        _ => return None,
    };
    Some(m)
}

/// Lines of `source`, each with its line ending, plus its byte offset.
fn lines(source: &str) -> impl Iterator<Item = (usize, &str)> {
    let mut offset = 0usize;
    source.split_inclusive('\n').map(move |l| {
        let start = offset;
        offset += l.len();
        (start, l)
    })
}

/// Split a script at its managed markers. Fails on an unterminated or stray
/// marker, a nested block, or a rule id used twice.
pub fn split(source: &str) -> Result<Vec<Segment<'_>>> {
    let mut out = Vec::new();
    let mut text_start = 0usize;
    // (marker, block start offset, body start offset, line number)
    let mut open: Option<(Marker, usize, usize, usize)> = None;
    let mut seen = BTreeSet::new();
    let mut have_require = false;

    for (lineno, (start, line)) in lines(source).enumerate() {
        let lineno = lineno + 1;
        let Some(m) = marker(line) else { continue };
        let end = start + line.len();
        match (open.take(), m) {
            (None, m @ (Marker::RuleBegin { .. } | Marker::RequireBegin)) => {
                if let Marker::RuleBegin { id, .. } = &m
                    && !seen.insert(id.clone())
                {
                    return Err(marker_error(lineno, &format!("rule `{id}` appears twice")));
                }
                if m == Marker::RequireBegin {
                    if have_require {
                        return Err(marker_error(lineno, "a second managed require block"));
                    }
                    have_require = true;
                }
                if text_start < start {
                    out.push(Segment::Text(&source[text_start..start]));
                }
                open = Some((m, start, end, lineno));
            }
            (None, _) => return Err(marker_error(lineno, "end marker without a begin")),
            (
                Some((Marker::RuleBegin { id, disabled }, bstart, body_start, _)),
                Marker::RuleEnd { id: end_id },
            ) if end_id == id => {
                out.push(Segment::Rule {
                    id,
                    disabled,
                    text: &source[bstart..end],
                    body: &source[body_start..start],
                });
                text_start = end;
            }
            (Some((Marker::RequireBegin, bstart, _, _)), Marker::RequireEnd) => {
                out.push(Segment::Require(&source[bstart..end]));
                text_start = end;
            }
            (Some((_, _, _, open_line)), _) => {
                return Err(marker_error(
                    lineno,
                    &format!("marker does not close the block opened on line {open_line}"),
                ));
            }
        }
    }
    if let Some((_, _, _, open_line)) = open {
        return Err(marker_error(open_line, "managed block is never closed"));
    }
    if text_start < source.len() {
        out.push(Segment::Text(&source[text_start..]));
    }
    Ok(out)
}

fn marker_error(line: usize, msg: &str) -> SieveError {
    SieveError::Parse(format!("line {line}: {msg}"))
}

/// The builder's view of a script: its managed rules in script order.
pub fn rules(source: &str) -> Result<Vec<Rule>> {
    let mut out = Vec::new();
    for seg in split(source)? {
        if let Segment::Rule {
            id, disabled, body, ..
        } = seg
        {
            out.push(read_rule(&id, disabled, body)?);
        }
    }
    Ok(out)
}

fn read_rule(id: &str, disabled: bool, body: &str) -> Result<Rule> {
    let text = if disabled {
        body.split_inclusive('\n')
            .map(|l| {
                let t = l.trim_start_matches([' ', '\t']);
                t.strip_prefix("# ")
                    .or_else(|| t.strip_prefix('#'))
                    .unwrap_or(t)
            })
            .collect::<String>()
    } else {
        body.to_string()
    };
    let mut parsed = crate::parse(&text).map_err(|e| {
        SieveError::Parse(format!("rule `{id}` was changed outside the builder: {e}"))
    })?;
    if parsed.len() != 1 {
        return Err(SieveError::Parse(format!(
            "rule `{id}` was changed outside the builder"
        )));
    }
    let mut rule = parsed.remove(0);
    rule.id = id.to_string();
    rule.enabled = !disabled;
    Ok(rule)
}

/// Only the text the user owns, in order (every managed block removed). Two
/// versions of a script with the same hand-written text differ only in what
/// the builder generated, which is how a server tells a builder save from a raw
/// edit.
pub fn hand_written(source: &str) -> Result<String> {
    Ok(split(source)?
        .iter()
        .filter_map(|s| match s {
            Segment::Text(t) => Some(*t),
            _ => None,
        })
        .collect())
}

fn valid_id(id: &str) -> bool {
    !id.is_empty()
        && id.len() <= 128
        && id
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || matches!(b, b'-' | b'_' | b'.'))
}

/// Rewrite the managed blocks of `source` to hold exactly `rules`.
///
/// - A rule whose id already has a block is rewritten in place. When the builder
///   reorders rules that already exist, they trade places among their existing
///   blocks, so hand-written text between them never moves.
/// - A block whose id is not in `rules` is removed (its lines, nothing else).
/// - New rules are appended at the end.
/// - The managed `require` block is rewritten for the enabled rules, inserted at
///   the very top when missing, and removed when no extension is needed.
///
/// Every byte outside the managed blocks is kept. Generated lines follow the
/// script's line ending (CRLF when the script uses it).
pub fn merge(source: &str, rules: &[Rule]) -> Result<String> {
    let mut ids = BTreeSet::new();
    for r in rules {
        if !valid_id(&r.id) {
            return Err(SieveError::Generate(format!(
                "rule id {:?} must be 1-128 letters, digits, `-`, `_` or `.`",
                r.id
            )));
        }
        if !ids.insert(r.id.as_str()) {
            return Err(SieveError::Generate(format!(
                "rule id `{}` is used twice",
                r.id
            )));
        }
    }
    let segments = split(source)?;
    let crlf = source.contains("\r\n");
    let eol = |s: String| if crlf { s.replace('\n', "\r\n") } else { s };

    let mut extensions: BTreeSet<&'static str> = BTreeSet::new();
    let mut blocks: HashMap<&str, String> = HashMap::new();
    for r in rules {
        let (body, ext) = generate_rule(r);
        if r.enabled {
            extensions.extend(ext);
        }
        blocks.insert(r.id.as_str(), eol(rule_block(r, &body)));
    }
    let require = (!extensions.is_empty()).then(|| {
        eol(format!(
            "{REQUIRE_BEGIN}\n{}{REQUIRE_END}\n",
            require_line(&extensions)
        ))
    });

    // Existing slots of rules that survive, filled in the builder's order.
    let surviving: Vec<&str> = segments
        .iter()
        .filter_map(|s| match s {
            Segment::Rule { id, .. } if ids.contains(id.as_str()) => Some(id.as_str()),
            _ => None,
        })
        .collect();
    let surviving_set: BTreeSet<&str> = surviving.iter().copied().collect();
    let mut slot_order = rules
        .iter()
        .map(|r| r.id.as_str())
        .filter(|id| surviving_set.contains(id));

    let mut out = String::with_capacity(source.len() + 256);
    let has_require = segments.iter().any(|s| matches!(s, Segment::Require(_)));
    if !has_require && let Some(req) = &require {
        out.push_str(req);
    }
    for seg in &segments {
        match seg {
            Segment::Text(t) => out.push_str(t),
            Segment::Require(_) => {
                if let Some(req) = &require {
                    out.push_str(req);
                }
            }
            Segment::Rule { id, .. } => {
                if ids.contains(id.as_str())
                    && let Some(next) = slot_order.next()
                {
                    out.push_str(&blocks[next]);
                }
            }
        }
    }
    for r in rules {
        if !surviving_set.contains(r.id.as_str()) {
            if !out.is_empty() && !out.ends_with('\n') {
                out.push_str(if crlf { "\r\n" } else { "\n" });
            }
            out.push_str(&blocks[r.id.as_str()]);
        }
    }
    Ok(out)
}

fn rule_block(rule: &Rule, body: &str) -> String {
    let mut out = String::new();
    if rule.enabled {
        out.push_str(&format!("{RULE_MARK}{} begin\n", rule.id));
        out.push_str(body);
    } else {
        out.push_str(&format!("{RULE_MARK}{} begin disabled\n", rule.id));
        for line in body.lines() {
            out.push_str("# ");
            out.push_str(line);
            out.push('\n');
        }
    }
    out.push_str(&format!("{RULE_MARK}{} end\n", rule.id));
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Action, Condition, MatchOp, StringTest};

    fn rule(id: &str, name: &str, from: &str, mailbox: &str) -> Rule {
        Rule {
            id: id.into(),
            name: name.into(),
            match_all: true,
            conditions: vec![Condition::From(StringTest {
                op: MatchOp::Contains,
                value: from.into(),
            })],
            actions: vec![Action::Move {
                mailbox: mailbox.into(),
            }],
            enabled: true,
        }
    }

    const HAND: &str = "# my own header\r\nrequire \"vacation\";\r\n\r\n";
    const HAND_TAIL: &str =
        "\r\n# keep the rest\r\nif header :is \"x-a\" \"1\" {\r\n  vacation \"away\";\r\n}\r\n";

    #[test]
    fn builder_round_trip_keeps_hand_written_text_byte_identical() {
        let source = format!("{HAND}{HAND_TAIL}");
        let r1 = rule("r1", "News", "news@example.com", "News");
        let r2 = rule("r2", "Bills", "bills@example.com", "Bills");

        let merged = merge(&source, &[r1.clone(), r2.clone()]).unwrap();
        assert_eq!(
            hand_written(&merged).unwrap(),
            source,
            "hand-written text changed"
        );
        assert_eq!(rules(&merged).unwrap(), vec![r1.clone(), r2.clone()]);
        crate::extensions::check(&merged).expect("merged script must validate");
        assert!(
            merged.contains("\r\n# mailwoman:rule r1 begin\r\n"),
            "CRLF followed: {merged:?}"
        );

        // Edit r2, drop r1, add r3: only managed blocks change.
        let mut r2b = r2.clone();
        r2b.actions = vec![Action::Copy {
            mailbox: "Paid".into(),
        }];
        let r3 = rule("r3", "Lists", "list@example.org", "Lists");
        let merged2 = merge(&merged, &[r2b.clone(), r3.clone()]).unwrap();
        assert_eq!(hand_written(&merged2).unwrap(), source);
        assert_eq!(rules(&merged2).unwrap(), vec![r2b.clone(), r3.clone()]);
        assert!(
            merged2.contains("\"copy\""),
            "require block follows the rules"
        );
        crate::extensions::check(&merged2).unwrap();

        // Removing every rule gives back the original script exactly.
        assert_eq!(merge(&merged2, &[]).unwrap(), source);
    }

    #[test]
    fn rules_between_hand_written_text_stay_in_their_slots() {
        let r1 = rule("a", "A", "a@x", "A");
        let r2 = rule("b", "B", "b@x", "B");
        let source = format!(
            "{}keep;\n{}stop;\n",
            rule_block(&r1, &generate_rule(&r1).0),
            rule_block(&r2, &generate_rule(&r2).0)
        );
        // Delete A: B must not move above `keep;`.
        let out = merge(&source, std::slice::from_ref(&r2)).unwrap();
        let keep = out.find("keep;").unwrap();
        let b = out.find("# mailwoman:rule b begin").unwrap();
        assert!(keep < b, "{out}");
        // Swap order: they trade blocks, `keep;` stays between them.
        let out = merge(&source, &[r2.clone(), r1.clone()]).unwrap();
        let first = out.find("# mailwoman:rule b begin").unwrap();
        let keep = out.find("keep;").unwrap();
        let second = out.find("# mailwoman:rule a begin").unwrap();
        assert!(first < keep && keep < second, "{out}");
        assert_eq!(rules(&out).unwrap(), vec![r2, r1]);
    }

    #[test]
    fn disabled_rules_survive_as_comments() {
        let mut r = rule("d1", "Off", "x@example.com", "X");
        r.enabled = false;
        let out = merge("keep;\n", std::slice::from_ref(&r)).unwrap();
        assert!(
            out.contains("# mailwoman:rule d1 begin disabled\n# # rule: Off\n# if address"),
            "{out}"
        );
        assert!(
            !out.contains("require"),
            "a disabled rule needs no extension: {out}"
        );
        assert_eq!(rules(&out).unwrap(), vec![r]);
        crate::extensions::check(&out).unwrap();
    }

    #[test]
    fn markers_are_checked() {
        assert!(split("# mailwoman:rule a begin\nkeep;\n").is_err());
        assert!(split("# mailwoman:rule a end\n").is_err());
        assert!(split("# mailwoman:rule a begin\n# mailwoman:rule b end\n").is_err());
        assert!(
            split("# mailwoman:rule a begin\n# mailwoman:rule a end\n# mailwoman:rule a begin\n# mailwoman:rule a end\n")
                .is_err()
        );
        // Text that merely resembles a marker is the user's.
        let s = split("# mailwoman:rule a beginning\nkeep;\n").unwrap();
        assert_eq!(s.len(), 1);
    }

    #[test]
    fn hand_edits_inside_a_block_are_reported() {
        let r = rule("a", "A", "a@x", "A");
        let merged = merge("", std::slice::from_ref(&r)).unwrap();
        let tampered = merged.replace("fileinto \"A\";", "fileinto \"A\"; discard;");
        let err = rules(&tampered).unwrap_err();
        assert!(err.to_string().contains("rule `a`"), "{err}");
    }

    #[test]
    fn ids_are_validated() {
        let mut r = rule("bad id", "A", "a@x", "A");
        assert!(merge("", std::slice::from_ref(&r)).is_err());
        r.id = "x\nkeep".into();
        assert!(merge("", std::slice::from_ref(&r)).is_err());
        let a = rule("same", "A", "a@x", "A");
        assert!(merge("", &[a.clone(), a]).is_err());
    }
}
