//! Validation and the extension inventory of a parsed [`Script`].
//!
//! [`analyze`] walks the syntax tree once and reports:
//! - errors with line and column: `require` after another command, `elsif`/`else`
//!   without an `if`, wrong test arity for `if`/`allof`/`anyof`/`not`, a block on
//!   a command that takes none, an unknown command or test, and a command, test,
//!   tag or comparator used without the `require` that enables it;
//! - the extensions the script **requires** (its `require` lists) and the ones it
//!   **uses** (derived from the commands, tests and tags in it);
//! - the facts the server policy needs: every `redirect` target, `vacation`,
//!   `notify` targets and `include` names, each with its position.
//!
//! Unknown commands and tests are errors only while every required capability is
//! one this module knows. A script that requires a capability outside
//! [`KNOWN_CAPABILITIES`] (or uses `ihave`) may legitimately use commands we do not
//! model, so they are reported in [`Analysis::unknown`] instead and the policy's
//! extension allow-list decides.

use std::collections::BTreeSet;

use crate::grammar::{Argument, Command, Script, ScriptError, Span, Test, parse_script};

/// Capabilities this module models (RFC 5228 core comparators excluded, they
/// need no `require`).
pub const KNOWN_CAPABILITIES: &[&str] = &[
    "fileinto",
    "reject",
    "ereject",
    "envelope",
    "imap4flags",
    "body",
    "variables",
    "vacation",
    "vacation-seconds",
    "relational",
    "comparator-i;ascii-numeric",
    "regex",
    "date",
    "index",
    "copy",
    "include",
    "mailbox",
    "editheader",
    "spamtest",
    "spamtestplus",
    "virustest",
    "subaddress",
    "duplicate",
    "enotify",
    "ihave",
];

/// Commands: name → the capability that enables it (`None` = core).
const COMMANDS: &[(&str, Option<&str>)] = &[
    ("require", None),
    ("if", None),
    ("elsif", None),
    ("else", None),
    ("stop", None),
    ("keep", None),
    ("discard", None),
    ("redirect", None),
    ("fileinto", Some("fileinto")),
    ("reject", Some("reject")),
    ("ereject", Some("ereject")),
    ("vacation", Some("vacation")),
    ("setflag", Some("imap4flags")),
    ("addflag", Some("imap4flags")),
    ("removeflag", Some("imap4flags")),
    ("set", Some("variables")),
    ("include", Some("include")),
    ("return", Some("include")),
    ("global", Some("include")),
    ("addheader", Some("editheader")),
    ("deleteheader", Some("editheader")),
    ("notify", Some("enotify")),
    ("error", Some("ihave")),
];

/// Tests: name → the capability that enables it (`None` = core).
const TESTS: &[(&str, Option<&str>)] = &[
    ("address", None),
    ("allof", None),
    ("anyof", None),
    ("exists", None),
    ("false", None),
    ("header", None),
    ("not", None),
    ("size", None),
    ("true", None),
    ("envelope", Some("envelope")),
    ("body", Some("body")),
    ("hasflag", Some("imap4flags")),
    ("string", Some("variables")),
    ("date", Some("date")),
    ("currentdate", Some("date")),
    ("mailboxexists", Some("mailbox")),
    ("spamtest", Some("spamtest")),
    ("virustest", Some("virustest")),
    ("duplicate", Some("duplicate")),
    ("valid_notify_method", Some("enotify")),
    ("notify_method_capability", Some("enotify")),
    ("ihave", Some("ihave")),
];

/// Tags that only exist with an extension. Tags of the core (`:is`, `:all`,
/// `:over`, ...) and tags scoped to an extension command or test (`:days` of
/// `vacation`, `:raw` of `body`) need nothing beyond that command's capability.
const TAGS: &[(&str, &str)] = &[
    ("copy", "copy"),
    ("flags", "imap4flags"),
    ("value", "relational"),
    ("count", "relational"),
    ("regex", "regex"),
    ("user", "subaddress"),
    ("detail", "subaddress"),
    ("create", "mailbox"),
];

/// Tags whose capability depends on where they appear: `:index`/`:last` need
/// `index` in a test (RFC 5260) but are part of editheader's `deleteheader`
/// (RFC 5293); `:seconds` needs `vacation-seconds` on `vacation` (RFC 6131) but
/// is part of the `duplicate` test (RFC 7352).
const TEST_TAGS: &[(&str, &str)] = &[("index", "index"), ("last", "index")];

/// A `redirect`/`notify` destination as written.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Target {
    /// The literal value, or `None` when it contains a `${...}` variable and so
    /// cannot be judged before execution.
    pub value: Option<String>,
    pub span: Span,
}

/// What [`analyze`] learned about a script.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Analysis {
    /// Capabilities named in `require` (lower-cased).
    pub required: BTreeSet<String>,
    /// Where each required capability is named (first mention).
    pub require_spans: Vec<(String, Span)>,
    /// Capabilities the commands, tests and tags in the script need.
    pub used: BTreeSet<String>,
    /// Command and test names not modelled here (only possible when an unknown
    /// capability is required, see the module docs).
    pub unknown: BTreeSet<String>,
    /// Every `redirect` command's address.
    pub redirects: Vec<Target>,
    /// Every `notify` command's method URI (enotify).
    pub notify_targets: Vec<Target>,
    /// Positions of `vacation` commands.
    pub vacations: Vec<Span>,
    /// Names given to `include`.
    pub includes: Vec<Target>,
}

/// Parse and validate `source` in one step.
pub fn check(source: &str) -> Result<(Script, Analysis), ScriptError> {
    let script = parse_script(source)?;
    let analysis = analyze(&script)?;
    Ok((script, analysis))
}

/// Validate a parsed script and build its [`Analysis`].
pub fn analyze(script: &Script) -> Result<Analysis, ScriptError> {
    let mut a = Analysis::default();

    // `require` is only allowed before every other command (RFC 5228 §3.2).
    let mut past_requires = false;
    for cmd in &script.commands {
        if cmd.is("require") {
            if past_requires {
                return Err(ScriptError::at(
                    cmd.span,
                    "`require` must come before every other command",
                ));
            }
            let caps = match cmd.arguments.as_slice() {
                [arg] if cmd.tests.is_empty() => arg.strings(),
                _ => None,
            }
            .ok_or_else(|| ScriptError::at(cmd.span, "`require` takes one string list"))?;
            for cap in caps {
                let name = cap.value.to_ascii_lowercase();
                if a.required.insert(name.clone()) {
                    a.require_spans.push((name, cap.span));
                }
            }
        } else {
            past_requires = true;
        }
    }

    let strict = !a.required.contains("ihave")
        && a.required
            .iter()
            .all(|c| KNOWN_CAPABILITIES.contains(&c.as_str()));
    let mut w = Walker { a, strict };
    w.commands(&script.commands, true)?;
    Ok(w.a)
}

struct Walker {
    a: Analysis,
    strict: bool,
}

impl Walker {
    fn need(&mut self, cap: &str, span: Span, what: &str) -> Result<(), ScriptError> {
        self.a.used.insert(cap.to_string());
        let satisfied = self.a.required.contains(cap)
            || (cap == "vacation" && self.a.required.contains("vacation-seconds"))
            || (cap == "spamtest" && self.a.required.contains("spamtestplus"));
        if satisfied {
            Ok(())
        } else {
            Err(ScriptError::at(
                span,
                format!("{what} needs `require \"{cap}\"`"),
            ))
        }
    }

    fn commands(&mut self, cmds: &[Command], top: bool) -> Result<(), ScriptError> {
        let mut after_if = false;
        for cmd in cmds {
            let name = cmd.name.to_ascii_lowercase();
            if name == "require" {
                if !top {
                    return Err(ScriptError::at(
                        cmd.span,
                        "`require` is only allowed at the top of the script",
                    ));
                }
                after_if = false;
                continue;
            }
            match COMMANDS.iter().find(|(n, _)| *n == name) {
                Some((_, Some(cap))) => self.need(cap, cmd.span, &format!("`{name}`"))?,
                Some((_, None)) => {}
                None if self.strict => {
                    return Err(ScriptError::at(
                        cmd.span,
                        format!("unknown command `{}`", cmd.name),
                    ));
                }
                None => {
                    self.a.unknown.insert(name.clone());
                }
            }
            let known = COMMANDS.iter().any(|(n, _)| *n == name);
            match name.as_str() {
                "if" | "elsif" => {
                    if name == "elsif" && !after_if {
                        return Err(ScriptError::at(
                            cmd.span,
                            "`elsif` without a preceding `if`",
                        ));
                    }
                    if cmd.tests.len() != 1 || cmd.test_list {
                        return Err(ScriptError::at(
                            cmd.span,
                            format!("`{name}` takes exactly one test"),
                        ));
                    }
                    if !cmd.arguments.is_empty() {
                        return Err(ScriptError::at(
                            cmd.arguments[0].span,
                            format!("`{name}` takes no arguments"),
                        ));
                    }
                    if cmd.block.is_none() {
                        return Err(ScriptError::at(cmd.span, format!("`{name}` needs a block")));
                    }
                    after_if = true;
                }
                "else" => {
                    if !after_if {
                        return Err(ScriptError::at(cmd.span, "`else` without a preceding `if`"));
                    }
                    if !cmd.tests.is_empty() || !cmd.arguments.is_empty() {
                        return Err(ScriptError::at(
                            cmd.span,
                            "`else` takes no test or arguments",
                        ));
                    }
                    if cmd.block.is_none() {
                        return Err(ScriptError::at(cmd.span, "`else` needs a block"));
                    }
                    after_if = false;
                }
                _ => {
                    after_if = false;
                    if known {
                        if cmd.block.is_some() {
                            return Err(ScriptError::at(
                                cmd.span,
                                format!("`{name}` takes no block"),
                            ));
                        }
                        if !cmd.tests.is_empty() {
                            return Err(ScriptError::at(
                                cmd.tests[0].span,
                                format!("`{name}` takes no test"),
                            ));
                        }
                    }
                }
            }
            self.arguments(&cmd.arguments, false)?;
            if name == "vacation"
                && let Some(arg) = cmd
                    .arguments
                    .iter()
                    .find(|a| a.tag().is_some_and(|t| t.eq_ignore_ascii_case("seconds")))
            {
                self.need("vacation-seconds", arg.span, "`:seconds`")?;
            }
            self.command_facts(&name, cmd);
            for t in &cmd.tests {
                self.test(t)?;
            }
            if let Some(block) = &cmd.block {
                self.commands(&block.commands, false)?;
            }
        }
        Ok(())
    }

    fn command_facts(&mut self, name: &str, cmd: &Command) {
        let last_string = || {
            cmd.arguments
                .iter()
                .rev()
                .find_map(|a| a.strings().and_then(|s| s.first()))
        };
        match name {
            "redirect" => {
                if let Some(s) = last_string() {
                    self.a.redirects.push(target(&s.value, s.span));
                } else {
                    self.a.redirects.push(Target {
                        value: None,
                        span: cmd.span,
                    });
                }
            }
            "notify" => {
                // `notify [tags] <method: string>` — the method is the last string.
                if let Some(s) = last_string() {
                    self.a.notify_targets.push(target(&s.value, s.span));
                }
            }
            "vacation" => self.a.vacations.push(cmd.span),
            "include" => {
                if let Some(s) = last_string() {
                    self.a.includes.push(target(&s.value, s.span));
                }
            }
            _ => {}
        }
    }

    fn test(&mut self, t: &Test) -> Result<(), ScriptError> {
        let name = t.name.to_ascii_lowercase();
        match TESTS.iter().find(|(n, _)| *n == name) {
            Some((_, Some(cap))) => self.need(cap, t.span, &format!("test `{name}`"))?,
            Some((_, None)) => {}
            None if self.strict => {
                return Err(ScriptError::at(
                    t.span,
                    format!("unknown test `{}`", t.name),
                ));
            }
            None => {
                self.a.unknown.insert(name.clone());
            }
        }
        match name.as_str() {
            "allof" | "anyof" => {
                if !t.test_list || !t.arguments.is_empty() {
                    return Err(ScriptError::at(
                        t.span,
                        format!("`{name}` takes a test list"),
                    ));
                }
            }
            "not" => {
                if t.tests.len() != 1 || t.test_list || !t.arguments.is_empty() {
                    return Err(ScriptError::at(t.span, "`not` takes exactly one test"));
                }
            }
            _ if TESTS.iter().any(|(n, _)| *n == name) && !t.tests.is_empty() => {
                return Err(ScriptError::at(
                    t.tests[0].span,
                    format!("test `{name}` takes no nested test"),
                ));
            }
            _ => {}
        }
        self.arguments(&t.arguments, true)?;
        for inner in &t.tests {
            self.test(inner)?;
        }
        Ok(())
    }

    fn arguments(&mut self, args: &[Argument], in_test: bool) -> Result<(), ScriptError> {
        for (i, arg) in args.iter().enumerate() {
            let Some(tag) = arg.tag() else { continue };
            let tag = tag.to_ascii_lowercase();
            let scoped = if in_test { TEST_TAGS } else { &[] };
            if let Some((_, cap)) = TAGS.iter().chain(scoped).find(|(n, _)| *n == tag) {
                self.need(cap, arg.span, &format!("`:{tag}`"))?;
            }
            if tag == "comparator" {
                let cmp = args
                    .get(i + 1)
                    .and_then(|a| a.strings())
                    .and_then(|s| s.first());
                match cmp {
                    Some(s) if s.value.eq_ignore_ascii_case("i;ascii-numeric") => {
                        self.need(
                            "comparator-i;ascii-numeric",
                            s.span,
                            "comparator `i;ascii-numeric`",
                        )?;
                    }
                    Some(_) => {}
                    None => {
                        return Err(ScriptError::at(
                            arg.span,
                            "`:comparator` needs a comparator name",
                        ));
                    }
                }
            }
        }
        Ok(())
    }
}

fn target(value: &str, span: Span) -> Target {
    Target {
        value: if value.contains("${") {
            None
        } else {
            Some(value.to_string())
        },
        span,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn bad(src: &str) -> ScriptError {
        check(src).expect_err("must be refused")
    }

    #[test]
    fn inventory_lists_required_and_used() {
        let (_, a) = check(
            "require [\"fileinto\", \"copy\", \"imap4flags\", \"vacation\", \"relational\", \
             \"comparator-i;ascii-numeric\"];\n\
             if header :value \"gt\" :comparator \"i;ascii-numeric\" \"x-spam\" \"5\" {\n\
               fileinto :copy :flags \"\\\\Seen\" \"Junk\";\n\
             }\n\
             vacation :days 3 \"away\";\n",
        )
        .unwrap();
        let req: Vec<&str> = a.required.iter().map(String::as_str).collect();
        assert_eq!(
            req,
            [
                "comparator-i;ascii-numeric",
                "copy",
                "fileinto",
                "imap4flags",
                "relational",
                "vacation"
            ]
        );
        assert_eq!(a.used, a.required, "every required capability is used here");
        assert_eq!(a.vacations.len(), 1);
    }

    #[test]
    fn missing_require_is_refused_at_the_use() {
        let e = bad("keep;\nfileinto \"x\";");
        assert_eq!((e.line, e.column), (2, 1));
        assert!(e.message.contains("require \"fileinto\""), "{e}");

        let e = bad("require \"fileinto\";\nfileinto :copy \"x\";");
        assert_eq!((e.line, e.column), (2, 10));
        assert!(e.message.contains("\"copy\""), "{e}");

        let e = bad("if body :text :contains \"x\" { keep; }");
        assert!(e.message.contains("\"body\""), "{e}");

        let e = bad("if header :comparator \"i;ascii-numeric\" :is \"x\" \"1\" { keep; }");
        assert!(e.message.contains("comparator-i;ascii-numeric"), "{e}");
    }

    #[test]
    fn context_scoped_tags() {
        // `:seconds` belongs to `duplicate`, but needs vacation-seconds on `vacation`.
        check(
            "require \"duplicate\";
if duplicate :seconds 60 { discard; }",
        )
        .unwrap();
        let e = bad("require \"vacation\";
vacation :seconds 60 \"x\";");
        assert!(e.message.contains("vacation-seconds"), "{e}");
        check(
            "require [\"vacation\", \"vacation-seconds\"];
vacation :seconds 60 \"x\";",
        )
        .unwrap();
        // `:index` belongs to `deleteheader`, but needs index in a test.
        check(
            "require \"editheader\";
deleteheader :index 1 \"x-a\";",
        )
        .unwrap();
        let e = bad("if header :index 1 :is \"x-a\" \"1\" { keep; }");
        assert!(e.message.contains("\"index\""), "{e}");
    }

    #[test]
    fn spamtestplus_satisfies_spamtest() {
        assert!(
            check(
                "require \"spamtestplus\";\nif spamtest :percent :value \"gt\" \"50\" { discard; }"
            )
            .is_err()
        );
        let src = "require [\"spamtestplus\", \"relational\"];\n\
                   if spamtest :percent :value \"gt\" \"50\" { discard; }";
        check(src).unwrap();
    }

    #[test]
    fn structure_is_checked() {
        assert!(
            bad("keep;\nrequire \"fileinto\";")
                .message
                .contains("before every other")
        );
        assert!(
            bad("if true { require \"fileinto\"; }")
                .message
                .contains("top of the script")
        );
        assert!(
            bad("elsif true { keep; }")
                .message
                .contains("without a preceding")
        );
        assert!(
            bad("keep; else { keep; }")
                .message
                .contains("without a preceding")
        );
        assert!(bad("if true keep;").message.contains("needs a block"));
        assert!(
            bad("if (true, false) { keep; }")
                .message
                .contains("exactly one test")
        );
        assert!(bad("if allof true { keep; }").message.contains("test list"));
        assert!(
            bad("if not (true, false) { keep; }")
                .message
                .contains("exactly one")
        );
        assert!(bad("keep { stop; }").message.contains("no block"));
        assert!(bad("discard true;").message.contains("no test"));
        assert!(
            bad("if size :over 1 true { keep; }")
                .message
                .contains("no nested test")
        );
        assert!(bad("frobnicate;").message.contains("unknown command"));
        assert!(bad("if frob { keep; }").message.contains("unknown test"));
        // if / elsif / else chains are fine, and a chain ends at the next command.
        check("if true { keep; } elsif false { stop; } else { discard; }").unwrap();
        assert!(
            bad("if true { keep; } keep; else { stop; }")
                .message
                .contains("without")
        );
    }

    #[test]
    fn an_unknown_required_capability_relaxes_unknown_names() {
        let (_, a) =
            check("require \"x-vendor\";\nfrobnicate \"a\";\nif vendortest { keep; }").unwrap();
        assert!(a.unknown.contains("frobnicate"));
        assert!(a.unknown.contains("vendortest"));
        assert!(a.required.contains("x-vendor"));
    }

    #[test]
    fn policy_facts_are_collected() {
        let a = check(
            "require [\"enotify\", \"variables\", \"include\"];\n\
             redirect \"a@example.com\";\n\
             if true { redirect :copy \"${dest}\"; }\n\
             notify :message \"hi\" \"mailto:b@example.org\";\n\
             include :personal \"common\";\n",
        );
        // `:copy` without `require "copy"` is refused...
        assert!(a.is_err());
        let (_, a) = check(
            "require [\"enotify\", \"variables\", \"include\", \"copy\"];\n\
             redirect \"a@example.com\";\n\
             if true { redirect :copy \"${dest}\"; }\n\
             notify :message \"hi\" \"mailto:b@example.org\";\n\
             include :personal \"common\";\n",
        )
        .unwrap();
        assert_eq!(a.redirects.len(), 2);
        assert_eq!(a.redirects[0].value.as_deref(), Some("a@example.com"));
        assert_eq!(a.redirects[0].span.line, 2);
        assert_eq!(
            a.redirects[1].value, None,
            "a variable target cannot be judged"
        );
        assert_eq!(
            a.notify_targets[0].value.as_deref(),
            Some("mailto:b@example.org")
        );
        assert_eq!(a.includes[0].value.as_deref(), Some("common"));
    }
}
