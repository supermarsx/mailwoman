//! Mail-family method dispatch (t16 J1–J5). Reached from `handle_jmap`'s core
//! dispatch (`jmap.rs`) for any mail-ext method, after the explicit `Email/*`
//! arms so `Email/get`/`set`/`query` still win over the `Email/copy|import|parse`
//! additions here (mirrors `pim/dispatch.rs` / `security/dispatch.rs`).

use serde_json::{Value, json};

use crate::engine::Engine;

/// The whole-family prefixes routed to [`Engine::dispatch_mail_ext`]: every
/// method under these belongs to the mail-ext surface.
const MAIL_EXT_FAMILIES: &[&str] = &["Thread/", "SearchSnippet/", "VacationResponse/", "Quota/"];

/// The individual `Email/*` methods the mail-ext surface adds (the rest of the
/// `Email/` family is handled explicitly in the core dispatch and must NOT route
/// here).
const EMAIL_EXT_METHODS: &[&str] = &["Email/copy", "Email/import", "Email/parse"];

/// Whether `method` is answered by the mail-ext dispatch (t16 J1–J5).
pub fn is_mail_ext_method(method: &str) -> bool {
    EMAIL_EXT_METHODS.contains(&method)
        || MAIL_EXT_FAMILIES.iter().any(|fam| method.starts_with(fam))
}

impl Engine {
    /// Dispatch one resolved mail-ext method call. Reached from `handle_jmap`
    /// for any method [`is_mail_ext_method`] accepts.
    pub(crate) async fn dispatch_mail_ext(
        &self,
        account_id: &str,
        name: &str,
        args: &Value,
    ) -> Value {
        match name {
            // ── Threads (RFC 8621 §3) — real JWZ threads from `thread.rs` ──
            "Thread/get" => self.thread_get(account_id, args).await,
            "Thread/changes" => self.thread_changes(account_id, args).await,
            // ── Search snippets (RFC 8621 §5) ──
            "SearchSnippet/get" => self.search_snippet_get(account_id, args).await,
            // ── Vacation response (RFC 8621 §8 — per-account singleton) ──
            "VacationResponse/get" => self.vacation_response_get(account_id, args).await,
            "VacationResponse/set" => self.vacation_response_set(account_id, args).await,
            // ── Quota (RFC 9425) ──
            "Quota/get" => self.quota_get(account_id, args).await,
            // ── Email copy / import / parse (RFC 8621 §4.7–§4.9) ──
            "Email/copy" => self.email_copy(account_id, args).await,
            "Email/import" => self.email_import(account_id, args).await,
            "Email/parse" => self.email_parse(account_id, args).await,
            other => json!({
                "type": "unknownMethod",
                "description": format!("engine does not implement mail method {other}")
            }),
        }
    }
}
