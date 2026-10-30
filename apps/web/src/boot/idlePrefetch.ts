// Idle prefetch of the next lazy chunks (t99).
//
// Measured on a seeded stack: the first message opened waited for nine string
// catalogs (`privacy`, `reader`, `security`, …), the first Settings open for
// seven, Calendar for its 93 kB module and Contacts for its 60 kB one, each
// fetched only when the click asked for it. Once the mailbox has shown its list
// and the browser is idle, these are fetched in the background, a few at a
// time, so the first open finds them in memory. Nothing is executed beyond
// module evaluation (catalog registration, component definitions); no request
// to the API is made. Skipped when the user asked to save data.

import { preloadCatalogs } from '../i18n/catalog.ts';

/** Catalogs the first message open needs (reader, privacy banner, verdicts, receipts). */
export const READER_CATALOGS = ['reader', 'privacy', 'remote-images', 'security', 'spam-score', 'mail-dsn', 'mailing-list', 'nextcloud', 'receipts'] as const;
/** Catalogs the first Settings open needs. */
export const SETTINGS_CATALOGS = ['settings', 'appearance', 'labels', 'packages', 'mail-transfer', 'settings-policy', 'color-picker'] as const;
/** The app modules most opened after mail, with their catalogs. */
export const MODULE_CATALOGS = ['calendar', 'contacts'] as const;

type Step = () => Promise<unknown>;

/** The prefetch steps, in order of how soon each is likely needed. */
export function prefetchSteps(): Step[] {
  return [
    () => preloadCatalogs(READER_CATALOGS),
    () => preloadCatalogs(SETTINGS_CATALOGS),
    () => preloadCatalogs(MODULE_CATALOGS),
    () => import('../shell/mounts.tsx'),
    () => import('../modules/contacts/index.tsx'),
  ];
}

interface IdleHost {
  requestIdleCallback?: (cb: () => void, opts?: { timeout: number }) => number;
  setTimeout: (cb: () => void, ms: number) => unknown;
  navigator?: { connection?: { saveData?: boolean; effectiveType?: string } };
}

/** Whether the user or the network asks us not to fetch ahead. */
export function prefetchAllowed(host: IdleHost = globalThis as unknown as IdleHost): boolean {
  const connection = host.navigator?.connection;
  if (connection?.saveData === true) return false;
  return !(connection?.effectiveType === 'slow-2g' || connection?.effectiveType === '2g');
}

let started = false;

/**
 * Run the prefetch steps one after another, each in an idle period (or after a
 * short delay where `requestIdleCallback` is missing). Once per page; a failed
 * step (offline, a removed chunk after a deploy) is skipped, the next one runs.
 */
export function scheduleIdlePrefetch(steps: Step[] = prefetchSteps(), host: IdleHost = globalThis as unknown as IdleHost): boolean {
  if (started || !prefetchAllowed(host)) return false;
  started = true;
  const idle = (fn: () => void): void => {
    if (typeof host.requestIdleCallback === 'function') host.requestIdleCallback(fn, { timeout: 4000 });
    else host.setTimeout(fn, 1500);
  };
  const run = (index: number): void => {
    const step = steps[index];
    if (!step) return;
    idle(() => {
      void Promise.resolve().then(step).catch(() => undefined).finally(() => run(index + 1));
    });
  };
  run(0);
  return true;
}

/** Tests: allow another schedule. */
export function resetIdlePrefetch(): void {
  started = false;
}
