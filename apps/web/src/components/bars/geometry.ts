// `assertRibbonGeometry(root)`: the 3-row ribbon contract as a check (t38 §2.2).
//
// Returns every violation under `root`:
//   - `height`: a group body, or a visible controls grid, is not
//     3 × row + 2 × gap tall (±0.5 px);
//   - `overflow`: an element whose content is taller than its box
//     (`scrollHeight > clientHeight`), which would scroll or clip vertically;
//   - `outside`: a control's box sticks out of its group's box (±0.5 px).
// Hidden subtrees (`[hidden]`, a closed group popover) are skipped. Row and gap
// come from the `--mw-ribbon-row` / `--mw-ribbon-gap` custom properties on
// `root`, or the options (jsdom computes no layout, tests pass them). In dev
// builds a non-empty result is also logged with `console.error`.

import { RIBBON_GAP, RIBBON_ROW_FINE, ribbonControlsHeight } from './ribbonMetrics.ts';

export type RibbonViolationKind = 'height' | 'overflow' | 'outside';

export interface RibbonViolation {
  kind: RibbonViolationKind;
  element: Element;
  detail: string;
}

export interface RibbonGeometryOptions {
  /** Row height in px; default: `--mw-ribbon-row` on `root`, else 28. */
  row?: number;
  /** Row gap in px; default: `--mw-ribbon-gap` on `root`, else 2. */
  gap?: number;
  /** Tolerance in px for heights and boxes (default 0.5). */
  tolerance?: number;
  /** Log violations with `console.error` in dev builds (default true). */
  log?: boolean;
}

function cssPx(element: Element, property: string): number | null {
  const raw = getComputedStyle(element).getPropertyValue(property).trim();
  const value = Number.parseFloat(raw);
  return raw.endsWith('px') && Number.isFinite(value) ? value : null;
}

function isHidden(element: Element, root: Element): boolean {
  for (let el: Element | null = element; el && el !== root.parentElement; el = el.parentElement) {
    if (el.hasAttribute('hidden')) return true;
  }
  return false;
}

function describe(element: Element): string {
  const tag = element.tagName.toLowerCase();
  const attrs = ['data-ribbon-group', 'data-ribbon-body', 'data-ribbon-controls', 'data-ribbon-item']
    .filter((a) => element.hasAttribute(a))
    .join(',');
  const label = element.getAttribute('aria-label');
  return `${tag}${attrs ? `[${attrs}]` : ''}${label ? ` "${label}"` : ''}`;
}

export function assertRibbonGeometry(root: Element, options: RibbonGeometryOptions = {}): RibbonViolation[] {
  const row = options.row ?? cssPx(root, '--mw-ribbon-row') ?? RIBBON_ROW_FINE;
  const gap = options.gap ?? cssPx(root, '--mw-ribbon-gap') ?? RIBBON_GAP;
  const tolerance = options.tolerance ?? 0.5;
  const expected = ribbonControlsHeight(row, gap);
  const violations: RibbonViolation[] = [];

  const sized = root.querySelectorAll('[data-ribbon-body], [data-ribbon-controls]');
  for (const element of Array.from(sized)) {
    if (isHidden(element, root)) continue;
    const height = element.getBoundingClientRect().height;
    if (Math.abs(height - expected) > tolerance) {
      violations.push({ kind: 'height', element, detail: `${describe(element)} is ${height}px tall, expected ${expected}px` });
    }
  }

  const all = [root, ...Array.from(root.querySelectorAll('*'))];
  for (const element of all) {
    if (!(element instanceof HTMLElement) || isHidden(element, root)) continue;
    // scrollHeight is rounded to whole pixels: one pixel of slack.
    if (element.scrollHeight > element.clientHeight + 1 && element.clientHeight > 0) {
      violations.push({
        kind: 'overflow',
        element,
        detail: `${describe(element)} holds ${element.scrollHeight}px in ${element.clientHeight}px`,
      });
    }
  }

  for (const group of Array.from(root.querySelectorAll('[data-ribbon-group]'))) {
    if (isHidden(group, root)) continue;
    const box = group.getBoundingClientRect();
    const children = Array.from(group.querySelectorAll('[data-ribbon-controls] > *, [data-ribbon-more]'));
    for (const control of children) {
      if (isHidden(control, root) || control.closest('[data-ribbon-popover]')) continue;
      const r = control.getBoundingClientRect();
      if (
        r.top < box.top - tolerance ||
        r.bottom > box.bottom + tolerance ||
        r.left < box.left - tolerance ||
        r.right > box.right + tolerance
      ) {
        violations.push({
          kind: 'outside',
          element: control,
          detail: `${describe(control)} [${r.left},${r.top} ${r.right},${r.bottom}] outside ${describe(group)} [${box.left},${box.top} ${box.right},${box.bottom}]`,
        });
      }
    }
  }

  if (violations.length > 0 && options.log !== false && import.meta.env.DEV) {
    console.error(`assertRibbonGeometry: ${violations.length} violation(s)`, violations.map((v) => v.detail));
  }
  return violations;
}
