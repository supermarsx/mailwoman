import {Show,type JSX} from 'solid-js';
import {useSetting} from './useSetting.ts';
import {t} from '../../i18n';
import {SettingRow} from './layout/SettingRow.tsx';
import {SettingsCard} from './layout/SettingsCard.tsx';
import type {CardSpec} from './layout/rows.tsx';
import * as css from './styles.css.ts';
import {SendDelayPreferences} from './SendDelayPreferences.tsx';
export const DRAFT_SAVE_CARD:CardSpec={id:'compose-card-drafts',section:'compose',title:'settings-draft-save',help:'settings-draft-save-help',rows:[{id:'draft-save-mode',label:'settings-draft-save-mode'},{id:'draft-debounce',label:'settings-draft-debounce'},{id:'draft-interval',label:'settings-draft-interval'}]};
export function DraftSavePreferences():JSX.Element{
 const mode=useSetting<string>('mail.draftSaveMode','realtime'),debounce=useSetting<number>('mail.draftDebounceMs',1000),interval=useSetting<number>('mail.draftIntervalSecs',30);
 const off=(h:{entry():unknown;locked():boolean;saving():boolean})=>!h.entry()||h.locked()||h.saving();
 return <><SendDelayPreferences/><SettingsCard id="compose-card-drafts" title={t('settings-draft-save')} description={t('settings-draft-save-help')}>
 <SettingRow controlId="draft-save-mode" label={t('settings-draft-save-mode')} handle={mode} onReset={()=>{void mode.reset();void debounce.reset();void interval.reset();}} resetDisabled={mode.locked()||debounce.locked()||interval.locked()||mode.saving()||debounce.saving()||interval.saving()}>{d=>
  <select id="draft-save-mode" class={css.select} value={mode.value()} disabled={off(mode)} aria-describedby={d()} onChange={e=>void mode.set(e.currentTarget.value)}><option value="realtime">{t('settings-draft-realtime')}</option><option value="periodic">{t('settings-draft-periodic')}</option><option value="manual">{t('settings-draft-manual')}</option></select>}</SettingRow>
 <Show when={mode.value()==='realtime'}><SettingRow controlId="draft-debounce" label={t('settings-draft-debounce')} handle={debounce}>{d=><input id="draft-debounce" class={css.input} type="number" min="500" max="10000" step="100" value={debounce.value()} disabled={off(debounce)} aria-describedby={d()} onChange={e=>{const n=e.currentTarget.valueAsNumber;if(Number.isInteger(n)&&n>=500&&n<=10000)void debounce.set(n);}}/>}</SettingRow></Show>
 <Show when={mode.value()==='periodic'}><SettingRow controlId="draft-interval" label={t('settings-draft-interval')} handle={interval}>{d=><input id="draft-interval" class={css.input} type="number" min="5" max="3600" value={interval.value()} disabled={off(interval)} aria-describedby={d()} onChange={e=>{const n=e.currentTarget.valueAsNumber;if(Number.isInteger(n)&&n>=5&&n<=3600)void interval.set(n);}}/>}</SettingRow></Show>
 <Show when={mode.error()||debounce.error()||interval.error()}><p role="alert">{mode.error()||debounce.error()||interval.error()}</p></Show>
 </SettingsCard></>;
}
