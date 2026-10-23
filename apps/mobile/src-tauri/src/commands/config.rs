//! The Mailwoman server the mobile shell talks to (t32 §2.7).
//!
//! Baked in at build time from `MW_SHELL_SERVER_URL` (`scripts/build-shells.*`);
//! the user never enters or changes it. Without it the SPA shows that the app has
//! not been set up. Android managed configuration (MDM app restrictions) is a
//! follow-up; there is no provisioning file on mobile. The checks mirror the
//! desktop shell's `serverconfig.rs` (separate crates, no shared one).

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

/// The server address for this run: the baked value, checked again.
pub fn server_url() -> Option<String> {
    match BAKED_SERVER_URL.map(validate_server_url) {
        Some(Ok(url)) => Some(url),
        Some(Err(problem)) => {
            eprintln!("[mailwoman] no server address: MW_SHELL_SERVER_URL: {problem}");
            None
        }
        None => {
            eprintln!("[mailwoman] no server address configured");
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
}
