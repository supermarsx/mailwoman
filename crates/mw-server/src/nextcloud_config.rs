//! Validation shared by administrator configuration and credential-bearing calls.
use reqwest::Url;

pub(crate) fn base_url(value: &str) -> Result<Url, &'static str> {
    if value.len() > 2048 || value.contains('\\') || value.contains('%') {
        return Err("invalid Nextcloud URL");
    }
    let mut url = Url::parse(value).map_err(|_| "invalid Nextcloud URL")?;
    if url.scheme() != "https"
        || url.host_str().is_none()
        || !url.username().is_empty()
        || url.password().is_some()
        || url.query().is_some()
        || url.fragment().is_some()
    {
        return Err("use an HTTPS instance URL without credentials, query or fragment");
    }
    // Encoded separators/dot segments are never an installation prefix.
    if url.path().contains('%') {
        return Err("encoded installation prefixes are not supported");
    }
    if !url.path().ends_with('/') {
        url.set_path(&format!("{}/", url.path()));
    }
    Ok(url)
}

pub(crate) fn username(value: &str) -> Result<(), &'static str> {
    if value.is_empty()
        || value.len() > 256
        || value.trim() != value
        || value == "."
        || value == ".."
        || value
            .chars()
            .any(|ch| ch.is_control() || matches!(ch, '/' | '\\' | ':'))
    {
        return Err("invalid Nextcloud username");
    }
    Ok(())
}

pub(crate) fn file_url(base: &str, user: &str, path: &str) -> Result<Url, &'static str> {
    username(user)?;
    if path.len() > 2048 || path.contains('\\') || path.chars().any(char::is_control) {
        return Err("invalid Nextcloud path");
    }
    let parts: Vec<_> = path
        .trim_matches('/')
        .split('/')
        .filter(|part| !part.is_empty())
        .collect();
    if parts.iter().any(|part| matches!(*part, "." | "..")) {
        return Err("invalid Nextcloud path");
    }
    let mut url = base_url(base)?;
    {
        let mut segments = url
            .path_segments_mut()
            .map_err(|_| "invalid instance URL")?;
        segments
            .pop_if_empty()
            .extend(["remote.php", "dav", "files", user]);
        for part in parts {
            segments.push(part);
        }
        if path.ends_with('/') {
            segments.push("");
        }
    }
    Ok(url)
}

pub(crate) fn ca_roots(pem: &str) -> Result<Vec<Vec<u8>>, &'static str> {
    use rustls_pki_types::{CertificateDer, pem::PemObject};
    if pem.is_empty() {
        return Ok(Vec::new());
    }
    if pem.len() > 65536 {
        return Err("CA bundle exceeds 64 KiB");
    }
    let roots = CertificateDer::pem_slice_iter(pem.as_bytes())
        .map(|cert| cert.map(|cert| cert.as_ref().to_vec()))
        .collect::<Result<Vec<_>, _>>()
        .map_err(|_| "invalid CA PEM bundle")?;
    if roots.is_empty() || roots.len() > 8 {
        return Err("provide one to eight CA certificates");
    }
    let mut store = rustls::RootCertStore::empty();
    for root in &roots {
        store
            .add(CertificateDer::from(root.clone()))
            .map_err(|_| "invalid CA certificate")?;
    }
    Ok(roots)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn credentials_and_file_paths_cannot_change_authority_or_escape_account_root() {
        for url in [
            "http://cloud.test",
            "https://user:secret@cloud.test",
            "https://cloud.test/?redirect=evil",
            "https://cloud.test/#x",
            "https://cloud.test/%2e%2e/",
        ] {
            assert!(base_url(url).is_err(), "{url}");
        }
        for path in [
            "../bob/secret",
            "/a/../../bob",
            "a\\..\\bob",
            "a\r\nHost: evil",
        ] {
            assert!(file_url("https://cloud.test/install", "alice", path).is_err());
        }
        let url = file_url(
            "https://cloud.test/install",
            "alice@example.test",
            "/Résumé #1/%2fsecret",
        )
        .unwrap();
        assert_eq!(url.host_str(), Some("cloud.test"));
        assert!(
            url.path()
                .starts_with("/install/remote.php/dav/files/alice@example.test/")
        );
        assert!(url.path().contains("%252fsecret"));
        assert!(url.fragment().is_none());
    }
}
