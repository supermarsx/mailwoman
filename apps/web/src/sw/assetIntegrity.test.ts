// @vitest-environment node
//
// t63: the build step that gives the service worker its manifest and the shell
// its Subresource Integrity (vite.config.ts `assetIntegrity`). Run against a
// small dist/ fixture, so a change in Vite's output shape that the regex no
// longer matches fails here, not silently in production.

import { createHash } from 'node:crypto';
import { mkdtempSync, mkdirSync, readFileSync, rmSync, writeFileSync } from 'node:fs';
import { tmpdir } from 'node:os';
import { join } from 'node:path';
import { afterEach, describe, expect, it } from 'vitest';
import { assetIntegrity } from '../../vite.config.ts';

const dirs: string[] = [];
afterEach(() => { for (const d of dirs.splice(0)) rmSync(d, { recursive: true, force: true }); });

function fixture(html: string): string {
  const root = mkdtempSync(join(tmpdir(), 'mw-integrity-'));
  dirs.push(root);
  const out = join(root, 'dist');
  mkdirSync(join(out, 'assets', 'analytics'), { recursive: true });
  writeFileSync(join(out, 'assets', 'index-a1.js'), 'export{}');
  writeFileSync(join(out, 'assets', 'boot-b2.js'), 'void 0');
  writeFileSync(join(out, 'assets', 'index-c3.css'), 'a{src:url(/fonts/a.woff2)}');
  writeFileSync(join(out, 'assets', 'analytics', 'x.js'), 'ignored');
  writeFileSync(join(out, 'sw.js'), "const ASSET_DIGESTS = /* @mw-asset-digests */ null;\n");
  writeFileSync(join(out, 'index.html'), html);
  return out;
}

function run(out: string): void {
  const plugin = assetIntegrity() as unknown as { configResolved: (c: unknown) => void; closeBundle: () => void };
  plugin.configResolved({ command: 'build', root: join(out, '..'), build: { outDir: 'dist' } });
  plugin.closeBundle();
}

const sha256 = (s: string | Buffer): string => createHash('sha256').update(s).digest('hex');
const sri = (s: string): string => `sha384-${createHash('sha384').update(s).digest('base64')}`;

describe('mw-asset-integrity', () => {
  it('writes the manifest into sw.js and SRI into the entry and boot scripts', () => {
    const out = fixture('<!doctype html><html><head><script src="./assets/boot-b2.js"></script><script type="module" crossorigin src="./assets/index-a1.js"></script><link rel="stylesheet" crossorigin href="./assets/index-c3.css"></head><body></body></html>');
    run(out);
    const html = readFileSync(join(out, 'index.html'), 'utf8');
    expect(html).toContain(`<script src="./assets/boot-b2.js" integrity="${sri('void 0')}">`);
    expect(html).toContain(`<script type="module" crossorigin src="./assets/index-a1.js" integrity="${sri('export{}')}">`);
    expect(html).toContain('<link rel="stylesheet" crossorigin href="./assets/index-c3.css">');
    const sw = readFileSync(join(out, 'sw.js'), 'utf8');
    const manifest = JSON.parse(/const ASSET_DIGESTS = (\{.*\});/.exec(sw)![1]!) as Record<string, string>;
    expect(manifest['/assets/index-a1.js']).toBe(sha256('export{}'));
    expect(manifest['/assets/index-c3.css']).toBe(sha256('a{src:url(/fonts/a.woff2)}'));
    expect(manifest['/index.html']).toBe(sha256(html));
    expect(manifest['/assets/analytics/x.js']).toBeUndefined();
  });
  it('fails the build when a script tag escapes SRI', () => {
    // A shape the SRI rewrite does not match (single quotes) must not pass silently.
    const out = fixture("<!doctype html><head><script type=module src='./assets/index-a1.js'></script></head>");
    expect(() => run(out)).toThrow(/without SRI/);
  });
  it('fails the build when sw.js lost its placeholder', () => {
    const out = fixture('<!doctype html><head></head>');
    writeFileSync(join(out, 'sw.js'), 'const ASSET_DIGESTS = null;');
    expect(() => run(out)).toThrow(/placeholder/);
  });
  it('gives a modulepreload link SRI too, if Vite emits one', () => {
    const out = fixture('<!doctype html><head><link rel="modulepreload" crossorigin href="./assets/index-a1.js"><script type="module" crossorigin src="./assets/index-a1.js"></script></head>');
    run(out);
    const html = readFileSync(join(out, 'index.html'), 'utf8');
    expect(html).toContain(`<link rel="modulepreload" crossorigin href="./assets/index-a1.js" integrity="${sri('export{}')}">`);
  });
});
