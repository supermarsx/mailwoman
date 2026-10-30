//! Complete-list recovery for quoted SMTPUTF8 mailboxes. The upstream lenient
//! parser treats angle brackets inside quoted local parts as delimiters.
//! This path never modifies message bytes and never returns a partial list.
use mail_parser::MessageParser;
use mw_jmap::EmailAddress;

pub(crate) fn parse(value: &str) -> Option<Vec<EmailAddress>> {
    if value.len() > 64 * 1024 {
        return None;
    }
    let value = value.trim();
    let mut unfolded = String::with_capacity(value.len());
    let mut chars = value.chars().peekable();
    while let Some(c) = chars.next() {
        if c == '\r' {
            if chars.next()? != '\n' || !matches!(chars.peek(), Some(' ' | '\t')) {
                return None;
            }
        } else if c == '\n' || (c.is_control() && c != '\t') {
            return None;
        } else {
            unfolded.push(c);
        }
    }
    let mut tokens = Vec::new();
    let mut token = String::new();
    let (mut quote, mut escape, mut comment, mut angle) = (false, false, 0usize, false);
    for c in unfolded.chars() {
        if escape {
            if comment == 0 {
                token.push(c);
            }
            escape = false;
            continue;
        }
        if c == '\\' && (quote || comment > 0) {
            if comment == 0 {
                token.push(c);
            }
            escape = true;
            continue;
        }
        if !quote && c == '(' {
            comment += 1;
            if comment > 32 {
                return None;
            }
            continue;
        }
        if comment > 0 {
            if c == ')' {
                comment -= 1;
            }
            continue;
        }
        if c == '"' {
            quote = !quote;
        } else if !quote {
            match c {
                '<' => {
                    if angle {
                        return None;
                    }
                    angle = true;
                }
                '>' => {
                    if !angle {
                        return None;
                    }
                    angle = false;
                }
                ',' if !angle => {
                    if token.trim().is_empty() {
                        return None;
                    }
                    tokens.push(std::mem::take(&mut token));
                    continue;
                }
                ';' | ')' => return None,
                ':' if !angle && !token.contains('@') => return None, // groups stay with upstream parser
                _ => {}
            }
        }
        token.push(c);
    }
    if quote || escape || comment > 0 || angle || token.trim().is_empty() {
        return None;
    }
    tokens.push(token);
    if tokens.len() > 1000 {
        return None;
    }
    tokens
        .into_iter()
        .map(|token| {
            let token = token.trim();
            let (mut quote, mut escape, mut open, mut close) = (false, false, None, None);
            for (at, c) in token.char_indices() {
                if escape {
                    escape = false;
                    continue;
                }
                if c == '\\' && quote {
                    escape = true;
                    continue;
                }
                if c == '"' {
                    quote = !quote;
                } else if !quote && c == '<' {
                    if open.is_some() {
                        return None;
                    }
                    open = Some(at);
                } else if !quote && c == '>' {
                    if close.is_some() {
                        return None;
                    }
                    close = Some(at);
                }
            }
            let (display, mailbox) = match (open, close) {
                (Some(a), Some(b)) if b > a && token[b + 1..].trim().is_empty() => {
                    (Some(token[..a].trim()), token[a + 1..b].trim())
                }
                (None, None) => (None, token),
                _ => return None,
            };
            crate::validate_mailbox(mailbox).ok()?;
            // Preserve the mature parser's encoded-word/display-name behavior,
            // using an ordinary placeholder so it never reparses the quoted path.
            let name = display.filter(|d| !d.is_empty()).and_then(|display| {
                let dummy = format!("From: {display} <address@example.invalid>\r\n\r\n");
                let parsed = MessageParser::default().parse(dummy.as_bytes())?;
                parsed.from()?.iter().next()?.name().map(str::to_owned)
            });
            Some(EmailAddress {
                name,
                email: mailbox.to_owned(),
            })
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn complete_quoted_lists_preserve_names_and_mailboxes() {
        let list = parse(
            "=?UTF-8?B?Sm9zw6k=?= <\"原,a<>@b\"@example.org>,\r\n \"Another\" <a@example.org>",
        )
        .unwrap();
        assert_eq!(list.len(), 2);
        assert_eq!(list[0].name.as_deref(), Some("José"));
        assert_eq!(list[0].email, "\"原,a<>@b\"@example.org");
        assert_eq!(list[1].email, "a@example.org");
    }
    #[test]
    fn incomplete_groups_injection_and_partial_lists_are_not_recovered() {
        for raw in [
            "\"a\"@example.org, bad",
            "Group: \"a\"@example.org;",
            "\"a\"@example.org\r\nBcc: evil@example.org",
            "Name <\"a\"@example.org> tail",
            "Name <\"a\"@example.org",
            "\"a\"@example.org,",
        ] {
            assert!(parse(raw).is_none(), "{raw}");
        }
    }
}
