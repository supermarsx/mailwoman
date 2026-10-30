// Recover a tab that is running an older build than the server now serves (t102).
//
// A tab opened before a deploy keeps the old entry script, and its lazy routes
// import chunks by that build's content-hashed names. The server keeps the last
// builds' hashed files for a grace period (crates/mw-server/src/static_resilience.rs),
// so most such imports still work. When one does fail, for example after the
// grace period or across a server without a data directory, this module checks
// whether the server now has a different build. If it does, the tab shows a short
// notice and reloads once to pick up the new build.
//
// Guards:
//   * Reload only when the server's shell names a different entry script than
//     the one this tab runs, so a chunk that is missing from the current build,
//     or a dropped request, never causes a reload.
//   * At most one automatic reload per server build, and none within
//     RELOAD_WINDOW_MS of the last one (sessionStorage, per tab), so a server that
//     keeps answering 404 cannot put the tab in a reload loop.
//   * No automatic reload while an editor has unsaved changes (a registered
//     navigation guard); the error card's "Reload application" stays available.

import { withBase } from '../api/basePath.ts';
import { toast, toastRegionMounted } from '../components/toast/toast.ts';
import { hasNavigationGuards } from '../shell/navigationGuard.ts';
import { t } from '../i18n';
import * as css from './bundleRecovery.css.ts';

const STORAGE_KEY = 'mw.bundleReload';
/** No second automatic reload within this long of the previous one. */
export const RELOAD_WINDOW_MS = 5 * 60 * 1000;
/** How long the notice is on screen before the reload. */
export const NOTICE_MS = 1500;

/** Browser messages for a module or chunk that could not be downloaded. */
export function isChunkLoadError(error: unknown): boolean {
  const message = error instanceof Error ? error.message : typeof error === 'string' ? error : '';
  return /dynamically imported module|loading chunk|loading css chunk|importing a module script|unable to preload css|module script failed/i.test(message);
}

/** The file name of the entry script named by a shell document (`index-AbC123.js`). */
export function entryName(html: string): string | null {
  const tags = html.match(/<script\b[^>]*\btype="module"[^>]*>/g) ?? [];
  for (const tag of tags) {
    const src = /\bsrc="([^"]+)"/.exec(tag)?.[1];
    if (src !== undefined && /(?:^|\/)assets\/[^/]+\.js$/.test(src)) return src.slice(src.lastIndexOf('/') + 1);
  }
  return null;
}

/** The entry script this tab is running. */
function runningEntry(): string | null {
  if (typeof document === 'undefined') return null;
  return entryName(document.documentElement.outerHTML);
}

/** The entry script the server's shell names now; `null` when it cannot tell. */
async function serverEntry(): Promise<string | null> {
  try {
    const response = await fetch(withBase('/index.html'), {
      cache: 'no-store',
      credentials: 'same-origin',
      headers: { accept: 'text/html' },
    });
    if (!response.ok) return null;
    return entryName(await response.text());
  } catch {
    return null;
  }
}

interface ReloadRecord { at: number; entry: string }

function lastReload(): ReloadRecord | null {
  try {
    const raw = sessionStorage.getItem(STORAGE_KEY);
    if (raw === null) return null;
    const value = JSON.parse(raw) as Partial<ReloadRecord>;
    return typeof value.at === 'number' && typeof value.entry === 'string' ? { at: value.at, entry: value.entry } : null;
  } catch {
    return null;
  }
}

function remember(record: ReloadRecord): boolean {
  try {
    sessionStorage.setItem(STORAGE_KEY, JSON.stringify(record));
    return true;
  } catch {
    // Without a record the loop guard cannot hold, so do not reload automatically.
    return false;
  }
}

export interface RecoveryDeps {
  running: () => string | null;
  server: () => Promise<string | null>;
  reload: () => void;
  now: () => number;
  notify: (message: string) => void;
  wait: (ms: number) => Promise<void>;
}

/** The notice: a toast when a region is on screen, else a standalone status line. */
export function showReloadNotice(message: string): void {
  if (toastRegionMounted()) {
    toast.info(message, { id: 'bundle-reload', duration: Infinity, testId: 'bundle-reload-notice' });
    return;
  }
  // The failure may have taken down the surface that mounts the toast region.
  if (typeof document === 'undefined') return;
  const el = document.createElement('div');
  el.className = css.notice;
  el.setAttribute('role', 'status');
  el.dataset['testid'] = 'bundle-reload-notice';
  el.textContent = message;
  document.body.append(el);
}

const defaults: RecoveryDeps = {
  running: runningEntry,
  server: serverEntry,
  reload: () => { window.location.reload(); },
  now: () => Date.now(),
  notify: showReloadNotice,
  wait: (ms) => new Promise((resolve) => { setTimeout(resolve, ms); }),
};

let inFlight: Promise<boolean> | null = null;

/**
 * For a chunk-load failure: reload into the server's newer build, once.
 * Resolves `true` when a reload was started, `false` when the caller should show
 * its own error (not a chunk failure, the same build, unsaved work, or the guard).
 */
export function recoverFromChunkError(error: unknown, deps: Partial<RecoveryDeps> = {}): Promise<boolean> {
  if (!isChunkLoadError(error)) return Promise.resolve(false);
  if (inFlight !== null) return inFlight;
  const d = { ...defaults, ...deps };
  inFlight = (async () => {
    if (hasNavigationGuards()) return false;
    const running = d.running();
    const server = await d.server();
    if (running === null || server === null || server === running) return false;
    const last = lastReload();
    if (last !== null && (last.entry === server || d.now() - last.at < RELOAD_WINDOW_MS)) return false;
    if (!remember({ at: d.now(), entry: server })) return false;
    d.notify(t('common-new-version-reloading'));
    await d.wait(NOTICE_MS);
    d.reload();
    return true;
  })().finally(() => { inFlight = null; });
  return inFlight;
}

/**
 * Catch chunk failures that no route boundary sees: Vite's preload helper
 * (`vite:preloadError`) and an unhandled rejected `import()`.
 */
export function installBundleRecovery(target: Window = window): () => void {
  const onPreload = (event: Event): void => {
    const payload = (event as Event & { payload?: unknown }).payload;
    void recoverFromChunkError(payload ?? new Error('Unable to preload CSS'));
  };
  const onRejection = (event: PromiseRejectionEvent): void => {
    if (isChunkLoadError(event.reason)) void recoverFromChunkError(event.reason);
  };
  target.addEventListener('vite:preloadError', onPreload);
  target.addEventListener('unhandledrejection', onRejection);
  return () => {
    target.removeEventListener('vite:preloadError', onPreload);
    target.removeEventListener('unhandledrejection', onRejection);
  };
}
