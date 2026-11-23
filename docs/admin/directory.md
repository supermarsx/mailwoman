---
screen: true
---
# LDAP directory configuration

The **LDAP / GAL directory** tab of **Integrations** lists the LDAP directory servers Mailwoman reads for address suggestions, group expansion, S/MIME certificates and photos. What Mailwoman does with a directory is described in [LDAP directory](../deploy/ldap.md); this page describes the form.

![The LDAP / GAL directory tab of Integrations with one new directory being filled in: server URL, search base, service bind DN, transport security and connection policy.](/screenshots/admin-access/directory-editor.webp){.mw-shot}

## Before you start

- Viewing the tab needs a role with `integrations.read`; saving needs `integrations.write`. Both must cover the whole deployment, not only some domains.
- Saving does not contact the directory, so it does not tell you whether the server is reachable or the password is right.
- The running server reads the directory configuration only when it starts. After saving, restart the server. Until then the tab shows "Saved configuration differs from the running directory. A server restart is required."

## Add a directory

1. Open **Integrations** and the **LDAP / GAL directory** tab (`{console}#/integrations/directory`).
2. Click **Add directory**. A new entry appears with **Block connections** selected and **Include in directory searches** off.
3. Fill in the entry:

![One directory entry: server URL, search base, bind DN, transport security, priority, connection policy, the two checkboxes, the stored-password action with a new password field, and the five attribute fields.](/screenshots/admin-access/directory-endpoint.webp){.mw-crop}

1. **Server URL**: `ldaps://host[:port]` for LDAPS, or `ldap://host[:port]` for StartTLS or plaintext. The URL may not contain a user name, password, path, query or fragment; the search base goes in **Search base DN**.
2. **Transport security**: **LDAPS**, **StartTLS** or **Plaintext (anonymous only)**. It must match the URL scheme. Choose plaintext only when the directory allows anonymous reads and the network path is trusted: no password can be stored for it.
3. **Priority**: lower numbers are asked first; see [Several directories](../deploy/ldap.md#several-directories).
4. **Connection policy**: see [Connection policy](#connection-policy).
5. **Include in directory searches**: off leaves the entry saved but unused: the server does not connect to it at all.
6. **Allow private LAN and loopback addresses**: needed when the directory's address is in a private range or on the same machine.
7. **Stored password action**: **Keep unchanged**, **Set or replace password** (shows **New service password**), or **Clear stored password**.
8. The five attribute fields: leave them empty to use the defaults.
{.mw-callouts}

4. Click **Save directory configuration**.
5. Restart the server.

To remove a directory, click **Remove from configuration** in its entry and save. Its stored password is removed with it.

If someone else saved the configuration since you opened the tab, saving is refused; click **Reload saved configuration** and make your change again.

## Connection policy

| Option | Effect |
|---|---|
| **Block connections** | The server never connects to this directory. Every new entry starts with this. |
| **Allow guarded direct connections** | The server connects directly, after checking every address the host name resolves to; if any of them is refused, it does not connect. Public addresses are allowed. Private IPv4 ranges, IPv6 unique-local addresses and loopback are allowed only when **Allow private LAN and loopback addresses** is ticked. Everything else (for example link-local addresses, including the cloud metadata address `169.254.169.254`) is refused. The TLS certificate is still checked against the host name in the URL. |

An entry whose stored password cannot be read when the server starts is treated as blocked.

While an outbound proxy is active (see [Outbound connections](../security/egress.md)), every LDAP connection is refused, because LDAP cannot be sent through the proxy. The tab then says "LDAP connections are blocked while an outbound proxy is active."

## Service account

**Service bind DN** is the account Mailwoman binds as before it searches. Without one, searches are anonymous.

- A password can be stored only together with a bind DN and LDAPS or StartTLS.
- The password is stored encrypted with the server key. It is never shown again, and the tab only says "A sealed service password is stored." or "No service password is stored."
- When you change the **Server URL** or **Service bind DN** of an entry that has a stored password, choose **Set or replace password** or **Clear stored password** as well; the server refuses to send the old password to a new place.
- A password set through the environment (`MW_DIRECTORY_BIND_DN` and `MW_DIRECTORY_BIND_PW`, both set) is used for the entry whose bind DN matches. The tab shows "The environment pins the service credential for this bind DN:" with the DN, and you cannot change that entry's URL, bind DN or password in the console.

## Options

| Field | Default | Notes |
|---|---|---|
| **Search base DN** | empty | The subtree every search starts from. |
| **Display name attribute** | `displayName` (falls back to `cn`) | |
| **Email attribute** | `mail` | Also used to find an entry by address. |
| **Group member attribute** | `member` | `uniqueMember` and `member` are read as well. |
| **Certificate attribute** | `userCertificate;binary` | |
| **Photo attribute** | `jpegPhoto` | Active Directory often uses `thumbnailPhoto`. |

At most 32 directories can be saved. Attribute names may contain letters, digits, `;` and `-`.

## Not configured here

- Whether sign-in passwords are checked against the directory: the environment variable `MW_LDAP_BIND_AUTH`; see [Checking passwords against the directory](../deploy/ldap.md#checking-passwords-against-the-directory).
- The directory configuration and its passwords are not copied when you move a deployment with `migrate-store`; set them up again on the new server.

## Troubleshooting

| Symptom | Cause | Fix |
|---|---|---|
| Address suggestions do not appear after saving | The server still runs the old configuration. | Restart the server. |
| Suggestions do not appear after a restart | The entry is blocked, not included in searches, or an outbound proxy is active. | Choose **Allow guarded direct connections**, tick **Include in directory searches**, and check the outbound proxy notice. |
| A directory on `10.x`, `192.168.x` or `localhost` is never reached | Private and loopback addresses need consent. | Tick **Allow private LAN and loopback addresses**. |
| Saving answers that the credential must be replaced or cleared | The URL or bind DN changed while a password is stored. | Choose **Set or replace password** or **Clear stored password**. |

## Related

- [LDAP directory](../deploy/ldap.md)
- [Kerberos](../deploy/kerberos.md)
- [Single sign-on](./sso.md)
- [Outbound connections](../security/egress.md)
