---
screen: true
---
# Temporary sign-in locks

After several rejected passwords from one source, Mailwoman refuses further
sign-ins from that source for a while. A source is one IPv4 address, or one IPv6
/64 network (every IPv6 address that shares its first 64 bits). Mailbox sign-in and console
sign-in are counted separately. The **Temporary sign-in locks** tab of the
**Security centre** (`#/security/locks`) lists the addresses that are locked now
and lets you unlock one early.

![The Temporary sign-in locks tab of the Security centre with no active locks.](/screenshots/admin-access/signin-locks-page.webp){.mw-shot}

The controls of the tab:

![The Temporary sign-in locks tab with no active locks.](/screenshots/admin-access/signin-locks-tab.webp){.mw-crop}

1. **Configure sign-in limits** opens the settings below.
2. **Sign-in surface** shows locks for **Administrator**, **Mailbox** or **All sign-in surfaces**.
3. **Refresh locks** reloads the list.
{.mw-callouts}

Each row shows the **IP address**, the surface, the number of **Rejected
credentials** and **Locked until**, with an **Unlock** button. For an IPv6 source
the address shown is the start of the locked /64 (for example `2001:db8:1:2::`),
and the lock covers every address in it.

## How a lock works

- A sign-in attempt counts against its source and surface when the
  credentials are rejected: a wrong password, a mail server that refuses the
  credentials, or a refused second-factor code.
- These do not count: an expired sign-in step, a disabled account, the mail server
  or database being unreachable, and errors while creating the session.
- When the count reaches the limit within the counting window, the source is
  locked for the lock duration. During the lock every sign-in from that source on
  that surface is refused with "Too many rejected credentials; wait before trying
  again" and a retry time, before the password is checked.
- Attempts made during a lock do not extend it.
- A successful sign-in from the source clears its counter.
- The lock covers new sign-ins of every kind on that surface, including single
  sign-on. It does not end sessions that are already open.

Password-spray protection can also lock a source and the lock appears in the same
list. It counts the different usernames a source failed with in
[`security.spray.windowSecs`](/admin/settings#security.spray.windowSecs) (900
seconds by default). At [`security.spray.lockAt`](/admin/settings#security.spray.lockAt)
different usernames (25 by default; three times as many for a source that signed in
successfully within [`security.trust.knownTtlDays`](/admin/settings#security.trust.knownTtlDays), 30 days by default) the source is locked on that surface for
[`security.spray.lockSecs`](/admin/settings#security.spray.lockSecs) (900 seconds by
default), whatever the failed-credential limit is. Spray protection is on by
default ([`security.spray.enabled`](/admin/settings#security.spray.enabled)).

The source address is the one Mailwoman resolves with its trusted-proxy settings
(see [Reverse proxy](/deploy/reverse-proxy)); a forwarded header from an untrusted
peer cannot change it. People behind one NAT address share a counter, so one of
them can lock the others out.

Counters and locks are stored in the database, so they survive a restart and are
shared by every server that uses the same database. At most 10,000
source-and-surface records are kept at a time, across both surfaces. When that is
full, a new source replaces the record that failed least recently and is not
locked; active locks are never dropped. Only if every record is an active lock are
failures from a new source not counted until a lock ends; that source can still
sign in, and is still slowed by the sign-in retry cooldown. Records expire after the counting window or the lock,
whichever is later. If the lock state cannot be read, sign-in is refused.

## Unlock an address

1. Open **Security centre** > **Temporary sign-in locks**.
2. Find the address (filter by **Sign-in surface** or search).
3. Select **Unlock** and confirm. You are asked to confirm it is you unless you did
   in the last five minutes.

Unlocking resets the counter for that address and surface. If the lock changed
since you loaded the list (for example it expired and the address was locked
again), the unlock is refused; refresh and try again. A ban on the
[IP ban list](./ip-bans.md) is separate and still applies.

Reading the list needs **bans.read** and unlocking needs **bans.write**, both
granted for all domains.

## Options

Set in **Settings** (or with **Configure sign-in limits**). All four are global.

| Setting | Key | Default | Range |
|---|---|---|---|
| **Mailbox failed-credential limit** | [`security.signin.mailboxFailureLimit`](/admin/settings#security.signin.mailboxfailurelimit) | 10 | 1 to 100 |
| **Administrator failed-credential limit** | [`security.signin.adminFailureLimit`](/admin/settings#security.signin.adminfailurelimit) | 5 | 1 to 100 |
| **Failed-credential counting window** | [`security.signin.failureWindowSecs`](/admin/settings#security.signin.failurewindowsecs) | 900 seconds | 30 to 86400 |
| **Temporary sign-in lock duration** | [`security.signin.lockSecs`](/admin/settings#security.signin.locksecs) | 900 seconds | 30 to 86400 |

These locks are separate from the short wait after every unsuccessful attempt,
**Sign-in retry cooldown** ([`signin.retryCooldownSecs`](/admin/settings#signin.retrycooldownsecs),
3 seconds by default).

## Troubleshooting

| Symptom | Cause | Fix |
|---|---|---|
| Every user in an office is refused at sign-in at once | They share one public address, and it reached the mailbox limit. | Unlock the address. Raise the mailbox limit if this repeats. |
| All sign-ins show the proxy's address in the list | The reverse proxy is not configured as trusted, so every request appears to come from it. | Set `MW_TRUSTED_PROXIES` and `MW_FORWARDED_MODE`. |

## Related

- [IP ban list](./ip-bans.md)
- [Sign-in country policy](./signin-geography.md)
- [Security centre](./security-center.md)
- [fail2ban](/deploy/fail2ban): block repeated failures at the firewall
