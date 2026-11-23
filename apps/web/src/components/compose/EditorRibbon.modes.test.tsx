// The composer ribbon's corner toggle: pin <-> auto-hide, Ctrl+F1, double-click on a tab, the lock
// and the layout.ribbonAutoHide switch. Hover and Esc timing belong to CollapsibleBar's own suite.

import { afterEach, describe, expect, it, vi } from 'vitest';
import { fireEvent, render, screen } from '@solidjs/testing-library';
import { createSignal } from 'solid-js';
import { createEditorRibbon, EditorRibbon } from './EditorRibbon.tsx';
import { HOVER_OPEN_MS } from '../bars/index.ts';
import type { BarMode, BarModeHandle } from '../bars/useBarMode.ts';

afterEach(() => vi.useRealTimers());

function fakeBar(initial: BarMode, locked = false): BarModeHandle & { calls: BarMode[] } {
  const [mode, setMode] = createSignal<BarMode>(initial);
  const calls: BarMode[] = [];
  return {
    calls,
    mode,
    locked: () => locked,
    saving: () => false,
    error: () => '',
    setMode: (next) => {
      calls.push(next);
      setMode(next);
      return Promise.resolve(true);
    },
  };
}

function mount(bar: BarModeHandle, toggle = true) {
  const state = createEditorRibbon<never>('format');
  return render(() => (
    <EditorRibbon
      state={state}
      bar={bar}
      history={() => null}
      showHistory={() => false}
      visible={() => ['format', 'insert']}
      modeToggle={() => toggle}
      tabs={[{ id: 'format', after: () => <button type="button">cmd</button> }, { id: 'insert' }]}
    />
  ));
}

const pin = (): HTMLElement => screen.getByTestId('compose-ribbon-pin');
const root = (): HTMLElement => document.querySelector<HTMLElement>('[data-bar="composerRibbon"]')!;

describe('composer ribbon corner toggle', () => {
  it('is in the heading, pressed while pinned, and switches to auto-hide on a click', () => {
    const bar = fakeBar('pinned');
    mount(bar);
    expect(pin().closest('.compose-ribbon__heading')).not.toBeNull();
    expect(pin()).toHaveAttribute('aria-pressed', 'true');
    expect(pin()).toHaveAttribute('aria-keyshortcuts', 'Control+F1');
    // A stable name; the pressed state says whether the ribbon is kept open.
    expect(pin()).toHaveAccessibleName('Always show ribbon (Ctrl+F1)');
    fireEvent.click(pin());
    expect(bar.calls).toEqual(['auto']);
    expect(root()).toHaveAttribute('data-mode', 'auto');
    expect(root()).toHaveAttribute('data-expanded', 'false');
    expect(pin()).toHaveAttribute('aria-pressed', 'false');
    expect(pin()).toHaveAccessibleName('Always show ribbon (Ctrl+F1)');
    fireEvent.click(pin());
    expect(bar.calls).toEqual(['auto', 'pinned']);
    expect(root()).toHaveAttribute('data-expanded', 'true');
  });

  it('moves a live ribbon between the workspace host and the composer without resetting its controls',()=>{
    const target=document.createElement('div');document.body.append(target);
    const [host,setHost]=createSignal<HTMLElement>();
    const [active,setActive]=createSignal(true);
    const state=createEditorRibbon<never>('format');
    const result=render(()=><EditorRibbon state={state} mount={host()} active={active()} bar={fakeBar('pinned')} history={()=>null} showHistory={()=>false} visible={()=>['format']} tabs={[{id:'format',after:()=> <input aria-label="Retained control"/>}]}/>);
    try {
      const control=screen.getByRole('textbox',{name:'Retained control'});fireEvent.input(control,{target:{value:'Keep me'}});
      setHost(target);expect(target.contains(control)).toBe(true);
      setActive(false);expect(screen.queryByRole('textbox',{name:'Retained control'})).toBeNull();
      setActive(true);expect(screen.getByRole('textbox',{name:'Retained control'})).toHaveValue('Keep me');
      setHost(undefined);expect(target.contains(control)).toBe(false);expect(screen.getByRole('textbox',{name:'Retained control'})).toBe(control);
    }finally{result.unmount();target.remove();}
  });

  it('Ctrl+F1 toggles the mode and other chords do nothing', () => {
    const bar = fakeBar('pinned');
    mount(bar);
    fireEvent.keyDown(document, { key: 'F1' });
    fireEvent.keyDown(document, { key: 'F1', ctrlKey: true, shiftKey: true });
    expect(bar.calls).toEqual([]);
    fireEvent.keyDown(document, { key: 'F1', ctrlKey: true });
    expect(bar.calls).toEqual(['auto']);
    fireEvent.keyDown(document, { key: 'F1', ctrlKey: true });
    expect(bar.calls).toEqual(['auto', 'pinned']);
  });

  it('in auto-hide a tab click opens the panel and a double click pins the ribbon', () => {
    const bar = fakeBar('auto');
    mount(bar);
    const insert = screen.getByRole('tab', { name: /insert/i });
    fireEvent.click(insert);
    expect(root()).toHaveAttribute('data-expanded', 'true');
    expect(insert).toHaveAttribute('aria-expanded', 'true');
    fireEvent.doubleClick(insert);
    expect(bar.calls).toEqual(['pinned']);
  });

  it('hover opens the panel after the intent delay and Esc closes it again', () => {
    vi.useFakeTimers();
    mount(fakeBar('auto'));
    fireEvent.pointerEnter(root());
    fireEvent.mouseEnter(root());
    expect(root()).toHaveAttribute('data-expanded', 'false');
    vi.advanceTimersByTime(HOVER_OPEN_MS + 20);
    expect(root()).toHaveAttribute('data-expanded', 'true');
    fireEvent.keyDown(root(), { key: 'Escape' });
    expect(root()).toHaveAttribute('data-expanded', 'false');
  });

  it('has no toggle and ignores Ctrl+F1 and double click when an administrator locks the mode', () => {
    const bar = fakeBar('auto', true);
    mount(bar);
    expect(screen.queryByTestId('compose-ribbon-pin')).toBeNull();
    fireEvent.keyDown(document, { key: 'F1', ctrlKey: true });
    fireEvent.doubleClick(screen.getByRole('tab', { name: /insert/i }));
    expect(bar.calls).toEqual([]);
  });

  it('has no toggle and no shortcut when the layout.ribbonAutoHide flag is off', () => {
    const bar = fakeBar('pinned');
    mount(bar, false);
    expect(screen.queryByTestId('compose-ribbon-pin')).toBeNull();
    fireEvent.keyDown(document, { key: 'F1', ctrlKey: true });
    expect(bar.calls).toEqual([]);
  });
});
