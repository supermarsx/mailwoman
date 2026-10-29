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

/**
 * Script origins the server admitted for this document's analytics tag (t63).
 *
 * The server writes `<meta name="mw-analytics" content="https://a https://*.b">`
 * only into the mail shell it serves to a signed-in user who consented to
 * analytics, and puts the same origins in that response's CSP `script-src`. A
 * provider tag injects its own `<script src>` (a TrustedScriptURL sink), so the
 * default policy must admit exactly those origins and nothing else. Read once,
 * when the policy is installed, before anything renders; never in the admin
 * console. Entries that are not `https://host[:port]` or `https://*.domain` are
 * ignored.
 */
export function analyticsScriptOrigins(doc: Document = document): readonly string[] {
  if (doc.querySelector('meta[name="mw-console"]') !== null) return [];
  const content = doc.querySelector('meta[name="mw-analytics"]')?.getAttribute('content') ?? '';
  return content.split(' ').filter(origin => /^https:\/\/(\*\.)?[a-z0-9.-]+(:\d{1,5})?$/.test(origin));
}

/** True when `input` is an https URL whose origin is one of `origins`, matching a
 *  leading `*.` the way a CSP host source does (subdomains only, same port). */
export function isAnalyticsScriptUrl(input: string, origins: readonly string[]): boolean {
  if (origins.length === 0) return false;
  let url: URL;
  try { url = new URL(input); } catch { return false; }
  if (url.protocol !== 'https:' || url.username !== '' || url.password !== '') return false;
  return origins.some(source => {
    if (source === url.origin) return true;
    if (!source.startsWith('https://*.')) return false;
    const [host = '', port = ''] = source.slice('https://*.'.length).split(':');
    return url.port === port && url.hostname.endsWith(`.${host}`);
  });
}

/** Shape of the bits of `window.trustedTypes` we use. The DOM lib in our TS
 *  target does not declare it; narrow it locally (no runtime dependency). */
export interface TrustedTypesShim {
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
 * `createScriptURL` enforces {@link isAppScriptUrl}, plus
 * {@link isAnalyticsScriptUrl} for the origins the server admitted for this
 * document's analytics tag, and throws otherwise.
 */
export function installTrustedTypesPolicy(doc: Document = document, tt: TrustedTypesShim | undefined = (window as Window & { trustedTypes?: TrustedTypesShim }).trustedTypes): void {
  if (tt === undefined) return;
  // A page served with a browser analytics tag lists only the app's two policy
  // names. If either already exists, something ran before the app and owns it:
  // refuse to start rather than render through it (t63 F2).
  const tagged = doc.querySelector('meta[name="mw-analytics"]') !== null;
  if (tt.defaultPolicy !== null) {
    if (tagged) refuseTaggedStart(doc, 'a default Trusted Types policy already exists');
    return;
  }
  const analytics = analyticsScriptOrigins(doc);
  tt.createPolicy('default', {
    createHTML: (html) => html,
    createScriptURL: (url) => {
      if (isAppScriptUrl(url) || isAnalyticsScriptUrl(url, analytics)) return url;
      throw new TypeError(
        `Trusted Types: refused to load script URL ${url} — not an app-owned ` +
          `script asset (same-origin /assets/*.js, /sw.js or /pdf.worker.mjs).`,
      );
    },
  });
  if (!reserveSveltePolicy(tt) && tagged) refuseTaggedStart(doc, `the ${SVELTE_POLICY} policy already exists`);
}

/**
 * Stop a tagged page from starting: drop the marker the analytics loader
 * requires (so a not-yet-run loader stands down) and throw, which ends the
 * entry module before anything renders.
 */
function refuseTaggedStart(doc: Document, why: string): never {
  for (const meta of doc.querySelectorAll('meta[name="mw-analytics"]')) meta.remove();
  throw new Error(`Trusted Types: ${why} before the app started; refusing to start a page with an analytics tag.`);
}

/** The policy Svelte (the ALTCHA widget) creates when its module loads. */
export const SVELTE_POLICY = 'svelte-trusted-html';

/**
 * Create Svelte's policy now, before any later script can (t63 T5). A page with
 * a browser analytics tag is served with `trusted-types default
 * svelte-trusted-html` and no duplicates, so whoever creates a listed name first
 * owns it: here that is always the app. When Svelte asks for it later, it gets
 * this same policy (an HTML pass-through, exactly what Svelte defines) instead of
 * a duplicate-name error. Any other name goes to the browser, which refuses
 * names the page's policy does not list.
 */
export function reserveSveltePolicy(tt: TrustedTypesShim): boolean {
  let svelte: unknown;
  try {
    svelte = tt.createPolicy(SVELTE_POLICY, { createHTML: (html) => html, createScriptURL: () => { throw new TypeError('Trusted Types: svelte-trusted-html creates HTML only.'); } });
  } catch {
    return false;
  }
  const create = tt.createPolicy.bind(tt);
  try {
    Object.defineProperty(tt, 'createPolicy', {
      configurable: false,
      writable: false,
      value: (name: string, rules: Parameters<TrustedTypesShim['createPolicy']>[1]) => name === SVELTE_POLICY ? svelte : create(name, rules),
    });
  } catch { /* frozen factory: Svelte's own call may then fail, as before */ }
  return true;
}
