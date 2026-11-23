---
screen: true
---
# Guided domain setup

**Guided add** on the **Domains** screen opens **Add a domain with guidance**: a three-step wizard that creates a domain and, if you want, its mail server in one go. Nothing is saved until you select **Create domain** on the last step.

![The Add a domain with guidance wizard on its first step, Domain identity, with corp.example.com entered as the domain name.](/screenshots/admin-access/guided-name.webp){.mw-shot}

## Before you start

- Creating a domain needs the domain write permission. Choosing its mail server needs, in addition, the upstream permission for all domains; without it the wizard only offers **Use server defaults** and says "Your role can create the domain using server defaults. Changing its mail servers requires upstream configuration permission."
- What the wizard offers depends on the server's mode, set for the whole server: in engine mode an IMAP or POP3 server (plus SMTP, ManageSieve or Exchange Web Services in the advanced option); in proxy mode a JMAP session URL.

## Add a domain

1. On **Domains**, select **Guided add**.
2. **Domain identity**: enter the **Domain name**, for example `example.org`, without a scheme, path or port. Select **Next**.
3. **Connection**: choose one of:
   - **Use server defaults**: the domain uses the server's default mail server (see [How a sign-in finds its mail server](./domains.md#how-a-sign-in-finds-its-mail-server)).
   - **Configure a mail server**: one incoming server and, optionally, one outgoing SMTP server (below).
   - **Configure multiple backends and advanced options**: a list of backends with the same fields as the backend editor (below).

   Select **Next**.
4. **Review**: check the summary and select **Create domain**. The domain is created enabled and its **Connections and discovery** page opens.

Select **Back** to change an earlier step. Leaving the wizard with unsaved entries asks whether to discard them.

## Configure a mail server

![The Connection step of the wizard with Configure a mail server chosen: IMAP on imap.example.com port 993 with TLS from connection start, and an outgoing SMTP server smtp.example.com on port 465.](/screenshots/admin-access/guided-connection.webp){.mw-shot}

In engine mode:

1. Choose the **Mail protocol**: IMAP or POP3.
2. Enter the **Mail hostname or IP address**.
3. Choose **Mail encryption**: **TLS from connection start** or **Required STARTTLS**. The **Mail port** changes to the usual port for the choice (993 or 143 for IMAP, 995 or 110 for POP3); change it if your server uses another.
4. To send through a domain-specific server, check **Configure an outgoing SMTP server** and enter **SMTP hostname or IP address**, **SMTP port** and **SMTP encryption** (465 for TLS, 587 for STARTTLS by default). Without it, the review shows "No domain-specific SMTP server".

In proxy mode, enter the **JMAP session URL** instead, for example `https://mail.example.org/.well-known/jmap`. It must be HTTPS, without credentials, query or fragment.

This option always creates servers with verified TLS, password sign-in and no private network access; the review lists this as **Connection policy**. It offers no plaintext connection. For a server on a private network, plaintext, a custom certificate authority or other authentication methods, use the advanced option or edit the backend afterwards.

The wizard never asks for account passwords. Users sign in with their own.

## Configure multiple backends and advanced options

This option builds the domain's backend list in the wizard (up to 32 backends) before the domain exists:

1. Optionally select **Discover configuration**. The server looks up the domain's published mail settings through its outbound connection policy and shows them as a proposal; **Review discovered endpoints** copies them into the list. A provider that reports OAuth shows a warning: the wizard does not set up OAuth, and password sign-in may not work there.
2. Select **Add backend** and fill in the backend: **Backend ID**, **Protocol and service** (IMAP, POP3, SMTP, JMAP, SIEVE or EWS in engine mode; JMAP in proxy mode), **Priority**, **Enabled**, **Preferred for this service**, the connection fields, **Authentication username format** and, for socket protocols, the authentication methods. The fields are those of the backend editor described in [Domains](./domains.md#connections-and-discovery); EWS fields are described in [Exchange (EWS) mail backends](./domain-providers.md).
3. Select **Back to backend list** and repeat for each backend. The row buttons edit a backend or remove it from the draft.
4. Select **Next** and review. Each backend is listed with its protocol, address, priority and options.

The wizard refuses to continue when a backend ID repeats or has characters other than letters, digits, hyphens and underscores, when two enabled backends of one role are preferred, when a protocol does not fit the server mode, or when the security or authentication order lists a method twice.

![The Review step of the wizard listing the domain corp.example.com, the IMAP and SMTP servers and the connection policy, with the Create domain button.](/screenshots/admin-access/guided-review.webp){.mw-shot}

## What the wizard does not set up

- Calendar and contact synchronisation. They use the Nextcloud connection; see [Nextcloud](./nextcloud.md).
- Sign-in allowlists and blocklists, domain aliases, login host names and the preferred domain. Set them on the domain's pages afterwards; see [Domains](./domains.md).
- OAuth sign-in to Microsoft 365 or Google mailboxes. Domain backends sign in with the user's password only.

## Troubleshooting

| Symptom | Cause | Fix |
|---|---|---|
| "Enter a domain name such as example.org, without a scheme, path or port." | The name has `https://`, a path, a port, or no dot | Enter only the domain name |
| "Enter valid server hostnames or IP addresses and ports between 1 and 65535." | A host name with a port in it (`imap.example.org:993`) or an invalid port | Put the port in its own field |
| Only **Use server defaults** can be chosen | Your role lacks the upstream permission for all domains | Ask an admin with that permission |
| "Load the server connection mode before configuring a mail server." | The server's mode could not be loaded | Reload the page |

## Related

- [Domains](./domains.md)
- [Exchange (EWS) mail backends](./domain-providers.md)
- [SMTP transport policies](./smtp-transport-policy.md)
