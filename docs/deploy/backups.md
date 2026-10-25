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
