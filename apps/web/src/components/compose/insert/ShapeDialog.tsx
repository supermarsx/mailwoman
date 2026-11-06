// Shapes gallery: pick a shape, fill, outline, size; inserted as a 2x PNG with alt
// text (many mail clients block SVG). The last choices are remembered per browser.

import { createSignal, For, type JSX } from 'solid-js';
import { t } from '../../../i18n/index.ts';
import { InsertDialog } from './InsertDialog.tsx';
import { shapeGallery } from './insertDialog.css.ts';
import { rasteriseShape, shapeSvg, SHAPE_IDS, type ShapeId, type ShapeSpec } from './shapes.ts';

const SWATCHES = ['#1a56b0', '#2e7d32', '#c62828', '#ef6c00', '#6a1b9a', '#212121', '#ffffff', 'none'] as const;
const KEY = 'mw.insert.shape';

function remembered(): Partial<ShapeSpec> {
  try { return JSON.parse(localStorage.getItem(KEY) ?? '{}') as Partial<ShapeSpec>; } catch { return {}; }
}

export function ShapeDialog(props: { returnTo: HTMLElement | null; onClose: () => void; onInsert: (file: File, alt: string) => void }): JSX.Element {
  const saved = remembered();
  const [id, setId] = createSignal<ShapeId>(saved.id ?? 'rectangle');
  const [fill, setFill] = createSignal(saved.fill ?? '#1a56b0');
  const [stroke, setStroke] = createSignal(saved.stroke ?? '#212121');
  const [strokeWidth, setStrokeWidth] = createSignal(saved.strokeWidth ?? 3);
  const [width, setWidth] = createSignal(saved.width ?? 160);
  const [height, setHeight] = createSignal(saved.height ?? 100);
  const [alt, setAlt] = createSignal('');
  const [busy, setBusy] = createSignal(false);
  const [error, setError] = createSignal<string | null>(null);
  const spec = (shape: ShapeId = id()): ShapeSpec => ({ id: shape, width: width(), height: height(), fill: fill(), stroke: stroke(), strokeWidth: strokeWidth() });
  const preview = (shape: ShapeId, size?: number): string => {
    const s = spec(shape);
    return shapeSvg(size === undefined ? s : { ...s, width: size, height: size * 0.7, strokeWidth: Math.min(s.strokeWidth, 3) });
  };
  const swatches = (label: string, value: () => string, set: (v: string) => void, testId: string): JSX.Element => (
    <fieldset class="mw-insert-fieldset">
      <legend>{label}</legend>
      <div class="mw-insert-row mw-insert-swatches">
        <For each={SWATCHES}>{(c) => (
          <button type="button" aria-label={c === 'none' ? t('ins-shape-none') : c} aria-pressed={value() === c} data-testid={`${testId}-${c}`} style={{ flex: '0 0 1.6rem', width: '1.6rem', height: '1.6rem', background: c === 'none' ? 'transparent' : c, border: value() === c ? '2px solid currentColor' : '1px solid #888', 'border-radius': '4px' }} onClick={() => set(c)}>{c === 'none' ? '∅' : ''}</button>
        )}</For>
        <input type="color" aria-label={`${label} (custom)`} class="mw-insert-custom" value={/^#[0-9a-f]{6}$/i.test(value()) ? value() : '#000000'} onInput={(e) => set(e.currentTarget.value)} />
      </div>
    </fieldset>
  );
  const insert = (): void => {
    setBusy(true);
    setError(null);
    const s = spec();
    try { localStorage.setItem(KEY, JSON.stringify(s)); } catch { /* remembering is optional */ }
    void rasteriseShape(s).then(({ blob }) => {
      props.onInsert(new File([blob], `${s.id}.png`, { type: 'image/png' }), alt().trim() || t(`ins-shape-${s.id}`));
    }).catch(() => setError(t('ins-shape-failed'))).finally(() => setBusy(false));
  };
  return (
    <InsertDialog title={t('ins-shapes')} testId="insert-shape-dialog" returnTo={props.returnTo} busy={busy()} onClose={props.onClose} onSubmit={insert}>
      <div class={shapeGallery} role="group" aria-label={t('ins-shapes')}>
        <For each={SHAPE_IDS}>{(shape) => (
          <button type="button" aria-label={t(`ins-shape-${shape}`)} data-tooltip={t(`ins-shape-${shape}`)} aria-pressed={id() === shape} data-testid={`insert-shape-${shape}`} onClick={() => setId(shape)} innerHTML={preview(shape, 36)} />
        )}</For>
      </div>
      {swatches(t('ins-shape-fill'), fill, setFill, 'insert-shape-fill')}
      {swatches(t('ins-shape-outline'), stroke, setStroke, 'insert-shape-stroke')}
      <div class="mw-insert-row">
        <label>{t('ins-shape-outline-width')}<input type="number" min="0" max="24" data-testid="insert-shape-stroke-width" value={strokeWidth()} onInput={(e) => setStrokeWidth(Math.max(0, Math.min(24, Number(e.currentTarget.value) || 0)))} /></label>
        <label>{t('ins-shape-width')}<input type="number" min="8" max="1200" data-testid="insert-shape-width" value={width()} onInput={(e) => setWidth(Math.max(8, Math.min(1200, Number(e.currentTarget.value) || 8)))} /></label>
        <label>{t('ins-shape-height')}<input type="number" min="8" max="1200" data-testid="insert-shape-height" value={height()} onInput={(e) => setHeight(Math.max(8, Math.min(1200, Number(e.currentTarget.value) || 8)))} /></label>
      </div>
      <label>{t('ins-alt')}<input data-testid="insert-shape-alt" value={alt()} placeholder={t(`ins-shape-${id()}`)} onInput={(e) => setAlt(e.currentTarget.value)} /></label>
      <div class="mw-insert-preview" aria-hidden="true" innerHTML={preview(id())} />
      <p class="mw-insert-note">{t('ins-shape-note')}</p>
      {error() && <p class="mw-insert-warn" role="alert">{error()}</p>}
    </InsertDialog>
  );
}
