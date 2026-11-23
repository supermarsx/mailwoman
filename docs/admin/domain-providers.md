---
screen: true
---
# Exchange (EWS) mail backends

A domain can use a Microsoft Exchange server as its mail backend through Exchange Web Services (EWS). Mailwoman then reads and sends the domain's mail through EWS with each user's own password, and the domain needs no IMAP, POP3 or SMTP server. This page describes the EWS backend of a domain; for the rest of the domain configuration see [Domains](./domains.md).

![The Edit backend screen for the backend exchange of example.net, with EWS chosen as the protocol, priority 100, and Enabled and Preferred for this service checked.](/screenshots/admin-access/domain-providers-editor.webp){.mw-shot}

## Before you start

- The server must run in engine mode. A proxy-mode server refuses an EWS backend.
- EWS runs in the bundled `bridge-ews` plugin. On **Plugins**, the plugin must be present and allowed to load, with its `net` and `account-backend` permissions granted. See [Plugins](./plugins.md).
- The **Plugins** feature ([`cap.plugins`](/admin/settings#cap.plugins), on by default) and the plugin's own integration permission must be on for the user's domain and account. Turned off, EWS sign-in is refused for them.
- EWS requests leave the server through the **Plugins** route of **Outbound connections**. An Exchange server on a private network needs an explicit Plugins destination permission there; the backend editor grants no network exception. Loopback, link-local and cloud metadata addresses are always refused. See [Outbound connections](../security/egress.md).
- Changing backends needs the upstream permission for all domains.

## Add an EWS backend

1. Open the domain on **Domains**, then **Connections and discovery**, and select **Add backend**. (In [Guided domain setup](./guided-domain-setup.md), choose **Configure multiple backends and advanced options** and add the backend there.)
2. Enter a **Backend ID** and choose **EWS** under **Protocol and service**. Check **Enabled** and **Preferred for this service**.
3. Enter the **Exchange Web Services endpoint**: the exact HTTPS address, normally ending in `/EWS/Exchange.asmx`. It must not contain a user name, password, query string or fragment.
4. Choose **EWS authentication**:
   - **HTTPS Basic**: the user's address (formatted by **Authentication username format**) and password, over verified TLS.
   - **NTLMv2**: enter the **Windows domain (NTLM)** (up to 128 letters, digits, periods, underscores or hyphens) and optionally a **Workstation (optional)** name.

   A failed authentication never falls back to the other method.

   ![The backend editor for the EWS backend exchange of example.net: the Exchange Web Services endpoint, EWS authentication set to NTLMv2 with the Windows domain EXAMPLE, and the optional CA certificates field.](/screenshots/admin-access/domain-providers-ews.webp){.mw-crop}
5. Optionally paste **Additional origin CA certificates (optional)**: 1 to 8 PEM certificates, at most 64 KiB, no private keys. They are trusted for this Exchange server only, not for an outbound proxy. Certificate chain and host name are always verified.
6. Disable any active SMTP and ManageSieve backends of the domain: EWS sends mail itself, and saving is refused while they are active ("The selected EWS backend supplies submission. …").
7. Select **Save backend**.

Saving checks the configuration only. It does not sign in to Exchange, and **Diagnostics** on the domain cannot test EWS either. The first real test is a user's sign-in.

## What happens at sign-in

1. The user signs in with their address and password as usual.
2. The server loads the `bridge-ews` plugin, signs in to the configured endpoint and lists the mailbox's folders. This has to finish within 20 seconds, and a mailbox with more than 4096 folders is refused.
3. Only then is the account created and the password stored. A failure at any point leaves no account behind.

The usual sign-in rules apply as for any other backend: the domain's sign-in lists, the first sign-in rule, two-factor requirements and session policy.

## Kerberos

Mailwoman does not speak Kerberos to Exchange. An Exchange server that accepts only Kerberos needs a gateway in front of it that accepts Basic or NTLMv2 from Mailwoman; see [Kerberos](../deploy/kerberos.md).

## Troubleshooting

| Symptom | Cause | Fix |
|---|---|---|
| "Enter a valid HTTPS EWS endpoint and supported authentication. NTLMv2 requires its domain. …" | The address is not HTTPS, contains credentials, a query or a fragment, or NTLMv2 has no Windows domain | Correct the endpoint or fill in **Windows domain (NTLM)** |
| "Enter 1–8 PEM CA certificates, no private keys, within 64 KiB. …" | The certificate field is not plain PEM certificates | Paste only the CA certificates |
| Saving says the EWS backend supplies submission | An SMTP or ManageSieve backend of the domain is still enabled | Disable it, then save |
| Every EWS sign-in is refused although the password is right | The `bridge-ews` plugin is missing, not allowed to load or lacks a permission, the Plugins feature is off for the user, or the outbound route refuses the Exchange host | Check **Plugins**, the user's features and **Outbound connections** |

## Related

- [Domains](./domains.md)
- [Guided domain setup](./guided-domain-setup.md)
- [Plugins](./plugins.md)
- [Kerberos](../deploy/kerberos.md)
