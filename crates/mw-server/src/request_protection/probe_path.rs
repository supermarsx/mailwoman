//! High-confidence foreign application/configuration probes, not generic 404s.
//! This matcher intentionally knows nothing about source identity, policy or the
//! actual admin route. The caller must exclude its configured admin subtree.

const MAX_PATH_BYTES: usize = 8 * 1024;
const DECODE_PASSES: usize = 3;

/// A bounded canonical display path for matching and the caller's admin-route
/// exclusion. Only origin-form paths are accepted; query/fragment text is never
/// classified, even when its delimiter was percent-encoded.
pub(super) fn normalized_path(input: &str) -> Option<String> {
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
pub(crate) fn hostile_probe_path(path: &str) -> bool {
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
            )
        {
            return false;
        }
    }
    count > 0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sensitive_env_files_use_whole_known_variants_at_any_depth() {
        for path in [
            "/.env",
            "/.ENV/",
            "/app/.env.local",
            "/backup/.env.production.local.bak",
            "/.env.development",
            "/.env.test",
            "/.env.staging",
            "/.env.example",
            "/.env~",
        ] {
            assert!(hostile_probe_path(path), "{path}");
        }
        for path in [
            "/.environment",
            "/.envoy",
            "/file.env",
            "/.env.js",
            "/.env.local.css",
            "/.env.unknown",
            "/.env/guide.html",
            "/docs/.env.example.txt",
            "/env",
        ] {
            assert!(!hostile_probe_path(path), "{path}");
        }
    }

    #[test]
    fn foreign_cms_endpoints_are_specific_and_may_be_nested() {
        for path in [
            "/wp-admin",
            "/blog/WP-ADMIN/",
            "/wordpress/wp-admin/load-scripts.php",
            "/wp-admin/css/login.css",
            "/blog/wp-login.php",
            "/xmlrpc.php",
            "/wp-config.php.old",
            "/blog/wp-config.php",
            "/administrator",
            "/joomla/administrator/index.php",
            "/drupal/core/install.php",
            "/sites/default/settings.php",
            "/sites/default/settings.php.bak",
            "/phpMyAdmin",
            "/tools/phpmyadmin5/index.php",
            "/phpmyadmin4/",
            "/pma/index.php",
            "/magento/downloader",
            "/downloader/index.php",
            "/typo3",
            "/site/TYPO3/backend.php",
            "/umbraco",
            "/umbraco/backoffice/login",
            "/bitrix/admin",
            "/portal/bitrix/admin/index.php",
        ] {
            assert!(hostile_probe_path(path), "{path}");
        }
    }

    #[test]
    fn git_metadata_is_matched_as_whole_segments_and_not_published_git_assets() {
        for path in [
            "/.git",
            "/repo/.git/config",
            "/.GIT/HEAD",
            "/.git/index",
            "/.git/objects/info/packs",
            "/%2egit/config",
            "/%252egit/config",
            "/safe/../.git/config",
            "/.gitconfig",
            "/.gitignore",
            "/backup/.gitconfig.old",
            "/assets/.env",
            "/static/.git/config",
            "/assets/wp-admin/install.php",
        ] {
            assert!(hostile_probe_path(path), "{path}");
        }
        for path in [
            "/.gitattributes",
            "/.github/workflows/test.yml",
            "/docs/git/config",
            "/.gitconfig.js",
            "/foo.git/config",
            "/.git/../readme.txt",
            "/api/files/.git/config",
            "/jmap/files/.gitconfig",
            "/dav/home/.git/config",
            "/docs?file=/.git/config",
            "/docs%3ffile=/.git/config",
        ] {
            assert!(!hostile_probe_path(path), "{path}");
        }
    }

    #[test]
    fn generic_routes_docs_and_static_resources_are_not_cms_probes() {
        for path in [
            "/",
            "/admin",
            "/login",
            "/user/login",
            "/administrator/help.html",
            "/wp-administration",
            "/my-wp-admin",
            "/wp-admin.js",
            "/wp-login.php.txt",
            "/xmlrpc.php.css",
            "/downloader/guide.pdf",
            "/core/install-guide.php",
            "/sites/other/settings.php",
            "/phpmyadmin5000",
            "/phpmyadmin-guide",
            "/typo3-guide",
            "/typo30",
            "/umbraco-assets",
            "/myumbraco",
            "/bitrix/administer",
            "/bitrix/admin.js",
            "/bitrix/public/index.php",
            "/api/files/.env",
            "/jmap/wp-admin",
            "/dav/collection/wp-login.php",
            "/assets/logo.svg",
            "/static/app.js",
            "/downloads/.env.production.txt",
            "/api/messages/../files/.env",
            "/notes?target=/.env",
            "/?url=/wp-admin",
            "/safe%3fnext=/.env",
            "/help#wp-admin",
        ] {
            assert!(!hostile_probe_path(path), "{path}");
        }
    }

    #[test]
    fn operator_requested_wordpress_cms_and_sensitive_paths_match_without_substring_guessing() {
        for path in [
            "/wp-content",
            "/blog/wp-content/uploads/payload.php",
            "/wp-includes/js/app.js",
            "/wordpress",
            "/wordpress/index.php",
            "/wp-json/wp/v2/users",
            "/wp-cron.php",
            "/wp-signup.php",
            "/wp-trackback.php",
            "/blog/wlwmanifest.xml",
            "/myadmin",
            "/myadmin/index.php",
            "/mysql",
            "/mysql/admin/login.php",
            "/admin.php",
            "/site/joomla/index.php",
            "/drupal/user/login",
            "/magento/app/etc/env.php",
            "/config.php",
            "/nested/install.php",
            "/setup.php",
            "/shell.php",
            "/.htaccess",
            "/backup/.htaccess.bak",
            "/.gitignore",
            "/repo/.gitignore.old",
            "/.ssh",
            "/home/.ssh/id_rsa",
            "/.aws",
            "/.aws/credentials",
            "/%77p-content/payload.php",
            "/%252eaws/credentials",
            "/nested/.SSH/authorized_keys",
            "/safe/../WP-JSON/users",
            "/assets/wp-includes/style.css",
        ] {
            assert!(hostile_probe_path(path), "{path}");
        }
        for path in [
            "/wp-contentful",
            "/wp-includes.js",
            "/wordpress-guide",
            "/wp-json.js",
            "/wp-cron.php.txt",
            "/wp-signup.php.css",
            "/wp-trackback.php.js",
            "/wlwmanifest.xml.svg",
            "/myadministrator",
            "/mysql-client.js",
            "/admin.php.txt",
            "/joomla-guide",
            "/drupal-guide",
            "/magento-assets",
            "/config.php.js",
            "/install.php.txt",
            "/setup.php.map",
            "/shell.php.png",
            "/.htaccess.txt",
            "/.gitignore.js",
            "/.ssh-agent",
            "/.awsome",
            "/.aws.json",
            "/admin",
            "/login",
            "/user/login",
            "/api/files/.gitignore",
            "/api/files/wp-content",
            "/jmap/.aws/credentials",
            "/dav/home/.ssh/id_rsa",
            "/api/files/config.php",
            "/notes?file=/.htaccess",
            "/safe%3fnext=/wp-content",
            "/.ssh/../readme.txt",
        ] {
            assert!(!hostile_probe_path(path), "{path}");
        }
    }

    #[test]
    fn explicit_cmd_sco_is_a_whole_segment_and_overrides_resource_exclusions() {
        for path in [
            "/cmd_sco",
            "/CMD_SCO/",
            "/nested/cmd_sco/payload",
            "/%63md_sco",
            "/api/cmd_sco/run",
            "/static/cmd_sco/run",
            "\\cmd_sco\\run",
        ] {
            assert!(hostile_probe_path(path), "{path}");
        }
        for path in [
            "/cmd_sco.js",
            "/cmd_sco-other",
            "/xcmd_sco",
            "/safe?cmd_sco",
            "/cmd_sco/../safe",
        ] {
            assert!(!hostile_probe_path(path), "{path}");
        }
    }

    #[test]
    fn bounded_decoding_case_separators_and_dot_segments_are_consistent() {
        for path in [
            "/%2eenv",
            "/%252eenv",
            "/%25252eenv",
            "/blog%2fWP-ADMIN%2finstall.php",
            "/foo%5c..%5c.env",
            "/safe/%2e%2e/.env",
            "/./blog//../.env",
            "\\BLOG\\wp-admin\\index.php",
            "/../../wp-login.php",
            "/.env?download=true",
        ] {
            assert!(hostile_probe_path(path), "{path}");
        }
        for path in [
            "/%2525252eenv",
            "/.env%00",
            "/.env%0a",
            "/%ff/.env",
            "/bad%zz/.env",
            "/bad%/.env",
            "/.env/../safe",
            "/wp-admin/../../api/.env",
            "https://example.org/.env",
            "not/a/path/.env",
        ] {
            assert!(!hostile_probe_path(path), "{path}");
        }
        assert_eq!(
            normalized_path("/A//b/%2e%2e/C?x=/.env").as_deref(),
            Some("/a/c")
        );
    }

    #[test]
    fn excessive_inputs_and_decoded_query_content_do_not_consume_unbounded_work() {
        assert!(!hostile_probe_path(&format!(
            "/{}/.env",
            "x".repeat(MAX_PATH_BYTES)
        )));
        assert!(!hostile_probe_path(&format!(
            "/safe?{}",
            "/.env".repeat(MAX_PATH_BYTES)
        )));
        assert!(hostile_probe_path(&format!(
            "/.env?{}",
            "x".repeat(MAX_PATH_BYTES)
        )));
        assert!(!hostile_probe_path("/safe%253fnext=%252fwp-admin"));
        assert!(!hostile_probe_path("/safe%2523/.env"));
    }
}
