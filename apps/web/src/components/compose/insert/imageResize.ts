// Picture presets for the Insert tab: downscale through a canvas, which also drops
// EXIF (the canvas re-encode carries no metadata).

export type ResizePreset = 'original' | 'large' | 'medium' | 'small';
export const RESIZE_MAX: Record<ResizePreset, number> = { original: Infinity, large: 1600, medium: 1024, small: 480 };

/** The size after the preset's longest-side cap; never upscales. */
export function presetSize(width: number, height: number, preset: ResizePreset): { width: number; height: number; scaled: boolean } {
  const longest = Math.max(width, height);
  const cap = RESIZE_MAX[preset];
  if (longest <= cap) return { width, height, scaled: false };
  const k = cap / longest;
  return { width: Math.max(1, Math.round(width * k)), height: Math.max(1, Math.round(height * k)), scaled: true };
}

export interface ResizeEnv {
  decode(file: File): Promise<{ width: number; height: number; source: CanvasImageSource; close?: () => void }>;
  draw(source: CanvasImageSource, width: number, height: number, type: string): Promise<Blob>;
}

export const browserResizeEnv: ResizeEnv = {
  async decode(file) {
    const bmp = await createImageBitmap(file);
    return { width: bmp.width, height: bmp.height, source: bmp, close: () => bmp.close() };
  },
  draw(source, width, height, type) {
    const canvas = document.createElement('canvas');
    canvas.width = width;
    canvas.height = height;
    canvas.getContext('2d')?.drawImage(source, 0, 0, width, height);
    return new Promise((ok, fail) => canvas.toBlob((b) => (b ? ok(b) : fail(new Error('encode failed'))), type, 0.88));
  },
};

/**
 * Apply the preset and the EXIF strip. A GIF is kept as is (a canvas would flatten
 * the animation). JPEG is always re-encoded when `stripExif` is on.
 */
export async function prepareImage(file: File, o: { preset: ResizePreset; stripExif: boolean }, env: ResizeEnv = browserResizeEnv): Promise<File> {
  if (file.type === 'image/gif') return file;
  if (o.preset === 'original' && !(o.stripExif && file.type === 'image/jpeg')) return file;
  let decoded;
  try { decoded = await env.decode(file); } catch { return file; }
  try {
    const size = presetSize(decoded.width, decoded.height, o.preset);
    if (!size.scaled && !(o.stripExif && file.type === 'image/jpeg')) return file;
    const type = file.type === 'image/png' || file.type === 'image/webp' ? file.type : 'image/jpeg';
    const blob = await env.draw(decoded.source, size.width, size.height, type);
    return new File([blob], file.name, { type: blob.type || type });
  } finally {
    decoded.close?.();
  }
}
