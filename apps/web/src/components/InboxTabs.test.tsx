import { describe, it, expect, beforeEach, afterEach } from 'vitest';
import { screen, fireEvent, waitFor } from '@solidjs/testing-library';
import { InboxTabs } from './InboxTabs.tsx';
import { mkEmail, renderWithApp } from './appHarness.tsx';
import { clearSettings, loadSettings, type Fetcher } from '../api/settings.ts';

const json = (status: number, body: unknown): Response =>
  new Response(JSON.stringify(body), { status, headers: { 'content-type': 'application/json' } });

/** The settings map with `mail.focusedInbox` as given (absent when undefined). */
async function focusedSetting(entry?: { value: unknown; locked?: boolean; source?: string }): Promise<void> {
  const settings = entry ? { 'mail.focusedInbox': { source: 'default', locked: false, ...entry } } : {};
  await loadSettings({ refresh: true, fetcher: (async () => json(200, { settings })) as Fetcher });
}

async function signedIn() {
  const harness = renderWithApp(() => <InboxTabs />, {
    emails: [
      mkEmail('person', { from: [{ name: 'A Person', email: 'a@example.org' }], keywords: {} }),
      mkEmail('news', { from: [{ name: null, email: 'newsletter@shop.example' }], keywords: {} }),
    ],
  });
  await harness.app.login({ username: 'me@example.org', password: 'p' });
  await waitFor(() => expect(harness.app.selectedMailboxId()).toBe('inbox'));
  return harness;
}

describe('InboxTabs (t73: mail.focusedInbox, off by default)', () => {
  beforeEach(() => { localStorage.clear(); clearSettings(); });
  afterEach(() => clearSettings());

  it('renders nothing by default: no tabs and no enable or turn-off button', async () => {
    const { app } = await signedIn();
    await focusedSetting({ value: false });
    expect(app.focusedInbox()).toBe(false);
    expect(screen.queryByTestId('inbox-tabs')).toBeNull();
    expect(screen.queryByRole('tab')).toBeNull();
    expect(screen.queryByRole('button')).toBeNull();
  });

  it('renders nothing against a server that does not know the key', async () => {
    const { app } = await signedIn();
    await focusedSetting();
    expect(app.focusedInbox()).toBe(false);
    expect(screen.queryByRole('tab')).toBeNull();
  });

  it('shows one compact Focused / Other pair with counts once the preference is on, live', async () => {
    const { app } = await signedIn();
    await focusedSetting({ value: false });
    expect(screen.queryByRole('tablist')).toBeNull();
    await focusedSetting({ value: true, source: 'user' });
    const list = await screen.findByRole('tablist', { name: 'Inbox filter' });
    const focused = screen.getByRole('tab', { name: /Focused/ });
    const other = screen.getByRole('tab', { name: /Other/ });
    expect(list.children).toHaveLength(2);
    expect(focused).toHaveAttribute('aria-selected', 'true');
    expect(focused.textContent).toContain('1');
    expect(other.textContent).toContain('1');
    // No switch in the header: turning it off happens in Settings.
    expect(screen.queryByRole('button', { name: /turn off|focused inbox/i })).toBeNull();

    fireEvent.click(other);
    expect(app.inboxTab()).toBe('other');
    expect(other).toHaveAttribute('aria-selected', 'true');
    expect(app.listMessages().map((m) => m.id)).toEqual(['news']);
    fireEvent.keyDown(other, { key: 'ArrowLeft' });
    expect(app.inboxTab()).toBe('focused');
    expect(app.listMessages().map((m) => m.id)).toEqual(['person']);

    // Turned off elsewhere (Settings, another tab, an administrator lock): the tabs go.
    await focusedSetting({ value: false, locked: true, source: 'global' });
    expect(screen.queryByRole('tablist')).toBeNull();
    expect(app.listMessages().map((m) => m.id)).toEqual(['person', 'news']);
  });

  it('is an Inbox view: another folder shows no tabs and every message', async () => {
    const { app } = await signedIn();
    await focusedSetting({ value: true, source: 'user' });
    expect(await screen.findByRole('tablist')).toBeInTheDocument();
    await app.selectMailbox('archive');
    await waitFor(() => expect(screen.queryByRole('tablist')).toBeNull());
    expect(app.focusedInboxShown()).toBe(false);
  });

  it('offers no unified-inbox control and the store has no such flag', async () => {
    const { app } = await signedIn();
    await focusedSetting({ value: true, source: 'user' });
    await screen.findByRole('tablist');
    expect(screen.queryByRole('checkbox')).toBeNull();
    expect(screen.queryByText(/unified/i)).toBeNull();
    expect('unifiedInbox' in app).toBe(false);
    expect('setUnifiedInbox' in app).toBe(false);
  });
});
