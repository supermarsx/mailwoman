// Roving-tabindex primitive for menus / toolbars / nav rails (plan §3 e3).
//
// Implements the WAI-ARIA roving-tabindex keyboard pattern: the composite
// (menu/toolbar/tablist/nav) is a SINGLE tab stop; exactly one child holds
// `tabindex="0"` and the rest `tabindex="-1"`, and arrow keys move the focused
// child (updating the roving stop). Home/End jump to the ends. This gives a
// keyboard user one Tab to enter the group and arrows to traverse it — the
// canonical shared helper the other executors' local menus migrate to later.
//
// Usage (Solid): give the container a ref and mark items with `data-roving-item`
// (or pass a custom `itemSelector`):
//
// ```tsx
// let nav!: HTMLElement;
// createRovingTabindex(() => nav, { orientation: 'vertical' });
// <nav ref={nav}>
//   <button data-roving-item>Domains</button>
//   <button data-roving-item>Users</button>
// </nav>
// ```

import { createEffect, onCleanup, type Accessor } from 'solid-js';
import { isVisible } from './focusable.ts';

export type RovingOrientation = 'horizontal' | 'vertical' | 'both';

export interface RovingTabindexOptions {
  /** Which arrow keys move focus. Default `horizontal` (Left/Right). */
  orientation?: RovingOrientation;
  /** Wrap from last→first and first→last. Default true. */
  loop?: boolean;
  /** Selector for the roving items. Default `[data-roving-item]`. */
  itemSelector?: string;
}

const isBrowser = (): boolean => typeof document !== 'undefined';

// Roving items intentionally carry `tabindex="-1"` (that is the pattern), so we
// enumerate by visibility + enabled state — NOT `isFocusable`, which rejects -1.
function itemsOf(container: HTMLElement, selector: string): HTMLElement[] {
  return Array.from(container.querySelectorAll<HTMLElement>(selector)).filter(
    (el) => !(el as HTMLButtonElement).disabled && el.getAttribute('aria-hidden') !== 'true' && isVisible(el),
  );
}

/** Apply the roving-tabindex keyboard pattern to `container`'s items. */
export function createRovingTabindex(
  container: Accessor<HTMLElement | undefined | null>,
  options: RovingTabindexOptions = {},
): void {
  if (!isBrowser()) return;
  const selector = options.itemSelector ?? '[data-roving-item]';
  const orientation = options.orientation ?? 'horizontal';
  const loop = options.loop !== false;

  createEffect(() => {
    const el = container();
    if (!el) return;

    // Seed the roving stop: the currently-checked/current item, else the first.
    const seed = (): void => {
      const items = itemsOf(el, selector);
      if (items.length === 0) return;
      const preferred =
        items.find((i) => i.getAttribute('aria-current') === 'true' || i.getAttribute('aria-selected') === 'true') ??
        items.find((i) => i.tabIndex === 0) ??
        items[0]!;
      for (const item of items) item.tabIndex = item === preferred ? 0 : -1;
    };
    seed();

    const focusAt = (items: HTMLElement[], index: number): void => {
      const target = items[index];
      if (!target) return;
      for (const item of items) item.tabIndex = item === target ? 0 : -1;
      target.focus();
    };

    const nextKey = orientation === 'vertical' ? 'ArrowDown' : orientation === 'horizontal' ? 'ArrowRight' : null;
    const prevKey = orientation === 'vertical' ? 'ArrowUp' : orientation === 'horizontal' ? 'ArrowLeft' : null;

    const isNext = (k: string): boolean =>
      k === nextKey || (orientation === 'both' && (k === 'ArrowRight' || k === 'ArrowDown'));
    const isPrev = (k: string): boolean =>
      k === prevKey || (orientation === 'both' && (k === 'ArrowLeft' || k === 'ArrowUp'));

    const onKeyDown = (e: KeyboardEvent): void => {
      const items = itemsOf(el, selector);
      if (items.length === 0) return;
      const current = document.activeElement as HTMLElement | null;
      const idx = current ? items.indexOf(current) : -1;
      if (idx === -1) return; // focus isn't on a roving item

      if (isNext(e.key)) {
        e.preventDefault();
        const n = idx + 1;
        focusAt(items, n >= items.length ? (loop ? 0 : items.length - 1) : n);
      } else if (isPrev(e.key)) {
        e.preventDefault();
        const p = idx - 1;
        focusAt(items, p < 0 ? (loop ? items.length - 1 : 0) : p);
      } else if (e.key === 'Home') {
        e.preventDefault();
        focusAt(items, 0);
      } else if (e.key === 'End') {
        e.preventDefault();
        focusAt(items, items.length - 1);
      }
    };

    // When an item is clicked/focused directly, make it the roving stop.
    const onFocusIn = (e: FocusEvent): void => {
      const target = e.target as HTMLElement | null;
      if (!target) return;
      const items = itemsOf(el, selector);
      if (!items.includes(target)) return;
      for (const item of items) item.tabIndex = item === target ? 0 : -1;
    };

    el.addEventListener('keydown', onKeyDown);
    el.addEventListener('focusin', onFocusIn);
    onCleanup(() => {
      el.removeEventListener('keydown', onKeyDown);
      el.removeEventListener('focusin', onFocusIn);
    });
  });
}
