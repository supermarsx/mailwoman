# Sending identities and sender authority

Users choose the address they send from among their **identities** (personal settings). An identity holds a display name, an address and optional profile fields such as a signature or a reply-to address. Which addresses a user may actually send from is decided by the server, not by the identity list. This page explains that decision and the server-side rules for identities; it is meant for operators. Users manage their identities in [their account settings](../user/settings/account.md).

## Which From addresses a user may send from

In engine mode (IMAP or POP3 with SMTP, JMAP submission or EWS), an address is allowed for an account when it is:

1. the account's own sign-in address; or
2. listed in the `MW_ALLOWED_FROMS` environment variable; or
3. an enabled masked alias that belongs to this account.

Nothing else grants permission: not an identity row the user created, not sharing the account's domain, not the identity ID a client sends. Comparison ignores letter case.

::: warning MW_ALLOWED_FROMS applies to every account
`MW_ALLOWED_FROMS` is one list for the whole server. Every account may send from every address in it. Use it only for addresses that all users of the server are meant to share, such as a common team address, and only when every account belongs to people trusted to use them.
:::

`MW_ALLOWED_FROMS` is either a list of addresses separated by commas, spaces or new lines, or a JSON array with display names and reply-to addresses:

```sh
MW_ALLOWED_FROMS="team@example.com, noreply@example.com"
MW_ALLOWED_FROMS='[{"email":"team@example.com","name":"Example Team","replyTo":"help@example.com"}]'
```

It is part of the server's environment, so a change takes effect after a restart; there is no console setting for it. A value that cannot be parsed counts as an empty list.

In proxy mode, the upstream JMAP server decides. When the web app resolves the sender of a message, Mailwoman asks the upstream server for the account's identities (`Identity/get`) and accepts only an address it lists; if that request fails, the address is refused. The submission itself is then checked by the upstream server.

## When the check runs

- **Not when an identity is saved.** An identity is a profile, not a permission: users can save any address in it. An identity with an address that is not allowed simply cannot be used to send.
- **When the web app picks the sender.** The composer asks the server whether the chosen From address is allowed (`POST /api/account/sender/resolve`).
- **When identities are listed to mail clients.** JMAP `Identity/get` returns only identities whose address is allowed at that moment, so an identity that lost its permission disappears from the composer of JMAP clients.
- **When a message is submitted.** The message must have exactly one From address, it must be allowed, and when the client names an identity, that identity must belong to the account and carry the same address. Otherwise the submission is refused with `forbiddenFrom` ("This sending address is not authorized for this account. Choose an authorized identity.").
- **When a queued message is finally sent.** Scheduled and queued mail is checked again against the message's own headers: exactly one `From` header, and its address still allowed. Removing an address from `MW_ALLOWED_FROMS` or disabling a masked alias therefore also stops mail already waiting in the queue. Mail already handed to the mail server cannot be recalled.

The same rule also applies to read receipts, calendar invitations, and drafts saved with a chosen identity.

## What the check does not prove

The check stops one user from sending as another address on this server. It does not prove that the domain belongs to you, sign messages (DKIM) or verify ownership of an address. The receiving side judges the message by your domain's SPF, DKIM and DMARC records.

## Default identity and variants

These rules apply to the identities a user stores in Mailwoman:

- Each account has exactly one default identity. The first identity created becomes the default; making another one the default clears the previous one. Deleting the default makes the oldest remaining base identity the default (or the oldest remaining variant when only variants are left). The default is what the composer and reply-address matching fall back to.
- A **variant** belongs to a base identity of the same account and has its own label, display name and address (which may equal the base's). Optional fields it leaves empty (reply-to, automatic Bcc, organization, OpenPGP key, signatures, Sent folder) are taken from the base. Variants are one level deep.
- Deleting a base identity that has variants is refused with `409` and the code `has_variants` unless the request says what to do with them: `?variants=delete` removes them too, `?variants=promote` keeps them as ordinary identities with the inherited fields copied in.
- A variant's address must be allowed on its own, like any identity.

Mail clients using JMAP see variants as ordinary identities with the inherited fields filled in.

## Troubleshooting

| Symptom | Cause | Fix |
|---|---|---|
| Sending from an identity is refused with `forbiddenFrom` | Its address is not the sign-in address, not in `MW_ALLOWED_FROMS` and not an enabled masked alias of the account | Use an allowed address, or add it to `MW_ALLOWED_FROMS` and restart, if every user may use it |
| An identity is missing from a JMAP mail client's list | Its address is not allowed | As above |
| A scheduled message failed with "message must have exactly one From header" | The message had no or several `From` headers | Send it again with one sender |

## Related

- [Domains](../admin/domains.md)
- [Masked email](../plugins/masked-email.md)
- [Configuration reference](../admin/configuration.md)
