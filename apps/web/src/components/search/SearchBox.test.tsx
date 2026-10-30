import { describe, expect, it } from 'vitest';
import { fireEvent, screen, waitFor, within } from '@solidjs/testing-library';
import { renderWithApp, mkEmail } from '../appHarness.tsx';
import { SearchBox } from './SearchBox.tsx';
import { relatedSearchActions } from './ReaderSearchMenu.tsx';
import type { AppState } from '../../state/store.ts';

async function setup(): Promise<{ app: AppState; input: HTMLInputElement }> {
  const { app } = renderWithApp(() => <SearchBox />, { emails: [mkEmail('a', { subject: 'Hello' })] });
  await app.login({ username: 'me@example.org', password: 'p' });
  await waitFor(() => expect(app.mailboxes().length).toBeGreaterThan(0));
  const input = screen.getByRole('searchbox', { name: 'Search mail' }) as HTMLInputElement;
  return { app, input };
}

const bar = (): HTMLElement => screen.getByRole('toolbar', { name: 'Search filters' });
const submit = (input: HTMLInputElement, text: string): void => {
  fireEvent.input(input, { target: { value: text } });
  fireEvent.submit(input.closest('form')!);
};

describe('SearchBox (t88)', () => {
  it('icon buttons: a magnifier submits, × clears; both have names and tooltips', async () => {
    const { app, input } = await setup();
    const go = screen.getByRole('button', { name: 'Search' });
    expect(go).toHaveAttribute('title', expect.stringContaining('Enter'));
    expect(go.querySelector('svg')).not.toBeNull();
    expect(go.textContent?.trim()).toBe('');
    expect(screen.queryByRole('button', { name: 'Clear' })).toBeNull();
    fireEvent.input(input, { target: { value: 'from:x' } });
    fireEvent.click(go);
    await waitFor(() => expect(app.searchActive()).toBe(true));
    const clear = screen.getByRole('button', { name: 'Clear' });
    await waitFor(() => expect(clear).toHaveAttribute('title', expect.stringContaining('Esc')));
    fireEvent.click(clear);
    await waitFor(() => expect(app.searchActive()).toBe(false));
    expect(input.value).toBe('');
    expect(screen.queryByRole('toolbar', { name: 'Search filters' })).toBeNull();
  });

  it('shows the parsed query as chips, follows typing, and Esc clears', async () => {
    const { app, input } = await setup();
    submit(input, 'from:x subject:y');
    await waitFor(() => expect(within(bar()).getByRole('button', { name: /^From: x$/ })).toBeInTheDocument());
    expect(within(bar()).getByRole('button', { name: /^Subject: y$/ })).toBeInTheDocument();
    expect(within(bar()).getByRole('button', { name: /Folder: All folders/ })).toBeInTheDocument();
    // Typing operators updates the chips before the search runs.
    fireEvent.input(input, { target: { value: 'from:x subject:y has:attachment' } });
    await waitFor(() => expect(within(bar()).getByRole('button', { name: /^Has attachment$/ })).toBeInTheDocument());
    fireEvent.keyDown(input, { key: 'Escape' });
    await waitFor(() => expect(app.searchActive()).toBe(false));
    expect(input.value).toBe('');
  });

  it('editing a chip rewrites the input with canonical syntax and searches', async () => {
    const { app, input } = await setup();
    submit(input, 'from:x subject:y');
    const from = await waitFor(() => within(bar()).getByRole('button', { name: /^From: x$/ }));
    fireEvent.click(from);
    const dialog = await screen.findByRole('dialog');
    const field = within(dialog).getByRole('textbox') as HTMLInputElement;
    expect(field.value).toBe('x');
    fireEvent.input(field, { target: { value: 'Ann "A" Lee' } });
    fireEvent.click(within(dialog).getByRole('checkbox', { name: 'Exclude matches' }));
    fireEvent.click(within(dialog).getByRole('button', { name: 'Apply' }));
    await waitFor(() => expect(input.value).toBe('-from:"Ann \\"A\\" Lee" subject:y'));
    expect(app.search()).toBe(input.value);
    expect(screen.queryByRole('dialog')).toBeNull();
    expect(within(bar()).getByRole('button', { name: /^From: not Ann "A" Lee$/ })).toBeInTheDocument();
  });

  it('toggles and removes chips, and removing the last one restores the folder', async () => {
    const { app, input } = await setup();
    submit(input, 'has:attachment subject:y');
    const has = await waitFor(() => within(bar()).getByRole('button', { name: /^Has attachment$/ }));
    fireEvent.click(has);
    await waitFor(() => expect(input.value).toBe('-has:attachment subject:y'));
    await waitFor(() => expect(within(bar()).getByRole('button', { name: /^No attachment$/ })).toBeInTheDocument());
    fireEvent.click(within(bar()).getByRole('button', { name: 'Remove filter: No attachment' }));
    await waitFor(() => expect(input.value).toBe('subject:y'));
    // Focus stays in the row (the next chip's remove button).
    await waitFor(() => expect(document.activeElement).toBe(within(bar()).getByRole('button', { name: 'Remove filter: Subject: y' })));
    fireEvent.click(within(bar()).getByRole('button', { name: 'Remove filter: Subject: y' }));
    await waitFor(() => expect(app.searchActive()).toBe(false));
    expect(input.value).toBe('');
  });

  it('adds filters from the menu and sets the folder scope', async () => {
    const { app, input } = await setup();
    submit(input, 'invoice');
    const add = await waitFor(() => within(bar()).getByRole('button', { name: /Add filter/ }));
    fireEvent.click(add);
    fireEvent.click(await screen.findByRole('menuitem', { name: 'Unread' }));
    await waitFor(() => expect(input.value).toBe('invoice is:unread'));
    fireEvent.click(within(bar()).getByRole('button', { name: /Add filter/ }));
    fireEvent.click(await screen.findByRole('menuitem', { name: 'Date' }));
    const dialog = await screen.findByRole('dialog');
    const [after] = within(dialog).getAllByLabelText(/After/) as HTMLInputElement[];
    fireEvent.input(after!, { target: { value: '2026-01-01' } });
    fireEvent.submit(dialog.querySelector('form')!);
    await waitFor(() => expect(input.value).toBe('invoice is:unread after:2026-01-01'));
    fireEvent.click(within(bar()).getByRole('button', { name: /Folder: All folders/ }));
    const scope = await screen.findByRole('dialog');
    fireEvent.change(within(scope).getByRole('combobox'), { target: { value: 'archive' } });
    fireEvent.click(within(scope).getByRole('button', { name: 'Apply' }));
    await waitFor(() => expect(input.value).toBe('invoice is:unread after:2026-01-01 in:Archive'));
    expect(within(bar()).getByRole('button', { name: /Folder: Archive/ })).toBeInTheDocument();
    expect(app.search()).toBe(input.value);
  });

  it('a search started elsewhere lands in the input and its chips', async () => {
    const { app, input } = await setup();
    await app.searchMessageField('from', 'sender@example.org');
    await waitFor(() => expect(input.value).toBe('from:"sender@example.org"'));
    await waitFor(() => expect(within(bar()).getByRole('button', { name: /^From: sender@example.org$/ })).toBeInTheDocument());
    await app.searchMessageField('subject', 'Hello there');
    await waitFor(() => expect(input.value).toBe('from:"sender@example.org" subject:"Hello there"'));
  });

  it('Esc in a chip editor closes it without clearing the search', async () => {
    const { app, input } = await setup();
    submit(input, 'subject:y');
    fireEvent.click(await waitFor(() => within(bar()).getByRole('button', { name: /^Subject: y$/ })));
    const dialog = await screen.findByRole('dialog');
    fireEvent.keyDown(within(dialog).getByRole('textbox'), { key: 'Escape' });
    await waitFor(() => expect(screen.queryByRole('dialog')).toBeNull());
    expect(app.searchActive()).toBe(true);
    expect(input.value).toBe('subject:y');
  });
});

describe('related search actions (t88)', () => {
  it('offers sender, subject and folder for a message', () => {
    const email = mkEmail('a', { subject: 'Hi', from: [{ name: 'A', email: 'a@example.org' }], mailboxIds: { inbox: true } });
    const runs: string[] = [];
    const rows = relatedSearchActions(email, ['inbox'], (field, value) => runs.push(`${field}=${value}`));
    expect(rows.map((r) => r.id)).toEqual(['search:from', 'search:subject', 'search:in']);
    rows.forEach((r) => void r.run());
    expect(runs).toEqual(['from=a@example.org', 'subject=Hi', 'in=inbox']);
    expect(relatedSearchActions({ ...email, subject: '' }, [], () => undefined).map((r) => r.id)).toEqual(['search:from']);
  });
});

