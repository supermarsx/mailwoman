//! Verdict family (`SecurityVerdict/get`, frozen §2.2) — server-side, all public
//! (plan §1.2). Computes the §7.3 [`SecurityVerdict`](super::types::SecurityVerdict):
//! `mail-auth` DKIM/SPF/DMARC/ARC verdicts (fixture-seeded resolver in CI),
//! Received-chain parse (`mail-parser`) + optional GeoIP (`maxminddb`), signature
//! verify via `mw-crypto` native, attachment risk (ext-vs-magic), and anomalies.
//! Lazy; cached in `security_verdicts` keyed by `emailId` + raw-hash.
//!
//! e0 skeleton — the frozen arm with a `todo!()` body. e6 wires `mail-auth`.

use serde_json::Value;

use crate::account::AccountRuntime;
use crate::engine::Engine;

impl Engine {
    /// `SecurityVerdict/get {ids}` → `{accountId,state,list:[SecurityVerdict],
    /// notFound}` (lazy; needs the raw message + DNS; cached in
    /// `security_verdicts`).
    pub(crate) async fn security_verdict_get(
        &self,
        _account_id: &str,
        _rt: &AccountRuntime,
        _args: &Value,
    ) -> Value {
        todo!("e6: mail-auth verdicts + Received-chain + signature/cert + attachment risk")
    }
}
