---
title: Masked email addresses
---

# Masked email addresses

A masked address is a generated alias that a person can give to a shop or a newsletter instead of their real address. Mailwoman keeps the list of aliases, lets the owner switch each one on or off, and lets mail be sent from an enabled alias.

::: warning Interface and delivery
- The web client has no screen for masked addresses and the composer has no control to pick or create one. Aliases are managed through the HTTP routes below.
- Mailwoman does not receive or forward mail for an alias. The alias domain's mail server must deliver mail for the alias to the mailbox. Switching an alias off in Mailwoman does not by itself stop delivery.
:::

This page is not about the `plugins/masked-email` source in the repository. That component is not built, not registered and not used; masked addresses are implemented in the server.

## Before you start

- The server runs in engine mode.
- The feature **masked** is on (`cap.masked`, default on) for the user.
- Choose the alias domain with the setting `maskedEmail.domain` (environment `MW_MASKED_EMAIL_DOMAIN`). When it is empty, aliases are created under the domain of the signed-in address. The admin settings help says an empty value turns masked addresses off; the server does not do that, so set the permission rather than relying on an empty domain.

## Manage aliases

All routes need a mailbox session and only ever see the signed-in account's aliases. An alias that belongs to another account answers as not found.

| Request | Effect |
|---|---|
| `GET /api/masked` | List the account's aliases that are not deleted, newest first. |
| `POST /api/masked` | Create an alias. The body is optional: `target` (where the alias forwards, default the account's own address), `description` and `forDomain` (the site the alias is for). The alias is enabled on creation. |
| `POST /api/masked/{id}/state` | Set the state to `enabled` or `disabled`. |
| `DELETE /api/masked/{id}` | Delete the alias. It is kept with the state `deleted` and is no longer listed. |

An alias looks like `<12 hex characters>@<alias domain>`. The characters are derived from a fresh random value, so they cannot be guessed, and the server checks that the address is not already in use.

Each alias is returned with `id`, `email`, `state`, `target`, `description`, `forDomain`, `createdAt` and `lastUsedAt`. Mail content is never stored.

## Sending from an alias

For accounts that send through SMTP, a message whose sender address is an alias is checked before it is submitted:

- An enabled alias that belongs to the account is used as the envelope sender, so bounces and replies route through the alias domain, and `lastUsedAt` is updated.
- A disabled or deleted alias, or another account's alias, is refused and the message is not sent.
- Any other sender address is sent unchanged.

Accounts served by a bridge (Graph, Gmail, EWS) send through the provider and are not checked. The server changes only the envelope sender; the `From:` header is left as the composer set it.

## Related

- [Mail provider bridges](../bridges/index.md)
- [Administrator settings](../admin/settings.md)
