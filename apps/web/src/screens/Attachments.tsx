// Paged account attachment table with local filters and an explicit cancellable
// account-wide metadata scan. Attachment bytes are fetched only by the viewer.

import { createEffect, createMemo, createResource, createSignal, onCleanup, For, Show, type JSX } from 'solid-js';
import { t } from '../i18n';
import {Icon} from '../components/Icon.tsx';
import {inspectFile} from '../viewers/fileInfo.ts';
import {AsyncPending, AsyncError} from '../components/AsyncState.tsx';
import * as table from './attachmentsTable.css.ts';
import '../viewers/viewers.css';
import {
  categoryOf,
  filterAttachments,
  formatSize,
  parseAttachmentQuery,
  searchAttachmentPages,
  type AttachmentSearchProgress,
  type AttachmentItem,
  type AttachmentPage,
  type TypeCategory,
} from '../viewers/attachments.ts';

export interface AttachmentsProps {
  /** Live loader (e.g. `() => loadAttachments(client, accountId)`). */
  load?: () => Promise<AttachmentItem[]>;
  loadPage?: (position: number) => Promise<AttachmentPage>;
  /** Preloaded rows (tests / offline slice); used when `load` is absent. */
  items?: AttachmentItem[];
  onOpen?: (item: AttachmentItem) => void;
  onOpenMessage?: (item: AttachmentItem) => void;
}

const CATEGORIES: Array<TypeCategory | 'all'> = [
  'all',
  'image',
  'pdf',
  'text',
  'audio',
  'video',
  'message',
  'other',
];
const COLUMNS = ['name','type','size','sender','subject','date'] as const;
type Column = typeof COLUMNS[number];

/** Localised label for a type-filter category (falls back to the raw key). */
function catLabel(c: TypeCategory | 'all'): string {
  return t(`common-attach-cat-${c}`);
}

export function Attachments(props: AttachmentsProps): JSX.Element {
  const [positions, setPositions] = createSignal([0]);
  const position = () => positions().at(-1) ?? 0;
  const [data,{refetch}] = createResource(position, async (offset): Promise<AttachmentPage> =>
    props.loadPage ? props.loadPage(offset) : {items: props.load ? await props.load() : props.items ?? [], position: 0, nextPosition: null},
  );
  const [query, setQuery] = createSignal('');
  const [cat, setCat] = createSignal<TypeCategory | 'all'>('all');
  const [sort, setSort] = createSignal<{column:Column;ascending:boolean}>({column:'date',ascending:false});
  const [search, setSearch] = createSignal<AttachmentSearchProgress>();
  const [searching, setSearching] = createSignal(false);
  const [searchError, setSearchError] = createSignal<unknown>();
  const [cancelled, setCancelled] = createSignal(false);
  let searchController: AbortController | undefined;
  function resetSearch(): void {
    searchController?.abort(); searchController = undefined;
    setSearch(undefined); setSearching(false); setSearchError(undefined); setCancelled(false);
  }
  createEffect(() => { query(); cat(); resetSearch(); });
  onCleanup(() => searchController?.abort());
  const filters = () => {
    const base = parseAttachmentQuery(query());
    return cat() === 'all' ? base : {...base, category: cat()};
  };
  async function searchAccount(): Promise<void> {
    if (!props.loadPage) return;
    resetSearch();
    const controller = new AbortController(); searchController = controller;
    setSearch({items: [], pages: 0, limited: false}); setSearching(true);
    try {
      await searchAttachmentPages(props.loadPage, filters(), controller.signal, value => {
        if (searchController === controller) setSearch(value);
      });
    } catch (error) {
      if (searchController === controller && !controller.signal.aborted) setSearchError(error);
    } finally {
      if (searchController === controller) setSearching(false);
    }
  }

  // The module owns its loading state. Reading a pending resource here would
  // suspend the enclosing mailbox route and replace the entire app with Loading.
  const current = () => data.error || data.state === 'pending' || data.state === 'unresolved' ? undefined : data.latest;
  const all = (): AttachmentItem[] => current()?.items ?? props.items ?? [];

  const filtered = createMemo<AttachmentItem[]>(() => {
    const rows = search()?.items ?? filterAttachments(all(), filters());
    const {column,ascending}=sort();
    const value=(item:AttachmentItem):string|number=>column==='type'?item.mime:column==='sender'?item.from:column==='date'?item.receivedAt:item[column];
    return [...rows].sort((a,b)=>{const av=value(a),bv=value(b);return (typeof av==='number'&&typeof bv==='number'?av-bv:String(av).localeCompare(String(bv),undefined,{numeric:true,sensitivity:'base'}))*(ascending?1:-1);});
  });

  return (
    <section class="mw-attach" data-mw-surface aria-label={t('common-attach-title')}>
      <header class="mw-attach__bar">
        <h2 class="mw-attach__title">{t('common-attach-title')}</h2>
        <button type="button" class={table.name} aria-label={t('common-attach-refresh')} title={t('common-attach-refresh')} disabled={data.loading} onClick={()=>void refetch()}><Icon name="refresh"/></button>
        <input
          class="mw-attach__search"
          type="search"
          placeholder={t('common-attach-search-placeholder')}
          aria-label={t('common-attach-search')}
          value={query()}
          onInput={(e) => setQuery(e.currentTarget.value)}
        />
        <select
          class="mw-attach__cat"
          aria-label={t('common-attach-filter-type')}
          value={cat()}
          onChange={(e) => setCat(e.currentTarget.value as TypeCategory | 'all')}
        >
          <For each={CATEGORIES}>{(c) => <option value={c}>{catLabel(c)}</option>}</For>
        </select>
        <Show when={props.loadPage}><button type="button" disabled={searching()} onClick={()=>void searchAccount()}>{t('common-attach-search-account')}</button></Show>
      </header>
      <Show when={props.loadPage && !search()}><p class="mw-attach__status">{t('common-attach-page-filter-note')}</p></Show>
      <Show when={search()}>{result=><div class={table.paging} role="status">
        <span>{t(searching()?'common-attach-scanning':cancelled()?'common-attach-scan-cancelled':searchError()?'common-attach-scan-incomplete':result().limited?'common-attach-scan-limited':'common-attach-scan-complete',{pages:result().pages,count:result().items.length})}</span>
        <Show when={searching()} fallback={<button type="button" onClick={resetSearch}>{t('common-attach-browse-pages')}</button>}><button type="button" onClick={()=>{searchController?.abort();setSearching(false);setCancelled(true);}}>{t('common-attach-cancel-search')}</button></Show>
      </div>}</Show>
      <Show when={searchError()}><AsyncError error={searchError()} onRetry={()=>void searchAccount()}/></Show>

      <Show when={!data.error} fallback={<AsyncError error={data.error} onRetry={()=>void refetch()}/> }><Show
        when={!data.loading||all().length>0}
        fallback={<AsyncPending variant="table" message={t('common-attach-loading')} onRetry={()=>void refetch()}/>}
      >
        <Show
          when={filtered().length > 0}
          fallback={<p class="mw-attach__status">{t('common-attach-empty')}</p>}
        >
          <div class={table.wrap}><table class={table.table} aria-label={t('common-attach-title')} aria-busy={data.loading||searching()}><thead><tr><For each={COLUMNS}>{key=><th scope="col" aria-sort={sort().column===key?(sort().ascending?'ascending':'descending'):'none'}><button type="button" class={table.name} onClick={()=>setSort(old=>({column:key,ascending:old.column===key?!old.ascending:true}))}>{t(`common-attach-column-${key}`)}<Show when={sort().column===key}><Icon name={sort().ascending?'chevronUp':'chevronDown'}/></Show></button></th>}</For></tr></thead><tbody>
            <For each={filtered()}>
              {(item) => (
                <tr>
                  <td>
                  <button
                    type="button"
                    class={table.name}
                    data-category={categoryOf(item.mime)}
                    onClick={() => props.onOpen?.(item)}
                  >
                    <Icon name={inspectFile(item.name,item.mime).icon}/><span>{item.name}</span>
                  </button>
                  <For each={inspectFile(item.name,item.mime).warnings}>{warning=><span class={table.warning} title={t(`mail-file-warning-${warning}`)} aria-label={t(`mail-file-warning-${warning}`)}><Icon name="alert"/></span>}</For>
                  </td><td title={item.mime}>{catLabel(categoryOf(item.mime))}</td><td>{formatSize(item.size)}</td><td>{item.from}</td><td><Show when={props.onOpenMessage} fallback={item.subject}><button type="button" class={table.name} onClick={()=>props.onOpenMessage?.(item)} title={t('common-attach-open-message')}>{item.subject}</button></Show></td><td>{Number.isNaN(Date.parse(item.receivedAt))?'—':new Date(item.receivedAt).toLocaleDateString()}</td>
                </tr>
              )}
            </For>
          </tbody></table></div>
        </Show>
      </Show></Show>
      <Show when={props.loadPage && !search()}><nav class={table.paging} aria-label={t('common-attach-pages')}>
        <button type="button" disabled={data.loading || positions().length===1} onClick={()=>setPositions(old=>old.slice(0,-1))}>{t('common-attach-previous')}</button>
        <span>{t('common-attach-page',{page:positions().length})}</span>
        <button type="button" disabled={data.loading || current()?.nextPosition == null} onClick={()=>{const next=current()?.nextPosition;if(next!=null)setPositions(old=>[...old,next]);}}>{t('common-attach-next')}</button>
      </nav></Show>
    </section>
  );
}

export default Attachments;
