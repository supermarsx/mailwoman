---
screen: true
---
# Security centre

The **Security centre** collects the console's security screens in one place: an
overview of what the server recorded recently, the sign-in requirements, the address
controls (bans, locks, country policy, inbound access), the deployment-wide security
policy, the deployment checks and the session list. Open it from **Security centre**
in the console's navigation (`#/security`).

![The Overview tab of the Security centre with its row of tabs, the country database warning and the summary tiles.](/screenshots/admin-access/security-center-overview.webp){.mw-shot}

## The tabs

Each tab is also an address of its own, so you can bookmark it. A tab you have no
permission for is not shown. Tabs marked "all domains" need the capability granted
for all domains, not only for some.

| Tab | Address | What it is for | Needs |
|---|---|---|---|
| **Overview** | `#/security/overview` | Summary tiles and recent events (below). | any one of security.read, bans.read, diagnostics.read, logs.read, admins.read, audit.read; all domains |
| **Aggregate and adaptive protection** | `#/security/protection` | Request budgets, unusual-request rules, protocol guard and admin-probe protection. See [Request protection](./request-protection.md). | bans.read, all domains |
| **Two-factor & sign-in requirements** | `#/security/requirements` | Who must use a second factor, and the passkey and session policies. [Below](#two-factor-sign-in-requirements). | security.read, all domains |
| **IP ban list** | `#/security/bans` | [IP ban list](./ip-bans.md) | bans.read, all domains |
| **Temporary sign-in locks** | `#/security/locks` | [Temporary sign-in locks](./signin-locks.md) | bans.read, all domains |
| **Sign-in country policy** | `#/security/geography` | [Sign-in country policy](./signin-geography.md) | bans.read, all domains |
| **Country database** | `#/security/geoip` | [Country database](./country-database.md) | security.read, all domains |
| **Sign-in policy** | `#/security/signin` | Who may sign in for the first time. See [Pre-provisioned users](./preprovisioned-users.md). | settings.read |
| **Security policy** | `#/security/policy` | [Security policy](./security-policy.md) | security.read, all domains |
| **Inbound access** | `#/security/inbound` | [Inbound access](./inbound-allowlists.md) | security.read, all domains |
| **Deployment checks** | `#/security/deployment` | [Deployment checks](./deployment-security.md) | diagnostics.read, all domains |
| **Console address** | `#/security/console` | The console's private path. See [Moving the admin console](/deploy/README#moving-the-admin-console). | security.read, all domains |
| **Client access** | `#/security/client-access` | [Client access](./client-access.md) | settings.read, all domains |
| **Sessions** | `#/security/sessions` | [Sessions](./sessions.md) | sessions.read |

Changing something on a tab needs the matching write capability as well, and some
changes (bans, unlocks, inbound access, the console address) ask you to confirm it is
you first.

::: tip
Always open console screens with the `#` form shown in the table
(`https://mail.example.org/<console path>#/security/bans`). Loading a path without
the `#`, such as `<console path>/security`, is treated as probing for hidden
addresses and blocks your address for five minutes by default (see
[Request protection](./request-protection.md)).
:::

## Overview

The **Overview** tab shows what the server has recorded recently. Each tile opens the
screen with the details:

| Tile | Shows |
|---|---|
| **Two-factor adoption** | The share of accounts with a second factor, split into passkey and authenticator app. |
| **Failed sign-ins, 24 h** | Failed and blocked sign-ins in the last 24 hours and 7 days. |
| **Request protection actions, 24 h** | Actions request protection took in the last 24 hours, with the counts it only observed. |
| **Temporary sign-in locks** | Addresses locked now after rejected passwords. |
| **Probing blocks** | Sources blocked for probing for the console, and requests refused. |
| **Admin sessions** | Administrators signed in now. |
| **Security posture** | How many [deployment checks](./deployment-security.md) need review. |

Below the tiles are panels with the details: second-factor adoption (including
administrators without a second factor), failed and blocked sign-ins per hour and per
day, recent security events from the audit log, the latest administrator sign-ins,
the security posture rows, and whether sessions survive a server restart
("Sessions survive restarts: yes" or "no", with the reason; see
[Sessions](./sessions.md#server-restarts)). **Refresh** reads the figures again. You
see only the tiles your capabilities allow.

### The country database warning

When a feature that needs a country database is switched on and no database is
loaded, every Security centre tab starts with **Country features are on but no
country database is loaded** and lists those features. On a new installation it lists
two features that are on by default:

- **Unknown-country admin console block** ([`security.trust.adminGeoEnabled`](/admin/settings#security.trust.admingeoenabled)):
  without a database it blocks nothing.
- **Session country-change alerts** ([`security.alerts.countryChangeEnabled`](/admin/settings#security.alerts.countrychangeenabled)):
  without a database it cannot tell a country change.

Install a database under **Country database**, or turn those settings off, to remove
the warning. See [Country database](./country-database.md).

## Two-factor & sign-in requirements

![The Two-factor & sign-in requirements tab.](/screenshots/admin-access/security-center-requirements.webp){.mw-crop}

1. **Require two-factor for the whole deployment**: every account needs a second factor.
2. **Per-domain requirements**: require it for the accounts of one domain only.
3. **Passkey and session policies**: links to the settings below.
{.mw-callouts}

A second factor is a passkey or an authenticator app. The requirement applies to an
account when any of these is on: the deployment-wide rule, the rule for the account's
domain, or the setting **Require two-factor authentication**
([`twofa.required`](/admin/settings#twofa.required)), which can also be set for one
account. The deployment-wide rule is the same as **Require a second
factor for every account** on the [Security policy](./security-policy.md) tab.

When a required account has a second factor, sign-in asks for it. When it has none,
the sign-in screen asks the person to set one up before the mailbox opens: an
authenticator app (scan the QR code or type the setup key, then enter a code), or a
passkey where passkeys are allowed and the browser supports them. The desktop and
Android apps offer only the authenticator app. The person then saves the one-time
recovery codes, confirms that they did, and is signed in. The setup has to be
finished within five minutes of entering the password; otherwise the screen returns
to the sign-in form.

The setup is done by whoever next signs in with the account's password, so an
account whose password may be known to someone else should get its factor from the
account holder, in their own settings, before you require it.

### Passkey and session policies

The list links to these settings. Each can be set for the whole server, a domain or
an account, and locked:

| Setting | Key | Default | What it does |
|---|---|---|---|
| **Require two-factor authentication** | [`twofa.required`](/admin/settings#twofa.required) | off | As above. |
| **Allow passkeys** | [`twofa.passkeysEnabled`](/admin/settings#twofa.passkeysenabled) | on | Off: passkeys are not offered as a second factor. An account whose only factor is a passkey then cannot sign in. |
| **Passkey authenticator type** | [`twofa.passkeyAttachment`](/admin/settings#twofa.passkeyattachment) | `any` | `platform`: passkeys stored on the device (Windows Hello, Touch ID, Android screen lock). `cross-platform`: security keys and phones. |
| **Passkey user verification** | [`twofa.passkeyUserVerification`](/admin/settings#twofa.passkeyuserverification) | `preferred` | `required`: the passkey must check a PIN or biometric at registration and every sign-in. |
| **Discoverable passkeys** | [`twofa.passkeyResidentKey`](/admin/settings#twofa.passkeyresidentkey) | `discouraged` | `required`: the passkey must be stored on the authenticator. |
| **Passkeys added before the current policy** | [`twofa.passkeyLegacy`](/admin/settings#twofa.passkeylegacy) | `allow` | What happens to an older passkey that does not meet a stricter policy. |
| **Removing the last second factor** | [`twofa.lastFactorRemoval`](/admin/settings#twofa.lastfactorremoval) | `allow` | `block`: a person cannot remove their only passkey or authenticator app until they add another. |
| **Remembered device lifetime** | [`twofa.rememberDays`](/admin/settings#twofa.rememberdays) | 0 (off) | Days a browser may skip the second factor after a correct password (up to 90). |
| **Concurrent sessions** | [`session.maxConcurrent`](/admin/settings#session.maxconcurrent) | 10 | Mailbox sessions per account. |
| **Session idle timeout** | [`session.idleSecs`](/admin/settings#session.idlesecs) | 7 days | See [Sessions](./sessions.md). |
| **Session lifetime** | [`session.absoluteSecs`](/admin/settings#session.absolutesecs) | 0 (none) | See [Sessions](./sessions.md). |

What the server can check about a passkey:

- **User verification** is signed by the authenticator, so `required` is enforced.
- **Authenticator type** is what the browser reports. Mailwoman does not ask for
  attestation, so it cannot prove which hardware was used; it refuses a browser that
  reports the wrong type, or none while the setting is not `any`.
- **Discoverable passkeys**: a registration that the browser reports as not
  discoverable is refused when the setting is `required`; a browser that reports
  nothing is trusted.
- In engine mode a passkey can also be used to sign in without a password, so
  registration there always asks for a discoverable, user-verified passkey; the
  settings can only make this stricter.

With **Passkeys added before the current policy** at `allow`, an older passkey keeps
working, and the person's settings mark it and ask them to add a new one. With
`refuse`, it is refused with a message that says so; the person can still sign in
with an authenticator app code or a recovery code. Check that people have another
factor before you choose `refuse`.

With **Removing the last second factor** at `block`, or when two-factor is required
for the account, removing the only passkey or authenticator app is refused; recovery
codes do not count as a factor. Administrators cannot remove a user's factors.
Refusals are recorded in the audit log as `passkey-policy-refused`.

## Related

- [Security policy](./security-policy.md)
- [Sessions](./sessions.md)
- [Request protection](./request-protection.md)
- [Admin console security](/security/admin-panel)
- [Administrator authenticator and passkeys](./authenticator.md)
