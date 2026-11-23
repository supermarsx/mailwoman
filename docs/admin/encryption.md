---
screen: true
---
# Encryption at rest

The **Encryption** tab of **Compliance** shows where the server keeps data on disk, whether those bytes are encrypted, and how each answer was found. This page explains the tab, the variables that turn database encryption on, and the commands that convert, re-key, back up and check the database. How the two keys work, and what they protect against, is explained in [Encryption at rest](/security/encryption-at-rest) of the security model; this page does not repeat it. Neither page says that a configuration meets a law or standard.

![The Encryption tab of Compliance on a default SQLite install: database Sealed columns only, mode Optional, no database key, and the first finding.](/screenshots/admin-privacy/encryption-overview.webp){.mw-shot}

## Before you start

- The tab needs `settings.read` on all domains; **Verify encryption**, **Dry run a migration** and **Encrypt existing archive and log files** need `settings.write`.
- Database encryption (SQLCipher) is configured with environment variables, because they are read before the database opens. The console cannot turn it on, change the key or convert the file.

::: warning What a default install protects
By default, `MW_DB_ENCRYPTION` is `optional` and no database key is set, so the SQLite file is plain. Message bodies, envelopes, credentials and other sealed columns are encrypted with the **server key**, and on a default SQLite install that key is created in a file beside the database (`<database>.server-key`). Anyone who copies the whole data directory has both. The finding "The key file is in the same directory as the database" checks only the database key file, not the server key. Keep the server key elsewhere (`MW_SERVER_KEY` or `MW_SERVER_KEY_FILE`) and set a database key if a copied data directory must not be readable. On Windows the generated server key file is protected with DPAPI for the account the server runs as; installs made with `deploy.sh` keep the server key in the environment file, outside the data volume.
:::

## Read the tab

The **Summary** tiles give the state of the **Database**, the **Mode**, the database **Key source** and when the key is next due for rotation (**Rotation due**). **Findings** lists what the server considers a weakness:

| Finding | Meaning |
| --- | --- |
| The database file is readable as plain SQLite | No database key is in use. |
| The database is encrypted but the mode is optional | A plaintext file put in its place would be opened. Set `MW_DB_ENCRYPTION=required`. |
| The search index on disk holds message text in the clear | Move it to memory (`MW_SEARCH_DIR=memory`) or onto an encrypted volume. |
| A plaintext copy of the database from an earlier migration is still beside the database file | Delete it with `mailwoman db purge-rollback` once the result is checked. |
| The database key is past its rotation interval | The key is older than **Database key rotation interval (days)**. |
| The key file is in the same directory as the database | The database key file (`MW_DB_KEY_FILE`) sits beside the database. |
| The connection to Postgres is not using TLS | Postgres only. |

The **Stores** table has one row per place data is kept.

![The Stores table of the Encryption tab: each place the server keeps data, whether it is encrypted, and how that was determined.](/screenshots/admin-privacy/encryption-stores.webp){.mw-crop}

A store is **Encrypted** only when its file contents were read and are ciphertext. **Sealed columns only** means the store can be read as a whole but its sensitive columns are sealed with the server key. **Uploaded files** is judged from file headers; an empty upload folder is reported as **Sealed columns only**. In the image the server key is supplied through the environment, which is how the documentation stack runs.

**Database key** shows the key source, its fingerprint, its age and the rotation interval. **Sealed columns** lists the columns the server seals before they are written, and some it does not.

## Check the encryption

![The Checks and commands section of the Encryption tab, with markers on its three buttons, and the list of commands.](/screenshots/admin-privacy/encryption-checks.webp){.mw-crop}

1. **Encrypt existing archive and log files** encrypts retention archive files and rotated log files that are still in plaintext. It is recorded in the audit log (`encryption-files-migrated`).
2. **Verify encryption** reads the database file and tests the key. It checks that the file does not start with the SQLite header, that a random key is refused, that the configured key opens the file, and that a row written and read back on a new connection does not appear in the file bytes. The result is recorded in the audit log (`encryption-verified`). It is available for SQLite only.
3. **Dry run a migration** copies the live data under a throwaway key, compares it table by table and deletes the copy. It tells you how long a real conversion takes without stopping the server. SQLite only.
{.mw-callouts}

The same checks are available on the command line as `mailwoman db verify`.

## Configuration

| Variable | Meaning |
| --- | --- |
| `MW_DB_ENCRYPTION` | `off`, `optional` (default) or `required` |
| `MW_DB_KEY`, `MW_DB_KEY_FILE`, `MW_DB_KEY_COMMAND` | The database key: 64 hexadecimal characters, or a passphrase of at least 16 characters; or a file holding it; or a program that prints it |
| `MW_DB_NEW_KEY`, `MW_DB_NEW_KEY_FILE`, `MW_DB_NEW_KEY_COMMAND` | The new key for `db encrypt` and `db rekey` |
| `MW_DB_BACKUP_KEY`, `MW_DB_BACKUP_KEY_FILE`, `MW_DB_BACKUP_KEY_COMMAND` | A separate key for `db backup` and `db restore`; the database key is used when none is set |
| `MW_DB_REQUIRE_TLS` | Postgres only: `1` refuses a connection string that does not force TLS |
| `MW_SERVER_KEY`, `MW_SERVER_KEY_FILE` | The server key; Postgres refuses to start without one of them |

At start, `required` refuses a plaintext file or a missing key; `optional` opens a plaintext file, or an encrypted file with its key; `off` refuses an encrypted file. A wrong key is reported as a wrong key. See [Startup rules](/security/encryption-at-rest#startup-rules). Use `required` once the file is encrypted.

## Runbook

The `mailwoman db` commands work on the SQLite file and read keys only from the environment. Stop the server first.

### Turn on encryption for a new install

1. Generate a key with `mailwoman db keygen` and store it in your secret manager.
2. Set `MW_DB_KEY` (or `MW_DB_KEY_FILE` or `MW_DB_KEY_COMMAND`) and `MW_DB_ENCRYPTION=required`.
3. Start the server. The database is created encrypted.

### Convert an existing database

1. Stop the server and back up the data directory.
2. Set `MW_DB_NEW_KEY` to the new key.
3. Run `mailwoman db encrypt --dry-run`, then `mailwoman db encrypt`. The data is copied to `<database>.mw-new`, compared, and swapped in; the original is kept as `<database>.pre-encrypt-<time>`.
4. Set the key as `MW_DB_KEY` and `MW_DB_ENCRYPTION=required`, and start.
5. Select **Verify encryption**, then delete the rollback copy with `mailwoman db purge-rollback <copy>`. Until then the copy is a plaintext copy of the data. To undo instead, use `mailwoman db rollback --copy <copy>`.

### Rotate the database key

1. Generate a new key and set it as `MW_DB_NEW_KEY`.
2. With the server stopped, run `mailwoman db rekey`.
3. Point `MW_DB_KEY` at the new key and start. The rotation date is written to `<database>.encryption.json`.

Re-keying re-encrypts the database file only. It does not change the server key, so the sealed columns, uploaded files, the search index, encrypted logs and the retention archive keep their keys. The server has no command to replace the server key.

### Back up and restore

`mailwoman db backup --to FILE` writes an encrypted copy under the backup key, and refuses to write a plaintext copy of an encrypted database unless you add `--plaintext`. `mailwoman db restore --from FILE --to NEW` restores it under the current key. A restore needs the server key as well; back it up separately from the database.

### Turn encryption off

`mailwoman db decrypt` converts the file back to plaintext. Then set `MW_DB_ENCRYPTION` to `off` or `optional`.

### A lost key

- **Database key lost everywhere**: the database cannot be opened. Restore from a backup whose key you still have.
- **Server key lost**: the database opens, but sealed columns (credentials, cached message bodies, notes, second-factor secrets), queued compliance email, encrypted logs, the retention archive and the search index cannot be read. A damaged server key file is never replaced automatically; the server stops instead.

## Postgres

The server cannot see a Postgres server's disk. The **Postgres** section shows what the connection reports: whether it uses TLS, the server's `ssl` setting, data checksums, whether `pgcrypto` is available, and settings or extensions whose names suggest disk encryption. None of these proves the disk is encrypted. Encrypt the volume and the Postgres backups yourself, and set `MW_DB_REQUIRE_TLS=1`.

## Options

| Setting | What it does | Default |
| --- | --- | --- |
| **Database key rotation interval (days)** ([`encryption.rotationDays`](/admin/settings#encryption.rotationDays)) | Days until the database key is reported as due; 0 for no due date | 365 |
| **Report encryption findings** ([`encryption.warnings.enabled`](/admin/settings#encryption.warnings.enabled)) | Raises the findings in the deployment security assessment. The Encryption tab shows its findings either way | on |

Both are server-wide.

## Related

- [Encryption at rest (security model)](/security/encryption-at-rest)
- [Zero-access storage](/security/zero-access)
- [Backups](/deploy/backups)
- [Postgres](/deploy/postgres)
