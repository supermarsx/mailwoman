import { defineConfig } from 'vitest/config';
import solid from 'vite-plugin-solid';

const UPSTREAM = 'http://localhost:8080';

export default defineConfig({
  plugins: [solid()],
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
  },
});
