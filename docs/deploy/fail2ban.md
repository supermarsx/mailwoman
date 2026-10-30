# fail2ban

Mailwoman writes one log line for each rejected credential, both on the admin
console (`POST /admin/login`, passkey and second-factor steps) and on webmail
sign-in. A fail2ban jail can read those lines and ban the source address at the
firewall. Mailwoman also keeps its own in-process ban list for the admin
console (5 failures in 15 minutes); the jail works in addition to it and covers
webmail as well.

The files are in `deploy/fail2ban/`:

| File | Install to |
|------|------------|
| `deploy/fail2ban/filter.d/mailwoman.conf` | `/etc/fail2ban/filter.d/mailwoman.conf` |
| `deploy/fail2ban/jail.d/mailwoman.conf` | `/etc/fail2ban/jail.d/mailwoman.conf` |
| `deploy/fail2ban/filter.d/mailwoman-admin-probe.conf` | `/etc/fail2ban/filter.d/mailwoman-admin-probe.conf` |

```sh
install -m 0644 deploy/fail2ban/filter.d/mailwoman.conf /etc/fail2ban/filter.d/
install -m 0644 deploy/fail2ban/filter.d/mailwoman-admin-probe.conf /etc/fail2ban/filter.d/
install -m 0644 deploy/fail2ban/jail.d/mailwoman.conf /etc/fail2ban/jail.d/
fail2ban-client reload
fail2ban-client status mailwoman
fail2ban-client status mailwoman-admin-probe
```

The jail file has two jails: `mailwoman` (failed sign-ins) and
`mailwoman-admin-probe` (sources the server blocked for searching for the admin
console, see [below](#admin-probe-blocks)).

## The log line

```text
2026-10-29T00:54:14.905392Z  WARN mailwoman.http.request{route_class="admin"}: mw_server::admin_accounts: 2026-10-29T00:54:14.904168600+00:00 mailwoman[auth]: authentication failure; logname=root rhost=203.0.113.7
```

Everything up to the second timestamp is the standard `tracing` prefix (time,
level, request span, module). The filter matches from `mailwoman[auth]:`
onwards, and fail2ban takes the time from the start of the line:

```ini
failregex = mailwoman\[auth\]: authentication failure; .*rhost=<HOST>
```

This is the same string as `mw_admin::FAIL2BAN_FAILREGEX` in
`crates/mw-admin/src/banlist.rs`. The test
`crates/mw-server/tests/t39_fail2ban_filter.rs` fails if the shipped filter and
the code differ.

- `rhost=` is always an IP address the server resolved, never a host name, so
  the jail does not need DNS lookups (`usedns` can stay at its default).
- `logname=` is the login name as submitted. Characters other than ASCII
  letters, digits and `@ . _ - +` are replaced with `_`, so a login name cannot
  add a second line or its own `rhost=` token.
- A failure for which the server could not determine a client address writes no
  line.
- On webmail, only failures the server classifies as rejected credentials
  (`signin_guard::credential_rejection`) are logged this way; an upstream
  outage, for example, is not.

## Admin-probe blocks

The server's [admin-probe protection](../security/admin-panel.md#admin-probe-protection)
counts requests that search for the admin console (`/wp-admin`, `/.env`, guessed
console paths, refused admin sign-ins) with a grace allowance, and blocks a
source that goes past it. Each block writes one line:

```text
2026-10-29T23:41:02.118Z  WARN mailwoman.http.request{route_class="static"}: mw_server::admin_probe: 2026-10-29T23:41:02Z mailwoman[admin-probe]: source blocked; signal=probe-path strike=1 blockSecs=900 rhost=203.0.113.7
```

```ini
failregex = mailwoman\[admin-probe\]: source blocked; .*rhost=<HOST>
```

This is `admin_probe::FAIL2BAN_PROBE_FAILREGEX` in
`crates/mw-server/src/admin_probe.rs`; a unit test there fails if the shipped
filter differs.

- The server has already applied its grace allowance, so the jail bans on the
  first line (`maxretry = 1`). The shipped jail bans for 6 hours, longer each
  time (`bantime.increment`).
- `rhost=` is always the full client address, whatever
  `security.adminProbe.auditIp` says for the audit log: a jail bans
  addresses. Turn the line off with `security.adminProbe.fail2banLog = false`.
- `signal=` is the miss that crossed the threshold (`probe-path`, `slug-guess`,
  `admin-signin-failed`, `admin-unauthenticated`), `strike=` the repeat count.
- An allowlisted address (`security.adminProbe.allowlist`) is never blocked, so
  it never appears. Add the same networks to the jail's `ignoreip`.
- The server's block is per instance; the firewall ban covers every instance
  behind the same host.

Example `/etc/fail2ban/jail.d/mailwoman.local` override for a stricter policy:

```ini
[mailwoman-admin-probe]
bantime  = 1d
bantime.increment = true
bantime.maxtime = 4w
ignoreip = 127.0.0.1/8 ::1 192.0.2.0/24
```

Test the filter with `fail2ban-regex` as below, using
`/etc/fail2ban/filter.d/mailwoman-admin-probe.conf`.

## Log source

Mailwoman has no log-file setting. It writes its log to standard output, and
the log level is set with `MW_LOG` (default `info`) or `RUST_LOG`. The
authentication-failure lines are at `WARN`, so any level from `warn` down keeps
them.

**Turn off colour codes.** By default the `tracing` prefix contains terminal
colour codes even when standard output is not a terminal. fail2ban then cannot
read the timestamp at the start of the line: a file-backed jail logs
`Found a match but no valid date/time found` and does not count the failure.
Set `NO_COLOR=1` in the server's environment. `docs/deploy/mailwoman.service`
sets it.

**systemd (`docs/deploy/mailwoman.service`).** Standard output goes to the
journal. The shipped jail uses `backend = systemd`, and the filter sets
`journalmatch = _SYSTEMD_UNIT=mailwoman.service`. If your unit has another
name, override `journalmatch` in the jail:

```ini
[mailwoman]
journalmatch = _SYSTEMD_UNIT=webmail.service
```

**Docker.** Run the container with the journald log driver and match on the
container name:

```sh
docker run --log-driver=journald --name mailwoman -e NO_COLOR=1 ...
```

```ini
[mailwoman]
journalmatch = CONTAINER_NAME=mailwoman
```

**A log file.** If you redirect standard output to a file, switch the jail to
file mode:

```ini
[mailwoman]
backend = auto
logpath = /var/log/mailwoman/mailwoman.log
```

`NO_COLOR=1` is required in this mode (see above).

## Client address behind a reverse proxy

The address in `rhost=` is the one the server accepts as the client. Behind a
reverse proxy (for example `docs/deploy/nginx.conf`), set `MW_TRUSTED_PROXIES`
to the proxy's address so Mailwoman takes the client address from the proxy's
forwarding header. Without it every line carries the proxy's address, and the
jail would ban the proxy.

The jail bans the ports clients connect to (`port = http,https`), which are the
reverse proxy's ports, not Mailwoman's own `MW_BIND` port.

## Tuning the jail

| Setting | Shipped value | Notes |
|---------|---------------|-------|
| `maxretry` | `5` | Failures from one address before a ban. Matches the in-process monitor. |
| `findtime` | `15m` | Window the failures are counted in. |
| `bantime` | `1h` | Ban length. `bantime.increment = true` lengthens repeat bans. |
| `port` | `http,https` | Ports the ban applies to. |
| `ignoreip` | not set | Add office or monitoring addresses, e.g. `ignoreip = 127.0.0.1/8 ::1 192.0.2.0/24`. |

Put overrides in `/etc/fail2ban/jail.d/mailwoman.local` rather than editing the
shipped file.

## Testing the filter

`fail2ban-regex` runs the filter against a log without starting a jail.

Against the journal on the server:

```sh
fail2ban-regex systemd-journal /etc/fail2ban/filter.d/mailwoman.conf
```

Against a file:

```sh
fail2ban-regex /var/log/mailwoman/mailwoman.log /etc/fail2ban/filter.d/mailwoman.conf
```

The output ends with a `Lines: N lines, 0 ignored, M matched` summary; `M`
should equal the number of failed sign-ins in the log.

To produce a sample log from the real server code without a deployment, run the
integration test with `MW_FAIL2BAN_SAMPLE_LOG` set. It signs in to the admin
console twice with wrong passwords and writes what the server's log layer
printed (the same formatter `main.rs` uses for standard output) to that path:

```sh
NO_COLOR=1 MW_FAIL2BAN_SAMPLE_LOG=/tmp/mailwoman-sample.log \
  cargo test -p mw-server --test t39_fail2ban_filter -- --test-threads=1
docker run --rm -v /tmp/mailwoman-sample.log:/data/mw.log:ro \
  -v "$PWD/deploy/fail2ban/filter.d/mailwoman.conf:/etc/fail2ban/filter.d/mailwoman.conf:ro" \
  --entrypoint fail2ban-regex crazymax/fail2ban:latest \
  /data/mw.log /etc/fail2ban/filter.d/mailwoman.conf
```

Expect `2 matched` (the missed lines are start-up messages) and one
date-template hit per line. `127.0.0.1` is the
test client; a running jail ignores it unless `ignoreself = false` is set.
