import {createEffect,createMemo,createSignal,onMount,Show,type JSX} from 'solid-js';
import {loadCatalog,t} from '../../i18n';
import {SnoozePresetEditor} from '../../components/SnoozePresetEditor.tsx';
import {DEFAULT_SNOOZE_PRESETS,SNOOZE_PRESETS_KEY,validSnoozePresets,type SnoozePreset} from '../../state/snooze.ts';
import {useSetting} from './useSetting.ts';
import {useEditorGuard} from './entityEditor.tsx';
import {SettingsCard} from './layout/SettingsCard.tsx';
import type {CardSpec} from './layout/rows.tsx';
import {LockNote} from './LockNote.tsx';
import {ControlRow} from './layout/SettingRow.tsx';
import * as css from './styles.css.ts';
export const SNOOZE_CARD:CardSpec={id:'reader-card-snooze',section:'reader',title:'snooze-title',help:'snooze-help',rows:[{id:'snooze-presets-editor',label:'snooze-title',keywords:'snooze later postpone hours days end week month presets custom date time'}]};
export function SnoozePreferences():JSX.Element{
 onMount(()=>void loadCatalog('snooze'));
 const setting=useSetting(SNOOZE_PRESETS_KEY,DEFAULT_SNOOZE_PRESETS,validSnoozePresets);
 const [draft,setDraft]=createSignal<SnoozePreset[]>(setting.value());
 const saved=createMemo(()=>JSON.stringify(setting.value()));
 createEffect(()=>setDraft(JSON.parse(saved()) as SnoozePreset[]));
 const dirty=()=>JSON.stringify(draft())!==JSON.stringify(setting.value());
 const disabled=()=>!setting.entry()||setting.locked()||setting.saving();
 const guard=useEditorGuard(dirty,setting.saving);
 return <SettingsCard id={SNOOZE_CARD.id} wide title={t('snooze-title')} description={t('snooze-help')}>
  <LockNote locked={setting.locked()}/>
  <ControlRow controlId="snooze-presets-editor" group label={t('snooze-title')}>{()=> <SnoozePresetEditor value={draft()} onChange={setDraft} disabled={disabled()}/>}</ControlRow>
  <Show when={!validSnoozePresets(draft())}><p role="alert" class={css.error}>{t('snooze-invalid-presets')}</p></Show>
  <Show when={setting.error()}><p role="alert" class={css.error}>{setting.error()}</p></Show>
  <div class={css.actions}><button type="button" class={css.button} disabled={disabled()||!dirty()||!validSnoozePresets(draft())} onClick={()=>void setting.set(draft())}>{t('snooze-save')}</button><button type="button" class={css.ghost} disabled={disabled()} onClick={()=>void setting.reset()}>{t('reader-inherit')}</button></div>
  {guard.dialog()}
 </SettingsCard>;
}
