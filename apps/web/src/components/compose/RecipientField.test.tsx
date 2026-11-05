import { beforeAll, describe, expect, it, vi } from 'vitest';
import { createSignal } from 'solid-js';
import { render, screen, fireEvent, within, waitFor } from '@solidjs/testing-library';
import { loadCatalog } from '../../i18n/index.ts';
import { RecipientField } from './RecipientField.tsx';
import type { RecipientSeparator } from './recipientChips.ts';

beforeAll(async () => { await loadCatalog('mail'); });

function setup(opts: { value?: string; separator?: RecipientSeparator; autoChip?: boolean; names?: Record<string, string>; openCard?: (a: string, n: string, el: HTMLElement) => void } = {}) {
  const changes: string[] = [];
  const [value, setValue] = createSignal(opts.value ?? '');
  const [auto, setAuto] = createSignal(opts.autoChip ?? true);
  render(() => <RecipientField field="to" groupLabel="Recipients in To" value={value()} separator={opts.separator ?? 'comma'} autoChip={auto()}
    onChange={next => { changes.push(next); setValue(next); }}
    nameOf={email => opts.names?.[email] ?? null}
    {...(opts.openCard ? { openCard: opts.openCard } : {})}
    inputAttrs={{ 'aria-label': 'To' }}/>);
  const input = screen.getByLabelText('To') as HTMLInputElement;
  const chips = () => Array.from(document.querySelectorAll<HTMLElement>('li[data-recipient]'));
  const texts = () => chips().map(chip => chip.dataset.recipient);
  const main = (index: number) => chips()[index]!.querySelector<HTMLButtonElement>('.compose-rf__chip-main')!;
  /** Type the way a person does: the trigger character arrives last. */
  const type = (text: string) => { fireEvent.input(input, { target: { value: text.slice(0, -1) } }); fireEvent.input(input, { target: { value: text } }); };
  return { input, changes, value, setValue, setAuto, chips, texts, main, type };
}

describe('RecipientField', () => {
  it.each([[' '], [','], [';']])('commits a valid address typed before %j and replaces the text with a chip', trigger => {
    const f = setup();
    f.input.focus();
    f.type(`a@b.example${trigger}`);
    expect(f.input.value).toBe('');
    expect(f.texts()).toEqual(['a@b.example']);
    expect(f.input.closest('.compose-rf__box')!.contains(f.chips()[0]!)).toBe(true);
    expect(f.input).toHaveFocus();
  });

  it('commits on Enter, Tab and blur', () => {
    const f = setup();
    f.input.focus();
    fireEvent.input(f.input, { target: { value: 'a@b.example' } });
    fireEvent.keyDown(f.input, { key: 'Enter' });
    fireEvent.input(f.input, { target: { value: 'c@d.example' } });
    fireEvent.keyDown(f.input, { key: 'Tab' });
    fireEvent.input(f.input, { target: { value: 'e@f.example' } });
    fireEvent.blur(f.input);
    expect(f.texts()).toEqual(['a@b.example', 'c@d.example', 'e@f.example']);
    expect(f.input.value).toBe('');
  });

  it('keeps invalid text while typing, makes an invalid chip on blur and edits it on click', () => {
    const f = setup();
    f.input.focus();
    f.type('nope ');
    expect(f.input.value).toBe('nope ');
    expect(f.chips()).toHaveLength(0);
    fireEvent.blur(f.input);
    expect(f.chips()[0]).toHaveAttribute('data-valid', 'false');
    expect(f.main(0)).toHaveAccessibleName(/nope.*not a valid address/);
    fireEvent.click(f.main(0));
    expect(f.input.value).toBe('nope');
    expect(f.chips()).toHaveLength(0);
  });

  it('turns the last chip back into editable text on Backspace in an empty input', () => {
    const f = setup({ value: 'a@b.example, Ann <c@d.example>' });
    expect(f.texts()).toEqual(['a@b.example', 'Ann <c@d.example>']);
    f.input.focus();
    fireEvent.keyDown(f.input, { key: 'Backspace' });
    expect(f.texts()).toEqual(['a@b.example']);
    expect(f.input.value).toBe('Ann <c@d.example>');
    expect(f.input).toHaveFocus();
    expect(f.input.selectionStart).toBe(f.input.value.length);
  });

  it('splits a paste into chips, including Name <addr> forms', () => {
    const f = setup();
    f.input.focus();
    fireEvent.paste(f.input, { clipboardData: { getData: () => 'a@b.example; Ann <c@d.example>\ne@f.example g@h.example' } });
    expect(f.texts()).toEqual(['a@b.example', 'Ann <c@d.example>', 'e@f.example', 'g@h.example']);
    expect(f.input.value).toBe('');
  });

  it('commits a single pasted address', () => {
    const f = setup();
    fireEvent.paste(f.input, { clipboardData: { getData: () => 'a@b.example' } });
    expect(f.texts()).toEqual(['a@b.example']);
  });

  it('moves chip focus with the arrows, removes with Delete and returns to the input', () => {
    const f = setup({ value: 'a@b.example, c@d.example, e@f.example' });
    f.input.focus();
    fireEvent.keyDown(f.input, { key: 'ArrowLeft' });
    expect(f.main(2)).toHaveFocus();
    fireEvent.keyDown(f.main(2), { key: 'ArrowLeft' });
    expect(f.main(1)).toHaveFocus();
    fireEvent.keyDown(f.main(1), { key: 'Delete' });
    return waitFor(() => {
      expect(f.texts()).toEqual(['a@b.example', 'e@f.example']);
      expect(f.main(1)).toHaveFocus();
    }).then(() => {
      fireEvent.keyDown(f.main(1), { key: 'ArrowRight' });
      expect(f.input).toHaveFocus();
    });
  });

  it('selects all chips with Ctrl+A and removes them with Delete', async () => {
    const f = setup({ value: 'a@b.example, c@d.example' });
    f.input.focus();
    fireEvent.keyDown(f.input, { key: 'a', ctrlKey: true });
    expect(f.chips().every(chip => chip.dataset.selected === 'true')).toBe(true);
    fireEvent.keyDown(document.activeElement!, { key: 'Delete' });
    await waitFor(() => expect(f.chips()).toHaveLength(0));
    expect(f.value()).toBe('');
  });

  it('Backspace or F2 on a focused chip edits it; Escape goes back to the input', () => {
    const f = setup({ value: 'a@b.example, c@d.example' });
    f.input.focus();
    fireEvent.keyDown(f.input, { key: 'ArrowLeft' });
    fireEvent.keyDown(f.main(1), { key: 'Escape' });
    expect(f.input).toHaveFocus();
    fireEvent.keyDown(f.input, { key: 'ArrowLeft' });
    fireEvent.keyDown(f.main(1), { key: 'F2' });
    expect(f.input.value).toBe('c@d.example');
    expect(f.texts()).toEqual(['a@b.example']);
  });

  it('opens the card callback only on click, not on hover or double click', () => {
    const open = vi.fn();
    const f = setup({ value: 'a@b.example', names: { 'a@b.example': 'Ann' }, openCard: open });
    fireEvent.mouseOver(f.main(0)); fireEvent.mouseEnter(f.main(0));
    expect(open).not.toHaveBeenCalled();
    fireEvent.click(f.main(0));
    expect(open).toHaveBeenCalledWith('a@b.example', 'Ann', f.main(0));
    fireEvent.dblClick(f.main(0));
    expect(f.input.value).toBe('a@b.example');
  });

  it('serialises with the comma or the semicolon separator and keeps the typed text last', () => {
    const comma = setup();
    comma.type('a@b.example ');
    comma.type('c@d.example ');
    expect(comma.changes.at(-1)).toBe('a@b.example, c@d.example, ');
    fireEvent.input(comma.input, { target: { value: 'dr' } });
    expect(comma.changes.at(-1)).toBe('a@b.example, c@d.example, dr');
  });

  it('serialises with the semicolon separator', () => {
    const semi = setup({ separator: 'semicolon' });
    semi.type('a@b.example ');
    semi.type('c@d.example,');
    expect(semi.changes.at(-1)).toBe('a@b.example; c@d.example; ');
  });

  it('parses an outside value change (reply prefill, recalled draft) into chips', () => {
    const f = setup();
    f.setValue('Alice <alice@example.org>, bob@example.org');
    expect(f.texts()).toEqual(['Alice <alice@example.org>', 'bob@example.org']);
    expect(f.input.value).toBe('');
    f.setValue('');
    expect(f.chips()).toHaveLength(0);
  });

  it('is a plain text field when auto chips are off', () => {
    const f = setup({ autoChip: false });
    f.input.focus();
    fireEvent.input(f.input, { target: { value: 'a@b.example ' } });
    fireEvent.keyDown(f.input, { key: 'Enter' });
    fireEvent.blur(f.input);
    expect(f.input.value).toBe('a@b.example ');
    expect(f.chips()).toHaveLength(0);
    expect(f.value()).toBe('a@b.example ');
  });

  it('falls back to plain text, and back to chips, when the setting flips', () => {
    const f = setup({ value: 'a@b.example, c@d.example' });
    expect(f.chips()).toHaveLength(2);
    f.setAuto(false);
    expect(f.chips()).toHaveLength(0);
    expect(f.input.value).toBe('a@b.example, c@d.example');
    f.setAuto(true);
    expect(f.texts()).toEqual(['a@b.example', 'c@d.example']);
  });

  it('shows the contact name and gives each chip a descriptive accessible name', () => {
    const f = setup({ value: 'alice@example.org', names: { 'alice@example.org': 'Alice Adams' } });
    expect(screen.getByRole('group', { name: 'Recipients in To' })).toBeInTheDocument();
    expect(screen.getByRole('list')).toBeInTheDocument();
    expect(within(f.chips()[0]!).getByText(/Alice Adams/)).toBeInTheDocument();
    expect(f.main(0)).toHaveAccessibleName(/Alice Adams.*alice@example\.org.*Backspace.*Delete/);
    expect(f.main(0)).toHaveAttribute('title', 'alice@example.org');
    expect(screen.getByRole('button', { name: /Remove .*alice@example\.org/ })).toBeInTheDocument();
  });

  it('announces additions and removals politely', async () => {
    const f = setup();
    f.input.focus();
    f.type('a@b.example ');
    const live = screen.getByRole('status');
    await waitFor(() => expect(live).toHaveTextContent(/Added .*a@b\.example/));
    fireEvent.click(screen.getByRole('button', { name: /Remove .*a@b\.example/ }));
    await waitFor(() => expect(live).toHaveTextContent(/Removed .*a@b\.example/));
  });

  it('focuses the input when the empty area of the box is clicked', () => {
    const f = setup({ value: 'a@b.example' });
    fireEvent.click(f.input.closest('.compose-rf__box')!);
    expect(f.input).toHaveFocus();
  });
});
