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
