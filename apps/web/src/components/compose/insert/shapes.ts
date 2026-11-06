// Shapes for the Insert tab: an SVG per shape, rasterised to a PNG at 2x for
// mail clients (many block SVG). The SVG source is kept in a data attribute on
// the image so the shape can be edited again; the attribute is stripped on send.

export type ShapeId = 'rectangle' | 'rounded' | 'ellipse' | 'line' | 'arrow' | 'doubleArrow' | 'triangle' | 'star' | 'bubble' | 'check' | 'cross' | 'heart' | 'chevrons' | 'callout';

export interface ShapeSpec {
  id: ShapeId;
  width: number;
  height: number;
  fill: string;
  stroke: string;
  strokeWidth: number;
}

export const SHAPE_IDS: readonly ShapeId[] = ['rectangle', 'rounded', 'ellipse', 'line', 'arrow', 'doubleArrow', 'triangle', 'star', 'bubble', 'check', 'cross', 'heart', 'chevrons', 'callout'];
export const SHAPE_SCALE = 2;
export const SHAPE_ATTR = 'data-mw-shape';
const OPEN: readonly ShapeId[] = ['line', 'arrow', 'doubleArrow', 'check', 'cross', 'chevrons'];

const COLOR = /^(#[0-9a-f]{3,8}|none|transparent|[a-z]{3,20}|rgba?\([\d\s.,%]+\))$/i;
export const safeColor = (c: string, fallback: string): string => (COLOR.test(c.trim()) ? c.trim() : fallback);

const f = (n: number): string => String(Math.round(n * 100) / 100);

/** The elements of a shape inside a w x h box (inset by half the stroke). */
function body(spec: ShapeSpec): string {
  const s = spec.strokeWidth, w = spec.width, h = spec.height;
  const x0 = s / 2, y0 = s / 2, x1 = w - s / 2, y1 = h - s / 2, cx = w / 2, cy = h / 2;
  const round = 'stroke-linecap="round" stroke-linejoin="round"';
  switch (spec.id) {
    case 'rectangle': return `<rect x="${f(x0)}" y="${f(y0)}" width="${f(x1 - x0)}" height="${f(y1 - y0)}"/>`;
    case 'rounded': return `<rect x="${f(x0)}" y="${f(y0)}" width="${f(x1 - x0)}" height="${f(y1 - y0)}" rx="${f(Math.min(w, h) / 6)}"/>`;
    case 'ellipse': return `<ellipse cx="${f(cx)}" cy="${f(cy)}" rx="${f((x1 - x0) / 2)}" ry="${f((y1 - y0) / 2)}"/>`;
    case 'line': return `<line x1="${f(x0)}" y1="${f(cy)}" x2="${f(x1)}" y2="${f(cy)}" ${round}/>`;
    case 'arrow': return `<path d="M${f(x0)} ${f(cy)} H${f(x1)} M${f(x1 - h / 3)} ${f(y0)} L${f(x1)} ${f(cy)} L${f(x1 - h / 3)} ${f(y1)}" ${round}/>`;
    case 'doubleArrow': return `<path d="M${f(x0)} ${f(cy)} H${f(x1)} M${f(x1 - h / 3)} ${f(y0)} L${f(x1)} ${f(cy)} L${f(x1 - h / 3)} ${f(y1)} M${f(x0 + h / 3)} ${f(y0)} L${f(x0)} ${f(cy)} L${f(x0 + h / 3)} ${f(y1)}" ${round}/>`;
    case 'triangle': return `<path d="M${f(cx)} ${f(y0)} L${f(x1)} ${f(y1)} H${f(x0)} Z" ${round}/>`;
    case 'star': {
      const pts: string[] = [];
      for (let i = 0; i < 10; i++) {
        const r = i % 2 === 0 ? 1 : 0.4, a = -Math.PI / 2 + (i * Math.PI) / 5;
        pts.push(`${f(cx + Math.cos(a) * r * (x1 - x0) / 2)},${f(cy + Math.sin(a) * r * (y1 - y0) / 2)}`);
      }
      return `<polygon points="${pts.join(' ')}" ${round}/>`;
    }
    case 'bubble': {
      const tail = y1 - h * 0.25;
      return `<path d="M${f(x0 + 8)} ${f(y0)} H${f(x1 - 8)} Q${f(x1)} ${f(y0)} ${f(x1)} ${f(y0 + 8)} V${f(tail)} Q${f(x1)} ${f(tail + 8)} ${f(x1 - 8)} ${f(tail + 8)} H${f(w * 0.3)} L${f(w * 0.15)} ${f(y1)} L${f(w * 0.18)} ${f(tail + 8)} H${f(x0 + 8)} Q${f(x0)} ${f(tail + 8)} ${f(x0)} ${f(tail)} V${f(y0 + 8)} Q${f(x0)} ${f(y0)} ${f(x0 + 8)} ${f(y0)} Z" ${round}/>`;
    }
    case 'check': return `<path d="M${f(w * 0.15)} ${f(h * 0.55)} L${f(w * 0.4)} ${f(h * 0.8)} L${f(w * 0.85)} ${f(h * 0.2)}" ${round}/>`;
    case 'cross': return `<path d="M${f(w * 0.2)} ${f(h * 0.2)} L${f(w * 0.8)} ${f(h * 0.8)} M${f(w * 0.8)} ${f(h * 0.2)} L${f(w * 0.2)} ${f(h * 0.8)}" ${round}/>`;
    case 'heart': return `<path d="M${f(cx)} ${f(y1)} C${f(x0 - 2)} ${f(h * 0.6)} ${f(x0)} ${f(y0)} ${f(w * 0.28)} ${f(y0 + h * 0.05)} C${f(w * 0.4)} ${f(y0 + h * 0.08)} ${f(cx)} ${f(h * 0.25)} ${f(cx)} ${f(h * 0.3)} C${f(cx)} ${f(h * 0.25)} ${f(w * 0.6)} ${f(y0 + h * 0.08)} ${f(w * 0.72)} ${f(y0 + h * 0.05)} C${f(x1)} ${f(y0)} ${f(x1 + 2)} ${f(h * 0.6)} ${f(cx)} ${f(y1)} Z" ${round}/>`;
    case 'chevrons': return `<path d="M${f(w * 0.2)} ${f(y0)} L${f(w * 0.5)} ${f(cy)} L${f(w * 0.2)} ${f(y1)} M${f(w * 0.55)} ${f(y0)} L${f(w * 0.85)} ${f(cy)} L${f(w * 0.55)} ${f(y1)}" ${round}/>`;
    case 'callout': return `<path d="M${f(x0)} ${f(y0)} H${f(x1)} V${f(y1 - h * 0.2)} H${f(w * 0.45)} L${f(w * 0.3)} ${f(y1)} V${f(y1 - h * 0.2)} H${f(x0)} Z" ${round}/>`;
  }
}

const clampSize = (n: number): number => Math.max(8, Math.min(1200, Math.round(n)));

/** The SVG document of a shape. Colours are checked, sizes clamped. */
export function shapeSvg(spec: ShapeSpec): string {
  const w = clampSize(spec.width), h = clampSize(spec.height);
  const clean: ShapeSpec = {
    ...spec, width: w, height: h,
    strokeWidth: Math.max(0, Math.min(24, spec.strokeWidth)),
    fill: safeColor(spec.fill, 'none'), stroke: safeColor(spec.stroke, '#000000'),
  };
  const fill = OPEN.includes(spec.id) ? 'none' : clean.fill;
  return `<svg xmlns="http://www.w3.org/2000/svg" width="${w}" height="${h}" viewBox="0 0 ${w} ${h}" fill="${fill}" stroke="${clean.stroke}" stroke-width="${clean.strokeWidth}">${body(clean)}</svg>`;
}

interface Canvas2D { drawImage(img: CanvasImageSource, x: number, y: number, w: number, h: number): void }
/** Browser pieces the rasteriser needs; injectable for tests. */
export interface RasterEnv {
  loadImage(svgUrl: string): Promise<CanvasImageSource>;
  createCanvas(width: number, height: number): { getContext(kind: '2d'): Canvas2D | null; toBlob(cb: (b: Blob | null) => void, type: string): void };
}

export const browserRasterEnv: RasterEnv = {
  loadImage: (url) => new Promise((resolve, reject) => { const img = new Image(); img.onload = () => resolve(img); img.onerror = () => reject(new Error('shape image failed')); img.src = url; }),
  createCanvas: (w, h) => { const c = document.createElement('canvas'); c.width = w; c.height = h; return c as unknown as ReturnType<RasterEnv['createCanvas']>; },
};

/** Rasterise a shape to a PNG at SHAPE_SCALE x. */
export async function rasteriseShape(spec: ShapeSpec, env: RasterEnv = browserRasterEnv): Promise<{ blob: Blob; svg: string; width: number; height: number }> {
  const svg = shapeSvg(spec);
  const width = clampSize(spec.width), height = clampSize(spec.height);
  const img = await env.loadImage(`data:image/svg+xml;charset=utf-8,${encodeURIComponent(svg)}`);
  const canvas = env.createCanvas(width * SHAPE_SCALE, height * SHAPE_SCALE);
  const ctx = canvas.getContext('2d');
  if (!ctx) throw new Error('canvas unavailable');
  ctx.drawImage(img, 0, 0, width * SHAPE_SCALE, height * SHAPE_SCALE);
  const blob = await new Promise<Blob>((resolve, reject) => canvas.toBlob((b) => (b ? resolve(b) : reject(new Error('png failed'))), 'image/png'));
  return { blob, svg, width, height };
}
