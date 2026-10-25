//! Server-side policy for Sieve scripts, checked before any `PUTSCRIPT`.
//!
//! [`enforce`] parses and validates the script ([`crate::extensions::check`]) and
//! then applies a [`Policy`]:
//! - size (`sieve.maxScriptBytes`), checked before parsing;
//! - `redirect` targets outside the account's own domains, unless external
//!   forwarding is allowed (`cap.sieve.forward_external`); a target built from
//!   variables cannot be judged, so it counts as external; `notify` methods are a
//!   way out of the domain too and follow the same rule;
//! - the number of `redirect` commands (`sieve.maxRedirects`);
//! - `vacation` (`sieve.vacationAllowed`);
//! - extensions outside the allow-list (`sieve.allowedExtensions`) or not
//!   advertised by the upstream server;
//! - raw edits (`cap.sieve.raw_edit`).
//!
//! [`check_script_count`] applies `sieve.maxScripts`. Every violation carries a
//! line and column. Nothing here logs the script.

use std::collections::BTreeSet;

use crate::extensions::{Analysis, Target, check};
use crate::grammar::{ScriptError, Span};

/// The limits and permissions one save is checked against. `None` limits are
/// unlimited; `None` extension sets mean "no restriction from this source".
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Policy {
    pub max_script_bytes: Option<usize>,
    pub max_scripts: Option<usize>,
    pub max_redirects: Option<usize>,
    /// The account's own domains (compared case-insensitively).
    pub own_domains: Vec<String>,
    /// `cap.sieve.forward_external`.
    pub forward_external: bool,
    /// `sieve.vacationAllowed`.
    pub vacation_allowed: bool,
    /// `sieve.allowedExtensions` (lower-case names).
    pub allowed_extensions: Option<BTreeSet<String>>,
    /// The upstream's `SIEVE` capability list (lower-case names).
    pub advertised: Option<BTreeSet<String>>,
    /// `cap.sieve.raw_edit`.
    pub raw_edit: bool,
}

/// How the script being saved was produced.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EditKind {
    /// Only builder-managed blocks changed (see [`crate::managed::hand_written`]).
    Builder,
    /// The user edited the script text.
    Raw,
}

/// Why a script was refused.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ViolationKind {
    TooLarge,
    TooManyScripts,
    Invalid,
    RawEditNotAllowed,
    ExternalTarget,
    TooManyRedirects,
    VacationNotAllowed,
    ExtensionNotAllowed,
    ExtensionNotAdvertised,
}

impl ViolationKind {
    /// A stable machine-readable code for API error bodies.
    pub fn code(self) -> &'static str {
        match self {
            ViolationKind::TooLarge => "sieve.tooLarge",
            ViolationKind::TooManyScripts => "sieve.tooManyScripts",
            ViolationKind::Invalid => "sieve.invalid",
            ViolationKind::RawEditNotAllowed => "sieve.rawEditNotAllowed",
            ViolationKind::ExternalTarget => "sieve.externalTarget",
            ViolationKind::TooManyRedirects => "sieve.tooManyRedirects",
            ViolationKind::VacationNotAllowed => "sieve.vacationNotAllowed",
            ViolationKind::ExtensionNotAllowed => "sieve.extensionNotAllowed",
            ViolationKind::ExtensionNotAdvertised => "sieve.extensionNotAdvertised",
        }
    }
}

/// One refusal, with the position it refers to (line 1, column 1 for refusals
/// about the whole script).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Violation {
    pub kind: ViolationKind,
    pub line: u32,
    pub column: u32,
    pub message: String,
}

impl Violation {
    fn at(kind: ViolationKind, span: Span, message: impl Into<String>) -> Self {
        Violation {
            kind,
            line: span.line.max(1),
            column: span.column.max(1),
            message: message.into(),
        }
    }

    fn whole(kind: ViolationKind, message: impl Into<String>) -> Self {
        Violation {
            kind,
            line: 1,
            column: 1,
            message: message.into(),
        }
    }
}

impl std::fmt::Display for Violation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "line {}, column {}: {}",
            self.line, self.column, self.message
        )
    }
}

impl From<ScriptError> for Violation {
    fn from(e: ScriptError) -> Self {
        Violation {
            kind: ViolationKind::Invalid,
            line: e.line,
            column: e.column,
            message: e.message,
        }
    }
}

/// Check one script against `policy`. Returns the analysis when it may be
/// stored, or every violation found (a syntax error stops the check early, as
/// nothing else can be judged without a tree).
pub fn enforce(source: &str, policy: &Policy, edit: EditKind) -> Result<Analysis, Vec<Violation>> {
    if let Some(max) = policy.max_script_bytes
        && source.len() > max
    {
        return Err(vec![Violation::whole(
            ViolationKind::TooLarge,
            format!("the script is {} bytes; the limit is {max}", source.len()),
        )]);
    }
    let mut out = Vec::new();
    if edit == EditKind::Raw && !policy.raw_edit {
        out.push(Violation::whole(
            ViolationKind::RawEditNotAllowed,
            "editing the script text is not allowed for this account",
        ));
    }
    let (_, analysis) = match check(source) {
        Ok(v) => v,
        Err(e) => {
            out.push(e.into());
            return Err(out);
        }
    };

    // A remote server may understand vendor commands, but we cannot prove what
    // they send or execute. Likewise, an include can hide actions in a script
    // outside this validation transaction. Refuse both until a trusted resolver
    // can recursively validate the complete execution graph under this policy.
    if !analysis.unknown.is_empty() || !analysis.includes.is_empty() {
        out.push(Violation::whole(
            ViolationKind::ExtensionNotAllowed,
            "unmodelled commands and unresolved includes cannot be policy checked",
        ));
    }

    for (cap, span) in &analysis.require_spans {
        if let Some(allowed) = &policy.allowed_extensions
            && !allowed.contains(cap)
        {
            out.push(Violation::at(
                ViolationKind::ExtensionNotAllowed,
                *span,
                format!("the extension `{cap}` is not allowed"),
            ));
        } else if let Some(adv) = &policy.advertised
            && !adv.contains(cap)
        {
            out.push(Violation::at(
                ViolationKind::ExtensionNotAdvertised,
                *span,
                format!("the mail server does not support the extension `{cap}`"),
            ));
        }
    }

    if !policy.vacation_allowed
        && let Some(span) = analysis.vacations.first()
    {
        out.push(Violation::at(
            ViolationKind::VacationNotAllowed,
            *span,
            "automatic replies are not allowed for this account",
        ));
    }

    if let Some(max) = policy.max_redirects
        && analysis.redirects.len() > max
    {
        let span = analysis.redirects[max].span;
        out.push(Violation::at(
            ViolationKind::TooManyRedirects,
            span,
            format!("at most {max} redirects are allowed"),
        ));
    }

    if !policy.forward_external {
        for t in &analysis.redirects {
            if !target_is_own(t.value.as_deref(), &policy.own_domains) {
                out.push(external(t, "redirect"));
            }
        }
        for t in &analysis.notify_targets {
            let own = t
                .value
                .as_deref()
                .and_then(mailto_addresses)
                .is_some_and(|addrs| {
                    !addrs.is_empty()
                        && addrs
                            .iter()
                            .all(|a| target_is_own(Some(a), &policy.own_domains))
                });
            if !own {
                out.push(external(t, "notification"));
            }
        }
    }

    if out.is_empty() {
        Ok(analysis)
    } else {
        Err(out)
    }
}

fn external(t: &Target, what: &str) -> Violation {
    let message = match &t.value {
        Some(v) => format!("{what} to {v:?} leaves your domain, which is not allowed"),
        None => {
            format!("a {what} target built from variables cannot be checked, so it is not allowed")
        }
    };
    Violation::at(ViolationKind::ExternalTarget, t.span, message)
}

/// Whether `address` is a mailbox in one of `domains`.
fn target_is_own(address: Option<&str>, domains: &[String]) -> bool {
    let Some(address) = address else { return false };
    let address = address.trim();
    let address = address
        .rsplit_once('<')
        .and_then(|(_, a)| a.strip_suffix('>'))
        .unwrap_or(address);
    let Some((local, domain)) = address.rsplit_once('@') else {
        return false;
    };
    if local.is_empty() || domain.is_empty() {
        return false;
    }
    let domain = domain.trim_end_matches('.');
    domains
        .iter()
        .any(|d| d.trim_end_matches('.').eq_ignore_ascii_case(domain))
}

/// The recipients of a `mailto:` URI (RFC 6068), without the query part.
fn mailto_addresses(uri: &str) -> Option<Vec<String>> {
    let rest = uri
        .get(..7)
        .filter(|p| p.eq_ignore_ascii_case("mailto:"))
        .map(|_| &uri[7..])?;
    let (to, query) = rest.split_once('?').unwrap_or((rest, ""));
    // `to=` / `cc=` / `bcc=` in the query add recipients too.
    let mut out: Vec<String> = to
        .split(',')
        .filter(|s| !s.is_empty())
        .map(percent_decode)
        .collect();
    for pair in query.split('&') {
        if let Some((k, v)) = pair.split_once('=')
            && matches!(k.to_ascii_lowercase().as_str(), "to" | "cc" | "bcc")
        {
            out.extend(v.split(',').filter(|s| !s.is_empty()).map(percent_decode));
        }
    }
    Some(out)
}

fn percent_decode(s: &str) -> String {
    let bytes = s.as_bytes();
    let mut out = Vec::with_capacity(bytes.len());
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'%'
            && let Some(hex) = s.get(i + 1..i + 3)
            && let Ok(v) = u8::from_str_radix(hex, 16)
        {
            out.push(v);
            i += 3;
            continue;
        }
        out.push(bytes[i]);
        i += 1;
    }
    String::from_utf8_lossy(&out).into_owned()
}

/// `sieve.maxScripts`: may an account holding `existing` scripts store one more?
/// Replacing a script that already exists never adds one.
pub fn check_script_count(
    existing: usize,
    replaces_existing: bool,
    policy: &Policy,
) -> Result<(), Violation> {
    match policy.max_scripts {
        Some(max) if !replaces_existing && existing >= max => Err(Violation::whole(
            ViolationKind::TooManyScripts,
            format!("at most {max} scripts are allowed"),
        )),
        _ => Ok(()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn policy() -> Policy {
        Policy {
            own_domains: vec!["example.com".into(), "Example.ORG".into()],
            vacation_allowed: true,
            raw_edit: true,
            ..Policy::default()
        }
    }

    fn kinds(r: Result<Analysis, Vec<Violation>>) -> Vec<ViolationKind> {
        r.err().unwrap_or_default().iter().map(|v| v.kind).collect()
    }

    #[test]
    fn redirects_inside_own_domains_pass_and_outside_are_refused() {
        let p = policy();
        enforce(
            "redirect \"a@example.com\";\nredirect \"B <b@EXAMPLE.org.>\";",
            &p,
            EditKind::Raw,
        )
        .unwrap();
        let err = enforce("keep;\nredirect \"x@evil.test\";", &p, EditKind::Raw).unwrap_err();
        assert_eq!(err.len(), 1);
        assert_eq!(err[0].kind, ViolationKind::ExternalTarget);
        assert_eq!((err[0].line, err[0].column), (2, 10));
        // Look-alike domains are not the account's.
        assert_eq!(
            kinds(enforce(
                "redirect \"a@example.com.evil.test\";",
                &p,
                EditKind::Raw
            )),
            [ViolationKind::ExternalTarget]
        );
        assert_eq!(
            kinds(enforce(
                "redirect \"a@sub.example.com\";",
                &p,
                EditKind::Raw
            )),
            [ViolationKind::ExternalTarget]
        );
        assert_eq!(
            kinds(enforce("redirect \"nobody\";", &p, EditKind::Raw)),
            [ViolationKind::ExternalTarget]
        );
    }

    #[test]
    fn variable_targets_count_as_external() {
        let p = policy();
        let src = "require \"variables\";\nset \"d\" \"a@example.com\";\nredirect \"${d}\";";
        assert_eq!(
            kinds(enforce(src, &p, EditKind::Raw)),
            [ViolationKind::ExternalTarget]
        );
        let open = Policy {
            forward_external: true,
            ..policy()
        };
        enforce(src, &open, EditKind::Raw).unwrap();
    }

    #[test]
    fn notify_targets_follow_the_redirect_rule() {
        let p = policy();
        let ok = "require \"enotify\";\nnotify \"mailto:me@example.com?subject=hi\";";
        enforce(ok, &p, EditKind::Raw).unwrap();
        for bad in [
            "require \"enotify\";\nnotify \"mailto:me@example.com?cc=x%40evil.test\";",
            "require \"enotify\";\nnotify \"mailto:x@evil.test\";",
            "require \"enotify\";\nnotify \"xmpp:x@example.com\";",
        ] {
            assert_eq!(
                kinds(enforce(bad, &p, EditKind::Raw)),
                [ViolationKind::ExternalTarget],
                "{bad}"
            );
        }
    }

    #[test]
    fn redirect_count_vacation_size_and_raw_edit() {
        let p = Policy {
            max_redirects: Some(1),
            vacation_allowed: false,
            raw_edit: false,
            max_script_bytes: Some(200),
            ..policy()
        };
        let src = "require \"vacation\";\nredirect \"a@example.com\";\nredirect \"b@example.com\";\nvacation \"x\";";
        let err = enforce(src, &p, EditKind::Builder).unwrap_err();
        let k: Vec<_> = err.iter().map(|v| v.kind).collect();
        assert_eq!(
            k,
            [
                ViolationKind::VacationNotAllowed,
                ViolationKind::TooManyRedirects
            ]
        );
        assert_eq!(
            err[1].line, 3,
            "points at the first redirect over the limit"
        );
        assert_eq!(
            kinds(enforce("keep;", &p, EditKind::Raw)),
            [ViolationKind::RawEditNotAllowed]
        );
        let big = format!("keep;{}", " ".repeat(300));
        assert_eq!(
            kinds(enforce(&big, &p, EditKind::Builder)),
            [ViolationKind::TooLarge]
        );
    }

    #[test]
    fn extensions_must_be_allowed_and_advertised() {
        let p = Policy {
            allowed_extensions: Some(["fileinto".to_string(), "vacation".to_string()].into()),
            advertised: Some(["fileinto".to_string()].into()),
            ..policy()
        };
        enforce("require \"fileinto\";\nfileinto \"A\";", &p, EditKind::Raw).unwrap();
        let err = enforce(
            "require [\"fileinto\", \"vacation\", \"editheader\"];\nkeep;",
            &p,
            EditKind::Raw,
        )
        .unwrap_err();
        let k: Vec<_> = err.iter().map(|v| v.kind).collect();
        assert_eq!(
            k,
            [
                ViolationKind::ExtensionNotAdvertised,
                ViolationKind::ExtensionNotAllowed
            ]
        );
        assert_eq!((err[0].line, err[0].column), (1, 22));
    }

    #[test]
    fn syntax_errors_are_violations_with_positions() {
        let err = enforce("keep;\nif true {", &policy(), EditKind::Raw).unwrap_err();
        assert_eq!(err[0].kind, ViolationKind::Invalid);
        assert_eq!(err[0].line, 2);
        assert_eq!(err[0].kind.code(), "sieve.invalid");
    }

    #[test]
    fn script_count() {
        let p = Policy {
            max_scripts: Some(2),
            ..policy()
        };
        assert!(check_script_count(1, false, &p).is_ok());
        assert!(check_script_count(2, true, &p).is_ok());
        assert_eq!(
            check_script_count(2, false, &p).unwrap_err().kind,
            ViolationKind::TooManyScripts
        );
        assert!(check_script_count(99, false, &policy()).is_ok());
    }
}

#[cfg(test)]
mod opaque_policy_regressions {
    use super::*;
    #[test]
    fn opaque_vendor_commands_and_unresolved_includes_cannot_hide_forwarding() {
        let policy = Policy {
            raw_edit: true,
            own_domains: vec!["example.org".into()],
            advertised: Some(["vnd.dovecot.pipe".into(), "include".into()].into()),
            ..Default::default()
        };
        for source in [
            "require [\"vnd.dovecot.pipe\"]; pipe \"exfiltrate\";",
            "require [\"include\"]; include :personal \"unvalidated-external-forward\";",
        ] {
            assert!(
                enforce(source, &policy, EditKind::Raw).is_err(),
                "opaque execution escaped policy: {source}"
            );
        }
    }
}
