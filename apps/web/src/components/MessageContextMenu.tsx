import {createEffect,createSignal,onCleanup,untrack,type Accessor} from 'solid-js';
import {useApp} from '../state/context.ts';
import {observePreferenceAccount,preferenceAccount} from '../api/accountPreferenceLifecycle.ts';
import {loadCatalog,t} from '../i18n';
import {relatedSearchValue,relatedSenderDomain} from './messageSearch.ts';
import type {Email} from '../api/jmap-types.ts';
import {snoozePresets} from './MessageActions.tsx';
import {mailFeature,mailFeatureOffTitle,spamFolderHidden} from '../state/mailFeatures.ts';
import {copyMessageToFolder,copyText,quickPrint,replyFromList} from './messageQuickActions.ts';
import {pickerOptions} from './FolderBulkDialogs.tsx';
import {createContextMenu,type ContextAction} from './ContextMenu.tsx';
import {createConfiguredClient} from '../api/transport.ts';
import {loadAttachmentPolicy,type AttachmentPolicy} from '../api/attachmentPolicy.ts';
import {fetchMessageExtras,type MessageExtras} from './messageRelations.ts';
import {blockedReason,describe,downloadOne,downloadZip,sizeLabel} from './messageAttachmentDownload.ts';
import {focusMessageInFolder} from './messageFocus.ts';
import {formatDateWith} from '../i18n/format.ts';
import {downloadAttachmentsOf} from './messageAttachmentsBulk.ts';
/** "Always move to Focused / Other" for a message's sender (t94), while the
 * Focused inbox is on. Shared by this menu (the row menu too) and the reader. */
export function focusActions(app:Partial<Pick<ReturnType<typeof useApp>,'focusedInbox'|'senderFocus'|'alwaysMoveSender'>>,message:Email):ContextAction[] {
 const from=(message.from?.[0]?.email??'').trim();
 // An app without the Focused inbox (a detached pane's reduced state) offers nothing.
 const {focusedInbox,senderFocus,alwaysMoveSender}=app;
 if(!focusedInbox||!senderFocus||!alwaysMoveSender||!focusedInbox()||from==='')return [];
 return (['focused','other'] as const).map(tab=>({id:`focus:${tab}`,label:t(tab==='focused'?'mail-focus-always-focused':'mail-focus-always-other'),icon:tab==='focused'?'star':'inbox',checked:senderFocus(from)===tab,run:()=>alwaysMoveSender(from,tab)}));
}
/** Always targets the indicated message, not an unrelated existing selection. */
/** A conversation head's menu acts on the conversation: `threadMode` says the menu was opened on a head, `threadMembers` lists the loaded messages of its thread. */
export interface ThreadMenuOptions {threadMode:Accessor<boolean>;threadMembers:(email:Email)=>Email[]}
export function createMessageContextMenu(email:Accessor<Email|undefined>,drafts?:{replyDraft:(emailId:string)=>{id:string}|undefined;onOpenReplyDraft?:(id:string)=>void},fetchExtras?:typeof fetchMessageExtras,threads?:ThreadMenuOptions) {
  void loadCatalog('search'); // t112: its ids show wherever it is mounted
 const app=useApp();
 // Attachment parts and reply/forward targets are not in the list row: fetched once per message when the menu opens, kept for the session.
 const [extras,setExtras]=createSignal<ReadonlyMap<string,MessageExtras|'failed'>>(new Map());
 const [policy,setPolicy]=createSignal<AttachmentPolicy>();
 const requested=new Set<string>();
 const prefetch=(message:Email)=>{
  const account=app.accountId(),key=`${account}:${message.id}`;
  if(!account||requested.has(key)||!(message.hasAttachment||message.keywords?.['$answered']||message.keywords?.['$forwarded']))return;
  requested.add(key);
  const sent=new Set(app.mailboxes().filter(box=>box.role==='sent').map(box=>box.id));
  void (fetchExtras??fetchMessageExtras)(createConfiguredClient(),account,message,sent).then(value=>setExtras(old=>new Map(old).set(key,value)),()=>{requested.delete(key);setExtras(old=>new Map(old).set(key,'failed'));});
  if(message.hasAttachment&&!policy())void loadAttachmentPolicy().then(setPolicy,()=>undefined);
 };
 const folderOf=(related:Email)=>Object.keys(related.mailboxIds??{}).find(box=>related.mailboxIds[box]&&app.mailboxes().some(value=>value.id===box));
 const goTo=(related:Email,from?:Email)=>{const folder=folderOf(related);if(folder)void focusMessageInFolder(app,folder,related.id,{open:true,...(related.threadId!==undefined&&related.threadId===from?.threadId?{thread:true}:{})});};
 /** A conversation head acts on every loaded message of its thread: attachments grouped per message, and Go to reply / forward from the message that was answered or forwarded. */
 const threadRows=(rows:ContextAction[],members:Email[]):void=>{
  const account=app.accountId();
  const state=(member:Email)=>extras().get(`${account}:${member.id}`);
  const loadedOf=(member:Email)=>{const found=state(member);return found!==undefined&&found!=='failed'?found:undefined;};
  const who=(member:Email)=>`${formatDateWith(member.sentAt??member.receivedAt,{dateStyle:"medium",timeStyle:"short"})} · ${member.from?.[0]?.name||member.from?.[0]?.email||''}`;
  const detail=(related:Email)=>`${formatDateWith(related.sentAt??related.receivedAt,{dateStyle:"medium",timeStyle:"short"})} · ${(related.to??[]).map(a=>a.name||a.email).join(', ')||t('mail-no-recipients')}`;
  const jump=(rowId:string,keyword:string,label:string,icon:'reply'|'forward',pick:(extra:MessageExtras)=>Email[],missing:string)=>{
   const sources=members.filter(member=>member.keywords?.[keyword]===true);
   if(!sources.length)return;
   const found=new Map<string,{owner:Email;value:Email}>();
   for(const owner of sources)for(const value of pick(loadedOf(owner)??{attachments:[],replies:[],forwards:[]}))if(folderOf(value)&&!found.has(value.id))found.set(value.id,{owner,value});
   const targets=[...found.values()],pending=sources.some(member=>state(member)===undefined);
   const at=rows.findIndex(row=>row.id==='forward')+1+(rows.some(row=>row.id==='open-reply-draft')?1:0)+(rowId==='go-forward'&&rows.some(row=>row.id==='go-reply')?1:0);
   if(!targets.length)rows.splice(at,0,{id:rowId,label,icon,disabled:true,title:pending?t('mail-context-searching'):missing,run:()=>{}});
   else if(targets.length===1)rows.splice(at,0,{id:rowId,label,icon,run:()=>goTo(targets[0]!.value,targets[0]!.owner)});
   else rows.splice(at,0,{id:rowId,label,icon,run:()=>{},children:targets.map((target,i)=>({id:`${rowId}:${i}`,label:`${who(target.owner)} → ${detail(target.value)}`,icon,run:()=>goTo(target.value,target.owner)}))});
  };
  jump('go-reply','$answered',t('mail-context-go-reply'),'reply',extra=>extra.replies,t('mail-context-reply-not-found'));
  jump('go-forward','$forwarded',t('mail-context-go-forward'),'forward',extra=>extra.forwards,t('mail-context-forward-not-found'));
  const withFiles=members.filter(member=>member.hasAttachment);
  if(!withFiles.length)return;
  const off=!mailFeature('allowDownload');
  const groups=withFiles.flatMap(member=>{const loaded=loadedOf(member);return loaded&&loaded.attachments.length?[{member,emailId:member.id,label:member.subject??'',items:loaded.attachments.map(describe)}]:[];});
  const base={icon:'download' as const,separatorBefore:true};
  if(off||!groups.length){rows.push({...base,id:'attachments',label:t('mail-context-download-attachments'),disabled:true,title:off?mailFeatureOffTitle():withFiles.some(member=>state(member)===undefined)?t('mail-context-loading-attachments'):t('mail-context-attachments-failed'),run:()=>{}});return;}
  rows.push({...base,id:'attachments',label:t('mail-context-download-attachments'),run:()=>{},children:[
   ...groups.map(group=>({id:`attachments:msg:${group.emailId}`,label:`${who(group.member)} (${group.items.length})`,icon:'attachment' as const,run:()=>{},children:[
    ...group.items.map((item,i)=>{const why=blockedReason(policy(),item);return {id:`attachments:msg:${group.emailId}:${i}`,label:`${item.name} (${sizeLabel(item)})`,icon:'attachment' as const,...(why?{disabled:true,title:why}:{title:`${item.mime} · ${sizeLabel(item)}`}),run:()=>downloadOne(app,group.emailId,item)};}),
    {id:`attachments:msg:${group.emailId}:all`,label:t('mail-context-download-all'),icon:'download' as const,separatorBefore:true,run:()=>downloadZip(app,[group],policy())}]})),
   {id:'attachments:all',label:t('mail-context-download-thread-attachments',{count:groups.reduce((sum,group)=>sum+group.items.length,0)}),icon:'download',separatorBefore:true,run:()=>downloadZip(app,groups,policy())}]});
 };
 const membersOf=(message:Email):Email[]=>{const list=threads?.threadMode()?threads.threadMembers(message):[];return list.length>1?[...list].sort((a,b)=>(a.sentAt??a.receivedAt).localeCompare(b.sentAt??b.receivedAt)):[];};
 const menu=createContextMenu({label:()=>t('mail-more-actions'),scope:()=>`${app.accountId()}:${preferenceAccount()?.generation}:${app.selectedMailboxId()}:${email()?.id}`,actions:()=>{
  const owner=preferenceAccount();if(!owner||owner.id!==app.accountId())return [];
  const message=app.messages().find(value=>value.id===email()?.id);if(!message)return [];const id=message.id;const busy=app.listLoading();
  const rows:ContextAction[]=[
   {id:'open',label:t('mail-context-open-message'),icon:'mail',run:()=>app.openMessage(id)},
   {id:'read',label:t(message.keywords?.$seen?'mail-mark-unread':'mail-mark-read'),icon:'mailRead',run:()=>app.markRead(id,!message.keywords?.$seen)},
   {id:'flag',label:t(message.keywords?.$flagged?'mail-unflag-message':'mail-flag-message'),icon:'flag',run:()=>app.flagMessage(id,!message.keywords?.$flagged)},
   {id:'pin',label:t(message.pinned?'mail-unpin':'mail-pin'),icon:'pin',run:()=>app.pinMessage(id,!message.pinned)},
  ];
  // Reply, reply all and forward go through the reader (it owns decryption, quoting and threading); the feature switches disable them with the usual tooltip.
  rows.splice(1,0,
   {id:'reply',label:t('mail-context-reply'),icon:'reply',separatorBefore:true,...(mailFeature('allowReply')?{}:{disabled:true,title:mailFeatureOffTitle()}),run:()=>{void replyFromList(app,id,'reply');}},
   {id:'reply-all',label:t('mail-context-reply-all'),icon:'replyAll',...(mailFeature('allowReply')?{}:{disabled:true,title:mailFeatureOffTitle()}),run:()=>{void replyFromList(app,id,'replyAll');}},
   {id:'forward',label:t('mail-context-forward'),icon:'forward',...(mailFeature('allowForward')?{}:{disabled:true,title:mailFeatureOffTitle()}),run:()=>{void replyFromList(app,id,'forward');}});
  {const draft=drafts?.onOpenReplyDraft?drafts.replyDraft(id):undefined,openDraft=drafts?.onOpenReplyDraft;
   if(draft&&openDraft){const at=rows.findIndex(row=>row.id==='forward');rows.splice(at+1,0,{id:'open-reply-draft',label:t('mail-reply-draft-open'),icon:'compose',run:()=>openDraft(draft.id)});}}
  const members=membersOf(message);
  if(members.length)threadRows(rows,members);
  else {const found=extras().get(`${app.accountId()}:${id}`);
   const loaded=found!==undefined&&found!=='failed'?found:undefined;
   const target=(related:Email)=>Object.keys(related.mailboxIds??{}).find(box=>related.mailboxIds[box]&&app.mailboxes().some(value=>value.id===box));
   const go=(related:Email)=>goTo(related,message);
   const detail=(related:Email)=>`${formatDateWith(related.sentAt??related.receivedAt,{dateStyle:"medium",timeStyle:"short"})} · ${(related.to??[]).map(a=>a.name||a.email).join(', ')||t('mail-no-recipients')}`;
   const jump=(rowId:string,keyword:string,label:string,icon:'reply'|'forward',list:Email[]|undefined,missing:string)=>{
    if(message.keywords?.[keyword]!==true)return;
    const targets=(list??[]).filter(value=>target(value));
    const at=rows.findIndex(row=>row.id==='forward')+1+(rows.some(row=>row.id==='open-reply-draft')?1:0)+(rowId==='go-forward'&&rows.some(row=>row.id==='go-reply')?1:0);
    if(!loaded)rows.splice(at,0,{id:rowId,label,icon,disabled:true,title:found==='failed'?missing:t('mail-context-searching'),run:()=>{}});
    else if(!targets.length)rows.splice(at,0,{id:rowId,label,icon,disabled:true,title:missing,run:()=>{}});
    else if(targets.length===1)rows.splice(at,0,{id:rowId,label,icon,run:()=>go(targets[0]!)});
    else rows.splice(at,0,{id:rowId,label,icon,run:()=>{},children:targets.map((value,i)=>({id:`${rowId}:${i}`,label:detail(value),icon,run:()=>go(value)}))});
   };
   jump('go-reply','$answered',t('mail-context-go-reply'),'reply',loaded?.replies,t('mail-context-reply-not-found'));
   jump('go-forward','$forwarded',t('mail-context-go-forward'),'forward',loaded?.forwards,t('mail-context-forward-not-found'));
   if(message.hasAttachment){
    const off=!mailFeature('allowDownload'),pending=!loaded;
    const items=(loaded?.attachments??[]).map(describe);
    const base={icon:'download' as const,separatorBefore:true};
    const rowFor=(item:ReturnType<typeof describe>,i:number):ContextAction=>{const why=blockedReason(policy(),item);return {id:`attachment:${i}`,label:t('mail-context-download-named',{name:item.name}),icon:'attachment',...(off||why?{disabled:true,title:off?mailFeatureOffTitle():why!}:{title:`${item.mime} · ${sizeLabel(item)}`}),run:()=>downloadOne(app,id,item)};};
    if(pending||off)rows.push({...base,id:'attachments',label:t('mail-context-download-attachments'),disabled:true,title:off?mailFeatureOffTitle():found==='failed'?t('mail-context-attachments-failed'):t('mail-context-loading-attachments'),run:()=>{}});
    else if(items.length===1)rows.push({...base,...rowFor(items[0]!,0),id:'attachments'});
    else if(items.length>1)rows.push({...base,id:'attachments',label:t('mail-context-download-attachments'),run:()=>{},children:[...items.map((item,i)=>({...rowFor(item,i),label:`${item.name} (${sizeLabel(item)})`})),{id:'attachments:all',label:t('mail-context-download-all'),icon:'download',separatorBefore:true,run:()=>downloadZip(app,[{emailId:id,label:message.subject??id,items}],policy())}]});}}
  {const selected=app.selectedIds(),chosen=selected.has(id)&&selected.size>1?app.messages().filter(value=>selected.has(value.id)):[];
   if(chosen.some(value=>value.hasAttachment)){
    const off=!mailFeature('allowDownload');
    rows.push({id:'attachments-selected',label:t('mail-context-download-selected-attachments',{count:chosen.filter(value=>value.hasAttachment).length}),icon:'download',separatorBefore:!rows.some(row=>row.id==='attachments'),...(off?{disabled:true,title:mailFeatureOffTitle()}:{}),run:()=>downloadAttachmentsOf(app,createConfiguredClient(),chosen.filter(value=>value.hasAttachment),policy())});
   }}
  {const current=new Set(Object.keys(message.mailboxIds??{}).filter(key=>message.mailboxIds[key]));
   const targets=pickerOptions(app.mailboxes(),box=>!current.has(box.id)&&box.mailwomanSearchQuery===undefined&&box.myRights?.mayAddItems!==false&&!spamFolderHidden(box));
   const copies:ContextAction[]=[];
   if(targets.length)copies.push({id:'copy:folder',label:t('mail-context-copy-to-folder'),icon:'copy',run:()=>{},children:targets.map(target=>({id:`copy:to:${target.id}`,label:target.label,icon:'folder',run:async()=>{if(!await copyMessageToFolder(app,id,target.id,app.refreshFolders))app.showToast('error',t('mail-context-failed'));}}))});
   const sender=(message.from?.[0]?.email??'').trim();
   const text=(label:string,value:string,action:string):ContextAction=>({id:`copy:${action}`,label,icon:'copy',run:async()=>{const ok=await copyText(value);app.showToast(ok?'success':'error',t(ok?'mail-context-copied':'mail-context-copy-failed'));}});
   if(sender)copies.push(text(t('mail-context-copy-sender'),sender,'sender'));
   if(message.subject)copies.push(text(t('mail-context-copy-subject'),message.subject,'subject'));
   if(copies.length)rows.push({id:'copy',label:t('mail-context-copy'),icon:'copy',children:copies,run:()=>{}});}
  if(mailFeature('allowPrint'))rows.push({id:'quick-print',label:t('mail-context-quick-print'),icon:'print',run:()=>quickPrint(app,id)});
  else rows.push({id:'quick-print',label:t('mail-context-quick-print'),icon:'print',disabled:true,title:mailFeatureOffTitle(),run:()=>{}});
  const searches:ContextAction[]=[];
  for(const field of ['from','subject'] as const){const value=relatedSearchValue(message,field);if(value)searches.push({id:`search:${field}`,label:t(field==='from'?'mail-context-search-sender':'mail-context-search-subject'),icon:'search',run:()=>app.searchMessageField(field,value)});}
  // t88: scope a search to the message's folder (its chip shows in the filter bar).
  {const folder=Object.keys(message.mailboxIds??{}).find(id=>message.mailboxIds[id]&&app.mailboxes().some(box=>box.id===id));if(folder)searches.push({id:'search:in',label:t('search-refine-folder'),icon:'search',run:()=>app.searchMessageField('in',folder)});}
  const domain=relatedSenderDomain(message);if(domain&&app.supportsFromDomainSearch())searches.splice(1,0,{id:'search:from-domain',label:t('mail-context-search-domain'),icon:'domains',run:()=>app.searchMessageField('from-domain',domain)});
  if(searches.length)rows.push({id:'search',label:t('search-refine-menu'),icon:'search',separatorBefore:true,children:searches,run:()=>{}});
  // t94: the sender's Focused inbox override; the current choice is checked.
  for(const row of focusActions(app,message))rows.push(row);
  for(const [role,label,icon,run] of [
   ['archive','mail-archive','archive',()=>app.archiveMessage(id)],
   ['junk','mail-mark-spam','spam',()=>app.markSpam(id)],
   ['trash','mail-delete','delete',()=>app.trashMessage(id)],
  ] as const)if(app.mailboxes().some(box=>box.role===role&&!message.mailboxIds[box.id]&&!spamFolderHidden(box)))rows.push({id:role,label:t(label),icon,run});
  // Hand-set answered / forwarded marks (the keywords an IMAP server keeps as flags too).
  {const selected=app.selectedIds(),ids=selected.has(id)&&selected.size>1?[...selected]:[id];
   const targets=app.messages().filter(value=>ids.includes(value.id));
   const all=(keyword:string)=>targets.length>0&&targets.every(value=>value.keywords?.[keyword]===true);
   const answered=all('$answered'),forwarded=all('$forwarded');
   rows.push({id:'mark-answered',label:t(answered?'mail-mark-unanswered':'mail-mark-answered'),icon:'reply',separatorBefore:true,run:()=>app.markReplyState(ids,'$answered',!answered)},
    {id:'mark-forwarded',label:t(forwarded?'mail-mark-not-forwarded':'mail-mark-forwarded'),icon:'forward',run:()=>app.markReplyState(ids,'$forwarded',!forwarded)});}
  rows.push({id:'snooze',label:t('mail-snooze'),icon:'clock',separatorBefore:true,run:()=>{},children:snoozePresets().map(preset=>({id:`snooze:${preset.labelId}`,label:t(preset.labelId),icon:'clock',run:()=>app.snoozeMessage(id,snoozePresets().find(value=>value.labelId===preset.labelId)!.at)}))});
  if(message.snoozedUntil)rows.push({id:'unsnooze',label:t('mail-unsnooze'),icon:'clock',run:()=>app.unsnoozeMessage(id)});
  rows.push({id:'followup',label:t(message.followUpAt?'mail-clear-flag':'mail-flag'),icon:'flag',run:()=>app.setFollowUp(id,message.followUpAt?null:new Date(Date.now()+86400000).toISOString())});
  if(app.tags().length)rows.push({id:'labels',label:t('mail-label'),icon:'tag',run:()=>{},children:app.tags().map(tag=>({id:`tag:${tag.id}`,label:tag.name,icon:message.keywords?.[tag.id]?'check':'tag',checked:message.keywords?.[tag.id]===true,run:()=>message.keywords?.[tag.id]?app.removeTag(id,tag.id):app.applyTag(id,tag.id)}))});
  if(mailFeature('allowDownload'))rows.push({id:'download' ,label:t('mail-context-download'),icon:'download',run:()=>app.exportMessage(id)});
  return rows.map(row=>({...row,disabled:busy||row.disabled===true}));
 },restoreFocus:trigger=>{const row=Array.from(document.querySelectorAll<HTMLElement>('.list__row')).find(value=>value.dataset['emailId']===email()?.id);const more=trigger?.classList.contains('msg-more__toggle')?row?.closest('.list__slot')?.querySelector<HTMLElement>('.msg-more__toggle'):undefined;(more??row)?.focus();},onError:()=>app.showToast('error',t('mail-context-failed'))});
 createEffect(()=>{const message=email();if(menu.isOpen()&&message)untrack(()=>{const current=app.messages().find(value=>value.id===message.id)??message;prefetch(current);for(const member of membersOf(current))prefetch(member);});});
 onCleanup(observePreferenceAccount(()=>menu.close(false)));
 return menu;
}
