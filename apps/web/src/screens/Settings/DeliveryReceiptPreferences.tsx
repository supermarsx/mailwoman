import {createEffect,createSignal,onMount,onCleanup,untrack,Show,type JSX} from 'solid-js';
import {loadCatalog,t} from '../../i18n';
import {useSetting} from './useSetting.ts';
import {useEditorGuard,useEntityNavigation} from './entityEditor.tsx';
import {LockNote} from './LockNote.tsx';
import * as css from './styles.css.ts';
export function DeliveryReceiptPreferences(props:{accountId:string}):JSX.Element {
 const nav=useEntityNavigation('receipts');const setting=useSetting<boolean>('mail.requestDeliveryReceipt',false);
 const [draft,setDraft]=createSignal(false);const [saved,setSaved]=createSignal(false);let version=0;let initialized=false;
 onMount(()=>{void loadCatalog('receipts');});
 onCleanup(()=>{version++;});
 createEffect(()=>{void props.accountId;version++;initialized=false;setDraft(untrack(()=>setting.value()));setSaved(false);});
 createEffect(()=>{const entry=setting.entry();if(entry&&(!initialized||setting.locked())){initialized=true;setDraft(setting.value());}});
 const dirty=()=>!saved()&&draft()!==setting.value();const guard=useEditorGuard(dirty,setting.saving);
 const back=()=>nav.navigate({section:'receipts'}).then(()=>undefined);
 async function persist(inherit=false):Promise<void>{if(setting.locked()||setting.saving()||!setting.entry())return;
  const epoch=version;const account=props.accountId;const ok=inherit?await setting.reset():await setting.set(draft());
  if(ok){if(epoch!==version||account!==props.accountId)return;setSaved(true);await back();}}
 function save(event:SubmitEvent):void{event.preventDefault();void persist();}
 return <Show when={nav.route().editor?.id==='delivery'} fallback={<section><h3 class={css.heading}>{t('receipts-delivery-title')}</h3><p>{t('receipts-delivery-description')}</p><button class={css.button} onClick={()=>void nav.navigate({section:'receipts',editor:{kind:'edit',id:'delivery'}})}>{t('receipts-delivery-edit')}</button></section>}>
  <h2 class={css.heading}>{t('receipts-delivery-edit')}</h2><p>{t('receipts-delivery-description')}</p>
  <form onSubmit={event=>void save(event)}><fieldset disabled={!setting.entry()||setting.locked()||setting.saving()}>
   <label class={css.check}><input type="checkbox" class={css.checkbox} checked={draft()} onChange={event=>{setDraft(event.currentTarget.checked);setSaved(false);}}/>{t('receipts-delivery-default')}</label>
   <button type="submit" class={css.button} disabled={!dirty()}>{t('receipts-save')}</button>
   <button type="button" class={css.ghost} onClick={()=>void persist(true)}>{t('reader-inherit')}</button>
  </fieldset></form><LockNote locked={setting.locked()}/><Show when={setting.error()}><p role="alert">{setting.error()}</p></Show>
  <button class={css.ghost} disabled={setting.saving()} onClick={()=>void guard.leave(back)}>{t('receipts-back')}</button>{guard.dialog()}
 </Show>;
}
