import { describe, it, expect, vi, beforeEach } from 'vitest';
import { createRoot } from 'solid-js';
import { createMailSlice, extractHtmlBody, type MailSlice } from './mail.ts';
import type { SliceContext } from './context.ts';
import type { Client, Me } from '../../api/client.ts';
import {
  CAP_MAIL,
  type Email,
  type JmapRequest,
  type JmapResponse,
  type JmapSession,
  type Mailbox,
  type EmailBodyPart,
} from '../../api/jmap-types.ts';

// ── fixtures ────────────────────────────────────────────────────────────────
const MAILBOXES: Mailbox[] = [
  { id: 'inbox', name: 'Inbox', parentId: null, role: 'inbox', sortOrder: 0, totalEmails: 0, unreadEmails: 0 },
  { id: 'archive', name: 'Archive', parentId: null, role: 'archive', sortOrder: 1, totalEmails: 0, unreadEmails: 0 },
  { id: 'trash', name: 'Trash', parentId: null, role: 'trash', sortOrder: 2, totalEmails: 0, unreadEmails: 0 },
  { id: 'junk', name: 'Spam', parentId: null, role: 'junk', sortOrder: 3, totalEmails: 0, unreadEmails: 0 },
];

/** A complete `EmailBodyPart` — the JMAP shape requires blobId + size. */
function part(partId: string, type: string): EmailBodyPart {
  return { partId, blobId: `b-${partId}`, size: 1, type };
}

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
function makeClient(
  seed: () => Email[],
  boxes: Mailbox[] = MAILBOXES,
): { client: Client; jmap: ReturnType<typeof vi.fn> } {
  const jmap = vi.fn(async (body: JmapRequest): Promise<JmapResponse> => {
    const names = body.methodCalls.map((c) => c[0]);
    if (names.includes('Mailbox/get')) {
      return { methodResponses: [['Mailbox/get', { accountId: 'acct1', state: 's', list: boxes, notFound: [] }, 'c0']], sessionState: 's' };
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
  run: (mail: MailSlice, ctx: { toast: ReturnType<typeof vi.fn>; jmap: ReturnType<typeof vi.fn>; client: Client; setSeed: (e: Email[]) => void }) => Promise<void>,
): Promise<void> {
  let current = seed;
  const { client, jmap } = makeClient(() => current);
  const toast = vi.fn();
  const ctx: SliceContext = { client, showToast: toast };
  await createRoot(async (dispose) => {
    const mail = createMailSlice(ctx);
    await mail.login({ jmapUrl: 'x', username: 'me@example.org', password: 'p' });
    await run(mail, { toast, jmap, client, setSeed: (e) => (current = e) });
    dispose();
  });
}

/**
 * As `withInbox`, but with the OPTIONAL `SliceContext` seams populated — the
 * offline queue, the offline search, and the peer-tab broadcast. A slice built
 * without them takes the direct/online path, which is what every test above
 * exercises; these are the branches that only exist once `store.ts` wires the
 * offline slice in. (t19-e12)
 */
async function withDeps(
  seed: Email[],
  deps: Omit<SliceContext, 'client' | 'showToast'>,
  run: (
    mail: MailSlice,
    ctx: {
      toast: ReturnType<typeof vi.fn>;
      jmap: ReturnType<typeof vi.fn>;
      client: Client;
      setSeed: (e: Email[]) => void;
    },
  ) => Promise<void>,
): Promise<void> {
  let current = seed;
  const { client, jmap } = makeClient(() => current);
  const toast = vi.fn();
  const ctx: SliceContext = { client, showToast: toast, ...deps };
  await createRoot(async (dispose) => {
    const mail = createMailSlice(ctx);
    await mail.login({ jmapUrl: 'x', username: 'me@example.org', password: 'p' });
    await run(mail, { toast, jmap, client, setSeed: (e) => (current = e) });
    dispose();
  });
}

/** Boot a slice against an account whose mailbox list is missing some roles. */
async function withBoxes(
  seed: Email[],
  boxes: Mailbox[],
  run: (mail: MailSlice, ctx: { toast: ReturnType<typeof vi.fn> }) => Promise<void>,
): Promise<void> {
  const { client } = makeClient(() => seed, boxes);
  const toast = vi.fn();
  await createRoot(async (dispose) => {
    const mail = createMailSlice({ client, showToast: toast });
    await mail.login({ jmapUrl: 'x', username: 'me@example.org', password: 'p' });
    await run(mail, { toast });
    dispose();
  });
}

/** Mailboxes minus the named roles, to drive the refusal paths. */
function withoutRoles(...roles: string[]): Mailbox[] {
  return MAILBOXES.filter((m) => !roles.includes(m.role ?? ''));
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

// ═════════════════════════════════════════════════════════════════════════════
// t19-e12 (tag 26.19). Everything below covers branches the suite above did not
// reach: relocation and its refusals, the sweep edges, reading a message, the
// search round trip, session lifecycle, the offline seams, and the pure body
// extractor that feeds the sandboxed reader iframe.
// ═════════════════════════════════════════════════════════════════════════════

describe('mail slice — archive / trash / spam / move', () => {
  const cases = [
    ['archiveMessage', 'archive', 'Archived'],
    ['trashMessage', 'trash', 'Moved to Trash'],
    ['markSpam', 'junk', 'Marked as spam'],
  ] as const;

  for (const [method, role, label] of cases) {
    it(`${method} removes the row and offers an undo that puts it back where it was`, async () => {
      await withInbox([email('a'), email('b'), email('c')], async (mail, { jmap }) => {
        jmap.mockClear();
        await mail[method]('b');
        // Removed from the list optimistically, before any server confirmation.
        expect(mail.messages().map((m) => m.id)).toEqual(['a', 'c']);

        // The move names the destination mailbox, not just "somewhere else".
        const move = jmap.mock.calls
          .map((call) => call[0] as JmapRequest)
          .flatMap((body) => body.methodCalls)
          .find((c) => c[0] === 'Email/set');
        expect(JSON.stringify(move)).toContain(role);

        expect(mail.pendingUndo()?.label).toBe(label);
        await mail.undoNow();
        // Restored at its ORIGINAL index — an undo that appends would silently
        // reorder the list the user is looking at.
        expect(mail.messages().map((m) => m.id)).toEqual(['a', 'b', 'c']);
      });
    });
  }

  it('refuses and explains when the destination folder does not exist', async () => {
    for (const [method, missing, message] of [
      ['archiveMessage', 'archive', 'No Archive folder'],
      ['trashMessage', 'trash', 'No Trash folder'],
      ['markSpam', 'junk', 'No Spam folder'],
    ] as const) {
      await withBoxes([email('a')], withoutRoles(missing), async (mail, { toast }) => {
        await mail[method]('a');
        expect(toast).toHaveBeenCalledWith('error', message);
        // The message must still be there — a refusal that also loses the row
        // would be worse than the missing folder.
        expect(mail.messages().map((m) => m.id)).toEqual(['a']);
        expect(mail.pendingUndo()).toBeNull();
      });
    }
  });

  it('moveMessage names the target folder in its undo label', async () => {
    await withInbox([email('a')], async (mail) => {
      await mail.moveMessage('a', 'archive');
      expect(mail.messages()).toEqual([]);
      expect(mail.pendingUndo()?.label).toContain('Archive');
    });
  });

  it('moveMessage falls back to a generic label for an id it cannot name', async () => {
    // The destination id always comes from the rendered mailbox list, so an
    // unknown id is a programming error rather than a user path. What matters
    // is that it degrades to a generic label instead of rendering "undefined"
    // in the undo toast — and that the move is still undoable.
    await withInbox([email('a')], async (mail) => {
      await mail.moveMessage('a', 'no-such-box');
      expect(mail.pendingUndo()?.label).toBe('Moved to folder');
      await mail.undoNow();
      expect(mail.messages().map((m) => m.id)).toEqual(['a']);
    });
  });

  it('relocating an id that is not in the list is a no-op, not a crash', async () => {
    await withInbox([email('a')], async (mail) => {
      await mail.archiveMessage('ghost');
      expect(mail.messages().map((m) => m.id)).toEqual(['a']);
      expect(mail.pendingUndo()).toBeNull();
    });
  });
});

describe('mail slice — follow-ups', () => {
  it('setFollowUp lists the message under follow-ups and undo clears it', async () => {
    await withInbox([email('a'), email('b')], async (mail) => {
      await mail.setFollowUp('a', '2026-09-01T09:00:00Z');
      expect(mail.followUps().map((m) => m.id)).toEqual(['a']);
      expect(mail.pendingUndo()?.label).toBe('Follow-up set');
      await mail.undoNow();
      expect(mail.followUps()).toEqual([]);
    });
  });

  it('clearing a follow-up uses its own label, and undo restores the time', async () => {
    await withInbox([email('a', { followUpAt: '2026-09-01T09:00:00Z' })], async (mail) => {
      expect(mail.followUps().map((m) => m.id)).toEqual(['a']);
      await mail.setFollowUp('a', null);
      expect(mail.followUps()).toEqual([]);
      expect(mail.pendingUndo()?.label).toBe('Follow-up cleared');
      await mail.undoNow();
      expect(mail.followUps().map((m) => m.id)).toEqual(['a']);
    });
  });
});

describe('mail slice — sweep edges', () => {
  const day = 86_400_000;
  const at = (msAgo: number): string => new Date(Date.now() - msAgo).toISOString();
  const aged = [
    email('new', { from: [{ name: null, email: 'news@shop.example' }], receivedAt: at(1 * day) }),
    email('old', { from: [{ name: null, email: 'news@shop.example' }], receivedAt: at(60 * day) }),
  ];

  it('older-than keeps anything inside the window', async () => {
    await withInbox(aged, async (mail) => {
      expect(mail.sweepPreview('news@shop.example', 'older-than', 30).map((m) => m.id)).toEqual([
        'old',
      ]);
      // A window wider than the oldest message matches nothing.
      expect(mail.sweepPreview('news@shop.example', 'older-than', 365)).toEqual([]);
    });
  });

  it('older-than defaults to 30 days when no window is given', async () => {
    await withInbox(aged, async (mail) => {
      expect(mail.sweepPreview('news@shop.example', 'older-than').map((m) => m.id)).toEqual(['old']);
    });
  });

  it('matches the sender case-insensitively and ignores surrounding space', async () => {
    await withInbox(aged, async (mail) => {
      expect(mail.sweepPreview('  NEWS@Shop.Example  ', 'all')).toHaveLength(2);
    });
  });

  it('says so instead of silently doing nothing when nothing matches', async () => {
    await withInbox(aged, async (mail, { toast }) => {
      await mail.executeSweep('nobody@example.org', 'all');
      expect(toast).toHaveBeenCalledWith('info', 'Nothing to sweep');
      expect(mail.messages()).toHaveLength(2);
      expect(mail.pendingUndo()).toBeNull();
    });
  });

  it('refuses a sweep with no Trash folder rather than deleting outright', async () => {
    await withBoxes(aged, withoutRoles('trash'), async (mail, { toast }) => {
      await mail.executeSweep('news@shop.example', 'all');
      expect(toast).toHaveBeenCalledWith('error', 'No Trash folder');
      expect(mail.messages()).toHaveLength(2);
    });
  });

  it('undo restores swept messages at their original positions', async () => {
    const mixed = [
      email('keep1'),
      email('s1', { from: [{ name: null, email: 'news@shop.example' }] }),
      email('keep2'),
      email('s2', { from: [{ name: null, email: 'news@shop.example' }] }),
    ];
    await withInbox(mixed, async (mail) => {
      await mail.executeSweep('news@shop.example', 'all');
      expect(mail.messages().map((m) => m.id)).toEqual(['keep1', 'keep2']);
      await mail.undoNow();
      expect(mail.messages().map((m) => m.id)).toEqual(['keep1', 's1', 'keep2', 's2']);
    });
  });

  it('does not record the same blocked sender twice', async () => {
    await withInbox(aged, async (mail) => {
      await mail.executeSweep('news@shop.example', 'block');
      await mail.executeSweep('news@shop.example', 'block');
      expect(mail.blockedSenders().filter((s) => s === 'news@shop.example')).toHaveLength(1);
    });
  });
});

describe('mail slice — reading a message', () => {
  const withBody = email('a', {
    htmlBody: [part('1', 'text/html')],
    bodyValues: { '1': { value: '<p>hello</p>', isEncodingProblem: false, isTruncated: false } },
  } as Partial<Email>);

  it('opens a message and sanitizes its body before exposing it', async () => {
    await withInbox([withBody], async (mail, { client }) => {
      await mail.openMessage('a');
      expect(mail.openEmail()?.id).toBe('a');
      // The reader NEVER renders raw remote HTML: everything it shows has been
      // through the sanitizer seam.
      expect(client.sanitize).toHaveBeenCalledWith('<p>hello</p>');
      expect(mail.sanitizedHtml()).toBe('<p>hello</p>');
      expect(mail.readLoading()).toBe(false);
    });
  });

  it('closeMessage drops the open message and its sanitized body together', async () => {
    await withInbox([withBody], async (mail) => {
      await mail.openMessage('a');
      mail.closeMessage();
      expect(mail.openEmail()).toBeNull();
      // Leaving the sanitized HTML behind would flash the previous message into
      // the reader the next time it mounts.
      expect(mail.sanitizedHtml()).toBeNull();
    });
  });

  it('reads from the cached list without a fetch when offline', async () => {
    await withDeps([withBody], { online: () => false }, async (mail, { jmap, client }) => {
      jmap.mockClear();
      await mail.openMessage('a');
      expect(mail.openEmail()?.id).toBe('a');
      expect(jmap).not.toHaveBeenCalled();
      // The cached path skips the sanitizer seam too — it renders the local
      // extraction directly, which is worth knowing.
      expect(client.sanitize).not.toHaveBeenCalled();
      expect(mail.sanitizedHtml()).toContain('hello');
    });
  });

  it('offline read of an id that is not cached yields nothing, not a stale body', async () => {
    await withDeps([withBody], { online: () => false }, async (mail) => {
      await mail.openMessage('missing');
      expect(mail.openEmail()).toBeNull();
      expect(mail.sanitizedHtml()).toBeNull();
    });
  });

  it('selectMailbox clears the open message, the search and its results', async () => {
    await withInbox([withBody], async (mail) => {
      await mail.openMessage('a');
      await mail.searchMessages('hello');
      expect(mail.searchActive()).toBe(true);

      await mail.selectMailbox('archive');
      expect(mail.selectedMailboxId()).toBe('archive');
      expect(mail.openEmail()).toBeNull();
      expect(mail.sanitizedHtml()).toBeNull();
      expect(mail.searchActive()).toBe(false);
      expect(mail.search()).toBe('');
    });
  });
});

describe('mail slice — search', () => {
  it('runs a query, marks search active, and clearing restores the mailbox', async () => {
    await withInbox([email('a')], async (mail) => {
      await mail.searchMessages('subject:hi');
      expect(mail.search()).toBe('subject:hi');
      expect(mail.searchActive()).toBe(true);
      await mail.clearSearch();
      expect(mail.search()).toBe('');
      expect(mail.searchActive()).toBe(false);
    });
  });

  it('an empty query clears instead of searching for nothing', async () => {
    await withInbox([email('a')], async (mail) => {
      await mail.searchMessages('   ');
      expect(mail.searchActive()).toBe(false);
      expect(mail.search()).toBe('');
    });
  });

  it('adds the semantic flag only when the caller asks for it', async () => {
    await withInbox([email('a')], async (mail, { jmap }) => {
      jmap.mockClear();
      await mail.searchMessages('hi');
      expect(JSON.stringify(jmap.mock.calls)).not.toContain('semantic');

      jmap.mockClear();
      await mail.searchMessages('hi', { semantic: true });
      expect(JSON.stringify(jmap.mock.calls)).toContain('semantic');
    });
  });

  it('uses the cached offline index instead of the server when offline', async () => {
    const offlineHit = [email('cached')];
    const searchOffline = vi.fn(() => offlineHit);
    await withDeps([email('a')], { online: () => false, searchOffline }, async (mail, { jmap }) => {
      jmap.mockClear();
      await mail.searchMessages('anything');
      expect(searchOffline).toHaveBeenCalledWith({ text: 'anything' });
      expect(mail.messages().map((m) => m.id)).toEqual(['cached']);
      expect(jmap).not.toHaveBeenCalled();
      expect(mail.searchActive()).toBe(true);
    });
  });

  it('refreshCurrentMailbox reloads in place, keeping the open message', async () => {
    await withInbox([email('a')], async (mail, { setSeed }) => {
      await mail.openMessage('a');
      setSeed([email('a'), email('b')]);
      await mail.refreshCurrentMailbox();
      expect(mail.messages().map((m) => m.id)).toEqual(['a', 'b']);
      // Unlike selectMailbox, a background refetch must not close the reader.
      expect(mail.openEmail()?.id).toBe('a');
    });
  });
});

describe('mail slice — offline mutation queue', () => {
  it('queues a relocation instead of calling JMAP, and still updates the list', async () => {
    const enqueueOffline = vi.fn(async () => undefined);
    await withDeps(
      [email('a'), email('b')],
      { online: () => false, enqueueOffline },
      async (mail, { jmap }) => {
        jmap.mockClear();
        await mail.archiveMessage('a');
        expect(jmap).not.toHaveBeenCalled();
        expect(enqueueOffline).toHaveBeenCalledWith('move', {
          accountId: 'acct1',
          emailId: 'a',
          mailboxIds: { archive: true },
        });
        // The row leaves the list immediately — the queue is a transport detail,
        // not something the user should have to wait on.
        expect(mail.messages().map((m) => m.id)).toEqual(['b']);
      },
    );
  });

  it('queues a send and says so rather than reporting it sent', async () => {
    const enqueueOffline = vi.fn(async () => undefined);
    await withDeps([email('a')], { online: () => false, enqueueOffline }, async (mail, { toast }) => {
      await mail.sendMessage({ to: 'you@example.org', subject: 'Hi', htmlBody: '<p>x</p>' });
      expect(enqueueOffline).toHaveBeenCalledWith(
        'send',
        expect.objectContaining({ accountId: 'acct1' }),
      );
      expect(toast).toHaveBeenCalledWith('info', 'Queued — will send when back online');
      // No undo window: there is nothing to cancel yet.
      expect(mail.pendingUndo()).toBeNull();
    });
  });

  it('tells peer tabs to refetch after a send', async () => {
    const broadcastChange = vi.fn();
    await withDeps([email('a')], { broadcastChange }, async (mail) => {
      await mail.sendMessage({ to: 'you@example.org', subject: 'Hi', htmlBody: '<p>x</p>' });
      expect(broadcastChange).toHaveBeenCalled();
    });
  });
});

describe('mail slice — session lifecycle', () => {
  it('logout clears every piece of account state', async () => {
    await withInbox([email('a')], async (mail, { client }) => {
      await mail.openMessage('a');
      await mail.pinMessage('a', true);
      await mail.logout();

      expect(client.logout).toHaveBeenCalled();
      expect(mail.me()).toBeNull();
      expect(mail.mailboxes()).toEqual([]);
      expect(mail.messages()).toEqual([]);
      expect(mail.selectedMailboxId()).toBeNull();
      expect(mail.openEmail()).toBeNull();
      expect(mail.sanitizedHtml()).toBeNull();
      // A pending undo would otherwise fire a mutation against the account the
      // user just left.
      expect(mail.pendingUndo()).toBeNull();
    });
  });

  it('dismissUndo drops the action without running it', async () => {
    await withInbox([email('a'), email('b')], async (mail) => {
      await mail.archiveMessage('a');
      mail.dismissUndo();
      expect(mail.pendingUndo()).toBeNull();
      await mail.undoNow(); // no-op — nothing pending
      expect(mail.messages().map((m) => m.id)).toEqual(['b']);
    });
  });
});

describe('extractHtmlBody', () => {
  // Pure, and load-bearing: its return value becomes the reader iframe's
  // srcdoc. The escaping branch is the one that must not regress.
  const base = email('x');

  it('prefers the HTML part when the message has one', () => {
    expect(
      extractHtmlBody({
        ...base,
        htmlBody: [part('1', 'text/html')],
        bodyValues: { '1': { value: '<p>hi</p>', isEncodingProblem: false, isTruncated: false } },
      } as Email),
    ).toBe('<p>hi</p>');
  });

  it('falls back to the text part, escaped inside a pre block', () => {
    expect(
      extractHtmlBody({
        ...base,
        textBody: [part('1', 'text/plain')],
        bodyValues: {
          '1': { value: 'plain & simple', isEncodingProblem: false, isTruncated: false },
        },
      } as Email),
    ).toBe('<pre>plain &amp; simple</pre>');
  });

  it('escapes markup in the text part so it cannot become live HTML', () => {
    // A text/plain body containing markup must render as VISIBLE text. Without
    // the escape this string reaches the iframe srcdoc as a real element.
    const out = extractHtmlBody({
      ...base,
      textBody: [part('1', 'text/plain')],
      bodyValues: {
        '1': {
          value: '<img src=x onerror=alert(1)>',
          isEncodingProblem: false,
          isTruncated: false,
        },
      },
    } as Email);
    expect(out).toBe('<pre>&lt;img src=x onerror=alert(1)&gt;</pre>');
    expect(out).not.toContain('<img');
  });

  it('escapes the preview fallback too, when there is no body at all', () => {
    expect(extractHtmlBody({ ...base, preview: 'a < b & c' } as Email)).toBe(
      '<pre>a &lt; b &amp; c</pre>',
    );
  });

  it('skips an HTML part whose body value was not fetched', () => {
    // `Email/get` can return the part list without the value (truncation, a
    // partial fetch). Returning `undefined` here would render "undefined".
    expect(
      extractHtmlBody({
        ...base,
        htmlBody: [part('1', 'text/html')],
        bodyValues: {},
        preview: 'fallback',
      } as Email),
    ).toBe('<pre>fallback</pre>');
  });
});
