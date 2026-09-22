// The shell's Trusted Types default policy (SPEC §7.4; 26.17 introduced it,
// 26.20 t24-e13 added the script-URL half).
//
// The shell ships under `require-trusted-types-for 'script'` (the `CSP` constant
// in crates/mw-server/src/lib.rs), so every string reaching a DOM injection sink
// must come from a Trusted Types policy or the browser refuses the assignment.
//
// ── Why this module exists ─────────────────────────────────────────────────
// 26.17 registered a `default` policy exposing ONLY `createHTML`, on the stated
// reasoning that "workers load via `new URL(...)`, which is not a TT sink". That
// reasoning is wrong. `new Worker(url)` / `new SharedWorker(url)` /
// `ServiceWorkerContainer.register(url)` all take a **TrustedScriptURL**: the URL
// object is stringified and handed to the default policy's `createScriptURL`, and
// when no such callback exists the browser throws
//
//   Failed to construct 'Worker': This document requires 'TrustedScriptURL'
//   assignment and no 'default' policy for 'TrustedScriptURL' has been defined.
//
// So under the shipped CSP every worker-backed feature was dead: the mw-crypto
// worker (PGP + S/MIME), the zero-access worker, the in-worker sanitizer, the
// PDF.js viewer worker, and the offline service worker. Nothing caught it because
// vitest runs in jsdom, which sends no CSP, and the vite dev server sends none
// either — only the app served by mw-server with its real headers reproduces it.
//
// ── Why the policy is narrow ───────────────────────────────────────────────
// The fix is NOT a passthrough `createScriptURL: (u) => u`. That would hand back
// any string at all, turning every worker constructor in the app into a
// script-execution sink for whatever URL an attacker could get into it — exactly
// the class of bug `require-trusted-types-for` exists to prevent, and it would
// make the directive decorative.
//
// Instead {@link isAppScriptUrl} accepts ONLY the app's own script assets:
//
//   * same origin as the document — `location.origin` compared exactly, so a
//     cross-origin CDN, a protocol downgrade, and the `data:`, `blob:`,
//     `javascript:` and `filesystem:` schemes are all refused (the app builds no
//     worker from a blob — every `URL.createObjectURL` call is a download or an
//     attachment preview, never a script);
//   * under the deploy prefix, then one of:
//       - `/assets/<name>.js` or `.mjs` — Vite's `build.assetsDir`, where every
//         emitted worker chunk lands (`assets/worker.entry-<hash>.js`);
//       - exactly `/sw.js` — the hand-rolled service worker, copied verbatim
//         from `public/` (src/sw/register.ts);
//       - exactly `/pdf.worker.mjs` — the vendored PDF.js worker, self-hosted in
//         `public/` and never a CDN (src/viewers/pdfWorkerSrc.ts).
//
// Anything else throws, which surfaces at the construction site as a TypeError
// rather than silently loading. `createScript` is deliberately still UNSUPPLIED:
// the app has no string-to-code sink (no `eval`, no `new Function`), so that one
// stays fail-closed as 26.17 intended.
//
// Note "unsupplied", not "absent": a `TrustedTypePolicy` exposes `createHTML`,
// `createScript` and `createScriptURL` on its prototype whichever callbacks were
// passed to `createPolicy`, and an unsupplied one throws only when CALLED. So
// `typeof policy.createScript === 'function'` is true either way and proves
// nothing — a test for this property has to invoke it (see
// apps/web/e2e/crypto-trustedtypes.spec.ts).
//
// Query strings and fragments are ignored (Vite does not add them to worker URLs,
// but a `?worker` style suffix must not defeat the extension check); the path is
// taken from the parsed `URL`, so `..` traversal is normalized away before the
// prefix test.

import { stripBase } from '../api/basePath.ts';

/** Root-served scripts that are not Vite chunks: the SW and the PDF.js worker. */
const ROOT_SCRIPTS = new Set(['/sw.js', '/pdf.worker.mjs']);

/** Vite's `build.assetsDir`, where every emitted worker chunk lands. */
const ASSETS_PREFIX = '/assets/';

/**
 * True when `input` names one of the app's own script assets, per the rules in
 * this module's header. `origin` and `base` are injectable so the rule is
 * testable under jsdom without touching globals.
 */
export function isAppScriptUrl(
  input: string,
  origin: string = location.origin,
  base?: string,
): boolean {
  let url: URL;
  try {
    // Resolve relative URLs the way the sink would, against the document.
    url = new URL(input, `${origin}/`);
  } catch {
    return false;
  }
  // Exact-origin match rejects every non-http(s) scheme too: `blob:`/`data:`
  // URLs parse with an origin of `null` (or the inner origin), never this one.
  if (url.origin !== origin) return false;

  const path = stripBase(url.pathname, base);
  if (ROOT_SCRIPTS.has(path)) return true;
  if (!path.startsWith(ASSETS_PREFIX)) return false;
  // A bare `/assets/` directory, or a nested path, is not a chunk name.
  const name = path.slice(ASSETS_PREFIX.length);
  if (name === '' || name.includes('/')) return false;
  return name.endsWith('.js') || name.endsWith('.mjs');
}

/** Shape of the bits of `window.trustedTypes` we use. The DOM lib in our TS
 *  target does not declare it; narrow it locally (no runtime dependency). */
interface TrustedTypesShim {
  readonly defaultPolicy: unknown;
  createPolicy(
    name: string,
    rules: {
      createHTML: (input: string) => string;
      createScriptURL: (input: string) => string;
    },
  ): unknown;
}

/**
 * Register the `default` Trusted Types policy.
 *
 * MUST run before `render()` and before any worker is constructed: Solid's very
 * first `template().innerHTML` boot write goes through `createHTML`, so without
 * the policy in place the SPA does not render at all under the enforced CSP.
 *
 * `createHTML` passes its input through unchanged, as it has since 26.17 — every
 * sink in this app is app-controlled or sanitized upstream: Solid compiles its
 * templates from static JSX (compile-time constants); the notes editor assigns
 * `sanitizeNoteHtml(...)` (modules/notes/Editor.tsx); the key card injects
 * module-generated QR SVG (modules/keys); the composer parses into a detached
 * container it never renders (components/compose/richtext.ts); the plugin host
 * builds its own iframe srcdoc (plugins-ui/host.ts). ProseMirror's own
 * `ProseMirrorClipboard` policy reuses this default when present
 * (prosemirror-view checks `trustedTypes.defaultPolicy` first).
 *
 * `createScriptURL` enforces {@link isAppScriptUrl} and throws otherwise.
 */
export function installTrustedTypesPolicy(): void {
  const tt = (window as Window & { trustedTypes?: TrustedTypesShim }).trustedTypes;
  if (tt === undefined || tt.defaultPolicy !== null) return;
  tt.createPolicy('default', {
    createHTML: (html) => html,
    createScriptURL: (url) => {
      if (isAppScriptUrl(url)) return url;
      throw new TypeError(
        `Trusted Types: refused to load script URL ${url} — not an app-owned ` +
          `script asset (same-origin /assets/*.js, /sw.js or /pdf.worker.mjs).`,
      );
    },
  });
}
