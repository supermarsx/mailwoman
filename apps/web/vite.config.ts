import {assetCompression} from './scripts/assetCompression.ts';
import {buildTimings} from './scripts/buildTimings.ts';
import {bundleGraph} from './scripts/bundleGraph.ts';
import {catalogAssets} from './scripts/catalogAssets.ts';
import {isAppCatalogPack} from './scripts/catalogPacks.ts';
import {chunkPlan,touchPlugin} from './scripts/chunkPlan.ts';
import {build as bundleWorker} from 'esbuild';
import { defineConfig } from 'vitest/config';
import {cachedSolid} from './scripts/cachedSolid.ts';
import { vanillaExtractPlugin } from '@vanilla-extract/vite-plugin';
import wasm from 'vite-plugin-wasm';
import { createHash } from 'node:crypto';
import { existsSync, readdirSync, readFileSync, writeFileSync } from 'node:fs';
import { join, relative, resolve, sep } from 'node:path';
import { fileURLToPath } from 'node:url';
import { transformWithEsbuild, type Plugin } from 'vite';
import { bundledPackages } from './scripts/about-bundled-packages.mjs';
import { swTrustedImport } from './scripts/swTrustedImport.ts';

const buildCacheStats={hits:0,misses:0};
const webRoot = fileURLToPath(new URL('.', import.meta.url));
const plan = chunkPlan(webRoot);
const UPSTREAM = 'http://localhost:8080';
// The web client's version, sent as `browser/<version>` in `X-Mailwoman-Client`.
const VERSION = (JSON.parse(readFileSync(new URL('./package.json', import.meta.url), 'utf8')) as { version: string }).version;

/**
 * The pre-paint boot script (t52, `src/boot/boot.js`; see `src/shell/bootSplash.ts`).
 * It must run before the first paint, so it is a CLASSIC script at the top of
 * `<head>`, which Vite does not bundle. This emits it as a hashed asset under
 * `assets/` instead of `public/`: `script-src 'self'` covers it, the server gives
 * it the immutable lifetime of the other hashed files, and the service worker
 * caches it cache-first with them, so an offline boot still has it.
 */
function bootScript(): Plugin {
  const source = (): string => readFileSync(new URL('./src/boot/boot.js', import.meta.url), 'utf8');
  let build = false;
  return {
    name: 'mw-boot-script',
    configResolved(config) { build = config.command === 'build'; },
    async buildStart() {
      if (!build) return;
      // It blocks the parser while it loads: ship it minified, comments and all stripped.
      const { code } = await transformWithEsbuild(source(), 'boot.js', { minify: true, target: 'es2015', legalComments: 'none' });
      this.emitFile({ type: 'asset', name: 'boot.js', source: code });
    },
    transformIndexHtml: {
      order: 'post',
      handler(_html, ctx) {
        let src = '/src/boot/boot.js';
        if (ctx.bundle !== undefined) {
          const asset = Object.values(ctx.bundle).find((o) => o.type === 'asset' && (o.names.includes('boot.js') || o.name === 'boot.js'));
          if (asset === undefined) throw new Error('mw-boot-script: boot.js was not emitted');
          src = `./${asset.fileName}`;
        }
        return [{ tag: 'script', attrs: { src }, injectTo: 'head-prepend' }];
      },
    },
  };
}

/**
 * t52w: the build's preload helper is `src/boot/chunkRetry.ts` instead of Vite's
 * own: every file of a dynamic import is fetched (and retried on 503 / network
 * error) before the module loader asks for it, because a failed module fetch
 * stays failed for the life of the page. Vite resolves the helper id itself;
 * this plugin only answers `load` for it first.
 */
const PRELOAD_HELPER_ID = '\0vite/preload-helper.js';
function chunkRetry(): Plugin {
  const helper = fileURLToPath(new URL('./src/boot/chunkRetry.ts', import.meta.url)).split(sep).join('/');
  return {
    name: 'mw-chunk-retry',
    enforce: 'pre',
    apply: 'build',
    load(id) {
      if (id !== PRELOAD_HELPER_ID) return null;
      return { code: `export { vitePreload as __vitePreload } from ${JSON.stringify(helper)};`, moduleSideEffects: false };
    },
  };
}

/**
 * t63 R1: integrity for what the service worker and the page load.
 *
 * Cache Storage is writable by any script of this origin, so the worker serves
 * a cached /assets/ file only when its SHA-256 matches the manifest written here
 * into dist/sw.js (the worker file itself is never stored in Cache Storage).
 * The entry and boot scripts in dist/index.html also get Subresource Integrity,
 * so even a page the worker does not control refuses a substituted entry. The
 * stylesheet does not: under a deploy prefix the server rewrites its font URLs,
 * which would change its bytes.
 */
let offlineBackgroundFile='';
/** Bundle the closed-tab replay worker into a classic, self-contained script.
 * The importing SW is never cache-served and pins this content-addressed file. */
function offlineBackground():Plugin {

 return {name:'mw-offline-background',apply:'build',async buildStart(){
  const result=await bundleWorker({entryPoints:[resolve('src/sw/background.ts')],bundle:true,write:false,format:'iife',platform:'browser',target:'es2022',minify:true,define:{__MW_VERSION__:JSON.stringify(VERSION),'process.env.NODE_ENV':'"production"'}});
  const code=result.outputFiles[0]!.text;
  offlineBackgroundFile=`assets/offline-background-${createHash('sha256').update(code).digest('hex').slice(0,16)}.js`;
  this.emitFile({type:'asset',fileName:offlineBackgroundFile,source:code});
 }};
}

export function assetIntegrity(): Plugin {
  let outDir = '';
  let build = false;
  let written = false;
  const walk = (dir: string): string[] => readdirSync(dir, { withFileTypes: true }).flatMap((entry) => {
    const full = join(dir, entry.name);
    return entry.isDirectory() ? walk(full) : [full];
  });
  return {
    name: 'mw-asset-integrity',
    apply: 'build',
    configResolved(config) { build = config.command === 'build'; outDir = resolve(config.root, config.build.outDir); },
    writeBundle() { written = true; },
    closeBundle() {
      // Rollup also calls closeBundle when the build FAILED, before anything was
      // written; throwing here would mask the real error with an ENOENT.
      if (!build || !written) return;
      const assets = join(outDir, 'assets');
      if (!existsSync(assets)) return;
      const digests: Record<string, string> = {};
      for (const file of walk(assets)) {
        const rel = `/assets/${relative(assets, file).split(sep).join('/')}`;
        if (rel.startsWith('/assets/analytics/')) continue;
        // Hidden source maps (MW_SOURCEMAP=hidden) are for the server's stack
        // symbolicator only: never served, never precached.
        if (rel.endsWith('.map') || rel.endsWith('.gz') || rel.endsWith('.br')) continue;
        digests[rel] = createHash('sha256').update(readFileSync(file)).digest('hex');
      }
      const pdf = join(outDir,'pdfjs');
      if(existsSync(pdf))for(const file of walk(pdf)){
        const rel=`/pdfjs/${relative(pdf,file).split(sep).join('/')}`;
        if(rel.endsWith('.gz')||rel.endsWith('.br'))continue;
        if(rel.endsWith('.html')&&rel!=='/pdfjs/web/mailwoman.html')continue;
        digests[rel]=createHash('sha256').update(readFileSync(file)).digest('hex');
      }
      const htmlPath = join(outDir, 'index.html');
      const html = readFileSync(htmlPath, 'utf8').replace(
        /<(script|link)\b([^>]*?)\s(src|href)="\.\/(assets\/[^"]+\.js)"([^>]*)>/g,
        (tag: string, name: string, before: string, attr: string, path: string, after: string) => {
          if (tag.includes(' integrity=')) return tag;
          const sri = `sha384-${createHash('sha384').update(readFileSync(join(outDir, path))).digest('base64')}`;
          return `<${name}${before} ${attr}="./${path}" integrity="${sri}"${after}>`;
        },
      );
      writeFileSync(htmlPath, html);
      // The shell itself, after SRI: the worker serves a stored shell only if
      // it is this file (with the server's base-shim tag removed).
      digests['/index.html'] = createHash('sha256').update(html).digest('hex');
      const unsri = html.match(/<script[^>]*\ssrc=[^>]*>/g)?.filter((tag) => tag.includes('assets/') && !tag.includes(' integrity=')) ?? [];
      if (unsri.length > 0) throw new Error(`mw-asset-integrity: script tags without SRI: ${unsri.join(' ')}`);
      const swPath = join(outDir, 'sw.js');
      const sw = readFileSync(swPath, 'utf8');
      const marker = '/* @mw-asset-digests */ null';
      if (!sw.includes(marker)) throw new Error('mw-asset-integrity: dist/sw.js has no asset-digest placeholder');
      writeFileSync(swPath, sw.replace(marker, JSON.stringify(digests)).replace('/* @mw-offline-assets */ []',JSON.stringify(Object.keys(digests).filter(path=>(path.startsWith('/pdfjs/')||/\.(?:js|mjs|css|wasm)$/.test(path)||isAppCatalogPack(path))&&!/^\/assets\/(?:admin|offline-background)/.test(path)))).replace('/* @mw-offline-background */',offlineBackgroundFile?swTrustedImport(offlineBackgroundFile):''));
    },
  };
}

export default defineConfig({
  // Sub-path hosting (t20 B4): a RELATIVE base so every emitted asset URL in
  // `dist/index.html` and every dynamic-import chunk resolves against the document
  // / importing chunk rather than the origin root. This is what lets the SAME built
  // bundle be served from `/` or from `/mail/` — the prefix is chosen at RUNTIME by
  // the server (`MW_BASE_PATH`), never baked in at build time.
  //
  // Consequence for code: `import.meta.env.BASE_URL` is now the literal './' and is
  // NOT the deploy prefix. Anything that needs the deploy prefix must go through
  // `basePath()` (`src/api/basePath.ts`), which reads the server-injected
  // `__MW_BASE__`.
  base: './',
  define: {
    __MW_I18N_PSEUDO__: JSON.stringify(process.env['MW_I18N_PSEUDO'] === '1'),
    __MW_VERSION__: JSON.stringify(VERSION),
  },
  // vanilla-extract compiles `*.css.ts` token/theme files to static CSS at build
  // (plan §2.3, e4). The hand-rolled Service Worker (e5) ships as `public/sw.js`,
  // copied verbatim into `dist/` — no bundling step needed for it.
  //
  // V4 (plan §2.5): `vite-plugin-wasm` lets the
  // crypto Web Worker import the wasm-pack `mw-crypto` (+ `mw-sanitize`) bundle
  // that e8 builds into `src/wasm/` via `scripts/build-wasm.*`. Added now so e8's
  // dynamic `import()` of the wasm module resolves; inert until that bundle exists.
  // Top-level await in the wasm modules is native (target es2022) and the workers are built as
  // ES modules (`worker.format`), so there is no `vite-plugin-top-level-await`: it re-printed each
  // chunk after its source map was written and misaligned every hidden map (docs/admin/observability.md).
  plugins: [chunkRetry(),catalogAssets(),cachedSolid({stats:buildCacheStats}), vanillaExtractPlugin(), wasm(), bundledPackages({ main: true }), bootScript(), offlineBackground(), assetIntegrity(),assetCompression(),buildTimings(buildCacheStats),bundleGraph()],
  // t32-e75: the bundle recorder also sees the worker builds; the main instance writes
  // node_modules/.cache/mailwoman-about/bundled-packages.json for the third-party list.
  worker: { format: 'es', plugins: () => [catalogAssets(undefined,{emit:false}),bundledPackages()] },
  server: {
    port: 5173,
    proxy: {
      '/api': { target: UPSTREAM, changeOrigin: true },
      '/jmap': { target: UPSTREAM, changeOrigin: true },
    },
  },
  build: {
    target: 'es2022',
    // `MW_SOURCEMAP=hidden` writes source maps next to the bundles WITHOUT a
    // `sourceMappingURL` comment. The server embeds them but refuses to serve any
    // `.map` and uses them to resolve browser error stack traces
    // (docs/admin/observability.md, Client errors).
    sourcemap: process.env['MW_SOURCEMAP'] === 'hidden' ? 'hidden' : false,
    // Small font subsets must remain same-origin files: the shell deliberately
    // permits font-src 'self', never data: font payloads in generated CSS.
    assetsInlineLimit: (filePath) => /\.(?:woff2?|ttf|otf|ftl)(?:\?|$)/i.test(filePath) ? false : undefined,
    // t52w: the chunk layout is explicit and layered (scripts/chunkPlan.ts);
    // scripts/check-load-budget.mjs (`pnpm run size`) holds it to load-budget.json.
    rollupOptions: {
      plugins: [plan.plugin, touchPlugin(webRoot, process.env['MW_LOAD_TOUCH'])],
      output: { manualChunks: plan.manualChunks },
      onwarn(warning, warn) {
        // A circular chunk import can run a module before its dependency: fail.
        if (warning.code === 'CIRCULAR_CHUNK') throw new Error(`chunkPlan: ${warning.message}`);
        warn(warning);
      },
    },
  },
  test: {
    // ── FLAKE POLICY (t22-e10) — there is deliberately NO `retry` here ────────
    //
    // Two specs flaked during 26.20, both only under host load, both green in
    // isolation and on re-run. `retry: 1` would have silenced both in one line.
    // It is not set, and should not be: a retry turns every nondeterminism into
    // a silent pass, which is the same defect as a test that passes for the
    // wrong reason — only installed as policy, applied to the whole suite, and
    // hiding the next one too. A flake nobody can see is a flake nobody fixes.
    //
    // The rule instead: **diagnose the mechanism, then match the fix to it.**
    // The two are worked examples, and they had DIFFERENT mechanisms despite
    // looking identical from the outside (green in isolation, red under load):
    //
    //   * `MessageList.lifecycle` — the TEST TIMEOUT. It mounts and scrolls 2000
    //     rows through jsdom: 2 382 ms of real work against the 5 000 ms default,
    //     a 2.1× margin. Fixed with a per-test timeout, row count untouched,
    //     because the 2000 rows are what make the bound it asserts meaningful.
    //
    //   * `Compose` — the FIND TIMEOUT, a different ceiling entirely, and it did
    //     not want a bigger budget at all. `findBy*` gives up after 1 000 ms
    //     regardless of `testTimeout`, and the first await in that file was
    //     waiting on a real dynamic import of the ~286 kB ProseMirror chunk,
    //     which in a full run competes with ~128 other files for the transform
    //     pipeline. Fixed by importing it in `beforeAll` — the cold cost moves
    //     out of an assertion budget into a hook with a generous one, and all
    //     three awaits KEEP the 1 s fast-fail. Proof: with the hook the file
    //     passes with the find budget starved to 1 ms; without it, at 1 ms,
    //     exactly the first await fails, with the failure text seen in the wild.
    //
    // The second one is the cautionary half. The first diagnosis of `Compose`
    // was "budget too tight, 3.1× margin" — measured in a single-file cold run,
    // which is not how it fails. Widening the budget on that basis would have
    // gone green and left the contended import inside a 1 s ceiling, ready to
    // flake again on a busier day. The 1 ms control is what disproved it.
    //
    // So: measure the real cost against the budget that actually applies, in the
    // configuration where it actually fails, and prefer removing the cost from
    // the budget over widening the budget. Do not widen globally, do not reach
    // for `retry`, and do not shrink the work being measured — that last one
    // silently weakens whatever the test was asserting.
    environment: 'jsdom',
    globals: true,
    setupFiles: ['./src/test/setup.ts'],
    include: ['src/**/*.{test,spec}.{ts,tsx}'],
    // solid needs the browser-condition build in tests
    server: { deps: { inline: [/solid-js/, /@solidjs\/testing-library/] } },
    // t19 (26.19) coverage harness. `@vitest/coverage-v8` is a devDependency and
    // never enters the shipped bundle. Thresholds deliberately live OUTSIDE this
    // file: `.github/coverage-floors.toml` is the single ratchet for Rust and web
    // alike, and `.github/workflows/coverage.yml` checks `coverage-summary.json`
    // against it. Reports go to `test-results/` because that path is already
    // gitignored repo-wide — no .gitignore change was needed.
    coverage: {
      provider: 'v8',
      reportsDirectory: 'test-results/coverage',
      reporter: ['text-summary', 'json-summary', 'lcov'],
      include: ['src/**/*.{ts,tsx}'],
      exclude: [
        'src/**/*.{test,spec}.{ts,tsx}',
        'src/test/**',
        // vanilla-extract `*.css.ts` are compiled to static CSS at build time;
        // they carry no runtime branches worth measuring.
        'src/**/*.css.ts',
        'src/**/*.d.ts',
        // wasm-pack output (generated bindings, built by scripts/build-wasm.*).
        'src/wasm/**',
      ],
    },
  },
});
