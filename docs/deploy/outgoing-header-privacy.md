---
title: Outgoing email header privacy
---

# Outgoing email header privacy

Some mail programs add headers that name the program that wrote a message, such as `X-Mailer` or `User-Agent`. Mailwoman can remove these headers from the messages it sends. This page describes what is removed, which messages are covered, and the one case in which the setting stops a send.

![The Settings screen searched for mailer, showing the Remove outgoing mailer headers setting, on by default.](/screenshots/admin-ops/settings-mailer-headers.webp){.mw-crop}

## Before you start

The setting applies to mail that Mailwoman submits itself, over SMTP or EWS, for accounts on its built-in mail engine. It does not cover messages that a JMAP server sends after Mailwoman forwards them to it, the HTTP `User-Agent` of browser requests, or headers that a relay adds later.

## What is removed

Only these top-level headers are removed, including their folded continuation lines: `X-Mailer`, `User-Agent` and `X-MimeOLE`. Mailwoman's own composer does not add an `X-Mailer` header.

Everything else is left exactly as it was: other headers, the MIME structure, the body, the attachments and attached messages. The `Message-ID` stays. A `User-Agent` inside an abuse report payload stays too, because it is part of the report's content and not a header of the email. Signed and encrypted bodies are not rewritten.

The policy is looked up each time a message is sent, including messages that waited in the [Outbox](/admin/outbox-recall), so a change takes effect for messages that are still queued.

## Options

| Label | What it does | Default | Who can change it |
| --- | --- | --- | --- |
| **Remove outgoing mailer headers** ([`privacy.stripMailerHeaders`](/admin/settings#privacy.stripmailerheaders)) | Removes the three headers from messages sent through SMTP and EWS | on | Administrators globally and per domain, and users in **Settings**, **Privacy**, unless locked |

## When it stops a send

If a message already carries a DKIM or ARC signature that covers one of the headers that would be removed, removing the header would invalidate the signature. Mailwoman then refuses the send with the error "outgoing header privacy conflicts with pre-signed DKIM/ARC headers", before it contacts the mail server. A signature header list that cannot be read is refused in the same way. To send such a message unchanged, switch the setting off for that account first, if you are allowed to. Messages composed in Mailwoman are not signed before this step, so they are not affected.

A header block of more than 128 KiB, or one with a malformed top-level header, is refused with "outgoing header privacy: invalid or oversized top-level header block".

## Troubleshooting

- **A send fails with "outgoing header privacy conflicts with pre-signed DKIM/ARC headers".** See [When it stops a send](#when-it-stops-a-send).
- **A message still has a mailer header at the recipient.** A relay or a JMAP server that sent the message added it after Mailwoman, or the message was sent for an account that Mailwoman forwards to a JMAP server.
- **The user cannot change the setting.** An administrator has locked it for the domain or globally.

## Related

- [Settings](/admin/settings) for `privacy.stripMailerHeaders`
- [Outbox, recall and scheduled sending](/admin/outbox-recall)
- [Outbound connections (egress)](/security/egress)
