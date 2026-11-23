# Internationalized SMTP submission

In engine mode, Mailwoman submits each message to the SMTP server configured for the account's domain. This page describes how it handles addresses and headers that are not plain ASCII, for example `josé@example.org` or a subject with accented letters. The SMTP server and the recipient's servers must support the same extensions; Mailwoman does not rewrite a message to get past a server that does not. In proxy mode the JMAP provider submits mail and handles these extensions itself.

## What Mailwoman asks of the SMTP server

| The message has | Mailwoman sends | If the server does not offer it |
|---|---|---|
| A non-ASCII envelope address (sender or recipient), or non-ASCII header bytes | `SMTPUTF8` on `MAIL FROM`, and `BODY=8BITMIME` | Sending stops before Mailwoman authenticates or sends the envelope |
| 8-bit content anywhere else in the message | `BODY=8BITMIME` | Sending stops before Mailwoman authenticates |

A message stopped this way is not retried automatically: the error is permanent for that server. The address or headers are never converted to ASCII, and a signed message is never re-encoded.

## Addresses

Mailwoman accepts internationalized local parts and domain names, quoted local parts, and IPv4 and `IPv6:` address literals. Lengths are counted in UTF-8 bytes: at most 64 for the local part and 254 for the whole address. Domain names are checked against IDNA and DNS label rules but sent as written. Control characters, malformed quoting, empty dot-separated parts, invalid domain labels and unquoted whitespace are refused.

## Delivery status and read receipts

- A delivery-status request (DSN) for a recipient with a non-ASCII address names it as `ORCPT=utf-8;…`; ASCII recipients use `ORCPT=rfc822;…`.
- A read receipt that contains non-ASCII recipient or report fields is sent as `message/global-disposition-notification`, and incoming reports of that type are recognized.

Standards: [RFC 6531](https://www.rfc-editor.org/rfc/rfc6531), [RFC 6532](https://www.rfc-editor.org/rfc/rfc6532), [RFC 6533](https://www.rfc-editor.org/rfc/rfc6533), [RFC 6152](https://www.rfc-editor.org/rfc/rfc6152).

## Related

- [SMTP transport policies](./smtp-transport-policy.md)
- [Domains](./domains.md)
