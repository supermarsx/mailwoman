// FROZEN Mailwoman PIM object types (plan §2.1) — the calendar / tasks / notes /
// contacts shapes the web client and the engine agree on, byte-for-byte with
// `crates/mw-engine/src/pim/types.rs`. Mailwoman-native, JSCalendar/JSContact-
// aligned, camelCase; NOT the IETF JMAP-Calendars/Contacts drafts (plan §1.1).
//
// Authored by e0; the four Batch-B web modules (e4–e7) consume these against a
// mock until e10 swaps in the real engine surface. Field names match §2.1
// EXACTLY so a future migration is mechanical.

import type { Id, UtcDate } from './jmap-types.ts';

/** A wall-clock local date-time without offset (`"2026-07-12T09:00:00"`). */
export type LocalDateTime = string;
/** An IANA time-zone id (`"Europe/Lisbon"`), or `null` for floating. */
export type Tzid = string;
/** An ISO 8601 duration (`"PT1H"`). */
export type Iso8601Duration = string;
/** A date-only value (`"2026-07-12"`). */
export type CalDate = string;

/** Mailwoman PIM capability URNs advertised in the session (plan §1.1/§2.2). */
export const CAP_CALENDARS = 'urn:mailwoman:calendars';
export const CAP_TASKS = 'urn:mailwoman:tasks';
export const CAP_NOTES = 'urn:mailwoman:notes';
export const CAP_CONTACTS = 'urn:mailwoman:contacts';

// ── Calendars ───────────────────────────────────────────────────────────────

/** One `shareWith` grant on a calendar (Mailwoman-native ACL sharing). */
export interface CalendarShare {
  principal: string;
  access: 'read' | 'readWrite';
}

/** A calendar collection (§2.1). */
export interface Calendar {
  id: Id;
  name: string;
  color: string;
  order: number;
  isVisible: boolean;
  isSubscribed: boolean;
  /** `"default"` for the primary calendar, else `null`. */
  role: 'default' | null;
  shareWith: CalendarShare[];
  caldavUrl: string | null;
  syncToken: string | null;
  isReadOnlyOverlay: boolean;
}

/** A named event location (§2.1 `locations:[{name}]`). */
export interface EventLocation {
  name: string;
}

/** An event participant (attendee/organizer), keyed by id in the event map. */
export interface Participant {
  name: string;
  email: string;
  role: string;
  participationStatus: 'needs-action' | 'accepted' | 'declined' | 'tentative';
  expectReply: boolean;
}

/** A VALARM reminder (§2.1 `alerts`), keyed by id in the event map. */
export interface Alert {
  /** `{offset}` (relative) or `{absolute}` — free-form JSCalendar trigger. */
  trigger: Record<string, unknown>;
  action: 'display' | 'email';
}

/** A calendar event (§2.1, JSCalendar-aligned). */
export interface CalendarEvent {
  id: Id;
  calendarId: Id;
  uid: string;
  title: string;
  description: string;
  locations: EventLocation[];
  start: LocalDateTime;
  timeZone: Tzid | null;
  duration: Iso8601Duration;
  showWithoutTime: boolean;
  /** Free-form JSCalendar recurrence rules. */
  recurrenceRules: Array<Record<string, unknown>>;
  /** `{date: PatchObject}` — per-instance overrides. */
  recurrenceOverrides: Record<string, Record<string, unknown>>;
  excludedRecurrenceDates: CalDate[];
  status: 'confirmed' | 'tentative' | 'cancelled';
  priority: number;
  freeBusyStatus: 'free' | 'busy';
  participants: Record<string, Participant>;
  alerts: Record<string, Alert>;
  sequence: number;
  etag: string | null;
}

// ── Tasks ───────────────────────────────────────────────────────────────────

/** A task (§2.1, VTODO-aligned). */
export interface Task {
  id: Id;
  listId: Id;
  uid: string;
  title: string;
  description: string;
  start: LocalDateTime | null;
  due: LocalDateTime | null;
  timeZone: Tzid | null;
  priority: number;
  percentComplete: number;
  status: 'needs-action' | 'in-process' | 'completed' | 'cancelled';
  progress: string;
  recurrenceRules: Array<Record<string, unknown>>;
  /** Parent task id for subtasks (RELATED-TO), or `null`. */
  parentId: Id | null;
  /** The date this task is pinned to My Day / Today, or `null`. */
  myDayDate: CalDate | null;
  etag: string | null;
}

// ── Notes ───────────────────────────────────────────────────────────────────

/** A cross-link from a note to a message / event / contact (§2.1 `links`). */
export interface NoteLink {
  type: 'email' | 'event' | 'contact';
  id: Id;
}

/** A Mailwoman-native note (§2.1). Body is sealed at rest server-side; the
 *  client sends/receives it in the clear over the same-origin channel. */
export interface Note {
  id: Id;
  notebookId: Id;
  title: string;
  tags: string[];
  color: string;
  pinned: boolean;
  bodyHtml: string;
  bodyText: string;
  links: NoteLink[];
  createdAt: UtcDate;
  updatedAt: UtcDate;
}

// ── Contacts ────────────────────────────────────────────────────────────────

/** An address book (§2.1). */
export interface AddressBook {
  id: Id;
  name: string;
  isDefault: boolean;
  carddavUrl: string | null;
  syncToken: string | null;
}

/** A structured contact name (§2.1 `name:{full,given,surname,prefix,suffix}`). */
export interface ContactName {
  full: string;
  given: string;
  surname: string;
  prefix: string;
  suffix: string;
}

/** A contact email with context + preference. */
export interface ContactEmail {
  context: string;
  value: string;
  pref: number;
}

/** A generic contexted contact value (phones / online services). */
export interface ContactValue {
  context: string;
  value: string;
}

/** A birthday / anniversary (§2.1 `anniversaries:[{kind,date}]`). */
export interface Anniversary {
  kind: 'birthday' | 'anniversary';
  date: CalDate;
}

/** A contact card (§2.1, JSContact-aligned). `pgpKey`/`smimeCert` are opaque
 *  placeholders — PGP/S-MIME wiring is V4. */
export interface ContactCard {
  id: Id;
  addressBookId: Id;
  uid: string;
  kind: 'individual' | 'org';
  name: ContactName;
  nicknames: string[];
  organizations: string[];
  titles: string[];
  emails: ContactEmail[];
  phones: ContactValue[];
  onlineServices: ContactValue[];
  addresses: Array<Record<string, unknown>>;
  anniversaries: Anniversary[];
  notes: string;
  photoBlobId: Id | null;
  isFavorite: boolean;
  groupIds: Id[];
  pgpKey: string | null;
  smimeCert: string | null;
  etag: string | null;
}

/** A contact group / distribution list (§2.1). */
export interface ContactGroup {
  id: Id;
  addressBookId: Id;
  name: string;
  memberIds: Id[];
}
