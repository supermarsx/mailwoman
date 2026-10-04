// Participant addressing rules, as the engine applies them.
//
// A `CalendarEvent/set` create or update is REFUSED (`invalidProperties`) when a
// participant-map key — or an entry's `email` — is not a mailbox
// (`check_participants`, `crates/mw-engine/src/pim/events.rs`, 26.20 t27-e2). The
// key is what an iTIP message is addressed to, and the stored projection is
// re-keyed by `email` on every write (`read_participants`,
// `crates/mw-ics/src/ical.rs:196-227`). So the editor keys every participant by
// its address and checks an address before it enters the list, instead of
// letting the whole save be refused later.

/** The byte cap `validate_mailbox` applies (`MAX_ADDR_BYTES`). */
const MAX_ADDR_BYTES = 320;

/**
 * Whether `addr` is a mailbox the engine accepts — a mirror of
 * `mw_smtp::validate_mailbox` + `check_chars` (`crates/mw-smtp/src/addr.rs`).
 * Refused: empty; over 320 bytes; any control character or whitespace; `<` `>`
 * `"` `\` `,` `;` `(` `)`; and anything that is not exactly one `@` with a
 * non-empty local part and a non-empty domain. Non-ASCII is allowed.
 */
export function isMailbox(addr: string): boolean {
  if (addr === '') return false;
  if (new TextEncoder().encode(addr).length > MAX_ADDR_BYTES) return false;
  if (/[\p{Cc}\p{White_Space}<>"\\,;()]/u.test(addr)) return false;
  const parts = addr.split('@');
  return parts.length === 2 && parts[0] !== '' && parts[1] !== '';
}

/**
 * The signed-in user's own entry in a participant map, or `null`. The engine
 * looks the user up by an EXACT match of the account identity against the map
 * key (`event_respond`: `parts.get_mut(&me)`), so this does too: a
 * case-insensitive match here would show response controls for an entry the
 * engine will not update.
 */
export function ownParticipant<P>(participants: Record<string, P>, identity: string | null): P | null {
  if (identity === null || identity === '') return null;
  return Object.prototype.hasOwnProperty.call(participants, identity) ? participants[identity]! : null;
}
