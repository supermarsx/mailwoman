import {createEffect,createMemo,createSignal,For,onCleanup,onMount,Show,type JSX} from 'solid-js';
import {loadCatalog,t,formatNumber,isolate} from '../i18n';
import {withBase} from '../api/basePath.ts';
import {linkedAccountsRevision} from '../api/linkedMailboxSession.ts';
import {workspaceFolders,workspaceMailboxes,workspaceMessages,updateWorkspaceMessages,type WorkspaceAction,type UnifiedRole,type UnifiedRow,type WorkspaceMailbox} from '../api/workspaceMail.ts';
import type {Email,Mailbox} from '../api/jmap-types.ts';
import {useApp} from '../state/context.ts';
import {Icon} from '../components/Icon.tsx';
import {InlineLoading} from '../components/InlineLoading.tsx';
import {MessageRowContent} from '../components/MessageList.tsx';
import {MessageOrderMenu} from '../components/MessageOrderMenu.tsx';
import {createMessageOrder} from '../components/messageOrder.ts';
import {groupMessageRows,groupedWindow} from '../components/messageGrouping.ts';
import {conversationsEnabled} from '../components/conversationPreference.ts';
import {messageTimeZone} from '../components/TimeHelpers.tsx';
import {priorityOf} from '../components/messagePriority.ts';
import type {ThreadVisualRow} from '../components/threads.ts';
import {createContextMenu,type ContextAction} from '../components/ContextMenu.tsx';
import {SelectionBar,type SelectionAction} from '../components/SelectionBar.tsx';
import {toast} from '../components/toast/toast.ts';
import * as rowCss from '../components/messageRow.css.ts';
import * as css from './unifiedMail.css.ts';
import {requestMessageTransfer,transfersEnabled} from '../state/messageTransferRequest.ts';
import {unifiedVirtualMode,unifiedFolderRole,setUnifiedFolderRole,setUnifiedFolderCounts} from '../state/unifiedMailbox.ts';

void loadCatalog('workspace-mail');
const roles:readonly UnifiedRole[]=['inbox','all','sent','drafts','archive','junk','trash'];
interface MailboxPage {target:WorkspaceMailbox;boxes?:Mailbox[];rows:UnifiedRow[];next:number|null;total:number|null;unread:number|null;failed:boolean}
const targetKey=(target:WorkspaceMailbox)=>target.linkId??'primary';
export const unifiedRowKey=(row:UnifiedRow)=>JSON.stringify([row.target.storageAuthority,row.email.id]);
export function unifiedMessageUrl(row:UnifiedRow):string {
 const params=new URLSearchParams({message:row.email.id});if(row.target.linkId)params.set('mailbox',row.target.linkId);
 const folder=row.folderId??Object.keys(row.email.mailboxIds??{}).find(id=>row.email.mailboxIds[id]);if(folder)params.set('folder',folder);
 return withBase(`/?${params.toString()}#/mail`);
}
export default function UnifiedMail(props:{activeLink:string|null}):JSX.Element {
 const app=useApp(),order=createMessageOrder(()=>toast.error(t('mail-context-failed')));
 const [role,setRole]=createSignal<UnifiedRole>('inbox'),[search,setSearch]=createSignal(''),[query,setQuery]=createSignal(''),[account,setAccount]=createSignal(''),[unreadOnly,setUnreadOnly]=createSignal(false);
 const [refresh,setRefresh]=createSignal(0),[targets,setTargets]=createSignal<WorkspaceMailbox[]>([]),[pages,setPages]=createSignal<MailboxPage[]>([]),[loading,setLoading]=createSignal(true),[error,setError]=createSignal(false),[busy,setBusy]=createSignal(false);
 const [selected,setSelected]=createSignal(new Set<string>()),[expanded,setExpanded]=createSignal(new Set<string>()),[top,setTop]=createSignal(0),[height,setHeight]=createSignal(600);
 let controller:AbortController|undefined,scroller:HTMLDivElement|undefined;
 createEffect(()=>{if(unifiedVirtualMode())setRole(unifiedFolderRole());});
 createEffect(()=>{
  const loaded=pages();if(account()||!loaded.length||loaded.length!==targets().length||loaded.some(page=>page.failed||!page.boxes))return;
  const counts=Object.fromEntries(roles.filter(role=>role!=='all').map(role=>{const folders=loaded.flatMap(page=>page.boxes!.filter(box=>box.role===role));return [role,{total:folders.every(box=>typeof box.totalEmails==='number')?folders.reduce((sum,box)=>sum+box.totalEmails,0):null,unread:folders.every(box=>typeof box.unreadEmails==='number')?folders.reduce((sum,box)=>sum+box.unreadEmails,0):null}];}));
  if(role()==='all'&&!query()&&!unreadOnly())counts['all']={total:loaded.every(page=>page.total!==null)?loaded.reduce((sum,page)=>sum+(page.total??0),0):null,unread:loaded.every(page=>page.unread!==null)?loaded.reduce((sum,page)=>sum+(page.unread??0),0):null};
  setUnifiedFolderCounts(old=>({...old,...counts}));
 });
 createEffect(()=>{linkedAccountsRevision();refresh();const active=props.activeLink,request=new AbortController();setTargets([]);setError(false);
  void workspaceMailboxes(active).then(rows=>{if(!request.signal.aborted){if(account()&&!rows.some(row=>targetKey(row)===account()))setAccount('');setTargets(rows);}}).catch(()=>{if(!request.signal.aborted){setError(true);setLoading(false);}});onCleanup(()=>request.abort());
 });
 createEffect(()=>{
  const accounts=targets().filter(target=>!account()||targetKey(target)===account());const currentRole=role(),currentQuery=query(),currentOrder=order.value(),onlyUnread=unreadOnly();
  controller?.abort();const request=new AbortController();controller=request;setPages([]);setSelected(new Set<string>());setExpanded(new Set<string>());setTop(0);if(scroller)scroller.scrollTop=0;
  if(!accounts.length){setLoading(!error());return;}setLoading(true);
  void Promise.all(accounts.map(target=>load({target,rows:[],next:0,total:null,unread:null,failed:false},currentRole,currentQuery,request.signal,currentOrder,onlyUnread))).then(result=>{if(!request.signal.aborted){setPages(result);setLoading(false);}});
  onCleanup(()=>request.abort());
 });
 async function load(page:MailboxPage,folder:UnifiedRole,text:string,signal:AbortSignal,sort=order.value(),onlyUnread=unreadOnly()):Promise<MailboxPage>{
  if(page.next===null)return page;
  try{const boxes=page.boxes??await workspaceFolders(page.target,signal),result=await workspaceMessages(page.target,boxes,folder,text,page.next,signal,sort,onlyUnread);if(result.next!==null&&result.next<=page.next)throw new Error('Mailbox pagination did not advance');const known=new Set(page.rows.map(unifiedRowKey));return {...page,boxes,rows:[...page.rows,...result.rows.filter(row=>!known.has(unifiedRowKey(row)))],next:result.next,total:result.total,unread:result.unread,failed:false};}
  catch{return {...page,failed:true};}
 }
 async function more():Promise<void>{if(loading()||!controller)return;const request=controller;setLoading(true);const result=await Promise.all(pages().map(page=>load(page,role(),query(),request.signal)));if(!request.signal.aborted){setPages(result);setLoading(false);}}
 const rows=createMemo(()=>{
  const sort=order.value(),collator=new Intl.Collator(undefined,{sensitivity:'base',numeric:true});
  const value=(email:Email):string|number=>sort.startsWith('sender')?(email.from?.[0]?.email??''):sort.startsWith('subject')?(email.subject??''):sort.startsWith('size')?(email.size??0):sort.startsWith('importance')?({high:3,normal:2,low:1}[priorityOf(email)]):Date.parse((sort.startsWith('sent-')?email.sentAt:null)||email.receivedAt)||0;
  const ascending=['oldest','sender','subject','sent-oldest','importance-asc','size-asc'].includes(sort);
  return pages().flatMap(page=>page.rows).sort((a,b)=>{const av=value(a.email),bv=value(b.email),comparison=typeof av==='number'&&typeof bv==='number'?av-bv:collator.compare(String(av),String(bv));return (ascending?comparison:-comparison)||unifiedRowKey(a).localeCompare(unifiedRowKey(b));});
 });
 const byKey=createMemo(()=>new Map(rows().map(row=>[unifiedRowKey(row),row])));
 const rowHeight=()=>({compact:76,cozy:94,relaxed:110}[app.density()]);
 const layout=createMemo(()=>groupMessageRows(rows().map(row=>({...row.email,id:unifiedRowKey(row),...(row.email.threadId?{threadId:JSON.stringify([row.target.storageAuthority,row.email.threadId])}:{})})),expanded(),order.group(),rowHeight(),messageTimeZone(),conversationsEnabled()));
 const win=createMemo(()=>groupedWindow(layout(),top(),height(),rowHeight(),layout().rows.length));
 const visible=()=>layout().rows.slice(win().startIndex,win().endIndex);
 const total=()=>pages().length&&pages().every(page=>!page.failed&&page.total!==null)?pages().reduce((sum,page)=>sum+(page.total??0),0):null;
 const unread=()=>pages().length&&pages().every(page=>!page.failed&&page.unread!==null)?pages().reduce((sum,page)=>sum+(page.unread??0),0):null;
 const counter=(value:number|null)=>value===null?'—':formatNumber(value);
 const hasMore=()=>pages().some(page=>page.next!==null);
 function fill():void{if(scroller&&scroller.clientHeight>0&&!loading()&&!pages().some(page=>page.failed)&&hasMore()&&scroller.scrollHeight<=scroller.clientHeight+rowHeight())void more();}
 onMount(()=>{
  if(scroller&&typeof ResizeObserver!=='undefined'){const observer=new ResizeObserver(()=>{if(scroller)setHeight(scroller.clientHeight);fill();});observer.observe(scroller);onCleanup(()=>observer.disconnect());}
  const update=()=>{if(scroller&&scroller.clientHeight>0&&scroller.scrollTop===0&&document.visibilityState==='visible'&&!loading()&&!busy()&&selected().size===0)setRefresh(value=>value+1);};
  const timer=setInterval(update,60_000);window.addEventListener('focus',update);onCleanup(()=>{clearInterval(timer);window.removeEventListener('focus',update);});
 });
 createEffect(()=>{loading();pages();queueMicrotask(fill);});
 const chosen=()=>rows().filter(row=>selected().has(unifiedRowKey(row)));

 function open(row:UnifiedRow):void{window.open(unifiedMessageUrl(row),'_blank','noopener');}
 const members=(visual:ThreadVisualRow):UnifiedRow[]=>visual.kind==='head'?rows().filter(row=>row.email.threadId&&JSON.stringify([row.target.storageAuthority,row.email.threadId])===visual.key):[byKey().get(visual.email.id)!];
 let anchor:string|undefined;
 function toggleVisual(visual:ThreadVisualRow):void{anchor=visual.email.id;const items=members(visual),all=items.every(row=>selected().has(unifiedRowKey(row)));setSelected(old=>{const next=new Set(old);for(const row of items){const key=unifiedRowKey(row);if(all)next.delete(key);else next.add(key);}return next;});}
 function selectRange(visual:ThreadVisualRow,add=false):void{const all=rows(),start=all.findIndex(row=>unifiedRowKey(row)===anchor),end=all.findIndex(row=>unifiedRowKey(row)===visual.email.id);if(start<0||end<0){toggleVisual(visual);return;}setSelected(old=>new Set([...(add?old:[]),...all.slice(Math.min(start,end),Math.max(start,end)+1).map(unifiedRowKey)]));}
 function keyboard(event:KeyboardEvent):void{
  if(event.key==='Escape'){setSelected(new Set<string>());menu.close();event.preventDefault();return;}
  if((event.target as HTMLElement).matches('input'))return;
  if((event.ctrlKey||event.metaKey)&&event.key.toLowerCase()==='a'){event.preventDefault();setSelected(new Set(rows().map(unifiedRowKey)));return;}
  const current=(event.target as HTMLElement).closest<HTMLElement>('[data-unified-index]');if(!current)return;
  const index=Number(current.dataset['unifiedIndex']),last=layout().rows.length-1;
  const next=event.key==='Home'?0:event.key==='End'?last:event.key==='ArrowDown'?index+1:event.key==='ArrowUp'?index-1:event.key==='PageDown'?index+Math.max(1,Math.floor(height()/rowHeight())):event.key==='PageUp'?index-Math.max(1,Math.floor(height()/rowHeight())):null;
  if(next===null)return;event.preventDefault();const target=Math.max(0,Math.min(last,next)),visual=layout().rows[target];if(!visual)return;if(event.shiftKey)selectRange(visual,event.ctrlKey||event.metaKey);else anchor=visual.email.id;
  const offset=layout().tops[target]??0;if(scroller&&(offset<top()||offset+rowHeight()>top()+height())){scroller.scrollTop=Math.max(0,offset-height()/2);setTop(scroller.scrollTop);}
  queueMicrotask(()=>scroller?.querySelector<HTMLButtonElement>(`[data-unified-index="${target}"] .list__row`)?.focus());
 }
 const [context,setContext]=createSignal<UnifiedRow[]>();
 const actionRows=()=>{const items=context(),row=items?.[0];return row?selected().has(unifiedRowKey(row))?chosen():items!:chosen();};
 const boxesFor=(row:UnifiedRow)=>pages().find(page=>page.target.storageAuthority===row.target.storageAuthority)?.boxes??[];
 async function mutate(items:UnifiedRow[],action:WorkspaceAction,destination?:string):Promise<void>{
  if(busy()||!controller||!items.length)return;const request=controller;setBusy(true);const updated=new Set<string>();
  try{
   await Promise.all(pages().map(async page=>{const subset=items.filter(row=>row.target.storageAuthority===page.target.storageAuthority);if(!subset.length)return;try{for(const id of await updateWorkspaceMessages(page.target,page.boxes??[],subset.map(row=>row.email),action,request.signal,destination))updated.add(JSON.stringify([page.target.storageAuthority,id]));}catch{/* Preserve failed rows and their selection. */}}));
   if(request.signal.aborted)return;
   setPages(old=>old.map(page=>{let removed=0,unreadDelta=0;const next=page.rows.flatMap(row=>{
    if(!updated.has(unifiedRowKey(row)))return [row];const wasUnread=row.email.keywords?.['$seen']!==true;
    if(['read','unread','flag','unflag'].includes(action)){const key=action==='read'||action==='unread'?'$seen':'$flagged',keywords={...row.email.keywords};if(action==='read'||action==='flag')keywords[key]=true;else delete keywords[key];if(action==='read'&&wasUnread)unreadDelta--;if(action==='unread'&&!wasUnread)unreadDelta++;if(unreadOnly()&&action==='read'){removed++;return [];}return [{...row,email:{...row.email,keywords}}];}
    const folder=action==='move'?page.boxes?.find(box=>box.id===destination):page.boxes?.find(box=>box.role===action);
    if(role()!=='all'&&folder?.role!==role()){removed++;if(wasUnread)unreadDelta--;return [];}
    return [{...row,...(folder?{folderId:folder.id}:{}),email:{...row.email,mailboxIds:folder?{[folder.id]:true}:row.email.mailboxIds}}];
   });
   const boxes=page.boxes?.map(box=>{let totalChange=0,unreadChange=0;const destinationBox=action==='move'?page.boxes?.find(box=>box.id===destination):page.boxes?.find(box=>box.role===action);
    for(const row of page.rows){if(!updated.has(unifiedRowKey(row)))continue;const wasHere=row.email.mailboxIds[box.id]===true,wasUnread=row.email.keywords?.['$seen']!==true;
     if(action==='read'||action==='unread'){if(wasHere)unreadChange+=(action==='unread'?1:0)-(wasUnread?1:0);}
     else if(['archive','trash','junk','move'].includes(action)){const willBeHere=destinationBox?.id===box.id;totalChange+=Number(willBeHere)-Number(wasHere);if(wasUnread)unreadChange+=Number(willBeHere)-Number(wasHere);}
    }
    return {...box,...(typeof box.totalEmails==='number'?{totalEmails:Math.max(0,box.totalEmails+totalChange)}:{}),...(typeof box.unreadEmails==='number'?{unreadEmails:Math.max(0,box.unreadEmails+unreadChange)}:{})};
   });return {...page,...(boxes?{boxes}:{}),rows:next,next:page.next===null?null:Math.max(0,page.next-removed),total:page.total===null?null:Math.max(0,page.total-removed),unread:page.unread===null?null:Math.max(0,page.unread+unreadDelta)};}));
   setSelected(old=>new Set([...old].filter(key=>!updated.has(key))));
   if(updated.size!==items.length)toast.error(t('unified-mail-action-failed',{count:items.length-updated.size}));else toast.success(t('unified-mail-updated',{count:updated.size}));
  }finally{setBusy(false);}
 }
 const actionsFor=(items:UnifiedRow[]):ContextAction[]=>{
  const first=items[0];if(!first)return [];const singleAccount=items.every(row=>row.target.storageAuthority===first.target.storageAuthority);
  return [
   {id:'open',label:t('mail-context-open-new-tab'),icon:'layoutTabs',disabled:items.length!==1,run:()=>open(first)},
   {id:'mark',label:t('mail-mark-as'),icon:'mailCheck',children:[['read','mail-mark-read','mailRead'],['unread','mail-mark-unread','mail'],['flag','mail-flag-message','flag'],['unflag','mail-unflag-message','flag']].map(([action,label,icon])=>({id:action!,label:t(label!),icon:icon as 'mail',run:()=>mutate(items,action as WorkspaceAction)})),run:()=>{}},
   {id:'archive',label:t('mail-archive'),icon:'archive',run:()=>mutate(items,'archive')},
   {id:'trash',label:t('mail-delete'),icon:'delete',run:()=>mutate(items,'trash')},
   {id:'spam',label:t('mail-spam'),icon:'spam',run:()=>mutate(items,'junk')},
   {id:'move',label:t('mail-reader-move-to'),icon:'moveTo',disabled:!singleAccount,searchLabel:t('mail-move-search'),children:singleAccount?boxesFor(first).filter(box=>box.myRights?.mayAddItems===true&&box.mailwomanSearchQuery===undefined).map(box=>({id:`move:${box.id}`,label:box.name,icon:'folder',run:()=>mutate(items,'move',box.id)})):[],run:()=>{}},
   ...(transfersEnabled()&&targets().length>1?[{id:'transfer',label:t('workspace-transfer-title'),icon:'copyTo' as const,disabled:items.length!==1,run:()=>requestMessageTransfer(first.email,first.target.linkId)}]:[]),
  ];
 };
 const menu=createContextMenu({label:()=>t('mail-more-actions'),scope:()=>JSON.stringify([props.activeLink,role(),account(),unreadOnly(),query(),order.value(),[...selected()].sort()]),actions:()=>actionsFor(actionRows()).map(action=>({...action,disabled:busy()||action.disabled===true})),onError:()=>toast.error(t('mail-context-failed'))});
 const bulkActions=():SelectionAction[]=>actionsFor(chosen()).filter(action=>action.id!=='open').map(action=>({id:action.id,label:action.label,icon:action.icon,disabled:action.disabled===true,...(action.children?.length?{menu:()=>action.children!}:{run:action.run})}));
 return <section class={css.root} aria-label={t('unified-mail-title')} data-mw-surface data-testid="unified-mail">
  <header class={css.bar}><Icon name="mailboxes"/><h2 class={css.title}>{t('unified-mail-title')}</h2><div class={css.counts} role="status"><span>{t('unified-mail-total',{count:counter(total())})}</span><span>{t('unified-mail-unread',{count:counter(unread())})}</span><span>{t('unified-mail-loaded',{count:rows().length})}</span></div><MessageOrderMenu order={order} scope={()=>JSON.stringify([props.activeLink,role(),query()])} class="btn btn--ghost"/><button type="button" class="btn btn--ghost" disabled={loading()||busy()} title={t('unified-mail-refresh')} aria-label={t('unified-mail-refresh')} onClick={()=>setRefresh(value=>value+1)}><Icon name="refresh"/></button></header>
  <form class={css.bar} onSubmit={event=>{event.preventDefault();setQuery(search().trim());}}>
   <label>{t('unified-mail-folder')}<select value={role()} onChange={event=>{const value=event.currentTarget.value as UnifiedRole;setRole(value);if(unifiedVirtualMode())setUnifiedFolderRole(value);}}><For each={roles}>{value=><option value={value}>{t(`unified-mail-role-${value}`)}</option>}</For></select></label>
   <label>{t('unified-mail-account')}<select value={account()} onChange={event=>setAccount(event.currentTarget.value)}><option value="">{t('unified-mail-all-accounts')}</option><For each={targets()}>{target=><option value={targetKey(target)}>{target.label}</option>}</For></select></label>
   <label>{t('unified-mail-search')}<input type="search" value={search()} onInput={event=>{setSearch(event.currentTarget.value);if(!event.currentTarget.value)setQuery('');}}/></label><button type="submit" class="btn btn--ghost" aria-label={t('unified-mail-search')}><Icon name="search"/></button>
   <label><input type="checkbox" checked={unreadOnly()} onChange={event=>setUnreadOnly(event.currentTarget.checked)}/>{t('mail-unread')}</label>
  </form>
  <div class={css.accounts}><For each={pages()}>{page=><button type="button" class="btn btn--ghost" aria-pressed={account()===targetKey(page.target)} onClick={()=>setAccount(account()===targetKey(page.target)?'':targetKey(page.target))}><Icon name="mail"/>{isolate(page.target.label)} <span>{counter(page.unread)} / {counter(page.total)}</span></button>}</For></div>
  <Show when={error()}><p role="alert">{t('unified-mail-load-failed')}</p></Show><For each={pages().filter(page=>page.failed)}>{page=><p role="alert">{t('unified-mail-account-failed',{account:page.target.label})}</p>}</For>
  <div ref={scroller} class={css.scroller} aria-busy={loading()} onScroll={event=>{setTop(event.currentTarget.scrollTop);if(event.currentTarget.scrollHeight-event.currentTarget.scrollTop-event.currentTarget.clientHeight<rowHeight()*3&&!pages().some(page=>page.failed))void more();}} onKeyDown={keyboard}>
   <Show when={rows().length} fallback={<Show when={!loading()&&!error()&&pages().every(page=>!page.failed)}><p>{t('unified-mail-empty')}</p></Show>}>
    <ul class={css.list} style={{height:`${layout().height}px`}} aria-label={t('unified-mail-title')}>
     <For each={layout().headers.filter(header=>header.top>=top()-32&&header.top<top()+height()+32)}>{header=><li class={css.heading} role="presentation" style={{top:`${header.top}px`}}>{header.label} <span>{header.count}</span></li>}</For>
     <For each={visible()}>{visual=>{const row=()=>byKey().get(visual.email.id)!,key=()=>unifiedRowKey(row()),index=()=>layout().rows.indexOf(visual);return <li class={css.slot} data-unified-index={index()} style={{top:`${layout().tops[index()]??0}px`,height:`${rowHeight()}px`}} aria-posinset={index()+1} aria-setsize={layout().rows.length}>
      <input class={css.checkbox} type="checkbox" checked={members(visual).every(item=>selected().has(unifiedRowKey(item)))} aria-label={`${t('mail-selected')}: ${row().email.subject??t('mail-no-subject')}`} onChange={()=>toggleVisual(visual)}/>
      <button type="button" class={`list__row ${rowCss.row} ${css.button}`} classList={{'list__row--unread':visual.unread,'list__row--selected':selected().has(key())}} onClick={event=>{if(event.shiftKey)selectRange(visual,event.ctrlKey||event.metaKey);else if(event.ctrlKey||event.metaKey)toggleVisual(visual);else open(row());}} onContextMenu={event=>{setContext(members(visual));menu.open(event);}} onKeyDown={event=>{if(event.key===' '){event.preventDefault();toggleVisual(visual);}else if(event.key==='ContextMenu'||event.shiftKey&&event.key==='F10'){event.preventDefault();setContext(members(visual));menu.openBelow(event.currentTarget);}}}><MessageRowContent email={row().email} selected={selected().has(key())} replyDraft={undefined} sourceMetadata={row().target.current}/></button>
      <div class={css.provenance}><span class={css.badge}>{isolate(row().target.label)}</span><span class={css.badge}><Icon name="folder"/>{isolate(boxesFor(row()).find(box=>box.id===row().folderId)?.name??boxesFor(row()).filter(box=>row().email.mailboxIds[box.id]).map(box=>box.name).join(', '))}</span></div>
      <button type="button" class={`btn btn--ghost ${css.menu}`} title={t('mail-more-actions')} aria-label={t('mail-more-actions')} onClick={event=>{setContext(members(visual));menu.openBelow(event.currentTarget);}}><Icon name="more"/></button>
      <Show when={visual.kind==='head'}><button type="button" class={`btn btn--ghost ${css.toggle}`} aria-expanded={visual.expanded} onClick={()=>setExpanded(old=>{const next=new Set(old);if(next.has(visual.key))next.delete(visual.key);else next.add(visual.key);return next;})}><Icon name={visual.expanded?'chevronDown':'chevron'}/>{visual.count}</button></Show>
     </li>;}}</For>
    </ul>
   </Show>
   <Show when={loading()}><InlineLoading message={t('common-loading')}/></Show><Show when={hasMore()}><button class="btn btn--ghost" type="button" disabled={loading()} onClick={()=>void more()}>{t('unified-mail-more')}</button></Show>
  </div>
  <SelectionBar count={selected().size} total={rows().length} scope={JSON.stringify([props.activeLink,role(),query()])} clear={()=>setSelected(new Set<string>())} selectAll={()=>setSelected(new Set(rows().map(unifiedRowKey)))} actions={bulkActions()} busy={busy()} returnFocus={()=>scroller?.querySelector<HTMLButtonElement>('.list__row')?.focus()}/>
  {menu.menu()}
 </section>;
}
