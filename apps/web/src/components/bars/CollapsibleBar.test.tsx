// CollapsibleBar (t38 §2.1): pin toggle and aria, hover intent and leave
// delays, focus-in, Esc with focus return, touch without hover listeners, the
// typing guard, the hold counter, reduced motion and push-mode caret keeping.

import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import { cleanup, fireEvent, render, screen } from '@solidjs/testing-library';
import { createSignal, type JSX } from 'solid-js';
import { CollapsibleBar, HOVER_OPEN_MS, LEAVE_CLOSE_MS, TYPING_GUARD_MS, useCollapsibleBar, type CollapsibleBarApi } from './CollapsibleBar.tsx';
import type { BarMode, BarModeHandle } from './useBarMode.ts';
import { REVEAL_MS } from './collapsibleBar.css.ts';

function fakeBar(initial: BarMode, locked = false): BarModeHandle & { calls: BarMode[] } {
  const [mode, setMode] = createSignal<BarMode>(initial);
  const calls: BarMode[] = [];
  return {
    calls,
    mode,
    locked: () => locked,
    saving: () => false,
    error: () => '',
    setMode: (m) => {
      calls.push(m);
      setMode(m);
      return Promise.resolve(true);
    },
  };
}

/** `matchMedia` answering `true` for the listed queries. */
function stubMedia(matching: string[]): void {
  vi.stubGlobal('matchMedia', (query: string) => ({
    matches: matching.includes(query),
    media: query,
    addEventListener: () => {},
    removeEventListener: () => {},
  }));
}

const root = (): HTMLElement => document.querySelector('[data-bar]') as HTMLElement;
const panel = (): HTMLElement => document.querySelector('[data-bar-panel]') as HTMLElement;
const handle = (): HTMLElement => document.querySelector('[data-bar-handle]') as HTMLElement;
const pin = (): HTMLElement => document.querySelector('[data-bar-pin]') as HTMLElement;
const isOpen = (): boolean => root().dataset['expanded'] === 'true';

function mount(bar: BarModeHandle, extra: Partial<Parameters<typeof CollapsibleBar>[0]> = {}, outside?: () => JSX.Element) {
  return render(() => (
    <div>
      <CollapsibleBar id="composerRibbon" revealPolicy="overlay" bar={bar} {...extra}>
        <button type="button">Bold</button>
        <button type="button">Italic</button>
      </CollapsibleBar>
      {outside?.()}
      <button type="button">Outside</button>
    </div>
  ));
}

beforeEach(() => {
  vi.useFakeTimers();
  stubMedia([]);
});
afterEach(() => {
  cleanup();
  vi.useRealTimers();
  vi.unstubAllGlobals();
  document.documentElement.removeAttribute('data-reduced-motion');
});

describe('CollapsibleBar: pinned', () => {
  it('stays in the flow with no handle; the pin is pressed and switches to auto-hide', async () => {
    const bar = fakeBar('pinned');
    mount(bar);
    expect(root()).toHaveAttribute('data-mode', 'pinned');
    expect(isOpen()).toBe(true);
    expect(handle()).toBeNull();
    expect(panel()).not.toHaveAttribute('inert');
    expect(pin()).toHaveAttribute('aria-pressed', 'true');
    expect(pin()).toHaveAccessibleName('Auto-hide the editor ribbon');
    fireEvent.click(pin());
    await Promise.resolve();
    expect(bar.calls).toEqual(['auto']);
    expect(root()).toHaveAttribute('data-mode', 'auto');
    // It stays open under the pointer until the usual close rules apply.
    expect(isOpen()).toBe(true);
  });

  it('ignores hover and focus while pinned', () => {
    mount(fakeBar('pinned'));
    fireEvent.pointerEnter(root());
    vi.advanceTimersByTime(HOVER_OPEN_MS + LEAVE_CLOSE_MS);
    fireEvent.pointerLeave(root());
    vi.advanceTimersByTime(LEAVE_CLOSE_MS);
    expect(isOpen()).toBe(true);
  });

  it('disables the pin with the administrator tooltip when the setting is locked', () => {
    const bar = fakeBar('pinned', true);
    mount(bar);
    expect(pin()).toBeDisabled();
    expect(pin()).toHaveAttribute('data-tooltip', 'Set by your administrator');
    fireEvent.click(pin());
    expect(bar.calls).toEqual([]);
  });
});

describe('CollapsibleBar: auto-hide', () => {
  it('shows only the handle, wired to the panel, and pins from inside the revealed bar', async () => {
    const bar = fakeBar('auto');
    mount(bar);
    expect(isOpen()).toBe(false);
    expect(panel()).toHaveAttribute('inert');
    expect(handle()).toHaveAttribute('aria-expanded', 'false');
    expect(handle()).toHaveAttribute('aria-controls', panel().id);
    expect(handle()).toHaveAccessibleName('Show the editor ribbon');
    fireEvent.pointerDown(handle());
    fireEvent.click(handle());
    expect(isOpen()).toBe(true);
    expect(handle()).toHaveAttribute('aria-expanded', 'true');
    expect(handle()).toHaveAccessibleName('Hide the editor ribbon');
    expect(pin()).toHaveAttribute('aria-pressed', 'false');
    expect(pin()).toHaveAccessibleName('Keep the editor ribbon open');
    fireEvent.click(pin());
    await Promise.resolve();
    expect(bar.calls).toEqual(['pinned']);
    expect(handle()).toBeNull();
    expect(isOpen()).toBe(true);
  });

  it('opens on hover after the intent delay and closes 400 ms after the pointer leaves', () => {
    mount(fakeBar('auto'));
    fireEvent.pointerEnter(root());
    vi.advanceTimersByTime(HOVER_OPEN_MS - 1);
    expect(isOpen()).toBe(false);
    vi.advanceTimersByTime(1);
    expect(isOpen()).toBe(true);
    fireEvent.pointerLeave(root());
    vi.advanceTimersByTime(LEAVE_CLOSE_MS - 1);
    expect(isOpen()).toBe(true);
    // Coming back cancels the close.
    fireEvent.pointerEnter(root());
    vi.advanceTimersByTime(LEAVE_CLOSE_MS);
    expect(isOpen()).toBe(true);
    fireEvent.pointerLeave(root());
    vi.advanceTimersByTime(LEAVE_CLOSE_MS);
    expect(isOpen()).toBe(false);
  });

  it('does not open when the pointer only passes through', () => {
    mount(fakeBar('auto'));
    fireEvent.pointerEnter(root());
    vi.advanceTimersByTime(HOVER_OPEN_MS - 50);
    fireEvent.pointerLeave(root());
    vi.advanceTimersByTime(HOVER_OPEN_MS);
    expect(isOpen()).toBe(false);
  });

  it('stays open after the pointer leaves while focus is inside', () => {
    mount(fakeBar('auto'));
    fireEvent.pointerEnter(root());
    vi.advanceTimersByTime(HOVER_OPEN_MS);
    screen.getByRole('button', { name: 'Bold' }).focus();
    fireEvent.pointerLeave(root());
    vi.advanceTimersByTime(LEAVE_CLOSE_MS * 2);
    expect(isOpen()).toBe(true);
    // Focus leaving closes it.
    screen.getByRole('button', { name: 'Outside' }).focus();
    vi.advanceTimersByTime(0);
    expect(isOpen()).toBe(false);
  });

  it('a click on a bar hover has opened keeps it open', () => {
    mount(fakeBar('auto'));
    fireEvent.pointerEnter(root());
    vi.advanceTimersByTime(HOVER_OPEN_MS);
    fireEvent.click(handle());
    expect(isOpen()).toBe(true);
    // A second click closes it.
    fireEvent.click(handle());
    expect(isOpen()).toBe(false);
  });

  it('opens when keyboard focus enters, and Esc closes it and returns focus to the handle', () => {
    mount(fakeBar('auto'));
    handle().focus();
    expect(isOpen()).toBe(true);
    screen.getByRole('button', { name: 'Italic' }).focus();
    fireEvent.keyDown(screen.getByRole('button', { name: 'Italic' }), { key: 'Escape' });
    expect(isOpen()).toBe(false);
    expect(document.activeElement).toBe(handle());
    // Returning focus does not reopen the bar.
    vi.advanceTimersByTime(LEAVE_CLOSE_MS);
    expect(isOpen()).toBe(false);
  });

  it('Esc returns focus to the header tab that opened it', () => {
    let api: CollapsibleBarApi | undefined;
    function Tabs(): JSX.Element {
      const bar = useCollapsibleBar()!;
      api = bar;
      return (
        <button type="button" aria-expanded={bar.expanded() ? 'true' : 'false'} onClick={(e) => bar.open(e.currentTarget)}>
          Insert
        </button>
      );
    }
    mount(fakeBar('auto'), {
      get header() {
        return <Tabs />;
      },
    });
    expect(handle()).toBeNull();
    const tab = screen.getByRole('button', { name: 'Insert' });
    expect(tab).toHaveAttribute('aria-expanded', 'false');
    fireEvent.pointerDown(tab);
    fireEvent.click(tab);
    expect(api?.expanded()).toBe(true);
    expect(tab).toHaveAttribute('aria-expanded', 'true');
    screen.getByRole('button', { name: 'Bold' }).focus();
    fireEvent.keyDown(document.activeElement!, { key: 'Escape' });
    expect(tab).toHaveAttribute('aria-expanded', 'false');
    expect(document.activeElement).toBe(tab);
  });

  it('a click or tap outside closes it', () => {
    mount(fakeBar('auto'));
    fireEvent.click(handle());
    expect(isOpen()).toBe(true);
    fireEvent.pointerDown(screen.getByRole('button', { name: 'Outside' }));
    expect(isOpen()).toBe(false);
  });

  it('never closes while held, and closes after the leave delay once released', () => {
    let api: CollapsibleBarApi | undefined;
    function Grab(): JSX.Element {
      api = useCollapsibleBar();
      return <span />;
    }
    render(() => (
      <div>
        <CollapsibleBar id="mailRibbon" revealPolicy="overlay" bar={fakeBar('auto')}>
          <Grab />
          <button type="button">Bold</button>
        </CollapsibleBar>
        <button type="button">Outside</button>
      </div>
    ));
    fireEvent.pointerEnter(root());
    vi.advanceTimersByTime(HOVER_OPEN_MS);
    const release = api!.hold();
    fireEvent.pointerLeave(root());
    vi.advanceTimersByTime(LEAVE_CLOSE_MS * 2);
    fireEvent.pointerDown(screen.getByRole('button', { name: 'Outside' }));
    fireEvent.keyDown(root(), { key: 'Escape' });
    expect(isOpen()).toBe(true);
    release();
    release(); // idempotent
    vi.advanceTimersByTime(LEAVE_CLOSE_MS - 1);
    expect(isOpen()).toBe(true);
    vi.advanceTimersByTime(1);
    expect(isOpen()).toBe(false);
  });

  it('turning to pinned closes the transient reveal state', () => {
    const bar = fakeBar('auto');
    mount(bar);
    fireEvent.click(handle());
    void bar.setMode('pinned');
    expect(root()).toHaveAttribute('data-mode', 'pinned');
    void bar.setMode('auto');
    expect(isOpen()).toBe(false);
  });
});

describe('CollapsibleBar: touch', () => {
  it('binds no hover listeners on a touch-only device; a tap opens and a tap outside closes', () => {
    stubMedia(['(hover: none)', '(pointer: coarse)']);
    const spy = vi.spyOn(HTMLElement.prototype, 'addEventListener');
    mount(fakeBar('auto'));
    const types = spy.mock.calls.filter((_, i) => spy.mock.contexts[i] === root()).map(([type]) => type);
    expect(types).not.toContain('pointerenter');
    expect(types).not.toContain('pointerleave');
    spy.mockRestore();
    fireEvent.pointerEnter(root());
    vi.advanceTimersByTime(HOVER_OPEN_MS * 2);
    expect(isOpen()).toBe(false);
    fireEvent.pointerDown(handle(), { pointerType: 'touch' });
    fireEvent.click(handle());
    expect(isOpen()).toBe(true);
    fireEvent.pointerDown(screen.getByRole('button', { name: 'Outside' }), { pointerType: 'touch' });
    expect(isOpen()).toBe(false);
  });

  it('binds hover listeners where hover is available', () => {
    const spy = vi.spyOn(HTMLElement.prototype, 'addEventListener');
    mount(fakeBar('auto'));
    const types = spy.mock.calls.filter((_, i) => spy.mock.contexts[i] === root()).map(([type]) => type);
    expect(types).toContain('pointerenter');
    spy.mockRestore();
  });
});

describe('CollapsibleBar: typing guard', () => {
  it('suppresses hover reveal while the user types in a field outside the bar; a click still opens', () => {
    mount(fakeBar('auto'), {}, () => <textarea aria-label="Body" />);
    const body = screen.getByLabelText('Body');
    body.focus();
    fireEvent.keyDown(body, { key: 'a' });
    fireEvent.pointerEnter(root());
    vi.advanceTimersByTime(HOVER_OPEN_MS);
    expect(isOpen()).toBe(false);
    fireEvent.pointerLeave(root());
    vi.advanceTimersByTime(TYPING_GUARD_MS);
    fireEvent.pointerEnter(root());
    vi.advanceTimersByTime(HOVER_OPEN_MS);
    expect(isOpen()).toBe(true);
    fireEvent.pointerDown(body);
    expect(isOpen()).toBe(false);
    fireEvent.keyDown(body, { key: 'b' });
    fireEvent.pointerDown(handle());
    fireEvent.click(handle());
    expect(isOpen()).toBe(true);
  });
});

describe('CollapsibleBar: motion and push', () => {
  it('marks reduced motion from the root attribute and from the media query', () => {
    document.documentElement.setAttribute('data-reduced-motion', '');
    mount(fakeBar('auto'));
    fireEvent.click(handle());
    expect(root()).toHaveAttribute('data-motion', 'reduced');
    cleanup();
    document.documentElement.removeAttribute('data-reduced-motion');
    stubMedia(['(prefers-reduced-motion: reduce)']);
    mount(fakeBar('auto'));
    fireEvent.click(handle());
    expect(root()).toHaveAttribute('data-motion', 'reduced');
    cleanup();
    stubMedia([]);
    mount(fakeBar('auto'));
    fireEvent.click(handle());
    expect(root()).toHaveAttribute('data-motion', 'full');
  });

  it('push mode keeps the caret outside the bar in view once the panel has slid in', () => {
    const editor = document.createElement('div');
    editor.textContent = 'hello';
    document.body.append(editor);
    const scroll = vi.fn();
    editor.scrollIntoView = scroll;
    window.getSelection()!.collapse(editor.firstChild!, 2);
    mount(fakeBar('auto'), { revealPolicy: 'push' });
    expect(root()).toHaveAttribute('data-reveal', 'push');
    fireEvent.click(handle());
    vi.advanceTimersByTime(REVEAL_MS - 1);
    expect(scroll).not.toHaveBeenCalled();
    vi.advanceTimersByTime(1);
    expect(scroll).toHaveBeenCalledWith({ block: 'nearest' });
    editor.remove();
  });

  it('shows a label on the slim handle', () => {
    mount(fakeBar('auto'), { id: 'workspaceTabs', handleLabel: 'Inbox' });
    expect(handle()).toHaveTextContent('Inbox');
    expect(handle()).toHaveAccessibleName('Show the workspace tabs');
  });
});
