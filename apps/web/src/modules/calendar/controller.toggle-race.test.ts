// Calendar visibility toggles against a slow engine (t33, the t32-h12 guard).
//
// A toggle flips the list on screen and then writes `Calendar/set`. Two things
// went wrong when toggles came faster than the engine answered:
//
//   - every write is pushed back as a change, and the push starts a `load()`;
//     one whose `Calendar/get` answered before the next toggle's write landed
//     put the older flag back on screen (Space twice left the box unticked);
//   - two writes in flight together could land in either order, so the stored
//     flag ended opposite to the one on screen.
//
// The backend is the in-memory mock with `Calendar/set` held at the "network":
// a held write has not reached the store yet, as on a real engine.

import { describe, it, expect } from 'vitest';
import type { JmapRequest, JmapResponse } from '../../api/jmap-types.ts';
import { createCalendarController } from './controller.ts';
import { createMockJmap, createMockStore } from './mock.ts';

function heldWrites() {
  const store = createMockStore();
  const inner = createMockJmap(store);
  const queue: Array<() => void> = [];
  const sent: boolean[] = [];
  const jmap = async (body: JmapRequest): Promise<JmapResponse> => {
    if (body.methodCalls.some(([name]) => name === 'Calendar/set')) {
      const update = (body.methodCalls[0]![1] as { update?: Record<string, { isVisible?: boolean }> }).update ?? {};
      const flag = Object.values(update)[0]?.isVisible;
      if (flag !== undefined) sent.push(flag);
      await new Promise<void>((resolve) => queue.push(resolve));
    }
    return inner(body);
  };
  const c = createCalendarController({ jmap, resolveAccount: () => Promise.resolve('acct-mock') });
  const settle = () => new Promise<void>((resolve) => setTimeout(resolve, 0));
  const releaseNext = async () => {
    queue.shift()?.();
    await settle();
  };
  const stored = () => store.calendars.find((cal) => cal.id === 'cal-work')!.isVisible;
  const shown = () => c.calendars().find((cal) => cal.id === 'cal-work')!.isVisible;
  return { c, queue, sent, settle, releaseNext, stored, shown };
}

describe('calendar controller — toggles faster than the engine', () => {
  it('a load that answers while a toggle is being written keeps the toggled flag on screen', async () => {
    const h = heldWrites();
    await h.c.load();
    expect(h.shown()).toBe(true);

    const toggle = h.c.toggleCalendar('cal-work');
    await h.settle();
    expect(h.shown()).toBe(false);
    // The push of an earlier write starts a load; the engine still holds this one.
    await h.c.load();
    expect(h.shown()).toBe(false);

    await h.releaseNext();
    await toggle;
    await h.settle();
    expect(h.stored()).toBe(false);
    expect(h.shown()).toBe(false);
  });

  it('two quick toggles reach the engine in order and end where they started', async () => {
    const h = heldWrites();
    await h.c.load();

    const first = h.c.toggleCalendar('cal-work');
    await h.settle();
    const second = h.c.toggleCalendar('cal-work');
    await h.settle();
    expect(h.shown()).toBe(true);
    // The second write waits for the first.
    expect(h.queue.length).toBe(1);
    const pushed = h.c.load();

    await h.releaseNext();
    expect(h.stored()).toBe(false);
    expect(h.shown()).toBe(true);
    await h.releaseNext();
    await Promise.all([first, second, pushed]);
    await h.settle();

    expect(h.sent).toEqual([false, true]);
    expect(h.stored()).toBe(true);
    expect(h.shown()).toBe(true);
  });
});
