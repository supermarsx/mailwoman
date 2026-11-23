# Command-line reference

`mailwoman` is one binary. Without a command it prints its help and exits with status 2.
`--help` works on every command and subcommand, and `--version` prints the build version.

Most commands that open the database take the same two options (`db` and `erasure` take only `--db-path`, and `migrate-store` takes neither; it has its own `--from` and `--to`):

| Option | Environment | Default | What it does |
|---|---|---|---|
| `--db-path <DSN>` | `MW_DB_PATH` | `mailwoman.db` | The database: a SQLite file path or a PostgreSQL DSN. `db` and `erasure` work on SQLite files only. |
| `--server-key <HEX>` | `MW_SERVER_KEY` | none | The server key, 64 hexadecimal characters. Without it the server reads `MW_SERVER_KEY_FILE`, and for SQLite uses a key file kept beside the database. |

A command that fails logs the error and exits with status 1. A command that
succeeds exits with 0. Options that read a secret from the environment
(`MW_OLD_PASSWORD`, `MW_NEW_PASSWORD`, `MW_MCP_TOKEN`) keep it out of the process list
and shell history; prefer them to the matching options. Every `MW_*` variable the
server reads is listed in the [environment reference](environment.md).

| Command | What it does |
|---|---|
| [`serve`](#serve) | Runs the HTTP or HTTPS server. |
| [`healthcheck`](#healthcheck) | Probes a running server. |
| [`doctor`](#doctor) | Prints the cache and sandbox posture. |
| [`admin`](#admin) | Manages console administrators, domains, users, policy and more. |
| [`db`](#db) | Encrypts, rekeys, backs up and cleans a SQLite database. |
| [`erasure`](#erasure) | Backs up and restores with erasure replay; scrubs logs. |
| [`maintenance`](#maintenance) | One-shot store maintenance. |
| [`migrate-store`](#migrate-store) | Copies a SQLite store into PostgreSQL. |
| [`plugin`](#plugin) | Lists and approves plugins. |
| [`password`](#password) | Changes an account password through the configured backend. |
| [`mcp-stdio`](#mcp-stdio) | Bridges standard input and output to a remote MCP endpoint. |
| [`fonts pull`](#fonts-pull) | Downloads Google Fonts for self-hosting. |

## serve

`mailwoman serve [OPTIONS]` runs the server until it receives a termination
signal, then drains open connections. It opens the database, applies its migrations,
and listens on `--bind`. Without TLS options it serves plain HTTP; run it behind a
TLS-terminating proxy or give it a certificate.

| Option | Environment | Default | What it does |
|---|---|---|---|
| `--bind <ADDR>` | `MW_BIND` | `0.0.0.0:8080` | Listen address. |
| `--db-path <DSN>` | `MW_DB_PATH` | `mailwoman.db` | The database. |
| `--server-key <HEX>` | `MW_SERVER_KEY` | none | The server key. If none is given it is persisted locally. |
| `--web-dir <DIR>` | `MW_WEB_DIR` | embedded assets | Serves the web app from this directory instead of the copy built into the binary. |
| `--cookie-secure` | `MW_COOKIE_SECURE` | off | Always mark cookies `Secure`. See the note below. |
| `--mode <MODE>` | `MW_MODE` | `proxy` | `engine` talks IMAP and POP3 directly; any other value runs in `proxy` (JMAP) mode. |
| `--acme <DOMAIN>[,<DOMAIN>…]` | `MW_ACME` | none | Gets and renews certificates from Let's Encrypt for these domains and serves HTTPS on `--bind`. Takes priority over `--tls-cert`. |
| `--acme-contact <EMAIL>` | `MW_ACME_CONTACT` | none | Contact address for the ACME account. |
| `--acme-cache <DIR>` | `MW_ACME_CACHE` | `acme-cache` | Where the ACME account and certificates are kept. |
| `--acme-staging` | `MW_ACME_STAGING` | off | Uses the Let's Encrypt staging directory. |
| `--tls-cert <FILE>` | `MW_TLS_CERT` | none | Certificate chain (PEM). Reloaded on `SIGHUP` on Unix systems; on Windows restart the server. |
| `--tls-key <FILE>` | `MW_TLS_KEY` | none | Private key (PEM). |
| `--coep` | `MW_COEP` | on | Sends `Cross-Origin-Embedder-Policy: require-corp`. |
| `--csrf-strict` | `MW_CSRF_STRICT` | off | Requires the double-submit CSRF token on every cookie-authorised write. |
| `--session-idle-secs <N>` | `MW_SESSION_IDLE_SECS` | `604800` | Idle session timeout, in seconds. |
| `--session-absolute-secs <N>` | `MW_SESSION_ABSOLUTE_SECS` | `0` | Absolute session lifetime, in seconds. `0` keeps sessions rolling. |

Notes:

- **Certificates.** `--tls-cert` and `--tls-key` take effect only together. With only
  one of them the server starts without TLS and logs nothing about the missing half.
  `--acme` wins over a certificate and key.
- **Secure cookies.** An explicit `MW_COOKIE_SECURE` of `true` or `false` forces the `Secure`
  attribute on or off. Without it, cookies are `Secure` when the request is HTTPS (judged
  from the scheme of `MW_PUBLIC_URL` or from trusted proxy headers) or when
  `--cookie-secure` is given.
- **Booleans.** `MW_COOKIE_SECURE`, `MW_CSRF_STRICT`, `MW_COEP` and `MW_ACME_STAGING` accept
  only the literal values `true` and `false`; anything else (`1`, `yes`, `on`) stops
  `serve` with an "invalid value" error. `--coep` is on by default and the flag cannot turn
  it off: set `MW_COEP=false` to omit the header.
- **Sessions.** The same limits can be set under the settings `session.idleSecs` and
  `session.absoluteSecs`; the shorter limit applies. A non-zero absolute lifetime is capped
  at 15 days.
- **Everything else** the server reads comes from environment variables and stored
  settings, not from options: see the [environment reference](environment.md) and
  [Settings](../admin/settings.md).
- The console path (`MW_ADMIN_PATH`) is generated on the first start when none is
  configured; an upgraded installation keeps `/admin`. See [`admin console-path`](#admin-console-path).

## healthcheck

`mailwoman healthcheck [--url <URL>]` requests the health URL once and exits with 0
when the answer is a success status, 1 otherwise (it prints the status or the
error to standard error). It is meant for container health checks.

| Option | Default | What it does |
|---|---|---|
| `--url <URL>` | `http://<MW_BIND>/healthz` | The full URL to probe. With no `--url`, `MW_BIND` (default `0.0.0.0:8080`) is read and `0.0.0.0` becomes `127.0.0.1`. |

If `MW_HEALTH_TOKEN` is set it is sent as `X-Health-Token`. The probe ignores
proxy environment variables (`HTTP_PROXY` and similar), so a proxy cannot make a
dead server look healthy.

## doctor

`mailwoman doctor [--db-path <DSN>] [--server-key <HEX>]` prints, then exits:

1. the cache posture: whether Redis or Valkey is configured (`MW_REDIS_URL`), whether the
   database fall-through is attached, and the layers and lifetime of each cache class; and
2. the posture of the kernel jail around the message renderer, which reports a
   weaker mode where the platform has no jail (for example on Windows).

It changes nothing. If the database cannot be opened it says so and shows the store
layer as unattached.

## admin

`mailwoman admin <noun> <verb> [ARGS]` is the command-line mirror of the admin console.
It takes `--db-path` and `--server-key`, writes to the same database as the console, and
records every change in the audit log with the actor `cli`. Changes to the security
policy and appearance reach a running server within 30 seconds.

### admin admins

Console administrator accounts (not mailbox users).

| Command | What it does |
|---|---|
| `admins list` | Prints one line per administrator: user name, display name, `enabled` or `disabled`. |
| `admins create <USERNAME> [--display-name <NAME>]` | Creates an administrator with no roles. The password is read from standard input and must be 12 to 1024 characters. |
| `admins reset-password <USERNAME>` | Sets a new password read from standard input and signs the administrator out everywhere. |
| `admins disable <USERNAME>` | Disables the account. Refused when it is the last enabled Super admin. |
| `admins grant-role <USERNAME> <ROLE> [--domain <DOMAIN>]…` | Gives the account the role, replacing any grant of the same role. With `--domain` (repeatable) the role is limited to those domains. Refused when it would leave no Super admin. |

A user name is up to 64 characters of lower-case letters, digits and `. _ @ -`. The
password is never an option; for example `printf '%s' "$PASSWORD" | mailwoman admin admins create alice`.

### admin domains

Mail domains. Names are normalized the way the console does.

| Command | What it does |
|---|---|
| `domains list` | Prints the domain names, one per line. |
| `domains show <NAME>` | Prints the domain as JSON: enabled, preferred, upstream, allow list, block list, aliases and hosts. |
| `domains create <NAME>` | Registers a domain, enabled, with an empty upstream. An existing domain is left as it is. Refused when the name is already an alias. |
| `domains delete <NAME>` | Deletes the domain. Refused when it is missing, preferred, or has signed-in accounts. |
| `domains set-upstream <NAME> <JSON>` | Replaces the upstream configuration with a validated JSON object. Refused when the upstream does not match `MW_MODE`: JMAP settings are refused in engine mode, and mail-server or JMAP submission settings are refused in proxy mode. |
| `domains enable <NAME>`, `domains disable <NAME>` | Turns the domain on or off. |
| `domains prefer <NAME>`, `domains unprefer` | Sets or clears the preferred domain. |
| `domains alias <NAME> <ALIAS> [--remove]` | Maps an alias to the domain, or removes the mapping. |
| `domains host <NAME> <HOST> [--remove]` | Maps a web host to the domain, or removes the mapping. |

### admin users

| Command | What it does |
|---|---|
| `users provision <DOMAIN> <USERNAME> [--bytes-limit <N>] [--msg-limit <N>]` | Provisions a user, or updates one, with a quota. Both limits default to `0`. |
| `users quota <ACCOUNT> [--bytes-limit <N>] [--msg-limit <N>]` | Sets the quota of an account. Both limits default to `0`. |
| `users revoke-sessions <ACCOUNT>` | Revokes every session of the account and prints how many. |
| `users flags <ACCOUNT> [--zero-access] [--force-password-change] [--remote-cache-wipe] [--disabled]` | Sets all four flags: a flag you give is switched on and a flag you leave out is switched off. |
| `users cache-wipe <ACCOUNT>` | Requests a one-shot remote cache wipe for the account. |

### admin security-policy

| Command | What it does |
|---|---|
| `security-policy show` | Prints the policy in force as JSON, with the names of the fields an environment variable pins. |
| `security-policy set [OPTIONS]` | Changes the options you give; the others keep their value. |

| Option | Values | What it sets |
|---|---|---|
| `--min-tls` | `1.2`, `1.3` | The lowest TLS version for the HTTPS listener and for outbound IMAP, SMTP, POP3, ManageSieve and proxy-tunnel connections. New connections only. LDAP, Redis, PostgreSQL and HTTP fetches are not covered. |
| `--require-2fa` | `true`, `false` | Requires a second factor for every account. |
| `--argon2-m-cost` | 19456 to 1048576 (KiB) | Argon2id memory for new password and recovery-code hashes. Existing hashes are upgraded at the next successful sign-in. |
| `--argon2-t-cost` | 1 to 10 | Argon2id passes. 1 needs at least 47104 KiB of memory. |
| `--argon2-p-cost` | 1 to 16 | Argon2id lanes. |
| `--capture-policy` | `off`, `protect` | `protect` asks the desktop and Android apps to keep their window out of screenshots. A browser cannot be asked. |
| `--max-security-floor` | `none`, `full-sanitized`, `sanitized-no-media`, `plain-text` | The least restrictive rendering mode the clients allow. |
| `--dlp-rules-json` | JSON array | DLP rules applied beside `MW_DLP_RULES` to mail sent through engine-mode accounts. |

### admin integrations

| Command | What it does |
|---|---|
| `integrations list` | Prints the status of webhooks and API-key oversight. LDAP and Nextcloud print `unknown`: this process cannot see what the running server loaded. |
| `integrations revoke-api-key <ID>` | Revokes an API key. |

### admin observability

| Command | What it does |
|---|---|
| `observability show` | Prints the stored telemetry record as JSON. |
| `observability set [--log-level <LEVEL>] [--otlp-dsn <DSN>] [--metrics <true\|false>]` | Changes the stored record. It is stored only: the running log level, OTLP exporter and metrics endpoint are set from the environment at start. An empty `--otlp-dsn` clears it. |
| `observability audit [--limit <N>]` | Lists recent audit entries, newest first (default 50). |
| `observability audit-export [--limit <N>]` | Prints recent audit entries as JSON Lines (default 1000). |
| `observability bans` | Lists the ban-list entries. |
| `observability ban <IP> [--reason <TEXT>] [--expires-at <TIME>]` | Adds a ban-list entry (reason defaults to `manual`). It is a record: this ban list does not make the server refuse the address. |
| `observability unban <IP>` | Removes a ban-list entry. |

### admin appearance

| Command | What it does |
|---|---|
| `appearance show` | Prints the deployment-default appearance: the stored values, or the built-in defaults when nothing is stored. |
| `appearance set [--theme <ID>] [--brand-name <NAME>] [--accent <ID>]` | Stores the options you give and keeps the rest. A theme id is 1 to 64 lower-case letters, digits and hyphens; an accent id is at most 32; the brand name is 1 to 64 characters without control characters. An empty `--accent` means the theme's own. |

### admin cache-scope

| Command | What it does |
|---|---|
| `cache-scope list` | Prints the cache scope matrix as JSON. |
| `cache-scope set <CLASS> [--layers-json <JSON>] [--ttl-secs <N>]` | Sets the layers and lifetime of one cache class. Defaults: `["memory","redis","store"]` and `300` seconds. |

### admin console-path

The console is served under a secret path. A change is stored, takes effect at the next
start, and then ends every console session.

| Command | What it does |
|---|---|
| `console-path show` | Prints the path, and says when the running server still uses an older one. |
| `console-path set <PATH>` | Stores a path of your own: `/` followed by at least 12 characters from `A-Z a-z 0-9 . _ ~ -`. |
| `console-path generate` | Stores a new random path of 20 characters. |

`set` and `generate` are refused while `MW_ADMIN_PATH` is set: change or remove the
variable instead.

## db

`mailwoman db [--db-path <FILE>] <command>` works on a SQLite file and cannot be used with
PostgreSQL. Keys come from the environment only, never from options:

- current key: `MW_DB_KEY`, `MW_DB_KEY_FILE` or `MW_DB_KEY_COMMAND`;
- new key for `encrypt` and `rekey`: `MW_DB_NEW_KEY`, `MW_DB_NEW_KEY_FILE` or
  `MW_DB_NEW_KEY_COMMAND` (`encrypt` falls back to the current-key variables);
- backup key for `backup` and `restore`: `MW_DB_BACKUP_KEY`, `MW_DB_BACKUP_KEY_FILE` or
  `MW_DB_BACKUP_KEY_COMMAND` (the current key is used when none is set).

See [Database encryption](../admin/encryption.md) for the procedure.

| Command | What it does |
|---|---|
| `status` | Prints the file's state and the configured key source. Changes nothing, and can run beside the server. |
| `keygen` | Prints a new random 256-bit key as 64 hexadecimal characters. |
| `online [--status] [--cancel] [TUNING]` | Asks the running server to convert the database without stopping. Queues a request, waits up to 10 seconds for the server to accept it, then returns while the server keeps working. `--status` prints the latest report; `--cancel` requests cancellation before the final publication step. `--status` and `--cancel` are exclusive and take no tuning options. |
| `cleanup [--apply --offline] [SELECTORS]` | Previews, or with `--apply` performs, offline cleanup. `--apply` needs `--offline`, which confirms that the server and every other writer are stopped. At least one selector is required. |
| `encrypt [--dry-run] [--no-rollback-copy]` | Encrypts a plaintext database in place by copying, verifying and swapping. Stop the server first. |
| `rekey [--dry-run] [--no-rollback-copy]` | Re-encrypts under a new key. Stop the server first. |
| `decrypt [--dry-run] [--no-rollback-copy]` | Turns an encrypted database back into a plaintext one. Stop the server first. |
| `verify` | Proves the database is encrypted: the header, a wrong key refused, the right key opening, and a marker row that round-trips and is absent from the file bytes. |
| `backup --to <FILE> [--plaintext]` | Writes an encrypted copy. `<FILE>` must not exist. A plaintext copy of an encrypted database is refused unless `--plaintext` is given. |
| `restore --from <FILE> --to <FILE>` | Restores a backup to a new file, re-keyed to the current key. |
| `rollback --copy <FILE>` | Puts a rollback copy written by `encrypt`, `rekey` or `decrypt` back in place. |
| `purge-rollback <FILE>` | Overwrites and deletes a rollback copy once the result is checked. |

`encrypt`, `rekey` and `decrypt` keep a rollback copy unless `--no-rollback-copy` is given;
with it the original is overwritten and deleted after a successful swap. `--dry-run` does
everything except the swap and reports what would happen.

`online` tuning options (all optional, with their accepted ranges):
`--batch-rows` 1 to 128, `--batch-mib` 1 to 256, `--journal-limit` 1024 to 2000000,
`--pause-ms` 0 to 1000, `--max-wal-mib` 32 to 4096, `--max-cutover-ms` 100 to 10000.

`cleanup` selectors:

| Selector | What it does |
|---|---|
| `--orphan-bodies` | Removes cached bodies that no message references. Live mail is never touched. |
| `--purge-search-index` | Purges the default search index beside the database; rebuild it after restarting. Not supported when `MW_SEARCH_DIR` is set. |
| `--logs-older-than-days <N>` | Purges operational diagnostics older than N days (`0` means all). |
| `--compact` | Rebuilds SQLite to reclaim free pages. May need twice the database size in free disk space. |
| `--enable-incremental-vacuum` | Enables incremental auto-vacuum. A database in `NONE` mode needs one offline rebuild. |

## erasure

`mailwoman erasure [--db-path <FILE>] <command>` backs up and restores a SQLite database
so that people erased after a backup are erased again after a restore. All commands, `scrub-logs`
included, need the server key in `MW_SERVER_KEY` (the one the databases were written
with). See [Erasure coverage](../admin/erasure-coverage.md) for what an erasure removes.

| Command | What it does |
|---|---|
| `watermark` | Prints the erasure tombstone watermark of the database. |
| `backup --out <FILE> [--label <TEXT>]` | Writes a consistent copy and a manifest `<FILE>.manifest.json` that records the watermark. `<FILE>` must not exist. The label defaults to `backup`. |
| `export-tombstones --since <N> --out <FILE>` | Writes the tombstones newer than a watermark, with the key generations they need, as JSON. |
| `restore --backup <FILE> --to <FILE> [--live-db <DSN>] [--tombstones <FILE>] [--no-propagate]` | Copies the backup to a new database file (which must not exist) and replays the erasures newer than the backup's watermark. The newer tombstones come from `--live-db`, the install being replaced, or from a file written by `export-tombstones`. With neither, nothing is replayed. `--no-propagate` restores only; the setting `backup.propagateErasure` can also turn replay off. |
| `scrub-logs --dir <DIR> --address <ADDRESS> [--dry-run]` | Replaces one address in the rotated log files of a directory with its pseudonym, and prints the number of lines and files changed. `--dry-run` reports what would change. |

## maintenance

`mailwoman maintenance [--db-path <DSN>] [--server-key <HEX>] <command>` runs one-shot
store maintenance. None of these runs automatically.

| Command | What it does |
|---|---|
| `rethread <ACCOUNT>` | Re-threads all stored mail of an account with the full JWZ algorithm, then prints the account, message, thread and reassigned counts. Safe to repeat: a repeat reassigns nothing. It re-keys thread ids, so run it deliberately. It reads the stored mail only and makes no connection to the mail server. |
| `gc-uploads [--older-than-hours <N>]` | Deletes unreferenced uploaded blobs older than N hours (default 24): the metadata row and the stored object. Set the same `MW_UPLOAD_DIR` the server uses, or the rows are reclaimed and the files stay for a later run. |
| `vacuum` | Runs a plain `VACUUM` (on PostgreSQL `VACUUM notes`, never `VACUUM FULL`) to clear the old bytes left in free pages by sealing note metadata. Unconditional and safe to repeat. |

## migrate-store

`mailwoman migrate-store --from <SQLITE> [--to <POSTGRES-DSN>] [--complete] [--upgrade-source]`
copies a SQLite store into a PostgreSQL database and prints the number of rows and tables
copied. Both stores must use the same server key, which is read from `MW_SERVER_KEY`
(required). See [PostgreSQL](../deploy/postgres.md).

| Option | Environment | What it does |
|---|---|---|
| `--from <SQLITE>` | `MW_MIGRATE_FROM` | The source SQLite database. Required. |
| `--to <DSN>` | `MW_MIGRATE_TO`, then `MW_DB_PATH` | The destination. It must start with `postgres://` or `postgresql://`. |
| `--complete` | none | Copies every application table, including deployment configuration and grants, not only the mail data. |
| `--upgrade-source` | none | Upgrades the source schema first. Needs `--complete`. Use it only on an offline working copy. |

## plugin

`mailwoman plugin [--db-path <DSN>] [--server-key <HEX>] <command>` works on the plugin
registry. See [Plugins](../admin/plugins.md).

| Command | What it does |
|---|---|
| `list` | Prints one line per plugin: id, version, `approved=…`, `enabled=…`; or `no plugins registered`. |
| `approve <ID>` | Approves a plugin as `cli`. |

## password

`mailwoman password [OPTIONS]` changes an account's password through the backend that
`MW_PASSWD_BACKEND` selects, then re-seals the account's stored upstream credentials
under the new password when the backend asks for it. It prints a summary line with
the number of credentials that were re-sealed, and records the change in the audit log. A backend that is selected but cannot be built makes
the command panic with the configuration error (exit status 101, not 1).

| Option | Environment | What it does |
|---|---|---|
| `--account-id <ID>` | `MW_ACCOUNT_ID` | The account whose password changes. Required. |
| `--old-password <PASSWORD>` | `MW_OLD_PASSWORD` | The current password. Required. |
| `--new-password <PASSWORD>` | `MW_NEW_PASSWORD` | The new password. Required. |
| `--db-path`, `--server-key` | `MW_DB_PATH`, `MW_SERVER_KEY` | As for every database command. |

## mcp-stdio

`mailwoman mcp-stdio --server <URL> [--token <TOKEN>]` reads JSON-RPC from standard input,
sends it to a remote `/mcp` endpoint, and writes the answers to
standard output. It is for MCP clients that start a local process. See
[MCP](../security/mcp.md).

| Option | Environment | What it does |
|---|---|---|
| `--server <URL>` | `MW_MCP_SERVER` | The remote MCP endpoint. Required. |
| `--token <TOKEN>` | `MW_MCP_TOKEN` | A bearer token (API key or OAuth token) sent to the remote. |

## fonts pull

`mailwoman fonts pull <FAMILY>… [OPTIONS]` downloads Google Fonts families, keeps their
per-range subsets, and writes self-hostable `woff2` files and a rewritten stylesheet so
that a site can serve them under `font-src 'self'`. It prints the number of font faces, the
stylesheet path and the number of `woff2` files. See [Fonts](../deploy/fonts.md).

| Argument or option | Default | What it does |
|---|---|---|
| `<FAMILY>…` | required | Family specs such as `Inter:wght@400;700` or `Fraunces:ital@1`. |
| `--out <DIR>` | `fonts` | Output directory. |
| `--text <CHARS>` | all | Restricts the subset to these characters. |
| `--url-prefix <PREFIX>` | `/fonts` | The origin-relative prefix written into the stylesheet's `url()` values. |
| `--css-name <FILE>` | `fonts.css` | The stylesheet file name inside `--out`. |
