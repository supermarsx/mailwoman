// RibbonPanel + RibbonGroup (t38 §2.2, decision D1): groups fold into dropdowns
// lowest priority first (ties: inline end first), open again in reverse order
// with a 1 px hysteresis, fall back to sideways scrolling with the wheel helper
// only once every group has folded, and stop a layout that does not settle.

import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import { cleanup, fireEvent, render, screen } from '@solidjs/testing-library';
import { createSignal, For, type JSX } from 'solid-js';

const wheel = vi.hoisted(() => ({ bind: vi.fn(), unbind: vi.fn() }));
vi.mock('../horizontalWheel.ts', () => ({
  bindHorizontalWheel: (el: HTMLElement) => {
    wheel.bind(el);
    return wheel.unbind;
  },
  horizontalWheel: () => {},
}));

import { RibbonPanel } from './RibbonPanel.tsx';
import { RibbonGroup, RibbonItem } from './RibbonGroup.tsx';
import { CollapsibleBar } from './CollapsibleBar.tsx';
import type { BarModeHandle } from './useBarMode.ts';

// ── a fake layout: the panel is `width` wide, each group as wide as its spec,
// a folded group 60 px, plus 8 px between groups ───────────────────────────
const FOLDED_PX = 60;
const GAP_PX = 8;
let width = 1000;
let clientWidthOverride: (() => number) | null = null;
const groupWidths = new Map<string, number>();

function contentWidth(panel: Element): number {
  const groups = Array.from(panel.querySelectorAll('[data-ribbon-group]'));
  const sum = groups.reduce(
    (acc, g) => acc + (g.getAttribute('data-folded') === 'true' ? FOLDED_PX : (groupWidths.get(g.getAttribute('aria-label')!) ?? 0)),
    0,
  );
  return sum + Math.max(0, groups.length - 1) * GAP_PX;
}

const observers: Array<{ cb: ResizeObserverCallback; targets: Element[] }> = [];
class FakeResizeObserver {
  private readonly record: { cb: ResizeObserverCallback; targets: Element[] };
  constructor(cb: ResizeObserverCallback) {
    this.record = { cb, targets: [] };
    observers.push(this.record);
  }
  observe(target: Element): void {
    this.record.targets.push(target);
  }
  unobserve(): void {}
  disconnect(): void {
    this.record.targets = [];
  }
}
/** The panel's width changed: what a real ResizeObserver would report. */
function resize(next: number): void {
  width = next;
  for (const o of observers) if (o.targets.length > 0) o.cb([], o as unknown as ResizeObserver);
}

const FRAME = 16;
const frame = (n = 1): void => {
  vi.advanceTimersByTime(FRAME * n);
};
const settle = (): void => frame(40);

const panelEl = (): HTMLElement => document.querySelector('[data-ribbon-panel]') as HTMLElement;
const foldedLabels = (): string[] =>
  Array.from(document.querySelectorAll('[data-ribbon-group][data-folded="true"]')).map((g) => g.getAttribute('aria-label')!);

interface Spec {
  label: string;
  priority: number;
  width: number;
}

function Groups(props: { specs: Spec[] }): JSX.Element {
  return (
    <For each={props.specs}>
      {(spec) => (
        <RibbonGroup label={spec.label} priority={spec.priority} icon="paste">
          <RibbonItem size="large">
            <button type="button">{`${spec.label} big`}</button>
          </RibbonItem>
          <RibbonItem>
            <button type="button">{`${spec.label} one`}</button>
          </RibbonItem>
        </RibbonGroup>
      )}
    </For>
  );
}

function mountPanel(specs: Spec[], onLayout?: (s: { folded: number; scrolling: boolean }) => void) {
  for (const s of specs) groupWidths.set(s.label, s.width);
  const result = render(() => (
    <RibbonPanel label="Home" {...(onLayout ? { onLayout } : {})}>
      <Groups specs={specs} />
    </RibbonPanel>
  ));
  const panel = panelEl();
  Object.defineProperty(panel, 'clientWidth', { configurable: true, get: () => clientWidthOverride?.() ?? width });
  Object.defineProperty(panel, 'scrollWidth', { configurable: true, get: () => contentWidth(panel) });
  return result;
}

const SPECS: Spec[] = [
  { label: 'Clipboard', priority: 100, width: 200 },
  { label: 'Font', priority: 50, width: 300 },
  { label: 'Table', priority: 10, width: 250 },
  { label: 'Delivery', priority: 50, width: 200 },
];
// Expanded: 200 + 300 + 250 + 200 + 3×8 = 974.

beforeEach(() => {
  vi.useFakeTimers({ toFake: ['setTimeout', 'clearTimeout', 'requestAnimationFrame', 'cancelAnimationFrame', 'Date', 'queueMicrotask'] });
  vi.stubGlobal('ResizeObserver', FakeResizeObserver);
  width = 1000;
  clientWidthOverride = null;
  groupWidths.clear();
  observers.length = 0;
  wheel.bind.mockClear();
  wheel.unbind.mockClear();
});
afterEach(() => {
  cleanup();
  vi.useRealTimers();
  vi.unstubAllGlobals();
});

describe('RibbonPanel: folding (D1)', () => {
  it('leaves every group open when the row fits', () => {
    mountPanel(SPECS);
    settle();
    expect(foldedLabels()).toEqual([]);
    expect(panelEl()).toHaveAttribute('data-scroll', 'false');
    expect(document.querySelectorAll('[data-ribbon-controls]')).toHaveLength(4);
  });

  it('folds the lowest priority first, then on a tie the group nearest the inline end, one per frame', () => {
    width = 800;
    mountPanel(SPECS);
    frame();
    // 974 > 800: Table (10) goes first.
    expect(foldedLabels()).toEqual(['Table']);
    frame();
    // 784 ≤ 800: fits, nothing else folds.
    expect(foldedLabels()).toEqual(['Table']);
    resize(700);
    frame();
    // 784 > 700: Font and Delivery tie at 50; Delivery is further along the row.
    expect(foldedLabels()).toEqual(['Table', 'Delivery']);
    settle();
    // 200 + 300 + 60 + 60 + 24 = 644 fits.
    expect(foldedLabels()).toEqual(['Table', 'Delivery']);
    resize(600);
    settle();
    expect(foldedLabels()).toEqual(['Font', 'Table', 'Delivery']);
    expect(panelEl()).toHaveAttribute('data-scroll', 'false');
  });

  it('turns a folded group into one large dropdown button with its own label', () => {
    width = 800;
    mountPanel(SPECS);
    settle();
    const group = screen.getByRole('group', { name: 'Table' });
    const more = group.querySelector('[data-ribbon-more]') as HTMLElement;
    expect(more).toHaveAttribute('aria-haspopup', 'true');
    expect(more).toHaveAttribute('aria-expanded', 'false');
    expect(more).toHaveAccessibleName('More Table commands');
    expect(more).toHaveTextContent('Table');
    // The controls wait, hidden, in the popover.
    const popover = group.querySelector('[data-ribbon-popover]') as HTMLElement;
    expect(popover).toHaveAttribute('hidden');
    expect(popover.querySelector('[data-ribbon-controls]')).not.toBeNull();
  });

  it('opens groups again in reverse order, each only 1 px past the width it folded at', () => {
    width = 600;
    mountPanel(SPECS);
    settle();
    // Folded at: Table 974, Delivery 784, Font 644.
    expect(foldedLabels()).toEqual(['Font', 'Table', 'Delivery']);
    resize(644);
    settle();
    expect(foldedLabels()).toEqual(['Font', 'Table', 'Delivery']);
    resize(645);
    settle();
    expect(foldedLabels()).toEqual(['Table', 'Delivery']);
    resize(785);
    settle();
    expect(foldedLabels()).toEqual(['Table']);
    resize(975);
    settle();
    expect(foldedLabels()).toEqual([]);
  });

  it('scrolls sideways with the wheel helper only once every group has folded, and stops when it fits', () => {
    width = 200;
    mountPanel(SPECS);
    settle();
    expect(foldedLabels()).toHaveLength(4);
    // 4 × 60 + 24 = 264 > 200.
    expect(panelEl()).toHaveAttribute('data-scroll', 'true');
    expect(wheel.bind).toHaveBeenCalledTimes(1);
    expect(wheel.bind).toHaveBeenCalledWith(panelEl());
    resize(300);
    settle();
    expect(panelEl()).toHaveAttribute('data-scroll', 'false');
    expect(wheel.unbind).toHaveBeenCalledTimes(1);
    expect(foldedLabels()).toHaveLength(4);
  });

  it('never folds while the wheel fallback is off and the row fits exactly', () => {
    width = 974;
    mountPanel(SPECS);
    settle();
    expect(foldedLabels()).toEqual([]);
    expect(wheel.bind).not.toHaveBeenCalled();
  });

  it('stops a layout that never settles and reports it', () => {
    const errors = vi.spyOn(console, 'error').mockImplementation(() => {});
    let flip = false;
    // A panel whose width swings on every read: fold, open, fold, …
    clientWidthOverride = () => {
      flip = !flip;
      return flip ? 500 : 2000;
    };
    const layouts: number[] = [];
    mountPanel(SPECS, (s) => layouts.push(s.folded));
    frame(200);
    expect(layouts.length).toBeLessThanOrEqual(4 * SPECS.length + 9);
    const count = layouts.length;
    frame(200);
    expect(layouts.length).toBe(count);
    expect(errors).toHaveBeenCalledWith(expect.stringContaining('did not settle'));
    errors.mockRestore();
  });

  it('keeps the controls (and their state) when a group folds and opens again', () => {
    groupWidths.set('Font', 400);
    render(() => (
      <RibbonPanel label="Home">
        <RibbonGroup label="Font" priority={1}>
          <RibbonItem inlineSize="8rem">
            <input aria-label="Font size" />
          </RibbonItem>
        </RibbonGroup>
      </RibbonPanel>
    ));
    const panel = panelEl();
    Object.defineProperty(panel, 'clientWidth', { configurable: true, get: () => width });
    Object.defineProperty(panel, 'scrollWidth', { configurable: true, get: () => contentWidth(panel) });
    const input = screen.getByLabelText('Font size') as HTMLInputElement;
    expect(input.parentElement).toHaveAttribute('data-ribbon-item', 'small');
    expect(input.parentElement!.style.inlineSize).toBe('8rem');
    input.value = '12';
    resize(300);
    settle();
    expect(foldedLabels()).toEqual(['Font']);
    expect(screen.getByLabelText('Font size')).toBe(input);
    resize(500);
    settle();
    expect(foldedLabels()).toEqual([]);
    expect(screen.getByLabelText('Font size')).toBe(input);
    expect(input.value).toBe('12');
  });

  it('a group outside a panel never folds', () => {
    render(() => (
      <RibbonGroup label="Lonely">
        <button type="button">x</button>
      </RibbonGroup>
    ));
    expect(screen.getByRole('group', { name: 'Lonely' })).toHaveAttribute('data-folded', 'false');
  });

  it('forgets a group that leaves while folded', () => {
    width = 800;
    const [specs, setSpecs] = createSignal(SPECS);
    for (const s of SPECS) groupWidths.set(s.label, s.width);
    render(() => (
      <RibbonPanel label="Home">
        <Groups specs={specs()} />
      </RibbonPanel>
    ));
    const panel = panelEl();
    Object.defineProperty(panel, 'clientWidth', { configurable: true, get: () => width });
    Object.defineProperty(panel, 'scrollWidth', { configurable: true, get: () => contentWidth(panel) });
    settle();
    expect(foldedLabels()).toEqual(['Table']);
    setSpecs(SPECS.filter((s) => s.label !== 'Table'));
    resize(2000);
    settle();
    expect(foldedLabels()).toEqual([]);
  });
});

describe('RibbonGroup: the folded dropdown', () => {
  function fakeAutoBar(): BarModeHandle {
    return {
      mode: () => 'auto',
      locked: () => false,
      saving: () => false,
      error: () => '',
      setMode: () => Promise.resolve(true),
    };
  }

  function mountInBar() {
    groupWidths.set('Table', 400);
    render(() => (
      <div>
        <CollapsibleBar id="composerRibbon" revealPolicy="push" bar={fakeAutoBar()}>
          <RibbonPanel label="Layout">
            <RibbonGroup label="Table" priority={1} icon="table">
              <button type="button">Insert row</button>
              <button type="button">Delete row</button>
            </RibbonGroup>
          </RibbonPanel>
        </CollapsibleBar>
        <button type="button">Elsewhere</button>
      </div>
    ));
    const panel = panelEl();
    Object.defineProperty(panel, 'clientWidth', { configurable: true, get: () => width });
    Object.defineProperty(panel, 'scrollWidth', { configurable: true, get: () => contentWidth(panel) });
    resize(100);
    settle();
  }
  const bar = (): HTMLElement => document.querySelector('[data-bar]') as HTMLElement;
  const more = (): HTMLElement => document.querySelector('[data-ribbon-more]') as HTMLElement;
  const popover = (): HTMLElement => document.querySelector('[data-ribbon-popover]') as HTMLElement;

  it('opens a popover with the same controls, holds the bar, and Esc closes only the popover first', () => {
    mountInBar();
    fireEvent.click(document.querySelector('[data-bar-handle]')!);
    expect(bar()).toHaveAttribute('data-expanded', 'true');
    fireEvent.click(more());
    expect(more()).toHaveAttribute('aria-expanded', 'true');
    expect(popover()).not.toHaveAttribute('hidden');
    expect(popover()).toHaveAttribute('id', more().getAttribute('aria-controls'));
    const row = screen.getByRole('button', { name: 'Insert row' });
    expect(popover().contains(row)).toBe(true);
    row.focus();
    // The bar is held: leaving does not close it.
    fireEvent.pointerLeave(bar());
    vi.advanceTimersByTime(1000);
    expect(bar()).toHaveAttribute('data-expanded', 'true');
    fireEvent.keyDown(row, { key: 'Escape' });
    expect(popover()).toHaveAttribute('hidden');
    expect(document.activeElement).toBe(more());
    expect(bar()).toHaveAttribute('data-expanded', 'true');
    fireEvent.keyDown(more(), { key: 'Escape' });
    expect(bar()).toHaveAttribute('data-expanded', 'false');
  });

  it('closes the popover on a click outside the group', () => {
    mountInBar();
    fireEvent.click(document.querySelector('[data-bar-handle]')!);
    fireEvent.click(more());
    fireEvent.pointerDown(screen.getByRole('button', { name: 'Elsewhere' }));
    expect(popover()).toHaveAttribute('hidden');
    expect(more()).toHaveAttribute('aria-expanded', 'false');
  });

  it('closes its popover when the ribbon grows and the group opens again', () => {
    mountInBar();
    fireEvent.click(document.querySelector('[data-bar-handle]')!);
    fireEvent.click(more());
    resize(1000);
    settle();
    expect(document.querySelector('[data-ribbon-popover]')).toBeNull();
    expect(screen.getByRole('group', { name: 'Table', hidden: true })).toBeTruthy();
    expect(document.querySelector('[data-ribbon-group] > [data-ribbon-body] > [data-ribbon-controls]')).not.toBeNull();
  });
});
