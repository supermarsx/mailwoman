//! The IMAP extension compliance table: every extension this client knows,
//! its RFC, how far Mailwoman supports it, and where it is used.
//!
//! This table is the single source for `docs/protocols/imap-extensions.md`.
//! `scripts/imap/gen-matrix.sh` (or `.ps1`) rewrites the document from
//! [`markdown`], and `tests/extensions_doc.rs` fails when the two differ.

/// How far an extension is supported.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Support {
    /// Implemented and used whenever the server advertises it.
    Used,
    /// The capability is read or the replies parse, but no code path uses it
    /// yet.
    Parsed,
    /// Not implemented: the client behaves as if the server lacked it.
    Missing,
}

impl Support {
    pub fn label(self) -> &'static str {
        match self {
            Support::Used => "used",
            Support::Parsed => "parsed, not used",
            Support::Missing => "not implemented",
        }
    }
}

/// One row of the compliance table.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Extension {
    /// The capability name as advertised (`AUTH=` mechanisms included).
    pub name: &'static str,
    /// The defining document, e.g. `RFC 7162`.
    pub rfc: &'static str,
    pub support: Support,
    /// What Mailwoman does with it, or what is missing.
    pub notes: &'static str,
}

const fn ext(
    name: &'static str,
    rfc: &'static str,
    support: Support,
    notes: &'static str,
) -> Extension {
    Extension {
        name,
        rfc,
        support,
        notes,
    }
}

use Support::{Missing, Parsed, Used};

/// The table, ordered by name.
pub const EXTENSIONS: &[Extension] = &[
    ext(
        "ACL",
        "RFC 4314",
        Used,
        "Folder sharing rights (GETACL, SETACL, DELETEACL, LISTRIGHTS, MYRIGHTS).",
    ),
    ext(
        "APPENDLIMIT",
        "RFC 7889",
        Missing,
        "Planned: refuse an upload that is too large before sending it.",
    ),
    ext(
        "AUTH=OAUTHBEARER",
        "RFC 7628",
        Parsed,
        "The SASL frame exists; the server never offers OAuth sign-in to an IMAP upstream.",
    ),
    ext(
        "AUTH=PLAIN",
        "RFC 4616",
        Used,
        "Password sign-in when SCRAM is not offered.",
    ),
    ext(
        "AUTH=SCRAM-SHA-1",
        "RFC 5802",
        Missing,
        "Planned only behind the weak-mechanism setting.",
    ),
    ext(
        "AUTH=SCRAM-SHA-256",
        "RFC 7677",
        Used,
        "Preferred password sign-in.",
    ),
    ext(
        "AUTH=SCRAM-SHA-256-PLUS",
        "RFC 7677, RFC 5929",
        Used,
        "Preferred over TLS: binds the sign-in to the server certificate (tls-server-end-point).",
    ),
    ext(
        "AUTH=SCRAM-SHA-512",
        "draft-melnikov-scram-sha-512",
        Missing,
        "Planned.",
    ),
    ext(
        "AUTH=XOAUTH2",
        "Google, Microsoft",
        Parsed,
        "The SASL frame exists; the server never offers OAuth sign-in to an IMAP upstream.",
    ),
    ext(
        "BINARY",
        "RFC 3516",
        Missing,
        "Planned: fetch decoded parts.",
    ),
    ext(
        "CATENATE",
        "RFC 4469",
        Missing,
        "Planned: forward as attachment without downloading.",
    ),
    ext(
        "COMPRESS=DEFLATE",
        "RFC 4978",
        Parsed,
        "Capability read; the DEFLATE stream exists but is not switched on.",
    ),
    ext(
        "CONDSTORE",
        "RFC 7162",
        Parsed,
        "Capability read; the sync path does not reach it yet.",
    ),
    ext(
        "CREATE-SPECIAL-USE",
        "RFC 6154",
        Missing,
        "Planned: create Drafts, Sent, Trash with their role.",
    ),
    ext(
        "ENABLE",
        "RFC 5161",
        Used,
        "Turns on the extensions below that need it.",
    ),
    ext(
        "ESEARCH",
        "RFC 4731",
        Parsed,
        "Replies parse; searches do not ask for them yet.",
    ),
    ext("ID", "RFC 2971", Used, "Sends the client name and version."),
    ext("IDLE", "RFC 2177", Used, "Push for INBOX only."),
    ext(
        "IMAP4rev2",
        "RFC 9051",
        Parsed,
        "Capability read; not enabled until ESEARCH replies are used.",
    ),
    ext(
        "INPROGRESS",
        "RFC 9585",
        Missing,
        "Planned: show progress of long commands in diagnostics.",
    ),
    ext(
        "LIST-EXTENDED",
        "RFC 5258",
        Parsed,
        "Extended LIST replies parse; no selection options are sent.",
    ),
    ext(
        "LIST-STATUS",
        "RFC 5819",
        Used,
        "Folder list and counts in one command.",
    ),
    ext(
        "LITERAL+",
        "RFC 7888",
        Missing,
        "Planned: uploads without waiting for the server's go-ahead.",
    ),
    ext(
        "LITERAL-",
        "RFC 7888",
        Missing,
        "Planned: as LITERAL+, up to 4096 bytes.",
    ),
    ext(
        "LOGINDISABLED",
        "RFC 9051",
        Missing,
        "Planned: never send LOGIN when the server forbids it.",
    ),
    ext(
        "MESSAGELIMIT",
        "RFC 9738",
        Missing,
        "Planned: keep batches within the server's limit.",
    ),
    ext(
        "METADATA",
        "RFC 5464",
        Used,
        "Server and folder annotations.",
    ),
    ext(
        "MOVE",
        "RFC 6851",
        Used,
        "Moves messages; COPY plus delete otherwise.",
    ),
    ext(
        "MULTIAPPEND",
        "RFC 3502",
        Missing,
        "Planned: faster imports.",
    ),
    ext(
        "NAMESPACE",
        "RFC 2342",
        Missing,
        "Planned: shared and other users' folders.",
    ),
    ext(
        "NOTIFY",
        "RFC 5465",
        Missing,
        "Planned: one connection watches all subscribed folders.",
    ),
    ext(
        "OBJECTID",
        "RFC 8474",
        Parsed,
        "Capability read; EMAILID and THREADID are not fetched.",
    ),
    ext(
        "PARTIAL",
        "RFC 9394",
        Missing,
        "Planned: paged search and fetch for large folders.",
    ),
    ext(
        "PREVIEW",
        "RFC 8970",
        Missing,
        "Planned: list previews without fetching bodies.",
    ),
    ext(
        "QRESYNC",
        "RFC 7162",
        Parsed,
        "Capability read; the sync path does not reach it yet.",
    ),
    ext(
        "QUOTA",
        "RFC 9208",
        Missing,
        "Planned: show the upstream mailbox quota.",
    ),
    ext(
        "REPLACE",
        "RFC 8508",
        Missing,
        "Planned: draft saves in one command.",
    ),
    ext(
        "SASL-IR",
        "RFC 4959",
        Missing,
        "Planned: one round trip less at sign-in.",
    ),
    ext("SAVEDATE", "RFC 8514", Missing, "Planned."),
    ext(
        "SEARCHRES",
        "RFC 5182",
        Missing,
        "Planned: reuse a search result in the next command.",
    ),
    ext(
        "SORT",
        "RFC 5256",
        Parsed,
        "Implemented; only tests call it.",
    ),
    ext(
        "SPECIAL-USE",
        "RFC 6154",
        Used,
        "Folder roles from LIST attributes.",
    ),
    ext("STARTTLS", "RFC 9051", Used, "Upgrade to TLS on port 143."),
    ext(
        "STATUS=SIZE",
        "RFC 8438",
        Missing,
        "Planned: folder sizes for diagnostics and quota display.",
    ),
    ext(
        "THREAD=ORDEREDSUBJECT",
        "RFC 5256",
        Parsed,
        "Implemented; only tests call it.",
    ),
    ext(
        "THREAD=REFERENCES",
        "RFC 5256",
        Parsed,
        "Implemented; only tests call it.",
    ),
    ext("UIDONLY", "RFC 9586", Missing, "Planned."),
    ext(
        "UIDPLUS",
        "RFC 4315",
        Used,
        "New UIDs after upload, copy and move.",
    ),
    ext(
        "UNSELECT",
        "RFC 3691",
        Used,
        "Leaves a folder before renaming or deleting it, without expunging.",
    ),
    ext(
        "UTF8=ACCEPT",
        "RFC 6855",
        Missing,
        "Planned: UTF-8 folder names; modified UTF-7 otherwise.",
    ),
    ext(
        "WITHIN",
        "RFC 5032",
        Missing,
        "Planned: date-relative search.",
    ),
];

/// Look up a row by capability name (case-insensitive).
pub fn find(name: &str) -> Option<&'static Extension> {
    EXTENSIONS
        .iter()
        .find(|e| e.name.eq_ignore_ascii_case(name))
}

/// The compliance document, generated from [`EXTENSIONS`].
pub fn markdown() -> String {
    let count = |s| EXTENSIONS.iter().filter(|e| e.support == s).count();
    let mut out = String::new();
    out.push_str("# IMAP extensions\n\n");
    out.push_str(
        "<!-- Generated from crates/mw-imap/src/extensions.rs by scripts/imap/gen-matrix.sh. \
         Do not edit by hand. -->\n\n",
    );
    out.push_str(
        "Which IMAP extensions Mailwoman's IMAP client supports when it talks to an upstream \
         server. Every extension is used only when the server advertises it; without it the \
         client falls back to plain IMAP4rev1 behaviour.\n\n",
    );
    out.push_str(&format!(
        "{} extensions: {} used, {} parsed but not used, {} not implemented.\n\n",
        EXTENSIONS.len(),
        count(Used),
        count(Parsed),
        count(Missing),
    ));
    out.push_str("| Extension | Defined in | Support | Notes |\n");
    out.push_str("|---|---|---|---|\n");
    for e in EXTENSIONS {
        out.push_str(&format!(
            "| `{}` | {} | {} | {} |\n",
            e.name,
            e.rfc,
            e.support.label(),
            e.notes
        ));
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn table_is_sorted_and_unique() {
        for pair in EXTENSIONS.windows(2) {
            assert!(
                pair[0].name < pair[1].name,
                "{} must sort before {}",
                pair[0].name,
                pair[1].name
            );
        }
    }

    #[test]
    fn rows_are_complete() {
        for e in EXTENSIONS {
            assert!(!e.rfc.is_empty() && !e.notes.is_empty(), "{}", e.name);
            assert!(!e.notes.contains('|'), "{} notes break the table", e.name);
        }
    }

    #[test]
    fn every_backend_capability_flag_has_a_row() {
        // The atoms CapabilitySet folds into BackendCaps.
        for name in [
            "IMAP4rev2",
            "QRESYNC",
            "CONDSTORE",
            "UIDPLUS",
            "MOVE",
            "SPECIAL-USE",
            "LIST-STATUS",
            "IDLE",
            "OBJECTID",
            "ESEARCH",
            "ENABLE",
            "ID",
            "COMPRESS=DEFLATE",
            "SORT",
            "THREAD=REFERENCES",
            "THREAD=ORDEREDSUBJECT",
            "ACL",
            "METADATA",
            "AUTH=PLAIN",
            "AUTH=XOAUTH2",
            "AUTH=OAUTHBEARER",
            "AUTH=SCRAM-SHA-256",
            "AUTH=SCRAM-SHA-256-PLUS",
        ] {
            assert!(find(name).is_some(), "{name} has no row");
        }
    }

    #[test]
    fn markdown_lists_every_row() {
        let doc = markdown();
        for e in EXTENSIONS {
            assert!(doc.contains(&format!("| `{}` |", e.name)), "{}", e.name);
        }
    }
}
