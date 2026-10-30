import {createSignal,For,Show,type Accessor,type JSX} from 'solid-js';
import type {FocusedSender} from '../../api/focusedSenders.ts';
import {t} from '../../i18n';
import * as css from './styles.css.ts';
// t94: "Manage Focused senders" under the Focused inbox switch (Settings >
// Folders and sidebar): the senders the user chose to always move to Focused
// or Other, each with a Remove button. Removing one lets the automatic sorting
// decide again; the Inbox updates at once. The list is the server's (per user).
export function FocusedSenders(props:{senders:Accessor<FocusedSender[]>;onRemove:(email:string)=>Promise<void>}):JSX.Element {
 const [busy,setBusy]=createSignal<string|null>(null);
 const [error,setError]=createSignal(false);
 async function remove(email:string):Promise<void> {
  setBusy(email);setError(false);
  try{await props.onRemove(email);}catch{setError(true);}finally{setBusy(null);}
 }
 return <div class={css.field} data-testid="focused-senders" role="group" aria-labelledby="focused-senders-title">
  <h3 id="focused-senders-title" class={css.subheading}>{t('settings-focused-senders-title')}</h3>
  <p class={css.meta}>{t('settings-focused-senders-help')}</p>
  <Show when={props.senders().length>0} fallback={<p class={css.meta} data-testid="focused-senders-empty">{t('settings-focused-senders-empty')}</p>}>
   <ul class={css.list}><For each={props.senders()}>{row=><li class={css.item} data-testid="focused-sender" data-email={row.email}>
    <span class={css.itemMain}><span class={css.itemName}>{row.email}</span><span class={css.meta}>{t(row.tab==='focused'?'settings-focused-senders-to-focused':'settings-focused-senders-to-other')}</span></span>
    <button type="button" class={css.ghost} disabled={busy()===row.email} aria-label={t('settings-focused-senders-remove-label',{sender:row.email})} onClick={()=>void remove(row.email)}>{t('settings-focused-senders-remove')}</button>
   </li>}</For></ul>
  </Show>
  <Show when={error()}><p role="alert" class={css.error}>{t('settings-focused-senders-remove-failed')}</p></Show>
 </div>;
}
