// t52w: which build files the service worker installs (`web.offline.precacheMode`)
// and which the page fetches while idle after the mailbox is up.
//
// Pure: vite.config.ts `assetIntegrity()` calls it with the chunk graph of the
// build and the built files, and writes the result into dist/sw.js (APP_ASSETS)
// and into the boot manifest (`idle`). Tested in offlineTiers.test.ts.
//
// Tiers, by chunk (scripts/chunkPlan.ts names):
//   app   what an offline start into the mailbox needs: the entry with its
//         static closure (vendor-core, core), the mailbox and its panes (mail,
//         mail-lazy), the UI-plugin tier the mailbox mounts (plugins), compose
//         with the editor, and the sign-in screen; their stylesheets; the
//         workers these chunks start (network, calendar, sanitizer, crypto, the
//         pbkdf2 helper) and the sanitizer's wasm, needed to show any HTML
//         message. Not mw_crypto's wasm (6.6 MB): encrypted mail is opened from
//         the cache once it was loaded online.
//   idle  Settings and the PIM modules (settings, pim), minus the app tier.
//   rest  everything else, cached on first use by the worker's cache-first
//         rule: admin, viewers, dialogs, brand marks, fonts, other locales'
//         packs, PDF.js.
// The active locale's app pack is not in either list: the worker does not know
// the locale at install time. The page fetches it through the worker first in
// the idle fill (boot/idleFill.ts), and every later visit's own request for it
// is stored by the cache-first rule.

/** Chunks of the app tier besides the entry's closure. */
export const APP_CHUNKS: readonly string[] = ['mail', 'mail-lazy', 'plugins', 'compose', 'editor', 'login'];
/** Chunks the idle fill downloads. */
export const IDLE_CHUNKS: readonly string[] = ['settings', 'pim'];
/** Built files never installed by the app tier, even when an app chunk names them. */
const NOT_INSTALLED = /^\/assets\/mw_crypto_bg-[^/]+\.wasm$/;

export interface TierChunk {
  fileName: string;
  name?: string | undefined;
  isEntry?: boolean;
  imports?: readonly string[];
  importedCss?: readonly string[];
}

/** Static closure (scripts and their stylesheets) of the named chunks, as `/assets/...` paths. */
function closure(chunks: readonly TierChunk[], roots: readonly TierChunk[]): Set<string> {
  const byFile = new Map(chunks.map((c) => [c.fileName, c] as const));
  const out = new Set<string>();
  const stack = roots.map((r) => r.fileName);
  while (stack.length > 0) {
    const file = stack.pop()!;
    if (out.has(`/${file}`)) continue;
    const chunk = byFile.get(file);
    if (!chunk) continue;
    out.add(`/${file}`);
    for (const sheet of chunk.importedCss ?? []) out.add(`/${sheet}`);
    stack.push(...(chunk.imports ?? []));
  }
  return out;
}

/**
 * The two lists. `files` are the build's `/assets/...` paths (the digest
 * manifest's keys); `read` returns a built script's text, to find the worker
 * and wasm files a chunk starts by name (`new URL("x-<hash>.js", import.meta.url)`).
 */
export function offlineTiers(chunks: readonly TierChunk[], files: readonly string[], read: (path: string) => string): { app: string[]; idle: string[] } {
  const named = (names: readonly string[]): TierChunk[] => {
    const found = chunks.filter((c) => c.name !== undefined && names.includes(c.name) && c.isEntry !== true);
    const missing = names.filter((n) => !found.some((c) => c.name === n));
    if (missing.length > 0) throw new Error(`offline tiers: no chunk named ${missing.join(', ')} in the build`);
    return found;
  };
  const entry = chunks.filter((c) => c.isEntry === true);
  const app = closure(chunks, [...entry, ...named(APP_CHUNKS)]);
  // The boot script is an asset of its own (vite.config.ts bootScript()).
  for (const file of files) if (/^\/assets\/boot-[^/]+\.js$/.test(file)) app.add(file);
  // Workers and wasm the app chunks (and those workers) start by file name.
  const chunkFiles = new Set(chunks.map((c) => `/${c.fileName}`));
  const assets = files.filter((f) => /^\/assets\/[^/]+\.(?:m?js|wasm)$/.test(f) && !chunkFiles.has(f));
  const queue = [...app].filter((f) => /\.m?js$/.test(f));
  while (queue.length > 0) {
    const source = read(queue.pop()!);
    for (const file of assets) {
      if (app.has(file) || NOT_INSTALLED.test(file)) continue;
      if (!source.includes(file.slice('/assets/'.length))) continue;
      app.add(file);
      if (/\.m?js$/.test(file)) queue.push(file);
    }
  }
  // The offline-background bundle is the worker's own imported script, and the
  // analytics loader is per consent: neither belongs in Cache Storage.
  for (const file of [...app]) if (/^\/assets\/(?:offline-background|analytics\/)/.test(file)) app.delete(file);
  const idle = [...closure(chunks, named(IDLE_CHUNKS))].filter((f) => !app.has(f));
  const present = new Set(files);
  const missing = [...app, ...idle].filter((f) => !present.has(f));
  if (missing.length > 0) throw new Error(`offline tiers: not in the build: ${missing.join(', ')}`);
  return { app: [...app].sort(), idle: idle.sort() };
}
