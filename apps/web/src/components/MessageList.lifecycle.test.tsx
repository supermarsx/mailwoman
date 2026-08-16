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
    // The 2000 rows are load-bearing: the bound is only meaningful against a list
    // far larger than the window, and a leaking implementation would also pass a
    // small one. So the row count stays and the TIMEOUT moves instead — this
    // mounts and scrolls 2000 rows through jsdom, takes ~2.4 s idle, and exceeds
    // vitest's 5 s default whenever the host is busy (it is: four Rust lanes
    // share this machine). Verified not a regression — reverting t22-e4's three
    // source files and removing its tests reproduces the identical failure.
  }, 20_000);

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

// ── t22-e5b: the scroll trigger, and the query total reaching the virtualizer ──
//
// `appHarness`'s `makeClient` ignores `position`/`limit` and sends no `total`, so
// every test above sees `app.total() === null` and one un-paged page. That is the
// right control for "nothing changed when the server does not page", but it
// cannot exercise paging at all — hence the local client below.
//
// The acceptance here is deliberately BEHAVIOURAL. A test that calls
// `app.loadMore()` and asserts rows appear passes against `3a8fefb` alone and
// says nothing about whether any DOM event reaches it; the whole failure mode
// this lane exists to close is a complete, tested paging API that no gesture
// invokes.

interface PagedQuery {
  position: number;
  limit: number;
  calculateTotal: boolean;
}

interface PagedMount {
  app: AppState;
  container: HTMLElement;
  scroller: HTMLElement;
  /** Every `Email/query` this mount issued, in order. */
  queries: PagedQuery[];
}

/** A client that really pages: honours `position`/`limit` and answers
 *  `calculateTotal` with the whole corpus size. */
function pagingClient(corpus: Email[], queries: PagedQuery[], alwaysFirstPage = false): Client {
  const base = makeClient({ emails: corpus });
  return {
    ...base,
    jmap: async (body): Promise<JmapResponse> => {
      const q = body.methodCalls.find((c) => c[0] === 'Email/query');
      if (q === undefined) return base.jmap(body);
      const args = q[1] as { position?: number; limit?: number; calculateTotal?: boolean };
      const position = args.position ?? 0;
      const limit = args.limit ?? 50;
      const calculateTotal = args.calculateTotal === true;
      queries.push({ position, limit, calculateTotal });
      const page = alwaysFirstPage ? corpus.slice(0, limit) : corpus.slice(position, position + limit);
      return {
        methodResponses: [
          [
            'Email/query',
            {
              accountId: 'acct1',
              queryState: 'q0',
              ids: page.map((e) => e.id),
              position,
              ...(calculateTotal ? { total: corpus.length } : {}),
            },
            'q',
          ],
          ['Email/get', { accountId: 'acct1', state: 's', list: [...page], notFound: [] }, 'g'],
        ],
        sessionState: 's',
      };
    },
  };
}

/** Mount over a folder of `size` messages named `Message 1 … Message size`. */
async function mountPaged(size: number, alwaysFirstPage = false): Promise<PagedMount> {
  const corpus = Array.from({ length: size }, (_, i) => mkEmail(`m${i + 1}`, { subject: `Message ${i + 1}` }));
  const queries: PagedQuery[] = [];
  const app = createAppState(pagingClient(corpus, queries, alwaysFirstPage));
  const result = render(() => <AppContext.Provider value={app}>{<MessageList />}</AppContext.Provider>);
  await app.login(CREDS);
  await waitFor(() => expect(app.messages().length).toBe(Math.min(50, size)));
  const scroller = result.container.querySelector('.list__scroll') as HTMLElement;
  makeScrollable(scroller);
  return { app, container: result.container, scroller, queries };
}

/** The subjects currently rendered as real rows. */
function renderedSubjects(root: ParentNode): string[] {
  return Array.from(root.querySelectorAll('.list__subject')).map((el) => el.textContent ?? '');
}

/** `Email/query` calls that asked for anything past the first page. */
function pageRequests(queries: PagedQuery[]): PagedQuery[] {
  return queries.filter((q) => q.position > 0);
}

describe('MessageList paging trigger (t22-e5b)', () => {
  beforeEach(() => localStorage.clear());

  it('a SCROLL EVENT reaches message 51 in a 20 000-message mailbox', async () => {
    const { app, container, scroller } = await mountPaged(20_000);

    // The state layer could reach message 51 from `3a8fefb`; the UI could not.
    expect(app.messages()).toHaveLength(50);
    expect(renderedSubjects(container)).not.toContain('Message 51');

    // The same event the browser fires. Nothing test-only is called.
    scrollTo(scroller, 45 * ROW);

    await waitFor(() => expect(app.messages().length).toBe(100));
    await waitFor(() => expect(renderedSubjects(container)).toContain('Message 51'));
  }, 20_000);

  it('does not page until something scrolls — the trigger is the gesture', async () => {
    // Control for the test above: without it, "scrolling loaded page 2" would
    // also hold for an implementation that pages on mount, on a timer, or from
    // an effect, none of which is a scroll trigger.
    const { app, queries } = await mountPaged(20_000);
    await new Promise((resolve) => setTimeout(resolve, 30));
    expect(app.messages()).toHaveLength(50);
    expect(pageRequests(queries)).toHaveLength(0);
  }, 20_000);

  it('issues ONE request per gesture, not one per scroll event', async () => {
    const { queries, scroller } = await mountPaged(20_000);

    // A real gesture delivers a burst. Ungated, each event calls loadMore.
    for (let i = 0; i < 12; i += 1) scrollTo(scroller, 45 * ROW + i);

    expect(pageRequests(queries)).toHaveLength(1);
  }, 20_000);

  it('asks for the total once and then stops asking', async () => {
    const { app, scroller, queries } = await mountPaged(20_000);
    scrollTo(scroller, 45 * ROW);
    await waitFor(() => expect(app.messages().length).toBe(100));
    scrollTo(scroller, 95 * ROW);
    await waitFor(() => expect(app.messages().length).toBe(150));

    expect(queries.map((q) => q.calculateTotal)).toEqual([true, false, false]);
    expect(queries.map((q) => q.position)).toEqual([0, 50, 100]);
  }, 20_000);

  it('stops requesting once the folder is exhausted', async () => {
    const { app, scroller, queries } = await mountPaged(120);
    scrollTo(scroller, 45 * ROW);
    await waitFor(() => expect(app.messages().length).toBe(100));
    scrollTo(scroller, 95 * ROW);
    await waitFor(() => expect(app.messages().length).toBe(120));
    expect(app.hasMore()).toBe(false);

    const settled = queries.length;
    for (let i = 0; i < 5; i += 1) scrollTo(scroller, 110 * ROW + i);
    expect(queries).toHaveLength(settled);
  }, 20_000);

  it('does not re-request a page that came back holding nothing new', async () => {
    // A query whose window shifted under the reader answers a continuation with
    // rows that are all already loaded. Nothing is appended, so the loaded extent
    // does not move — and `hasMore()` stays true, because the server said there is
    // more. An unguarded trigger therefore re-issues the identical request on
    // every subsequent scroll event, forever.
    const { app, scroller, queries } = await mountPaged(20_000, true);

    scrollTo(scroller, 45 * ROW);
    await waitFor(() => expect(pageRequests(queries)).toHaveLength(1));
    expect(app.messages()).toHaveLength(50); // deduplicated away, as designed
    expect(app.hasMore()).toBe(true); // and the query still claims more exists

    for (let i = 0; i < 10; i += 1) scrollTo(scroller, 46 * ROW + i * ROW);
    await new Promise((resolve) => setTimeout(resolve, 20));

    // Asking again cannot make a page that added nothing add something.
    expect(pageRequests(queries)).toHaveLength(1);
  }, 20_000);

  it('a mailbox switch clears the no-progress guard', async () => {
    // The guard remembers the extent it last requested at. Two folders whose
    // first pages happen to be the same size would otherwise let one folder's
    // extent suppress the other's second page — permanently, since nothing else
    // moves it.
    const { app, container, scroller, queries } = await mountPaged(20_000);
    scrollTo(scroller, 45 * ROW);
    await waitFor(() => expect(app.messages().length).toBe(100));

    await app.selectMailbox('archive');
    await waitFor(() => expect(app.messages().length).toBe(50));
    const before = pageRequests(queries).length;

    // `listLoading` swaps the whole scroller for the loading fallback, so the
    // node captured at mount is detached by now — events on it reach nothing.
    // Re-acquiring it is also what a real user gets: a replaced list starts at
    // scrollTop 0, which is why nothing pages until they scroll again.
    const live = container.querySelector('.list__scroll') as HTMLElement;
    expect(live).not.toBe(scroller);
    makeScrollable(live);

    scrollTo(live, 45 * ROW);
    await waitFor(() => expect(pageRequests(queries).length).toBe(before + 1));
  }, 20_000);
});

describe('MessageList describes the folder, not the page (t22-e5b, L4)', () => {
  beforeEach(() => localStorage.clear());

  it('sizes the scrollbar and aria-setsize from the QUERY total', async () => {
    const { app, container } = await mountPaged(20_000);

    expect(app.total()).toBe(20_000);
    expect(app.loadedRange()).toEqual({ start: 0, end: 50 });

    const items = container.querySelector('.list__items') as HTMLElement;
    // Was 50 * 72 = 3600px — a 20 000-message folder with a 3 600px scrollbar.
    expect(items.style.height).toBe(`${20_000 * ROW}px`);

    const first = container.querySelector('.list__slot') as HTMLElement;
    expect(first.getAttribute('aria-setsize')).toBe('20000'); // was "50"
    expect(first.getAttribute('aria-posinset')).toBe('1');
  }, 20_000);

  it('marks slots past the loaded page as pending, not as empty messages', async () => {
    const { container, scroller } = await mountPaged(20_000);

    // Drag far past anything loaded — the scrollbar now permits this.
    scrollTo(scroller, 5_000 * ROW);

    const pending = container.querySelectorAll('.list__slot[aria-busy="true"]');
    expect(pending.length).toBeGreaterThan(0);
    // Nothing is loaded out here, so no row may claim to be a message.
    expect(container.querySelectorAll('.list__row')).toHaveLength(0);
    expect(pending[0]!.getAttribute('aria-setsize')).toBe('20000');
    // Same height as a real row, so an arriving page does not shift the scrollbar.
    expect((pending[0] as HTMLElement).style.height).toBe(`${ROW}px`);
  }, 20_000);

  it('never renders a loaded row and a pending slot at the same position', async () => {
    // An off-by-one in the split puts a placeholder ON TOP of a real row at the
    // same offset, which reads as a flicker rather than as a bug.
    const { container, scroller } = await mountPaged(20_000);
    scrollTo(scroller, 40 * ROW);

    const seen = new Map<string, number>();
    for (const el of container.querySelectorAll('.list__slot')) {
      const pos = el.getAttribute('aria-posinset') ?? '?';
      seen.set(pos, (seen.get(pos) ?? 0) + 1);
    }
    expect(seen.size).toBeGreaterThan(0);
    for (const [pos, count] of seen) expect(`${pos}:${count}`).toBe(`${pos}:1`);
  }, 20_000);

  it('falls back to the loaded count when the server reports no total', async () => {
    // The un-paged path every other spec in this file uses: `makeClient` sends no
    // `total`, and an unknown total must never be guessed at.
    const { app, container } = await mountList(200);

    expect(app.total()).toBeNull();
    const items = container.querySelector('.list__items') as HTMLElement;
    expect(items.style.height).toBe(`${200 * ROW}px`);
    expect(container.querySelectorAll('[aria-busy="true"]')).toHaveLength(0);
    const first = container.querySelector('.list__slot') as HTMLElement;
    expect(first.getAttribute('aria-setsize')).toBe('200');
  }, 20_000);
});
