//! One sealing key for server startup and CLI commands. Never silently rotate a
//! damaged key file: doing so would strand sessions and other sealed records.
use anyhow::{Context, bail};
use mw_store::ServerKey;
use sqlx::sqlite::SqliteConnectOptions;
use std::{
    fs,
    io::Write,
    path::{Path, PathBuf},
    str::FromStr,
};
use zeroize::Zeroizing;

/// Explicit key wins. MW_SERVER_KEY_FILE supports managed/shared deployments;
/// otherwise SQLite gets a sidecar beside its actual (URL-decoded) database.
pub fn resolve(database: &str, explicit: Option<&str>) -> anyhow::Result<ServerKey> {
    let key_file = std::env::var_os("MW_SERVER_KEY_FILE").map(PathBuf::from);
    resolve_with_file(database, explicit, key_file.as_deref())
}

pub(crate) fn is_persistent(database: &str, explicit: Option<&str>) -> anyhow::Result<bool> {
    if explicit.is_some()
        || std::env::var_os("MW_SERVER_KEY_FILE").is_some()
        || database.starts_with("postgres://")
        || database.starts_with("postgresql://")
    {
        return Ok(true);
    }
    let url = if database.starts_with("sqlite:") {
        database.to_owned()
    } else {
        format!("sqlite://{database}?mode=rwc")
    };
    SqliteConnectOptions::from_str(&url)?;
    Ok(!is_memory_url(&url))
}

fn is_memory_url(url: &str) -> bool {
    // Match sqlx's SQLite parser without to_url_lossy(): that method panics on
    // its synthetic in-memory filename (which contains a colon).
    let raw = url
        .trim_start_matches("sqlite://")
        .trim_start_matches("sqlite:");
    let (database, params) = raw.split_once('?').unwrap_or((raw, ""));
    database == ":memory:"
        || url::form_urlencoded::parse(params.as_bytes())
            .any(|(key, value)| key == "mode" && value == "memory")
}

fn resolve_with_file(
    database: &str,
    explicit: Option<&str>,
    configured: Option<&Path>,
) -> anyhow::Result<ServerKey> {
    if let Some(value) = explicit {
        return ServerKey::from_hex(value)
            .map_err(|_| anyhow::anyhow!("MW_SERVER_KEY must be a hex-encoded 32-byte key"));
    }
    let path = if let Some(path) = configured {
        path.to_path_buf()
    } else {
        if database.starts_with("postgres://") || database.starts_with("postgresql://") {
            bail!(
                "PostgreSQL requires MW_SERVER_KEY or MW_SERVER_KEY_FILE shared by all server instances"
            );
        }
        let url = if database.starts_with("sqlite:") {
            database.to_owned()
        } else {
            format!("sqlite://{database}?mode=rwc")
        };
        let options =
            SqliteConnectOptions::from_str(&url).context("invalid SQLite database path")?;
        if is_memory_url(&url) {
            return Ok(ServerKey::generate());
        }
        let filename = options.get_filename();
        if fs::symlink_metadata(filename).is_ok_and(|metadata| metadata.file_type().is_symlink())
            && !filename.exists()
        {
            bail!(
                "database path is a dangling symlink; create its target before starting the server"
            );
        }
        // Resolve the database itself if present, so alternate symlink spellings
        // do not generate different keys for the same database.
        let actual = if filename.exists() {
            filename.canonicalize()?
        } else {
            let parent = filename
                .parent()
                .filter(|p| !p.as_os_str().is_empty())
                .unwrap_or(Path::new("."));
            parent
                .canonicalize()?
                .join(filename.file_name().context("database has no filename")?)
        };
        let mut name = actual.into_os_string();
        name.push(".server-key");
        PathBuf::from(name)
    };
    load_or_create(&path)
        .with_context(|| format!("cannot load persistent server key {}", path.display()))
}

fn read_key(path: &Path) -> anyhow::Result<ServerKey> {
    let metadata = fs::symlink_metadata(path)?;
    if !metadata.is_file() || metadata.len() > 16_384 {
        bail!("server key must be a small regular file");
    }
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        if metadata.permissions().mode() & 0o077 != 0 {
            bail!("server key file must be private (chmod 600)");
        }
    }
    let bytes = Zeroizing::new(fs::read(path)?);
    let value = decode(&bytes)?;
    ServerKey::from_hex(value.trim()).map_err(|_| {
        anyhow::anyhow!("invalid server key file; restore the original key, do not replace it")
    })
}

fn load_or_create(path: &Path) -> anyhow::Result<ServerKey> {
    match fs::symlink_metadata(path) {
        Ok(_) => return read_key(path),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
        Err(error) => return Err(error.into()),
    }
    let key = ServerKey::generate();
    let bytes = encode(&Zeroizing::new(key.to_hex()))?;
    let parent = path
        .parent()
        .filter(|p| !p.as_os_str().is_empty())
        .unwrap_or(Path::new("."));
    let temporary = parent.join(format!(".mailwoman-key-{}.tmp", uuid::Uuid::new_v4()));
    let mut options = fs::OpenOptions::new();
    options.write(true).create_new(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        options.mode(0o600);
    }
    let mut file = options.open(&temporary)?;
    let result = (|| -> anyhow::Result<ServerKey> {
        file.write_all(&bytes)?;
        file.sync_all()?;
        drop(file);
        // Publish only complete bytes, atomically and without replacing another
        // process's key. A concurrent creator always reads the winning file.
        match fs::hard_link(&temporary, path) {
            Ok(()) => {
                #[cfg(unix)]
                fs::File::open(parent)?.sync_all()?;
                Ok(key)
            }
            Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => read_key(path),
            Err(error) => Err(error.into()),
        }
    })();
    let _ = fs::remove_file(&temporary);
    result
}

#[cfg(not(windows))]
fn encode(value: &str) -> anyhow::Result<Zeroizing<Vec<u8>>> {
    Ok(Zeroizing::new(value.as_bytes().to_vec()))
}
#[cfg(not(windows))]
fn decode(value: &[u8]) -> anyhow::Result<Zeroizing<String>> {
    Ok(Zeroizing::new(
        String::from_utf8(value.to_vec()).context("invalid key encoding")?,
    ))
}

// Windows generated files are DPAPI-protected for the server's OS identity,
// avoiding a plaintext key in a directory with inherited Windows permissions.
#[cfg(windows)]
const HEADER: &[u8] = b"mailwoman-dpapi-key-v1\n";
#[cfg(windows)]
fn encode(value: &str) -> anyhow::Result<Zeroizing<Vec<u8>>> {
    let mut bytes = Zeroizing::new(HEADER.to_vec());
    bytes.extend_from_slice(&dpapi(value.as_bytes(), false)?);
    Ok(bytes)
}
#[cfg(windows)]
fn decode(value: &[u8]) -> anyhow::Result<Zeroizing<String>> {
    let bytes = if let Some(sealed) = value.strip_prefix(HEADER) {
        dpapi(sealed, true)?
    } else {
        Zeroizing::new(value.to_vec())
    };
    Ok(Zeroizing::new(
        String::from_utf8(bytes.to_vec()).context("invalid key encoding")?,
    ))
}
#[cfg(windows)]
fn dpapi(value: &[u8], decrypt: bool) -> anyhow::Result<Zeroizing<Vec<u8>>> {
    use windows_sys::Win32::{
        Foundation::LocalFree,
        Security::Cryptography::{
            CRYPT_INTEGER_BLOB, CRYPTPROTECT_UI_FORBIDDEN, CryptProtectData, CryptUnprotectData,
        },
    };
    let input = CRYPT_INTEGER_BLOB {
        cbData: value.len().try_into()?,
        pbData: value.as_ptr().cast_mut(),
    };
    let mut output = CRYPT_INTEGER_BLOB {
        cbData: 0,
        pbData: std::ptr::null_mut(),
    };
    // The API reads input for this call and owns its output until LocalFree.
    // UI is forbidden; no machine-wide flag, so another OS identity cannot open it.
    let ok = unsafe {
        if decrypt {
            CryptUnprotectData(
                &input,
                std::ptr::null_mut(),
                std::ptr::null(),
                std::ptr::null(),
                std::ptr::null(),
                CRYPTPROTECT_UI_FORBIDDEN,
                &mut output,
            )
        } else {
            CryptProtectData(
                &input,
                std::ptr::null(),
                std::ptr::null(),
                std::ptr::null(),
                std::ptr::null(),
                CRYPTPROTECT_UI_FORBIDDEN,
                &mut output,
            )
        }
    };
    if ok == 0 {
        return Err(std::io::Error::last_os_error())
            .context("Windows could not protect/open the server key for this OS identity");
    }
    let result = unsafe {
        let slice = std::slice::from_raw_parts_mut(output.pbData, output.cbData as usize);
        let result = Zeroizing::new(slice.to_vec());
        zeroize::Zeroize::zeroize(slice);
        LocalFree(output.pbData.cast());
        result
    };
    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;
    struct Directory(PathBuf);
    impl Directory {
        fn new() -> Self {
            let path = std::env::temp_dir().join(format!("mw-key-test-{}", uuid::Uuid::new_v4()));
            fs::create_dir(&path).unwrap();
            Self(path)
        }
    }
    impl Drop for Directory {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.0);
        }
    }
    #[test]
    fn durable_key_matches_cli_and_uri_spelling() {
        let dir = Directory::new();
        let database = dir.0.join("mail.db");
        let path = database.to_str().unwrap();
        let first = resolve_with_file(path, None, None).unwrap().to_hex();
        assert_eq!(
            first,
            resolve_with_file(&format!("sqlite://{path}?mode=rwc"), None, None)
                .unwrap()
                .to_hex()
        );
        assert_eq!(first, resolve_with_file(path, None, None).unwrap().to_hex());
    }
    #[test]
    fn concurrent_creators_all_use_one_key() {
        let dir = Directory::new();
        let path = dir.0.join("key");
        let handles: Vec<_> = (0..8)
            .map(|_| {
                let path = path.clone();
                std::thread::spawn(move || load_or_create(&path).unwrap().to_hex())
            })
            .collect();
        let keys: Vec<_> = handles.into_iter().map(|h| h.join().unwrap()).collect();
        assert!(keys.iter().all(|key| key == &keys[0]));
        assert_eq!(fs::read_dir(&dir.0).unwrap().count(), 1);
    }
    #[test]
    fn explicit_key_wins_and_bad_file_is_never_rotated() {
        let dir = Directory::new();
        let path = dir.0.join("key");
        load_or_create(&path).unwrap();
        fs::write(&path, b"damaged").unwrap();
        let explicit = "11".repeat(32);
        assert_eq!(
            resolve_with_file("postgres://unused", Some(&explicit), Some(&path))
                .unwrap()
                .to_hex(),
            explicit
        );
        assert!(resolve_with_file("postgres://unused", None, Some(&path)).is_err());
        assert_eq!(fs::read(&path).unwrap(), b"damaged");
        assert!(resolve_with_file("postgres://unused", None, None).is_err());
        assert!(resolve_with_file("unused", Some("bad"), None).is_err());
    }
    #[test]
    fn memory_is_ephemeral_and_configured_file_is_durable() {
        assert_ne!(
            resolve_with_file("sqlite::memory:", None, None)
                .unwrap()
                .to_hex(),
            resolve_with_file("sqlite::memory:", None, None)
                .unwrap()
                .to_hex()
        );
        let dir = Directory::new();
        let path = dir.0.join("managed-key");
        assert_eq!(
            resolve_with_file("postgres://unused", None, Some(&path))
                .unwrap()
                .to_hex(),
            resolve_with_file("postgres://unused", None, Some(&path))
                .unwrap()
                .to_hex()
        );
        #[cfg(windows)]
        assert!(fs::read(&path).unwrap().starts_with(HEADER));
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            assert_eq!(
                fs::metadata(&path).unwrap().permissions().mode() & 0o777,
                0o600
            );
        }
    }

    #[test]
    fn sqlite_url_escapes_resolve_to_the_same_sidecar() {
        let dir = Directory::new();
        let bare = dir.0.join("mail box.db");
        let uri = format!(
            "sqlite://{}?mode=rwc",
            bare.to_str().unwrap().replace(' ', "%20")
        );
        assert_eq!(
            resolve_with_file(bare.to_str().unwrap(), None, None)
                .unwrap()
                .to_hex(),
            resolve_with_file(&uri, None, None).unwrap().to_hex()
        );
    }

    #[cfg(unix)]
    #[test]
    fn rejects_dangling_database_symlink_and_public_key_file() {
        use std::os::unix::{fs::PermissionsExt, fs::symlink};
        let dir = Directory::new();
        let alias = dir.0.join("alias.db");
        symlink(dir.0.join("missing.db"), &alias).unwrap();
        assert!(resolve_with_file(alias.to_str().unwrap(), None, None).is_err());
        let key = dir.0.join("key");
        load_or_create(&key).unwrap();
        fs::set_permissions(&key, fs::Permissions::from_mode(0o644)).unwrap();
        assert!(load_or_create(&key).is_err());
    }
}
