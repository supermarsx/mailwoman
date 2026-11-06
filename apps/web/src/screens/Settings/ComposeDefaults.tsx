import {For,onCleanup,type JSX} from 'solid-js';
import {preferenceAccount} from '../../api/accountPreferenceLifecycle.ts';
import {featureOn} from '../../state/features.ts';
import {t} from '../../i18n';
import {toast} from '../../components/toast/index.ts';
import {useSetting,type SettingHandle} from './useSetting.ts';
import {SelectRow,ToggleRow,type CardSpec} from './layout/rows.tsx';
import {SettingRow} from './layout/SettingRow.tsx';
import {SettingsCard} from './layout/SettingsCard.tsx';
export const COMPOSE_DEFAULTS_CARD: CardSpec = { id: 'compose-card-defaults', section: 'compose', title: 'settings-compose-card-defaults', help: 'settings-compose-receipts-help', rows: [
  { id: 'compose-window', label: 'settings-compose-window', help: 'settings-compose-window-help' },
  { id: 'compose-read-receipt', label: 'settings-compose-read-receipt' }, { id: 'compose-delivery-receipt', label: 'settings-compose-delivery-receipt' }, { id: 'compose-reply-copy', label: 'settings-compose-reply-copy' },
  { id: 'compose-tracking', label: 'settings-compose-open-tracking', help: 'settings-compose-open-tracking-help' },
  { id: 'compose-forward', label: 'settings-compose-forward' },
] };
export function ComposeDefaults():JSX.Element{
 const windowMode=useSetting('mail.composeWindow','tab',(v):v is string=>v==='tab'||v==='window');
 const forward=useSetting('mail.forwardMode','inline',(v):v is string=>v==='inline'||v==='attachment');
 const tracking=useSetting('mail.openTracking',false),trackingCap=useSetting('cap.compose.open_tracking',true);
 const trackingAllowed=()=>trackingCap.value()&&featureOn('compose.open_tracking');
 const toggles=[{handle:useSetting('mail.requestReadReceipt',false),label:'settings-compose-read-receipt'},{handle:useSetting('mail.requestDeliveryReceipt',false),label:'settings-compose-delivery-receipt'},{handle:useSetting('mail.replyCopyToOriginal',false),label:'settings-compose-reply-copy'}];
 const disabled=(handle:SettingHandle<unknown>)=>!handle.entry()||handle.locked()||handle.saving();
 let disposed=false;const pending=new Set<string>();onCleanup(()=>{disposed=true;for(const id of pending)toast.dismiss(id);});
 async function save<T>(handle:SettingHandle<T>,value:T):Promise<void>{if(disabled(handle))return;const owner=preferenceAccount(),id=toast.info(t('settings-compose-saving'),{busy:true,duration:Infinity});pending.add(id);const ok=await handle.set(value);pending.delete(id);if(disposed||preferenceAccount()!==owner){toast.dismiss(id);return;}if(ok)toast.success(t('settings-compose-saved'),{id});else toast.error(handle.error(),{id});}
 async function resetTracking():Promise<void>{if(disabled(tracking)||!trackingAllowed())return;const owner=preferenceAccount(),id=toast.info(t('settings-compose-saving'),{busy:true,duration:Infinity});pending.add(id);const ok=await tracking.reset();pending.delete(id);if(disposed||preferenceAccount()!==owner){toast.dismiss(id);return;}if(ok)toast.success(t('settings-compose-saved'),{id});else toast.error(tracking.error(),{id});}
 return <SettingsCard id="compose-card-defaults" title={t('settings-compose-card-defaults')} description={t('settings-compose-receipts-help')}>
  <SelectRow id="compose-window" label={t('settings-compose-window')} help={t('settings-compose-window-help')} handle={windowMode} accept={(v):v is string=>v==='tab'||v==='window'} onPick={v=>void save(windowMode,v)} options={[{value:'tab',label:t('settings-compose-window-tab')},{value:'window',label:t('settings-compose-window-new')}]}/>
  <For each={toggles}>{(item,i)=><ToggleRow id={['compose-read-receipt','compose-delivery-receipt','compose-reply-copy'][i()]!} label={t(item.label)} handle={item.handle} onChange={v=>void save(item.handle,v)}/>}</For>
  <SettingRow controlId="compose-tracking" label={t('settings-compose-open-tracking')} help={t('settings-compose-open-tracking-help')} handle={tracking} lockNote={tracking.locked()||!trackingAllowed()} resetLabel={t('settings-compose-open-tracking-inherit')} resetDisabled={disabled(tracking)||!trackingAllowed()} onReset={()=>void resetTracking()}>{describedBy=>
   <input id="compose-tracking" type="checkbox" checked={trackingAllowed()&&tracking.value()} disabled={disabled(tracking)||!trackingAllowed()} aria-describedby={describedBy()} onChange={e=>{if(trackingAllowed())void save(tracking,e.currentTarget.checked);}}/>}</SettingRow>
  <SelectRow id="compose-forward" label={t('settings-compose-forward')} handle={forward} accept={(v):v is string=>v==='inline'||v==='attachment'} onPick={v=>void save(forward,v)} options={[{value:'inline',label:t('settings-compose-forward-inline')},{value:'attachment',label:t('settings-compose-forward-attachment')}]}/>
 </SettingsCard>;
}
