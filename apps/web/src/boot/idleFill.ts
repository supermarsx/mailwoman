// t52w: the idle fill. Once the mailbox is up and `web.prefetch.idleDelayMs`
// has passed (App.tsx -> scheduleIdlePrefetch, first step), the page downloads
// what the service worker's install set leaves out but a later offline start
// is likely to want:
//   1. the active locale's app translation pack. The worker cannot know the
//      locale when it installs; the page fetched the pack before the worker
//      controlled it, so this request goes through the worker (cache-first,
//      digest-checked, stored) and is normally an HTTP cache hit;
//   2. the Settings and PIM chunks with their stylesheets (manifest `idle`,
//      sw/offlineTiers.ts).
// Never the admin pack, the admin console, PDF.js or another locale's pack.
//
// Bounded: one file at a time, FILL_GAP_MS between files, each with the chunk
// retry (503/429 honour Retry-After, chunkRetry.ts), so the fill cannot become
// a burst the server refuses. It waits up to CONTROLLER_WAIT_MS for the worker
// to take control first, so the files land in its cache rather than only in
// the HTTP cache. Nothing is executed. The caller (scheduleIdlePrefetch) has
// already checked `web.prefetch.enabled`, Save-Data and 2G.

import { pageManifest } from './bootManifest.ts';
import { prefetchFile } from './chunkRetry.ts';

/** Pause between two files of the fill. */
export const FILL_GAP_MS = 250;
/** How long the fill waits for the service worker to control the page. */
export const CONTROLLER_WAIT_MS = 30_000;

export interface FillHost {
  document: Document;
  serviceWorker?: { controller: unknown; addEventListener(type: 'controllerchange', listener: () => void): void; removeEventListener(type: 'controllerchange', listener: () => void): void } | undefined;
  sleep: (ms: number) => Promise<void>;
  fetchFile: (url: string) => Promise<boolean>;
  native: boolean;
}

const defaultHost = (): FillHost => ({
  document,
  serviceWorker: typeof navigator !== 'undefined' && 'serviceWorker' in navigator ? navigator.serviceWorker : undefined,
  sleep: (ms) => new Promise((resolve) => { setTimeout(resolve, ms); }),
  fetchFile: prefetchFile,
  native: typeof (globalThis as { __TAURI_INTERNALS__?: unknown }).__TAURI_INTERNALS__ !== 'undefined',
});

/** The files of the fill, in order, as absolute URLs. */
export function fillList(doc: Document): string[] {
  const manifest = pageManifest(doc);
  if (manifest === null) return [];
  const lang = doc.documentElement.lang;
  const pack = Object.prototype.hasOwnProperty.call(manifest.packs, lang) ? manifest.packs[lang] : undefined;
  return [...(pack === undefined ? [] : [pack]), ...(manifest.idle ?? [])]
    .filter((href) => href.startsWith('./assets/'))
    .map((href) => new URL(href, doc.baseURI).href);
}

function controlled(host: FillHost): Promise<void> {
  const container = host.serviceWorker;
  if (container === undefined || container.controller) return Promise.resolve();
  return new Promise((resolve) => {
    const done = (): void => { container.removeEventListener('controllerchange', done); resolve(); };
    container.addEventListener('controllerchange', done);
    void host.sleep(CONTROLLER_WAIT_MS).then(done);
  });
}

/** Run the fill once; resolves when every file was tried. */
export async function idleFill(host: FillHost = defaultHost()): Promise<void> {
  // A native shell serves the bundle from disk and runs no worker.
  if (host.native) return;
  const files = fillList(host.document);
  if (files.length === 0) return;
  await controlled(host);
  for (const [i, url] of files.entries()) {
    if (i > 0) await host.sleep(FILL_GAP_MS);
    await host.fetchFile(url);
  }
}
