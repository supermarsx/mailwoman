//! The local country database (t96): open, validate and hot-swap a MaxMind DB
//! (`.mmdb`) file without a restart.
//!
//! Two sources, in this order of precedence:
//! 1. **`MW_GEOIP_DB`** — an operator-supplied path. When the variable is set the
//!    environment owns the database: it is opened once per path (failure cached
//!    too) and nothing below is consulted. This is the pre-t96 behaviour.
//! 2. **The managed database** — downloaded, uploaded or seeded by `mw-server`
//!    into its data directory and installed here with [`install`]. Replacing it is
//!    a pointer swap under a lock: a lookup clones the `Arc` and releases the lock
//!    before it reads, so a lookup already running finishes against the database
//!    it started with and the next one sees the new database. No lookup observes a
//!    "no database" gap during a swap.
//!
//! No database is bundled and nothing here touches the network.

use std::collections::HashMap;
use std::fmt;
use std::net::IpAddr;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex, OnceLock, RwLock};

use maxminddb::{PathElement, Reader};
use sha2::{Digest, Sha256};

/// The environment variable that pins the database to an operator-supplied file.
pub const ENV_PATH: &str = "MW_GEOIP_DB";

/// The largest database accepted (decompressed). The DB-IP and GeoLite2 country
/// files are under 16 MB and IPinfo Lite is under 64 MB.
pub const MAX_DATABASE_BYTES: usize = 256 * 1024 * 1024;

/// Why a file is not a usable country database. The messages are shown to
/// administrators; they never include file contents.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum InvalidDatabase {
    /// Larger than [`MAX_DATABASE_BYTES`].
    TooLarge,
    /// Not a MaxMind DB file, or its search tree or data section is damaged.
    NotMmdb,
    /// A MaxMind DB of a format version other than 2.
    UnsupportedFormat,
    /// A valid MaxMind DB that holds no country codes (an ASN-only database).
    NoCountryData,
    /// The file could not be read.
    Unreadable,
}

impl fmt::Display for InvalidDatabase {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::TooLarge => "the database is larger than the 256 MB limit",
            Self::NotMmdb => "the file is not a valid MaxMind DB (.mmdb) database",
            Self::UnsupportedFormat => "the MaxMind DB format version is not supported",
            Self::NoCountryData => "the database contains no country codes",
            Self::Unreadable => "the database file could not be read",
        })
    }
}

impl std::error::Error for InvalidDatabase {}

/// What an administrator sees about a database. No file contents, no path.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DatabaseInfo {
    /// `database_type` from the file's metadata, e.g. `DBIP-Country-Lite`.
    pub database_type: String,
    /// Build time from the file's metadata, Unix seconds.
    pub build_epoch: u64,
    /// 4 or 6.
    pub ip_version: u16,
    /// Search-tree nodes.
    pub node_count: u32,
    /// The English description from the metadata, when present.
    pub description: Option<String>,
    /// File size in bytes.
    pub size_bytes: u64,
    /// SHA-256 of the file, lowercase hex.
    pub sha256: String,
}

/// An opened database.
pub struct CountryDatabase {
    reader: Reader<Vec<u8>>,
    info: DatabaseInfo,
    /// Networks that carry a country code, counted once on demand (it walks the
    /// whole tree, so it is never done on a lookup path).
    networks: OnceLock<Result<u64, InvalidDatabase>>,
}

impl fmt::Debug for CountryDatabase {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("CountryDatabase")
            .field("info", &self.info)
            .finish_non_exhaustive()
    }
}

/// A sample lookup made during validation: the first address of the first
/// network that has a country, and that country.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Sample {
    pub network: String,
    pub address: IpAddr,
    pub country: String,
}

fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}

fn is_country_code(code: &str) -> bool {
    code.len() == 2 && code.bytes().all(|b| b.is_ascii_uppercase())
}

/// The country code of one lookup result, from either schema in use:
/// `country.iso_code` (MaxMind GeoIP2/GeoLite2, DB-IP) or `country_code`
/// (IPinfo Lite). A field of another shape is not a country.
fn country_of<S: AsRef<[u8]>>(result: &maxminddb::LookupResult<'_, S>) -> Option<String> {
    if !result.has_data() {
        return None;
    }
    let nested = result
        .decode_path::<String>(&[PathElement::Key("country"), PathElement::Key("iso_code")])
        .ok()
        .flatten();
    let flat = || {
        result
            .decode_path::<String>(&[PathElement::Key("country_code")])
            .ok()
            .flatten()
    };
    nested
        .or_else(flat)
        .map(|code| code.to_ascii_uppercase())
        .filter(|code| is_country_code(code))
}

impl CountryDatabase {
    /// Open `bytes` as a MaxMind DB, checking only that it opens. Used for the
    /// operator's `MW_GEOIP_DB`, which may be an ASN or City database: whatever it
    /// carries is used, and a missing field is simply absent.
    pub fn open_lenient(bytes: Vec<u8>) -> Result<Self, InvalidDatabase> {
        if bytes.len() > MAX_DATABASE_BYTES {
            return Err(InvalidDatabase::TooLarge);
        }
        let size_bytes = bytes.len() as u64;
        let sha256 = hex(&Sha256::digest(&bytes));
        let reader = Reader::from_source(bytes).map_err(|_| InvalidDatabase::NotMmdb)?;
        let meta = reader.metadata();
        if meta.binary_format_major_version != 2 {
            return Err(InvalidDatabase::UnsupportedFormat);
        }
        if meta.node_count == 0 || !matches!(meta.ip_version, 4 | 6) {
            return Err(InvalidDatabase::NotMmdb);
        }
        let info = DatabaseInfo {
            database_type: meta.database_type.chars().take(128).collect(),
            build_epoch: meta.build_epoch,
            ip_version: meta.ip_version,
            node_count: meta.node_count,
            description: meta
                .description
                .get("en")
                .map(|d| d.chars().take(256).collect()),
            size_bytes,
            sha256,
        };
        Ok(Self {
            reader,
            info,
            networks: OnceLock::new(),
        })
    }

    /// Open `bytes` and validate it as a country database: the whole file passes
    /// the MaxMind DB structural check, at least one network carries a country
    /// code, and a lookup of that network's first address answers the same
    /// country. Walks the whole tree; call it off the async runtime.
    pub fn open_validated(bytes: Vec<u8>) -> Result<(Self, Sample), InvalidDatabase> {
        let db = Self::open_lenient(bytes)?;
        db.reader.verify().map_err(|_| InvalidDatabase::NotMmdb)?;
        let sample = db.first_sample()?;
        match db.country_networks() {
            Ok(0) => return Err(InvalidDatabase::NoCountryData),
            Ok(_) => {}
            Err(e) => return Err(e),
        }
        if db.country(sample.address).as_deref() != Some(sample.country.as_str()) {
            return Err(InvalidDatabase::NotMmdb);
        }
        Ok((db, sample))
    }

    /// [`Self::open_validated`] over a file.
    pub fn open_validated_path(path: &Path) -> Result<(Self, Sample), InvalidDatabase> {
        let bytes = read_capped(path)?;
        Self::open_validated(bytes)
    }

    fn whole_tree(&self) -> &'static str {
        if self.info.ip_version == 6 {
            "::/0"
        } else {
            "0.0.0.0/0"
        }
    }

    fn first_sample(&self) -> Result<Sample, InvalidDatabase> {
        let networks = self
            .reader
            .within(
                self.whole_tree()
                    .parse()
                    .map_err(|_| InvalidDatabase::NotMmdb)?,
                Default::default(),
            )
            .map_err(|_| InvalidDatabase::NotMmdb)?;
        for item in networks {
            let result = item.map_err(|_| InvalidDatabase::NotMmdb)?;
            if let Some(country) = country_of(&result) {
                let network = result.network().map_err(|_| InvalidDatabase::NotMmdb)?;
                // An IPv4 network inside an IPv6 tree is reported as ::a.b.c.d/n;
                // canonical form is the address an administrator would type.
                let address = match network.network() {
                    IpAddr::V6(v6) if network.prefix() >= 96 && v6.segments()[..6] == [0; 6] => {
                        let o = v6.octets();
                        IpAddr::from([o[12], o[13], o[14], o[15]])
                    }
                    other => other.to_canonical(),
                };
                return Ok(Sample {
                    network: network.to_string(),
                    address,
                    country,
                });
            }
        }
        Err(InvalidDatabase::NoCountryData)
    }

    /// Networks that carry a country code (the "record count"). Computed once.
    pub fn country_networks(&self) -> Result<u64, InvalidDatabase> {
        self.networks
            .get_or_init(|| {
                let networks = self
                    .reader
                    .within(
                        self.whole_tree()
                            .parse()
                            .map_err(|_| InvalidDatabase::NotMmdb)?,
                        Default::default(),
                    )
                    .map_err(|_| InvalidDatabase::NotMmdb)?;
                let mut count = 0u64;
                for item in networks {
                    let result = item.map_err(|_| InvalidDatabase::NotMmdb)?;
                    if country_of(&result).is_some() {
                        count += 1;
                    }
                }
                Ok(count)
            })
            .clone()
    }

    /// The database's metadata and digest.
    pub fn info(&self) -> &DatabaseInfo {
        &self.info
    }

    /// The country of `ip`, or `None` when the database has no country for it.
    pub fn country(&self, ip: IpAddr) -> Option<String> {
        let result = self.reader.lookup(ip.to_canonical()).ok()?;
        country_of(&result)
    }

    pub(crate) fn reader(&self) -> &Reader<Vec<u8>> {
        &self.reader
    }
}

/// Read a file of at most [`MAX_DATABASE_BYTES`].
pub fn read_capped(path: &Path) -> Result<Vec<u8>, InvalidDatabase> {
    let meta = std::fs::metadata(path).map_err(|_| InvalidDatabase::Unreadable)?;
    if meta.len() > MAX_DATABASE_BYTES as u64 {
        return Err(InvalidDatabase::TooLarge);
    }
    std::fs::read(path).map_err(|_| InvalidDatabase::Unreadable)
}

// ── the managed database ──────────────────────────────────────────────────────

static MANAGED: RwLock<Option<Arc<CountryDatabase>>> = RwLock::new(None);

/// Install (or with `None`, remove) the managed database, returning the one it
/// replaces. Lookups in progress keep the database they started with.
pub fn install(db: Option<Arc<CountryDatabase>>) -> Option<Arc<CountryDatabase>> {
    let mut slot = MANAGED.write().unwrap_or_else(|e| e.into_inner());
    std::mem::replace(&mut *slot, db)
}

/// The managed database, when one is installed.
pub fn installed() -> Option<Arc<CountryDatabase>> {
    MANAGED.read().unwrap_or_else(|e| e.into_inner()).clone()
}

// ── the environment database ──────────────────────────────────────────────────

type EnvCache = Mutex<HashMap<PathBuf, Result<Arc<CountryDatabase>, InvalidDatabase>>>;

fn env_cache() -> &'static EnvCache {
    static CACHE: OnceLock<EnvCache> = OnceLock::new();
    CACHE.get_or_init(|| Mutex::new(HashMap::new()))
}

/// `MW_GEOIP_DB` when it is set to a non-empty value. While it is set the
/// environment owns the database, whether or not the file exists.
pub fn environment_path() -> Option<PathBuf> {
    std::env::var_os(ENV_PATH)
        .filter(|v| !v.is_empty())
        .map(PathBuf::from)
}

/// The database at `path`, opened once per path; the outcome, including a
/// failure, is cached so a bad file is not re-read on every lookup.
pub fn environment_database(path: &Path) -> Result<Arc<CountryDatabase>, InvalidDatabase> {
    let mut cache = env_cache().lock().unwrap_or_else(|e| e.into_inner());
    if let Some(entry) = cache.get(path) {
        return entry.clone();
    }
    let opened = read_capped(path)
        .and_then(CountryDatabase::open_lenient)
        .map(Arc::new);
    cache.insert(path.to_path_buf(), opened.clone());
    opened
}

/// Forget the cached environment database so the next lookup re-opens the file
/// (after an operator replaced or repaired it).
pub fn forget_environment_database() {
    env_cache()
        .lock()
        .unwrap_or_else(|e| e.into_inner())
        .clear();
}

/// The database lookups use now: the environment's when `MW_GEOIP_DB` is set
/// (`None` when that file is missing or invalid), otherwise the managed one.
pub fn active() -> Option<Arc<CountryDatabase>> {
    match environment_path() {
        // A file that went away is no database, as before t96 (the cache keeps
        // the reader of a file that is still there).
        Some(path) if !path.exists() => None,
        Some(path) => environment_database(&path).ok(),
        None => installed(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::security::geoip_fixture::{self, Entry};

    fn fixture() -> Vec<u8> {
        geoip_fixture::country_mmdb(
            &[
                Entry::v4("81.2.69.0/24", "GB"),
                Entry::v4("89.114.0.0/16", "PT"),
                Entry::v6("2a01:4f8::/32", "DE"),
            ],
            1_790_000_000,
        )
    }

    #[test]
    fn validated_open_reports_metadata_sample_and_count() {
        let (db, sample) = CountryDatabase::open_validated(fixture()).unwrap();
        assert_eq!(db.info().database_type, "Mailwoman-Test-Country");
        assert_eq!(db.info().build_epoch, 1_790_000_000);
        assert_eq!(db.info().ip_version, 6);
        assert_eq!(db.info().sha256.len(), 64);
        assert_eq!(db.country_networks(), Ok(3));
        assert_eq!(sample.country, "GB");
        assert_eq!(sample.address, "81.2.69.0".parse::<IpAddr>().unwrap());
        assert_eq!(
            db.country("89.114.7.1".parse().unwrap()).as_deref(),
            Some("PT")
        );
        assert_eq!(
            db.country("2a01:4f8::1".parse().unwrap()).as_deref(),
            Some("DE")
        );
        assert_eq!(
            db.country("::ffff:81.2.69.160".parse().unwrap()).as_deref(),
            Some("GB")
        );
        assert_eq!(db.country("127.0.0.1".parse().unwrap()), None);
    }

    #[test]
    fn ipinfo_flat_schema_resolves() {
        let bytes = geoip_fixture::ipinfo_mmdb(&[Entry::v4("89.114.0.0/16", "PT")], 1);
        let (db, sample) = CountryDatabase::open_validated(bytes).unwrap();
        assert_eq!(sample.country, "PT");
        assert_eq!(
            db.country("89.114.0.9".parse().unwrap()).as_deref(),
            Some("PT")
        );
    }

    #[test]
    fn corrupt_truncated_and_country_less_files_are_refused() {
        assert_eq!(
            CountryDatabase::open_validated(b"not a database at all".to_vec()).unwrap_err(),
            InvalidDatabase::NotMmdb
        );
        let mut damaged = fixture();
        // Flip bytes in the search tree: the structural check must catch it.
        for b in damaged.iter_mut().take(64) {
            *b = 0xff;
        }
        assert!(CountryDatabase::open_validated(damaged).is_err());
        let truncated = fixture()[..40].to_vec();
        assert!(CountryDatabase::open_validated(truncated).is_err());
        let asn_only = geoip_fixture::asn_mmdb(&[("81.2.69.0/24", 20712)], 1);
        assert!(CountryDatabase::open_lenient(asn_only.clone()).is_ok());
        assert_eq!(
            CountryDatabase::open_validated(asn_only).unwrap_err(),
            InvalidDatabase::NoCountryData
        );
    }
}
