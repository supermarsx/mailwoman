# Kerberos

Mailwoman does not implement Kerberos. It cannot sign people in with a Kerberos ticket, and it never presents one to another server. This page lists what that means for each place where Kerberos might be expected, and what you can use instead.

## What is not available

| Where | What Mailwoman does instead |
|---|---|
| Signing in to webmail from a domain-joined browser (SPNEGO, "Negotiate") | Not supported. People sign in with a password, through a [single sign-on provider](../admin/sso.md) (OpenID Connect or SAML 2.0), or through [header authentication](./reverse-proxy.md#header-authentication) behind a proxy that has already authenticated them. |
| Connecting to IMAP, POP3 or SMTP servers with GSSAPI | Not supported. Mailwoman signs in to mail servers with the user's password. |
| Binding to an LDAP directory with GSSAPI | Not supported. Directory connections use a simple bind (a DN and password) over LDAPS or StartTLS, or no bind at all. See [LDAP directory](./ldap.md). |
| Connecting to Exchange Web Services (EWS) | Mailwoman uses Basic or NTLMv2 authentication over verified TLS. The console says so beside the EWS settings: "Kerberos requires a separately configured gateway." |

## Exchange servers that accept only Kerberos

If your Exchange server accepts only Kerberos for EWS, put a gateway in front of it that you run yourself:

1. The gateway accepts Basic or NTLMv2 over TLS from Mailwoman.
2. The gateway authenticates to Exchange with Kerberos on the user's behalf.
3. In the domain's EWS settings, enter the gateway's address as the **Exchange Web Services endpoint**, and set **EWS authentication** to the method the gateway accepts. See [Domain-selected EWS accounts](../admin/domain-providers.md).

Mailwoman does not check how the gateway authenticates to Exchange: from Mailwoman's side it is an ordinary EWS server. Configuring the gateway (for example Kerberos constrained delegation) is outside Mailwoman and is not covered by this manual.

::: warning
With Basic authentication, Mailwoman sends the user's password to the gateway. Use TLS between Mailwoman and the gateway, and restrict who can reach the gateway.
:::

## Related

- [Single sign-on](../admin/sso.md)
- [Header authentication](./reverse-proxy.md#header-authentication)
- [LDAP directory](./ldap.md)
- [Domain-selected EWS accounts](../admin/domain-providers.md)
- [EWS bridge](../bridges/ews.md)
