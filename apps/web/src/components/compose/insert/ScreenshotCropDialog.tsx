// Crop a captured screenshot to a rectangle before it is inserted. The rectangle
// is chosen by dragging on the picture or by the four number fields (keyboard).

import { createMemo, createSignal, onCleanup, type JSX } from 'solid-js';
import { t } from '../../../i18n/index.ts';
import { InsertDialog } from './InsertDialog.tsx';
import { cropDialogBox } from './insertDialog.css.ts';
import { loadCatalog } from '../../../i18n/catalog.ts';
// i18n: the ids this file shows live in these catalogs, which load with it.
void loadCatalog('compose-insert');

export interface CropRect { x: number; y: number; w: number; h: number }

/** Clamp a rectangle into a w x h picture, at least 1 px. */
export function clampRect(r: CropRect, width: number, height: number): CropRect {
  const x = Math.max(0, Math.min(width - 1, Math.round(r.x)));
  const y = Math.max(0, Math.min(height - 1, Math.round(r.y)));
  return { x, y, w: Math.max(1, Math.min(width - x, Math.round(r.w))), h: Math.max(1, Math.min(height - y, Math.round(r.h))) };
}

/** Crop `file` (an image) to `rect` through a canvas. */
export async function cropImage(file: File, rect: CropRect): Promise<File> {
  const bitmap = await createImageBitmap(file);
  try {
    const r = clampRect(rect, bitmap.width, bitmap.height);
    const canvas = document.createElement('canvas');
    canvas.width = r.w;
    canvas.height = r.h;
    canvas.getContext('2d')?.drawImage(bitmap, r.x, r.y, r.w, r.h, 0, 0, r.w, r.h);
    const blob = await new Promise<Blob>((ok, fail) => canvas.toBlob((b) => (b ? ok(b) : fail(new Error('crop failed'))), 'image/png'));
    return new File([blob], file.name, { type: 'image/png' });
  } finally { bitmap.close(); }
}

export function ScreenshotCropDialog(props: { file: File; returnTo: HTMLElement | null; onClose: () => void; onDone: (file: File) => void }): JSX.Element {
  const url = URL.createObjectURL(props.file);
  onCleanup(() => URL.revokeObjectURL(url));
  const [natural, setNatural] = createSignal({ w: 0, h: 0 });
  const [rect, setRect] = createSignal<CropRect>({ x: 0, y: 0, w: 1, h: 1 });
  const [busy, setBusy] = createSignal(false);
  const [error, setError] = createSignal<string | null>(null);
  let img!: HTMLImageElement;
  let start: { x: number; y: number } | null = null;
  const toNatural = (event: PointerEvent): { x: number; y: number } => {
    const box = img.getBoundingClientRect();
    const k = natural().w / Math.max(1, box.width);
    return { x: Math.max(0, Math.min(natural().w, (event.clientX - box.left) * k)), y: Math.max(0, Math.min(natural().h, (event.clientY - box.top) * k)) };
  };
  const shown = createMemo(() => {
    const n = natural(), r = rect();
    return n.w === 0 ? { left: 0, top: 0, width: 0, height: 0 } : { left: (r.x / n.w) * 100, top: (r.y / n.h) * 100, width: (r.w / n.w) * 100, height: (r.h / n.h) * 100 };
  });
  const field = (key: keyof CropRect, label: string): JSX.Element => (
    <label>{label}<input type="number" min="0" data-testid={`insert-crop-${key}`} value={rect()[key]} onInput={(e) => setRect(clampRect({ ...rect(), [key]: Number(e.currentTarget.value) || 0 }, natural().w, natural().h))} /></label>
  );
  const apply = (): void => {
    setBusy(true);
    setError(null);
    void cropImage(props.file, rect()).then(props.onDone).catch(() => setError(t('ins-crop-failed'))).finally(() => setBusy(false));
  };
  return (
    <InsertDialog title={t('ins-crop-title')} testId="insert-crop-dialog" returnTo={props.returnTo} busy={busy()} onClose={props.onClose} onSubmit={apply} submitLabel={t('ins-crop-apply')}>
      <div class={cropDialogBox}
        onPointerDown={(e) => { if (natural().w === 0) return; (e.currentTarget as HTMLElement).setPointerCapture(e.pointerId); start = toNatural(e); setRect(clampRect({ ...start, w: 1, h: 1 }, natural().w, natural().h)); }}
        onPointerMove={(e) => { if (!start) return; const p = toNatural(e); setRect(clampRect({ x: Math.min(start.x, p.x), y: Math.min(start.y, p.y), w: Math.abs(p.x - start.x), h: Math.abs(p.y - start.y) }, natural().w, natural().h)); }}
        onPointerUp={() => { start = null; }}>
        <img ref={img} src={url} alt={t('ins-crop-alt')} draggable={false} onLoad={() => { setNatural({ w: img.naturalWidth, h: img.naturalHeight }); setRect({ x: 0, y: 0, w: img.naturalWidth, h: img.naturalHeight }); }} />
        <div class="mw-crop-sel" data-testid="insert-crop-selection" style={{ left: `${shown().left}%`, top: `${shown().top}%`, width: `${shown().width}%`, height: `${shown().height}%` }} />
      </div>
      <p class="mw-insert-note">{t('ins-crop-help')}</p>
      <div class="mw-insert-row">{field('x', 'X')}{field('y', 'Y')}{field('w', t('ins-shape-width'))}{field('h', t('ins-shape-height'))}</div>
      {error() && <p class="mw-insert-warn" role="alert">{error()}</p>}
    </InsertDialog>
  );
}
