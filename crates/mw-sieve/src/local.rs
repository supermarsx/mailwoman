//! Fail-closed local Sieve execution for mail already delivered to INBOX.
//! Produces actions only after validating the entire script, including branches
//! that do not match. The engine applies actions after this pure evaluation.
use crate::grammar::{Argument, ArgumentKind, Command, ScriptError, Span, Test};
#[path = "local_date.rs"]
mod date;

pub const EXTENSIONS: &[&str] = &[
    "fileinto",
    "imap4flags",
    "envelope",
    "copy",
    "subaddress",
    "vacation",
    "vacation-seconds",
    "date",
    "index",
    "relational",
    "comparator-i;ascii-numeric",
];

#[derive(Default, Clone)]
pub struct Message {
    pub current_time: Option<i64>,
    pub headers: Vec<(String, String)>,
    pub body: String,
    pub flags: Vec<String>,
    pub size: u64,
    pub envelope_from: Option<String>,
    pub envelope_to: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Action {
    Vacation(Vacation),
    Keep,
    Discard,
    FileInto {
        mailbox: String,
        copy: bool,
        flags: Option<Vec<String>>,
    },
    Flags {
        operation: FlagOperation,
        flags: Vec<String>,
    },
    Redirect {
        address: String,
        copy: bool,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Vacation {
    pub seconds: u64,
    pub subject: Option<String>,
    pub from: Option<String>,
    pub addresses: Vec<String>,
    pub mime: bool,
    pub handle: Option<String>,
    pub reason: String,
}

fn vacation(command: &Command) -> Result<Vacation, ScriptError> {
    let invalid = || ScriptError::at(command.span, "invalid vacation arguments");
    let mut result = Vacation {
        seconds: 7 * 86400,
        subject: None,
        from: None,
        addresses: vec![],
        mime: false,
        handle: None,
        reason: String::new(),
    };
    let mut seen = std::collections::HashSet::new();
    let mut args = command.arguments.iter();
    while let Some(arg) = args.next() {
        let Some(tag) = arg.tag() else {
            let values = arg.strings().ok_or_else(invalid)?;
            if values.len() != 1 || args.next().is_some() {
                return Err(invalid());
            }
            result.reason = values[0].value.clone();
            return Ok(result);
        };
        let tag = tag.to_ascii_lowercase();
        if !seen.insert(tag.clone()) {
            return Err(invalid());
        }
        match tag.as_str() {
            "mime" => result.mime = true,
            "days" | "seconds" => {
                let ArgumentKind::Number(value) = args.next().ok_or_else(invalid)?.kind else {
                    return Err(invalid());
                };
                if seen.contains("days") && seen.contains("seconds")
                    || tag == "seconds" && value >= (1 << 31)
                {
                    return Err(invalid());
                }
                result.seconds = if tag == "days" {
                    value.clamp(1, 31) * 86400
                } else {
                    value
                };
            }
            "subject" | "from" | "addresses" | "handle" => {
                let values = args
                    .next()
                    .and_then(Argument::strings)
                    .ok_or_else(invalid)?;
                if values.len() != 1 && tag != "addresses" {
                    return Err(invalid());
                }
                let values: Vec<String> = values.iter().map(|v| v.value.clone()).collect();
                match tag.as_str() {
                    "subject" => {
                        if values[0].chars().any(char::is_control) {
                            return Err(invalid());
                        }
                        result.subject = Some(values[0].clone());
                    }
                    "from" => result.from = Some(values[0].clone()),
                    "addresses" => result.addresses = values,
                    _ => result.handle = Some(values[0].clone()),
                }
            }
            _ => return Err(invalid()),
        }
    }
    Err(invalid())
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FlagOperation {
    Set,
    Add,
    Remove,
}

pub fn validate(source: &str) -> Result<crate::Script, ScriptError> {
    if source.len() > 1_048_576 {
        return Err(ScriptError::at(
            Span {
                line: 1,
                column: 1,
                ..Span::default()
            },
            "local script exceeds 1 MiB",
        ));
    }
    let (script, analysis) = crate::extensions::check(source)?;
    for (extension, span) in &analysis.require_spans {
        if !EXTENSIONS.contains(&extension.as_str()) {
            return Err(ScriptError::at(
                *span,
                format!("extension {extension} is unavailable during local execution"),
            ));
        }
    }
    validate_commands(&script.commands)?;
    Ok(script)
}

pub fn evaluate(source: &str, message: &Message) -> Result<Vec<Action>, ScriptError> {
    let script = validate(source)?;
    let mut message = message.clone();
    message
        .current_time
        .get_or_insert_with(|| chrono::Utc::now().timestamp());
    let mut actions = Vec::new();
    run(
        &script.commands,
        &message,
        &mut actions,
        &mut message.flags.clone(),
    )?;
    if actions
        .iter()
        .filter(|a| matches!(a, Action::Vacation(_)))
        .count()
        > 1
    {
        return Err(ScriptError::at(
            Span::default(),
            "multiple vacation actions executed",
        ));
    }
    Ok(actions)
}

#[test]
fn vacation_parameters_and_multiple_execution_fail_closed() {
    let actions = evaluate(r#"require "vacation-seconds"; vacation :seconds 0 :subject "Away" :handle "holiday" :addresses ["alias@example.org"] "Back soon";"#, &Message::default()).unwrap();
    let Action::Vacation(v) = &actions[0] else {
        panic!("expected vacation")
    };
    assert_eq!(v.seconds, 0);
    assert_eq!(v.handle.as_deref(), Some("holiday"));
    assert_eq!(v.addresses, vec!["alias@example.org"]);
    for source in [
        r#"require ["vacation", "vacation-seconds"]; vacation :days 1 :seconds 1 "x";"#,
        r#"require "vacation"; vacation :subject "one" :subject "two" "x";"#,
        r#"require "vacation"; vacation "x"; vacation "y";"#,
        r#"require "vacation-seconds"; vacation :seconds 2147483648 "x";"#,
    ] {
        assert!(evaluate(source, &Message::default()).is_err(), "{source}");
    }
}

fn strings(arguments: &[Argument]) -> Vec<Vec<String>> {
    let mut result = Vec::new();
    let mut skip = false;
    for arg in arguments {
        if skip {
            skip = false;
            continue;
        }
        if arg.tag().is_some_and(|t| {
            ["comparator", "flags", "value", "count", "zone"]
                .contains(&t.to_ascii_lowercase().as_str())
        }) {
            skip = true;
        } else if let Some(values) = arg.strings() {
            result.push(values.iter().map(|v| v.value.clone()).collect());
        }
    }
    result
}
fn tagged_values(arguments: &[Argument], tag: &str) -> Option<Vec<String>> {
    let i = arguments
        .iter()
        .position(|a| a.tag().is_some_and(|t| t.eq_ignore_ascii_case(tag)))?;
    Some(
        arguments
            .get(i + 1)?
            .strings()?
            .iter()
            .map(|v| v.value.clone())
            .collect(),
    )
}
fn flag_values(values: &[String]) -> Vec<String> {
    values
        .iter()
        .flat_map(|v| v.split_whitespace().map(str::to_owned))
        .collect()
}
fn tagged(arguments: &[Argument], tag: &str) -> bool {
    arguments
        .iter()
        .filter_map(Argument::tag)
        .any(|v| v.eq_ignore_ascii_case(tag))
}
fn tags(arguments: &[Argument], allowed: &[&str], span: Span) -> Result<(), ScriptError> {
    for tag in arguments.iter().filter_map(Argument::tag) {
        if !allowed.iter().any(|v| tag.eq_ignore_ascii_case(v)) {
            return Err(ScriptError::at(
                span,
                format!("tag :{tag} is unavailable during local execution"),
            ));
        }
    }
    Ok(())
}
fn validate_commands(commands: &[Command]) -> Result<(), ScriptError> {
    for command in commands {
        let values = strings(&command.arguments);
        let name = command.name.to_ascii_lowercase();
        if tagged(&command.arguments, "flags") {
            let flags = tagged_values(&command.arguments, "flags")
                .ok_or_else(|| ScriptError::at(command.span, "missing flag list"))?;
            for flag in flag_values(&flags) {
                let system = ["\\seen", "\\answered", "\\flagged", "\\deleted", "\\draft"]
                    .contains(&flag.to_ascii_lowercase().as_str());
                if !system
                    && flag
                        .chars()
                        .any(|c| c.is_control() || "(){%*\"\\]".contains(c))
                {
                    return Err(ScriptError::at(
                        command.span,
                        "invalid or read-only IMAP flag",
                    ));
                }
            }
        }
        match name.as_str() {
            "vacation" => {
                vacation(command)?;
            }
            "require" | "if" | "elsif" | "else" => tags(&command.arguments, &[], command.span)?,
            "keep" => {
                tags(&command.arguments, &["flags"], command.span)?;
                if !values.is_empty()
                    || (tagged(&command.arguments, "flags")
                        && tagged_values(&command.arguments, "flags").is_none())
                {
                    return Err(ScriptError::at(command.span, "invalid keep flags"));
                }
            }
            "discard" | "stop" => {
                if !command.arguments.is_empty() {
                    return Err(ScriptError::at(command.span, "unexpected action arguments"));
                }
            }
            "fileinto" | "redirect" => {
                tags(
                    &command.arguments,
                    if name == "fileinto" {
                        &["copy", "flags"]
                    } else {
                        &["copy"]
                    },
                    command.span,
                )?;
                if values.len() != 1
                    || values[0].len() != 1
                    || values[0][0].is_empty()
                    || command
                        .arguments
                        .iter()
                        .any(|a| matches!(a.kind, ArgumentKind::Number(_)))
                {
                    return Err(ScriptError::at(
                        command.span,
                        "action requires one nonempty string",
                    ));
                }
            }
            "setflag" | "addflag" | "removeflag" => {
                tags(&command.arguments, &[], command.span)?;
                if values.len() != 1 {
                    return Err(ScriptError::at(
                        command.span,
                        "flag action requires one string list",
                    ));
                }
                for flag in values[0].iter().flat_map(|value| value.split_whitespace()) {
                    let system = ["\\seen", "\\answered", "\\flagged", "\\deleted", "\\draft"]
                        .contains(&flag.to_ascii_lowercase().as_str());
                    if !system
                        && flag
                            .chars()
                            .any(|c| c.is_control() || "(){%*\"\\]".contains(c))
                    {
                        return Err(ScriptError::at(
                            command.span,
                            "invalid or read-only IMAP flag",
                        ));
                    }
                }
            }
            _ => {
                return Err(ScriptError::at(
                    command.span,
                    format!("command {name} is unavailable during local execution"),
                ));
            }
        }
        for test in &command.tests {
            validate_test(test)?;
        }
        if let Some(block) = &command.block {
            validate_commands(&block.commands)?;
        }
    }
    Ok(())
}
fn validate_test(test: &Test) -> Result<(), ScriptError> {
    let values = strings(&test.arguments);
    if tagged(&test.arguments, "comparator")
        && tagged_values(&test.arguments, "comparator").is_none_or(|v| {
            v.len() != 1
                || !["i;octet", "i;ascii-casemap", "i;ascii-numeric"].contains(&v[0].as_str())
        })
    {
        return Err(ScriptError::at(test.span, "unsupported comparator"));
    }
    for tag in ["value", "count"] {
        if tagged(&test.arguments, tag)
            && tagged_values(&test.arguments, tag).is_none_or(|v| {
                v.len() != 1 || !["gt", "ge", "lt", "le", "eq", "ne"].contains(&v[0].as_str())
            })
        {
            return Err(ScriptError::at(test.span, "invalid relational operator"));
        }
    }
    if ["is", "contains", "matches", "value", "count"]
        .iter()
        .filter(|tag| tagged(&test.arguments, tag))
        .count()
        > 1
        || tagged(&test.arguments, "last") && !tagged(&test.arguments, "index")
        || tagged(&test.arguments, "index") && index(&test.arguments).is_none()
    {
        return Err(ScriptError::at(
            test.span,
            "conflicting match types or invalid index",
        ));
    }
    let valid = match test.name.to_ascii_lowercase().as_str() {
        "date" | "currentdate" => date::validate(test)?,
        "allof" | "anyof" | "not" | "true" | "false" => test.arguments.is_empty(),
        "exists" => values.len() == 1 && test.arguments.len() == 1,
        "header" | "address" | "envelope" => {
            tags(
                &test.arguments,
                &[
                    "is",
                    "contains",
                    "matches",
                    "all",
                    "localpart",
                    "domain",
                    "user",
                    "detail",
                    "comparator",
                    "value",
                    "count",
                    "index",
                    "last",
                ],
                test.span,
            )?;
            values.len() == 2
                && (!test.is("envelope")
                    || (!tagged(&test.arguments, "index") && !tagged(&test.arguments, "last")))
        }
        "hasflag" => {
            tags(
                &test.arguments,
                &["is", "contains", "matches", "comparator", "value", "count"],
                test.span,
            )?;
            values.len() == 1
        }
        "body" => {
            tags(
                &test.arguments,
                &["is", "contains", "matches", "text"],
                test.span,
            )?;
            values.len() == 1
        }
        "size" => {
            tags(&test.arguments, &["over", "under"], test.span)?;
            test.arguments.len() == 2
                && (tagged(&test.arguments, "over") != tagged(&test.arguments, "under"))
                && test
                    .arguments
                    .iter()
                    .any(|a| matches!(a.kind, ArgumentKind::Number(_)))
        }
        _ => false,
    };
    if !valid {
        return Err(ScriptError::at(
            test.span,
            format!("unsupported or invalid local test {}", test.name),
        ));
    }
    for nested in &test.tests {
        validate_test(nested)?;
    }
    Ok(())
}

fn run(
    commands: &[Command],
    message: &Message,
    actions: &mut Vec<Action>,
    current_flags: &mut Vec<String>,
) -> Result<bool, ScriptError> {
    let mut matched_branch = false;
    for command in commands {
        let name = command.name.to_ascii_lowercase();
        if matches!(name.as_str(), "if" | "elsif" | "else") {
            if name == "if" {
                matched_branch = false;
            }
            let matches = !matched_branch
                && (name == "else" || test_matches(&command.tests[0], message, current_flags));
            if matches {
                matched_branch = true;
                if let Some(block) = &command.block
                    && run(&block.commands, message, actions, current_flags)?
                {
                    return Ok(true);
                }
            }
            continue;
        }
        matched_branch = false;
        let values = strings(&command.arguments);
        match name.as_str() {
            "vacation" => actions.push(Action::Vacation(vacation(command)?)),
            "require" => {}
            "stop" => return Ok(true),
            "keep" => {
                if let Some(flags) = tagged_values(&command.arguments, "flags") {
                    actions.push(Action::Flags {
                        operation: FlagOperation::Set,
                        flags: flag_values(&flags),
                    });
                }
                actions.push(Action::Keep);
            }
            "discard" => actions.push(Action::Discard),
            "fileinto" => actions.push(Action::FileInto {
                mailbox: values[0][0].clone(),
                flags: tagged_values(&command.arguments, "flags").map(|v| flag_values(&v)),
                copy: tagged(&command.arguments, "copy"),
            }),
            "redirect" => actions.push(Action::Redirect {
                address: values[0][0].clone(),
                copy: tagged(&command.arguments, "copy"),
            }),
            "setflag" | "addflag" | "removeflag" => {
                let flags = flag_values(&values[0]);
                if name == "setflag" {
                    *current_flags = flags.clone();
                } else if name == "removeflag" {
                    current_flags.retain(|f| !flags.iter().any(|v| v.eq_ignore_ascii_case(f)));
                } else {
                    for flag in &flags {
                        if !current_flags.iter().any(|f| f.eq_ignore_ascii_case(flag)) {
                            current_flags.push(flag.clone());
                        }
                    }
                }
                actions.push(Action::Flags {
                    operation: match name.as_str() {
                        "setflag" => FlagOperation::Set,
                        "removeflag" => FlagOperation::Remove,
                        _ => FlagOperation::Add,
                    },
                    flags: values[0]
                        .iter()
                        .flat_map(|value| value.split_whitespace().map(str::to_string))
                        .collect(),
                });
            }
            _ => return Err(ScriptError::at(command.span, "unsupported local action")),
        }
        if actions.len() > 1024 {
            return Err(ScriptError::at(command.span, "local action limit exceeded"));
        }
    }
    Ok(false)
}

fn test_matches(test: &Test, message: &Message, current_flags: &[String]) -> bool {
    let values = strings(&test.arguments);
    match test.name.to_ascii_lowercase().as_str() {
        "date" | "currentdate" => date::matches(test, message),
        "true" => true,
        "false" => false,
        "not" => !test_matches(&test.tests[0], message, current_flags),
        "allof" => test
            .tests
            .iter()
            .all(|t| test_matches(t, message, current_flags)),
        "anyof" => test
            .tests
            .iter()
            .any(|t| test_matches(t, message, current_flags)),
        "hasflag" => match_values(
            &test.arguments,
            &values[0],
            &current_flags.iter().map(String::as_str).collect::<Vec<_>>(),
        ),
        "exists" => values[0].iter().all(|name| {
            message
                .headers
                .iter()
                .any(|(key, _)| key.eq_ignore_ascii_case(name))
        }),
        "size" => {
            let limit = test
                .arguments
                .iter()
                .find_map(|a| {
                    if let ArgumentKind::Number(n) = a.kind {
                        Some(n)
                    } else {
                        None
                    }
                })
                .unwrap_or(0);
            if tagged(&test.arguments, "over") {
                message.size > limit
            } else {
                message.size < limit
            }
        }
        "body" => values[0]
            .iter()
            .any(|key| matches_text(&test.arguments, key, &message.body)),
        "header" => match_values(
            &test.arguments,
            &values[1],
            &values[0]
                .iter()
                .flat_map(|name| header_values(message, name, &test.arguments))
                .collect::<Vec<_>>(),
        ),
        "address" | "envelope" => {
            let mut addresses = Vec::new();
            for name in &values[0] {
                if test.is("envelope") {
                    if name.eq_ignore_ascii_case("from") {
                        addresses.extend(message.envelope_from.iter().map(String::as_str));
                    }
                    if name.eq_ignore_ascii_case("to") {
                        addresses.extend(message.envelope_to.iter().map(String::as_str));
                    }
                } else {
                    addresses.extend(header_values(message, name, &test.arguments));
                }
            }
            let selected: Vec<_> = addresses
                .iter()
                .flat_map(|v| v.split(','))
                .map(|address| {
                    let address = address.trim();
                    let address = address
                        .rsplit_once('<')
                        .and_then(|(_, v)| v.strip_suffix('>'))
                        .unwrap_or(address);
                    let (local, domain) = address.rsplit_once('@').unwrap_or((address, ""));
                    if tagged(&test.arguments, "localpart") {
                        local
                    } else if tagged(&test.arguments, "domain") {
                        domain
                    } else if tagged(&test.arguments, "user") {
                        local.split('+').next().unwrap_or(local)
                    } else if tagged(&test.arguments, "detail") {
                        local.split_once('+').map_or("", |(_, detail)| detail)
                    } else {
                        address
                    }
                })
                .collect();
            match_values(&test.arguments, &values[1], &selected)
        }
        _ => false,
    }
}
fn matches_text(arguments: &[Argument], key: &str, text: &str) -> bool {
    let numeric = tagged_values(arguments, "comparator").is_some_and(|v| v == ["i;ascii-numeric"]);
    let octet = tagged_values(arguments, "comparator").is_some_and(|v| v == ["i;octet"]);
    let key = if octet {
        key.to_string()
    } else {
        key.to_ascii_lowercase()
    };
    let text = if octet {
        text.to_string()
    } else {
        text.to_ascii_lowercase()
    };
    if tagged(arguments, "value") || tagged(arguments, "count") {
        let order = if numeric {
            numeric_order(&text, &key)
        } else {
            text.cmp(&key)
        };
        let relation = tagged_values(arguments, "value")
            .or_else(|| tagged_values(arguments, "count"))
            .unwrap();
        match relation[0].as_str() {
            "gt" => order.is_gt(),
            "ge" => !order.is_lt(),
            "lt" => order.is_lt(),
            "le" => !order.is_gt(),
            "ne" => !order.is_eq(),
            _ => order.is_eq(),
        }
    } else if tagged(arguments, "contains") {
        text.contains(&key)
    } else if tagged(arguments, "matches") {
        crate::eval::glob_matches_exact(&key, &text)
    } else {
        if numeric {
            numeric_order(&text, &key).is_eq()
        } else {
            text == key
        }
    }
}

fn numeric_order(left: &str, right: &str) -> std::cmp::Ordering {
    fn number(s: &str) -> Option<&str> {
        let end = s.bytes().take_while(u8::is_ascii_digit).count();
        (end > 0).then(|| s[..end].trim_start_matches('0'))
    }
    match (number(left), number(right)) {
        (None, None) => std::cmp::Ordering::Equal,
        (None, Some(_)) => std::cmp::Ordering::Greater,
        (Some(_), None) => std::cmp::Ordering::Less,
        (Some(l), Some(r)) => l.len().cmp(&r.len()).then_with(|| l.cmp(r)),
    }
}
fn match_values(arguments: &[Argument], keys: &[String], values: &[&str]) -> bool {
    if tagged(arguments, "count") {
        let count = values.len().to_string();
        keys.iter().any(|key| matches_text(arguments, key, &count))
    } else {
        values
            .iter()
            .any(|value| keys.iter().any(|key| matches_text(arguments, key, value)))
    }
}
fn index(arguments: &[Argument]) -> Option<usize> {
    let at = arguments
        .iter()
        .position(|a| a.tag().is_some_and(|t| t.eq_ignore_ascii_case("index")))?;
    match arguments.get(at + 1)?.kind {
        ArgumentKind::Number(n) if n > 0 => usize::try_from(n).ok(),
        _ => None,
    }
}
fn header_values<'a>(message: &'a Message, name: &str, arguments: &[Argument]) -> Vec<&'a str> {
    let mut values: Vec<_> = message
        .headers
        .iter()
        .filter(|(header, _)| header.eq_ignore_ascii_case(name))
        .map(|(_, value)| value.as_str())
        .collect();
    if let Some(index) = index(arguments) {
        if tagged(arguments, "last") {
            values.reverse();
        }
        values.into_iter().nth(index - 1).into_iter().collect()
    } else {
        values
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn flags_tests_follow_mutations_and_comparators() {
        let message = Message {
            flags: vec!["Old".into()],
            ..Message::default()
        };
        let source = r#"require "imap4flags"; addflag "New"; if hasflag :comparator "i;octet" "new" { discard; } removeflag "Old"; if allof (hasflag "new", not hasflag "old") { keep :flags "\\Seen"; }"#;
        let actions = evaluate(source, &message).unwrap();
        assert!(!actions.contains(&Action::Discard));
        assert_eq!(actions.last(), Some(&Action::Keep));
        assert!(actions.contains(&Action::Flags {
            operation: FlagOperation::Set,
            flags: vec!["\\Seen".into()]
        }));
    }
    #[test]
    fn explicit_fileinto_flags_do_not_change_default_flags() {
        let source = r#"require ["imap4flags", "fileinto", "copy"]; fileinto :copy :flags "\\Seen" "Archive"; if hasflag "\\Seen" { discard; }"#;
        assert_eq!(
            evaluate(source, &Message::default()).unwrap(),
            vec![Action::FileInto {
                mailbox: "Archive".into(),
                copy: true,
                flags: Some(vec!["\\Seen".into()])
            }]
        );
    }
    #[test]
    fn incomplete_body_extension_is_not_advertised() {
        assert!(!EXTENSIONS.contains(&"body"));
        assert!(validate(r#"require "body"; if body :raw :contains "x" { keep; }"#).is_err());
    }
    #[test]
    fn nested_branches_flags_and_stop() {
        let message = Message {
            headers: vec![("Subject".into(), "Invoice 42".into())],
            size: 20,
            ..Message::default()
        };
        let source = r#"require ["fileinto", "imap4flags"]; if header :contains "Subject" "invoice" { if size :over 10 { addflag "\\Seen"; fileinto "Bills"; stop; } } elsif true { discard; } keep;"#;
        assert_eq!(
            evaluate(source, &message).unwrap(),
            vec![
                Action::Flags {
                    operation: FlagOperation::Add,
                    flags: vec!["\\Seen".into()]
                },
                Action::FileInto {
                    mailbox: "Bills".into(),
                    copy: false,
                    flags: None,
                }
            ]
        );
    }
    #[test]
    fn rejects_unsupported_actions_even_in_unmatched_branches() {
        let source = "require \"reject\"; if false { reject \"no\"; } keep;";
        let error = evaluate(source, &Message::default()).unwrap_err();
        assert_eq!(error.line, 1);
        assert!(error.message.contains("unavailable"));
    }
    #[test]
    fn envelope_uses_delivery_envelope_not_header_from() {
        let message = Message {
            headers: vec![("From".into(), "forged@example.org".into())],
            envelope_from: Some("sender@example.net".into()),
            ..Message::default()
        };
        assert_eq!(evaluate("require \"envelope\"; if envelope :domain :is \"from\" \"example.org\" { discard; } else { keep; }", &message).unwrap(), vec![Action::Keep]);
    }
}
