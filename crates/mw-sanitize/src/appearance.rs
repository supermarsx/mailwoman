//! Strict, owner-scoped appearance CSS. This policy is separate from email CSS.
use cssparser::{
    AtRuleParser, BasicParseErrorKind, CowRcStr, DeclarationParser, ParseError, Parser,
    ParserInput, ParserState, QualifiedRuleParser, RuleBodyItemParser, RuleBodyParser,
    StyleSheetParser, ToCss, Token,
};

pub const MAX_CSS_BYTES: usize = 16 * 1024;
#[derive(Clone, Copy)]
pub enum Scope {
    Mailbox,
    Admin,
}
impl Scope {
    pub fn class(self) -> &'static str {
        match self {
            Self::Mailbox => "mw-custom-mail",
            Self::Admin => "mw-custom-admin",
        }
    }
}

/// Only visual declarations, simple selectors and state pseudo-classes are accepted.
/// External resources, at-rules, selector functions/attributes and positioning are refused.
pub fn stylesheet(source: &str, scope: Scope) -> Result<String, &'static str> {
    if source.len() > MAX_CSS_BYTES {
        return Err("Custom CSS is limited to 16 KiB.");
    }
    let mut input = ParserInput::new(source);
    let mut parser = Parser::new(&mut input);
    let mut rules = Rules {
        scope,
        declarations: 0,
    };
    let mut result = String::new();
    for (count, rule) in StyleSheetParser::new(&mut parser, &mut rules).enumerate() {
        if count >= 128 {
            return Err("Custom CSS is limited to 128 rules.");
        }
        match rule {
            Ok(rule) => result.push_str(&rule),
            Err((error, _)) => return Err(message(error)),
        }
    }
    Ok(result)
}
fn message(error: ParseError<'_, &'static str>) -> &'static str {
    match error.kind {
        cssparser::ParseErrorKind::Custom(message) => message,
        _ => "Invalid CSS. At-rules and external resources are not supported.",
    }
}
struct Rules {
    scope: Scope,
    declarations: usize,
}
impl<'i> AtRuleParser<'i> for Rules {
    type Prelude = ();
    type AtRule = String;
    type Error = &'static str;
}
impl<'i> QualifiedRuleParser<'i> for Rules {
    type Prelude = String;
    type QualifiedRule = String;
    type Error = &'static str;
    fn parse_prelude<'t>(
        &mut self,
        input: &mut Parser<'i, 't>,
    ) -> Result<String, ParseError<'i, Self::Error>> {
        let mut selectors = Vec::new();
        let mut current = String::new();
        while !input.is_exhausted() {
            let token = input.next_including_whitespace_and_comments()?.clone();
            match &token {
                Token::Ident(_)
                | Token::IDHash(_)
                | Token::Hash(_)
                | Token::Delim('.' | '*' | '>' | '+' | '~') => {
                    token.to_css(&mut current).expect("String writer");
                }
                Token::WhiteSpace(_) | Token::Comment(_) => current.push(' '),
                Token::Comma => {
                    selectors.push(std::mem::take(&mut current));
                }
                Token::Colon => {
                    let name = input.expect_ident_cloned()?;
                    if !matches!(
                        name.as_ref(),
                        "root"
                            | "hover"
                            | "focus"
                            | "focus-visible"
                            | "focus-within"
                            | "active"
                            | "disabled"
                            | "enabled"
                            | "checked"
                            | "first-child"
                            | "last-child"
                            | "empty"
                    ) {
                        return Err(input.new_custom_error("Unsupported CSS pseudo-class."));
                    }
                    current.push(':');
                    current.push_str(&name);
                }
                _ => return Err(input.new_custom_error(
                    "Use simple CSS selectors without attributes, functions or pseudo-elements.",
                )),
            }
        }
        selectors.push(current);
        let mut output = Vec::new();
        for selector in selectors {
            let selector = selector.trim();
            if selector.is_empty() || selector.len() > 2048 {
                return Err(input.new_custom_error("Invalid or oversized CSS selector."));
            }
            let class = self.scope.class();
            // Only exact root aliases target the owner root. A sibling selector
            // such as `body + .other` stays under the descendant constraint.
            output.push(if matches!(selector, "body" | "html" | ":root") {
                format!(".{class}")
            } else {
                format!(".{class} :is({selector})")
            });
        }
        Ok(output.join(","))
    }
    fn parse_block<'t>(
        &mut self,
        selectors: String,
        _: &ParserState,
        input: &mut Parser<'i, 't>,
    ) -> Result<String, ParseError<'i, Self::Error>> {
        let mut declarations = Declarations;
        let mut output = String::new();
        for item in RuleBodyParser::new(input, &mut declarations) {
            self.declarations += 1;
            if self.declarations > 512 {
                return Err(input.new_custom_error("Custom CSS is limited to 512 declarations."));
            }
            let declaration = item.map_err(|(error, _)| error)?;
            output.push_str(&declaration);
            output.push(';');
        }
        Ok(format!("{selectors}{{{output}}}"))
    }
}
struct Declarations;
impl<'i> AtRuleParser<'i> for Declarations {
    type Prelude = ();
    type AtRule = String;
    type Error = &'static str;
}
impl<'i> QualifiedRuleParser<'i> for Declarations {
    type Prelude = ();
    type QualifiedRule = String;
    type Error = &'static str;
}
impl<'i> RuleBodyItemParser<'i, String, &'static str> for Declarations {
    fn parse_declarations(&self) -> bool {
        true
    }
    fn parse_qualified(&self) -> bool {
        false
    }
}
impl<'i> DeclarationParser<'i> for Declarations {
    type Declaration = String;
    type Error = &'static str;
    fn parse_value<'t>(
        &mut self,
        name: CowRcStr<'i>,
        input: &mut Parser<'i, 't>,
        _: &ParserState,
    ) -> Result<String, ParseError<'i, Self::Error>> {
        let name = name.to_ascii_lowercase();
        if !matches!(
            name.as_str(),
            "color"
                | "background"
                | "background-color"
                | "border-color"
                | "border-width"
                | "border-style"
                | "border-radius"
                | "outline-color"
                | "outline-width"
                | "outline-style"
                | "font-family"
                | "font-size"
                | "font-weight"
                | "font-style"
                | "line-height"
                | "letter-spacing"
                | "text-decoration"
                | "text-decoration-color"
                | "text-transform"
                | "text-align"
                | "padding"
                | "padding-inline"
                | "padding-block"
                | "gap"
                | "row-gap"
                | "column-gap"
        ) {
            return Err(input.new_custom_error("Unsupported CSS property. Use visual color, typography, border or spacing properties."));
        }
        let mut value = String::new();
        values(input, &mut value, 0)?;
        if value.trim().is_empty() {
            return Err(input.new_custom_error("A CSS value is required."));
        }
        Ok(format!("{name}:{value}"))
    }
}
fn values<'i>(
    input: &mut Parser<'i, '_>,
    output: &mut String,
    depth: usize,
) -> Result<(), ParseError<'i, &'static str>> {
    if depth > 8 {
        return Err(input.new_custom_error("CSS functions are nested too deeply."));
    }
    loop {
        let token = match input.next_including_whitespace_and_comments() {
            Ok(token) => token.clone(),
            Err(error) if matches!(error.kind, BasicParseErrorKind::EndOfInput) => return Ok(()),
            Err(error) => return Err(error.into()),
        };
        match &token {
            Token::Function(name) => {
                if !matches!(
                    name.to_ascii_lowercase().as_str(),
                    "rgb"
                        | "rgba"
                        | "hsl"
                        | "hsla"
                        | "oklch"
                        | "oklab"
                        | "lab"
                        | "lch"
                        | "color-mix"
                        | "calc"
                        | "min"
                        | "max"
                        | "clamp"
                        | "var"
                        | "linear-gradient"
                        | "radial-gradient"
                        | "conic-gradient"
                        | "repeating-linear-gradient"
                        | "repeating-radial-gradient"
                ) {
                    return Err(input.new_custom_error(
                        "External resources and this CSS function are not supported.",
                    ));
                }
                token.to_css(output).expect("String writer");
                input.parse_nested_block(|nested| values(nested, output, depth + 1))?;
                output.push(')');
            }
            Token::Ident(_)
            | Token::Hash(_)
            | Token::IDHash(_)
            | Token::QuotedString(_)
            | Token::Comma
            | Token::Delim('+' | '-' | '*' | '/') => token.to_css(output).expect("String writer"),
            Token::Number { value, .. }
            | Token::Percentage {
                unit_value: value, ..
            }
            | Token::Dimension { value, .. }
                if value.is_finite() && value.abs() <= 4096.0 =>
            {
                token.to_css(output).expect("String writer")
            }
            Token::WhiteSpace(_) | Token::Comment(_) => output.push(' '),
            _ => return Err(input.new_custom_error("Unsafe or unsupported CSS value.")),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn appearance_rules_cannot_escape_the_owner_or_load_resources() {
        assert_eq!(
            stylesheet(
                "body { color: red } button:hover,.tag {border-radius:4px}",
                Scope::Mailbox
            )
            .unwrap(),
            ".mw-custom-mail{color: red ;}.mw-custom-mail :is(button:hover),.mw-custom-mail :is(.tag){border-radius:4px;}"
        );
        let siblings = stylesheet("body + .admin{color:red}", Scope::Mailbox).unwrap();
        assert!(siblings.starts_with(".mw-custom-mail :is(body + .admin)"));
        assert!(!siblings.starts_with(".mw-custom-mail +"));
        assert!(
            stylesheet("body{color:blue}", Scope::Admin)
                .unwrap()
                .starts_with(".mw-custom-admin{")
        );
        for css in [
            "@import 'https://evil.test/x';",
            "@font-face{font-family:x;src:url(https://evil.test)}",
            "a{background:url(https://evil.test)}",
            "a{background:u\\72l('https://evil.test')}",
            "a{background:image-set('https://evil.test')}",
            "a{position:fixed}",
            "a{color:var(--x,url(https://evil.test))}",
            "a{color:red!important}",
            "[data-secret]{color:red}",
            "a:has(b){color:red}",
            "a{--x:url(https://evil.test)}",
        ] {
            assert!(stylesheet(css, Scope::Mailbox).is_err(), "{css}");
        }
    }
}
