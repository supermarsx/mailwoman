import js from '@eslint/js';
import tseslint from 'typescript-eslint';
import solid from 'eslint-plugin-solid/configs/typescript';

export default tseslint.config(
  {
    ignores: [
      'dist/**',
      'node_modules/**',
      'e2e/**',
      'coverage/**',
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
