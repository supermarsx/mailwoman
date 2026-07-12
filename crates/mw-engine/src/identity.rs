//! Sending identities (plan §0.7, §2.1) — multiple from-addresses, server-pulled
//! allowed-froms, and signature templates.
//!
//! ## Scaffolder note (e0)
//! e0 freezes the [`Identity`] shape returned by `Identity/get`/`Identity/query`;
//! e9 fills configured + server-pulled allowed-froms and the signature store.

use serde::{Deserialize, Serialize};

/// A sending identity (§2.1).
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Identity {
    pub id: String,
    pub name: String,
    pub email: String,
    pub reply_to: Option<String>,
    pub signature_html: Option<String>,
    pub signature_text: Option<String>,
    pub sent_mailbox_id: Option<String>,
}
