// Client paging + the stale-response race (t22-e4).
//
// A separate file from `mail.test.ts` so the pre-paging behaviour there stays a
// control: every assertion in it must keep passing unchanged, and none of it is
// edited to accommodate paging.
//
// The interleaving harness below is the one the render/scale verifiers used to
// FIND the bug, reproduced here as a regression: two per-mailbox responses held
// open, released out of order. Asserting a generation guard only against
// responses that arrive in order proves nothing at all — in-order responses pass
// on master too — so `stale_response_control` is the paired negative control
// showing the harness can observe an overwrite when one is legitimate.

import { describe, it, expect, vi } from 'vitest';
import { createRoot } from 'solid-js';
import { createMailSlice, PAGE_SIZE, type MailSlice } from './mail.ts';
import type { SliceContext } from './context.ts';
import type { Client, Me } from '../../api/client.ts';
import {
  CAP_MAIL,
  type Email,
  type JmapRequest,
  type JmapResponse,
  type JmapSession,
  type Mailbox,
} from '../../api/jmap-types.ts';

// ── fixtures ────────────────────────────────────────────────────────────────

const MAILBOXES: Mailbox[] = [
  { id: 'inbox', name: 'Inbox', parentId: null, role: 'inbox', sortOrder: 0, totalEmails: 0, unreadEmails: 0 },
  { id: 'archive', name: 'Archive', parentId: null, role: 'archive', sortOrder: 1, totalEmails: 0, unreadEmails: 0 },
  { id: 'trash', name: 'Trash', parentId: null, role: 'trash', sortOrder: 2, totalEmails: 0, unreadEmails: 0 },
];

const SESSION: JmapSession = {
  capabilities: {},
  accounts: { acct1: { name: 'T', isPersonal: true, isReadOnly: false, accountCapabilities: {} } },
  primaryAccounts: { [CAP_MAIL]: 'acct1' },
  username: 'me@example.org',
  apiUrl: '/jmap/api',
  downloadUrl: '/d',
  uploadUrl: '/u',
  eventSourceUrl: '/e',
  state: 's0',
};

function email(id: string, subject: string): Email {
  return {
    id,
    mailboxIds: { inbox: true },
    from: [{ name: null, email: `${id}@example.org` }],
    to: [{ name: null, email: 'me@example.org' }],
    subject,
    receivedAt: '2026-01-01T00:00:00Z',
    preview: `preview ${id}`,
    keywords: {},
  };
}

/** `n` messages in `box`, named `box-1 … box-n` in query order. */
function corpus(box: string, n: number): Email[] {
  return Array.from({ length: n }, (_, i) => email(`${box}-${i + 1}`, `${box} message ${i + 1}`));
}

// ── a JMAP server that actually honours position/limit/calculateTotal ───────

interface ListCall {
  /** The mailbox queried, or `'search'` for a text query. */
  readonly key: string;
  readonly position: number;
  readonly limit: number;
  readonly calculateTotal: boolean;
  /** The abort signal the slice attached to this request, if any. */
  readonly signal: AbortSignal | undefined;
  /** Deliver this call's response (held keys only). */
  readonly release: () => void;
}

interface ServerOptions {
  /** Answer a `calculateTotal` request with NO total, as a truncated search
   *  index legitimately must (t22 V9). */
  readonly omitTotal?: boolean;
  /** Every page returns the FIRST page's rows, modelling a query whose window
   *  shifted under the reader so a continuation fully overlaps what is held. */
  readonly repeatFirstPage?: boolean;
}

function makeServer(folders: Record<string, Email[]>, opts: ServerOptions = {}) {
  const calls: ListCall[] = [];
  const held = new Set<string>();

  const jmap = vi.fn((body: JmapRequest, callOpts?: { signal?: AbortSignal }): Promise<JmapResponse> => {
    const names = body.methodCalls.map((c) => c[0]);
    if (names.includes('Mailbox/get')) {
      return Promise.resolve({
        methodResponses: [['Mailbox/get', { accountId: 'acct1', state: 's', list: MAILBOXES, notFound: [] }, 'c0']],
        sessionState: 's',
      });
    }
    const query = body.methodCalls.find((c) => c[0] === 'Email/query');
    if (query === undefined) {
      return Promise.resolve({
        methodResponses: body.methodCalls.map((c) => [c[0], {}, c[2]] as JmapResponse['methodResponses'][number]),
        sessionState: 's',
      });
    }
    const args = query[1] as {
      filter?: { inMailbox?: string };
      position?: number;
      limit?: number;
      calculateTotal?: boolean;
    };
    const key = args.filter?.inMailbox ?? 'search';
    const position = args.position ?? 0;
    const limit = args.limit ?? 0;
    const calculateTotal = args.calculateTotal === true;
    const all = folders[key] ?? [];
    const page = opts.repeatFirstPage === true ? all.slice(0, limit) : all.slice(position, position + limit);
    const response: JmapResponse = {
      methodResponses: [
        [
          'Email/query',
          {
            accountId: 'acct1',
            queryState: 'q0',
            ids: page.map((e) => e.id),
            position,
            ...(calculateTotal && opts.omitTotal !== true ? { total: all.length } : {}),
          },
          'q',
        ],
        ['Email/get', { accountId: 'acct1', state: 's', list: page, notFound: [] }, 'g'],
      ],
      sessionState: 's',
    };

    let release = (): void => undefined;
    const promise = held.has(key)
      ? new Promise<JmapResponse>((resolve) => {
          release = () => resolve(response);
        })
      : Promise.resolve(response);
    calls.push({ key, position, limit, calculateTotal, signal: callOpts?.signal, release });
    return promise;
  });

  const client: Client = {
    login: vi.fn(async (): Promise<Me> => ({ username: 'me@example.org', accountId: 'acct1' })),
    logout: vi.fn(async () => undefined),
    me: vi.fn(async (): Promise<Me> => ({ username: 'me@example.org', accountId: 'acct1' })),
    session: vi.fn(async () => SESSION),
    // Takes the real two-parameter shape, so the double observes the abort
    // signal the slice attaches without any cast.
    jmap,
    sanitize: vi.fn(async (h: string) => h),
    onNetwork: vi.fn(() => () => undefined),
  };

  return {
    client,
    calls,
    /** Hold every subsequent response for `key` until `release()` is called. */
    hold: (key: string): void => void held.add(key),
    /** Release every held response for `key`, oldest first. */
    releaseAll: (key: string): void => {
      for (const c of calls.filter((c) => c.key === key)) c.release();
    },
    listCalls: (key: string): ListCall[] => calls.filter((c) => c.key === key),
  };
}

type Server = ReturnType<typeof makeServer>;

/** Let queued microtasks (and any resolved fetch continuations) run. */
async function flush(): Promise<void> {
  await new Promise((resolve) => setTimeout(resolve, 0));
}

async function withServer(
  server: Server,
  run: (mail: MailSlice, toast: ReturnType<typeof vi.fn>) => Promise<void>,
): Promise<void> {
  const toast = vi.fn();
  const ctx: SliceContext = { client: server.client, showToast: toast };
  await createRoot(async (dispose) => {
    const mail = createMailSlice(ctx);
    await mail.login({ jmapUrl: 'x', username: 'me@example.org', password: 'p' });
    await run(mail, toast);
    dispose();
  });
}

// ── message 51 ──────────────────────────────────────────────────────────────

describe('paging — reaching past the first page', () => {
  it('reaches message 51 of a 120-message folder, which was unreachable at any scroll depth', async () => {
    const server = makeServer({ inbox: corpus('inbox', 120) });
    await withServer(server, async (mail) => {
      // Page 1: exactly what master delivered, and all it could ever deliver.
      expect(mail.messages()).toHaveLength(PAGE_SIZE);
      expect(mail.messages().map((m) => m.id)).not.toContain('inbox-51');
      expect(mail.hasMore()).toBe(true);

      await mail.loadMore();

      expect(mail.messages()).toHaveLength(100);
      const fiftyFirst = mail.messages()[50];
      expect(fiftyFirst?.id).toBe('inbox-51');
      // Assert on CONTENT, not a count: a count passes for a page of anything.
      expect(fiftyFirst?.subject).toBe('inbox message 51');
      expect(mail.loadedRange()).toEqual({ start: 0, end: 100 });

      await mail.loadMore();
      expect(mail.messages()).toHaveLength(120);
      expect(mail.messages()[119]?.subject).toBe('inbox message 120');
      expect(mail.hasMore()).toBe(false);
    });
  });

  it('asks for each page by position, and asks for the total exactly once', async () => {
    const server = makeServer({ inbox: corpus('inbox', 120) });
    await withServer(server, async (mail) => {
      await mail.loadMore();
      await mail.loadMore();
      const calls = server.listCalls('inbox');
      expect(calls.map((c) => c.position)).toEqual([0, 50, 100]);
      expect(calls.map((c) => c.limit)).toEqual([50, 50, 50]);
      // A COUNT(*) over the folder per page is the cost `calculateTotal`-on-
      // request exists to avoid; the total cannot change while paging one query.
      expect(calls.map((c) => c.calculateTotal)).toEqual([true, false, false]);
    });
  });

  it('an exhausted query makes loadMore a no-op — no request, and the same array', async () => {
    const server = makeServer({ inbox: corpus('inbox', 20) });
    await withServer(server, async (mail) => {
      expect(mail.hasMore()).toBe(false);
      const before = mail.messages();
      await mail.loadMore();
      expect(server.listCalls('inbox')).toHaveLength(1);
      // Reference identity, not deep equality: a fresh-but-equal array notifies
      // every downstream memo and rebuilds every mounted row, which is the churn
      // `57046c7` removed from the scroll path. Append must not put it back.
      expect(mail.messages()).toBe(before);
    });
  });

  it('a fully overlapping page adds nothing and keeps the SAME array', async () => {
    const server = makeServer({ inbox: corpus('inbox', 120) }, { repeatFirstPage: true });
    await withServer(server, async (mail) => {
      const before = mail.messages();
      await mail.loadMore();
      expect(server.listCalls('inbox')).toHaveLength(2); // the request was made
      expect(mail.messages()).toHaveLength(PAGE_SIZE); // and deduplicated away
      expect(mail.messages()).toBe(before);
    });
  });

  it('a page that adds nothing ENDS the query, however full it was', async () => {
    // The page is PAGE_SIZE long, so "short page means done" does not catch it —
    // every id in it is simply already loaded, because the query's window shifted
    // under the reader. The loaded extent therefore does not move, and the next
    // request would be byte-identical to the one that just returned.
    //
    // Left un-ended this is an unbounded request loop: an append-on-scroll caller
    // re-issues that identical request on every scroll event, and the only symptom
    // a user reports is that the list feels slow. Measured at 8 extra seconds in
    // one component spec before the query was ended here.
    const server = makeServer({ inbox: corpus('inbox', 120) }, { repeatFirstPage: true });
    await withServer(server, async (mail) => {
      expect(mail.hasMore()).toBe(true);
      await mail.loadMore();

      expect(mail.hasMore()).toBe(false);
      // And the guard holds: a further call issues no request at all.
      const settled = server.listCalls('inbox').length;
      await mail.loadMore();
      expect(server.listCalls('inbox')).toHaveLength(settled);
    });
  });

  it('a short page still ends the query, and a full one that DOES add rows does not', async () => {
    // The control for the pair above: ending on "added nothing" must not end a
    // query that is merely mid-way through.
    const server = makeServer({ inbox: corpus('inbox', 120) });
    await withServer(server, async (mail) => {
      await mail.loadMore(); // full page, 50 new rows
      expect(mail.hasMore()).toBe(true);
      await mail.loadMore(); // 20 rows — short, so done
      expect(mail.messages()).toHaveLength(120);
      expect(mail.hasMore()).toBe(false);
    });
  });
});

// ── total ───────────────────────────────────────────────────────────────────

describe('paging — total describes the query, not the page', () => {
  it('reports the folder size while one page is loaded', async () => {
    const server = makeServer({ inbox: corpus('inbox', 1000) });
    await withServer(server, async (mail) => {
      expect(mail.messages()).toHaveLength(50);
      expect(mail.total()).toBe(1000);
      expect(mail.loadedRange()).toEqual({ start: 0, end: 50 });

      await mail.loadMore();
      expect(mail.total()).toBe(1000); // unchanged, and not re-asked for
      expect(mail.loadedRange()).toEqual({ start: 0, end: 100 });
      expect(mail.hasMore()).toBe(true);
    });
  });

  it('carries an absent total through as unknown rather than as the page length', async () => {
    // A search over a truncated index cannot produce an honest count, so it
    // sends none (t22 V9). Substituting `messages().length` here is exactly the
    // defect that made a 20 000-message folder announce "1 of 50".
    const server = makeServer({ inbox: corpus('inbox', 120) }, { omitTotal: true });
    await withServer(server, async (mail) => {
      expect(mail.total()).toBeNull();
      expect(mail.total()).not.toBe(mail.messages().length);
      // Unknown must not mean "done": paging continues until a page comes short.
      expect(mail.hasMore()).toBe(true);
      await mail.loadMore();
      expect(mail.messages()).toHaveLength(100);
      await mail.loadMore();
      expect(mail.messages()).toHaveLength(120);
      expect(mail.hasMore()).toBe(false);
    });
  });

  it('total tracks the query when the query changes', async () => {
    const server = makeServer({ inbox: corpus('inbox', 1000), archive: corpus('archive', 7) });
    await withServer(server, async (mail) => {
      expect(mail.total()).toBe(1000);
      await mail.selectMailbox('archive');
      expect(mail.total()).toBe(7);
      expect(mail.loadedRange()).toEqual({ start: 0, end: 7 });
    });
  });
});

// ── the stale-response race (L5) ────────────────────────────────────────────

describe('paging — superseded responses cannot write', () => {
  it('releases trash then archive out of order and the NEWER selection wins', async () => {
    const server = makeServer({
      inbox: corpus('inbox', 1),
      archive: corpus('archive', 3),
      trash: corpus('trash', 2),
    });
    await withServer(server, async (mail) => {
      server.hold('archive');
      server.hold('trash');

      const archiveDone = mail.selectMailbox('archive');
      const trashDone = mail.selectMailbox('trash');

      // Selection is intent and is recorded in call order, not fetch order.
      expect(mail.selectedMailboxId()).toBe('trash');
      expect(mail.listLoading()).toBe(true);

      // The newer response lands first…
      server.releaseAll('trash');
      await flush();
      expect(mail.messages().map((m) => m.id)).toEqual(['trash-1', 'trash-2']);
      expect(mail.listLoading()).toBe(false);

      // …and the OLDER one lands after it. On master this overwrote the list
      // with archive's rows under a trash selection, spinner already cleared.
      server.releaseAll('archive');
      await Promise.all([archiveDone, trashDone]);

      expect(mail.selectedMailboxId()).toBe('trash');
      expect(mail.messages().map((m) => m.id)).toEqual(['trash-1', 'trash-2']);
      expect(mail.listLoading()).toBe(false);
      expect(mail.total()).toBe(2);
      expect(mail.loadedRange()).toEqual({ start: 0, end: 2 });
    });
  });

  it('stale_response_control — the harness DOES observe a write when the payload is current', async () => {
    // Without this, "archive's rows never appeared" is unfalsifiable: it would
    // also hold if the harness could not deliver archive's rows at all.
    const server = makeServer({ inbox: corpus('inbox', 1), archive: corpus('archive', 3) });
    await withServer(server, async (mail) => {
      server.hold('archive');
      const done = mail.selectMailbox('archive');
      server.releaseAll('archive');
      await done;
      expect(mail.messages().map((m) => m.id)).toEqual(['archive-1', 'archive-2', 'archive-3']);
    });
  });

  it('wins in the other release order too — the guard is not an ordering accident', async () => {
    const server = makeServer({
      inbox: corpus('inbox', 1),
      archive: corpus('archive', 3),
      trash: corpus('trash', 2),
    });
    await withServer(server, async (mail) => {
      server.hold('archive');
      server.hold('trash');
      const archiveDone = mail.selectMailbox('archive');
      const trashDone = mail.selectMailbox('trash');
      server.releaseAll('archive'); // superseded payload first this time
      await flush();
      expect(mail.messages().map((m) => m.id)).not.toContain('archive-1');
      server.releaseAll('trash');
      await Promise.all([archiveDone, trashDone]);
      expect(mail.messages().map((m) => m.id)).toEqual(['trash-1', 'trash-2']);
    });
  });

  it('aborts the superseded request and leaves the current one alive', async () => {
    const server = makeServer({
      inbox: corpus('inbox', 1),
      archive: corpus('archive', 3),
      trash: corpus('trash', 2),
    });
    await withServer(server, async (mail) => {
      server.hold('archive');
      server.hold('trash');
      const archiveDone = mail.selectMailbox('archive');
      const trashDone = mail.selectMailbox('trash');

      const archiveCall = server.listCalls('archive')[0];
      const trashCall = server.listCalls('trash')[0];
      expect(archiveCall?.signal).toBeDefined();
      expect(archiveCall?.signal?.aborted).toBe(true);
      expect(trashCall?.signal?.aborted).toBe(false);

      server.releaseAll('trash');
      server.releaseAll('archive');
      await Promise.all([archiveDone, trashDone]);
    });
  });

  it('a superseded FAILURE is not surfaced as the current query erroring', async () => {
    const server = makeServer({ inbox: corpus('inbox', 1), archive: corpus('archive', 2) });
    // Make the archive fetch reject; the trash selection that supersedes it must
    // still resolve normally and own the outcome.
    const failing = vi.fn((body: JmapRequest, o?: { signal?: AbortSignal }) => {
      const q = body.methodCalls.find((c) => c[0] === 'Email/query');
      const key = (q?.[1] as { filter?: { inMailbox?: string } } | undefined)?.filter?.inMailbox;
      if (key === 'archive') return Promise.reject(new Error('archive fetch failed'));
      return server.client.jmap(body, o);
    });
    const client: Client = { ...server.client, jmap: failing };
    const toast = vi.fn();
    await createRoot(async (dispose) => {
      const mail = createMailSlice({ client, showToast: toast });
      await mail.login({ jmapUrl: 'x', username: 'me@example.org', password: 'p' });
      const archiveDone = mail.selectMailbox('archive').catch(() => 'threw');
      const trashDone = mail.selectMailbox('inbox');
      await expect(Promise.all([archiveDone, trashDone])).resolves.toBeDefined();
      expect(await archiveDone).toBeUndefined(); // swallowed, not rethrown
      expect(mail.listLoading()).toBe(false);
      dispose();
    });
  });
});

// ── append does not blank the list ──────────────────────────────────────────

describe('paging — appending is not loading', () => {
  it('an in-flight append leaves listLoading false so the rows stay on screen', async () => {
    const server = makeServer({ inbox: corpus('inbox', 120) });
    await withServer(server, async (mail) => {
      server.hold('inbox');
      const more = mail.loadMore();
      // `MessageList` swaps the whole list for a spinner on `listLoading`.
      expect(mail.listLoading()).toBe(false);
      expect(mail.loadingMore()).toBe(true);
      expect(mail.messages()).toHaveLength(50);

      server.releaseAll('inbox');
      await more;
      expect(mail.loadingMore()).toBe(false);
      expect(mail.messages()).toHaveLength(100);
    });
  });

  it('a mailbox switch during an append discards the append and clears its flag', async () => {
    const server = makeServer({ inbox: corpus('inbox', 120), archive: corpus('archive', 4) });
    await withServer(server, async (mail) => {
      server.hold('inbox');
      const more = mail.loadMore();
      const appendCall = server.listCalls('inbox')[1];

      const switched = mail.selectMailbox('archive');
      expect(appendCall?.signal?.aborted).toBe(true);
      expect(mail.loadingMore()).toBe(false);

      server.releaseAll('inbox');
      await Promise.all([more, switched]);

      expect(mail.messages().map((m) => m.id)).toEqual(['archive-1', 'archive-2', 'archive-3', 'archive-4']);
      expect(mail.loadingMore()).toBe(false);
      expect(mail.listLoading()).toBe(false);
    });
  });

  it('refuses to append while a replace is in flight', async () => {
    const server = makeServer({ inbox: corpus('inbox', 120), archive: corpus('archive', 200) });
    await withServer(server, async (mail) => {
      server.hold('archive');
      const switched = mail.selectMailbox('archive');
      await mail.loadMore();
      // No second archive request: the replace owns the spinner, and superseding
      // it with an append would abandon a spinner nothing else clears.
      expect(server.listCalls('archive')).toHaveLength(1);
      expect(mail.listLoading()).toBe(true);
      server.releaseAll('archive');
      await switched;
      expect(mail.listLoading()).toBe(false);
    });
  });

  it('a failed page is reported and leaves the query retryable', async () => {
    const server = makeServer({ inbox: corpus('inbox', 120) });
    let fail = false;
    const wrapped = vi.fn((body: JmapRequest, o?: { signal?: AbortSignal }) => {
      const q = body.methodCalls.find((c) => c[0] === 'Email/query');
      if (fail && q !== undefined) return Promise.reject(new Error('page failed'));
      return server.client.jmap(body, o);
    });
    const client: Client = { ...server.client, jmap: wrapped };
    const toast = vi.fn();
    await createRoot(async (dispose) => {
      const mail = createMailSlice({ client, showToast: toast });
      await mail.login({ jmapUrl: 'x', username: 'me@example.org', password: 'p' });
      fail = true;
      // Must not throw: this runs from a scroll handler, where a rejection is an
      // unhandled promise rather than a visible error.
      await expect(mail.loadMore()).resolves.toBeUndefined();
      expect(toast).toHaveBeenCalledWith('error', 'Could not load more messages');
      expect(mail.loadingMore()).toBe(false);
      expect(mail.hasMore()).toBe(true);
      fail = false;
      await mail.loadMore();
      expect(mail.messages()).toHaveLength(100);
      dispose();
    });
  });
});

// ── refresh keeps the loaded extent ─────────────────────────────────────────

describe('paging — in-place refresh', () => {
  it('renews the whole loaded extent instead of collapsing to page 1', async () => {
    const server = makeServer({ inbox: corpus('inbox', 120) });
    await withServer(server, async (mail) => {
      await mail.loadMore();
      expect(mail.messages()).toHaveLength(100);

      await mail.refreshCurrentMailbox();

      const last = server.listCalls('inbox').at(-1);
      expect(last?.position).toBe(0);
      expect(last?.limit).toBe(100); // not 50 — a push tick must not un-scroll the reader
      expect(mail.messages()).toHaveLength(100);
      expect(mail.messages()[50]?.id).toBe('inbox-51');
    });
  });

  it('declines rather than refetching a very deep extent', async () => {
    const server = makeServer({ inbox: corpus('inbox', 700) });
    await withServer(server, async (mail) => {
      for (let i = 0; i < 11; i += 1) await mail.loadMore();
      expect(mail.messages()).toHaveLength(600);
      const before = server.listCalls('inbox').length;

      await mail.refreshCurrentMailbox();

      // A 600-row Email/get on every push tick is the cost this tag removes.
      expect(server.listCalls('inbox')).toHaveLength(before);
      expect(mail.messages()).toHaveLength(600);
    });
  });
});

// ── search pages too ────────────────────────────────────────────────────────

describe('paging — search', () => {
  it('pages search results and reports the search total', async () => {
    const server = makeServer({ search: corpus('hit', 130) });
    await withServer(server, async (mail) => {
      await mail.searchMessages('needle');
      expect(mail.searchActive()).toBe(true);
      expect(mail.messages()).toHaveLength(50);
      expect(mail.total()).toBe(130);

      await mail.loadMore();
      expect(mail.messages()).toHaveLength(100);
      expect(mail.messages()[50]?.subject).toBe('hit message 51');
      expect(server.listCalls('search').map((c) => c.position)).toEqual([0, 50]);
    });
  });

  it('a search supersedes an in-flight mailbox load', async () => {
    const server = makeServer({ inbox: corpus('inbox', 3), archive: corpus('archive', 3), search: corpus('hit', 2) });
    await withServer(server, async (mail) => {
      server.hold('archive');
      const switched = mail.selectMailbox('archive');
      const searched = mail.searchMessages('needle');
      server.releaseAll('archive');
      await Promise.all([switched, searched]);
      expect(mail.searchActive()).toBe(true);
      expect(mail.messages().map((m) => m.id)).toEqual(['hit-1', 'hit-2']);
    });
  });

  it('clearing search returns to the mailbox and resets the cursor', async () => {
    const server = makeServer({ inbox: corpus('inbox', 120), search: corpus('hit', 130) });
    await withServer(server, async (mail) => {
      await mail.loadMore();
      expect(mail.loadedRange().end).toBe(100);
      await mail.searchMessages('needle');
      expect(mail.loadedRange().end).toBe(50);
      await mail.clearSearch();
      expect(mail.searchActive()).toBe(false);
      expect(mail.total()).toBe(120);
      expect(mail.loadedRange()).toEqual({ start: 0, end: 50 });
    });
  });
});

// ── offline ─────────────────────────────────────────────────────────────────

describe('paging — offline search', () => {
  it('is not a server query, so it reports its own size and pages no further', async () => {
    const server = makeServer({ inbox: corpus('inbox', 120) });
    const cached = corpus('cached', 3);
    const toast = vi.fn();
    await createRoot(async (dispose) => {
      const mail = createMailSlice({
        client: server.client,
        showToast: toast,
        online: () => false,
        searchOffline: () => cached,
      });
      await mail.login({ jmapUrl: 'x', username: 'me@example.org', password: 'p' });
      await mail.searchMessages('needle');
      expect(mail.messages()).toHaveLength(3);
      expect(mail.total()).toBe(3); // the cached slice IS the whole result here
      expect(mail.hasMore()).toBe(false);
      const before = server.calls.length;
      await mail.loadMore();
      expect(server.calls).toHaveLength(before);
      dispose();
    });
  });
});

// ── logout ──────────────────────────────────────────────────────────────────

describe('paging — logout', () => {
  it('drops the cursor and cannot be written by a fetch issued as the old user', async () => {
    const server = makeServer({ inbox: corpus('inbox', 1), archive: corpus('archive', 3) });
    await withServer(server, async (mail) => {
      server.hold('archive');
      const switched = mail.selectMailbox('archive');
      await mail.logout();
      server.releaseAll('archive');
      await switched;

      expect(mail.messages()).toEqual([]);
      expect(mail.total()).toBeNull();
      expect(mail.hasMore()).toBe(false);
      expect(mail.listLoading()).toBe(false);
      expect(mail.loadedRange()).toEqual({ start: 0, end: 0 });
    });
  });
});

// ── the shape t22-e5b consumes ──────────────────────────────────────────────

describe('paging — what the virtualizer is handed', () => {
  it('exposes a query total and a loaded range that disagree, which is the point', async () => {
    const server = makeServer({ inbox: corpus('inbox', 20000) });
    await withServer(server, async (mail) => {
      // `aria-setsize` and the scrollbar must come from `total()`; the rows that
      // exist come from `loadedRange()`. Reading either from the other is the bug.
      expect(mail.total()).toBe(20000);
      expect(mail.loadedRange()).toEqual({ start: 0, end: 50 });
      expect(mail.messages()).toHaveLength(50);
    });
  });

  it('loadedRange is value-compared, so an identical window is the same object', async () => {
    const server = makeServer({ inbox: corpus('inbox', 120) });
    await withServer(server, async (mail) => {
      const first = mail.loadedRange();
      await mail.refreshCurrentMailbox(); // same window, refetched
      // Identity preserved, exactly as `sameWindow` does for the virtualizer's
      // own memo — an equal-but-fresh object would notify it on every push tick.
      expect(mail.loadedRange()).toBe(first);
    });
  });
});
