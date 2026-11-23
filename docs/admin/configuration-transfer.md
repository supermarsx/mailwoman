---
title: Configuration & backups
screen: true
---
# Configuration & backups

**Configuration & backups** (under **Configuration** in the console navigation) moves the administrator's global settings between installations and sends a copy of them to Nextcloud. It also lists the destinations for full deployment backups, which you set up on the deployment host. The portable file holds the stored global setting values, their locks and the rules for user overrides. It does not hold domains, user data, credentials, environment variables or the records of integrations such as hosting panels. For recovering a deployment use a full backup.

![Configuration and backups with the Export settings tab open: a note on what portable files contain, three tabs, and the Download settings button.](/screenshots/admin-ops/opsa-transfer-export.webp){.mw-shot}

## Before you start

- Opening the page needs `config.read`. **Download settings** and the Nextcloud backup need `config.export`. **Import settings** needs `config.import`.
- Applying an import, and saving or removing the Nextcloud destination, ask for a recent confirmation that it is you (your password and second factor, or a passkey).

## Export settings

1. Open the **Export settings** tab.
2. Choose **Download settings**. Your browser saves `mailwoman-settings.json`.
3. The page reports how many policies were exported and how many dedicated policies were left out (**Exported 12 policies; 3 dedicated policies were excluded.**).

The file is a versioned JSON document (`"format": "mailwoman-settings"`, `"version": 1`) with one entry per stored setting: `key`, `globalValue`, `locked` and `allowUserOverride`. A setting that was never stored is not in it, so it stays unset on the target. Settings that have their own security confirmation, and the last-administrator safeguard, are left out. At most 1,024 stored policies can be exported at once.

## Import settings

1. Open the **Import settings** tab.
2. Under **Settings JSON file**, choose a file of less than 2 MiB. The server validates every entry before anything is changed; a file that does not validate is refused with its reason.
3. Read **Review settings changes**. For each setting it shows the **Setting**, the **Current policy** and the **Imported policy**. Keys that are not in the file are left unchanged.
4. Choose **Apply reviewed changes**, read the confirmation and confirm. You are asked to confirm that it is you (your password and second factor, or a passkey).

An import is not one transaction. Each setting is written and audited on its own, with a check of its current revision, so a concurrent change can stop the import after earlier entries were applied. The result then says **Applied 5 policies, then stopped at <key> (HTTP 409). Reload and review the file again before continuing.** If the server fails after it wrote the current setting, the result says that outcome is uncertain and asks you to review a fresh preview instead of repeating it. The setting that allows losing the last administrator cannot be imported.

## Send settings backups to Nextcloud

![The Settings backups to Nextcloud form: server URL, username, app password, target folder, copies to keep, keep days and private certificate authority, with Save destination, Test connection and Back up now.](/screenshots/admin-ops/opsa-transfer-nextcloud.webp){.mw-crop}

In **Backup destinations**, **Settings backups to Nextcloud** sends the same export from this server to a Nextcloud folder you own. It uses its own app password, separate from the accounts that users link in their settings, which are never used for backups.

1. Fill in the form.

   | Field | Meaning |
   |---|---|
   | **Nextcloud server URL** | HTTPS only, for example `https://cloud.example.com` or `https://example.com/nextcloud`. The WebDAV path is added. |
   | **Nextcloud username** | The account whose files hold the backups. |
   | **App password** | An app password made in Nextcloud under **Personal settings → Security → Devices & sessions**, not the login password. |
   | **Target folder** | A folder in that account, for example `Backups/Mailwoman`. Missing folders are created. |
   | **Copies to keep** | How many copies stay; 7 for a new destination. |
   | **Keep days (optional)** | A copy outside the newest ones is removed only when it is also older than this many days. |
   | **Private certificate authority** | A disclosure that opens **Additional CA certificates (PEM)**, only for a server whose certificate a private CA signed. |

2. Choose **Save destination**. The app password is sealed with the server key and never shown again: the page says **An app password is saved and cannot be shown. Enter a new one only to replace it.** Changing the URL or the username drops the saved password until you enter it again.
3. Choose **Test connection** (it stays unavailable until a destination is saved). It signs in, opens or creates the folder, writes a small test file, reads it back and deletes it. A failure names the step (**Configuration**, **Sign-in**, **Folder access**, **Upload**, **Verification**, **Deletion** or **Retention**) and the cause, for example **Nextcloud rejected the username or app password. Create a new app password and save it.**
4. Choose **Back up now** (also unavailable until a destination is saved). It uploads `mailwoman-settings-<UTC time>.json` without overwriting a file, downloads it again and compares its size and checksum, and only then applies the retention. Retention removes only files named `mailwoman-settings-<time>.json` in that folder, never the copy just written. If it fails, extra copies stay and the result says so.
5. The page shows the last result (**Last backup: …**). Each run is audited. Backups run when an administrator starts them; there is no schedule. The file has the format of an export, so **Import settings** accepts it.

Every request goes through the guarded outbound path for Nextcloud traffic: HTTPS with certificate verification, no redirects, and the address policy that refuses private, loopback and metadata addresses. A Nextcloud on a private network is reached through an egress proxy route for the Nextcloud class (see [Outbound connections](/deploy/egress-proxy)). One backup or test runs at a time.

A settings backup is not a recovery backup. It holds no database, uploads or server key.

## Full deployment backups

The lower half of **Backup destinations** lists **Supported backup destinations** for the backups that `deploy.sh backup` makes (the database, uploads, configuration and server key, encrypted). Destinations are configured on the deployment host, not in the console: the web server gets no access to the host's files or to Docker.

| Method | What you need |
|---|---|
| **Local directory** | An existing absolute directory on the host. |
| **S3-compatible object storage** | A named S3 rclone remote with bucket and path. An endpoint that is set must use HTTPS. |
| **SFTP / SSH tunnel** | A named SFTP remote with a pinned `known_hosts_file`; accepting new host keys automatically is refused. |
| **WebDAV** | A named WebDAV remote over HTTPS. |
| **Nextcloud** | The server URL, a user, a file with an app password and a target folder. It uses the WebDAV route with chunked uploads. |
| **SMB / CIFS** | A named SMB remote with the share and path. |
| **NFS** | An existing NFS mount. A directory that is not an NFS mount is refused instead of writing to the local disk. |

1. Choose **Download destination template**. It is a JSON file with every destination disabled; downloading it changes nothing.
2. Edit it on the host: enable the destinations you want and fill in their fields. Save it where only the deployment account can read it.
3. In `.deploy.local/deploy.env`, set:

   ```dotenv
   BACKUP_TARGETS_FILE=/etc/mailwoman/backup-targets.json
   BACKUP_RECIPIENT=age1YOUR_PUBLIC_RECIPIENT
   ```

4. Install Python 3 and, for the S3, SFTP, WebDAV, Nextcloud and SMB methods, rclone. Configure the named remotes with `rclone config` in an unencrypted file that only the deployment account can read. Remote credentials are not in the template.
5. Check and test:

   ```sh
   bash deploy.sh doctor
   python3 scripts/backup-destinations.py test --config /etc/mailwoman/backup-targets.json --destination object-store
   bash deploy.sh backup
   ```

   `doctor` checks the configuration and the installed tools, not the connection. `test` signs in, writes a test file, checks it and deletes it.

An example template:

```json
{
  "version": 1,
  "destinations": [
    {"id": "local-copy", "type": "local", "path": "/srv/mailwoman-backups", "enabled": true},
    {"id": "object-store", "type": "s3", "remote": "backup-s3", "path": "my-bucket/mailwoman", "rcloneConfig": "/etc/mailwoman/rclone.conf", "enabled": false},
    {"id": "ssh-copy", "type": "sftp", "remote": "backup-ssh", "path": "mailwoman", "rcloneConfig": "/etc/mailwoman/rclone.conf", "enabled": false},
    {"id": "nextcloud-copy", "type": "nextcloud", "url": "https://cloud.example.com", "user": "mailwoman-backup", "appPasswordFile": "/etc/mailwoman/nextcloud-app-password", "path": "Backups/Mailwoman", "keep": 7, "enabled": false},
    {"id": "nfs-copy", "type": "nfs", "path": "/mnt/backups/mailwoman", "enabled": false}
  ]
}
```

Local and NFS directories must exist. `timeoutSeconds` limits each transfer (900 by default). `keep` and `keepDays` set a remote retention with the same meaning as `BACKUP_KEEP` and `BACKUP_KEEP_DAYS`; without them the destination keeps everything and local retention is not affected. For a `nextcloud` destination, `chunkSizeMiB` sets the upload chunk size (10 by default, 0 turns chunking off), and `appPasswordFile` must be an absolute path to a regular file that only its owner can read (`chmod 600`).

`deploy.sh backup` makes the snapshot and the encryption, copies the encrypted archive and its release metadata to every enabled destination, checks the sizes and SHA-256 hashes by downloading the copies, and writes the completion manifest last. Plan for the bandwidth of those downloads. A file that already exists is not overwritten on purpose. If a copy fails, the local backup is kept and local retention is skipped; other destinations are still tried for ordinary transfer failures. The same command runs from the timers described in [Backups](/deploy/backups).

The helper refuses an rclone remote that sets `global.*` or `override.*` options, and it refuses `RCLONE_*` environment variables except `RCLONE_CA_CERT`, which adds a private CA. A refusal names the option, not its value.

### Retrieve and restore

```sh
python3 scripts/backup-destinations.py fetch \
  --config /etc/mailwoman/backup-targets.json \
  --destination object-store \
  --name SNAPSHOT.tar.age \
  --output /srv/mailwoman-recovery
```

`fetch` verifies the completion manifest and both files before it puts them in an existing local directory. It does not touch the running services. To restore, run `deploy.sh restore` with the retrieved archive and its decryption identity, and keep the `.release.env` file next to it. Test a restore before you rely on a backup policy: a checksum finds a damaged transfer, but trust in the manifest depends on who can write to the destination.

## Options

| Label | What it does | Default | Who can change it |
|---|---|---|---|
| **Copies to keep** (Nextcloud settings backup) | Copies of the settings file that stay. | 7 | Administrators with `config.export` |
| **Keep days (optional)** | Age after which a copy outside the newest ones may be removed. | None | Same |
| `BACKUP_TARGETS_FILE`, `BACKUP_RECIPIENT` | Destination file and age recipient for full backups, in `deploy.env`. | Not set | Whoever administers the host |

## Troubleshooting

- **"Choose a settings file smaller than 2 MiB."** The file is not a settings export, or it is too large.
- **The import preview is empty.** Every entry already equals the current policy.
- **"Applied N policies, then stopped at … (HTTP …)."** Another change interrupted the import. Reload, review the file again and apply it again.
- **Test connection: "Nextcloud redirected the request (HTTP …). Enter the final HTTPS server URL."** The URL is an old or alternative address.
- **Test connection: "The outbound-connection policy blocks this address. A server on a private network needs an egress proxy route for Nextcloud traffic."** See [Outbound connections](/deploy/egress-proxy).
- **Test connection: "The folder is missing and this account may not create it (HTTP …)."** Create the folder in Nextcloud or pick another one.
- **Test connection: "The Nextcloud account is out of storage space."** Free space or raise the quota.
- **`deploy.sh doctor` passes, but a destination fails during a backup.** Doctor does not test connectivity. Run `scripts/backup-destinations.py test` for that destination.
- **`Use the configured rclone file; only RCLONE_CA_CERT is allowed.`** An `RCLONE_*` environment variable is set. Remove it and put the setting in the rclone file.

## Related

- [Settings reference](/admin/settings)
- [Backups](/deploy/backups)
- [Outbound connections](/deploy/egress-proxy)
- [Nextcloud](/admin/nextcloud)
