//! Redaction at capture for the application log store (t32-ext §L, f08/e26).
//!
//! Every log record the process-local capture layer keeps passes through
//! [`Redactor::field`] / [`Redactor::text`]
//! first. What never survives:
//!
//! * **message content**: a field named like a message part (`subject`, `body`,
//!   `snippet`, `html`, `preview`, `content`, …) keeps only its length;
//! * **credentials**: a field named like a secret (`password`, `token`, `secret`,
//!   `authorization`, `cookie`, `api_key`, `dsn`, …) is replaced whole; in any other
//!   text, `Bearer …` / `Basic …` values, `password=…`-style pairs, session cookies,
//!   JWT-shaped strings, `mwk_` API keys and URL userinfo are cut out;
//! * **the values of secret environment variables** this process was started with
//!   (names ending in `PASSWORD`, `_PW`, `SECRET`, `TOKEN`, `_KEY`, `API_KEY`, `DSN`;
//!   the same rule as t32-cfg §8 A1), wherever they appear;
//! * **the admin path** (`MW_ADMIN_PATH`, t32-h15): the console's location is not
//!   published, so a logged request line names it `[admin-path]`.
//!
//! The rules are deliberately broad: a false positive costs a log line some
//! detail, a false negative publishes a secret to everyone with `logs.read`.

use std::sync::RwLock;

/// What replaces a redacted value.
pub const MARK: &str = "[redacted]";
/// What replaces the admin path.
pub const ADMIN_MARK: &str = "[admin-path]";

/// Field-name fragments whose value is a credential (compared lower-case).
const SECRET_NAMES: &[&str] = &[
    "password",
    "passwd",
    "passphrase",
    "secret",
    "token",
    "authorization",
    "cookie",
    "api_key",
    "apikey",
    "private_key",
    "credential",
    "bearer",
    "dsn",
    "signature",
    "otp",
    "totp",
    "recovery_code",
    "code",
    "sealed",
];

/// Field names whose value is message content (compared lower-case, exact).
const CONTENT_NAMES: &[&str] = &[
    "subject",
    "body",
    "snippet",
    "html",
    "text_body",
    "html_body",
    "preview",
    "content",
    "mime",
    "raw",
    "attachment_name",
    "filename",
    "search_text",
    "query_text",
];

/// Key names in `k=v` / `k: v` text whose value is cut.
const SECRET_KEYS: &[&str] = &[
    "password",
    "passwd",
    "pass",
    "pwd",
    "secret",
    "token",
    "access_token",
    "refresh_token",
    "id_token",
    "client_secret",
    "bind_password",
    "otp",
    "totp",
    "recovery_code",
    "api_key",
    "apikey",
    "key",
    "code",
    "authorization",
    "mw_session",
    "mw_admin_session",
    "mw_csrf",
    "cookie",
    "session",
];

/// Environment variable names whose values are secrets (t32-cfg §8 A1).
pub fn is_secret_env_name(name: &str) -> bool {
    let n = name.to_ascii_uppercase();
    [
        "PASSWORD", "_PW", "SECRET", "TOKEN", "_KEY", "API_KEY", "DSN",
    ]
    .iter()
    .any(|s| n.ends_with(s))
}

/// The redaction state: secret values known to the process and the admin path.
#[derive(Debug, Default)]
pub struct Redactor {
    /// Literal nonempty secret values, longest first.
    secrets: RwLock<Vec<String>>,
}

impl Redactor {
    /// A redactor seeded from this process's environment.
    pub fn from_env() -> Self {
        let r = Redactor::default();
        r.add_secrets(
            std::env::vars()
                .filter(|(k, _)| k.starts_with("MW_") && is_secret_env_name(k))
                .map(|(_, v)| v),
        );
        r
    }

    /// Add literal secret values to cut wherever they appear.
    pub fn add_secrets(&self, values: impl IntoIterator<Item = String>) {
        let mut g = self
            .secrets
            .write()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        for v in values {
            let v = v.trim().to_string();
            if !v.is_empty() && !g.contains(&v) {
                g.push(v);
            }
        }
        g.sort_by_key(|s| std::cmp::Reverse(s.len()));
    }

    /// The kept value of field `name`.
    pub fn field(&self, name: &str, value: &str) -> String {
        let lower = name.to_ascii_lowercase();
        if CONTENT_NAMES.contains(&lower.as_str()) {
            return format!("[content: {} chars]", value.chars().count());
        }
        if SECRET_NAMES.iter().any(|s| lower.contains(s)) {
            return MARK.to_string();
        }
        self.text(value)
    }

    /// Free text (the event message, an ordinary field) with secrets cut out.
    pub fn text(&self, input: &str) -> String {
        let mut s = input.to_string();
        {
            let g = self
                .secrets
                .read()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            for secret in g.iter() {
                if s.contains(secret.as_str()) {
                    s = s.replace(secret.as_str(), MARK);
                }
            }
        }
        s = cut_auth_schemes(&s);
        s = cut_url_userinfo(&s);
        s = cut_key_values(&s);
        s = cut_token_shapes(&s);
        cut_admin_path(&s, &mw_admin::config::admin_path())
    }
}

fn is_token_char(c: char) -> bool {
    c.is_ascii_alphanumeric() || matches!(c, '-' | '_' | '.' | '~' | '+' | '/' | '=' | '%')
}

/// `Bearer xyz` / `Basic xyz` (any case) → `Bearer [redacted]`.
fn cut_auth_schemes(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let mut rest = s;
    loop {
        let lower = rest.to_ascii_lowercase();
        let hit = ["bearer ", "basic ", "token "]
            .iter()
            .filter_map(|p| lower.find(p).map(|i| (i, p.len())))
            .min();
        let Some((i, plen)) = hit else {
            out.push_str(rest);
            return out;
        };
        // Only at a word start.
        let at_word = i == 0 || !rest[..i].ends_with(|c: char| c.is_ascii_alphanumeric());
        out.push_str(&rest[..i + plen]);
        let after = &rest[i + plen..];
        let n = after
            .find(|c: char| !is_token_char(c))
            .unwrap_or(after.len());
        if at_word && n > 0 {
            out.push_str(MARK);
            rest = &after[n..];
        } else {
            rest = after;
        }
    }
}

/// `scheme://user:pass@host` → `scheme://[redacted]@host`.
fn cut_url_userinfo(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let mut rest = s;
    while let Some(i) = rest.find("://") {
        out.push_str(&rest[..i + 3]);
        let after = &rest[i + 3..];
        let end = after
            .find(|c: char| c == '/' || c == '?' || c == '#' || c.is_whitespace() || c == '"')
            .unwrap_or(after.len());
        let authority = &after[..end];
        if let Some(at) = authority.rfind('@') {
            out.push_str(MARK);
            out.push_str(&authority[at..]);
        } else {
            out.push_str(authority);
        }
        rest = &after[end..];
    }
    out.push_str(rest);
    out
}

/// `password=xyz`, `token: xyz`, `"secret":"xyz"`, `?code=xyz&` → value cut.
fn cut_key_values(s: &str) -> String {
    let bytes = s.as_bytes();
    let mut out = String::with_capacity(s.len());
    let mut i = 0;
    let mut copied = 0;
    while i < bytes.len() {
        // Find a key: a run of [A-Za-z0-9_-] starting at a non-key char.
        if (bytes[i].is_ascii_alphanumeric() || bytes[i] == b'_')
            && (i == 0 || !(bytes[i - 1].is_ascii_alphanumeric() || bytes[i - 1] == b'_'))
        {
            let start = i;
            let mut j = i;
            while j < bytes.len()
                && (bytes[j].is_ascii_alphanumeric() || bytes[j] == b'_' || bytes[j] == b'-')
            {
                j += 1;
            }
            let key = s[start..j].to_ascii_lowercase();
            // Separator: optional closing quote, then `=` or `:`, spaces, optional quote.
            let mut k = j;
            if k < bytes.len() && matches!(bytes[k], b'"' | b'\'') {
                k += 1;
            }
            while k < bytes.len() && bytes[k].is_ascii_whitespace() {
                k += 1;
            }
            if k < bytes.len()
                && (bytes[k] == b'=' || bytes[k] == b':')
                && SECRET_KEYS.contains(&key.as_str())
            {
                k += 1;
                while k < bytes.len() && bytes[k].is_ascii_whitespace() {
                    k += 1;
                }
                let quote = bytes.get(k).copied().filter(|b| matches!(b, b'"' | b'\''));
                if quote.is_some() {
                    k += 1;
                }
                let vstart = k;
                while k < bytes.len() {
                    let c = bytes[k];
                    let stop = if let Some(quote) = quote {
                        c == quote && (k == 0 || bytes[k - 1] != b'\\')
                    } else {
                        c == b'&'
                            || c == b';'
                            || c == b','
                            || c == b' '
                            || c == b'"'
                            || c == b'\''
                            || c == b'}'
                            || c == b')'
                            || c == b'\n'
                    };
                    if stop {
                        break;
                    }
                    k += 1;
                }
                let value = &s[vstart..k];
                if !value.is_empty() {
                    out.push_str(&s[copied..vstart]);
                    out.push_str(MARK);
                    copied = k;
                }
                i = k.max(j);
                continue;
            }
            i = j;
            continue;
        }
        i += 1;
    }
    out.push_str(&s[copied..]);
    out
}

/// JWT-shaped (`eyJ….….…`), `mwk_…` API keys and long opaque tokens (≥ 32
/// characters of base64url/hex with both letters and digits).
fn cut_token_shapes(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let mut word = String::new();
    let flush = |word: &mut String, out: &mut String| {
        if looks_like_token(word) {
            out.push_str(MARK);
        } else {
            out.push_str(word);
        }
        word.clear();
    };
    for c in s.chars() {
        if c.is_ascii_alphanumeric() || matches!(c, '-' | '_' | '.') {
            word.push(c);
        } else {
            flush(&mut word, &mut out);
            out.push(c);
        }
    }
    flush(&mut word, &mut out);
    out
}

fn looks_like_token(w: &str) -> bool {
    if w.starts_with("eyJ") && w.matches('.').count() >= 2 {
        return true;
    }
    if w.starts_with("mwk_") && w.len() > 8 {
        return true;
    }
    if w.len() >= 32 && !w.contains('.') {
        let letters = w.bytes().any(|b| b.is_ascii_alphabetic());
        let digits = w.bytes().any(|b| b.is_ascii_digit());
        return letters && digits;
    }
    false
}

/// Every occurrence of the admin path followed by a boundary → [`ADMIN_MARK`].
fn cut_admin_path(s: &str, admin: &str) -> String {
    if admin.len() < 2 || !s.contains(admin) {
        return s.to_string();
    }
    let mut out = String::with_capacity(s.len());
    let mut rest = s;
    while let Some(i) = rest.find(admin) {
        let after = &rest[i + admin.len()..];
        let boundary = after.is_empty()
            || after.starts_with(|c: char| {
                matches!(
                    c,
                    '/' | '?' | '#' | '"' | '\'' | ' ' | ',' | ')' | ']' | '}'
                ) || c.is_whitespace()
            });
        out.push_str(&rest[..i]);
        if boundary {
            out.push_str(ADMIN_MARK);
        } else {
            out.push_str(admin);
        }
        rest = after;
    }
    out.push_str(rest);
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn content_fields_keep_only_their_length() {
        let r = Redactor::default();
        assert_eq!(
            r.field("subject", "Quarterly numbers"),
            "[content: 17 chars]"
        );
        assert_eq!(r.field("Body", "hello"), "[content: 5 chars]");
    }

    #[test]
    fn secret_fields_are_replaced_whole() {
        let r = Redactor::default();
        for name in [
            "password",
            "admin_password",
            "refresh_token",
            "Authorization",
            "set_cookie",
            "sentry_dsn",
        ] {
            assert_eq!(r.field(name, "hunter2-value"), MARK, "{name}");
        }
        assert_eq!(r.field("status", "200"), "200");
    }

    #[test]
    fn secret_env_values_are_cut_anywhere() {
        let r = Redactor::default();
        r.add_secrets(["S3cr3t-Admin-Pw".to_string(), "abc".to_string()]);
        assert_eq!(
            r.text("login with S3cr3t-Admin-Pw failed"),
            format!("login with {MARK} failed")
        );
        assert_eq!(r.text("abc"), MARK);
        assert!(is_secret_env_name("MW_ADMIN_PASSWORD"));
        assert!(is_secret_env_name("MW_SERVER_KEY"));
        assert!(is_secret_env_name("MW_SENTRY_DSN"));
        assert!(is_secret_env_name("MW_DIRECTORY_BIND_PW"));
        assert!(!is_secret_env_name("MW_BASE_PATH"));
    }

    #[test]
    fn credentials_in_text_are_cut() {
        let r = Redactor::default();
        let t = r.text("Authorization: Bearer abc.def-ghi and Basic dXNlcjpwYXNz");
        assert!(
            !t.contains("abc.def-ghi") && !t.contains("dXNlcjpwYXNz"),
            "{t}"
        );
        let t = r.text("GET /oauth/cb?code=XYZ123abc&state=ok password=hunter2 token: t0k3n");
        assert!(
            !t.contains("XYZ123abc") && !t.contains("hunter2") && !t.contains("t0k3n"),
            "{t}"
        );
        assert!(t.contains("state=ok"), "{t}");
        let t = r.text(r#"{"client_secret":"shh-value","name":"x"}"#);
        assert!(
            !t.contains("shh-value") && t.contains("\"name\":\"x\""),
            "{t}"
        );
        let t = r.text("Cookie: mw_session=deadbeef; mw_admin_session=cafebabe");
        assert!(!t.contains("deadbeef") && !t.contains("cafebabe"), "{t}");
        let t = r.text("dial imaps://alice:pw123@mail.example:993/INBOX");
        assert!(
            !t.contains("pw123") && !t.contains("alice:") && t.contains("@mail.example:993"),
            "{t}"
        );
        let t = r.text("jwt eyJhbGciOiJIUzI1NiJ9.eyJzdWIiOiIxIn0.c2ln key mwk_live_abcdef123");
        assert!(!t.contains("eyJhbGci") && !t.contains("mwk_live"), "{t}");
        let t = r.text("opaque 0123456789abcdef0123456789abcdef01 ok");
        assert!(!t.contains("0123456789abcdef0123"), "{t}");
        assert_eq!(r.text("code: 123456"), format!("code: {MARK}"));
        for text in [
            "password = 'two words'",
            "'token': 'secret value'",
            "key=123456",
        ] {
            let safe = r.text(text);
            assert!(
                !safe.contains("two words")
                    && !safe.contains("secret value")
                    && !safe.contains("123456"),
                "{safe}"
            );
        }
    }

    #[test]
    fn the_admin_path_is_named_not_shown() {
        assert_eq!(
            cut_admin_path("GET /ops-7f3k/users 200", "/ops-7f3k"),
            format!("GET {ADMIN_MARK}/users 200")
        );
        assert_eq!(
            cut_admin_path("GET /ops-7f3k", "/ops-7f3k"),
            format!("GET {ADMIN_MARK}")
        );
        // A longer segment that merely starts with it is not the console.
        assert_eq!(cut_admin_path("/ops-7f3kx/a", "/ops-7f3k"), "/ops-7f3kx/a");
    }
}
