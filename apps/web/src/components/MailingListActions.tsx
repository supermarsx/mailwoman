import {createEffect,createMemo,createSignal,onMount,Show,type JSX} from 'solid-js';
import {mailingListCommands,mailingListName,listActions,type ListAction} from '../api/mailingList.ts';
import {inspectMessageLink,type MessageLink} from '../viewers/messageLinks.ts';
import type {ComposeInitial} from './compose/reply.ts';
import type {IconName} from './Icon.tsx';
import {createContextMenu,type ContextAction} from './ContextMenu.tsx';
import {LinkReview} from './MessageLinks.tsx';
import {NoticeButton,ReaderNoticeBar,noticeCss} from './ReaderNoticeBar.tsx';
import {loadCatalog,t} from '../i18n';

const ICONS:Record<ListAction,IconName>={help:'info',subscribe:'enable',unsubscribe:'disable',post:'compose',owner:'person',archive:'archive'};
const isolate=(s:string):string=>`\u2068${s}\u2069`;

/** The mailing-list bar (t110): the list's name and its commands, on the shared
 * reader notice frame, first in the bar stack. Sender-provided commands: never
 * fetched, POSTed or sent merely by rendering. Every command, Unsubscribe too,
 * opens the link review dialog (MessageLinks.tsx) so a web destination goes
 * through the same review and trusted-domain rules as a body link, and a mail
 * command becomes a draft the user reviews. There is no one-click (RFC 8058)
 * request: this server has no route that would POST it on the user's behalf. */
export function MailingListActions(props:{headers:string;scope:string;active:boolean;onCompose?:(draft:ComposeInitial)=>void}):JSX.Element {
 const commands=createMemo(()=>mailingListCommands(props.headers));
 const name=createMemo(()=>mailingListName(props.headers));
 const [review,setReview]=createSignal<{action:ListAction;links:MessageLink[];scope:string}|null>(null);
 onMount(()=>{void loadCatalog('mailing-list');});
 createEffect(()=>{void props.scope;void props.headers;setReview(null);});
 const current=()=>{const value=review();return value?.scope===props.scope&&props.active?value:null;};
 function open(action:ListAction):void {
  const links=(commands().actions[action]??[]).map(link=>inspectMessageLink(link.href,'')).filter((link):link is MessageLink=>link!==null);
  if(links.length>0)setReview({action,links,scope:props.scope});
 }
 const visible=()=>props.active&&(Object.keys(commands().actions).length>0||commands().postingForbidden);
 const headline=()=>t('mailing-list-bar');
 const detail=()=>{const n=name();return n?isolate(n):null;};
 const tooltip=()=>[[headline(),detail()].filter(Boolean).join(' · '),t('mailing-list-unverified'),commands().postingForbidden?t('mailing-list-no-post'):null].filter(Boolean).join('\n');
 const menuActions=createMemo<ContextAction[]>(()=>listActions.filter(action=>action!=='unsubscribe'&&commands().actions[action]).map(action=>({id:action,label:t(`mailing-list-${action}`),icon:ICONS[action],run:()=>open(action)})));
 const menu=createContextMenu({label:()=>t('mailing-list-title'),scope:()=>props.scope,actions:menuActions});
 return <Show when={visible()}>
  <ReaderNoticeBar label={t('mailing-list-title')} testid="mailing-list-bar" state={commands().actions.unsubscribe?'unsubscribe':'list'} icon="mail" tooltip={tooltip()}
   actions={<>
    <Show when={commands().actions.unsubscribe}><NoticeButton icon="disable" label={t('mailing-list-unsubscribe')} tooltip={t('mailing-list-unsubscribe-help')} haspopup="dialog" onClick={()=>open('unsubscribe')}/></Show>
    <Show when={menuActions().length>0}><NoticeButton icon="chevron" menu iconOnly label={t('mailing-list-more')} haspopup="menu" expanded={menu.isOpen()} onClick={e=>menu.open(e)} onKeyDown={menu.keydown}/></Show>
   </>}
   after={<>{menu.menu()}<Show when={current()}>{value=><LinkReview links={value().links} truncated={false} title={t(`mailing-list-${value().action}`)} help={`${t('mailing-list-unverified')} ${value().action==='unsubscribe'?t('mailing-list-unsubscribe-review'):''}`.trim()} onClose={()=>setReview(null)} {...(props.onCompose?{onCompose:props.onCompose}:{})}/>}</Show></>}>
   <span class={noticeCss.headline}>{headline()}</span><Show when={detail()}>{text=><span class={noticeCss.detail}>{' · '}<bdi>{text()}</bdi></span>}</Show>
  </ReaderNoticeBar>
 </Show>;
}
