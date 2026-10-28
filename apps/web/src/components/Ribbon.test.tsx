import { describe, it, expect, beforeEach, vi } from 'vitest';
import { createSignal } from 'solid-js';
import { screen, fireEvent } from '@solidjs/testing-library';
import { Ribbon } from './Ribbon.tsx';
import { renderWithApp } from './appHarness.tsx';
import { assertRibbonGeometry, type BarMode, type BarModeHandle } from './bars/index.ts';

// The ribbon is the named WCAG 2.2 target (t8-e1): a WAI-ARIA tablist with
// roving tabindex over Home / View / Folder, each controlling the panel.
describe('Ribbon', () => {
  beforeEach(() => localStorage.clear());

  it('exposes a tablist with the three ribbon tabs', () => {
    renderWithApp(() => <Ribbon onCompose={() => undefined} onOpenSettings={() => undefined} />);
    const tablist = screen.getByRole('tablist', { name: 'Ribbon' });
    expect(tablist).toBeInTheDocument();
    expect(screen.getAllByRole('tab')).toHaveLength(3);
    expect(screen.getByRole('tab', { name: 'Home' })).toHaveAttribute('aria-selected', 'true');
  });

  it('uses roving tabindex: only the selected tab is in the tab order', () => {
    renderWithApp(() => <Ribbon onCompose={() => undefined} onOpenSettings={() => undefined} />);
    const home = screen.getByRole('tab', { name: 'Home' });
    const view = screen.getByRole('tab', { name: 'View' });
    expect(home).toHaveAttribute('tabindex', '0');
    expect(view).toHaveAttribute('tabindex', '-1');
  });

  it('ArrowRight moves selection (activation follows focus) and shows that panel', () => {
    renderWithApp(() => <Ribbon onCompose={() => undefined} onOpenSettings={() => undefined} />);
    const tablist = screen.getByRole('tablist', { name: 'Ribbon' });
    fireEvent.keyDown(tablist, { key: 'ArrowRight' });

    const view = screen.getByRole('tab', { name: 'View' });
    expect(view).toHaveAttribute('aria-selected', 'true');
    expect(view).toHaveAttribute('tabindex', '0');
    // The View panel (theme/density/settings groups) is now shown.
    expect(screen.getByRole('tabpanel')).toHaveAttribute('aria-labelledby', 'ribbon-tab-view');
  });

  it('is the mailRibbon collapsible bar with a pin and no legacy collapse button', () => {
    const bar = fakeBar('pinned');
    renderWithApp(() => <Ribbon onCompose={() => undefined} onOpenSettings={() => undefined} bar={bar} />);
    expect(document.querySelector('[data-bar="mailRibbon"]')).toHaveAttribute('data-mode', 'pinned');
    expect(screen.queryByRole('button', { name: /^(Collapse|Expand)/ })).toBeNull();
    const pin = document.querySelector('[data-bar-pin="mailRibbon"]') as HTMLButtonElement;
    expect(pin).toHaveAttribute('aria-pressed', 'true');
    fireEvent.click(pin);
    expect(bar.calls).toEqual(['auto']);
  });

  it('auto-hide keeps the tab names; a click on a tab opens the groups and Esc closes them', () => {
    const bar = fakeBar('auto');
    renderWithApp(() => <Ribbon onCompose={() => undefined} onOpenSettings={() => undefined} bar={bar} />);
    const root = document.querySelector('[data-bar="mailRibbon"]') as HTMLElement;
    const view = screen.getByRole('tab', { name: 'View' });
    expect(root).toHaveAttribute('data-expanded', 'false');
    expect(view).toHaveAttribute('aria-expanded', 'false');
    fireEvent.pointerDown(view);
    fireEvent.click(view);
    expect(root).toHaveAttribute('data-expanded', 'true');
    expect(view).toHaveAttribute('aria-selected', 'true');
    expect(view).toHaveAttribute('aria-expanded', 'true');
    const density = screen.getByRole('button', { name: 'compact' });
    density.focus();
    fireEvent.keyDown(density, { key: 'Escape' });
    expect(root).toHaveAttribute('data-expanded', 'false');
    expect(document.activeElement).toBe(view);
  });

  it('every group of every tab is a 3-row RibbonGroup that passes assertRibbonGeometry', async () => {
    vi.stubGlobal('ResizeObserver', undefined);
    renderWithApp(() => <Ribbon onCompose={() => undefined} onOpenSettings={() => undefined} bar={fakeBar('pinned')} />);
    const tablist = screen.getByRole('tablist', { name: 'Ribbon' });
    expect(screen.getByRole('button', { name: 'Compose' }).closest('[data-ribbon-item]')).toHaveAttribute('data-ribbon-item', 'large');
    expect(screen.getByRole('button', { name: 'Log out' }).closest('[data-ribbon-item]')).toHaveAttribute('data-ribbon-item', 'small');
    const seen: string[] = [];
    for (let tab = 0; tab < 3; tab++) {
      const groups = Array.from(document.querySelectorAll('#ribbon-panel [data-ribbon-group]'));
      expect(groups.length).toBeGreaterThan(0);
      for (const g of groups) seen.push(g.getAttribute('data-testid') ?? '');
      layOut(groups);
      expect(assertRibbonGeometry(document.getElementById('ribbon-panel')!, { row: 28, gap: 2, log: false })).toEqual([]);
      fireEvent.keyDown(tablist, { key: 'ArrowRight' });
    }
    expect(seen).toEqual([
      'ribbon-group-new', 'ribbon-group-session',
      'ribbon-group-theme', 'ribbon-group-density', 'ribbon-group-settings',
      'ribbon-group-folders',
    ]);
    vi.unstubAllGlobals();
  });
});

function fakeBar(initial: BarMode): BarModeHandle & { calls: BarMode[] } {
  const [mode, setMode] = createSignal<BarMode>(initial);
  const calls: BarMode[] = [];
  return {
    calls,
    mode,
    locked: () => false,
    saving: () => false,
    error: () => '',
    setMode: (m) => {
      calls.push(m);
      setMode(m);
      return Promise.resolve(true);
    },
  };
}

function rect(top: number, left: number, height: number, width: number): DOMRect {
  return { top, left, height, width, bottom: top + height, right: left + width, x: left, y: top, toJSON: () => ({}) } as DOMRect;
}

/** jsdom computes no layout: give each group the box the 3-row contract produces (28 px rows, 2 px gap). */
function layOut(groups: Element[]): void {
  groups.forEach((g, i) => {
    const left = i * 300;
    const set = (el: Element, r: DOMRect): void => {
      (el as HTMLElement).getBoundingClientRect = () => r;
    };
    set(g, rect(0, left, 104, 280));
    set(g.querySelector('[data-ribbon-body]')!, rect(0, left, 88, 280));
    set(g.querySelector('[data-ribbon-controls]')!, rect(0, left, 88, 280));
    let small = 0;
    let col = 0;
    for (const item of Array.from(g.querySelectorAll('[data-ribbon-controls] > [data-ribbon-item]'))) {
      if (item.getAttribute('data-ribbon-item') === 'large') {
        set(item, rect(0, left + col * 60, 88, 56));
        col++;
      } else {
        set(item, rect(30 * (small % 3), left + col * 60, 28, 56));
        if (++small % 3 === 0) col++;
      }
    }
  });
}
