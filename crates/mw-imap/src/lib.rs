#![forbid(unsafe_code)]
//! `mw-imap` — a hardened IMAP4rev2 (RFC 9051) client with RFC 3501 fallback,
//! built directly over `tokio` + `tokio-rustls` (ring provider) with
//! `imap-proto` as the response parser and a hand-written command/tag layer
//! (plan §1.1). It implements the frozen [`mw_engine::backend::AccountBackend`]
//! seam as [`ImapBackend`].
//!
//! ## Layers
//! - [`transport`] — TCP + implicit/STARTTLS/plaintext stream.
//! - [`connection`] — the tagged command engine (tags, demux, continuations,
//!   the streaming `imap-proto` read loop).
//! - [`sasl`] — PLAIN / LOGIN / XOAUTH2 initial-response frames.
//! - [`caps`] — `CAPABILITY` → [`mw_engine::backend::BackendCaps`].
//! - [`session`] — high-level commands and response → backend-type mapping.
//! - [`sync`] — the QRESYNC → CONDSTORE → UID-window fallback ladder (§1.8).
//! - [`backend`] — [`ImapBackend`] wiring it all behind the trait.
//!
//! Feature-detection drives every extension: nothing is assumed present, and
//! the sync ladder + MOVE/COPY fallback degrade to whatever the server offers.

pub mod backend;
pub mod caps;
pub mod connection;
pub mod error;
pub mod sasl;
pub mod session;
pub mod sync;
mod tls;
pub mod transport;

pub use backend::{ImapBackend, ImapConfig};
pub use error::{ImapError, ImapResult};
pub use session::{Credentials, FetchItem, SelectMode, SelectResult, Session};
pub use transport::{ImapStream, TlsMode};

/// Fuzz/robustness entry point: repeatedly parse a byte buffer as a stream of
/// server responses, exactly as [`connection::Connection`] does, and must never
/// panic on arbitrary input.
///
/// Drives the `cargo-fuzz` target (`fuzz/fuzz_targets/parse_response.rs`) and
/// the in-crate corpus smoke test.
pub fn fuzz_parse_responses(mut data: &[u8]) {
    // Bound the iteration count so a pathological zero-consuming loop can't spin.
    for _ in 0..10_000 {
        match imap_proto::Response::from_bytes(data) {
            Ok((remaining, resp)) => {
                let _owned = resp.into_owned();
                if remaining.len() == data.len() {
                    // No forward progress; stop to avoid an infinite loop.
                    break;
                }
                data = remaining;
                if data.is_empty() {
                    break;
                }
            }
            // Incomplete or malformed: nothing more to do with this buffer.
            Err(_) => break,
        }
    }
}

#[cfg(test)]
mod smoke {
    use super::*;

    /// A corpus of realistic and adversarial server lines must parse without
    /// panicking (the fuzz invariant, runnable under plain `cargo test`).
    #[test]
    fn corpus_never_panics() {
        let corpus: &[&[u8]] = &[
            b"* OK [CAPABILITY IMAP4rev1 IMAP4rev2 QRESYNC CONDSTORE MOVE UIDPLUS] ready\r\n",
            b"* CAPABILITY IMAP4rev1 STARTTLS AUTH=PLAIN AUTH=XOAUTH2\r\n",
            b"* LIST (\\HasNoChildren \\Sent) \"/\" \"Sent\"\r\n",
            b"* STATUS \"INBOX\" (MESSAGES 3 UNSEEN 1 UIDNEXT 12 UIDVALIDITY 99)\r\n",
            b"* 1 FETCH (UID 5 FLAGS (\\Seen) MODSEQ (42) INTERNALDATE \"01-Jan-2026 00:00:00 +0000\")\r\n",
            b"* VANISHED (EARLIER) 41,43:45\r\n",
            b"* OK [HIGHESTMODSEQ 90210]\r\n",
            b"A0001 OK [APPENDUID 99 7] APPEND completed\r\n",
            b"A0002 OK [COPYUID 99 5 8] Move completed\r\n",
            b"+ ready for literal\r\n",
            b"* BYE logging out\r\n",
            // Adversarial / truncated inputs.
            b"* 1 FETCH (BODY[] {5}\r\nhel",
            b"{{{{{{{{",
            b"* LIST (",
            b"\xff\xfe\x00\x01 garbage",
            b"",
            b"* 999999999999999999999999 EXISTS\r\n",
        ];
        for bytes in corpus {
            fuzz_parse_responses(bytes);
        }
    }

    #[test]
    fn uid_set_formatting_collapses_runs() {
        assert_eq!(session::format_uid_set(&[1, 2, 3, 5, 9, 10]), "1:3,5,9:10");
        assert_eq!(session::format_uid_set(&[7]), "7");
        assert_eq!(session::format_uid_set(&[3, 1, 2, 2]), "1:3");
        assert_eq!(session::format_uid_set(&[]), "");
    }
}
