# Internationalized SMTP submission

Mailwoman submits mail to the account's configured SMTP relay; it is not an inbound SMTP MTA. The relay and the destination must support the required extensions.

The native submitter negotiates `SMTPUTF8` when an envelope mailbox or the serialized message headers contain UTF-8. It requests `BODY=8BITMIME` for these transactions and for other messages containing 8-bit MIME. If the relay lacks a required extension, submission fails before authentication or an envelope is sent. This is a permanent capability error, not an automatic retry. Mailwoman does not silently downgrade international addresses or rewrite already-signed MIME to work around a relay limitation.

Mailbox validation accepts internationalized dot-atoms and quoted local parts, including quoted spaces and escaped printable ASCII. It preserves the supplied local part and domain. Limits are measured in UTF-8 bytes: 64 for the local part and 254 for the mailbox inside the SMTP path. DNS domains are checked using IDNA and DNS label limits without rewriting the transmitted spelling; IPv4 and `IPv6:` address literals are accepted. Controls, malformed quoting, empty dot-atoms, invalid DNS labels, and unquoted whitespace are refused. Unicode whitespace is also refused inside local parts, except ordinary ASCII spaces in quoted local parts.

Delivery-status requests use `ORCPT=utf-8` and RFC 6533 unitext escaping for international recipients; ASCII recipients retain `ORCPT=rfc822`. Read receipts containing international recipient or report fields use `message/global-disposition-notification`. Incoming global delivery/disposition reports are parsed as untrusted reports, not as proof of delivery or identity.

The browser and native MIME builder use the same mailbox grammar. A JMAP proxy provider still owns its own SMTP negotiation and delivery behavior. Successful local wire tests do not establish interoperability with every external provider.

Standards: [RFC 6531](https://www.rfc-editor.org/rfc/rfc6531), [RFC 6532](https://www.rfc-editor.org/rfc/rfc6532), [RFC 6533](https://www.rfc-editor.org/rfc/rfc6533), [RFC 6152](https://www.rfc-editor.org/rfc/rfc6152).
