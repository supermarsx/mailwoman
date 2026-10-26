import {createEffect,For,onMount,Show,type JSX} from 'solid-js';
import {useApp} from '../state/context.ts';
import {createConfiguredClient} from '../api/transport.ts';
import {createFolderView} from './folderView.ts';
import {AsyncPending} from './AsyncState.tsx';
import {senderLabel} from './MessageList.tsx';
import {modifierSelectionEnabled} from './mailInteractions.ts';
import {useActions} from '../keymap/index.ts';
import {loadCatalog,t} from '../i18n/index.ts';
import * as css from './folderPane.css.ts';

export function FolderPane(props:{mailboxId:string;active:boolean;onOpen:(id:string)=>void;registerRefresh?:(refresh:()=>void)=>void}):JSX.Element{
 const app=useApp();
 const box=()=>app.mailboxes().find(value=>value.id===props.mailboxId);
 const view=createFolderView(createConfiguredClient(),props.mailboxId,()=>!!box());
 let viewport:HTMLDivElement|undefined;
 onMount(()=>{props.registerRefresh?.(()=>void view.load());void loadCatalog('folder-tabs');void view.load();});
 createEffect(()=>{if(props.active&&viewport)viewport.scrollTop=view.scroll();});
 function step(delta:number):boolean{
  if(!props.active||!view.current())return false;
  const rows=view.messages();if(!rows.length)return true;
  const selected=[...view.selection()].at(-1),index=rows.findIndex(row=>row.id===selected);
  const row=rows[index<0?(delta>0?0:rows.length-1):Math.max(0,Math.min(rows.length-1,index+delta))];
  if(row){view.select(row.id);props.onOpen(row.id);}return true;
 }
 useActions({next:()=>step(1),previous:()=>step(-1),refresh:()=>{if(!props.active)return false;void view.load();return true;}});
 function key(event:KeyboardEvent,id:string){
  const rows=viewport?.querySelectorAll<HTMLButtonElement>('[data-folder-message]');if(!rows)return;
  const at=[...rows].findIndex(row=>row.dataset['folderMessage']===id);
  const next=event.key==='ArrowDown'?at+1:event.key==='ArrowUp'?at-1:event.key==='Home'?0:event.key==='End'?rows.length-1:null;
  if(next!==null){event.preventDefault();const target=rows[Math.max(0,Math.min(rows.length-1,next))];target?.focus();if(modifierSelectionEnabled()&&event.shiftKey&&target?.dataset['folderMessage'])view.select(target.dataset['folderMessage'],true);}
 }
 return <div class={css.pane}>
  <header class={css.toolbar}><h2>{box()?.name??t('folder-tabs-unavailable')}</h2><button type="button" class="btn btn--ghost" disabled={view.loading()||!box()} onClick={()=>void view.load()}>{t('folder-tabs-refresh')}</button></header>
  <Show when={view.error()}><p role="alert">{t('folder-tabs-error')}</p></Show>
  <Show when={view.loading()&&view.messages().length===0}><AsyncPending variant="list"/></Show>
  <div class={css.viewport} ref={viewport} onScroll={event=>view.setScroll(event.currentTarget.scrollTop)}>
   <ul class={css.list} aria-label={box()?.name??t('folder-tabs-unavailable')}>
    <For each={view.messages()}>{email=><li class={css.row} data-selected={view.selection().has(email.id)} data-unread={!email.keywords?.['$seen']}>
     <input type="checkbox" checked={view.selection().has(email.id)} aria-label={t('folder-tabs-select',{subject:email.subject??''})} onChange={()=>view.select(email.id,false,true)}/>
     <button type="button" class={css.message} data-folder-message={email.id} onKeyDown={event=>key(event,email.id)} onClick={event=>{const extend=modifierSelectionEnabled()&&event.shiftKey,toggle=modifierSelectionEnabled()&&(event.ctrlKey||event.metaKey);view.select(email.id,extend,toggle);if(!extend&&!toggle&&view.current())props.onOpen(email.id);}}>
      <strong>{email.subject||t('mail-no-subject')}</strong><span>{senderLabel(email.from)}</span><span class="meta">{email.preview}</span>
     </button>
    </li>}</For>
   </ul>
   <Show when={!view.loading()&&!view.error()&&view.messages().length===0}><p>{t('folder-tabs-empty')}</p></Show>
  </div>
  <footer class={css.toolbar}><button type="button" class="btn" disabled={view.loading()||view.position()===0} onClick={()=>void view.load(Math.max(0,view.position()-100))}>{t('folder-tabs-previous')}</button><span role="status">{t('folder-tabs-position',{start:view.messages().length?view.position()+1:0,end:view.position()+view.messages().length,total:view.total()??'?',selected:view.selection().size})}</span><button type="button" class="btn" disabled={view.loading()||(view.total()===null?view.messages().length<100:view.position()+view.messages().length>=view.total()!)} onClick={()=>void view.load(view.position()+100)}>{t('folder-tabs-next')}</button></footer>
 </div>;
}
