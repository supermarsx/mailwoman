import { basePath, withBase } from './basePath.ts';
import { apiFetch } from './http.ts';
import { isNativeAuth, nativeServerBase } from './serverAddress.ts';
export function remoteServerBase(): string { return isNativeAuth() ? nativeServerBase() ?? location.origin : location.origin + basePath(); }

export type RemoteMode = 'block' | 'ask' | 'proxy' | 'direct';
export interface RemotePolicy { mode: RemoteMode; trackers: boolean; cleanLinks: boolean; allowDirect: boolean; inlineParts?: boolean; remoteImageDownloads?: boolean; locked: { mode: boolean; trackers: boolean; cleanLinks: boolean } }
export interface SignedContent { images: { url: string; src: string; ref?: string }[]; parts: { blobId: string; src: string }[]; files: { url: string; src: string }[]; imageDownloads?:{url:string;src:string}[]; refused: { url?: string; blobId?: string; reason: string }[] }
export interface ReaderReferences { urls: string[]; cids: string[]; trackers: number; files: string[]; links: string[] }
export const BLOCKED_POLICY: RemotePolicy = { mode: 'block', trackers: true, cleanLinks: true, allowDirect: false, locked: { mode: true, trackers: true, cleanLinks: true } };
export const EMPTY_SIGNED: SignedContent = { images: [], parts: [], files: [], refused: [] };
const EMPTY_IMAGE = 'data:image/gif;base64,R0lGODlhAQABAIAAAAAAAP///yH5BAEAAAAALAAAAAABAAEAAAIBRAA7';
const modes: readonly string[] = ['block', 'ask', 'proxy', 'direct'];
export async function remotePolicy(): Promise<RemotePolicy> {
  const r = await apiFetch(withBase('/api/rc/policy'));
  if (!r.ok) throw new Error('Unable to load remote-content policy');
  const value = await r.json() as Partial<RemotePolicy>;
  if (!modes.includes(value.mode ?? '') || typeof value.trackers !== 'boolean' || typeof value.cleanLinks !== 'boolean' || typeof value.allowDirect !== 'boolean' || !value.locked || ['mode', 'trackers', 'cleanLinks'].some(k => typeof (value.locked as unknown as Record<string, unknown>)[k] !== 'boolean')) throw new Error('Invalid remote-content policy');
  return { ...value, mode: value.mode === 'direct' && !value.allowDirect ? 'ask' : value.mode } as RemotePolicy;
}
/** `retry: true` ("Retry images", t56) makes the server forget this account's
 *  remembered failures (timeouts etc., kept 60 s) for the `urls` signed, so the
 *  fresh links fetch again instead of answering the cached failure. */
const retryNext = new Set<string>();
/** The loading toast's Retry (t67): the next image signing for `emailId`, the
 *  one the reader makes when it reloads the body, is sent with `retry: true`. */
export function requestImageRetry(emailId: string): void { retryNext.add(emailId); }
export async function signContent(emailId: string, refs: { urls?: string[]; parts?: string[]; files?: string[];imageDownloads?:string[] }, expectedAccount?:string, opts: { retry?: boolean } = {}): Promise<SignedContent> {
  const retry = opts.retry ?? ((refs.urls?.length ?? 0) > 0 && retryNext.delete(emailId));
  const r = await apiFetch(withBase('/api/rc/sign'), { method: 'POST', headers: { 'Content-Type': 'application/json',...(expectedAccount?{'X-Mailwoman-Account':expectedAccount}:{}) }, body: JSON.stringify({ emailId, ...refs, ...(retry ? { retry: true } : {}) }) });
  if (!r.ok) throw new Error('Unable to authorize remote content');
  return await r.json() as SignedContent;
}
function documentOf(html: string): Document { return new DOMParser().parseFromString(html, 'text/html'); }
function httpUrl(raw: string, httpsOnly = false): string | null {
  try { const u = new URL(raw); return (u.protocol === 'https:' || (!httpsOnly && u.protocol === 'http:')) && !u.username && !u.password ? u.href : null; } catch { return null; }
}
function cid(raw: string): string { try { return decodeURIComponent(raw.replace(/^cid:/i, '')).replace(/^<|>$/g, ''); } catch { return ''; } }
const CID_TOKEN = /cid:[^"'()\s,<>\\]+/gi;
/** Only call on output of the Rust reader sanitizer, never a sender's raw HTML. */
export function readerReferences(html: string): ReaderReferences {
  const doc = documentOf(html);
  const urls = [...new Set([...doc.querySelectorAll('[data-mw-rc-url]')].map(e => e.getAttribute('data-mw-rc-url') ?? '').filter(v => httpUrl(v) !== null))].slice(0, 256);
  const cids = [...new Set([...doc.querySelectorAll('[src],[srcset],[style],[background]')].flatMap(e => ['src','srcset','style','background'].flatMap(attr => (e.getAttribute(attr) ?? '').match(CID_TOKEN) ?? [])).map(cid))].filter(Boolean).slice(0, 256);
  const links = [...new Set([...doc.querySelectorAll('a[href]')].map(e => httpUrl(e.getAttribute('href') ?? '')).filter((url): url is string => url !== null))].slice(0, 256);
  const files = links.filter(url => /\.(?:pdf|zip|gz|tar|7z|docx?|xlsx?|pptx?|txt|csv|rtf|odt|ods|epub|ics)$/i.test(new URL(url).pathname)).slice(0, 32);
  return { urls, cids, files, links, trackers: doc.querySelectorAll('[data-mw-tracker]').length };
}
/** Signed URLs must remain in this server's narrow token endpoints. */
export function signedSource(src: string, kind: 'i' | 'p' | 'f' | 'd', base = remoteServerBase()): string | null {
  try { const server = new URL(base); const prefix = server.pathname.replace(/\/$/, '') + `/api/rc/${kind}/`; const url = new URL(src, server); return !url.username && !url.password && url.origin === server.origin && url.pathname.startsWith(prefix) && /^[A-Za-z0-9_.-]+$/.test(url.pathname.slice(prefix.length)) && !url.search && !url.hash ? url.href : null; } catch { return null; }
}
/** Refresh only failed signed sources, retaining successful URLs and their cache entries. */
export function replaceFailedImages(html: string, images: SignedContent['images'], failedRefs: readonly string[], replacements: SignedContent['images'], base = remoteServerBase()): { html: string; imageSources: SignedContent['images'] } {
  const refs = new Set(failedRefs);
  const byUrl = new Map(replacements.map(image => [image.url, image]));
  const imageSources = images.map(image => {
    if (!image.ref || !refs.has(image.ref)) return image;
    const replacement = byUrl.get(image.url);
    if (!replacement) return image;
    const oldSrc = signedSource(image.src, 'i', base), newSrc = signedSource(replacement.src, 'i', base);
    if (!oldSrc || !newSrc) return image;
    html = html.split(oldSrc).join(newSrc);
    return replacement;
  });
  return { html, imageSources };
}
/** Replaces sanitizer placeholders only. Unapproved references remain inert. */
export function rewriteReaderContent(html: string, signed: SignedContent, policy: RemotePolicy, cidParts: Readonly<Record<string, string>>, origin = remoteServerBase()): string {
  const doc = documentOf(html);
  const imageMap = new Map(signed.images.map(v => [v.url, signedSource(v.src, 'i', origin)]));
  const partMap = new Map(signed.parts.map(v => [v.blobId, signedSource(v.src, 'p', origin)]));
  const refs = new Map<string, string>();
  for (const marker of doc.querySelectorAll('[data-mw-rc][data-mw-rc-url]')) {
    const id = marker.getAttribute('data-mw-rc') ?? '';
    const url = marker.getAttribute('data-mw-rc-url') ?? '';
    if (!/^\d+$/.test(id) || (policy.trackers && marker.hasAttribute('data-mw-tracker'))) continue;
    const src = policy.mode === 'direct' && policy.allowDirect ? httpUrl(url, true) : policy.mode === 'block' ? null : imageMap.get(url);
    if (src) refs.set(id, src);
  }
  for (const element of doc.querySelectorAll('[src],[srcset],[style],[background]')) {
    for (const attr of ['src', 'srcset', 'style', 'background']) {
      const raw = element.getAttribute(attr); if (raw === null) continue;
      if (attr === 'src' && /^cid:/i.test(raw)) { const blob = cidParts[cid(raw)]; const src = blob ? partMap.get(blob) : undefined; if (src) element.setAttribute(attr, src); else element.removeAttribute(attr); continue; }
      const withParts = raw.replace(CID_TOKEN, token => { const blob = cidParts[cid(token)]; return (blob ? partMap.get(blob) : null) ?? EMPTY_IMAGE; });
      const value = withParts.replace(/mw-rc:(\d+)/g, (_, id: string) => (refs.get(id) ?? EMPTY_IMAGE).replace(/["'()\\\s<>]/g, c => '%' + c.charCodeAt(0).toString(16).toUpperCase()));
      element.setAttribute(attr, value);
    }
  }
  for (const marker of doc.querySelectorAll('[data-mw-link-cleaned][data-mw-link-original]')) {
    const cleaned = marker.getAttribute('data-mw-link-cleaned'); const original = marker.getAttribute('data-mw-link-original');
    if (original) for (const link of doc.querySelectorAll('a[href]')) if (link.getAttribute('href') === cleaned) { link.setAttribute('title', original); if (!policy.cleanLinks && httpUrl(original)) link.setAttribute('href', original); }
  }
  return doc.body.innerHTML;
}

/** Server-side counters for one open message. `imagesLoaded`/`imagesFailed` are
 *  optional (t56 adds them): absent means the server does not report per-image
 *  results, and callers must not invent a failure count. */
export interface RemoteContentStats { count: number; limited: boolean; windowSecs: number; imagesLoaded?: number; imagesFailed?: number; failedImageRefs?: string[]; retryableImageRefs?: string[] }
const counter = (v: unknown): v is number => typeof v === 'number' && Number.isSafeInteger(v) && v >= 0;
export async function decodedTrackerStats(emailId: string): Promise<RemoteContentStats> {
  const response = await apiFetch(withBase(`/api/rc/stats?emailId=${encodeURIComponent(emailId)}`));
  if (!response.ok) throw new Error('Unable to load image diagnostics');
  const value = await response.json() as { decodedTrackersReplaced?: unknown; limited?: unknown; windowSecs?: unknown; imagesLoaded?: unknown; imagesFailed?: unknown; failedImageRefs?: unknown; retryableImageRefs?: unknown };
  if (!counter(value.decodedTrackersReplaced)) throw new Error('Invalid image diagnostics');
  if (typeof value.limited !== 'boolean' || typeof value.windowSecs !== 'number' || !Number.isSafeInteger(value.windowSecs) || value.windowSecs <= 0) throw new Error('Invalid image diagnostic window');
  const references = (v: unknown): v is string[] => Array.isArray(v) && v.length <= 256 && v.every(x => typeof x === "string" && /^[a-f0-9]{64}$/.test(x)) && new Set(v).size === v.length;
  return { ...(references(value.failedImageRefs) ? {failedImageRefs:value.failedImageRefs} : {}), ...(references(value.retryableImageRefs) ? {retryableImageRefs:value.retryableImageRefs} : {}), count: value.decodedTrackersReplaced, limited: value.limited, windowSecs: value.windowSecs, ...(counter(value.imagesLoaded) ? { imagesLoaded: value.imagesLoaded } : {}), ...(counter(value.imagesFailed) ? { imagesFailed: value.imagesFailed } : {}) };
}
