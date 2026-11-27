import {createEffect,createSignal,createUniqueId,For,Show,type JSX} from 'solid-js';
import {Portal} from 'solid-js/web';
import {preferenceAccountIsCurrent} from '../api/accountPreferenceLifecycle.ts';
import {pendingSmartArchive,finishSmartArchive,type SmartArchiveRequest} from '../state/smartArchiveRequest.ts';
import {smartArchivePolicy} from '../state/smartArchive.ts';
import {createFocusTrap} from './a11y';
import {formatDateTime,t} from '../i18n';
import {Icon} from './Icon.tsx';
import * as css from '../screens/Settings/styles.css.ts';
import * as layout from '../screens/Settings/editor.css.ts';
export function SmartArchiveDialog():JSX.Element{return <Show when={pendingSmartArchive()} keyed>{request=><Picker request={request}/>}</Show>;}
function Picker(props:{request:SmartArchiveRequest}):JSX.Element{
 const uid=createUniqueId(),[element,setElement]=createSignal<HTMLDivElement>();
 const [choices,setChoices]=createSignal(new Map(props.request.groups.map(group=>[group.key,group.target])));
 const close=()=>{if(pendingSmartArchive()!==props.request)return;finishSmartArchive();if(props.request.trigger?.isConnected)props.request.trigger.focus();};
 createFocusTrap(element,{onEscape:close});
 createEffect(()=>{if(!smartArchivePolicy().enabled||!preferenceAccountIsCurrent(props.request.owner))close();});
 const valid=()=>props.request.groups.every(group=>group.candidates.some(candidate=>candidate.id===choices().get(group.key)));
 const submit=(event:SubmitEvent)=>{event.preventDefault();if(!valid()||!smartArchivePolicy().enabled||!preferenceAccountIsCurrent(props.request.owner)||pendingSmartArchive()!==props.request)return;finishSmartArchive(props.request.groups.map(group=>({...group,target:choices().get(group.key)!})));if(props.request.trigger?.isConnected)props.request.trigger.focus();};
 return <Portal mount={props.request.trigger?.ownerDocument.body??document.body}><div class={layout.scrim}/><div ref={setElement} class={layout.dialog} role="dialog" aria-modal="true" aria-labelledby={`${uid}-title`} aria-describedby={`${uid}-help`} tabindex="-1">
  <h2 id={`${uid}-title`} class={css.heading}><Icon name="smartArchive"/>{t('mail-smart-archive')}</h2>
  <p id={`${uid}-help`} class={css.meta}>{t('mail-smart-archive-review')}</p>
  <form class={layout.fields} onSubmit={submit}>
   <For each={props.request.groups}>{(group,index)=><div class={layout.fields}>
    <label class={css.field} for={`${uid}-${index()}`}>{t('mail-smart-archive-group',{sender:group.label,count:group.ids.length})}
     <select id={`${uid}-${index()}`} class={css.select} required value={choices().get(group.key)??''} onChange={event=>setChoices(old=>new Map(old).set(group.key,event.currentTarget.value))}>
      <option value="">{t('mail-smart-archive-choose')}</option><For each={group.candidates}>{candidate=><option value={candidate.id}>{candidate.name}{candidate.count?` · ${t('mail-smart-archive-history',{count:candidate.count,date:formatDateTime(candidate.latest)})}`:''}</option>}</For>
     </select>
    </label>
    <Show when={group.fallback}><p class={css.meta}>{t('mail-smart-archive-no-history')}</p></Show>
   </div>}</For>
   <div class={css.actions}><button class={css.button} type="submit" disabled={!valid()}>{t('mail-smart-archive')}</button><button class={css.ghost} type="button" onClick={close}>{t('common-cancel')}</button></div>
  </form>
 </div></Portal>;
}
