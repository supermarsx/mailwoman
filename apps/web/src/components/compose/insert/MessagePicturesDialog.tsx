// Picture / attachment picker over recent messages: image attachments with
// thumbnails, a search by sender or subject, one or many. Bytes come through the
// host's policy-checked download.

import { createEffect, createSignal, For, onCleanup, Show, type JSX } from 'solid-js';
import { t } from '../../../i18n/index.ts';
import type { AttachmentItem, AttachmentPage } from '../../../viewers/attachments.ts';
import { InsertDialog } from './InsertDialog.tsx';
import { loadCatalog } from '../../../i18n/catalog.ts';
// i18n: the ids this file shows live in these catalogs, which load with it.
void loadCatalog('compose-insert');

const IMAGE = /^image\/(png|jpe?g|gif|webp)$/i;

export function filterMessageItems(items: readonly AttachmentItem[], query: string, imagesOnly: boolean): AttachmentItem[] {
  const q = query.trim().toLowerCase();
  return items.filter((i) => (!imagesOnly || IMAGE.test(i.mime)) && (q === '' || i.from.toLowerCase().includes(q) || i.subject.toLowerCase().includes(q) || i.name.toLowerCase().includes(q)));
}

const key = (i: AttachmentItem): string => `${i.emailId}\u0000${i.blobId}\u0000${i.name}`;

export function MessagePicturesDialog(props: {
  as: 'inline' | 'attach';
  loadPage: (position: number) => Promise<AttachmentPage>;
  fetchPicture: (item: AttachmentItem) => Promise<File>;
  returnTo: HTMLElement | null;
  onClose: () => void;
  onPick: (files: File[]) => void;
}): JSX.Element {
  const imagesOnly = props.as === 'inline';
  const [items, setItems] = createSignal<AttachmentItem[]>([]);
  const [next, setNext] = createSignal<number | null>(0);
  const [query, setQuery] = createSignal('');
  const [selected, setSelected] = createSignal<ReadonlySet<string>>(new Set());
  const [thumbs, setThumbs] = createSignal<Record<string, string>>({});
  const [loading, setLoading] = createSignal(false);
  const [error, setError] = createSignal<string | null>(null);
  const urls: string[] = [];
  onCleanup(() => { for (const u of urls) URL.revokeObjectURL(u); });
  let disposed = false;
  onCleanup(() => { disposed = true; });

  const more = async (): Promise<void> => {
    const at = next();
    if (at === null || loading()) return;
    setLoading(true);
    setError(null);
    try {
      const page = await props.loadPage(at);
      if (disposed) return;
      setItems((cur) => [...cur, ...page.items.filter((i) => (!imagesOnly || IMAGE.test(i.mime)))]);
      setNext(page.nextPosition);
    } catch { setError(t('ins-pick-failed')); }
    finally { setLoading(false); }
  };
  void more();

  const shown = (): AttachmentItem[] => filterMessageItems(items(), query(), imagesOnly).slice(0, 60);
  // Thumbnails load lazily for what is shown, a few at a time.
  const requested = new Set<string>();
  createEffect(() => {
    if (!imagesOnly) return;
    for (const item of shown().slice(0, 24)) {
      const k = key(item);
      if (requested.has(k)) continue;
      requested.add(k);
      void props.fetchPicture(item).then((file) => {
        if (disposed) return;
        const url = URL.createObjectURL(file);
        urls.push(url);
        setThumbs((cur) => ({ ...cur, [k]: url }));
      }).catch(() => undefined);
    }
  });
  const toggle = (item: AttachmentItem): void => { setSelected((cur) => { const n = new Set(cur); const k = key(item); if (n.has(k)) n.delete(k); else n.add(k); return n; }); };
  const choose = (): void => {
    const chosen = items().filter((i) => selected().has(key(i)));
    setLoading(true);
    void Promise.all(chosen.map((i) => props.fetchPicture(i))).then((files) => props.onPick(files)).catch(() => setError(t('ins-pick-failed'))).finally(() => setLoading(false));
  };
  return (
    <InsertDialog title={t(imagesOnly ? 'ins-picture-messages' : 'ins-attach-messages')} testId="insert-messages-dialog" returnTo={props.returnTo} busy={loading()} onClose={props.onClose} onSubmit={choose} submitLabel={t('ins-pick-count', { count: selected().size })} submitDisabled={selected().size === 0}>
      <label>{t('ins-pick-search')}<input type="search" data-testid="insert-messages-search" value={query()} onInput={(e) => setQuery(e.currentTarget.value)} /></label>
      <div class="mw-insert-grid" role="listbox" aria-multiselectable="true" aria-label={t('ins-pick-list')}>
        <For each={shown()} fallback={<p class="mw-insert-note">{loading() ? t('ins-loading') : t('ins-pick-none')}</p>}>{(item) => (
          <button type="button" role="option" class="mw-insert-thumb" data-testid="insert-messages-item" aria-selected={selected().has(key(item))} aria-pressed={selected().has(key(item))} title={`${item.name} — ${item.from} — ${item.subject}`} onClick={() => toggle(item)}>
            <Show when={imagesOnly && thumbs()[key(item)]} fallback={<div class="mw-insert-thumb-ph" aria-hidden="true">{imagesOnly ? '…' : '\u{1F4CE}'}</div>}>
              {(src) => <img src={src()} alt="" />}
            </Show>
            <span>{item.name}</span>
            <span>{item.subject}</span>
          </button>
        )}</For>
      </div>
      <Show when={next() !== null}><button type="button" class="btn btn--ghost" data-testid="insert-messages-more" disabled={loading()} onClick={() => void more()}>{t('ins-pick-more')}</button></Show>
      <Show when={error()}>{(m) => <p class="mw-insert-warn" role="alert">{m()}</p>}</Show>
    </InsertDialog>
  );
}
