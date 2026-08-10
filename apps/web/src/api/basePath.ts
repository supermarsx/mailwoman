// Sub-path hosting (t20 B4) — the ONE resolver for the deploy prefix.
//
// mailwoman may be served from the origin root (`/`) or from a prefix (`/mail`).
// Which one is a RUNTIME decision made by the server (`MW_BASE_PATH`), never a
// build-time one: the same `dist/` — and therefore the same single binary and the
// same container image — must serve from either. The server injects the chosen
// prefix into `index.html` as `globalThis.__MW_BASE__`; this module is where the
// SPA reads it.
//
// Two things are deliberately NOT this module's job:
//
//   * Build assets. `vite.config.ts` sets `base: './'`, so `index.html`, every
//     dynamic-import chunk, every worker and the public-dir `url()` references in
//     the compiled CSS are already importer-relative and resolve under any prefix
//     on their own. Only `fetch`/`WebSocket`/`EventSource` targets and the
//     service-worker registration — which the bundler cannot see — need this.
//
//   * Imports. This module has NONE, on purpose. `api/transport.ts` imports
//     `platform/index.ts`, which imports `platform/browser.ts`, which needs the
//     prefix; any import here would close that cycle. (`platform/browser.ts:56`
//     already hand-copies `serverBase()` rather than import `transport.ts`, with a
//     comment naming the same cycle.)
//
// When `__MW_BASE__` is absent — the dev server, the native Tauri shell, a server
// that has not been configured with a prefix — `basePath()` is `''` and every
// path this module produces is byte-identical to the pre-sub-path code.

/**
 * Canonicalize an operator-supplied prefix. Accepts `/mail`, `mail`, `/mail/`,
 * `/`, `''` or a non-string; yields either `''` (origin root) or a leading-slash,
 * no-trailing-slash prefix such as `/mail`.
 */
export function normalizeBase(raw: unknown): string {
  if (typeof raw !== 'string') return '';
  let s = raw.trim();
  if (s === '') return '';
  if (!s.startsWith('/')) s = `/${s}`;
  while (s.length > 1 && s.endsWith('/')) s = s.slice(0, -1);
  return s === '/' ? '' : s;
}

/** The deploy prefix: `''` at the origin root, else e.g. `/mail`. */
export function basePath(): string {
  return normalizeBase((globalThis as { __MW_BASE__?: unknown }).__MW_BASE__);
}

/**
 * Prefix a root-absolute app path. `path` must start with `/`; the result is
 * `path` verbatim at the root and `/mail/api/…` under a prefix.
 */
export function withBase(path: string): string {
  return `${basePath()}${path}`;
}

/**
 * The app-shell URL — `/` or `/mail/`, always with a trailing slash. This is both
 * the service-worker `scope` and the shell entry the SW precaches for offline
 * navigation.
 */
export function shellBase(): string {
  return `${basePath()}/`;
}

/**
 * Remove the deploy prefix from a same-origin pathname, so prefix-blind matchers
 * (`/api/…`, `/assets/…`) keep working under a sub-path. A pathname that does not
 * start with the prefix is returned unchanged.
 */
export function stripBase(pathname: string, base = basePath()): string {
  if (base === '' || !pathname.startsWith(base)) return pathname;
  const rest = pathname.slice(base.length);
  // Only strip on a segment boundary: `/mailbox` must not become `box`.
  if (rest === '') return '/';
  return rest.startsWith('/') ? rest : pathname;
}
