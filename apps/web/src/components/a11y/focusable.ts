// Shared focusable-element utilities (plan §3 e3 — the canonical a11y primitives).
//
// The single source of truth for "what can hold keyboard focus" used by the
// focus-trap and roving-tabindex helpers in this directory. Kept dependency-free
// and DOM-only so it is trivially unit-testable under jsdom.

/**
 * The selector for natively focusable / explicitly-focusable elements. Excludes
 * negative tabindex (programmatic-only) — `focusableWithin` filters the rest.
 */
const FOCUSABLE_SELECTOR = [
  'a[href]',
  'area[href]',
  'button:not([disabled])',
  'input:not([disabled])',
  'select:not([disabled])',
  'textarea:not([disabled])',
  'iframe',
  'audio[controls]',
  'video[controls]',
  '[contenteditable]:not([contenteditable="false"])',
  '[tabindex]',
].join(',');

/**
 * Is `el` currently rendered? Layout-independent (works under jsdom, which has no
 * box model): an element is treated as hidden only if it, or an ancestor, is
 * `hidden` / `display:none`, or it computes `visibility:hidden`. We deliberately
 * avoid `offsetParent`/`getClientRects` (both are meaningless without layout).
 */
export function isVisible(el: HTMLElement): boolean {
  const getStyle = typeof getComputedStyle === 'function' ? getComputedStyle : null;
  let node: HTMLElement | null = el;
  let depth = 0;
  while (node && depth < 1000) {
    if (node.hidden) return false;
    const style = getStyle?.(node);
    if (style) {
      if (style.display === 'none') return false;
      // `visibility` inherits, so checking the element itself is enough — but a
      // cheap ancestor sweep also catches an explicitly-hidden wrapper.
      if (node === el && style.visibility === 'hidden') return false;
    }
    node = node.parentElement;
    depth += 1;
  }
  return true;
}

/** Is `el` keyboard-focusable right now (visible, enabled, not tabindex="-1")? */
export function isFocusable(el: HTMLElement): boolean {
  if (el.getAttribute('tabindex') === '-1') return false;
  if ((el as HTMLButtonElement).disabled) return false;
  if (el.getAttribute('aria-hidden') === 'true') return false;
  return isVisible(el);
}

/**
 * Every keyboard-focusable descendant of `container`, in DOM (tab) order. The
 * container itself is not included.
 */
export function focusableWithin(container: HTMLElement): HTMLElement[] {
  const nodes = Array.from(container.querySelectorAll<HTMLElement>(FOCUSABLE_SELECTOR));
  return nodes.filter(isFocusable);
}

/** The first focusable descendant of `container`, or `null`. */
export function firstFocusable(container: HTMLElement): HTMLElement | null {
  return focusableWithin(container)[0] ?? null;
}
