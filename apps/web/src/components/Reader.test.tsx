import { describe, it, expect, beforeEach, afterEach, vi } from 'vitest';
import { cleanup, render, screen, fireEvent, waitFor, within } from '@solidjs/testing-library';
import { Suspense } from 'solid-js';
import { Reader } from './Reader.tsx';
import { makeClient, mkEmail } from './appHarness.tsx';
import { createAppState, type AppState } from '../state/store.ts';
import { AppContext } from '../state/context.ts';
import type { ComposeInitial } from './compose/reply.ts';
import type { Email, Identity, JmapRequest, JmapResponse } from '../api/jmap-types.ts';

// Reply / Reply all / Forward from the reader toolbar: what the composer is
// handed. The reader's own requests outside the app client (security verdict,
// image grants, the raw-message download) go through `fetch`, stubbed below;
// only the download answers.

const RAW_HEADERS =
  'Message-ID: <orig@example.org>\r\nReferences: <root@example.org>\r\nSubject: Plans\r\n\r\nbody\r\n';

/** The attribution date, formatted the way the reader formats it. */
const shownDate = (iso: string): string =>
  new Date(iso).toLocaleString(undefined, { dateStyle: 'medium', timeStyle: 'short' });

const IDENTITIES: Identity[] = [
  { id: 'id1', name: 'Me', email: 'me@example.org', replyTo: null, signatureHtml: null, signatureText: null, sentMailboxId: null },
  { id: 'id2', name: 'Work', email: 'alias@corp.example', replyTo: null, signatureHtml: null, signatureText: null, sentMailboxId: null },
];

const ORIGINAL: Email = mkEmail('m1', {
  blobId: 'm1',
  from: [{ name: 'Alice Example', email: 'alice@example.org' }],
  to: [
    { name: 'Me', email: 'me@example.org' },
    { name: 'Bob', email: 'bob@example.org' },
  ],
  cc: [
    { name: null, email: 'carol@example.org' },
    { name: null, email: 'alias@corp.example' },
  ],
  subject: 'Re: Plans',
  sentAt: '2026-03-04T10:30:00Z',
  receivedAt: '2026-03-04T10:31:00Z',
  htmlBody: [{ partId: '1', blobId: null, size: 0, type: 'text/html' }],
  bodyValues: {
    '1': {
      value:
        '<p>See you <b>there</b>.</p><img src="http://tracker.example/pixel.gif" width="1" height="1">' +
        '<p style="background:url(http://tracker.example/bg.png)">Second paragraph</p>',
    },
  },
});

let downloads: string[] = [];
let downloadStatus = 200;

async function openReader(
  email: Email,
  opts: { identities?: Identity[]; ownKey?: boolean } = {},
): Promise<{ app: AppState; onCompose: ReturnType<typeof vi.fn<(i: ComposeInitial) => void>> }> {
  const client = makeClient({ emails: [email], identities: opts.identities ?? IDENTITIES });
  if (opts.ownKey === true) {
    const base = client.jmap;
    client.jmap = vi.fn(async (body: JmapRequest, o?: { signal?: AbortSignal }): Promise<JmapResponse> => {
      if (!body.methodCalls.some((c) => c[0].startsWith('CryptoKey/'))) return base(body, o);
      const key = { id: 'k1', kind: 'pgp', isOwn: true, addresses: ['me@example.org'], encryptedPrivateBackup: 'BUNDLE', publicKeyArmored: null };
      return {
        methodResponses: body.methodCalls.map((c) => [c[0], { accountId: 'acct1', ids: ['k1'], list: [key], notFound: [] }, c[2]]),
        sessionState: 's',
      };
    }) as typeof client.jmap;
  }
  const app = createAppState(client);
  const onCompose = vi.fn<(i: ComposeInitial) => void>();
  render(() => (
    <AppContext.Provider value={app}>
      <Reader onCompose={onCompose} />
    </AppContext.Provider>
  ));
  await app.login({ jmapUrl: 'x', username: 'me@example.org', password: 'p' });
  await app.loadIdentities();
  await app.openMessage(email.id);
  await screen.findByRole('toolbar', { name: 'Message actions' });
  return { app, onCompose };
}

async function click(name: 'Reply' | 'Reply all' | 'Forward', onCompose: ReturnType<typeof vi.fn>): Promise<ComposeInitial> {
  const toolbar = screen.getByRole('toolbar', { name: 'Message actions' });
  fireEvent.click(within(toolbar).getByRole('button', { name }));
  await waitFor(() => expect(onCompose).toHaveBeenCalledTimes(1));
  return onCompose.mock.calls[0]![0] as ComposeInitial;
}

describe('Reader: opening a message suspends nothing around it', () => {
  afterEach(() => vi.unstubAllGlobals());

  it('the list beside the reader stays the same mounted node while the verdict and grants load', async () => {
    // Every request the reader makes outside the app client is held open, so
    // its resources are pending for as long as this test looks.
    const held: Array<(r: Response) => void> = [];
    vi.stubGlobal(
      'fetch',
      vi.fn(() => new Promise<Response>((resolve) => held.push(resolve))),
    );
    let fallbacks = 0;
    const Fallback = () => {
      fallbacks += 1;
      return <p data-testid="screen-pending" />;
    };
    const client = makeClient({ emails: [ORIGINAL], identities: IDENTITIES });
    const app = createAppState(client);
    // The mailbox screen as App.tsx mounts it: one Suspense boundary (LazyRoute)
    // around the list and the reader.
    render(() => (
      <AppContext.Provider value={app}>
        <Suspense fallback={<Fallback />}>
          <ul data-testid="list-stand-in">
            <li>row</li>
          </ul>
          <Reader />
        </Suspense>
      </AppContext.Provider>
    ));
    await app.login({ jmapUrl: 'x', username: 'me@example.org', password: 'p' });
    const list = screen.getByTestId('list-stand-in');

    await app.openMessage('m1');
    await screen.findByRole('toolbar', { name: 'Message actions' });
    // Precondition: the reader did start its own requests and they are still out.
    expect(held.length).toBeGreaterThan(0);
    expect(fallbacks).toBe(0);
    expect(screen.queryByTestId('screen-pending')).toBeNull();
    expect(screen.getByTestId('list-stand-in')).toBe(list);
    expect(list.isConnected).toBe(true);

    // And nothing is swapped when they come back.
    for (const resolve of held.splice(0)) resolve(new Response('[]', { status: 200 }));
    await waitFor(() => expect(screen.getByTitle('Message body')).toBeInTheDocument());
    expect(fallbacks).toBe(0);
    expect(screen.getByTestId('list-stand-in')).toBe(list);
  });
});

describe('Reader: Reply, Reply all, Forward', () => {
  beforeEach(() => {
    localStorage.clear();
    downloads = [];
    downloadStatus = 200;
    vi.stubGlobal(
      'fetch',
      vi.fn(async (input: RequestInfo | URL) => {
        const url = String(input);
        if (url.startsWith('/d')) {
          downloads.push(url);
          return new Response(downloadStatus === 200 ? RAW_HEADERS : 'no', { status: downloadStatus });
        }
        return new Response('{}', { status: 404 });
      }),
    );
  });
  afterEach(() => vi.unstubAllGlobals());

  it('shows the three actions first in the toolbar, as labelled buttons, and the Cc line', async () => {
    await openReader(ORIGINAL);
    const buttons = within(screen.getByRole('toolbar', { name: 'Message actions' })).getAllByRole('button');
    expect(buttons.slice(0, 3).map((b) => b.textContent)).toEqual(['Reply', 'Reply all', 'Forward']);
    expect(buttons.slice(0, 3).every((b) => b.tabIndex === 0 && !(b as HTMLButtonElement).disabled)).toBe(true);
    expect(screen.getByText(/^Cc: /)).toHaveTextContent('carol@example.org, alias@corp.example');
  });

  it('Reply: the author only, one Re:, the thread ids from the raw message, the sanitised body quoted', async () => {
    const { onCompose } = await openReader(ORIGINAL);
    const initial = await click('Reply', onCompose);
    expect(downloads).toHaveLength(1);
    expect(initial).toEqual({
      mode: 'reply',
      to: 'Alice Example <alice@example.org>',
      cc: '',
      subject: 'Re: Plans',
      bodyHtml:
        `<p></p><p>On ${shownDate('2026-03-04T10:30:00Z')}, Alice Example wrote:</p>` +
        '<blockquote><p>See you <strong>there</strong>.</p><p>Second paragraph</p></blockquote>',
      bodyText:
        `\n\nOn ${shownDate('2026-03-04T10:30:00Z')}, Alice Example wrote:\n` +
        '> See you there.\n>\n> Second paragraph',
      inReplyTo: ['orig@example.org'],
      references: ['root@example.org', 'orig@example.org'],
      attachments: [],
      quotesDecrypted: false,
      source: { emailId: 'm1', keyword: '$answered' },
    });
  });

  it('Reply all: the other To and Cc recipients, without any of the user’s own addresses', async () => {
    const { onCompose } = await openReader(ORIGINAL);
    const initial = await click('Reply all', onCompose);
    expect({ to: initial.to, cc: initial.cc }).toEqual({
      to: 'Alice Example <alice@example.org>, Bob <bob@example.org>',
      cc: 'carol@example.org',
    });
    expect(`${initial.to} ${initial.cc}`).not.toMatch(/me@example\.org|alias@corp\.example/);
  });

  it('Reply all honours Reply-To', async () => {
    const { onCompose } = await openReader({
      ...ORIGINAL,
      replyTo: [{ name: 'The List', email: 'list@example.org' }],
    });
    const initial = await click('Reply all', onCompose);
    expect(initial.to).toBe('The List <list@example.org>, Bob <bob@example.org>');
  });

  it('uses the ids Email/get returned, when it returns them, without downloading the message', async () => {
    const { onCompose } = await openReader({
      ...ORIGINAL,
      messageId: ['from-get@example.org'],
      references: ['r1@example.org'],
    });
    const initial = await click('Reply', onCompose);
    expect(downloads).toEqual([]);
    expect({ inReplyTo: initial.inReplyTo, references: initial.references }).toEqual({
      inReplyTo: ['from-get@example.org'],
      references: ['r1@example.org', 'from-get@example.org'],
    });
  });

  it('takes null from Email/get as the answer: no download, no thread ids, and it says so', async () => {
    const { app, onCompose } = await openReader({ ...ORIGINAL, messageId: null, inReplyTo: null, references: null });
    const initial = await click('Reply', onCompose);
    expect(downloads).toEqual([]);
    expect({ inReplyTo: initial.inReplyTo, references: initial.references }).toEqual({ inReplyTo: [], references: [] });
    expect(app.toast()?.message).toBe(
      'The ID of the original message could not be read. This reply will not be threaded with it.',
    );
  });

  it('with no References, a single In-Reply-To id is the chain and several are not', async () => {
    const one = await openReader({ ...ORIGINAL, messageId: ['m@x'], references: null, inReplyTo: ['p@x'] });
    expect((await click('Reply', one.onCompose)).references).toEqual(['p@x', 'm@x']);
    cleanup();
    const two = await openReader({ ...ORIGINAL, messageId: ['m@x'], references: null, inReplyTo: ['p@x', 'q@x'] });
    expect((await click('Reply', two.onCompose)).references).toEqual(['m@x']);
  });

  it('when the original id cannot be read, the reply opens without thread ids and says so', async () => {
    downloadStatus = 500;
    const { app, onCompose } = await openReader(ORIGINAL);
    const initial = await click('Reply', onCompose);
    expect({ inReplyTo: initial.inReplyTo, references: initial.references }).toEqual({ inReplyTo: [], references: [] });
    expect(app.toast()?.message).toBe(
      'The ID of the original message could not be read. This reply will not be threaded with it.',
    );
  });

  it('Forward: no recipients, one Fwd:, the header block, the attachments by blob id, no thread ids', async () => {
    const withFiles = {
      ...ORIGINAL,
      subject: 'Fwd: Plans',
      attachments: [
        { partId: '2', blobId: 'm1.2', size: 1234, type: 'application/pdf', name: 'plan.pdf' },
        { partId: '3', blobId: 'm1.3', size: 9, type: '', name: null },
        { partId: '4', blobId: null, size: 1, type: 'text/plain', name: 'no-blob.txt' },
      ],
    } as Email;
    const { onCompose } = await openReader(withFiles);
    const initial = await click('Forward', onCompose);
    expect(downloads).toEqual([]);
    const header =
      '---------- Forwarded message ----------<br>' +
      'From: Alice Example &lt;alice@example.org&gt;<br>' +
      `Date: ${shownDate('2026-03-04T10:30:00Z')}<br>` +
      'Subject: Fwd: Plans<br>' +
      'To: Me &lt;me@example.org&gt;, Bob &lt;bob@example.org&gt;<br>' +
      'Cc: carol@example.org, alias@corp.example';
    expect(initial).toEqual({
      mode: 'forward',
      to: '',
      cc: '',
      subject: 'Fwd: Plans',
      bodyHtml: `<p></p><p>${header}</p><p>See you <strong>there</strong>.</p><p>Second paragraph</p>`,
      bodyText:
        '\n\n---------- Forwarded message ----------\n' +
        'From: Alice Example <alice@example.org>\n' +
        `Date: ${shownDate('2026-03-04T10:30:00Z')}\n` +
        'Subject: Fwd: Plans\n' +
        'To: Me <me@example.org>, Bob <bob@example.org>\n' +
        'Cc: carol@example.org, alias@corp.example\n\n' +
        'See you there.\n\nSecond paragraph',
      inReplyTo: [],
      references: [],
      attachments: [
        { name: 'plan.pdf', blobId: 'm1.2', size: 1234, contentType: 'application/pdf' },
        { name: '(unnamed)', blobId: 'm1.3', size: 9, contentType: null },
      ],
      quotesDecrypted: false,
      source: { emailId: 'm1', keyword: '$forwarded' },
    });
  });

  it('an encrypted message that has not been decrypted is quoted as nothing', async () => {
    const armored = {
      ...ORIGINAL,
      bodyValues: { '1': { value: '<pre>-----BEGIN PGP MESSAGE-----\nabc\n-----END PGP MESSAGE-----</pre>' } },
    };
    const { onCompose } = await openReader(armored);
    expect(screen.getByTestId('reader-decrypt')).toBeInTheDocument();
    const initial = await click('Reply', onCompose);
    expect({ bodyHtml: initial.bodyHtml, bodyText: initial.bodyText, quotesDecrypted: initial.quotesDecrypted }).toEqual({
      bodyHtml: '',
      bodyText: '',
      quotesDecrypted: false,
    });
  });

  it('once decrypted, the quote is the decrypted text and is marked as such', async () => {
    const armored = {
      ...ORIGINAL,
      bodyValues: { '1': { value: '<pre>-----BEGIN PGP MESSAGE-----\nabc\n-----END PGP MESSAGE-----</pre>' } },
    };
    const { app, onCompose } = await openReader(armored, { ownKey: true });
    await app.loadKeys();
    fireEvent.input(screen.getByTestId('decrypt-passphrase'), { target: { value: 'pw' } });
    fireEvent.click(screen.getByTestId('decrypt-submit'));
    await waitFor(() => expect(screen.queryByTestId('reader-decrypt')).toBeNull());

    const initial = await click('Reply', onCompose);
    expect(initial.quotesDecrypted).toBe(true);
    expect(initial.bodyHtml).toBe(
      `<p></p><p>On ${shownDate('2026-03-04T10:30:00Z')}, Alice Example wrote:</p>` +
        '<blockquote><p>STUB decrypted body</p></blockquote>',
    );
    expect(initial.bodyHtml).not.toContain('BEGIN PGP MESSAGE');
  });
});
