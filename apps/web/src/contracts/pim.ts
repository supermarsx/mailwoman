// FROZEN Mailwoman PIM method-family contract (plan §2.2). Every family rides
// the existing `/jmap/api` envelope (methodCalls, `#`-result-references,
// `{accountId,state,list,notFound}` / `{created,updated,destroyed,...}` shapes),
// so the web transport/offline/push layers are reused verbatim (plan §1.1).
//
// Authored by e0; e4–e7 call these against a mock, e8 implements them engine-
// side, e10 wires the real surface. Mirrors the engine's `dispatch_pim` arms in
// `crates/mw-engine/src/pim/dispatch.rs` — the two sets MUST stay in lockstep.

import {
  CAP_CALENDARS,
  CAP_CONTACTS,
  CAP_NOTES,
  CAP_TASKS,
} from '../api/pim-types.ts';

/** The Mailwoman PIM capability URNs, in `JmapRequest.using` order (§2.2). */
export const PIM_CAPABILITIES = [
  CAP_CALENDARS,
  CAP_TASKS,
  CAP_NOTES,
  CAP_CONTACTS,
] as const;

/** Calendar + event method names (§2.2). */
export type CalendarMethod =
  | 'Calendar/get'
  | 'Calendar/set'
  | 'Calendar/changes'
  | 'Calendar/freeBusy'
  | 'Calendar/detectConflicts'
  | 'CalendarEvent/get'
  | 'CalendarEvent/set'
  | 'CalendarEvent/query'
  | 'CalendarEvent/queryChanges'
  | 'CalendarEvent/changes'
  | 'CalendarEvent/expand'
  | 'CalendarEvent/parse'
  | 'CalendarEvent/import'
  | 'CalendarEvent/export'
  | 'CalendarEvent/respond';

/** Task method names (§2.2). */
export type TaskMethod =
  | 'Task/get'
  | 'Task/set'
  | 'Task/query'
  | 'Task/queryChanges'
  | 'Task/changes';

/** Note method names (§2.2). */
export type NoteMethod = 'Note/get' | 'Note/set' | 'Note/query' | 'Note/changes';

/** Contact method names (§2.2). */
export type ContactMethod =
  | 'AddressBook/get'
  | 'AddressBook/set'
  | 'AddressBook/changes'
  | 'ContactCard/get'
  | 'ContactCard/set'
  | 'ContactCard/query'
  | 'ContactCard/queryChanges'
  | 'ContactCard/changes'
  | 'ContactCard/import'
  | 'ContactCard/export'
  | 'ContactCard/merge'
  | 'ContactCard/autocomplete'
  | 'ContactGroup/get'
  | 'ContactGroup/set'
  | 'ContactGroup/changes';

/** The full PIM method-name union (§2.2). */
export type PimMethod = CalendarMethod | TaskMethod | NoteMethod | ContactMethod;

/** The PIM datatypes carried in a `StateChange.changed[account]` map + the
 *  per-type changes families (plan §1.8/§2.2). Kept in sync with `push.ts`. */
export type PimStateType =
  | 'Calendar'
  | 'CalendarEvent'
  | 'Task'
  | 'Note'
  | 'AddressBook'
  | 'ContactCard'
  | 'ContactGroup';

/** The PIM datatypes as an array, for realtime change-type dispatch (e10). */
export const PIM_STATE_TYPES: readonly PimStateType[] = [
  'Calendar',
  'CalendarEvent',
  'Task',
  'Note',
  'AddressBook',
  'ContactCard',
  'ContactGroup',
];
