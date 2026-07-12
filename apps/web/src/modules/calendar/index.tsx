// Calendar module placeholder (plan §2.5, §3 e0 → filled by e4). e0 provides
// the mount seam + a placeholder view; e4 builds all calendar views (day /
// 3-day / work-week / week / month / tri-month / schedule / agenda-list / year)
// over `state/slices/calendar.ts` and the frozen `Calendar/*`/`CalendarEvent/*`
// surface, reusing the ribbon / command-palette / virtualized-list / tokens.

import type { JSX } from 'solid-js';

export function CalendarModule(): JSX.Element {
  return (
    <section aria-label="Calendar" data-module="calendar">
      <h1>Calendar</h1>
      <p>The calendar module mounts here (e4).</p>
    </section>
  );
}
