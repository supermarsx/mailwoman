import { describe, it, expect, beforeEach, afterEach } from 'vitest';
import { cleanup, fireEvent, render, waitFor } from '@solidjs/testing-library';
import { MessageList } from './MessageList.tsx';
import { makeClient, mkEmail } from './appHarness.tsx';
import { createAppState, type AppState } from '../state/store.ts';
import { AppContext } from '../state/context.ts';
import type { Email } from '../api/jmap-types.ts';

// t44: arrow keys in the message list keep working after the list updates.
// Opening a message marks it read about a second later, and every sync or flag
// change replaces the list's messages. Each replace used to remount every row
// (the rows were keyed by object identity and rebuilt from scratch), so the
// focused row left the document, focus fell to <body> and the list's own
// keydown handler never saw another arrow key.

const EMAILS: Email[] = ['a', 'b', 'c', 'd', 'e'].map((id) => mkEmail(id, { subject: `Subject ${id}`, keywords: {} }));

async function setup(): Promise<AppState> {
  const app = createAppState(makeClient({ emails: EMAILS }));
  render(() => (
    <AppContext.Provider value={app}>
      <MessageList />
    </AppContext.Provider>
  ));
  await app.login({ username: 'me@example.org', password: 'p' });
  await waitFor(() => expect(app.messages().length).toBe(EMAILS.length));
  return app;
}

const row = (subject: string): HTMLElement =>
  [...document.querySelectorAll<HTMLElement>('.list__row')].find((el) => el.textContent?.includes(subject))!;
const focusedSubject = (): string | null | undefined =>
  (document.activeElement as HTMLElement | null)?.querySelector('.list__subject')?.textContent;
const tick = (): Promise<void> => new Promise((r) => setTimeout(r, 0));

beforeEach(() => localStorage.clear());
afterEach(() => {
  cleanup();
  localStorage.clear();
});

describe('MessageList arrow keys across list updates', () => {
  it('keeps the focused row (same element) when a message in the list changes', async () => {
    const app = await setup();
    const first = row('Subject a');
    first.focus();
    fireEvent.keyDown(first, { key: 'ArrowDown' });
    await tick();
    expect(focusedSubject()).toBe('Subject b');
    const focused = document.activeElement;

    // The focused message is marked read (what the open-delay does), then another one.
    await app.markRead('b', true);
    await app.markRead('d', true);
    await tick();
    expect(document.activeElement).toBe(focused);
    expect(row('Subject b').classList.contains('list__row--unread')).toBe(false);

    fireEvent.keyDown(document.activeElement!, { key: 'ArrowDown' });
    await tick();
    expect(focusedSubject()).toBe('Subject c');
  });

  it('a focused row that leaves the list hands focus to the row now in its place', async () => {
    const app = await setup();
    const b = row('Subject b');
    b.focus();
    fireEvent.keyDown(row('Subject a'), { key: 'ArrowDown' });
    await tick();
    expect(focusedSubject()).toBe('Subject b');
    await app.trashMessage('b');
    await tick();
    await waitFor(() => expect(focusedSubject()).toBe('Subject c'));
    fireEvent.keyDown(document.activeElement!, { key: 'ArrowUp' });
    await tick();
    expect(focusedSubject()).toBe('Subject a');
  });

  it('PageDown and PageUp move by a page, clamped to the list', async () => {
    await setup();
    row('Subject a').focus();
    fireEvent.keyDown(document.activeElement!, { key: 'PageDown' });
    await tick();
    expect(focusedSubject()).toBe('Subject e');
    fireEvent.keyDown(document.activeElement!, { key: 'PageUp' });
    await tick();
    expect(focusedSubject()).toBe('Subject a');
  });

  it('the roving cursor follows the focused message when rows above it are removed', async () => {
    const app = await setup();
    const c = row('Subject c');
    c.focus();
    fireEvent.click(c);
    await tick();
    await app.trashMessage('a');
    await tick();
    expect(focusedSubject()).toBe('Subject c');
    expect(row('Subject c').getAttribute('tabindex')).toBe('0');
    fireEvent.keyDown(document.activeElement!, { key: 'ArrowDown' });
    await tick();
    expect(focusedSubject()).toBe('Subject d');
  });
});
