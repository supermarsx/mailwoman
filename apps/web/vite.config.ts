import { defineConfig } from 'vitest/config';
import solid from 'vite-plugin-solid';
import { vanillaExtractPlugin } from '@vanilla-extract/vite-plugin';
import wasm from 'vite-plugin-wasm';
import topLevelAwait from 'vite-plugin-top-level-await';

const UPSTREAM = 'http://localhost:8080';

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
  // vanilla-extract compiles `*.css.ts` token/theme files to static CSS at build
  // (plan §2.3, e4). The hand-rolled Service Worker (e5) ships as `public/sw.js`,
  // copied verbatim into `dist/` — no bundling step needed for it.
  //
  // V4 (plan §2.5): `vite-plugin-wasm` + `vite-plugin-top-level-await` let the
  // crypto Web Worker import the wasm-pack `mw-crypto` (+ `mw-sanitize`) bundle
  // that e8 builds into `src/wasm/` via `scripts/build-wasm.*`. Added now so e8's
  // dynamic `import()` of the wasm module resolves; inert until that bundle exists.
  plugins: [solid(), vanillaExtractPlugin(), wasm(), topLevelAwait()],
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
