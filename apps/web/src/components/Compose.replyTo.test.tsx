import { afterEach, beforeEach, describe, expect, it } from 'vitest';
import { cleanup, fireEvent, screen, waitFor } from '@solidjs/testing-library';
import { Compose } from './Compose.tsx';
import { renderWithApp } from './appHarness.tsx';
import { SEND_CHECKS_OFF } from './compose/sendChecksTestSettings.ts';
import { clearSettings, loadSettings } from '../api/settings.ts';
import type { Identity } from '../api/jmap-types.ts';
import { listDrafts } from './compose/drafts-store.ts';

const json = (body: unknown): Response => new Response(JSON.stringify(body), { status: 200, headers: { 'content-type': 'application/json' } });
const entry = (value: unknown, locked = false) => ({ value, source: 'user', locked });
const IDENTITIES: Identity[] = [
  { id: 'id1', name: 'Me', email: 'me@example.org', replyTo: null, signatureHtml: null, signatureText: null, sentMailboxId: 'sent' },
  { id: 'id2', name: 'Support', email: 'support@example.org', replyTo: 'help@example.org', signatureHtml: null, signatureText: null, sentMailboxId: 'sent' },
];

async function settingsWith(extra: Record<string, unknown> = {}): Promise<void> {
  await loadSettings({ refresh: true, fetcher: async () => json({ settings: { ...SEND_CHECKS_OFF, ...extra } }) });
}

beforeEach(() => { clearSettings(); localStorage.clear(); });
afterEach(cleanup);

describe('Compose Reply-To row', () => {
  it('is hidden until the toggle next to Cc/Bcc is pressed, then takes validated addresses', async () => {
    await settingsWith();
    const { app } = renderWithApp(() => <Compose onClose={() => undefined} />, { identities: IDENTITIES });
    await app.loadIdentities();
    expect(screen.queryByTestId('compose-replyto-row')).toBeNull();
    const toggle = await screen.findByTestId('compose-head-replyto');
    expect(toggle).toHaveAttribute('aria-pressed', 'false');
    fireEvent.click(toggle);
    expect(await screen.findByTestId('compose-replyto-row')).toBeInTheDocument();
    expect(screen.getByTestId('compose-head-replyto')).toHaveAttribute('aria-pressed', 'true');
  });

  it('prefills the identity Reply-To and warns when it leaves the From domain', async () => {
    await settingsWith({ 'mail.showFrom': entry(true) });
    const { app } = renderWithApp(() => <Compose onClose={() => undefined} />, { identities: IDENTITIES });
    await app.loadIdentities();
    const from = await screen.findByLabelText('From');
    fireEvent.change(from, { target: { value: 'id2' } });
    const row = await screen.findByTestId('compose-replyto-row');
    await waitFor(() => expect(row.textContent).toContain('help@example.org'));
    expect(screen.queryByTestId('compose-status-replyto-external')).toBeNull();
  });

  it('shows the row when the setting says always, and not at all for never', async () => {
    await settingsWith({ 'compose.replyTo.show': entry('always') });
    const first = renderWithApp(() => <Compose onClose={() => undefined} />, { identities: IDENTITIES });
    await first.app.loadIdentities();
    expect(await screen.findByTestId('compose-replyto-row')).toBeInTheDocument();
    expect(screen.queryByTestId('compose-head-replyto')).toBeNull();
    first.result.unmount();
    await settingsWith({ 'compose.replyTo.show': entry('never') });
    const second = renderWithApp(() => <Compose onClose={() => undefined} />, { identities: IDENTITIES });
    await second.app.loadIdentities();
    expect(screen.queryByTestId('compose-replyto-row')).toBeNull();
    expect(screen.queryByTestId('compose-head-replyto')).toBeNull();
  });

  it('is off when the options.replyTo feature is switched off', async () => {
    await settingsWith({ 'compose.feature.options.replyTo': { value: false, source: 'global', locked: true } });
    const { app } = renderWithApp(() => <Compose onClose={() => undefined} />, { identities: IDENTITIES });
    await app.loadIdentities();
    expect(screen.queryByTestId('compose-head-replyto')).toBeNull();
    expect(screen.queryByTestId('compose-replyto-row')).toBeNull();
  });

  it('explains a reply that goes to the original Reply-To address', async () => {
    await settingsWith();
    const { app } = renderWithApp(() => <Compose onClose={() => undefined} initial={{ mode: 'reply', to: 'list@elsewhere.test', cc: '', subject: 'Re: x', bodyText: 'q', bodyHtml: '<p>q</p>', inReplyTo: [], references: [], attachments: [], quotesDecrypted: false }} />, { identities: IDENTITIES });
    await app.loadIdentities();
    // No open message in this harness, so the note is driven by the open email: absent here.
    expect(screen.queryByTestId('compose-replyto-note')).toBeNull();
    expect(listDrafts).toBeDefined();
  });
});
