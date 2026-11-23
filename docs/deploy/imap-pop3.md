---
title: IMAP and POP3 mail servers
---

# IMAP and POP3 mail servers

In **engine mode** (`MW_MODE=engine`) Mailwoman does not relay to a JMAP server. It connects to the user's IMAP or POP3 mail server itself, keeps a synchronised copy in its own database, reads the messages locally, and sends mail through SMTP. The web app speaks the same JMAP interface in both modes, and the sign-in form is the same: an address and a password. This page covers how to point the server at an IMAP or POP3 mail server and its SMTP submission service. Mailwoman does not receive or deliver mail; you need a mail server that does.

## Before you start

- `MW_MODE=engine` is set. The default is `proxy`, which expects a JMAP server instead.
- You know the mail server's host name, and whether it offers IMAP or POP3 and which transport security it uses.

## Which server an address signs in to

The sign-in form never asks for a server. The server decides, in this order: the connection settings of the address's domain if the domain is configured and enabled under **Domains** in the admin console; then the `upstream.default` setting; then `MW_UPSTREAM_DEFAULT`. A sign-in body that names a server is refused with `400`. With none of these configured, sign-in answers "This address is not set up on this server". The order is described in [Install and configure](README#which-mail-server-users-sign-in-to); per-domain settings are in [Domains](/admin/domains).

## The deployment default

The default mail server is a URL. The scheme is required to choose the protocol and transport; a value with no scheme is read as `imaps://`.

| URL | Protocol | Transport | Default port |
|---|---|---|---|
| `imaps://host` | IMAP | implicit TLS | 993 |
| `imap://host` | IMAP | STARTTLS | 143 |
| `pop3s://host` | POP3 | implicit TLS | 995 |
| `pop3://host` | POP3 | STARTTLS | 110 |

`pops://` and `pop://` are accepted as aliases of `pop3s://` and `pop3://`. An explicit `:port` replaces the default port. The user name and password are whatever the mail server expects.

`MW_ENGINE_TLS` (`implicit`, `starttls` or `plaintext`) replaces the transport of the deployment default for every sign-in that uses it. It does not apply to an address whose domain has its own connection settings. `plaintext` sends passwords unprotected; use it only against a test server.

## Sending mail

IMAP and POP3 only receive. Outgoing mail goes through an SMTP submission service, which the server reaches with the same user name and password as the receiving account:

```sh
MW_SMTP_HOST=smtp.example.org   # defaults to the mail server's host
MW_SMTP_SECURITY=starttls       # starttls (default), implicit or plaintext
MW_SMTP_PORT=587                # default 587 for starttls, 465 for implicit, 25 for plaintext
```

These three variables apply to addresses that use the deployment default. A domain configured under **Domains** carries its own sending settings. After a message is submitted the server appends a copy to the Sent folder.

## IMAP accounts

The server maps special-use folders (Sent, Drafts, Junk, Trash) to their roles through the folder attributes the mail server reports, and uses the extensions that the server advertises for incremental synchronisation. Which extensions a given mail server offers decides how much work each synchronisation does; the server falls back where an extension is missing. The extensions the server understands are listed in [IMAP extensions](/protocols/imap-extensions).

Importing mail from a file into a folder needs an IMAP connection that is writable and advertises UIDPLUS or IMAP4rev2; see [Mail import and export](/deploy/mail-import-export).

## POP3 accounts

A POP3 account has one folder, the inbox. The server finds new messages by comparing the mail server's message identifiers (`UIDL`) with the ones it has already stored; flags and other folders exist only in Mailwoman, because POP3 has no such concepts. Mailwoman never deletes messages from the POP3 server: it always leaves them there. Sending uses the SMTP settings above.

## Other providers

Gmail, Microsoft 365 and Exchange accounts that cannot use a plain IMAP password sign-in are handled by the provider bridges; see [Mail provider bridges](/bridges/).

## Options

| Name | What it does | Default | Who can change it |
|---|---|---|---|
| `MW_MODE` | `proxy` or `engine` | `proxy` | Operator, at start |
| `MW_UPSTREAM_DEFAULT` | The default mail server URL | Not set | Operator, at start; the `upstream.default` setting is read first |
| `MW_ENGINE_TLS` | Replaces the transport of the default mail server | Not set | Operator, at start |
| `MW_SMTP_HOST`, `MW_SMTP_PORT`, `MW_SMTP_SECURITY` | The SMTP submission service for the default mail server | Mail server host, 587, `starttls` | Operator, at start |

## Troubleshooting

- **This address is not set up on this server.** No domain configuration matches and no default is set. Add the domain under **Domains** or set the default mail server.
- Sign-in is refused with `400` and **the server chooses the mail server; a sign-in sends only the address and the password**: the request names a server (a field such as `server`, `host`, `imapUrl` or `upstream`). The sign-in form never does; a custom client must send only the address and the password.
- The server logs `MW_UPSTREAM_DEFAULT=... is not ...; no sign-in can use it` when the default is not a URL the current mode can use: in engine mode it must be an `imaps://`, `imap://`, `pop3s://` or `pop3://` address.
- Sending fails while receiving works: check the SMTP host, port and security. With none set the server submits to the mail server's own host on port 587 with STARTTLS.

## Related

- [Install and configure](README#which-mail-server-users-sign-in-to)
- [Domains](/admin/domains)
- [Mail import and export](/deploy/mail-import-export)
- [Mail provider bridges](/bridges/)
- [CalDAV and CardDAV](/deploy/caldav-carddav)
