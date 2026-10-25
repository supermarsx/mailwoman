# Fail2ban

Mailwoman emits `mailwoman[auth]: authentication failure` records for rejected
admin credentials and known webmail password/factor failures. The final `rhost`
field is a parsed source IP; usernames cannot insert another record or host.
Backend timeouts, certificate failures, disabled accounts, and missing domain
configuration do not trigger webmail bans. Fail2ban performs firewall enforcement;
the Mailwoman admin ban list alone does not install firewall rules.

Install `filter.d/mailwoman.conf` into `/etc/fail2ban/filter.d/`. Copy the example
jail into `/etc/fail2ban/jail.d/mailwoman.local`, set the actual service unit,
management allowlist, ports, and thresholds, then enable it. Check configuration
before restarting fail2ban:

```sh
fail2ban-client -t
fail2ban-regex systemd-journal /etc/fail2ban/filter.d/mailwoman.conf --journalmatch='_SYSTEMD_UNIT=mailwoman.service'
systemctl restart fail2ban
fail2ban-client status mailwoman
```

For file collection, use `backend = auto`, remove `journalmatch`, and set `logpath`
to the actual collector-managed plain-text log. Disable ANSI colors in that
collector. Test the file with `fail2ban-regex` before enabling bans. The filter is
for plain tracing output or raw auth records, not Docker JSON log envelopes.

For Docker, collect container output on the host (for example with Docker's
journald driver), and set `journalmatch` to that container's actual journal fields.
Run fail2ban on the host or at the public reverse proxy. Select the host firewall
action/chain that actually handles incoming traffic: Docker-published ports may
bypass the ordinary INPUT chain. This package does not modify host firewall rules
or grant the application container firewall privileges.

Configure Mailwoman's trusted proxies before accepting forwarded client IPs. An
untrusted forwarding header must never identify the address to ban. When the
application cannot resolve a source IP, it emits no ban record. Verify a real
failed login in your deployed topology and confirm that the reported IP is the
client, not the reverse proxy, before enabling enforcement.

The in-memory admin application-log view is diagnostic only. Fail2ban must read
the original console/journal/file stream, where individual failures remain
individual events. Set the server log filter to include warnings.

Reference: [fail2ban-regex documentation](https://github.com/fail2ban/fail2ban/blob/master/man/fail2ban-regex.1).
