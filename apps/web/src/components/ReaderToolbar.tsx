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
import {useApp} from '../state/context.ts';
import {settings} from '../api/settings.ts';
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
import {Icon,type IconName} from './Icon.tsx';
import * as a11y from './mailA11y.css.ts';
import * as css from './readerToolbar.css.ts';
import {t} from '../i18n/index.ts';
import {zoomRowAction} from './ReaderZoom.tsx';

/** Fold order: lower stays on the row longer. */
export const PRIORITY: Record<string,number> = {
 reply:0,'reply-all':1,forward:2,delete:3,archive:4,conversation:5,links:6,read:7,flag:8,pin:9,spam:10,security:11,sweep:12,print:13,export:14,zoom:10.5,
};

interface RowAction {id:string;groupEnd?:boolean;inline:()=>JSX.Element;menu:()=>ContextAction[]}

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
 onCompose?:(draft:ComposeInitial)=>void;
 security:{value:SecurityMode;floor:SecurityMode|null;onChange:(mode:SecurityMode)=>void};
}):JSX.Element {
 const app=useApp();
 const [sweeping,setSweeping]=createSignal(false);
 const id=()=>props.email.id;
 const sender=()=>props.email.from?.[0]?.email??'';
 const pinned=()=>props.email.pinned===true;
 const unread=()=>props.email.keywords?.['$seen']!==true;
 const flagged=()=>props.email.keywords?.['$flagged']===true;
 const conversation=createConversation({threadId:()=>props.email.threadId??undefined,messageId:id,active:()=>props.active});
 const linkReview=createLinkReview({links:()=>props.links,truncated:()=>props.linksTruncated,scope:()=>props.linkScope,active:()=>props.active,...(props.onCompose?{onCompose:props.onCompose}:{})});

 /** A plain button action: the same descriptor drives the row and the menu. */
 function plain(spec:{id:string;icon:()=>IconName;label:()=>string;help:()=>string;testId?:string;disabled?:()=>boolean;pressed?:()=>boolean;run:()=>void;groupEnd?:boolean}):RowAction {
  return {id:spec.id,...(spec.groupEnd?{groupEnd:true}:{}),
   inline:()=><ReaderAction icon={spec.icon()} label={spec.label()} help={spec.help()} {...(spec.testId?{testId:spec.testId}:{})} disabled={spec.disabled?.()===true} pressed={spec.pressed?.()} onClick={()=>spec.run()}/>,
   menu:()=>[{id:spec.id,label:spec.label(),icon:spec.icon(),disabled:spec.disabled?.()===true,...(spec.pressed?{checked:spec.pressed()}:{}),run:spec.run}]};
 }
 const icon=(name:IconName)=>()=>name;
 // Built once: each descriptor reads its state reactively, so a changed state
 // (read, flagged, a new link count) updates its button in place and never
 // re-creates a button that has focus. `ids` decides which are present.
 const all:Record<string,RowAction>={
  reply:plain({id:'reply',icon:icon('reply'),label:()=>t('mail-reply'),help:()=>t('mail-action-help-reply'),testId:'reader-reply',disabled:()=>props.preparing,run:()=>props.onReply('reply')}),
  'reply-all':plain({id:'reply-all',icon:icon('replyAll'),label:()=>t('mail-reply-all'),help:()=>t('mail-action-help-reply-all'),testId:'reader-reply-all',disabled:()=>props.preparing,run:()=>props.onReply('reply-all')}),
  forward:plain({id:'forward',icon:icon('forward'),label:()=>t('mail-forward'),help:()=>t('mail-action-help-forward'),testId:'reader-forward',disabled:()=>props.preparing,run:()=>props.onReply('forward'),groupEnd:true}),
  conversation:{id:'conversation',
   inline:()=><ReaderAction icon="conversation" label={t('mail-conversation-title')} help={t('mail-conversation-title')} testId="reader-conversation" popup="dialog" expanded={conversation.open()} onClick={()=>conversation.toggle()}/>,
   menu:()=>[{id:'conversation',label:t('mail-conversation-title'),icon:'conversation',run:()=>conversation.toggle()}]},
  links:{id:'links',
   inline:()=><ReaderAction icon="link" label={t('mail-links-review',{count:linkReview.count()})} help={t('mail-links-help')} testId="reader-links" popup="dialog" expanded={linkReview.isOpen()} badge={linkReview.count()} onClick={()=>linkReview.open()}/>,
   menu:()=>[{id:'links',label:t('mail-links-review',{count:linkReview.count()}),icon:'link',run:()=>linkReview.open()}]},
  pin:plain({id:'pin',icon:icon('pin'),label:()=>pinned()?t('mail-unpin'):t('mail-pin'),help:()=>t('mail-action-help-pin'),pressed:pinned,run:()=>void app.pinMessage(id(),!pinned())}),
  read:plain({id:'read',icon:()=>unread()?'mailRead':'mail',label:()=>unread()?t('mail-mark-read'):t('mail-mark-unread'),help:()=>t('mail-action-help-read'),testId:'reader-toggle-read',run:()=>void app.markRead(id(),unread())}),
  flag:plain({id:'flag',icon:icon('flag'),label:()=>flagged()?t('mail-unflag-message'):t('mail-flag-message'),help:()=>t('mail-action-help-flag'),testId:'reader-flag',pressed:flagged,run:()=>void app.flagMessage(id(),!flagged())}),
  archive:plain({id:'archive',icon:icon('archive'),label:()=>t('mail-archive'),help:()=>t('mail-action-help-archive'),run:()=>void app.archiveMessage(id())}),
  delete:plain({id:'delete',icon:icon('delete'),label:()=>t('mail-delete'),help:()=>t('mail-action-help-delete'),run:()=>void app.trashMessage(id())}),
  spam:plain({id:'spam',icon:icon('spam'),label:()=>t('mail-spam'),help:()=>t('mail-action-help-spam'),run:()=>void app.markSpam(id())}),
  export:plain({id:'export',icon:icon('download'),label:()=>t('mail-export'),help:()=>t('mail-action-help-export'),testId:'reader-export',run:()=>void app.exportMessage()}),
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
 };
 const ids=createMemo(()=>['reply','reply-all','forward',...(conversation.available()?['conversation']:[]),...(linkReview.count()>0?['links']:[]),'pin','read','flag','archive','delete','spam','export','print','zoom',...(sender()!==''?['sweep']:[]),'security'],undefined,{equals:(a,b)=>a.length===b.length&&a.every((value,index)=>value===b[index])});

 // ── Fitting ────────────────────────────────────────────────────────────────
 const [folded,setFolded]=createSignal<ReadonlySet<string>>(new Set(),{equals:(a,b)=>a.size===b.size&&[...a].every(value=>b.has(value))});
 const widths=new Map<string,number>();
 let row!:HTMLDivElement;
 let moreButton!:HTMLButtonElement;
 let moreWidth=0;
 function fit():void {
  if(!row?.isConnected)return;
  for(const element of Array.from(row.querySelectorAll<HTMLElement>('[data-reader-item]')))if(!element.hidden)widths.set(element.dataset['readerItem']!,element.getBoundingClientRect().width);
  const measuredMore=moreButton.getBoundingClientRect().width;if(measuredMore>0)moreWidth=measuredMore;
  const style=getComputedStyle(row);const gap=Number.parseFloat(style.columnGap)||0;
  const available=row.clientWidth-(Number.parseFloat(style.paddingLeft)||0)-(Number.parseFloat(style.paddingRight)||0);
  if(available<=0)return;
  setFolded(foldActions(ids().map(key=>({id:key,priority:PRIORITY[key]??99,width:widths.get(key)??0})),available,gap,moreWidth));
 }
 const observer=typeof ResizeObserver==='function'?new ResizeObserver(()=>fit()):undefined;
 onMount(()=>{observer?.observe(row);observer?.observe(moreButton);fit();});
 onCleanup(()=>observer?.disconnect());
 // A new set of actions (a conversation found, links parsed) or a label-style
 // change: show everything for one measure, then fold again.
 createEffect(()=>{void ids();void settings()['reader.actionStyle']?.value;setFolded(new Set<string>());queueMicrotask(fit);});

 // ── More menu ──────────────────────────────────────────────────────────────
 const menu=createContextMenu({label:()=>t('mail-more-actions'),scope:()=>`${app.accountId()}:${id()}`,actions:()=>ids().filter(key=>folded().has(key)).flatMap(key=>all[key]?.menu()??[]),restoreFocus:()=>moreButton?.focus()});
 // A pointerdown on this button while the menu is open closes it (ContextMenu's
 // outside-press handler); the click that follows must not open it again. This
 // capture listener is registered before the menu's, so it sees the press first.
 let reopenGuard=false;
 onMount(()=>{const press=(event:PointerEvent):void=>{reopenGuard=menu.isOpen()&&event.target instanceof Node&&moreButton.contains(event.target);};document.addEventListener('pointerdown',press,true);onCleanup(()=>document.removeEventListener('pointerdown',press,true));});
 function toggleMore(event:MouseEvent):void {if(menu.isOpen()){menu.close();return;}if(reopenGuard){reopenGuard=false;return;}menu.open(event);}

 return (
  <div ref={row} class={`reader__toolbar ${css.row}`} role="toolbar" aria-label={t('mail-reader-actions')}>
   <For each={ids()}>{key=>{const value=all[key]!;return <span ref={element=>{observer?.observe(element);onCleanup(()=>observer?.unobserve(element));}} class={css.item} classList={{[css.groupEnd]:value.groupEnd===true}} data-reader-item={key} hidden={folded().has(key)}>{value.inline()}</span>;}}</For>
   <button ref={moreButton} type="button" class={`btn btn--ghost reader-action reader-action--compact ${a11y.focusable} ${css.more}`} classList={{[css.moreIdle]:folded().size===0}} aria-label={t('mail-more-actions')} title={t('mail-more-actions')} aria-haspopup="menu" aria-expanded={menu.isOpen()} aria-hidden={folded().size===0?'true':undefined} tabindex={folded().size===0?-1:0} data-testid="reader-more" onClick={toggleMore}><Icon name="more"/></button>
   {menu.menu()}
   {conversation.dialog()}
   {linkReview.dialog()}
   <Show when={sweeping()}><SweepDialog fromEmail={sender()} onClose={()=>setSweeping(false)}/></Show>
  </div>
 );
}
