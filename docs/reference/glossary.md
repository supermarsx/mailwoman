# Glossary

The terms Mailwoman uses in its screens and in this manual, each with a one-sentence meaning and a link to the page that explains it. Screen labels are written as they appear in the interface.

## A

- **Admin console**: The web interface where administrators manage the server. It is served under a console path of its own and has its own sign-in, separate from the mailbox. See [Administration](../admin/index.md).
- **AI assistant**: An optional feature that sends requests to an AI provider you configure; Mailwoman includes no model and no default provider. See [AI assistant: data flows and controls](../assist.md) and [Using the AI assistant](../user/assist.md).
- **API key**: A credential a user creates so that a script or application can act on their mailbox with a limited scope. See [API keys and OAuth](../security/api-keys-oauth.md).

## B

- **Bridge**: A connection between a mailbox and a provider that does not offer plain IMAP and SMTP: Microsoft 365 and Exchange Online (Microsoft Graph), Gmail and Google Workspace (Gmail API), and on-premises Exchange (EWS). See [Mail provider bridges](../bridges/index.md).

## C

- **Console path**: The secret URL path the admin console is served under, set with `MW_ADMIN_PATH` or stored in the database and changed with `mailwoman admin console-path`. See [`admin console-path`](cli.md#admin-console-path).
- **Country database**: The local file that maps IP addresses to countries for the country-based sign-in policies and country labels. Lookups never leave the server. See [Country database](../admin/country-database.md).

## D

- **Data request**: A person's request to see what the server holds about them, to receive a copy, or to have their data corrected, erased or restricted, or to object to a use of it. See [Data requests](../admin/data-requests.md).
- **Delivery status notification (DSN)**: A report a mail server sends about what happened to a message you sent. The **Delivery status requests** setting controls whether users may ask for one. See [Incoming delivery status reports](../deploy/delivery-status.md).
- **Domain**: A mail domain configured in the console. It decides which mail server an address signs in to, who may sign in, and which other names lead to it. See [Domains](../admin/domains.md).

## E

- **Engine mode**: The server mode in which Mailwoman talks IMAP and POP3 to the mail servers itself, selected with `MW_MODE=engine`. The other mode is **proxy mode**. See [`serve`](cli.md#serve).
- **Erasure**: Removal of the data the server keeps about one person. A **tombstone** records each erasure so that a restore from an older backup can erase the person again. See [Erasure and export coverage](../admin/erasure-coverage.md) and [`erasure`](cli.md#erasure).

## F

- **Feature profile**: A named set of setting values that an administrator assigns to users or groups. A profile does not grant mailbox access, sender aliases or credentials. See [Feature profiles](../admin/feature-governance.md).
- **Folder**: A container of messages; a message is in one folder at a time. See [Folders and labels](../user/mailbox/folders-labels.md).

## G

- **Group**: A set of users that an administrator can give setting values or a feature profile to. See [Feature profiles](../admin/feature-governance.md).

## L

- **Label**: A tag you add to a message. A message can have several labels, and labels do not move it. See [Folders and labels](../user/mailbox/folders-labels.md).
- **Linked mailbox**: Another mailbox you open from Mailwoman while you stay signed in with your primary account. Each keeps its own preferences, identities, signatures, folders and drafts. See [Linked mailboxes](../user/mailbox/linked-accounts.md).
- **Lock**: An administrator's lock on a setting. A locked value applies to everyone in its scope, and no narrower value or user choice changes it. See [How a value is chosen](../admin/settings.md#how-a-value-is-chosen).

## M

- **Masked address**: A generated alias that a person can give to a shop or newsletter instead of their real address. See [Masked email addresses](../plugins/masked-email.md).
- **MCP**: The Model Context Protocol. Mailwoman includes an MCP server so that an AI agent can work with a mailbox under authority the owner grants per tool and can revoke. See [MCP server](../security/mcp.md).

## O

- **Offline cache**: An encrypted copy of your mail, calendars, tasks, notes and contacts kept in the browser, so that you can read what was downloaded while the connection is down. See [Sync and offline access](../user/offline-sync.md).
- **Outbox**: The queue of messages sent from the web client and not yet delivered. A queued message survives closing the browser and a server restart. See [Outbox, recall and scheduled sending](../admin/outbox-recall.md).

## P

- **Passkey**: A WebAuthn credential used to sign in, for mailbox accounts and, separately, for administrator accounts. See [Passkeys](../security/passkeys.md).
- **Pin (environment)**: The way an environment variable relates to a setting: while the variable is set its value applies and the console refuses to change the stored value. A **fallback** variable is used only when nothing is stored. See [Environment variables](environment.md).
- **Plugin**: An engine plugin is a WebAssembly component that runs inside the server; a **UI plugin** extends the web client. See [Plugins](../admin/plugins.md) and [UI plugins](../plugins/ui-plugins.md).
- **Pre-added user**: An address an administrator adds in advance so that its owner can create an account at first sign-in when the **First sign-in** rule requires it. See [First sign-in and pre-added users](../admin/preprovisioned-users.md).
- **Proxy mode**: The default server mode, in which Mailwoman proxies JMAP to your mail servers. See **engine mode** and [`serve`](cli.md#serve).

## R

- **Read receipt**: A message that tells the sender you opened their message. What the server may send is controlled by the `mail.readReceipts` setting. See [Read-receipt authority and delivery outcomes](../deploy/read-receipts.md).
- **Request protection**: The checks that decide whether the server answers a request at all, before sign-in, the mailbox or the admin console: scanner-path blocks, request budgets and adaptive protection. See [Aggregate and adaptive protection](../admin/request-protection.md).
- **Restriction**: A pause on chosen kinds of processing for one person while their data stays stored. See [Restrictions and consent](../admin/consent.md).
- **Retention**: How long the server keeps each kind of stored record before it deletes it, controlled by the `retention.*` and `logging.*` settings. See [Retention limits and log rotation](../admin/retention.md).
- **Rule**: A condition and an action that sort incoming mail for you, under **Rules & filters**. See [Rules and filters](../user/rules-filters.md).

## S

- **Scope**: The level at which a setting value is stored: the whole server, a domain, a feature profile, a group, one user chosen by an administrator, or the user's own choice. The narrowest value that is set applies unless a lock says otherwise. See [How a value is chosen](../admin/settings.md#how-a-value-is-chosen).
- **Second factor**: A second proof at sign-in besides the password: an authenticator app code, a passkey, or both. Administrators set up their own. See [Administrator authenticator and passkeys](../admin/authenticator.md).
- **Server key**: The 32-byte key that seals every stored secret in the database, set with `MW_SERVER_KEY` or a key file. See [Environment variables](environment.md#database-keys-and-storage).
- **Session**: What keeps a person signed in after they enter their password. See [Sessions](../admin/sessions.md).
- **Single sign-on (SSO)**: Signing in through an OpenID Connect or SAML 2.0 identity provider that an administrator has enabled. See [Single sign-on](../admin/sso.md).
- **Snooze**: Setting a time for a message to come back to your attention. See [Snooze messages](../user/snooze.md).

## T

- **Template**: A saved message or snippet you use often. See [Templates](../user/mailbox/templates.md).
- **Tombstone**: The record of an erasure. See **erasure**.

## U

- **Unified mail**: One list that shows the mail of your main mailbox and your linked mailboxes together. See [Unified mail](../user/unified-mail.md).
- **Upstream**: The mail server that Mailwoman connects to for a domain. See [Domains](../admin/domains.md).

## Z

- **Zero-access**: A per-account mode in which a key hierarchy exists only on the user's devices and several server features that would keep a readable form of the account's mail are switched off. It does not by itself make the server store mail it cannot read. See [Zero-access storage](../security/zero-access.md).
