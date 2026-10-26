import {createEffect,createMemo,createSignal,For,onMount,Show,type JSX} from 'solid-js';
import {mailingListCommands,listActions,listMailDraft,type ListAction,type ListLink} from '../api/mailingList.ts';
import type {ComposeInitial} from './compose/reply.ts';
import {loadCatalog,t} from '../i18n';

/** Sender-provided commands: never fetch, POST, or send merely by rendering. */
export function MailingListActions(props:{headers:string;scope:string;active:boolean;onCompose?:(draft:ComposeInitial)=>void}):JSX.Element {
 const commands=createMemo(()=>mailingListCommands(props.headers));const [selected,setSelected]=createSignal<{action:ListAction;link:ListLink;scope:string}|null>(null);
 onMount(()=>{void loadCatalog('mailing-list');});
 createEffect(()=>{void props.scope;void props.headers;setSelected(null);});
 const current=()=>{const value=selected();return value?.scope===props.scope?value:null;};
 function compose():void {const value=current();if(!value||!props.active)return;const draft=listMailDraft(value.link);if(!draft)return;
  setSelected(null);props.onCompose?.({...draft,mode:'new',cc:'',inReplyTo:[],references:[],attachments:[],quotesDecrypted:false});
 }
 return <Show when={props.active&&(Object.keys(commands().actions).length>0||commands().postingForbidden)}><details class="reader__list-actions"><summary>{t('mailing-list-title')}</summary>
  <p>{t('mailing-list-unverified')}</p><Show when={commands().postingForbidden}><p>{t('mailing-list-no-post')}</p></Show>
  <For each={listActions}>{action=><Show when={commands().actions[action]}>{links=><div><strong>{t(`mailing-list-${action}`)}</strong><ol><For each={links()}>{link=><li><button type="button" class="btn btn--ghost" onClick={()=>setSelected({action,link,scope:props.scope})}>{link.host}</button></li>}</For></ol></div>}</Show>}</For>
  <Show when={current()}>{value=><section aria-label={t('mailing-list-review')}><h3>{t(`mailing-list-${value().action}`)}</h3><p style={{'overflow-wrap':'anywhere'}}>{value().link.href}</p>
   <Show when={value().link.kind==='web'} fallback={<button type="button" class="btn" disabled={!props.onCompose} onClick={compose}>{t('mailing-list-compose')}</button>}><p>{t('mailing-list-external')}</p><a class="btn" href={value().link.href} target="_blank" rel="noopener noreferrer" referrerpolicy="no-referrer">{t('mailing-list-open')}</a></Show>
   <button type="button" class="btn btn--ghost" onClick={()=>setSelected(null)}>{t('mailing-list-cancel')}</button>
  </section>}</Show>
 </details></Show>;
}
