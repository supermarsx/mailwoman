// `load()` when two calls overlap (t30-h1, 26.20).
//
// `load()` makes its requests one after another and writes state at the end. A
// write made while the calendar is still loading (an import right after the
// module opens) starts a second `load()`; if the first one answers after it,
// the first must not put its older events back.
//
// The backend is the in-memory mock (the surface of `t16-pim.test.tsx`), with
// the first `CalendarEvent/expand` response held: it is computed when asked, so
// it is the answer from before the write, and delivered only once released.

import { describe, it, expect } from 'vitest';
import type { JmapRequest, JmapResponse } from '../../api/jmap-types.ts';
import { createCalendarController } from './controller.ts';
import { dateToLocal } from './datetime.ts';
import { createMockJmap, createMockStore } from './mock.ts';

describe('calendar controller — overlapping load()', () => {
  it('keeps the events of the load that started last when an earlier one answers after it', async () => {
    const store = createMockStore();
    const inner = createMockJmap(store);

    let release: () => void = () => undefined;
    const released = new Promise<void>((resolve) => {
      release = resolve;
    });
    let reached: () => void = () => undefined;
    const held = new Promise<void>((resolve) => {
      reached = resolve;
    });
    let expands = 0;
    const jmap = async (body: JmapRequest): Promise<JmapResponse> => {
      const res = await inner(body);
      if (body.methodCalls.some(([name]) => name === 'CalendarEvent/expand')) {
        expands += 1;
        if (expands === 1) {
          reached();
          await released;
        }
      }
      return res;
    };

    const c = createCalendarController({ jmap, resolveAccount: () => Promise.resolve('acct-mock') });
    const first = c.load();
    await held;

    // The write, and the load it starts, run to the end while the first load
    // is still waiting for its expansion. The event is inside the open window.
    const start = new Date(c.focusDate());
    start.setHours(10);
    const id = await c.createEvent({ calendarId: 'cal-work', title: 'Added while loading', start: dateToLocal(start) });
    expect(id).not.toBeNull();
    expect(c.instances().some((i) => i.event.id === id)).toBe(true);

    release();
    await first;

    expect(c.instances().some((i) => i.event.id === id)).toBe(true);
    expect(c.masters().some((m) => m.id === id)).toBe(true);
    expect(c.loading()).toBe(false);
    expect(c.error()).toBeNull();
  });
});
