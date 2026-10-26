//! Administrator and user stylesheets (t32-ext §P, DA24; worker f16).
//!
//! [`sanitize_custom_css`] turns a stylesheet someone typed into the admin
//! Branding screen or Settings → Customisation into one that can only change how
//! the scoped part of the page looks. It never copies source text: every kept
//! selector, property and value is rebuilt from `cssparser` tokens, so escapes
//! (`ex\70 ression`), comments inside tokens (`exp/**/ression`) and odd spacing
//! are decoded before the policy sees them and cannot reach the output.
//!
//! What survives, for every profile:
//! * qualified rules whose selectors use type, class and id selectors, `*`,
//!   the four combinators, presence-only attribute selectors (`[open]`) and a
//!   short list of pseudo-classes; each selector is prefixed with the scope class
//!   ([`CssScope::class`]), and `html`, `body` and `:root` map onto it;
//! * `@media` blocks (one level), whose rules follow the same policy;
//! * declarations whose property is on the profile's list and whose value is
//!   made of keywords, numbers, colours and a short list of functions.
//!
//! What is refused (dropped with a reason, see [`Dropped`]): every other at-rule
//! (`@import`, `@font-face`, `@charset`, `@namespace`, `@keyframes`, …), `url()`
//! in any spelling, strings outside `font-family` and `asset()`, `expression()`,
//! `behavior`, `-moz-binding`, attribute selectors that match a value (`[x^=…]`,
//! `[x="…"]`; they leak attribute values through which rules apply), `:has()`,
//! custom properties, and every property that moves, hides or layers content
//! (`position`, `z-index`, `transform`, `inset`, `display`, `visibility`,
//! `clip-path`, `content`, …).
//!
//! Images come only from the server's own asset store: `asset("<id>")` becomes
//! `url("<prefix><id>")` when `<id>` is one of the assets the caller says the
//! scope owns. The result is served as a same-origin stylesheet, so the page's
//! `style-src 'self'` stays as it is.
//!
//! Native only: the browser never sanitises custom CSS, and keeping this module
//! out of the wasm guest leaves the committed guest bytes unchanged.

use std::collections::BTreeSet;
use std::fmt::Write as _;

use cssparser::{
    AtRuleParser, BasicParseErrorKind, CowRcStr, DeclarationParser, ParseError, ParseErrorKind,
    Parser, ParserInput, ParserState, QualifiedRuleParser, RuleBodyItemParser, RuleBodyParser,
    SourceLocation, StyleSheetParser, ToCss, Token, serialize_identifier, serialize_string,
};

/// Largest stylesheet accepted, in bytes of source.
pub const MAX_SOURCE_BYTES: usize = 16 * 1024;
/// Most qualified rules accepted (rules inside `@media` count).
pub const MAX_RULES: usize = 200;
/// Most compound selectors in one selector (`a b c d`).
pub const MAX_SELECTOR_DEPTH: usize = 4;
/// Longest kept selector list, in characters, after rebuilding.
const MAX_SELECTOR_CHARS: usize = 512;
/// Deepest function nesting in a value (`calc(min(…))`).
const MAX_VALUE_NESTING: usize = 4;

/// Which property list applies.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CssProfile {
    /// The sign-in page: colours, backgrounds, borders, radius, the allowed fonts,
    /// font weight, letter spacing, and bounded padding and margin.
    Login,
    /// The app, strict: [`CssProfile::Login`] plus gap and opacity.
    AppStrict,
    /// The app, standard: [`CssProfile::AppStrict`] plus transitions, filters and
    /// shadows.
    AppStandard,
}

impl CssProfile {
    /// The wire name (`login`, `app-strict`, `app-standard`).
    pub fn as_str(self) -> &'static str {
        match self {
            CssProfile::Login => "login",
            CssProfile::AppStrict => "app-strict",
            CssProfile::AppStandard => "app-standard",
        }
    }

    /// Parse a wire name.
    pub fn parse(s: &str) -> Option<Self> {
        match s {
            "login" => Some(CssProfile::Login),
            "app-strict" | "strict" => Some(CssProfile::AppStrict),
            "app-standard" | "standard" => Some(CssProfile::AppStandard),
            _ => None,
        }
    }
}

/// Which container the rules are confined to.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CssScope {
    /// `.mw-scope-login`, the sign-in page's styled container.
    Login,
    /// `.mw-scope-app`, the webmail's styled container.
    App,
}

impl CssScope {
    /// The class every selector is prefixed with.
    pub fn class(self) -> &'static str {
        match self {
            CssScope::Login => "mw-scope-login",
            CssScope::App => "mw-scope-app",
        }
    }
}

/// What the caller allows besides the policy itself.
#[derive(Debug, Clone, Default)]
pub struct CssContext {
    /// Asset ids `asset("<id>")` may name (the scope's own assets).
    pub assets: BTreeSet<String>,
    /// Written before an asset id to form its URL (`../assets/`).
    pub asset_url_prefix: String,
    /// An extra font family name a branded font is registered under, allowed in
    /// `font-family` next to the built-in set.
    pub brand_font_family: Option<String>,
}

/// One thing the sanitiser removed, for the editor to show next to the line.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Dropped {
    /// 1-based line in the source.
    pub line: u32,
    /// 1-based column in the source.
    pub column: u32,
    /// A stable code (`url`, `at-rule`, `property`, `selector-attribute`, …).
    pub reason: &'static str,
    /// What was refused (`@import`, `position`, `[href^=…]`), at most 80 chars.
    pub detail: String,
}

/// The kept stylesheet and what was removed.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct SanitizedCss {
    pub css: String,
    pub dropped: Vec<Dropped>,
    /// Qualified rules kept.
    pub rules: usize,
}

/// A stylesheet refused as a whole.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CustomCssError {
    /// Longer than [`MAX_SOURCE_BYTES`].
    TooLarge { max: usize, got: usize },
    /// More than [`MAX_RULES`] qualified rules.
    TooManyRules { max: usize },
}

impl CustomCssError {
    /// A stable code for the API.
    pub fn code(&self) -> &'static str {
        match self {
            CustomCssError::TooLarge { .. } => "cssTooLarge",
            CustomCssError::TooManyRules { .. } => "cssTooManyRules",
        }
    }
}

impl std::fmt::Display for CustomCssError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CustomCssError::TooLarge { max, got } => {
                write!(f, "the stylesheet is {got} bytes; at most {max} are accepted")
            }
            CustomCssError::TooManyRules { max } => {
                write!(f, "the stylesheet has more than {max} rules")
            }
        }
    }
}

impl std::error::Error for CustomCssError {}

/// Sanitise `src` for `profile`, confined to `scope`. See the module docs.
pub fn sanitize_custom_css(
    src: &str,
    profile: CssProfile,
    scope: CssScope,
    ctx: &CssContext,
) -> Result<SanitizedCss, CustomCssError> {
    if src.len() > MAX_SOURCE_BYTES {
        return Err(CustomCssError::TooLarge {
            max: MAX_SOURCE_BYTES,
            got: src.len(),
        });
    }
    let mut state = State {
        profile,
        scope,
        ctx,
        dropped: Vec::new(),
        rules: 0,
        in_media: false,
    };
    let mut input = ParserInput::new(src);
    let mut parser = Parser::new(&mut input);
    let css = rules(&mut parser, &mut state);
    if state.rules > MAX_RULES {
        return Err(CustomCssError::TooManyRules { max: MAX_RULES });
    }
    Ok(SanitizedCss {
        css,
        dropped: state.dropped,
        rules: state.rules,
    })
}

// ── shared state ─────────────────────────────────────────────────────────────

/// A refusal raised while parsing: reason code and what was refused.
#[derive(Debug, Clone)]
struct Refuse {
    reason: &'static str,
    detail: String,
}

fn refuse(reason: &'static str, detail: impl Into<String>) -> Refuse {
    let mut detail = detail.into();
    if detail.chars().count() > 80 {
        detail = detail.chars().take(79).collect::<String>() + "…";
    }
    Refuse { reason, detail }
}

type PResult<'i, T> = Result<T, ParseError<'i, Refuse>>;

fn fail<'i, T>(input: &Parser<'i, '_>, r: Refuse) -> PResult<'i, T> {
    Err(input.new_custom_error(r))
}

struct State<'c> {
    profile: CssProfile,
    scope: CssScope,
    ctx: &'c CssContext,
    dropped: Vec<Dropped>,
    rules: usize,
    in_media: bool,
}

impl State<'_> {
    fn record(&mut self, loc: SourceLocation, r: Refuse) {
        self.dropped.push(Dropped {
            line: loc.line + 1,
            column: loc.column,
            reason: r.reason,
            detail: r.detail,
        });
    }

    fn record_error(&mut self, e: ParseError<'_, Refuse>, slice: &str) {
        let r = match e.kind {
            ParseErrorKind::Custom(r) => r,
            ParseErrorKind::Basic(BasicParseErrorKind::AtRuleInvalid(name)) => {
                refuse("at-rule", format!("@{name}"))
            }
            ParseErrorKind::Basic(_) => refuse("syntax", slice.trim()),
        };
        self.record(e.location, r);
    }
}

/// Parse a rule list, keeping what the policy allows.
fn rules(parser: &mut Parser, state: &mut State) -> String {
    let mut out = String::new();
    let mut errors = Vec::new();
    {
        let iter = StyleSheetParser::new(parser, &mut *state);
        for item in iter {
            match item {
                Ok(rule) => out.push_str(&rule),
                Err((e, slice)) => errors.push((e, slice.to_string())),
            }
        }
    }
    for (e, slice) in errors {
        state.record_error(e, &slice);
    }
    out
}

// ── rules ────────────────────────────────────────────────────────────────────

impl<'i> QualifiedRuleParser<'i> for State<'_> {
    type Prelude = String;
    type QualifiedRule = String;
    type Error = Refuse;

    fn parse_prelude<'t>(&mut self, input: &mut Parser<'i, 't>) -> PResult<'i, String> {
        let scope = self.scope.class();
        selector_list(input, scope)
    }

    fn parse_block<'t>(
        &mut self,
        selectors: String,
        _start: &ParserState,
        input: &mut Parser<'i, 't>,
    ) -> PResult<'i, String> {
        self.rules += 1;
        let mut decls = String::new();
        let mut errors = Vec::new();
        {
            let mut sink = Decls {
                profile: self.profile,
                ctx: self.ctx,
            };
            let iter = RuleBodyParser::new(input, &mut sink);
            for item in iter {
                match item {
                    Ok((prop, value)) => {
                        decls.push_str(&prop);
                        decls.push(':');
                        decls.push_str(&value);
                        decls.push(';');
                    }
                    Err((e, slice)) => errors.push((e, slice.to_string())),
                }
            }
        }
        for (e, slice) in errors {
            self.record_error(e, &slice);
        }
        if decls.is_empty() {
            return Ok(String::new());
        }
        decls.pop();
        Ok(format!("{selectors}{{{decls}}}"))
    }
}

impl<'i> AtRuleParser<'i> for State<'_> {
    type Prelude = String;
    type AtRule = String;
    type Error = Refuse;

    fn parse_prelude<'t>(
        &mut self,
        name: CowRcStr<'i>,
        input: &mut Parser<'i, 't>,
    ) -> PResult<'i, String> {
        if !name.eq_ignore_ascii_case("media") {
            return fail(input, refuse("at-rule", format!("@{name}")));
        }
        if self.in_media {
            return fail(input, refuse("at-rule-nested", "@media inside @media"));
        }
        media_query(input)
    }

    fn rule_without_block(&mut self, _prelude: String, _start: &ParserState) -> Result<String, ()> {
        Ok(String::new())
    }

    fn parse_block<'t>(
        &mut self,
        prelude: String,
        _start: &ParserState,
        input: &mut Parser<'i, 't>,
    ) -> PResult<'i, String> {
        self.in_media = true;
        let inner = rules(input, self);
        self.in_media = false;
        if inner.is_empty() {
            return Ok(String::new());
        }
        Ok(format!("@media {prelude}{{{inner}}}"))
    }
}

/// Rebuild a media query from allowed tokens.
fn media_query<'i>(input: &mut Parser<'i, '_>) -> PResult<'i, String> {
    let mut out = String::new();
    media_tokens(input, &mut out, 0)?;
    let out = out.trim().to_string();
    if out.is_empty() {
        return fail(input, refuse("at-rule", "@media without a query"));
    }
    Ok(out)
}

fn media_tokens<'i>(input: &mut Parser<'i, '_>, out: &mut String, depth: usize) -> PResult<'i, ()> {
    loop {
        let token = match input.next_including_whitespace() {
            Ok(t) => t.clone(),
            Err(_) => return Ok(()),
        };
        match token {
            Token::WhiteSpace(_) => push_space(out),
            Token::Ident(ref s) => serialize_identifier(s, out).map_err(|_| syntax(input))?,
            Token::Number { .. } | Token::Colon | Token::Comma => {
                token.to_css(out).map_err(|_| syntax(input))?
            }
            Token::Dimension { ref unit, .. } if allowed_unit(unit) => {
                token.to_css(out).map_err(|_| syntax(input))?
            }
            Token::Delim(c) if matches!(c, '<' | '>' | '=' | '/') => out.push(c),
            Token::ParenthesisBlock if depth < 2 => {
                out.push('(');
                input.parse_nested_block(|nested| media_tokens(nested, out, depth + 1))?;
                out.push(')');
            }
            other => {
                return fail(input, refuse("at-rule", format!("@media {}", token_text(&other))));
            }
        }
    }
}

fn syntax<'i>(input: &Parser<'i, '_>) -> ParseError<'i, Refuse> {
    input.new_custom_error(refuse("syntax", "unreadable token"))
}

fn push_space(out: &mut String) {
    if !out.is_empty() && !out.ends_with(' ') {
        out.push(' ');
    }
}

fn token_text(t: &Token) -> String {
    let mut s = String::new();
    let _ = t.to_css(&mut s);
    s
}

// ── selectors ────────────────────────────────────────────────────────────────

/// Pseudo-classes a selector may use.
const PSEUDO_CLASSES: &[&str] = &[
    "hover",
    "focus",
    "focus-visible",
    "focus-within",
    "active",
    "disabled",
    "enabled",
    "checked",
    "empty",
    "first-child",
    "last-child",
    "only-child",
    "first-of-type",
    "last-of-type",
    "placeholder-shown",
    "root",
];

/// Pseudo-elements a selector may use (no `::before`/`::after`: `content` is
/// never allowed, so they could only add boxes).
const PSEUDO_ELEMENTS: &[&str] = &["placeholder", "selection"];

/// Functional pseudo-classes a selector may use.
const PSEUDO_FUNCTIONS: &[&str] = &["not", "nth-child", "nth-last-child", "nth-of-type"];

/// Parse and rebuild a selector list, prefixing each selector with `.scope`.
fn selector_list<'i>(input: &mut Parser<'i, '_>, scope: &str) -> PResult<'i, String> {
    let mut out = String::new();
    loop {
        let one = selector(input, scope)?;
        if !out.is_empty() {
            out.push(',');
        }
        out.push_str(&one);
        match input.next() {
            Ok(Token::Comma) => continue,
            Err(_) => break,
            Ok(other) => {
                let t = other.clone();
                return fail(input, refuse("selector", token_text(&t)));
            }
        }
    }
    if out.len() > MAX_SELECTOR_CHARS {
        return fail(input, refuse("selector-too-long", out));
    }
    Ok(out)
}

/// One complex selector up to a top-level comma or the end.
fn selector<'i>(input: &mut Parser<'i, '_>, scope: &str) -> PResult<'i, String> {
    // Compounds separated by combinators; each compound is a list of parts.
    let mut compounds: Vec<(Option<char>, String)> = Vec::new();
    let mut current = String::new();
    let mut pending: Option<char> = None;
    let mut saw_space = false;
    loop {
        let state = input.state();
        let token = match input.next_including_whitespace() {
            Ok(t) => t.clone(),
            Err(_) => break,
        };
        match token {
            Token::Comma => {
                input.reset(&state);
                break;
            }
            Token::WhiteSpace(_) => {
                saw_space = true;
                continue;
            }
            Token::Delim(c @ ('>' | '+' | '~')) => {
                if current.is_empty() && compounds.is_empty() {
                    return fail(input, refuse("selector", format!("leading {c}")));
                }
                if !current.is_empty() {
                    compounds.push((pending.take(), std::mem::take(&mut current)));
                }
                pending = Some(c);
                saw_space = false;
                continue;
            }
            _ => {}
        }
        if saw_space && !current.is_empty() {
            compounds.push((pending.take(), std::mem::take(&mut current)));
            pending = Some(' ');
        } else if saw_space && current.is_empty() && pending.is_none() && !compounds.is_empty() {
            pending = Some(' ');
        }
        saw_space = false;
        simple_selector(input, token, &mut current)?;
    }
    if !current.is_empty() {
        compounds.push((pending.take(), current));
    } else if pending.is_some() {
        return fail(input, refuse("selector", "trailing combinator"));
    }
    if compounds.is_empty() {
        return fail(input, refuse("selector", "empty selector"));
    }
    if compounds.len() > MAX_SELECTOR_DEPTH {
        return fail(
            input,
            refuse(
                "selector-depth",
                format!("{} levels; at most {MAX_SELECTOR_DEPTH}", compounds.len()),
            ),
        );
    }
    // `html`, `body` and `:root` at the start are the scope container itself.
    let mut out = String::new();
    let (_, first) = &compounds[0];
    let rest_of_first = ["html", "body", ":root"].iter().find_map(|kw| {
        first
            .strip_prefix(kw)
            .filter(|after| after.is_empty() || after.starts_with(['.', '#', ':', '[']))
    });
    match rest_of_first {
        Some(after) => {
            let _ = write!(out, ".{scope}{after}");
        }
        None => {
            let _ = write!(out, ".{scope} {first}");
        }
    }
    for (comb, compound) in &compounds[1..] {
        match comb {
            Some(' ') | None => out.push(' '),
            Some(c) => {
                let _ = write!(out, " {c} ");
            }
        }
        out.push_str(compound);
    }
    Ok(out)
}

/// Append one simple selector (starting with `token`) to `out`.
fn simple_selector<'i>(
    input: &mut Parser<'i, '_>,
    token: Token<'i>,
    out: &mut String,
) -> PResult<'i, ()> {
    match token {
        Token::Ident(ref name) => {
            if !out.is_empty() {
                return fail(input, refuse("selector", format!("misplaced type {name}")));
            }
            let lower = name.to_ascii_lowercase();
            if !is_plain_name(&lower) {
                return fail(input, refuse("selector", lower));
            }
            out.push_str(&lower);
        }
        Token::Delim('*') => {
            if !out.is_empty() {
                return fail(input, refuse("selector", "misplaced *"));
            }
            out.push('*');
        }
        Token::Delim('.') => match input.next_including_whitespace() {
            Ok(Token::Ident(name)) => {
                let name = name.clone();
                out.push('.');
                serialize_identifier(&name, out).map_err(|_| syntax(input))?;
            }
            _ => return fail(input, refuse("selector", ". without a class name")),
        },
        Token::IDHash(ref name) => {
            out.push('#');
            serialize_identifier(name, out).map_err(|_| syntax(input))?;
        }
        Token::Hash(ref name) => {
            return fail(input, refuse("selector", format!("#{name}")));
        }
        Token::SquareBracketBlock => {
            let attr = input.parse_nested_block(|nested| {
                let name = match nested.next() {
                    Ok(Token::Ident(n)) => n.to_ascii_lowercase(),
                    _ => {
                        return fail(
                            nested,
                            refuse("selector-attribute", "attribute selector without a name"),
                        );
                    }
                };
                if nested.is_exhausted() {
                    Ok(name)
                } else {
                    // `[x=…]`, `[x^=…]`, `[x*=…]`, `[x$=…]`, `[x~=…]`, `[x|=…]`.
                    fail(nested, refuse("selector-attribute", format!("[{name}…]")))
                }
            })?;
            if !is_plain_name(&attr) {
                return fail(input, refuse("selector-attribute", format!("[{attr}]")));
            }
            let _ = write!(out, "[{attr}]");
        }
        Token::Colon => {
            let next = input.next_including_whitespace().map(|t| t.clone());
            match next {
                Ok(Token::Ident(name)) => {
                    let lower = name.to_ascii_lowercase();
                    if !PSEUDO_CLASSES.contains(&lower.as_str()) {
                        return fail(input, refuse("selector-pseudo", format!(":{lower}")));
                    }
                    let _ = write!(out, ":{lower}");
                }
                Ok(Token::Colon) => match input.next_including_whitespace() {
                    Ok(Token::Ident(name)) => {
                        let lower = name.to_ascii_lowercase();
                        if !PSEUDO_ELEMENTS.contains(&lower.as_str()) {
                            return fail(input, refuse("selector-pseudo", format!("::{lower}")));
                        }
                        let _ = write!(out, "::{lower}");
                    }
                    _ => return fail(input, refuse("selector-pseudo", "::")),
                },
                Ok(Token::Function(name)) => {
                    let lower = name.to_ascii_lowercase();
                    if lower == "has" {
                        return fail(input, refuse("selector-has", ":has()"));
                    }
                    if !PSEUDO_FUNCTIONS.contains(&lower.as_str()) {
                        return fail(input, refuse("selector-pseudo", format!(":{lower}()")));
                    }
                    let inner = input.parse_nested_block(|nested| {
                        if lower == "not" {
                            not_argument(nested)
                        } else {
                            nth_argument(nested)
                        }
                    })?;
                    let _ = write!(out, ":{lower}({inner})");
                }
                _ => return fail(input, refuse("selector-pseudo", ":")),
            }
        }
        other => return fail(input, refuse("selector", token_text(&other))),
    }
    Ok(())
}

/// `:not()` takes compound selectors without combinators.
fn not_argument<'i>(input: &mut Parser<'i, '_>) -> PResult<'i, String> {
    let mut out = String::new();
    let mut current = String::new();
    loop {
        let token = match input.next() {
            Ok(t) => t.clone(),
            Err(_) => break,
        };
        if let Token::Comma = token {
            if current.is_empty() {
                return fail(input, refuse("selector", ":not() with an empty part"));
            }
            out.push_str(&current);
            out.push(',');
            current.clear();
            continue;
        }
        if matches!(token, Token::Colon) {
            // Nested functional pseudo-classes inside :not() are refused, so a
            // `:not(:has(…))` probe cannot slip through.
            let state = input.state();
            if let Ok(Token::Function(name)) = input.next_including_whitespace() {
                let name = name.to_ascii_lowercase();
                return fail(
                    input,
                    refuse(
                        if name == "has" {
                            "selector-has"
                        } else {
                            "selector-pseudo"
                        },
                        format!(":{name}() inside :not()"),
                    ),
                );
            }
            input.reset(&state);
        }
        simple_selector(input, token, &mut current)?;
    }
    if current.is_empty() {
        return fail(input, refuse("selector", ":not() with an empty part"));
    }
    out.push_str(&current);
    Ok(out)
}

/// `an+b`, `odd` or `even`, rebuilt.
fn nth_argument<'i>(input: &mut Parser<'i, '_>) -> PResult<'i, String> {
    let (a, b) = cssparser::parse_nth(input)
        .map_err(|_| input.new_custom_error(refuse("selector-pseudo", "bad an+b")))?;
    if !input.is_exhausted() {
        return fail(input, refuse("selector-pseudo", "an+b followed by more"));
    }
    Ok(match (a, b) {
        (0, b) => format!("{b}"),
        (a, 0) => format!("{a}n"),
        (a, b) if b > 0 => format!("{a}n+{b}"),
        (a, b) => format!("{a}n{b}"),
    })
}

fn is_plain_name(s: &str) -> bool {
    !s.is_empty()
        && s.len() <= 64
        && s.starts_with(|c: char| c.is_ascii_lowercase())
        && s.chars().all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-')
}

// ── declarations ─────────────────────────────────────────────────────────────

/// What a property's value may contain beyond the shared value grammar.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ValueRule {
    /// Keywords, numbers, colours and the shared functions.
    General,
    /// [`ValueRule::General`] plus gradients and `asset()`.
    Image,
    /// Font family names from the allowed set.
    FontFamily,
    /// Non-negative lengths of at most 64px / 4em / 10% (padding, margin).
    Spacing,
    /// Lengths of at most 16px / 1em either way (letter spacing).
    SmallLength,
    /// A number or percentage of at least 0.2 / 20%.
    Opacity,
}

/// The property's value rule under `profile`, or `None` when the property is not
/// allowed there.
fn property_rule(profile: CssProfile, name: &str) -> Option<ValueRule> {
    use ValueRule::*;
    let login = match name {
        "color" | "background-color" | "accent-color" | "caret-color" | "outline-color"
        | "text-decoration-color" | "background-position" | "background-size"
        | "background-repeat" | "font-weight" | "border-radius" | "border-top-left-radius"
        | "border-top-right-radius" | "border-bottom-left-radius"
        | "border-bottom-right-radius" | "border-start-start-radius"
        | "border-start-end-radius" | "border-end-start-radius" | "border-end-end-radius"
        | "border" | "border-color" | "border-style" | "border-width" => Some(General),
        "background" | "background-image" => Some(Image),
        "font-family" => Some(FontFamily),
        "letter-spacing" => Some(SmallLength),
        "padding" | "padding-top" | "padding-right" | "padding-bottom" | "padding-left"
        | "padding-block" | "padding-inline" | "padding-block-start" | "padding-block-end"
        | "padding-inline-start" | "padding-inline-end" | "margin" | "margin-top"
        | "margin-right" | "margin-bottom" | "margin-left" | "margin-block" | "margin-inline"
        | "margin-block-start" | "margin-block-end" | "margin-inline-start"
        | "margin-inline-end" => Some(Spacing),
        _ if is_border_side(name) => Some(General),
        _ => None,
    };
    if login.is_some() {
        return login;
    }
    let strict = match name {
        "gap" | "row-gap" | "column-gap" => Some(Spacing),
        "opacity" => Some(Opacity),
        _ => None,
    };
    if profile == CssProfile::Login {
        return None;
    }
    if strict.is_some() {
        return strict;
    }
    if profile == CssProfile::AppStrict {
        return None;
    }
    match name {
        "transition" | "transition-property" | "transition-duration"
        | "transition-timing-function" | "transition-delay" | "filter" | "backdrop-filter"
        | "box-shadow" | "text-shadow" => Some(General),
        _ => None,
    }
}

/// `border-{top,right,bottom,left,block,inline,block-start,…}[-{color,style,width}]`.
fn is_border_side(name: &str) -> bool {
    let Some(rest) = name.strip_prefix("border-") else {
        return false;
    };
    let side = ["color", "style", "width"]
        .iter()
        .find_map(|p| rest.strip_suffix(&format!("-{p}")[..]))
        .unwrap_or(rest);
    matches!(
        side,
        "top"
            | "right"
            | "bottom"
            | "left"
            | "block"
            | "inline"
            | "block-start"
            | "block-end"
            | "inline-start"
            | "inline-end"
    )
}

/// Functions any value may use.
const FUNCTIONS: &[&str] = &[
    "rgb", "rgba", "hsl", "hsla", "hwb", "lab", "lch", "oklab", "oklch", "color-mix", "var",
    "calc", "min", "max", "clamp", "cubic-bezier", "steps", "blur", "brightness", "contrast",
    "grayscale", "hue-rotate", "invert", "saturate", "sepia", "drop-shadow",
];

/// Functions only image properties may use.
const IMAGE_FUNCTIONS: &[&str] = &[
    "linear-gradient",
    "radial-gradient",
    "conic-gradient",
    "repeating-linear-gradient",
    "repeating-radial-gradient",
    "repeating-conic-gradient",
];

/// Length units a value may use.
fn allowed_unit(unit: &str) -> bool {
    matches!(
        unit.to_ascii_lowercase().as_str(),
        "px" | "em"
            | "rem"
            | "ch"
            | "ex"
            | "vh"
            | "vw"
            | "vmin"
            | "vmax"
            | "pt"
            | "s"
            | "ms"
            | "deg"
            | "rad"
            | "turn"
            | "grad"
            | "fr"
            | "dppx"
    )
}

/// Font families allowed in `font-family`, lower case.
const FONT_FAMILIES: &[&str] = &[
    "jost",
    "system-ui",
    "sans-serif",
    "serif",
    "monospace",
    "ui-sans-serif",
    "ui-serif",
    "ui-monospace",
    "inherit",
];

struct Decls<'c> {
    profile: CssProfile,
    ctx: &'c CssContext,
}

impl<'i> DeclarationParser<'i> for Decls<'_> {
    type Declaration = (String, String);
    type Error = Refuse;

    fn parse_value<'t>(
        &mut self,
        name: CowRcStr<'i>,
        input: &mut Parser<'i, 't>,
        _start: &ParserState,
    ) -> PResult<'i, (String, String)> {
        let property = name.to_ascii_lowercase();
        if property.starts_with("--") {
            return fail(input, refuse("custom-property", property));
        }
        let Some(rule) = property_rule(self.profile, &property) else {
            return fail(input, refuse("property", property));
        };
        let mut value = String::new();
        let important = match rule {
            ValueRule::FontFamily => font_family(input, self.ctx, &mut value)?,
            _ => {
                let mut v = ValueWriter {
                    rule,
                    ctx: self.ctx,
                    property: &property,
                };
                v.write(input, &mut value, 0)?
            }
        };
        let value = value.trim().to_string();
        if value.is_empty() {
            return fail(input, refuse("value", format!("{property}: (empty)")));
        }
        let value = if important {
            format!("{value} !important")
        } else {
            value
        };
        Ok((property, value))
    }
}

impl<'i> AtRuleParser<'i> for Decls<'_> {
    type Prelude = ();
    type AtRule = (String, String);
    type Error = Refuse;

    fn parse_prelude<'t>(
        &mut self,
        name: CowRcStr<'i>,
        input: &mut Parser<'i, 't>,
    ) -> PResult<'i, ()> {
        fail(input, refuse("at-rule", format!("@{name}")))
    }
}

impl<'i> QualifiedRuleParser<'i> for Decls<'_> {
    type Prelude = ();
    type QualifiedRule = (String, String);
    type Error = Refuse;

    fn parse_prelude<'t>(&mut self, input: &mut Parser<'i, 't>) -> PResult<'i, ()> {
        fail(input, refuse("nested-rule", "rules inside a rule"))
    }
}

impl<'i> RuleBodyItemParser<'i, (String, String), Refuse> for Decls<'_> {
    fn parse_declarations(&self) -> bool {
        true
    }
    fn parse_qualified(&self) -> bool {
        false
    }
}

/// Consume a trailing `!important` if one follows; refuse anything else.
fn bang_important<'i>(input: &mut Parser<'i, '_>) -> PResult<'i, ()> {
    match input.next() {
        Ok(Token::Ident(s)) if s.eq_ignore_ascii_case("important") => {}
        _ => return fail(input, refuse("value", "! not followed by important")),
    }
    if !input.is_exhausted() {
        return fail(input, refuse("value", "tokens after !important"));
    }
    Ok(())
}

/// Rebuild a `font-family` list; every family must be in the allowed set.
/// Returns whether `!important` closed it.
fn font_family<'i>(input: &mut Parser<'i, '_>, ctx: &CssContext, out: &mut String) -> PResult<'i, bool> {
    let brand = ctx.brand_font_family.as_deref().map(str::to_ascii_lowercase);
    let mut name = String::new();
    let mut families: Vec<String> = Vec::new();
    let mut important = false;
    loop {
        let token = match input.next() {
            Ok(t) => t.clone(),
            Err(_) => break,
        };
        match token {
            Token::Ident(ref s) => {
                if !name.is_empty() {
                    name.push(' ');
                }
                name.push_str(&s.to_ascii_lowercase());
            }
            Token::QuotedString(ref s) if name.is_empty() => name = s.to_ascii_lowercase(),
            Token::Comma => families.push(std::mem::take(&mut name)),
            Token::Delim('!') => {
                bang_important(input)?;
                important = true;
                break;
            }
            other => return fail(input, refuse("font", token_text(&other))),
        }
    }
    families.push(name);
    for (i, family) in families.iter().enumerate() {
        let known = FONT_FAMILIES.contains(&family.as_str()) || brand.as_deref() == Some(family);
        if !known {
            return fail(input, refuse("font", family.clone()));
        }
        if i > 0 {
            out.push_str(", ");
        }
        if family.contains(' ') {
            serialize_string(family, out).map_err(|_| syntax(input))?;
        } else {
            serialize_identifier(family, out).map_err(|_| syntax(input))?;
        }
    }
    Ok(important)
}

struct ValueWriter<'a> {
    rule: ValueRule,
    ctx: &'a CssContext,
    property: &'a str,
}

impl ValueWriter<'_> {
    /// Rebuild a value from allowed tokens. Returns whether `!important` closed it.
    fn write<'i>(&mut self, input: &mut Parser<'i, '_>, out: &mut String, depth: usize) -> PResult<'i, bool> {
        if depth > MAX_VALUE_NESTING {
            return fail(input, refuse("value", "functions nested too deeply"));
        }
        loop {
            let token = match input.next_including_whitespace() {
                Ok(t) => t.clone(),
                Err(_) => return Ok(false),
            };
            match token {
                Token::WhiteSpace(_) => push_space(out),
                Token::Delim('!') if depth == 0 => {
                    bang_important(input)?;
                    return Ok(true);
                }
                Token::Ident(ref s) => {
                    let lower = s.to_ascii_lowercase();
                    if lower.contains("expression") || lower.starts_with("javascript") {
                        return fail(input, refuse("expression", lower));
                    }
                    if self.limited() && !matches!(lower.as_str(), "auto" | "inherit" | "initial" | "unset" | "normal") {
                        return fail(input, refuse("value", format!("{}: {lower}", self.property)));
                    }
                    serialize_identifier(s, out).map_err(|_| syntax(input))?;
                }
                Token::Number { value, .. } => {
                    self.check_number(input, value, None)?;
                    token.to_css(out).map_err(|_| syntax(input))?;
                }
                Token::Percentage { unit_value, .. } => {
                    self.check_number(input, unit_value * 100.0, Some("%"))?;
                    token.to_css(out).map_err(|_| syntax(input))?;
                }
                Token::Dimension { value, ref unit, .. } => {
                    if !allowed_unit(unit) {
                        return fail(input, refuse("value", format!("unit {unit}")));
                    }
                    let unit = unit.to_ascii_lowercase();
                    self.check_number(input, value, Some(&unit))?;
                    token.to_css(out).map_err(|_| syntax(input))?;
                }
                Token::Hash(ref h) | Token::IDHash(ref h) => {
                    let ok = matches!(h.len(), 3 | 4 | 6 | 8) && h.chars().all(|c| c.is_ascii_hexdigit());
                    if !ok || self.limited() {
                        return fail(input, refuse("value", format!("#{h}")));
                    }
                    out.push('#');
                    out.push_str(&h.to_ascii_lowercase());
                }
                Token::Comma => {
                    if out.ends_with(' ') {
                        out.pop();
                    }
                    out.push_str(", ");
                }
                Token::Delim(c @ ('/' | '+' | '-' | '*')) => {
                    if c != '/' && depth == 0 {
                        return fail(input, refuse("value", format!("{c} outside a function")));
                    }
                    push_space(out);
                    out.push(c);
                    out.push(' ');
                }
                Token::Function(ref name) => {
                    let lower = name.to_ascii_lowercase();
                    self.function(input, &lower, out, depth)?;
                }
                Token::ParenthesisBlock if depth > 0 => {
                    out.push('(');
                    input.parse_nested_block(|nested| self.write(nested, out, depth + 1).map(|_| ()))?;
                    out.push(')');
                }
                Token::UnquotedUrl(ref u) => {
                    return fail(input, refuse("url", format!("url({u})")));
                }
                Token::BadUrl(_) => return fail(input, refuse("url", "url(…)")),
                Token::QuotedString(ref s) => {
                    let low = s.to_ascii_lowercase();
                    let reason = if low.trim_start().starts_with("javascript:") {
                        "expression"
                    } else {
                        "string"
                    };
                    return fail(input, refuse(reason, format!("\"{s}\"")));
                }
                other => {
                    return fail(input, refuse("value", token_text(&other)));
                }
            }
        }
    }

    fn limited(&self) -> bool {
        matches!(
            self.rule,
            ValueRule::Spacing | ValueRule::SmallLength | ValueRule::Opacity
        )
    }

    fn function<'i>(&mut self, input: &mut Parser<'i, '_>, name: &str, out: &mut String, depth: usize) -> PResult<'i, ()> {
        if name == "url" || name == "image-set" || name == "-webkit-image-set" || name == "image" {
            return fail(input, refuse("url", format!("{name}(…)")));
        }
        if name.contains("expression") {
            return fail(input, refuse("expression", format!("{name}(…)")));
        }
        if name == "asset" {
            if self.rule != ValueRule::Image {
                return fail(input, refuse("asset", format!("asset() in {}", self.property)));
            }
            let id = input.parse_nested_block(|nested| {
                let id = match nested.next() {
                    Ok(Token::QuotedString(s)) => s.to_string(),
                    _ => return fail(nested, refuse("asset", "asset() needs a quoted id")),
                };
                if !nested.is_exhausted() {
                    return fail(nested, refuse("asset", "asset() takes one id"));
                }
                Ok(id)
            })?;
            let well_formed = !id.is_empty() && id.len() <= 64 && id.chars().all(|c| c.is_ascii_hexdigit());
            if !well_formed || !self.ctx.assets.contains(&id) {
                return fail(input, refuse("asset", format!("asset(\"{id}\") is not an asset of this scope")));
            }
            out.push_str("url(");
            serialize_string(&format!("{}{}", self.ctx.asset_url_prefix, id), out)
                .map_err(|_| syntax(input))?;
            out.push(')');
            return Ok(());
        }
        let allowed = FUNCTIONS.contains(&name)
            || (self.rule == ValueRule::Image && IMAGE_FUNCTIONS.contains(&name));
        if !allowed || self.limited() {
            return fail(input, refuse("function", format!("{name}()")));
        }
        out.push_str(name);
        out.push('(');
        if name == "var" {
            // `var(--mw-…)`: the name only, no fallback (a fallback could carry
            // anything the grammar below would not see as a value).
            let var = input.parse_nested_block(|nested| {
                let v = match nested.next() {
                    Ok(Token::Ident(s)) if s.starts_with("--") => s.to_string(),
                    _ => return fail(nested, refuse("value", "var() needs a --name")),
                };
                if !nested.is_exhausted() {
                    return fail(nested, refuse("value", "var() fallback"));
                }
                Ok(v)
            })?;
            serialize_identifier(&var, out).map_err(|_| syntax(input))?;
        } else {
            input.parse_nested_block(|nested| self.write(nested, out, depth + 1).map(|_| ()))?;
        }
        if out.ends_with(' ') {
            out.pop();
        }
        out.push(')');
        Ok(())
    }

    fn check_number<'i>(&self, input: &Parser<'i, '_>, value: f32, unit: Option<&str>) -> PResult<'i, ()> {
        if !value.is_finite() {
            return fail(input, refuse("value", "not a finite number"));
        }
        let detail = || format!("{}: {value}{}", self.property, unit.unwrap_or(""));
        match self.rule {
            ValueRule::Spacing => {
                let max = match unit {
                    None if value == 0.0 => return Ok(()),
                    Some("px") => 64.0,
                    Some("em" | "rem" | "ch") => 4.0,
                    Some("%") => 10.0,
                    _ => return fail(input, refuse("spacing", detail())),
                };
                if !(0.0..=max).contains(&value) {
                    return fail(input, refuse("spacing", detail()));
                }
            }
            ValueRule::SmallLength => {
                let max = match unit {
                    None if value == 0.0 => return Ok(()),
                    Some("px") => 16.0,
                    Some("em" | "rem") => 1.0,
                    _ => return fail(input, refuse("value", detail())),
                };
                if value.abs() > max {
                    return fail(input, refuse("value", detail()));
                }
            }
            ValueRule::Opacity => {
                let v = match unit {
                    None => value,
                    Some("%") => value / 100.0,
                    _ => return fail(input, refuse("value", detail())),
                };
                if !(0.2..=1.0).contains(&v) {
                    return fail(input, refuse("opacity", detail()));
                }
            }
            ValueRule::General | ValueRule::Image | ValueRule::FontFamily => {}
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ctx() -> CssContext {
        CssContext {
            assets: ["0123456789abcdef0123456789abcdef".to_string()].into_iter().collect(),
            asset_url_prefix: "../assets/".into(),
            brand_font_family: None,
        }
    }

    fn app(src: &str) -> SanitizedCss {
        sanitize_custom_css(src, CssProfile::AppStandard, CssScope::App, &ctx()).unwrap()
    }

    fn login(src: &str) -> SanitizedCss {
        sanitize_custom_css(src, CssProfile::Login, CssScope::Login, &ctx()).unwrap()
    }

    #[test]
    fn keeps_and_scopes_an_ordinary_rule() {
        let out = app(".panel h2 { color: #AABBCC; padding: 4px 8px }");
        assert_eq!(out.css, ".mw-scope-app .panel h2{color:#aabbcc;padding:4px 8px}");
        assert!(out.dropped.is_empty(), "{:?}", out.dropped);
        assert_eq!(out.rules, 1);
    }

    #[test]
    fn body_and_root_map_onto_the_scope() {
        assert_eq!(app("body { color: red }").css, ".mw-scope-app{color:red}");
        assert_eq!(app(":root.x a { color: red }").css, ".mw-scope-app.x a{color:red}");
        assert_eq!(login("html>div{color:red}").css, ".mw-scope-login > div{color:red}");
    }

    #[test]
    fn combinators_and_pseudo_classes_survive() {
        assert_eq!(
            app("ul > li + li ~ li:hover{color:red}").css,
            ".mw-scope-app ul > li + li ~ li:hover{color:red}"
        );
        assert_eq!(
            app("li:nth-child(2n+1):not(.a,.b){color:red}").css,
            ".mw-scope-app li:nth-child(2n+1):not(.a,.b){color:red}"
        );
    }

    #[test]
    fn asset_tokens_become_relative_urls() {
        let out = app(r#".x{background-image:asset("0123456789abcdef0123456789abcdef")}"#);
        assert_eq!(
            out.css,
            r#".mw-scope-app .x{background-image:url("../assets/0123456789abcdef0123456789abcdef")}"#
        );
    }

    #[test]
    fn media_blocks_are_rebuilt() {
        let out = app("@media (max-width: 600px) { .a { color: red } }");
        assert_eq!(out.css, "@media (max-width:600px){.mw-scope-app .a{color:red}}");
    }

    #[test]
    fn profiles_differ() {
        let src = ".a{opacity:.5;transition:color 200ms ease;gap:4px}";
        assert_eq!(login(src).css, "");
        let strict = sanitize_custom_css(src, CssProfile::AppStrict, CssScope::App, &ctx()).unwrap();
        assert_eq!(strict.css, ".mw-scope-app .a{opacity:.5;gap:4px}");
        assert_eq!(app(src).css, ".mw-scope-app .a{opacity:.5;transition:color 200ms ease;gap:4px}");
    }

    #[test]
    fn drops_carry_line_and_column() {
        let out = app(".a{\n  color: red;\n  position: fixed;\n}");
        assert_eq!(out.css, ".mw-scope-app .a{color:red}");
        assert_eq!(out.dropped.len(), 1);
        assert_eq!(out.dropped[0].reason, "property");
        assert_eq!(out.dropped[0].detail, "position");
        assert_eq!(out.dropped[0].line, 3);
    }

    #[test]
    fn whole_sheet_limits() {
        let big = "a{color:red}".repeat(MAX_SOURCE_BYTES / 12 + 1);
        assert!(matches!(
            sanitize_custom_css(&big, CssProfile::AppStrict, CssScope::App, &ctx()),
            Err(CustomCssError::TooLarge { .. })
        ));
        let many = "a{color:red}".repeat(MAX_RULES + 1);
        assert_eq!(
            sanitize_custom_css(&many, CssProfile::AppStrict, CssScope::App, &ctx()),
            Err(CustomCssError::TooManyRules { max: MAX_RULES })
        );
    }

    #[test]
    fn font_family_is_limited_to_the_set() {
        assert_eq!(app(".a{font-family: Jost, sans-serif}").css, ".mw-scope-app .a{font-family:jost, sans-serif}");
        let out = app(".a{font-family: \"Comic Sans MS\"}");
        assert_eq!(out.css, "");
        assert_eq!(out.dropped[0].reason, "font");
        let branded = CssContext {
            brand_font_family: Some("MW Brand".into()),
            ..ctx()
        };
        let out = sanitize_custom_css(".a{font-family:\"MW Brand\", jost}", CssProfile::Login, CssScope::Login, &branded).unwrap();
        assert_eq!(out.css, ".mw-scope-login .a{font-family:\"mw brand\", jost}");
    }

    #[test]
    fn important_is_kept() {
        assert_eq!(app(".a{color:red !important}").css, ".mw-scope-app .a{color:red !important}");
    }
}
