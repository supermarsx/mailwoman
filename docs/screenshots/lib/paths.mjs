// Paths and small helpers shared by the screenshot harness.
import { createRequire } from 'node:module';
import { tmpdir } from 'node:os';
import { dirname, join, resolve } from 'node:path';
import { fileURLToPath, pathToFileURL } from 'node:url';
import net from 'node:net';

export const HERE = resolve(dirname(fileURLToPath(import.meta.url)), '..');
export const REPO = resolve(HERE, '..', '..');
export const WEB_APP = join(REPO, 'apps', 'web');
export const DEMO = join(HERE, 'demo');
export const MANIFESTS = join(HERE, 'manifest');
export const META = join(HERE, 'meta');
export const OUTPUT = join(REPO, 'docs', 'public', 'screenshots');
/** Scratch root for web builds, binary copies, run directories and capture slots. */
export const SCRATCH = resolve(process.env.MW_SHOTS_TMP ?? join(tmpdir(), 'mw-shots'));

/** `require` that resolves packages from apps/web (Playwright, Fluent). */
export const webRequire = createRequire(join(WEB_APP, 'package.json'));

/** Import an ESM-or-CJS package installed under apps/web. */
export async function importFromWeb(name) {
  return import(pathToFileURL(webRequire.resolve(name)).href);
}

/** A TCP port that was free a moment ago on 127.0.0.1. */
export function freePort() {
  return new Promise((ok, fail) => {
    const server = net.createServer();
    server.unref();
    server.once('error', fail);
    server.listen(0, '127.0.0.1', () => {
      const { port } = server.address();
      server.close(() => ok(port));
    });
  });
}

export const sleep = (ms) => new Promise((r) => setTimeout(r, ms));

/** Poll `check` until it returns truthy or `ms` passes. */
export async function waitFor(check, what, ms = 120_000) {
  const end = Date.now() + ms;
  let last;
  for (;;) {
    try {
      const value = await check();
      if (value) return value;
    } catch (error) {
      last = error;
    }
    if (Date.now() > end) throw new Error(`timed out waiting for ${what}${last ? `: ${last.message}` : ''}`);
    await sleep(400);
  }
}

export function log(...parts) {
  console.log(`[shots ${new Date().toISOString().slice(11, 19)}]`, ...parts);
}
