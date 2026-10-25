//! ManageSieve (RFC 5804) client — upload/list/activate/fetch scripts on servers
//! that advertise it (plan §0.6, §3 e2; t32-ext §S).
//!
//! The protocol logic lives on [`Connection`], generic over any async byte
//! stream, so the transcript tests drive it over an in-memory pipe while
//! production uses [`crate::transport::SieveStream`]. ManageSieve is a
//! line-oriented text protocol: a response is zero or more data lines followed
//! by an `OK`/`NO`/`BYE` completion, which may carry a parenthesised response
//! code (`NO (QUOTA/MAXSIZE) "..."`). Strings are quoted (`"..."` with `\`
//! escapes) or literals (`{n}` / `{n+}` then `n` octets), and a literal may sit
//! anywhere in a line (`{4}\r\nname ACTIVE`), so responses are read as logical
//! lines of items rather than as text.
//!
//! Implemented commands: `CAPABILITY`, `AUTHENTICATE` (SASL PLAIN, LOGIN,
//! SCRAM-SHA-256, OAUTHBEARER, XOAUTH2), `STARTTLS` (framing only — the transport
//! does the upgrade), `PUTSCRIPT`, `CHECKSCRIPT`, `HAVESPACE`, `LISTSCRIPTS`,
//! `SETACTIVE`, `GETSCRIPT`, `RENAMESCRIPT` (with the RFC 5804 §2.11.1 fallback
//! for servers older than VERSION 1.0), `DELETESCRIPT`, `NOOP`, `LOGOUT`.
//!
//! Bounds: one physical line is at most [`MAX_LINE`] bytes, one literal at most
//! [`MAX_LITERAL`], one whole response at most [`MAX_RESPONSE`]. Script names
//! with control characters are refused before anything is sent (RFC 5804 §1.6),
//! so a name can never end the command line early. No credential, SASL payload
//! or script body is ever traced.

use base64::Engine;
use base64::engine::general_purpose::STANDARD as B64;
use tokio::io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt};

use crate::sasl;
use crate::{Result, SieveError};

/// Hard cap on one CRLF-terminated line, bounding memory against a hostile or
/// broken server that never sends a newline.
pub const MAX_LINE: usize = 65_536;

/// Largest literal accepted from the server. Scripts are small (Pigeonhole's
/// default limit is 1 MiB); the length is server-chosen, so it is not trusted.
pub const MAX_LITERAL: usize = 8 * 1024 * 1024;

/// Largest whole response (all data lines and literals) accepted.
pub const MAX_RESPONSE: usize = 16 * 1024 * 1024;

/// Server capabilities advertised in the greeting / `CAPABILITY` response.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Capabilities {
    /// `"IMPLEMENTATION"` — server name/version string.
    pub implementation: String,
    /// `"VERSION"` — ManageSieve protocol version (empty before RFC 5804 1.0).
    pub version: String,
    /// `"SASL"` — offered SASL mechanisms, upper-cased.
    pub sasl: Vec<String>,
    /// `"SIEVE"` — Sieve extensions the server supports.
    pub sieve: Vec<String>,
    /// `"STARTTLS"` advertised.
    pub starttls: bool,
    /// `"NOTIFY"` — notification methods (e.g. `mailto`).
    pub notify: Vec<String>,
    /// `"MAXREDIRECTS"` — redirects allowed per script execution.
    pub max_redirects: Option<u32>,
    /// `"OWNER"` — the authenticated user, after login.
    pub owner: Option<String>,
    /// `"LANGUAGE"` — the language of human-readable responses.
    pub language: Option<String>,
}

impl Capabilities {
    /// Whether a SASL mechanism (case-insensitive) was advertised.
    pub fn offers_sasl(&self, mech: &str) -> bool {
        self.sasl.iter().any(|m| m.eq_ignore_ascii_case(mech))
    }

    /// Whether a Sieve extension (case-insensitive) is supported.
    pub fn supports(&self, ext: &str) -> bool {
        self.sieve.iter().any(|e| e.eq_ignore_ascii_case(ext))
    }

    /// Whether the server speaks RFC 5804 1.0 (and so has `RENAMESCRIPT`,
    /// `CHECKSCRIPT` and `NOOP`).
    pub fn is_rfc5804(&self) -> bool {
        !self.version.is_empty()
    }

    fn absorb(&mut self, key: &str, value: &str) {
        let words = || value.split_whitespace().map(|s| s.to_string());
        match key.to_ascii_uppercase().as_str() {
            "IMPLEMENTATION" => self.implementation = value.to_string(),
            "VERSION" => self.version = value.to_string(),
            "SASL" => {
                self.sasl = value
                    .split_whitespace()
                    .map(|s| s.to_ascii_uppercase())
                    .collect()
            }
            "SIEVE" => self.sieve = words().collect(),
            "NOTIFY" => self.notify = words().collect(),
            "STARTTLS" => self.starttls = true,
            "MAXREDIRECTS" => self.max_redirects = value.trim().parse().ok(),
            "OWNER" => self.owner = Some(value.to_string()),
            "LANGUAGE" => self.language = Some(value.to_string()),
            _ => {}
        }
    }
}

/// One entry from `LISTSCRIPTS`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ScriptInfo {
    pub name: String,
    pub active: bool,
}

/// SASL credentials for `AUTHENTICATE`. `Debug` never prints the secret.
#[derive(Clone)]
pub enum Credentials {
    Plain { username: String, password: String },
    Login { username: String, password: String },
    ScramSha256 { username: String, password: String },
    OAuthBearer { username: String, token: String },
    XOAuth2 { username: String, token: String },
}

impl std::fmt::Debug for Credentials {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let (mech, user) = match self {
            Credentials::Plain { username, .. } => ("PLAIN", username),
            Credentials::Login { username, .. } => ("LOGIN", username),
            Credentials::ScramSha256 { username, .. } => ("SCRAM-SHA-256", username),
            Credentials::OAuthBearer { username, .. } => ("OAUTHBEARER", username),
            Credentials::XOAuth2 { username, .. } => ("XOAUTH2", username),
        };
        f.debug_struct("Credentials")
            .field("mechanism", &mech)
            .field("username", user)
            .finish_non_exhaustive()
    }
}

impl Credentials {
    /// The strongest password mechanism `caps` offers: SCRAM-SHA-256, then
    /// PLAIN, then LOGIN. PLAIN when the server lists none of them (some list no
    /// SASL line before TLS).
    pub fn password_for(caps: &Capabilities, username: &str, password: &str) -> Self {
        let (username, password) = (username.to_string(), password.to_string());
        if caps.offers_sasl("SCRAM-SHA-256") {
            Credentials::ScramSha256 { username, password }
        } else if !caps.offers_sasl("PLAIN") && caps.offers_sasl("LOGIN") {
            Credentials::Login { username, password }
        } else {
            Credentials::Plain { username, password }
        }
    }

    fn mechanism(&self) -> &'static str {
        match self {
            Credentials::Plain { .. } => "PLAIN",
            Credentials::Login { .. } => "LOGIN",
            Credentials::ScramSha256 { .. } => "SCRAM-SHA-256",
            Credentials::OAuthBearer { .. } => "OAUTHBEARER",
            Credentials::XOAuth2 { .. } => "XOAUTH2",
        }
    }
}

/// A response code from a completion line (`NO (QUOTA/MAXSIZE) "..."`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResponseCode {
    /// Upper-cased, with any `/` sub-code (`QUOTA/MAXSCRIPTS`).
    pub name: String,
    /// The argument, unquoted (`SASL "<base64>"`, `TAG "x"`).
    pub arg: Option<String>,
}

impl ResponseCode {
    fn parse(inner: &str) -> Option<Self> {
        let inner = inner.trim();
        if inner.is_empty() {
            return None;
        }
        let (name, rest) = inner.split_once(' ').unwrap_or((inner, ""));
        let rest = rest.trim();
        let arg = if rest.is_empty() {
            None
        } else {
            let (items, _) = tokenize(rest);
            Some(items.first().map_or_else(|| rest.to_string(), Item::text))
        };
        Some(ResponseCode {
            name: name.to_ascii_uppercase(),
            arg,
        })
    }

    /// `QUOTA`, `QUOTA/MAXSCRIPTS` or `QUOTA/MAXSIZE`.
    pub fn is_quota(&self) -> bool {
        self.name == "QUOTA" || self.name.starts_with("QUOTA/")
    }
}

/// A `NO` or `BYE` from the server, with its code and text.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Refusal {
    pub bye: bool,
    pub code: Option<ResponseCode>,
    pub text: String,
}

impl std::fmt::Display for Refusal {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let word = if self.bye { "BYE" } else { "NO" };
        match &self.code {
            Some(c) => write!(f, "server said {word} ({}): {}", c.name, self.text),
            None => write!(f, "server said {word}: {}", self.text),
        }
    }
}

/// The server's verdict on `CHECKSCRIPT`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Checked {
    /// The script compiles; `warnings` is the text of an `OK (WARNINGS)`.
    Valid { warnings: Option<String> },
    /// The script does not compile; `problems` are the positions the server
    /// named, `text` its full message.
    Invalid {
        problems: Vec<ScriptProblem>,
        text: String,
    },
}

/// One error position in a server's compile message.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ScriptProblem {
    pub line: Option<u32>,
    pub column: Option<u32>,
    pub message: String,
}

/// Pull `line N[, column M]: message` / `line N:M: message` / `N:M: message`
/// entries out of a server's compile error text (Pigeonhole writes
/// `line 2: error: unknown command 'x'.`, Cyrus `line 2: ...`, Stalwart
/// `line 2, column 5: ...`). Text without positions is one problem with none.
pub fn script_problems(text: &str) -> Vec<ScriptProblem> {
    let mut out = Vec::new();
    for raw in text.lines() {
        let line = raw.trim();
        if line.is_empty() {
            continue;
        }
        // Pigeonhole prefixes the script name (`name.sieve: line 2: ...`), so
        // the position may start anywhere on the line.
        let found = positioned(line).or_else(|| {
            let lower = line.to_ascii_lowercase();
            lower
                .match_indices("line ")
                .find_map(|(i, _)| positioned(&line[i..]))
        });
        if let Some(p) = found {
            out.push(p);
        }
    }
    if out.is_empty() && !text.trim().is_empty() {
        out.push(ScriptProblem {
            line: None,
            column: None,
            message: text.trim().to_string(),
        });
    }
    out
}

fn positioned(line: &str) -> Option<ScriptProblem> {
    let lower = line.to_ascii_lowercase();
    let rest = lower
        .strip_prefix("line ")
        .map(|_| &line[5..])
        .unwrap_or(line);
    let digits = |s: &str| -> Option<(u32, usize)> {
        let n = s.bytes().take_while(u8::is_ascii_digit).count();
        if n == 0 || n > 9 {
            return None;
        }
        Some((s[..n].parse().ok()?, n))
    };
    let (ln, n) = digits(rest)?;
    let mut rest = &rest[n..];
    let mut column = None;
    if let Some(r) = rest
        .strip_prefix(", column ")
        .or_else(|| rest.strip_prefix(", col "))
    {
        let (c, n) = digits(r)?;
        column = Some(c);
        rest = &r[n..];
    } else if let Some(r) = rest.strip_prefix(':')
        && let Some((c, n)) = digits(r)
        && r[n..].starts_with(':')
    {
        column = Some(c);
        rest = &r[n..];
    }
    let message = rest.strip_prefix(':')?.trim().to_string();
    Some(ScriptProblem {
        line: Some(ln),
        column,
        message,
    })
}

// ── response model ─────────────────────────────────────────────────────────

/// One item of a response line.
#[derive(Debug, Clone, PartialEq, Eq)]
enum Item {
    Quoted(String),
    Literal(Vec<u8>),
    Atom(String),
    /// A parenthesised response code, inner text as written.
    Code(String),
}

impl Item {
    fn text(&self) -> String {
        match self {
            Item::Quoted(s) | Item::Atom(s) | Item::Code(s) => s.clone(),
            Item::Literal(b) => String::from_utf8_lossy(b).into_owned(),
        }
    }

    fn is_string(&self) -> bool {
        matches!(self, Item::Quoted(_) | Item::Literal(_))
    }
}

/// A logical response line: its items and its physical text (literals left out).
#[derive(Debug, Clone, Default)]
struct Line {
    items: Vec<Item>,
    raw: String,
}

/// The completion status of a ManageSieve response.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Status {
    Ok,
    No,
    Bye,
}

/// A parsed response: data lines plus the completion.
#[derive(Debug, Clone)]
struct Response {
    lines: Vec<Line>,
    status: Status,
    code: Option<ResponseCode>,
    text: Option<String>,
}

impl Response {
    fn into_ok(self) -> Result<Self> {
        match self.status {
            Status::Ok => Ok(self),
            Status::No | Status::Bye => Err(SieveError::Refused(self.refusal())),
        }
    }

    fn refusal(&self) -> Refusal {
        Refusal {
            bye: self.status == Status::Bye,
            code: self.code.clone(),
            text: self.text.clone().unwrap_or_default(),
        }
    }
}

/// Split one physical line into items. Returns the items and, when the line
/// ends in a literal marker (`{n}` / `{n+}`), the literal's announced length.
/// Never fails: anything unparseable becomes an atom.
fn tokenize(line: &str) -> (Vec<Item>, Option<u64>) {
    let mut items = Vec::new();
    let mut chars = line.char_indices().peekable();
    while let Some(&(i, c)) = chars.peek() {
        match c {
            ' ' | '\t' => {
                chars.next();
            }
            '"' => {
                chars.next();
                let mut s = String::new();
                while let Some((_, c)) = chars.next() {
                    match c {
                        '"' => break,
                        '\\' => {
                            if let Some((_, n)) = chars.next() {
                                s.push(n);
                            }
                        }
                        other => s.push(other),
                    }
                }
                items.push(Item::Quoted(s));
            }
            '(' => {
                chars.next();
                let start = i + 1;
                let mut end = line.len();
                let mut in_quote = false;
                let mut depth = 1usize;
                while let Some((j, c)) = chars.next() {
                    match c {
                        '\\' if in_quote => {
                            chars.next();
                        }
                        '"' => in_quote = !in_quote,
                        '(' if !in_quote => depth += 1,
                        ')' if !in_quote => {
                            depth -= 1;
                            if depth == 0 {
                                end = j;
                                break;
                            }
                        }
                        _ => {}
                    }
                }
                items.push(Item::Code(line[start..end].to_string()));
            }
            _ => {
                let start = i;
                let mut end = line.len();
                while let Some(&(j, c)) = chars.peek() {
                    if c == ' ' || c == '\t' {
                        end = j;
                        break;
                    }
                    chars.next();
                }
                let atom = &line[start..end];
                if end == line.len()
                    && let Some(n) = literal_marker(atom)
                {
                    return (items, Some(n));
                }
                items.push(Item::Atom(atom.to_string()));
            }
        }
    }
    (items, None)
}

/// `{n}` / `{n+}` → `n` (saturating, so an absurd length is still refused by
/// the caller's bound rather than misread).
fn literal_marker(atom: &str) -> Option<u64> {
    let inner = atom.strip_prefix('{')?.strip_suffix('}')?;
    let digits = inner.strip_suffix('+').unwrap_or(inner);
    if digits.is_empty() || !digits.bytes().all(|b| b.is_ascii_digit()) {
        return None;
    }
    Some(digits.parse().unwrap_or(u64::MAX))
}

fn status_of(line: &Line) -> Option<Status> {
    match line.items.first() {
        Some(Item::Atom(a)) => match a.to_ascii_uppercase().as_str() {
            "OK" => Some(Status::Ok),
            "NO" => Some(Status::No),
            "BYE" => Some(Status::Bye),
            _ => None,
        },
        _ => None,
    }
}

fn completion(line: Line, status: Status, lines: Vec<Line>) -> Response {
    let mut rest = line.items.into_iter().skip(1).peekable();
    let code = match rest.peek() {
        Some(Item::Code(c)) => {
            let c = ResponseCode::parse(c);
            rest.next();
            c
        }
        _ => None,
    };
    let rest: Vec<Item> = rest.collect();
    let text = match rest.as_slice() {
        [] => None,
        [one] if one.is_string() => Some(one.text()),
        many => Some(many.iter().map(Item::text).collect::<Vec<_>>().join(" ")),
    };
    Response {
        lines,
        status,
        code,
        text,
    }
}

/// Fuzz entry point for the response tokenizer and the compile-error parser:
/// arbitrary bytes must never panic either.
pub fn fuzz_response(data: &[u8]) {
    let text = String::from_utf8_lossy(data);
    for physical in text.split('\n') {
        let (items, _) = tokenize(physical.trim_end_matches('\r'));
        let line = Line {
            items,
            raw: physical.to_string(),
        };
        if let Some(status) = status_of(&line) {
            let _ = completion(line, status, Vec::new()).refusal().to_string();
        } else if let Some(Item::Code(c)) = line.items.first() {
            let _ = ResponseCode::parse(c);
        }
    }
    let _ = script_problems(&text);
}

// ── connection ─────────────────────────────────────────────────────────────

/// A ManageSieve connection with a small read buffer for line framing.
pub struct Connection<S> {
    stream: S,
    rbuf: Vec<u8>,
    caps: Capabilities,
}

impl<S: AsyncRead + AsyncWrite + Unpin> Connection<S> {
    /// Wrap an established stream and read the server greeting (a capability
    /// listing terminated by `OK`), populating [`capabilities`](Self::capabilities).
    pub async fn open(stream: S) -> Result<Self> {
        let mut conn = Connection {
            stream,
            rbuf: Vec::with_capacity(2048),
            caps: Capabilities::default(),
        };
        let resp = conn.read_response().await?;
        conn.caps = parse_capabilities(&resp.lines);
        resp.into_ok()?;
        Ok(conn)
    }

    /// The capabilities from the greeting (refreshed by [`capability`](Self::capability)).
    pub fn capabilities(&self) -> &Capabilities {
        &self.caps
    }

    /// Recover the inner stream for a `STARTTLS` transport upgrade. Any buffered
    /// unread bytes before the handshake are a protocol violation.
    pub fn into_inner(self) -> Result<S> {
        if self.rbuf.is_empty() {
            Ok(self.stream)
        } else {
            Err(SieveError::ManageSieve(
                "unexpected data buffered before STARTTLS handshake".into(),
            ))
        }
    }

    async fn fill(&mut self, what: &str) -> Result<()> {
        let mut tmp = [0u8; 4096];
        let n = self.stream.read(&mut tmp).await?;
        if n == 0 {
            return Err(SieveError::ManageSieve(format!(
                "connection closed mid-{what}"
            )));
        }
        self.rbuf.extend_from_slice(&tmp[..n]);
        Ok(())
    }

    /// One physical line, CRLF (or bare LF) removed.
    async fn read_physical(&mut self) -> Result<String> {
        loop {
            if let Some(pos) = self.rbuf.iter().position(|&b| b == b'\n') {
                if pos > MAX_LINE {
                    return Err(SieveError::ManageSieve(
                        "response line exceeded limit".into(),
                    ));
                }
                let mut line: Vec<u8> = self.rbuf.drain(..=pos).collect();
                line.pop();
                if line.last() == Some(&b'\r') {
                    line.pop();
                }
                return Ok(String::from_utf8_lossy(&line).into_owned());
            }
            if self.rbuf.len() > MAX_LINE {
                return Err(SieveError::ManageSieve(
                    "response line exceeded limit".into(),
                ));
            }
            self.fill("response").await?;
        }
    }

    /// Exactly `len` literal octets.
    async fn read_literal(&mut self, len: usize) -> Result<Vec<u8>> {
        while self.rbuf.len() < len {
            self.fill("literal").await?;
        }
        Ok(self.rbuf.drain(..len).collect())
    }

    /// One logical line: physical lines joined across literals. `budget` is the
    /// remaining response allowance, reduced by what this line consumed.
    async fn read_line(&mut self, budget: &mut usize) -> Result<Line> {
        let mut line = Line::default();
        loop {
            let physical = self.read_physical().await?;
            *budget = budget.checked_sub(physical.len() + 2).ok_or_else(too_big)?;
            let (mut items, literal) = tokenize(&physical);
            line.items.append(&mut items);
            if !line.raw.is_empty() {
                line.raw.push(' ');
            }
            line.raw.push_str(&physical);
            let Some(len) = literal else { return Ok(line) };
            let len = usize::try_from(len)
                .ok()
                .filter(|&n| n <= MAX_LITERAL)
                .ok_or_else(|| {
                    SieveError::ManageSieve(format!(
                        "server literal of {len} bytes exceeds the {MAX_LITERAL}-byte limit"
                    ))
                })?;
            *budget = budget.checked_sub(len).ok_or_else(too_big)?;
            let blob = self.read_literal(len).await?;
            line.items.push(Item::Literal(blob));
        }
    }

    /// Read a full response: data lines until an `OK`/`NO`/`BYE`.
    async fn read_response(&mut self) -> Result<Response> {
        let mut lines = Vec::new();
        let mut budget = MAX_RESPONSE;
        loop {
            let line = self.read_line(&mut budget).await?;
            if let Some(status) = status_of(&line) {
                return Ok(completion(line, status, lines));
            }
            lines.push(line);
        }
    }

    async fn write_all(&mut self, bytes: &[u8]) -> Result<()> {
        self.stream.write_all(bytes).await?;
        self.stream.flush().await?;
        Ok(())
    }

    /// Send a command line and read its response. `line` must not hold secrets:
    /// it is traced.
    async fn command(&mut self, line: &str) -> Result<Response> {
        tracing::trace!(target: "mw_sieve::wire", "C: {}", line.trim_end());
        self.write_all(line.as_bytes()).await?;
        self.read_response().await
    }

    /// Send `HEAD {n+}\r\n<body>\r\n` and read the response. The body is not
    /// traced.
    async fn command_with_literal(&mut self, head: &str, body: &[u8]) -> Result<Response> {
        let line = format!("{head} {{{}+}}\r\n", body.len());
        tracing::trace!(target: "mw_sieve::wire", "C: {} <{} bytes>", head, body.len());
        let mut out = Vec::with_capacity(line.len() + body.len() + 2);
        out.extend_from_slice(line.as_bytes());
        out.extend_from_slice(body);
        out.extend_from_slice(b"\r\n");
        self.write_all(&out).await?;
        self.read_response().await
    }

    /// Re-query `CAPABILITY`, refreshing the stored capabilities.
    pub async fn capability(&mut self) -> Result<&Capabilities> {
        let resp = self.command("CAPABILITY\r\n").await?;
        self.caps = parse_capabilities(&resp.lines);
        resp.into_ok()?;
        Ok(&self.caps)
    }

    /// Frame the `STARTTLS` command and require its `OK` (the caller then swaps
    /// the transport to TLS and re-reads capabilities).
    pub async fn starttls(&mut self) -> Result<()> {
        self.command("STARTTLS\r\n").await?.into_ok()?;
        Ok(())
    }

    /// Authenticate with the given SASL credentials. None of the mechanisms
    /// negotiates a security layer, so the capabilities stay as they were.
    pub async fn authenticate(&mut self, creds: &Credentials) -> Result<()> {
        let mut mech = Mech::new(creds);
        let name = creds.mechanism();
        let head = match mech.initial() {
            Some(ir) => format!("AUTHENTICATE \"{name}\" \"{}\"\r\n", B64.encode(ir)),
            None => format!("AUTHENTICATE \"{name}\"\r\n"),
        };
        tracing::trace!(target: "mw_sieve::wire", "C: AUTHENTICATE \"{name}\" <credentials>");
        self.write_all(head.as_bytes()).await?;
        let mut budget = MAX_RESPONSE;
        loop {
            let line = self.read_line(&mut budget).await?;
            if let Some(status) = status_of(&line) {
                let resp = completion(line, status, Vec::new()).into_ok()?;
                let server_final = match &resp.code {
                    Some(c) if c.name == "SASL" => match c.arg.as_deref().map(|a| B64.decode(a)) {
                        Some(Ok(bytes)) => Some(bytes),
                        _ => {
                            return Err(SieveError::ManageSieve(
                                "server sent a malformed SASL response code".into(),
                            ));
                        }
                    },
                    _ => None,
                };
                mech.finish(server_final.as_deref())
                    .map_err(SieveError::ManageSieve)?;
                return Ok(());
            }
            // A server challenge: one string holding base64.
            let challenge = match line.items.first() {
                Some(item) if item.is_string() => B64.decode(item.text().trim()).ok(),
                _ => None,
            };
            let reply = match challenge {
                Some(c) => mech.step(&c),
                None => Err("server sent a malformed SASL challenge".to_string()),
            };
            match reply {
                Ok(bytes) => {
                    self.write_all(format!("\"{}\"\r\n", B64.encode(bytes)).as_bytes())
                        .await?;
                }
                Err(e) => {
                    // RFC 5804 §2.1: `"*"` cancels; the server answers NO.
                    self.write_all(b"\"*\"\r\n").await?;
                    let _ = self.read_response().await;
                    return Err(SieveError::ManageSieve(e));
                }
            }
        }
    }

    /// `PUTSCRIPT "name" {n+}` + body — upload/replace a script. Returns the
    /// text of an `OK (WARNINGS)` completion, if any. A `NO` carries its code
    /// (`QUOTA/MAXSIZE`, `QUOTA/MAXSCRIPTS`) in [`SieveError::Refused`].
    pub async fn put_script(&mut self, name: &str, body: &str) -> Result<Option<String>> {
        let head = format!("PUTSCRIPT {}", quote_name(name)?);
        let resp = self
            .command_with_literal(&head, body.as_bytes())
            .await?
            .into_ok()?;
        Ok(warnings(&resp))
    }

    /// `CHECKSCRIPT {n+}` + body — ask the server whether a script compiles,
    /// without storing it.
    pub async fn check_script(&mut self, body: &str) -> Result<Checked> {
        let resp = self
            .command_with_literal("CHECKSCRIPT", body.as_bytes())
            .await?;
        match resp.status {
            Status::Ok => Ok(Checked::Valid {
                warnings: warnings(&resp),
            }),
            Status::No if resp.code.as_ref().is_none_or(|c| c.name == "WARNINGS") => {
                let text = resp.text.clone().unwrap_or_default();
                Ok(Checked::Invalid {
                    problems: script_problems(&text),
                    text,
                })
            }
            Status::No | Status::Bye => Err(SieveError::Refused(resp.refusal())),
        }
    }

    /// `HAVESPACE "name" size` — whether a script of `size` bytes under `name`
    /// would fit. A full quota is `Err(SieveError::Refused)` with a `QUOTA*` code.
    pub async fn have_space(&mut self, name: &str, size: u64) -> Result<()> {
        self.command(&format!("HAVESPACE {} {size}\r\n", quote_name(name)?))
            .await?
            .into_ok()?;
        Ok(())
    }

    /// `LISTSCRIPTS` — every stored script and whether it is active.
    pub async fn list_scripts(&mut self) -> Result<Vec<ScriptInfo>> {
        let resp = self.command("LISTSCRIPTS\r\n").await?;
        let scripts = resp.lines.iter().filter_map(parse_script_line).collect();
        resp.into_ok()?;
        Ok(scripts)
    }

    /// `SETACTIVE "name"` — activate a script (empty name deactivates all).
    pub async fn set_active(&mut self, name: &str) -> Result<()> {
        let arg = if name.is_empty() {
            "\"\"".to_string()
        } else {
            quote_name(name)?
        };
        self.command(&format!("SETACTIVE {arg}\r\n"))
            .await?
            .into_ok()?;
        Ok(())
    }

    /// `GETSCRIPT "name"` — fetch a stored script's source.
    pub async fn get_script(&mut self, name: &str) -> Result<String> {
        let resp = self
            .command(&format!("GETSCRIPT {}\r\n", quote_name(name)?))
            .await?;
        let resp = resp.into_ok()?;
        let body = resp
            .lines
            .first()
            .map(|l| match l.items.first() {
                Some(item) if item.is_string() => item.text(),
                // Some servers return a short script as a bare line.
                _ => l.raw.clone(),
            })
            .unwrap_or_default();
        Ok(body)
    }

    /// `RENAMESCRIPT "old" "new"`. A server older than RFC 5804 1.0 has no
    /// `RENAMESCRIPT`, so the rename is done the §2.11.1 way: fetch, store under
    /// the new name, carry over the active flag, delete the old one.
    pub async fn rename_script(&mut self, old: &str, new: &str) -> Result<()> {
        let (old_q, new_q) = (quote_name(old)?, quote_name(new)?);
        if self.caps.is_rfc5804() {
            self.command(&format!("RENAMESCRIPT {old_q} {new_q}\r\n"))
                .await?
                .into_ok()?;
            return Ok(());
        }
        let scripts = self.list_scripts().await?;
        if scripts.iter().any(|s| s.name == new) {
            return Err(SieveError::Refused(Refusal {
                bye: false,
                code: Some(ResponseCode {
                    name: "ALREADYEXISTS".into(),
                    arg: None,
                }),
                text: format!("a script named {new:?} already exists"),
            }));
        }
        let was_active = scripts.iter().any(|s| s.name == old && s.active);
        let body = self.get_script(old).await?;
        self.put_script(new, &body).await?;
        if was_active {
            self.set_active(new).await?;
        }
        self.delete_script(old).await
    }

    /// `DELETESCRIPT "name"`.
    pub async fn delete_script(&mut self, name: &str) -> Result<()> {
        self.command(&format!("DELETESCRIPT {}\r\n", quote_name(name)?))
            .await?
            .into_ok()?;
        Ok(())
    }

    /// `NOOP` — keepalive.
    pub async fn noop(&mut self) -> Result<()> {
        self.command("NOOP\r\n").await?.into_ok()?;
        Ok(())
    }

    /// `LOGOUT` — best-effort; the server answers `OK`/`BYE` and closes.
    pub async fn logout(&mut self) -> Result<()> {
        let resp = self.command("LOGOUT\r\n").await?;
        // Both OK and BYE are acceptable terminations.
        match resp.status {
            Status::Ok | Status::Bye => Ok(()),
            Status::No => resp.into_ok().map(|_| ()),
        }
    }
}

fn too_big() -> SieveError {
    SieveError::ManageSieve(format!(
        "server response exceeded the {MAX_RESPONSE}-byte limit"
    ))
}

fn warnings(resp: &Response) -> Option<String> {
    match &resp.code {
        Some(c) if c.name == "WARNINGS" => Some(resp.text.clone().unwrap_or_default()),
        _ => None,
    }
}

// ── SASL driver ────────────────────────────────────────────────────────────

/// One mechanism's client state.
enum Mech {
    Plain(Vec<u8>),
    Login {
        answers: [String; 2],
        next: usize,
    },
    Scram {
        scram: sasl::ScramSha256,
        first: String,
        verified: bool,
    },
    OAuth {
        initial: Vec<u8>,
        bearer: bool,
    },
}

impl Mech {
    fn new(creds: &Credentials) -> Self {
        match creds {
            Credentials::Plain { username, password } => {
                Mech::Plain(sasl::plain(username, password))
            }
            Credentials::Login { username, password } => Mech::Login {
                answers: [username.clone(), password.clone()],
                next: 0,
            },
            Credentials::ScramSha256 { username, password } => {
                let (scram, first) =
                    sasl::ScramSha256::new(username, password, &sasl::client_nonce());
                Mech::Scram {
                    scram,
                    first,
                    verified: false,
                }
            }
            Credentials::OAuthBearer { username, token } => Mech::OAuth {
                initial: sasl::oauthbearer(username, token),
                bearer: true,
            },
            Credentials::XOAuth2 { username, token } => Mech::OAuth {
                initial: sasl::xoauth2(username, token),
                bearer: false,
            },
        }
    }

    /// The initial response sent with `AUTHENTICATE`, if the mechanism has one.
    fn initial(&mut self) -> Option<Vec<u8>> {
        match self {
            Mech::Plain(p) => Some(p.clone()),
            Mech::Login { .. } => None,
            Mech::Scram { first, .. } => Some(first.clone().into_bytes()),
            Mech::OAuth { initial, .. } => Some(initial.clone()),
        }
    }

    /// Answer one server challenge.
    fn step(&mut self, challenge: &[u8]) -> std::result::Result<Vec<u8>, String> {
        match self {
            // A server that ignored the initial response asks with an empty
            // challenge; anything else is unexpected for PLAIN.
            Mech::Plain(p) if challenge.is_empty() => Ok(p.clone()),
            Mech::Plain(_) => Err("unexpected SASL challenge for PLAIN".into()),
            Mech::Login { answers, next } => {
                let answer = answers.get(*next).cloned().unwrap_or_default();
                *next += 1;
                Ok(answer.into_bytes())
            }
            Mech::Scram {
                scram, verified, ..
            } => {
                let text = std::str::from_utf8(challenge)
                    .map_err(|_| "SCRAM challenge is not UTF-8".to_string())?;
                if text.starts_with("v=") || text.starts_with("e=") {
                    scram.verify(text)?;
                    *verified = true;
                    Ok(Vec::new())
                } else {
                    Ok(scram.client_final(text)?.into_bytes())
                }
            }
            // RFC 7628 §3.2.3: an error challenge is answered with %x01; XOAUTH2
            // with an empty response. The server then says NO.
            Mech::OAuth { bearer: true, .. } => Ok(vec![0x01]),
            Mech::OAuth { bearer: false, .. } => Ok(Vec::new()),
        }
    }

    /// Called on `OK`, with the decoded `(SASL "...")` data if any. SCRAM
    /// must have seen and verified the server's signature by now.
    fn finish(&mut self, server_final: Option<&[u8]>) -> std::result::Result<(), String> {
        match self {
            Mech::Scram {
                scram, verified, ..
            } => {
                if let Some(data) = server_final {
                    let text = std::str::from_utf8(data)
                        .map_err(|_| "SCRAM server-final is not UTF-8".to_string())?;
                    scram.verify(text)?;
                    *verified = true;
                }
                if *verified {
                    Ok(())
                } else {
                    Err("server accepted SCRAM without proving it knows the password".into())
                }
            }
            _ => Ok(()),
        }
    }
}

// ── parsing helpers ────────────────────────────────────────────────────────

/// Quote a script name for a command, refusing what RFC 5804 §1.6 forbids in a
/// name: the empty string and control characters (so a CR or LF can never end
/// the command line and start another).
fn quote_name(name: &str) -> Result<String> {
    if name.is_empty() {
        return Err(SieveError::ManageSieve(
            "a script name cannot be empty".into(),
        ));
    }
    if name
        .chars()
        .any(|c| c.is_control() || c == '\u{2028}' || c == '\u{2029}')
    {
        return Err(SieveError::ManageSieve(
            "a script name cannot contain control characters".into(),
        ));
    }
    Ok(format!(
        "\"{}\"",
        name.replace('\\', "\\\\").replace('"', "\\\"")
    ))
}

/// Parse one `LISTSCRIPTS` line: a string (quoted or literal), then optionally
/// `ACTIVE`.
fn parse_script_line(line: &Line) -> Option<ScriptInfo> {
    let first = line.items.first().filter(|i| i.is_string())?;
    let active =
        matches!(line.items.get(1), Some(Item::Atom(a)) if a.eq_ignore_ascii_case("ACTIVE"));
    Some(ScriptInfo {
        name: first.text(),
        active,
    })
}

/// Parse capability data lines into [`Capabilities`]. Each line is `"KEY"
/// ["VALUE"]`; `STARTTLS` may appear bare.
fn parse_capabilities(lines: &[Line]) -> Capabilities {
    let mut caps = Capabilities::default();
    for line in lines {
        let Some(key) = line.items.first() else {
            continue;
        };
        let value = line.items.get(1).map(Item::text).unwrap_or_default();
        caps.absorb(&key.text(), &value);
    }
    caps
}

#[cfg(test)]
mod tests {
    use super::*;

    fn line(s: &str) -> Line {
        Line {
            items: tokenize(s).0,
            raw: s.to_string(),
        }
    }

    #[test]
    fn status_needs_a_bare_atom() {
        assert_eq!(status_of(&line("OK \"done\"")), Some(Status::Ok));
        assert_eq!(status_of(&line("NO (QUOTA) \"full\"")), Some(Status::No));
        assert_eq!(status_of(&line("bye")), Some(Status::Bye));
        assert_eq!(status_of(&line("\"OKmail\"")), None);
        assert_eq!(status_of(&line("\"OK\" ACTIVE")), None, "a script named OK");
        assert_eq!(status_of(&line("OKmail")), None);
    }

    #[test]
    fn completions_carry_code_and_text() {
        let r = completion(line("NO (QUOTA/MAXSIZE) \"Too big\""), Status::No, vec![]);
        assert_eq!(r.code.as_ref().unwrap().name, "QUOTA/MAXSIZE");
        assert!(r.code.as_ref().unwrap().is_quota());
        assert_eq!(r.text.as_deref(), Some("Too big"));
        let r = completion(line("OK (SASL \"dj1hYmM=\")"), Status::Ok, vec![]);
        let c = r.code.unwrap();
        assert_eq!(
            (c.name.as_str(), c.arg.as_deref()),
            ("SASL", Some("dj1hYmM="))
        );
        assert_eq!(r.text, None);
        let r = completion(line("OK"), Status::Ok, vec![]);
        assert_eq!((r.code, r.text), (None, None));
        let r = completion(line("NO (TAG \"a)b\") \"x\""), Status::No, vec![]);
        assert_eq!(
            r.code.unwrap().arg.as_deref(),
            Some("a)b"),
            "a `)` inside quotes"
        );
        assert_eq!(r.text.as_deref(), Some("x"));
    }

    #[test]
    fn literal_markers_only_at_line_end() {
        assert_eq!(tokenize("{42}").1, Some(42));
        assert_eq!(tokenize("{42+}").1, Some(42));
        assert_eq!(tokenize("NO {7}").1, Some(7));
        assert_eq!(tokenize("\"text {5}\"").1, None, "inside quotes");
        assert_eq!(tokenize("{5} more").1, None);
        assert_eq!(tokenize("{}").1, None);
        assert_eq!(tokenize("{a}").1, None);
        assert_eq!(tokenize("{99999999999999999999999}").1, Some(u64::MAX));
    }

    #[test]
    fn quoted_names_unescape_fully() {
        let l = line(r#""with \"quotes\" inside" ACTIVE"#);
        assert_eq!(
            parse_script_line(&l),
            Some(ScriptInfo {
                name: "with \"quotes\" inside".into(),
                active: true
            })
        );
        let caps =
            parse_capabilities(&[line(r#""IMPL\"EMENTATION" "x""#), line("\"SIEVE\" \"a b\"")]);
        assert!(caps.supports("b"));
    }

    #[test]
    fn capabilities_absorb() {
        let lines = [
            "\"IMPLEMENTATION\" \"Dovecot Pigeonhole\"",
            "\"SIEVE\" \"fileinto vacation imap4flags\"",
            "\"SASL\" \"plain login\"",
            "\"STARTTLS\"",
            "\"VERSION\" \"1.0\"",
            "\"MAXREDIRECTS\" \"4\"",
            "\"OWNER\" \"alice\"",
        ]
        .map(line);
        let caps = parse_capabilities(&lines);
        assert_eq!(caps.implementation, "Dovecot Pigeonhole");
        assert!(caps.supports("fileinto"));
        assert!(caps.offers_sasl("PLAIN"));
        assert!(caps.starttls);
        assert!(caps.is_rfc5804());
        assert_eq!(caps.max_redirects, Some(4));
        assert_eq!(caps.owner.as_deref(), Some("alice"));
        assert!(matches!(
            Credentials::password_for(&caps, "u", "p"),
            Credentials::Plain { .. }
        ));
    }

    #[test]
    fn mechanism_choice() {
        let mut caps = Capabilities::default();
        assert!(matches!(
            Credentials::password_for(&caps, "u", "p"),
            Credentials::Plain { .. }
        ));
        caps.sasl = vec!["LOGIN".into()];
        assert!(matches!(
            Credentials::password_for(&caps, "u", "p"),
            Credentials::Login { .. }
        ));
        caps.sasl = vec!["LOGIN".into(), "PLAIN".into()];
        assert!(matches!(
            Credentials::password_for(&caps, "u", "p"),
            Credentials::Plain { .. }
        ));
    }

    #[test]
    fn credentials_debug_hides_secrets() {
        let c = Credentials::Plain {
            username: "alice".into(),
            password: "hunter2".into(),
        };
        let d = format!("{c:?}");
        assert!(d.contains("alice") && d.contains("PLAIN"), "{d}");
        assert!(!d.contains("hunter2"), "{d}");
        let c = Credentials::OAuthBearer {
            username: "alice".into(),
            token: "tok-secret".into(),
        };
        assert!(!format!("{c:?}").contains("tok-secret"));
    }

    #[test]
    fn names_with_control_characters_are_refused() {
        assert!(quote_name("ok name").is_ok());
        assert_eq!(quote_name(r#"a"b\c"#).unwrap(), r#""a\"b\\c""#);
        for bad in [
            "",
            "a\r\nDELETESCRIPT \"x\"",
            "a\nb",
            "nul\0",
            "del\u{7f}",
            "ls\u{2028}",
        ] {
            assert!(quote_name(bad).is_err(), "{bad:?}");
        }
    }

    #[test]
    fn script_problems_find_positions() {
        let p = script_problems(
            "line 2: error: unknown command 'frob' (only reported once at first occurrence).\r\n\
             error: validation failed.",
        );
        assert_eq!(p.len(), 1);
        assert_eq!((p[0].line, p[0].column), (Some(2), None));
        assert!(p[0].message.starts_with("error: unknown command"));

        let p = script_problems("line 3, column 7: unexpected `}`\nline 9:2: bad");
        assert_eq!((p[0].line, p[0].column), (Some(3), Some(7)));
        assert_eq!(
            (p[1].line, p[1].column, p[1].message.as_str()),
            (Some(9), Some(2), "bad")
        );

        let p = script_problems(
            "1792933068.M1.host.tmp: line 4: error: x.
error: validation failed.",
        );
        assert_eq!((p[0].line, p[0].message.as_str()), (Some(4), "error: x."));

        let p = script_problems("script is broken");
        assert_eq!(
            (p[0].line, p[0].message.as_str()),
            (None, "script is broken")
        );
        assert!(script_problems("").is_empty());
    }

    #[test]
    fn fuzz_response_survives_junk() {
        for junk in [
            &b"("[..],
            b"OK (",
            b"NO (\"",
            b"\"\\",
            b"{",
            b"NO {99999999999999999999}",
            b"line 99999999999: x",
            b"line 1, column : x",
            b"\xff\xfe(\"\\",
        ] {
            fuzz_response(junk);
        }
    }
}
