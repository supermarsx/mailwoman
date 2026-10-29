// Message-body zoom for the reading pane (t59).
//
// The level is a per-user server preference (`reader.zoom`, a whole percent the
// registry bounds to 50-200) written through the settings API, so it carries the
// storage authority like every other personal preference and applies to every
// message, in every reader: the mail pane, message tabs and detached windows.
// An administrator can set the default or lock it like any `reader.*` key.
//
// Only the message body scales. The level reaches the body as one validated
// number in the frame document's own stylesheet (`readerAppearanceCss`), the
// same channel as the reading font size, so the sandbox and the app CSP stay as
// they are. The app UI never zooms.
//
// Shortcuts (Ctrl/Cmd + `=` / `-` / `0`, configurable in Keyboard & display) are
// actions in the app's one dispatcher. They answer only while focus is inside
// this reader; anywhere else they decline, so the browser keeps its own page
// zoom in the message list, the composer and the rest of the app.
//
// Ctrl/Cmd + wheel over the body: the body is a script-free sandboxed document,
// and its wheel and key events never reach the app. While Ctrl or Cmd is held
// in the app document the frame stops taking pointer events (a CSS class, not
// an inline style), so the wheel lands on the reader, which zooms the body and
// keeps the browser from zooming the page. With focus inside the body itself
// the browser keeps its own shortcuts: nothing outside the sandbox sees them.

import { createSignal, onCleanup, onMount } from 'solid-js';
import { loadSettings, settings, SettingsApiError, writeSetting } from '../api/settings.ts';
import { useActions } from '../keymap/index.ts';
import * as css from './readerZoom.css.ts';

export const ZOOM_SETTING = 'reader.zoom';
export const ZOOM_STEPS = [50, 67, 75, 90, 100, 110, 125, 150, 175, 200] as const;
export const DEFAULT_ZOOM = 100;
export const MIN_ZOOM = ZOOM_STEPS[0];
export const MAX_ZOOM = ZOOM_STEPS[ZOOM_STEPS.length - 1]!;

/** The step nearest to `value`; anything that is not a finite number is 100%. */
export function snapZoom(value: unknown): number {
  const n = typeof value === 'number' ? value : typeof value === 'string' && value.trim() !== '' ? Number(value) : NaN;
  if (!Number.isFinite(n)) return DEFAULT_ZOOM;
  let best: number = ZOOM_STEPS[0];
  for (const step of ZOOM_STEPS) if (Math.abs(step - n) < Math.abs(best - n)) best = step;
  return best;
}

/** The next step above (`direction` 1) or below (-1) `level`, clamped at the ends. */
export function stepZoom(level: number, direction: 1 | -1): number {
  const current = snapZoom(level);
  const index = ZOOM_STEPS.indexOf(current as (typeof ZOOM_STEPS)[number]);
  return ZOOM_STEPS[Math.max(0, Math.min(ZOOM_STEPS.length - 1, index + direction))]!;
}

// The level the user just chose, shown at once while its write is in flight
// (and kept for the session when the write cannot be stored: signed out,
// offline). `null` means "what the settings map says".
const [chosen, setChosen] = createSignal<number | null>(null);

/** The effective zoom level in percent (reactive). */
export function readerZoom(): number {
  return chosen() ?? snapZoom(settings()[ZOOM_SETTING]?.value ?? DEFAULT_ZOOM);
}

/** An administrator holds the level: the controls show it but cannot change it. */
export function readerZoomLocked(): boolean {
  return settings()[ZOOM_SETTING]?.locked === true;
}

const CHANNEL = 'mw-reader-zoom';
let channel: BroadcastChannel | null | undefined;
function peers(): BroadcastChannel | null {
  if (channel !== undefined) return channel;
  try {
    channel = typeof BroadcastChannel === 'function' ? new BroadcastChannel(CHANNEL) : null;
    // Another window of this app (a detached reader) stored a new level: read
    // it back from the server, so the account the window is signed in to decides.
    channel?.addEventListener('message', () => void loadSettings({ refresh: true }));
  } catch {
    channel = null;
  }
  return channel;
}

/** Set the level (snapped to a step) and store it as the user's preference. */
export function setReaderZoom(level: number): void {
  if (readerZoomLocked()) return;
  const next = snapZoom(level);
  if (next === readerZoom()) return;
  setChosen(next);
  peers();
  writeSetting(ZOOM_SETTING, next).then(
    () => {
      if (chosen() === next) setChosen(null);
      try { channel?.postMessage({ type: 'zoom' }); } catch { /* no peers */ }
    },
    (e: unknown) => {
      // A lock that arrived since the map loaded wins; any other failure keeps
      // the chosen level for this session only.
      if (e instanceof SettingsApiError && e.code === 'setting_locked' && chosen() === next) setChosen(null);
    },
  );
}

export const zoomIn = (): void => setReaderZoom(stepZoom(readerZoom(), 1));
export const zoomOut = (): void => setReaderZoom(stepZoom(readerZoom(), -1));
export const resetZoom = (): void => setReaderZoom(DEFAULT_ZOOM);

/** Forget the session-only level (sign-out, tests). */
export function clearReaderZoom(): void {
  setChosen(null);
}

/** Wheel distance (CSS px) for one zoom step; a mouse notch is ~100, a pinch less. */
const WHEEL_STEP = 50;

/** Wire the zoom shortcuts and Ctrl/Cmd + wheel to one reader. `root` is the
 *  reader element; `active` is false while the reader is a background tab. */
export function useReaderZoom(opts: { root: () => HTMLElement | undefined; active: () => boolean }): void {
  const inReader = (e: KeyboardEvent): boolean => {
    const root = opts.root();
    if (!root || !opts.active()) return false;
    const target = e.target instanceof Node ? e.target : null;
    const focused = root.ownerDocument.activeElement;
    return (target !== null && root.contains(target)) || (focused !== null && root.contains(focused));
  };
  const run = (action: () => void) => (e: KeyboardEvent): boolean => {
    if (!inReader(e)) return false;
    if (!e.repeat || action !== resetZoom) action();
    return true;
  };
  useActions({ zoomIn: run(zoomIn), zoomOut: run(zoomOut), zoomReset: run(resetZoom) });

  onMount(() => {
    const root = opts.root();
    if (!root) return;
    const doc = root.ownerDocument;
    const win = doc.defaultView ?? window;
    let held = false;
    const hold = (on: boolean): void => {
      const next = on && opts.active();
      if (next === held) return;
      held = next;
      root.classList.toggle(css.gesture, next);
    };
    const keys = (e: KeyboardEvent): void => {
      if (e.key === 'Control' || e.key === 'Meta') hold(e.type === 'keydown');
      else hold(e.ctrlKey || e.metaKey);
    };
    const release = (): void => hold(false);
    // A keyup can be missed (the key was released in another window): any
    // pointer movement in the app document says whether it is still held.
    const pointer = (e: PointerEvent | MouseEvent): void => { if (held && !e.ctrlKey && !e.metaKey) hold(false); };

    let distance = 0;
    let idle: ReturnType<typeof setTimeout> | undefined;
    const overBody = (e: WheelEvent): boolean => {
      for (const frame of Array.from(root.querySelectorAll<HTMLIFrameElement>('iframe.reader__frame'))) {
        const r = frame.getBoundingClientRect();
        if (e.clientX >= r.left && e.clientX <= r.right && e.clientY >= r.top && e.clientY <= r.bottom) return true;
      }
      return false;
    };
    const wheel = (e: WheelEvent): void => {
      if (!(e.ctrlKey || e.metaKey) || !opts.active() || !overBody(e)) return;
      e.preventDefault();
      const scale = e.deltaMode === 1 ? 16 : e.deltaMode === 2 ? 400 : 1;
      distance += e.deltaY * scale;
      clearTimeout(idle);
      idle = setTimeout(() => { distance = 0; }, 400);
      if (Math.abs(distance) < WHEEL_STEP) return;
      if (distance < 0) zoomIn(); else zoomOut();
      distance = 0;
    };

    doc.addEventListener('keydown', keys, true);
    doc.addEventListener('keyup', keys, true);
    doc.addEventListener('pointermove', pointer, true);
    win.addEventListener('blur', release);
    doc.addEventListener('visibilitychange', release);
    root.addEventListener('wheel', wheel, { passive: false });
    onCleanup(() => {
      clearTimeout(idle);
      hold(false);
      doc.removeEventListener('keydown', keys, true);
      doc.removeEventListener('keyup', keys, true);
      doc.removeEventListener('pointermove', pointer, true);
      win.removeEventListener('blur', release);
      doc.removeEventListener('visibilitychange', release);
      root.removeEventListener('wheel', wheel);
    });
  });
}
