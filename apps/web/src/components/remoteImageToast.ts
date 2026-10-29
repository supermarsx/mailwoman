// Remote-image loading feedback for the reader (t55): one in-place toast per
// message view, and the settled tracker count the remote-images banner shows.
//
// Why this exists: the old reader rendered a "N recently decoded tracking images
// blocked" line from a createResource that every mounted reader (also hidden
// tabs) refetched every 5 s and on every frame load. Each refetch put the
// resource in `refreshing`, the `state === 'ready'` guard hid the line, the
// answer showed it again — the line blinked, the body under it jumped, it read
// "0 …", and /api/rc/stats was hit for messages no longer on screen. Here the
// counters are read ONCE per displayed body (STATS_SETTLE_MS after the frame
// reports it displayed, debounced), only for the reader on screen, and the
// result goes into a signal that only ever rises above zero.
//
// The toast: when a body with proxied images is handed to the frame, `start`
// arms a 300 ms timer; a body displayed sooner never toasts. Otherwise one info
// toast "Loading images…" is shown, then replaced in place (same toast id) by
// "Images loaded" (2 s) or "k images couldn't load" with Retry. The failure
// count comes from the optional `imagesFailed` counter on /api/rc/stats (t56);
// without it no failure is claimed. There is no per-image progress: the frame
// is an opaque sandbox and polling the server per image is what we removed.

import { createEffect, createSignal, on, onCleanup, type Accessor } from 'solid-js';
import { decodedTrackerStats, type RemoteContentStats } from '../api/remote-content.ts';
import { t } from '../i18n/index.ts';
import { toast as toastStore, type ToastOptions, type ToastTone } from './toast/toast.ts';

/** A load displayed sooner than this shows no toast. */
export const TOAST_DELAY_MS = 300;
/** "Images loaded" stays this long. */
export const TOAST_DONE_MS = 2_000;
/** After the frame displayed a body, wait this long, then read the counters once. */
export const STATS_SETTLE_MS = 1_500;
/** Give up waiting for counters this long after start. */
export const LOAD_CAP_MS = 30_000;

export interface ToastSink {
  show: (tone: ToastTone, message: string, opts?: ToastOptions) => string;
  dismiss: (id: string) => void;
}

export interface ImageLoadToast {
  /** A body with `total` proxied images was handed to the frame. Same key again is a no-op. */
  start(key: string, total: number): void;
  /** The load is over. `failed` undefined = unknown (no failure claimed). */
  finish(key: string, failed: number | undefined): void;
  /** The frame displayed the body. Before the delay ran out: no loading toast at all. */
  displayed(key: string): void;
  /** Message switch / unmount / disabled: close whatever this controller shows. */
  cancel(): void;
  /** The key of the load in progress, or null. */
  running(): string | null;
}

export function createImageLoadToast(opts: { id: string; retry: () => void; sink?: ToastSink }): ImageLoadToast {
  const sink = opts.sink ?? toastStore;
  let key: string | null = null;
  let finished: string | null = null;
  let shown = false;
  let lastText = '';
  let timer: ReturnType<typeof setTimeout> | undefined;

  function render(): void {
    const text = t('remote-images-toast-loading');
    if (text === lastText) return;
    lastText = text;
    sink.show('info', text, { id: opts.id, duration: Infinity, testId: 'remote-images-toast' });
  }
  function close(): void {
    clearTimeout(timer); timer = undefined;
    if (shown || finished !== null) sink.dismiss(opts.id);
    shown = false; lastText = ''; key = null; finished = null;
  }
  return {
    start(next, count) {
      if (next === key || next === finished) return;
      close();
      if (count <= 0) return;
      key = next;
      timer = setTimeout(() => { timer = undefined; if (key !== next) return; shown = true; render(); }, TOAST_DELAY_MS);
    },
    finish(at, failed) {
      if (at !== key) return;
      clearTimeout(timer); timer = undefined;
      const wasShown = shown;
      key = null; shown = false; lastText = ''; finished = at;
      if (failed !== undefined && failed > 0) {
        // A failure is worth a toast even when it came back fast (negative cache).
        sink.show('warning', t('remote-images-toast-failed', { count: failed }), {
          id: opts.id, testId: 'remote-images-toast',
          action: { label: t('remote-images-toast-retry'), run: () => { finished = null; opts.retry(); } },
        });
      } else if (wasShown) {
        sink.show('success', t('remote-images-toast-done'), { id: opts.id, duration: TOAST_DONE_MS, testId: 'remote-images-toast' });
      }
    },
    displayed(at) {
      if (at !== key || shown) return;
      clearTimeout(timer); timer = undefined;
    },
    cancel: close,
    running: () => key,
  };
}

/** What the reader feeds the hook. */
export interface RemoteImageLoadSource {
  /** account:email of the open message; changes on message switch. */
  scope: string;
  emailId: string;
  /** Identity of the body handed to the frame (changes on reload/Retry). */
  load: string;
  /** Signed proxied images in that body; 0 = nothing loads through the server. */
  total: number;
  /** Whether /api/rc/stats means anything for this body (tracker removal on and
   *  images going through the server). */
  stats: boolean;
}

export interface SettledTrackers { count: number; limited: boolean }

/** Wires the one stats read, the tracker count and the toast for one reader.
 *  `source` must be null while the reader is not the one on screen: then nothing
 *  is fetched and nothing is toasted. */
export function createRemoteImageLoad(opts: {
  source: Accessor<RemoteImageLoadSource | null>;
  /** Bumps each time the frame reports the current document displayed. */
  displayed: Accessor<number>;
  /** `privacy.remoteImagesToast` is on. */
  toastEnabled: Accessor<boolean>;
  retry: () => void;
  fetchStats?: (emailId: string) => Promise<RemoteContentStats>;
  sink?: ToastSink;
  toastId?: string;
}): { trackers: Accessor<SettledTrackers | null> } {
  const fetchStats = opts.fetchStats ?? decodedTrackerStats;
  const toast = createImageLoadToast({ id: opts.toastId ?? 'remote-images', retry: opts.retry, ...(opts.sink ? { sink: opts.sink } : {}) });
  const [trackers, setTrackers] = createSignal<SettledTrackers | null>(null, { equals: (a, b) => a?.count === b?.count && a?.limited === b?.limited });
  let statsTimer: ReturnType<typeof setTimeout> | undefined;
  let capTimer: ReturnType<typeof setTimeout> | undefined;
  let generation = 0;
  /** Loads whose stats were already read: one request per load, ever. */
  const read = new Set<string>();

  function offer(next: SettledTrackers): void {
    // Never zero, never backwards within one message view.
    const best = trackers();
    if (next.count <= 0 || (best !== null && next.count <= best.count)) return;
    setTrackers(next);
  }
  function stopTimers(): void { clearTimeout(statsTimer); clearTimeout(capTimer); statsTimer = capTimer = undefined; }

  async function readStats(src: RemoteImageLoadSource, gen: number): Promise<void> {
    if (gen !== generation || read.has(src.load)) return;
    read.add(src.load);
    let stats: RemoteContentStats | null = null;
    try { stats = await fetchStats(src.emailId); } catch { stats = null; }
    if (gen !== generation) return;
    if (stats) offer({ count: stats.count, limited: stats.limited });
    toast.finish(src.load, stats?.imagesFailed);
  }

  createEffect(on(opts.source, (src, prev) => {
    if (src?.scope !== prev?.scope || src === null) {
      // Message switch, closed, or no longer on screen: drop everything.
      generation++; toast.cancel(); stopTimers(); read.clear(); setTrackers(null);
    }
    if (src === null) return;
    if (src.load === prev?.load && src.stats === prev.stats && src.total === prev.total) return;
    const gen = ++generation;
    stopTimers();
    if (src.total > 0 && opts.toastEnabled()) {
      toast.start(src.load, src.total);
      capTimer = setTimeout(() => { if (gen === generation) toast.finish(src.load, undefined); }, LOAD_CAP_MS);
    } else toast.cancel();
  }));

  createEffect(on(opts.displayed, (n) => {
    const src = opts.source();
    if (src === null || n === 0) return;
    toast.displayed(src.load);
    if (!src.stats) { toast.finish(src.load, undefined); return; }
    // Read the counters once the images have had time to settle; a second
    // display of the same load only pushes the read back (debounce).
    const gen = generation;
    clearTimeout(statsTimer);
    statsTimer = setTimeout(() => void readStats(src, gen), STATS_SETTLE_MS);
  }, { defer: true }));

  createEffect(on(opts.toastEnabled, (on) => { if (!on) toast.cancel(); }, { defer: true }));

  onCleanup(() => { generation++; stopTimers(); toast.cancel(); });
  return { trackers };
}
