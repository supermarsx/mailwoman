// Background loading feedback in the corner toast (t111, user request: "loading
// messages toast should be a neat toast on the bottom right like the other
// neat toast").
//
// The message list used to draw its own "Loading messages…" box in the list's
// top corner while a folder reloaded under rows already on screen. That work is
// now one busy toast in the shared ToastRegion: the same box, tones, placement
// (bottom inline-end; top full width on a phone), lift above the bulk bar and
// polite live region as every other toast.
//
//   - Nothing is shown for work that ends within DELAY_MS: a fast reload never
//     flashes a toast.
//   - One toast id per controller, so a second load (or a switch from "Loading
//     messages…" to "Loading older messages…") replaces the text in place; it
//     never stacks a duplicate.
//   - Done: the toast closes. A failure the caller wants to report replaces it
//     in place (same id) with a short error toast.
//   - Closed by the user: it stays closed until that piece of work ends.
//
// Empty lists keep their skeleton (t43): the caller only starts this while
// content is already on screen.

import { toast as toastStore, type ToastOptions, type ToastTone } from './toast/toast.ts';

/** Work that finishes sooner than this shows no toast. */
export const BACKGROUND_TOAST_DELAY_MS = 400;

export interface BackgroundToastSink {
  show: (tone: ToastTone, message: string, opts?: ToastOptions) => string;
  dismiss: (id: string) => void;
}

export interface BackgroundLoadToast {
  /** Work `message` is running. The same message again is a no-op; another one replaces the text. */
  start(message: string): void;
  /** The work ended. `failure`: show it in place of the busy toast (shown or not). */
  finish(failure?: string): void;
  /** Drop everything without a message (unmount, scope change). */
  cancel(): void;
  /** Whether the busy toast is on screen. */
  showing(): boolean;
}

export function createBackgroundLoadToast(opts: {
  id: string;
  testId?: string;
  delayMs?: number;
  sink?: BackgroundToastSink;
}): BackgroundLoadToast {
  const sink = opts.sink ?? toastStore;
  const delay = opts.delayMs ?? BACKGROUND_TOAST_DELAY_MS;
  const testId = opts.testId ?? 'background-load-toast';
  let message: string | null = null;
  let shown = false;
  let closedByUser = false;
  let timer: ReturnType<typeof setTimeout> | undefined;
  /** Bumped on every close we cause, so our own dismiss is not read as the user's. */
  let generation = 0;

  function render(): void {
    if (message === null || closedByUser) return;
    const mine = generation;
    shown = true;
    sink.show('info', message, {
      id: opts.id,
      busy: true,
      duration: Infinity,
      testId,
      onClose: (reason) => {
        if (mine !== generation || reason === 'cleared') return;
        if (reason === 'dismiss') closedByUser = true;
        shown = false;
      },
    });
  }
  function stop(): void {
    clearTimeout(timer);
    timer = undefined;
    generation += 1;
    if (shown) sink.dismiss(opts.id);
    shown = false;
    message = null;
    closedByUser = false;
  }
  return {
    start(next) {
      if (next === message) return;
      const running = message !== null;
      message = next;
      // On screen: the new text replaces the old one in place.
      if (shown) { generation += 1; render(); return; }
      // Still inside the delay (or closed by the user): the running timer, or nothing, covers it.
      if (!running) timer = setTimeout(() => { timer = undefined; render(); }, delay);
    },
    finish(failure) {
      stop();
      if (failure !== undefined) sink.show('error', failure, { id: opts.id, testId });
    },
    cancel: stop,
    showing: () => shown,
  };
}
