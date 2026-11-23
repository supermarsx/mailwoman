---
title: Hosting panels
screen: true
---
# Hosting panels

**Hosting panels** (in **Integrations**, under **Configuration** in the console navigation) lets you manage mail domains and mailboxes of a hosting control panel from the console. A connection to a Virtualmin server (Virtualmin is a Webmin module; a Webmin server without Virtualmin has no mail domain model and is not supported) or to a HestiaCP server is saved here, and you can then list its domains and mailboxes and change them. This is not a sign-in for panel users: it provides no panel login and no single sign-on.

![The Hosting panels page under Integrations: the description, the New hosting connection button, and the empty list.](/screenshots/admin-ops/opsa-hosting-list.webp){.mw-shot}

## Before you start

- Opening the page needs `integrations.read`. Creating, changing and removing a connection, and every change to the panel, needs `integrations.write` without a domain restriction and a recent confirmation of your password. Choosing an SSH tunnel as the route also needs `tunnels.bind`.
- In the panel, create a credential that is allowed to run only the commands you need. Virtualmin uses an administrator name and password, for the remote API at `/virtual-server/remote.cgi`. HestiaCP uses an account owner and an access key with its secret key, for the API at `/api/`.
- The panel must be reachable over HTTPS from the server. The origin has no path, query, user name, password or fragment.
- Panel versions differ in the commands they allow. Try the connection on test data and check the permissions of your own panel version before you use it on production data.

## Add a connection

1. Open **Integrations** and the **Hosting panels** tab, then choose **New hosting connection**.
2. Under **Panel identity**, enter a **Connection name** (up to 120 characters), choose the **Provider** and enter the **HTTPS panel origin**, for example `https://panel.example.org:10000`. For Virtualmin, enter the **Virtualmin administrator username**. For HestiaCP, enter the **Hestia account owner** and the **Hestia access key**.
3. Tick **Enable this connection** if the connection should be usable at once. A connection that is not enabled cannot query the panel.
4. Under **Panel credential**, choose **Set or replace credential** and enter the **Virtualmin password** or the **Hestia secret key**. The credential is sealed with the server key and never returned to the browser: the page says **A credential is saved and sealed. Its value is never returned to the browser.** When you edit a connection you choose **Keep current credential**, **Set or replace credential** or **Clear credential**. Replace or clear it when you change the destination, the panel identity, the TLS authority or the tunnel. A connection cannot be enabled without a credential.
5. Under **Connection and TLS**:
   - **Connection route**: **Explicit direct connection**, or **Saved SSH tunnel** with one of your enabled tunnels (**Choose an enabled saved tunnel**; **Manage SSH tunnels** opens them). If the tunnel is not available, the connection fails; it never falls back to a direct connection.
   - **Allow this configured destination on a private or loopback address** is needed when the panel is on a private network. Metadata, link-local and reserved addresses stay blocked.
   - **TLS server name override** and **Additional trusted CA certificates (PEM)** for a panel that uses a private CA or another certificate name.
   - **Minimum TLS version**: 1.2 (the default) or 1.3.
6. For Virtualmin, **Virtualmin domain lifecycle** has **Explicitly allow whole virtual-server lifecycle actions**. It is off by default. Creating, disabling and deleting a Virtualmin domain affects the whole virtual server, including its websites, data and subservers. Mailbox actions are separate.
7. Choose **Save connection**.

![The New hosting connection form: connection name, provider, HTTPS panel origin, credential choice, connection route and TLS options.](/screenshots/admin-ops/opsa-hosting-form.webp){.mw-shot}

Certificate and hostname verification always stay on, and an unsafe destination or an unsupported proxy route is refused. A change made by someone else after you opened the form must be reloaded before you save.

## Manage domains and mailboxes

1. In the list, choose **Manage domains and mailboxes** for an enabled connection. The inventory lists the domains the panel returns.
2. Open a domain to see its mailboxes. You can **Create domain** (see below), **Create mailbox**, and for a mailbox **Change mailbox quota** (a whole number of MiB from 0 to 4294967295), **Reset mailbox password**, **Add mailbox alias** (the part before the `@`; the alias stays in the same domain), **Remove alias**, **Enable**, **Disable** and **Delete mailbox**.
3. **Create domain**, **Enable**, **Disable** and **Delete domain** for a Virtualmin connection need the lifecycle option above. With HestiaCP they are always offered.
4. Deleting asks you to type the exact name of the remote target.

Every change is made at once on the remote panel. The inventory and the actions use a fixed set of panel operations; no remote command, path or free-form panel output is accepted or shown. If the connection fails after the request was sent, the result is **The result is uncertain. Inspect the remote state before retrying; this request was not automatically repeated.** Look at the panel before you try again.

Virtualmin mailboxes are Virtualmin user accounts. Changing the password or disabling the account also affects other logins that are enabled for that user, including FTP, and deleting the account removes its home directory and files (the page says so on the domain page). See the Virtualmin documentation of [modify-user](https://www.virtualmin.com/docs/development/api-programs/modify-user/), [delete-user](https://www.virtualmin.com/docs/development/api-programs/delete-user/) and [create-user](https://www.virtualmin.com/docs/development/api-programs/create-user/); users created from here receive the default email-only access described there.

Removing a saved connection (**Remove connection**, after typing its name) deletes the stored credential here. It does not remove any remote mailbox or domain.

Saved hosting connections belong to one deployment. They are not copied when the database is moved to another backend with `migrate-store`, and their sealed credentials are not either; set them up again on the new deployment.

## Options

| Label | What it does | Default | Who can change it |
|---|---|---|---|
| **Connection name** | Name in the list. | Empty | Administrators with `integrations.write` |
| **Provider** | **Virtualmin (Webmin module)** or **HestiaCP**. | Virtualmin | Same |
| **Enable this connection** | Allows queries and changes. | Off | Same |
| **Connection route** | **Explicit direct connection** or **Saved SSH tunnel**. | Direct | Same, with `tunnels.bind` for a tunnel |
| **Allow this configured destination on a private or loopback address** | Permits a private panel address. | Off | Same |
| **Minimum TLS version** | 1.2 or 1.3. | 1.2 | Same |
| **Explicitly allow whole virtual-server lifecycle actions** | Virtualmin only: domain create, enable, disable, delete. | Off | Same |

## Troubleshooting

- **The list says "No hosting connections configured."** Nothing is saved yet.
- **"This connection is disabled. Enable it in its dedicated settings page before querying the panel."** Open **Edit hosting connection** and tick **Enable this connection**.
- **"The configured route is unavailable or unsupported. Check the active proxy and selected SSH tunnel; direct fallback is refused."** The tunnel is not enabled, or the active outbound route is one the connection cannot use.
- **"Whole virtual-server lifecycle actions are disabled for this connection. Mailbox management remains available."** Tick the lifecycle option on the connection.
- **"Enter a whole quota from 0 to 4294967295 MiB."** The quota field takes whole numbers only.
- **"You can view this connection, but cannot change it."** Your role has `integrations.read` only.

## Related

- [SSH tunnels](/admin/ssh-tunnels)
- [Outbound connections](/deploy/egress-proxy)
- [Domains](/admin/domains)
