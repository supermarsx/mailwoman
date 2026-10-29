import { describe, expect, it, vi } from 'vitest';
import { fireEvent, render, screen } from '@solidjs/testing-library';
import { createSignal } from 'solid-js';
import { MultiSelect, type MultiSelectOption } from './MultiSelect.tsx';

const OPTIONS: MultiSelectOption[] = [
  { value: 'a', label: 'Ana Silva', detail: 'ana@example.test' },
  { value: 'b', label: 'Bruno Costa', detail: 'bruno@example.test' },
  { value: 'c', label: 'Carla Ândrade', detail: 'carla@corp.test' },
  { value: 'd', label: 'Dário Silva' },
];

function harness(initial: string[] = [], disabled = false) {
  const changes: string[][] = [];
  const [selected, setSelected] = createSignal<string[]>(initial);
  render(() => <MultiSelect label="Members" options={OPTIONS} selected={selected()} disabled={disabled} onChange={values => { changes.push(values); setSelected(values); }} />);
  return { changes, selected, input: () => screen.getByRole('combobox', { name: 'Members' }) as HTMLInputElement };
}

describe('MultiSelect', () => {
  it('is a combobox over a multi-selectable listbox that opens on ArrowDown', () => {
    const { input } = harness();
    expect(input()).toHaveAttribute('aria-expanded', 'false');
    expect(screen.queryByRole('listbox')).toBeNull();
    fireEvent.keyDown(input(), { key: 'ArrowDown' });
    expect(input()).toHaveAttribute('aria-expanded', 'true');
    const list = screen.getByRole('listbox');
    expect(list).toHaveAttribute('aria-multiselectable', 'true');
    expect(input()).toHaveAttribute('aria-controls', list.id);
    expect(screen.getAllByRole('option')).toHaveLength(4);
    expect(input().getAttribute('aria-activedescendant')).toBe(screen.getAllByRole('option')[0]!.id);
  });

  it('moves with the arrows, Home and End, and toggles with Enter without losing focus', () => {
    const { input, changes } = harness();
    input().focus();
    fireEvent.keyDown(input(), { key: 'ArrowDown' });
    fireEvent.keyDown(input(), { key: 'ArrowDown' });
    const options = () => screen.getAllByRole('option');
    expect(input().getAttribute('aria-activedescendant')).toBe(options()[1]!.id);
    fireEvent.keyDown(input(), { key: 'Enter' });
    expect(changes.at(-1)).toEqual(['b']);
    expect(options()[1]).toHaveAttribute('aria-selected', 'true');
    fireEvent.keyDown(input(), { key: 'End' });
    expect(input().getAttribute('aria-activedescendant')).toBe(options()[3]!.id);
    fireEvent.keyDown(input(), { key: 'ArrowDown' });
    expect(input().getAttribute('aria-activedescendant')).toBe(options()[0]!.id);
    fireEvent.keyDown(input(), { key: 'ArrowUp' });
    expect(input().getAttribute('aria-activedescendant')).toBe(options()[3]!.id);
    fireEvent.keyDown(input(), { key: 'Home' });
    fireEvent.keyDown(input(), { key: 'Enter' });
    expect(changes.at(-1)).toEqual(['b', 'a']);
    fireEvent.keyDown(input(), { key: 'Enter' });
    expect(changes.at(-1)).toEqual(['b']);
    expect(document.activeElement).toBe(input());
  });

  it('filters by typeahead, ignoring case and accents, with word starts first', () => {
    const { input, changes } = harness();
    fireEvent.input(input(), { target: { value: 'silva' } });
    expect(screen.getAllByRole('option').map(o => o.querySelector('bdi')!.textContent)).toEqual(['Ana Silva', 'Dário Silva']);
    fireEvent.input(input(), { target: { value: 'andr' } });
    expect(screen.getAllByRole('option').map(o => o.querySelector('bdi')!.textContent)).toEqual(['Carla Ândrade']);
    fireEvent.input(input(), { target: { value: 'corp' } });
    expect(screen.getAllByRole('option')).toHaveLength(1);
    fireEvent.keyDown(input(), { key: 'Enter' });
    expect(changes.at(-1)).toEqual(['c']);
    expect(input().value).toBe('');
    expect(screen.getAllByRole('option')).toHaveLength(4);
    fireEvent.input(input(), { target: { value: 'zzz' } });
    expect(screen.queryAllByRole('option')).toHaveLength(0);
    expect(screen.getByText('No matching options')).toBeVisible();
    fireEvent.keyDown(input(), { key: 'Enter' });
    expect(changes).toHaveLength(1);
  });

  it('shows chips with their own remove buttons, Backspace removes the last, and announces the count', () => {
    const { input, changes } = harness(['a', 'c']);
    const chips = screen.getByRole('list', { name: 'Selected: Members' });
    expect(chips.querySelectorAll('li')).toHaveLength(2);
    fireEvent.click(screen.getByRole('button', { name: 'Remove Ana Silva' }));
    expect(changes.at(-1)).toEqual(['c']);
    expect(document.activeElement).toBe(input());
    expect(screen.getByRole('status')).toHaveTextContent('1 selected');
    fireEvent.keyDown(input(), { key: 'Backspace' });
    expect(changes.at(-1)).toEqual([]);
    expect(screen.getByRole('status')).toHaveTextContent('0 selected');
    fireEvent.keyDown(input(), { key: 'Backspace' });
    expect(changes).toHaveLength(2);
  });

  it('Escape closes the list first and then clears the filter; clicking an option toggles it', () => {
    const { input, changes } = harness();
    fireEvent.input(input(), { target: { value: 'bru' } });
    const outer = vi.fn(); document.addEventListener('keydown', outer);
    fireEvent.keyDown(input(), { key: 'Escape' });
    expect(outer).not.toHaveBeenCalled();
    document.removeEventListener('keydown', outer);
    expect(screen.queryByRole('listbox')).toBeNull();
    expect(input().value).toBe('bru');
    fireEvent.keyDown(input(), { key: 'Escape' });
    expect(input().value).toBe('');
    fireEvent.click(input());
    fireEvent.click(screen.getByRole('option', { name: 'Dário Silva' }));
    expect(changes.at(-1)).toEqual(['d']);
  });

  it('does nothing while disabled', () => {
    const { input, changes } = harness(['a'], true);
    expect(input()).toBeDisabled();
    expect(screen.getByRole('button', { name: 'Remove Ana Silva' })).toBeDisabled();
    fireEvent.keyDown(input(), { key: 'ArrowDown' });
    fireEvent.keyDown(input(), { key: 'Backspace' });
    expect(screen.queryByRole('listbox')).toBeNull();
    expect(changes).toEqual([]);
  });

  it('keeps a selected value that is not among the options as a removable chip', () => {
    const { changes } = harness(['gone']);
    fireEvent.click(screen.getByRole('button', { name: 'Remove gone' }));
    expect(changes).toEqual([[]]);
  });
});
