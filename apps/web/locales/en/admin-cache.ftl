admin-cache-title = Cache maintenance
admin-cache-help = By default, cached email content expires after 60 days without use. Opening an expired message downloads its content again. Original messages, drafts and protected local content are preserved.
admin-cache-expiry = Cached message lifetime
admin-cache-orphans = Unreferenced bodies and space reclamation
admin-cache-advanced = Batch limits and timing
admin-cache-settings-help = Changes use the normal administrator settings policy, validation and audit trail.
admin-cache-expiry-enabled = Automatically expire cached message bodies
admin-cache-retention-days = Days without use before expiry
admin-cache-expiry-status = Cached body expiry
admin-cache-expiry-result = Scanned: { $scanned } · Detached cache references: { $detached } · Protected: { $protected }
admin-cache-expiry-help = Only bodies confirmed to be recoverable from an upstream mailbox are eligible. Scans and header refreshes do not renew the inactivity lease.
admin-cache-inspect = Inspect cache storage
admin-cache-inspect-help = Open database observability to measure cached payload, referenced bodies and orphaned copies.
admin-cache-no-status = No expiry batch has completed in this server process.
admin-cache-settings-unavailable = Cache settings could not be loaded.
admin-cache-change-log = Sync history
admin-cache-change-log-help = Old change records expire after 60 days by default. Clients using older sync tokens receive a full-resync response.
admin-cache-no-sources = No connected mailbox currently has a confirmed refetch path. Protected content is kept.
admin-cache-wal-recycled = Recycled allocated WAL space; cleanup can continue.
admin-cache-wal-recovery = Cleanup is waiting for WAL recovery or checkpointing. Pending journal data is preserved.

admin-cache-removed-payload = Cached payload removed
admin-cache-oversized = Large bodies deferred
admin-cache-change-removed = Sync-history rows removed

admin-cache-parsed-copies = Parsed body copies cleared
admin-cache-parsed-bytes = Parsed body data removed
admin-cache-change-chunked = Sync states cleaned in chunks
