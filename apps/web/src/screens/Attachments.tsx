// Paged account attachment table with local filters and an explicit cancellable
// account-wide metadata scan. Attachment bytes are fetched only by the viewer.

import { batch, createEffect, createMemo, createResource, createSignal, onCleanup, For, Show, type JSX } from 'solid-js';
import { t, formatDate, loadCatalog } from '../i18n';
import {accountUiPreferences,setAccountUiPreferences} from '../api/accountPreferences.ts';
import {observePreferenceAccount,preferenceAccount,preferenceAccountIsCurrent} from '../api/accountPreferenceLifecycle.ts';
import {createConfiguredClient} from '../api/transport.ts';
import {Icon} from '../components/Icon.tsx';
import {inspectFile} from '../viewers/fileInfo.ts';
import { AsyncError } from '../components/AsyncState.tsx';
import { InlineLoading } from '../components/InlineLoading.tsx';
import * as table from './attachmentsTable.css.ts';
import '../viewers/viewers.css';
import {
  categoryOf,
  filterAttachments,
  formatSize,
  loadPinnedAttachments,
  parseAttachmentQuery,
  searchAttachmentPages,
  type AttachmentFilters,
  type AttachmentSearchProgress,
  type AttachmentItem,
  type AttachmentPage,
  type TypeCategory,
} from '../viewers/attachments.ts';

export interface AttachmentsProps {
  /** Live loader (e.g. `() => loadAttachments(client, accountId)`). */
  load?: () => Promise<AttachmentItem[]>;
  accountId?: string | null;
  loadPage?: (position: number) => Promise<AttachmentPage>;
  /** Preloaded rows (tests / offline slice); used when `load` is absent. */
  items?: AttachmentItem[];
  mailboxes?: readonly {id:string;name:string;parentId?:string|null;mailwomanSearchQuery?:string}[];
  onOpen?: (item: AttachmentItem) => void;
  onOpenMessage?: (item: AttachmentItem) => void;
}

const CATEGORIES: Array<TypeCategory | 'all'> = [
  'all',
  'image',
  'pdf',
  'document',
  'spreadsheet',
  'archive',
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

interface FilterFields {from:string;subject:string;filename:string;extensions:string;mailboxId:string;date:string;after:string;before:string;size:string;min:string;max:string}
const emptyFields=():FilterFields=>({from:'',subject:'',filename:'',extensions:'',mailboxId:'',date:'any',after:'',before:'',size:'any',min:'',max:''});
const SEARCH_DELAY=400;

export function Attachments(props: AttachmentsProps): JSX.Element {
  void loadCatalog('attachments-filters');
  const [pinnedOnly,setPinnedOnly]=createSignal(false);
  const pins=()=>accountUiPreferences().pinnedAttachments??[];
  const isPinned=(item:AttachmentItem)=>pins().some(pin=>pin.emailId===item.emailId&&pin.blobId===item.blobId);
  const [pinOwner,setPinOwner]=createSignal(preferenceAccount());
  onCleanup(observePreferenceAccount(setPinOwner));
  const ownsPins=()=>Boolean(props.accountId&&pinOwner()?.id===props.accountId);
  function togglePin(item:AttachmentItem):void {
    const owner=preferenceAccount();
    if(!owner||owner.id!==props.accountId||!preferenceAccountIsCurrent(owner))return;
    if(isPinned(item)){
      setAccountUiPreferences({pinnedAttachments:pins().filter(pin=>pin.emailId!==item.emailId||pin.blobId!==item.blobId)});
    }else if(pins().length<100){
      setAccountUiPreferences({pinnedAttachments:[...pins(),{emailId:item.emailId,blobId:item.blobId}]});
    }
  }
  const pinQuery=createMemo(()=>pinnedOnly()&&ownsPins()&&pins().length?{account:props.accountId!,pins:pins()}:false);
  const [pinnedRows,{refetch:retryPins}]=createResource(pinQuery,async ({account,pins})=>{
    const owner=preferenceAccount();
    if(!owner||owner.id!==account)return [];
    const rows=await loadPinnedAttachments(createConfiguredClient(),account,pins);
    return preferenceAccountIsCurrent(owner)?rows:[];
  });
  const [fields,setFields]=createSignal<FilterFields>(emptyFields());
  const [searchScope,setSearchScope]=createSignal<'all'|'page'>('all');
  const change=(key:keyof FilterFields,value:string)=>setFields(previous=>({...previous,[key]:value}));
  let searchTimer:ReturnType<typeof setTimeout>|undefined;
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
  const active=()=>query().trim()!==''||cat()!=='all'||Object.entries(fields()).some(([key,value])=>key==='date'||key==='size'?value!=='any':value!=='');
  const invalid=()=>{
    const f=fields();
    if(f.date==='custom'&&f.after&&f.before&&f.after>f.before)return t('attach-filters-date-error');
    if(f.size==='custom'&&([f.min,f.max].some(value=>value!==''&&(!Number.isFinite(Number(value))||Number(value)<0))||f.min!==''&&f.max!==''&&Number(f.min)>Number(f.max)))return t('attach-filters-size-error');
    return '';
  };
  createEffect(()=>{
    query();cat();fields();searchScope();clearTimeout(searchTimer);resetSearch();
    if(props.loadPage&&searchScope()==='all'&&active()&&!invalid()&&!pinnedOnly())searchTimer=setTimeout(()=>void searchAccount(),SEARCH_DELAY);
  });
  onCleanup(()=>{clearTimeout(searchTimer);searchController?.abort();});
  function clearFilters():void {batch(()=>{setQuery('');setCat('all');setFields(emptyFields());});}
  const folderOptions=createMemo(()=>{
    const boxes=(props.mailboxes??[]).filter(box=>box.mailwomanSearchQuery===undefined),byId=new Map(boxes.map(box=>[box.id,box]));
    return boxes.map(box=>{const names=[box.name],seen=new Set([box.id]);let parent=box.parentId;while(parent&&!seen.has(parent)){seen.add(parent);const value=byId.get(parent);if(!value)break;names.unshift(value.name);parent=value.parentId;}return{id:box.id,name:names.join(' / ')};});
  });
  const filters = ():AttachmentFilters => {
    const f=fields(),structured:AttachmentFilters={};
    if(cat()!=='all')structured.category=cat();
    for(const key of ['from','subject','filename','mailboxId'] as const)if(f[key].trim())structured[key]=f[key].trim();
    if(f.extensions.trim())structured.extensions=f.extensions.split(/[\s,;]+/).filter(Boolean);
    if(f.date==='custom'){if(f.after)structured.after=f.after;if(f.before)structured.before=f.before;}
    else if(f.date!=='any'){const now=new Date();structured.after=f.date==='year'?new Date(Date.UTC(now.getUTCFullYear(),0,1)).toISOString():new Date(now.valueOf()-Number(f.date)*86400000).toISOString();}
    if(f.size==='small')structured.maxSize=1e6;
    else if(f.size==='medium'){structured.minSize=1e6;structured.maxSize=10e6;}
    else if(f.size==='large')structured.minSize=10e6;
    else if(f.size==='custom'){if(f.min!=='')structured.minSize=Number(f.min)*1e6;if(f.max!=='')structured.maxSize=Number(f.max)*1e6;}
    return {and:[parseAttachmentQuery(query()),structured]};
  };
  const chips=()=>{
    const f=fields(),out:{key:string;label:string;clear:()=>void}[]=[];
    if(query().trim())out.push({key:'query',label:query(),clear:()=>setQuery('')});
    if(cat()!=='all')out.push({key:'type',label:catLabel(cat()),clear:()=>setCat('all')});
    for(const key of ['from','subject','filename','extensions','mailboxId'] as const)if(f[key])out.push({key,label:`${t(`attach-filters-${key}`)}: ${key==='mailboxId'?folderOptions().find(box=>box.id===f[key])?.name??f[key]:f[key]}`,clear:()=>change(key,'')});
    if(f.date!=='any')out.push({key:'date',label:f.date==='custom'?`${t('attach-filters-date')}: ${f.after||'…'} – ${f.before||'…'}`:t(`attach-filters-date-${f.date}`),clear:()=>setFields(previous=>({...previous,date:'any',after:'',before:''}))});
    if(f.size!=='any')out.push({key:'size',label:f.size==='custom'?t('attach-filters-size-custom-chip',{min:f.min||'0',max:f.max||'∞'}):t(`attach-filters-size-${f.size}`),clear:()=>setFields(previous=>({...previous,size:'any',min:'',max:''}))});
    return out;
  };
  async function searchAccount(): Promise<void> {
    clearTimeout(searchTimer);
    if (!props.loadPage||invalid()||pinnedOnly()) return;
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
    const rows = pinnedOnly()
      ? filterAttachments(pinnedRows.state==='ready'?pinnedRows.latest??[]:[],filters()).filter(isPinned)
      : search()?.items ?? filterAttachments(all(), filters());
    const {column,ascending}=sort();
    const value=(item:AttachmentItem):string|number=>column==='type'?item.mime:column==='sender'?item.from:column==='date'?item.receivedAt:item[column];
    return [...rows].sort((a,b)=>{const av=value(a),bv=value(b);return (typeof av==='number'&&typeof bv==='number'?av-bv:String(av).localeCompare(String(bv),undefined,{numeric:true,sensitivity:'base'}))*(ascending?1:-1);});
  });

  return (
    <section class="mw-attach" data-mw-surface aria-label={t('common-attach-title')}>
      <header class="mw-attach__bar">
        <h2 class="mw-attach__title">{t('common-attach-title')}</h2>
        <Show when={ownsPins()}><button type="button" class="btn btn--ghost" aria-pressed={pinnedOnly()} onClick={()=>setPinnedOnly(value=>!value)}><Icon name="pin"/>{t('attach-pins-only')}</button></Show>
        <span class={table.resultCount}>{t('attach-filters-results',{count:filtered().length})}</span>
        <button type="button" class={table.name} aria-label={t('common-attach-refresh')} title={t('common-attach-refresh')} disabled={data.loading||searching()} onClick={()=>{if(pinnedOnly())void retryPins();else if(search())void searchAccount();else void refetch();}}><Icon name="refresh"/></button>
      </header>
      <form class={table.filters} aria-label={t('attach-filters-title')} onSubmit={event=>{event.preventDefault();clearTimeout(searchTimer);if(props.loadPage&&searchScope()==='all'&&!pinnedOnly())void searchAccount();}}>
        <div class={table.searchRow}>
          <label class={table.searchField}><span><Icon name="search"/>{t('common-attach-search')}</span><input class={table.control} type="search" placeholder={t('attach-filters-placeholder')} aria-label={t('common-attach-search')} value={query()} onInput={event=>setQuery(event.currentTarget.value)}/></label>
          <button class="btn" type="submit" disabled={searching()||!!invalid()}><Icon name="search"/>{t(props.loadPage&&searchScope()==='all'?'common-attach-search-account':'attach-filters-apply')}</button>
          <button class="btn btn--ghost" type="button" disabled={!active()} onClick={clearFilters}>{t('attach-filters-clear')}</button>
        </div>
        <details class={table.more}><summary><Icon name="filter"/>{t('attach-filters-more')}</summary><div class={table.filterGrid}>
          <label class={table.field}><span><Icon name="file"/>{t('common-attach-filter-type')}</span><select class={table.control} value={cat()} onChange={event=>setCat(event.currentTarget.value as TypeCategory|'all')}><For each={CATEGORIES}>{value=><option value={value}>{catLabel(value)}</option>}</For></select></label>
          <label class={table.field}><span><Icon name="person"/>{t('attach-filters-from')}</span><input class={table.control} type="search" value={fields().from} placeholder={t('attach-filters-from-placeholder')} onInput={event=>change('from',event.currentTarget.value)}/></label>
          <label class={table.field}><span><Icon name="mail"/>{t('attach-filters-subject')}</span><input class={table.control} type="search" value={fields().subject} onInput={event=>change('subject',event.currentTarget.value)}/></label>
          <Show when={folderOptions().length}><label class={table.field}><span><Icon name="folder"/>{t('attach-filters-mailboxId')}</span><select class={table.control} value={fields().mailboxId} onChange={event=>change('mailboxId',event.currentTarget.value)}><option value="">{t('attach-filters-all-folders')}</option><For each={folderOptions()}>{box=><option value={box.id}>{box.name}</option>}</For></select></label></Show>
          <label class={table.field}><span><Icon name="calendar"/>{t('attach-filters-date')}</span><select class={table.control} value={fields().date} onChange={event=>setFields(previous=>({...previous,date:event.currentTarget.value,after:'',before:''}))}><For each={['any','7','30','year','custom']}>{value=><option value={value}>{t(`attach-filters-date-${value}`)}</option>}</For></select></label>
          <label class={table.field}><span><Icon name="package"/>{t('attach-filters-size')}</span><select class={table.control} value={fields().size} onChange={event=>setFields(previous=>({...previous,size:event.currentTarget.value,min:'',max:''}))}><For each={['any','small','medium','large','custom']}>{value=><option value={value}>{t(`attach-filters-size-${value}`)}</option>}</For></select></label>
          <Show when={props.loadPage}><label class={table.field}><span><Icon name="globe"/>{t('attach-filters-scope')}</span><select class={table.control} value={searchScope()} onChange={event=>setSearchScope(event.currentTarget.value as 'all'|'page')}><option value="all">{t('attach-filters-scope-all')}</option><option value="page">{t('attach-filters-scope-page')}</option></select></label></Show>
          <label class={table.field}><span>{t('attach-filters-filename')}</span><input class={table.control} type="search" value={fields().filename} onInput={event=>change('filename',event.currentTarget.value)}/></label>
          <label class={table.field}><span>{t('attach-filters-extensions')}</span><input class={table.control} type="text" value={fields().extensions} placeholder={t('attach-filters-extensions-placeholder')} onInput={event=>change('extensions',event.currentTarget.value)}/></label>
        </div><p class={table.help}>{t('attach-filters-operators')}</p>
        <Show when={fields().date==='custom'}><div class={table.filterGrid}>
          <label class={table.field}><span>{t('attach-filters-after')}</span><input class={table.control} type="date" value={fields().after} max={fields().before||undefined} onInput={event=>change('after',event.currentTarget.value)}/></label>
          <label class={table.field}><span>{t('attach-filters-before')}</span><input class={table.control} type="date" value={fields().before} min={fields().after||undefined} onInput={event=>change('before',event.currentTarget.value)}/></label>
        </div></Show>
        <Show when={fields().size==='custom'}><div class={table.filterGrid}>
          <label class={table.field}><span>{t('attach-filters-min')}</span><input class={table.control} type="number" min="0" step="any" value={fields().min} onInput={event=>change('min',event.currentTarget.value)}/></label>
          <label class={table.field}><span>{t('attach-filters-max')}</span><input class={table.control} type="number" min="0" step="any" value={fields().max} onInput={event=>change('max',event.currentTarget.value)}/></label>
        </div></Show>
        <Show when={invalid()}><p class={table.validation} role="alert">{invalid()}</p></Show>
        </details>
        <Show when={chips().length}><div class={table.chips} role="group" aria-label={t('attach-filters-active')}><For each={chips()}>{chip=><button class={table.chip} type="button" onClick={chip.clear} aria-label={t('attach-filters-remove',{filter:chip.label})}><span>{chip.label}</span><Icon name="close"/></button>}</For></div></Show>
      </form>
      <Show when={search()}>{result=><div class={table.paging} role="status">
        <span>{t(searching()?'common-attach-scanning':cancelled()?'common-attach-scan-cancelled':searchError()?'common-attach-scan-incomplete':result().limited?'common-attach-scan-limited':'common-attach-scan-complete',{pages:result().pages,count:result().items.length})}</span>
        <Show when={searching()} fallback={<button type="button" onClick={()=>setSearchScope('page')}>{t('common-attach-browse-pages')}</button>}><button type="button" onClick={()=>{searchController?.abort();setSearching(false);setCancelled(true);}}>{t('common-attach-cancel-search')}</button></Show>
      </div>}</Show>
      <Show when={searchError()}><AsyncError error={searchError()} onRetry={()=>void searchAccount()}/></Show>

      <Show when={pinnedOnly()||!data.error} fallback={<AsyncError error={data.error} onRetry={()=>void refetch()}/> }><Show
        when={pinnedOnly()||!data.loading||all().length>0}
        fallback={<InlineLoading message={t('common-attach-loading')} onRetry={()=>void refetch()}/>}
      >
        <Show
          when={filtered().length > 0}
          fallback={<Show when={pinnedOnly()&&pinnedRows.error} fallback={<Show when={searching()||(pinnedOnly()&&pinnedRows.loading)} fallback={<p class="mw-attach__status">{t('common-attach-empty')}</p>}><InlineLoading message={t('common-attach-loading')}/></Show>}><p role="alert">{t('attach-pins-error')} <button type="button" onClick={()=>void retryPins()}>{t('common-retry')}</button></p></Show>}
        >
          <div class={table.wrap} role="region" aria-label={t('common-attach-title')} tabIndex={0}><table class={table.table} aria-label={t('common-attach-title')} aria-busy={data.loading||searching()}><thead><tr><For each={COLUMNS}>{key=><th scope="col" aria-sort={sort().column===key?(sort().ascending?'ascending':'descending'):'none'}><button type="button" class={table.name} onClick={()=>setSort(old=>({column:key,ascending:old.column===key?!old.ascending:true}))}>{t(`common-attach-column-${key}`)}<Show when={sort().column===key}><Icon name={sort().ascending?'chevronUp':'chevronDown'}/></Show></button></th>}</For></tr></thead><tbody>
            <For each={filtered()}>
              {(item) => (
                <tr>
                  <td>
                  <button
                    type="button"
                    class={table.name}
                    data-category={categoryOf(item.mime,item.name)}
                    onClick={() => props.onOpen?.(item)}
                  >
                    <Icon name={inspectFile(item.name,item.mime).icon}/><span>{item.name}</span>
                  </button>
                  <Show when={ownsPins()}><button type="button" class="btn btn--ghost" aria-pressed={isPinned(item)} title={t('attach-pin')} aria-label={t('attach-pin')} disabled={!isPinned(item)&&pins().length>=100} onClick={()=>togglePin(item)}><Icon name="pin"/></button></Show>
                  <For each={inspectFile(item.name,item.mime).warnings}>{warning=><span class={table.warning} title={t(`mail-file-warning-${warning}`)} aria-label={t(`mail-file-warning-${warning}`)}><Icon name="alert"/></span>}</For>
                  </td><td title={item.mime}>{catLabel(categoryOf(item.mime,item.name))}</td><td>{formatSize(item.size)}</td><td>{item.from}</td><td><Show when={props.onOpenMessage} fallback={item.subject}><button type="button" class={table.name} onClick={()=>props.onOpenMessage?.(item)} title={t('common-attach-open-message')}>{item.subject}</button></Show></td><td>{Number.isNaN(Date.parse(item.receivedAt))?'—':formatDate(new Date(item.receivedAt))}</td>
                </tr>
              )}
            </For>
          </tbody></table></div>
        </Show>
      </Show></Show>
      <Show when={props.loadPage && !search() && !pinnedOnly()}><nav class={table.paging} aria-label={t('common-attach-pages')}>
        <button type="button" disabled={data.loading || positions().length===1} onClick={()=>setPositions(old=>old.slice(0,-1))}>{t('common-attach-previous')}</button>
        <span>{t('common-attach-page',{page:positions().length})}</span>
        <button type="button" disabled={data.loading || current()?.nextPosition == null} onClick={()=>{const next=current()?.nextPosition;if(next!=null)setPositions(old=>[...old,next]);}}>{t('common-attach-next')}</button>
      </nav></Show>
    </section>
  );
}

export default Attachments;
