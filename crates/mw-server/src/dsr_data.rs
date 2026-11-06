//! Data subject requests: building an export and carrying out an erasure.
//!
//! Export sections are built one at a time and stored as parts (see
//! `mw_store::dsr`); a restarted worker skips the sections already stored. The
//! ZIP is assembled while it is sent, one stored part at a time. An erasure
//! deletes the person's rows from every personal table, removes native account
//! rows (their cascade takes the local mail cache and PIM rows), drops the search
//! index documents and sessions, and records what was deleted and what is kept.
use crate::AppState;
use serde_json::{Value, json};
use std::collections::BTreeMap;

/// Who the data is about, as resolved when the request was filed.
#[derive(Clone, Debug, Default)]
pub(crate) struct Subj {
    pub address: String,
    pub account_ids: Vec<String>,
    pub owners: Vec<String>,
}

/// Every section an export can hold, in the order they are built.
pub(crate) const SECTIONS: &[&str] = &[
    "profile",
    "settings",
    "identities",
    "contacts",
    "calendars",
    "notes",
    "rules",
    "mailstate",
    "tasks",
    "security",
    "integrations",
    "delegation",
    "usage",
    "records",
    "sessions",
    "consent",
    "audit",
    "requests",
    "mail",
];

/// What the export's options are.
#[derive(Clone, Debug)]
pub(crate) struct Options {
    pub csv: bool,
    pub mail: bool,
    pub mail_eml: bool,
    pub mail_cap_bytes: usize,
}

/// One built section: files and the number of records.
pub(crate) struct Built {
    pub files: Vec<(String, Vec<u8>)>,
    pub count: usize,
    pub note: Option<String>,
}

fn pretty(v: &Value) -> Vec<u8> {
    serde_json::to_vec_pretty(v).unwrap_or_default()
}

/// A CSV cell: quoted, with a leading formula character neutralised.
fn cell(v: &Value) -> String {
    let s = match v {
        Value::Null => String::new(),
        Value::String(s) => s.clone(),
        other => other.to_string(),
    };
    let s = if s.starts_with(['=', '+', '-', '@', '\t', '\r']) {
        format!("'{s}")
    } else {
        s
    };
    format!("\"{}\"", s.replace('"', "\"\""))
}

pub(crate) fn csv(rows: &[Value], cols: &[&str]) -> Vec<u8> {
    let mut out = cols.join(",");
    out.push_str("\r\n");
    for row in rows {
        let line: Vec<String> = cols.iter().map(|c| cell(&row[*c])).collect();
        out.push_str(&line.join(","));
        out.push_str("\r\n");
    }
    out.into_bytes()
}

fn safe(name: &str) -> String {
    let s: String = name
        .chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() || c == '-' || c == '_' {
                c
            } else {
                '_'
            }
        })
        .take(60)
        .collect();
    if s.is_empty() { "unnamed".into() } else { s }
}

fn with_csv(
    files: &mut Vec<(String, Vec<u8>)>,
    opts: &Options,
    name: &str,
    rows: &[Value],
    cols: &[&str],
) {
    if opts.csv {
        files.push((format!("{name}.csv"), csv(rows, cols)));
    }
}

/// Build one section.
pub(crate) async fn build_section(
    state: &AppState,
    name: &str,
    s: &Subj,
    opts: &Options,
    requests: &[Value],
) -> Result<Built, String> {
    let store = &state.store;
    let err = |e: mw_store::StoreError| format!("{name}: {e}");
    let mut files: Vec<(String, Vec<u8>)> = Vec::new();
    let mut note = None;
    let count;
    match name {
        "profile" => {
            let mut v = store.dsr_profile(&s.address).await.map_err(err)?;
            v["boundAccounts"] = json!(s.account_ids.len());
            files.push(("profile.json".into(), pretty(&v)));
            count = 1;
        }
        "settings" => {
            let mut ids = s.account_ids.clone();
            ids.push(s.address.clone());
            let mut rows = store.dsr_user_settings(&ids).await.map_err(err)?;
            for owner in &s.owners {
                for r in store.personal_setting_values(owner).await.map_err(err)? {
                    rows.push(json!({
                        "key": r.key,
                        "value": serde_json::from_str::<Value>(&r.value).unwrap_or(Value::Null),
                        "updatedAt": r.updated_at,
                    }));
                }
            }
            count = rows.len();
            files.push(("settings.json".into(), pretty(&json!(rows))));
            with_csv(&mut files, opts, "settings", &rows, &["key", "value", "updatedAt"]);
        }
        "identities" => {
            let mut rows = Vec::new();
            let mut all = Vec::new();
            for id in &s.account_ids {
                all.extend(store.list_identities(id).await.map_err(err)?);
            }
            for owner in &s.owners {
                all.extend(store.list_personal_identities(owner).await.map_err(err)?);
            }
            for i in all {
                rows.push(json!({
                    "id": i.id, "name": i.name, "email": i.email, "replyTo": i.reply_to,
                    "organization": i.organization, "autoBcc": i.auto_bcc,
                    "signatureName": i.signature_name, "signatureText": i.signature_text,
                    "source": i.source, "isDefault": i.is_default,
                }));
            }
            count = rows.len();
            files.push(("identities.json".into(), pretty(&json!(rows))));
            with_csv(&mut files, opts, "identities", &rows, &["id", "name", "email", "replyTo", "organization", "source"]);
        }
        "contacts" => {
            let mut rows = Vec::new();
            let mut vcf = String::new();
            for acct in &s.account_ids {
                for book in store.list_address_books(acct).await.map_err(err)? {
                    for c in store.list_contacts(&book.id).await.map_err(err)? {
                        rows.push(json!({
                            "id": c.id, "addressBook": book.name, "uid": c.uid,
                            "fullName": c.full_name, "isFavorite": c.is_favorite,
                        }));
                        vcf.push_str(c.vcard_raw.trim_end());
                        vcf.push_str("\r\n");
                    }
                }
            }
            count = rows.len();
            files.push(("contacts.vcf".into(), vcf.into_bytes()));
            files.push(("contacts.json".into(), pretty(&json!(rows))));
            with_csv(&mut files, opts, "contacts", &rows, &["id", "addressBook", "uid", "fullName", "isFavorite"]);
        }
        "calendars" => {
            let mut rows = Vec::new();
            let mut n = 0usize;
            for acct in &s.account_ids {
                for cal in store.list_calendars(acct).await.map_err(err)? {
                    let events = store.list_events(&cal.id).await.map_err(err)?;
                    let mut ics = String::new();
                    for e in &events {
                        ics.push_str(e.ical_raw.trim_end());
                        ics.push_str("\r\n");
                    }
                    n += events.len();
                    files.push((format!("calendars/{}-{}.ics", safe(&cal.name), safe(&cal.id)), ics.into_bytes()));
                    rows.push(json!({
                        "id": cal.id, "name": cal.name, "component": cal.component,
                        "events": events.len(),
                    }));
                }
            }
            count = n;
            files.push(("calendars.json".into(), pretty(&json!(rows))));
            with_csv(&mut files, opts, "calendars", &rows, &["id", "name", "component", "events"]);
        }
        "notes" => {
            let mut rows = Vec::new();
            for acct in &s.account_ids {
                let books = store.list_notebooks(acct).await.map_err(err)?;
                for n in store.list_notes(acct).await.map_err(err)? {
                    let book = books
                        .iter()
                        .find(|b| Some(&b.id) == n.notebook_id.as_ref())
                        .map(|b| b.name.clone());
                    rows.push(json!({
                        "id": n.id, "notebook": book, "title": n.title,
                        "tags": serde_json::from_str::<Value>(&n.tags_json).unwrap_or(Value::Null),
                        "pinned": n.pinned, "bodyText": n.body_text,
                        "createdAt": n.created_at, "updatedAt": n.updated_at,
                    }));
                }
            }
            count = rows.len();
            files.push(("notes.json".into(), pretty(&json!(rows))));
            with_csv(&mut files, opts, "notes", &rows, &["id", "notebook", "title", "pinned", "createdAt", "updatedAt", "bodyText"]);
        }
        "rules" => {
            let mut rows = Vec::new();
            for acct in &s.account_ids {
                for sc in store.list_sieve_scripts(acct).await.map_err(err)? {
                    files.push((format!("rules/sieve-{}.sieve", safe(&sc.name)), sc.content.clone().into_bytes()));
                    rows.push(json!({"name": sc.name, "active": sc.active, "updatedAt": sc.updated_at}));
                }
            }
            count = rows.len();
            files.push(("rules.json".into(), pretty(&json!(rows))));
        }
        "sessions" => {
            let mut sessions = Vec::new();
            let mut devices = Vec::new();
            let mut history = Vec::new();
            for acct in &s.account_ids {
                for x in store.list_security_sessions(Some(acct), 200).await.map_err(err)? {
                    sessions.push(serde_json::to_value(&x).unwrap_or(Value::Null));
                }
                for d in store.remembered_devices(acct, 0).await.map_err(err)? {
                    devices.push(serde_json::to_value(&d).unwrap_or(Value::Null));
                }
                for e in store.signin_history(acct, i64::MAX / 2, None, 500).await.map_err(err)? {
                    history.push(serde_json::to_value(&e).unwrap_or(Value::Null));
                }
            }
            count = sessions.len() + devices.len() + history.len();
            files.push(("sessions.json".into(), pretty(&json!({"sessions": sessions, "rememberedDevices": devices, "signInHistory": history}))));
            with_csv(&mut files, opts, "signin-history", &history, &["id", "ts", "result", "method", "ip", "userAgent", "geoCountry", "reason"]);
        }
        "consent" => {
            let rows = match store.dsr_consent_records(&s.account_ids).await {
                Ok(r) => r,
                Err(e) => {
                    note = Some(format!("consent records could not be read: {e}"));
                    Vec::new()
                }
            };
            count = rows.len();
            files.push(("consent.json".into(), pretty(&json!(rows))));
            with_csv(&mut files, opts, "consent", &rows, &["id", "domain", "policyVersion", "decision", "region", "signal", "createdAt"]);
        }
        "audit" => {
            let rows = store.dsr_audit_entries(&s.account_ids, 5000).await.map_err(err)?;
            count = rows.len();
            files.push(("audit.json".into(), pretty(&json!(rows))));
            with_csv(&mut files, opts, "audit", &rows, &["id", "ts", "actor", "actorKind", "action", "target"]);
        }
        "mailstate" | "tasks" | "security" | "integrations" | "delegation" | "usage" | "records" => {
            // Table-driven sections come from the data map. Sealed secrets are
            // listed as "present, not exported" markers, never their content.
            let p = person_ids(state, s).await.map_err(err)?;
            let rows = store.dsr_export_table_section(name, &p).await.map_err(err)?;
            count = rows.len();
            files.push((format!("{name}.json"), pretty(&json!(rows))));
        }
        "requests" => {
            count = requests.len();
            files.push(("data-requests.json".into(), pretty(&json!(requests))));
        }
        "mail" => {
            let Some(engine) = &state.engine else {
                return Ok(Built {
                    files,
                    count: 0,
                    note: Some("mail is held on the upstream mail server and is not copied here; export it with a mail client".into()),
                });
            };
            let mut used = 0usize;
            let mut n = 0usize;
            let mut mbox = Vec::new();
            for acct in &s.account_ids {
                let ids = store.dsr_message_ids(acct, 100_000).await.map_err(err)?;
                for sid in ids {
                    let blob = engine.fetch_blob(acct, &sid).await.map_err(|e| format!("mail: {e}"))?;
                    let Some(blob) = blob else { continue };
                    if used + blob.bytes.len() > opts.mail_cap_bytes {
                        note = Some("mail stopped at the export size cap".into());
                        break;
                    }
                    used += blob.bytes.len();
                    n += 1;
                    if opts.mail_eml {
                        files.push((format!("mail/{}.eml", safe(&sid)), blob.bytes));
                    } else {
                        mbox.extend_from_slice(b"From MAILER-DAEMON Thu Jan  1 00:00:00 1970\n");
                        for line in blob.bytes.split(|b| *b == b'\n') {
                            if line.starts_with(b"From ") {
                                mbox.push(b'>');
                            }
                            mbox.extend_from_slice(line);
                            mbox.push(b'\n');
                        }
                        mbox.push(b'\n');
                    }
                }
            }
            if !opts.mail_eml && !mbox.is_empty() {
                files.push(("mail/mail.mbox".into(), mbox));
            }
            count = n;
        }
        other => return Err(format!("unknown section {other}")),
    }
    Ok(Built { files, count, note })
}

// ─── ZIP ──────────────────────────────────────────────────────────────────────

/// DOS time and date for a unix timestamp.
fn dos(ts: i64) -> (u16, u16) {
    use chrono::{Datelike, TimeZone, Timelike};
    let dt = chrono::Utc
        .timestamp_opt(ts, 0)
        .single()
        .unwrap_or_else(chrono::Utc::now);
    let year = (dt.year().clamp(1980, 2107) - 1980) as u16;
    (
        ((dt.hour() as u16) << 11) | ((dt.minute() as u16) << 5) | (dt.second() as u16 / 2),
        (year << 9) | ((dt.month() as u16) << 5) | dt.day() as u16,
    )
}

/// Incremental writer of a ZIP archive with stored (uncompressed) entries.
pub(crate) struct ZipWriter {
    offset: u64,
    central: Vec<u8>,
    count: u16,
    time: (u16, u16),
}

impl ZipWriter {
    pub(crate) fn new(ts: i64) -> Self {
        Self {
            offset: 0,
            central: Vec::new(),
            count: 0,
            time: dos(ts),
        }
    }

    /// The bytes to send for one file: its local header followed by the data.
    pub(crate) fn entry(&mut self, name: &str, data: &[u8]) -> Vec<u8> {
        let mut crc = flate2::Crc::new();
        crc.update(data);
        let crc = crc.sum();
        let name_b = name.as_bytes();
        let (t, d) = self.time;
        let mut out = Vec::with_capacity(30 + name_b.len() + data.len());
        out.extend_from_slice(&0x0403_4b50u32.to_le_bytes());
        out.extend_from_slice(&20u16.to_le_bytes());
        out.extend_from_slice(&0x0800u16.to_le_bytes());
        out.extend_from_slice(&0u16.to_le_bytes());
        out.extend_from_slice(&t.to_le_bytes());
        out.extend_from_slice(&d.to_le_bytes());
        out.extend_from_slice(&crc.to_le_bytes());
        out.extend_from_slice(&(data.len() as u32).to_le_bytes());
        out.extend_from_slice(&(data.len() as u32).to_le_bytes());
        out.extend_from_slice(&(name_b.len() as u16).to_le_bytes());
        out.extend_from_slice(&0u16.to_le_bytes());
        out.extend_from_slice(name_b);
        out.extend_from_slice(data);
        let c = &mut self.central;
        c.extend_from_slice(&0x0201_4b50u32.to_le_bytes());
        c.extend_from_slice(&20u16.to_le_bytes());
        c.extend_from_slice(&20u16.to_le_bytes());
        c.extend_from_slice(&0x0800u16.to_le_bytes());
        c.extend_from_slice(&0u16.to_le_bytes());
        c.extend_from_slice(&t.to_le_bytes());
        c.extend_from_slice(&d.to_le_bytes());
        c.extend_from_slice(&crc.to_le_bytes());
        c.extend_from_slice(&(data.len() as u32).to_le_bytes());
        c.extend_from_slice(&(data.len() as u32).to_le_bytes());
        c.extend_from_slice(&(name_b.len() as u16).to_le_bytes());
        c.extend_from_slice(&[0u8; 12]);
        c.extend_from_slice(&(self.offset as u32).to_le_bytes());
        c.extend_from_slice(name_b);
        self.offset += out.len() as u64;
        self.count = self.count.saturating_add(1);
        out
    }

    /// The central directory and end record.
    pub(crate) fn finish(&mut self) -> Vec<u8> {
        let mut out = std::mem::take(&mut self.central);
        let size = out.len() as u32;
        out.extend_from_slice(&0x0605_4b50u32.to_le_bytes());
        out.extend_from_slice(&[0u8; 4]);
        out.extend_from_slice(&self.count.to_le_bytes());
        out.extend_from_slice(&self.count.to_le_bytes());
        out.extend_from_slice(&size.to_le_bytes());
        out.extend_from_slice(&(self.offset as u32).to_le_bytes());
        out.extend_from_slice(&0u16.to_le_bytes());
        out
    }
}

/// The archive's file names for a request: its section parts, sorted.
/// A part's stored name is the file name inside the ZIP.
pub(crate) fn zip_stream(
    state: AppState,
    request_id: String,
    names: Vec<String>,
    ts: i64,
) -> impl futures_util::Stream<Item = Result<axum::body::Bytes, std::io::Error>> {
    struct St {
        state: AppState,
        id: String,
        names: std::vec::IntoIter<String>,
        zip: ZipWriter,
        done: bool,
    }
    let st = St {
        state,
        id: request_id,
        names: names.into_iter(),
        zip: ZipWriter::new(ts),
        done: false,
    };
    futures_util::stream::unfold(st, |mut st| async move {
        if st.done {
            return None;
        }
        loop {
            match st.names.next() {
                Some(name) => match st.state.store.dsr_part_read(&st.id, &name).await {
                    Ok(Some(data)) => {
                        let chunk = st.zip.entry(&name, &data);
                        return Some((Ok(axum::body::Bytes::from(chunk)), st));
                    }
                    Ok(None) => continue,
                    Err(e) => {
                        st.done = true;
                        return Some((Err(std::io::Error::other(e.to_string())), st));
                    }
                },
                None => {
                    st.done = true;
                    let tail = st.zip.finish();
                    return Some((Ok(axum::body::Bytes::from(tail)), st));
                }
            }
        }
    })
}

// ─── erasure ──────────────────────────────────────────────────────────────────

/// What an erasure did.
#[derive(Debug, Default)]
pub(crate) struct Erased {
    pub counts: BTreeMap<String, u64>,
    pub errors: Vec<String>,
    pub remaining: u64,
}

/// The identifiers of a subject, with the principals resolved.
pub(crate) async fn person_ids(
    state: &AppState,
    s: &Subj,
) -> Result<mw_store::erasure::PersonIds, mw_store::StoreError> {
    let principals = state
        .store
        .dsr_resolve_principals(&s.account_ids, &s.address)
        .await?;
    Ok(mw_store::erasure::PersonIds {
        address: s.address.clone(),
        account_ids: s.account_ids.clone(),
        owners: s.owners.clone(),
        principals,
    })
}

/// Delete one person's data from every store this server holds.
///
/// The rows are found through the data map (`mw_store::data_map`); the verify
/// pass counts the same registry, so a table that is only mapped is checked too.
/// A tombstone (a keyed hash of the address) is written when the erasure is clean,
/// and the person's identifiers are scrubbed from audit and log history when
/// `scrub.autoRunOnErasure` is on.
pub(crate) async fn erase(state: &AppState, s: &Subj, request_id: Option<&str>) -> Erased {
    let store = &state.store;
    let mut out = Erased::default();
    let add = |out: &mut Erased, k: &str, n: u64| {
        if n > 0 {
            *out.counts.entry(k.into()).or_insert(0) += n;
        }
    };
    let people = match person_ids(state, s).await {
        Ok(p) => p,
        Err(e) => {
            out.errors.push(format!("principals: {e}"));
            return out;
        }
    };
    // Search index and embeddings first: they key on the account ids.
    for id in &s.account_ids {
        if let Some(engine) = &state.engine {
            add(&mut out, "search_index_documents", engine.erase_account_index(id));
        }
        // In-memory sessions are dropped before the rows that name them go.
        match store.sessions_by_account(id).await {
            Ok(sessions) => {
                for sess in sessions {
                    state.sessions.forget(&sess.id);
                }
            }
            Err(e) => out.errors.push(format!("sessions: {e}")),
        }
        match store.delete_account_message_embeddings(id).await {
            Ok(n) => add(&mut out, "message_embeddings", n as u64),
            Err(e) => out.errors.push(format!("message_embeddings: {e}")),
        }
    }
    let (counts, errors) = store.dsr_purge_person(&people).await;
    for (k, n) in counts {
        add(&mut out, &k, n);
    }
    out.errors.extend(errors);
    match store.user_lifecycle(&s.address).await {
        Ok(Some(l)) if l.status != "deleted" => {
            let now = chrono::Utc::now().timestamp();
            match store.change_user_lifecycle(&l, "deleted", "dsr-erasure", now).await {
                Ok(mw_store::user_lifecycle::LifecycleChange::Applied { sessions, .. }) => {
                    for id in sessions {
                        state.sessions.forget(&id);
                    }
                    add(&mut out, "directory_entry_closed", 1);
                }
                Ok(_) => {}
                Err(e) => out.errors.push(format!("directory: {e}")),
            }
        }
        Ok(_) => {}
        Err(e) => out.errors.push(format!("directory: {e}")),
    }
    // Verify: the registry's count of every rule must find nothing.
    let (left, verify_errors) = store.dsr_verify_checked(&people).await;
    out.errors.extend(verify_errors);
    out.remaining = left.values().sum();
    if out.errors.is_empty() && out.remaining == 0 {
        let now = chrono::Utc::now().timestamp();
        match store.erasure_record(&s.address, "erasure", request_id, now).await {
            Ok(_) => add(&mut out, "erasure_tombstone", 1),
            Err(e) => out.errors.push(format!("tombstone: {e}")),
        }
        let cfg = crate::scrub_admin::ScrubConfig::load(state).await;
        if cfg.enabled && cfg.auto_on_erasure {
            match crate::scrub_admin::scrub_subject(state, s, &cfg, false, request_id).await {
                Ok(r) => {
                    for (k, n) in r.counts {
                        add(&mut out, &format!("scrubbed:{k}"), n);
                    }
                    out.errors.extend(r.errors);
                }
                Err(e) => out.errors.push(format!("scrub: {e}")),
            }
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn u16le(b: &[u8], at: usize) -> usize {
        u16::from_le_bytes([b[at], b[at + 1]]) as usize
    }
    fn u32le(b: &[u8], at: usize) -> usize {
        u32::from_le_bytes([b[at], b[at + 1], b[at + 2], b[at + 3]]) as usize
    }

    /// Read the archive back through its central directory.
    fn entries(zip: &[u8]) -> Vec<(String, Vec<u8>)> {
        let end = zip.len() - 22;
        assert_eq!(u32le(zip, end), 0x0605_4b50);
        let count = u16le(zip, end + 10);
        let mut at = u32le(zip, end + 16);
        let mut out = Vec::new();
        for _ in 0..count {
            assert_eq!(u32le(zip, at), 0x0201_4b50);
            let crc = u32le(zip, at + 16) as u32;
            let size = u32le(zip, at + 24);
            let name_len = u16le(zip, at + 28);
            let local = u32le(zip, at + 42);
            let name = String::from_utf8(zip[at + 46..at + 46 + name_len].to_vec()).unwrap();
            assert_eq!(u32le(zip, local), 0x0403_4b50);
            let data_at = local + 30 + u16le(zip, local + 26) + u16le(zip, local + 28);
            let data = zip[data_at..data_at + size].to_vec();
            let mut check = flate2::Crc::new();
            check.update(&data);
            assert_eq!(check.sum(), crc, "{name} CRC");
            out.push((name, data));
            at += 46 + name_len;
        }
        out
    }

    #[test]
    fn zip_round_trips_names_and_bytes_with_valid_crcs() {
        let mut z = ZipWriter::new(1_800_000_000);
        let mut bytes = Vec::new();
        bytes.extend(z.entry("manifest.json", b"{\"a\":1}"));
        bytes.extend(z.entry("calendars/Work é.ics", "ünï\r\n".as_bytes()));
        bytes.extend(z.entry("empty.txt", b""));
        bytes.extend(z.finish());
        let got = entries(&bytes);
        assert_eq!(got.len(), 3);
        assert_eq!(got[0], ("manifest.json".into(), b"{\"a\":1}".to_vec()));
        assert_eq!(got[1].0, "calendars/Work é.ics");
        assert_eq!(got[1].1, "ünï\r\n".as_bytes());
        assert!(got[2].1.is_empty());
    }

    #[test]
    fn csv_quotes_and_neutralises_formulas() {
        let rows = [json!({"a": "=cmd|' /C calc'!A0", "b": "say \"hi\", ok", "c": 3, "d": null})];
        let out = String::from_utf8(csv(&rows, &["a", "b", "c", "d"])).unwrap();
        assert_eq!(out, "a,b,c,d\r\n\"'=cmd|' /C calc'!A0\",\"say \"\"hi\"\", ok\",\"3\",\"\"\r\n");
    }

    #[test]
    fn file_names_inside_the_archive_stay_plain() {
        assert_eq!(safe("../../etc/passwd"), "______etc_passwd");
        assert_eq!(safe(""), "unnamed");
    }
}
