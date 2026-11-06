//! Retention for a person's calendar events and notes, run by the same job as
//! the mail rules.
//!
//! Both are off by default and opt-in:
//!
//! * **Calendar.** The person chooses calendars and a number of days. Events in
//!   those calendars that ended more than that many days ago are deleted through
//!   the engine (so a CalDAV-backed calendar is told). Recurring events have no
//!   end and are never touched. Contacts are not covered: no interaction history
//!   is stored, so there is nothing to prune.
//! * **Notes.** Notes not changed for that many days (pinned notes excepted) are
//!   removed. A sealed copy is kept for `retention.user.undoDays`, and the person
//!   can restore it from Settings > Retention until then.
//!
//! The administrator's switches (`retention.user.calendar.*`, `.notes.*`) decide
//! whether either is available and the day limits. Legal-hold and restricted
//! accounts are skipped. Run records and audit entries hold counts only.

use super::*;

/// Most items one run handles per account and kind; the rest waits.
const CAP: i64 = 500;
const MAX_CALENDARS: usize = 50;

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(super) struct CalendarCfg {
    #[serde(default)]
    pub enabled: bool,
    /// Days after an event ended.
    #[serde(default)]
    pub days: i64,
    /// Calendars the person opted in.
    #[serde(default)]
    pub calendars: Vec<String>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(super) struct NotesCfg {
    #[serde(default)]
    pub enabled: bool,
    /// Days since the note was last changed.
    #[serde(default)]
    pub days: i64,
}

/// The administrator's switches and limits.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct PimPolicy {
    pub cal_enabled: bool,
    pub cal_min: i64,
    pub cal_max: i64,
    pub notes_enabled: bool,
    pub notes_min: i64,
    pub notes_max: i64,
    pub undo_days: i64,
}

pub(super) async fn pim_policy(store: &Store, account: &str, username: &str) -> PimPolicy {
    let who = Who::of(account, username);
    let n = |v: Value, d: i64| v.as_i64().unwrap_or(d);
    PimPolicy {
        cal_enabled: setting(store, "retention.user.calendar.enabled", &who).await.as_bool().unwrap_or(false),
        cal_min: n(setting(store, "retention.user.calendar.minDays", &who).await, 30),
        cal_max: n(setting(store, "retention.user.calendar.maxDays", &who).await, 3650),
        notes_enabled: setting(store, "retention.user.notes.enabled", &who).await.as_bool().unwrap_or(false),
        notes_min: n(setting(store, "retention.user.notes.minDays", &who).await, 30),
        notes_max: n(setting(store, "retention.user.notes.maxDays", &who).await, 3650),
        undo_days: n(setting(store, "retention.user.undoDays", &who).await, 7),
    }
}

fn refuse(code: &'static str, detail: impl Into<String>) -> Result<(), (&'static str, String)> {
    Err((code, detail.into()))
}

fn days_ok(days: i64, min: i64, max: i64) -> Result<(), (&'static str, String)> {
    if days < min {
        return refuse("belowMinimum", format!("This server keeps this data at least {min} days."));
    }
    if days > max {
        return refuse("aboveMaximum", format!("This server allows at most {max} days."));
    }
    Ok(())
}

/// Check a calendar opt-in against the administrator's switches. Turning it off
/// is always accepted.
pub(super) fn validate_calendar(c: &CalendarCfg, p: &PimPolicy) -> Result<(), (&'static str, String)> {
    if !c.enabled {
        return Ok(());
    }
    if !p.cal_enabled {
        return refuse("pimDisabled", "Calendar retention is disabled by your administrator.");
    }
    if c.calendars.is_empty() {
        return refuse("calendarRequired", "Choose at least one calendar.");
    }
    if c.calendars.len() > MAX_CALENDARS {
        return refuse("tooMany", "Choose at most 50 calendars.");
    }
    days_ok(c.days, p.cal_min, p.cal_max)
}

pub(super) fn validate_notes(c: &NotesCfg, p: &PimPolicy) -> Result<(), (&'static str, String)> {
    if !c.enabled {
        return Ok(());
    }
    if !p.notes_enabled {
        return refuse("pimDisabled", "Notes retention is disabled by your administrator.");
    }
    days_ok(c.days, p.notes_min, p.notes_max)
}

async fn load<T: serde::de::DeserializeOwned + Default>(store: &Store, account: &str, kind: &str) -> T {
    store
        .pim_retention_config(account, kind)
        .await
        .ok()
        .flatten()
        .and_then(|s| serde_json::from_str(&s).ok())
        .unwrap_or_default()
}

#[derive(Debug, Default, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct PimSummary {
    pub skipped: Option<&'static str>,
    pub events_matched: u64,
    pub events_deleted: u64,
    pub notes_matched: u64,
    pub notes_removed: u64,
    /// Switched on by the person but off at the administrator: not run.
    pub disabled_skipped: u64,
    pub errors: u64,
}

impl PimSummary {
    pub fn changed(&self) -> u64 {
        self.events_deleted + self.notes_removed
    }
}

fn cutoff_of(now: i64, days: i64) -> chrono::DateTime<chrono::Utc> {
    chrono::DateTime::from_timestamp(now - days * rr::DAY, 0).unwrap_or_default()
}

/// Run (or preview) one account's calendar and notes retention.
pub(crate) async fn run_pim(state: &AppState, account: &str, username: &str, dry: bool, trigger: &str) -> Result<PimSummary, String> {
    let store = &state.store;
    let now = chrono::Utc::now().timestamp();
    let mut s = PimSummary::default();
    if is_held(store, account).await {
        s.skipped = Some("legalHold");
        return Ok(s);
    }
    let who = Who::of(account, username);
    if !setting(store, "retention.user.runForRestricted", &who).await.as_bool().unwrap_or(false)
        && matches!(store.restriction_active(account, now).await, Ok(Some(_)))
    {
        s.skipped = Some("restricted");
        return Ok(s);
    }
    let p = pim_policy(store, account, username).await;
    let cal: CalendarCfg = load(store, account, "calendar").await;
    let notes: NotesCfg = load(store, account, "notes").await;
    let cal_on = cal.enabled && p.cal_enabled && !cal.calendars.is_empty();
    let notes_on = notes.enabled && p.notes_enabled;
    s.disabled_skipped = u64::from(cal.enabled && !p.cal_enabled) + u64::from(notes.enabled && !p.notes_enabled);
    if !cal_on && !notes_on {
        if s.disabled_skipped == 0 {
            s.skipped = Some("noRules");
        }
        return Ok(s);
    }
    let port = JmapPort::connect(state, account).await?;

    if cal_on {
        let days = cal.days.clamp(p.cal_min, p.cal_max);
        let cutoff = cutoff_of(now, days).format("%Y-%m-%dT%H:%M:%SZ").to_string();
        let ended = store.events_ended_before(account, &cutoff, CAP * 4).await.map_err(|e| e.to_string())?;
        let ids: Vec<String> = ended.into_iter().filter(|e| cal.calendars.contains(&e.calendar_id)).take(CAP as usize).map(|e| e.id).collect();
        s.events_matched = ids.len() as u64;
        if !dry && !ids.is_empty() {
            for chunk in ids.chunks(100) {
                match port.call("CalendarEvent/set", json!({"destroy": chunk})).await {
                    Ok(r) => {
                        let n = r["destroyed"].as_array().map_or(0, Vec::len) as u64;
                        s.events_deleted += n;
                        s.errors += (chunk.len() as u64).saturating_sub(n);
                    }
                    Err(_) => s.errors += chunk.len() as u64,
                }
            }
        }
    }

    if notes_on {
        let days = notes.days.clamp(p.notes_min, p.notes_max);
        let cutoff = cutoff_of(now, days).to_rfc3339_opts(chrono::SecondsFormat::Secs, true);
        let ids = store.notes_untouched_since(account, &cutoff, CAP).await.map_err(|e| e.to_string())?;
        s.notes_matched = ids.len() as u64;
        if !dry && !ids.is_empty() {
            // Keep a sealed copy first; only notes whose copy is safe are removed.
            let mut safe = Vec::new();
            for id in &ids {
                if p.undo_days > 0 {
                    match store.note_hold_copy(account, id, now, now + p.undo_days * rr::DAY).await {
                        Ok(true) => safe.push(id.clone()),
                        Ok(false) => {}
                        Err(_) => s.errors += 1,
                    }
                } else {
                    safe.push(id.clone());
                }
            }
            for chunk in safe.chunks(100) {
                match port.call("Note/set", json!({"destroy": chunk})).await {
                    Ok(r) => {
                        let gone: Vec<String> = r["destroyed"].as_array().map(|a| a.iter().filter_map(|v| v.as_str().map(str::to_string)).collect()).unwrap_or_default();
                        s.notes_removed += gone.len() as u64;
                        s.errors += (chunk.len() - gone.len()) as u64;
                        // A hold whose note was not removed would only confuse the undo list.
                        if p.undo_days > 0 {
                            for id in chunk.iter().filter(|c| !gone.contains(c)) {
                                let _ = store.note_restore(account, id).await;
                            }
                        }
                    }
                    Err(_) => s.errors += chunk.len() as u64,
                }
            }
        }
    }

    if !dry && (s.changed() > 0 || s.errors > 0) {
        let run = RunRow {
            id: uuid::Uuid::new_v4().to_string(),
            account_id: account.to_string(),
            started_at: now,
            finished_at: chrono::Utc::now().timestamp(),
            dry_run: false,
            triggered_by: trigger.to_string(),
            matched: (s.events_matched + s.notes_matched) as i64,
            acted: s.changed() as i64,
            errors: s.errors as i64,
            counts_json: json!({"pim": s}).to_string(),
        };
        let _ = store.retention_run_put(&run).await;
        let entry = mw_admin::AuditLogEntry {
            id: uuid::Uuid::new_v4().to_string(),
            ts: chrono::Utc::now().to_rfc3339(),
            actor: if trigger == "scheduler" { "system".into() } else { who.address.clone() },
            actor_kind: if trigger == "scheduler" { mw_admin::ActorKind::System } else { mw_admin::ActorKind::User },
            action: "user-pim-retention-run".into(),
            target: Some(account.to_string()),
            detail_json: mw_admin::redact_detail(&json!({
                "trigger": trigger, "eventsDeleted": s.events_deleted, "notesRemoved": s.notes_removed, "errors": s.errors,
            })),
            ip: None,
        };
        if let Err(e) = state.v6.admin.audit(entry).await {
            tracing::warn!("pim retention audit entry not recorded: {e}");
        }
    }
    Ok(s)
}

// ─── routes ────────────────────────────────────────────────────────────────────

pub(super) fn router() -> Router<AppState> {
    Router::new()
        .route("/api/retention/pim", get(overview).put(save))
        .route("/api/retention/pim/preview", post(preview))
        .route("/api/retention/pim/run", post(run))
        .route("/api/retention/pim/notes/{id}/restore", post(restore))
}

async fn overview(State(state): State<AppState>, headers: HeaderMap) -> Response {
    let me = match me(&state, &headers).await {
        Ok(m) => m,
        Err(r) => return r,
    };
    let store = &state.store;
    let p = pim_policy(store, &me.account_id, &me.username).await;
    let cal: CalendarCfg = load(store, &me.account_id, "calendar").await;
    let notes: NotesCfg = load(store, &me.account_id, "notes").await;
    let calendars: Vec<Value> = store
        .list_calendars(&me.account_id)
        .await
        .unwrap_or_default()
        .into_iter()
        .filter(|c| c.component == "VEVENT")
        .map(|c| json!({"id": c.id, "name": c.name}))
        .collect();
    let held: Vec<Value> = store
        .note_holds(&me.account_id)
        .await
        .unwrap_or_default()
        .into_iter()
        .map(|h| json!({"noteId": h.note_id, "title": h.title, "heldAt": h.held_at, "dueAt": h.due_at}))
        .collect();
    no_store(Json(json!({
        "policy": {
            "calendar": {"allowed": p.cal_enabled, "minDays": p.cal_min, "maxDays": p.cal_max},
            "notes": {"allowed": p.notes_enabled, "minDays": p.notes_min, "maxDays": p.notes_max},
            "undoDays": p.undo_days,
        },
        "calendar": cal, "notes": notes, "calendars": calendars, "held": held,
    })))
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct SaveIn {
    #[serde(default)]
    calendar: Option<CalendarCfg>,
    #[serde(default)]
    notes: Option<NotesCfg>,
}

async fn save(State(state): State<AppState>, headers: HeaderMap, Json(input): Json<SaveIn>) -> Response {
    let me = match me(&state, &headers).await {
        Ok(m) => m,
        Err(r) => return r,
    };
    let store = &state.store;
    let p = pim_policy(store, &me.account_id, &me.username).await;
    let now = chrono::Utc::now().timestamp();
    if let Some(c) = &input.calendar {
        if let Err((code, detail)) = validate_calendar(c, &p) {
            return fail(StatusCode::UNPROCESSABLE_ENTITY, code, detail);
        }
        let own: Vec<String> = store.list_calendars(&me.account_id).await.unwrap_or_default().into_iter().map(|c| c.id).collect();
        if c.calendars.iter().any(|id| !own.contains(id)) {
            return fail(StatusCode::UNPROCESSABLE_ENTITY, "unknownCalendar", "One of the calendars was not found.");
        }
        if let Err(e) = store.pim_retention_config_put(&me.account_id, "calendar", &serde_json::to_string(c).unwrap_or_default(), now).await {
            return store_fail(e);
        }
    }
    if let Some(n) = &input.notes {
        if let Err((code, detail)) = validate_notes(n, &p) {
            return fail(StatusCode::UNPROCESSABLE_ENTITY, code, detail);
        }
        if let Err(e) = store.pim_retention_config_put(&me.account_id, "notes", &serde_json::to_string(n).unwrap_or_default(), now).await {
            return store_fail(e);
        }
    }
    overview(State(state), headers).await
}

async fn preview(State(state): State<AppState>, headers: HeaderMap) -> Response {
    match me(&state, &headers).await {
        Ok(me) => match run_pim(&state, &me.account_id, &me.username, true, "preview").await {
            Ok(s) => no_store(Json(s)),
            Err(e) => fail(StatusCode::BAD_GATEWAY, "mailUnavailable", e),
        },
        Err(r) => r,
    }
}

async fn run(State(state): State<AppState>, headers: HeaderMap) -> Response {
    match me(&state, &headers).await {
        Ok(me) => match run_pim(&state, &me.account_id, &me.username, false, "user").await {
            Ok(s) => no_store(Json(s)),
            Err(e) => fail(StatusCode::BAD_GATEWAY, "mailUnavailable", e),
        },
        Err(r) => r,
    }
}

async fn restore(State(state): State<AppState>, headers: HeaderMap, Path(id): Path<String>) -> Response {
    let me = match me(&state, &headers).await {
        Ok(m) => m,
        Err(r) => return r,
    };
    match state.store.note_restore(&me.account_id, &id).await {
        Ok(true) => no_store(Json(json!({"restored": true}))),
        Ok(false) => fail(StatusCode::NOT_FOUND, "notFound", "No such note in the undo list."),
        Err(e) => store_fail(e),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn policy() -> PimPolicy {
        PimPolicy { cal_enabled: true, cal_min: 30, cal_max: 365, notes_enabled: false, notes_min: 30, notes_max: 3650, undo_days: 7 }
    }

    #[test]
    fn calendar_opt_in_is_validated_against_the_administrators_limits() {
        let p = policy();
        let ok = CalendarCfg { enabled: true, days: 90, calendars: vec!["c1".into()] };
        assert!(validate_calendar(&ok, &p).is_ok());
        assert_eq!(validate_calendar(&CalendarCfg { days: 10, ..ok.clone() }, &p).unwrap_err().0, "belowMinimum");
        assert_eq!(validate_calendar(&CalendarCfg { days: 400, ..ok.clone() }, &p).unwrap_err().0, "aboveMaximum");
        assert_eq!(validate_calendar(&CalendarCfg { calendars: vec![], ..ok.clone() }, &p).unwrap_err().0, "calendarRequired");
        let off = PimPolicy { cal_enabled: false, ..p };
        assert_eq!(validate_calendar(&ok, &off).unwrap_err().0, "pimDisabled");
        assert!(validate_calendar(&CalendarCfg::default(), &off).is_ok(), "turning it off is always allowed");
    }

    #[test]
    fn notes_are_refused_while_the_administrator_has_them_off() {
        let p = policy();
        let on = NotesCfg { enabled: true, days: 180 };
        assert_eq!(validate_notes(&on, &p).unwrap_err().0, "pimDisabled");
        let allowed = PimPolicy { notes_enabled: true, ..p };
        assert!(validate_notes(&on, &allowed).is_ok());
        assert_eq!(validate_notes(&NotesCfg { enabled: true, days: 5 }, &allowed).unwrap_err().0, "belowMinimum");
    }

    async fn store() -> (Store, std::path::PathBuf) {
        let dir = std::env::temp_dir().join(format!("mw-pim-{}", uuid::Uuid::new_v4()));
        std::fs::create_dir_all(&dir).unwrap();
        let s = Store::open(dir.join("s.sqlite").to_str().unwrap(), mw_store::ServerKey::from_bytes(&[6u8; 32]).unwrap()).await.unwrap();
        (s, dir)
    }

    #[tokio::test]
    async fn policy_defaults_to_off_and_reads_the_settings() {
        let (s, dir) = store().await;
        let p = pim_policy(&s, "a", "a@example.test").await;
        assert!(!p.cal_enabled && !p.notes_enabled);
        assert_eq!((p.cal_min, p.cal_max, p.notes_min, p.notes_max), (30, 3650, 30, 3650));
        s.put_setting_policy(&mw_store::settings_policy::SettingPolicyRow {
            key: "retention.user.notes.enabled".into(),
            global_value: Some("true".into()),
            allow_user_override: false,
            global_locked: true,
            updated_by: "test".into(),
            updated_at: 1,
        })
        .await
        .unwrap();
        assert!(pim_policy(&s, "a", "a@example.test").await.notes_enabled);
        std::fs::remove_dir_all(dir).ok();
    }

    #[tokio::test]
    async fn config_round_trips_and_a_held_note_is_restored_sealed() {
        let (s, dir) = store().await;
        let acct = s
            .create_account(
                &mw_store::NewAccount { kind: mw_store::AccountKind::Imap, host: "h", port: 993, tls: "implicit", username: "a@example.test", sync_policy_json: "{}" },
                &mw_store::Credentials { username: "a@example.test".into(), password: "pw".into() },
            )
            .await
            .unwrap();
        s.pim_retention_config_put(&acct, "notes", r#"{"enabled":true,"days":90}"#, 1).await.unwrap();
        assert_eq!(s.pim_retention_config(&acct, "notes").await.unwrap().as_deref(), Some(r#"{"enabled":true,"days":90}"#));
        let note = mw_store::NoteRow {
            id: "n1".into(), account_id: acct.clone(), notebook_id: None, title: "Old plan".into(), tags_json: "[\"x\"]".into(),
            color: "".into(), pinned: false, body_html: "<p>secret</p>".into(), body_text: "secret".into(), links_json: "[]".into(),
            created_at: "2020-01-01T00:00:00Z".into(), updated_at: "2020-01-02T00:00:00Z".into(),
        };
        s.upsert_note(&note).await.unwrap();
        let pinned = mw_store::NoteRow { id: "n2".into(), pinned: true, ..note.clone() };
        s.upsert_note(&pinned).await.unwrap();
        let old = s.notes_untouched_since(&acct, "2021-01-01T00:00:00Z", 10).await.unwrap();
        assert_eq!(old, vec!["n1".to_string()], "pinned notes are never selected");
        assert!(s.notes_untouched_since(&acct, "2019-01-01T00:00:00Z", 10).await.unwrap().is_empty());
        assert!(s.note_hold_copy(&acct, "n1", 100, 100 + 7 * 86400).await.unwrap());
        s.delete_note("n1").await.unwrap();
        let held = s.note_holds(&acct).await.unwrap();
        assert_eq!((held.len(), held[0].title.as_str()), (1, "Old plan"));
        assert!(!s.note_restore("someone-else", "n1").await.unwrap());
        assert!(s.note_restore(&acct, "n1").await.unwrap());
        let back = s.get_note("n1").await.unwrap().unwrap();
        assert_eq!((back.title.as_str(), back.body_text.as_str(), back.updated_at.as_str()), ("Old plan", "secret", "2020-01-02T00:00:00Z"));
        assert!(s.note_holds(&acct).await.unwrap().is_empty());
        // A hold past its due time is purged.
        s.note_hold_copy(&acct, "n1", 100, 200).await.unwrap();
        assert_eq!(s.note_holds_purge(150).await.unwrap(), 0);
        assert_eq!(s.note_holds_purge(250).await.unwrap(), 1);
        std::fs::remove_dir_all(dir).ok();
    }
}
