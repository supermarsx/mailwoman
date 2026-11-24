// @vitest-environment node
import { describe, expect, it } from 'vitest';
import { APP_CHUNKS, IDLE_CHUNKS, offlineTiers, type TierChunk } from './offlineTiers.ts';

const chunk = (name: string, imports: string[] = [], css: string[] = [], isEntry = false): TierChunk => ({ fileName: `assets/${name}-1.js`, name, imports: imports.map((n) => `assets/${n}-1.js`), importedCss: css, isEntry });

const chunks: TierChunk[] = [
  chunk('index', ['core', 'vendor-core'], [], true),
  chunk('core', ['vendor-core'], ['assets/core-1.css']),
  chunk('vendor-core'),
  chunk('shared', ['core'], ['assets/shared-1.css']),
  chunk('mail', ['shared', 'core', 'index'], ['assets/mail-1.css']),
  chunk('mail-lazy', ['mail']),
  chunk('plugins', ['shared-lazy']),
  chunk('shared-lazy', ['admin-lazy']),
  chunk('admin-lazy'),
  chunk('editor', ['shared-lazy']),
  chunk('compose', ['editor', 'shared-lazy'], ['assets/compose-1.css']),
  chunk('login', ['core'], ['assets/login-1.css']),
  chunk('settings', ['shared-lazy', 'shared'], ['assets/settings-1.css']),
  chunk('pim', ['shared'], ['assets/pim-1.css']),
  chunk('admin', ['admin-lazy']),
  chunk('viewers', ['shared']),
];
const workers: Record<string, string> = {
  '/assets/core-1.js': 'new URL("networkWorker-9.js",import.meta.url);new URL("worker.entry-S.js",import.meta.url);new URL("worker.entry-C.js",import.meta.url);fetch(new URL("mw_sanitize_bg-1.wasm",import.meta.url))',
  '/assets/worker.entry-C.js': 'new URL("mw_crypto_bg-1.wasm",import.meta.url);new URL("mw_sanitize_bg-1.wasm",import.meta.url)',
  '/assets/settings-1.js': 'new URL("worker.entry-Z.js",import.meta.url)',
};
const files = [
  ...chunks.map((c) => `/${c.fileName}`),
  ...chunks.flatMap((c) => (c.importedCss ?? []).map((f) => `/${f}`)),
  '/assets/boot-1.js', '/assets/networkWorker-9.js', '/assets/worker.entry-S.js', '/assets/worker.entry-C.js', '/assets/worker.entry-Z.js',
  '/assets/mw_sanitize_bg-1.wasm', '/assets/mw_crypto_bg-1.wasm', '/assets/offline-background-1.js',
  '/assets/catalog.en.app-1.json', '/assets/jost-latin-1.woff2',
];
const read = (path: string): string => workers[path] ?? '';

describe('offline tiers (t52w)', () => {
  const tiers = offlineTiers(chunks, files, read);

  it('app: the entry closure, the mailbox, plugins, compose, editor and sign-in, their CSS, the boot script', () => {
    for (const name of ['index', 'core', 'vendor-core', 'shared', 'mail', 'mail-lazy', 'plugins', 'shared-lazy', 'admin-lazy', 'editor', 'compose', 'login']) {
      expect(tiers.app, name).toContain(`/assets/${name}-1.js`);
    }
    for (const css of ['core', 'shared', 'mail', 'compose', 'login']) expect(tiers.app).toContain(`/assets/${css}-1.css`);
    expect(tiers.app).toContain('/assets/boot-1.js');
  });

  it('app: the workers the app chunks start and the sanitizer wasm, never mw_crypto wasm', () => {
    expect(tiers.app).toEqual(expect.arrayContaining(['/assets/networkWorker-9.js', '/assets/worker.entry-S.js', '/assets/worker.entry-C.js', '/assets/mw_sanitize_bg-1.wasm']));
    expect(tiers.app).not.toContain('/assets/mw_crypto_bg-1.wasm');
    // Started only by Settings (the zero-access worker): first use.
    expect(tiers.app).not.toContain('/assets/worker.entry-Z.js');
  });

  it('app: no admin console, viewers, translation pack, font or the worker\'s own import', () => {
    for (const path of ['/assets/admin-1.js', '/assets/viewers-1.js', '/assets/catalog.en.app-1.json', '/assets/jost-latin-1.woff2', '/assets/offline-background-1.js', '/assets/settings-1.js', '/assets/pim-1.js']) {
      expect(tiers.app, path).not.toContain(path);
    }
  });

  it('idle: Settings and PIM with their CSS, minus what the app tier has', () => {
    expect(tiers.idle).toEqual(['/assets/pim-1.css', '/assets/pim-1.js', '/assets/settings-1.css', '/assets/settings-1.js']);
  });

  it('fails the build when a named chunk is missing', () => {
    for (const name of [...APP_CHUNKS, ...IDLE_CHUNKS]) {
      expect(() => offlineTiers(chunks.filter((c) => c.name !== name), files, read), name).toThrow(/no chunk named/);
    }
  });
});
