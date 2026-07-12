import { describe, it, expect, vi, beforeEach } from 'vitest';
import { createRoot } from 'solid-js';
import { createMailSlice, type MailSlice } from './mail.ts';
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
  { id: 'junk', name: 'Spam', parentId: null, role: 'junk', sortOrder: 3, totalEmails: 0, unreadEmails: 0 },
];

function email(id: string, over: Partial<Email> = {}): Email {
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

/** A fake JMAP client whose inbox listing returns `seed()`. */
function makeClient(seed: () => Email[]): { client: Client; jmap: ReturnType<typeof vi.fn> } {
  const jmap = vi.fn(async (body: JmapRequest): Promise<JmapResponse> => {
    const names = body.methodCalls.map((c) => c[0]);
    if (names.includes('Mailbox/get')) {
      return { methodResponses: [['Mailbox/get', { accountId: 'acct1', state: 's', list: MAILBOXES, notFound: [] }, 'c0']], sessionState: 's' };
    }
    if (names.includes('EmailSubmission/set')) {
      return {
        methodResponses: [
          ['Email/set', { accountId: 'acct1', created: { draft: { id: 'draft1' } }, notCreated: null }, 'set'],
          ['EmailSubmission/set', { accountId: 'acct1', created: { send: { id: 'sub1' } }, notCreated: null }, 'submit'],
        ],
        sessionState: 's',
      };
    }
    if (names.includes('Email/get')) {
      return {
        methodResponses: [
          ['Email/query', { accountId: 'acct1', ids: seed().map((e) => e.id) }, 'q'],
          ['Email/get', { accountId: 'acct1', state: 's', list: seed(), notFound: [] }, 'g'],
        ],
        sessionState: 's',
      };
    }
    // Email/set mutations + EmailSubmission cancel: echo an empty ok per call.
    return { methodResponses: body.methodCalls.map((c) => [c[0], {}, c[2]] as JmapResponse['methodResponses'][number]), sessionState: 's' };
  });
  const client: Client = {
    login: vi.fn(async (): Promise<Me> => ({ username: 'me@example.org', accountId: 'acct1' })),
    logout: vi.fn(async () => undefined),
    me: vi.fn(async (): Promise<Me> => ({ username: 'me@example.org', accountId: 'acct1' })),
    session: vi.fn(async () => SESSION),
    jmap,
    sanitize: vi.fn(async (h: string) => h),
    onNetwork: vi.fn(() => () => undefined),
  };
  return { client, jmap };
}

async function withInbox(
  seed: Email[],
  run: (mail: MailSlice, ctx: { toast: ReturnType<typeof vi.fn>; jmap: ReturnType<typeof vi.fn>; setSeed: (e: Email[]) => void }) => Promise<void>,
): Promise<void> {
  let current = seed;
  const { client, jmap } = makeClient(() => current);
  const toast = vi.fn();
  const ctx: SliceContext = { client, showToast: toast };
  await createRoot(async (dispose) => {
    const mail = createMailSlice(ctx);
    await mail.login({ jmapUrl: 'x', username: 'me@example.org', password: 'p' });
    await run(mail, { toast, jmap, setSeed: (e) => (current = e) });
    dispose();
  });
}

// ── tests ───────────────────────────────────────────────────────────────────
describe('mail slice — list + pins', () => {
  it('loads the inbox on login', async () => {
    await withInbox([email('a'), email('b')], async (mail) => {
      expect(mail.messages().map((m) => m.id)).toEqual(['a', 'b']);
      expect(mail.selectedMailboxId()).toBe('inbox');
    });
  });

  it('floats pinned messages to the top, preserving order otherwise', async () => {
    await withInbox([email('a'), email('b', { pinned: true }), email('c')], async (mail) => {
      expect(mail.visibleMessages().map((m) => m.id)).toEqual(['b', 'a', 'c']);
    });
  });

  it('pinMessage reorders and offers an undo that reverts', async () => {
    await withInbox([email('a'), email('b')], async (mail) => {
      await mail.pinMessage('b', true);
      expect(mail.visibleMessages()[0]!.id).toBe('b');
      expect(mail.pendingUndo()?.label).toBe('Pinned');
      await mail.undoNow();
      expect(mail.visibleMessages().map((m) => m.id)).toEqual(['a', 'b']);
    });
  });
});

describe('mail slice — tags', () => {
  it('applyTag adds the keyword and undo removes it', async () => {
    await withInbox([email('a')], async (mail) => {
      await mail.applyTag('a', 'work');
      expect(mail.messages()[0]!.keywords?.['work']).toBe(true);
      expect(mail.pendingUndo()?.label).toBe('Label added');
      await mail.undoNow();
      expect(mail.messages()[0]!.keywords?.['work']).toBeUndefined();
    });
  });

  it('removeTag deletes the keyword and undo restores it', async () => {
    await withInbox([email('a', { keywords: { work: true } })], async (mail) => {
      await mail.removeTag('a', 'work');
      expect(mail.messages()[0]!.keywords?.['work']).toBeUndefined();
      await mail.undoNow();
      expect(mail.messages()[0]!.keywords?.['work']).toBe(true);
    });
  });
});

describe('mail slice — snooze', () => {
  it('hides a snoozed message from the visible list and resurfaces on unsnooze', async () => {
    await withInbox([email('a'), email('b')], async (mail) => {
      const future = new Date(Date.now() + 3_600_000).toISOString();
      await mail.snoozeMessage('a', future);
      expect(mail.visibleMessages().map((m) => m.id)).toEqual(['b']);
      expect(mail.snoozedMessages().map((m) => m.id)).toEqual(['a']);
      await mail.unsnoozeMessage('a');
      expect(mail.visibleMessages().map((m) => m.id)).toContain('a');
    });
  });

  it('treats an elapsed snooze time as visible again', async () => {
    const past = new Date(Date.now() - 1000).toISOString();
    await withInbox([email('a', { snoozedUntil: past })], async (mail) => {
      expect(mail.visibleMessages().map((m) => m.id)).toEqual(['a']);
    });
  });
});

describe('mail slice — sweep', () => {
  const bulk = [
    email('m1', { from: [{ name: null, email: 'news@shop.example' }], receivedAt: '2026-03-01T00:00:00Z' }),
    email('m2', { from: [{ name: null, email: 'news@shop.example' }], receivedAt: '2026-02-01T00:00:00Z' }),
    email('m3', { from: [{ name: null, email: 'news@shop.example' }], receivedAt: '2026-01-01T00:00:00Z' }),
    email('keep', { from: [{ name: null, email: 'friend@example.org' }] }),
  ];

  it('previews all mail from a sender', async () => {
    await withInbox(bulk, async (mail) => {
      expect(mail.sweepPreview('news@shop.example', 'all').map((m) => m.id)).toEqual(['m1', 'm2', 'm3']);
    });
  });

  it('keep-latest previews everything but the newest', async () => {
    await withInbox(bulk, async (mail) => {
      expect(mail.sweepPreview('news@shop.example', 'keep-latest').map((m) => m.id)).toEqual(['m2', 'm3']);
    });
  });

  it('executes a sweep, removing the victims, and undo restores them', async () => {
    await withInbox(bulk, async (mail) => {
      await mail.executeSweep('news@shop.example', 'all');
      expect(mail.messages().map((m) => m.id)).toEqual(['keep']);
      expect(mail.pendingUndo()?.label).toContain('Swept 3');
      await mail.undoNow();
      expect(mail.messages().map((m) => m.id)).toEqual(['m1', 'm2', 'm3', 'keep']);
    });
  });

  it('block strategy records the sender in the blocklist', async () => {
    await withInbox(bulk, async (mail) => {
      await mail.executeSweep('news@shop.example', 'block');
      expect(mail.blockedSenders()).toContain('news@shop.example');
    });
  });
});

describe('mail slice — focused / unified inbox', () => {
  it('splits bulk senders into Other by heuristic', async () => {
    const seed = [
      email('person', { from: [{ name: 'A Person', email: 'a@example.org' }] }),
      email('news', { from: [{ name: null, email: 'newsletter@shop.example' }] }),
    ];
    await withInbox(seed, async (mail) => {
      expect(mail.focusedMessages().map((m) => m.id)).toEqual(['person']);
      expect(mail.otherMessages().map((m) => m.id)).toEqual(['news']);
    });
  });

  it('listMessages shows everything until focused mode is enabled', async () => {
    const seed = [
      email('person', { from: [{ name: 'A Person', email: 'a@example.org' }] }),
      email('news', { from: [{ name: null, email: 'newsletter@shop.example' }] }),
    ];
    await withInbox(seed, async (mail) => {
      expect(mail.listMessages().map((m) => m.id)).toEqual(['person', 'news']);
      mail.setFocusedInbox(true);
      expect(mail.listMessages().map((m) => m.id)).toEqual(['person']);
      mail.setInboxTab('other');
      expect(mail.listMessages().map((m) => m.id)).toEqual(['news']);
    });
  });

  it('sender training overrides the heuristic', async () => {
    await withInbox([email('news', { from: [{ name: null, email: 'newsletter@shop.example' }] })], async (mail) => {
      expect(mail.otherMessages().map((m) => m.id)).toEqual(['news']);
      mail.trainSender('newsletter@shop.example', 'focused');
      expect(mail.focusedMessages().map((m) => m.id)).toEqual(['news']);
    });
  });
});

describe('mail slice — undo-send', () => {
  beforeEach(() => localStorage.clear());

  it('sending shows a Cancel toast; cancel calls EmailSubmission/set canceled', async () => {
    await withInbox([email('a')], async (mail, { jmap }) => {
      await mail.sendMessage({ to: 'you@example.org', subject: 'Hi', htmlBody: '<p>x</p>', holdSeconds: 10 });
      const undo = mail.pendingUndo();
      expect(undo?.label).toBe('Message sent');
      expect(undo?.actionLabel).toBe('Cancel');

      jmap.mockClear();
      await mail.undoNow();
      const cancelCall = jmap.mock.calls.find((call) => {
        const body = call[0] as JmapRequest;
        return body.methodCalls.some((c) => c[0] === 'EmailSubmission/set' && 'update' in c[1]);
      });
      expect(cancelCall).toBeDefined();
    });
  });

  it('send-later shows a scheduled toast, not an undo window', async () => {
    await withInbox([email('a')], async (mail, { toast }) => {
      await mail.sendMessage({
        to: 'you@example.org',
        subject: 'Later',
        htmlBody: '<p>x</p>',
        sendAt: new Date(Date.now() + 3_600_000).toISOString(),
      });
      expect(mail.pendingUndo()).toBeNull();
      expect(toast).toHaveBeenCalledWith('success', 'Scheduled to send');
    });
  });
});
