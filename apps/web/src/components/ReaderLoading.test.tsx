import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import { render, screen, waitFor } from '@solidjs/testing-library';
import { Reader } from './Reader.tsx';
import { makeClient, mkEmail } from './appHarness.tsx';
import { createAppState } from '../state/store.ts';
import { AppContext } from '../state/context.ts';
import { createMessageView } from '../state/slices/messageView.ts';
import { clearSettings } from '../api/settings.ts';
import { clearFeatures } from '../state/features.ts';
import type { Email } from '../api/jmap-types.ts';

const mk = (id: string, subject: string, body: string): Email => mkEmail(id, { subject, from: [{ name: 'Sender', email: 'sender@example.test' }], preview: `preview ${id}`, htmlBody: [{ partId: '1', blobId: null, size: body.length, type: 'text/html' }], bodyValues: { '1': { value: body } } });
beforeEach(() => { clearFeatures(); clearSettings(); });
afterEach(() => { vi.restoreAllMocks(); });

async function setup() {
  const emails = [mk('a', 'Subject A', '<p>Body A secret</p>'), mk('b', 'Subject B', '<p>Body B</p>')];
  const app = createAppState(makeClient({ emails }));
  await app.login({ username: 'me@example.org', password: 'p' });
  const gates = new Map<string, (value: { email: Email; html: string }) => void>();
  const view = createMessageView({
    account: app.accountId,
    load: (_a, id) => new Promise((resolve) => { gates.set(id, resolve); }),
    readDelay: () => null,
    markRead: vi.fn(),
  });
  render(() => <AppContext.Provider value={app}><Reader view={view} /></AppContext.Provider>);
  return { app, view, gates, emails };
}

describe('Reader loading state', () => {
  it('shows the empty text only when nothing is open and nothing is loading', async () => {
    await setup();
    expect(screen.getByText('Select a message to read')).toBeInTheDocument();
    expect(document.querySelector('[data-testid="reader-pending"]')).toBeNull();
  });

  it('shows the list entry header and a busy status, not the empty text, while loading; never the previous body', async () => {
    const { view, gates, emails } = await setup();
    void view.openMessage('a');
    await waitFor(() => expect(document.querySelector('[data-testid="reader-pending"]')).not.toBeNull());
    expect(screen.queryByText('Select a message to read')).toBeNull();
    expect(document.querySelector('[data-testid="reader-pending"]')?.getAttribute('aria-busy')).toBe('true');
    expect(screen.getByText('Subject A')).toBeInTheDocument();
    expect(screen.getByRole('status')).toBeInTheDocument();
    gates.get('a')!({ email: emails[0]!, html: '<p>Body A secret</p>' });
    await waitFor(() => expect(view.openEmail()?.id).toBe('a'));
    // Switch to B: A's subject/body must not stay under B's id.
    // The reader stays mounted (toolbar, header, frame); only the body region shows the pending state.
    const header = document.querySelector('.reader__header');
    void view.openMessage('b');
    await waitFor(() => expect(view.pendingId()).toBe('b'));
    expect(document.querySelector('.reader__header')).toBe(header);
    expect(document.querySelector('[data-testid="reader-pending"]')).toBeNull();
    await waitFor(() => expect(screen.getAllByRole('status').length).toBeGreaterThan(0));
    expect(screen.queryByText('Select a message to read')).toBeNull();
    expect(screen.queryByText(/Body A secret/)).toBeNull();
    expect(screen.queryByText('Subject A')).toBeNull();
    expect(screen.getByText('Subject B')).toBeInTheDocument();
  });

  it('exposes pendingId while loading and clears it afterwards', async () => {
    const { view, gates, emails } = await setup();
    const done = view.openMessage('b');
    expect(view.pendingId()).toBe('b');
    gates.get('b')!({ email: emails[1]!, html: '<p>x</p>' });
    await done;
    expect(view.pendingId()).toBeNull();
  });
});
