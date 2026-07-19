import { describe, it, expect, beforeEach, afterEach, vi } from 'vitest';
import { screen, fireEvent, waitFor, within } from '@solidjs/testing-library';
import { Compose } from './Compose.tsx';
import { renderWithApp } from './appHarness.tsx';
import { AssistService } from '../modules/assist/index.ts';
import type { Identity } from '../api/jmap-types.ts';

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
  if (input.includes('/api/nextcloud/list')) {
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
];

describe('Compose', () => {
  beforeEach(() => localStorage.clear());

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
    expect(screen.getByRole('option', { name: /Work/ })).toBeInTheDocument();
    expect(screen.getByRole('option', { name: /Personal/ })).toBeInTheDocument();
  });

  it('sends via the chosen identity', async () => {
    const { app } = renderWithApp(() => <Compose onClose={() => undefined} />, { identities: IDENTITIES });
    await app.login({ jmapUrl: 'x', username: 'me@example.org', password: 'p' });
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
    await app.login({ jmapUrl: 'x', username: 'me@example.org', password: 'p' });

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
    await app.login({ jmapUrl: 'x', username: 'me@example.org', password: 'p' });

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
});
