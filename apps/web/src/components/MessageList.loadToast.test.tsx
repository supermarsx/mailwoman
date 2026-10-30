// t111: a folder load under rows already on screen (the old folder's rows stay
// while the next one loads) reports itself in the corner toast
// (after 400 ms, once), not in a box drawn inside the list; a fast reload shows
// nothing; an empty folder keeps its skeleton and gets no toast.
import { afterEach, describe, expect, it } from 'vitest';
import { cleanup, render, screen, waitFor } from '@solidjs/testing-library';
import { MessageList } from './MessageList.tsx';
import { makeClient, mkEmail } from './appHarness.tsx';
import { createAppState } from '../state/store.ts';
import { AppContext } from '../state/context.ts';
import { suspendAccountPreferences } from '../api/accountPreferenceLifecycle.ts';
import { ToastRegion, resetToasts, toasts } from './toast/index.ts';
import type { JmapRequest, JmapResponse } from '../api/jmap-types.ts';

afterEach(() => { cleanup(); resetToasts(); suspendAccountPreferences(); localStorage.clear(); });

async function mount(emails: ReturnType<typeof mkEmail>[]) {
  const client = makeClient({ emails });
  const real = client.jmap as (r: JmapRequest) => Promise<JmapResponse>;
  const gate = { hold: false, release: () => undefined as void };
  client.jmap = (async (r: JmapRequest) => {
    if (gate.hold && r.methodCalls.some((c) => c[0] === 'Email/get')) {
      await new Promise<void>((done) => { gate.release = done; });
    }
    return real(r);
  }) as typeof client.jmap;
  const app = createAppState(client);
  render(() => <AppContext.Provider value={app}><MessageList /><ToastRegion /></AppContext.Provider>);
  await app.login({ username: 'me@example.org', password: 'p' });
  return { app, gate };
}
const loadingToasts = () => toasts().filter((i) => i.id === 'mail-list-loading');
const sleep = (ms: number) => new Promise((done) => setTimeout(done, ms));

describe('MessageList background loading toast', () => {
  it('a slow folder load under listed rows shows one busy toast after 400 ms and closes it when done', async () => {
    const { app, gate } = await mount([mkEmail('a', { subject: 'Already here' })]);
    await screen.findByText('Already here');
    await waitFor(() => expect(app.listLoading()).toBe(false));
    gate.hold = true;
    const reload = app.selectMailbox('archive');
    await waitFor(() => expect(app.listLoading()).toBe(true));
    await sleep(250);
    expect(loadingToasts()).toHaveLength(0);
    await sleep(250);
    expect(loadingToasts()).toHaveLength(1);
    expect(loadingToasts()[0]).toMatchObject({ tone: 'info', busy: true, message: 'Loading messages…' });
    const el = screen.getByTestId('mail-loading-toast');
    expect(el.closest('[aria-live="polite"]')).not.toBeNull();
    // The list itself draws no notice of its own any more.
    expect(document.querySelector('.list__loading')).toBeNull();
    expect(screen.getByText('Already here')).toBeTruthy();
    gate.hold = false; gate.release();
    await reload;
    await waitFor(() => expect(loadingToasts()).toHaveLength(0));
  });

  it('a fast folder load shows no toast at all', async () => {
    const { app } = await mount([mkEmail('a', { subject: 'Already here' })]);
    await screen.findByText('Already here');
    await waitFor(() => expect(app.listLoading()).toBe(false));
    await app.selectMailbox('archive');
    await sleep(500);
    expect(loadingToasts()).toHaveLength(0);
  });

  it('an empty folder keeps its skeleton and raises no toast', async () => {
    const { app, gate } = await mount([]);
    gate.hold = true;
    const reload = app.selectMailbox('archive');
    await waitFor(() => expect(app.listLoading()).toBe(true));
    await sleep(500);
    expect(screen.getByTestId('async-pending')).toBeTruthy();
    expect(loadingToasts()).toHaveLength(0);
    gate.hold = false; gate.release();
    await reload;
  });
});
