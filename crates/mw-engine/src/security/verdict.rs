//! Verdict family (`SecurityVerdict/get`, frozen §2.2) — server-side, all public
//! (plan §1.2). Computes the §7.3 [`SecurityVerdict`](super::types::SecurityVerdict):
//! `mail-auth` DKIM/SPF/DMARC/ARC verdicts (a fixture-seeded [`SeededTxtCache`] in
//! CI, the system resolver in prod), Received-chain parse (`mail-parser`),
//! signature/cert + encryption detection (via `mw-crypto` native), attachment risk
//! (ext-vs-magic), and anomalies. Lazy; cached in `security_verdicts` keyed by
//! `emailId` + raw-hash.

use std::borrow::Borrow;
use std::collections::HashMap;
use std::hash::Hash;
use std::sync::Mutex;
use std::time::Instant;

use std::net::IpAddr;

use mail_auth::common::parse::TxtRecordParser;
use mail_auth::common::verify::DomainKey;
use mail_auth::dmarc::verify::DmarcParameters;
use mail_auth::spf::verify::SpfParameters;
use mail_auth::{
    AuthenticatedMessage, DkimResult, DmarcResult, MessageAuthenticator, Parameters, ResolverCache,
    SpfOutput, SpfResult, Txt,
};
use mail_parser::{Host, MessageParser, MimeHeaders};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};

use crate::account::AccountRuntime;
use crate::engine::Engine;
use crate::security::types::{
    ArcVerdict, AttachmentRisk, AuthVerdict, DkimVerdict, DmarcVerdict, EncryptionInfo,
    ReceivedHop, SecurityVerdict, SignatureVerdict, SpfVerdict,
};

// ── A minimal in-memory TXT cache (the fixture-seeded offline resolver, §1.6) ──

/// A HashMap-backed [`ResolverCache`] for TXT records — the fixture-seeded offline
/// resolver used in CI/tests (prod passes `None` so the system resolver answers).
/// Only TXT is seeded (DKIM keys, SPF/DMARC records); the other record types use
/// `mail-auth`'s `NoCache`.
#[derive(Default)]
pub struct SeededTxtCache {
    map: Mutex<HashMap<Box<str>, Txt>>,
}

impl SeededTxtCache {
    /// Seed one TXT record (`name` is the FQDN, trailing-dot form).
    pub fn insert_txt(&self, name: &str, value: Txt) {
        self.map.lock().unwrap().insert(name.into(), value);
    }

    /// Seed a DKIM public-key record (`selector._domainkey.domain` → the
    /// `v=DKIM1; k=...; p=...` value).
    pub fn insert_dkim(&self, name: &str, record: &str) -> bool {
        match DomainKey::parse(record.as_bytes()) {
            Ok(k) => {
                self.insert_txt(name, k.into());
                true
            }
            Err(_) => false,
        }
    }

    /// Seed an SPF record (`domain` in trailing-dot form → the `v=spf1 …` value).
    pub fn insert_spf(&self, name: &str, record: &str) -> bool {
        match mail_auth::spf::Spf::parse(record.as_bytes()) {
            Ok(spf) => {
                self.insert_txt(name, spf.into());
                true
            }
            Err(_) => false,
        }
    }
}

impl ResolverCache<Box<str>, Txt> for SeededTxtCache {
    fn get<Q>(&self, name: &Q) -> Option<Txt>
    where
        Box<str>: Borrow<Q>,
        Q: Hash + Eq + ?Sized,
    {
        self.map.lock().unwrap().get(name).cloned()
    }

    fn remove<Q>(&self, name: &Q) -> Option<Txt>
    where
        Box<str>: Borrow<Q>,
        Q: Hash + Eq + ?Sized,
    {
        self.map.lock().unwrap().remove(name)
    }

    fn insert(&self, key: Box<str>, value: Txt, _valid_until: Instant) {
        self.map.lock().unwrap().insert(key, value);
    }
}

impl Engine {
    /// `SecurityVerdict/get {ids}` → `{accountId,state,list:[SecurityVerdict],
    /// notFound}` (lazy; cached in `security_verdicts` keyed by email + raw-hash).
    pub(crate) async fn security_verdict_get(
        &self,
        account_id: &str,
        _rt: &AccountRuntime,
        args: &Value,
    ) -> Value {
        let ids: Vec<String> = args
            .get("ids")
            .and_then(Value::as_array)
            .map(|a| {
                a.iter()
                    .filter_map(Value::as_str)
                    .map(String::from)
                    .collect()
            })
            .unwrap_or_default();

        // Build the system-resolver authenticator once (prod: real DNS). A build
        // failure degrades every auth result to "none" rather than erroring.
        let authenticator = MessageAuthenticator::new_system_conf().ok();

        let mut list = Vec::new();
        let mut not_found = Vec::new();
        for id in &ids {
            match self.raw_for_email(account_id, id).await {
                Some(raw) => {
                    let verdict = self
                        .verdict_for(account_id, id, &raw, authenticator.as_ref())
                        .await;
                    list.push(serde_json::to_value(verdict).unwrap_or(Value::Null));
                }
                None => not_found.push(json!(id)),
            }
        }
        let state = self.session_state(account_id).await;
        super::get_response(account_id, &state, list, not_found)
    }

    /// The raw RFC5322 bytes for a stored message (draft or received), or `None`.
    async fn raw_for_email(&self, _account_id: &str, email_id: &str) -> Option<Vec<u8>> {
        let msg = self.store().get_message(email_id).await.ok()?;
        let blob = msg.blob_ref.as_ref()?;
        self.store().get_body(blob).await.ok().flatten()
    }

    /// Compute (or read-through the cache) the verdict for one message.
    async fn verdict_for(
        &self,
        account_id: &str,
        email_id: &str,
        raw: &[u8],
        authenticator: Option<&MessageAuthenticator>,
    ) -> SecurityVerdict {
        let raw_hash = hex_sha256(raw);
        if let Ok(Some(bytes)) = self.store().get_security_verdict(email_id, &raw_hash).await
            && let Ok(v) = serde_json::from_slice::<SecurityVerdict>(&bytes)
        {
            return v;
        }
        let verdict =
            compute_verdict(email_id, raw, authenticator, None, authenticator.is_some()).await;
        if let Ok(bytes) = serde_json::to_vec(&verdict) {
            let _ = self
                .store()
                .upsert_security_verdict(&mw_store::SecurityVerdictRow {
                    email_id: email_id.to_string(),
                    account_id: account_id.to_string(),
                    raw_hash,
                    verdict_json: bytes,
                    computed_at: chrono::Utc::now().to_rfc3339(),
                })
                .await;
        }
        verdict
    }
}

/// Compute the full §7.3 verdict for a raw message. `txt_cache` seeds an offline
/// resolver (CI); `allow_network` gates the SPF/DMARC/ARC DNS lookups (DKIM uses
/// the cache-or-resolver). Public function so tests drive it directly with a
/// seeded cache (the acceptance's DKIM pass/fail fixtures).
pub(crate) async fn compute_verdict(
    email_id: &str,
    raw: &[u8],
    authenticator: Option<&MessageAuthenticator>,
    txt_cache: Option<&SeededTxtCache>,
    allow_network: bool,
) -> SecurityVerdict {
    let parsed = MessageParser::default().parse(raw);
    // The connecting-client context (IP/HELO/MAIL-FROM) SPF is evaluated against,
    // recovered from the Received chain (no live SMTP session here).
    let spf_ctx = parsed.as_ref().and_then(spf_context);

    let auth = match (authenticator, AuthenticatedMessage::parse(raw)) {
        (Some(a), Some(msg)) => {
            compute_auth(a, txt_cache, allow_network, &msg, spf_ctx.as_ref()).await
        }
        _ => empty_auth(),
    };

    let received = parsed.as_ref().map(received_chain).unwrap_or_default();
    let attachments = parsed.as_ref().map(attachment_risks).unwrap_or_default();
    let anomalies = parsed.as_ref().map(detect_anomalies).unwrap_or_default();
    let (signature, encryption) = detect_crypto(raw);

    let plain_language = plain_language(&auth, &signature, &encryption, &attachments, &anomalies);

    SecurityVerdict {
        email_id: email_id.to_string(),
        auth,
        plain_language,
        received,
        signature,
        encryption,
        attachments,
        anomalies,
    }
}

// ── Auth (DKIM/SPF/DMARC/ARC via mail-auth) ──────────────────────────────────

async fn compute_auth(
    authr: &MessageAuthenticator,
    txt_cache: Option<&SeededTxtCache>,
    allow_network: bool,
    msg: &AuthenticatedMessage<'_>,
    spf_ctx: Option<&SpfContext>,
) -> AuthVerdict {
    // DKIM — always (cache first, resolver on miss).
    let dkim_out = match txt_cache {
        Some(c) => {
            authr
                .verify_dkim(Parameters::new(msg).with_txt_cache(c))
                .await
        }
        None => authr.verify_dkim(Parameters::new(msg)).await,
    };
    let dkim = dkim_out
        .first()
        .map(|o| {
            let sig = o.signature();
            DkimVerdict {
                result: dkim_result_str(o.result()).to_string(),
                domain: sig.map(|s| s.d.clone()),
                selector: sig.map(|s| s.s.clone()),
            }
        })
        .unwrap_or(DkimVerdict {
            result: "none".into(),
            domain: None,
            selector: None,
        });

    // ARC — network-gated (a full chain walk needs the ARC-signing DNS keys).
    let arc = if allow_network {
        let arc_out = match txt_cache {
            Some(c) => {
                authr
                    .verify_arc(Parameters::new(msg).with_txt_cache(c))
                    .await
            }
            None => authr.verify_arc(Parameters::new(msg)).await,
        };
        ArcVerdict {
            result: dkim_result_str(arc_out.result()).to_string(),
            chain_length: arc_out.sets().len() as i64,
        }
    } else {
        ArcVerdict {
            result: "none".into(),
            chain_length: 0,
        }
    };

    // SPF/DMARC — network-gated (they need the sender IP + the domain's records).
    let from_domain = domain_of(msg.from());
    // SPF: evaluate the connecting-client IP (from the Received chain) against the
    // MAIL-FROM domain's published `v=spf1` record. Without a client context or
    // when offline, the result stays "none". The native `SpfOutput` is reused for
    // DMARC's SPF-alignment leg so the two never disagree.
    let spf_out = if allow_network {
        match spf_ctx {
            Some(ctx) => compute_spf(authr, txt_cache, ctx).await,
            None => SpfOutput::new(String::new()).with_result(SpfResult::None),
        }
    } else {
        SpfOutput::new(String::new()).with_result(SpfResult::None)
    };
    let spf = SpfVerdict {
        result: spf_result_str(spf_out.result()).to_string(),
        domain: spf_ctx
            .map(|c| c.domain.clone())
            .or_else(|| from_domain.clone()),
    };
    let mut dmarc = DmarcVerdict {
        result: "none".into(),
        policy: None,
        aligned: false,
    };
    if allow_network {
        // DMARC always consults the published policy for the From domain, aligning
        // the real DKIM + SPF results computed above.
        if let Some(fd) = &from_domain {
            let dmarc_out = match txt_cache {
                Some(c) => {
                    authr
                        .verify_dmarc(
                            Parameters::new(DmarcParameters::new(msg, &dkim_out, fd, &spf_out))
                                .with_txt_cache(c),
                        )
                        .await
                }
                None => {
                    authr
                        .verify_dmarc(DmarcParameters::new(msg, &dkim_out, fd, &spf_out))
                        .await
                }
            };
            let aligned = matches!(dmarc_out.dkim_result(), DmarcResult::Pass)
                || matches!(dmarc_out.spf_result(), DmarcResult::Pass);
            // DMARC passes iff an aligned authenticated identifier passed.
            let combined = if aligned {
                "pass"
            } else if matches!(dmarc_out.dkim_result(), DmarcResult::None)
                && matches!(dmarc_out.spf_result(), DmarcResult::None)
            {
                "none"
            } else {
                "fail"
            };
            dmarc = DmarcVerdict {
                result: combined.to_string(),
                policy: Some(policy_str(&dmarc_out).to_string()),
                aligned,
            };
        }
    }

    AuthVerdict {
        dkim,
        spf,
        dmarc,
        arc,
    }
}

/// The connecting-client context SPF is evaluated against, recovered from the
/// message's `Received` chain (no live SMTP session is available server-side).
#[derive(Debug, Clone)]
struct SpfContext {
    /// The connecting client's IP.
    ip: IpAddr,
    /// The HELO/EHLO identity (falls back to the `from` hostname).
    helo: String,
    /// The MAIL-FROM address (falls back to `postmaster@<from-domain>`).
    mail_from: String,
    /// The MAIL-FROM domain (what SPF actually checks).
    domain: String,
}

/// Recover the SPF client context from the parsed message: the most-recent
/// `Received` hop that carries a `from` IP is the client that connected to our
/// boundary MX; its `helo`/`from` host is the HELO identity; the envelope
/// return-path (or the `From` domain) supplies the MAIL-FROM. `None` when no
/// Received hop exposes an IP (SPF then stays "none").
fn spf_context(msg: &mail_parser::Message) -> Option<SpfContext> {
    let hop = msg.received_all().find(|r| r.from_ip().is_some())?;
    let ip = hop.from_ip()?;
    let helo = hop
        .helo()
        .map(host_str)
        .or_else(|| hop.from.as_ref().map(host_str))
        .unwrap_or_default();
    // Envelope MAIL-FROM (Return-Path) is authoritative; else From-domain postmaster.
    let from_domain = first_addr_domain(msg.from());
    let (mail_from, domain) = match msg.return_address() {
        Some(rp) if rp.contains('@') => {
            let d = rp.rsplit_once('@').map(|(_, d)| d.to_lowercase());
            (rp.to_string(), d.or_else(|| from_domain.clone()))
        }
        _ => match &from_domain {
            Some(d) => (format!("postmaster@{d}"), Some(d.clone())),
            None => (String::new(), None),
        },
    };
    Some(SpfContext {
        ip,
        helo,
        mail_from,
        domain: domain.unwrap_or_default(),
    })
}

/// Run a single SPF `check_host` for the recovered client context, honouring the
/// seeded TXT cache (CI) or the system resolver (prod). Returns the native
/// `SpfOutput` so callers read both the string result and reuse it for DMARC.
async fn compute_spf(
    authr: &MessageAuthenticator,
    txt_cache: Option<&SeededTxtCache>,
    ctx: &SpfContext,
) -> SpfOutput {
    if ctx.domain.is_empty() {
        return SpfOutput::new(String::new()).with_result(SpfResult::None);
    }
    let params = SpfParameters::verify_mail_from(ctx.ip, &ctx.helo, "", &ctx.mail_from);
    match txt_cache {
        Some(c) => {
            authr
                .verify_spf(Parameters::new(params).with_txt_cache(c))
                .await
        }
        None => authr.verify_spf(params).await,
    }
}

fn spf_result_str(r: SpfResult) -> &'static str {
    match r {
        SpfResult::Pass => "pass",
        SpfResult::Fail => "fail",
        SpfResult::SoftFail => "softfail",
        SpfResult::Neutral => "neutral",
        SpfResult::TempError => "temperror",
        SpfResult::PermError => "permerror",
        SpfResult::None => "none",
    }
}

fn empty_auth() -> AuthVerdict {
    AuthVerdict {
        dkim: DkimVerdict {
            result: "none".into(),
            domain: None,
            selector: None,
        },
        spf: SpfVerdict {
            result: "none".into(),
            domain: None,
        },
        dmarc: DmarcVerdict {
            result: "none".into(),
            policy: None,
            aligned: false,
        },
        arc: ArcVerdict {
            result: "none".into(),
            chain_length: 0,
        },
    }
}

fn dkim_result_str(r: &DkimResult) -> &'static str {
    match r {
        DkimResult::Pass => "pass",
        DkimResult::Fail(_) => "fail",
        DkimResult::Neutral(_) => "neutral",
        DkimResult::PermError(_) => "permerror",
        DkimResult::TempError(_) => "temperror",
        DkimResult::None => "none",
    }
}

fn policy_str(out: &mail_auth::DmarcOutput) -> &'static str {
    // `Policy` renders via Display as "none"/"quarantine"/"reject"; normalize.
    match format!("{:?}", out.policy()).to_lowercase() {
        s if s.contains("reject") => "reject",
        s if s.contains("quarantine") => "quarantine",
        _ => "none",
    }
}

fn domain_of(addr: &str) -> Option<String> {
    addr.rsplit_once('@').map(|(_, d)| d.to_lowercase())
}

// ── Received chain (mail-parser) ─────────────────────────────────────────────

fn received_chain(msg: &mail_parser::Message) -> Vec<ReceivedHop> {
    let hops: Vec<&mail_parser::Received> = msg.received_all().collect();
    let mut out = Vec::new();
    for (i, r) in hops.iter().enumerate() {
        let ts = r.date.as_ref().map(|d| d.to_rfc3339());
        // Delay from the NEXT-older hop (received chains list newest-first).
        let delay_ms = match (
            r.date.as_ref(),
            hops.get(i + 1).and_then(|n| n.date.as_ref()),
        ) {
            (Some(cur), Some(older)) => {
                let d = cur.to_timestamp() - older.to_timestamp();
                if d >= 0 { Some(d * 1000) } else { None }
            }
            _ => None,
        };
        let geo = geoip_enrich(r.from_ip());
        out.push(ReceivedHop {
            index: i as i64,
            by_host: r.by.as_ref().map(host_str),
            from_host: r.from.as_ref().map(host_str),
            protocol: r.with.map(|p| p.to_string()),
            timestamp: ts,
            delay_ms,
            asn: geo.asn,
            asn_org: geo.asn_org,
            country: geo.country,
        });
    }
    out
}

fn host_str(h: &Host) -> String {
    match h {
        Host::Name(n) => n.to_string(),
        Host::IpAddr(ip) => ip.to_string(),
    }
}

// ── GeoIP / ASN enrichment (BYO database) ────────────────────────────────────

/// ASN/country enrichment for one Received hop.
#[derive(Debug, Default, Clone, PartialEq, Eq)]
struct GeoEnrichment {
    asn: Option<i64>,
    asn_org: Option<String>,
    country: Option<String>,
}

/// The admin-supplied GeoIP database path (`MW_GEOIP_DB`), when it points at an
/// existing file. `None` (the default) leaves every hop's ASN/country unset.
fn geoip_db_path() -> Option<std::path::PathBuf> {
    std::env::var_os("MW_GEOIP_DB")
        .map(std::path::PathBuf::from)
        .filter(|p| p.exists())
}

/// BYO GeoIP/ASN enrichment seam (SPEC §7.3). MaxMind GeoLite2 is NOT permissively
/// redistributable (account + attribution required), so **no database is bundled**
/// (t12 §5 flag 3 — the user's decision). ASN/country are resolved only from an
/// admin-supplied database pointed to by `MW_GEOIP_DB`; unset (the default) leaves
/// the fields `None`. The database *reader* is intentionally out of tree until a
/// permissively-licensed source is chosen — this hook establishes the wiring and
/// the admin-supplied-path contract without shipping a licence-encumbered DB or
/// pulling a new dependency.
fn geoip_enrich(ip: Option<IpAddr>) -> GeoEnrichment {
    match (ip, geoip_db_path()) {
        (Some(_ip), Some(_db)) => {
            // An admin database is configured; ASN/country resolution against it
            // activates when a permissively-licensed reader is wired in. No
            // bundled DB, no new dependency — the fields stay `None` until then.
            GeoEnrichment::default()
        }
        _ => GeoEnrichment::default(),
    }
}

// ── Attachment risk (ext-vs-magic) ───────────────────────────────────────────

fn attachment_risks(msg: &mail_parser::Message) -> Vec<AttachmentRisk> {
    let mut out = Vec::new();
    for att in msg.attachments() {
        let name = att.attachment_name().unwrap_or("attachment").to_string();
        let declared = att
            .content_type()
            .map(|c| content_type_string(c))
            .filter(|s| !s.is_empty());
        let bytes = att.contents();
        let detected = sniff_magic(bytes).map(String::from);
        let ext_type = mime_guess::from_path(&name)
            .first()
            .map(|m| m.essence_str().to_string());
        // Mismatch = the extension-implied type disagrees with the sniffed magic.
        let mismatch = match (&ext_type, &detected) {
            (Some(e), Some(d)) => essence(e) != essence(d),
            _ => false,
        };
        let risk = classify_attachment(&name, bytes, mismatch);
        out.push(AttachmentRisk {
            name,
            declared_type: declared.or(ext_type),
            detected_type: detected,
            mismatch,
            risk,
        });
    }
    out
}

fn essence(t: &str) -> &str {
    t.split(';').next().unwrap_or(t).trim()
}

fn content_type_string(c: &mail_parser::ContentType) -> String {
    match c.subtype() {
        Some(sub) => format!("{}/{}", c.ctype(), sub),
        None => c.ctype().to_string(),
    }
}

/// Sniff a handful of magic signatures (no `infer` dep) → an essence MIME string.
fn sniff_magic(b: &[u8]) -> Option<&'static str> {
    if b.len() >= 4 && &b[..4] == b"%PDF" {
        return Some("application/pdf");
    }
    if b.len() >= 8 && &b[..8] == b"\x89PNG\r\n\x1a\n" {
        return Some("image/png");
    }
    if b.len() >= 3 && &b[..3] == b"\xff\xd8\xff" {
        return Some("image/jpeg");
    }
    if b.len() >= 2 && &b[..2] == b"PK" {
        return Some("application/zip");
    }
    if b.len() >= 2 && &b[..2] == b"MZ" {
        return Some("application/x-msdownload");
    }
    if b.len() >= 4 && &b[..4] == b"\x7fELF" {
        return Some("application/x-executable");
    }
    None
}

fn classify_attachment(name: &str, bytes: &[u8], _mismatch: bool) -> String {
    let lower = name.to_lowercase();
    // Double-extension (e.g. invoice.pdf.exe).
    let exec_exts = [
        ".exe", ".scr", ".bat", ".cmd", ".com", ".pif", ".js", ".vbs", ".jar", ".msi",
    ];
    let parts: Vec<&str> = lower.rsplitn(3, '.').collect();
    if parts.len() == 3 {
        let last = format!(".{}", parts[0]);
        let mid = format!(".{}", parts[1]);
        let benign = [
            ".pdf", ".doc", ".docx", ".jpg", ".png", ".txt", ".xls", ".zip",
        ];
        if exec_exts.contains(&last.as_str()) && benign.contains(&mid.as_str()) {
            return "double-extension".into();
        }
    }
    if exec_exts.iter().any(|e| lower.ends_with(e))
        || sniff_magic(bytes) == Some("application/x-msdownload")
        || sniff_magic(bytes) == Some("application/x-executable")
    {
        return "executable".into();
    }
    if [".docm", ".xlsm", ".pptm"]
        .iter()
        .any(|e| lower.ends_with(e))
    {
        return "macro".into();
    }
    if [".zip", ".rar", ".7z"].iter().any(|e| lower.ends_with(e)) && encrypted_zip(bytes) {
        return "encrypted-archive".into();
    }
    "none".into()
}

/// A ZIP whose first local-file-header has the encryption bit (bit 0) set.
fn encrypted_zip(b: &[u8]) -> bool {
    b.len() >= 8 && &b[..4] == b"PK\x03\x04" && (b[6] & 0x01) != 0
}

// ── Signature / encryption detection ─────────────────────────────────────────

fn detect_crypto(raw: &[u8]) -> (Option<SignatureVerdict>, EncryptionInfo) {
    let text = String::from_utf8_lossy(raw);
    let lower = text.to_lowercase();
    // Encryption.
    let encryption = if lower.contains("multipart/encrypted")
        || lower.contains("application/pgp-encrypted")
        || lower.contains("-----begin pgp message-----")
    {
        EncryptionInfo {
            kind: "pgp".into(),
            is_encrypted: true,
            decrypts_client_side: true,
        }
    } else if lower.contains("application/pkcs7-mime")
        || lower.contains("smime-type=enveloped-data")
    {
        EncryptionInfo {
            kind: "smime".into(),
            is_encrypted: true,
            decrypts_client_side: true,
        }
    } else {
        EncryptionInfo {
            kind: "none".into(),
            is_encrypted: false,
            decrypts_client_side: false,
        }
    };
    // Signature (public verify; the signer key may be absent → unverified-key).
    let signature = if lower.contains("application/pgp-signature")
        || lower.contains("-----begin pgp signature-----")
    {
        Some(unverified_signature("pgp"))
    } else if lower.contains("application/pkcs7-signature")
        || lower.contains("smime-type=signed-data")
        || lower.contains("multipart/signed")
    {
        Some(unverified_signature("smime"))
    } else {
        None
    };
    (signature, encryption)
}

/// A signature detected in the MIME but not yet verified against a stored key
/// (the client WASM worker completes verification with the harvested/contact key,
/// plan §1.2). Server-side we surface the presence + 3-state placeholder.
fn unverified_signature(kind: &str) -> SignatureVerdict {
    SignatureVerdict {
        kind: kind.to_string(),
        status: "unverified-key".into(),
        signer_key_id: None,
        algorithm: None,
        key_created_at: None,
        key_expires_at: None,
        chain_status: Some("unknown".into()),
        revocation_status: Some("unknown".into()),
        key_changed: false,
    }
}

// ── Anomalies ────────────────────────────────────────────────────────────────

fn detect_anomalies(msg: &mail_parser::Message) -> Vec<String> {
    let mut out = Vec::new();
    let from_domain = first_addr_domain(msg.from());
    if let (Some(fd), Some(rd)) = (&from_domain, first_addr_domain(msg.reply_to()))
        && fd != &rd
    {
        out.push("replyToMismatch".to_string());
    }
    if let Some(fd) = &from_domain {
        if let Some(env) = msg.return_address()
            && let Some((_, ed)) = env.rsplit_once('@')
            && !ed.eq_ignore_ascii_case(fd)
        {
            out.push("envelopeFromDivergence".to_string());
        }
        if let Some(mid) = msg.message_id()
            && let Some((_, md)) = mid.rsplit_once('@')
            && !md.trim_end_matches('>').eq_ignore_ascii_case(fd)
        {
            out.push("messageIdDomainAnomaly".to_string());
        }
        if fd.contains("xn--") {
            out.push("punycodeSender".to_string());
        }
    }
    // Date skew: Date header far (>1 day) from the top Received timestamp.
    if let (Some(date), Some(top)) = (msg.date(), msg.received_all().next())
        && let Some(rdate) = &top.date
    {
        let skew = (date.to_timestamp() - rdate.to_timestamp()).abs();
        if skew > 86_400 {
            out.push("dateSkew".to_string());
        }
    }
    out
}

fn first_addr_domain(addr: Option<&mail_parser::Address>) -> Option<String> {
    addr.and_then(|a| a.first())
        .and_then(|a| a.address())
        .and_then(|s| s.rsplit_once('@').map(|(_, d)| d.to_lowercase()))
}

// ── Plain-language summary ───────────────────────────────────────────────────

fn plain_language(
    auth: &AuthVerdict,
    signature: &Option<SignatureVerdict>,
    encryption: &EncryptionInfo,
    attachments: &[AttachmentRisk],
    anomalies: &[String],
) -> String {
    let mut parts = Vec::new();
    let passed = auth.dkim.result == "pass";
    if passed {
        parts.push("This message passed DKIM sender authentication.".to_string());
    } else if auth.dkim.result == "fail" {
        parts.push(
            "This message FAILED DKIM authentication — treat the sender with caution.".to_string(),
        );
    } else {
        parts.push("This message has no verifiable DKIM signature.".to_string());
    }
    if encryption.is_encrypted {
        parts.push(format!(
            "It is {}-encrypted and will be decrypted on your device.",
            encryption.kind.to_uppercase()
        ));
    }
    if let Some(sig) = signature {
        parts.push(match sig.status.as_str() {
            "verified" => "Its signature is verified.".into(),
            "invalid" => "Its signature is INVALID.".into(),
            _ => "It is signed, but the signing key is not yet trusted.".into(),
        });
    }
    let risky = attachments
        .iter()
        .filter(|a| a.risk != "none" || a.mismatch)
        .count();
    if risky > 0 {
        parts.push(format!("{risky} attachment(s) look risky."));
    }
    if !anomalies.is_empty() {
        parts.push(format!(
            "{} header anomaly/anomalies detected.",
            anomalies.len()
        ));
    }
    parts.join(" ")
}

fn hex_sha256(bytes: &[u8]) -> String {
    let mut h = Sha256::new();
    h.update(bytes);
    let out = h.finalize();
    let mut s = String::with_capacity(out.len() * 2);
    for b in out {
        use std::fmt::Write;
        let _ = write!(s, "{b:02x}");
    }
    s
}

#[cfg(test)]
mod tests {
    use super::*;
    use mail_auth::common::crypto::Ed25519Key;
    use mail_auth::dkim::DkimSigner;
    use mail_parser::decoders::base64::base64_decode;

    // RFC 8463 ed25519 test key (same vector mail-auth ships).
    const ED_SEED: &str = "nWGxne/9WmC6hEr0kuwsxERJxWl7MmkZcDusAxyuf2A=";
    const ED_PUB_P: &str = "11qYAYKxCrfVS/7TyWQHOg7hcvPapiMlrwIaaPcHURo=";
    const ED_RECORD: &str = "v=DKIM1; k=ed25519; p=11qYAYKxCrfVS/7TyWQHOg7hcvPapiMlrwIaaPcHURo=";

    const MESSAGE: &str = "From: alice@example.com\r\n\
         To: bob@example.org\r\n\
         Subject: Hello\r\n\
         \r\n\
         This is the body.\r\n";

    /// Sign MESSAGE with the ed25519 key → the raw DKIM-signed message bytes.
    fn signed_message() -> Vec<u8> {
        let key = Ed25519Key::from_seed_and_public_key(
            &base64_decode(ED_SEED.as_bytes()).unwrap(),
            &base64_decode(ED_PUB_P.as_bytes()).unwrap(),
        )
        .unwrap();
        let sig = DkimSigner::from_key(key)
            .domain("example.com")
            .selector("ed")
            .headers(["From", "To", "Subject"])
            .sign(MESSAGE.as_bytes())
            .unwrap();
        let mut raw = Vec::new();
        sig.write(&mut raw, true);
        raw.extend_from_slice(MESSAGE.as_bytes());
        raw
    }

    fn seeded_cache() -> SeededTxtCache {
        let cache = SeededTxtCache::default();
        assert!(cache.insert_dkim("ed._domainkey.example.com.", ED_RECORD));
        cache
    }

    #[tokio::test]
    async fn dkim_pass_on_valid_signature() {
        let raw = signed_message();
        let authr = MessageAuthenticator::new_system_conf().unwrap();
        let cache = seeded_cache();
        let v = compute_verdict("e1", &raw, Some(&authr), Some(&cache), false).await;
        assert_eq!(v.auth.dkim.result, "pass", "expected DKIM pass");
        assert_eq!(v.auth.dkim.domain.as_deref(), Some("example.com"));
        assert_eq!(v.auth.dkim.selector.as_deref(), Some("ed"));
        assert!(v.plain_language.contains("DKIM"));
    }

    #[tokio::test]
    async fn dkim_fail_on_tampered_header() {
        // Tamper a signed header (Subject) after signing → signature verify fails.
        let raw = signed_message();
        let tampered = String::from_utf8(raw)
            .unwrap()
            .replace("Subject: Hello", "Subject: Hacked");
        let authr = MessageAuthenticator::new_system_conf().unwrap();
        let cache = seeded_cache();
        let v = compute_verdict("e2", tampered.as_bytes(), Some(&authr), Some(&cache), false).await;
        assert_eq!(v.auth.dkim.result, "fail", "expected DKIM fail on tamper");
    }

    #[test]
    fn spf_context_recovers_client_ip_and_mail_from() {
        let raw = "Received: from mail.example.com (mail.example.com [192.0.2.10])\r\n\
             \tby mx.example.org with ESMTPS; Mon, 13 Jul 2026 09:00:00 +0000\r\n\
             Return-Path: <alice@example.com>\r\n\
             From: alice@example.com\r\n\
             To: bob@example.org\r\n\
             Subject: Hi\r\n\r\nbody\r\n"
            .as_bytes();
        let parsed = MessageParser::default().parse(raw).unwrap();
        let ctx = spf_context(&parsed).expect("client context recovered from Received");
        assert_eq!(ctx.ip, "192.0.2.10".parse::<IpAddr>().unwrap());
        assert_eq!(ctx.domain, "example.com");
        assert_eq!(ctx.mail_from, "alice@example.com");
    }

    #[tokio::test]
    async fn spf_pass_and_fail_against_seeded_record() {
        // Offline: an `ip4:` record needs only the (seeded) TXT lookup.
        let authr = MessageAuthenticator::new_system_conf().unwrap();
        let cache = SeededTxtCache::default();
        assert!(cache.insert_spf("example.com.", "v=spf1 ip4:192.0.2.10 -all"));

        let pass_ctx = SpfContext {
            ip: "192.0.2.10".parse().unwrap(),
            helo: "mail.example.com".into(),
            mail_from: "alice@example.com".into(),
            domain: "example.com".into(),
        };
        let out = compute_spf(&authr, Some(&cache), &pass_ctx).await;
        assert_eq!(spf_result_str(out.result()), "pass", "authorized IP passes");

        let fail_ctx = SpfContext {
            ip: "198.51.100.7".parse().unwrap(),
            ..pass_ctx.clone()
        };
        let out = compute_spf(&authr, Some(&cache), &fail_ctx).await;
        assert_eq!(
            spf_result_str(out.result()),
            "fail",
            "unauthorized IP hard-fails (-all)"
        );
    }

    #[test]
    fn geoip_enrich_is_none_without_a_configured_db() {
        // No `MW_GEOIP_DB` set in the test env ⇒ no bundled DB ⇒ all fields None.
        let geo = geoip_enrich("192.0.2.10".parse::<IpAddr>().ok());
        assert_eq!(geo, GeoEnrichment::default());
    }

    #[tokio::test]
    async fn received_chain_attachments_and_anomalies() {
        // Two Received hops, a reply-to domain mismatch, and an attachment whose
        // .png extension disagrees with its %PDF magic bytes.
        let raw = "Received: from relay.example.net (relay.example.net)\r\n\
             \tby mx.example.org with ESMTPS; Mon, 13 Jul 2026 09:00:10 +0000\r\n\
             Received: from sender.example.com (sender.example.com)\r\n\
             \tby relay.example.net with ESMTP; Mon, 13 Jul 2026 09:00:00 +0000\r\n\
             From: alice@example.com\r\n\
             Reply-To: attacker@evil.example\r\n\
             To: bob@example.org\r\n\
             Subject: See attached\r\n\
             MIME-Version: 1.0\r\n\
             Content-Type: multipart/mixed; boundary=\"b1\"\r\n\
             \r\n\
             --b1\r\n\
             Content-Type: text/plain\r\n\r\n\
             Body.\r\n\
             --b1\r\n\
             Content-Type: application/octet-stream\r\n\
             Content-Disposition: attachment; filename=\"invoice.png\"\r\n\r\n\
             %PDF-1.4 fake pdf bytes\r\n\
             --b1--\r\n"
            .as_bytes();
        let v = compute_verdict("e3", raw, None, None, false).await;
        assert_eq!(v.received.len(), 2, "two Received hops parsed");
        assert!(
            v.attachments.iter().any(|a| a.mismatch),
            "ext-vs-magic mismatch flagged: {:?}",
            v.attachments
        );
        assert!(
            v.anomalies.contains(&"replyToMismatch".to_string()),
            "reply-to mismatch anomaly: {:?}",
            v.anomalies
        );
    }
}
