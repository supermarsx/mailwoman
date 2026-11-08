// The folder Properties dialog: a summary header (icon, name, path, type, switches
// and four stat tiles) over a tab rail and a scrolling panel with a fixed footer.
// Tabs: General, Appearance, Remote location, Size and storage, Offline cache,
// Counters and badges (per-folder overrides), Notes and Actions.
// What shows is bounded by the administrator's `properties.*` and `export.*`
// settings (policy.ts); a locked setting shows its lock instead of a control.
import {For,Show,createEffect,createMemo,createSignal,createUniqueId,onCleanup,onMount,type JSX} from 'solid-js';
import {Portal} from 'solid-js/web';
import {useApp} from '../../state/context.ts';
import type {Mailbox} from '../../api/jmap-types.ts';
import {canRename} from '../../api/renameFolder.ts';
import {isolate,loadCatalog,t} from '../../i18n';
import {wireDialogFocus} from '../../modules/contacts/dialogFocus.ts';
import {useRenameContext} from '../RenameFolder.tsx';
import {Icon,type IconName} from '../Icon.tsx';
import {folderSections,type FolderSection} from './policy.ts';
import {useFolderStats} from './useFolderStats.ts';
import {FolderHeader} from './FolderHeader.tsx';
import {General,Appearance,Remote} from './FolderInfoTabs.tsx';
import {Size,Offline} from './FolderStorageTabs.tsx';
import {Overrides,Notes} from './FolderOverridesTab.tsx';
import {Actions} from './FolderActionsTab.tsx';
import * as css from './folderDialog.css.ts';

export {folderTypeKey} from './folderDialogUtil.ts';

/** What the dialog can ask the folder menu to do; an action not supplied is not offered. */
export interface FolderDialogActions {
 onEmpty?:()=>void;onMarkRead?:()=>void;onRename?:()=>void;onMove?:()=>void;onDelete?:()=>void;onRetention?:()=>void;onRefresh?:()=>void;
}

type Tab=FolderSection|'appearance';
const TAB_ICONS:Record<Tab,IconName>={general:'info',appearance:'paint',remote:'globe',size:'chart',offline:'cloud',overrides:'tag',notes:'edit',actions:'more'};
const TAB_KEY='mw.folderProps.tab';
function remembered(account:string):string|null {try{return localStorage.getItem(`${TAB_KEY}:${account}`);}catch{return null;}}
function remember(account:string,tab:string):void {try{localStorage.setItem(`${TAB_KEY}:${account}`,tab);}catch{/* private mode: the tab is simply not remembered */}}

export function FolderPropertiesDialog(props:{box:Mailbox;actions?:FolderDialogActions;onClose:()=>void}):JSX.Element {
 const app=useApp(),titleId=`props-${createUniqueId()}`;
 onMount(()=>{void loadCatalog('properties');void loadCatalog('mail');});
 const tabs=createMemo<readonly Tab[]>(()=>{
  const list=folderSections();
  // Appearance belongs to General: it is offered wherever General is.
  return list.includes('general')?['general','appearance',...list.filter(section=>section!=='general')]:list;
 });
 const account=()=>app.accountId()??'';
 const first=():Tab=>{const saved=remembered(account());return (saved&&tabs().includes(saved as Tab)?saved:tabs()[0]??'general') as Tab;};
 const [active,setActive]=createSignal<Tab>(first());
 createEffect(()=>{if(!tabs().includes(active()))setActive(tabs()[0]??'general');});
 let chosen=false;
 const choose=(tab:Tab)=>{chosen=true;setActive(tab);remember(account(),tab);};
 // The account id can arrive just after the dialog opens: restore the remembered tab then, unless the user already picked one.
 createEffect(()=>{const acc=account();if(chosen||!acc)return;const saved=remembered(acc);if(saved&&tabs().includes(saved as Tab))setActive(saved as Tab);});
 const box=createMemo(()=>app.mailboxes().find(value=>value.id===props.box.id)??props.box);
 const stats=useFolderStats(box);
 const [renaming,setRenaming]=createSignal(false);
 const renameContext=useRenameContext();
 const renamable=()=>canRename(box(),renameContext());
 const tabRefs:HTMLButtonElement[]=[];
 const [narrow,setNarrow]=createSignal(false);
 onMount(()=>{
  if(typeof matchMedia!=='function')return;
  const query=matchMedia('(max-width: 40rem)');setNarrow(query.matches);
  const listen=()=>setNarrow(query.matches);query.addEventListener?.('change',listen);onCleanup(()=>query.removeEventListener?.('change',listen));
 });
 function onTabKey(event:KeyboardEvent):void {
  const list=tabs(),at=list.indexOf(active());
  const next=event.key==='ArrowRight'||event.key==='ArrowDown'?(at+1)%list.length:event.key==='ArrowLeft'||event.key==='ArrowUp'?(at-1+list.length)%list.length:event.key==='Home'?0:event.key==='End'?list.length-1:-1;
  if(next<0)return;event.preventDefault();choose(list[next]!);tabRefs[next]?.focus();
 }
 let element:HTMLDivElement|undefined;
 onMount(()=>{if(element)onCleanup(wireDialogFocus(element,props.onClose));});
 const openMessage=(id:string)=>{props.onClose();void app.openMessage(id);};
 return <Portal><div class={css.backdrop} onClick={event=>{if(event.target===event.currentTarget)props.onClose();}}>
  <div ref={element} class={`${css.dialog} ${css.phone}`} role="dialog" aria-modal="true" aria-labelledby={`${titleId}-hint`} tabIndex={-1} data-testid="folder-properties">
   <span hidden id={`${titleId}-hint`}>{t('props-folder-title',{name:isolate(box().name)})}</span>
   <button type="button" class={css.closeButton} aria-label={t('mail-compose-close')} onClick={props.onClose}><Icon name="close"/></button>
   <FolderHeader box={box()} stats={stats} titleId={titleId} renaming={renaming()} onRenaming={setRenaming}/>
   <div class={css.main}>
    <div class={css.rail} role="tablist" aria-label={t('props-sections')} aria-orientation={narrow()?'horizontal':'vertical'} onKeyDown={onTabKey}>
     <For each={tabs()}>{(tab,index)=><button type="button" role="tab" id={`${titleId}-tab-${tab}`} aria-selected={active()===tab} aria-controls={`${titleId}-panel`} tabindex={active()===tab?0:-1} class={css.tab} ref={el=>{tabRefs[index()]=el;}} onClick={()=>choose(tab)}><Icon name={TAB_ICONS[tab]}/>{t(`props-tab-${tab}`)}</button>}</For>
    </div>
    <div class={css.panel} role="tabpanel" id={`${titleId}-panel`} aria-labelledby={`${titleId}-tab-${active()}`} tabindex="0">
     <Show when={active()==='general'}><General box={box()}/></Show>
     <Show when={active()==='appearance'}><Appearance box={box()}/></Show>
     <Show when={active()==='remote'}><Remote box={box()} stats={stats}/></Show>
     <Show when={active()==='size'}><Size box={box()} stats={stats} onOpenMessage={openMessage}/></Show>
     <Show when={active()==='offline'}><Offline box={box()} stats={stats} onRefresh={props.actions?.onRefresh}/></Show>
     <Show when={active()==='overrides'}><Overrides box={box()} onRetention={props.actions?.onRetention?()=>{props.onClose();props.actions!.onRetention!();}:undefined}/></Show>
     <Show when={active()==='notes'}><Notes box={box()}/></Show>
     <Show when={active()==='actions'}><Actions box={box()} actions={props.actions} onClose={props.onClose} onStartRename={renamable()?()=>setRenaming(true):undefined}/></Show>
    </div>
   </div>
   <footer class={css.footer}>
    <span class={css.footerHint}>{t('props-footer-hint')}</span>
    <button type="button" class="btn btn--primary" data-testid="props-done" onClick={props.onClose}>{t('props-done')}</button>
   </footer>
  </div>
 </div></Portal>;
}
