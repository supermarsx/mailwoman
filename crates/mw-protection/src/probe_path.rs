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
        let path_len = path_only(&decoded).len();
        decoded.truncate(path_len);
        bytes = decoded;
    }
    // Do not guess what another decoder would do with deeper encodings, bad
    // UTF-8 or control bytes. Matching work is bounded independently of input.
    if bytes.contains(&b'%') || bytes.iter().any(u8::is_ascii_control) {
        return None;
    }
    let path = String::from_utf8(bytes).ok()?;
    let path = if path.contains('\\') {
        path.replace('\\', "/")
    } else {
        path
    };
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
            _ => segments.push(segment),
        }
    }
    let mut normalized = String::with_capacity(path.len().max(1));
    normalized.push('/');
    for (index, segment) in segments.into_iter().enumerate() {
        if index != 0 {
            normalized.push('/');
        }
        normalized.push_str(segment);
    }
    normalized.make_ascii_lowercase();
    Some(normalized)
}

fn path_only(bytes: &[u8]) -> &[u8] {
    &bytes[..memchr::memchr2(b'?', b'#', bytes).unwrap_or(bytes.len())]
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
    // One bounded automaton is built for the curated catalog. A match must
    // still end at a whole segment, and a bare console remains terminal-only.
    static TARGETS: std::sync::LazyLock<CatalogMatcher> = std::sync::LazyLock::new(|| {
        let paths: Vec<String> = serde_json::from_str(crate::DEFAULT_SCAN_PATHS_JSON)
            .expect("validated scanner path catalog");
        CatalogMatcher::new(paths.iter().map(String::as_str))
    });
    if TARGETS.matches(&path) {
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

struct CatalogMatcher {
    automaton: aho_corasick::AhoCorasick,
    terminal_only: Vec<bool>,
}
impl CatalogMatcher {
    fn new<'a>(paths: impl IntoIterator<Item = &'a str>) -> Self {
        let patterns: Vec<String> = paths
            .into_iter()
            .map(|target| {
                let segments: Vec<_> = target.split('/').filter(|part| !part.is_empty()).collect();
                assert!(
                    !segments.is_empty(),
                    "scanner catalog must not contain an empty target"
                );
                format!("/{}", segments.join("/"))
            })
            .collect();
        let terminal_only = patterns
            .iter()
            .map(|pattern| pattern == "/console")
            .collect();
        let automaton = aho_corasick::AhoCorasick::builder()
            .match_kind(aho_corasick::MatchKind::Standard)
            .build(patterns)
            .expect("bounded built-in scanner catalog");
        Self {
            automaton,
            terminal_only,
        }
    }
    fn matches(&self, path: &str) -> bool {
        self.automaton.find_overlapping_iter(path).any(|found| {
            let end = found.end();
            (end == path.len() || path.as_bytes().get(end) == Some(&b'/'))
                && (!self.terminal_only[found.pattern().as_usize()] || end == path.len())
        })
    }
}

#[cfg(test)]
#[path = "probe_path_legacy.rs"]
mod legacy;

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn consoles_exposure_and_fingerprinting_endpoints_use_whole_paths() {
        for path in [
            "/manager/html",
            "/tools/host-manager/html",
            "/JMX-CONSOLE/",
            "/jolokia/version",
            "/ACTUATOR/heapdump",
            "/app/actuator/env/profiles",
            "/actuator/gateway/routes",
            "/jenkins/scriptText",
            "/solr/admin/info/system",
            "/h2-console/login.jsp",
            "/_ignition/execute-solution",
            "/_profiler/phpinfo",
            "/debug/default/view",
            "/vendor/phpunit/phpunit/src/Util/PHP/eval-stdin.php",
            "/nested/phpinfo.php",
            "/.docker/config.json",
            "/.kube/config",
            "/.vscode/sftp.json",
            "/.svn/entries",
            "/WEB-INF/web.xml",
            "/app/etc/local.xml",
            "/proc/self/environ",
            "/etc/passwd",
            "/%61ctuator/%65nv",
            "/app/../jenkins/script",
            "/nested/r57.php",
        ] {
            assert!(hostile_probe_path(path), "{path}");
        }
        for path in [
            "/manager",
            "/actuator-guide/env",
            "/actuator/env.js",
            "/jenkins/scripture",
            "/solr/admin-guide",
            "/phpinfo.php.txt",
            "/cmd.php.png",
            "/proc/self/status",
            "/api/files/actuator/env",
            "/jmap/download/a/phpinfo.php",
            "/dav/files/.kube/config",
            "/api/openapi.json",
            "/openapi.json",
            "/swagger",
            "/health",
            "/metrics",
            "/.well-known/jmap",
            "/.well-known/oauth-authorization-server",
            "/autodiscover/autodiscover.xml",
            "/autodiscover/autodiscover.json",
            "/Microsoft-Server-ActiveSync/default.eas",
            "/ews/exchange.asmx",
            "/help?path=/actuator/env",
            "/safe%3fpath=/manager/html",
        ] {
            assert!(!hostile_probe_path(path), "{path}");
        }
    }

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
            "/administrator-guide/help.html",
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
            "/bitrix-assets/administer",
            "/bitrix-assets/admin.js",
            "/bitrix-assets/public/index.php",
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
    #[test]
    fn exposed_environment_variants_and_foreign_admin_routes_are_probes() {
        for path in [
            "/.git/config",
            "/.git/HEAD",
            "/.env",
            "/.env.backup",
            "/.env.bak",
            "/.env.ci",
            "/.env.dev",
            "/.env.development",
            "/.env.docker",
            "/.env.example",
            "/.env.local",
            "/.env.old",
            "/.env.prod",
            "/.env.production",
            "/.env.remote",
            "/.env.sample",
            "/.env.save",
            "/.env.stage",
            "/.env.staging",
            "/.env.test",
            "/admin/users",
            "/console/",
        ] {
            assert!(hostile_probe_path(path), "{path}");
        }
        for path in [
            "/pdfjs/bootstrap.mjs",
            "/pdfjs/mailwoman.css",
            "/pdfjs/web/viewer.css",
        ] {
            assert!(!hostile_probe_path(path), "{path}");
        }
    }

    #[test]
    fn foreign_asset_fingerprints_are_probes_but_normal_browser_requests_are_not() {
        for path in [
            "/js/lkk_ch.js",
            "/js/qrcode_twint.js",
            "/js/twint_ch.js",
            "/js/raif_ch.js",
            "/js/antibot-client.js",
            "/bot-connect.js",
            "/licensor.js",
            "/css/support_parent.css",
            "/static/style/protect/index.js",
            "/static/style/sys_files/index.js",
            "/assets/js/qr_modal.js",
            "/JS/%71rcode_twint.js",
        ] {
            assert!(hostile_probe_path(path), "{path}");
        }
        for path in [
            "/robots.txt",
            "/pdfjs/build/pdf.mjs.map",
            "/pdfjs/web/viewer.mjs.map",
            "/assets/js/auth.js",
            "/assets/js/message.js",
            "/api/files/bot-connect.js",
            "/jmap/download/a/js/qrcode_twint.js",
            "/console/domains/example.test/detect",
        ] {
            assert!(!hostile_probe_path(path), "{path}");
        }
    }

    #[test]
    fn curated_catalog_is_valid_shared_and_matches_encoded_nested_probes() {
        let paths: Vec<String> = serde_json::from_str(crate::DEFAULT_SCAN_PATHS_JSON).unwrap();
        let defaults: Vec<String> = serde_json::from_str(crate::DEFAULT_PROBE_PATHS_JSON).unwrap();
        assert_eq!(paths.len(), 292);
        assert!(defaults.len() <= 512);
        let unique: std::collections::BTreeSet<_> = paths.iter().collect();
        assert_eq!(unique.len(), paths.len());
        for path in paths {
            assert!(path.len() <= 128);
            assert_eq!(normalized_path(&path).as_deref(), Some(path.as_str()));
            assert!(defaults.contains(&path), "admin tracker missing {path}");
            assert!(hostile_probe_path(&path), "{path}");
            assert!(
                hostile_probe_path(&format!(
                    "/nested{}{}",
                    path.to_uppercase(),
                    if path == "/console" { "" } else { "/extra" }
                )),
                "nested {path}"
            );
            assert!(
                !hostile_probe_path(&format!("/api/files{path}")),
                "resource {path}"
            );
        }
        for path in [
            "/%63panel",
            "/roundcube/%69nstaller",
            "/host-manager/%74ext",
            "/%65cp",
        ] {
            assert!(hostile_probe_path(path), "{path}");
        }
    }
    #[test]
    fn cms_folders_cover_all_descendants_including_static_assets() {
        for path in [
            "/media/system/js/core.js",
            "/wp-includes/js/jquery/jquery.js",
            "/WP-INCLUDES/custom/arbitrary.css",
            "/wp-content/uploads/arbitrary.jpg",
            "/wp-admin/arbitrary/file.js",
            "/media/system/custom/file.png",
            "/blog/media/jui/js/jquery.min.js",
            "/media/vendor/joomla/js/custom.js",
            "/components/com_custom/new/file.css",
            "/modules/mod_custom/tmpl/default.php",
            "/administrator/components/com_users/view.html",
            "/bitrix/public/index.php",
            "/core/misc/drupal.js",
            "/sites/default/files/example.png",
            "/typo3temp/assets/css/anything.css",
            "/pub/static/custom/arbitrary.js",
            "/opencart/catalog/custom.css",
            "/media/%73ystem/js/core.js",
        ] {
            assert!(hostile_probe_path(path), "{path}");
        }
        for path in [
            "/media/profile.jpg",
            "/media/systematic/js/core.js",
            "/wp-includes-extra/file.js",
            "/components/common/file.js",
            "/modules/modern/file.js",
            "/templates/custom/app.css",
            "/api/files/media/system/js/core.js",
            "/jmap/download/a/wp-includes/js/jquery/jquery.js",
        ] {
            assert!(!hostile_probe_path(path), "{path}");
        }
    }
}

#[cfg(test)]
#[path = "policy_benchmark.rs"]
pub(crate) mod benchmark;

#[cfg(test)]
mod differential_tests {
    use super::*;
    #[test]
    fn optimized_paths_preserve_frozen_classifier_and_normalization_decisions() {
        let catalog: Vec<String> = serde_json::from_str(crate::DEFAULT_SCAN_PATHS_JSON).unwrap();
        let mut cases = vec![
            "/".to_owned(),
            "/api/download/.env".into(),
            "/api/../.git/config".into(),
            "/api/cmd_sco".into(),
            "/api/{token}/domains/example.test/detect".into(),
            "/console/{token}/domains/example.test/detect".into(),
            "/pdfjs/build/pdf.mjs.map".into(),
            "/pdfjs/web/viewer.mjs.map".into(),
            "/pdfjs/bootstrap.mjs".into(),
            "/assets/index-Ab12.js".into(),
            "/robots.txt".into(),
            "/á/../wp-admin".into(),
            "/./..//safe?name=wp-admin".into(),
            "/.git/config%3Fsafe".into(),
            "/%252eenv".into(),
            "/bad%GG".into(),
            "relative/path".into(),
            "/nul\0byte".into(),
        ];
        for target in catalog {
            for prefix in ["", "/nested", "/api", "/jmap", "/caldav", "/dav"] {
                for suffix in ["", "/child", "x", ".js", "?x=/wp-admin", "#/.env"] {
                    cases.push(format!("{prefix}{target}{suffix}"));
                }
            }
            cases.push(target.to_ascii_uppercase());
            cases.push(target.replace('/', "%2f"));
            cases.push(target.replace('/', "%252f"));
            cases.push(target.replace('/', "\\"));
        }
        cases.push(format!("/{}", "x".repeat(8191)));
        cases.push(format!("/{}", "x".repeat(8192)));
        cases.push(format!("/{}/.git/config", "x/".repeat(1800)));
        cases.push(format!("/{}", "benign/".repeat(1000)));
        let fragments = [
            "safe",
            ".",
            "..",
            ".env",
            "wp-admin",
            "wp-administration",
            "api",
            "jmap",
            "cmd_sco",
            "%2e",
            "%252e",
            "%25",
            "%GG",
            "console",
            "/",
            "\\",
            "É",
            "%2F",
            "%25253F",
            "config",
            ".git",
            "%00",
        ];
        let mut seed = 0x8f27_63a9u64;
        for _ in 0..2000 {
            let mut input = String::new();
            for _ in 0..12 {
                seed = seed.wrapping_mul(6364136223846793005).wrapping_add(1);
                input.push('/');
                input.push_str(fragments[(seed >> 32) as usize % fragments.len()]);
            }
            cases.push(input);
        }
        for input in cases {
            assert_eq!(
                normalized_path(&input),
                legacy::normalized_path(&input),
                "normalization: {input:?}"
            );
            assert_eq!(
                hostile_probe_path(&input),
                legacy::hostile_probe_path(&input),
                "classification: {input:?}"
            );
        }
    }
    #[test]
    fn compiled_catalog_keeps_overlaps_whole_segments_and_terminal_console() {
        let matcher = CatalogMatcher::new(["/a", "/a/b", "/console", "/b/c"]);
        for (path, wanted) in [
            ("/a/b", true),
            ("/x/b/c/next", true),
            ("/console", true),
            ("/x/console", true),
            ("/console/legit", false),
            ("/able", false),
            ("/x/b/case", false),
        ] {
            assert_eq!(matcher.matches(path), wanted, "{path}");
        }
    }
    fn legacy_catalog_match(paths: &[Vec<String>], path: &str) -> bool {
        let segments: Vec<_> = path.split('/').filter(|s| !s.is_empty()).collect();
        paths.iter().any(|needle| {
            needle.len() <= segments.len()
                && segments.windows(needle.len()).enumerate().any(|(i, w)| {
                    (needle != &["console"] || i + needle.len() == segments.len())
                        && w.iter().zip(needle).all(|(a, b)| *a == b.as_str())
                })
        })
    }
    #[test]
    #[ignore = "manual release benchmark; run without concurrent builds"]
    fn policy_match_benchmark_paths() {
        assert!(
            !cfg!(debug_assertions),
            "use cargo test --release for performance measurements"
        );
        let ordinary = [
            "/",
            "/assets/index-Ab12.js",
            "/pdfjs/web/viewer.css",
            "/mail/inbox",
            "/api/account/preferences",
            "/api/account/drafts",
            "/jmap/ws",
            "/robots.txt",
            "/console/{token}/domains/example.test/detect",
        ]
        .map(String::from);
        benchmark::compare(
            "paths/ordinary",
            &ordinary,
            legacy::hostile_probe_path,
            hostile_probe_path,
        );
        let hostile = [
            "/.git/config",
            "/wp-includes/js/jquery/jquery.js",
            "/app/actuator/heapdump",
            "/media/system/js/core.js",
            "/nested/.env.production",
            "/nested/console",
        ]
        .map(String::from);
        benchmark::compare(
            "paths/hostile",
            &hostile,
            legacy::hostile_probe_path,
            hostile_probe_path,
        );
        let adversarial = vec![
            format!("/{}", "word/".repeat(1400)),
            format!("/{}/wp-administration", "word/".repeat(1300)),
            format!("/{}", "x".repeat(8191)),
        ];
        benchmark::compare(
            "paths/long-negative",
            &adversarial,
            legacy::hostile_probe_path,
            hostile_probe_path,
        );
        let encoded = [
            "/%252eenv.production",
            "/nested/%2577p-admin",
            "/safe%3Fwp-admin",
            "/a/../%2e%67it/config",
        ]
        .map(String::from);
        benchmark::compare(
            "paths/encoded",
            &encoded,
            legacy::hostile_probe_path,
            hostile_probe_path,
        );
        for count in [1, 32, 256, 512] {
            let patterns: Vec<_> = (0..count)
                .map(|n| format!("/foreign-{n}/admin-{n}"))
                .collect();
            let old: Vec<Vec<String>> = patterns
                .iter()
                .map(|p| {
                    p.split('/')
                        .filter(|s| !s.is_empty())
                        .map(str::to_owned)
                        .collect()
                })
                .collect();
            let matcher = CatalogMatcher::new(patterns.iter().map(String::as_str));
            let inputs = vec![
                "/safe/application/path".into(),
                format!("/a/foreign-{}/admin-{}/child", count - 1, count - 1),
                "/safe".repeat(100),
            ];
            benchmark::compare(
                &format!("catalog/{count}-rules"),
                &inputs,
                |p| legacy_catalog_match(&old, p),
                |p| {
                    // Both production paths already construct these segments
                    // for namespace/explicit-token checks before catalog lookup.
                    let segments: Vec<_> = p.split('/').filter(|part| !part.is_empty()).collect();
                    std::hint::black_box(&segments);
                    matcher.matches(p)
                },
            );
        }
    }
}
