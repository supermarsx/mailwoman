//! `CAPABILITY` / `ENABLED` parsing and the mapping onto [`BackendCaps`].
//!
//! `imap-proto` surfaces `IMAP4rev1` as its own variant and everything else as
//! an atom (`IMAP4rev2`, `QRESYNC`, `MOVE`, `COMPRESS=DEFLATE`, …) or an
//! `AUTH=<mech>` entry. We normalise to an uppercased set so feature-detection
//! is a simple membership test, then fold it into the frozen [`BackendCaps`].

use std::collections::HashSet;

use imap_proto::Capability;
use mw_engine::backend::BackendCaps;

/// Normalised view of a server's advertised capabilities.
#[derive(Debug, Clone, Default)]
pub struct CapabilitySet {
    atoms: HashSet<String>,
    auth: HashSet<String>,
}

impl CapabilitySet {
    /// Merge a parsed `CAPABILITY`/`ENABLED` list into the set.
    pub fn extend_from(&mut self, caps: &[Capability<'_>]) {
        for cap in caps {
            match cap {
                Capability::Imap4rev1 => {
                    self.atoms.insert("IMAP4REV1".to_string());
                }
                Capability::Atom(a) => {
                    self.atoms.insert(a.to_ascii_uppercase());
                }
                Capability::Auth(m) => {
                    self.auth.insert(m.to_ascii_uppercase());
                }
            }
        }
    }

    /// Whether a bare capability atom is advertised (case-insensitive).
    pub fn has(&self, atom: &str) -> bool {
        self.atoms.contains(&atom.to_ascii_uppercase())
    }

    /// Whether a SASL mechanism (`AUTH=<mech>`) is advertised.
    pub fn has_auth(&self, mech: &str) -> bool {
        self.auth.contains(&mech.to_ascii_uppercase())
    }

    /// True once the set carries any capability (used to decide whether an
    /// explicit `CAPABILITY` command is still required after connect/login).
    pub fn is_empty(&self) -> bool {
        self.atoms.is_empty() && self.auth.is_empty()
    }

    /// Fold the normalised set into the frozen backend capability flags.
    pub fn to_backend_caps(&self) -> BackendCaps {
        BackendCaps {
            imap4rev2: self.has("IMAP4REV2"),
            qresync: self.has("QRESYNC"),
            condstore: self.has("CONDSTORE"),
            uidplus: self.has("UIDPLUS"),
            r#move: self.has("MOVE"),
            special_use: self.has("SPECIAL-USE"),
            list_status: self.has("LIST-STATUS"),
            idle: self.has("IDLE"),
            objectid: self.has("OBJECTID"),
            esearch: self.has("ESEARCH"),
            enable: self.has("ENABLE"),
            id: self.has("ID"),
            compress: self.has("COMPRESS=DEFLATE"),
            sort: self.has("SORT"),
            thread_references: self.has("THREAD=REFERENCES"),
            thread_orderedsubject: self.has("THREAD=ORDEREDSUBJECT"),
            acl: self.has("ACL"),
            metadata: self.has("METADATA") || self.has("METADATA-SERVER"),
            sasl_plain: self.has_auth("PLAIN"),
            sasl_login: self.has_auth("LOGIN"),
            sasl_xoauth2: self.has_auth("XOAUTH2"),
            sasl_oauthbearer: self.has_auth("OAUTHBEARER"),
            sasl_scram_sha256: self.has_auth("SCRAM-SHA-256"),
            sasl_scram_sha256_plus: self.has_auth("SCRAM-SHA-256-PLUS"),
        }
    }
}
