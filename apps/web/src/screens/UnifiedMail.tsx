import {createEffect,createMemo,createSignal,For,onCleanup,Show,type JSX} from 'solid-js';
import {loadCatalog,t,formatDate} from '../i18n';
import {withBase} from '../api/basePath.ts';
import {linkedAccountsRevision} from '../api/linkedMailboxSession.ts';
import {workspaceFolders,workspaceMailboxes,workspaceMessages,type UnifiedRole,type UnifiedRow,type WorkspaceMailbox} from '../api/workspaceMail.ts';
import type {Mailbox} from '../api/jmap-types.ts';
import {Icon} from '../components/Icon.tsx';
import {InlineLoading} from '../components/InlineLoading.tsx';
import * as css from './attachmentsTable.css.ts';
import {requestMessageTransfer,transfersEnabled} from '../state/messageTransferRequest.ts';

void loadCatalog('workspace-mail');
const roles:readonly UnifiedRole[]=['inbox','all','sent','drafts','archive','junk','trash'];
interface MailboxPage {target:WorkspaceMailbox;boxes?:Mailbox[];rows:UnifiedRow[];next:number|null;failed:boolean}
const targetKey=(target:WorkspaceMailbox)=>target.linkId??'primary';
const rowKey=(row:UnifiedRow)=>JSON.stringify([targetKey(row.target),row.email.id]);
function messageUrl(row:UnifiedRow):string {
 const params=new URLSearchParams({message:row.email.id});
 if(row.target.linkId)params.set('mailbox',row.target.linkId);
 const folder=row.folderId??Object.keys(row.email.mailboxIds??{}).find(id=>row.email.mailboxIds[id]===true);
 if(folder)params.set('folder',folder);
 return withBase(`/?${params.toString()}#/mail`);
}

export default function UnifiedMail(props:{activeLink:string|null}):JSX.Element {
 const [role,setRole]=createSignal<UnifiedRole>('inbox');
 const [search,setSearch]=createSignal('');
 const [query,setQuery]=createSignal('');
 const [selection,setSelection]=createSignal('');
 const [refresh,setRefresh]=createSignal(0);
 const [targets,setTargets]=createSignal<WorkspaceMailbox[]>([]);
 const [pages,setPages]=createSignal<MailboxPage[]>([]);
 const [loading,setLoading]=createSignal(true);
 const [error,setError]=createSignal(false);
 let controller:AbortController|undefined;
 createEffect(()=>{
  linkedAccountsRevision();refresh();
  const active=props.activeLink;
  const request=new AbortController();
  setTargets([]);setError(false);
  void workspaceMailboxes(active).then(rows=>{if(!request.signal.aborted)setTargets(rows);})
   .catch(()=>{if(!request.signal.aborted){setError(true);setLoading(false);}});
  onCleanup(()=>request.abort());
 });
 createEffect(()=>{
  const accounts=targets().filter(account=>!selection()||targetKey(account)===selection());
  const currentRole=role(),currentQuery=query();
  controller?.abort();
  const request=new AbortController();
  controller=request;
  setPages([]);
  if(!accounts.length){setLoading(!error());return;}
  setLoading(true);
  void Promise.all(accounts.map(target=>load({target,rows:[],next:0,failed:false},currentRole,currentQuery,request.signal)))
   .then(result=>{if(!request.signal.aborted){setPages(result);setLoading(false);}});
  onCleanup(()=>request.abort());
 });
 async function load(page:MailboxPage,role:UnifiedRole,search:string,signal:AbortSignal):Promise<MailboxPage>{
  const position=page.next;
  if(position===null)return page;
  try{
   const boxes=page.boxes??await workspaceFolders(page.target,signal);
   const result=await workspaceMessages(page.target,boxes,role,search,position,signal);
   const known=new Set(page.rows.map(rowKey));
   return {...page,boxes,rows:[...page.rows,...result.rows.filter(row=>!known.has(rowKey(row)))],next:result.next,failed:false};
  }catch{return {...page,failed:true};}
 }
 async function more():Promise<void>{
  if(loading()||!controller)return;
  const request=controller;
  setLoading(true);
  const result=await Promise.all(pages().map(page=>load(page,role(),query(),request.signal)));
  if(!request.signal.aborted){setPages(result);setLoading(false);}
 }
 const rows=createMemo(()=>pages().flatMap(page=>page.rows).sort((a,b)=>b.email.receivedAt.localeCompare(a.email.receivedAt)||rowKey(a).localeCompare(rowKey(b))));
 return <section class="mw-attach" data-mw-surface aria-label={t('unified-mail-title')}>
  <header class="mw-attach__bar"><h2>{t('unified-mail-title')}</h2><button type="button" class="btn btn--ghost" onClick={()=>setRefresh(value=>value+1)}><Icon name="refresh"/>{t('unified-mail-refresh')}</button></header>
  <form class={css.filters} onSubmit={event=>{event.preventDefault();setQuery(search().trim());}}>
   <div class={css.filterGrid}>
    <label class={css.field}><span>{t('unified-mail-folder')}</span><select class={css.control} value={role()} onChange={event=>setRole(event.currentTarget.value as UnifiedRole)}><For each={roles}>{value=><option value={value}>{t(`unified-mail-role-${value}`)}</option>}</For></select></label>
    <label class={css.field}><span>{t('unified-mail-account')}</span><select class={css.control} value={selection()} onChange={event=>setSelection(event.currentTarget.value)}><option value="">{t('unified-mail-all-accounts')}</option><For each={targets()}>{target=><option value={targetKey(target)}>{target.label}</option>}</For></select></label>
    <label class={css.field}><span>{t('unified-mail-search')}</span><input type="search" class={css.control} value={search()} onInput={event=>{setSearch(event.currentTarget.value);if(!event.currentTarget.value)setQuery('');}}/></label>
   </div>
   <button type="submit" class="btn"><Icon name="search"/>{t('unified-mail-search')}</button>
  </form>
  <Show when={error()}><p role="alert">{t('unified-mail-load-failed')}</p></Show>
  <For each={pages().filter(page=>page.failed)}>{page=><p role="status">{t('unified-mail-account-failed',{account:page.target.label})}</p>}</For>
  <Show when={rows().length>0} fallback={<Show when={!loading()&&!error()&&pages().every(page=>!page.failed)}><p>{t('unified-mail-empty')}</p></Show>}>
   <div class={css.wrap}><table class={css.table} aria-busy={loading()}><thead><tr>
    <th scope="col">{t('unified-mail-account')}</th><th scope="col">{t('unified-mail-from')}</th><th scope="col">{t('unified-mail-subject')}</th><th scope="col">{t('unified-mail-received')}</th>
    <Show when={transfersEnabled()&&targets().length>1}><th scope="col">{t('unified-mail-actions')}</th></Show>
   </tr></thead><tbody><For each={rows()}>{row=><tr>
    <td>{row.target.label}</td><td>{row.email.from?.[0]?.name||row.email.from?.[0]?.email||'—'}</td>
    <td><a class={css.name} href={messageUrl(row)} target="_blank" rel="noopener">{row.email.subject||t('mail-no-subject')}</a><Show when={row.email.hasAttachment}><Icon name="attachment"/></Show></td>
    <td>{Number.isNaN(Date.parse(row.email.receivedAt))?'—':formatDate(new Date(row.email.receivedAt),{dateStyle:'medium',timeStyle:'short'})}</td>
    <Show when={transfersEnabled()&&targets().length>1}><td><button type="button" class="btn btn--ghost" title={t('workspace-transfer-title')} aria-label={t('workspace-transfer-title')} onClick={()=>requestMessageTransfer(row.email,row.target.linkId)}><Icon name="moveTo"/></button></td></Show>
   </tr>}</For></tbody></table></div>
  </Show>
  <Show when={loading()}><InlineLoading message={t('common-loading')}/></Show>
  <Show when={pages().some(page=>page.next!==null)}><button class="btn btn--ghost" type="button" disabled={loading()} onClick={()=>void more()}>{t('unified-mail-more')}</button></Show>
 </section>;
}
