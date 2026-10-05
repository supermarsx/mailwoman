import { describe, it, expect, beforeAll, beforeEach, vi } from 'vitest';
import { cleanup, render, screen, fireEvent, waitFor } from '@solidjs/testing-library';
import { onMount, type JSX } from 'solid-js';
import { Compose } from './Compose.tsx';
import { makeClient } from './appHarness.tsx';
import { createAppState } from '../state/store.ts';
import { AppContext } from '../state/context.ts';
import { listDrafts } from './compose/drafts-store.ts';
import type { ComposeInitial } from './compose/reply.ts';
import type { ComposeCryptoState } from './compose-crypto.tsx';
import type { JmapRequest } from '../api/jmap-types.ts';

// The crypto panel is replaced by a stand-in that reports whatever state the
// test put in `cryptoState`, once, when it mounts. The real panel needs key
// lookups and a worker to reach "encryption on, ciphertext ready"; what these
// tests are about is what the composer does with that state.
let cryptoState: ComposeCryptoState | null = null;
vi.mock('./compose-crypto.tsx', () => ({
  ComposeCrypto: (props: { onChange?: (s: ComposeCryptoState) => void }): JSX.Element => {
    onMount(() => {
      if (cryptoState !== null) props.onChange?.(cryptoState);
    });
    return null as unknown as JSX.Element;
  },
}));

const ENCRYPTING = {
  encrypt: true,
  sign: false,
  protectSubject: false,
  capability: 'e2ee',
  canSend: true,
  encryptedDraft: { armoredCiphertext: '-----BEGIN PGP MESSAGE-----\nCIPHER\n-----END PGP MESSAGE-----', encryptedSubjectApplied: false },
  verdicts: [],
  recipients: [],
} as unknown as ComposeCryptoState;

function reply(over: Partial<ComposeInitial> = {}): ComposeInitial {
  return {
    mode: 'reply-all',
    to: 'Alice <alice@example.org>',
    cc: 'bob@example.org',
    subject: 'Re: Plans',
    bodyHtml: '<p></p><p>On 1 Jan 2026, Alice wrote:</p><blockquote><p>original text</p></blockquote>',
    bodyText: '\n\nOn 1 Jan 2026, Alice wrote:\n> original text',
    inReplyTo: ['orig@example.org'],
    references: ['root@example.org', 'orig@example.org'],
    attachments: [],
    quotesDecrypted: false,
    ...over,
  };
}

async function open(initial?: ComposeInitial) {
  const client = makeClient();
  const app = createAppState(client);
  const onClose = vi.fn();
  render(() => (
    <AppContext.Provider value={app}>
      <Compose onClose={onClose} {...(initial !== undefined ? { initial } : {})} />
    </AppContext.Provider>
  ));
  await app.login({ jmapUrl: 'x', username: 'me@example.org', password: 'p' });
  return { client, app, onClose };
}

function sends(client: ReturnType<typeof makeClient>): JmapRequest[] {
  return vi
    .mocked(client.jmap)
    .mock.calls.map((c) => c[0])
    .filter((r) => r.methodCalls.some((m) => m[0] === 'EmailSubmission/set' && 'create' in m[1]));
}

/** The whole compose+submit request for the harness account (no Drafts or
 *  Sent folder: the draft is held in the inbox and nothing is moved). */
function expectedRequest(draft: Record<string, unknown>, rcptTo: string[]): JmapRequest['methodCalls'] {
  return [
    [
      'Email/set',
      {
        accountId: 'acct1',
        create: {
          draft: {
            mailboxIds: { inbox: true },
            keywords: { $draft: true, $seen: true },
            from: [{ name: null, email: 'me@example.org' }],
            ...draft,
            htmlBody: [{ partId: 'body', type: 'text/html' }],
          },
        },
      },
      'set',
    ],
    [
      'EmailSubmission/set',
      {
        accountId: 'acct1',
        create: {
          send: {
            emailId: '#draft',
            envelope: { mailFrom: { email: 'me@example.org' }, rcptTo: rcptTo.map((email) => ({ email })) },
            mailwomanHoldSeconds: 10,
          },
        },
      },
      'submit',
    ],
  ];
}

describe('Compose opened as a reply or a forward', () => {
  beforeAll(async () => {
    // See Compose.test.tsx: pay for the editor chunk outside a `findBy` budget.
    await import('./compose/RichTextEditor.tsx');
  }, 60_000);

  beforeEach(() => {
    localStorage.clear();
    cryptoState = null;
  });

  it('a new message has no Cc or Bcc field until asked, and sends neither', async () => {
    const { client, onClose } = await open();
    expect(screen.queryByLabelText('Cc')).toBeNull();
    expect(screen.queryByLabelText('Bcc')).toBeNull();
    expect(screen.getByRole('heading', { name: 'New message' })).toBeInTheDocument();
    fireEvent.click(screen.getByTestId('format-toggle')); // plain text
    fireEvent.input(screen.getByLabelText('To'), { target: { value: 'you@example.org' } });
    fireEvent.input(screen.getByLabelText('Subject'), { target: { value: 'Hi' } });
    fireEvent.input(screen.getByLabelText('Body'), { target: { value: 'b' } });
    fireEvent.click(screen.getByRole('button', { name: 'Send' }));
    await waitFor(() => expect(onClose).toHaveBeenCalledTimes(1));
    expect(sends(client)).toHaveLength(1);
    expect(sends(client)[0]!.methodCalls).toEqual(
      expectedRequest(
        { to: [{ name: null, email: 'you@example.org' }], subject: 'Hi', bodyValues: { body: { value: '<p>b</p>' } } },
        ['you@example.org'],
      ),
    );
  });

  it('typed Cc and Bcc go on the draft and into the envelope', async () => {
    const { client, onClose } = await open();
    fireEvent.click(screen.getByTestId('compose-show-cc-bcc'));
    fireEvent.click(screen.getByTestId('format-toggle'));
    fireEvent.input(screen.getByLabelText('To'), { target: { value: 'you@example.org' } });
    fireEvent.input(screen.getByLabelText('Cc'), { target: { value: 'Carol <carol@example.org>' } });
    fireEvent.input(screen.getByLabelText('Bcc'), { target: { value: 'dave@example.org; you@example.org' } });
    fireEvent.input(screen.getByLabelText('Subject'), { target: { value: 'Hi' } });
    fireEvent.input(screen.getByLabelText('Body'), { target: { value: 'b' } });
    fireEvent.click(screen.getByRole('button', { name: 'Send' }));
    await waitFor(() => expect(onClose).toHaveBeenCalledTimes(1));
    expect(sends(client)[0]!.methodCalls).toEqual(
      expectedRequest(
        {
          to: [{ name: null, email: 'you@example.org' }],
          cc: [{ name: 'Carol', email: 'carol@example.org' }],
          bcc: [
            { name: null, email: 'dave@example.org' },
            { name: null, email: 'you@example.org' },
          ],
          subject: 'Hi',
          bodyValues: { body: { value: '<p>b</p>' } },
        },
        ['you@example.org', 'carol@example.org', 'dave@example.org'],
      ),
    );
  });

  it('a Bcc-only message is sent: To is not required', async () => {
    const { client, onClose } = await open();
    fireEvent.click(screen.getByTestId('compose-show-cc-bcc'));
    fireEvent.click(screen.getByTestId('format-toggle'));
    fireEvent.input(screen.getByLabelText('Bcc'), { target: { value: 'dave@example.org' } });
    fireEvent.input(screen.getByLabelText('Body'), { target: { value: 'b' } });
    fireEvent.click(screen.getByRole('button', { name: 'Send' }));
    await waitFor(() => expect(onClose).toHaveBeenCalledTimes(1));
    expect(sends(client)[0]!.methodCalls).toEqual(
      expectedRequest(
        { to: [], bcc: [{ name: null, email: 'dave@example.org' }], subject: '', bodyValues: { body: { value: '<p>b</p>' } } },
        ['dave@example.org'],
      ),
    );
  });

  it('with no recipient in any field nothing is sent and the composer says why', async () => {
    const { client, onClose } = await open();
    fireEvent.click(screen.getByRole('button', { name: 'Send' }));
    expect(await screen.findByRole('alert')).toHaveTextContent('Add at least one recipient.');
    expect(sends(client)).toHaveLength(0);
    expect(onClose).not.toHaveBeenCalled();
  });

  it('a reply-all opens with its recipients, subject and quote, and sends them with the thread ids', async () => {
    const { client, onClose } = await open(reply());
    expect(screen.getByRole('heading', { name: 'Reply all' })).toBeInTheDocument();
    expect((screen.getByLabelText('To') as HTMLInputElement).value).toBe('Alice <alice@example.org>');
    expect((screen.getByLabelText('Cc') as HTMLInputElement).value).toBe('bob@example.org');
    expect((screen.getByLabelText('Bcc') as HTMLInputElement).value).toBe('');
    expect((screen.getByLabelText('Subject') as HTMLInputElement).value).toBe('Re: Plans');
    const editor = await screen.findByTestId('compose-richtext');
    expect(editor.querySelector('blockquote')?.textContent).toBe('original text');

    fireEvent.click(screen.getByRole('button', { name: 'Send' }));
    await waitFor(() => expect(onClose).toHaveBeenCalledTimes(1));
    expect(sends(client)).toHaveLength(1);
    expect(sends(client)[0]!.methodCalls).toEqual(
      expectedRequest(
        {
          to: [{ name: 'Alice', email: 'alice@example.org' }],
          cc: [{ name: null, email: 'bob@example.org' }],
          inReplyTo: ['orig@example.org'],
          references: ['root@example.org', 'orig@example.org'],
          subject: 'Re: Plans',
          bodyValues: {
            body: {
              value: '<p></p><p>On 1 Jan 2026, Alice wrote:</p><blockquote><p>original text</p></blockquote>',
            },
          },
        },
        ['alice@example.org', 'bob@example.org'],
      ),
    );
  });

  it('a sent reply marks the message it answers; a new message marks nothing', async () => {
    const marks = (client: ReturnType<typeof makeClient>): unknown[] =>
      vi
        .mocked(client.jmap)
        .mock.calls.map((c) => c[0].methodCalls)
        .filter((calls) => calls.length === 1 && calls[0]![0] === 'Email/set' && 'update' in calls[0]![1]);

    const replied = await open(reply({ cc: '', source: { emailId: 'm1', keyword: '$answered' } }));
    await screen.findByTestId('compose-richtext');
    // Precondition: nothing is marked before the send.
    expect(marks(replied.client)).toEqual([]);
    fireEvent.click(screen.getByRole('button', { name: 'Send' }));
    await waitFor(() => expect(replied.onClose).toHaveBeenCalledTimes(1));
    expect(marks(replied.client)).toEqual([
      [['Email/set', { accountId: 'acct1', update: { m1: { 'keywords/$answered': true } } }, 'set']],
    ]);
    cleanup();

    const fresh = await open();
    fireEvent.click(screen.getByTestId('format-toggle'));
    fireEvent.input(screen.getByLabelText('To'), { target: { value: 'you@example.org' } });
    fireEvent.click(screen.getByRole('button', { name: 'Send' }));
    await waitFor(() => expect(fresh.onClose).toHaveBeenCalledTimes(1));
    expect(marks(fresh.client)).toEqual([]);
  });

  it('seeded HTML goes through the schema again in the editor: an image in it is not sent', async () => {
    const { client, onClose } = await open(
      reply({
        cc: '',
        bodyHtml: '<p>x<img src="http://tracker.example/p.gif" onerror="steal()"></p><script>alert(1)</script>',
      }),
    );
    await screen.findByTestId('compose-richtext');
    fireEvent.click(screen.getByRole('button', { name: 'Send' }));
    await waitFor(() => expect(onClose).toHaveBeenCalledTimes(1));
    const create = sends(client)[0]!.methodCalls[0]![1]['create'] as Record<string, Record<string, unknown>>;
    expect(create['draft']!['bodyValues']).toEqual({ body: { value: '<p>x</p>' } });
  });

  it('a forward opens with the original attachments and sends them by blob id, with no thread ids', async () => {
    const { client, onClose } = await open(
      reply({
        mode: 'forward',
        to: '',
        cc: '',
        subject: 'Fwd: Plans',
        inReplyTo: [],
        references: [],
        bodyHtml: '<p></p><p>---------- Forwarded message ----------</p><p>original text</p>',
        attachments: [
          { name: 'plan.pdf', blobId: 'm1.2', size: 1234, contentType: 'application/pdf' },
          { name: 'notes', blobId: 'm1.3', size: 0, contentType: null },
        ],
      }),
    );
    expect(screen.getByRole('heading', { name: 'Forward' })).toBeInTheDocument();
    expect(screen.getByTestId('compose-attachments')).toHaveTextContent('plan.pdf');
    await screen.findByTestId('compose-richtext');
    fireEvent.input(screen.getByLabelText('To'), { target: { value: 'carol@example.org' } });
    fireEvent.click(screen.getByRole('button', { name: 'Send' }));
    await waitFor(() => expect(onClose).toHaveBeenCalledTimes(1));
    expect(sends(client)[0]!.methodCalls).toEqual(
      expectedRequest(
        {
          to: [{ name: null, email: 'carol@example.org' }],
          subject: 'Fwd: Plans',
          bodyValues: {
            body: { value: '<p></p><p>---------- Forwarded message ----------</p><p>original text</p>' },
          },
          attachments: [
            { blobId: 'm1.2', name: 'plan.pdf', type: 'application/pdf', size: 1234, disposition: 'attachment' },
            { blobId: 'm1.3', name: 'notes', type: 'application/octet-stream', disposition: 'attachment' },
          ],
        },
        ['carol@example.org'],
      ),
    );
  });

  it('auto-saves a reply with its Cc and thread ids, and resuming it brings them back', async () => {
    vi.useFakeTimers();
    try {
      await open(reply());
      await vi.advanceTimersByTimeAsync(900);
    } finally {
      vi.useRealTimers();
    }
    const saved = listDrafts();
    expect(saved).toHaveLength(1);
    expect(saved[0]).toMatchObject({
      to: 'Alice <alice@example.org>',
      cc: 'bob@example.org',
      inReplyTo: ['orig@example.org'],
      references: ['root@example.org', 'orig@example.org'],
      subject: 'Re: Plans',
    });
  });
});

describe('Compose quoting the decrypted text of an encrypted message', () => {
  beforeAll(async () => {
    await import('./compose/RichTextEditor.tsx');
  }, 60_000);

  beforeEach(() => {
    localStorage.clear();
    cryptoState = null;
  });

  it('is never written to the local drafts store', async () => {
    vi.useFakeTimers();
    try {
      await open(reply({ quotesDecrypted: true }));
      await vi.advanceTimersByTimeAsync(5_000);
    } finally {
      vi.useRealTimers();
    }
    expect(listDrafts()).toEqual([]);
    expect(localStorage.getItem('mw.compose.drafts.v1')).toBeNull();
  });

  it('asks before sending unencrypted; nothing is sent until the user confirms', async () => {
    const { client, onClose } = await open(reply({ quotesDecrypted: true, cc: '' }));
    await screen.findByTestId('compose-richtext');
    fireEvent.click(screen.getByRole('button', { name: 'Send' }));

    const dialog = await screen.findByRole('alertdialog', { name: 'Send decrypted text without encryption?' });
    expect(dialog).toHaveTextContent('the quoted text would be sent unprotected');
    expect(sends(client)).toHaveLength(0);
    expect(onClose).not.toHaveBeenCalled();

    // Going back closes the question and still sends nothing; asking again asks again.
    fireEvent.click(screen.getByRole('button', { name: 'Go back' }));
    expect(screen.queryByRole('alertdialog')).toBeNull();
    expect(sends(client)).toHaveLength(0);
    fireEvent.click(screen.getByRole('button', { name: 'Send' }));
    await screen.findByRole('alertdialog');
    expect(sends(client)).toHaveLength(0);

    fireEvent.click(screen.getByRole('button', { name: 'Send unencrypted' }));
    await waitFor(() => expect(onClose).toHaveBeenCalledTimes(1));
    expect(sends(client)).toHaveLength(1);
  });

  it('a reply that does not quote decrypted text is sent without the question', async () => {
    const { client, onClose } = await open(reply({ quotesDecrypted: false, cc: '' }));
    await screen.findByTestId('compose-richtext');
    fireEvent.click(screen.getByRole('button', { name: 'Send' }));
    await waitFor(() => expect(onClose).toHaveBeenCalledTimes(1));
    expect(screen.queryByRole('alertdialog')).toBeNull();
    expect(sends(client)).toHaveLength(1);
  });

  it('an encrypted reply is sent as ciphertext without the question', async () => {
    cryptoState = ENCRYPTING;
    const { client, onClose } = await open(reply({ quotesDecrypted: true, cc: '' }));
    await screen.findByTestId('compose-richtext');
    fireEvent.click(screen.getByRole('button', { name: 'Send' }));
    await waitFor(() => expect(onClose).toHaveBeenCalledTimes(1));
    expect(screen.queryByRole('alertdialog')).toBeNull();
    const create = sends(client)[0]!.methodCalls[0]![1]['create'] as Record<string, Record<string, unknown>>;
    expect(create['draft']!['bodyValues']).toEqual({
      body: { value: '-----BEGIN PGP MESSAGE-----\nCIPHER\n-----END PGP MESSAGE-----' },
    });
  });

  it('encryption switched on but no ciphertext yet still asks: the body would go out as plaintext', async () => {
    cryptoState = { ...ENCRYPTING, encryptedDraft: null };
    const { client } = await open(reply({ quotesDecrypted: true, cc: '' }));
    await screen.findByTestId('compose-richtext');
    fireEvent.click(screen.getByRole('button', { name: 'Send' }));
    await screen.findByRole('alertdialog');
    expect(sends(client)).toHaveLength(0);
  });
});

describe('Compose: Bcc on an encrypted message', () => {
  beforeEach(() => {
    localStorage.clear();
    cryptoState = ENCRYPTING;
  });

  it('is refused with the reason, and nothing is sent', async () => {
    const { client, onClose } = await open();
    fireEvent.click(screen.getByTestId('compose-show-cc-bcc'));
    fireEvent.input(screen.getByLabelText('To'), { target: { value: 'you@example.org' } });
    fireEvent.input(screen.getByLabelText('Bcc'), { target: { value: 'dave@example.org' } });
    fireEvent.click(screen.getByRole('button', { name: 'Send' }));
    expect(await screen.findByRole('alert')).toHaveTextContent(
      'An encrypted message lists the key of every recipient, so a Bcc recipient would be visible to the others.',
    );
    expect(sends(client)).toHaveLength(0);
    expect(onClose).not.toHaveBeenCalled();

    // Precondition of the refusal: the same message without the Bcc is sent.
    fireEvent.input(screen.getByLabelText('Bcc'), { target: { value: '' } });
    fireEvent.click(screen.getByRole('button', { name: 'Send' }));
    await waitFor(() => expect(onClose).toHaveBeenCalledTimes(1));
    expect(sends(client)).toHaveLength(1);
  });
});
