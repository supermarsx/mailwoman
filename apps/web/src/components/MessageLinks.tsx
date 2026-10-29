import {createEffect,createMemo,createSignal,For,onCleanup,onMount,Show,type JSX} from 'solid-js';
import {Portal} from 'solid-js/web';
import type {MessageLink} from '../viewers/messageLinks.ts';
import type {ComposeInitial} from './compose/reply.ts';
import {listMailDraft} from '../api/mailingList.ts';
import {wireDialogFocus} from '../modules/contacts/dialogFocus.ts';
import {Icon} from './Icon.tsx';
import {t} from '../i18n';
import './sweepDialog.css.ts';

function LinkReview(props:{links:MessageLink[];truncated:boolean;onClose:()=>void;onCompose?:(draft:ComposeInitial)=>void}):JSX.Element{
 const [selected,setSelected]=createSignal<MessageLink>();let dialog!:HTMLDivElement;
 onMount(()=>onCleanup(wireDialogFocus(dialog,props.onClose)));
 function compose(link:MessageLink):void{const draft=listMailDraft(link);if(!draft)return;props.onClose();props.onCompose?.({...draft,mode:'new',cc:'',inReplyTo:[],references:[],attachments:[],quotesDecrypted:false});}
 return <Portal><div ref={dialog} class="sweep__backdrop" role="dialog" aria-modal="true" aria-label={t('mail-links-title')} tabindex="-1"><section class="sweep"><header class="sweep__header"><h2>{t('mail-links-title')}</h2><button type="button" class="btn btn--ghost" aria-label={t('mail-compose-close')} onClick={props.onClose}><Icon name="close"/></button></header>
 <p>{t('mail-links-help')}</p><Show when={props.truncated}><p>{t('mail-links-limit')}</p></Show>
 <ul class="sweep__preview"><For each={props.links}>{link=><li class="sweep__row message-links__row"><button type="button" class="btn btn--ghost" title={link.href} onClick={()=>setSelected(link)}><bdi>{link.label||link.host}</bdi></button><bdi>{link.host}</bdi></li>}</For></ul>
 <Show when={selected()}>{link=><section class="message-links__destination" aria-label={t('mail-links-destination')}><h3>{t('mail-links-destination')}</h3><p dir="ltr">{link().href}</p><ul><For each={link().warnings}>{warning=><li>{t(`mail-links-warning-${warning}`)}</li>}</For></ul><p>{t('mail-links-external')}</p>
 <Show when={link().kind==='web'} fallback={<button type="button" class="btn" disabled={!props.onCompose} onClick={()=>compose(link())}>{t('mail-links-compose')}</button>}><a class="btn" href={link().href} target="_blank" rel="noopener noreferrer" referrerpolicy="no-referrer">{t('mail-links-open')}</a></Show></section>}</Show>
 </section></div></Portal>;
}
/** The reader's "Review links" action (t57): the trigger lives in the reader
 * action row (ReaderToolbar.tsx), so this owns only the review snapshot and its
 * dialog. `count` is the number of reviewable links already parsed from the
 * body; the action is offered only while it is above zero. */
export function createLinkReview(props:{links:()=>MessageLink[];truncated:()=>boolean;scope:()=>string;active:()=>boolean;onCompose?:(draft:ComposeInitial)=>void}){
 const [snapshot,setSnapshot]=createSignal<{links:MessageLink[];truncated:boolean}|null>(null);const scope=createMemo(()=>props.scope()),active=createMemo(()=>props.active());createEffect(()=>{void scope();void active();setSnapshot(null);});
 // The review owns its explicit snapshot. Background read-state refreshes may
 // temporarily clear the body but must not dismiss a destination under review.
 const count=():number=>props.active()?props.links().length:0;
 function open():void{if(count()===0)return;setSnapshot({links:props.links().map(link=>({...link,warnings:[...link.warnings]})),truncated:props.truncated()});}
 const dialog=():JSX.Element=><Show when={snapshot()}>{value=><LinkReview links={value().links} truncated={value().truncated} onClose={()=>setSnapshot(null)} {...(props.onCompose?{onCompose:props.onCompose}:{})}/>}</Show>;
 return {count,open,isOpen:()=>snapshot()!==null,dialog};
}
