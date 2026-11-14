import {createEffect,createMemo,createSignal,onMount,Show,type JSX} from 'solid-js';
import {loadCatalog,t} from '../../i18n';
import {SnoozePresetEditor} from '../../components/SnoozePresetEditor.tsx';
import {DEFAULT_SNOOZE_PRESETS,SNOOZE_PRESETS_KEY,FOLLOW_UP_PRESETS_KEY,validSnoozePresets,type SnoozePreset} from '../../state/snooze.ts';
import {useSetting} from './useSetting.ts';
import {useEditorGuard} from './entityEditor.tsx';
import {SettingsCard} from './layout/SettingsCard.tsx';
import type {CardSpec} from './layout/rows.tsx';
import {LockNote} from './LockNote.tsx';
import {ControlRow} from './layout/SettingRow.tsx';
import * as css from './styles.css.ts';
export const SNOOZE_CARD:CardSpec={id:'reader-card-snooze',section:'reader',title:'snooze-title',help:'snooze-help',rows:[{id:'snooze-presets-editor',label:'snooze-title',keywords:'snooze later postpone hours days end week month presets custom date time'}]};
export const FOLLOW_UP_CARD:CardSpec={id:'reader-card-follow-up',section:'reader',title:'follow-up-title',help:'follow-up-help',rows:[{id:'follow-up-presets-editor',label:'follow-up-title',keywords:'follow up flag reminder weekday ordinal month year presets'}]};
export function SnoozePreferences(props:{followUp?:boolean}={}):JSX.Element{
 const card=()=>props.followUp?FOLLOW_UP_CARD:SNOOZE_CARD;
 onMount(()=>void loadCatalog('snooze'));
 const setting=useSetting(props.followUp?FOLLOW_UP_PRESETS_KEY:SNOOZE_PRESETS_KEY,DEFAULT_SNOOZE_PRESETS,validSnoozePresets);
 const [draft,setDraft]=createSignal<SnoozePreset[]>(setting.value());
 const saved=createMemo(()=>JSON.stringify(setting.value()));
 createEffect(()=>setDraft(JSON.parse(saved()) as SnoozePreset[]));
 const dirty=()=>JSON.stringify(draft())!==JSON.stringify(setting.value());
 const disabled=()=>!setting.entry()||setting.locked()||setting.saving();
 const guard=useEditorGuard(dirty,setting.saving);
 return <SettingsCard id={card().id} wide title={t(card().title)} description={t(card().help!)}>
  <LockNote locked={setting.locked()}/><p class={css.meta}>{t('schedule-defaults-help')}</p>
  <ControlRow controlId={props.followUp?'follow-up-presets-editor':'snooze-presets-editor'} group label={t(card().title)}>{()=> <SnoozePresetEditor value={draft()} onChange={setDraft} disabled={disabled()}/>}</ControlRow>
  <Show when={!validSnoozePresets(draft())}><p role="alert" class={css.error}>{t('snooze-invalid-presets')}</p></Show>
  <Show when={setting.error()}><p role="alert" class={css.error}>{setting.error()}</p></Show>
  <div class={css.actions}><button type="button" class={css.button} disabled={disabled()||!dirty()||!validSnoozePresets(draft())} onClick={()=>void setting.set(draft())}>{t(props.followUp?'follow-up-save':'snooze-save')}</button><button type="button" class={css.ghost} disabled={disabled()} onClick={()=>void setting.reset()}>{t('reader-inherit')}</button></div>
  {guard.dialog()}
 </SettingsCard>;
}
