---
screen: true
---
# Sign-in country policy

The sign-in country policy allows or refuses new sign-ins by the country of the
source IP address, looked up in a country database on the server. It is off by
default. The **Sign-in country policy** tab of the **Security centre**
(`#/security/geography`) shows what the policy would decide for your own address
right now, so you can check it before turning it on.

![The Sign-in country policy tab of the Security centre on a server without a country database.](/screenshots/admin-access/signin-geography-page.webp){.mw-shot}

The diagnostic, on a server where no country database is loaded yet:

![The Sign-in country policy tab on a server without a country database.](/screenshots/admin-access/signin-geography-diagnostic.webp){.mw-crop}

1. **Configure country policy** opens the policy settings.
2. **Configure existing-session checks** opens the setting that also checks requests from signed-in sessions.
3. **Your administrator sign-in decision** (and the mailbox decision below it): what the policy decides for the address you are using now.
{.mw-callouts}

The diagnostic shows the **Trusted source IP** (your address as the server resolves
it through its trusted-proxy settings), the **Resolved country**, the state of the
**Local country database**, the scope, list mode, country codes and unmapped-address
policy, and the two decisions. It only ever looks up your own address; you cannot
enter another one. Reading it needs **bans.read** granted for all domains; changing
the policy needs permission to change settings.

## Before you start

The policy needs a country database. None is bundled, and lookups never leave the
server. Download or upload one under **Security centre** > **Country database**, or
point `MW_GEOIP_DB` at a MaxMind-format country or city database; see
[Country database](./country-database.md). A database installed from the console
takes effect without a restart. A file named by `MW_GEOIP_DB` is read once and kept
by path, so restart the server after replacing it.

While no database is loaded, a warning at the top of every Security centre tab,
**Country features are on but no country database is loaded**, lists the
country-dependent features that are switched on.

## Turn on the policy

1. Install a country database (see above).
2. Open **Security centre** > **Sign-in country policy** and select
   **Configure country policy**.
3. Set **Sign-in country codes** to uppercase two-letter codes, for example `PT`
   and `ES`.
4. Set **Country allow or deny list**: `allow` admits only the listed countries,
   `deny` refuses the listed countries.
5. Set **Sign-in country policy scope** to `mailbox`, `admin` or `all`.
6. Return to the diagnostic, select **Refresh diagnostic**, and check that
   **Your administrator sign-in decision** is **Allow** before you include the
   console in the scope.

## How a decision is made

For a sign-in on a surface in scope:

| Source | Decision |
|---|---|
| Private, loopback or link-local address | Always allowed: a local network has no country. |
| Country found, policy `allow` | Allowed if the country is in the list, refused otherwise. |
| Country found, policy `deny` | Refused if the country is in the list, allowed otherwise. |
| Address not in the database | **Unmapped source address policy** (`deny` by default). |
| No database loaded, or it cannot be read | **Instance sign-in policy without a database** (`block` by default: refused). |

A refused sign-in gets "Sign-in is not permitted from this source location"; with no
usable database it gets a "temporarily unavailable" answer.

The check runs before Mailwoman contacts the mail server, on password sign-in, the
second-factor step, passkey sign-in and single sign-on. It does not end sessions
that already exist, unless you also turn on **Enforce country policy on existing
sessions**: then requests from signed-in mailbox sessions, native app sessions and
signed message content are checked too, and refused ones are answered with an
error. That setting does not delete sessions, and a refused client can still sign
out.

A country comes from an IP database. VPNs, mobile networks and proxies can put a
person in another country, and databases are sometimes wrong. Treat the policy as a
filter on addresses, not as proof of where someone is.

## Options

All of these are global settings.

| Setting | Key | Default |
|---|---|---|
| **Sign-in country policy scope** | [`security.signin.geoScope`](/admin/settings#security.signin.geoscope) | `off` (`mailbox`, `admin`, `all`) |
| **Country allow or deny list** | [`security.signin.geoMode`](/admin/settings#security.signin.geomode) | `allow` |
| **Sign-in country codes** | [`security.signin.geoCountries`](/admin/settings#security.signin.geocountries) | empty |
| **Unmapped source address policy** | [`security.signin.geoUnknown`](/admin/settings#security.signin.geounknown) | `deny` |
| **Instance sign-in policy without a database** | [`security.signin.geoNoDatabase`](/admin/settings#security.signin.geonodatabase) | `block` |
| **Enforce country policy on existing sessions** | [`security.signin.geoEnforceSessions`](/admin/settings#security.signin.geoenforcesessions) | off |

::: warning
With the policy in `allow` mode and an empty country list, every sign-in in scope
from a public address is refused. With the default `block` and no database, every
sign-in in scope from a public address is refused.
:::

A separate, per-account policy, **Per-account sign-in country policy**
([`security.signin.accountGeoMode`](/admin/settings#security.signin.accountgeomode),
off by default), can refuse mailbox sign-ins from a country an account has not used
before, or from outside the account's allowed countries. It applies only to mailbox
sign-ins, not to the console.

## Troubleshooting

| Symptom | Cause | Fix |
|---|---|---|
| The diagnostic shows **Unavailable — active policies refuse sign-in** | No country database is loaded. | Install one under **Country database**. |
| Everyone appears to come from the same address | The reverse proxy is not configured as trusted. | Set `MW_TRUSTED_PROXIES` and `MW_FORWARDED_MODE` (see [Reverse proxy](/deploy/reverse-proxy)). |
| You are locked out of the console by the policy | Your address is in a refused country, or the database is missing and the scope includes `admin`. | Sign in from an allowed address, or from a private or loopback address (for example through an SSH tunnel to the server's own port), which the policy always allows. Then change the scope. |

## Related

- [Country database](./country-database.md)
- [Temporary sign-in locks](./signin-locks.md)
- [Security centre](./security-center.md)
- [Settings reference](./settings.md)
