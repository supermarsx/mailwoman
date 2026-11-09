// The reader's action row (t57): every message action on ONE line, with the
// actions that do not fit folded into a "More" menu instead of wrapping.
//
// Reply, Reply all and Forward come first; View conversation (only for a thread
// of more than one message) and Review links (only when the body has links,
// with their count as a badge) follow; then the actions on the message itself
// and the view-mode dropdown (ReaderViewMode.tsx). Each action has a fold priority
// (`PRIORITY`): when the row is too narrow the least important ones move into
// More, in that order, so at phone width the row is still one line of icons.
// Both dialogs (conversation, link review) are the existing themed modals.

import {createEffect,createMemo,createSignal,For,onCleanup,onMount,Show,type JSX} from 'solid-js';
import {useRealtime} from '../realtime/context.ts';
import {ReaderMoveMenu,moveActions} from './ReaderMoveMenu.tsx';
import {ReaderSearchMenu,relatedSearchActions} from './search/ReaderSearchMenu.tsx';
import {useApp} from '../state/context.ts';
import {settings} from '../api/settings.ts';
import {createReplyDraftIndex} from './compose/replyDrafts.ts';
import {mailFeature} from '../state/mailFeatures.ts';
import type {Email} from '../api/jmap-types.ts';
import type {ReplyMode} from './compose/reply.ts';
import type {ComposeInitial} from './compose/reply.ts';
import type {MessageLink} from '../viewers/messageLinks.ts';
import type {SecurityMode} from '../viewers/max-security.ts';
import {ReaderViewMode,viewModeActions} from './ReaderViewMode.tsx';
import {SweepDialog} from './SweepDialog.tsx';
import {ReaderAction} from './ReaderAction.tsx';
import {createConversation} from './Conversation.tsx';
import {createLinkReview} from './MessageLinks.tsx';
import {createContextMenu,type ContextAction} from './ContextMenu.tsx';
import {foldActions} from './readerToolbarFit.ts';
import {READER_ACTIONS,readerActionPlacement,readerMenuActionId} from './readerActionPlacement.ts';
import {Icon,type IconName} from './Icon.tsx';
import * as a11y from './mailA11y.css.ts';
import * as css from './readerToolbar.css.ts';
import {t} from '../i18n/index.ts';
import {zoomRowAction} from './ReaderZoom.tsx';
import {focusActions} from './MessageContextMenu.tsx';
import {openMessageProperties} from './properties/host.tsx';
import {dialogActionsFor,messageActionRows} from './messageMenuActions.ts';
import {exportFromMenu} from './exportDialog/openExport.ts';
import {saveRowsFor} from '../modules/contacts/saveFromMessage.rows.ts';
import {messageDialogEnabled} from './properties/policy.ts';
import {colorRowAction,type ReaderColorProps} from './ReaderColorMenu.tsx';
import {RawMessageHeaders} from './RawMessageHeaders.tsx';
import {ACTION_ICONS} from '../actionIcons.ts';
import {isDraftMessage} from './draftMessage.ts';
import { loadCatalog } from '../i18n/catalog.ts';
// i18n: the ids this file shows live in these catalogs, which load with it.
void loadCatalog('properties'); void loadCatalog('search');

/** Fold order: lower stays on the row longer. */
export const PRIORITY: Record<string,number> = {
 reply:0,'reply-all':1,forward:2,'edit-draft':-1,'open-draft':2.5,delete:3,archive:4,move:4.5,conversation:5,links:6,read:7,flag:8,pin:9,spam:10,security:11,headers:11.5,sweep:12,print:13,export:14,properties:15,zoom:10.5,search:6.5,colors:10.7,
};

interface RowAction {id:string;groupEnd?:boolean;inline:()=>JSX.Element;menu:()=>ContextAction[]}

/** Promote a menu descriptor without flattening its submenu or losing its live state. */
function ReaderMenuAction(props:{id:string;action:()=>ContextAction|undefined;scope:()=>string;onError:()=>void}):JSX.Element {
 let button:HTMLButtonElement|undefined;
 const action=()=>props.action();
 const children=()=>action()?.children??[];
 const menu=createContextMenu({label:()=>action()?.label??'',scope:props.scope,actions:children,searchLabel:()=>action()?.searchLabel??'',onError:props.onError,restoreFocus:()=>button?.focus()});
 return <>
  <ReaderAction icon={action()?.icon??'more'} label={action()?.label??''} help={action()?.title??action()?.label??''} testId={`reader-${props.id}`}
   disabled={action()?.disabled===true} pressed={action()?.checked}
   {...(children().length?{popup:'menu' as const,expanded:menu.isOpen()}:{})}
   onClick={event=>{
    const trigger=event.currentTarget as HTMLButtonElement;button=trigger;const current=action();if(!current||current.disabled)return;
    if(current.children?.length){menu.openBelow(trigger);return;}
    try{void Promise.resolve(current.run()).catch(props.onError);}catch{props.onError();}
   }}/>
  {menu.menu()}
 </>;
}

export function ReaderToolbar(props:{
 email:Email;
 /** Open a composer answering or forwarding this message. */
 onReply:(mode:ReplyMode)=>void;
 /** True while a reply is being prepared; the three reply buttons are disabled. */
 preparing:boolean;
 onPrint:()=>void;
 /** False while this reader is a background tab: dialogs are not shown. */
 active:boolean;
 links:MessageLink[];
 linksTruncated:boolean;
 /** Identity of the displayed document; a new one closes the link review. */
 linkScope:string;
 /** Exact raw-message download URL and account/message authority scope. */
 headersUrl?:string|null;
 headersScope?:string;
 onCompose?:(draft:ComposeInitial)=>void;
 /** Opens the saved reply to this message; the action appears only while one exists. */
 onOpenReplyDraft?:(draftId:string)=>void;
 /** Opens this message in the composer when it is a draft; the action appears only then. */
 onEditDraft?:(messageId:string)=>void;
 security:{value:SecurityMode;floor:SecurityMode|null;onChange:(mode:SecurityMode)=>void};
 /** Per-message colour mode (the same modes as Settings > Reader); omitted when the reader offers none. */
 colors?:ReaderColorProps;
}):JSX.Element {
 const app=useApp();
 const realtime=useRealtime();
 const [sweeping,setSweeping]=createSignal(false);
 const [headersOpen,setHeadersOpen]=createSignal(false);
 const id=()=>props.email.id;
 const replyDrafts=createReplyDraftIndex(app.accountId);
 const isDraft=()=>props.onEditDraft!==undefined&&isDraftMessage(props.email,app.mailboxes());
 const replyDraft=()=>props.onOpenReplyDraft?replyDrafts().get(id()):undefined;
 const sender=()=>props.email.from?.[0]?.email??'';
 const pinned=()=>props.email.pinned===true;
 const unread=()=>props.email.keywords?.['$seen']!==true;
 const flagged=()=>props.email.keywords?.['$flagged']===true;
 const conversation=createConversation({threadId:()=>props.email.threadId??undefined,messageId:id,active:()=>props.active});
 const linkReview=createLinkReview({links:()=>props.links,truncated:()=>props.linksTruncated,scope:()=>props.linkScope,active:()=>props.active,...(props.onCompose?{onCompose:props.onCompose}:{})});
 createEffect(()=>{void props.headersUrl;void props.headersScope;void app.accountId();void id();void props.active;setHeadersOpen(false);});
 function closeHeaders():void {setHeadersOpen(false);if(placement('headers')!=='toolbar'||folded().has('headers'))moreButton?.focus();else row?.querySelector<HTMLButtonElement>('[data-testid="reader-headers"]')?.focus();}

 /** A plain button action: the same descriptor drives the row and the menu. */
 function plain(spec:{id:string;icon:()=>IconName;label:()=>string;help:()=>string;testId?:string;disabled?:()=>boolean;pressed?:()=>boolean;run:()=>void;groupEnd?:boolean;local?:boolean}):RowAction {
  return {id:spec.id,...(spec.groupEnd?{groupEnd:true}:{}),
   inline:()=><ReaderAction icon={spec.icon()} label={spec.label()} help={spec.help()} {...(spec.testId?{testId:spec.testId}:{})} disabled={spec.disabled?.()===true} pressed={spec.pressed?.()} local={spec.local===true} onClick={()=>spec.run()}/>,
   menu:()=>[{id:spec.id,label:spec.label(),icon:spec.icon(),disabled:spec.disabled?.()===true,...(spec.pressed?{checked:spec.pressed()}:{}),run:spec.run}]};
 }
 const icon=(name:IconName)=>()=>name;
 // Built once: each descriptor reads its state reactively, so a changed state
 // (read, flagged, a new link count) updates its button in place and never
 // re-creates a button that has focus. `ids` decides which are present.
 const all:Record<string,RowAction>={
  'finish-followup':plain({id:'finish-followup',icon:icon('check'),label:()=>t('mail-mark-finished'),help:()=>t('mail-mark-finished'),testId:'reader-finish-followup',run:()=>{void app.completeFollowUp(id()).catch(()=>app.showToast('error',t('mail-context-failed')));}}),
  reply:plain({id:'reply',icon:icon(ACTION_ICONS.reply),label:()=>t('mail-reply'),help:()=>t('mail-action-help-reply'),testId:'reader-reply',disabled:()=>props.preparing,run:()=>props.onReply('reply')}),
  'reply-all':plain({id:'reply-all',icon:icon(ACTION_ICONS.replyAll),label:()=>t('mail-reply-all'),help:()=>t('mail-action-help-reply-all'),testId:'reader-reply-all',disabled:()=>props.preparing,run:()=>props.onReply('reply-all')}),
  forward:plain({id:'forward',icon:icon(ACTION_ICONS.forward),label:()=>t('mail-forward'),help:()=>t('mail-action-help-forward'),testId:'reader-forward',disabled:()=>props.preparing,run:()=>props.onReply('forward'),groupEnd:true}),
  'edit-draft':plain({id:'edit-draft',icon:icon('compose'),label:()=>t('mail-draft-edit'),help:()=>t('mail-draft-edit-help'),testId:'reader-edit-draft',run:()=>props.onEditDraft?.(id())}),
  'open-draft':plain({id:'open-draft',icon:icon('compose'),label:()=>t('mail-reply-draft-open'),help:()=>t('mail-reply-draft'),testId:'reader-open-draft',run:()=>{const draft=replyDraft();if(draft)props.onOpenReplyDraft?.(draft.id);}}),
  conversation:{id:'conversation',
   inline:()=><ReaderAction icon="conversation" label={t('mail-conversation-title')} help={t('mail-conversation-title')} testId="reader-conversation" popup="dialog" expanded={conversation.open()} onClick={()=>conversation.toggle()}/>,
   menu:()=>[{id:'conversation',label:t('mail-conversation-title'),icon:'conversation',run:()=>conversation.toggle()}]},
  links:{id:'links',
   inline:()=><ReaderAction icon="link" label={t('mail-links-review',{count:linkReview.count()})} help={t('mail-links-help')} testId="reader-links" popup="dialog" expanded={linkReview.isOpen()} badge={linkReview.count()} onClick={()=>linkReview.open()}/>,
   menu:()=>[{id:'links',label:t('mail-links-review',{count:linkReview.count()}),icon:'link',run:()=>linkReview.open()}]},
  search:{id:'search',inline:()=><ReaderSearchMenu email={props.email}/>,menu:()=>[{id:'search-related',label:t('search-refine-menu'),icon:'search',children:relatedSearchActions(props.email,app.mailboxes().map(m=>m.id),(field,value)=>{if(realtime.subTabs.tabs().some(tab=>tab.id==='mail'))realtime.subTabs.activate('mail');void app.searchMessageField(field,value);},app.supportsFromDomainSearch()),run:()=>{}}]},
  pin:plain({id:'pin',icon:icon('pin'),label:()=>pinned()?t('mail-unpin'):t('mail-pin'),help:()=>t('mail-action-help-pin'),pressed:pinned,run:()=>void app.pinMessage(id(),!pinned()),local:true}),
  read:plain({id:'read',icon:()=>unread()?ACTION_ICONS.markRead:ACTION_ICONS.markUnread,label:()=>unread()?t('mail-mark-read'):t('mail-mark-unread'),help:()=>t('mail-action-help-read'),testId:'reader-toggle-read',run:()=>void app.markRead(id(),unread()),local:true}),
  flag:plain({id:'flag',icon:icon('flag'),label:()=>flagged()?t('mail-unflag-message'):t('mail-flag-message'),help:()=>t('mail-action-help-flag'),testId:'reader-flag',pressed:flagged,run:()=>void app.flagMessage(id(),!flagged()),local:true}),
  archive:plain({id:'archive',icon:icon(ACTION_ICONS.archive),label:()=>t('mail-archive'),help:()=>t('mail-action-help-archive'),run:()=>void app.archiveMessage(id())}),
  move:{id:'move',inline:()=><ReaderMoveMenu email={props.email}/>,menu:()=>[{id:'move',searchLabel:t('mail-move-search'),label:t('mail-reader-move-to'),icon:ACTION_ICONS.moveTo,children:moveActions(props.email,app),run:()=>{}}]},
  delete:plain({id:'delete',icon:icon('delete'),label:()=>t('mail-delete'),help:()=>t('mail-action-help-delete'),run:()=>void app.trashMessage(id())}),
  spam:plain({id:'spam',icon:icon('spam'),label:()=>t('mail-spam'),help:()=>t('mail-action-help-spam'),run:()=>void app.markSpam(id())}),
  export:plain({id:'export',icon:icon(ACTION_ICONS.export),label:()=>t('mail-export'),help:()=>t('mail-action-help-export'),testId:'reader-export',run:()=>{if(!exportFromMenu(app,id(),props.email))void app.exportMessage(id());}}),
  properties:plain({id:'properties',icon:icon('info'),label:()=>t('props-menu-properties'),help:()=>t('props-menu-properties'),testId:'reader-properties',run:()=>{openMessageProperties(props.email,dialogActionsFor(app,props.email));}}),
  print:plain({id:'print',icon:icon('print'),label:()=>t('mail-print'),help:()=>t('mail-print-help'),run:()=>props.onPrint()}),
  // t59: zoom out / level (reset) / zoom in; folds as three More entries.
  zoom:zoomRowAction(),
  sweep:plain({id:'sweep',icon:icon('sweep'),label:()=>t('mail-sweep-sender'),help:()=>t('mail-action-help-sweep'),run:()=>setSweeping(true)}),
  // Max-security opening switch (plan §7.2): drives the body render mode
  // (full / no-media / plain-text) for this sender. Per-sender + global
  // policy, admin floor clamps up. One dropdown (t57); folded, its three
  // modes join the More menu.
  security:{id:'security',
   inline:()=><ReaderViewMode value={props.security.value} floor={props.security.floor} onChange={props.security.onChange} scope={`${app.accountId()}:${id()}`}/>,
   menu:()=>viewModeActions(props.security,true)},
  colors:colorRowAction(()=>props.colors!,()=>`${app.accountId()}:${id()}`),
 headers:{id:'headers',
   inline:()=><ReaderAction icon="serverMetadata" label={t('mail-headers-title')} help={t('mail-headers-action-help')} testId="reader-headers" popup="dialog" expanded={headersOpen()} onClick={()=>setHeadersOpen(true)}/>,
   menu:()=>[{id:'headers',label:t('mail-headers-title'),icon:'serverMetadata',run:()=>{setHeadersOpen(true);}}]},
 };
 const primaryIds=createMemo(()=>[...((props.email.followUpAt||props.email.keywords?.['$flagged'])&&!props.email.keywords?.['mw-followup-completed']?['finish-followup']:[]),...(isDraft()?['edit-draft']:[]),...(mailFeature('allowReply')&&!isDraft()?['reply','reply-all']:[]),...(mailFeature('allowForward')&&!isDraft()?['forward']:[]),...(replyDraft()?['open-draft']:[]),...(conversation.available()?['conversation']:[]),...(linkReview.count()>0?['links']:[]),'pin','read','flag','archive','move','delete',...(mailFeature('showSpam')?['spam']:[]),...(mailFeature('allowDownload')?['export']:[]),...(mailFeature('allowPrint')?['print']:[]),'zoom','search',...(sender()!==''?['sweep']:[]),'security',...(props.colors?['colors']:[]),...(props.headersUrl?['headers']:[]),...(messageDialogEnabled()?['properties']:[])],undefined,{equals:(a,b)=>a.length===b.length&&a.every((value,index)=>value===b[index])});
 const extraRows=createMemo(()=>[...saveRowsFor(app,[props.email]),...focusActions(app,props.email),...messageActionRows(app,props.email,new Set(['move-to']))]);
 const extraId=(value:ContextAction)=>readerMenuActionId(value.id)??value.id;
 const extraIds=createMemo(()=>[...new Set(extraRows().map(extraId))].filter(key=>!primaryIds().includes(key)));
 const extraRow=(key:string)=>extraRows().find(value=>extraId(value)===key);
 const placement=(key:string)=>readerActionPlacement(key);
 const sameIds=(a:readonly string[],b:readonly string[])=>a.length===b.length&&a.every((value,index)=>value===b[index]);
 const ids=createMemo(()=>[...primaryIds(),...extraIds()],[] as string[],{equals:sameIds});
 const toolbarIds=createMemo(()=>ids().filter(key=>placement(key)==='toolbar'),[] as string[],{equals:sameIds});
 const forcedMenuIds=createMemo(()=>ids().filter(key=>placement(key)==='menu'),[] as string[],{equals:sameIds});
 function actionFor(key:string):RowAction {
  return all[key]??(all[key]={id:key,
   inline:()=><ReaderMenuAction id={key} action={()=>extraRow(key)} scope={()=>`${app.accountId()}:${id()}:${placement(key)}`} onError={()=>app.showToast?.('error',t('mail-context-failed'))}/>,
   menu:()=>{const value=extraRow(key);return value?[value]:[];},
  });
 }


 // ── Fitting ────────────────────────────────────────────────────────────────
 const [folded,setFolded]=createSignal<ReadonlySet<string>>(new Set(),{equals:(a,b)=>a.size===b.size&&[...a].every(value=>b.has(value))});
 const widths=new Map<string,number>();
 let row!:HTMLDivElement;
 let moreButton!:HTMLButtonElement;
 let moreWidth=0;
 function fit():void {
  if(!row?.isConnected)return;
  for(const element of Array.from(row.querySelectorAll<HTMLElement>('[data-reader-item]')))if(!element.hidden&&placement(element.dataset['readerItem']!)==='toolbar'){const style=getComputedStyle(element);widths.set(element.dataset['readerItem']!,element.getBoundingClientRect().width+(Number.parseFloat(style.marginLeft)||0)+(Number.parseFloat(style.marginRight)||0));}
  const measuredMore=moreButton.getBoundingClientRect().width;if(measuredMore>0)moreWidth=measuredMore;
  const style=getComputedStyle(row);const gap=Number.parseFloat(style.columnGap)||0;
  const available=row.clientWidth-(Number.parseFloat(style.paddingLeft)||0)-(Number.parseFloat(style.paddingRight)||0);
  if(available<=0)return;
  setFolded(foldActions(toolbarIds().map(key=>({id:key,priority:PRIORITY[key]??99,width:widths.get(key)??0})),available,gap,moreWidth,forcedMenuIds().length>0));
 }
 const observer=typeof ResizeObserver==='function'?new ResizeObserver(()=>fit()):undefined;
 onMount(()=>{observer?.observe(row);observer?.observe(moreButton);fit();});
 onCleanup(()=>observer?.disconnect());
 // A new set of actions (a conversation found, links parsed) or a label-style
 // change: show everything for one measure, then fold again.
 createEffect(()=>{void toolbarIds();void settings()['reader.actionStyle']?.value;void forcedMenuIds().length;setFolded(new Set<string>());queueMicrotask(fit);});

 // ── More menu ──────────────────────────────────────────────────────────────
 // t94: entries that live only in More (never on the row): the sender's
 // Focused inbox override while the Focused inbox is on.
 const idle=()=>folded().size===0&&forcedMenuIds().length===0;
 const inMore=(key:string)=>placement(key)==='menu'||(placement(key)==='toolbar'&&folded().has(key));
 const menu=createContextMenu({label:()=>t('mail-more-actions'),scope:()=>`${app.accountId()}:${id()}`,actions:()=>{
  const rows=primaryIds().filter(inMore).flatMap(key=>actionFor(key).menu());
  const extra=extraIds().filter(inMore).flatMap(key=>actionFor(key).menu()).map((value,index)=>index===0&&rows.length>0?{...value,separatorBefore:true}:value);
  return [...rows,...extra];
 },restoreFocus:()=>moreButton?.focus()});
 // A preference change can hide the trigger of an already open dropdown.
 // Dismiss context menus before moving their actions, while keeping row nodes.
 let previousPlacements:string|undefined;
 createEffect(()=>{
  const next=READER_ACTIONS.map(item=>placement(item.id)).join('|');
  if(previousPlacements!==undefined&&next!==previousPlacements){
   menu.close(false);const doc=row?.ownerDocument;
   if(doc){const event=doc.createEvent('Event');event.initEvent('mailwoman-context-open',false,false);doc.dispatchEvent(event);}
  }
  previousPlacements=next;
 });

 // A pointerdown on this button while the menu is open closes it (ContextMenu's
 // outside-press handler); the click that follows must not open it again. This
 // capture listener is registered before the menu's, so it sees the press first.
 let reopenGuard=false;
 onMount(()=>{const press=(event:PointerEvent):void=>{reopenGuard=menu.isOpen()&&event.target instanceof Node&&moreButton.contains(event.target);};document.addEventListener('pointerdown',press,true);onCleanup(()=>document.removeEventListener('pointerdown',press,true));});
 function toggleMore(event:MouseEvent):void {if(menu.isOpen()){menu.close();return;}if(reopenGuard){reopenGuard=false;return;}menu.open(event);}

 return (
  <div ref={row} class={`reader__toolbar ${css.row}`} role="toolbar" aria-label={t('mail-reader-actions')}>
   <For each={ids()}>{key=>{const value=actionFor(key);return <span ref={element=>{observer?.observe(element);onCleanup(()=>observer?.unobserve(element));}} class={css.item} classList={{[css.groupEnd]:value.groupEnd===true}} data-reader-item={key} data-reader-placement={placement(key)} hidden={placement(key)!=='toolbar'||folded().has(key)}>{value.inline()}</span>;}}</For>
   <button ref={moreButton} type="button" class={`btn btn--ghost reader-action reader-action--compact ${a11y.focusable} ${css.more}`} classList={{[css.moreIdle]:idle()}} aria-label={t('mail-more-actions')} title={t('mail-more-actions')} aria-haspopup="menu" aria-expanded={menu.isOpen()} aria-hidden={idle()?'true':undefined} tabindex={idle()?-1:0} data-testid="reader-more" onClick={toggleMore}><Icon name="more"/></button>
   {menu.menu()}
   {conversation.dialog()}
   {linkReview.dialog()}
   <Show when={headersOpen()&&props.active&&props.headersUrl}>{url=><RawMessageHeaders url={url()} scope={props.headersScope??`${app.accountId()}:${id()}`} onClose={closeHeaders}/>}</Show>
   <Show when={sweeping()}><SweepDialog fromEmail={sender()} onClose={()=>setSweeping(false)}/></Show>
  </div>
 );
}
