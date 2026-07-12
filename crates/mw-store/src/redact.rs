//! Privacy-typed wrapper for PII (subjects, bodies, addresses) — plan §1.13,
//! SPEC §21.1.
//!
//! IMAP/POP3 wire tracing is allowed but must be structurally safe from leaking
//! message content. [`Redacted<T>`] wraps a value so that its `Debug` and
//! `Display` render `<redacted>` regardless of the inner type: a subject or
//! address logged through a `tracing` field simply cannot print its payload.
//! The plaintext is reachable only via the explicit [`Redacted::reveal`] /
//! [`Redacted::into_inner`] accessors, which read as intent at the call site.

use std::fmt;

/// A value whose plaintext never appears in `Debug`/`Display` output.
///
/// Use it for anything derived from message content — subjects, body previews,
/// email addresses — that might pass through a log field. Equality and hashing
/// still operate on the inner value (so redacted values remain usable as map
/// keys and in assertions); only the rendered forms are masked.
#[derive(Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Redacted<T>(T);

impl<T> Redacted<T> {
    /// Wrap `value` so its content will not be rendered by `Debug`/`Display`.
    pub fn new(value: T) -> Self {
        Self(value)
    }

    /// Borrow the underlying plaintext. Naming the call makes the reveal
    /// explicit at the use site (e.g. when actually sending it upstream).
    pub fn reveal(&self) -> &T {
        &self.0
    }

    /// Consume the wrapper and return the underlying plaintext.
    pub fn into_inner(self) -> T {
        self.0
    }
}

impl<T> fmt::Debug for Redacted<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("<redacted>")
    }
}

impl<T> fmt::Display for Redacted<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("<redacted>")
    }
}

impl<T> From<T> for Redacted<T> {
    fn from(value: T) -> Self {
        Self(value)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn debug_and_display_hide_content() {
        let secret = Redacted::new("Re: quarterly numbers");
        assert_eq!(format!("{secret:?}"), "<redacted>");
        assert_eq!(format!("{secret}"), "<redacted>");
        // The payload must not appear anywhere in either rendering.
        assert!(!format!("{secret:?} {secret}").contains("quarterly"));
    }

    #[test]
    fn reveal_and_into_inner_return_plaintext() {
        let addr = Redacted::new("alice@example.org".to_string());
        assert_eq!(addr.reveal(), "alice@example.org");
        assert_eq!(addr.into_inner(), "alice@example.org");
    }

    #[test]
    fn equality_uses_inner_value() {
        assert_eq!(Redacted::new(42), Redacted::from(42));
        assert_ne!(Redacted::new(1), Redacted::new(2));
    }
}
