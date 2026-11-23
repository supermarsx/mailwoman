---
title: Exchange (EWS) bridge
---

# Exchange (EWS) bridge

The EWS bridge connects mailboxes on an on-premises Exchange Server to Mailwoman through Exchange Web Services (SOAP over HTTPS). Mail comes from Exchange, and free/busy lookups can use it. For Exchange Online use the [Microsoft Graph bridge](./graph.md).

Unlike the Graph and Gmail bridges, the EWS bridge is configured by an administrator on a domain, not by each user. Users sign in with their Exchange username and password.

The bridge plugin (`bridge-ews`) can reach only the EWS host of the account it is bound to. The server keeps the credential sealed and hands it to the plugin for each call; the plugin does not store it between calls.

## Before you start

- The server runs in engine mode.
- The Exchange server is reachable over HTTPS from the Mailwoman server and its certificate is trusted, or you have the CA certificates to add (see below).
- Exchange accepts HTTPS Basic or NTLMv2 on the EWS endpoint. Kerberos is not spoken by the bridge; see [Kerberos](#kerberos).
- An administrator has approved and enabled `bridge-ews` and granted it `account-backend` and `net`. See [Plugins](../plugins/index.md#approve-and-enable). Private Exchange servers also need an internal-destination entry for the **Approved plugin and provider requests** traffic class in **Egress**.
- **Plugins** and the individual integration permission for `bridge-ews` are on for the users (`cap.plugins`, default on).

## Set up the domain

1. In the admin console open **Domains** and select the domain.
2. Choose **Add backend**.
3. Under **Protocol and service** choose **EWS**. EWS handles both mail and submission, so disable separate SMTP and Sieve backends on this domain, or select another mail backend; the form refuses to save otherwise.
4. Enter the **Exchange Web Services endpoint**, an exact HTTPS URL such as `https://exchange.example.org/EWS/Exchange.asmx`. A URL with credentials, a query string or a fragment is refused.
5. Choose the **EWS authentication**: **HTTPS Basic** or **NTLMv2**.
6. For NTLMv2, enter the **Windows domain (NTLM)** (required, up to 128 letters, digits, `.`, `_` or `-`) and, optionally, the **Workstation (optional)** (up to 64 of the same characters).
7. Optionally paste **Additional origin CA certificates (optional)**: PEM certificates only, one to eight, at most 64 KiB, no private keys. Hostname and certificate chain checks stay on.
8. Save.

<!-- screenshot: admin-domain-ews-backend | Admin > Domains > Backend configuration with protocol EWS, endpoint, NTLMv2 authentication, Windows domain and workstation fields | element: backend editor form | callouts: 1 protocol, 2 endpoint, 3 authentication, 4 Windows domain, 5 CA certificates | stack M | seed admin-basic with a domain in engine mode -->

Saving checks the configuration only, not the credentials or the Exchange server. The credentials are checked when a user signs in. If the first sign-in is rejected, the account is not created.

::: warning
Authentication failures never fall back to another method or another backend.
:::

## Sign-in and usernames

A user signs in to Mailwoman with their address and password. The server sends the username to Exchange as follows:

- **HTTPS Basic:** the username configured for the backend (by default the full email address).
- **NTLMv2:** the user name without a domain. `user@example.org` is sent as `user`. `CORP\user` is accepted when `CORP` matches the configured Windows domain (case-insensitive); another domain is refused.

## What works

| Area | Behaviour |
|---|---|
| Mail | Folder hierarchy, item sync, fetch, flags, move and send. Sending uses `SendAndSaveCopy`. Item identity uses the stable item ID, so a read flag or delete works on the cached message. |
| Free/busy | Looking up another person's free/busy asks Exchange when no shared calendar answers. |
| Calendar and tasks | The plugin can read the primary calendar and the task folder, but the server does not copy them into the calendar and task screens. See [Calendars and tasks through a bridge](./pim.md). |

The plugin also contains code for the global address list, rooms, Out-of-Office, recall and voting. The server and web client do not call it, so none of these features are available.

## Authentication details

- **HTTPS Basic** and **NTLMv2** both require an exact HTTPS endpoint with a verified certificate. Neither follows a redirect.
- NTLMv2 is implemented in the plugin in Rust. It runs the three NTLM messages over one connection that the server owns, including through a configured outbound proxy. The final message carries a message integrity code, the certificate channel binding of the verified TLS connection and an `HTTP/<endpoint host>` service binding. If the TLS binding is missing or a weaker mode is negotiated, the server refuses.

## Kerberos

The bridge does not speak Kerberos or SPNEGO and never holds a Kerberos ticket. The only supported way to use Kerberos-protected Exchange is a Kerberos-capable reverse proxy that you run in front of EWS: the bridge signs in to the proxy with Basic or NTLMv2 and the proxy authenticates to Exchange. See [Kerberos](../deploy/kerberos.md) for proxy configuration. The domain form states the same: Kerberos requires a separately configured gateway.

## Limits

- The server requests EWS schema version Exchange2013, so Exchange 2013 or later is needed.
- Folder and item syncs read every page before they store a new position. A SOAP error cannot be taken for an empty result, and expired sync state makes the server read the mailbox again.
- Changing the EWS endpoint or disabling the domain backend stops requests for its accounts. Removing the generic plugin hostname alone does not revoke the endpoint.
- The guided domain setup does not offer EWS; use **Add backend**.

## Troubleshooting

| Symptom | Cause | Fix |
|---|---|---|
| The form refuses to save with a service conflict. | An SMTP or Sieve backend is enabled on the same domain. | Disable them, or select another mail backend. |
| Sign-in fails for every user. | Wrong NTLM domain, or Exchange does not accept the chosen method. | Check **Windows domain (NTLM)** and the authentication methods on the EWS virtual directory. |
| Sign-in fails on a private network. | The server is not allowed to contact a private address. | Add an internal-destination permission for the **Approved plugin and provider requests** class. |
| Certificate errors. | The Exchange certificate is not trusted by the server. | Add its CA under **Additional origin CA certificates (optional)**. |

## Related

- [Mail provider bridges](./index.md)
- [Plugins](../plugins/index.md)
- [Kerberos](../deploy/kerberos.md)
- [Domains](../admin/domains.md)
