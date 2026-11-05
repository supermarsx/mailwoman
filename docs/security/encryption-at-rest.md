# Encryption at rest

The operator guide, with the per-store table, the configuration variables and the runbook, is [`../admin/encryption.md`](../admin/encryption.md). This page records the design and its limits.

## Design

- **Whole-file encryption for SQLite.** SQLCipher 4 encrypts each database page (AES-256-CBC with a per-page HMAC-SHA512) and so also the write-ahead log. The key comes from the environment (`MW_DB_KEY`, `MW_DB_KEY_FILE`, `MW_DB_KEY_COMMAND`), is applied to every pooled connection, and is never written to the database or the settings table.
- **No OpenSSL.** `deny.toml` bans `openssl` (SPEC section 5.1). SQLCipher's crypto provider interface is filled by Rust code on RustCrypto crates. The SQLCipher C source is the amalgamation inside `libsqlite3-sys` 0.30.1, vendored at `vendor/libsqlite3-sys-0.30.1` with the single change documented in its `MAILWOMAN-PATCH.md`. Stock SQLCipher 4 can read the files.
- **Application-level page-less encryption is not a substitute** and is not used. The existing per-column sealing (XChaCha20-Poly1305 under the server key) remains and is independent: the database key protects the file; the server key protects columns when the file is readable (Postgres, or SQLite without a database key).
- **Two keys on purpose.** A leaked database file without the database key reveals nothing. A leaked database key and file still do not reveal sealed columns without the server key. The reverse holds as well.

## Threats it addresses

Theft or loss of the database file, of a disk image or snapshot, or of a backup of the file; another local account reading the file; a plaintext copy turning up in a support bundle.

## Threats it does not address

An attacker who can run code as the server (it holds the key in memory); a reader of process memory, swap or core dumps; the search index, log files and retention archive on disk (listed in the guide); the key file or environment when the same attacker can read them; Postgres at rest (the application cannot reach the server's disk).

## Failure behaviour

The server refuses to start rather than fall back: wrong key, no key for an encrypted file, plaintext file under `required`, encrypted file under `off`. The migration tool copies, verifies and swaps, and restores the original on any failure. See the table in the guide.

## Verification

`mailwoman db verify` and the console button read the file's first bytes, refuse a random key, open with the real key, round-trip a marker row and search the file bytes for it. The tests in `crates/mw-store/tests/db_encryption.rs` read the raw bytes of the database and WAL.

## Postgres

Detection and reporting only (TLS state, `data_checksums`, `pgcrypto`, TDE indicators, tablespaces), `MW_DB_REQUIRE_TLS=1` to refuse a non-TLS DSN, and a list of sealed columns. Disk or volume encryption, TDE and encrypted Postgres backups are the operator's responsibility.
