import { defineConfig } from 'vitest/config';
import solid from 'vite-plugin-solid';
import { vanillaExtractPlugin } from '@vanilla-extract/vite-plugin';
import wasm from 'vite-plugin-wasm';
import topLevelAwait from 'vite-plugin-top-level-await';
import { readFileSync } from 'node:fs';
import { transformWithEsbuild, type Plugin } from 'vite';
import { bundledPackages } from './scripts/about-bundled-packages.mjs';

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
  // V4 (plan §2.5): `vite-plugin-wasm` + `vite-plugin-top-level-await` let the
  // crypto Web Worker import the wasm-pack `mw-crypto` (+ `mw-sanitize`) bundle
  // that e8 builds into `src/wasm/` via `scripts/build-wasm.*`. Added now so e8's
  // dynamic `import()` of the wasm module resolves; inert until that bundle exists.
  plugins: [solid(), vanillaExtractPlugin(), wasm(), topLevelAwait(), bundledPackages({ main: true }), bootScript()],
  // t32-e75: the bundle recorder also sees the worker builds; the main instance writes
  // node_modules/.cache/mailwoman-about/bundled-packages.json for the third-party list.
  worker: { plugins: () => [bundledPackages()] },
  server: {
    port: 5173,
    proxy: {
      '/api': { target: UPSTREAM, changeOrigin: true },
      '/jmap': { target: UPSTREAM, changeOrigin: true },
    },
  },
  build: {
    target: 'es2022',
    sourcemap: false,
    // Small font subsets must remain same-origin files: the shell deliberately
    // permits font-src 'self', never data: font payloads in generated CSS.
    assetsInlineLimit: (filePath) => /\.(?:woff2?|ttf|otf)(?:\?|$)/i.test(filePath) ? false : undefined,
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
