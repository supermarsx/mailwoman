import { describe, it, expect, beforeEach, vi } from 'vitest';
import { render, screen, fireEvent, waitFor } from '@solidjs/testing-library';
import { Compose } from './Compose.tsx';
import { makeClient } from './appHarness.tsx';
import { createAppState } from '../state/store.ts';
import { AppContext } from '../state/context.ts';

// The rich editor's chunk never arrives in this file: the dynamic import stays
// pending for the life of the test, which is what a slow or stalled network
// looks like to `lazy()`. The composer then shows its plain textarea in the
// Suspense fallback while still in rich mode, with no editor handle.
//
// That is one of the three "editor is not there" states the send path has to
// build the body from the textarea in. (The failed-import state cannot be
// reached under vitest — see the comment at the AsyncBoundary in Compose.tsx —
// and is covered in a browser by e2e/offline.spec.ts.)
vi.mock('./compose/RichTextEditor.tsx', () => new Promise(() => undefined));

describe('Compose — the rich editor chunk has not arrived', () => {
  beforeEach(() => localStorage.clear());

  it('sends what was typed into the fallback textarea, not an empty body', async () => {
    const client = makeClient();
    const app = createAppState(client);
    const onClose = vi.fn();
    render(() => (
      <AppContext.Provider value={app}>
        <Compose onClose={onClose} />
      </AppContext.Provider>
    ));
    await app.login({ jmapUrl: 'x', username: 'me@example.org', password: 'p' });

    // Precondition: still in rich mode (the toggle offers "Plain text"), the
    // editor is absent, and Body is the fallback textarea.
    expect(screen.getByTestId('format-toggle')).toHaveTextContent('Plain text');
    expect(screen.queryByTestId('compose-richtext')).toBeNull();
    const body = screen.getByLabelText('Body') as HTMLTextAreaElement;
    expect(body.tagName).toBe('TEXTAREA');

    fireEvent.input(screen.getByLabelText('To'), { target: { value: 'you@example.org' } });
    fireEvent.input(screen.getByLabelText('Subject'), { target: { value: 'Hi' } });
    fireEvent.input(body, { target: { value: 'typed in the fallback\nsecond line' } });
    fireEvent.click(screen.getByRole('button', { name: 'Send' }));
    await waitFor(() => expect(onClose).toHaveBeenCalledTimes(1));

    const sends = vi
      .mocked(client.jmap)
      .mock.calls.map((c) => c[0])
      .filter((r) => r.methodCalls.some((m) => m[0] === 'EmailSubmission/set' && 'create' in m[1]));
    expect(sends).toHaveLength(1);
    const create = sends[0]!.methodCalls[0]![1]['create'] as Record<string, Record<string, unknown>>;
    expect(create['draft']!['bodyValues']).toEqual({
      body: { value: '<p>typed in the fallback<br>second line</p>' },
    });
  });

  it('auto-saves the typed text, not an empty rich body', async () => {
    vi.useFakeTimers();
    try {
      const app = createAppState(makeClient());
      render(() => (
        <AppContext.Provider value={app}>
          <Compose onClose={() => undefined} />
        </AppContext.Provider>
      ));
      fireEvent.input(screen.getByLabelText('Body'), { target: { value: 'draft text' } });
      vi.advanceTimersByTime(1000);
      const stored = JSON.parse(localStorage.getItem('mw.compose.drafts.v1') ?? '[]') as {
        bodyHtml: string;
        bodyText: string;
      }[];
      expect(stored).toHaveLength(1);
      expect({ bodyHtml: stored[0]!.bodyHtml, bodyText: stored[0]!.bodyText }).toEqual({
        bodyHtml: '<p>draft text</p>',
        bodyText: 'draft text',
      });
    } finally {
      vi.useRealTimers();
    }
  });
});
