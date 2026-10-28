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
 const readOnly=()=>!setting.entry()||setting.locked()||setting.saving();
 return <Show when={nav.route().editor?.id==='delivery'} fallback={<section class={css.section}><h3 class={css.heading}>{t('receipts-delivery-title')}</h3><p class={css.prose}>{t('receipts-delivery-description')}</p><p>{t(setting.value()?'receipts-delivery-current-on':'receipts-delivery-current-off')}</p><div class={css.actions}><button class={css.button} onClick={()=>void nav.navigate({section:'receipts',editor:{kind:'edit',id:'delivery'}})}>{t('receipts-delivery-edit')}</button></div></section>}>
  <section class={css.section}><h2 class={css.heading}>{t('receipts-delivery-edit')}</h2><p class={css.prose}>{t('receipts-delivery-description')}</p>
  <form class={css.group} onSubmit={event=>void save(event)}><fieldset class={css.group} disabled={readOnly()}>
   <label class={css.check}><input type="checkbox" class={css.checkbox} checked={draft()} onChange={event=>{setDraft(event.currentTarget.checked);setSaved(false);}}/>{t('receipts-delivery-default')}</label>
  </fieldset><LockNote locked={setting.locked()}/><Show when={setting.error()}><p role="alert">{setting.error()}</p></Show>
  {/* Save, inherit and back share one row; back stays usable while the default is held. */}
  <div class={css.actions}>
   <button type="submit" class={css.button} disabled={readOnly()||!dirty()}>{t('receipts-save')}</button>
   <button type="button" class={css.ghost} disabled={readOnly()} onClick={()=>void persist(true)}>{t('reader-inherit')}</button>
   <button type="button" class={css.ghost} disabled={setting.saving()} onClick={()=>void guard.leave(back)}>{t('receipts-back')}</button>
  </div></form>{guard.dialog()}</section>
 </Show>;
}
