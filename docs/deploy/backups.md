# Backups with deploy.sh

This page covers what happens to `deploy.sh` snapshots after they are taken: retention, encryption, copies to other hosts and scheduling. How a snapshot is taken and restored is described in [Deploy with deploy.sh](script.md#backups-and-recovery). The end of the page lists the backup commands for installations that do not use `deploy.sh`.

## Before you start

`bash deploy.sh backup` stops the app for the length of the copy and writes a snapshot directory under `.deploy.local/backups/`, named `YYYYMMDDTHHMMSSZ-<pid>-<random>` (the UTC time it was taken). It holds:

| File | Content |
|---|---|
| `data.tar` | All of `/data`: the SQLite database and its WAL files, uploads and the retained web assets. On a full stack, also a PostgreSQL dump. |
| `app.env`, `deploy.env`, `compose.yaml`, `release.env` | The applied configuration and the exact image IDs. |
| `SHA256SUMS` | Checksums of the five files above. |
| `COMPLETE` | Written last; a snapshot without it is incomplete. |

::: warning
Every snapshot contains `app.env`, so it holds `MW_SERVER_KEY` and the admin password as well as the mail data cached on the server. Treat each one as a secret.
:::

When `backup` runs, the steps happen in this order: snapshot, encryption (if `BACKUP_RECIPIENT` is set), copies to `BACKUP_TARGETS_FILE` destinations, `BACKUP_HOOK`, then retention (if `BACKUP_KEEP` or `BACKUP_KEEP_DAYS` is set). A failure at any step stops the steps after it and keeps the local backup.

## Retention: `prune`

```sh
bash deploy.sh prune                    # apply the retention settings below
bash deploy.sh prune --replaced         # the same, and list what --yes would delete
bash deploy.sh prune --replaced --yes   # also delete replaced volumes and old images
```

| Key in `deploy.env` | What it does | Default |
|---|---|---|
| `BACKUP_KEEP` | Number of complete snapshots to keep, newest first (1 to 9999). | `prune` uses 7 when empty |
| `BACKUP_KEEP_DAYS` | When set, a snapshot outside the newest `BACKUP_KEEP` is removed only if it is also older than this many days (1 to 99999). | empty |

A complete snapshot is removed only when both rules allow it. When either key is set, every successful `backup` runs the same retention afterwards; with both empty, snapshots are removed only by an explicit `prune`. The keys are read from `deploy.env` as saved; no deploy is needed after changing them.

`prune` never removes:

- the snapshot named in `rollback.path` (the one `rollback` restores, taken just before the last deploy);
- an incomplete snapshot (no `COMPLETE` marker, for example from an interrupted backup) until it is older than seven days;
- anything in `backups/` whose name is not a snapshot name.

The age of a snapshot is read from its name, so copying the directory elsewhere and back does not change it.

### Replaced volumes and old images

`restore` and `rollback` start the restored data in a new volume and keep the volume they replaced; a failed update's volume and the original SQLite volume after a full-stack migration are kept too. Each deploy from source leaves one image named `mailwoman-deploy:<project>-<time>-<pid>`. `prune` lists, with sizes:

- every data or PostgreSQL volume labelled `mailwoman.deploy.project=<project>` except the active ones and any volume a container still uses;
- every image `deploy.sh` built for this project that neither the active release nor a snapshot kept in `backups/` uses. A snapshot can be restored only with its image, so those images are not listed.

They are deleted only with `prune --replaced --yes`, and each one is checked again just before deletion. Images pulled with `--image` and the Caddy image are never listed; remove those yourself with `docker image rm`. Save an image with `docker image save` first if an older snapshot kept elsewhere may need it.

## Encryption: `BACKUP_RECIPIENT`

With `BACKUP_RECIPIENT` set in `deploy.env`, `backup` writes the snapshot as one encrypted archive, `backups/<name>.tar.age` or `backups/<name>.tar.gpg`, and removes the clear-text directory. The archive holds the same files as a snapshot directory, `app.env` included. Beside it, `<name>.release.env` keeps the image IDs in clear text (no secrets), so `prune` knows which images the archive needs. Only the public key is used: the host does not need a key that can decrypt its own backups, and should not have one.

| `BACKUP_RECIPIENT` | Tool | Restore with |
|---|---|---|
| an age recipient (`age1…`) or an SSH public key (`ssh-ed25519 AAAA…` or `ssh-rsa AAAA…`, without a comment) | [age](https://age-encryption.org) | `bash deploy.sh restore backups/<name>.tar.age --identity /path/to/key` |
| a full OpenPGP key fingerprint (40 or 64 hex digits) | `gpg --encrypt --recipient` | `bash deploy.sh restore backups/<name>.tar.gpg`, as a user whose gpg keyring holds the private key |

For gpg, import only the public key into the keyring of the user that runs `deploy.sh` (`gpg --import backup-key.pub.asc`). A full fingerprint is required so that exactly one key can match. `doctor` reports which tool is used, fails when the tool is not installed or the gpg key is missing, and notes when the host also holds the private key. Password-based encryption is not offered.

Restoring an archive decrypts it into a private directory under the state directory, checks it exactly like a snapshot directory, and removes the decrypted copy when the command ends. Without the private key (no `--identity` for age, or no secret key in the gpg keyring) nothing is changed.

### What stays in clear text

- **The snapshot `deploy` takes before an update.** Automatic rollback must restore it unattended, so it is not encrypted. With `BACKUP_RECIPIENT` set, each successful `deploy` removes the clear-text snapshot the previous deploy took, so one such snapshot is on the host at a time.
- **The snapshots `restore`, `rollback` and `maintenance --apply` take.** The `restore` and `rollback` ones are the only copy of the replaced installation's last data, so they are not removed automatically. Move them off the host, or delete them when you do not need them; retention also removes them once they fall outside `BACKUP_KEEP`.
- **A snapshot whose encryption failed.** `backup` exits non-zero and leaves the clear-text snapshot in place, so the backup is not lost.

## Off-host copies: `BACKUP_HOOK`

`BACKUP_HOOK` in `deploy.env` is the absolute path of an executable that `backup` runs after each completed backup, with one argument: the path of the archive (`backups/<name>.tar.age` or `.tar.gpg`) or, without encryption, of the snapshot directory. It runs as the user that runs `deploy.sh`, with no input. The path may contain only letters, digits and `._/+@-`.

When the hook exits non-zero, `backup` says so, exits non-zero, keeps the local copy and skips retention. A hook that succeeds does not remove the local copy either; retention decides that. `doctor` checks that the hook is an executable file.

Whatever the hook copies to needs the same protection as the host. With `BACKUP_RECIPIENT` set, only ciphertext leaves the host. Examples:

```sh
#!/bin/sh
# rsync over SSH to a backup host (the archive, or the snapshot directory)
exec rsync -a --chmod=go-rwx "$1" backup@backup.example.org:/srv/mailwoman/
```

```sh
#!/bin/sh
# rclone to any remote it supports
exec rclone copy "$1" offsite:mailwoman-backups/
```

```sh
#!/bin/sh
# restic, which deduplicates and encrypts its repository
export RESTIC_REPOSITORY=sftp:backup@backup.example.org:/srv/restic RESTIC_PASSWORD_FILE="$HOME/.restic-password"
exec restic backup --quiet "$1"
```

## Destinations: `BACKUP_TARGETS_FILE`

`BACKUP_TARGETS_FILE` in `deploy.env` is the absolute path of a JSON file listing destinations that `backup` copies each encrypted archive to and verifies. It requires:

- `BACKUP_RECIPIENT`: only encrypted archives are copied;
- `python3` on the host, which runs `scripts/backup-destinations.py`;
- `rclone` for every type except `local` and `nfs`.

| `type` | Where it copies | Connection settings |
|---|---|---|
| `local`, `nfs` | An absolute directory path (`path`), for example an NFS mount. | none |
| `s3`, `sftp`, `webdav`, `smb` | A relative `path` on a named rclone remote. | `remote` and `rcloneConfig` (absolute path of your rclone configuration). WebDAV URLs and S3 endpoints must use HTTPS. |
| `nextcloud` | A relative folder (`path`), created when missing. | `url` (HTTPS), `user` and `appPasswordFile` (an absolute path to a `chmod 600` file with a Nextcloud app password, not the login password). |

Every destination also takes `id` (unique), `enabled` (default `true`), `timeoutSeconds` (10 to 7200, default 900) and optionally `keep` and `keepDays`, which apply the `BACKUP_KEEP` / `BACKUP_KEEP_DAYS` rules to that destination's copies. The file holds `"version": 1` and 1 to 16 destinations:

```json
{
  "version": 1,
  "destinations": [
    {"id": "nextcloud-copy", "type": "nextcloud", "url": "https://cloud.example.com",
     "user": "mailwoman-backup", "appPasswordFile": "/etc/mailwoman/nextcloud-app-password",
     "path": "Backups/Mailwoman", "keep": 7, "enabled": true}
  ]
}
```

For each destination the helper copies the archive and its `.release.env`, downloads them again to check size and SHA-256, and only then writes a `<archive>.complete.json` manifest. A Nextcloud destination uses rclone's WebDAV route with chunked uploads (`chunkSizeMiB`, default 10, `0` turns chunking off).

A failing destination does not stop the others, but `backup` then exits non-zero ("Destination copy failed; local backup retained and retention skipped") and `BACKUP_HOOK` does not run. `doctor` and the start of each `backup` check the file offline; connections are tested only when copying.

To test the connections, or to fetch a backup back before a restore:

```sh
python3 scripts/backup-destinations.py test --config /etc/mailwoman/backup-targets.json
python3 scripts/backup-destinations.py fetch --config /etc/mailwoman/backup-targets.json \
    --destination nextcloud-copy --name <name>.tar.age --output /srv/restore
```

`fetch` verifies the files against the completion manifest. See [Configuration transfer and backup destinations](../admin/configuration-transfer.md#send-settings-backups-to-nextcloud) for more on each type.

## Scheduling: `timers`

```sh
bash deploy.sh timers
```

writes four systemd units for this installation into `.deploy.local/systemd/` and prints them, followed by the equivalent cron lines:

| Unit | Runs | When |
|---|---|---|
| `mailwoman-<project>-backup.service` / `.timer` | `deploy.sh backup` | daily at 03:30, delayed at random by up to 30 minutes; a run missed while the host was off runs at the next boot |
| `mailwoman-<project>-check.service` / `.timer` | `deploy.sh check` ([Monitoring](monitoring.md)) | every five minutes |

The units name this checkout's `deploy.sh`, its state directory and the user who ran `timers` (`User=`, left out for root), so they work only for this installation. Nothing is written outside the state directory. To install them, as root:

```sh
cp .deploy.local/systemd/mailwoman-<project>-*.service .deploy.local/systemd/mailwoman-<project>-*.timer /etc/systemd/system/
systemctl daemon-reload
systemctl enable --now mailwoman-<project>-backup.timer mailwoman-<project>-check.timer
```

Or add the two printed cron lines with `crontab -e` as that user. The paths may contain only letters, digits and `._/+@-`, so that neither systemd nor cron needs quoting; `timers` refuses other paths. Run `timers` again after moving the checkout or the state directory. With gpg encryption, the scheduled backup uses the keyring of the `User=` account, so import the public key there.

## Restores and erasure requests

A restored snapshot contains everyone who was in the data when it was taken, including people erased since. `deploy.sh restore` does not replay those erasures. Register `deploy.sh` snapshots in the admin console so they are marked when they contain erased people, and after a restore replay the newer erasures by hand; see [Erasure coverage](../admin/erasure-coverage.md).

## Backups without deploy.sh

For a server you run yourself (systemd, your own Compose file), the server has its own commands. Run them with the same `MW_DB_PATH` and `MW_SERVER_KEY` as the server.

| Command | What it does |
|---|---|
| `mailwoman erasure backup --out FILE [--label NAME]` | Writes a consistent copy of the SQLite database and `FILE.manifest.json` with the erasure watermark. |
| `mailwoman erasure restore --backup FILE --to NEWFILE [--live-db DB \| --tombstones FILE] [--no-propagate]` | Restores to a new file and replays erasures made since the backup. |
| `mailwoman db backup --to FILE [--plaintext]` | Writes an encrypted copy of an encrypted SQLite database; a plaintext copy needs `--plaintext`. See [Encryption](../admin/encryption.md). |
| `mailwoman db restore --from FILE --to NEWFILE` | Restores such a copy to a new file, re-keyed to the current key. |

These commands copy the database only: back up the upload directory too when `MW_UPLOAD_DIR` is set, and keep `MW_SERVER_KEY`. The Helm chart has no backup job; back up its `/data` volume with your cluster's tools.

## Related

- [Deploy with deploy.sh](script.md)
- [Monitoring](monitoring.md)
- [Configuration transfer and backup destinations](../admin/configuration-transfer.md)
- [Erasure coverage](../admin/erasure-coverage.md)
