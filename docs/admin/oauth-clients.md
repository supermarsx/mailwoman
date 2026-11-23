---
screen: true
---
# OAuth clients

The **OAuth clients** page decides whether apps may register themselves with Mailwoman's OAuth server (dynamic client registration, RFC 7591), which addresses they may send people back to, and what a self-registered app may do once a person approves it. It does not list or edit clients, and it does not affect clients added by other means.

This OAuth server gives apps access to the mail API. It is separate from the [identity provider](./identity-provider.md), which lets apps sign people in with Mailwoman, and from [single sign-on](./sso.md), which lets people sign in to Mailwoman with another provider.

![The OAuth clients page with self-registration off, no allowed redirect host, and the shared registration token not required.](/screenshots/admin-access/oauth-clients-page.webp){.mw-shot}

1. **Let apps register themselves**
2. **Allowed redirect hosts**
3. **Require the shared registration token**
{.mw-callouts}

## Before you start

- Viewing the page needs a role with `integrations.read`; saving needs `oauth_clients.write`.
- Self-registration is off until you turn it on. While it is off, `POST /oauth/register` is answered `403`.
- The server reads the policy from the database for every registration request, so a saved change applies to the next registration. The page's note "Applies within 30 seconds on every server process" is the console's general wording; this policy takes effect sooner.

## Allow apps to register

1. Add at least one host under **Allowed redirect hosts**: type it and click **Add**. With an empty list every registration is refused, and the page does not save the switch on without a host.
2. Choose what a registered app gets under **What a registered app gets**.
3. If only apps you hand a token to may register, tick **Require the shared registration token** and set `MW_DCR_INITIAL_ACCESS_TOKEN` in the server's environment.
4. Tick **Let apps register themselves**.
5. Click **Save**.

## Options

<a id="enabled"></a>
### Let apps register themselves

Opens `/oauth/register` to apps. Off by default.

A registered app is always a public client that uses the authorization code flow with PKCE: the server refuses any other `token_endpoint_auth_method` than `none`, any grant type other than `authorization_code` and `refresh_token`, and any response type other than `code`. The app receives a registration access token with which it can read, change or delete its own registration (`/oauth/register/<client id>`).

<a id="allowedRedirectHostSuffixes"></a>
### Allowed redirect hosts

Every redirect address an app registers is read as a browser would read it and must pass these checks, or the registration is refused:

- It is a complete address, without a backslash, control characters, spaces, a user name or password (`user@`) or a fragment (`#...`).
- It uses `https`, or `http` to a loopback IP address such as `http://127.0.0.1:8765/callback` or `http://[::1]/cb`. `http` to any other host, including `localhost`, and app-specific schemes such as `com.example.app:/cb` are refused.
- Its host is on this list: an entry `example.org` (or `.example.org` or `*.example.org`) allows `example.org` and every name ending in `.example.org`, such as `app.example.org`, but not `notexample.org`. A loopback address must be on the list too (`127.0.0.1` or `::1`). An empty list refuses every registration.

When an app later asks for sign-in, the redirect address must be exactly one it registered, port included, and it is checked against the same shape and scheme rules again; an app whose stored address no longer passes gets no authorization code. Clients registered by an administrator rather than by themselves keep app-specific schemes, but an address with a user name, a backslash or a fragment gets no code either.

<a id="requireInitialAccessToken"></a>
### Require the shared registration token

Registration then needs the header `Authorization: Bearer <token>` with the value of `MW_DCR_INITIAL_ACCESS_TOKEN`. If that variable is not set where the server starts, every registration is refused. Off by default.

<a id="defaultScope"></a>
### What a registered app gets

![The What a registered app gets group: the Read, Send and Delete permissions, the Mail and Calendar, contacts, tasks and notes parts of the account, and the requests-per-minute limit.](/screenshots/admin-access/oauth-clients-scope.webp){.mw-crop}

**Granted scope** is the scope every self-registered app gets: the permissions **Read**, **Send** and **Delete**, over **Mail** and/or **Calendar, contacts, tasks and notes**, with an optional limit of **Requests per minute** (empty for no limit). A permission without a part of the account grants nothing. Default: nothing.

What an app asks for at registration never widens this. When a person approves the app, the access it receives is narrowed to this scope. Parts of a stored scope that the page has no control for (account and folder selection, an address allowlist, an expiry, MCP tools) are kept as they are when you save.

## What the page does not do

- It does not list self-registered clients, and there is no console screen to remove one. An app removes its own registration with its registration access token.
- Saving does not write an entry to the console's audit log, so **History** finds no entries for this page.

## Related

- [API keys and OAuth](../security/api-keys-oauth.md)
- [Mailwoman identity provider](./identity-provider.md)
- [Single sign-on](./sso.md)
