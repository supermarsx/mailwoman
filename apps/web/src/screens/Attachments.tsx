// The global Attachments module (plan §0.10 / §2.4): a grid/list of every
// attachment across the account, filtered by type / sender / size / date and by
// the shared search operators (`filename:` / `type:` / `larger:` / `from:` …).
// Data comes from `Email/query{filter:{hasAttachment:true}}` via `loadAttachments`
// (online → engine `mw-search` backs the operators; e9). The component takes its
// data as a `load` callback or preloaded `items`, so it is wireable AND testable.

import { createMemo, createResource, createSignal, For, Show, type JSX } from 'solid-js';
import { t } from '../i18n';
import {Icon, type IconName} from '../components/Icon.tsx';
import {AsyncPending, AsyncError} from '../components/AsyncState.tsx';
import * as table from './attachmentsTable.css.ts';
import '../viewers/viewers.css';
import {
  categoryOf,
  filterAttachments,
  formatSize,
  parseAttachmentQuery,
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

  // The module owns its loading state. Reading a pending resource here would
  // suspend the enclosing mailbox route and replace the entire app with Loading.
  const current = () => data.error || data.state === 'pending' || data.state === 'unresolved' ? undefined : data.latest;
  const all = (): AttachmentItem[] => current()?.items ?? props.items ?? [];
  const icons:Partial<Record<TypeCategory,IconName>>={image:'image',pdf:'filePdf',text:'file',audio:'fileAudio',video:'fileVideo',message:'mail'};

  const filtered = createMemo<AttachmentItem[]>(() => {
    const base = parseAttachmentQuery(query());
    const filters = cat() === 'all' ? base : { ...base, category: cat() };
    return filterAttachments(all(), filters);
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
      </header>
      <Show when={props.loadPage}><p class="mw-attach__status">{t('common-attach-page-filter-note')}</p></Show>

      <Show when={!data.error} fallback={<AsyncError error={data.error} onRetry={()=>void refetch()}/> }><Show
        when={!data.loading||all().length>0}
        fallback={<AsyncPending variant="table" message={t('common-attach-loading')} onRetry={()=>void refetch()}/>}
      >
        <Show
          when={filtered().length > 0}
          fallback={<p class="mw-attach__status">{t('common-attach-empty')}</p>}
        >
          <div class={table.wrap}><table class={table.table} aria-label={t('common-attach-title')} aria-busy={data.loading}><thead><tr><For each={['name','type','size','sender','subject','date']}>{key=><th scope="col">{t(`common-attach-column-${key}`)}</th>}</For></tr></thead><tbody>
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
                    <Icon name={icons[categoryOf(item.mime)]??'file'}/><span>{item.name}</span>
                  </button>
                  </td><td title={item.mime}>{catLabel(categoryOf(item.mime))}</td><td>{formatSize(item.size)}</td><td>{item.from}</td><td><Show when={props.onOpenMessage} fallback={item.subject}><button type="button" class={table.name} onClick={()=>props.onOpenMessage?.(item)} title={t('common-attach-open-message')}>{item.subject}</button></Show></td><td>{Number.isNaN(Date.parse(item.receivedAt))?'—':new Date(item.receivedAt).toLocaleDateString()}</td>
                </tr>
              )}
            </For>
          </tbody></table></div>
        </Show>
      </Show></Show>
      <Show when={props.loadPage}><nav class={table.paging} aria-label={t('common-attach-pages')}>
        <button type="button" disabled={data.loading || positions().length===1} onClick={()=>setPositions(old=>old.slice(0,-1))}>{t('common-attach-previous')}</button>
        <span>{t('common-attach-page',{page:positions().length})}</span>
        <button type="button" disabled={data.loading || current()?.nextPosition == null} onClick={()=>{const next=current()?.nextPosition;if(next!=null)setPositions(old=>[...old,next]);}}>{t('common-attach-next')}</button>
      </nav></Show>
    </section>
  );
}

export default Attachments;
