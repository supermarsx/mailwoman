# Admin > Compliance > Encryption: what is encrypted at rest and how that was determined.
# This reports configuration and observed file contents. It does not certify compliance with any law or standard.

admin-encryption-title = Encryption at rest
admin-encryption-intro = Where this server keeps data on disk, whether the bytes there are encrypted, and how each answer was found. A store is "encrypted" only when the file contents were read and are ciphertext. Reported states describe this server's configuration; they do not by themselves make a deployment compliant with any law or standard.
admin-encryption-docs = The key sources, the migration commands, rotation, recovery and what a lost key means are in docs/admin/encryption.md.
admin-encryption-loading = Loading encryption status…
admin-encryption-summary = Summary
admin-encryption-tile-database = Database
admin-encryption-tile-mode = Mode
admin-encryption-tile-key = Key source
admin-encryption-tile-rotation = Rotation due
admin-encryption-mode-off = Off
admin-encryption-mode-optional = Optional
admin-encryption-mode-required = Required
admin-encryption-key-none = Not configured
admin-encryption-no-due = No due date

admin-encryption-findings = Findings
admin-encryption-warning-databasePlaintext = The database file is readable as plain SQLite. Credentials, message bodies and uploads are sealed inside it; everything else is not.
admin-encryption-warning-modeNotRequired = The database is encrypted but the mode is optional, so a replaced plaintext file would be opened without complaint. Set MW_DB_ENCRYPTION=required.
admin-encryption-warning-searchIndexPlaintext = The search index on disk holds message text in the clear. Move it to memory (MW_SEARCH_DIR=memory) or put its directory on an encrypted volume.
admin-encryption-warning-plaintextRollbackCopy = A plaintext copy of the database from an earlier migration is still beside the database file. Delete it once the result is checked.
admin-encryption-warning-rotationOverdue = The database key is past its rotation interval.
admin-encryption-warning-keyBesideDatabase = The key file is in the same directory as the database. Anyone who can copy the database can copy the key.
admin-encryption-warning-postgresNoTls = The connection to Postgres is not using TLS.

admin-encryption-stores = Stores
admin-encryption-stores-note = One row per place data is kept. "Sealed columns only" means the store can be read as a whole but the sensitive columns inside it are sealed with the server key.
admin-encryption-col-store = Store
admin-encryption-col-state = State
admin-encryption-col-evidence = How this was determined
admin-encryption-store-database = Database
admin-encryption-store-searchIndex = Search index
admin-encryption-store-uploads = Uploaded files
admin-encryption-store-backups = Database backups
admin-encryption-store-logs = Log files
admin-encryption-store-exports = Data exports
admin-encryption-store-archive = Retention archive
admin-encryption-store-secrets = Server secrets
admin-encryption-state-encrypted = Encrypted
admin-encryption-state-sealed-columns-only = Sealed columns only
admin-encryption-state-plaintext = Plaintext
admin-encryption-state-memory-only = Memory only
admin-encryption-state-not-stored = Not stored
admin-encryption-state-unknown = Unknown
admin-encryption-counts = Encrypted: { $encrypted }. Sealed columns only: { $sealed_columns_only }. Plaintext: { $plaintext }. Memory only: { $memory_only }. Not stored: { $not_stored }. Unknown: { $unknown }.

admin-encryption-evidence-db-header = The database file was read: its first 16 bytes are not the SQLite header. { $cipher }. The write-ahead log is covered by the same key.
admin-encryption-evidence-db-plain-header = The database file starts with the SQLite header, so it is readable without a key.
admin-encryption-evidence-db-empty = The database file is empty or missing.
admin-encryption-evidence-db-memory = The database is in memory; nothing is written to disk.
admin-encryption-evidence-db-postgres = The server's disk cannot be observed from this application. The Postgres section below lists what the connection reveals. Disk or volume encryption is the operator's responsibility.
admin-encryption-evidence-error = Could not be read: { $error }
admin-encryption-evidence-search-disk = The index directory holds { $files } files of message text (sender, recipients, subject, body, attachment text). They are not encrypted.
admin-encryption-evidence-search-memory = The index is kept in memory and rebuilt from the database at start.
admin-encryption-evidence-uploads = Each object is sealed with the server key before it is written. { $sampled } files were sampled: { $plainLooking } looked like a known plain format.
admin-encryption-evidence-uploads-off = No upload directory is configured, so nothing is stored.
admin-encryption-evidence-backups-leftover = { $leftovers } rollback copy or copies from an earlier migration sit beside the database and at least one is plaintext.
admin-encryption-evidence-backups-unseen = The server does not create backups itself, so their state cannot be observed. `mailwoman db backup` writes encrypted copies; deploy.sh backups can be encrypted with age or gpg.
admin-encryption-evidence-logs-file = Log files in { $directory } are written as text with addresses, addresses of clients and tokens redacted. They are not encrypted.
admin-encryption-evidence-logs-stdout = Logs go to standard output only; whatever collects them decides how they are kept.
admin-encryption-evidence-exports = Data exports are stored in the database. They are covered when the database is encrypted; otherwise only when export sealing is turned on (sealing on: { $dsrExportSealed }).
admin-encryption-evidence-archive = Retention archive files are compressed and carry an integrity seal (HMAC), which detects edits but does not hide the content. Keep the archive directory on an encrypted volume.
admin-encryption-evidence-secrets = The server key is held as: { $serverKey }. It is a separate key from the database key. Key files are plain files protected only by file permissions.

admin-encryption-key = Database key
admin-encryption-key-source = Key source
admin-encryption-key-fingerprint = Key fingerprint
admin-encryption-key-age = Key age
admin-encryption-age-unknown = Not known for this key source
admin-encryption-age-days = { $days } days
admin-encryption-key-rotation = Rotation
admin-encryption-rotation-every = Every { $days } days
admin-encryption-overdue = Overdue

admin-encryption-actions = Checks and commands
admin-encryption-actions-note = Verify reads the database file and tests the key. The dry run exports a copy of the live data under a throwaway key, compares it table by table and deletes it. Changing the key or converting the file is done with the commands below while the server is stopped.
admin-encryption-verify = Verify encryption
admin-encryption-verifying = Verifying…
admin-encryption-rehearse = Dry run a migration
admin-encryption-rehearsing = Running…
admin-encryption-refresh = Refresh
admin-encryption-verify-passed = Verified: the database file is encrypted and the key works.
admin-encryption-verify-failed = Not verified.
admin-encryption-pass = Pass
admin-encryption-fail = Fail
admin-encryption-check-header-not-sqlite = The file does not start with the SQLite header
admin-encryption-check-wrong-key-rejected = A random wrong key is refused
admin-encryption-check-right-key-opens = The configured key opens the file
admin-encryption-check-row-round-trip = A known row is written, read back on a new connection and removed
admin-encryption-check-marker-not-in-file = That row's text does not appear in the file bytes
admin-encryption-check-key-configured = A database key is configured
admin-encryption-check-sqlite-file = The database is a SQLite file
admin-encryption-rehearse-ok = The copy matched the source: { $tables } tables, { $rows } rows, { $millis } ms. Nothing was changed.
admin-encryption-rehearse-failed = The dry run failed: { $error }
admin-encryption-runbook = Commands
admin-encryption-runbook-note = Run these on the server with the server stopped. The key is read from MW_DB_KEY, MW_DB_KEY_FILE or MW_DB_KEY_COMMAND; a new key from the MW_DB_NEW_* equivalents.

admin-encryption-postgres = Postgres
admin-encryption-postgres-note = What this connection can see. None of it is proof that the server's disk is encrypted, and the application does not encrypt it.
admin-encryption-pg-connection = This connection
admin-encryption-pg-tls-on = TLS ({ $version })
admin-encryption-pg-tls-off = Not using TLS
admin-encryption-pg-unknown = Could not be read
admin-encryption-pg-server-ssl = Server ssl setting
admin-encryption-pg-checksums = data_checksums (corruption detection, not encryption)
admin-encryption-pg-pgcrypto = pgcrypto
admin-encryption-pg-installed = Installed
admin-encryption-pg-available = Available, not installed
admin-encryption-pg-absent = Not available
admin-encryption-pg-tde = Transparent data encryption indicators
admin-encryption-pg-none-seen = None found

admin-encryption-columns = Sealed columns
admin-encryption-columns-note = Columns the application seals with the server key before writing, and some that it does not. The list is not exhaustive: a column that is absent is neither a claim that it is sealed nor that it is plain.
admin-encryption-col-table = Table
admin-encryption-col-column = Column
admin-encryption-col-holds = Holds
admin-encryption-col-sealed = Sealed
admin-encryption-sealed = Sealed
admin-encryption-not-sealed = Not sealed

admin-encryption-files-encrypt = Encrypt existing archive and log files
admin-encryption-files-help = Existing files are authenticated and encrypted before replacement. External storage, stdout and old backups remain the operator’s responsibility.
admin-encryption-files-done = File migration finished. Refresh the coverage results.
admin-encryption-evidence-files-encrypted = { $files }; { admin-encryption-state-plaintext }: { $plaintext }; { admin-encryption-state-unknown }: { $unknown }
