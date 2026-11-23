---
screen: true
---

# The admin console

The admin console is where you configure the server: mail domains and their mail servers, who may sign in and how, the security centre, integrations, and the server's operation. It is part of the web app but has its own address, its own accounts and its own sessions; a mailbox account cannot open it. This page shows how to reach it, sign in, find your way around and what the dashboard tells you.

![The admin console on the Dashboard: the navigation grouped under My account, Overview, People, Security and Configuration on the left, the search button at the top right, and the setup checklist, account counts and request history in the main column.](/screenshots/admin-access/admin-index-console.webp){.mw-shot}

## Before you start

- **The console address.** Each installation has its own console path (for example `https://mail.example.org/k3v9q2/`). The install script prints it, and `mailwoman admin console-path show` prints it on the server. `MW_ADMIN_PATH` pins it. See [Moving the admin console](/deploy/README#moving-the-admin-console).
- **Console screens are addresses after `#`.** The Users screen is `<console path>#/users`, the Security centre's IP ban list is `<console path>#/security/bans`. Bookmark that form. The same words typed as a path without the `#` never open a screen: `<console path>/users` is a console API route that answers with data, and most other paths answer with a "page does not exist" page. Some such paths also match the server's built-in list of paths that scanners try (for example any path ending in `/console`, such as `<console path>/security/console`, or `/admin/users` on a server whose console is elsewhere); with the default settings the first such request blocks the address you came from, for every page of the server, for five minutes. See [Console addresses and scanner paths](/admin/request-protection#console-addresses-and-scanner-paths).
- **An admin account.** The first account comes from the environment credential (`MW_ADMIN_USER` and `MW_ADMIN_PASSWORD`). The first successful sign-in with it on a server that has no admin accounts creates that account with the **Super admin** role and stores a hash of the password; from then on only the stored password signs in, and changing `MW_ADMIN_PASSWORD` does not change it. `MW_ADMIN_ENV_LOGIN=off` stops the environment credential from signing in. Details are in [Admin console security](/security/admin-panel#turning-the-console-on-and-the-environment-credential).

## Sign in

1. Open the console address.
2. Enter the **Admin username** and **Password** and select **Sign in**.

   ![The admin console sign-in form with the Admin username and Password fields and the Sign in button, markers 1 and 2 on the two fields.](/screenshots/admin-access/admin-index-signin.webp){.mw-crop}

   1. **Admin username**: the name of an admin account, not a mailbox address.
   2. **Password**: that account's password.
   {.mw-callouts}

3. If the account has an authenticator app, the next step asks for the **Authentication code** from the app; select **Verify**. If it has a passkey, the browser asks for it. **Sign in with a passkey** signs in with a passkey alone. Setting these up is described in [Administrator authenticator and passkeys](/admin/authenticator).

Too many attempts from one network are refused with "Too many sign-in attempts. Wait a few minutes and try again." Sign-in checks, pacing and lockouts are described in [Admin console security](/security/admin-panel#sign-in).

## Find your way around

The navigation on the left lists the sections in groups. You see only the sections your roles include; an address naming a section you may not open shows "This section does not exist, or your role does not include it." Roles are described in [Roles and capabilities](/security/admin-panel#roles-and-capabilities).

![The console navigation with markers 1 to 6 on its groups: My account, Overview, People, Security, Configuration and Operations, each listing its sections.](/screenshots/admin-access/admin-index-nav.webp){.mw-crop}

1. **My account**: **My preferences**, your own appearance, reading and dashboard layout, and **Personal security** (your authenticator, passkeys and password).
2. **Overview**: the **Dashboard**.
3. **People**: **Users** ([user access](/admin/user-access-lifecycle), [first sign-in and pre-added users](/admin/preprovisioned-users)) and **Domains** ([domains and their mail servers](/admin/domains), [guided setup](/admin/guided-domain-setup)).
4. **Security**: **Identity provider** ([page](/admin/identity-provider)), **Attachment policy** ([privacy](/admin/privacy)), **Security centre** ([page](/admin/security-center)), **Single sign-on** ([page](/admin/sso)), **Admin accounts** and **Roles** ([admin console security](/security/admin-panel)), **OAuth clients** ([page](/admin/oauth-clients)), **Compliance** ([page](/admin/compliance)) and **Privacy** ([page](/admin/privacy)).
5. **Configuration**: **Configuration & backups**, **Package tracking**, **Nextcloud**, **Usage analytics**, **Branding**, **Settings** ([settings reference](/admin/settings)), **Effective configuration**, **AI assistant**, **Integrations** (tabs **LDAP / GAL directory**, **Webhooks**, **API & MCP keys**, **Hosting panels** and **SSH tunnels**: [directory](/admin/directory), [webhooks](/admin/webhooks), [hosting panels](/admin/hosting), [SSH tunnels](/admin/ssh-tunnels)), **Feature profiles**, **Plugins**, **UI plugins** and **Outbound connections**.
6. **Operations**: **Logs**, **Diagnostics**, **Observability**, **Maintenance** and **Server metadata**.
{.mw-callouts}

The bar above each screen shows where you are (for example **Dashboard** > **My preferences**). The navigation can be resized by dragging its edge, or hidden with **Hide navigation sidebar**. Below 761 pixels of width it becomes a **Menu** button that opens the same list.

### Search

Press **Ctrl+K** (**Cmd+K** on macOS), or select **Search** at the top right, to open **Search the admin console**. Type part of a name and press Enter, or select a result.

![The console search window with domain typed, listing matching sections, domains and settings.](/screenshots/admin-access/admin-index-search.webp){.mw-crop}

The search finds sections (**Section**) and the individual fields of the screens you may open (**Field**); opening a field result goes to its screen and highlights the field. It does not find users or domains by name: the server has no search for them, and the window says "This server cannot search users, domains and settings yet, so only sections are listed." Use the filter on **Users** or **Domains** instead.

## The dashboard

The **Dashboard** is the first screen after sign-in. Its sections can be reordered and hidden with **Customize dashboard**, the button beside the title, and reloaded with the refresh button next to it.

- **Setup checklist** lists three items, each marked **Done** or **Open**: a mail domain exists, a domain has a mail server (or a default upstream is set), and, optionally, a preferred domain is chosen so users can sign in without typing it. The count in the heading leaves out the optional item. The checklist is not shown to an admin whose roles do not cover domains.

  ![The dashboard setup checklist with one item done and two open, each with an Open link and a Hide button; markers on a Done status, an Open link and a Hide button.](/screenshots/admin-access/admin-index-checklist.webp){.mw-crop}

  1. The item's state, **Done** or **Open**.
  2. **Open** goes to the screen where you complete the item.
  3. **Hide** removes the item from your checklist in this browser; **Show hidden items** brings it back.
  {.mw-callouts}

- **Accounts and sessions** shows **Provisioned users**, **Accounts signed in, last 24 hours**, **Accounts signed in, last 7 days**, **Domains** (with how many have a mail server and how many are disabled), **Active webmail sessions** and **Active admin sessions**.
- **Request history** shows the requests this server handled over the chosen **Period**, with errors and response times; a period with no recorded history shows **Not available**. See [Observability](/admin/observability).

## Your session

The bar along the bottom of the console shows who is signed in and how long the session has left.

![The session bar at the bottom of the console: Signed in as admin and the idle timeout countdown.](/screenshots/admin-access/admin-index-session-bar.webp){.mw-crop}

A console session ends after 30 minutes without a console request, or 12 hours after sign-in, whichever comes first. Each console request moves the idle deadline forward but never past the 12-hour limit. When less than five minutes remain, the bar changes colour and offers **Extend**; after the session has ended it says "Session expired — sign in again". Select the countdown for **Session details**.

| Setting | Key | Default | Range the server applies |
|---|---|---|---|
| Idle timeout | [`admin.session.idleSecs`](/admin/settings#admin.session.idlesecs) | 1800 s | 300 to 7200 s |
| Absolute limit | [`admin.session.absoluteSecs`](/admin/settings#admin.session.absolutesecs) | 43200 s | 3600 to 86400 s |
| Session id rotation | [`admin.session.rotateSecs`](/admin/settings#admin.session.rotatesecs) | 900 s (0 = only on step-up) | 0 to 86400 s |

::: warning
The settings accept larger values (idle up to 86400 s, absolute up to 604800 s) than the server uses: a value above the right-hand column is applied as that maximum.
:::

Some changes (for example outbound proxies, the identity provider's clients, client access, IP bans, or setting up an authenticator) ask you to confirm it is you in a **Confirm it is you** step before they are saved: your password, plus your second factor if you have one, or a passkey. A confirmation lasts five minutes for that session. Single sign-on changes do not ask for it.

## Troubleshooting

| Symptom | Cause | Fix |
|---|---|---|
| Every page of the server answers `429` for about five minutes | A path you opened matched the scanner-path list, for example a console screen typed without `#` that ends in `/console` | Wait until the block ends, then use the `#` form. See [Request protection](/admin/request-protection#console-addresses-and-scanner-paths). |
| The console address shows "page does not exist" | The path is wrong, or it was changed | Run `mailwoman admin console-path show` on the server. |
| A section you expect is missing from the navigation | Your roles do not include it, or your role is limited to some domains | Ask a Super admin to change your roles. |
| A changed `MW_ADMIN_PASSWORD` is refused at sign-in | The account stored its own password hash at the first sign-in | Sign in with the old password and change it in **My preferences** > **Personal security**, or reset it from another admin account. |

## Related

- [Admin console security](/security/admin-panel)
- [Security centre](/admin/security-center)
- [Domains](/admin/domains)
- [Settings reference](/admin/settings)
- [Quick install](/getting-started/install-quick)
