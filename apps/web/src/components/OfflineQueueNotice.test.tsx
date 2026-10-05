import { describe, it, expect, beforeEach, vi } from 'vitest';
import { render, screen, fireEvent, waitFor, within } from '@solidjs/testing-library';
import type { JSX } from 'solid-js';
import { OfflineQueueNotice } from './OfflineQueueNotice.tsx';
import { MessageList } from './MessageList.tsx';
import { makeClient, mkEmail, type HarnessOpts } from './appHarness.tsx';
import { AppContext } from '../state/context.ts';
import { createAppState } from '../state/store.ts';
import { stripIsolates } from '../i18n/index.ts';
import type { JmapRequest, JmapResponse } from '../api/jmap-types.ts';
import type { AppState } from '../state/store.ts';
import type { Client } from '../api/client.ts';

const move = { accountId: 'acct1', emailId: 'm1', mailboxIds: { archive: true } };
const send = {
  accountId: 'acct1',
  draft: {
    from: { name: null, email: 'me@example.org' },
    to: 'bob',
    subject: 'Quarterly numbers',
    htmlBody: '<p>x</p>',
    draftMailboxId: 'inbox',
  },
};

/** As the harness's `renderWithApp`, but handing back the client too. */
function renderWithApp(ui: () => JSX.Element, opts: HarnessOpts = {}): { app: AppState; client: Client } {
  const client = makeClient(opts);
  const app = createAppState(client);
  render(() => <AppContext.Provider value={app}>{ui()}</AppContext.Provider>);
  return { app, client };
}

/** Whether a request is a replay of a queued mutation (not a list or a lookup). */
const isReplay = (r: JmapRequest): boolean =>
  r.methodCalls.some((m) => (m[0] === 'Email/set' && ('update' in m[1] || 'create' in m[1])));

/** Make the server refuse every replay until `accept()` is called. */
function refuseReplays(client: Client): { accept(): void; replays(): number } {
  const answer = vi.mocked(client.jmap).getMockImplementation()!;
  let accepting = false;
  let count = 0;
  vi.mocked(client.jmap).mockImplementation(async (body, opts): Promise<JmapResponse> => {
    if (!isReplay(body)) return answer(body, opts);
    count += 1;
    if (body.methodCalls.length === 2) {
      // A send: the engine's refusal of a draft with a bad recipient.
      return accepting
        ? answer(body, opts)
        : {
            methodResponses: [
              ['Email/set', { created: null, notCreated: { draft: { type: 'invalidProperties', description: 'to: "bob": no @' } } }, 'set'],
              ['EmailSubmission/set', { created: null, notCreated: { send: { type: 'invalidProperties' } } }, 'submit'],
            ],
            sessionState: 's',
          };
    }
    return {
      methodResponses: [
        ['Email/set', accepting ? { updated: { m1: null }, notUpdated: null } : { updated: null, notUpdated: { m1: { type: 'notFound' } } }, 'set'],
      ],
      sessionState: 's',
    };
  });
  return { accept: () => (accepting = true), replays: () => count };
}

/** Queue a move and a send, and replay them against a server that refuses both. */
async function twoFailures(app: AppState, client: Client) {
  const server = refuseReplays(client);
  await app.enqueueOffline('move', move);
  await app.enqueueOffline('send', send);
  await app.replayOffline();
  return server;
}

describe('OfflineQueueNotice', () => {
  beforeEach(() => localStorage.clear());

  it('renders nothing while no queued change has failed, including with one still pending', async () => {
    const { app } = renderWithApp(() => <OfflineQueueNotice />);
    expect(screen.queryByTestId('offline-queue-failed')).toBeNull();
    await app.enqueueOffline('move', move);
    expect(app.offlineQueuePending()).toBe(1);
    expect(screen.queryByTestId('offline-queue-failed')).toBeNull();
  });

  it('lists each failed change with what it was and why it failed', async () => {
    const { app, client } = renderWithApp(() => <OfflineQueueNotice />);
    await twoFailures(app, client);

    const notice = await screen.findByRole('region', { name: 'Changes that could not be applied' });
    expect(within(notice).getByRole('heading')).toHaveTextContent('2 changes made offline could not be applied');
    const rows = within(notice)
      .getAllByRole('listitem')
      .map((li) => stripIsolates(li.textContent ?? ''));
    expect(rows).toEqual([
      'Move a messageReason: notFoundRetryDiscard',
      'Send “Quarterly numbers”Reason: to: "bob": no @RetryDiscard',
    ]);
  });

  it('Retry replays that change; once the server applies it, it leaves the notice', async () => {
    const { app, client } = renderWithApp(() => <OfflineQueueNotice />);
    const server = await twoFailures(app, client);
    await screen.findByTestId('offline-queue-failed');
    expect(server.replays()).toBe(2);

    server.accept();
    fireEvent.click(screen.getByRole('button', { name: 'Retry: Move a message' }));

    await waitFor(() => expect(app.offlineFailed().map((i) => i.type)).toEqual(['send']));
    // Only the retried item was replayed; the other failed one was left alone.
    expect(server.replays()).toBe(3);
    expect(screen.getAllByRole('listitem')).toHaveLength(1);
    expect(screen.getByRole('heading')).toHaveTextContent('1 change made offline could not be applied');
  });

  it('Discard removes that change without sending it, and the notice goes when none is left', async () => {
    const { app, client } = renderWithApp(() => <OfflineQueueNotice />);
    const server = await twoFailures(app, client);
    await screen.findByTestId('offline-queue-failed');

    fireEvent.click(screen.getByRole('button', { name: /^Discard: Send/ }));
    await waitFor(() => expect(app.offlineFailed().map((i) => i.type)).toEqual(['move']));
    fireEvent.click(screen.getByRole('button', { name: 'Discard: Move a message' }));
    await waitFor(() => expect(screen.queryByTestId('offline-queue-failed')).toBeNull());

    expect(server.replays()).toBe(2);
    expect(await app.replayOffline()).toEqual({ sent: 0, failed: 0 });
    expect(server.replays()).toBe(2);
  });
});

describe('MessageList — where the notice is shown', () => {
  it('shows the failed-queue notice above the messages', async () => {
    const { app, client } = renderWithApp(() => <MessageList />, { emails: [mkEmail('a')] });
    await app.login({ jmapUrl: 'x', username: 'me@example.org', password: 'p' });
    await waitFor(() => expect(app.messages().length).toBe(1));
    expect(screen.queryByTestId('offline-queue-failed')).toBeNull();

    await twoFailures(app, client);

    const notice = await screen.findByTestId('offline-queue-failed');
    const scroller = document.querySelector('.list__scroll')!;
    expect(notice.compareDocumentPosition(scroller) & Node.DOCUMENT_POSITION_FOLLOWING).toBeTruthy();
  });
});
