// Sandboxed-container helpers for the attachment viewers (plan §1.7 / §2.4).
//
// Image / text / audio / video render inside an `<iframe sandbox="">` — the
// sandbox attribute intentionally omits BOTH `allow-scripts` and
// `allow-same-origin`, so the frame gets an opaque origin, runs no script, and
// cannot reach the parent (mirrors the message-body iframe in Reader.tsx). A
// per-document `<meta http-equiv="Content-Security-Policy">` further pins the
// frame to `default-src 'none'` plus only the one resource type it needs.
//
// Because the frame has an opaque origin it CANNOT read a parent-created
// `blob:` object URL (blob URLs are same-origin only). So media is inlined as a
// self-contained `data:` URL inside `srcdoc`, which an opaque origin can load.
// Text is inlined directly (escaped) — no external fetch at all. This keeps the
// "no transcode" rule (native <img>/<audio>/<video>) while staying script-free.

import { apiFetch } from '../api/http.ts';
import { frameTipParts, withoutImageTitles, type FrameTips } from './frameTooltip.ts';
import { adaptReaderColors, readerAppearanceCss, readerStyleDeclarations, type ReaderAppearance } from './readerAppearance.ts';

/** The frozen sandbox token set: no scripts, no same-origin, opaque frame. */
export const SANDBOX_TOKENS = '';

// Every policy in this file is delivered through `<meta http-equiv>` inside a
// srcdoc, where `frame-ancestors` is ignored by specification (CSP3 §6.4.2) and
// Firefox logs a warning for it on every message open. It is therefore not
// emitted here; embedding is restricted by the HTTP `frame-ancestors` and
// `X-Frame-Options` on the pages that host these frames (t56).

/** CSP a media frame is pinned to — only inline styles + a `data:` resource. */
function mediaCsp(resourceDir: 'img-src' | 'media-src'): string {
  return ["default-src 'none'", `${resourceDir} data:`, "style-src 'unsafe-inline'"].join('; ');
}

/** CSP a text frame is pinned to — inline styles only, zero external loads. */
const TEXT_CSP = ["default-src 'none'", "style-src 'unsafe-inline'"].join('; ');

const FRAME_STYLE =
  'html,body{margin:0;height:100%;background:transparent}' +
  'body{display:flex;align-items:center;justify-content:center}' +
  'img,video,audio{max-width:100%;max-height:100%;display:block}' +
  'pre{margin:0;padding:12px;width:100%;height:100%;box-sizing:border-box;overflow:auto;' +
  "white-space:pre-wrap;word-break:break-word;font:13px/1.5 ui-monospace,SFMono-Regular,Menlo,monospace}";

function frameDoc(csp: string, body: string): string {
  return (
    '<!doctype html><html><head><meta charset="utf-8">' +
    `<meta http-equiv="Content-Security-Policy" content="${csp}">` +
    `<style>${FRAME_STYLE}</style></head><body marginwidth="0" marginheight="0">${body}</body></html>`
  );
}

/** `srcdoc` for an image/audio/video frame; `dataUrl` is a self-contained `data:`. */
export function mediaFrameDoc(kind: 'image' | 'audio' | 'video', dataUrl: string, controls=true): string {
  const dir = kind === 'image' ? 'img-src' : 'media-src';
  const el =
    kind === 'image'
      ? `<img alt="attachment" width="100%" src="${dataUrl}">`
      : kind === 'audio'
        ? `<audio ${controls?'controls':''} src="${dataUrl}"></audio>`
        : `<video ${controls?'controls':''} playsinline src="${dataUrl}"></video>`;
  return frameDoc(mediaCsp(dir), el);
}

/** `srcdoc` for a text frame; `text` is escaped and inlined (no fetch). */
export function textFrameDoc(text: string): string {
  return frameDoc(TEXT_CSP, `<pre>${escapeHtml(text)}</pre>`);
}

export function escapeHtml(s: string): string {
  return s.replace(/&/g, '&amp;').replace(/</g, '&lt;').replace(/>/g, '&gt;');
}

// ── Max-security opening mode (plan §3 e5, §7.2) ─────────────────────────────
//
// The message-body frame (Reader.tsx) can be pinned to one of three security
// postures. This is ADDITIVE: `bodyCsp`/`bodyFrameDoc` default to
// `full-sanitized`, which reproduces the pre-V4 body render (sanitized HTML,
// images allowed). The policy/precedence that PICKS a mode lives in
// `max-security.ts`; this file only knows how to turn a chosen mode into a CSP
// + a `srcdoc`. The sandbox contract above is unchanged — the body frame still
// carries `sandbox=""` (no allow-scripts, no allow-same-origin); the CSP here
// is defense-in-depth layered on top.

/** The three max-security opening positions, ordered least→most restrictive:
 *  `full-sanitized` (default, current behavior) → `sanitized-no-media`
 *  (images/media blocked) → `plain-text` (HTML not rendered at all). */
export type SecurityMode = 'full-sanitized' | 'sanitized-no-media' | 'plain-text';

/** CSP the sanitized message-body frame is pinned to for a given mode.
 *  `full-sanitized` keeps the pre-V4 permissive-image body; the other two drop
 *  every image/media source so nothing external loads (belt-and-braces with the
 *  sanitizer, which also strips the tags). */
export function bodyCsp(mode: SecurityMode = 'full-sanitized', direct = false, serverBase?: string): string {
  if (mode === 'full-sanitized') {
    // Opaque sandbox origins do not match CSP 'self' in Firefox.
    // Only the application-selected server origin may supply signed images.
    let imageOrigin = '';
    try { const server = new URL(serverBase ?? ''); if (['http:', 'https:'].includes(server.protocol) && !server.username && !server.password) imageOrigin = ` ${server.origin}`; } catch { /* no additional source */ }
    return [
      "default-src 'none'",
      `img-src 'self' data: blob:${imageOrigin}${direct ? ' https:' : ''}`,
      "media-src 'none'",
      "style-src 'unsafe-inline'",
      'font-src data:',
    ].join('; ');
  }
  // sanitized-no-media AND plain-text: no external/media source of any kind.
  return ["default-src 'none'", "style-src 'unsafe-inline'"].join('; ');
}

/** The 14px "image unavailable" glyph: a picture frame struck through. */
const BROKEN_IMAGE_ICON =
  "url(\"data:image/svg+xml,%3Csvg xmlns='http://www.w3.org/2000/svg' viewBox='0 0 16 16' fill='none' stroke='%236b7280' stroke-width='1.3'%3E%3Crect x='1.5' y='2.5' width='13' height='11' rx='1.5'/%3E%3Cpath d='M1.5 11l3.5-3.5 3 3 2-2 4.5 4.5'/%3E%3Cpath d='M2 2l12 12'/%3E%3C/svg%3E\")";

/**
 * How an image that failed to load looks in the message body (t56): a small
 * bordered box with the glyph and the sender's alt text, in the image's own box.
 *
 * The body frame is `sandbox=""` and runs no script, so nothing can listen for an
 * image `error` and swap in an element; the browser's own broken-image rendering
 * is what is styled. `::before`/`::after` on an `<img>` are generated ONLY while
 * it shows no image (Chromium and Firefox; WebKit falls back to its native
 * rendering), so none of this touches an image that loaded. A failed proxied
 * image is an HTTP error from `/api/rc/i/…` (a timeout is a 504), cached briefly
 * by the server and the browser, so it settles once instead of retrying.
 *
 * Nothing jumps: a sized image (`width`+`height`) keeps exactly that box — the UA
 * maps the attributes to `aspect-ratio: auto w / h`, which a broken image still
 * honours under `height:auto` — and the label is clipped inside it. An image with
 * neither has no known size, so it takes one line of label. `alt=""` marks a
 * decorative image: it collapses to nothing, as before. No `alt` at all keeps the
 * browser's native icon.
 *
 * The two engines size a broken image differently: Chromium from its native
 * icon-plus-alt line (so the label is laid over it), Firefox from generated
 * content in flow (an absolutely positioned label would leave it 0×0) — hence the
 * one `@supports (-moz-appearance:none)` rule. Probe screenshots: logs/t56-e1.md.
 */
export const BROKEN_IMAGE_STYLE =
  'img{display:inline-block;position:relative;vertical-align:middle;overflow:hidden;' +
  'font:12px/20px ui-sans-serif,system-ui,sans-serif;color:transparent}' +
  'img::before{content:"";position:absolute;inset:0;box-sizing:border-box;' +
  'border:1px solid var(--mw-color-border,rgba(127,127,127,.35));border-radius:4px;' +
  `background:var(--mw-color-surface,#f3f4f6) ${BROKEN_IMAGE_ICON} no-repeat 4px 4px/14px 14px}` +
  'img::after{content:attr(alt);position:absolute;left:0;right:0;top:0;box-sizing:border-box;height:22px;' +
  'padding:1px 6px 1px 22px;color:var(--mw-color-text-dim,#6b7280);white-space:nowrap;overflow:hidden;text-overflow:ellipsis}' +
  'img[alt=""]::before,img[alt=""]::after,img:not([alt])::before,img:not([alt])::after{content:none}' +
  '@supports (-moz-appearance:none){img::after{position:relative;display:block;max-width:100%;min-width:22px}}';

/** Per-image loading chrome is removed by the hash-authorized frame handler.
 * Sender sizing remains authoritative; only a genuinely empty unsized box
 * receives a small temporary minimum. Stop animation after thirty seconds,
 * leaving a static placeholder if the network itself has not settled. */
export const IMAGE_LOADING_STYLE =
  'img[data-mw-image-state="loading"]{background-color:var(--mw-color-surface,#f3f4f6)!important;' +
  // Fallback first: an engine without color-mix() drops the second declaration only.
  'background-image:linear-gradient(100deg,transparent 15%,rgba(127,127,127,.38) 50%,transparent 85%)!important;' +
  'background-image:linear-gradient(100deg,transparent 15%,color-mix(in srgb,var(--mw-color-text,#1c1e21) 16%,transparent) 50%,transparent 85%)!important;' +
  'background-size:200% 100%!important;background-repeat:no-repeat!important;' +
  // A sweep plus a soft opacity pulse, so a tiny box or a low-contrast theme still visibly moves.
  'animation:mw-reader-image-loading 1.5s ease-in-out 40,mw-reader-image-pulse 1.5s ease-in-out 40}' +
  'img[data-mw-image-state="loading"]::before,img[data-mw-image-state="loading"]::after{content:none}' +
  'img[data-mw-image-state="loading"][data-mw-image-unsized]{min-width:24px;min-height:18px}' +
  '@keyframes mw-reader-image-loading{from{background-position:100% 0}to{background-position:-100% 0}}' +
  '@keyframes mw-reader-image-pulse{0%,100%{opacity:1}50%{opacity:.6}}' +
  '@media(prefers-reduced-motion:reduce){img[data-mw-image-state="loading"]{animation:mw-reader-image-pulse 3s ease-in-out 20!important}}';

/** Minimal readable body styling (distinct from the media-centering FRAME_STYLE).
 *  The opaque-origin body frame can't inherit the parent's theme vars, so e8
 *  passes the stable `--mw-*` block via `opts.themeVars`. */
const BODY_STYLE =
  'html,body{margin:0;background:transparent}' +
  'body{padding:0;font:14px/1.6 ui-sans-serif,system-ui,sans-serif;word-break:break-word;' +
  'color:var(--mw-color-text,#1c1e21)}' +
  // `max-width:100%;height:auto` is deliberately kept: it is the pairing that
  // makes an intrinsic `width`/`height` USABLE. The UA maps the two attributes
  // to `aspect-ratio`, and `height:auto` lets the box shrink along that ratio
  // rather than being pinned to the attribute — so a sized image reserves its
  // space before it decodes. The attributes themselves come from
  // `withImageLoadingHints` below (`loading`/`decoding`) and from the sender's
  // own `width`/`height`, which the sanitizer's img allow-list keeps.
  'img,video{max-width:100%;height:auto}' +
  'pre{white-space:pre-wrap;word-break:break-word;margin:0;' +
  'font:13px/1.5 ui-monospace,SFMono-Regular,Menlo,monospace}';

/**
 * Add `loading="eager"` + `decoding="async"` to every `<img>` in an ALREADY
 * SANITIZED body fragment (t22 L6).
 *
 * The sanitizer's allow-list is untouched by design: `mw-sanitize` keeps
 * ammonia's default `img` attribute set (`align`/`alt`/`height`/`src`/`width`),
 * so a sender's intrinsic `width`/`height` survives sanitization but `loading`
 * and `decoding` do not. Rather than widen the allow-list — a `mw-sanitize`
 * change, and a wider strip surface for one layout hint — the hints are applied
 * here, on the client, AFTER sanitization. Nothing that reaches the frame gains
 * an attribute the sanitizer rejected; two inert layout hints are added to
 * output the sanitizer already approved.
 *
 * The rewrite goes through `DOMParser`, never a regex. A regex inserting after a
 * literal `<img` is only safe while the producer escapes `<` inside attribute
 * values — ammonia does today (`imageHints.test.ts` pins it), the DOM serializer
 * does not — and that is a property of a producer this code does not own, checked
 * nowhere, and silently load-bearing if assumed. Parsing already-sanitized markup
 * and re-serializing it is invariant to it, and cannot introduce script. Bodies
 * with no `<img>` at all are returned byte-identical (the common case, and it
 * keeps the default cleartext body's `srcdoc` contract unchanged for them).
 */
export function withImageLoadingHints(html: string): string {
  if (!/<img\b/i.test(html)) return html;
  if (typeof DOMParser === 'undefined') return html;
  let doc: Document;
  try {
    doc = new DOMParser().parseFromString(html, 'text/html');
  } catch {
    return html;
  }
  const imgs = doc.body.querySelectorAll('img');
  if (imgs.length === 0) return html;
  // Never `lazy`: the frame is as tall as its content and the reader scrolls
  // outside it, so a lazy image below the fold waits for a scroll to start loading.
  // Every image starts at once; the first few (the ones in view) go first.
  Array.from(imgs).forEach((img, index) => {
    img.setAttribute('loading', 'eager');
    if (!img.hasAttribute('decoding')) img.setAttribute('decoding', 'async');
    if (!img.hasAttribute('fetchpriority')) img.setAttribute('fetchpriority', index < 4 ? 'high' : 'auto');
  });
  return doc.body.innerHTML;
}

/** The 1x1 transparent GIF `rewriteReaderContent` puts in place of a blocked image. */
const BLOCKED_GIF_PREFIX = 'data:image/gif;base64,R0lGODlhAQABAI';
const PH_GIF = `img[data-mw-ph][src^="data:"][src*="R0lGODlhAQABAI"]`;
// A placeholder whose real source was just patched in (Show images) keeps its box
// while the shimmer runs, so the layout does not collapse and re-expand.
const PH_LOADING = 'img[data-mw-ph][data-mw-image-state="loading"]';
const PH_ANY = `img[data-mw-ph]:not([src]),${PH_GIF},${PH_LOADING}`;
const PH_CROSS =
  'linear-gradient(to top right,transparent calc(50% - .75px),rgba(127,127,127,.45) 50%,transparent calc(50% + .75px)),' +
  'linear-gradient(to bottom right,transparent calc(50% - .75px),rgba(127,127,127,.45) 50%,transparent calc(50% + .75px))';
const phKind = (kind: string): string => `img[data-mw-ph="${kind}"]:not([src]),img[data-mw-ph="${kind}"][src^="data:"][src*="R0lGODlhAQABAI"],img[data-mw-ph="${kind}"][data-mw-image-state="loading"]`;

/**
 * Placeholder for an image that is blocked or has nothing to load (the default
 * safe mode). Pure CSS on the `<img>` itself: the element and its position in the
 * document never change, so the in-place hot patcher (which addresses body
 * elements by index and later sets `src`) keeps working. The rules match only
 * while the image has NO `src` (or still the transparent blocked GIF); once a
 * signed source is patched in they stop matching and the picture takes over.
 * No `data:` URL is used (the no-media CSP forbids it); the generic-image look is
 * a gradient cross. The `data-mw-image-state="loading"` shimmer still wins on top.
 *
 *   hidden  tracker-sized (<= 2px) or display:none / visibility:hidden: invisible
 *   sized   width and height declared: that box (aspect-ratio set inline)
 *   partial one dimension declared: the other follows a 2:1 ratio
 *   box     nothing declared: a default 160x80 box
 *   min     a declared dimension <= 16px (spacer/icon): space kept, no chrome
 *   deco    alt="" and nothing declared: collapses to a 1px dot, as a spacer would
 */
export const BLOCKED_IMAGE_STYLE =
  `${PH_ANY}{display:inline-block;box-sizing:border-box;max-width:100%;overflow:hidden;vertical-align:middle;` +
  'border:1px dashed var(--mw-color-border,rgba(127,127,127,.5));border-radius:4px;' +
  'background-color:var(--mw-color-surface,#f3f4f6);color:var(--mw-color-text-dim,#6b7280);' +
  'font:12px/18px ui-sans-serif,system-ui,sans-serif;padding:2px 6px;text-align:start;unicode-bidi:plaintext}' +
  `img[data-mw-ph]:not([src]):not([alt]),img[data-mw-ph]:not([src])[alt=""],${PH_GIF}:not([alt]),${PH_GIF}[alt=""]{background-image:${PH_CROSS}}` +
  `${phKind('box')}{min-width:160px;min-height:80px;height:auto}` +
  `${phKind('partial')}{aspect-ratio:2/1;height:auto;min-width:24px;min-height:12px}` +
  `${phKind('min')}{border:0;background:var(--mw-color-surface,rgba(127,127,127,.15));padding:0;font-size:0;min-width:2px;min-height:2px}` +
  `${phKind('deco')}{border:0;background:none;padding:0;width:1px;height:1px;min-width:0}` +
  `${phKind('hidden')}{visibility:hidden;border:0;background:none;padding:0}`;

type Dim = { px: number } | { pct: number } | null;
function parseDim(value: string | null | undefined): Dim {
  const m = /^\s*(\d+(?:\.\d+)?)\s*(px|%)?\s*(?:!important)?\s*$/i.exec(value ?? '');
  if (!m) return null;
  const n = Number(m[1]);
  return m[2] === '%' ? { pct: n } : { px: n };
}
function styleProp(style: string, name: string): string | null {
  let found: string | null = null;
  for (const part of readerStyleDeclarations(style)) {
    const at = part.indexOf(':');
    if (at > 0 && part.slice(0, at).trim().toLowerCase() === name) found = part.slice(at + 1).trim();
  }
  return found;
}

/**
 * Mark every `<img>` that cannot show a picture (no `src`/`srcset`, the blocked
 * transparent GIF, an unresolved `mw-rc:`/`cid:` reference; every image when
 * `all`) as a placeholder: drop `src`/`srcset`, set `data-mw-ph` (kind, see
 * BLOCKED_IMAGE_STYLE) and `data-mw-blocked-src` (the sanitizer reference such as
 * `mw-rc:3`, else empty), and `role="img"` + `aria-label` from a non-empty `alt`
 * (`alt=""` becomes `aria-hidden`). Attributes only: no element is added, moved
 * or removed. Run on ALREADY SANITIZED html; alt is never parsed as markup.
 */
export function withBlockedImagePlaceholders(html: string, all = false): string {
  if (!/<img\b/i.test(html) || typeof DOMParser === 'undefined') return html;
  let doc: Document;
  try { doc = new DOMParser().parseFromString(html, 'text/html'); } catch { return html; }
  let changed = false;
  for (const img of Array.from(doc.body.querySelectorAll('img'))) {
    const src = (img.getAttribute('src') ?? '').trim();
    const srcset = (img.getAttribute('srcset') ?? '').trim();
    const blocked = all || src.startsWith(BLOCKED_GIF_PREFIX) || /^(?:mw-rc:|cid:)/i.test(src) || (src === '' && (srcset === '' || /mw-rc:|cid:/i.test(srcset)));
    if (!blocked || img.hasAttribute('data-mw-ph')) continue;
    changed = true;
    const style = img.getAttribute('style') ?? '';
    const w = parseDim(styleProp(style, 'width') ?? img.getAttribute('width'));
    const h = parseDim(styleProp(style, 'height') ?? img.getAttribute('height'));
    const alt = img.getAttribute('alt');
    const hiddenStyle = /^none$/i.test(styleProp(style, 'display') ?? '') || /^(?:hidden|collapse)$/i.test(styleProp(style, 'visibility') ?? '');
    const wpx = w && 'px' in w ? w.px : null, hpx = h && 'px' in h ? h.px : null;
    let kind: string;
    if (hiddenStyle || (wpx !== null && wpx <= 2 && (hpx === null || hpx <= 2)) || (wpx === null && hpx !== null && hpx <= 2)) kind = 'hidden';
    else if ((wpx !== null && wpx <= 16) || (hpx !== null && hpx <= 16)) kind = 'min';
    else if (w && h) kind = 'sized';
    else if (w || h) kind = 'partial';
    else kind = alt === '' ? 'deco' : 'box';
    if (kind === 'sized' && wpx !== null && hpx !== null) {
      const ratio = `aspect-ratio:${wpx}/${hpx}`;
      img.setAttribute('style', style.trim() === '' ? ratio : `${style.replace(/;\s*$/, '')};${ratio}`);
    }
    img.setAttribute('data-mw-blocked-src', /^mw-rc:\d+$/.test(src) ? src : '');
    img.setAttribute('data-mw-ph', kind);
    img.removeAttribute('src');
    img.removeAttribute('srcset');
    if (alt !== null && alt.trim() !== '') {
      img.setAttribute('role', 'img');
      img.setAttribute('aria-label', alt);
      if (!img.hasAttribute('dir')) img.setAttribute('dir', 'auto');
    } else if (alt !== null) img.setAttribute('aria-hidden', 'true');
  }
  return changed ? doc.body.innerHTML : html;
}

/** Remove media but keep images as source-less placeholders, so the no-media CSP
 * has nothing to reject and the layout the sender declared is preserved. */
function withoutBodyMedia(html: string): string {
  // Images stay as source-less attribute-only placeholders (BLOCKED_IMAGE_STYLE).
  const doc = new DOMParser().parseFromString(withBlockedImagePlaceholders(html, true), 'text/html');
  for (const media of doc.body.querySelectorAll('video,audio,source,track')) media.remove();
  for (const element of doc.body.querySelectorAll('[background],[style]')) {
    element.removeAttribute('background');
    const style = element.getAttribute('style');
    if (style === null) continue;
    const kept = readerStyleDeclarations(style).filter(part => !/url\s*\(|image-set\s*\(/i.test(part));
    if (kept.length) element.setAttribute('style', kept.join(';'));
    else element.removeAttribute('style');
  }
  return doc.body.innerHTML;
}

/** Last line of defence for the script-free body frame: a navigation attribute naming the local machine or an
 *  executable scheme is dropped, so no click or load can make the browser try `file:`, `javascript:`, `vbscript:` or
 *  a `data:` document (the sanitizer already removes them; Firefox reports each attempt as "may not load or link
 *  to file:///"). `data:image/*` stays valid for `src`. The frame has no script, so there is nothing to toast from. */
const UNSAFE_NAV = /^(?:file|filesystem|javascript|vbscript|data|blob|view-source|jar|resource|chrome|moz-extension|ms-help)$/i;
export function withoutUnsafeLinks(html: string): string {
  return html.replace(/(\s(?:href|xlink:href|action|formaction|src|poster|background|data)\s*=\s*)("([^"]*)"|'([^']*)'|([^\s>"']+))/gi, (whole, lead: string, _v: string, dq?: string, sq?: string, bare?: string) => {
    const value = (dq ?? sq ?? bare ?? '').replace(/&(?:#x0*([0-9a-f]+)|#0*(\d+));?/gi, (_m, h?: string, d?: string) => String.fromCodePoint(h !== undefined ? parseInt(h, 16) : parseInt(d!, 10)));
    // eslint-disable-next-line no-control-regex
    const compact = value.replace(/[\u0000-\u0020\u007f]+/g, '');
    const scheme = /^([a-z][a-z0-9+.-]*):/i.exec(compact)?.[1];
    if (scheme === undefined || !UNSAFE_NAV.test(scheme)) return whole;
    if (/^data$/i.test(scheme) && /^\s*(?:src|poster)\s*=/i.test(lead.trimStart()) && /^data:image\/(?:png|jpe?g|gif|webp);base64,/i.test(compact)) return whole;
    if (/^blob$/i.test(scheme)) return whole;
    return '';
  });
}

/** The colour mode, theme variables and tooltip theme live in a second, url-free `<style>` so a colour-mode change
 *  is one text patch of that element (the first holds data: URLs, which the patch channel refuses). */
/** Build the message-body `srcdoc` for a mode. In `plain-text` the content is
 *  rendered as escaped text (no HTML); otherwise the (already-sanitized) HTML is
 *  inlined. Either way a per-document CSP `<meta>` pins the frame. */
export function bodyFrameDoc(
  mode: SecurityMode,
  content: { html?: string | null; text?: string | null },
  opts: { themeVars?: string; direct?: boolean; serverBase?: string; appearance?: ReaderAppearance; tips?: FrameTips } = {},
): string {
  const styleVars = opts.themeVars !== undefined && opts.themeVars.length > 0 ? opts.themeVars : '';
  const rawInner =
    mode === 'plain-text'
      ? `<pre>${escapeHtml(content.text ?? '')}</pre>`
      : mode === 'sanitized-no-media'
        ? withoutBodyMedia(withoutImageTitles(opts.appearance?.colorMode === 'adaptive' ? adaptReaderColors(content.html ?? '') : content.html ?? ''))
        : withBlockedImagePlaceholders(withImageLoadingHints(withoutImageTitles(opts.appearance?.colorMode === 'adaptive' ? adaptReaderColors(content.html ?? '') : content.html ?? '')));
  const inner = mode === 'plain-text' ? rawInner : withoutUnsafeLinks(rawInner);
  // The themed image tooltip is drawn by the frame handler, so only HTML modes get it.
  const tip = mode === 'plain-text' ? { css: '', bodyAttrs: '' } : frameTipParts(opts.tips);
  return (
    '<!doctype html><html><head><meta charset="utf-8">' +
    `<meta http-equiv="Content-Security-Policy" content="${bodyCsp(mode, opts.direct === true, opts.serverBase)}">` +
    `<style>${BODY_STYLE}${mode === 'full-sanitized' ? BROKEN_IMAGE_STYLE + IMAGE_LOADING_STYLE : ''}${mode === 'plain-text' ? '' : BLOCKED_IMAGE_STYLE}</style><style>${styleVars}${opts.appearance ? readerAppearanceCss(opts.appearance) : ''}${tip.css}</style></head><body${tip.bodyAttrs}>${inner}</body></html>`
  );
}

/** Read an already-fetched `blob:` object URL back into a `data:` URL. */
export async function blobUrlToDataUrl(blobUrl: string, declaredMime?: string): Promise<string> {
  const blob = await (await apiFetch(blobUrl)).blob();
  return await new Promise<string>((resolve, reject) => {
    const reader = new FileReader();
    reader.onload = () => resolve(reader.result as string);
    reader.onerror = () => reject(reader.error ?? new Error('failed to read attachment'));
    // Downloads stay octet-stream; only the image renderer opts into SVG bytes.
    const image = declaredMime?.toLowerCase().startsWith('image/') ? new Blob([blob], { type: declaredMime.split(';')[0]!.toLowerCase() }) : blob;
    reader.readAsDataURL(image);
  });
}

/** Read a `blob:` object URL as decoded text (for the text viewer). */
export async function blobUrlToText(blobUrl: string): Promise<string> {
  return await (await apiFetch(blobUrl)).text();
}
