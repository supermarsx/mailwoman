// assertRibbonGeometry (t38 §2.2): a demo-free mount of real RibbonPanel /
// RibbonGroup markup, with the layout jsdom does not compute supplied per
// element. It finds a wrong controls height, vertical overflow and a control
// outside its group, and skips a closed group popover.

import { afterEach, describe, expect, it, vi } from 'vitest';
import { cleanup, render } from '@solidjs/testing-library';
import { RibbonPanel } from './RibbonPanel.tsx';
import { RibbonGroup, RibbonItem } from './RibbonGroup.tsx';
import { assertRibbonGeometry } from './geometry.ts';
import { RIBBON_ROW_COARSE, RIBBON_ROW_FINE, ribbonControlsHeight } from './ribbonMetrics.ts';

function rect(top: number, left: number, height: number, width: number): DOMRect {
  return { top, left, height, width, bottom: top + height, right: left + width, x: left, y: top, toJSON: () => ({}) } as DOMRect;
}
function setRect(el: Element, r: DOMRect): void {
  (el as HTMLElement).getBoundingClientRect = () => r;
}
function setScroll(el: Element, scrollHeight: number, clientHeight: number): void {
  Object.defineProperty(el, 'scrollHeight', { configurable: true, get: () => scrollHeight });
  Object.defineProperty(el, 'clientHeight', { configurable: true, get: () => clientHeight });
}

/** A ribbon with two groups laid out to the contract at 28 px rows. */
function mountRibbon() {
  vi.stubGlobal('ResizeObserver', undefined);
  render(() => (
    <RibbonPanel label="Home">
      <RibbonGroup label="Clipboard" testId="g1">
        <RibbonItem size="large">
          <button type="button">Paste</button>
        </RibbonItem>
        <RibbonItem>
          <button type="button">Cut</button>
        </RibbonItem>
        <RibbonItem>
          <button type="button">Copy</button>
        </RibbonItem>
      </RibbonGroup>
      <RibbonGroup label="Font" testId="g2">
        <RibbonItem>
          <button type="button">Bold</button>
        </RibbonItem>
      </RibbonGroup>
    </RibbonPanel>
  ));
  const groups = Array.from(document.querySelectorAll('[data-ribbon-group]'));
  groups.forEach((g, i) => {
    const left = i * 200;
    setRect(g, rect(0, left, 104, 180));
    setRect(g.querySelector('[data-ribbon-body]')!, rect(0, left, 88, 180));
    setRect(g.querySelector('[data-ribbon-controls]')!, rect(0, left, 88, 180));
    let small = 0;
    for (const item of Array.from(g.querySelectorAll('[data-ribbon-controls] > *'))) {
      setRect(item, item.getAttribute('data-ribbon-item') === 'large' ? rect(0, left, 88, 50) : rect(30 * small++, left + 52, 28, 60));
    }
  });
  return groups;
}

afterEach(() => {
  cleanup();
  vi.unstubAllGlobals();
});

describe('ribbon metrics', () => {
  it('three rows and two gaps: 88 px on a fine pointer, 136 px on a coarse one', () => {
    expect(ribbonControlsHeight()).toBe(88);
    expect(ribbonControlsHeight(RIBBON_ROW_FINE, 2)).toBe(3 * 28 + 2 * 2);
    expect(ribbonControlsHeight(RIBBON_ROW_COARSE, 2)).toBe(136);
  });
});

describe('assertRibbonGeometry', () => {
  it('passes a ribbon laid out to the contract', () => {
    mountRibbon();
    expect(assertRibbonGeometry(document.body, { row: 28, gap: 2, log: false })).toEqual([]);
  });

  it('reports a group whose controls are not exactly three rows tall (more or fewer)', () => {
    const [g1, g2] = mountRibbon();
    setRect(g1!.querySelector('[data-ribbon-controls]')!, rect(0, 0, 118, 180)); // a 4th row
    setRect(g2!.querySelector('[data-ribbon-body]')!, rect(0, 200, 58, 180)); // two rows
    const violations = assertRibbonGeometry(document.body, { row: 28, gap: 2, log: false });
    expect(violations.map((v) => v.kind)).toEqual(['height', 'height']);
    expect(violations[0]!.detail).toContain('118px tall, expected 88px');
    // ±0.5 px is tolerated.
    setRect(g1!.querySelector('[data-ribbon-controls]')!, rect(0, 0, 88.4, 180));
    setRect(g2!.querySelector('[data-ribbon-body]')!, rect(0, 200, 87.6, 180));
    expect(assertRibbonGeometry(document.body, { row: 28, gap: 2, log: false })).toEqual([]);
  });

  it('checks coarse rows at 136 px', () => {
    mountRibbon();
    const violations = assertRibbonGeometry(document.body, { row: 44, gap: 2, log: false });
    expect(violations.filter((v) => v.kind === 'height')).toHaveLength(4);
    expect(violations[0]!.detail).toContain('expected 136px');
  });

  it('reports any element that overflows vertically', () => {
    const [g1] = mountRibbon();
    const cut = g1!.querySelector('button')!;
    setScroll(cut, 40, 28);
    setScroll(g1!, 104, 104);
    const violations = assertRibbonGeometry(document.body, { row: 28, gap: 2, log: false });
    expect(violations).toHaveLength(1);
    expect(violations[0]).toMatchObject({ kind: 'overflow', element: cut });
    // A rounding pixel is not overflow.
    setScroll(cut, 29, 28);
    expect(assertRibbonGeometry(document.body, { row: 28, gap: 2, log: false })).toEqual([]);
  });

  it('does not count a collapsible bar root, whose overlay panel hangs below it, as overflow', () => {
    mountRibbon();
    const bar = document.createElement('div');
    bar.setAttribute('data-bar', 'mailTopBar');
    document.body.append(bar);
    setScroll(bar, 60, 12);
    expect(assertRibbonGeometry(document.body, { row: 28, gap: 2, log: false })).toEqual([]);
    bar.remove();
  });

  it('skips the content of a closed <details>, which is not rendered', () => {
    const [g1] = mountRibbon();
    const details = document.createElement('details');
    const summary = document.createElement('summary');
    const list = document.createElement('div');
    details.append(summary, list);
    g1!.querySelector('[data-ribbon-controls] > *')!.append(details);
    setScroll(list, 19, 17);
    expect(assertRibbonGeometry(document.body, { row: 28, gap: 2, log: false })).toEqual([]);
    details.open = true;
    const violations = assertRibbonGeometry(document.body, { row: 28, gap: 2, log: false });
    expect(violations).toHaveLength(1);
    expect(violations[0]).toMatchObject({ kind: 'overflow', element: list });
  });

  it('reports a control outside its group box', () => {
    const [, g2] = mountRibbon();
    const bold = g2!.querySelector('[data-ribbon-item]')!;
    setRect(bold, rect(90, 252, 28, 60)); // below the controls, inside no row
    const violations = assertRibbonGeometry(document.body, { row: 28, gap: 2, log: false });
    expect(violations).toHaveLength(1);
    expect(violations[0]).toMatchObject({ kind: 'outside', element: bold });
  });

  it('skips a hidden subtree and logs in dev builds', () => {
    const [g1] = mountRibbon();
    g1!.setAttribute('hidden', '');
    setRect(g1!.querySelector('[data-ribbon-controls]')!, rect(0, 0, 0, 0));
    expect(assertRibbonGeometry(document.body, { row: 28, gap: 2, log: false })).toEqual([]);
    g1!.removeAttribute('hidden');
    const errors = vi.spyOn(console, 'error').mockImplementation(() => {});
    expect(assertRibbonGeometry(document.body, { row: 28, gap: 2 })).toHaveLength(1);
    expect(errors).toHaveBeenCalledWith(expect.stringContaining('1 violation'), expect.any(Array));
    errors.mockRestore();
  });

  it('reads row and gap from the custom properties when not given', () => {
    mountRibbon();
    document.body.style.setProperty('--mw-ribbon-row', '44px');
    document.body.style.setProperty('--mw-ribbon-gap', '2px');
    expect(assertRibbonGeometry(document.body, { log: false }).filter((v) => v.kind === 'height')).toHaveLength(4);
    document.body.style.removeProperty('--mw-ribbon-row');
    document.body.style.removeProperty('--mw-ribbon-gap');
  });
});
