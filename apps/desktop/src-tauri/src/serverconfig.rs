//! The Mailwoman server the desktop shell talks to (t32 §2.7).
//!
//! The user never enters or changes it. It comes from, in this order:
//!   1. a **provisioning file** an administrator installs, `server.json` with
//!      `{ "serverUrl": "https://…" }`, at [`provisioning_path`]
//!      (`%ProgramData%\Mailwoman\server.json` on Windows,
//!      `/Library/Application Support/Mailwoman/server.json` on macOS,
//!      `/etc/mailwoman/server.json` elsewhere);
//!   2. the address **baked in at build time** from `MW_SHELL_SERVER_URL`
//!      (`scripts/build-shells.*`);
//!   3. none: the SPA then shows that the app has not been set up.
//!
//! A provisioning file that exists but cannot be used (unreadable, not JSON, not a
//! valid address, or writable by someone other than an administrator) gives **no
//! server**, not the baked one: the administrator meant to override the build, and
//! falling back would silently connect to a server they replaced. The reason is
//! logged to stderr.
//!
//! The file is trusted only when an administrator owns it and its folder (Windows:
//! BUILTIN\Administrators or SYSTEM; Unix: root, and neither writable by group or
//! others). On Windows a standard user can create folders and files under
//! `%ProgramData%`, so without this check any local user could point every other
//! user's app at their own server.

use serde::Deserialize;
use std::path::{Path, PathBuf};

/// The address baked in at build time, if any. The build fails on a value that
/// is not an https address (or http to a loopback host), below.
pub const BAKED_SERVER_URL: Option<&str> = option_env!("MW_SHELL_SERVER_URL");

/// `MW_SHELL_SERVER_URL`, when set, must be `https://host…` or `http://` to a
/// loopback host, with no user name, query, fragment or whitespace: checked here
/// so a build with any other value fails to compile, whoever runs it.
/// [`validate_server_url`] checks it again at start.
pub const fn baked_url_is_acceptable(url: &str) -> bool {
    const fn starts_with(b: &[u8], prefix: &[u8]) -> bool {
        if b.len() < prefix.len() {
            return false;
        }
        let mut i = 0;
        while i < prefix.len() {
            if b[i] != prefix[i] {
                return false;
            }
            i += 1;
        }
        true
    }
    let b = url.as_bytes();
    let mut i = 0;
    while i < b.len() {
        let c = b[i];
        if c <= b' ' || c == 0x7f || c == b'@' || c == b'?' || c == b'#' || c == b'\\' {
            return false;
        }
        i += 1;
    }
    let (https, start) = if starts_with(b, b"https://") {
        (true, 8)
    } else if starts_with(b, b"http://") {
        (false, 7)
    } else {
        return false;
    };
    // The host runs to the first '/' or, outside brackets, ':'.
    let mut end = start;
    let mut bracket = false;
    while end < b.len() {
        let c = b[end];
        if c == b'[' {
            bracket = true;
        } else if c == b']' {
            bracket = false;
        } else if c == b'/' || (c == b':' && !bracket) {
            break;
        }
        end += 1;
    }
    if end == start {
        return false;
    }
    if https {
        return true;
    }
    let host = b.split_at(end).0.split_at(start).1;
    if host.len() == 9 && starts_with(host, b"localhost") {
        return true;
    }
    if host.len() == 5 && starts_with(host, b"[::1]") {
        return true;
    }
    if !starts_with(host, b"127.") {
        return false;
    }
    let mut k = 0;
    while k < host.len() {
        if !(host[k].is_ascii_digit() || host[k] == b'.') {
            return false;
        }
        k += 1;
    }
    true
}

const _: () = {
    if let Some(url) = BAKED_SERVER_URL {
        assert!(
            baked_url_is_acceptable(url),
            "MW_SHELL_SERVER_URL must be https://host[:port][/path] (http only to localhost, 127.x or [::1])"
        );
    }
};

/// The largest provisioning file read (it holds one URL).
const MAX_FILE_BYTES: u64 = 16 * 1024;

/// Where the server address came from (logged at start).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Source {
    ProvisioningFile,
    Build,
    None,
}

/// The resolved server address and where it came from.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Resolved {
    pub url: Option<String>,
    pub source: Source,
    /// Why a provisioning file that exists was not used.
    pub problem: Option<String>,
}

/// Check and canonicalise a server address: `https://host[:port][/path]`, or
/// `http://` to a loopback host (`localhost`, `127.0.0.0/8`, `::1`) for testing.
/// No user name or password, query or fragment. A trailing slash is dropped.
pub fn validate_server_url(raw: &str) -> Result<String, String> {
    let url = tauri::Url::parse(raw.trim()).map_err(|e| format!("not a URL ({e})"))?;
    let host = url.host_str().unwrap_or("");
    if host.is_empty() {
        return Err("the address has no host".into());
    }
    match url.scheme() {
        "https" => {}
        "http" if is_loopback(&url) => {}
        "http" => return Err("the address must use https (http only to a loopback host)".into()),
        other => return Err(format!("the address must use https, not {other}")),
    }
    if !url.username().is_empty() || url.password().is_some() {
        return Err("the address must not carry a user name or password".into());
    }
    if url.query().is_some() || url.fragment().is_some() {
        return Err("the address must not carry a query or fragment".into());
    }
    let path = url.path().trim_end_matches('/');
    Ok(format!("{}{path}", url.origin().ascii_serialization()))
}

fn is_loopback(url: &tauri::Url) -> bool {
    let host = url.host_str().unwrap_or("");
    host.eq_ignore_ascii_case("localhost")
        || host
            .trim_start_matches('[')
            .trim_end_matches(']')
            .parse::<std::net::IpAddr>()
            .is_ok_and(|ip| ip.is_loopback())
}

/// The fixed provisioning file path for this OS.
pub fn provisioning_path() -> PathBuf {
    #[cfg(windows)]
    {
        let base = std::env::var_os("ProgramData")
            .map(PathBuf::from)
            .unwrap_or_else(|| PathBuf::from(r"C:\ProgramData"));
        base.join("Mailwoman").join("server.json")
    }
    #[cfg(target_os = "macos")]
    {
        PathBuf::from("/Library/Application Support/Mailwoman/server.json")
    }
    #[cfg(not(any(windows, target_os = "macos")))]
    {
        PathBuf::from("/etc/mailwoman/server.json")
    }
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct ProvisioningFile {
    server_url: String,
}

/// Read the provisioning file at `path`: `Ok(None)` when there is none, the
/// checked address when it is usable, `Err(reason)` when it exists but is not.
/// `check_owner` is the ownership check (`admin_owned` in the app).
pub fn read_provisioning_file(
    path: &Path,
    check_owner: impl Fn(&Path) -> Result<(), String>,
) -> Result<Option<String>, String> {
    let meta = match std::fs::metadata(path) {
        Ok(m) => m,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(None),
        Err(e) => return Err(format!("{}: {e}", path.display())),
    };
    if !meta.is_file() {
        return Err(format!("{} is not a file", path.display()));
    }
    if meta.len() > MAX_FILE_BYTES {
        return Err(format!(
            "{} is larger than {MAX_FILE_BYTES} bytes",
            path.display()
        ));
    }
    check_owner(path)?;
    if let Some(dir) = path.parent() {
        check_owner(dir)?;
    }
    let bytes = std::fs::read(path).map_err(|e| format!("{}: {e}", path.display()))?;
    let file: ProvisioningFile = serde_json::from_slice(&bytes).map_err(|e| {
        format!(
            "{}: not {{\"serverUrl\": \"https://…\"}} ({e})",
            path.display()
        )
    })?;
    validate_server_url(&file.server_url)
        .map(Some)
        .map_err(|e| format!("{}: {e}", path.display()))
}

/// Apply the precedence: provisioning file > baked value > none.
pub fn resolve(provisioned: Result<Option<String>, String>, baked: Option<&str>) -> Resolved {
    match provisioned {
        Ok(Some(url)) => Resolved {
            url: Some(url),
            source: Source::ProvisioningFile,
            problem: None,
        },
        Err(problem) => Resolved {
            url: None,
            source: Source::None,
            problem: Some(problem),
        },
        Ok(None) => match baked.map(validate_server_url) {
            Some(Ok(url)) => Resolved {
                url: Some(url),
                source: Source::Build,
                problem: None,
            },
            // Unreachable for a build that compiled (lib.rs checks the value);
            // kept so a bad value can never reach the SPA.
            Some(Err(problem)) => Resolved {
                url: None,
                source: Source::None,
                problem: Some(format!("MW_SHELL_SERVER_URL: {problem}")),
            },
            None => Resolved {
                url: None,
                source: Source::None,
                problem: None,
            },
        },
    }
}

/// The server address for this run, read once at start.
pub fn server_url() -> Resolved {
    let path = provisioning_path();
    let resolved = resolve(read_provisioning_file(&path, admin_owned), BAKED_SERVER_URL);
    match (&resolved.source, &resolved.problem) {
        (_, Some(problem)) => eprintln!("[mailwoman] no server address: {problem}"),
        (Source::ProvisioningFile, None) => {
            eprintln!("[mailwoman] server address from {}", path.display())
        }
        (Source::Build, None) => eprintln!("[mailwoman] server address from the build"),
        (Source::None, None) => eprintln!("[mailwoman] no server address configured"),
    }
    resolved
}

/// Unix: owned by root and not writable by group or others.
#[cfg(unix)]
pub fn admin_owned(path: &Path) -> Result<(), String> {
    use std::os::unix::fs::MetadataExt;
    let meta = std::fs::metadata(path).map_err(|e| format!("{}: {e}", path.display()))?;
    if meta.uid() != 0 {
        return Err(format!("{} is not owned by root", path.display()));
    }
    if meta.mode() & 0o022 != 0 {
        return Err(format!("{} is writable by group or others", path.display()));
    }
    Ok(())
}

/// Windows: owned by BUILTIN\Administrators or SYSTEM. Raw advapi32 calls, as
/// `lib.rs` does for `MessageBoxW`, so no crate feature is needed.
#[cfg(windows)]
pub fn admin_owned(path: &Path) -> Result<(), String> {
    use std::os::windows::ffi::OsStrExt;
    #[link(name = "advapi32")]
    unsafe extern "system" {
        fn GetNamedSecurityInfoW(
            name: *const u16,
            object_type: u32,
            info: u32,
            owner: *mut *mut core::ffi::c_void,
            group: *mut *mut core::ffi::c_void,
            dacl: *mut *mut core::ffi::c_void,
            sacl: *mut *mut core::ffi::c_void,
            descriptor: *mut *mut core::ffi::c_void,
        ) -> u32;
        fn IsWellKnownSid(sid: *mut core::ffi::c_void, kind: i32) -> i32;
    }
    #[link(name = "kernel32")]
    unsafe extern "system" {
        fn LocalFree(mem: *mut core::ffi::c_void) -> *mut core::ffi::c_void;
    }
    const SE_FILE_OBJECT: u32 = 1;
    const OWNER_SECURITY_INFORMATION: u32 = 1;
    const WIN_LOCAL_SYSTEM_SID: i32 = 22;
    const WIN_BUILTIN_ADMINISTRATORS_SID: i32 = 26;

    let wide: Vec<u16> = path
        .as_os_str()
        .encode_wide()
        .chain(std::iter::once(0))
        .collect();
    let mut owner = std::ptr::null_mut();
    let mut descriptor = std::ptr::null_mut();
    // SAFETY: `wide` is NUL-terminated and outlives the call; the out-pointers
    // are valid; `owner` points into `descriptor`, which is freed after use.
    let ok = unsafe {
        let rc = GetNamedSecurityInfoW(
            wide.as_ptr(),
            SE_FILE_OBJECT,
            OWNER_SECURITY_INFORMATION,
            &mut owner,
            std::ptr::null_mut(),
            std::ptr::null_mut(),
            std::ptr::null_mut(),
            &mut descriptor,
        );
        if rc != 0 {
            return Err(format!(
                "{}: the owner could not be read (error {rc})",
                path.display()
            ));
        }
        let ok = IsWellKnownSid(owner, WIN_BUILTIN_ADMINISTRATORS_SID) != 0
            || IsWellKnownSid(owner, WIN_LOCAL_SYSTEM_SID) != 0;
        LocalFree(descriptor);
        ok
    };
    if ok {
        Ok(())
    } else {
        Err(format!(
            "{} is not owned by Administrators or SYSTEM",
            path.display()
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn temp_dir(tag: &str) -> PathBuf {
        let dir =
            std::env::temp_dir().join(format!("mw-desktop-provision-{tag}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        dir
    }

    fn trusted(_: &Path) -> Result<(), String> {
        Ok(())
    }

    #[test]
    fn the_compile_time_check_matches_the_runtime_one() {
        for ok in [
            "https://mail.example.org",
            "https://mail.example.org:8443/mail/",
            "http://localhost:8080",
            "http://127.0.0.1:37920",
            "http://[::1]:1/",
        ] {
            assert!(baked_url_is_acceptable(ok), "{ok}");
            assert!(validate_server_url(ok).is_ok(), "{ok}");
        }
        for bad in [
            "",
            "mail.example.org",
            "https://",
            "http://mail.example.org",
            "http://localhost.example.org",
            "http://127.evil.example",
            "http://10.0.0.1",
            "https://u:p@mail.example.org",
            "https://mail.example.org/?x",
            "https://mail.example.org/#x",
            "https://mail example.org",
            "ftp://mail.example.org",
        ] {
            assert!(!baked_url_is_acceptable(bad), "{bad}");
            assert!(validate_server_url(bad).is_err(), "{bad}");
        }
        // The value this build was compiled with (if any) passes both.
        if let Some(url) = BAKED_SERVER_URL {
            assert!(validate_server_url(url).is_ok());
        }
    }

    #[test]
    fn validates_and_canonicalises() {
        assert_eq!(
            validate_server_url("https://mail.example.org/").unwrap(),
            "https://mail.example.org"
        );
        assert_eq!(
            validate_server_url(" https://mail.example.org:8443/mail/ ").unwrap(),
            "https://mail.example.org:8443/mail"
        );
        assert_eq!(
            validate_server_url("http://127.0.0.1:37920").unwrap(),
            "http://127.0.0.1:37920"
        );
        assert_eq!(
            validate_server_url("http://localhost:8080/").unwrap(),
            "http://localhost:8080"
        );
        assert_eq!(
            validate_server_url("http://[::1]:1").unwrap(),
            "http://[::1]:1"
        );
        for bad in [
            "",
            "mail.example.org",
            "http://mail.example.org",
            "http://10.0.0.1",
            "ftp://mail.example.org",
            "javascript:alert(1)",
            "https://u:p@mail.example.org",
            "https://mail.example.org/?x=1",
            "https://mail.example.org/#x",
            "file:///etc/passwd",
        ] {
            assert!(validate_server_url(bad).is_err(), "accepted {bad:?}");
        }
    }

    #[test]
    fn provisioning_file_wins_over_the_build() {
        let dir = temp_dir("wins");
        let path = dir.join("server.json");
        std::fs::write(&path, br#"{ "serverUrl": "https://fleet.example.org/" }"#).unwrap();
        let r = resolve(
            read_provisioning_file(&path, trusted),
            Some("https://baked.example.org"),
        );
        assert_eq!(r.url.as_deref(), Some("https://fleet.example.org"));
        assert_eq!(r.source, Source::ProvisioningFile);
        std::fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn the_build_is_used_without_a_file_and_none_without_either() {
        let dir = temp_dir("absent");
        let path = dir.join("server.json");
        let r = resolve(
            read_provisioning_file(&path, trusted),
            Some("https://baked.example.org/"),
        );
        assert_eq!(r.url.as_deref(), Some("https://baked.example.org"));
        assert_eq!(r.source, Source::Build);

        let r = resolve(read_provisioning_file(&path, trusted), None);
        assert_eq!(
            r,
            Resolved {
                url: None,
                source: Source::None,
                problem: None
            }
        );
        std::fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn an_unusable_file_gives_no_server_not_the_build() {
        let dir = temp_dir("bad");
        let path = dir.join("server.json");
        for body in [
            &br#"{ "serverUrl": "http://evil.example.org" }"#[..],
            br#"{ "serverUrl": "https://ok.example.org", "extra": 1 }"#,
            br#"not json"#,
            br#"{}"#,
        ] {
            std::fs::write(&path, body).unwrap();
            let r = resolve(
                read_provisioning_file(&path, trusted),
                Some("https://baked.example.org"),
            );
            assert_eq!(r.url, None, "body {:?}", String::from_utf8_lossy(body));
            assert!(r.problem.is_some());
        }
        // Too large.
        std::fs::write(&path, vec![b' '; (MAX_FILE_BYTES + 1) as usize]).unwrap();
        assert!(read_provisioning_file(&path, trusted).is_err());
        std::fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn a_file_not_owned_by_an_administrator_is_refused() {
        let dir = temp_dir("owner");
        let path = dir.join("server.json");
        std::fs::write(&path, br#"{ "serverUrl": "https://fleet.example.org" }"#).unwrap();
        let r = resolve(
            read_provisioning_file(&path, |p| {
                if p == path {
                    Err("not owned by an administrator".into())
                } else {
                    Ok(())
                }
            }),
            Some("https://baked.example.org"),
        );
        assert_eq!(r.url, None);
        // The folder is checked too.
        let r = read_provisioning_file(&path, |p| {
            if p == dir {
                Err("folder not owned by an administrator".into())
            } else {
                Ok(())
            }
        });
        assert!(r.is_err());
        std::fs::remove_dir_all(&dir).unwrap();
    }

    /// The real ownership check against what the OS reports.
    #[test]
    fn admin_owned_reads_the_real_owner() {
        #[cfg(windows)]
        {
            // %ProgramData% itself is owned by SYSTEM.
            let program_data = provisioning_path()
                .parent()
                .unwrap()
                .parent()
                .unwrap()
                .to_path_buf();
            assert_eq!(admin_owned(&program_data), Ok(()));
        }
        #[cfg(unix)]
        {
            use std::os::unix::fs::MetadataExt;
            assert_eq!(admin_owned(Path::new("/")), Ok(()));
            let dir = temp_dir("unix-owner");
            let path = dir.join("server.json");
            std::fs::write(&path, b"{}").unwrap();
            if std::fs::metadata(&path).unwrap().uid() != 0 {
                assert!(admin_owned(&path).is_err());
            }
            std::fs::remove_dir_all(&dir).unwrap();
        }
    }

    #[test]
    fn the_fixed_path_ends_in_server_json() {
        let p = provisioning_path();
        assert!(p.ends_with("server.json"));
        assert!(
            p.parent().unwrap().ends_with("Mailwoman")
                || p.parent().unwrap().ends_with("mailwoman")
        );
    }
}
