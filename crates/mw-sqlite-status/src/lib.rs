//! Small audited FFI boundary. No SQL, configuration changes or counter resets.
//! All pointer use is confined to a live SQLx LockedSqliteHandle, which excludes
//! SQLx's worker until this function returns. Only fixed read-only operations are
//! accepted: callers cannot provide pointers, operation numbers or reset flags.
#![deny(unsafe_op_in_unsafe_fn)]

#[derive(Clone, Copy, Debug)]
pub struct CacheStatus {
    pub used_bytes: Option<u64>,
    pub hits: Option<u64>,
    pub misses: Option<u64>,
    pub writes: Option<u64>,
    pub spills: Option<u64>,
}
/// Read cumulative counters without resetting them. SQLite uses signed 32-bit
/// counters; a negative value or unsuccessful operation is unknown, not zero.
pub async fn cache_status(
    connection: &mut sqlx::SqliteConnection,
) -> Result<CacheStatus, sqlx::Error> {
    let mut handle = connection.lock_handle().await?;
    let raw = handle.as_raw_handle().as_ptr();
    let read = |operation| {
        let (mut current, mut high) = (0, 0);
        // SAFETY: raw is nonnull and belongs to the live LockedSqliteHandle.
        // This guard excludes the SQLx worker, all output references remain
        // alive for this call, operations are fixed db-status queries, and
        // resetFlag=0 never changes the connection or resets a counter.
        let rc = unsafe {
            libsqlite3_sys::sqlite3_db_status(raw, operation, &mut current, &mut high, 0)
        };
        (rc == libsqlite3_sys::SQLITE_OK)
            .then(|| u64::try_from(current).ok())
            .flatten()
    };
    Ok(CacheStatus {
        used_bytes: read(libsqlite3_sys::SQLITE_DBSTATUS_CACHE_USED),
        hits: read(libsqlite3_sys::SQLITE_DBSTATUS_CACHE_HIT),
        misses: read(libsqlite3_sys::SQLITE_DBSTATUS_CACHE_MISS),
        writes: read(libsqlite3_sys::SQLITE_DBSTATUS_CACHE_WRITE),
        spills: read(libsqlite3_sys::SQLITE_DBSTATUS_CACHE_SPILL),
    })
}
