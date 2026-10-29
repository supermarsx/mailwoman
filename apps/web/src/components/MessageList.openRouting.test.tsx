import { describe, it, expect, beforeEach, afterEach, vi } from 'vitest';
import { cleanup, fireEvent, render, waitFor } from '@solidjs/testing-library';
import { MessageList } from './MessageList.tsx';
import { makeClient, mkEmail } from './appHarness.tsx';
import { createAppState } from '../state/store.ts';
import { AppContext } from '../state/context.ts';
import { getKeymap } from '../keymap/index.ts';
import type { Email } from '../api/jmap-types.ts';

// t74: the list reports a plain activation and an explicit ask for a tab
// separately; the host (Mailbox) decides pane or tab by the reading pane.

const EMAILS: Email[] = ['a', 'b', 'c'].map((id) => mkEmail(id, { subject: `Subject ${id}` }));

async function mount(withTab = true) {
  getKeymap();
  const client = makeClient({ emails: EMAILS });
  const app = createAppState(client);
  const open = vi.fn<(id: string) => void>(), tab = vi.fn<(id: string) => void>();
  render(() => (
    <AppContext.Provider value={app}>
      {withTab ? <MessageList onOpenMessage={open} onOpenMessageTab={tab} /> : <MessageList onOpenMessage={open} />}
    </AppContext.Provider>
  ));
  await app.login({ username: 'me@example.org', password: 'p' });
  await waitFor(() => expect(app.messages().length).toBe(EMAILS.length));
  return { app, open, tab };
}

const row = (subject: string): HTMLElement =>
  [...document.querySelectorAll<HTMLElement>('.list__row')].find((el) => el.textContent?.includes(subject))!;

beforeEach(() => localStorage.clear());
afterEach(() => { cleanup(); localStorage.clear(); });

describe('MessageList open routing', () => {
  it('a click activates once; the double-click that follows asks for a tab once', async () => {
    const { open, tab } = await mount();
    fireEvent.click(row('Subject b'), { detail: 1 });
    fireEvent.click(row('Subject b'), { detail: 2 });
    fireEvent.dblClick(row('Subject b'), { detail: 2 });
    expect(open.mock.calls).toEqual([['b']]);
    expect(tab.mock.calls).toEqual([['b']]);
  });

  it('Ctrl+click and Shift+click select without opening, and a Ctrl+double-click opens no tab', async () => {
    const { app, open, tab } = await mount();
    fireEvent.click(row('Subject a'), { ctrlKey: true, detail: 1 });
    fireEvent.click(row('Subject a'), { ctrlKey: true, detail: 2 });
    fireEvent.dblClick(row('Subject a'), { ctrlKey: true });
    fireEvent.click(row('Subject c'), { shiftKey: true, detail: 1 });
    expect(open).not.toHaveBeenCalled();
    expect(tab).not.toHaveBeenCalled();
    expect(app.selectedIds().size).toBe(3);
  });

  it('Enter activates the focused row; Shift+Enter opens it in a tab', async () => {
    const { open, tab } = await mount();
    const target = row('Subject c');
    target.focus();
    // A button's Enter is a click with detail 0.
    fireEvent.click(target, { detail: 0 });
    expect(open.mock.calls).toEqual([['c']]);
    const event = new KeyboardEvent('keydown', { key: 'Enter', shiftKey: true, bubbles: true, cancelable: true });
    target.dispatchEvent(event);
    expect(tab.mock.calls).toEqual([['c']]);
    // Handled, so the button's own Shift+Enter click (a range select) does not run.
    expect(event.defaultPrevented).toBe(true);
  });

  it('without a tab handler the explicit ask falls back to the activation', async () => {
    const { open } = await mount(false);
    fireEvent.dblClick(row('Subject a'));
    expect(open.mock.calls).toEqual([['a']]);
  });
});
