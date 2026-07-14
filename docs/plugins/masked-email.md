# Masked-email (SPEC §28.4)

> **Status:** scaffold (t10-e0). Filled by t10-e7; live-E2E by t10-e14/e15.

Masked-email lets a user generate a per-target alias address so a recipient (a shop,
a newsletter) never sees the real mailbox. Aliases can be enabled, disabled, and
deleted; incoming mail to a disabled/deleted alias is dropped upstream.

## Two halves

- **Server-side lifecycle** (`crates/mw-server/src/masked.rs` + `mw-store` 0010
  `masked_email`, `mw_store::MaskedEmailRow`): generate / list / enable / disable /
  delete an alias, plus a user-facing target description. Routes:
  `GET/POST /api/masked`, `POST /api/masked/{id}/state`, `DELETE /api/masked/{id}`.
- **On-send rewrite** (`plugins/masked-email`, a `message-pipeline::message-out`
  component): rewrites the outgoing message's sender to the selected alias so the
  recipient only ever sees the masked address.

## Composer surfacing

The composer offers "send from a masked alias" and can mint a new alias inline (e7 +
the web compose surface). The alias set is scoped to the session account.

## Data

`masked_email(id, account_id, alias_addr, target_desc, state, created_at,
last_used_at)`. `state` is `enabled | disabled | deleted`. No mail content is stored.

<!-- e7: fill the alias generator (unique local-part), the on-send header rewrite,
and the composer integration. -->
