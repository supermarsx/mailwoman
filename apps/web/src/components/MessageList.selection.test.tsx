import { describe, it, expect, beforeEach, afterEach, vi, type MockedFunction } from 'vitest';
import { cleanup, fireEvent, render, screen, waitFor, within } from '@solidjs/testing-library';
import { MessageList } from './MessageList.tsx';
import { makeClient, mkEmail } from './appHarness.tsx';
import { createAppState, type AppState } from '../state/store.ts';
import { AppContext } from '../state/context.ts';
import { getKeymap } from '../keymap/index.ts';
import { DEFAULT_PREFS, savePrefs } from '../screens/Settings/prefs.ts';
import type { Client } from '../api/client.ts';
import type { Email, JmapRequest } from '../api/jmap-types.ts';
import * as barCss from './selectionBar.css.ts';

// Selection and the bulk bar in the message list (t28-e16). The harness answers
// every `Email/get` with the whole list, so which message a click or a shortcut
// opens is read from the `openMessage` call, not from `openEmail()`.

const EMAILS: Email[] = ['a', 'b', 'c', 'd', 'e'].map((id) => mkEmail(id, { subject: `Subject ${id}` }));

async function setup(emails: Email[] = EMAILS): Promise<{ app: AppState; jmap: MockedFunction<Client['jmap']> }> {
  const client = makeClient({ emails });
  const app = createAppState(client);
  render(() => (
    <AppContext.Provider value={app}>
      <MessageList />
    </AppContext.Provider>
  ));
  await app.login({ username: 'me@example.org', password: 'p' });
  await waitFor(() => expect(app.messages().length).toBe(emails.length));
  return { app, jmap: vi.mocked(client.jmap) };
}

const row = (subject: string): HTMLElement =>
  [...document.querySelectorAll<HTMLElement>('.list__row')].find((el) => el.textContent?.includes(subject))!;

const selectedSubjects = (): string[] =>
  [...document.querySelectorAll<HTMLElement>('.list__row--selected .list__subject')].map((el) => el.textContent ?? '');

beforeEach(() => localStorage.clear());
afterEach(() => {
  cleanup();
  localStorage.clear();
});

describe('MessageList selection', () => {
  it('keeps the bulk bar open while selection mode is active, including after selecting none', async () => {
    const { app } = await setup();
    const toggle = screen.getByRole('button', { name: 'Select messages' });
    fireEvent.click(toggle);
    expect(toggle).toHaveAttribute('aria-pressed', 'true');
    expect(toggle.querySelector('[data-icon="bulkSelect"]')).not.toBeNull();
    expect(screen.getByTestId('bulk-bar')).toBeInTheDocument();
    expect(screen.getByTestId('bulk-delete')).toBeDisabled();
    fireEvent.click(row('Subject a'));
    expect(screen.getByTestId('bulk-delete')).toBeEnabled();
    fireEvent.click(row('Subject a'));
    expect(app.selectedIds().size).toBe(0);
    expect(screen.getByTestId('bulk-bar')).toBeInTheDocument();
    fireEvent.click(screen.getByTestId('bulk-select-all'));
    expect(app.selectedIds().size).toBe(EMAILS.length);
    fireEvent.click(screen.getByTestId('bulk-select-all'));
    expect(app.selectedIds().size).toBe(0);
    expect(toggle).toHaveAttribute('aria-pressed', 'true');
    expect(screen.getByTestId('bulk-bar')).toBeInTheDocument();
    fireEvent.click(toggle);
    expect(screen.queryByTestId('bulk-bar')).not.toBeInTheDocument();
  });
  it('selects by ordinary taps in explicit selection mode and clears back to opening messages',async()=>{
    const {app}=await setup();const open=vi.spyOn(app,'openMessage');
    fireEvent.click(screen.getByRole('button',{name:'Select messages'}));fireEvent.click(row('Subject a'));fireEvent.click(row('Subject b'));
    expect([...app.selectedIds()]).toEqual(['a','b']);expect(open).not.toHaveBeenCalled();
    fireEvent.click(screen.getByRole('button',{name:'Clear selection'}));fireEvent.click(row('Subject c'));expect(open).toHaveBeenCalledWith('c');
  });
  it('Ctrl+click toggles rows without opening them; a plain click opens and clears', async () => {
    const { app } = await setup();
    const open = vi.spyOn(app, 'openMessage');
    fireEvent.click(row('Subject b'), { ctrlKey: true });
    fireEvent.click(row('Subject d'), { metaKey: true });
    expect(selectedSubjects()).toEqual(['Subject b', 'Subject d']);
    expect(open).not.toHaveBeenCalled();
    // Marked without relying on colour: a check mark and screen-reader text.
    expect(within(row('Subject b')).getByText('Selected')).toBeInTheDocument();
    expect(row('Subject b').querySelector('svg[data-icon="check"]')).not.toBeNull();

    fireEvent.click(row('Subject d'), { ctrlKey: true });
    expect(selectedSubjects()).toEqual(['Subject b']);

    fireEvent.click(row('Subject e'));
    expect(selectedSubjects()).toEqual([]);
    expect(open).toHaveBeenCalledWith('e');
  });

  it('Shift+click selects the range from the last clicked row', async () => {
    await setup();
    fireEvent.click(row('Subject b'), { ctrlKey: true });
    fireEvent.click(row('Subject d'), { shiftKey: true });
    expect(selectedSubjects()).toEqual(['Subject b', 'Subject c', 'Subject d']);
  });

  it('keyboard: Space toggles, Shift+ArrowDown extends, Ctrl+A selects all, Escape clears', async () => {
    await setup();
    const first = row('Subject a');
    first.focus();
    fireEvent.keyDown(first, { key: ' ' });
    expect(selectedSubjects()).toEqual(['Subject a']);
    fireEvent.keyDown(first, { key: 'ArrowDown', shiftKey: true });
    fireEvent.keyDown(first, { key: 'ArrowDown', shiftKey: true });
    expect(selectedSubjects()).toEqual(['Subject a', 'Subject b', 'Subject c']);
    fireEvent.keyDown(first, { key: 'a', ctrlKey: true });
    expect(selectedSubjects()).toHaveLength(5);
    fireEvent.keyDown(first, { key: 'Escape' });
    expect(selectedSubjects()).toEqual([]);
  });

  it('the floating selection bar appears over the list without replacing the toolbar, and Archive is one request for all of them', async () => {
    const { app, jmap } = await setup();
    expect(screen.queryByTestId('bulk-bar')).toBeNull();
    const scroller = document.querySelector<HTMLElement>('.list__scroll')!;
    expect(scroller.className).not.toContain(barCss.listInset);

    for (const s of ['Subject a', 'Subject c', 'Subject e']) fireEvent.click(row(s), { ctrlKey: true });
    const bar = screen.getByRole('toolbar', { name: 'Selection actions' });
    expect(within(bar).getByTestId('bulk-count').textContent).toBe('3 selected');
    // Non-modal overlay: the view toolbar stays, the bar sits after the rows in
    // the pane, and the list gets bottom padding so the last row clears it.
    expect(screen.getByRole('group', { name: 'View options' })).toBeInTheDocument();
    expect(bar.parentElement).toBe(document.querySelector('section.list'));
    expect(scroller.compareDocumentPosition(bar) & Node.DOCUMENT_POSITION_FOLLOWING).toBeTruthy();
    expect(scroller.className).toContain(barCss.listInset);

    jmap.mockClear();
    fireEvent.click(within(bar).getByRole('button', { name: 'Archive' }));
    await waitFor(() => expect(app.messages().map((m) => m.id)).toEqual(['b', 'd']));
    const sets = jmap.mock.calls
      .map((c) => (c[0] as JmapRequest).methodCalls)
      .filter((calls) => calls.some((c) => c[0] === 'Email/set'));
    expect(sets).toHaveLength(1);
    expect(sets[0]![0]![1]['update']).toEqual({
      a: { mailboxIds: { archive: true } },
      c: { mailboxIds: { archive: true } },
      e: { mailboxIds: { archive: true } },
    });
    // The selection went with the messages, so the bar and the padding are gone.
    expect(screen.queryByTestId('bulk-bar')).toBeNull();
    expect(scroller.className).not.toContain(barCss.listInset);
  });

  it('the list stays interactive under the bar: clicks, ranges and keys change the selection, and focus stays on the row', async () => {
    const { app } = await setup();
    const first = row('Subject a');
    first.focus();
    fireEvent.keyDown(first, { key: ' ' });
    expect(screen.getByTestId('bulk-count').textContent).toBe('1 selected');
    // The bar appeared; the row kept focus (t51).
    expect(document.activeElement).toBe(first);
    fireEvent.keyDown(first, { key: 'ArrowDown', shiftKey: true });
    expect(screen.getByTestId('bulk-count').textContent).toBe('2 selected');
    fireEvent.click(row('Subject e'), { ctrlKey: true });
    fireEvent.click(row('Subject a'), { ctrlKey: true });
    expect(selectedSubjects()).toEqual(['Subject b', 'Subject e']);
    fireEvent.click(row('Subject d'), { shiftKey: true });
    expect(selectedSubjects()).toEqual(['Subject a', 'Subject b', 'Subject c', 'Subject d']);
    // Select all / Select none from the bar.
    fireEvent.click(screen.getByRole('button', { name: 'Select all' }));
    expect(app.selectedIds().size).toBe(5);
    fireEvent.click(screen.getByRole('button', { name: 'Select none' }));
    expect(app.selectedIds().size).toBe(0);
    expect(screen.queryByTestId('bulk-bar')).toBeNull();
  });

  it('F6 moves focus from the list to the bar and Esc returns it to the focused row', async () => {
    await setup();
    row('Subject a').focus();
    fireEvent.keyDown(row('Subject a'), { key: 'ArrowDown' });
    await waitFor(() => expect(document.activeElement).toBe(row('Subject b')));
    fireEvent.keyDown(row('Subject b'), { key: ' ' });
    expect(selectedSubjects()).toEqual(['Subject b']);
    fireEvent.keyDown(document.activeElement!, { key: 'F6' });
    const bar = screen.getByRole('toolbar', { name: 'Selection actions' });
    await waitFor(() => expect(bar.contains(document.activeElement)).toBe(true));
    fireEvent.keyDown(document.activeElement!, { key: 'Escape' });
    await waitFor(() => expect(document.activeElement).toBe(row('Subject b')));
    // Esc in the list (not the bar) clears the selection, as before.
    fireEvent.keyDown(document.activeElement!, { key: 'Escape' });
    expect(screen.queryByTestId('bulk-bar')).toBeNull();
  });

  it('Move to folder opens a folder menu and moves the selection with one request', async () => {
    const { app, jmap } = await setup();
    fireEvent.click(row('Subject a'), { ctrlKey: true });
    fireEvent.click(row('Subject b'), { ctrlKey: true });
    jmap.mockClear();
    fireEvent.click(screen.getByRole('button', { name: 'Move to folder' }));
    const menu = await screen.findByRole('menu', { name: 'Move to folder' });
    // The current folder is not offered as a destination.
    expect(within(menu).queryByRole('menuitem', { name: 'Inbox' })).toBeNull();
    fireEvent.click(within(menu).getByRole('menuitem', { name: 'Trash' }));
    await waitFor(() => expect(app.messages().map((m) => m.id)).toEqual(['c', 'd', 'e']));
    const sets = jmap.mock.calls.filter((c) => (c[0] as JmapRequest).methodCalls.some((m) => m[0] === 'Email/set'));
    expect(sets).toHaveLength(1);
  });

  it('a flagged message shows its flag on the row', async () => {
    await setup([mkEmail('f', { subject: 'Flagged one', keywords: { $seen: true, $flagged: true } })]);
    expect(within(row('Flagged one')).getByRole('img', { name: 'Flagged' })).toBeInTheDocument();
  });

  it('the next shortcut opens the following message through the one dispatcher', async () => {
    savePrefs({ ...DEFAULT_PREFS, keyboardPreset: 'gmail' });
    const { app } = await setup();
    const open = vi.spyOn(app, 'openMessage').mockResolvedValue(undefined);
    getKeymap(); // the app's dispatcher, listening on the document
    fireEvent.click(row('Subject b'));
    expect(open).toHaveBeenLastCalledWith('b');
    fireEvent.keyDown(document.body, { key: 'j' });
    expect(open).toHaveBeenLastCalledWith('c');
    fireEvent.keyDown(document.body, { key: 'j' });
    expect(open).toHaveBeenLastCalledWith('d');
    fireEvent.keyDown(document.body, { key: 'k' });
    expect(open).toHaveBeenLastCalledWith('c');
    // With shortcuts off the same key is not a shortcut.
    savePrefs({ ...DEFAULT_PREFS, keyboardPreset: 'off' });
    open.mockClear();
    fireEvent.keyDown(document.body, { key: 'j' });
    expect(open).not.toHaveBeenCalled();
  });
});
