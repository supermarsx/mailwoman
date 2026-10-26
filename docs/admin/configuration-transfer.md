# Configuration transfer and backup destinations

Admin → Configuration transfer exports and imports **registered global setting policies**. Exports include stored values, locks and user-override rules; they exclude domain definitions, account preferences, environment variables, credentials and dedicated integration records. Use a full deployment backup for recovery of the database and its configuration.

Imports first validate every entry and show a before/after preview. Applying requires the configuration-import permission and a fresh authentication proof. The preview records revisions: changed settings must be reviewed again. Imports update listed keys only. Writes are individually audited and protected against concurrent changes, but the whole bundle is **not a transaction**; if a concurrent change interrupts application, the result identifies applied, failed and unattempted entries. The last-superadmin-loss setting cannot be imported.

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
    {"id": "smb-copy", "type": "smb", "remote": "backup-smb", "path": "backups/mailwoman", "rcloneConfig": "/etc/mailwoman/rclone.conf", "enabled": false},
    {"id": "nfs-copy", "type": "nfs", "path": "/mnt/backups/mailwoman", "enabled": false}
  ]
}
```

Install Python 3 and, for S3/SFTP/WebDAV/SMB, rclone on the deployment host. Configure named remotes using `rclone config`; keep that file private to the deployment account. The helper expects a readable, unencrypted rclone configuration file, while stored backup archives must be encrypted. Remote credentials are not included in the destination JSON. `timeoutSeconds` bounds each transfer command (default 900; allowed 10–7200).

Local and NFS directories must already exist. NFS requires `findmnt` and a detected NFS mount; the helper refuses a directory that has fallen back to local storage. Operators remain responsible for maintaining mounts during transfers. WebDAV requires HTTPS. SFTP requires a pinned `known_hosts_file`; automatic acceptance of new host keys is refused.

For SSH tunnelling/jump hosts, configure the SFTP remote's external `ssh` command with `ssh -o StrictHostKeyChecking=yes -o BatchMode=yes` and an SSH host alias with `ProxyJump`. Pin host keys for both the destination and jump host in the operator's SSH configuration. External SSH uses OpenSSH authentication and host configuration, rather than rclone's internal SFTP settings; see the [rclone SFTP documentation](https://rclone.org/sftp/). SMB paths include the share name; see [rclone SMB](https://rclone.org/smb/).

```sh
bash deploy.sh doctor
bash deploy.sh backup
```

Doctor checks configuration and installed prerequisites, not provider connectivity. Backup performs the existing snapshot and encryption, copies the encrypted archive and release metadata, then verifies file sizes and SHA-256 hashes. It publishes the completion manifest last. Remote copies are downloaded for verification, so plan for that bandwidth. Existing conflicting files are not intentionally overwritten. A failed copy retains the local backup and skips local retention. Other enabled destinations are still attempted for ordinary transfer failures.

Existing deployment timers invoke the same backup command. Local retention remains controlled by `BACKUP_KEEP` and `BACKUP_KEEP_DAYS`; **remote retention is not implemented**. Configure storage lifecycle rules separately, preserving archive, release metadata and completion manifest together. Deployment images and decryption keys must remain available for recovery.

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

These require rclone with the relevant `serve` command, OpenSSL, and `ssh-keygen` for SFTP. They check publication, retrieval and tamper rejection; SFTP also checks host-key mismatch rejection. They do not replace acceptance against the operator's storage provider, SMB server, NFS mount or SSH jump-host configuration.
