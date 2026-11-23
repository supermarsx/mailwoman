# Encryption at rest

This page explains how Mailwoman protects data on disk, which keys exist, where they are kept, and what each protection does and does not cover. The operator guide with the per-store table, the configuration variables and the step-by-step runbook is [Encryption at rest (admin)](/admin/encryption).

## Two keys, two layers

Mailwoman uses two independent keys.

| Key | Protects | Where it lives | Created by |
|---|---|---|---|
| **Database key** | The whole SQLite database file, its write-ahead log and journal (SQLCipher) | Outside the database: `MW_DB_KEY`, `MW_DB_KEY_FILE` or the output of `MW_DB_KEY_COMMAND`. Never written to the database or the settings table | The operator (`mailwoman db keygen` prints a random key) |
| **Server key** | Individual sensitive columns and objects ("sealed" data), plus purpose-specific subkeys | `MW_SERVER_KEY` (hex), the file named by `MW_SERVER_KEY_FILE`, or, for SQLite only, a sidecar file `<database path>.server-key` created on first start | The server, on first start, unless you supply one |

The layers are independent. A copy of the database file without the database key is unreadable. A copy of the database file together with the database key still does not reveal sealed columns without the server key.

::: warning Default deployments
Database encryption is off until you configure a database key: the default mode is `optional`, and with no key a plaintext SQLite file is used. The server key's default location is a file next to the database. Someone who copies the whole data directory of a default SQLite install therefore gets both the database and the key that opens its sealed columns. Keep the server key on a different volume or in a secret store (`MW_SERVER_KEY_FILE`, `MW_SERVER_KEY`) and set a database key if that matters in your threat model.
:::

## Whole-file encryption (SQLCipher)

- SQLite builds link SQLCipher 4. Every page of the database file and of the write-ahead log is encrypted (AES-256-CBC with a per-page HMAC-SHA512, the SQLCipher 4 defaults). Temporary tables and indices are kept in memory by default (compile flag `SQLITE_TEMP_STORE=2`).
- The key is applied as `PRAGMA key` on every pooled connection, so migrations, background jobs and the command-line tools all open the file the same way.
- A 64-character hex value is used as the 256-bit key directly. Any other value is treated as a passphrase: it must be at least 16 characters, and it is stretched with PBKDF2-HMAC-SHA512 (310,000 iterations, a fixed application salt) once per process. Prefer a random hex key.
- OpenSSL is not used. SQLCipher's crypto-provider table is filled by Rust code on the RustCrypto crates (AES, CBC, HMAC, SHA-1/256/512, PBKDF2) and the system random source. The SQLCipher C source is the amalgamation shipped with `libsqlite3-sys` 0.30.1, vendored with one build patch described in `vendor/libsqlite3-sys-0.30.1/MAILWOMAN-PATCH.md`. Stock SQLCipher 4 tools can open the files with the same key.

### Startup rules

`MW_DB_ENCRYPTION` selects the mode. The server refuses to start instead of falling back:

| Mode | Plaintext file | Encrypted file |
|---|---|---|
| `off` | Opens | Refused |
| `optional` (default) | Opens, even when a key is configured | Opens with the key; refused without one |
| `required` | Refused | Opens with the key; refused without one |

A wrong key is reported as a wrong key, not as a corrupt database. `mailwoman db encrypt`, `db rekey` and `db decrypt` copy the database, verify every table's row count and contents in the copy, then swap it in, and leave the original untouched if anything fails. Backups made with `db backup` of an encrypted database are refused in plaintext unless you explicitly allow it.

### Proving it works

`mailwoman db verify` (and the matching button on the console's encryption view) checks that the first 16 bytes of the file are not the plain SQLite header, that a random key is refused, that the configured key opens the file, and that a marker row written through one connection can be read back through a new one.

## Sealed columns and objects (server key)

Sealing is authenticated encryption with XChaCha20-Poly1305: a random 24-byte nonce followed by the ciphertext and tag. Some records also bind associated data (a purpose string and the owning account), so a sealed value copied into another row or used for another purpose fails to open.

What is sealed: upstream credentials and tokens, cached message bodies and envelopes, notes, embeddings, second-factor secrets, uploaded files, and other secrets. The exact list per store is in the [admin guide](/admin/encryption).

Purpose-specific keys are derived from the server key as `HMAC-SHA256(server key, "mailwoman-subkey:" ‖ label)`, so each use (for example the encrypted search index files, remote-content URL signing, or browser storage namespaces) has its own key and none reveals the server key.

How the server key file is protected:

- On Linux and macOS a generated key file is created with mode `0600`, and the server refuses to read a key file that is readable by group or others.
- On Windows a generated key file is encrypted with DPAPI for the account the server runs as.
- With Postgres there is no sidecar file: the server refuses to start unless `MW_SERVER_KEY` or `MW_SERVER_KEY_FILE` is set, and every server instance must use the same key.

A damaged or replaced server key file is never regenerated silently; the server stops, because a new key would make every sealed record unreadable.

The server also writes a copy of the server key wrapped with a hybrid X25519 + ML-KEM-768 key (table `store_key_material`). The secret needed to unwrap it is not kept, so this copy is not used for anything and gives no additional protection; the server key itself is protected only as described above.

## What this protects against

- Theft or loss of the database file, a disk image, a snapshot or a file-level backup, when the database key is not stored with them.
- Another local account reading the database file.
- A database copy turning up in a support bundle or ticket.
- With the server key kept elsewhere: a reader of the database (SQLite or Postgres) seeing credentials, message bodies, notes and the other sealed values.

## What it does not protect against

- Anyone who can run code as the server process, read its memory, swap or core dumps, or read its environment: the running server holds both keys in memory.
- An attacker who can read the database key or server key from where you stored them.
- Plaintext copies left by earlier operations, such as rollback files `<db>.pre-*` after `db encrypt` until you purge them, old snapshots, or SSD blocks.
- Postgres at rest. Mailwoman cannot reach the database server's disk. It reports what it can observe (whether the connection uses TLS, `data_checksums`, `pgcrypto`, transparent-encryption indicators and tablespaces) and can refuse a connection string that does not force TLS (`MW_DB_REQUIRE_TLS=1`). Volume encryption and encrypted Postgres backups are the operator's job.
- The Redis/Valkey cache, your reverse proxy and your mail servers.

## Related

- [Encryption at rest (admin guide)](/admin/encryption)
- [Zero-access storage](./zero-access.md), which keeps mail encrypted under keys the server never holds
- [Backups](/deploy/backups)
- [Postgres](/deploy/postgres)
