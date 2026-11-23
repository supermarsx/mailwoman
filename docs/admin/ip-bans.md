---
screen: true
---
# IP ban list

The **IP ban list** refuses requests from client IP addresses you name. A banned
address gets no answer from the web app, the APIs or the admin console, including
requests that carry an existing mailbox or console session. Open it in the console
under **Security centre** > **IP ban list** (`#/security/bans`).

![The IP ban list tab of the Security centre, below the country database warning: the explanation of what a ban does, Add ban, the search field and two bans.](/screenshots/admin-access/ip-bans-tab.webp){.mw-shot}

The list shows each ban with its address, reason, status and expiry:

![The IP ban list tab of the Security centre with two bans and the Add ban button.](/screenshots/admin-access/ip-bans-list.webp){.mw-crop}

1. **Add ban** opens the ban editor.
2. **Active bans only** hides bans whose expiry has passed.
3. **Unban** removes that ban.
{.mw-callouts}

## Before you start

- You need the **bans.read** capability to see the list and **bans.write** to add
  or remove a ban, both granted for all domains. A role limited to some domains
  cannot open this tab.
- Adding or removing a ban asks you to confirm it is you (password, plus your
  second factor if you have one) unless you confirmed in the last five minutes.
- Behind a reverse proxy, set the trusted proxies first (`MW_TRUSTED_PROXIES`,
  `MW_FORWARDED_MODE`; see [Reverse proxy](/deploy/reverse-proxy)). Mailwoman bans
  the address it resolves for the client: the socket peer, or the forwarded
  address when the peer is a trusted proxy. A `X-Forwarded-For` header from an
  untrusted peer is ignored, so it cannot be used to dodge or redirect a ban.

## Ban an address

1. Open **Security centre** > **IP ban list** and select **Add ban**.
2. Enter the **IP address**: one IPv4 or IPv6 address. Ranges (CIDR) are not
   accepted. An IPv4-mapped IPv6 address (`::ffff:192.0.2.77`) is stored as the
   IPv4 address.
3. Enter a **Reason** (up to 512 characters, no control characters).
4. Leave **Expiry (this device's local time)** empty for a ban that lasts until you
   remove it, or choose a time in the future.
5. Select **Save IP ban** and confirm it is you if asked.

![The ban editor with an IP address and a reason filled in.](/screenshots/admin-access/ip-bans-editor.webp){.mw-crop}

1. **IP address**
2. **Reason**
3. **Expiry (this device's local time)**
4. **Save IP ban**
{.mw-callouts}

The ban applies to the next request from that address, on every server that shares
the database. Open connections are not cut; their next HTTP request is refused.

::: warning
There is no exemption for loopback or private addresses. Banning the address of
your reverse proxy, a NAT gateway or `127.0.0.1` blocks everyone who reaches the
server through it, including you.
:::

## Remove a ban

Select **Unban** on the row and confirm. If you have locked yourself out of the
console, remove the ban on the server with the command line, which works on the
database directly:

```sh
mailwoman admin observability bans               # list
mailwoman admin observability unban 192.0.2.77   # remove
```

The command needs the same `MW_DB_PATH` and `MW_SERVER_KEY` as the server.

## What a ban does and does not do

| | |
|---|---|
| Refused | Every HTTP request from the address, with `403` and no detail. |
| Still answered | `/healthz` and `/readyz`, so deployment health checks keep working. |
| Not affected | SMTP, IMAP and other listeners, and anything in front of Mailwoman. A ban is an application check, not a firewall rule. Fail2ban and firewall rules work independently (see [fail2ban](/deploy/fail2ban)). |
| Expiry | At its expiry time a ban stops applying. An expired ban stays in the list until it is removed, or until any new ban is added, which removes every expired ban. |
| Same address again | Saving a ban for an address that is already banned replaces the earlier ban (reason and expiry). |
| Limit | At most 10,000 bans. Expired bans are pruned when a new ban is added. |
| Database unreachable | If the ban list cannot be read, requests are refused with `503` rather than let through. |

Adding and removing bans is recorded in the audit log.

## Related

- [Temporary sign-in locks](./signin-locks.md): automatic, short locks after
  failed passwords
- [Inbound access](./inbound-allowlists.md): allow only listed addresses
- [Security centre](./security-center.md)
- [fail2ban](/deploy/fail2ban)
