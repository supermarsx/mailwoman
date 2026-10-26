import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest';
import { createRoot } from 'solid-js';
import { createMailSlice, type MailSlice } from './mail.ts';
import type { SliceContext } from './context.ts';
import { decideMarkRead } from './readPolicy.ts';
import { DEFAULT_PREFS, savePrefs } from '../../screens/Settings/prefs.ts';
import type { Client, Me } from '../../api/client.ts';
import { CAP_MAIL, type Email, type JmapRequest, type JmapResponse, type JmapSession, type Mailbox } from '../../api/jmap-types.ts';

vi.mock('../../api/accountPreferences.ts', () => { let value: Record<string, unknown> = {}; return { accountUiPreferences: () => value, setAccountUiPreferences: (patch: Record<string, unknown>) => { value = {...value, ...patch}; } }; });

// Read state, flags, selection and bulk actions (t28-e16).
//
// The fake server below keeps its own copy of every message and computes the
// folder counts from it, so what these tests compare against is the server's
// state after the requests — not something the slice told it.

const BOXES: Mailbox[] = [
  { id: 'inbox', name: 'Inbox', parentId: null, role: 'inbox', sortOrder: 0, totalEmails: 0, unreadEmails: 0 },
  { id: 'archive', name: 'Archive', parentId: null, role: 'archive', sortOrder: 1, totalEmails: 0, unreadEmails: 0 },
  { id: 'trash', name: 'Trash', parentId: null, role: 'trash', sortOrder: 2, totalEmails: 0, unreadEmails: 0 },
  { id: 'work', name: 'Work', parentId: null, role: null, sortOrder: 3, totalEmails: 0, unreadEmails: 0 },
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

function msg(id: string, over: Partial<Email> = {}): Email {
  return {
    id,
    mailboxIds: { inbox: true },
    from: [{ name: null, email: `${id}@example.org` }],
    to: [{ name: null, email: 'me@example.org' }],
    subject: `Subject ${id}`,
    receivedAt: '2026-01-01T00:00:00Z',
    preview: `preview ${id}`,
    keywords: {},
    ...over,
  };
}

interface FakeServer {
  client: Client;
  jmap: ReturnType<typeof vi.fn>;
  /** The server's own copy of a message. */
  stored(id: string): Email;
  /** Ids the next `Email/set` refuses (notUpdated). */
  refuse: Set<string>;
  /** The `Email/set` requests seen, in order. */
  sets(): Record<string, Record<string, unknown>>[];
}

function fakeServer(seed: Email[]): FakeServer {
  const store = new Map(seed.map((e) => [e.id, structuredClone(e)]));
  const refuse = new Set<string>();
  const counts = (): Mailbox[] =>
    BOXES.map((b) => {
      const inBox = [...store.values()].filter((e) => e.mailboxIds?.[b.id] === true);
      return { ...b, totalEmails: inBox.length, unreadEmails: inBox.filter((e) => e.keywords?.['$seen'] !== true).length };
    });
  const jmap = vi.fn(async (body: JmapRequest): Promise<JmapResponse> => {
    const out: JmapResponse['methodResponses'] = [];
    let queried: string[] = [];
    for (const [name, args, callId] of body.methodCalls) {
      if (name === 'Mailbox/get') {
        out.push(['Mailbox/get', { accountId: 'acct1', state: 's', list: counts(), notFound: [] }, callId]);
      } else if (name === 'Email/query') {
        const inMailbox = (args['filter'] as { inMailbox?: string } | undefined)?.inMailbox;
        queried = [...store.values()].filter((e) => inMailbox === undefined || e.mailboxIds?.[inMailbox]).map((e) => e.id);
        out.push(['Email/query', { accountId: 'acct1', ids: queried, total: queried.length }, callId]);
      } else if (name === 'Email/get') {
        const ids = (args['ids'] as string[] | undefined) ?? queried;
        out.push(['Email/get', { accountId: 'acct1', state: 's', list: ids.map((id) => structuredClone(store.get(id)!)), notFound: [] }, callId]);
      } else if (name === 'Email/set') {
        const update = (args['update'] ?? {}) as Record<string, Record<string, unknown>>;
        const updated: Record<string, null> = {};
        const notUpdated: Record<string, { type: string }> = {};
        for (const [id, patch] of Object.entries(update)) {
          const e = store.get(id);
          if (e === undefined || refuse.has(id)) {
            notUpdated[id] = { type: 'notFound' };
            continue;
          }
          for (const [path, value] of Object.entries(patch)) {
            if (path === 'mailboxIds') e.mailboxIds = value as Record<string, boolean>;
            else if (path.startsWith('keywords/')) {
              const kw = path.slice('keywords/'.length);
              const keywords = { ...(e.keywords ?? {}) };
              if (value === true) keywords[kw] = true;
              else delete keywords[kw];
              e.keywords = keywords;
            }
          }
          updated[id] = null;
        }
        out.push(['Email/set', { accountId: 'acct1', updated, notUpdated: Object.keys(notUpdated).length > 0 ? notUpdated : null }, callId]);
      } else {
        out.push([name, {}, callId]);
      }
    }
    return { methodResponses: out, sessionState: 's' };
  });
  const client = {
    login: vi.fn(async (): Promise<Me> => ({ username: 'me@example.org', accountId: 'acct1' })),
    logout: vi.fn(async () => undefined),
    me: vi.fn(async (): Promise<Me> => ({ username: 'me@example.org', accountId: 'acct1' })),
    session: vi.fn(async () => SESSION),
    jmap,
    sanitize: vi.fn(async (h: string) => h),
    onNetwork: vi.fn(() => () => undefined),
  } as unknown as Client;
  return {
    client,
    jmap,
    stored: (id) => store.get(id)!,
    refuse,
    sets: () =>
      jmap.mock.calls
        .flatMap((c) => (c[0] as JmapRequest).methodCalls)
        .filter((c) => c[0] === 'Email/set')
        .map((c) => (c[1]['update'] ?? {}) as Record<string, Record<string, unknown>>),
  };
}

async function withSlice(
  seed: Email[],
  run: (mail: MailSlice, server: FakeServer, toast: ReturnType<typeof vi.fn>) => Promise<void>,
): Promise<void> {
  const server = fakeServer(seed);
  const toast = vi.fn();
  const ctx: SliceContext = { client: server.client, showToast: toast };
  await createRoot(async (dispose) => {
    const mail = createMailSlice(ctx);
    await mail.login({ username: 'me@example.org', password: 'p' });
    await run(mail, server, toast);
    dispose();
  });
}

const inboxUnread = (mail: MailSlice): number => mail.mailboxes().find((b) => b.id === 'inbox')!.unreadEmails;

beforeEach(() => {
  localStorage.clear();
  savePrefs(DEFAULT_PREFS);
});
afterEach(() => {
  vi.useRealTimers();
  localStorage.clear();
});

describe('mark read on open', () => {
  it('marks an opened unread message read after the delay, on the server, and the unread count follows', async () => {
    vi.useFakeTimers({ toFake: ['setTimeout', 'clearTimeout'] });
    await withSlice([msg('a'), msg('b')], async (mail, server) => {
      // Precondition: unread on the server and counted as such.
      expect(server.stored('a').keywords?.['$seen']).toBeUndefined();
      expect(inboxUnread(mail)).toBe(2);

      await mail.openMessage('a');
      await vi.advanceTimersByTimeAsync(DEFAULT_PREFS.markReadDelayMs - 1);
      // Not yet: a message glanced at for less than the delay stays unread.
      expect(server.sets()).toEqual([]);

      await vi.advanceTimersByTimeAsync(1);
      expect(server.sets()).toEqual([{ a: { 'keywords/$seen': true } }]);
      expect(server.stored('a').keywords?.['$seen']).toBe(true);
      expect(mail.messages().find((m) => m.id === 'a')?.keywords?.['$seen']).toBe(true);
      expect(inboxUnread(mail)).toBe(1);
    });
  });

  it('does not mark a message the reader left before the delay', async () => {
    vi.useFakeTimers({ toFake: ['setTimeout', 'clearTimeout'] });
    await withSlice([msg('a'), msg('b')], async (mail, server) => {
      await mail.openMessage('a');
      await vi.advanceTimersByTimeAsync(300);
      await mail.openMessage('b');
      await vi.advanceTimersByTimeAsync(DEFAULT_PREFS.markReadDelayMs);
      // Only b, which stayed open for the whole delay.
      expect(server.sets()).toEqual([{ b: { 'keywords/$seen': true } }]);
      mail.closeMessage();
      expect(server.stored('a').keywords?.['$seen']).toBeUndefined();
    });
  });

  it('marks nothing when the policy is "never", and honours a configured delay', async () => {
    vi.useFakeTimers({ toFake: ['setTimeout', 'clearTimeout'] });
    await withSlice([msg('a')], async (mail, server) => {
      savePrefs({ ...DEFAULT_PREFS, markRead: 'never' });
      await mail.openMessage('a');
      await vi.advanceTimersByTimeAsync(60_000);
      expect(server.sets()).toEqual([]);
    });
    await withSlice([msg('a')], async (mail, server) => {
      savePrefs({ ...DEFAULT_PREFS, markReadDelayMs: 5000 });
      await mail.openMessage('a');
      await vi.advanceTimersByTimeAsync(4999);
      expect(server.sets()).toEqual([]);
      await vi.advanceTimersByTimeAsync(1);
      expect(server.stored('a').keywords?.['$seen']).toBe(true);
    });
  });

  it('marking the open message unread cancels the pending mark-read', async () => {
    vi.useFakeTimers({ toFake: ['setTimeout', 'clearTimeout'] });
    await withSlice([msg('a', { keywords: { $seen: true } })], async (mail, server) => {
      await mail.openMessage('a');
      await mail.markRead('a', false);
      expect(server.stored('a').keywords?.['$seen']).toBeUndefined();
      await vi.advanceTimersByTimeAsync(60_000);
      // Already seen when opened, so nothing was pending; and nothing re-marked it.
      expect(server.sets()).toEqual([{ a: { 'keywords/$seen': null } }]);
    });
    await withSlice([msg('b')], async (mail, server) => {
      await mail.openMessage('b');
      await mail.markRead('b', false); // nothing to change: it is unread
      await vi.advanceTimersByTimeAsync(60_000);
      expect(server.sets()).toEqual([]);
    });
  });

  it('a failed automatic mark is put back without a toast', async () => {
    vi.useFakeTimers({ toFake: ['setTimeout', 'clearTimeout'] });
    await withSlice([msg('a')], async (mail, server, toast) => {
      server.refuse.add('a');
      await mail.openMessage('a');
      await vi.advanceTimersByTimeAsync(DEFAULT_PREFS.markReadDelayMs);
      expect(mail.messages()[0]?.keywords?.['$seen']).toBeUndefined();
      expect(inboxUnread(mail)).toBe(1);
      expect(toast).not.toHaveBeenCalled();
    });
  });
});

describe('decideMarkRead', () => {
  it('marks an unread message after the policy delay, clamped to a minute', () => {
    const email = msg('a');
    expect(decideMarkRead({ mode: 'on-open', delayMs: 800 }, { kind: 'open', email, mailboxId: 'inbox' })).toEqual({ mark: true, delayMs: 800 });
    expect(decideMarkRead({ mode: 'on-open', delayMs: -5 }, { kind: 'open', email, mailboxId: null })).toEqual({ mark: true, delayMs: 0 });
    expect(decideMarkRead({ mode: 'on-open', delayMs: 1e9 }, { kind: 'open', email, mailboxId: null })).toEqual({ mark: true, delayMs: 60_000 });
  });
  it('leaves a seen message, and everything under "never"', () => {
    expect(decideMarkRead({ mode: 'on-open', delayMs: 0 }, { kind: 'open', email: msg('a', { keywords: { $seen: true } }), mailboxId: null })).toEqual({ mark: false });
    expect(decideMarkRead({ mode: 'never', delayMs: 0 }, { kind: 'open', email: msg('a'), mailboxId: null })).toEqual({ mark: false });
  });
});

describe('bulk actions are one request', () => {
  it('labels a selection in one request, preserves existing labels on undo and reports partial refusal',async()=>{
    await withSlice([msg('a'),msg('b',{keywords:{work:true}}),msg('c')],async(mail,server,toast)=>{
      server.refuse.add('c');server.jmap.mockClear();
      await mail.tagMessages(['a','b','c'],'work',true,'Trabalho');
      expect(mail.pendingUndo()?.label).toContain('Trabalho');
      expect(server.sets()).toEqual([{a:{'keywords/work':true},c:{'keywords/work':true}}]);
      expect(server.stored('a').keywords?.work).toBe(true);expect(server.stored('b').keywords?.work).toBe(true);expect(server.stored('c').keywords?.work).not.toBe(true);
      expect(toast).toHaveBeenCalledWith('error','The server refused the change for one message');
      await mail.undoNow();expect(server.stored('a').keywords?.work).not.toBe(true);expect(server.stored('b').keywords?.work).toBe(true);
      server.jmap.mockClear();await mail.tagMessages(['a'],'$seen',true);expect(server.jmap).not.toHaveBeenCalled();
    });
  });
  it('archiving three selected messages sends one Email/set naming all three, and undo puts each back', async () => {
    await withSlice([msg('a'), msg('b'), msg('c'), msg('d', { mailboxIds: { inbox: true } })], async (mail, server) => {
      mail.setSelection(['a', 'b', 'c']);
      server.jmap.mockClear();
      await mail.archiveMessage(mail.actionTargets());
      expect(server.jmap).toHaveBeenCalledTimes(1);
      expect(server.sets()).toEqual([
        { a: { mailboxIds: { archive: true } }, b: { mailboxIds: { archive: true } }, c: { mailboxIds: { archive: true } } },
      ]);
      for (const id of ['a', 'b', 'c']) expect(server.stored(id).mailboxIds).toEqual({ archive: true });
      expect(mail.messages().map((m) => m.id)).toEqual(['d']);
      expect(mail.selectedIds().size).toBe(0);
      expect(mail.pendingUndo()?.label).toBe('3 messages archived');
      // The counts came back in the same response.
      expect(mail.mailboxes().find((b) => b.id === 'archive')?.totalEmails).toBe(3);

      server.jmap.mockClear();
      await mail.undoNow();
      expect(server.jmap).toHaveBeenCalledTimes(1);
      for (const id of ['a', 'b', 'c']) expect(server.stored(id).mailboxIds).toEqual({ inbox: true });
      expect(mail.messages().map((m) => m.id)).toEqual(['a', 'b', 'c', 'd']);
    });
  });

  it('a message the server refuses comes back to the list; the others stay moved', async () => {
    await withSlice([msg('a'), msg('b'), msg('c')], async (mail, server, toast) => {
      server.refuse.add('b');
      await mail.moveMessage(['a', 'b', 'c'], 'work');
      expect(mail.messages().map((m) => m.id)).toEqual(['b']);
      expect(server.stored('a').mailboxIds).toEqual({ work: true });
      expect(server.stored('b').mailboxIds).toEqual({ inbox: true });
      expect(toast).toHaveBeenCalledWith('error', 'The server refused the change for one message');
      expect(mail.pendingUndo()?.label).toBe(`2 messages moved to ${'⁨'}Work${'⁩'}`);
    });
  });

  it('mark read, mark unread and flag cover the selection in one request each, and undo restores each message', async () => {
    await withSlice(
      [msg('a'), msg('b', { keywords: { $seen: true } }), msg('c', { keywords: { $flagged: true } })],
      async (mail, server) => {
        expect(inboxUnread(mail)).toBe(2);
        server.jmap.mockClear();
        await mail.markRead(['a', 'b', 'c'], true);
        expect(server.jmap).toHaveBeenCalledTimes(1);
        // b was already read, so it is not in the request.
        expect(server.sets()).toEqual([{ a: { 'keywords/$seen': true }, c: { 'keywords/$seen': true } }]);
        expect(inboxUnread(mail)).toBe(0);
        await mail.undoNow();
        expect(server.stored('a').keywords?.['$seen']).toBeUndefined();
        expect(server.stored('b').keywords?.['$seen']).toBe(true);
        expect(inboxUnread(mail)).toBe(2);

        server.jmap.mockClear();
        await mail.flagMessage(['a', 'b', 'c'], true);
        expect(server.jmap).toHaveBeenCalledTimes(1);
        expect(server.sets()).toEqual([{ a: { 'keywords/$flagged': true }, b: { 'keywords/$flagged': true } }]);
        expect(mail.pendingUndo()?.label).toBe('2 messages flagged');
        await mail.undoNow();
        expect(server.stored('a').keywords?.['$flagged']).toBeUndefined();
        expect(server.stored('c').keywords?.['$flagged']).toBe(true);
      },
    );
  });

  it('a mark that changes nothing sends nothing and offers no undo', async () => {
    await withSlice([msg('a', { keywords: { $seen: true } })], async (mail, server) => {
      server.jmap.mockClear();
      await mail.markRead('a', true);
      expect(server.jmap).not.toHaveBeenCalled();
      expect(mail.pendingUndo()).toBeNull();
    });
  });
});

describe('selection', () => {
  it('toggles, selects all in view, follows the list, and clears on a folder change', async () => {
    await withSlice([msg('a'), msg('b'), msg('c')], async (mail) => {
      mail.toggleSelection(['b']);
      mail.toggleSelection(['a']);
      // In list order, whatever order they were picked in.
      expect(mail.actionTargets()).toEqual(['a', 'b']);
      mail.toggleSelection(['a', 'b']); // all in: removes both
      expect(mail.selectedIds().size).toBe(0);

      mail.selectAllInView();
      expect([...mail.selectedIds()]).toEqual(['a', 'b', 'c']);
      await mail.trashMessage(['c']);
      // A message that left the list is not selected any more.
      expect([...mail.selectedIds()]).toEqual(['a', 'b']);

      await mail.selectMailbox('inbox');
      expect(mail.selectedIds().size).toBe(0);
    });
  });

  it('without a selection, actions target the open message', async () => {
    await withSlice([msg('a'), msg('b')], async (mail) => {
      expect(mail.actionTargets()).toEqual([]);
      await mail.openMessage('b');
      expect(mail.actionTargets()).toEqual(['b']);
      mail.setSelection(['a']);
      expect(mail.actionTargets()).toEqual(['a']);
    });
  });
});


describe('authoritative folder count reconciliation',()=>{
  it('refreshes counts after another device changes read state',async()=>{
    await withSlice([msg('a'),msg('b')],async(mail,server)=>{
      server.stored('a').keywords={$seen:true};await mail.refreshCurrentMailbox();
      expect(inboxUnread(mail)).toBe(1);
    });
  });
  it('rejects a foreign-account counts payload and reconciles from the server',async()=>{
    await withSlice([msg('a')],async(mail,server)=>{
      const original=server.jmap.getMockImplementation() as (body:JmapRequest)=>Promise<JmapResponse>;
      server.jmap.mockImplementation(async(body:JmapRequest)=>{const response=await original(body);if(body.methodCalls.some(call=>call[0]==='Email/set'))for(const call of response.methodResponses)if(call[0]==='Mailbox/get'){call[1]['accountId']='foreign';for(const box of call[1]['list'] as Mailbox[])box.unreadEmails=999;}return response;});
      await mail.markRead('a',true);expect(inboxUnread(mail)).toBe(0);
    });
  });
  it('does not let an older completed mutation replace newer authoritative counts',async()=>{
    await withSlice([msg('a'),msg('b')],async(mail,server)=>{
      const original=server.jmap.getMockImplementation() as (body:JmapRequest)=>Promise<JmapResponse>;let release:(()=>void)|undefined,held=false;
      server.jmap.mockImplementation(async(body:JmapRequest)=>{const response=await original(body);if(!held&&body.methodCalls.some(call=>call[0]==='Email/set')){held=true;await new Promise<void>(resolve=>{release=resolve;});}return response;});
      const first=mail.markRead('a',true);await vi.waitFor(()=>expect(release).toBeDefined());await mail.markRead('b',true);release!();await first;expect(inboxUnread(mail)).toBe(0);
    });
  });
  it('does not restore a refused move undo or fabricate its counts',async()=>{
    await withSlice([msg('a'),msg('b')],async(mail,server)=>{
      await mail.archiveMessage(['a','b']);server.refuse.add('a');await mail.undoNow();
      expect(mail.messages().map(message=>message.id)).toEqual(['b']);
      expect(mail.mailboxes().find(box=>box.id==='inbox')?.totalEmails).toBe(1);
      expect(mail.mailboxes().find(box=>box.id==='archive')?.totalEmails).toBe(1);
    });
  });
  it('coalesces refresh count reads while retaining authoritative results',async()=>{
    await withSlice([msg('a')],async(mail,server)=>{
      const original=server.jmap.getMockImplementation() as (body:JmapRequest)=>Promise<JmapResponse>;
      let release:(()=>void)|undefined,reads=0;
      server.jmap.mockImplementation(async(body:JmapRequest)=>{
        if(body.methodCalls.length===1&&body.methodCalls[0]?.[0]==='Mailbox/get'){
          reads++;if(reads===1)await new Promise<void>(resolve=>{release=resolve;});
        }return original(body);
      });
      const first=mail.refreshCurrentMailbox();await vi.waitFor(()=>expect(release).toBeDefined());
      const second=mail.refreshCurrentMailbox();server.stored('a').keywords={$seen:true};release!();await Promise.all([first,second]);
      expect(inboxUnread(mail)).toBe(0);expect(reads).toBeLessThanOrEqual(2);
    });
  });
  it('does not publish an old move undo into a later sign-in of the same account',async()=>{
    await withSlice([msg('a')],async(mail,server)=>{
      const original=server.jmap.getMockImplementation() as (body:JmapRequest)=>Promise<JmapResponse>;let release:(()=>void)|undefined,held=false;
      server.jmap.mockImplementation(async(body:JmapRequest)=>{const response=await original(body);if(!held&&body.methodCalls.some(call=>call[0]==='Email/set')){held=true;await new Promise<void>(resolve=>{release=resolve;});}return response;});
      const moved=mail.archiveMessage('a');await vi.waitFor(()=>expect(release).toBeDefined());await mail.logout();await mail.login({username:'me@example.org',password:'p'});release!();await moved;expect(mail.pendingUndo()).toBeNull();
    });
  });
});
