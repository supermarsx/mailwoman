# Sending identity authority

Native submission checks the selected identity belongs to the authenticated account and matches the message From address. Without an explicit identity, the From address still needs current authority. Both initial queue admission and the final dispatch check current authority; changing a stored draft or revoking an alias does not preserve an earlier grant.

Authority comes from the authenticated runtime address, the operator's current explicit `MW_ALLOWED_FROMS` list, or an enabled masked alias owned by this account. A configured profile row, its `source` string, shared domain membership, and a client-supplied identity ID do not establish permission. Provider/proxy identity write admission consults bounded authenticated upstream `Identity/get`; upstream failures do not grant permission.

Previously saved arbitrary From profiles may now be refused and omitted from native JMAP identity selection. Choose the primary address or ask the administrator to authorize the exact alias. This compatibility change intentionally closes implicit sender grants. Editing an existing profile cannot change it to an unauthorized address. Current server allowlist removal and masked alias disablement take effect before future dispatch; already accepted upstream messages cannot be recalled.

The dispatch path checks the actual message headers, requires one From header and checks the stored envelope sender agrees with those bytes. Multiple From addresses are not supported. Redirecting an original message under somebody else's From address is refused; forwarding as an attached original remains available.

This follows [RFC 8621 section 9.6](https://www.rfc-editor.org/rfc/rfc8621.html#section-9.6). It is not proof of public-domain ownership, DKIM signing, or a complete identity verification workflow. Explicit operator allowlists remain operator authority; Mailwoman does not infer permission from an email domain.
