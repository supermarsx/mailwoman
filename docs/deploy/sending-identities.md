# Sending identity authority

Native submission checks the selected identity belongs to the authenticated account and matches the message From address. Without an explicit identity, the From address still needs current authority. Both initial queue admission and the final dispatch check current authority; changing a stored draft or revoking an alias does not preserve an earlier grant.

Authority comes from the authenticated runtime address, the operator's current explicit `MW_ALLOWED_FROMS` list, or an enabled masked alias owned by this account. A configured profile row, its `source` string, shared domain membership, and a client-supplied identity ID do not establish permission. Provider/proxy identity write admission consults bounded authenticated upstream `Identity/get`; upstream failures do not grant permission.

Previously saved arbitrary From profiles may now be refused and omitted from native JMAP identity selection. Choose the primary address or ask the administrator to authorize the exact alias. This compatibility change intentionally closes implicit sender grants. Editing an existing profile cannot change it to an unauthorized address. Current server allowlist removal and masked alias disablement take effect before future dispatch; already accepted upstream messages cannot be recalled.

The dispatch path checks the actual message headers, requires one From header and checks the stored envelope sender agrees with those bytes. Multiple From addresses are not supported. Redirecting an original message under somebody else's From address is refused; forwarding as an attached original remains available.

This follows [RFC 8621 section 9.6](https://www.rfc-editor.org/rfc/rfc8621.html#section-9.6). It is not proof of public-domain ownership, DKIM signing, or a complete identity verification workflow. Explicit operator allowlists remain operator authority; Mailwoman does not infer permission from an email domain.

## Default identity and variants

Each account has exactly one default identity. The first identity created is the default; making another one the default clears the previous one in the same transaction (`POST /api/account/identities/{id}/default`); deleting the default makes the oldest remaining base identity the default, and deleting the last identity leaves none. Saving an identity without `isDefault` never clears the flag. The default is what the composer and reply-identity matching (`mail.replyFromMode`) fall back to.

A variant is an identity with `variantOf` (the id of a base identity in the same account) and a required `label`. It has its own display name and address, which may equal the base's address. The optional fields it leaves empty (reply-to, automatic Bcc, organization, OpenPGP key, new-message and reply signature, and the Sent folder) are inherited from the base. An empty `signatureName` is an own value meaning "no signature". Variants are one level deep: a variant cannot have variants, and a base that has variants cannot become a variant. A variant can be the default.

Where inheritance is resolved:

- `/api/account/identities` returns the raw stored fields (`isDefault`, `variantOf`, `label` included). The settings page works out the inherited fields for display.
- JMAP `Identity/get` and every sending path (submission, signatures, read-receipt identity) use the server-resolved effective profile, so a client that does not know about variants sees an ordinary identity. `isDefault` is added to every identity; `variantOf` and `label` appear on variants only. Clients that ignore unknown properties are unaffected.
- Sending authority is unchanged: a variant's address must be allowed on its own, exactly like any other identity.

Deleting a base that has variants is refused with `409 {"code":"has_variants","variants":N}` unless the request says what to do: `DELETE /api/account/identities/{id}?variants=delete` removes the variants too; `?variants=promote` keeps them as standalone identities with the inherited fields copied in. Ownership is checked on every call; another account's ids behave as not found.
