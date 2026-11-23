---
screen: true
---
# Identity provider

The **Identity provider** page lets other applications (a wiki, a booking tool, a desktop app) sign people in with their Mailwoman account, using OpenID Connect. Mailwoman is then the identity provider and the application is the client. It is off until you configure it.

This is separate from [single sign-on](./sso.md), where another provider signs people in to Mailwoman, and from the mail API's [OAuth clients](./oauth-clients.md): a client registered here cannot get access to mail, calendars or contacts.

![The Identity provider page with the issuer enabled at https://mail.example.com/oidc and two identity clients, Team wiki and Room booking desktop app.](/screenshots/admin-access/identity-provider-page.webp){.mw-shot}

1. **Configure issuer** sets the public address of the provider, turns it on or off, and rotates its signing key.
2. **Add identity client** registers an application.
{.mw-callouts}

The box under the description shows whether the provider is **Enabled** or **Disabled**, its issuer address and **Open discovery document**. The client list can be searched and filtered by **Status**; each client has **Edit client** and **Delete client**.

## Before you start

- Viewing the page needs a role with `integrations.read`. Changing anything needs `oauth_clients.write` for the whole deployment, and the console asks you to confirm it is you if you have not done so recently.
- Mailwoman must be reachable over HTTPS: the issuer must be an `https://` address.

## Turn the provider on

1. Click **Configure issuer**.
2. Enter the **Public HTTPS issuer URL**: your server's public HTTPS address followed by `/oidc`, including the path prefix if Mailwoman runs under one (for example `https://mail.example.com/oidc`). The server refuses any other path, and addresses with a user name, password, query or fragment.
3. Tick **Enabled** and click **Save**. The first time, Mailwoman creates a signing key.

![The issuer form of the identity provider with the public HTTPS issuer URL and the signing key rotation.](/screenshots/admin-access/identity-provider-issuer.webp){.mw-crop}

Applications find the endpoints in the discovery document at `<issuer>/.well-known/openid-configuration`: authorization, token, user information, signing keys (JWKS), revocation and sign-out, all under `/oidc`.

::: warning
Saving the issuer settings (changing the issuer, or turning the provider off and on again) ends every grant people have given to applications: they must approve each application again. Saving a client likewise ends the grants given to that client.
:::

**Rotate signing key** creates a new key. The previous public key stays published for 15 minutes so tokens issued just before the rotation can still be checked.

## Add an application

1. Click **Add identity client**.
2. Enter a **Name** (shown to people when they approve the application).
3. Choose **Client authentication**:
   - **Public client (PKCE, no secret)** for applications that cannot keep a secret, such as desktop, mobile and browser apps;
   - client secret sent with HTTP Basic (`client_secret_basic`) or in the request body (`client_secret_post`) for applications that run on a server.
4. Under **Registered destinations**, enter the **Authorization callback URIs** and, if the application signs people out through Mailwoman, the **Post-logout redirect URIs**: one exact address per line, 1 to 20 of each. Each must be `https://`, or `http://` on the numeric loopback address `127.0.0.1` or `[::1]` for applications on the person's own computer. Addresses with a fragment, a user name or password, or a `code`, `state` or `error` query parameter are refused; wildcards are not possible.
5. Choose the **Allowed identity scopes**: `openid` (required), `profile`, `email` and `offline_access`.
6. Save. For a client with a secret, the page shows the **Generated client secret** once. Copy it into the application at once; it cannot be shown again. If it is lost, edit the client and choose **Replace the existing client secret**.

![The editor of the identity client Room booking desktop app: public client authentication, a loopback callback URI and the allowed identity scopes.](/screenshots/admin-access/identity-provider-client.webp){.mw-crop}

**Delete client** ends the grants people gave the application and removes its registration.

## What applications receive

- The authorization code flow only, with PKCE (`S256`) required for every client. There is no implicit flow, no password grant, no client credentials grant and no device flow.
- People are always asked to approve an application, also when they approved it before. A request with `prompt=none` is answered `consent_required` (or `login_required` without a session).
- ID tokens are signed with RS256. Access tokens and ID tokens are valid for at most 5 minutes.
- The `email` claim is the person's sign-in address when it is an email address and the `email` scope was granted. Mailwoman does not invent a name or a verified-email flag it does not have.
- A refresh token is issued only with `offline_access`. Each use returns a new refresh token; using an old one again ends the whole grant. A grant also ends when the Mailwoman session it was given in ends, and at the latest 15 days after the person signed in to Mailwoman.
- People see the applications they approved in their settings, under **Applications using your identity**, and can end each with **Revoke identity access**.
- Signing out through `<issuer>/logout` asks the person to confirm, then ends their Mailwoman session. It does not sign them out of other applications.

## Not carried over

Moving a deployment with `migrate-store` does not copy the issuer configuration, signing keys, clients, approvals or tokens. Configure the provider and the clients again on the new server; people approve the applications again.

## Related

- [Single sign-on](./sso.md)
- [OAuth clients](./oauth-clients.md)
- [API keys and OAuth](../security/api-keys-oauth.md)
