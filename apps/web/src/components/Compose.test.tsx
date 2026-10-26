import { describe, it, expect, beforeAll, beforeEach, afterEach, vi } from 'vitest';
import { render, screen, fireEvent, waitFor, within } from '@solidjs/testing-library';
import { Suspense } from 'solid-js';
import { Compose } from './Compose.tsx';
import { renderWithApp, makeClient } from './appHarness.tsx';
import { createAppState } from '../state/store.ts';
import { AppContext } from '../state/context.ts';
import { AssistService } from '../modules/assist/index.ts';
import { isolate } from '../i18n/index.ts';
import type { Identity, JmapRequest } from '../api/jmap-types.ts';
import { resetClientPolicy } from '../api/policy.ts';
import { clearSettings, loadSettings } from '../api/settings.ts';

// ── V7 (e14b) integration doubles ────────────────────────────────────────────
function json(body: unknown, status = 200): Response {
  return new Response(JSON.stringify(body), { status, headers: { 'content-type': 'application/json' } });
}

/** A directory fetcher: one person + one distribution group; group expands to 2 leaves. */
const directoryFetcher = async (input: string): Promise<Response> => {
  if (input.includes('/api/directory/search')) {
    return json({
      entries: [
        { dn: 'cn=alice', displayName: 'Alice Example', mail: 'alice@corp.example', isGroup: false },
        { dn: 'cn=team', displayName: 'Team All', mail: 'team@corp.example', isGroup: true },
      ],
      page: 0,
      hasMore: false,
    });
  }
  if (input.includes('/api/directory/group/')) {
    return json({
      members: [
        { dn: 'cn=bob', displayName: 'Bob', mail: 'bob@corp.example', isGroup: false },
        { dn: 'cn=carol', displayName: 'Carol', mail: 'carol@corp.example', isGroup: false },
      ],
    });
  }
  return json({}, 404);
};

/** An Assist gateway that reports enabled with the composer/dictation capabilities. */
function enabledAssistService(): AssistService {
  return new AssistService(async (input: string) => {
    if (input.includes('/api/assist/config')) {
      return json({
        availability: 'enabled',
        capabilities: ['grammar', 'draft', 'dictation'],
        endpoint_host: 'assist.local',
        include_e2ee: false,
        include_attachments: false,
      });
    }
    return json({}, 404);
  });
}

/** A Nextcloud fetcher: one file to attach; attach materialises a blob. */
const nextcloudFetcher = async (input: string): Promise<Response> => {
  if (input.includes('/api/account/nextcloud')) {
    return json({ accountId: 'acct1', instances: [{id: 'cloud', name: 'Cloud'}], links: [{instanceId: 'cloud', ready: true}] });
  }
  if (input.includes('/api/nextcloud/files')) {
    return json({
      entries: [
        { name: 'report.pdf', path: '/report.pdf', isDir: false, size: 2048, modified: null, contentType: 'application/pdf' },
      ],
    });
  }
  if (input.includes('/api/nextcloud/attach')) {
    return json({ attachments: [{ name: 'report.pdf', blobId: 'blob-1', size: 2048, contentType: 'application/pdf' }] });
  }
  return json({}, 404);
};

const IDENTITIES: Identity[] = [
  { id: 'id1', name: 'Personal', email: 'me@example.org', replyTo: null, signatureHtml: null, signatureText: 'Sent from Mailwoman', sentMailboxId: 'sent' },
  { id: 'id2', name: 'Work', email: 'work@corp.example', replyTo: null, signatureHtml: null, signatureText: null, sentMailboxId: 'sent' },
  // A stored HTML signature holding what the composer's schema does not allow.
  {
    id: 'id3',
    name: 'Markup',
    email: 'markup@corp.example',
    replyTo: null,
    signatureHtml:
      '<p style="position:fixed">Regards, <a href="javascript:alert(1)">me</a><img src="https://t.example/p.gif" width="1" height="1"></p><script>alert(2)</script>',
    signatureText: null,
    sentMailboxId: 'sent',
  },
];

describe('Compose', () => {
  beforeEach(() => localStorage.clear());
  afterEach(() => clearSettings());

  /**
   * Pay for the rich editor's chunk HERE, not inside a `findBy` (t22-e10 flake).
   *
   * `Compose` reaches its editor through `lazy(() => import(…))`, so the first
   * `await screen.findByTestId('compose-richtext')` in this file was waiting on a
   * real dynamic import of the ~286 kB ProseMirror module. `findBy*` carries its
   * OWN 1 000 ms ceiling, independent of the 5 000 ms test timeout — so this was
   * never a test-timeout problem, and raising `testTimeout` would not have moved
   * it. In a full-suite run that import competes with ~128 other files for the
   * transform pipeline (and with whatever else is on the host), which is why the
   * spec was green in isolation and red under load.
   *
   * Establishing that it is the IMPORT and not the editor: with this hook in
   * place the whole file passes with the `findBy` budget starved to **1 ms**.
   * Without it, at 1 ms, exactly one test fails — the first one to await the
   * editor — with `Unable to find an element by: [data-testid="compose-richtext"]`,
   * which is the failure seen in the wild. The second await was always warm.
   *
   * So the fix is not a bigger budget. It moves the cold, contended cost out of
   * an assertion budget and into a hook with a generous one, and every `findBy`
   * in this file keeps its 1 s fast-fail — a genuinely missing element still
   * fails in a second.
   */
  beforeAll(async () => {
    await import('./compose/RichTextEditor.tsx');
  }, 60_000);

  it('keeps the core To/Subject/Body labels + Send button (e2e contract)', () => {
    renderWithApp(() => <Compose onClose={() => undefined} />);
    expect(screen.getByLabelText('To')).toBeInTheDocument();
    expect(screen.getByLabelText('Subject')).toBeInTheDocument();
    expect(screen.getByLabelText('Body')).toBeInTheDocument();
    expect(screen.getByRole('button', { name: 'Send' })).toBeInTheDocument();
  });

  it('offers the sending identities once loaded', async () => {
    const { app } = renderWithApp(() => <Compose onClose={() => undefined} />, { identities: IDENTITIES });
    await app.loadIdentities();
    const select = await screen.findByLabelText('From');
    expect(select).toBeInTheDocument();
    // Scope to the From select — the signature picker (W12) also renders an
    // option named after each identity's signature.
    expect(within(select).getByRole('option', { name: /Work/ })).toBeInTheDocument();
    expect(within(select).getByRole('option', { name: /Personal/ })).toBeInTheDocument();
  });

  it('expands a quick reply without losing its draft or external-recipient safeguard', async () => {
    const { app } = renderWithApp(() => <Compose embedded initial={{ compact: true, mode: 'reply', to: 'alice@outside.test', cc: '', subject: 'Re: Plans', bodyText: 'Quoted text', bodyHtml: '<p>Quoted text</p>', inReplyTo: ['original@example.org'], references: [], attachments: [], quotesDecrypted: false }} onClose={() => undefined} />, { identities: IDENTITIES });
    await app.login({ username: 'me@example.org', password: 'p' });
    expect(screen.queryByLabelText('To')).toBeNull();
    expect(screen.getByRole('region', { name: 'Quick reply' })).toHaveTextContent('alice@outside.test');
    expect(screen.getByText('1 recipient(s) outside trusted domains')).toBeVisible();
    fireEvent.click(screen.getByTestId('format-toggle'));
    fireEvent.input(screen.getByLabelText('Body'), { target: { value: 'My quick reply\nQuoted text' } });
    fireEvent.click(screen.getByRole('button', { name: 'Edit recipients and subject' }));
    expect(screen.getByLabelText('To')).toHaveValue('alice@outside.test');
    expect(screen.getByLabelText('Subject')).toHaveValue('Re: Plans');
    expect(screen.getByLabelText('Body')).toHaveValue('My quick reply\nQuoted text');
  });

  it('sends via the chosen identity', async () => {
    const { app } = renderWithApp(() => <Compose onClose={() => undefined} />, { identities: IDENTITIES });
    await app.login({ username: 'me@example.org', password: 'p' });
    await app.loadIdentities();

    fireEvent.input(screen.getByLabelText('To'), { target: { value: 'you@example.org' } });
    fireEvent.change(await screen.findByLabelText('From'), { target: { value: 'id2' } });
    fireEvent.click(screen.getByRole('button', { name: 'Send' }));

    // The compose flow completes (draft + submission built with the identity)
    // and surfaces the undo-send toast rather than an error.
    await waitFor(() => expect(app.pendingUndo()?.actionLabel).toBe('Cancel'));
    expect(screen.queryByRole('alert')).toBeNull();
  });

  it('switches the Send button to Schedule when a send-later time is set', () => {
    renderWithApp(() => <Compose onClose={() => undefined} />);
    const later = screen.getByLabelText('Send later');
    fireEvent.input(later, { target: { value: '2099-01-01T09:00' } });
    expect(screen.getByRole('button', { name: 'Schedule' })).toBeInTheDocument();
  });

  // ── V7 (e14b): GAL / Assist / Nextcloud last-mile mailbox integration ────────

  it('surfaces GAL autocomplete in the recipient field when a directory is configured', async () => {
    const { app } = renderWithApp(() => <Compose onClose={() => undefined} />, {
      deps: { directoryFetcher },
    });
    app.directory.setEnabled(true);
    fireEvent.input(screen.getByLabelText('To'), { target: { value: 'ali' } });
    const listbox = await screen.findByRole('listbox', { name: 'Directory matches' });
    expect(within(listbox).getByText('Alice Example')).toBeInTheDocument();
    // Picking a person inserts their directory address as a recipient.
    fireEvent.click(within(listbox).getByText('Alice Example'));
    expect((screen.getByLabelText('To') as HTMLInputElement).value).toContain('alice@corp.example');
  });

  it('expands a distribution group to its leaf recipients before send', async () => {
    const { app } = renderWithApp(() => <Compose onClose={() => undefined} />, {
      deps: { directoryFetcher },
    });
    app.directory.setEnabled(true);
    fireEvent.input(screen.getByLabelText('To'), { target: { value: 'team' } });
    const listbox = await screen.findByRole('listbox', { name: 'Directory matches' });
    fireEvent.click(within(listbox).getByText('Team All'));

    // The group is addressed and the expand-before-send control appears.
    const panel = await screen.findByTestId('group-expand');
    fireEvent.click(within(panel).getByRole('button', { name: /who is actually in this/i }));
    await within(panel).findByTestId('member-count');
    fireEvent.click(within(panel).getByRole('button', { name: /Replace group with 2 recipients/i }));

    const to = (screen.getByLabelText('To') as HTMLInputElement).value;
    expect(to).toContain('bob@corp.example');
    expect(to).toContain('carol@corp.example');
    expect(to).not.toContain('team@corp.example');
  });

  it('shows the inline Assist composer tools when the gateway is enabled', async () => {
    const { app } = renderWithApp(() => <Compose onClose={() => undefined} />, {
      deps: { assistService: enabledAssistService() },
    });
    await app.assist.loadConfig();
    expect(await screen.findByTestId('compose-assist')).toBeInTheDocument();
    expect(screen.getByRole('button', { name: 'Fix grammar' })).toBeInTheDocument();
  });

  it('attaches a Nextcloud file into the composer', async () => {
    const { app } = renderWithApp(() => <Compose onClose={() => undefined} />, {
      deps: { nextcloudFetcher },
    });
    await app.login({ username: 'me@example.org', password: 'p' });
    app.nextcloud.setEnabled(true);
    fireEvent.click(await screen.findByRole('button', { name: 'Attach from Nextcloud' }));
    // The picker lists the linked Nextcloud; select the file then attach it.
    const file = await screen.findByText('report.pdf');
    fireEvent.click(file);
    fireEvent.click(await screen.findByRole('button', { name: /Attach 1 file/i }));
    const list = await screen.findByTestId('compose-attachments');
    expect(within(list).getByText('report.pdf')).toBeInTheDocument();
  });

  it('opens the signing-key unlock panel when the sign toggle is switched on', async () => {
    renderWithApp(() => <Compose onClose={() => undefined} />);
    fireEvent.input(screen.getByLabelText('To'), { target: { value: 'you@example.org' } });
    // Enabling the sign toggle while the signing key is locked prompts to unlock
    // it (once per composer) rather than sending silently unsigned.
    fireEvent.click(await screen.findByTestId('sign-toggle'));
    expect(await screen.findByTestId('compose-sign-unlock')).toBeInTheDocument();
    expect(screen.getByTestId('sign-passphrase')).toBeInTheDocument();
  });

  // ── 26.15 (§1): new-file local blob upload ───────────────────────────────────

  /** A global-`fetch` double for the composer's OWN `jmapClient` (created via
   *  `createConfiguredClient()`, which uses the global `fetch`): serves the
   *  session probe (uploadUrl + a small `maxSizeUpload`) and the per-account
   *  upload endpoint. The harness `app` still uses its own fake client. */
  function mockUploadFetch(): ReturnType<typeof vi.fn> {
    return vi.fn(async (input: RequestInfo | URL): Promise<Response> => {
      const url = typeof input === 'string' ? input : input.toString();
      if (url.includes('/jmap/session')) {
        return json({
          capabilities: { 'urn:ietf:params:jmap:core': { maxSizeUpload: 20 } },
          accounts: {},
          primaryAccounts: {},
          username: 'me@example.org',
          apiUrl: '/jmap/api',
          downloadUrl: '/d',
          uploadUrl: '/jmap/upload/{accountId}',
          eventSourceUrl: '/e',
          state: 's0',
        });
      }
      if (url.includes('/jmap/upload/')) {
        return json({ accountId: 'acct1', blobId: 'Ublob-file-1', type: 'text/plain', size: 5 });
      }
      return json({}, 404);
    });
  }

  afterEach(() => vi.unstubAllGlobals());

  it('uploads a locally-picked file and adds it as an attachment', async () => {
    vi.stubGlobal('fetch', mockUploadFetch());
    const { app } = renderWithApp(() => <Compose onClose={() => undefined} />, { identities: IDENTITIES });
    await app.login({ username: 'me@example.org', password: 'p' });

    const input = (await screen.findByLabelText('Attach a file')) as HTMLInputElement;
    // The picker enables once the session probe lands (uploadUrl + accountId).
    await waitFor(() => expect(input.disabled).toBe(false));

    const file = new File(['hello'], 'note.txt', { type: 'text/plain' });
    fireEvent.change(input, { target: { files: [file] } });

    const list = await screen.findByTestId('compose-attachments');
    expect(within(list).getByText('note.txt')).toBeInTheDocument();
  });

  it('refuses a file larger than the upload limit and never uploads it', async () => {
    const fetchSpy = mockUploadFetch();
    vi.stubGlobal('fetch', fetchSpy);
    const { app } = renderWithApp(() => <Compose onClose={() => undefined} />, { identities: IDENTITIES });
    await app.login({ username: 'me@example.org', password: 'p' });

    const input = (await screen.findByLabelText('Attach a file')) as HTMLInputElement;
    await waitFor(() => expect(input.disabled).toBe(false));

    // The mocked session caps uploads at 20 bytes; this file is over it.
    const big = new File(['x'.repeat(64)], 'big.bin', { type: 'application/octet-stream' });
    fireEvent.change(input, { target: { files: [big] } });

    const attach = screen.getByTestId('compose-attach');
    expect(await within(attach).findByRole('alert')).toHaveTextContent(/maximum upload size/i);
    // Nothing attached, and the upload endpoint was never hit (only the probe).
    expect(screen.queryByTestId('compose-attachments')).toBeNull();
    const uploadCalls = fetchSpy.mock.calls.filter((c) => String(c[0]).includes('/jmap/upload/'));
    expect(uploadCalls).toHaveLength(0);
  });

  it('is unchanged when Assist is disabled and no directory/Nextcloud is configured', async () => {
    renderWithApp(() => <Compose onClose={() => undefined} />);
    // Type a recipient — contacts autocomplete still works; NO directory GAL, no
    // Assist tools, no Nextcloud affordance render (disabled-path regression).
    fireEvent.input(screen.getByLabelText('To'), { target: { value: 'someone' } });
    // Give any probe/debounce a chance; nothing gated should appear.
    await waitFor(() => expect(screen.getByLabelText('To')).toBeInTheDocument());
    expect(screen.queryByTestId('compose-gal')).toBeNull();
    expect(screen.queryByRole('listbox', { name: 'Directory matches' })).toBeNull();
    expect(screen.queryByTestId('compose-assist')).toBeNull();
    expect(screen.queryByTestId('compose-nextcloud')).toBeNull();
    // The core composer contract is intact.
    expect(screen.getByRole('button', { name: 'Send' })).toBeInTheDocument();
  });

  // ── 26.16 (W1): rich-text editor + plain-text / format=flowed toggle ─────────

  it('defaults to the rich editor and toggles to a plain-text textarea', async () => {
    renderWithApp(() => <Compose onClose={() => undefined} />);
    // The rich editor mounts lazily; its editable region carries the Body label.
    expect(await screen.findByTestId('compose-richtext')).toBeInTheDocument();
    // Toggle to plain text: a textarea takes over, still labelled Body.
    fireEvent.click(screen.getByTestId('format-toggle'));
    expect(screen.queryByTestId('compose-richtext')).toBeNull();
    expect((screen.getByLabelText('Body') as HTMLElement).tagName).toBe('TEXTAREA');
  });

  it('round-trips the body text between plain and rich modes', async () => {
    renderWithApp(() => <Compose onClose={() => undefined} />);
    await screen.findByTestId('compose-richtext');
    fireEvent.click(screen.getByTestId('format-toggle')); // rich -> plain
    fireEvent.input(screen.getByLabelText('Body'), { target: { value: 'hello\nworld' } });
    fireEvent.click(screen.getByTestId('format-toggle')); // plain -> rich (seeds editor)
    await screen.findByTestId('compose-richtext');
    fireEvent.click(screen.getByTestId('format-toggle')); // rich -> plain
    expect((screen.getByLabelText('Body') as HTMLTextAreaElement).value).toBe('hello\nworld');
  });

  // ── The send payload, whole ─────────────────────────────────────────────────

  /** A composer over a client whose calls the test can read back. */
  async function composeWithClient(opts: Parameters<typeof makeClient>[0] = {}) {
    const client = makeClient(opts);
    const app = createAppState(client);
    const onClose = vi.fn();
    render(() => (
      <AppContext.Provider value={app}>
        <Compose onClose={onClose} />
      </AppContext.Provider>
    ));
    await app.login({ username: 'me@example.org', password: 'p' });
    return { client, app, onClose };
  }

  /** How many compose+submit requests the client was handed. */
  function sendCount(client: ReturnType<typeof makeClient>): number {
    return vi
      .mocked(client.jmap)
      .mock.calls.filter((c) => c[0].methodCalls.some((m) => m[0] === 'EmailSubmission/set' && 'create' in m[1]))
      .length;
  }

  /** The body the one send carried. */
  it('requires renewed confirmation when external recipients change before sending', async () => {
    const { client, app } = await composeWithClient({ identities: IDENTITIES });
    await app.loadIdentities();
    await loadSettings({ refresh: true, fetcher: async () => json({ settings: {
      'mail.externalRecipientWarning': { value: 'confirm', source: 'global', locked: true },
      'mail.trustedRecipientDomains': { value: ['partner.test'], source: 'global', locked: true },
    } }) });
    fillPlain('Hello', 'recipient@outside.test');
    fireEvent.click(screen.getByRole('button', { name: 'Send' }));
    expect(sendCount(client)).toBe(0);
    expect(screen.getByText('Review and confirm the external recipients before sending.')).toBeVisible();
    fireEvent.click(screen.getByLabelText('I have checked these external recipients.'));
    fireEvent.input(screen.getByLabelText('To'), { target: { value: 'different@outside.test' } });
    expect(screen.getByLabelText('I have checked these external recipients.')).not.toBeChecked();
    fireEvent.click(screen.getByRole('button', { name: 'Send' }));
    expect(sendCount(client)).toBe(0);
    fireEvent.click(screen.getByLabelText('I have checked these external recipients.'));
    fireEvent.click(screen.getByRole('button', { name: 'Send' }));
    await waitFor(() => expect(sendCount(client)).toBe(1));
  });

  function sentBody(client: ReturnType<typeof makeClient>): unknown {
    const create = sentRequest(client).methodCalls[0]![1]['create'] as Record<string, Record<string, unknown>>;
    return create['draft']!['bodyValues'];
  }

  /** Fill To / Subject / Body in plain-text mode. */
  function fillPlain(body: string, to = 'you@example.org'): void {
    fireEvent.click(screen.getByTestId('format-toggle'));
    fireEvent.input(screen.getByLabelText('To'), { target: { value: to } });
    fireEvent.input(screen.getByLabelText('Subject'), { target: { value: 'Hi' } });
    fireEvent.input(screen.getByLabelText('Body'), { target: { value: body } });
  }

  /** The one compose+submit request the client was handed. */
  function sentRequest(client: ReturnType<typeof makeClient>): JmapRequest {
    const sends = vi
      .mocked(client.jmap)
      .mock.calls.map((c) => c[0])
      .filter((r) => r.methodCalls.some((m) => m[0] === 'EmailSubmission/set' && 'create' in m[1]));
    expect(sends).toHaveLength(1);
    return sends[0]!;
  }

  // Precondition for the t29-e12 cases below: a server that is neither this
  // server's engine nor has open tracking on gets no send options at all.
  it('offers no receipt or tracking control where the server can do neither, and sends exactly the typed body', async () => {
    const { client, app } = await composeWithClient({ identities: IDENTITIES });
    expect(screen.queryByTestId('compose-send-options')).toBeNull();
    expect(screen.queryByTestId('opt-tracking')).toBeNull();
    expect(screen.queryByTestId('opt-receipt')).toBeNull();

    fireEvent.click(screen.getByTestId('format-toggle')); // plain text
    fireEvent.input(screen.getByLabelText('To'), { target: { value: 'you@example.org' } });
    fireEvent.input(screen.getByLabelText('Subject'), { target: { value: 'Hi' } });
    fireEvent.input(screen.getByLabelText('Body'), { target: { value: 'plain body' } });
    fireEvent.click(screen.getByRole('button', { name: 'Send' }));
    await waitFor(() => expect(app.pendingUndo()?.actionLabel).toBe('Cancel'));

    expect(sentRequest(client).methodCalls).toEqual([
      [
        'Email/set',
        {
          accountId: 'acct1',
          create: {
            draft: {
              mailboxIds: { inbox: true },
              keywords: { $draft: true, $seen: true },
              from: [{ name: null, email: 'me@example.org' }],
              to: [{ name: null, email: 'you@example.org' }],
              subject: 'Hi',
              htmlBody: [{ partId: 'body', type: 'text/html' }],
              bodyValues: { body: { value: '<p>plain body</p>' } },
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
              envelope: { mailFrom: { email: 'me@example.org' }, rcptTo: [{ email: 'you@example.org' }] },
              mailwomanHoldSeconds: 10,
            },
          },
        },
        'submit',
      ],
    ]);
  });

  // ── Send later ───────────────────────────────────────────────────────────────

  it('offers no send-later time earlier than the next minute', () => {
    renderWithApp(() => <Compose onClose={() => undefined} />);
    const min = screen.getByLabelText('Send later').getAttribute('min');
    expect(min).toMatch(/^\d{4}-\d{2}-\d{2}T\d{2}:\d{2}$/);
    expect(new Date(min!).getTime()).toBeGreaterThan(Date.now());
    expect(new Date(min!).getTime()).toBeLessThanOrEqual(Date.now() + 60_000);
  });

  it('schedules a future time: the submission carries it and the composer closes', async () => {
    const { client, onClose } = await composeWithClient();
    fillPlain('later');
    fireEvent.input(screen.getByLabelText('Send later'), { target: { value: '2099-01-01T09:00' } });
    fireEvent.click(screen.getByRole('button', { name: 'Schedule' }));
    await waitFor(() => expect(onClose).toHaveBeenCalledTimes(1));
    const send = (sentRequest(client).methodCalls[1]![1]['create'] as Record<string, Record<string, unknown>>)['send']!;
    expect(send).toEqual({
      emailId: '#draft',
      envelope: { mailFrom: { email: 'me@example.org' }, rcptTo: [{ email: 'you@example.org' }] },
      sendAt: new Date('2099-01-01T09:00').toISOString(),
      mailwomanHoldSeconds: 0,
    });
  });

  it('refuses a past send-later time: says so, sends nothing, stays open', async () => {
    const { client, onClose } = await composeWithClient();
    fillPlain('later');
    const later = screen.getByLabelText('Send later') as HTMLInputElement;
    fireEvent.input(later, { target: { value: '2001-01-01T09:00' } });
    fireEvent.click(screen.getByRole('button', { name: 'Schedule' }));
    expect(await screen.findByRole('alert')).toHaveTextContent(
      'Choose a send time in the future, or clear the field to send now.',
    );
    expect(sendCount(client)).toBe(0);
    expect(onClose).not.toHaveBeenCalled();
    // The field keeps what was typed, so it can be corrected.
    expect(later.value).toBe('2001-01-01T09:00');
  });

  it('refuses a time that passed while the composer was open (the field minimum is stale)', async () => {
    const { client, onClose } = await composeWithClient();
    fillPlain('later');
    const later = screen.getByLabelText('Send later') as HTMLInputElement;
    // What a long-open composer looks like: the minimum was computed long ago,
    // so the browser's own range check accepts a time that is now in the past.
    later.setAttribute('min', '2000-01-01T00:00');
    fireEvent.input(later, { target: { value: '2001-01-01T09:00' } });
    expect(later.validity.valid).toBe(true);
    fireEvent.click(screen.getByRole('button', { name: 'Schedule' }));
    expect(await screen.findByRole('alert')).toHaveTextContent(
      'Choose a send time in the future, or clear the field to send now.',
    );
    expect(sendCount(client)).toBe(0);
    expect(onClose).not.toHaveBeenCalled();
  });

  // ── A refused send ───────────────────────────────────────────────────────────

  it('shows why the server refused the draft and keeps the composer open', async () => {
    const { client, onClose } = await composeWithClient();
    const answer = vi.mocked(client.jmap).getMockImplementation()!;
    vi.mocked(client.jmap).mockImplementation(async (body, opts) => {
      if (!body.methodCalls.some((m) => m[0] === 'EmailSubmission/set' && 'create' in m[1])) return answer(body, opts);
      // The engine's shape for a draft it will not store (mw-engine jmap.rs,
      // `InvalidProperty::set_error`).
      return {
        methodResponses: [
          [
            'Email/set',
            {
              accountId: 'acct1',
              created: null,
              notCreated: {
                draft: { type: 'invalidProperties', properties: ['from'], description: 'from: "a b": contains whitespace' },
              },
            },
            'set',
          ],
          ['EmailSubmission/set', { accountId: 'acct1', created: null, notCreated: { send: { type: 'invalidProperties' } } }, 'submit'],
        ],
        sessionState: 's',
      };
    });
    fillPlain('hello');
    fireEvent.click(screen.getByRole('button', { name: 'Send' }));
    expect(await screen.findByRole('alert')).toHaveTextContent(
      `The server refused this message: ${isolate('from: "a b": contains whitespace')}. Nothing was sent.`,
    );
    expect(sendCount(client)).toBe(1);
    expect(onClose).not.toHaveBeenCalled();
    expect(screen.getByRole('dialog', { name: 'Compose message' })).toBeInTheDocument();
    expect((screen.getByLabelText('Body') as HTMLTextAreaElement).value).toBe('hello');
  });

  it('names a recipient that is not an address, sends nothing, stays open', async () => {
    const { client, onClose } = await composeWithClient();
    fillPlain('hello', 'you@example.org, bob');
    fireEvent.click(screen.getByRole('button', { name: 'Send' }));
    expect(await screen.findByRole('alert')).toHaveTextContent(`This is not an email address: ${isolate('bob')}`);
    expect(sendCount(client)).toBe(0);
    expect(onClose).not.toHaveBeenCalled();
  });

  // ── The identity signature ───────────────────────────────────────────────────

  it('appends the identity signature once, reduced to the composer schema', async () => {
    const { client, app, onClose } = await composeWithClient({ identities: IDENTITIES });
    await app.loadIdentities();
    fireEvent.change(await screen.findByLabelText('From'), { target: { value: 'id3' } });
    fillPlain('hi');
    fireEvent.click(screen.getByRole('button', { name: 'Send' }));
    await waitFor(() => expect(onClose).toHaveBeenCalledTimes(1));
    // No <img>, no javascript: link, no style attribute, no <script>.
    expect(sentBody(client)).toEqual({ body: { value: '<p>hi</p><br><p>Regards, me</p>' } });
  });

  it('appends a text-only signature as escaped text', async () => {
    const { client, app, onClose } = await composeWithClient({ identities: IDENTITIES });
    await app.loadIdentities();
    fireEvent.change(await screen.findByLabelText('From'), { target: { value: 'id1' } });
    expect(await screen.findByTestId('compose-auto-signature')).toHaveTextContent('Sent from Mailwoman');
    fillPlain('hi');
    fireEvent.click(screen.getByRole('button', { name: 'Send' }));
    await waitFor(() => expect(onClose).toHaveBeenCalledTimes(1));
    expect(sentBody(client)).toEqual({ body: { value: '<p>hi</p><br><p>Sent from Mailwoman</p>' } });
  });

  it('does not append a signature the picker already put in the body', async () => {
    const { client, app, onClose } = await composeWithClient({ identities: IDENTITIES });
    await app.loadIdentities();
    fireEvent.change(await screen.findByLabelText('From'), { target: { value: 'id1' } });
    fillPlain('hi');
    // Precondition: the send would append it.
    expect(screen.getByTestId('compose-auto-signature')).toBeInTheDocument();
    const picker = await screen.findByTestId('compose-signature');
    fireEvent.change(within(picker).getByRole('combobox'), { target: { value: 'id1' } });
    expect(screen.queryByTestId('compose-auto-signature')).toBeNull();
    fireEvent.click(screen.getByRole('button', { name: 'Send' }));
    await waitFor(() => expect(onClose).toHaveBeenCalledTimes(1));
    expect(sentBody(client)).toEqual({
      body: { value: '<p>hi<br><br>-- <br>Sent from Mailwoman</p>' },
    });
  });

  it('appends no signature to a signed send', async () => {
    const { app } = await composeWithClient({ identities: IDENTITIES });
    await app.loadIdentities();
    fireEvent.change(await screen.findByLabelText('From'), { target: { value: 'id1' } });
    fireEvent.input(screen.getByLabelText('To'), { target: { value: 'you@example.org' } });
    // Precondition: unsigned, the signature would be appended.
    expect(await screen.findByTestId('compose-auto-signature')).toBeInTheDocument();
    fireEvent.click(await screen.findByTestId('sign-toggle'));
    await waitFor(() => expect(screen.queryByTestId('compose-auto-signature')).toBeNull());
  });

  // ── W9 / W10 / W12: drawers + signature picker ───────────────────────────────

  it('typing in the body does not suspend anything outside the crypto panel', async () => {
    // The shell renders the mailbox under a Suspense boundary (LazyRoute in
    // App.tsx). The crypto panel's DLP scan goes pending on every body change;
    // if that reached the outer boundary, the composer would leave the document
    // mid-keystroke and the field being typed in would lose focus.
    const app = createAppState(makeClient());
    render(() => (
      <AppContext.Provider value={app}>
        <Suspense fallback={<p data-testid="outer-fallback" />}>
          <Compose onClose={() => undefined} />
        </Suspense>
      </AppContext.Provider>
    ));
    await app.login({ username: 'me@example.org', password: 'p' });
    fireEvent.click(screen.getByTestId('format-toggle')); // plain text
    const body = screen.getByLabelText('Body');
    expect(screen.queryByTestId('outer-fallback')).toBeNull();

    fireEvent.input(body, { target: { value: 'h' } });
    // Checked at once: the scan is pending right now.
    expect(screen.queryByTestId('outer-fallback')).toBeNull();
    expect(body.isConnected).toBe(true);
    fireEvent.input(screen.getByLabelText('To'), { target: { value: 'you@example.org' } });
    expect(screen.queryByTestId('outer-fallback')).toBeNull();
    expect(body.isConnected).toBe(true);
  });

  it('opens the Drafts drawer from the header', () => {
    renderWithApp(() => <Compose onClose={() => undefined} />);
    fireEvent.click(screen.getByTestId('open-drafts'));
    expect(screen.getByTestId('compose-drafts')).toBeInTheDocument();
  });

  it('inserts a chosen signature into the plain-text body', async () => {
    const { app } = renderWithApp(() => <Compose onClose={() => undefined} />, { identities: IDENTITIES });
    await app.loadIdentities();
    fireEvent.click(screen.getByTestId('format-toggle')); // plain text
    const picker = await screen.findByTestId('compose-signature');
    fireEvent.change(within(picker).getByRole('combobox'), { target: { value: 'id1' } });
    expect((screen.getByLabelText('Body') as HTMLTextAreaElement).value).toContain('Sent from Mailwoman');
  });

  // ── t29-e12: read receipt, open tracking, inline images ──────────────────────

  describe('send options and inline images (t29-e12)', () => {
    /** The composer's own fetches: the session probe (engine capability when
     *  asked), `/api/policy`, the upload endpoint and the open-tracking routes. */
    function serverFetch(opts: { engine: boolean; openTracking: boolean }): ReturnType<typeof vi.fn> {
      return vi.fn(async (input: RequestInfo | URL, init?: RequestInit): Promise<Response> => {
        const url = typeof input === 'string' ? input : input.toString();
        if (url.includes('/jmap/session')) {
          return json({
            capabilities: {
              'urn:ietf:params:jmap:core': { maxSizeUpload: 1_000_000 },
              ...(opts.engine ? { 'urn:mailwoman:security': {} } : {}),
            },
            accounts: {},
            primaryAccounts: {},
            username: 'me@example.org',
            apiUrl: '/jmap/api',
            downloadUrl: '/jmap/download/{accountId}/{blobId}/{name}?type={type}',
            uploadUrl: '/jmap/upload/{accountId}',
            eventSourceUrl: '/e',
            state: 's0',
          });
        }
        if (url.endsWith('/api/policy')) {
          return json({ maxSecurityFloor: null, capturePolicy: 'off', openTracking: opts.openTracking });
        }
        if (url.includes('/jmap/upload/')) {
          return json({ accountId: 'acct1', blobId: 'Gimg1', type: 'image/png', size: 4 });
        }
        if (url.endsWith('/api/track/open') && init?.method === 'POST') {
          return json({ token: 'tok', url: 'https://mail.example/t/tok.gif' });
        }
        if (url.includes('/api/track/open/') && init?.method === 'PUT') return new Response(null, { status: 204 });
        return json({}, 404);
      });
    }

    beforeEach(() => resetClientPolicy());
    afterEach(() => {
      resetClientPolicy();
      vi.unstubAllGlobals();
    });

    function draftOf(client: ReturnType<typeof makeClient>): Record<string, unknown> {
      const req = sentRequest(client);
      return (req.methodCalls[0]![1]['create'] as Record<string, Record<string, unknown>>)['draft']!;
    }

    function fillAndSend(): void {
      fireEvent.input(screen.getByLabelText('To'), { target: { value: 'you@example.org' } });
      fireEvent.input(screen.getByLabelText('Subject'), { target: { value: 'Hi' } });
      fireEvent.click(screen.getByRole('button', { name: 'Send' }));
    }

    it("on this server's engine the receipt request is offered, off, and sent only when ticked", async () => {
      vi.stubGlobal('fetch', serverFetch({ engine: true, openTracking: false }));
      const { client, app } = await composeWithClient();
      const receipt = (await screen.findByTestId('compose-request-receipt')) as HTMLInputElement;
      expect(receipt.checked).toBe(false);
      expect(screen.getByText(/may ask them, send one without asking, or ignore the request/)).toBeInTheDocument();
      expect(screen.queryByTestId('compose-track-opens')).toBeNull();

      fireEvent.click(receipt);
      fillAndSend();
      await waitFor(() => expect(app.pendingUndo()?.actionLabel).toBe('Cancel'));
      expect(draftOf(client)['mailwomanRequestReadReceipt']).toBe(true);
    });

    it('open tracking is offered only when the server records opens, and not for a signed message', async () => {
      vi.stubGlobal('fetch', serverFetch({ engine: false, openTracking: true }));
      await composeWithClient();
      const track = (await screen.findByTestId('compose-track-opens')) as HTMLInputElement;
      expect(track.checked).toBe(false);
      expect(screen.getByText(/does not prove the message was read/)).toBeInTheDocument();
      // No engine: no receipt option.
      expect(screen.queryByTestId('compose-request-receipt')).toBeNull();

      fireEvent.input(screen.getByLabelText('To'), { target: { value: 'you@example.org' } });
      fireEvent.click(await screen.findByTestId('sign-toggle'));
      await waitFor(() => expect(screen.queryByTestId('compose-track-opens')).toBeNull());
    });

    it('a tracked send carries the image from the minted URL after the body', async () => {
      const fetchSpy = serverFetch({ engine: false, openTracking: true });
      vi.stubGlobal('fetch', fetchSpy);
      const { client, app } = await composeWithClient();
      fireEvent.click(await screen.findByTestId('compose-track-opens'));
      fireEvent.click(screen.getByTestId('format-toggle')); // plain text
      fireEvent.input(screen.getByLabelText('Body'), { target: { value: 'tracked' } });
      fillAndSend();
      await waitFor(() => expect(app.pendingUndo()?.actionLabel).toBe('Cancel'));
      expect(draftOf(client)['bodyValues']).toEqual({
        body: { value: '<p>tracked</p><img src="https://mail.example/t/tok.gif" width="1" height="1" alt="">' },
      });
      await waitFor(() =>
        expect(fetchSpy.mock.calls.some((c) => String(c[0]).endsWith('/api/track/open/tok'))).toBe(true),
      );
    });

    it('an image placed in the body is uploaded, shown from a local preview, and sent as an inline part', async () => {
      vi.stubGlobal('fetch', serverFetch({ engine: true, openTracking: false }));
      const createObjectURL = vi.fn(() => 'blob:preview-1');
      const revokeObjectURL = vi.fn();
      Object.assign(URL, { createObjectURL, revokeObjectURL });
      const { client, app } = await composeWithClient();
      await screen.findByTestId('compose-richtext');
      await waitFor(() => expect(screen.queryByTestId('compose-request-receipt')).not.toBeNull());
      const input = (await screen.findByLabelText('Insert an image')) as HTMLInputElement;

      fireEvent.change(input, { target: { files: [new File(['PNG'], 'chart.png', { type: 'image/png' })] } });
      const shown = await waitFor(() => {
        const img = screen.getByTestId('compose-richtext').querySelector('img.compose-rt__img');
        expect(img).not.toBeNull();
        return img!;
      });
      expect(shown.getAttribute('src')).toBe('blob:preview-1');

      fillAndSend();
      await waitFor(() => expect(app.pendingUndo()?.actionLabel).toBe('Cancel'));
      const draft = draftOf(client);
      const attachments = draft['attachments'] as { cid: string }[];
      expect(attachments).toEqual([
        { blobId: 'Gimg1', cid: attachments[0]!.cid, disposition: 'inline', type: 'image/png', name: 'chart.png' },
      ]);
      expect(attachments[0]!.cid).toMatch(/@mailwoman$/);
      const body = (draft['bodyValues'] as { body: { value: string } }).body.value;
      expect(body).toContain(`<img src="cid:${attachments[0]!.cid}" alt="chart.png">`);
      expect(body).not.toContain('blob:');
    });

    it('refuses an SVG for the body and uploads nothing', async () => {
      const fetchSpy = serverFetch({ engine: true, openTracking: false });
      vi.stubGlobal('fetch', fetchSpy);
      await composeWithClient();
      await screen.findByTestId('compose-richtext');
      await waitFor(() => expect(screen.queryByTestId('compose-request-receipt')).not.toBeNull());
      const input = (await screen.findByLabelText('Insert an image')) as HTMLInputElement;
      fireEvent.change(input, { target: { files: [new File(['<svg/>'], 'x.svg', { type: 'image/svg+xml' })] } });
      expect(await screen.findByText(/cannot be placed in the text/)).toBeInTheDocument();
      expect(fetchSpy.mock.calls.filter((c) => String(c[0]).includes('/jmap/upload/'))).toHaveLength(0);
    });
  });
});
