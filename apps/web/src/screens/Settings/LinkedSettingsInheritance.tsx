import {For,Show,onMount,type JSX} from 'solid-js';
import {useSetting} from './useSetting.ts';
import {loadSettings} from '../../api/settings.ts';
import {t,loadCatalog} from '../../i18n';
import {SettingsCard} from './layout/SettingsCard.tsx';
import {LockNote} from './LockNote.tsx';
import * as css from './styles.css.ts';
const groups=[['reader','st-group-reader'],['compose','st-group-compose'],['mail','st-group-mail'],['folders','st-group-folders'],['notifications','st-group-notifications'],['time','regional-title'],['interface','st-group-appearance'],['privacy','privacy-title']] as const;
export function LinkedSettingsInheritance():JSX.Element{
 const choice=useSetting<string[]>('accounts.settings.inheritGroups',[],(value):value is string[]=>Array.isArray(value)&&value.every(item=>typeof item==='string'));
 onMount(()=>{void loadCatalog('settings-transfer');void loadCatalog('regional');void loadCatalog('privacy');});
 return <SettingsCard id="linked-settings-inheritance" title={t('linked-settings-inheritance-title')} description={t('linked-settings-inheritance-help')}>
  <fieldset class={css.bareFieldset} disabled={!choice.entry()||choice.locked()||choice.saving()}><For each={groups}>{([group,label])=><label class={css.check}><input type="checkbox" checked={choice.value().includes(group)} onChange={event=>{const next=event.currentTarget.checked?[...choice.value(),group]:choice.value().filter(value=>value!==group);void choice.set(next).then(saved=>{if(saved)void loadSettings({refresh:true});});}}/>{t(label)}</label>}</For></fieldset>
  <LockNote locked={choice.locked()}/><Show when={choice.error()}><p role="alert">{choice.error()}</p></Show>
 </SettingsCard>;
}
