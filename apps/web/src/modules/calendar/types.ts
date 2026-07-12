// View-local types for the calendar module (plan §3 e4). These are UI/rendering
// types layered over the frozen `CalendarEvent` surface (`api/pim-types.ts`);
// they never leave the module.

import type { CalendarEvent } from '../../api/pim-types.ts';

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

/** A pair of instances that overlap in time (from `Calendar/detectConflicts`). */
export interface ConflictPair {
  a: string;
  b: string;
}
