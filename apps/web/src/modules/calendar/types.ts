// View-local types for the calendar module (plan §3 e4). These are UI/rendering
// types layered over the frozen `CalendarEvent` surface (`api/pim-types.ts`);
// they never leave the module.

import type { CalendarEvent, Participant } from '../../api/pim-types.ts';

/** The nine calendar views (plan §0.1 / §3 e4). */
export type CalendarView =
  | 'day'
  | '3day'
  | 'work-week'
  | 'week'
  | 'month'
  | 'tri-month'
  | 'schedule'
  | 'agenda'
  | 'year';

/** All views in nav order, with a short label. */
export const CALENDAR_VIEWS: ReadonlyArray<{ id: CalendarView; label: string }> = [
  { id: 'day', label: 'Day' },
  { id: '3day', label: '3 Day' },
  { id: 'work-week', label: 'Work Week' },
  { id: 'week', label: 'Week' },
  { id: 'month', label: 'Month' },
  { id: 'tri-month', label: 'Quarter' },
  { id: 'schedule', label: 'Schedule' },
  { id: 'agenda', label: 'Agenda' },
  { id: 'year', label: 'Year' },
];

/**
 * One concrete, dated occurrence of an event within a queried window — the shape
 * every view renders. The engine (`CalendarEvent/expand`) produces these; the
 * mock + the client-side recurrence helper produce the same shape so views are
 * agnostic to the backend (plan §2.1: "the web receives expanded instances").
 */
export interface EventInstance {
  /** Stable per-instance key (`event.id` + occurrence start epoch). */
  key: string;
  /** The master event this occurrence derives from. */
  event: CalendarEvent;
  /** Occurrence start (absolute instant, resolved from the local wall time). */
  start: Date;
  /** Occurrence end (`start` + duration). */
  end: Date;
  /** True when this instance is an all-day / date-only event. */
  allDay: boolean;
  /** True when produced by a recurrence rule (not the single master). */
  recurring: boolean;
  /** The calendar's display color, resolved for rendering. */
  color: string;
}

/** A pair of overlapping instances (from `Calendar/detectConflicts`), carrying
 *  the overlap window so the resolver can render + act on it. */
export interface ConflictPair {
  a: string;
  b: string;
  /** Overlap window start (`LocalDateTime` / RFC3339 from the engine). */
  overlapStart: string;
  /** Overlap window end. */
  overlapEnd: string;
}

// ── Attendee roles + kind (iCal ROLE / CUTYPE, JSCalendar-aligned) ────────────
//
// The event editor's attendee picker binds to the JSCalendar attendee-role
// fields the ICS layer (e-ics) round-trips: `roles` — a JSMap set of role→true
// (from iCal ROLE: CHAIR / REQ-PARTICIPANT / OPT-PARTICIPANT / NON-PARTICIPANT)
// — and `kind` (from iCal CUTYPE: INDIVIDUAL / GROUP / RESOURCE / ROOM). These
// are additive over the frozen `Participant` shape (api/pim-types.ts): when
// present they are read/written, otherwise the module derives the role from the
// legacy singular `role` string and defaults the kind to `individual`.

/** The ROLE picker values (iCal ROLE → JSCalendar `roles` JSMap keys). */
export type AttendeeRole = 'chair' | 'required' | 'optional' | 'non-participant';
export const ATTENDEE_ROLES: readonly AttendeeRole[] = ['chair', 'required', 'optional', 'non-participant'];

/** The CUTYPE picker values (iCal CUTYPE → JSCalendar participant `kind`). */
export type AttendeeCutype = 'individual' | 'group' | 'resource' | 'room';
export const ATTENDEE_CUTYPES: readonly AttendeeCutype[] = ['individual', 'group', 'resource', 'room'];

/**
 * `Participant` overlaid with the optional JSCalendar attendee-role fields
 * (`roles` JSMap + `kind`) that e-ics emits. Built against the frozen shape:
 * both fields are optional so the module round-trips them when present and
 * degrades to the legacy singular `role` string when absent.
 */
export interface ParticipantExt extends Participant {
  roles?: Record<string, boolean>;
  kind?: string;
}

/** Map a UI `AttendeeRole` to the JSCalendar `roles` JSMap e-ics serializes. */
export function attendeeRoleToRoles(role: AttendeeRole): Record<string, boolean> {
  switch (role) {
    case 'chair':
      return { chair: true, attendee: true };
    case 'optional':
      return { attendee: true, optional: true };
    case 'non-participant':
      return { informational: true };
    case 'required':
    default:
      return { attendee: true };
  }
}

/** The legacy singular `role` string kept in sync for back-compat. */
export function attendeeRoleToLegacy(role: AttendeeRole): string {
  return role === 'required' ? 'attendee' : role;
}

/**
 * Derive the UI `AttendeeRole` from a participant: prefer the `roles` JSMap,
 * else fall back to the legacy singular `role` string.
 */
export function participantRole(p: ParticipantExt): AttendeeRole {
  const roles = p.roles;
  if (roles !== undefined) {
    if (roles['chair'] === true) return 'chair';
    if (roles['optional'] === true) return 'optional';
    if (roles['informational'] === true) return 'non-participant';
    if (roles['attendee'] === true) return 'required';
  }
  const legacy = p.role.toLowerCase();
  if (legacy === 'chair' || legacy === 'owner' || legacy === 'organizer') return 'chair';
  if (legacy === 'optional') return 'optional';
  if (legacy === 'non-participant' || legacy === 'informational') return 'non-participant';
  return 'required';
}

/** Derive the UI `AttendeeCutype` from a participant's `kind` (default individual). */
export function participantCutype(p: ParticipantExt): AttendeeCutype {
  const kind = (p.kind ?? '').toLowerCase();
  return (ATTENDEE_CUTYPES as readonly string[]).includes(kind) ? (kind as AttendeeCutype) : 'individual';
}

// ── Event categories + attachments (P4 / P5) ─────────────────────────────────
//
// The frozen `CalendarEvent` surface (`api/pim-types.ts`) predates the P4
// categories + P5 attachments the engine (e11) now accepts on create/set. Rather
// than edit that frozen shape, the module overlays the two optional fields here —
// the same additive pattern `ParticipantExt` uses for the JSCalendar attendee
// fields. Both are optional so an event that predates them round-trips unchanged
// (the editor reads `categories`/`attachments` when present, defaults to `[]`).

/** A JSCalendar `links`-style attachment on an event (P5). One of `blobId`/`uri`
 *  identifies the payload; `title` is the display name shown in the editor. */
export interface EventAttachment {
  /** An uploaded-blob id (Mailwoman blob store), when the file was uploaded. */
  blobId?: string;
  /** An external URI (e.g. a shared-drive link), when not an uploaded blob. */
  uri?: string;
  /** Display name for the attachment. */
  title?: string;
  /** MIME type, when known. */
  contentType?: string;
}

/**
 * `CalendarEvent` overlaid with the optional P4/P5 fields the engine round-trips
 * on `CalendarEvent/set` (`categories` free-form tags, `attachments`). Built over
 * the frozen shape: both are optional, so the module reads/writes them when
 * present and degrades cleanly when absent.
 */
export interface CalendarEventExt extends CalendarEvent {
  categories?: string[];
  attachments?: EventAttachment[];
}

/** Read an event's categories (P4), tolerating the frozen shape without them. */
export function eventCategories(ev: CalendarEvent): string[] {
  const cats = (ev as CalendarEventExt).categories;
  return Array.isArray(cats) ? cats : [];
}

/** Read an event's attachments (P5), tolerating the frozen shape without them. */
export function eventAttachments(ev: CalendarEvent): EventAttachment[] {
  const atts = (ev as CalendarEventExt).attachments;
  return Array.isArray(atts) ? atts : [];
}
