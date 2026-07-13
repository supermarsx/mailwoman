// Engine-backed Calendar host (plan §3 e10 — the mock→engine SWAP for Calendar).
//
// Calendar is the one PIM module whose standalone `mount()` was self-contained
// and MOCK-backed (`CalendarModule` → `makeMockController`). Here it renders over
// the app store's ENGINE-backed controller (`app.calendarController()`, which runs
// on `client.jmap` — the real `Calendar/*` / `CalendarEvent/*` surface). Default
// export so `shell/modules.ts` can `lazy(() => import('./mounts.tsx'))` it into its
// own chunk, off the mail critical path (plan risk #10). Tasks/Notes/Contacts need
// no wrapper — their slices already speak `client.jmap`, so the registry lazy-
// imports their module components directly.

import type { JSX } from 'solid-js';
import { useApp } from '../state/context.ts';
import { CalendarApp } from '../modules/calendar/index.tsx';

export default function CalendarMount(): JSX.Element {
  const app = useApp();
  return <CalendarApp controller={app.calendarController()} />;
}
