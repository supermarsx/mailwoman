# Configuration transfer and backup destinations

Admin → Configuration transfer exports and imports **registered global setting policies**. Exports include stored values, locks and user-override rules; they exclude domain definitions, account preferences, environment variables, credentials and dedicated integration records. Use a full deployment backup for recovery of the database and its configuration.

Imports first validate every entry and show a before/after preview. Applying requires the configuration-import permission and a fresh authentication proof. The preview records revisions: changed settings must be reviewed again. Imports update listed keys only. Writes are individually audited and protected against concurrent changes, but the whole bundle is **not a transaction**; if a concurrent change interrupts application, the result identifies applied, failed and unattempted entries. A server error can occur after the current key was committed: the response marks that outcome uncertain, and the UI requires a fresh review rather than retrying automatically. The last-superadmin-loss setting cannot be imported.

## Settings backups to Nextcloud

Admin → Configuration & backups → Backup destinations → **Settings backups to Nextcloud** sends the same portable export to a Nextcloud folder, from this server. It needs the configuration-export permission; saving or removing the destination also needs a fresh authentication proof.

| Field | Meaning |
|---|---|
| Nextcloud server URL | HTTPS only, for example `https://cloud.example.com` or `https://example.com/nextcloud`. The WebDAV address `…/remote.php/dav/files/<username>/<folder>/` is built from it. |
| Nextcloud username | The account whose files hold the backups. |
| App password | An app password made in Nextcloud under Personal settings → Security → Devices & sessions ([Nextcloud documentation](https://docs.nextcloud.com/server/latest/user_manual/en/session_management.html#managing-devices)), not the login password. |
| Target folder | A folder in that account, such as `Backups/Mailwoman`. Missing segments are created with `MKCOL`. |
| Copies to keep / Keep days | Retention with the meaning of `BACKUP_KEEP` and `BACKUP_KEEP_DAYS` (default 7 copies; days optional). |
| Additional CA certificates | Only for a server certificate signed by a private CA; it is the only extra trust added. |

The app password is sealed with the server key (XChaCha20-Poly1305, bound to the server URL and username). The API accepts it but never returns it; reads answer only whether one is saved, and changing the URL or username drops it until it is entered again. It is not written to logs or audit records. This credential is separate from the account-isolated Nextcloud links users make in their own settings, which are never used for server backups.

**Test connection** signs in, opens (or creates) the folder, uploads a small probe file, downloads it to compare it, and deletes it. Each failure names its step and cause: rejected app password, unknown user, a path that is a file, a folder the account may not create, no write permission, quota exceeded, a redirect, a timeout, a connection or certificate failure, or an address the outbound-connection policy blocks.

**Back up now** uploads `mailwoman-settings-<UTC time>.json` (never overwriting a file), downloads it again and compares its size and SHA-256, and only then applies retention. Retention removes only files named `mailwoman-settings-<time>.json` in that folder, never the copy just written; a failed retention leaves extra copies and is reported with the result. The last result is shown on the page and each run is audited. The file has the export's format, so Import settings accepts it.

Every request uses the guarded outbound path for Nextcloud traffic: HTTPS with certificate verification, no redirects, the address policy that refuses private, loopback and metadata addresses, and the egress proxy routes an operator configured for the `nextcloud` class (a Nextcloud on a private network is reached through such a route). Files are limited to 8 MiB. One backup or test runs at a time. Backups run when an administrator starts them; there is no schedule.

This is a settings backup, not a recovery backup. For the database, uploads and server key, use the host-side deployment backups below, which can also go to Nextcloud.

## Destination setup

The Backup destinations tab downloads a disabled JSON template. This is host configuration, not a web-server file writer or remote scheduler. Enable selected destinations in an operator-owned file, and set these entries in `.deploy.local/deploy.env`:

```dotenv
BACKUP_TARGETS_FILE=/etc/mailwoman/backup-targets.json
BACKUP_RECIPIENT=age1YOUR_PUBLIC_RECIPIENT
```

Example:

```json
{
  "version": 1,
  "destinations": [
    {"id": "local-copy", "type": "local", "path": "/srv/mailwoman-backups", "enabled": true},
    {"id": "object-store", "type": "s3", "remote": "backup-s3", "path": "my-bucket/mailwoman", "rcloneConfig": "/etc/mailwoman/rclone.conf", "enabled": false},
    {"id": "ssh-copy", "type": "sftp", "remote": "backup-ssh", "path": "mailwoman", "rcloneConfig": "/etc/mailwoman/rclone.conf", "enabled": false},
    {"id": "dav-copy", "type": "webdav", "remote": "backup-dav", "path": "mailwoman", "rcloneConfig": "/etc/mailwoman/rclone.conf", "enabled": false},
    {"id": "nextcloud-copy", "type": "nextcloud", "url": "https://cloud.example.com", "user": "mailwoman-backup", "appPasswordFile": "/etc/mailwoman/nextcloud-app-password", "path": "Backups/Mailwoman", "keep": 7, "enabled": false},
    {"id": "smb-copy", "type": "smb", "remote": "backup-smb", "path": "backups/mailwoman", "rcloneConfig": "/etc/mailwoman/rclone.conf", "enabled": false},
    {"id": "nfs-copy", "type": "nfs", "path": "/mnt/backups/mailwoman", "enabled": false}
  ]
}
```

Install Python 3 and, for S3/SFTP/WebDAV/Nextcloud/SMB, rclone on the deployment host. Configure named remotes using `rclone config`; keep that file private to the deployment account. The helper expects a readable, unencrypted rclone configuration file, while stored backup archives must be encrypted. Remote credentials are not included in the destination JSON. `timeoutSeconds` bounds each transfer command (default 900; allowed 10–7200).

Local and NFS directories must already exist. NFS requires `findmnt` and a detected NFS mount; the helper refuses a directory that has fallen back to local storage. Operators remain responsible for maintaining mounts during transfers. WebDAV requires HTTPS. SFTP requires a pinned `known_hosts_file`; automatic acceptance of new host keys is refused.

### Nextcloud

A `nextcloud` destination is a preset over the WebDAV route and needs no rclone remote. `url` is the HTTPS server URL (no credentials, query or `remote.php` path), `user` the Nextcloud username, `path` the target folder in that account, and `appPasswordFile` an absolute path to a file holding one [app password](https://docs.nextcloud.com/server/latest/user_manual/en/session_management.html#managing-devices), readable only by the deployment account (`chmod 600`; the helper refuses a group- or world-readable file). For each run the helper checks the credentials, creates missing folder segments with `MKCOL`, and writes a private temporary rclone remote (`vendor = nextcloud`, password obscured, file mode 0600) that is removed afterwards. Large archives are uploaded in chunks through Nextcloud's chunked-upload endpoint; `chunkSizeMiB` sets the size (default 10, `0` disables chunking). Requests never follow redirects. A private CA is trusted through `RCLONE_CA_CERT`, as for the other remotes.

```sh
install -m 600 /dev/null /etc/mailwoman/nextcloud-app-password
printf '%s\n' 'APP-PASSWORD' > /etc/mailwoman/nextcloud-app-password
python3 scripts/backup-destinations.py test --config /etc/mailwoman/backup-targets.json --destination nextcloud-copy
```

`test` signs in, opens or creates the folder, writes, verifies and deletes a probe file, and reports a rejected app password, a missing user and a folder that cannot be created as separate errors.

For SSH tunnelling/jump hosts, configure the SFTP remote's external `ssh` command with `ssh -o StrictHostKeyChecking=yes -o BatchMode=yes` and an SSH host alias with `ProxyJump`. Pin host keys for both the destination and jump host in the operator's SSH configuration. External SSH uses OpenSSH authentication and host configuration, rather than rclone's internal SFTP settings; see the [rclone SFTP documentation](https://rclone.org/sftp/). SMB paths include the share name; see [rclone SMB](https://rclone.org/smb/).

```sh
bash deploy.sh doctor
bash deploy.sh backup
```

Doctor checks configuration and installed prerequisites, not provider connectivity; `python3 scripts/backup-destinations.py test --config FILE [--destination ID]` checks connectivity and write access with a probe file it deletes again. Backup performs the existing snapshot and encryption, copies the encrypted archive and release metadata, then verifies file sizes and SHA-256 hashes. It publishes the completion manifest last. Remote copies are downloaded for verification, so plan for that bandwidth. Existing conflicting files are not intentionally overwritten. A failed copy retains the local backup and skips local retention. Other enabled destinations are still attempted for ordinary transfer failures.

Existing deployment timers invoke the same backup command. Local retention remains controlled by `BACKUP_KEEP` and `BACKUP_KEEP_DAYS`. Remote retention is off unless a destination sets `keep` and/or `keepDays`, with the same meaning (a copy outside the newest `keep`, default 7, is removed only when it is also older than `keepDays`). It runs after a verified copy and removes only complete sets (archive, release metadata and completion manifest; the manifest first) whose names are deploy.sh snapshot names; archives without a manifest, other files and the archive just published are kept. A failed retention is reported and leaves extra copies. Without these keys, configure storage lifecycle rules separately, preserving archive, release metadata and completion manifest together. Deployment images and decryption keys must remain available for recovery.

## Retrieve and restore

Fetch checks the completion manifest and verifies both files before publishing them into an existing local recovery directory:

```sh
python3 scripts/backup-destinations.py fetch \
  --config /etc/mailwoman/backup-targets.json \
  --destination object-store \
  --name SNAPSHOT.tar.age \
  --output /srv/mailwoman-recovery
```

Fetch does not modify running services. Use the existing `deploy.sh restore` command with the retrieved archive and the appropriate decryption identity when intentionally restoring. Keep the adjacent `.release.env` file. SHA-256 detects transfer corruption; trust in the completion manifest depends on access control of the destination. Test recovery before relying on a backup policy.

## Verification

`python3 scripts/test_backup_destinations.py` checks local copies, manifests, bounds and failure handling. `bash scripts/test-deploy-backup-targets.sh` exercises deployment ordering and retention on failure. Optional transport tests run disposable loopback servers and never contact configured storage accounts:

```sh
python3 scripts/test-backup-transports.py --rclone /absolute/path/to/rclone --method s3
python3 scripts/test-backup-transports.py --rclone /absolute/path/to/rclone --method webdav
python3 scripts/test-backup-transports.py --rclone /absolute/path/to/rclone --method sftp
```

A live Nextcloud check runs against a disposable account (for example `nextcloud:stable` with an app password from `occ user:auth-tokens:add`) and covers folder creation, publication, chunked upload, fetch, tamper refusal, pruning, a wrong app password, an uncreatable folder and failure isolation:

```sh
RCLONE_CA_CERT=/path/to/test-ca.pem python3 scripts/test-backup-nextcloud.py \
  --url https://cloud.test/ --user backup --app-password-file /path/to/app-password --readonly-folder ReadOnlyShare
```

These require rclone with the relevant `serve` command, OpenSSL, and `ssh-keygen` for SFTP. They check publication, retrieval and tamper rejection; SFTP also checks host-key mismatch rejection. They do not replace acceptance against the operator's storage provider, SMB server, NFS mount or SSH jump-host configuration.

### Backup helper configuration boundaries

Nonempty custom S3 endpoints must use HTTPS; an empty endpoint keeps the standard provider endpoint. The selected rclone remote may not contain `global.*` or `override.*` options. Runtime `RCLONE_*` environment overrides are refused, except `RCLONE_CA_CERT` for explicit private-CA trust. Put credentials and remote settings in the operator-owned configuration file. HTTP(S) proxy environment routing remains available. Refusal diagnostics name options or environment variables but do not print their values.

External SSH commands must put `StrictHostKeyChecking=yes` and `BatchMode=yes` in leading `-o` options, followed by one destination. Extra remote-command arguments are refused; rclone supplies its own SFTP subsystem command. Jump-host trust remains the operator's separately pinned SSH configuration responsibility.
