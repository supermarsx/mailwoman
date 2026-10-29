// The reader's zoom group (t59): zoom out, the current level (a button that
// resets to 100%) and zoom in. It scales only the message body; see readerZoom.ts.
// `zoomRowAction` is the same group as one item of the reader action row, which
// folds into the row's "More" menu as three entries when the row is too narrow.

import { onMount, type JSX } from 'solid-js';
import { loadCatalog, t } from '../i18n/index.ts';
import type { ContextAction } from './ContextMenu.tsx';
import { Icon } from './Icon.tsx';
import { DEFAULT_ZOOM, MAX_ZOOM, MIN_ZOOM, readerZoom, readerZoomLocked, resetZoom, zoomIn, zoomOut } from './readerZoom.ts';
import * as a11y from './mailA11y.css.ts';
import * as css from './readerZoom.css.ts';
import './reader-action.css';

const button = `btn btn--ghost reader-action reader-action--compact ${a11y.focusable}`;

export function ReaderZoom(): JSX.Element {
  onMount(() => void loadCatalog('reader'));
  const level = (): number => readerZoom();
  return (
    <span class={css.group} role="group" aria-label={t('reader-zoom-group')} data-testid="reader-zoom">
      <button type="button" class={button} aria-label={t('reader-zoom-out')} title={t('reader-zoom-out-help')} disabled={readerZoomLocked() || level() <= MIN_ZOOM} data-testid="reader-zoom-out" onClick={zoomOut}><Icon name="minus" /></button>
      <button type="button" class={`${button} ${css.level}`} aria-label={t('reader-zoom-reset-label', { level: level() })} title={t('reader-zoom-reset-help')} disabled={readerZoomLocked()} data-testid="reader-zoom-level" onClick={resetZoom}>
        <span aria-hidden="true">{t('reader-zoom-level', { level: level() })}</span>
      </button>
      <button type="button" class={button} aria-label={t('reader-zoom-in')} title={t('reader-zoom-in-help')} disabled={readerZoomLocked() || level() >= MAX_ZOOM} data-testid="reader-zoom-in" onClick={zoomIn}><Icon name="plus" /></button>
    </span>
  );
}

/** The zoom group's entries in the action row's "More" menu. */
export function zoomMenuActions(): ContextAction[] {
  const locked = readerZoomLocked();
  const level = readerZoom();
  return [
    { id: 'zoom:out', label: t('reader-zoom-out'), icon: 'minus', disabled: locked || level <= MIN_ZOOM, separatorBefore: true, run: zoomOut },
    { id: 'zoom:in', label: t('reader-zoom-in'), icon: 'plus', disabled: locked || level >= MAX_ZOOM, run: zoomIn },
    { id: 'zoom:reset', label: t('reader-zoom-reset-menu', { level }), icon: 'restore', disabled: locked || level === DEFAULT_ZOOM, run: resetZoom },
  ];
}

/** One item of the reader action row (ReaderToolbar.tsx): the whole group
 *  inline, or its three menu entries once folded. */
export function zoomRowAction(): { id: string; inline: () => JSX.Element; menu: () => ContextAction[] } {
  return { id: 'zoom', inline: () => <ReaderZoom />, menu: zoomMenuActions };
}
