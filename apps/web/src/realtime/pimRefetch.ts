// PIM realtime refetch registration (plan §1.8/§2.2, §3 e10).
//
// The V2 push controller reconciles granular per-type state tokens
// (`realtime/changes.ts`) for mail. For PIM, t5-e8 deliberately kept the wire
// `StateChange` at the five mail fields but BROADCASTS a coarse ping (and
// advances `sessionState`) on every PIM mutation — the ping reaches connected
// WS/SSE clients. So the open PIM module refetches on the ping's ARRIVAL; it does
// not need granular PIM keys in `StateChange` (no Rust cross-crate change — no
// escalation). If e9 later adds granular PIM keys, `pimTypesInChange` picks them
// up and this still works (a superset signal).
//
// This module is the pure, testable half: given the open surface and an incoming
// `StateChange`, decide whether to refetch. The shell (`screens/Mailbox.tsx`)
// subscribes to `app.onRealtimeChange` and calls the matching module load.

import type { StateChange, TypeStates } from '../contracts/push.ts';
import type { ShellSurface } from '../shell/router.ts';

/** The PIM datatypes each module surface reads (for granular refetch, when present). */
export const MODULE_PIM_TYPES: Record<string, readonly (keyof TypeStates)[]> = {
  calendar: ['Calendar', 'CalendarEvent'],
  // Task lists are VTODO `Calendar` rows, so a `Calendar` change is relevant too.
  tasks: ['Task', 'Calendar'],
  notes: ['Note'],
  contacts: ['AddressBook', 'ContactCard', 'ContactGroup'],
};

const PIM_TYPE_KEYS: readonly (keyof TypeStates)[] = [
  'Calendar',
  'CalendarEvent',
  'Task',
  'Note',
  'AddressBook',
  'ContactCard',
  'ContactGroup',
];

/** The PIM datatypes named in a `StateChange` (across all accounts). Empty when
 *  the change carries only mail keys — i.e. it is the coarse ping. */
export function pimTypesInChange(change: StateChange): (keyof TypeStates)[] {
  const seen = new Set<keyof TypeStates>();
  for (const states of Object.values(change.changed)) {
    for (const key of PIM_TYPE_KEYS) {
      if (states[key] !== undefined) seen.add(key);
    }
  }
  return [...seen];
}

/**
 * Should the open PIM `surface` refetch on this `StateChange`?
 *   - If the change names granular PIM keys, refetch only when one is relevant
 *     to the surface (precise).
 *   - If it names none (the coarse ping t5-e8 broadcasts on every PIM mutation),
 *     refetch the open surface — the ping's arrival is the signal.
 * A non-PIM surface (mail/outbox/attachments) never triggers a PIM refetch here.
 */
export function shouldRefetchPim(surface: ShellSurface, change: StateChange): boolean {
  const relevant = MODULE_PIM_TYPES[surface];
  if (relevant === undefined) return false;
  const named = pimTypesInChange(change);
  if (named.length === 0) return true; // coarse ping → refetch the open module
  return named.some((t) => relevant.includes(t));
}
