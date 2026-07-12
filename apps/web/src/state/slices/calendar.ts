// Calendar store slice (plan §2.5, §3 e0 → filled by e4). e0 provides the frozen
// slice seam composed into `AppState`; e4 fills the signals + actions over the
// `Calendar/*`/`CalendarEvent/*` surface (mock until e10). Disjoint file — no
// `store.ts` collision with the other PIM slices (same discipline as V2).

import { createSignal, type Accessor } from 'solid-js';
import type { Calendar, CalendarEvent } from '../../api/pim-types.ts';
import type { SliceContext } from './context.ts';

export interface CalendarSlice {
  calendars: Accessor<Calendar[]>;
  calendarEvents: Accessor<CalendarEvent[]>;
  /** Load the account's calendars + a window of events (e4 fills). */
  loadCalendars(): Promise<void>;
}

export function createCalendarSlice(_ctx: SliceContext): CalendarSlice {
  const [calendars] = createSignal<Calendar[]>([]);
  const [calendarEvents] = createSignal<CalendarEvent[]>([]);

  return {
    calendars,
    calendarEvents,
    loadCalendars: () => Promise.resolve(),
  };
}
