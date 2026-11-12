# Encryption at rest

This page says what Mailwoman encrypts on disk, what it does not, and how to turn the encryption on, rotate its key, recover, and prove that it works. It describes configuration and observed file contents. It does not say that a deployment complies with any law or standard; that is for the operator to assess.

The live view is **Administration > Compliance > Encryption**. Every row there is one of:

| State | Meaning |
|---|---|
| Encrypted | The bytes on disk were read and are ciphertext. |
| Sealed columns only | The store can be read as a whole, but the sensitive columns inside it are sealed with the server key. |
| Plaintext | Readable by anyone who can read the file or directory. |
| Memory only / Not stored | Nothing is written to disk. |
| Unknown | The application cannot observe it (for example a Postgres server's disk). |

## What is protected, store by store

Implementation updated 2026-11-12. Check the live coverage view for the files present in your deployment.

| Store | SQLite, key configured | SQLite, no key | Postgres |
|---|---|---|---|
| Database file, WAL and journal | Encrypted (SQLCipher 4, AES-256-CBC, HMAC-SHA512 per page) | Plain SQLite file. Credentials, cached message bodies and envelopes, notes, embeddings, second-factor secrets and other secrets are sealed per column (XChaCha20-Poly1305 under the server key); everything else is readable | Not encrypted by the application. Same per-column sealing. Disk or volume encryption is the operator's |
| Data export parts (`dsr_part`) | Encrypted with the database | Sealed only when `dsr.export.encrypt` is on, otherwise plaintext | Same as the left column, minus the file |
| Search index (`<db>.search-index` or `MW_SEARCH_DIR`) | Authenticated XChaCha20-Poly1305 files, with a purpose-separated server-key subkey | Same | Memory by default; encrypted files when a directory is configured |
| Uploaded files (`MW_UPLOAD_DIR`) | Sealed per object with the server key | Same | Same |
| Embeddings | Inside the database, sealed per vector | Same | Same |
| Log files (`logging.file.*`) | Independently authenticated encrypted records, with redaction before encryption | Same | Same |
| Retention archive (`retention.archive.directory`) | Gzip compressed in memory, then authenticated encryption; HMAC chain retained | Same | Same |
| Server key (`<db>.server-key`, `MW_SERVER_KEY`, `MW_SERVER_KEY_FILE`) | A plain file protected by permissions, or an environment variable | Same | Same |
| Database key (`MW_DB_KEY*`) | Never in the database or settings. A file, environment variable or command output | n/a | n/a |
| Rollback copies (`<db>.pre-*`) | Plaintext after `db encrypt` until you purge them | n/a | n/a |
| `deploy.sh backup` archives | Encrypted with age or gpg when a recipient is configured; the database file inside stays encrypted too | Plain unless a recipient is configured | Same |

Not covered by anything here: process memory, swap, core dumps, the clear-text key while the server runs, the Redis/Valkey cache (see `docs/deploy/cache.md`), and your reverse proxy and mail servers.

Persistent search migrates existing Tantivy metadata and segment files while
holding the real index writer lock. File encryption binds each ciphertext to
its filename. Wrong keys or tampering refuse opening; the server may rebuild a
memory-only index from the database instead of writing plaintext. Decrypted
index data lives in process memory, so encrypted search uses more RAM than
plaintext memory mapping.

**Administration → Compliance → Encryption → Encrypt existing archive and log
files** migrates historical app-owned logs and archives. Archive metadata changes
only after the new ciphertext is synced, and the old plaintext file is removed
afterward. Migrated archives retain their original logical content hashes so the
existing integrity chain remains valid. Read, export, restore, verification and
log scrubbing support the encrypted formats. The current log migrates before
its next append; the explicit action covers rotated historical logs too.

The coverage view reads current files, authenticates samples and archive
metadata, and flags legacy plaintext or unverifiable files rather than assuming
that an enabled setting proves encryption. Disabled file logging does not hide
existing log files. An interrupted migration may leave extra files; investigate
reported plaintext/unknown results before declaring coverage complete.

Migration cannot erase old filesystem snapshots, SSD blocks or existing backups.
Use encrypted host volumes and protect the server key separately. Container/stdout
logs and remote-provider storage remain the operator's responsibility.

## How it works

SQLite builds that include SQLCipher encrypt every page of the database file and of the write-ahead log, and cover temporary data too (`temp_store` is memory). The key is applied with `PRAGMA key` as the first statement of every pooled connection, so migrations, background jobs and the CLI tools are all covered.

- The crypto provider is written in Rust on the RustCrypto crates (`aes`, `cbc`, `hmac`, `sha2`, `pbkdf2`). OpenSSL is not used, which keeps `deny.toml`'s ban on `openssl` intact. The SQLCipher C code is built from the amalgamation shipped with `libsqlite3-sys` 0.30.1, vendored with one patch (`vendor/libsqlite3-sys-0.30.1/MAILWOMAN-PATCH.md`). The file format is stock SQLCipher 4.
- A 64-character hex key is used as the key itself (no per-connection stretching). Any other secret is a passphrase of at least 16 characters, stretched once per process with PBKDF2-HMAC-SHA512 (310,000 rounds, fixed application salt). Prefer a random hex key: `mailwoman db keygen`.
- A file that is not encrypted stays openable when no key is configured. The build always contains SQLCipher; encryption is chosen by configuration.

## Configuration

These are read from the environment before the database opens, which is why they are not settings (the settings table lives inside the database).

| Variable | Meaning |
|---|---|
| `MW_DB_ENCRYPTION` | `off`, `optional` (default) or `required`. |
| `MW_DB_KEY` | The key (64 hex characters, or a passphrase). |
| `MW_DB_KEY_FILE` | A file holding the key. Its age is shown in the console. |
| `MW_DB_KEY_COMMAND` | A program that prints the key, for a secret manager (`vault kv get -field=key secret/mailwoman`, a cloud CLI). Split on spaces, no shell. |
| `MW_DB_NEW_KEY`, `_FILE`, `_COMMAND` | The new key for `db encrypt` and `db rekey`. |
| `MW_DB_BACKUP_KEY`, `_FILE`, `_COMMAND` | A separate key for `db backup` and `db restore`; the database key is used when none is set. |
| `MW_TEST_DB_KEY` | Test toggle: lowest-priority key source, used by the test suite (see below). Do not set it in production. |
| `MW_DB_REQUIRE_TLS` | Postgres only. `1` refuses a DSN whose `sslmode` is not `require`, `verify-ca` or `verify-full`. |

Behaviour at start:

| Mode | Key | File on disk | Result |
|---|---|---|---|
| `required` | missing | any | Refuses to start: no key |
| `required` | set | plaintext | Refuses to start: run `mailwoman db encrypt` first |
| `required` | set | encrypted or new | Starts; a new file is created encrypted |
| `optional` | set | new | Starts; created encrypted |
| `optional` | set | plaintext | Starts on the plaintext file and the console flags it |
| `optional` | missing | encrypted | Refuses to start: encrypted file, no key |
| `off` | any | encrypted | Refuses to start: file is encrypted but encryption is off |
| any | wrong key | encrypted | Refuses to start: "the database key does not open ..." |

Use `required` in production once the file is encrypted. `optional` would also accept a plaintext file that someone put in place of the real one.

Settings (global): `encryption.rotationDays` (days until the key is reported as due, 0 for no due date) and `encryption.warnings.enabled` (show findings in the deployment assessment). The key itself is never stored in settings.

## Runbook

All `mailwoman db` commands work on the file named by `MW_DB_PATH` (or `--db-path`) and read keys only from the environment. Stop the server first; the commands refuse to run when another process holds the database.

### Enable on a new install

```sh
export MW_DB_KEY=$(mailwoman db keygen)      # store it in your secret manager now
export MW_DB_ENCRYPTION=required
mailwoman serve
```

### Convert an existing plaintext database

1. Stop the server and take a backup of the data directory.
2. Set `MW_DB_KEY*` to a new key and keep the key somewhere other than the server.
3. `mailwoman db encrypt --dry-run` reports the tables and rows it would copy and changes nothing.
4. `mailwoman db encrypt`. It exports to `<db>.mw-new`, compares every table (row counts and a content digest) and `integrity_check`, renames the original to `<db>.pre-encrypt-<time>`, renames the new file into place and opens it once more. Any failure restores the original.
5. Start with `MW_DB_ENCRYPTION=required`.
6. Check in the console (Compliance > Encryption > Verify encryption), then `mailwoman db purge-rollback <db>.pre-encrypt-<time>`. Until you do, that file is a **plaintext copy** of the data, and the console warns about it. Purging overwrites with zeros and deletes; on SSDs and journaling file systems old blocks can survive, so use an encrypted volume if that matters.

To undo before purging: `mailwoman db rollback --copy <db>.pre-encrypt-<time>`.

### Rotate the key

1. Generate a new key (`mailwoman db keygen`) and store it.
2. With the server stopped: `MW_DB_NEW_KEY=... mailwoman db rekey --dry-run`, then without `--dry-run`.
3. Point `MW_DB_KEY*` at the new key and start. Keep the old key until you have purged the rollback copy, which is encrypted under the old key.
4. The rotation date is recorded in `<db>.encryption.json` (fingerprints and dates, no key). The console shows it and, when `encryption.rotationDays` passes, a finding.

Rotation re-encrypts the whole file; it takes about as long as copying it. The dry run in the console (**Dry run a migration**) does the same copy against the live data under a throwaway key, compares it and deletes it, which tells you how long a real run will take without stopping anything.

### Backups

`mailwoman db backup --to FILE` writes a consistent, verified copy encrypted under `MW_DB_BACKUP_KEY*` (or the database key). It refuses to write a plaintext copy of an encrypted database unless you pass `--plaintext`. `mailwoman db restore --from FILE --to NEW` restores it under the current key. Test a restore before you rely on it.

The sealed columns inside the database are encrypted with the **server key**, which is separate from the database key. A restore needs both. Back up the server key too (see `docs/deploy/backups.md`), separately from the database.

### Decrypt

`mailwoman db decrypt` converts back to plaintext. Set `MW_DB_ENCRYPTION=off` or `optional` afterwards.

### Recovery and a lost key

- **Wrong key.** Nothing is damaged. The server refuses to start and names the file. Fix the key source and start again.
- **Key file lost, key in a secret manager or a copy.** Restore the key; no change to the database.
- **A failed `db encrypt`, `rekey` or `decrypt`.** The original is untouched or restored; look for `<db>.mw-new` and delete it.
- **The key is lost everywhere.** The database cannot be opened and the data cannot be recovered. SQLCipher has no back door and neither does Mailwoman. Restore from a backup whose key you still have. This is the cost of the protection, so keep at least two copies of the key in two places, separate from the database and its backups.
- **The server key is lost.** The database opens, but sealed columns (credentials, cached bodies, notes, second-factor secrets) cannot be read. See `docs/security/zero-access.md`.

## Checking that it works

The **Verify encryption** button (and `mailwoman db verify`) does this against the live file:

1. reads the first 16 bytes and checks they are not `SQLite format 3`;
2. tries to open the file with a random key and expects a refusal;
3. opens it with the configured key and reads the schema;
4. writes a marker row, reads it back on a new connection, removes it, and checks that the marker text does not appear in the file bytes.

Step 4 writes and deletes one row in the table `mw_encryption_probe`. The result is audited (`encryption-verified`).

The tests that prove this in CI are `crates/mw-store/tests/db_encryption.rs`: raw bytes of the file and its WAL contain neither the SQLite header nor a known marker; a wrong or missing key fails; plaintext to encrypted keeps every row including the migrations table; rekey and decrypt; rollback; backups with a separate key; verification flags a plaintext file.

To run the existing store tests against encrypted databases, set a key and use one thread:

```sh
MW_TEST_DB_KEY=$(mailwoman db keygen) cargo test -p mw-store -- --test-threads=1
```

With the variable set, every `Store::open` in the test run creates an encrypted file. Tests that read the file with their own `sqlx` pool use `mw_store::encryption::raw_options(path, read_only)`, which applies the key when the file is encrypted. The CI `rust` job runs this leg after the normal one. On 2026-11-05 the lib tests (388) and the 34 integration targets tracked in git of `mw-store` passed under it on Windows.

## Postgres

The application cannot encrypt a Postgres server's disk and does not claim to. What it does:

- **Detection.** The Postgres section of the console shows, from the connection: whether this connection uses TLS (`pg_stat_ssl`) and the TLS version, the server's `ssl` setting, `data_checksums` (corruption detection, not encryption), whether `pgcrypto` is installed or available, any setting or extension whose name contains `tde` or `encrypt`, and the tablespaces. None of that proves the disk is encrypted; absence of a TDE indicator proves nothing either.
- **Mandatory TLS.** `MW_DB_REQUIRE_TLS=1` refuses a DSN that does not force TLS. Use `sslmode=verify-full` with the server's CA.
- **Column sealing.** The coverage table in the console lists the columns the application seals before they reach Postgres and some that it does not.
- **Operator's job.** Encrypt the volume (LUKS, cloud disk encryption) or use a Postgres build with TDE, encrypt Postgres backups and WAL archives, and restrict who can read `pg_dump` output.

## Build and CI

SQLCipher is compiled from C in `libsqlite3-sys` with the `cc` crate: a C compiler is needed on every platform (MSVC Build Tools on Windows, `cc`/`clang` on Linux and macOS), which the previous bundled-SQLite build already needed. The SQLCipher amalgamation is larger than plain SQLite's, so a clean build of that crate takes tens of seconds longer. No OpenSSL, perl or NASM is needed. The vendored crate adds about 10 MB to the repository. Licences: SQLCipher community edition is BSD-style (Zetetic); the RustCrypto crates were already in `Cargo.lock`.
