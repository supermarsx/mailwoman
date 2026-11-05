// Commands and pixel work behind the Picture ribbon tab (rich-text editor).
//
// The attribute commands are plain ProseMirror commands on the selected image
// node. Rotate, flip and crop redraw the picture on a canvas and store the
// result as a new image that replaces the old one (a new upload for a draft
// image, a new `data:` URL for an embedded one), so what is sent is exactly
// what was edited. The canvas and the storage are injected (`ImageIO`), so the
// logic is tested without a browser.

import { NodeSelection, type Command, type EditorState, type Transaction } from 'prosemirror-state';
import type { Node as PMNode } from 'prosemirror-model';
import { cleanImageLook, cleanTitle, type ImageLook } from './imageAttrs.ts';

export interface SelectedImage { node: PMNode; pos: number }

/** The image node the selection is on, or null. */
export function selectedImage(state: EditorState): SelectedImage | null {
  const selection = state.selection;
  if (selection instanceof NodeSelection && selection.node.type.name === 'image') return { node: selection.node, pos: selection.from };
  return null;
}

/** Changing a node's attributes drops a node selection; put it back so the next edit still finds the picture. */
function keepSelected(tr: Transaction, pos: number): Transaction {
  return tr.setSelection(NodeSelection.create(tr.doc, pos));
}

/** Change a picture's size, alignment, title, border or radius. */
export function setImageLook(patch: Partial<ImageLook>): Command {
  return (state, dispatch) => {
    const found = selectedImage(state);
    if (!found) return false;
    const next = cleanImageLook({ ...found.node.attrs, ...patch });
    dispatch?.(keepSelected(state.tr.setNodeMarkup(found.pos, undefined, { ...found.node.attrs, ...next }), found.pos).scrollIntoView());
    return true;
  };
}

export function setImageAlt(alt: string): Command {
  return (state, dispatch) => {
    const found = selectedImage(state);
    if (!found) return false;
    dispatch?.(keepSelected(state.tr.setNodeMarkup(found.pos, undefined, { ...found.node.attrs, alt: cleanTitle(alt).slice(0, 200) }), found.pos));
    return true;
  };
}

/** Point the picture at a new stored image (a replacement or an edited copy). */
export function setImageSource(source: { cid?: string; src?: string }, resetSize: boolean): Command {
  return (state, dispatch) => {
    const found = selectedImage(state);
    if (!found) return false;
    const attrs = { ...found.node.attrs, cid: source.cid ?? '', src: source.src ?? null, ...(resetSize ? { width: null, height: null } : {}) };
    dispatch?.(keepSelected(state.tr.setNodeMarkup(found.pos, undefined, attrs), found.pos));
    return true;
  };
}

export const removeImage: Command = (state, dispatch) => {
  const found = selectedImage(state);
  if (!found) return false;
  dispatch?.(state.tr.delete(found.pos, found.pos + found.node.nodeSize).scrollIntoView());
  return true;
};

// ── Transforms ─────────────────────────────────────────────────────────────

export type ImageOp =
  | { kind: 'rotate'; dir: 'cw' | 'ccw' }
  | { kind: 'flip'; axis: 'h' | 'v' }
  /** Fractions of each side to remove, each 0 to 0.9, leaving at least 10% across and down. */
  | { kind: 'crop'; left: number; top: number; right: number; bottom: number };

export interface TransformPlan {
  /** The canvas size to draw into. */
  width: number;
  height: number;
  /** The part of the source to read, in source pixels. */
  source: { x: number; y: number; width: number; height: number };
  /** The canvas transform `[a, b, c, d, e, f]` applied before drawing. */
  matrix: [number, number, number, number, number, number];
}

const fraction = (n: number): number => (Number.isFinite(n) ? Math.min(0.9, Math.max(0, n)) : 0);

/** Where each source pixel goes for `op` on a picture of `width` x `height`. */
export function planTransform(op: ImageOp, width: number, height: number): TransformPlan {
  const whole = { x: 0, y: 0, width, height };
  switch (op.kind) {
    case 'rotate':
      return op.dir === 'cw'
        ? { width: height, height: width, source: whole, matrix: [0, 1, -1, 0, height, 0] }
        : { width: height, height: width, source: whole, matrix: [0, -1, 1, 0, 0, width] };
    case 'flip':
      return op.axis === 'h'
        ? { width, height, source: whole, matrix: [-1, 0, 0, 1, width, 0] }
        : { width, height, source: whole, matrix: [1, 0, 0, -1, 0, height] };
    case 'crop': {
      let left = fraction(op.left), right = fraction(op.right), top = fraction(op.top), bottom = fraction(op.bottom);
      if (left + right > 0.9) { const k = 0.9 / (left + right); left *= k; right *= k; }
      if (top + bottom > 0.9) { const k = 0.9 / (top + bottom); top *= k; bottom *= k; }
      const x = Math.round(left * width), y = Math.round(top * height);
      const w = Math.max(1, width - x - Math.round(right * width)), h = Math.max(1, height - y - Math.round(bottom * height));
      return { width: w, height: h, source: { x, y, width: w, height: h }, matrix: [1, 0, 0, 1, 0, 0] };
    }
  }
}

/** The capabilities `transformImage` needs from the page (injected in tests). */
export interface ImageIO {
  /** A URL the picture can be read from, or null when it has no local copy. */
  sourceUrl: (node: PMNode) => string | null;
  /** Draw `op` on the picture at `url` and return the new image file's bytes. */
  render: (url: string, op: ImageOp) => Promise<Blob>;
  /** Keep a new image of this draft; null when it was refused (too large, wrong type). */
  store: (file: File) => Promise<{ cid?: string; src?: string } | null>;
}

export type TransformResult = 'done' | 'no-image' | 'no-source' | 'failed';

export interface ViewLike { state: EditorState; dispatch: (tr: Transaction) => void }

/** Redraw the selected picture with `op` and swap it in. */
export async function transformImage(view: ViewLike, op: ImageOp, io: ImageIO): Promise<TransformResult> {
  const found = selectedImage(view.state);
  if (!found) return 'no-image';
  const url = io.sourceUrl(found.node);
  if (url === null || url === '') return 'no-source';
  try {
    const blob = await io.render(url, op);
    const type = blob.type === 'image/jpeg' || blob.type === 'image/webp' ? blob.type : 'image/png';
    const stored = await io.store(new File([blob], `edited.${type === 'image/jpeg' ? 'jpg' : type === 'image/webp' ? 'webp' : 'png'}`, { type }));
    if (stored === null) return 'failed';
    // The selection may have moved while the canvas worked: re-find the same picture.
    const now = selectedImage(view.state);
    if (!now || now.pos !== found.pos) return 'failed';
    const command = setImageSource(stored, op.kind !== 'flip');
    command(view.state, (tr) => view.dispatch(tr));
    return 'done';
  } catch {
    return 'failed';
  }
}

/** The real canvas renderer. Reads from a blob: or data: URL of this page only. */
export async function renderWithCanvas(url: string, op: ImageOp, type = 'image/png'): Promise<Blob> {
  const image = await new Promise<HTMLImageElement>((resolve, reject) => {
    const el = new Image();
    el.onload = () => resolve(el);
    el.onerror = () => reject(new Error('image load failed'));
    el.src = url;
  });
  const plan = planTransform(op, image.naturalWidth, image.naturalHeight);
  const canvas = document.createElement('canvas');
  canvas.width = plan.width;
  canvas.height = plan.height;
  const context = canvas.getContext('2d');
  if (!context) throw new Error('no 2d context');
  context.setTransform(...plan.matrix);
  const { x, y, width, height } = plan.source;
  // The rotate and flip matrices map the whole source; a crop draws its window at the origin.
  context.drawImage(image, x, y, width, height, 0, 0, width, height);
  const wanted = type === 'image/jpeg' || type === 'image/webp' ? type : 'image/png';
  return await new Promise<Blob>((resolve, reject) => canvas.toBlob((blob) => (blob ? resolve(blob) : reject(new Error('encode failed'))), wanted, 0.92));
}
