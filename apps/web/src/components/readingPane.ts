// Reading-pane position preference (W3): right | bottom | off.
//
// A module-level singleton (same shape as the reader's max-security store): the
// signal drives the toolbar control's pressed state, and the value is reflected
// onto `:root[data-reading-pane]` so the layout switch is pure CSS (see
// readerPane.css.ts). Persisted to localStorage — there is no per-user settings
// endpoint yet (mirrors the theme slice's localStorage note); swapping the
// load/save pair moves the backend without touching callers.

import { createSignal, type Accessor } from 'solid-js';

export type ReadingPane = 'right' | 'bottom' | 'off';

export const READING_PANE_OPTIONS: readonly ReadingPane[] = ['right', 'bottom', 'off'];
const STORAGE_KEY = 'mw.mail.readingPane';

function load(): ReadingPane {
  if (typeof localStorage === 'undefined') return 'right';
  try {
    const raw = localStorage.getItem(STORAGE_KEY);
    return READING_PANE_OPTIONS.includes(raw as ReadingPane) ? (raw as ReadingPane) : 'right';
  } catch {
    return 'right';
  }
}

/** Reflect the choice onto :root so the CSS layout override applies. */
function apply(mode: ReadingPane): void {
  if (typeof document === 'undefined') return;
  document.documentElement.setAttribute('data-reading-pane', mode);
}

const [pane, setPaneSig] = createSignal<ReadingPane>(load());
apply(pane());

/** Current reading-pane position (reactive). */
export const readingPane: Accessor<ReadingPane> = pane;

/** Set + persist + reflect the reading-pane position. */
export function setReadingPane(mode: ReadingPane): void {
  setPaneSig(mode);
  apply(mode);
  try {
    localStorage?.setItem(STORAGE_KEY, mode);
  } catch {
    /* private mode / quota — best-effort, the attribute is already applied */
  }
}
