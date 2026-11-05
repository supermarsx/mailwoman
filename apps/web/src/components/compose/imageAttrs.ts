// What the composer keeps about a picture besides where it comes from: its
// size, alignment, title, border and corner radius. Pure and DOM-light, so the
// schema (richtext.ts), the editor's node view and the Picture ribbon tab all
// agree on one validated form. Everything is bounded and URL-free: a value that
// is not in range is dropped, never passed through.

export type ImageAlign = 'left' | 'center' | 'right';
export const IMAGE_ALIGNS: readonly ImageAlign[] = ['left', 'center', 'right'];
export const IMAGE_SIZE_MAX = 4000;
export const IMAGE_BORDERS = [0, 1, 2, 4] as const;
export const IMAGE_RADII = [0, 4, 8, 16, 9999] as const;
export const IMAGE_BORDER_COLOR = '#808080';

export interface ImageLook {
  /** Pixels, or null for the picture's own size. */
  width: number | null;
  height: number | null;
  /** null = inline with the text. */
  align: ImageAlign | null;
  title: string;
  /** Border width in px; 0 = none. */
  border: number;
  /** Corner radius in px; 9999 = a circle or pill. */
  radius: number;
}

export const IMAGE_LOOK_DEFAULTS: ImageLook = { width: null, height: null, align: null, title: '', border: 0, radius: 0 };

/** The schema's attribute specs for these (spread into the image node). */
export const imageLookAttrs = {
  width: { default: null },
  height: { default: null },
  align: { default: null },
  title: { default: '' },
  border: { default: 0 },
  radius: { default: 0 },
};

export function cleanSize(value: unknown): number | null {
  const n = typeof value === 'string' ? Number(value.replace(/px$/i, '')) : value;
  return typeof n === 'number' && Number.isInteger(n) && n >= 1 && n <= IMAGE_SIZE_MAX ? n : null;
}
const cleanAlign = (value: unknown): ImageAlign | null => (typeof value === 'string' && (IMAGE_ALIGNS as readonly string[]).includes(value) ? (value as ImageAlign) : null);
const cleanBorder = (value: unknown): number => (typeof value === 'number' && (IMAGE_BORDERS as readonly number[]).includes(value) ? value : 0);
const cleanRadius = (value: unknown): number => (typeof value === 'number' && (IMAGE_RADII as readonly number[]).includes(value) ? value : 0);
export const cleanTitle = (value: unknown): string => (typeof value === 'string' ? value.replace(/[\r\n]+/g, ' ').slice(0, 200) : '');

/** Clamp any attribute bag to the form the schema keeps. */
export function cleanImageLook(raw: Record<string, unknown>): ImageLook {
  return {
    width: cleanSize(raw['width']),
    height: cleanSize(raw['height']),
    align: cleanAlign(raw['align']),
    title: cleanTitle(raw['title']),
    border: cleanBorder(raw['border']),
    radius: cleanRadius(raw['radius']),
  };
}

/** The inline style declarations of a picture, as [property, value] pairs. */
export function imageStyle(look: ImageLook): [string, string][] {
  const out: [string, string][] = [];
  if (look.width !== null) out.push(['width', `${look.width}px`]);
  if (look.height !== null) out.push(['height', `${look.height}px`]);
  else if (look.width !== null) out.push(['height', 'auto']);
  if (look.align !== null) {
    out.push(['display', 'block']);
    out.push(['margin-left', look.align === 'left' ? '0' : 'auto']);
    out.push(['margin-right', look.align === 'right' ? '0' : 'auto']);
  }
  if (look.border > 0) out.push(['border', `${look.border}px solid ${IMAGE_BORDER_COLOR}`]);
  if (look.radius > 0) out.push(['border-radius', look.radius >= 9999 ? '50%' : `${look.radius}px`]);
  return out;
}

/** The same declarations as one `style` attribute value (for serialised HTML). */
export function imageStyleText(look: ImageLook): string {
  return imageStyle(look).map(([p, v]) => `${p}:${v}`).join(';');
}

/** Read a picture's look from parsed markup: its attributes and style, bounded. */
export function readImageLook(dom: HTMLElement): ImageLook {
  const style = dom.style;
  const width = cleanSize(style.width) ?? cleanSize(dom.getAttribute('width'));
  const height = cleanSize(style.height) ?? cleanSize(dom.getAttribute('height'));
  let align: ImageAlign | null = null;
  if (style.display === 'block' || style.float !== '') {
    if (style.float === 'right' || (style.marginLeft === 'auto' && style.marginRight !== 'auto')) align = 'right';
    else if (style.float === 'left' || (style.marginRight === 'auto' && style.marginLeft !== 'auto')) align = 'left';
    else if (style.marginLeft === 'auto' && style.marginRight === 'auto') align = 'center';
  }
  const borderWidth = Number.parseInt(style.borderTopWidth || style.borderWidth || '0', 10);
  const radiusText = style.borderRadius;
  const radius = radiusText === '50%' ? 9999 : Number.parseInt(radiusText || '0', 10);
  return cleanImageLook({
    width,
    height,
    align,
    title: dom.getAttribute('title') ?? '',
    border: Number.isNaN(borderWidth) ? 0 : borderWidth,
    radius: Number.isNaN(radius) ? 0 : radius,
  });
}

/** A size scaled by `factor` (a preset such as 0.25), keeping the aspect ratio of `natural`. */
export function scaledSize(natural: { width: number; height: number }, factor: number): { width: number; height: number } {
  const clamp = (n: number): number => Math.max(1, Math.min(IMAGE_SIZE_MAX, Math.round(n)));
  return { width: clamp(natural.width * factor), height: clamp(natural.height * factor) };
}

/** The other side of a size when the aspect ratio is locked. */
export function lockedSide(changed: 'width' | 'height', value: number, ratio: number): number {
  const other = changed === 'width' ? value / ratio : value * ratio;
  return Math.max(1, Math.min(IMAGE_SIZE_MAX, Math.round(other)));
}
