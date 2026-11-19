//! Frozen pre-optimization oracle, compiled only for tests/benchmarks.
//! High-confidence foreign application/configuration probes, not generic 404s.
//! This matcher intentionally knows nothing about source identity, policy or the
//! actual admin route. The caller must exclude its configured admin subtree.

const MAX_PATH_BYTES: usize = 8 * 1024;
const DECODE_PASSES: usize = 3;

/// A bounded canonical display path for matching and the caller's admin-route
/// exclusion. Only origin-form paths are accepted; query/fragment text is never
/// classified, even when its delimiter was percent-encoded.
pub fn normalized_path(input: &str) -> Option<String> {
    let raw_path = path_only(input.as_bytes());
    if raw_path.len() > MAX_PATH_BYTES {
        return None;
    }
    let mut bytes = raw_path.to_vec();
    for _ in 0..DECODE_PASSES {
        if !bytes.contains(&b'%') {
            break;
        }
        let mut decoded = Vec::with_capacity(bytes.len());
        let mut index = 0;
        while index < bytes.len() {
            if bytes[index] == b'%' {
                let high = hex(*bytes.get(index + 1)?)?;
                let low = hex(*bytes.get(index + 2)?)?;
                decoded.push(high * 16 + low);
                index += 3;
            } else {
                decoded.push(bytes[index]);
                index += 1;
            }
        }
        bytes = path_only(&decoded).to_vec();
    }
    // Do not guess what another decoder would do with deeper encodings, bad
    // UTF-8 or control bytes. Matching work is bounded independently of input.
    if bytes.contains(&b'%') || bytes.iter().any(u8::is_ascii_control) {
        return None;
    }
    let path = String::from_utf8(bytes).ok()?.replace('\\', "/");
    if !path.starts_with('/') {
        return None;
    }
    let mut segments = Vec::new();
    for segment in path.split('/') {
        match segment {
            "" | "." => {}
            ".." => {
                segments.pop();
            }
            _ => segments.push(segment.to_ascii_lowercase()),
        }
    }
    Some(format!("/{}", segments.join("/")))
}

fn path_only(bytes: &[u8]) -> &[u8] {
    &bytes[..bytes
        .iter()
        .position(|byte| matches!(byte, b'?' | b'#'))
        .unwrap_or(bytes.len())]
}

fn hex(byte: u8) -> Option<u8> {
    match byte {
        b'0'..=b'9' => Some(byte - b'0'),
        b'a'..=b'f' => Some(byte - b'a' + 10),
        b'A'..=b'F' => Some(byte - b'A' + 10),
        _ => None,
    }
}

/// Foreign application endpoints and sensitive environment files only. Whole
/// segments prevent `wp-administration`, `cmd_sco.js` or `.environment` matches.
pub fn hostile_probe_path(path: &str) -> bool {
    let Some(path) = normalized_path(path) else {
        return false;
    };
    let segments: Vec<_> = path.split('/').filter(|part| !part.is_empty()).collect();
    // Explicit operator-requested trigger, including nested/encoded variants.
    if segments.contains(&"cmd_sco") {
        return true;
    }
    // Resource identifiers and static filenames are not requested filesystem
    // paths. A .env mailbox/download name or /api/.../wp-admin is not a probe.
    if segments
        .first()
        .is_some_and(|part| matches!(*part, "api" | "jmap" | "dav" | "caldav" | "carddav"))
    {
        return false;
    }
    // Shared curated catalog: whole segments, with transport resource namespaces
    // excluded above. Lazy parsing keeps matching bounded and allocation-free
    // apart from the normalized request path and its segment vector.
    static TARGETS: std::sync::LazyLock<Vec<Vec<String>>> = std::sync::LazyLock::new(|| {
        serde_json::from_str::<Vec<String>>(crate::DEFAULT_SCAN_PATHS_JSON)
            .expect("validated scanner path catalog")
            .iter()
            .map(|target| {
                target
                    .split('/')
                    .filter(|part| !part.is_empty())
                    .map(str::to_owned)
                    .collect()
            })
            .collect()
    });
    if TARGETS.iter().any(|needle| {
        needle.len() <= segments.len()
            && segments
                .windows(needle.len())
                .enumerate()
                .any(|(index, window)| {
                    // A bare foreign console is a probe; a domain-discovery API
                    // path is not classified merely because it contains console.
                    (needle != &["console"] || index + needle.len() == segments.len())
                        && window.iter().zip(needle).all(|(a, b)| *a == b.as_str())
                })
    }) {
        return true;
    }
    let Some(last) = segments.last().copied() else {
        return false;
    };
    if segments
        .iter()
        .any(|segment| matches!(*segment, ".git" | ".ssh" | ".aws"))
        || environment_file(last)
        || config_file(last, ".gitconfig")
        || config_file(last, ".htaccess")
        || config_file(last, ".gitignore")
        || matches!(
            last,
            "wp-login.php"
                | "xmlrpc.php"
                | "wp-cron.php"
                | "wp-signup.php"
                | "wp-trackback.php"
                | "wlwmanifest.xml"
                | "admin.php"
                | "config.php"
                | "install.php"
                | "setup.php"
                | "shell.php"
        )
        || config_file(last, "wp-config.php")
    {
        return true;
    }
    // Joomla extension directories use well-known component/module prefixes,
    // including third-party extensions whose names cannot be enumerated.
    if segments.windows(2).any(|pair| {
        matches!((pair[0], pair[1].split_once('_')),
            ("components", Some(("com", suffix))) | ("modules", Some(("mod", suffix)))
                if !suffix.is_empty())
    }) {
        return true;
    }
    for (index, segment) in segments.iter().enumerate() {
        let tail = &segments[index + 1..];
        match *segment {
            "wp-admin" | "wp-content" | "wp-includes" | "wordpress" | "wp-json" | "typo3"
            | "umbraco" | "myadmin" | "mysql" | "joomla" | "drupal" | "magento" => return true,
            "bitrix" if tail.first() == Some(&"admin") => return true,
            "administrator" | "phpmyadmin" | "phpmyadmin4" | "phpmyadmin5" | "pma"
            | "downloader"
                if tail.is_empty() || tail == ["index.php"] =>
            {
                return true;
            }
            "core" if tail == ["install.php"] => return true,
            "sites"
                if tail.len() == 2
                    && tail[0] == "default"
                    && config_file(tail[1], "settings.php") =>
            {
                return true;
            }
            _ => {}
        }
    }
    false
}

fn config_file(value: &str, base: &str) -> bool {
    value == base
        || value.strip_prefix(base).is_some_and(|suffix| {
            matches!(
                suffix,
                ".bak" | ".backup" | ".old" | ".save" | ".orig" | "~"
            )
        })
}

fn environment_file(value: &str) -> bool {
    if value == ".env" || value == ".env~" {
        return true;
    }
    let Some(suffix) = value.strip_prefix(".env.") else {
        return false;
    };
    let mut count = 0;
    for part in suffix.split('.') {
        count += 1;
        if count > 3
            || !matches!(
                part,
                "local"
                    | "prod"
                    | "production"
                    | "dev"
                    | "development"
                    | "test"
                    | "testing"
                    | "stage"
                    | "staging"
                    | "backup"
                    | "bak"
                    | "old"
                    | "save"
                    | "orig"
                    | "dist"
                    | "example"
                    | "sample"
                    | "default"
                    | "ci"
                    | "docker"
                    | "remote"
            )
        {
            return false;
        }
    }
    count > 0
}
