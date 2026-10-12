import js from '@eslint/js';
import tseslint from 'typescript-eslint';
import solid from 'eslint-plugin-solid/configs/typescript';

// `mw/no-global-fetch` — every request the web client makes goes through
// `apiFetch` (`src/api/http.ts`), which is what puts the CSRF token on a write. A
// direct `fetch` sends none, and a server running `MW_CSRF_STRICT=true` (the
// `deploy.sh` default) answers such a write 403.
//
// Reports every reference to the global `fetch`: a call, an alias
// (`const f = fetch`), a default (`fetcher = fetch`). Not reported: a local
// binding that shadows it, a property named `fetch`, and `typeof fetch`, which
// asks whether the global exists and sends nothing. `window.fetch`,
// `globalThis.fetch` and `self.fetch` are covered by `no-restricted-properties`
// beside it.
const noGlobalFetch = {
  meta: {
    type: 'problem',
    schema: [],
    messages: {
      direct:
        'Call apiFetch from src/api/http.ts instead of fetch: a direct fetch sends no X-CSRF-Token, and the server refuses the write when MW_CSRF_STRICT is on.',
    },
  },
  create(context) {
    return {
      'Program:exit'(program) {
        const globalScope = context.sourceCode.getScope(program);
        const declared = globalScope.set.get('fetch')?.references ?? [];
        const seen = new Set();
        for (const ref of [...globalScope.through, ...declared]) {
          const id = ref.identifier;
          if (id.name !== 'fetch' || seen.has(id)) continue;
          seen.add(id);
          const parent = id.parent;
          if (parent?.type === 'UnaryExpression' && parent.operator === 'typeof') continue;
          if (parent?.type === 'TSTypeQuery') continue;
          context.report({ node: id, messageId: 'direct' });
        }
      },
    };
  },
};

const FETCH_PROPERTY_MESSAGE = 'Call apiFetch from src/api/http.ts instead: it adds the CSRF token to a write.';

export default tseslint.config(
  {
    ignores: [
      'dist/**',
      'node_modules/**',
      // `e2e/**` is NOT ignored (t24-e13). The Playwright specs gate the product,
      // so leaving them the one unlinted, untypechecked TypeScript in the repo
      // meant the gates themselves had no gate. They are hand-authored source and
      // are cheap to keep clean — bringing them in cost one real fix and two stale
      // `eslint-disable` directives. `tsconfig.json` includes them for the same
      // reason; see the note there for what the typecheck surfaced.
      'coverage/**',
      // Generated Playwright live-E2E output (gitignored; left on disk after a
      // local run). Minified trace bundles — never hand-authored, not shipped.
      'playwright-report/**',
      'test-results/**',
      // Vendored third-party build artifact (self-hosted pdf.js worker, e8).
      'public/pdf.worker.mjs',
      // Generated wasm-pack bundle (mw-crypto → src/wasm, built by
      // scripts/build-wasm.*; committed but not hand-authored, plan §3 e8).
      'src/wasm/**',
    ],
  },
  js.configs.recommended,
  ...tseslint.configs.recommended,
  {
    files: ['src/**/*.{ts,tsx}'],
    ...solid,
    languageOptions: {
      parser: tseslint.parser,
      parserOptions: {
        project: './tsconfig.json',
      },
    },
    rules: {
      '@typescript-eslint/no-explicit-any': 'error',
      '@typescript-eslint/no-unused-vars': [
        'error',
        { argsIgnorePattern: '^_', varsIgnorePattern: '^_' },
      ],
    },
  },
  {
    // One caller of `fetch` in the web client (see `noGlobalFetch` above). Outside
    // the rule: `src/api/http.ts` itself, the service-worker code under `src/sw`
    // (it runs where there is no `document.cookie`), and tests, which stub `fetch`.
    files: ['src/**/*.{ts,tsx}'],
    ignores: ['src/api/http.ts', 'src/sw/**', 'src/**/*.{test,spec}.{ts,tsx}', 'src/test/**'],
    plugins: { mw: { rules: { 'no-global-fetch': noGlobalFetch } } },
    rules: {
      'mw/no-global-fetch': 'error',
      'no-restricted-properties': [
        'error',
        { object: 'window', property: 'fetch', message: FETCH_PROPERTY_MESSAGE },
        { object: 'globalThis', property: 'fetch', message: FETCH_PROPERTY_MESSAGE },
        { object: 'self', property: 'fetch', message: FETCH_PROPERTY_MESSAGE },
      ],
    },
  },
  {
    files: ['**/*.{test,spec}.{ts,tsx}', 'src/test/**'],
    rules: {
      '@typescript-eslint/no-non-null-assertion': 'off',
    },
  },
  {
    // Node build scripts (not shipped) — allow node globals.
    files: ['scripts/**/*.mjs'],
    languageOptions: {
      globals: {
        console: 'readonly',
        process: 'readonly',
        URL: 'readonly',
      },
    },
  },
  {
    // Service Worker (public/sw.js, e5) runs in the ServiceWorkerGlobalScope.
    files: ['public/**/*.js'],
    languageOptions: {
      globals: {
        self: 'readonly',
        caches: 'readonly',
        clients: 'readonly',
        fetch: 'readonly',
        Request: 'readonly',
        Response: 'readonly',
        URL: 'readonly',
      },
    },
  },
);
