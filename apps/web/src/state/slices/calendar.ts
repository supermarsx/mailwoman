// Calendar store slice (plan §2.5, §3 e0 → filled by e4). The app-store data
// seam for the calendar module: it builds a `CalendarController` over the shared
// `Client.jmap` transport (the frozen `Calendar/*` / `CalendarEvent/*` surface,
// §2.2) and exposes it to the module. Disjoint file — no `store.ts` collision
// with the other PIM slices (same discipline as V2).
//
// The module renders MOCK-BACKED by default (engine is e8); e10 wires the app
// shell to pass THIS controller (`calendarController()`) into `CalendarApp`,
// swapping the mock for the real engine surface. Until then the frozen accessors
// resolve to empty and `load()` is a no-op-on-error, so mounting is safe.

import { type Accessor } from 'solid-js';
import type { Id } from '../../api/jmap-types.ts';
import type { Calendar, CalendarEvent } from '../../api/pim-types.ts';
import { createCalendarController, type CalendarBackend, type CalendarController } from '../../modules/calendar/controller.ts';
import { CAP_CALENDARS } from '../../api/pim-types.ts';
import type { SliceContext } from './context.ts';

export interface CalendarSlice {
  /** The account's calendars (frozen accessor — proxies the controller). */
  calendars: Accessor<Calendar[]>;
  /** The loaded event masters for the focused window (frozen accessor). */
  calendarEvents: Accessor<CalendarEvent[]>;
  /** The full reactive controller the calendar module renders over (e10 wiring). */
  calendarController: Accessor<CalendarController>;
  /** Load the account's calendars + the focused window of events. */
  loadCalendars(): Promise<void>;
}

export function createCalendarSlice(ctx: SliceContext): CalendarSlice {
  const { client } = ctx;

  // Resolve + cache the calendar account from the session (same pattern as the
  // tasks slice); the controller calls this before every engine round-trip.
  let cachedAccount: Id | null = null;
  async function resolveAccount(): Promise<Id | null> {
    if (cachedAccount !== null) return cachedAccount;
    try {
      const session = await client.session();
      cachedAccount =
        session.primaryAccounts[CAP_CALENDARS] ?? Object.keys(session.accounts)[0] ?? null;
    } catch {
      cachedAccount = null;
    }
    return cachedAccount;
  }

  const backend: CalendarBackend = {
    jmap: (body) => client.jmap(body),
    resolveAccount,
  };

  const controller = createCalendarController(backend);

  return {
    calendars: controller.calendars,
    calendarEvents: controller.masters,
    calendarController: () => controller,
    loadCalendars: () => controller.load(),
  };
}
