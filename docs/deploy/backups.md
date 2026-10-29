# Backups with deploy.sh

`bash deploy.sh backup` stops the app for the length of the copy and writes a
snapshot of the whole `/data` volume (SQLite database, WAL files, uploads), the
applied configuration and the exact image IDs to `.deploy.local/backups/` (see
[script.md](script.md#backups-and-recovery) for what a snapshot holds and how
`restore` and `rollback` use it). This page covers what happens to snapshots
afterwards.

Every snapshot contains `app.env`, so it holds `MW_SERVER_KEY` and the admin
password in addition to the mail data cached on the server. Treat each one as a
secret.

## Retention: `prune`

```sh
bash deploy.sh prune                    # apply the retention settings below
bash deploy.sh prune --replaced         # the same, and say what --yes would delete
bash deploy.sh prune --replaced --yes   # also delete replaced volumes and old images
```

Two keys in `deploy.env` set how many snapshots stay:

| Key | Meaning |
|---|---|
| `BACKUP_KEEP` | Number of complete snapshots to keep, newest first. `prune` uses 7 when it is empty. |
| `BACKUP_KEEP_DAYS` | When set, a snapshot outside the newest `BACKUP_KEEP` is removed only if it is also older than this many days. |

A complete snapshot is removed only when both rules allow it. When either key
is set, every successful `backup` runs the same retention afterwards; with both
empty, snapshots are removed only by an explicit `prune`. These keys are read
from `deploy.env` as it is saved; no deploy is needed after changing them.

`prune` never removes:

- the snapshot named in `rollback.path` (the one `rollback` restores, taken just
  before the last deploy);
- an incomplete snapshot (no `COMPLETE` marker, for example from an interrupted
  backup) until it is older than seven days;
- anything in `backups/` whose name is not a snapshot name
  (`YYYYMMDDTHHMMSSZ-<pid>-<random>`).

The age of a snapshot is read from its name, which is the UTC time it was
taken, so copying the directory elsewhere and back does not change it.

### Replaced volumes and old images

`restore` and `rollback` start the restored data in a new volume and keep the
volume they replaced; a failed update's volume is kept too. Deploys from source
leave one `mailwoman-deploy:<project>-<time>-<pid>` image each. `prune` lists
both kinds with their sizes:

- every data volume of this project (label `mailwoman.deploy.project`) except
  the active one and any volume a container still uses;
- every image `deploy.sh` built for this project that neither the active
  release nor a snapshot kept in `backups/` uses. A snapshot can be restored
  only with its image, so those images are not listed.

They are deleted only with `prune --replaced --yes`. Each one is checked again
just before it is deleted. Images pulled with `--image` and the Caddy image are
never listed; remove those with `docker image rm` yourself.

## Encryption: `BACKUP_RECIPIENT`

With `BACKUP_RECIPIENT` set in `deploy.env`, `backup` writes the snapshot as one
encrypted archive, `backups/<name>.tar.age` or `backups/<name>.tar.gpg`, and
removes the clear-text directory. The archive holds the same files as a snapshot
directory, `app.env` included. Beside it, `<name>.release.env` keeps the image
IDs in the clear (no secrets), so `prune` knows which images the archive needs.
Only the public key is used: the host needs no key that can decrypt its own
backups, and should not have one.

| `BACKUP_RECIPIENT` | Tool | Restore with |
|---|---|---|
| `age1...` (an age recipient) or `ssh-ed25519 AAAA...` / `ssh-rsa AAAA...` (an SSH public key, without a comment) | [age](https://age-encryption.org) | `bash deploy.sh restore backups/<name>.tar.age --identity /path/to/key` |
| a full OpenPGP key fingerprint (40 hex digits) | `gpg --encrypt --recipient` | `bash deploy.sh restore backups/<name>.tar.gpg`, as a user whose gpg keyring holds the private key |

For gpg, import only the public key into the keyring of the user that runs
`deploy.sh` (`gpg --import backup-key.pub.asc`); a fingerprint is required so
that exactly one key can match. `doctor` reports which tool is used, fails when
it is not installed or the gpg key is missing, and notes when the host holds the
private key as well. A password-based `openssl enc` is not offered: it does not
authenticate the archive, and the password would have to sit beside the data.

Restoring an archive decrypts it into a private directory under the state
directory, checks it exactly as a snapshot directory is checked, and removes the
decrypted copy when the command ends. Without the private key (no `--identity`
for age, or no secret key in the gpg keyring) nothing is changed.

### What stays in clear text

- The snapshot `deploy` takes just before an update. Automatic rollback has to
  restore it unattended, so it is not encrypted. It is no more exposed than the
  live volume and `app.env` beside it. With `BACKUP_RECIPIENT` set, each
  successful `deploy` removes the clear-text snapshot the previous deploy took,
  so one such snapshot is on the host at a time.
- The snapshot `restore` and `rollback` take of the installation they replace.
  It is the only copy of that installation's last data, so it is not removed
  automatically; move it off the host, or delete it once it is no longer needed
  (retention also removes it once it falls outside `BACKUP_KEEP`).
- If encryption fails, `backup` exits non-zero and leaves the clear-text
  snapshot where it was, so the backup itself is not lost.

## Off-host copies: `BACKUP_HOOK`

`BACKUP_HOOK` in `deploy.env` is the absolute path of an executable that
`backup` runs after each completed backup, with one argument: the path of the
archive (`backups/<name>.tar.age` / `.tar.gpg`) or, without encryption, of the
snapshot directory. It runs as the user that runs `deploy.sh`, with no input.

When the hook exits non-zero, `backup` exits non-zero, says so, and keeps the
local copy; the retention that would follow the backup does not run. A hook that
succeeds does not remove the local copy either; retention decides that.

The backup contains `MW_SERVER_KEY` and the admin password (in `app.env`), so
whatever the hook copies to needs the same protection as the host. Encrypting
with `BACKUP_RECIPIENT` first means only ciphertext leaves the host. Examples:

```sh
#!/bin/sh
# rsync over SSH to a backup host (the archive, or the snapshot directory)
exec rsync -a --chmod=go-rwx "$1" backup@backup.example.org:/srv/mailwoman/
```

```sh
#!/bin/sh
# rclone to any remote it supports (S3, B2, SFTP, ...)
exec rclone copy "$1" offsite:mailwoman-backups/
```

```sh
#!/bin/sh
# restic, which deduplicates and encrypts the repository itself
export RESTIC_REPOSITORY=sftp:backup@backup.example.org:/srv/restic RESTIC_PASSWORD_FILE="$HOME/.restic-password"
exec restic backup --quiet "$1"
```

`doctor` checks that the hook is an executable file.

## Destinations, including Nextcloud: `BACKUP_TARGETS_FILE`

`BACKUP_TARGETS_FILE` in `deploy.env` names a JSON file of destinations that
`backup` copies each encrypted archive to and verifies: a local directory, S3,
SFTP, WebDAV, Nextcloud, SMB or NFS. A Nextcloud destination takes the server
URL, the username, a `chmod 600` file with an app password (not the login
password) and a target folder, which is created when missing:

```json
{"id": "nextcloud-copy", "type": "nextcloud", "url": "https://cloud.example.com",
 "user": "mailwoman-backup", "appPasswordFile": "/etc/mailwoman/nextcloud-app-password",
 "path": "Backups/Mailwoman", "keep": 7, "enabled": true}
```

It uses the same rclone WebDAV route as `webdav` (HTTPS only, chunked uploads
for large archives, download verification of size and SHA-256), and `keep` /
`keepDays` give it the retention of `BACKUP_KEEP` / `BACKUP_KEEP_DAYS`. A
failing destination does not stop the others and keeps the local backup. See
[configuration transfer and backup destinations](../admin/configuration-transfer.md#nextcloud)
for every field, `test`, and fetching a backup back.

## Scheduling: `timers`

```sh
bash deploy.sh timers
```

writes four systemd units for this installation into `.deploy.local/systemd/`
and prints them, followed by the equivalent cron lines:

| Unit | Runs | When |
|---|---|---|
| `mailwoman-<project>-backup.service` / `.timer` | `deploy.sh backup` | daily at 03:30 (up to 30 minutes later, at random); a run missed while the host was off runs at the next boot |
| `mailwoman-<project>-check.service` / `.timer` | `deploy.sh check` ([monitoring.md](monitoring.md)) | every five minutes |

The units name this checkout's `deploy.sh`, its state directory and the user who
ran `timers` (`User=`, left out for root), so they work only for this
installation. Nothing is installed or written outside the state directory; to
install, as root:

```sh
cp .deploy.local/systemd/mailwoman-<project>-*.service .deploy.local/systemd/mailwoman-<project>-*.timer /etc/systemd/system/
systemctl daemon-reload
systemctl enable --now mailwoman-<project>-backup.timer mailwoman-<project>-check.timer
```

Or add the two printed cron lines with `crontab -e` as that user. The paths may
contain only letters, digits and `._/+@-`, so that neither systemd nor cron needs
quoting; `timers` refuses others. Run `timers` again after moving the checkout or
the state directory. With gpg encryption, the scheduled backup uses the keyring
of the `User=` account, so import the public key there.
