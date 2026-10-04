// The narrow-viewport (phone-width) switch for the mailbox shell.
//
// The LAYOUT is CSS: `styles/app.css` turns the shell into a single-pane stack
// under `@media (max-width: 760px)`, and `readerPane.css.ts` guards the desktop
// reading-pane overrides to the complementary `min-width: 761px`. This module is
// the JS-observable companion to that breakpoint, for the things CSS cannot do:
// moving focus into the reader when it covers the list, making the covered list
// inert, and rendering the top bar only where the folder list is a drawer.
//
// jsdom-safe: without `matchMedia` the viewport reads as wide and never changes.

import { createSignal, onCleanup, type Accessor } from 'solid-js';

/** Must stay in step with the `max-width` in styles/app.css. */
export const NARROW_QUERY = '(max-width: 760px)';

/**
 * A live "is the viewport at phone width" accessor. Call inside a component (or
 * another reactive owner): the media-query listener is removed on cleanup.
 */
export function createNarrowViewport(): Accessor<boolean> {
  const mql = typeof matchMedia === 'function' ? matchMedia(NARROW_QUERY) : null;
  const [narrow, setNarrow] = createSignal(mql?.matches === true);
  if (mql !== null && typeof mql.addEventListener === 'function') {
    const onChange = (e: MediaQueryListEvent): void => {
      setNarrow(e.matches);
    };
    mql.addEventListener('change', onChange);
    onCleanup(() => mql.removeEventListener('change', onChange));
  }
  return narrow;
}

/**
 * Focus the list row to return to after the reader closes: the row that was
 * marked current if it is still mounted, otherwise the list's roving-focus row.
 * Returns whether a row took focus.
 */
export function focusListRow(previous: HTMLElement | null): boolean {
  const target =
    previous !== null && previous.isConnected
      ? previous
      : document.querySelector<HTMLElement>('.list__row[tabindex="0"]');
  if (target === null) return false;
  target.focus();
  return document.activeElement === target;
}
