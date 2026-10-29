//! Configuration backups sent to Nextcloud (t64).
//!
//! The portable settings export (`/admin/config/export`) can be uploaded to one
//! administrator-owned Nextcloud folder. This is a server credential, kept apart
//! from the users' account-isolated Nextcloud links: the app password is sealed
//! with the server key, bound to the server URL and username, is write-only in
//! this API and is never logged or returned.
//!
//! Every request goes through the guarded egress plan as a credential-bearing
//! DAV request (`TrafficClass::Nextcloud`): HTTPS only, no redirects, the SSRF
//! address policy, the operator's proxy routes, and the optional pinned CA
//! bundle as the only extra trust. A copy is verified by downloading it again and
//! comparing size and SHA-256 before retention removes older copies, and retention
//! only removes files named like the ones this module writes.
use super::*;
use mw_egress::{AuthorizedDav, DavMethod, OutBody, Refusal};
use serde::Serialize;
use sha2::{Digest, Sha256};

const KEY: &str = "admin.configBackup.nextcloud.v1";
const LAST_KEY: &str = "admin.configBackup.nextcloud.last.v1";
const PREFIX: &str = "mailwoman-settings-";
const DEFAULT_KEEP: u32 = 7;
pub(super) const APP_PASSWORD_DOCS: &str = "https://docs.nextcloud.com/server/latest/user_manual/en/session_management.html#managing-devices";

#[derive(Clone, Debug, Default, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(super) struct Config {
    base_url: String,
    username: String,
    folder: String,
    keep: u32,
    #[serde(default)]
    keep_days: Option<u32>,
    #[serde(default)]
    ca_pem: String,
    /// Base64 of the sealed app password.
    #[serde(default)]
    app_password_sealed: Option<String>,
}
impl Config {
    fn public(&self) -> Value {
        json!({
            "baseUrl": self.base_url,
            "username": self.username,
            "folder": self.folder,
            "keep": self.keep,
            "keepDays": self.keep_days,
            "caPem": self.ca_pem,
            "hasAppPassword": self.app_password_sealed.is_some(),
            "webdavUrl": crate::nextcloud_config::file_url(&self.base_url, &self.username, &format!("{}/", self.folder)).ok().map(|url| url.to_string()),
        })
    }
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct Update {
    base_url: String,
    username: String,
    folder: String,
    #[serde(default)]
    keep: Option<u32>,
    #[serde(default)]
    keep_days: Option<u32>,
    #[serde(default)]
    ca_pem: String,
    /// Write-only. Absent keeps the stored password when URL and username are unchanged.
    #[serde(default)]
    app_password: Option<String>,
    #[serde(default)]
    clear_app_password: bool,
}

pub(super) fn folder(value: &str) -> Result<String, &'static str> {
    let trimmed = value.trim().trim_matches('/');
    if trimmed.is_empty()
        || trimmed.len() > 512
        || trimmed.contains('\\')
        || trimmed.chars().any(char::is_control)
    {
        return Err("Enter a target folder such as Backups/Mailwoman.");
    }
    let parts: Vec<_> = trimmed.split('/').collect();
    if parts.len() > 16
        || parts
            .iter()
            .any(|part| part.is_empty() || matches!(*part, "." | "..") || part.trim() != *part)
    {
        return Err("The target folder may not contain empty, dot or space-padded segments.");
    }
    Ok(parts.join("/"))
}

fn app_password(value: &str) -> Result<(), &'static str> {
    if value.is_empty()
        || value.len() > 256
        || value.chars().any(|c| c.is_control() || c.is_whitespace())
    {
        return Err("Enter the Nextcloud app password (no spaces).");
    }
    Ok(())
}

impl Update {
    fn prepare(self, previous: &Config, store: &mw_store::Store) -> Result<Config, AdminApiError> {
        let bad = |code: &'static str, message: &'static str, field: &'static str| {
            AdminApiError::invalid(code, message, field)
        };
        let base = crate::nextcloud_config::base_url(self.base_url.trim())
            .map_err(|m| bad("invalidUrl", m, "/baseUrl"))?;
        if base.path().contains("/remote.php") {
            return Err(bad(
                "invalidUrl",
                "Enter the Nextcloud server URL only; the WebDAV path is added automatically.",
                "/baseUrl",
            ));
        }
        crate::nextcloud_config::username(&self.username)
            .map_err(|m| bad("invalidUsername", m, "/username"))?;
        let folder = folder(&self.folder).map_err(|m| bad("invalidFolder", m, "/folder"))?;
        let keep = self.keep.unwrap_or(DEFAULT_KEEP);
        if !(1..=1000).contains(&keep) {
            return Err(bad("invalidKeep", "Keep 1 to 1000 copies.", "/keep"));
        }
        if self
            .keep_days
            .is_some_and(|days| !(1..=36500).contains(&days))
        {
            return Err(bad(
                "invalidKeepDays",
                "Keep days must be 1 to 36500.",
                "/keepDays",
            ));
        }
        crate::nextcloud_config::ca_roots(&self.ca_pem)
            .map_err(|m| bad("invalidCa", m, "/caPem"))?;
        if self.clear_app_password && self.app_password.is_some() {
            return Err(bad(
                "invalidAppPassword",
                "Choose either a new app password or removal.",
                "/appPassword",
            ));
        }
        let mut config = Config {
            base_url: base.to_string(),
            username: self.username,
            folder,
            keep,
            keep_days: self.keep_days,
            ca_pem: self.ca_pem,
            app_password_sealed: None,
        };
        if let Some(secret) = self.app_password {
            app_password(&secret).map_err(|m| bad("invalidAppPassword", m, "/appPassword"))?;
            let sealed = store
                .seal_config_backup_secret(secret.as_bytes(), &config.base_url, &config.username)
                .map_err(|_| AdminApiError::internal("seal"))?;
            config.app_password_sealed =
                Some(base64::engine::general_purpose::STANDARD.encode(sealed));
        } else if !self.clear_app_password
            && previous.base_url == config.base_url
            && previous.username == config.username
        {
            // A credential is bound to its URL and user; changing either needs it again.
            config.app_password_sealed = previous.app_password_sealed.clone();
        }
        Ok(config)
    }
}

use base64::Engine as _;

async fn load(store: &mw_store::Store) -> Result<(Option<String>, Option<Config>), AdminApiError> {
    let raw = store
        .get_setting(KEY)
        .await
        .map_err(AdminApiError::internal)?;
    let config = match raw.as_deref() {
        None | Some("") => None,
        Some(raw) => {
            Some(serde_json::from_str(raw).map_err(|_| AdminApiError::internal("config backup"))?)
        }
    };
    Ok((raw, config))
}

pub(super) fn register(r: &mut impl AdminRegistrar) {
    r.admin_endpoint(
        Method::GET,
        "/config/backup/nextcloud",
        Capability::ConfigExport,
        read,
    );
    r.admin_endpoint(
        Method::PUT,
        "/config/backup/nextcloud",
        Capability::ConfigExport,
        save,
    );
    r.admin_endpoint(
        Method::DELETE,
        "/config/backup/nextcloud",
        Capability::ConfigExport,
        remove,
    );
    r.admin_endpoint(
        Method::POST,
        "/config/backup/nextcloud/test",
        Capability::ConfigExport,
        test,
    );
    r.admin_endpoint(
        Method::POST,
        "/config/backup/nextcloud/run",
        Capability::ConfigExport,
        run,
    );
}

fn etag(config: &Option<Config>) -> String {
    etag_of(&json!(config))
}

async fn view(state: &AppState, config: &Option<Config>) -> Result<Response, AdminApiError> {
    let last = state
        .store
        .get_setting(LAST_KEY)
        .await
        .map_err(AdminApiError::internal)?
        .and_then(|raw| serde_json::from_str::<Value>(&raw).ok());
    Ok(with_etag(
        (
            [(axum::http::header::CACHE_CONTROL, "private, no-store")],
            Json(json!({
                "config": config.as_ref().map(Config::public),
                "lastRun": last,
                "appPasswordHelp": APP_PASSWORD_DOCS,
            })),
        ),
        &etag(config),
    ))
}

async fn read(State(state): State<AppState>, ctx: AdminCtx) -> Result<Response, AdminApiError> {
    ctx.require_unscoped(Capability::ConfigExport)?;
    let (_, config) = load(&state.store).await?;
    view(&state, &config).await
}

async fn save(
    State(state): State<AppState>,
    ctx: AdminCtx,
    headers: HeaderMap,
    AdminJson(update): AdminJson<Update>,
) -> Result<Response, AdminApiError> {
    ctx.require_unscoped(Capability::ConfigExport)?;
    crate::step_up::require_admin(
        &state,
        &ctx,
        crate::step_up::StepUpAction::IntegrationManage,
    )
    .await?;
    let (raw, previous) = load(&state.store).await?;
    if !headers.contains_key("if-match") {
        return Err(AdminApiError::new(
            StatusCode::PRECONDITION_REQUIRED,
            "ifMatchRequired",
            "Reload the configuration before saving.",
        ));
    }
    check_if_match(&headers, &etag(&previous), None, None)?;
    let config = update.prepare(&previous.clone().unwrap_or_default(), &state.store)?;
    let serialized = serde_json::to_string(&config).expect("config backup");
    if !state
        .store
        .compare_and_set_setting(KEY, raw.as_deref(), &serialized)
        .await
        .map_err(AdminApiError::internal)?
    {
        return Err(AdminApiError::stale(None, None));
    }
    let credential = match (&previous, &config.app_password_sealed) {
        (_, None) => "none",
        (Some(p), Some(s)) if p.app_password_sealed.as_ref() == Some(s) => "kept",
        _ => "replaced",
    };
    crate::admin_accounts::audit(
        &state,
        &ctx,
        "config-backup-nextcloud-configured",
        "config-backup:nextcloud",
        None,
        Some(json!({"baseUrl":config.base_url,"username":config.username,"folder":config.folder,"keep":config.keep,"keepDays":config.keep_days,"appPassword":credential})),
    )
    .await;
    view(&state, &Some(config)).await
}

async fn remove(
    State(state): State<AppState>,
    ctx: AdminCtx,
    headers: HeaderMap,
) -> Result<Response, AdminApiError> {
    ctx.require_unscoped(Capability::ConfigExport)?;
    crate::step_up::require_admin(
        &state,
        &ctx,
        crate::step_up::StepUpAction::IntegrationManage,
    )
    .await?;
    let (raw, previous) = load(&state.store).await?;
    check_if_match(&headers, &etag(&previous), None, None)?;
    if previous.is_none() {
        return Err(AdminApiError::not_found(
            "No Nextcloud backup destination is configured.",
        ));
    }
    if !state
        .store
        .compare_and_set_setting(KEY, raw.as_deref(), "")
        .await
        .map_err(AdminApiError::internal)?
    {
        return Err(AdminApiError::stale(None, None));
    }
    crate::admin_accounts::audit(
        &state,
        &ctx,
        "config-backup-nextcloud-removed",
        "config-backup:nextcloud",
        None,
        None,
    )
    .await;
    view(&state, &None).await
}

// ─── transport ────────────────────────────────────────────────────────────────

/// A failure the UI can name. `code` selects the translated message.
#[derive(Debug, PartialEq)]
pub(super) struct Failure {
    pub step: &'static str,
    pub code: &'static str,
    pub status: Option<u16>,
}
fn fail(step: &'static str, code: &'static str, status: Option<u16>) -> Failure {
    Failure { step, code, status }
}

/// Map a transport refusal; the status codes that mean "credentials" or
/// "missing" differ by step, so the step supplies its own reading of them.
pub(super) fn transport(step: &'static str, refusal: &Refusal) -> Failure {
    match refusal {
        Refusal::Status(401) => fail(step, "authRejected", Some(401)),
        Refusal::Status(507) => fail(step, "quotaExceeded", Some(507)),
        Refusal::Status(code @ 300..=399) => fail(step, "redirected", Some(*code)),
        Refusal::Status(code) => fail(step, "refused", Some(*code)),
        Refusal::Blocked => fail(step, "egressBlocked", None),
        Refusal::Timeout => fail(step, "timeout", None),
        Refusal::TooLarge => fail(step, "tooLarge", None),
        Refusal::BadRequest(_) => fail(step, "invalidRequest", None),
        Refusal::Upstream => fail(step, "unreachable", None),
    }
}

struct Ready {
    base: String,
    user: String,
    roots: Vec<Vec<u8>>,
    folder: String,
    authorization: zeroize::Zeroizing<String>,
}
impl Ready {
    fn new(state: &AppState, config: &Config) -> Result<Self, Failure> {
        let sealed =
            config
                .app_password_sealed
                .as_deref()
                .ok_or(fail("config", "noAppPassword", None))?;
        let sealed = base64::engine::general_purpose::STANDARD
            .decode(sealed)
            .map_err(|_| fail("config", "credentialUnreadable", None))?;
        let password = state
            .store
            .open_config_backup_secret(&sealed, &config.base_url, &config.username)
            .map_err(|_| fail("config", "credentialUnreadable", None))?;
        let password = zeroize::Zeroizing::new(
            String::from_utf8(password)
                .map_err(|_| fail("config", "credentialUnreadable", None))?,
        );
        let roots = crate::nextcloud_config::ca_roots(&config.ca_pem)
            .map_err(|_| fail("config", "invalidCa", None))?;
        let authorization = zeroize::Zeroizing::new(format!(
            "Basic {}",
            base64::engine::general_purpose::STANDARD.encode(format!(
                "{}:{}",
                config.username,
                password.as_str()
            ))
        ));
        Ok(Self {
            base: config.base_url.clone(),
            user: config.username.clone(),
            roots,
            folder: config.folder.clone(),
            authorization,
        })
    }
    fn url(&self, path: &str) -> Result<reqwest::Url, Failure> {
        crate::nextcloud_config::file_url(&self.base, &self.user, path)
            .map_err(|_| fail("config", "invalidRequest", None))
    }
    fn file(&self, name: &str) -> Result<reqwest::Url, Failure> {
        self.url(&format!("{}/{}", self.folder, name))
    }
    async fn call(
        &self,
        state: &AppState,
        url: reqwest::Url,
        method: DavMethod,
        body: Option<(&str, &[u8])>,
        depth: Option<u8>,
        create_only: bool,
    ) -> Result<Vec<u8>, Refusal> {
        let plan = crate::image_proxy::load_plan(&state.store)
            .await
            .map_err(|_| Refusal::BadRequest("egress unavailable"))?;
        let request = mw_egress::plan::Request::dav(
            AuthorizedDav {
                additional_roots: &self.roots,
                method,
                authorization: &self.authorization,
                depth,
                ocs: true,
                if_match: None,
                if_none_match: create_only.then_some("*"),
                destination: None,
            },
            body.map(|(content_type, bytes)| OutBody {
                content_type,
                bytes,
            }),
        )?
        .response_limit(mw_egress::MAX_IMAGE_BYTES);
        match tokio::time::timeout(
            std::time::Duration::from_secs(30),
            mw_egress::plan::send_with(&plan, url, &request),
        )
        .await
        {
            Ok(response) => response.outcome,
            Err(_) => Err(Refusal::Timeout),
        }
    }

    /// Check the credentials on the user's files root, then create every missing
    /// segment of the target folder with MKCOL. Returns whether anything was created.
    async fn ensure_folder(&self, state: &AppState) -> Result<bool, Failure> {
        let root = self.url("/")?;
        match self
            .call(
                state,
                root,
                DavMethod::Propfind,
                Some(("application/xml", crate::nextcloud_transport::PROPERTIES)),
                Some(0),
                false,
            )
            .await
        {
            Ok(_) => {}
            Err(Refusal::Status(404)) => return Err(fail("auth", "userNotFound", Some(404))),
            Err(other) => return Err(transport("auth", &other)),
        }
        let mut created = false;
        let mut path = String::new();
        for segment in self.folder.split('/') {
            path.push('/');
            path.push_str(segment);
            let url = self.url(&format!("{path}/"))?;
            match self
                .call(
                    state,
                    url.clone(),
                    DavMethod::Propfind,
                    Some(("application/xml", crate::nextcloud_transport::PROPERTIES)),
                    Some(0),
                    false,
                )
                .await
            {
                Ok(body) if is_collection(&body) => continue,
                Ok(_) => return Err(fail("folder", "folderIsFile", None)),
                Err(Refusal::Status(404)) => {}
                Err(other) => return Err(transport("folder", &other)),
            }
            match self
                .call(state, url, DavMethod::Mkcol, None, None, false)
                .await
            {
                Ok(_) | Err(Refusal::Status(405)) => created = true,
                Err(Refusal::Status(code @ (403 | 409))) => {
                    return Err(fail("folder", "folderCreateDenied", Some(code)));
                }
                Err(other) => return Err(transport("folder", &other)),
            }
        }
        Ok(created)
    }

    /// PUT a new file (never overwriting), then download it and compare size and SHA-256.
    async fn upload_verified(
        &self,
        state: &AppState,
        name: &str,
        content_type: &str,
        bytes: &[u8],
    ) -> Result<(), Failure> {
        self.call(
            state,
            self.file(name)?,
            DavMethod::Put,
            Some((content_type, bytes)),
            None,
            true,
        )
        .await
        .map_err(|refusal| match refusal {
            Refusal::Status(412) => fail("write", "fileExists", Some(412)),
            Refusal::Status(code @ (403 | 409)) => fail("write", "writeDenied", Some(code)),
            other => transport("write", &other),
        })?;
        let copy = self
            .call(state, self.file(name)?, DavMethod::Get, None, None, false)
            .await
            .map_err(|refusal| transport("verify", &refusal))?;
        if copy.len() != bytes.len() || Sha256::digest(&copy) != Sha256::digest(bytes) {
            return Err(fail("verify", "verifyMismatch", None));
        }
        Ok(())
    }

    async fn delete(&self, state: &AppState, name: &str) -> Result<(), Failure> {
        match self
            .call(
                state,
                self.file(name)?,
                DavMethod::Delete,
                None,
                None,
                false,
            )
            .await
        {
            Ok(_) | Err(Refusal::Status(404)) => Ok(()),
            Err(Refusal::Status(code @ (403 | 409 | 423))) => {
                Err(fail("delete", "deleteDenied", Some(code)))
            }
            Err(other) => Err(transport("delete", &other)),
        }
    }

    async fn list(&self, state: &AppState) -> Result<Vec<String>, Failure> {
        let folder = self.url(&format!("/{}/", self.folder))?;
        let body = self
            .call(
                state,
                folder.clone(),
                DavMethod::Propfind,
                Some(("application/xml", crate::nextcloud_transport::PROPERTIES)),
                Some(1),
                false,
            )
            .await
            .map_err(|refusal| transport("prune", &refusal))?;
        let root = self.url("/")?;
        let entries = crate::nextcloud_xml::entries(&body, &root, &folder)
            .map_err(|_| fail("prune", "invalidListing", None))?;
        Ok(entries
            .into_iter()
            .filter(|entry| !entry.is_dir)
            .map(|entry| entry.name)
            .collect())
    }
}

fn is_collection(body: &[u8]) -> bool {
    let text = String::from_utf8_lossy(body);
    text.contains(":collection") || text.contains("<collection")
}

/// `mailwoman-settings-YYYYMMDDTHHMMSSZ.json` → its timestamp.
pub(super) fn stamp_of(name: &str) -> Option<&str> {
    let stamp = name.strip_prefix(PREFIX)?.strip_suffix(".json")?;
    let bytes = stamp.as_bytes();
    (bytes.len() == 16
        && bytes[8] == b'T'
        && bytes[15] == b'Z'
        && bytes[..8]
            .iter()
            .chain(&bytes[9..15])
            .all(u8::is_ascii_digit))
    .then_some(stamp)
}

/// Which copies `keep`/`keepDays` remove: outside the newest `keep`, and (when
/// keepDays is set) also older than it. Never the copy just written.
pub(super) fn prune_plan(
    names: &[String],
    keep: u32,
    keep_days: Option<u32>,
    current: &str,
    now: chrono::DateTime<chrono::Utc>,
) -> Vec<String> {
    let mut ours: Vec<&String> = names
        .iter()
        .filter(|name| stamp_of(name).is_some())
        .collect();
    ours.sort_by(|a, b| b.cmp(a));
    ours.dedup();
    let cutoff = keep_days.map(|days| {
        (now - chrono::Duration::days(i64::from(days)))
            .format("%Y%m%dT%H%M%SZ")
            .to_string()
    });
    ours.into_iter()
        .enumerate()
        .filter(|(index, name)| {
            let stamp = stamp_of(name).unwrap_or_default();
            *name != current
                && *index as u32 >= keep
                && cutoff.as_deref().is_none_or(|cutoff| stamp < cutoff)
        })
        .map(|(_, name)| name.clone())
        .collect()
}

fn failure_json(failure: &Failure) -> Value {
    json!({"ok":false,"step":failure.step,"code":failure.code,"status":failure.status,"appPasswordHelp":APP_PASSWORD_DOCS})
}

static RUNNING: tokio::sync::Mutex<()> = tokio::sync::Mutex::const_new(());

async fn ready(state: &AppState) -> Result<(Config, Ready), Result<Response, AdminApiError>> {
    let (_, config) = load(&state.store).await.map_err(Err)?;
    let Some(config) = config else {
        return Err(Err(AdminApiError::not_found(
            "Configure the Nextcloud destination first.",
        )));
    };
    match Ready::new(state, &config) {
        Ok(ready) => Ok((config, ready)),
        Err(failure) => Err(Ok(Json(failure_json(&failure)).into_response())),
    }
}

async fn test(State(state): State<AppState>, ctx: AdminCtx) -> Result<Response, AdminApiError> {
    ctx.require_unscoped(Capability::ConfigExport)?;
    let Ok(_guard) = RUNNING.try_lock() else {
        return Err(AdminApiError::conflict(
            "backupRunning",
            "A Nextcloud backup or test is already running.",
        ));
    };
    let (_, ready) = match ready(&state).await {
        Ok(ready) => ready,
        Err(response) => return response,
    };
    let outcome = async {
        let created = ready.ensure_folder(&state).await?;
        let name = format!("mailwoman-probe-{}.txt", uuid::Uuid::new_v4().simple());
        ready
            .upload_verified(
                &state,
                &name,
                "text/plain",
                b"Mailwoman backup destination probe\n",
            )
            .await?;
        ready.delete(&state, &name).await?;
        Ok::<_, Failure>(created)
    }
    .await;
    Ok(Json(match outcome {
        Ok(created) => json!({"ok":true,"folderCreated":created,"steps":["auth","folder","write","verify","delete"]}),
        Err(failure) => failure_json(&failure),
    })
    .into_response())
}

async fn run(State(state): State<AppState>, ctx: AdminCtx) -> Result<Response, AdminApiError> {
    ctx.require_unscoped(Capability::ConfigExport)?;
    ctx.require_unscoped(Capability::SettingsRead)?;
    let Ok(_guard) = RUNNING.try_lock() else {
        return Err(AdminApiError::conflict(
            "backupRunning",
            "A Nextcloud backup or test is already running.",
        ));
    };
    let (config, ready) = match ready(&state).await {
        Ok(ready) => ready,
        Err(response) => return response,
    };
    let (bundle, excluded) = super::portable_bundle(&state).await?;
    let bytes = serde_json::to_vec_pretty(&json!({
        "bundle": bundle,
        "excluded": excluded,
        "scope": "registered-global-policies",
        "includesSecrets": false,
        "includesDomains": false,
        "includesMail": false,
    }))
    .map_err(AdminApiError::internal)?;
    let now = chrono::Utc::now();
    let name = format!("{PREFIX}{}.json", now.format("%Y%m%dT%H%M%SZ"));
    let digest: String = Sha256::digest(&bytes)
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect();
    let outcome = async {
        ready.ensure_folder(&state).await?;
        ready
            .upload_verified(&state, &name, "application/json", &bytes)
            .await?;
        // Retention follows a verified copy; its failure leaves extra copies, never fewer.
        let removed = match ready.list(&state).await {
            Ok(names) => {
                let mut removed = Vec::new();
                for victim in prune_plan(&names, config.keep, config.keep_days, &name, now) {
                    if ready.delete(&state, &victim).await.is_ok() {
                        removed.push(victim);
                    }
                }
                Ok(removed)
            }
            Err(failure) => Err(failure),
        };
        Ok::<_, Failure>(removed)
    }
    .await;
    let record = match &outcome {
        Ok(removed) => json!({
            "at": now.timestamp(), "ok": true, "file": name, "bytes": bytes.len(), "sha256": digest,
            "removed": removed.as_ref().map(Vec::len).ok(), "pruneError": removed.as_ref().err().map(|f| f.code),
        }),
        Err(failure) => {
            json!({"at": now.timestamp(), "ok": false, "step": failure.step, "code": failure.code, "status": failure.status})
        }
    };
    let _ = state.store.set_setting(LAST_KEY, &record.to_string()).await;
    crate::admin_accounts::audit(
        &state,
        &ctx,
        "config-backup-nextcloud-run",
        "config-backup:nextcloud",
        None,
        Some(record.clone()),
    )
    .await;
    Ok(Json(match outcome {
        Ok(removed) => json!({"ok":true,"file":name,"bytes":bytes.len(),"sha256":digest,"removed":removed.unwrap_or_default(),"folder":config.folder}),
        Err(failure) => failure_json(&failure),
    })
    .into_response())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn folder_is_relative_bounded_and_without_traversal() {
        assert_eq!(folder("/Backups/Mailwoman/").unwrap(), "Backups/Mailwoman");
        for bad in [
            "", "/", "a/../b", "a//b", "./a", "a\\b", "a\nb", "a/ b",
        ] {
            assert!(folder(bad).is_err(), "{bad:?}");
        }
    }

    #[test]
    fn only_own_timestamped_files_are_pruned_and_never_the_current_one() {
        let now = chrono::DateTime::parse_from_rfc3339("2026-10-29T00:00:00Z")
            .unwrap()
            .to_utc();
        let names: Vec<String> = [
            "mailwoman-settings-20261029T000000Z.json",
            "mailwoman-settings-20261028T000000Z.json",
            "mailwoman-settings-20261001T000000Z.json",
            "mailwoman-settings-20250101T000000Z.json",
            "mailwoman-settings-latest.json",
            "notes.txt",
            "mailwoman-settings-20240101T000000Z.json.bak",
        ]
        .map(String::from)
        .to_vec();
        assert_eq!(
            prune_plan(
                &names,
                2,
                None,
                "mailwoman-settings-20261029T000000Z.json",
                now
            ),
            [
                "mailwoman-settings-20261001T000000Z.json",
                "mailwoman-settings-20250101T000000Z.json"
            ]
        );
        assert_eq!(
            prune_plan(
                &names,
                1,
                Some(60),
                "mailwoman-settings-20261029T000000Z.json",
                now
            ),
            ["mailwoman-settings-20250101T000000Z.json"]
        );
        // The copy just written survives even when keep would drop it.
        assert!(
            !prune_plan(
                &names,
                1,
                None,
                "mailwoman-settings-20250101T000000Z.json",
                now
            )
            .contains(&"mailwoman-settings-20250101T000000Z.json".to_string())
        );
    }

    #[test]
    fn transport_failures_map_to_named_codes() {
        assert_eq!(
            transport("auth", &Refusal::Status(401)).code,
            "authRejected"
        );
        assert_eq!(
            transport("write", &Refusal::Status(507)).code,
            "quotaExceeded"
        );
        assert_eq!(transport("auth", &Refusal::Blocked).code, "egressBlocked");
        assert_eq!(transport("auth", &Refusal::Status(302)).code, "redirected");
        assert_eq!(transport("auth", &Refusal::Upstream).code, "unreachable");
    }

    #[tokio::test]
    async fn app_password_is_sealed_bound_write_only_and_dropped_when_authority_changes() {
        let store = mw_store::Store::open_in_memory(mw_store::ServerKey::generate())
            .await
            .unwrap();
        let update = |password: Option<&str>, user: &str| Update {
            base_url: "https://cloud.example/nc".into(),
            username: user.into(),
            folder: "Backups/Mailwoman".into(),
            keep: None,
            keep_days: Some(30),
            ca_pem: String::new(),
            app_password: password.map(String::from),
            clear_app_password: false,
        };
        let config = update(Some("Abcde-Fghij-Klmno-Pqrst-Uvwxy"), "backup")
            .prepare(&Config::default(), &store)
            .unwrap();
        assert_eq!(config.base_url, "https://cloud.example/nc/");
        assert_eq!(config.keep, DEFAULT_KEEP);
        let serialized = serde_json::to_string(&config).unwrap();
        assert!(!serialized.contains("Abcde-Fghij"));
        let public = config.public().to_string();
        assert!(!public.contains("Abcde") && !public.contains("appPasswordSealed"));
        assert!(public.contains("\"hasAppPassword\":true"));
        assert!(public.contains("/nc/remote.php/dav/files/backup/Backups/Mailwoman/"));
        let kept = update(None, "backup").prepare(&config, &store).unwrap();
        assert_eq!(kept.app_password_sealed, config.app_password_sealed);
        let moved = update(None, "other").prepare(&config, &store).unwrap();
        assert!(moved.app_password_sealed.is_none());
        for bad in [
            Update {
                base_url: "http://cloud.example".into(),
                ..update(None, "backup")
            },
            Update {
                base_url: "https://cloud.example/remote.php/dav/files/backup".into(),
                ..update(None, "backup")
            },
            Update {
                folder: "../x".into(),
                ..update(None, "backup")
            },
            Update {
                keep: Some(0),
                ..update(None, "backup")
            },
            update(Some("has space"), "backup"),
        ] {
            assert!(bad.prepare(&config, &store).is_err());
        }
    }
}
