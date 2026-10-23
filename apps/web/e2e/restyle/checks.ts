import AxeBuilder from '@axe-core/playwright';
import type { Page } from '@playwright/test';

/**
 * The rendered checks behind contrast.restyle.ts, shared with selftest.restyle.ts,
 * which runs them on a fixed page with one good and one bad case per check.
 */
const FOCUS_STOPS = 12;

export interface Finding {
  check: string;
  element: string;
  detail: string;
}

/** In-page helpers, installed once per document as `window.__t32`. */
function installHelpers(): void {
  const w = window as unknown as { __t32?: unknown };
  if (w.__t32) return;
  const canvas = document.createElement('canvas');
  canvas.width = canvas.height = 1;
  const g = canvas.getContext('2d', { willReadFrequently: true })!;
  /** Any CSS colour → [r, g, b, a] in sRGB 0-255 / 0-1, through the canvas. */
  const rgba = (css: string): [number, number, number, number] => {
    if (!css || css === 'transparent') return [0, 0, 0, 0];
    g.clearRect(0, 0, 1, 1);
    g.fillStyle = '#000';
    g.fillStyle = css;
    g.fillRect(0, 0, 1, 1);
    const d = g.getImageData(0, 0, 1, 1).data;
    return [d[0]!, d[1]!, d[2]!, d[3]! / 255];
  };
  const over = (top: number[], bottom: number[]) => {
    const a = top[3]!;
    return [0, 1, 2].map((i) => top[i]! * a + bottom[i]! * (1 - a)).concat(1);
  };
  /** Background behind `el` (its own fill included when `self`), composited to opaque. */
  const backdrop = (el: Element | null, self: boolean) => {
    const layers: number[][] = [];
    let node: Element | null = self ? el : (el?.parentElement ?? null);
    while (node) {
      const c = rgba(getComputedStyle(node).backgroundColor);
      if (c[3] > 0) layers.push(c);
      if (c[3] >= 1) break;
      node = node.parentElement;
    }
    let acc = [255, 255, 255, 1];
    for (const l of layers.reverse()) acc = over(l, acc);
    return acc;
  };
  const lum = (c: number[]) => {
    const f = (v: number) => {
      const s = v / 255;
      return s <= 0.03928 ? s / 12.92 : ((s + 0.055) / 1.055) ** 2.4;
    };
    return 0.2126 * f(c[0]!) + 0.7152 * f(c[1]!) + 0.0722 * f(c[2]!);
  };
  const ratio = (a: number[], b: number[]) => {
    const [l1, l2] = [lum(a), lum(b)].sort((x, y) => y - x);
    return Math.round(((l1! + 0.05) / (l2! + 0.05)) * 100) / 100;
  };
  const describe = (el: Element) => {
    const cls = [...el.classList].filter((c) => /__|--|^btn|^field/.test(c)).join('.');
    const name = (el.getAttribute('aria-label') ?? el.textContent ?? '').replace(/\s+/g, ' ').trim().slice(0, 32);
    const role = el.getAttribute('role');
    return `${el.tagName.toLowerCase()}${cls ? '.' + cls : ''}${role ? `[role=${role}]` : ''} "${name}"`;
  };
  /** Rendered and reachable: not inside an `inert` subtree (the page behind a dialog or the phone reader). */
  const visible = (el: Element) => {
    if (el.closest('[inert]')) return false;
    const r = el.getBoundingClientRect();
    const cs = getComputedStyle(el);
    return r.width > 0 && r.height > 0 && cs.visibility !== 'hidden' && cs.display !== 'none';
  };
  const hex = (c: number[]) => `rgb(${c.slice(0, 3).map(Math.round).join(',')})`;
  /** Split a computed box-shadow list into { color, x, y, blur, spread, inset }. */
  const shadows = (v: string) => {
    if (!v || v === 'none') return [];
    const parts: string[] = [];
    let depth = 0;
    let cur = '';
    for (const ch of v) {
      if (ch === '(') depth++;
      if (ch === ')') depth--;
      if (ch === ',' && depth === 0) {
        parts.push(cur);
        cur = '';
      } else cur += ch;
    }
    parts.push(cur);
    return parts.map((p) => {
      const color = (p.match(/(rgba?|color|oklch|oklab|lab|lch|hsla?)\([^)]*\)|#[0-9a-f]+/i) ?? ['transparent'])[0];
      const nums = (p.replace(color, '').match(/-?[\d.]+px|\b0\b/g) ?? []).map((n) => parseFloat(n));
      return { color, x: nums[0] ?? 0, y: nums[1] ?? 0, blur: nums[2] ?? 0, spread: nums[3] ?? 0, inset: /inset/.test(p) };
    });
  };
  const pseudoMark = (el: Element) =>
    ['::before', '::after'].some((pe) => {
      const cs = getComputedStyle(el, pe);
      return cs.content !== 'none' && cs.content !== 'normal' && cs.display !== 'none' &&
        (parseFloat(cs.width) >= 2 || parseFloat(cs.height) >= 2 || cs.content.replace(/["']/g, '').length > 0);
    });
  w.__t32 = { rgba, over, backdrop, ratio, describe, visible, hex, shadows, pseudoMark };
}

type Helpers = {
  rgba: (c: string) => number[];
  over: (a: number[], b: number[]) => number[];
  backdrop: (el: Element | null, self: boolean) => number[];
  ratio: (a: number[], b: number[]) => number;
  describe: (el: Element) => string;
  visible: (el: Element) => boolean;
  hex: (c: number[]) => string;
  shadows: (v: string) => { color: string; x: number; y: number; blur: number; spread: number; inset: boolean }[];
  pseudoMark: (el: Element) => boolean;
};

export async function nonTextChecks(page: Page, exclude: string[]): Promise<{ checked: number; findings: Finding[] }> {
  await page.evaluate(installHelpers);
  return page.evaluate((excl) => {
    const h = (window as unknown as { __t32: Helpers }).__t32;
    const findings: Finding[] = [];
    let checked = 0;
    const sel =
      'input:not([type=hidden]):not([type=checkbox]):not([type=radio]):not([type=color]):not([type=range]), select, textarea, button, input[type=checkbox], input[type=radio]';
    for (const el of document.querySelectorAll(sel)) {
      if (!h.visible(el) || excl.some((x) => el.closest(x))) continue;
      const cs = getComputedStyle(el);
      if ((el as HTMLInputElement).type === 'checkbox' || (el as HTMLInputElement).type === 'radio') {
        if (cs.appearance !== 'none') continue; // native control, painted by the browser
      }
      const sides = ['Top', 'Right', 'Bottom', 'Left'] as const;
      const bordered = sides.filter(
        (s) => parseFloat(cs.getPropertyValue(`border-${s.toLowerCase()}-width`)) > 0 &&
          !['none', 'hidden'].includes(cs.getPropertyValue(`border-${s.toLowerCase()}-style`)) &&
          h.rgba(cs.getPropertyValue(`border-${s.toLowerCase()}-color`))[3]! > 0,
      );
      const isButton = el.tagName === 'BUTTON';
      // A button is boundary-checked only when it is outlined on all four sides; a one-sided
      // border on a button (a list row's divider) is decoration, not the control's boundary.
      if (isButton && bordered.length < 4) continue;
      checked++;
      const outside = h.backdrop(el, false);
      const fill = h.backdrop(el, true);
      const fillRatio = h.ratio(fill, outside);
      let borderRatio = 0;
      for (const s of bordered) {
        const c = h.over(h.rgba(cs.getPropertyValue(`border-${s.toLowerCase()}-color`)), outside);
        borderRatio = Math.max(borderRatio, h.ratio(c, outside));
      }
      if (borderRatio < 3 && fillRatio < 3) {
        findings.push({
          check: 'non-text boundary < 3:1',
          element: h.describe(el),
          detail: `border ${borderRatio}:1 (${bordered.length ? cs.borderTopColor : 'none'}), fill ${fillRatio}:1 (${h.hex(fill)}) against ${h.hex(outside)}`,
        });
      }
    }
    return { checked, findings };
  }, exclude);
}

export async function focusChecks(page: Page, exclude: string[]): Promise<{ checked: number; findings: Finding[] }> {
  await page.evaluate(installHelpers);
  const findings: Finding[] = [];
  const seen = new Set<string>();
  let checked = 0;
  // Start from the document so Tab walks the page (or the open dialog) from its first stop.
  await page.evaluate(() => (document.activeElement as HTMLElement | null)?.blur());
  for (let i = 0; i < FOCUS_STOPS; i++) {
    await page.keyboard.press('Tab');
    // A focus style may arrive through a transition (even a reduced-motion one of 0.01 ms);
    // read the end state, not the first frame.
    await page.evaluate(async () => {
      for (const a of document.getAnimations()) {
        try {
          a.finish();
        } catch {
          /* an infinite animation cannot finish; it is not a focus style */
        }
      }
      await new Promise<void>((r) => requestAnimationFrame(() => requestAnimationFrame(() => r())));
    });
    const r = await page.evaluate((excl) => {
      const h = (window as unknown as { __t32: Helpers }).__t32;
      const el = document.activeElement;
      if (!el || el === document.body || el.tagName === 'IFRAME' || excl.some((x) => el.closest(x))) return null;
      const cs = getComputedStyle(el);
      const outside = h.backdrop(el, false);
      const inside = h.backdrop(el, true);
      let best = 0;
      let how = 'none';
      if (cs.outlineStyle !== 'none' && parseFloat(cs.outlineWidth) >= 1) {
        const against = parseFloat(cs.outlineOffset) < 0 ? inside : outside;
        // `outline-style: auto` is Chrome's own ring, drawn in two tones (#101010 inside a white
        // edge), so the better of the two is what stands out against the background.
        const tones = cs.outlineStyle === 'auto' ? ['rgb(16, 16, 16)', 'rgb(255, 255, 255)'] : [cs.outlineColor];
        best = Math.max(...tones.map((t) => h.ratio(h.over(h.rgba(t), outside), against)));
        how = `outline ${cs.outlineWidth} ${cs.outlineStyle} ${cs.outlineColor}`;
      }
      const ring = h
        .shadows(cs.boxShadow)
        .filter((s) => !s.inset && s.spread > 0 && h.rgba(s.color)[3]! > 0)
        .sort((a, b) => b.spread - a.spread)[0];
      if (ring) {
        const c = h.over(h.rgba(ring.color), outside);
        const rr = h.ratio(c, outside);
        if (rr > best) {
          best = rr;
          how = `box-shadow ring ${ring.spread}px ${ring.color}`;
        }
      }
      return { element: h.describe(el), ratio: best, how, against: h.hex(outside) };
    }, exclude);
    if (r === null || seen.has(r.element)) continue;
    seen.add(r.element);
    checked++;
    if (r.ratio < 3) {
      findings.push({
        check: r.how === 'none' ? 'focus: no outline or ring' : 'focus indicator < 3:1',
        element: r.element,
        detail: `${r.how} → ${r.ratio}:1 against ${r.against}`,
      });
    }
  }
  return { checked, findings };
}

export async function signalChecks(
  page: Page,
  exclude: string[],
): Promise<{ checked: { unread: number; selected: number; links: number; tabs: number }; findings: Finding[] }> {
  await page.evaluate(installHelpers);
  return page.evaluate((excl) => {
    const h = (window as unknown as { __t32: Helpers }).__t32;
    const findings: Finding[] = [];
    const checked = { unread: 0, selected: 0, links: 0, tabs: 0 };
    const keep = (el: Element) => h.visible(el) && !excl.some((x) => el.closest(x));

    for (const el of [...document.querySelectorAll('.list__row--unread')].filter(keep)) {
      checked.unread++;
      const heavy = [el, ...el.querySelectorAll('.list__sender, .list__subject')].some(
        (e) => Number(getComputedStyle(e).fontWeight) >= 600,
      );
      const marker = h.pseudoMark(el) || [...el.querySelectorAll('[class*="unread"]')].some((e) => h.visible(e));
      if (!heavy && !marker) findings.push({ check: 'unread row: colour only', element: h.describe(el), detail: 'no weight >= 600, no marker' });
    }

    const selected = [
      ...document.querySelectorAll('.list__row--active, [role=listbox] [role=option][aria-selected=true]'),
    ].filter(keep);
    for (const el of selected) {
      checked.selected++;
      const cs = getComputedStyle(el);
      const bar = h.shadows(cs.boxShadow).some((s) => s.inset && (Math.abs(s.x) >= 2 || Math.abs(s.y) >= 2 || s.spread >= 2) && h.rgba(s.color)[3]! > 0);
      const outline = cs.outlineStyle !== 'none' && parseFloat(cs.outlineWidth) >= 1;
      const startBorder = parseFloat(cs.borderInlineStartWidth) >= 3 && cs.borderInlineStartStyle !== 'none';
      if (!bar && !outline && !startBorder && !h.pseudoMark(el)) {
        findings.push({ check: 'selected row: colour only', element: h.describe(el), detail: `box-shadow ${cs.boxShadow}; outline ${cs.outlineStyle}` });
      }
    }

    for (const el of [...document.querySelectorAll('a[href]')].filter(keep)) {
      let block: Element | null = el.parentElement;
      while (block && getComputedStyle(block).display.startsWith('inline')) block = block.parentElement;
      const own = (el.textContent ?? '').trim().length;
      const around = (block?.textContent ?? '').trim().length;
      if (around <= own + 5) continue; // a standalone link, not one inside running text
      checked.links++;
      const cs = getComputedStyle(el);
      if (!cs.textDecorationLine.includes('underline') && parseFloat(cs.borderBottomWidth) < 1) {
        findings.push({ check: 'in-text link not underlined', element: h.describe(el), detail: `text-decoration ${cs.textDecorationLine}` });
      }
    }

    for (const el of [...document.querySelectorAll('[role=tab][aria-selected=true]')].filter(keep)) {
      checked.tabs++;
      const cs = getComputedStyle(el);
      const bottom = parseFloat(cs.borderBottomWidth) >= 2 && cs.borderBottomStyle !== 'none' && h.rgba(cs.borderBottomColor)[3]! > 0;
      const inset = h.shadows(cs.boxShadow).some((s) => s.inset && (Math.abs(s.y) >= 2 || s.spread >= 2) && h.rgba(s.color)[3]! > 0);
      if (!bottom && !inset && !h.pseudoMark(el)) {
        findings.push({ check: 'active tab: no bar', element: h.describe(el), detail: `border-bottom ${cs.borderBottomWidth} ${cs.borderBottomStyle}; box-shadow ${cs.boxShadow}` });
      }
    }
    return { checked, findings };
  }, exclude);
}

/** All rendered checks on the current page; `exclude` lists subtrees to skip. */
export async function check(page: Page, exclude: string[]) {
  let axe = new AxeBuilder({ page }).withRules(['color-contrast']);
  for (const x of exclude) axe = axe.exclude(x);
  const axeResult = await axe.analyze();
  const findings: Finding[] = [];
  for (const v of axeResult.violations) {
    for (const n of v.nodes) {
      findings.push({
        check: `axe ${v.id}`,
        element: n.target.join(' '),
        detail: (n.any[0]?.message ?? n.failureSummary ?? '').replace(/\s+/g, ' ').slice(0, 200),
      });
    }
  }
  const nonText = await nonTextChecks(page, exclude);
  const signals = await signalChecks(page, exclude);
  // Focus last: Tab moves focus and may scroll.
  const focus = await focusChecks(page, exclude);
  findings.push(...nonText.findings, ...signals.findings, ...focus.findings);
  return {
    checked: {
      axeNodes: axeResult.passes.reduce((n, p) => n + p.nodes.length, 0) + axeResult.violations.reduce((n, v) => n + v.nodes.length, 0),
      nonText: nonText.checked,
      focus: focus.checked,
      ...signals.checked,
    },
    findings,
  };
}
