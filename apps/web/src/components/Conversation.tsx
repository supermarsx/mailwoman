import {createEffect,createMemo,createSignal,For,onCleanup,onMount,Show,untrack,type JSX} from 'solid-js';
import {Portal} from 'solid-js/web';
import {wireDialogFocus} from '../modules/contacts/dialogFocus.ts';
import {useApp} from '../state/context.ts';
import {createConfiguredClient} from '../api/transport.ts';
import {conversationSnapshot,conversationPage,CONVERSATION_LIMIT,type ConversationSnapshot} from '../api/conversation.ts';
import {preferenceAccount,preferenceAccountIsCurrent,observePreferenceAccount} from '../api/accountPreferenceLifecycle.ts';
import type {Email} from '../api/jmap-types.ts';
import {Icon} from './Icon.tsx';
import {t} from '../i18n';
import './conversation.css.ts';
import './sweepDialog.css.ts';

function ConversationDialog(props:{onClose:()=>void;children:JSX.Element}):JSX.Element{
 let dialog!:HTMLDivElement;onMount(()=>onCleanup(wireDialogFocus(dialog,props.onClose)));
 return <Portal><div ref={dialog} class="sweep__backdrop" role="dialog" aria-modal="true" aria-label={t('mail-conversation-title')} tabindex="-1"><section class="sweep"><header class="sweep__header"><h2>{t('mail-conversation-title')}</h2><button type="button" class="btn btn--ghost" aria-label={t('mail-compose-close')} onClick={props.onClose}><Icon name="close"/></button></header>{props.children}</section></div></Portal>;
}

/** The reader's "View conversation" action (t57): its trigger lives in the
 * reader action row (ReaderToolbar.tsx), so this owns only the state and the
 * dialog. On mount (and for every new message or account) it asks Thread/get
 * how many messages the thread holds; `available` is true only for a thread of
 * more than one, so a single message shows no conversation action. Opening
 * reuses that snapshot and pages members in as before; the count and "Load more"
 * appear once the first page is in, so the dialog never reads "0 of N" while it loads. */
export function createConversation(props:{threadId:()=>string|undefined;messageId:()=>string;active:()=>boolean}){
 const app=useApp(),client=createConfiguredClient();const scope=createMemo(()=>JSON.stringify([app.accountId(),props.threadId()??null]));
 const [open,setOpen]=createSignal(false),[busy,setBusy]=createSignal(false),[error,setError]=createSignal(false),[snapshot,setSnapshot]=createSignal<ConversationSnapshot>(),[messages,setMessages]=createSignal<Email[]>([]),[next,setNext]=createSignal(0),[missing,setMissing]=createSignal(0);
 let abort:AbortController|undefined,probe:AbortController|undefined;
 function reset():void{abort?.abort();abort=undefined;probe?.abort();probe=undefined;setOpen(false);setBusy(false);setError(false);setSnapshot(undefined);setMessages([]);setNext(0);setMissing(0);}
 /** Size the thread without loading any member. A failure leaves the action hidden. */
 async function size():Promise<void>{
  const threadId=props.threadId();const account=preferenceAccount();if(!threadId||!account||account.id!==app.accountId()||snapshot()||probe)return;const heldScope=scope();const controller=new AbortController();probe=controller;
  try{const held=await conversationSnapshot(client,account.id,threadId,controller.signal);if(!controller.signal.aborted&&preferenceAccountIsCurrent(account)&&scope()===heldScope&&!snapshot())setSnapshot(held);}catch{/* hidden: the size is unknown */}finally{if(probe===controller)probe=undefined;}
 }
 createEffect(()=>{void scope();const active=props.active();untrack(()=>{reset();if(active)void size();});});
 onCleanup(observePreferenceAccount(()=>{reset();queueMicrotask(()=>{if(props.active())void size();});}));onCleanup(()=>{abort?.abort();probe?.abort();});
 async function load():Promise<void>{
  const threadId=props.threadId();if(busy()||!threadId)return;const account=preferenceAccount();if(!account||account.id!==app.accountId())return;const heldScope=scope();const controller=new AbortController();abort=controller;setBusy(true);setError(false);
  const current=()=>!controller.signal.aborted&&preferenceAccountIsCurrent(account)&&scope()===heldScope;
  try{const held=snapshot()??await conversationSnapshot(client,account.id,threadId,controller.signal);if(!current())return;const result=await conversationPage(client,held,next(),controller.signal);if(!current())return;setSnapshot(held);setMessages(previous=>[...previous,...result.messages]);setNext(result.next);setMissing(previous=>previous+result.missing);}catch{if(current())setError(true);}finally{if(current())setBusy(false);}
 }
 const available=():boolean=>props.threadId()!==undefined&&(snapshot()?.total??0)>1;
 function toggle():void{const opening=!open();setOpen(opening);if(opening&&messages().length===0&&!busy())void load();}
 const dialog=():JSX.Element=><Show when={open()&&props.active()}><ConversationDialog onClose={()=>setOpen(false)}><p>{t('mail-conversation-help')}</p><Show when={next()>0?snapshot():undefined}>{value=><p>{t('mail-conversation-count',{loaded:messages().length,total:value().total})}</p>}</Show><ol class="conversation__members"><For each={messages()}>{message=><li><button type="button" class="btn btn--ghost" aria-current={message.id===props.messageId()?'true':undefined} onClick={()=>{setOpen(false);void app.openMessage(message.id);}}><span><bdi>{message.subject||t('mail-no-subject')}</bdi></span><small><bdi>{message.from?.map(sender=>sender.name||sender.email).join(', ')}</bdi></small><small><For each={Object.keys(message.mailboxIds??{})}>{id=><span>{app.mailboxes().find(box=>box.id===id)?.name??t('mail-conversation-other-folder')} </span>}</For></small></button></li>}</For></ol><Show when={busy()}><p role="status">{t('mail-conversation-loading')}</p></Show><Show when={missing()}><p role="status">{t('mail-conversation-missing',{count:missing()})}</p></Show><Show when={error()}><p role="alert">{t('mail-conversation-error')}</p><button class="btn" onClick={()=>void load()}>{t('common-retry')}</button></Show><Show when={!error()&&next()>0&&snapshot()&&next()<snapshot()!.ids.length}><button class="btn" disabled={busy()} onClick={()=>void load()}>{t('mail-conversation-more')}</button></Show><Show when={snapshot()&&snapshot()!.total>CONVERSATION_LIMIT}><p>{t('mail-conversation-limit',{count:CONVERSATION_LIMIT})}</p></Show></ConversationDialog></Show>;
 return {available,open,toggle,total:()=>snapshot()?.total??0,dialog};
}
