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

**Not built (recorded 26.19).** This section described the composer offering "send
from a masked alias" and minting a new alias inline. Neither affordance exists in
`Compose.tsx`. The alias set is scoped to the session account and is reachable
through the routes above and the settings surface only.

Two related facts worth having in one place, because the naming invites the wrong
conclusion:

- **The masked-email feature works.** Alias lifecycle and on-send `From`
  enforcement are server-side (`crates/mw-server/src/masked.rs`, `MaskedSubmitter`),
  with a live end-to-end test.
- **The `plugins/masked-email` crate does not.** It is the abandoned first design
  and is unreachable from anything: no crate depends on it, `plugins/dist/masked-email.wasm`
  is absent while the other seven are present, it has no `FIRST_PARTY_DIGESTS`
  entry so the host could never resolve it, and it has no `plugin.toml`, no
  `build.sh` and no CI step. Its one hook is still the original
  `fn message_out(raw) { Ok(raw) }` scaffold. Deleting the crate or finishing and
  registering it is an open decision; until then, the "on-send rewrite" half of
  the section above describes the crate's *intent*, and the enforcement that
  actually runs is the server-side one.

## Data

`masked_email(id, account_id, alias_addr, target_desc, state, created_at,
last_used_at)`. `state` is `enabled | disabled | deleted`. No mail content is stored.

<!-- e7: fill the alias generator (unique local-part), the on-send header rewrite,
and the composer integration. -->
