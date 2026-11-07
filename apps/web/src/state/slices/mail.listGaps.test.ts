// "Message lists don't show all messages": a page whose Email/get answered fewer
// rows than Email/query named (notFound, a dropped id) used to count as a SHORT
// page, which ended the query and hid every message after it. These walk a
// 400-message folder through the slice with a server that omits ids, shifts
// under the reader, and asserts the loaded list equals the query in order.

import { describe, it, expect, vi } from 'vitest';
import { createRoot } from 'solid-js';
import { createMailSlice, type MailSlice } from './mail.ts';
import type { Client, Me } from '../../api/client.ts';
import { CAP_MAIL, type Email, type JmapRequest, type JmapResponse, type JmapSession } from '../../api/jmap-types.ts';

const SESSION: JmapSession = {
  capabilities: {},
  accounts: { acct1: { name: 'T', isPersonal: true, isReadOnly: false, accountCapabilities: {} } },
  primaryAccounts: { [CAP_MAIL]: 'acct1' },
  username: 'me@example.org', apiUrl: '/jmap/api', downloadUrl: '/d', uploadUrl: '/u', eventSourceUrl: '/e', state: 's0',
};
const MAILBOXES = [{ id: 'inbox', name: 'Inbox', parentId: null, role: 'inbox', sortOrder: 0, totalEmails: 0, unreadEmails: 0 }];

const mail1 = (id: string): Email => ({ id, mailboxIds: { inbox: true }, from: [{ name: null, email: `${id}@x.org` }], to: [], subject: id, receivedAt: '2026-01-01T00:00:00Z', preview: id, keywords: {} });

interface Opts {
  /** Ids Email/get never returns (a message expunged under the query). */
  gone?: (id: string) => boolean;
  /** Ids the FIRST Email/get omits but a retry for them returns. */
  flaky?: (id: string) => boolean;
}

function makeServer(folder: { rows: Email[] }, opts: Opts = {}) {
  const seenFlaky = new Set<string>();
  const queries: number[] = [];
  const jmap = vi.fn(async (body: JmapRequest): Promise<JmapResponse> => {
    if (body.methodCalls.some((c) => c[0] === 'Mailbox/get')) return { methodResponses: [['Mailbox/get', { accountId: 'acct1', state: 's', list: MAILBOXES, notFound: [] }, 'c0']], sessionState: 's' };
    const q = body.methodCalls.find((c) => c[0] === 'Email/query');
    const get = body.methodCalls.find((c) => c[0] === 'Email/get');
    const out: JmapResponse['methodResponses'] = [];
    let ids: string[];
    if (q !== undefined) {
      const a = q[1] as { position?: number; limit?: number; calculateTotal?: boolean };
      const pos = a.position ?? 0;
      queries.push(pos);
      ids = folder.rows.slice(pos, pos + (a.limit ?? 0)).map((e) => e.id);
      out.push(['Email/query', { accountId: 'acct1', queryState: 'q', ids, position: pos, ...(a.calculateTotal ? { total: folder.rows.length } : {}) }, 'q']);
    } else ids = (get?.[1] as { ids: string[] }).ids;
    if (get !== undefined) {
      const byId = new Map(folder.rows.map((e) => [e.id, e] as const));
      const list: Email[] = [];
      for (const id of ids) {
        const e = byId.get(id);
        if (e === undefined || opts.gone?.(id)) continue;
        if (opts.flaky?.(id) && !seenFlaky.has(id)) { seenFlaky.add(id); continue; }
        list.push(e);
      }
      out.push(['Email/get', { accountId: 'acct1', state: 's', list, notFound: [] }, 'g']);
    }
    return { methodResponses: out, sessionState: 's' };
  });
  const client: Client = {
    login: vi.fn(async (): Promise<Me> => ({ username: 'me@example.org', accountId: 'acct1' })),
    logout: vi.fn(async () => undefined), me: vi.fn(async (): Promise<Me> => ({ username: 'me@example.org', accountId: 'acct1' })),
    session: vi.fn(async () => SESSION), jmap, sanitize: vi.fn(async (h: string) => h), onNetwork: vi.fn(() => () => undefined),
  };
  return { client, queries };
}

async function walk(folder: { rows: Email[] }, opts: Opts, between?: (mail: MailSlice, page: number) => void): Promise<{ ids: string[]; total: number | null; hasMore: boolean; queries: number[] }> {
  const server = makeServer(folder, opts);
  let result!: { ids: string[]; total: number | null; hasMore: boolean };
  await createRoot(async (dispose) => {
    const mail = createMailSlice({ client: server.client, showToast: vi.fn() });
    await mail.login({ username: 'me@example.org', password: 'p' });
    for (let i = 0; i < 40 && mail.hasMore(); i += 1) { between?.(mail, i); await mail.loadMore(); }
    result = { ids: mail.messages().map((m) => m.id), total: mail.total(), hasMore: mail.hasMore() };
    dispose();
  });
  return { ...result, queries: server.queries };
}

const folder400 = (): { rows: Email[] } => ({ rows: Array.from({ length: 400 }, (_, i) => mail1(`m${i + 1}`)) });
const all = (f: { rows: Email[] }): string[] => f.rows.map((e) => e.id);

describe('list paging — every message of the folder is reachable', () => {
  it('a plain 400-message walk loads all rows in query order and the total matches', async () => {
    const f = folder400();
    const r = await walk(f, {});
    expect(r.ids).toEqual(all(f));
    expect(r.total).toBe(400);
    expect(r.hasMore).toBe(false);
  });

  it('ids Email/get omits for good do not end the query: the rest of the folder still loads', async () => {
    const f = folder400();
    const gone = (id: string): boolean => Number(id.slice(1)) % 7 === 0;
    const r = await walk(f, { gone });
    expect(r.ids).toEqual(all(f).filter((id) => !gone(id)));
    expect(r.total).toBe(r.ids.length);
    expect(r.hasMore).toBe(false);
  });

  it('ids Email/get omits once are fetched again and show up in place', async () => {
    const f = folder400();
    const r = await walk(f, { flaky: (id) => Number(id.slice(1)) % 5 === 0 });
    expect(r.ids).toEqual(all(f));
    expect(r.total).toBe(400);
  });

  it('a page that is entirely missing does not strand the messages behind it', async () => {
    const f = folder400();
    const gone = (id: string): boolean => { const n = Number(id.slice(1)); return n > 50 && n <= 100; };
    const r = await walk(f, { gone });
    expect(r.ids).toEqual(all(f).filter((id) => !gone(id)));
  });

  it('mail arriving on top between pages never loses one that is already below', async () => {
    const f = folder400();
    const original = all(f);
    const r = await walk(f, {}, (_m, page) => { if (page === 1 || page === 3) f.rows.unshift(mail1(`new${page}`)); });
    for (const id of original) expect(r.ids).toContain(id);
    expect(new Set(r.ids).size).toBe(r.ids.length);
    expect(r.ids.filter((id) => !id.startsWith('new'))).toEqual(original);
  });
});
