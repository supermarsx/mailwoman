import { defineConfig } from 'vitest/config';
import solid from 'vite-plugin-solid';
import { vanillaExtractPlugin } from '@vanilla-extract/vite-plugin';
import wasm from 'vite-plugin-wasm';
import topLevelAwait from 'vite-plugin-top-level-await';

const UPSTREAM = 'http://localhost:8080';

export default defineConfig({
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
