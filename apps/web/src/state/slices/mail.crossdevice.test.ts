import { describe, it, expect, vi, beforeEach } from 'vitest';
import { createRoot } from 'solid-js';
import { createMailSlice, type MailSlice } from './mail.ts';
import type { SliceContext } from './context.ts';
import { DEFAULT_PREFS, savePrefs } from '../../screens/Settings/prefs.ts';
import type { Client, Me } from '../../api/client.ts';
import { CAP_MAIL, type Email, type JmapRequest, type JmapResponse, type JmapSession, type Mailbox } from '../../api/jmap-types.ts';

vi.mock('../../api/accountPreferences.ts', () => { let value: Record<string, unknown> = {}; return { accountUiPreferences: () => value, setAccountUiPreferences: (patch: Record<string, unknown>) => { value = {...value, ...patch}; } }; });

// Open readers follow changes made on another device (t39 L8). The fake server
// keeps the messages; "another device" edits that copy directly, then the push
// path (refreshCurrentMailbox) runs, as the realtime StateChange makes it run.

let BOXES: Mailbox[] = [
  { id: 'inbox', name: 'Inbox', parentId: null, role: 'inbox', sortOrder: 0, totalEmails: 0, unreadEmails: 0 },
  { id: 'archive', name: 'Archive', parentId: null, role: 'archive', sortOrder: 1, totalEmails: 0, unreadEmails: 0 },
  { id: 'trash', name: 'Trash', parentId: null, role: 'trash', sortOrder: 2, totalEmails: 0, unreadEmails: 0 },
];
const SESSION: JmapSession = {
  capabilities: {}, accounts: { acct1: { name: 'T', isPersonal: true, isReadOnly: false, accountCapabilities: {} } },
  primaryAccounts: { [CAP_MAIL]: 'acct1' }, username: 'me@example.org', apiUrl: '/jmap/api', downloadUrl: '/d', uploadUrl: '/u', eventSourceUrl: '/e', state: 's0',
};
const msg = (id: string): Email => ({ id, mailboxIds: { inbox: true }, from: [{ name: null, email: `${id}@example.org` }], to: [{ name: null, email: 'me@example.org' }], subject: `Subject ${id}`, receivedAt: '2026-01-01T00:00:00Z', preview: id, keywords: { $seen: true } }) as Email;

function server(seed: Email[]) {
  const store = new Map(seed.map((e) => [e.id, structuredClone(e)]));
  const jmap = vi.fn(async (body: JmapRequest): Promise<JmapResponse> => {
    const out: JmapResponse['methodResponses'] = [];
    let queried: string[] = [];
    for (const [name, args, callId] of body.methodCalls) {
      if (name === 'Mailbox/get') out.push(['Mailbox/get', { accountId: 'acct1', state: 's', list: BOXES, notFound: [] }, callId]);
      else if (name === 'Email/query') {
        const inMailbox = (args['filter'] as { inMailbox?: string } | undefined)?.inMailbox;
        queried = [...store.values()].filter((e) => inMailbox === undefined || e.mailboxIds?.[inMailbox]).map((e) => e.id);
        out.push(['Email/query', { accountId: 'acct1', ids: queried, total: queried.length }, callId]);
      } else if (name === 'Email/get') {
        const ids = (args['ids'] as string[] | undefined) ?? queried;
        out.push(['Email/get', { accountId: 'acct1', state: 's', list: ids.filter((id) => store.has(id)).map((id) => structuredClone(store.get(id)!)), notFound: ids.filter((id) => !store.has(id)) }, callId]);
      } else if (name === 'Email/set') {
        const update = (args['update'] ?? {}) as Record<string, Record<string, unknown>>;
        for (const [id, patch] of Object.entries(update)) if (patch['mailboxIds']) store.get(id)!.mailboxIds = patch['mailboxIds'] as Record<string, boolean>;
        out.push(['Email/set', { accountId: 'acct1', updated: Object.fromEntries(Object.keys(update).map((id) => [id, null])) }, callId]);
      } else out.push([name, {}, callId]);
    }
    return { methodResponses: out, sessionState: 's' };
  });
  const client = {
    login: vi.fn(async (): Promise<Me> => ({ username: 'me@example.org', accountId: 'acct1' })), logout: vi.fn(async () => undefined),
    me: vi.fn(async (): Promise<Me> => ({ username: 'me@example.org', accountId: 'acct1' })), session: vi.fn(async () => SESSION),
    jmap, sanitize: vi.fn(async (h: string) => h), onNetwork: vi.fn(() => () => undefined),
  } as unknown as Client;
  return { client, store };
}

async function withSlice(seed: Email[], run: (mail: MailSlice, s: ReturnType<typeof server>) => Promise<void>): Promise<void> {
  const s = server(seed);
  const ctx: SliceContext = { client: s.client, showToast: vi.fn() };
  await createRoot(async (dispose) => {
    const mail = createMailSlice(ctx);
    await mail.login({ username: 'me@example.org', password: 'p' });
    await run(mail, s);
    dispose();
  });
}

const BASE_BOXES = [...BOXES];
beforeEach(() => { localStorage.clear(); savePrefs(DEFAULT_PREFS); BOXES = [...BASE_BOXES]; });

describe('open messages follow another device', () => {
  it('a flag and a move made elsewhere reach the open reader; the move is a notice, the body stays', async () => {
    await withSlice([msg('a')], async (mail, s) => {
      const view = mail.createReaderView();
      await view.openMessage('a');
      s.store.get('a')!.keywords = { $seen: true, $flagged: true };
      s.store.get('a')!.mailboxIds = { archive: true };
      await mail.refreshCurrentMailbox();
      await vi.waitFor(() => expect(view.remoteChange()).toEqual({ kind: 'moved', mailboxIds: { archive: true } }));
      expect(view.openEmail()?.keywords?.['$flagged']).toBe(true);
      expect(view.openEmail()?.id).toBe('a');
      view.dispose();
    });
  });

  it('a message deleted elsewhere stays open with a notice', async () => {
    await withSlice([msg('a')], async (mail, s) => {
      const view = mail.createReaderView();
      await view.openMessage('a');
      s.store.delete('a');
      await mail.refreshCurrentMailbox();
      await vi.waitFor(() => expect(view.remoteChange()).toEqual({ kind: 'deleted' }));
      expect(view.openEmail()?.id).toBe('a');
      view.dispose();
    });
  });

  it("this window's own move is not reported as a change from elsewhere", async () => {
    await withSlice([msg('a')], async (mail, s) => {
      const view = mail.createReaderView();
      await view.openMessage('a');
      await mail.archiveMessage('a');
      expect(s.store.get('a')!.mailboxIds).toEqual({ archive: true });
      await mail.refreshCurrentMailbox();
      await new Promise((resolve) => setTimeout(resolve, 0));
      expect(view.remoteChange()).toBeNull();
      expect(view.openEmail()?.mailboxIds).toEqual({ archive: true });
      view.dispose();
    });
  });

  it('a folder created, then deleted, on another device appears and leaves the folder list', async () => {
    await withSlice([msg('a')], async (mail) => {
      BOXES = [...BOXES, { id: 'new', name: 'Projects', parentId: null, role: null, sortOrder: 9, totalEmails: 0, unreadEmails: 0 }];
      await mail.refreshFolders();
      await vi.waitFor(() => expect(mail.mailboxes().map((b) => b.name)).toContain('Projects'));
      BOXES = BOXES.filter((b) => b.id !== 'new');
      await mail.refreshFolders();
      await vi.waitFor(() => expect(mail.mailboxes().map((b) => b.name)).not.toContain('Projects'));
      expect(mail.mailboxes().map((b) => b.id)).toEqual(expect.arrayContaining(['inbox', 'archive', 'trash']));
    });
  });
});
