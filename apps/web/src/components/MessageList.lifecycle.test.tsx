// Component-LIFECYCLE spec for the virtualized list (26.20 / t22-e5a).
//
// `virtual.test.ts` covers the windowing MATHS; this file covers what the
// component does with it, which is where the churn lived:
//
//  * a one-row scroll used to tear down and rebuild every mounted row, because
//    `slice()` wrapped each row in a fresh `{row, index}` object and `<For>`
//    keys on reference (measured: 0 of 15 overlapping rows reused);
//  * a sub-row scroll that leaves the window unchanged did the same, because
//    `computeWindow` returns a fresh object and the `win()` memo had no
//    equality comparator — so the churn was per scroll EVENT, not per tick;
//  * the row-ref map was keyed by index and never pruned (Solid does not call a
//    `ref` callback back on dispose), so it grew to one entry per row ever
//    scrolled past — measured at exactly 2000 entries for a 2000-row list.
//
// The prepend case below is a REGRESSION GUARD, not a reproduction: an
// index-keyed map was measured to survive a prepend, because `moveCursor`
// scrolls the target into view first and that re-mount rewrites the entry
// before the focus microtask reads it. Keying by row identity is what keeps
// that true once rows become identity-stable across a list update — at which
// point indices shift under reused rows and no ref re-runs to correct them.
//
// The instruments here are COUNTS — DOM node identity, map size, which element
// holds focus — not timings. jsdom performs no layout and no paint, so nothing
// in this file says anything about frame cost.

import { describe, it, expect, beforeEach } from 'vitest';
import { render, waitFor, fireEvent } from '@solidjs/testing-library';
import { MessageList, rowRefCount } from './MessageList.tsx';
import { makeClient, mkEmail } from './appHarness.tsx';
import { AppContext } from '../state/context.ts';
import { createAppState, type AppState } from '../state/store.ts';
import type { Client } from '../api/client.ts';
import type { Email, EmailGetResponse, JmapResponse } from '../api/jmap-types.ts';

const ROW = 72; // cozy density, the harness default
const CREDS = { jmapUrl: 'x', username: 'me@example.org', password: 'p' };

/** `makeClient` returns the SAME array instance from every `Email/get`, so a
 *  refetch after mutating it would never notify the `messages` signal. Clone
 *  the list so a refetch behaves like a real one (this is how a prepend from
 *  paging will arrive). */
function refetchingClient(box: Email[]): Client {
  const base = makeClient({ emails: box });
  return {
    ...base,
    jmap: async (body): Promise<JmapResponse> => {
      const res = await base.jmap(body);
      return {
        ...res,
        methodResponses: res.methodResponses.map((mr) => {
          if (mr[0] !== 'Email/get') return mr;
          const args = mr[1] as unknown as EmailGetResponse;
          return [mr[0], { ...args, list: [...args.list] }, mr[2]];
        }),
      };
    },
  };
}

/** jsdom has no layout, so `scrollTop` is a no-op property there. Make it a
 *  real writable one: the component both reads it (`onScroll`) and writes it
 *  (`moveCursor` scrolling a row into view). */
function makeScrollable(el: HTMLElement): void {
  Object.defineProperty(el, 'scrollTop', { value: 0, writable: true, configurable: true });
}

function scrollTo(el: HTMLElement, top: number): void {
  el.scrollTop = top;
  fireEvent.scroll(el);
}

/** The currently mounted window as `data-index` -> the row's `<button>`. */
function mountedRows(root: ParentNode): Map<string, Element> {
  const m = new Map<string, Element>();
  for (const el of root.querySelectorAll('.list__row')) m.set(el.getAttribute('data-index') ?? '?', el);
  return m;
}

/** Of the indices present in both windows, how many kept the SAME DOM node.
 *  A reused node means Solid kept the row; a new node means it was rebuilt. */
function nodeReuse(
  before: Map<string, Element>,
  after: Map<string, Element>,
): { overlap: number; reused: number } {
  let overlap = 0;
  let reused = 0;
  for (const [index, el] of before) {
    const now = after.get(index);
    if (now === undefined) continue;
    overlap += 1;
    if (now === el) reused += 1;
  }
  return { overlap, reused };
}

interface Mounted {
  app: AppState;
  container: HTMLElement;
  scroller: HTMLElement;
  box: Email[];
}

async function mountList(n: number, subject = (i: number): string => `Message ${i}`): Promise<Mounted> {
  const box = Array.from({ length: n }, (_, i) => mkEmail(`m${i}`, { subject: subject(i) }));
  const app = createAppState(refetchingClient(box));
  const result = render(() => <AppContext.Provider value={app}>{<MessageList />}</AppContext.Provider>);
  await app.login(CREDS);
  await waitFor(() => expect(app.messages().length).toBe(n));
  const scroller = result.container.querySelector('.list__scroll') as HTMLElement;
  makeScrollable(scroller);
  return { app, container: result.container, scroller, box };
}

describe('MessageList lifecycle (t22-e5a)', () => {
  beforeEach(() => localStorage.clear());

  it('reuses the overlapping rows across a one-row scroll tick', async () => {
    const { container, scroller } = await mountList(200);

    // At the top the window is [0,15); one 72px tick moves it to [0,16), so 15
    // indices survive. Every one of their DOM nodes must survive with them.
    const before = mountedRows(container);
    expect(before.size).toBe(15);

    scrollTo(scroller, ROW);
    const { overlap, reused } = nodeReuse(before, mountedRows(container));

    expect(overlap).toBe(15);
    expect(reused).toBe(15); // was 0 of 15 before the wrapper `.map()` went
  });

  it('rebuilds nothing when a sub-row scroll leaves the window unchanged', async () => {
    const { container, scroller } = await mountList(200);

    // Mid-list, so the window is the full 21 rows (overscan on both sides).
    scrollTo(scroller, 10 * ROW);
    const before = mountedRows(container);
    expect(before.size).toBe(21);

    // 5px is inside one row: `computeWindow` returns an EQUAL window, and the
    // `equals` comparator on `win()` must stop the notification there.
    scrollTo(scroller, 10 * ROW + 5);
    const { overlap, reused } = nodeReuse(before, mountedRows(container));

    expect(overlap).toBe(21);
    expect(reused).toBe(21); // was 0 of 21: churn fired on every scroll event
  });

  it('keeps the row-ref map bounded by the mounted window, not by the list', async () => {
    const N = 2000;
    const { container, scroller } = await mountList(N);

    // Strides of 15 rows are smaller than the ~21-row window, so every index is
    // mounted at some point on the way down.
    const everMounted = new Set<Element>();
    const note = (): void => {
      for (const el of container.querySelectorAll('.list__row')) everMounted.add(el);
    };
    note();
    for (let top = 0; top <= (N - 1) * ROW; top += 15 * ROW) {
      scrollTo(scroller, top);
      note();
    }

    // Disposal control. If rows were never torn down this stays near the window
    // size, and a bounded map would prove nothing. Reaching ~N says every row
    // really was mounted and then disposed.
    expect(everMounted.size).toBeGreaterThan(N * 0.9);

    // The map holds the mounted window and nothing else. Before the fix this
    // reached exactly N (2000), because the `else { delete }` branch was dead:
    // Solid never calls a `ref` callback back on dispose.
    expect(rowRefCount()).toBeGreaterThan(0);
    expect(rowRefCount()).toBeLessThanOrEqual(40);
  });

  it('keeps Home/End focus on the right row after rows are prepended', async () => {
    const { app, scroller, box } = await mountList(100);

    fireEvent.keyDown(scroller, { key: 'End' });
    await waitFor(() => expect(document.activeElement?.getAttribute('data-index')).toBe('99'));

    // What paging does: 50 rows arrive ahead of the list, shifting every
    // existing index by 50. Focus must still land on an in-document node whose
    // content is the row the key names — not a leftover from the old indexing.
    box.unshift(...Array.from({ length: 50 }, (_, i) => mkEmail(`p${i}`, { subject: `Prepended ${i}` })));
    await app.refreshCurrentMailbox();
    await waitFor(() => expect(app.messages().length).toBe(150));

    fireEvent.keyDown(scroller, { key: 'Home' });
    await waitFor(() => {
      const el = document.activeElement as HTMLElement;
      expect(el.getAttribute('data-index')).toBe('0');
      expect(el.isConnected).toBe(true);
      expect(el.textContent).toContain('Prepended 0');
    });

    fireEvent.keyDown(scroller, { key: 'End' });
    await waitFor(() => {
      const el = document.activeElement as HTMLElement;
      expect(el.getAttribute('data-index')).toBe('149');
      expect(el.isConnected).toBe(true);
      expect(el.textContent).toContain('Message 99');
    });
  });
});
