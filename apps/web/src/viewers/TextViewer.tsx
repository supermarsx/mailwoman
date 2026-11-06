import { createEffect, createMemo, createResource, createSignal, onCleanup, onMount, Show, type JSX } from 'solid-js';
import type { ViewerProps } from '../contracts/viewer.ts';
import { apiFetch } from '../api/http.ts';
import { loadCatalog } from '../i18n/catalog.ts';
import { formatBytes, formatNumber, t } from '../i18n/index.ts';
import { mailFeature } from '../state/mailFeatures.ts';
import { decodeText, TEXT_PREVIEW_CAP, type DecodedText } from './textDecode.ts';
import './text-viewer.css';

/** What the viewer shows: a decoded file, or a contained failure. */
type Loaded = DecodedText | { kind: 'error' };

/** Fetch and decode; never rejects, so a failure stays inside this panel. */
export async function loadText(blobUrl: string, mime: string): Promise<Loaded> {
  try {
    const response = await apiFetch(blobUrl);
    if (!response.ok) throw new Error(`status ${response.status}`);
    return decodeText(new Uint8Array(await response.arrayBuffer()), mime);
  } catch (error) {
    // Context only (type, error class): never the file's content or name.
    console.warn('[viewer] text preview failed', { type: mime, error: error instanceof Error ? error.name : typeof error });
    return { kind: 'error' };
  }
}

/** Plain-text attachment. The content only ever reaches the DOM as `textContent`
 *  of a `<pre>` (never HTML), so a file cannot inject markup or script. */
export function TextViewer(props: ViewerProps): JSX.Element {
  onMount(() => void loadCatalog('text-viewer'));
  const [loaded, { refetch }] = createResource(
    () => [props.blobUrl, props.mime] as const,
    ([url, mime]) => loadText(url, mime),
  );
  const [numbers, setNumbers] = createSignal(false);
  const [wrap, setWrap] = createSignal(true);
  const [query, setQuery] = createSignal('');
  const [current, setCurrent] = createSignal(0);
  const [copied, setCopied] = createSignal<'idle' | 'ok' | 'failed'>('idle');
  let code: HTMLPreElement | undefined;
  let scroller: HTMLDivElement | undefined;
  let resetTimer: ReturnType<typeof setTimeout> | undefined;
  onCleanup(() => clearTimeout(resetTimer));

  const value = (): Loaded | undefined => (loaded.state === 'ready' || loaded.state === 'refreshing' ? loaded() : undefined);
  const text = createMemo(() => {
    const v = value();
    return v?.kind === 'text' ? v : undefined;
  });
  const matches = createMemo(() => {
    const doc = text();
    const needle = query().toLowerCase();
    const found: number[] = [];
    if (!doc || needle === '') return found;
    const hay = doc.text.toLowerCase();
    // Lower-casing can change the length for a few scripts; offsets would drift.
    if (hay.length !== doc.text.length) return found;
    for (let at = hay.indexOf(needle); at !== -1 && found.length < 5000; at = hay.indexOf(needle, at + needle.length)) found.push(at);
    return found;
  });
  const gutter = createMemo(() => {
    const doc = text();
    return doc ? Array.from({ length: doc.lines }, (_, i) => String(i + 1)).join('\n') : '';
  });

  // textContent only: the file never passes through an HTML parser.
  createEffect(() => {
    const doc = text();
    if (code) code.textContent = doc?.text ?? '';
  });
  createEffect(() => {
    query();
    setCurrent(0);
  });
  createEffect(() => {
    const at = matches()[current()];
    const node = code?.firstChild;
    if (at === undefined || !node || !scroller) return;
    try {
      const range = document.createRange();
      range.setStart(node, at);
      range.setEnd(node, at + query().length);
      const selection = window.getSelection();
      selection?.removeAllRanges();
      selection?.addRange(range);
      const box = range.getBoundingClientRect();
      const view = scroller.getBoundingClientRect();
      if (box.top < view.top || box.bottom > view.bottom) scroller.scrollTop += box.top - view.top - view.height / 3;
    } catch {
      /* a selection that cannot be placed is only a missed highlight */
    }
  });

  function step(delta: number): void {
    const count = matches().length;
    if (count > 0) setCurrent((current() + delta + count) % count);
  }
  async function copy(): Promise<void> {
    const doc = text();
    if (!doc) return;
    try {
      await navigator.clipboard.writeText(doc.text);
      setCopied('ok');
    } catch {
      setCopied('failed');
    }
    clearTimeout(resetTimer);
    resetTimer = setTimeout(() => setCopied('idle'), 2000);
  }
  const download = (): JSX.Element => (
    <Show when={mailFeature('allowDownload')}>
      <a class="btn" href={props.blobUrl} download={props.name.replace(/[\\/]/g, '_')}>{t('text-viewer-download')}</a>
    </Show>
  );

  return (
    <Show when={value()} fallback={<p class="mw-viewer__loading" role="status">{t('text-viewer-loading')}</p>}>
      {(state) => (
        <>
          <Show when={state().kind === 'error'}>
            <div class="mw-text__state" role="alert" data-text-state="error">
              <p><strong>{t('text-viewer-failed')}</strong></p>
              <p>{t('text-viewer-failed-help')}</p>
              <div class="mw-text__actions">
                <button type="button" class="btn" data-testid="text-retry" onClick={() => void refetch()}>{t('text-viewer-retry')}</button>
                {download()}
              </div>
            </div>
          </Show>
          <Show when={state().kind === 'empty'}>
            <div class="mw-text__state" role="status" data-text-state="empty"><p>{t('text-viewer-empty')}</p></div>
          </Show>
          <Show when={state().kind === 'binary'}>
            <div class="mw-text__state" role="status" data-text-state="binary">
              <p>{t('text-viewer-binary')}</p>
              <div class="mw-text__actions">{download()}</div>
            </div>
          </Show>
          <Show when={text()}>
            {(doc) => (
              <div class="mw-text" data-text-state="text" data-wrap={wrap() ? 'true' : 'false'}>
                <div class="mw-text__bar">
                  <label><input type="checkbox" checked={numbers() && !wrap()} disabled={wrap()} data-testid="text-numbers" onChange={(e) => setNumbers(e.currentTarget.checked)} /> {t('text-viewer-line-numbers')}</label>
                  <label><input type="checkbox" checked={wrap()} data-testid="text-wrap" onChange={(e) => setWrap(e.currentTarget.checked)} /> {t('text-viewer-wrap')}</label>
                  <input
                    type="search"
                    value={query()}
                    placeholder={t('text-viewer-search')}
                    aria-label={t('text-viewer-search')}
                    data-testid="text-search"
                    onInput={(e) => setQuery(e.currentTarget.value)}
                    onKeyDown={(e) => {
                      if (e.key === 'Enter') {
                        e.preventDefault();
                        step(e.shiftKey ? -1 : 1);
                      }
                    }}
                  />
                  <Show when={query() !== ''}>
                    <button type="button" class="btn btn--ghost" aria-label={t('text-viewer-search-prev')} disabled={matches().length === 0} onClick={() => step(-1)}>&lsaquo;</button>
                    <button type="button" class="btn btn--ghost" aria-label={t('text-viewer-search-next')} disabled={matches().length === 0} onClick={() => step(1)}>&rsaquo;</button>
                    <span role="status" data-testid="text-search-count">
                      {matches().length === 0
                        ? t('text-viewer-search-none')
                        : t('text-viewer-search-count', { current: formatNumber(current() + 1), total: formatNumber(matches().length) })}
                    </span>
                  </Show>
                  <button type="button" class="btn btn--ghost" data-testid="text-copy" onClick={() => void copy()}>
                    {copied() === 'ok' ? t('text-viewer-copied') : copied() === 'failed' ? t('text-viewer-copy-failed') : t('text-viewer-copy')}
                  </button>
                  <span class="mw-text__meta">{t('text-viewer-lines', { count: formatNumber(doc().lines) })} · {doc().encoding}</span>
                </div>
                <div class="mw-text__scroll" ref={scroller} tabindex="0" aria-label={props.name}>
                  <Show when={numbers() && !wrap()}><pre class="mw-text__gutter" aria-hidden="true">{gutter()}</pre></Show>
                  <pre class="mw-text__code" ref={code} data-testid="text-content" />
                </div>
                <Show when={doc().truncated}>
                  <p class="mw-text__note">{t('text-viewer-truncated', { shown: formatBytes(TEXT_PREVIEW_CAP), total: formatBytes(doc().totalBytes) })} {download()}</p>
                </Show>
                <Show when={doc().lineCut}><p class="mw-text__note">{t('text-viewer-line-cut')}</p></Show>
              </div>
            )}
          </Show>
        </>
      )}
    </Show>
  );
}

export default TextViewer;
