import {For,onCleanup,type JSX} from 'solid-js';
import {preferenceAccount} from '../../api/accountPreferenceLifecycle.ts';
import {t} from '../../i18n';
import {toast} from '../../components/toast/index.ts';
import {useSetting,type SettingHandle} from './useSetting.ts';
import {LockNote} from './LockNote.tsx';
import * as css from './styles.css.ts';
export function ComposeDefaults():JSX.Element{
 const windowMode=useSetting('mail.composeWindow','tab',(v):v is string=>v==='tab'||v==='window');
 const forward=useSetting('mail.forwardMode','inline',(v):v is string=>v==='inline'||v==='attachment');
 const toggles=[{handle:useSetting('mail.requestReadReceipt',false),label:'settings-compose-read-receipt'},{handle:useSetting('mail.requestDeliveryReceipt',false),label:'settings-compose-delivery-receipt'},{handle:useSetting('mail.replyCopyToOriginal',false),label:'settings-compose-reply-copy'}];
 const disabled=(handle:SettingHandle<unknown>)=>!handle.entry()||handle.locked()||handle.saving();
 let disposed=false;const pending=new Set<string>();onCleanup(()=>{disposed=true;for(const id of pending)toast.dismiss(id);});
 async function save<T>(handle:SettingHandle<T>,value:T):Promise<void>{if(disabled(handle))return;const owner=preferenceAccount(),id=toast.info(t('settings-compose-saving'),{busy:true,duration:Infinity});pending.add(id);const ok=await handle.set(value);pending.delete(id);if(disposed||preferenceAccount()!==owner){toast.dismiss(id);return;}if(ok)toast.success(t('settings-compose-saved'),{id});else toast.error(handle.error(),{id});}
 return <>
  <div class={css.field}><label for="compose-window">{t('settings-compose-window')}</label><select id="compose-window" class={css.select} value={windowMode.value()} disabled={disabled(windowMode)} onChange={e=>void save(windowMode,e.currentTarget.value)}><option value="tab">{t('settings-compose-window-tab')}</option><option value="window">{t('settings-compose-window-new')}</option></select><p class={css.meta}>{t('settings-compose-window-help')}</p><LockNote locked={windowMode.locked()}/></div>
  <For each={toggles}>{item=><div class={css.field}><label class={css.check}><input type="checkbox" class={css.checkbox} checked={item.handle.value()} disabled={disabled(item.handle)} onChange={e=>void save(item.handle,e.currentTarget.checked)}/>{t(item.label)}</label><LockNote locked={item.handle.locked()}/></div>}</For>
  <p class={css.meta}>{t('settings-compose-receipts-help')}</p>
  <div class={css.field}><label for="compose-forward">{t('settings-compose-forward')}</label><select id="compose-forward" class={css.select} value={forward.value()} disabled={disabled(forward)} onChange={e=>void save(forward,e.currentTarget.value)}><option value="inline">{t('settings-compose-forward-inline')}</option><option value="attachment">{t('settings-compose-forward-attachment')}</option></select><LockNote locked={forward.locked()}/></div>
 </>;
}
