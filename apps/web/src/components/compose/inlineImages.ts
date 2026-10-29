// Images placed in the composer body (t29-e12).
//
// An image is uploaded to the account's JMAP upload endpoint like an attachment
// and given a Content-ID of its own. The body refers to it as
// `<img src="cid:<id>">` (the rich-text schema keeps only ids this draft owns,
// `richtext.ts`), and the send carries it on the `Email/set` create as
// `{ blobId, cid, disposition: 'inline' }`, which the engine builds into a
// `multipart/related` part. While composing, the editor shows a local object URL
// for it instead of the `cid:` reference.

import type { Id } from '../../api/jmap-types.ts';
import type { StoredInlineImage } from './drafts-store.ts';

/** An image of this draft: what is sent, plus the editor's local preview. */
export interface InlineImage extends StoredInlineImage {
  /** An object URL for the editor to show, or `null` until one is made. */
  previewUrl: string | null;
}

/** The inline entry of an `Email/set` create's `attachments`. */
export interface InlineAttachment {
  blobId: Id;
  cid: string;
  disposition: 'inline';
  type: string;
  name: string;
}

/** Image types that can be placed in the body. Raster formats only: an SVG can
 *  carry script, and the engine accepts nothing but `image/*`. */
export const INLINE_IMAGE_TYPES: ReadonlySet<string> = new Set([
  'image/png',
  'image/jpeg',
  'image/gif',
  'image/webp',
]);

/** A new Content-ID: a random UUID at a fixed right-hand side, which is atext
 *  and `@` only, as the engine requires. */
export function newCid(random: () => string = () => crypto.randomUUID()): string {
  return `${random()}@mailwoman`;
}

/** Why `file` cannot be placed in the body, or `null` when it can. */
export function inlineRefusal(file: { type: string; size: number }, maxBytes: number): 'type' | 'size' | null {
  if (!INLINE_IMAGE_TYPES.has(file.type)) return 'type';
  if (file.size > maxBytes) return 'size';
  return null;
}

/** The image files in a paste or a drop, in order. */
export function imageFiles(data: DataTransfer | null): File[] {
  if (data === null) return [];
  return Array.from(data.files).filter((f) => f.type.startsWith('image/'));
}

/** The Content-IDs a serialised body refers to (`src="cid:<id>"`), in order of
 *  first use. Here rather than in `richtext.ts` so the send path does not pull
 *  the editor's chunk. */
export function referencedCids(html: string): string[] {
  const out: string[] = [];
  for (const m of html.matchAll(/src="cid:([^"]+)"/gi)) {
    const cid = m[1]!;
    if (!out.includes(cid)) out.push(cid);
  }
  return out;
}

/** The images the send carries: those the body still refers to, in the order
 *  the body uses them. An image removed from the text is not sent (the engine
 *  refuses an inline part the HTML does not name). */
export function inlineAttachments(images: readonly StoredInlineImage[], referenced: readonly string[]): InlineAttachment[] {
  const out: InlineAttachment[] = [];
  for (const cid of referenced) {
    const img = images.find((i) => i.cid === cid);
    if (img === undefined) continue;
    out.push({ blobId: img.blobId, cid: img.cid, disposition: 'inline', type: img.type, name: img.name });
  }
  return out;
}

/** The part of an image a draft stores (no preview URL: it does not outlive
 *  the page). */
export function storedImage(img: InlineImage): StoredInlineImage {
  return { cid: img.cid, blobId: img.blobId, type: img.type, name: img.name, size: img.size };
}

/** The `src="data:image/...;base64,..."` references of a serialised signature
 *  (t49: a signature keeps its images embedded, `richtext.ts` EMBEDDED_IMAGE),
 *  each once, in order of first use. */
export function embeddedImageSources(html: string): string[] {
  const out: string[] = [];
  for (const m of html.matchAll(/src="(data:image\/(?:png|jpeg|gif|webp);base64,[A-Za-z0-9+/]+={0,2})"/g)) {
    const src = m[1]!;
    if (!out.includes(src)) out.push(src);
  }
  return out;
}

/** An embedded image as a file the composer can upload, or `null` when it is
 *  not a raster `data:` URL. No `fetch`: the page's connect-src does not admit
 *  `data:`. */
export function embeddedImageFile(src: string, name: string): File | null {
  const m = /^data:(image\/(?:png|jpeg|gif|webp));base64,([A-Za-z0-9+/]+={0,2})$/.exec(src);
  if (m === null) return null;
  let binary: string;
  try {
    binary = atob(m[2]!);
  } catch {
    return null;
  }
  const bytes = new Uint8Array(binary.length);
  for (let i = 0; i < binary.length; i++) bytes[i] = binary.charCodeAt(i);
  return new File([bytes], name, { type: m[1]! });
}

/** A signature's HTML for a draft: every embedded image uploaded through
 *  `upload` and referenced by the `cid:` it returns. An image that cannot be
 *  uploaded keeps its `data:` source, which the composer's schema drops. */
export async function embedSignatureImages(
  html: string,
  upload: (files: File[]) => Promise<{ cid: string }[]>,
): Promise<string> {
  let out = html;
  let index = 0;
  for (const src of embeddedImageSources(html)) {
    const type = /^data:image\/([a-z]+);/.exec(src)?.[1] ?? 'png';
    const file = embeddedImageFile(src, `signature-${++index}.${type === 'jpeg' ? 'jpg' : type}`);
    if (file === null) continue;
    const placed = await upload([file]);
    const cid = placed[0]?.cid;
    if (cid !== undefined) out = out.split(`src="${src}"`).join(`src="cid:${cid}"`);
  }
  return out;
}
