import {createSignal,For,Show,onCleanup,createUniqueId} from 'solid-js';
import {accountInsertService,type AccountInsertService,type AccountItem,type AccountItemKind} from './accountInsert.ts';
import {observePreferenceAccount,preferenceAccount,preferenceAccountIsCurrent,type PreferenceAccount} from '../../api/accountPreferenceLifecycle.ts';
import {t} from '../../i18n/index.ts';
import {Icon} from '../Icon.tsx';
import * as css from './insertAccountItem.css.ts';

export interface InsertAccountItemProps {
  onFiles:(files:File[])=>void|Promise<void>;
  disabled?:boolean;
  service?:(owner:PreferenceAccount)=>AccountInsertService;
}
/** Explicit account exports; no hidden send or modification of the selected item. */
export function InsertAccountItem(props:InsertAccountItemProps){
  const id=createUniqueId();const [open,setOpen]=createSignal(false),[busy,setBusy]=createSignal(false),[error,setError]=createSignal('');
  const [kinds,setKinds]=createSignal<AccountItemKind[]>([]),[kind,setKind]=createSignal<AccountItemKind>('message');
  const [items,setItems]=createSignal<AccountItem[]>([]),[selected,setSelected]=createSignal(''),[search,setSearch]=createSignal(''),[limited,setLimited]=createSignal(false);
  let owner:PreferenceAccount|null=null,service:AccountInsertService|null=null,controller:AbortController|null=null,sequence=0;
  function cancel(){sequence++;controller?.abort();controller=null;setBusy(false);}
  function close(){cancel();setOpen(false);setItems([]);setSelected('');setError('');}
  const stop=observePreferenceAccount(next=>{if(owner&&(!next||next.generation!==owner.generation||next.storageAuthority!==owner.storageAuthority)){close();owner=null;service=null;}});
  onCleanup(()=>{stop();cancel();});
  function current(token:number):boolean{return token===sequence&&owner!==null&&preferenceAccountIsCurrent(owner);}
  async function run(action:(signal:AbortSignal,token:number)=>Promise<void>){
    cancel();const token=sequence;controller=new AbortController();const operation=controller;const signal=operation.signal;const timer=setTimeout(()=>operation.abort(),30000);setBusy(true);setError('');
    try{await action(signal,token);}catch{if(current(token))setError(t('mail-insert-account-error'));}finally{clearTimeout(timer);if(current(token))setBusy(false);}
  }
  async function load(next:AccountItemKind,signal:AbortSignal,token:number){
    setItems([]);setSelected('');setSearch('');setKind(next);const page=await service!.list(next,signal);if(!current(token))return;setItems(page.items);setLimited(page.limited);
  }
  function show(){
    if(props.disabled)return;owner=preferenceAccount();if(!owner?.storageAuthority)return;service=(props.service??accountInsertService)(owner);setOpen(true);
    void run(async(signal,token)=>{const available=await service!.capabilities(signal);if(!current(token))return;setKinds(available);if(available.length)await load(available.includes('message')?'message':available[0]!,signal,token);});
  }
  function change(next:AccountItemKind){if(!busy()&&kinds().includes(next))void run((signal,token)=>load(next,signal,token));}
  function attach(){
    const item=items().find(item=>item.id===selected());if(!item||props.disabled||busy())return;
    void run(async(signal,token)=>{const file=await service!.file(kind(),item,signal);if(!current(token)||props.disabled||signal.aborted)return;await props.onFiles([file]);if(current(token))close();});
  }
  const filtered=()=>items().filter(item=>item.label.toLocaleLowerCase().includes(search().trim().toLocaleLowerCase()));
  return <div class={css.root}>
    <button type="button" class="btn btn--ghost" disabled={props.disabled} aria-expanded={open()} aria-controls={id} data-tooltip={t('mail-insert-account-help')} onClick={()=>open()?close():show()}><Icon name="attachment"/>{t('mail-insert-account-title')}</button>
    <Show when={open()}><section id={id} class={css.panel} aria-label={t('mail-insert-account-title')} aria-busy={busy()} onKeyDown={event=>{if(event.key==='Escape'){event.preventDefault();event.stopPropagation();close();}}}>
      <div class={css.header}><strong>{t('mail-insert-account-title')}</strong><button type="button" class="btn btn--ghost" aria-label={t('mail-insert-account-close')} onClick={close}><Icon name="close"/></button></div>
      <Show when={kinds().length>0} fallback={<Show when={!busy()&&!error()}><p>{t('mail-insert-account-unavailable')}</p></Show>}>
        <label>{t('mail-insert-account-kind')}<select value={kind()} disabled={busy()||props.disabled} onChange={event=>change(event.currentTarget.value as AccountItemKind)}><For each={kinds()}>{value=><option value={value}>{t(`mail-insert-account-${value}`)}</option>}</For></select></label>
        <label>{t('mail-insert-account-search')}<input type="search" value={search()} disabled={busy()} onInput={event=>setSearch(event.currentTarget.value)}/></label>
        <p class={css.help}>{t('mail-insert-account-scope')}</p>
        <Show when={limited()}><p role="status">{t('mail-insert-account-limited')}</p></Show>
        <label>{t('mail-insert-account-selection')}<select size={6} value={selected()} disabled={busy()||props.disabled} onChange={event=>setSelected(event.currentTarget.value)}><For each={filtered()}>{item=><option value={item.id}>{item.label}</option>}</For></select></label>
        <Show when={!busy()&&filtered().length===0}><p>{t('mail-insert-account-empty')}</p></Show>
        <button type="button" class="btn btn--primary" disabled={busy()||props.disabled||!selected()} onClick={attach}><Icon name="attachment"/>{t('mail-insert-account-attach')}</button>
      </Show>
      <Show when={busy()}><p role="status">{t('mail-insert-account-loading')}</p></Show>
      <Show when={error()}><p role="alert">{error()}</p></Show>
    </section></Show>
  </div>;
}
