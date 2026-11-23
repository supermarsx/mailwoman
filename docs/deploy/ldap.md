# LDAP directory

Mailwoman can read an LDAP directory (for example OpenLDAP or Active Directory) to offer a global address list while people write messages, to expand distribution groups, and to look up the S/MIME certificates and photos people publish there. It can also check each sign-in password against the directory. It never writes to the directory.

You set up the directory servers in the admin console, under **Integrations** > **LDAP / GAL directory**; see [LDAP directory configuration](../admin/directory.md). This page explains what Mailwoman does with the directory.

## What the directory is used for

| Feature | What Mailwoman asks the directory |
|---|---|
| Address suggestions in the message editor | A subtree search under the search base for entries whose display name, email, `cn`, `givenName`, `sn` or `sAMAccountName` contains what the person typed. Entries without an email address are left out, except groups. |
| Expanding a distribution group | Reads the group's `member` and `uniqueMember` values (or the attribute you map), then reads each member; nested groups are followed up to 10 levels and at most 5,000 members are collected. In the message editor, picking a group offers to replace it with its members. |
| S/MIME certificates | Reads `userCertificate;binary` (or the attribute you map) of the entry with the person's address. The **Contact security** panel of a contact lists what it finds. |
| Photos | Reads `jpegPhoto` (or the attribute you map) of the entry with the address. |
| Checking sign-in passwords | Optional, off by default. See [Checking passwords against the directory](#checking-passwords-against-the-directory). |

An entry counts as a group when its object class is `group`, `groupOfNames`, `groupOfUniqueNames` or `groupOfURLs`, or when it has the group member attribute.

Search results are not cached: every suggestion, expansion and lookup asks the directory again.

The address suggestions appear only when a directory is configured. The status check the web app uses for this does not contact the directory.

## Several directories

You can list up to 32 directory servers. Mailwoman queries them in order of **Priority**, lowest number first:

- Address suggestions combine the results of all of them; when two directories return the same address, the one with the lower priority number wins.
- If one enabled directory fails during a search, the whole search fails.
- A certificate lookup collects certificates from all directories; a photo lookup uses the first photo found.

## Connection security

| Transport security | URL | What happens |
|---|---|---|
| LDAPS | `ldaps://host` (port 636 by default) | TLS from the first byte |
| StartTLS | `ldap://host` (port 389 by default) | Connects in plain text, then upgrades to TLS before binding |
| Plaintext (anonymous only) | `ldap://host` | No encryption. A service password cannot be saved for it, and password checks are refused. |

Mailwoman verifies the directory's certificate against the certificate store of the operating system the server runs on (the container image's store in Docker). There is no setting for a separate CA file: if your directory uses a private CA, add that CA to the system store.

Every connection is opened fresh for each request. Before connecting, Mailwoman applies the connection policy you chose for that server (see [LDAP directory configuration](../admin/directory.md#connection-policy)). LDAP connections cannot go through an outbound proxy: while an outbound proxy is active, every LDAP connection is refused.

## Checking passwords against the directory

With `MW_LDAP_BIND_AUTH=1` (or `true`) in the server's environment, every mailbox sign-in with a password is first checked against the directory:

1. Mailwoman searches each directory (as the service account, if one is set) for exactly one entry whose `mail` (or the mapped email attribute), `uid`, `sAMAccountName` or `userPrincipalName` equals the sign-in name. If none or several match, it moves to the next directory. A sign-in name that contains `=` is not searched for: it is used as the entry's DN as typed.
2. It binds as that entry with the password typed.
3. If a directory accepts the bind, the check passes. If no directory accepts it, the sign-in is refused.

The check comes **before** the usual check with the mail server, not instead of it: the person still needs a password the mail server accepts. An empty password is always refused, without contacting the directory.

These count as a wrong password, and towards the [temporary sign-in locks](../admin/signin-locks.md): an empty password, and, when no directory failed, no directory with exactly one matching entry or a bind refused with "invalid credentials" (code 49). Any other failure (directory unreachable, busy, TLS error, or no usable directory) refuses the sign-in as well, but is recorded as "directory unavailable" rather than as a wrong password. The person sees the same message in both cases.

::: warning
The environment variable is the only switch. The setting `signin.ldapBindPrecheck` that appears in the [settings reference](../admin/settings.md) is not read by the server, and the directory screen has no control for it.
:::

The check applies to the password sign-in only. Sign-ins through [single sign-on](../admin/sso.md) or [header authentication](./reverse-proxy.md#header-authentication) do not consult the directory.

## Password changes

Changing a password in the directory is a separate feature with its own settings (`MW_PASSWD_BACKEND=ldap3062`); it does not use the directory servers configured here. See [Password changes](../security/password-change.md).

## Environment variables

| Variable | Effect |
|---|---|
| `MW_LDAP_BIND_AUTH` | `1` or `true` turns on the password check against the directory. Off by default. |
| `MW_DIRECTORY_BIND_DN`, `MW_DIRECTORY_BIND_PW` | Together, supply the service password for the directory whose **Service bind DN** equals `MW_DIRECTORY_BIND_DN`, instead of a password stored in the console. |

Directory changes in the console and these variables take effect after a server restart.

## Related

- [LDAP directory configuration](../admin/directory.md)
- [Kerberos](./kerberos.md)
- [Temporary sign-in locks](../admin/signin-locks.md)
- [Password changes](../security/password-change.md)
