import { describe, it, expect, vi, afterEach, beforeEach } from 'vitest';
import { cleanup, fireEvent, render, screen, waitFor, within } from '@solidjs/testing-library';
import { createSignal } from 'solid-js';
import { SelectionBar, inlineCount, type SelectionAction } from './SelectionBar.tsx';

// The floating selection bar (t58) on its own: it appears and disappears with
// the selection, never takes focus by itself, fits one line by moving actions
// into More, and hands focus back to the list on Esc.

function actions(spy: (id: string) => void): SelectionAction[] {
  return ['archive', 'delete', 'read', 'flag', 'spam', 'forward', 'download'].map((id) => ({
    id, label: `Do ${id}`, icon: 'archive', testId: `act-${id}`, run: () => spy(id),
  }));
}

function mount(opts: { width?: number; total?: number } = {}) {
  const [count, setCount] = createSignal(0);
  const ran: string[] = [];
  const selectAll = vi.fn(() => setCount(opts.total ?? 10));
  const clear = vi.fn(() => setCount(0));
  const returnFocus = vi.fn(() => document.querySelector<HTMLElement>('#row')?.focus());
  const move = vi.fn();
  const list: SelectionAction[] = [
    ...actions((id) => ran.push(id)).slice(0, 2),
    { id: 'move', label: 'Move to folder', icon: 'folder', testId: 'act-move', menu: () => [{ id: 'm:x', label: 'Projects', icon: 'folder', run: move }] },
    ...actions((id) => ran.push(id)).slice(2),
  ];
  render(() => (
    <section id="pane" style={{ position: 'relative' }}>
      <button id="row" type="button">Row</button>
      <button id="row2" type="button">Row 2</button>
      <SelectionBar count={count()} total={opts.total ?? 10} selectAll={selectAll} clear={clear} actions={list} returnFocus={returnFocus} scope="s" />
    </section>
  ));
  return { setCount, ran, selectAll, clear, returnFocus, move };
}

/** Lay the bar out at `pane` px: buttons are 36px, the count 80px. */
function stubLayout(pane: number): void {
  vi.spyOn(HTMLElement.prototype, 'offsetWidth', 'get').mockImplementation(function (this: HTMLElement) {
    if (this.tagName === 'BUTTON') return 36;
    if (this.dataset['testid'] === 'bulk-count') return 80;
    return 1;
  });
  vi.spyOn(HTMLElement.prototype, 'clientWidth', 'get').mockImplementation(function (this: HTMLElement) {
    return this.id === 'pane' ? pane : 0;
  });
}

beforeEach(() => localStorage.clear());
afterEach(() => { cleanup(); vi.restoreAllMocks(); });

describe('inlineCount', () => {
  it('keeps every action when they fit, else leaves a slot for More', () => {
    expect(inlineCount(8, 1000, 150, 36)).toBe(8);
    expect(inlineCount(8, 150 + 8 * 36, 150, 36)).toBe(8);
    expect(inlineCount(8, 374, 154, 36)).toBe(5);
    expect(inlineCount(8, 100, 154, 36)).toBe(0);
    // Unmeasured: everything, rather than nothing.
    expect(inlineCount(8, 0, 154, 36)).toBe(8);
  });
});

describe('SelectionBar', () => {
  it('appears with the first selection, shows and announces the count, and goes when it is empty', async () => {
    const { setCount } = mount();
    expect(screen.queryByRole('toolbar')).toBeNull();
    setCount(3);
    const bar = screen.getByRole('toolbar', { name: 'Selection actions' });
    expect(within(bar).getByTestId('bulk-count').textContent).toBe('3 selected');
    expect(screen.getByTestId('selection-bar-status').textContent).toBe('3 messages selected');
    expect(screen.getByTestId('selection-bar-status').getAttribute('aria-live')).toBe('polite');
    setCount(1);
    expect(screen.getByTestId('selection-bar-status').textContent).toBe('1 message selected');
    setCount(0);
    expect(screen.queryByRole('toolbar')).toBeNull();
    expect(screen.getByTestId('selection-bar-status').textContent).toBe('');
  });

  it('never takes focus when it appears or updates; the list underneath keeps it', () => {
    const { setCount } = mount();
    const row = document.querySelector<HTMLElement>('#row')!;
    row.focus();
    setCount(1);
    expect(document.activeElement).toBe(row);
    setCount(4);
    expect(document.activeElement).toBe(row);
    // The rows are still reachable and clickable: no backdrop, no inert, no trap.
    const row2 = document.querySelector<HTMLElement>('#row2')!;
    row2.focus();
    expect(document.activeElement).toBe(row2);
    expect(document.querySelector('[inert], [aria-modal="true"]')).toBeNull();
  });

  it('runs an action, and Select all turns into Select none once everything is selected', () => {
    const { setCount, ran, selectAll, clear } = mount({ total: 5 });
    setCount(2);
    fireEvent.click(screen.getByRole('button', { name: 'Do archive' }));
    expect(ran).toEqual(['archive']);
    fireEvent.click(screen.getByRole('button', { name: 'Select all' }));
    expect(selectAll).toHaveBeenCalledOnce();
    fireEvent.click(screen.getByRole('button', { name: 'Select none' }));
    expect(clear).toHaveBeenCalledOnce();
    expect(screen.queryByRole('toolbar')).toBeNull();
  });

  it('× clears the selection', () => {
    const { setCount, clear } = mount();
    setCount(2);
    fireEvent.click(screen.getByRole('button', { name: 'Clear selection' }));
    expect(clear).toHaveBeenCalledOnce();
  });

  it('a menu action opens a menu above it and runs the chosen item', async () => {
    const { setCount, move } = mount();
    setCount(2);
    const trigger = screen.getByRole('button', { name: 'Move to folder' });
    expect(trigger.getAttribute('aria-haspopup')).toBe('menu');
    fireEvent.click(trigger);
    const menu = await screen.findByRole('menu', { name: 'Move to folder' });
    expect(trigger.getAttribute('aria-expanded')).toBe('true');
    fireEvent.click(within(menu).getByRole('menuitem', { name: 'Projects' }));
    expect(move).toHaveBeenCalledOnce();
  });

  it('at phone width the actions that do not fit go into More, and the bar does not wrap', async () => {
    stubLayout(390);
    const { setCount, ran } = mount();
    setCount(2);
    const bar = screen.getByRole('toolbar');
    // 374px available: count 80 + select all 36 + × 36 + 2 rules = 154; 5 actions + More.
    await waitFor(() => expect(within(bar).getAllByRole('button')).toHaveLength(1 + 5 + 1 + 1));
    expect(within(bar).queryByRole('button', { name: 'Do forward' })).toBeNull();
    fireEvent.click(within(bar).getByRole('button', { name: 'More selection actions' }));
    const menu = await screen.findByRole('menu', { name: 'More selection actions' });
    expect(within(menu).getAllByRole('menuitem').map((el) => el.textContent)).toEqual(['Do spam', 'Do forward', 'Do download']);
    fireEvent.click(within(menu).getByRole('menuitem', { name: 'Do forward' }));
    expect(ran).toEqual(['forward']);
  });

  it('at desktop width everything is on the bar and there is no More', async () => {
    stubLayout(1440);
    const { setCount } = mount();
    setCount(2);
    await Promise.resolve();
    expect(screen.queryByRole('button', { name: 'More selection actions' })).toBeNull();
    expect(screen.getByRole('button', { name: 'Do download' })).toBeInTheDocument();
  });

  it('F6 in the list moves focus to the bar; arrows move inside it; Esc returns to the list', async () => {
    const { setCount, returnFocus } = mount();
    setCount(2);
    const row = document.querySelector<HTMLElement>('#row')!;
    row.focus();
    fireEvent.keyDown(row, { key: 'F6' });
    await waitFor(() => expect(document.activeElement).toBe(screen.getByRole('button', { name: 'Select all' })));
    // One tab stop: the focused control.
    const bar = screen.getByRole('toolbar');
    expect(within(bar).getAllByRole('button').filter((b) => b.tabIndex === 0)).toHaveLength(1);
    fireEvent.keyDown(document.activeElement!, { key: 'ArrowRight' });
    expect(document.activeElement).toBe(screen.getByRole('button', { name: 'Do archive' }));
    fireEvent.keyDown(document.activeElement!, { key: 'End' });
    expect(document.activeElement).toBe(screen.getByRole('button', { name: 'Clear selection' }));
    fireEvent.keyDown(document.activeElement!, { key: 'Escape' });
    expect(returnFocus).toHaveBeenCalledOnce();
    expect(document.activeElement).toBe(row);
  });

  it('when the bar closes under focus, focus goes back to the list', async () => {
    const { setCount, returnFocus } = mount();
    setCount(1);
    screen.getByRole('button', { name: 'Do archive' }).focus();
    setCount(0);
    await waitFor(() => expect(returnFocus).toHaveBeenCalled());
    expect(document.activeElement?.id).toBe('row');
  });
});
