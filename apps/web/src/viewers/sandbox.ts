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

/** The frozen sandbox token set: no scripts, no same-origin, opaque frame. */
export const SANDBOX_TOKENS = '';

/** CSP a media frame is pinned to — only inline styles + a `data:` resource. */
function mediaCsp(resourceDir: 'img-src' | 'media-src'): string {
  return [
    "default-src 'none'",
    `${resourceDir} data:`,
    "style-src 'unsafe-inline'",
    "frame-ancestors 'none'",
  ].join('; ');
}

/** CSP a text frame is pinned to — inline styles only, zero external loads. */
const TEXT_CSP = ["default-src 'none'", "style-src 'unsafe-inline'", "frame-ancestors 'none'"].join(
  '; ',
);

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
export function bodyCsp(mode: SecurityMode = 'full-sanitized'): string {
  if (mode === 'full-sanitized') {
    return [
      "default-src 'none'",
      'img-src data: https: http:',
      'media-src data: https: http:',
      "style-src 'unsafe-inline'",
      'font-src data:',
      "frame-ancestors 'none'",
    ].join('; ');
  }
  // sanitized-no-media AND plain-text: no external/media source of any kind.
  return ["default-src 'none'", "style-src 'unsafe-inline'", "frame-ancestors 'none'"].join('; ');
}

/** Minimal readable body styling (distinct from the media-centering FRAME_STYLE).
 *  The opaque-origin body frame can't inherit the parent's theme vars, so e8
 *  passes the stable `--mw-*` block via `opts.themeVars`. */
const BODY_STYLE =
  'html,body{margin:0;background:transparent}' +
  'body{padding:12px;font:14px/1.6 ui-sans-serif,system-ui,sans-serif;word-break:break-word;' +
  'color:var(--mw-text,#1c1e21)}' +
  'img,video{max-width:100%;height:auto}' +
  'pre{white-space:pre-wrap;word-break:break-word;margin:0;' +
  'font:13px/1.5 ui-monospace,SFMono-Regular,Menlo,monospace}';

/** Build the message-body `srcdoc` for a mode. In `plain-text` the content is
 *  rendered as escaped text (no HTML); otherwise the (already-sanitized) HTML is
 *  inlined. Either way a per-document CSP `<meta>` pins the frame. */
export function bodyFrameDoc(
  mode: SecurityMode,
  content: { html?: string | null; text?: string | null },
  opts: { themeVars?: string } = {},
): string {
  const styleVars = opts.themeVars !== undefined && opts.themeVars.length > 0 ? opts.themeVars : '';
  const inner =
    mode === 'plain-text'
      ? `<pre>${escapeHtml(content.text ?? '')}</pre>`
      : (content.html ?? '');
  return (
    '<!doctype html><html><head><meta charset="utf-8">' +
    `<meta http-equiv="Content-Security-Policy" content="${bodyCsp(mode)}">` +
    `<style>${styleVars}${BODY_STYLE}</style></head><body>${inner}</body></html>`
  );
}

/** Read an already-fetched `blob:` object URL back into a `data:` URL. */
export async function blobUrlToDataUrl(blobUrl: string): Promise<string> {
  const blob = await (await fetch(blobUrl)).blob();
  return await new Promise<string>((resolve, reject) => {
    const reader = new FileReader();
    reader.onload = () => resolve(reader.result as string);
    reader.onerror = () => reject(reader.error ?? new Error('failed to read attachment'));
    reader.readAsDataURL(blob);
  });
}

/** Read a `blob:` object URL as decoded text (for the text viewer). */
export async function blobUrlToText(blobUrl: string): Promise<string> {
  return await (await fetch(blobUrl)).text();
}
