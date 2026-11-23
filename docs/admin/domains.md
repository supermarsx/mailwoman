---
screen: true
---
# Domains

**Domains** in the admin console decides, for each mail domain, which mail server an address signs in to, who may sign in, and which other names (domain aliases and login host names) lead to the domain. Users never choose a server themselves: the server picks it from the address they type.

![The Domains screen: Quick add and Guided add buttons, a search field, and the table of example.com, example.net and example.org with their aliases, status, connections and row actions.](/screenshots/admin-access/domains-list.webp){.mw-shot}

1. **Quick add** creates a domain from its name only. It then uses the server default mail server until you configure one.
2. **Guided add** opens a step-by-step setup that can also set the mail server. See [Guided domain setup](./guided-domain-setup.md).
3. A domain name opens the domain's configuration pages.
4. The row actions are **Disable domain** (or **Enable domain**), **Edit**, **Clone** and **Delete**.
{.mw-callouts}

## Before you start

- The server runs in one mode for all domains, shown on each domain as **Server mode: engine** or **Server mode: proxy**. In engine mode a domain's mail server is IMAP or POP3 (with optional SMTP and ManageSieve, or Exchange Web Services); in proxy mode it is a JMAP server. The mode is set when the server is installed (`MW_MODE`), not here.
- Your admin role decides what you can change. Creating, enabling and disabling a domain needs the domain write permission; changing its mail servers needs the upstream permission for all domains; changing its sign-in lists needs the sign-in policy permission for that domain; deleting needs the delete permission, and forcing a deletion needs it for all domains.

## How a sign-in finds its mail server

When someone signs in, the server takes the address they typed and:

1. If it has no `@`, adds a domain: the domain whose **Login hostnames** list contains the host name the browser used, otherwise the preferred domain. Without either, the name stays as typed.
2. Replaces a domain alias by its parent domain, so `alex@corp.example.org` is handled as `alex@example.org`.
3. Uses the domain's own mail server if the domain is listed here. A listed domain that is disabled, or has no enabled mail backend, refuses the sign-in.
4. For a domain that is not listed: refuses it when **Sign-in domains** ([`signin.domainMode`](/admin/settings#signin.domainMode)) is `listed`; otherwise uses **Default mail server** ([`upstream.default`](/admin/settings#upstream.default)), then the `MW_UPSTREAM_DEFAULT` environment variable, then, in proxy mode, the JMAP server when `MW_JMAP_UPSTREAMS` lists exactly one.

If no step finds a server, the sign-in screen answers "This address is not set up on this server".

::: warning Login host names and MW_PUBLIC_URL
The host name used in step 1 is the one in `MW_PUBLIC_URL` when that variable is set; otherwise the forwarded host from a trusted proxy, otherwise the request's `Host`. With `MW_PUBLIC_URL` set, every bare username is matched against that single host name.
:::

## Add a domain

1. Select **Quick add**, type the domain name and select **Create and configure**. The domain is enabled and uses the server default mail server.
2. Open **Connections and discovery** to give it its own mail server (see below), or use **Guided add** instead to do both at once.

When accounts exist whose domain has no usable mail server, the screen shows the card **Accounts blocked by missing connection settings**: how many accounts each domain has waiting and, when known, the server they used before. **Configure** followed by the domain name opens that domain.

## A domain's configuration pages

![The configuration pages of example.org (Overview, Connections and discovery, Sign-in policy, Domain aliases, Login hostnames, Diagnostics) above the overview counts of backends, aliases, login hosts and sign-in rules.](/screenshots/admin-access/domains-overview.webp){.mw-crop}

**Overview** counts the domain's connected accounts (engine mode only), backends, aliases, login host names and sign-in rules, and has the buttons **Enable domain** or **Disable domain**, **Set preferred domain** or **Clear preferred domain**, **Delete** and **Reload server version**.

### Connections and discovery

A domain's mail servers are a list of **Service backends**. Each backend serves one role: **Mail access** (IMAP, POP3, JMAP or EWS), **Message submission** (SMTP, or JMAP in engine mode) or **Server rules** (ManageSieve). A domain holds at most 32 backends.

![The Service backends list of example.org: the failover checkbox turned on, and three backends, imap-primary, imap-standby and smtp, with protocol, host and port, priority and status; imap-primary and smtp are marked Selected.](/screenshots/admin-access/domains-backends.webp){.mw-crop}

1. **Add backend** opens an empty backend editor.
2. **Fail over to the next mail backend when the selected one cannot be reached** (see [Failover](#failover)).
3. **Selected** marks the backend each role uses now.
{.mw-callouts}

For each role the server selects the enabled backend marked **Preferred for this service**; without one, the lowest **Priority** number; on a tie, the alphabetically first **Backend ID**. A password that one backend rejects is never tried on another.

A domain created without backends shows a simpler form instead: **Incoming mail protocol** (IMAP, POP3 or **Use server default**), **Configure SMTP**, **Configure SIEVE**, or in proxy mode **JMAP session URL**. Adding or editing a backend converts the domain to the backend list; from then on it no longer falls back to the server default.

To add or change a backend:

1. Select **Add backend**, or the edit button of a backend row.
2. Fill in **Service and selection**: **Backend ID** (letters, digits, hyphens and underscores, up to 64; it cannot be renamed later), **Protocol and service**, **Priority**, **Enabled** and **Preferred for this service**.
3. Fill in **Connection**: **Hostname or IP address**, **Port** and **Connection security**: **TLS**, **STARTTLS** or **Plaintext (explicit insecure choice)**.
4. Optionally set **Authentication username format** (below) and select **Preview authentication username** to see the exact name that will be sent, without contacting the server.
5. Select **Save backend**.

![The backend editor for imap-primary: Service and selection with ID, protocol, priority, Enabled and Preferred, then the Connection fields host, port, connection security and private-network permission.](/screenshots/admin-access/domains-backend-editor.webp){.mw-shot}

Connection details:

- A host name is resolved on every new connection. Private network and loopback addresses are refused unless **Permit this endpoint on a private network** is checked; link-local and cloud metadata addresses are always refused.
- Certificates and host names are always verified. **Customize transport security** lets you give an ordered list of methods to try (**Auto, with explicit priority**), a **TLS server name override** for connecting to an IP address, and **Additional trusted CA certificates (PEM)** (public certificates only, at most 64 KiB). Auto moves to the next method only when the server says the method is not supported; a certificate error or time-out stops the attempt. Plaintext is used only if listed and **Explicitly permit plaintext** is checked.
- **Customize authentication methods** limits the password mechanisms (SASL PLAIN, LOGIN, SCRAM-SHA-256, SCRAM-SHA-256-PLUS, the IMAP LOGIN command, POP3 USER/PASS). OAuth and client certificates are not available for domain backends.
- For SMTP backends, **DANE verification** and **MTA-STS verification** can be required. See [SMTP transport policies](./smtp-transport-policy.md).
- **Authentication username format**: **Full email address** (the default), **Local username**, **DOMAIN\username**, **username@realm** or **Custom template** with `{user}`, `{domain}`, `{email}` and `{realm}`. It changes only the name sent to that backend, never the account's address. JMAP mail access always uses the full address.

Changing a domain's mail server or its enabled state disconnects the domain's engine-mode accounts; users sign in again and connect to the new server.

**Detect connection settings** asks the server to look up the domain's published mail settings and shows them as a proposal. Nothing is saved until you select **Use proposal in editor** and then review and apply. A proposal that needs OAuth cannot be applied.

### Failover

Failover applies to IMAP and POP3 mail backends and is off by default. With **Fail over to the next mail backend when the selected one cannot be reached** checked:

- Before sending the password, the server checks the selected backend without credentials (DNS, connection, TLS or STARTTLS, greeting). Only on a DNS, connection, time-out, TLS or certificate failure does it check the next enabled mail backend of the same protocol, by priority; at most three backends per attempt.
- The first backend that passes the check is the only one that receives the password. If it rejects the password, the sign-in fails.
- A backend that failed its check is moved behind the others for 30 seconds, doubling with each further failure up to 15 minutes, then tried first again. This record is kept in memory and starts empty after a restart.
- All backends in the chain must use the same connection security, authentication methods and username format; saving a chain that differs is refused. Turning failover on asks existing accounts of the domain to sign in again.
- Each event is logged at WARN level under `mailwoman::backend_failover` with `event` set to `check-failed`, `failover`, `selected-backend-resting` or `returned-to-selected`, the domain and the backend IDs.

SMTP, ManageSieve, JMAP and EWS backends never fail over.

### Sign-in policy

Each domain has an **Allowlist** and a **Blocklist** of address patterns, one per line.

![The Allowlist field containing *@example.org and the Blocklist field containing former.*@example.org.](/screenshots/admin-access/domains-policy.webp){.mw-crop}

- Matching ignores letter case; `*` matches any run of characters (`*@example.org`, `sales*@example.org`). There are no other wildcards.
- The mail server must accept the password first. Then a disabled domain refuses; a matching **Blocklist** entry refuses, even if the **Allowlist** also matches; a non-empty **Allowlist** refuses every address that matches none of its entries. An empty list adds no restriction.
- An alias address is checked as the parent domain's address.
- Each list holds at most 1000 patterns of up to 320 characters.
- The lists are also checked on later requests of an open mailbox session, so adding an address to the **Blocklist** or disabling the domain signs that user out.

A refused sign-in may show as "Invalid credentials" when **Refusal messages** ([`signin.refusalDetail`](/admin/settings#signin.refusalDetail)) is set to give the same answer as a wrong password.

### Review and apply

Changes on **Connections and discovery** and **Sign-in policy** are not saved until you review them:

1. Select **Review changes**. The server checks the draft and shows each changed setting with its value before and after.
2. Select **Apply changes**.

![The Apply domain changes? dialog listing the Sign-in blocklist before and after the change, with Cancel and Apply changes buttons.](/screenshots/admin-access/domains-review.webp){.mw-crop}

If someone else changed the domain in the meantime, applying is refused with "This domain changed on the server. Your draft is preserved." Select **Reload server version** and make the change again.

### Domain aliases

An alias is another domain name for the same configuration: same mail servers, same sign-in rules. The username is sent to the mail server as the user typed it.

![The Domain aliases page of example.org with the alias corp.example.org and the Add alias button.](/screenshots/admin-access/domains-aliases.webp){.mw-crop}

Select **Add alias** and enter the full alias domain name. The row buttons rename the alias in one step (**Edit alias**), create another alias for the same domain (**Clone alias**) or remove it. A name that is already a domain cannot become an alias, and a name cannot be an alias of two domains.

### Login hostnames

A login host name lets people on that web address sign in with only the part before `@`.

![The Login host mappings page of example.org with the host webmail.example.org and a field to add another.](/screenshots/admin-access/domains-hosts.webp){.mw-crop}

Enter the host name under **New login host** and select **Add**. A host name belongs to one domain only. It takes precedence over the preferred domain (see the warning above about `MW_PUBLIC_URL`).

### Diagnostics

![The Connection and DNS checks card with the buttons Test saved connections and Check DNS.](/screenshots/admin-access/domains-diagnostics.webp){.mw-crop}

- **Test saved connections** connects to each saved endpoint and reports reachability, time taken, TLS version, certificate expiry, the greeting and the advertised capabilities. It sends no credentials and does not test the server default or EWS sign-in.
- **Check DNS** looks up the domain's MX, SPF, DMARC and mail SRV records.

Apply or discard a draft first; the checks use the saved settings. Each admin session may run six checks of each kind per minute. The checks need the diagnostics permission and the upstream permission for all domains.

## Preferred domain, disabling and deleting

- **Set preferred domain** makes this domain the one added to bare usernames when no login host name matches. Only one domain is preferred, and a disabled domain cannot be made preferred.
- **Disable domain** refuses new sign-ins, ends open mailbox sessions of the domain on their next request, and disconnects its engine-mode accounts. Mail, accounts and settings stay stored. **Enable domain** reverses it.
- **Delete** removes the domain's configuration, aliases and login host names after you type the domain name. It is refused while the domain is preferred or has signed-in accounts, unless you check **Force deletion even if this domain is preferred or has signed-in accounts** (needs the delete permission for all domains).
- **Clone** copies the connections, discovery setting and sign-in lists to a new domain, which is created disabled. Aliases, login host names and the preferred flag stay with the original. It needs both the upstream and the sign-in policy permission.

## Options

| Label as shown | What it does | Default | Who can change it |
|---|---|---|---|
| **Default mail server** ([`upstream.default`](/admin/settings#upstream.default)) | Mail server for domains not listed here | none (`MW_UPSTREAM_DEFAULT` when set) | admin, Settings |
| **Sign-in domains** ([`signin.domainMode`](/admin/settings#signin.domainMode)) | `any`: unlisted domains use the default mail server; `listed`: only listed domains may sign in | `any` | admin, Settings |
| **Refusal messages** ([`signin.refusalDetail`](/admin/settings#signin.refusalDetail)) | Whether a refused sign-in says why or looks like a wrong password | explain | admin, Sign-in policy |
| **Discovery policy** (per domain) | Stored, but nothing reads it: it has no effect. **Detect connection settings** works with either value | **Off** | admin with upstream permission |
| **CalDAV URL**, **CardDAV URL** (per domain, simple form) | Stored and checked by **Test saved connections** only. Calendar and contact sync does not use them; see [Nextcloud](./nextcloud.md) | empty | admin with upstream permission |

## Command line

The `mailwoman admin domains` commands change the same records offline. Run them with the deployment's database and server key (`--db-path` or `MW_DB_PATH`, `--server-key` or `MW_SERVER_KEY`), and with `MW_MODE` set to the server's mode when you set an upstream:

```sh
mailwoman admin domains list
mailwoman admin domains show example.org
mailwoman admin domains create example.org
mailwoman admin domains set-upstream example.org '{"imap":{"host":"imap.example.org","port":993,"tls":"implicit"},"smtp":{"host":"smtp.example.org","port":465,"tls":"implicit"}}'
mailwoman admin domains prefer example.org
mailwoman admin domains unprefer
mailwoman admin domains alias example.org corp.example.org [--remove]
mailwoman admin domains host example.org webmail.example.org [--remove]
mailwoman admin domains enable example.org
mailwoman admin domains disable example.org
mailwoman admin domains delete example.org
```

`delete` refuses a domain that is preferred or has signed-in accounts; the command line has no force option.

## Troubleshooting

| Symptom | Cause | Fix |
|---|---|---|
| Sign-in says "This address is not set up on this server" | The domain is disabled, has no enabled mail backend, or is not listed while **Sign-in domains** is `listed` and there is no default | Enable the domain or configure a mail backend |
| A bare username signs in to the wrong domain | `MW_PUBLIC_URL` is set, so only its host name is matched against **Login hostnames**; otherwise the preferred domain is used | Map the `MW_PUBLIC_URL` host, or ask users to type the full address |
| "This domain changed on the server" | Another admin saved the domain after you opened it | **Reload server version**, then redo the change |
| A check answers "Run at most six checks per minute." | The per-session limit of the diagnostics | Wait a minute |
| Saving a backend list with failover is refused | The backends in the chain differ in connection security, authentication or username format | Make them match, or turn failover off |

## Related

- [Guided domain setup](./guided-domain-setup.md)
- [Exchange (EWS) mail backends](./domain-providers.md)
- [First sign-in and pre-added users](./preprovisioned-users.md)
- [SMTP transport policies](./smtp-transport-policy.md)
- [Settings reference](./settings.md)
