# fail2ban

Mailwoman writes a log line that fail2ban can read in two cases: a rejected sign-in password or code (webmail and the admin console), and a source that probing protection has blocked. A fail2ban jail can read those lines and ban the address at the firewall, which covers every port and every Mailwoman instance behind that firewall. The server's own protections ([sign-in locks](../admin/signin-locks.md), [request protection](../admin/request-protection.md)) work without fail2ban; the jail is an addition.

The filter and jail files are in `deploy/fail2ban/`:

| File | Install to |
|------|------------|
| `deploy/fail2ban/filter.d/mailwoman.conf` | `/etc/fail2ban/filter.d/mailwoman.conf` |
| `deploy/fail2ban/filter.d/mailwoman-admin-probe.conf` | `/etc/fail2ban/filter.d/mailwoman-admin-probe.conf` |
| `deploy/fail2ban/filter.d/mailwoman-request-protection.conf` | `/etc/fail2ban/filter.d/mailwoman-request-protection.conf` |
| `deploy/fail2ban/jail.d/mailwoman.conf` | `/etc/fail2ban/jail.d/mailwoman.conf` |

```sh
install -m 0644 deploy/fail2ban/filter.d/mailwoman.conf /etc/fail2ban/filter.d/
install -m 0644 deploy/fail2ban/filter.d/mailwoman-admin-probe.conf /etc/fail2ban/filter.d/
install -m 0644 deploy/fail2ban/filter.d/mailwoman-request-protection.conf /etc/fail2ban/filter.d/
install -m 0644 deploy/fail2ban/jail.d/mailwoman.conf /etc/fail2ban/jail.d/
fail2ban-client reload
fail2ban-client status mailwoman
fail2ban-client status mailwoman-admin-probe
```

The jail file defines three jails: `mailwoman` (rejected sign-ins), `mailwoman-admin-probe` (sources blocked for probing) and `mailwoman-request-protection` (sources blocked for a recognized scan path; off by default, see [Blocked scan-path sources](#blocked-scan-path-sources)).

## Before you start

- **The log must reach fail2ban in plain text with full addresses.** The setting **IP addresses in logs** ([`logging.redaction.ips`](/admin/settings#logging.redaction.ips)) applies to standard output as well, and its default is `full`. Any other value breaks the jail: `truncated` writes the network (`203.0.113.0`), so the jail bans the wrong address, and `hashed` or `none` leave no address at all.
- **Behind a reverse proxy**, set `MW_TRUSTED_PROXIES` to the proxy's address, so the server takes the client address from the proxy's forwarding header. Without it every line carries the proxy's address and the jail bans the proxy. See [Reverse proxy](./reverse-proxy.md).

## Rejected sign-ins

One line is written for each rejected sign-in:

- webmail: a password or second-factor code that the mail server or Mailwoman rejected. Failures for other reasons, such as an unreachable mail server, write no line;
- the admin console: a wrong password, a wrong authenticator code, and a refused passkey sign-in.

```text
2026-11-23T04:52:28.863228Z  WARN mw_server_core::admin_accounts: 2026-11-23T04:52:28.860590200+00:00 mailwoman[auth]: authentication failure; logname=admin rhost=203.0.113.7
```

Everything up to the second timestamp is the log prefix (time, level, module). The filter matches from `mailwoman[auth]:` on:

```ini
failregex = mailwoman\[auth\]: authentication failure; .*rhost=<HOST>
```

- `rhost=` is the IP address the server accepted as the client, never a host name, so the jail needs no DNS lookups. A failure for which the server has no client address writes no line.
- `logname=` is the sign-in name as typed. Characters other than ASCII letters, digits and `@ . _ - +` are replaced with `_`, so a sign-in name cannot add a line or a second `rhost=`.
- The supplied jail bans after 5 lines from one address in 15 minutes, for 1 hour.

## Blocked probing sources

[Probing protection](../admin/request-protection.md#probing) counts requests that look for the admin console and blocks a source after its grace allowance. Each block writes one line:

```text
2026-10-29T23:41:02.118Z  WARN mw_server_core::admin_probe: 2026-10-29T23:41:02Z mailwoman[admin-probe]: source blocked; signal=probe-path strike=1 blockSecs=900 rhost=203.0.113.7
```

```ini
failregex = mailwoman\[admin-probe\]: source blocked; .*rhost=<HOST>
```

- The server has already applied its grace allowance, so the jail bans on the first line (`maxretry = 1`) for 6 hours, longer for repeat offenders (`bantime.increment`).
- `signal=` is the miss that crossed the threshold (`probe-path`, `slug-guess`, `admin-signin-failed`, `admin-unauthenticated`), `strike=` the repeat count, `blockSecs=` the server's own block length.
- `rhost=` is the full address whatever **Source address in the audit log** says. **IP addresses in logs** still applies to it (see above).
- An address in **Never-blocked networks** (`security.adminProbe.allowlist`) is never blocked, so it never appears. Add the same networks to the jail's `ignoreip`.
- The **Recognized scan paths** block of request protection writes its own line, read by a separate jail; see [Blocked scan-path sources](#blocked-scan-path-sources).

The line is switched on and off on the **Probing** sub-tab of **Security centre** > **Aggregate and adaptive protection**:

![The Audit and logging group of the Probing sub-tab: the source address in the audit log set to masked and the fail2ban log line on block switched on.](/screenshots/admin-access/fail2ban-probe-logging.webp){.mw-crop}

**fail2ban log line on block** ([`security.adminProbe.fail2banLog`](/admin/settings#security.adminProbe.fail2banLog)) is on by default.

## Blocked scan-path sources

The **Recognized scan paths** block of [request protection](../admin/request-protection.md) blocks a source that requests a catalogued scan path such as `/.env` or `/.git/config`. When a request starts an enforced block, one line is written:

```text
2026-11-24T10:02:11.402Z  WARN request_protection: 2026-11-24T10:02:11Z mailwoman[request-protection]: source blocked; reason=probe-path blockSecs=300 rhost=203.0.113.7
```

```ini
failregex = mailwoman\[request-protection\]: source blocked; .*rhost=<HOST>
```

- A request a browser made for another site's page (`Sec-Fetch-Site` is `cross-site` or `same-site`, for example an `<img>` on a third-party page pointing at `/.env`) is not recorded and writes no line. Requests without the header, as scanners send them, and typed URLs (`none`) are recorded.
- A request that carries a verified mailbox or admin console session is not recorded, and passes an active block of its address. Requests from the same address without a session stay blocked.
- The line is written when a block starts. Requests in the block's first second can repeat it; later scan-path requests during the block write no line.
- `rhost=` is the canonical client address. **IP addresses in logs** applies to it (see above).
- The supplied jail `mailwoman-request-protection` is **off by default** (`enabled = false`): one request is enough for a line, and a firewall ban covers every user behind that address. To use it, set `enabled = true` in `/etc/fail2ban/jail.d/mailwoman.conf` and add your own networks to `ignoreip`. As supplied it bans on the first line (`maxretry = 1`) for 1 hour.

## Log source

Mailwoman writes its log to standard output. The level is set with `MW_LOG` (default `info`) or `RUST_LOG`; both kinds of line are written at `WARN`, so any level that includes warnings keeps them.

The log files Mailwoman writes itself when **Write log files** (`logging.file.enabled`) is on are encrypted with the server key. fail2ban cannot read them; point the jail at standard output, through the journal or a redirect.

**Turn off colour codes.** The log prefix contains terminal colour codes even when standard output is not a terminal. fail2ban then cannot read the timestamp at the start of the line: a file-backed jail logs `Found a match but no valid date/time found` and does not count the failure. Set `NO_COLOR=1` in the server's environment. The example unit `docs/deploy/mailwoman.service` sets it.

**systemd.** Standard output goes to the journal. The supplied jails use `backend = systemd`, and the filters set `journalmatch = _SYSTEMD_UNIT=mailwoman.service`. If your unit has another name, override `journalmatch` in the jail:

```ini
[mailwoman]
journalmatch = _SYSTEMD_UNIT=webmail.service
```

**Docker.** Run the container with the journald log driver and match on the container name:

```sh
docker run --log-driver=journald --name mailwoman -e NO_COLOR=1 ...
```

```ini
[mailwoman]
journalmatch = CONTAINER_NAME=mailwoman
```

**Standard output redirected to a file.** Switch the jail to file mode:

```ini
[mailwoman]
backend = auto
logpath = /var/log/mailwoman/mailwoman.log
```

## Tuning the jails

| Setting | `mailwoman` | `mailwoman-admin-probe` | Notes |
|---------|-------------|-------------------------|-------|
| `maxretry` | `5` | `1` | Lines from one address before a ban. |
| `findtime` | `15m` | `1h` | Window the lines are counted in. |
| `bantime` | `1h` | `6h`, `bantime.increment = true` | Ban length. |
| `port` | `http,https` | `http,https` | The ports clients connect to, which are the reverse proxy's ports, not Mailwoman's own `MW_BIND` port. |
| `ignoreip` | not set | not set | Add office or monitoring networks, for example `ignoreip = 127.0.0.1/8 ::1 192.0.2.0/24`. |

Put overrides in `/etc/fail2ban/jail.d/mailwoman.local` rather than editing the supplied file:

```ini
[mailwoman-admin-probe]
bantime  = 1d
bantime.maxtime = 4w
ignoreip = 127.0.0.1/8 ::1 192.0.2.0/24
```

## Testing the filters

`fail2ban-regex` runs a filter against a log without starting a jail:

```sh
fail2ban-regex systemd-journal /etc/fail2ban/filter.d/mailwoman.conf
fail2ban-regex /var/log/mailwoman/mailwoman.log /etc/fail2ban/filter.d/mailwoman-admin-probe.conf
```

The output ends with `Lines: N lines, 0 ignored, M matched`; `M` should equal the number of rejected sign-ins (or blocks) in the log. To produce a line on purpose, sign in to the console once with a wrong password from a test machine.

## Troubleshooting

| Symptom | Cause | Fix |
|---|---|---|
| `Found a match but no valid date/time found` | Colour codes before the timestamp | Set `NO_COLOR=1` for the server. |
| The jail bans the reverse proxy | `MW_TRUSTED_PROXIES` is not set | Set it to the proxy's address. |
| Lines match but `rhost=` is `203.0.113.0`, `h:…` or `[ip]` | **IP addresses in logs** is not `full` | Set `logging.redaction.ips` back to `full`, or do not use fail2ban. |
| No `mailwoman[admin-probe]` lines | **fail2ban log line on block** is off, or the source is in **Never-blocked networks** | Check the **Audit and logging** group on the **Probing** sub-tab. |

## Related

- [Aggregate and adaptive protection](../admin/request-protection.md)
- [Temporary sign-in locks](../admin/signin-locks.md)
- [IP bans](../admin/ip-bans.md)
- [Reverse proxy](./reverse-proxy.md)
