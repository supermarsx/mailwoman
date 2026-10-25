# Admin console: Single sign-on

The **Single sign-on** page (`#/sso`) lists the OpenID Connect and SAML 2.0 sign-in providers. Each
enabled provider is a "Sign in with …" button on the mail sign-in page. The list shows the protocol,
where the provider applies, whether it is on and, for SAML, whether its signing certificates are valid,
expire within 30 days or have expired. A provider opens at `#/sso/<id>`; **Add a login backend** opens
an empty form at `#/sso/new`. Turning a provider off and deleting it ask first.

The server stores providers per scope (`GET /admin/sso?scope=deployment|domain:<d>`); the page asks for
the deployment and for every domain the console lists.

<a id="id"></a>
## Backend ID

Lower-case letters, digits and hyphens. It is part of this server's addresses for the provider
(`/api/sso/<id>/callback`, `/api/sso/<id>/acs`, `/api/sso/<id>/metadata`), so it cannot change once the
provider exists: delete it and add it again to use another id.

<a id="displayName"></a>
## Display name

The text of the sign-in button.

<a id="kind"></a>
## Protocol

OpenID Connect or SAML 2.0. The form shows the fields of the chosen protocol.

<a id="scope"></a>
## Scope and domain

**Whole deployment** shows the button to everyone; **One domain** only for addresses in that domain.

<a id="firstLoginPolicy"></a>
## First-login policy

**Allowlist** (the default) refuses a person whose account does not exist yet. **Auto-create** makes an
account at the first sign-in.

<a id="enabled"></a>
## Enabled

Off keeps the settings and takes the button off the sign-in page.

<a id="issuerUrl"></a>
## Issuer URL (OIDC)

The provider's issuer; the server reads the discovery document from it.

<a id="clientId"></a>
## Client ID and client secret (OIDC)

The client this server was registered as at the provider. The secret is write-only: it is stored sealed,
never returned, and kept when the field is left empty.

<a id="redirectUrl"></a>
## Redirect URL (OIDC)

Left empty, the form uses this server's `/api/sso/<id>/callback`. Register the same address at the
provider.

<a id="scopes"></a>
## Scopes (OIDC)

The scopes asked for at sign-in, one per entry; `openid` is required. Default `openid email profile`.

<a id="idpMetadata"></a>
## Provider metadata (SAML)

Upload or paste the identity provider's metadata. The page shows the provider's entity id, its sign-in
address for the HTTP-Redirect binding, its sign-out address and each signing certificate with its end
date. **Use these values** fills the sign-in address, sign-out address and certificate list, and keeps the
metadata text with the provider. The metadata address field is kept for reference only: the server does
not fetch metadata from it.

<a id="idpSsoUrl"></a>
## IdP SSO URL (SAML)

Where people are sent to sign in (HTTP-Redirect binding).

<a id="idpSloUrl"></a>
## IdP logout URL (SAML)

Where people are sent to sign out; empty when the provider has none.

<a id="idpCerts"></a>
## IdP signing certificates (SAML)

The certificates the provider signs with. A response signed with any other certificate is refused. Each
certificate shows its subject and end date; add one from a `.pem`/`.crt` file or pasted PEM. At least one
is required.

<a id="spEntityId"></a>
## SP entity ID (SAML)

The name this server uses towards the provider; left empty, `/api/sso/<id>/metadata` on this server.

<a id="acsUrl"></a>
## ACS URL (SAML)

Where the provider posts its answer; left empty, `/api/sso/<id>/acs` on this server.

<a id="nameidFormat"></a>
## NameID format (SAML)

The kind of user identifier asked from the provider. Default
`urn:oasis:names:tc:SAML:2.0:nameid-format:persistent`.

<a id="wantAssertionsSigned"></a>
## Require signed assertions (SAML)

Refuse assertions not signed by one of the certificates. On by default.

<a id="wantEncrypted"></a>
## Require encrypted assertions and the service provider key (SAML)

Refuse assertions that are not encrypted to this server. Needs the service provider private key (PEM),
which is write-only like the OIDC client secret.

<a id="claims"></a>
## Claim mapping

Which claim (OIDC) or attribute (SAML) fills the email address, the account name, the display name and
the groups. Email and account name are required.
