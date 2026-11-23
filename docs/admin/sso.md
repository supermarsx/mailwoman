---
screen: true
---
# Single sign-on

The **Single sign-on** page lists the OpenID Connect (OIDC) and SAML 2.0 providers people can use to sign in to their mailbox. Each enabled provider appears as a button on the mailbox sign-in page, either for everyone or only for addresses in one domain.

![The Single sign-on page with two OpenID Connect providers: Staff sign-in for the whole deployment, enabled, and Partner sign-in for example.org, disabled.](/screenshots/admin-access/sso-list.webp){.mw-shot}

1. **Add a login backend** opens an empty editor.
2. **Filter by scope** shows the providers of the whole deployment or of one domain. The search box and the **OIDC**, **SAML 2.0**, **Enabled** and **Disabled** buttons narrow the list further; they change only what is shown.
{.mw-callouts}

Each row has buttons to edit the provider, turn it off or on, and delete it. Turning a provider off and deleting it ask first; accounts made through a provider are kept in both cases.

When you save a provider, the server checks the protocol settings and refuses the save with an error that names the field when one is missing or not a valid address: for OpenID Connect the issuer URL (an `http` or `https` address without a query or fragment), client ID, redirect URL and the `openid` scope; for SAML the IdP SSO URL, at least one signing certificate, the entity ID, the ACS URL and the NameID format. Field names it does not know are refused too.

::: warning Providers saved with an older version of the editor
An earlier console sent the protocol settings under names the server did not read, so providers created or saved from the console were stored with an empty issuer URL, client ID, redirect URL and first-login policy (OIDC), or empty SAML fields, and cannot sign anyone in. Open each provider: if those fields are empty, enter them again and save. For a SAML provider, also check that **Require signed assertions** is on: those saves stored it off. The server's provider list (`GET <console path>/sso`) marks such a provider with a non-empty `problems` list naming the missing fields; the console page does not show that list.
:::

## Before you start

- Register Mailwoman at your provider first. For OIDC you need the issuer address, a client ID and a client secret; for SAML, the provider's metadata or its sign-in address and signing certificate.
- Mailwoman's addresses for a provider contain its **Backend ID**: `/api/sso/<id>/callback` (OIDC), `/api/sso/<id>/acs` and `/api/sso/<id>/metadata` (SAML), under your server's public address.
- Viewing the page needs a role with `sso.read`, changing it `sso.write`; both must cover the whole deployment.

## Add a provider

1. Click **Add a login backend**.
2. Fill in the **Provider** group: backend ID, display name, protocol, scope, first-login policy and **Enabled**.
3. Fill in the group for the protocol you chose: **OpenID Connect**, or **Identity provider (SAML)** and **This server (SAML)**.
4. Check **Claim mapping**.
5. Click **Add backend** (or **Save changes** when editing). The page returns to the list.

![The editor of the Staff sign-in provider: backend ID, display name, protocol OIDC, scope, first-login policy and the OpenID Connect fields.](/screenshots/admin-access/sso-editor-oidc.webp){.mw-shot}


**Back to sign-in providers** and **Cancel** ask before they discard unsaved changes.

## Provider

<a id="id"></a>
### Backend ID

Lower-case letters, digits and hyphens, starting with a letter or digit. It is part of Mailwoman's addresses for the provider, so it cannot change once the provider exists: delete the provider and add it again to use another ID.

<a id="displayName"></a>
### Display name

The text of the button on the sign-in page.

<a id="kind"></a>
### Protocol

**OIDC** or **SAML 2.0**. The form shows the fields of the chosen protocol.

<a id="scope"></a>
### Scope

**Whole deployment** shows the button to everyone; **One domain** shows it only for addresses in the chosen domain.

<a id="firstLoginPolicy"></a>
### First-login policy

What happens when the provider confirms someone who has no account on this server yet:

- **Allowlist (deny unknown users)**, the default: the sign-in is refused.
- **Auto-create accounts on first login**: an account is made with the address from the provider. The server's rule for new users of the type "single sign-on" still applies; see [Pre-provisioned users and the first-sign-in rule](./preprovisioned-users.md).

<a id="enabled"></a>
### Enabled

Off keeps the settings but takes the button off the sign-in page, and nobody can sign in with the provider until it is turned on again.

## OpenID Connect

<a id="issuerUrl"></a>
### Issuer URL

The provider's issuer address. Mailwoman reads the provider's discovery document (`.well-known/openid-configuration`) and signing keys from it at each sign-in.

<a id="clientId"></a>
### Client ID and client secret

The client this server was registered as at the provider. The secret is stored encrypted and never shown again; leaving the field empty when editing keeps the stored secret.

<a id="redirectUrl"></a>
### Redirect URL

Leave empty to use this server's `/api/sso/<id>/callback`. Register the same address at the provider.

<a id="scopes"></a>
### Scopes

The scopes asked for at sign-in. `openid` is required. Default: `openid`, `email`, `profile`.

Sign-in uses the authorization code flow with PKCE (S256) and a nonce. Mailwoman checks the ID token's signature against the provider's published keys, its issuer, audience, expiry and nonce, then also reads the provider's user information endpoint if it has one.

## SAML 2.0

![The SAML part of a new provider: provider metadata upload, IdP SSO and logout URLs, signing certificates, and this server's entity ID, ACS URL, NameID format and the signed and encrypted assertion switches.](/screenshots/admin-access/sso-saml.webp){.mw-crop}

<a id="idpMetadata"></a>
### Provider metadata

Upload or paste the provider's metadata. The page shows what it contains (entity ID, sign-in address for the HTTP-Redirect binding, sign-out address, signing certificates with their dates). **Use these values** copies the sign-in address, sign-out address and certificates into the form and keeps the metadata with the provider. The **IdP metadata URL** field is kept for reference only: the server does not fetch metadata from it.

<a id="idpSsoUrl"></a>
### IdP SSO URL

Where people are sent to sign in. Mailwoman sends its request with the HTTP-Redirect binding.

<a id="idpSloUrl"></a>
### IdP logout (SLO) URL

Where people are sent to sign out; leave empty when the provider has none.

<a id="idpCerts"></a>
### IdP signing certificates (PEM)

The certificates the provider signs with. A signature by any other certificate is refused. Add one from a `.pem` or `.crt` file or paste it. At least one is required. The list on the provider page shows whether each certificate is valid, expires within 30 days, or has expired.

<a id="spEntityId"></a>
### SP entity ID

The name this server uses towards the provider. Leave empty to use `/api/sso/<id>/metadata` on this server.

<a id="acsUrl"></a>
### ACS URL

Where the provider posts its answer. Leave empty to use `/api/sso/<id>/acs` on this server.

<a id="nameidFormat"></a>
### NameID format

The kind of user identifier asked from the provider. Default: `urn:oasis:names:tc:SAML:2.0:nameid-format:persistent`.

<a id="wantAssertionsSigned"></a>
### Require signed assertions

On by default. Mailwoman accepts an answer when a signature by one of the certificates covers the assertion, either on the assertion itself or on the whole response.

::: danger
Do not turn this off. Without it, Mailwoman accepts an unsigned answer, and anyone who can send a request to the ACS address can sign in as any user of this provider. The editor shows the switch on for a new provider, and a save through the console API that leaves the switch out also stores it on. A provider row whose stored settings lack the switch altogether (for example one written to the database by other means) is read as off.
:::

<a id="wantEncrypted"></a>
### Require encrypted assertions and the service provider private key

Mailwoman cannot decrypt SAML assertions: an answer that contains an encrypted assertion is always refused, whatever this switch says. The switch and the **Service provider private key** are stored but have no effect. Configure the provider to send unencrypted, signed assertions.

## Claim mapping

![The Claim mapping group: email, username, display-name and groups claim fields.](/screenshots/admin-access/sso-claims.webp){.mw-crop}

<a id="claims"></a>
Which claim (OIDC) or attribute (SAML) of the provider's answer fills each part of the account.

| Field | Used for |
|---|---|
| **Email claim** | The address the person signs in as. Default: `email`. |
| **Username claim** | Used as the address only when the answer has no email. |
| **Display-name claim** | Read, but not used by the server. |
| **Groups claim** | Read, but not used by the server: group membership does not give roles or access. |

## How a sign-in through a provider works

1. The person clicks the provider's button on the sign-in page and signs in at the provider.
2. Mailwoman checks the answer as described above. Each sign-in attempt can be completed only once and must finish within 10 minutes.
3. Mailwoman takes the address from the mapped email claim, then the email the provider asserted, then the username claim, then the provider's subject identifier, and looks for an account with that address (case does not matter). Without one, the first-login policy decides.
4. The sign-in is refused if the address's domain may not sign in, if an administrator disabled the account, or if the sign-in source is locked ([temporary sign-in locks](./signin-locks.md)).
5. Mailwoman opens a mailbox session.

Apart from the first-sign-in rule for new users, every refusal shows the same message, so the person cannot tell which check failed. The reason is recorded in the server's sign-in records.

Points to plan for:

- **No second factor from Mailwoman.** A sign-in through a provider does not ask for the authenticator code or passkey the person may have set up in Mailwoman. Require a second factor at the provider.
- **Matching by address.** Mailwoman matches accounts by the address in the answer and does not look at an `email_verified` claim. Use a provider where people cannot choose or change their own email address, or map **Email claim** to an attribute only your administrators can set.
- **No mail password.** A session opened through a provider stores no mail password and no mail server address. Actions that ask the person to confirm their password cannot use one for these sessions.
- **Signing out.** Signing out of Mailwoman ends the Mailwoman session only. The web app does not send people to the provider's sign-out address, so they stay signed in at the provider, and the next click on the provider's button may sign them in again without a password.

## Header authentication

A reverse proxy that authenticates people itself can pass the user name to Mailwoman in a header instead (`MW_HEADER_AUTH`). It is configured with environment variables, not on this page. Before you turn it on, read [Header authentication](../deploy/reverse-proxy.md#header-authentication): the proxy must remove any copy of the header that the browser sends, and none of the example proxy configurations does this.

## Troubleshooting

| Symptom | Cause | Fix |
|---|---|---|
| The button does not appear on the sign-in page | The provider is off, or scoped to another domain. | Turn it on, or check **Scope**. |
| Everyone gets a refusal after the provider's sign-in | **Allowlist (deny unknown users)** and no account exists for the address, or the claim mapping yields another address. | Create the accounts, choose **Auto-create accounts on first login**, or correct **Email claim**. |
| SAML sign-ins are always refused | The provider sends encrypted assertions, or signs with a certificate that is not in the list. | Turn off assertion encryption at the provider; add its current signing certificate. |

## Related

- [Header authentication](../deploy/reverse-proxy.md#header-authentication)
- [Pre-provisioned users and the first-sign-in rule](./preprovisioned-users.md)
- [Mailwoman identity provider](./identity-provider.md)
- [Signing in](../getting-started/first-sign-in.md)
