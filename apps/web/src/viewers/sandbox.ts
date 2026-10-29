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
import { adaptReaderColors, readerAppearanceCss, type ReaderAppearance } from './readerAppearance.ts';

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
    `<style>${FRAME_STYLE}</style></head><body>${body}</body></html>`
  );
}

/** `srcdoc` for an image/audio/video frame; `dataUrl` is a self-contained `data:`. */
export function mediaFrameDoc(kind: 'image' | 'audio' | 'video', dataUrl: string): string {
  const dir = kind === 'image' ? 'img-src' : 'media-src';
  const el =
    kind === 'image'
      ? `<img alt="attachment" src="${dataUrl}">`
      : kind === 'audio'
        ? `<audio controls src="${dataUrl}"></audio>`
        : `<video controls playsinline src="${dataUrl}"></video>`;
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

/** Minimal readable body styling (distinct from the media-centering FRAME_STYLE).
 *  The opaque-origin body frame can't inherit the parent's theme vars, so e8
 *  passes the stable `--mw-*` block via `opts.themeVars`. */
const BODY_STYLE =
  'html,body{margin:0;background:transparent}' +
  'body{padding:12px;font:14px/1.6 ui-sans-serif,system-ui,sans-serif;word-break:break-word;' +
  'color:var(--mw-color-text,#1c1e21)}' +
  // `max-width:100%;height:auto` is deliberately kept: it is the pairing that
  // makes an intrinsic `width`/`height` USABLE. The UA maps the two attributes
  // to `aspect-ratio`, and `height:auto` lets the box shrink along that ratio
  // rather than being pinned to the attribute — so a sized image reserves its
  // space before it decodes. The attributes themselves come from
  // `withImageLoadingHints` below (`loading`/`decoding`) and from the sender's
  // own `width`/`height`, which the sanitizer's img allow-list keeps.
  'img,video{max-width:100%;height:auto}' +
  BROKEN_IMAGE_STYLE +
  'pre{white-space:pre-wrap;word-break:break-word;margin:0;' +
  'font:13px/1.5 ui-monospace,SFMono-Regular,Menlo,monospace}';

/**
 * Add `loading="lazy"` + `decoding="async"` to every `<img>` in an ALREADY
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
  for (const img of Array.from(imgs)) {
    if (!img.hasAttribute('loading')) img.setAttribute('loading', 'lazy');
    if (!img.hasAttribute('decoding')) img.setAttribute('decoding', 'async');
  }
  return doc.body.innerHTML;
}

/** Build the message-body `srcdoc` for a mode. In `plain-text` the content is
 *  rendered as escaped text (no HTML); otherwise the (already-sanitized) HTML is
 *  inlined. Either way a per-document CSP `<meta>` pins the frame. */
export function bodyFrameDoc(
  mode: SecurityMode,
  content: { html?: string | null; text?: string | null },
  opts: { themeVars?: string; direct?: boolean; serverBase?: string; appearance?: ReaderAppearance } = {},
): string {
  const styleVars = opts.themeVars !== undefined && opts.themeVars.length > 0 ? opts.themeVars : '';
  const inner =
    mode === 'plain-text'
      ? `<pre>${escapeHtml(content.text ?? '')}</pre>`
      : withImageLoadingHints(opts.appearance?.colorMode === 'adaptive' ? adaptReaderColors(content.html ?? '') : content.html ?? '');
  return (
    '<!doctype html><html><head><meta charset="utf-8">' +
    `<meta http-equiv="Content-Security-Policy" content="${bodyCsp(mode, opts.direct === true, opts.serverBase)}">` +
    `<style>${BODY_STYLE}${styleVars}${opts.appearance ? readerAppearanceCss(opts.appearance) : ''}</style></head><body>${inner}</body></html>`
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
    const image = declaredMime?.split(';')[0]?.trim().toLowerCase() === 'image/svg+xml' ? new Blob([blob], { type: 'image/svg+xml' }) : blob;
    reader.readAsDataURL(image);
  });
}

/** Read a `blob:` object URL as decoded text (for the text viewer). */
export async function blobUrlToText(blobUrl: string): Promise<string> {
  return await (await apiFetch(blobUrl)).text();
}
