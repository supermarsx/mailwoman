import { createEffect, createMemo, createSignal, For, onCleanup, Show, type JSX } from 'solid-js';
import { validTimeZone, supportedTimeZones } from '../../i18n/timezones.ts';
import {SearchableSelect} from '../../components/SearchableSelect.tsx';
import { t } from '../../i18n';
import { useSetting } from './useSetting.ts';
import { LockNote } from './LockNote.tsx';
import type { SettingsNavigation } from './navigation.tsx';
import * as css from './styles.css.ts';

function ZoneField(props:{kind:'displayZone'|'comparisonZone';navigation:SettingsNavigation}):JSX.Element {
  const setting=useSetting<string>(`time.${props.kind}`,props.kind==='displayZone'?'local':'');
  const [draft,setDraft]=createSignal(setting.value());const [error,setError]=createSignal('');
  const saved=createMemo(()=>setting.value());createEffect(()=>setDraft(saved()));
  const disabled=()=>!setting.entry()||setting.locked()||setting.saving();
  const options=[{value:props.kind==='displayZone'?'local':'',label:t(props.kind==='displayZone'?'settings-time-local':'settings-time-none')},...supportedTimeZones().map(value=>({value,label:value.replaceAll('_',' ')}))];
  onCleanup(props.navigation.beforeLeave(()=>draft()===setting.value()||window.confirm(t('settings-time-discard'))));
  async function save(event:SubmitEvent):Promise<void> {
    event.preventDefault();const value=draft().trim();
    if(!validTimeZone(value)&&!(props.kind==='displayZone'&&value==='local')&&!(props.kind==='comparisonZone'&&value==='')){setError(t('settings-time-invalid'));return;}
    setError('');if(await setting.set(value))setDraft(value);
  }
  return <form class={css.section} onSubmit={event=>void save(event)}>
    <SearchableSelect label={t(`settings-time-${props.kind}`)} value={draft()} options={options} disabled={disabled()} onChange={setDraft} describedBy={`time-${props.kind}-help`}/>
    <p id={`time-${props.kind}-help`} class={css.meta}>{t(`settings-time-${props.kind}-help`)}</p><LockNote locked={setting.locked()}/>
    <Show when={error()||setting.error()}><p role="alert">{error()||setting.error()}</p></Show>
    <div class={css.actions}><button type="submit" class={css.button} disabled={disabled()||draft()===setting.value()}>{t('settings-time-save')}</button><button type="button" class={css.ghost} disabled={disabled()} onClick={()=>void setting.reset()}>{t('reader-inherit')}</button></div>
  </form>;
}
export function TimeZones(props:{navigation:SettingsNavigation}):JSX.Element {
  const helpers=useSetting<boolean>('time.showHelpers',true);
  return <section class={css.section} aria-label={t('settings-time-title')}><h3 class={css.heading}>{t('settings-time-title')}</h3><p class={css.meta}>{t('settings-time-help')}</p>
    <label class={css.check}><input class={css.checkbox} type="checkbox" checked={helpers.value()} disabled={!helpers.entry()||helpers.locked()||helpers.saving()} onChange={event=>void helpers.set(event.currentTarget.checked)}/>{t('settings-time-helpers')}</label><LockNote locked={helpers.locked()}/>
    <Show when={helpers.error()}><p role="alert">{helpers.error()}</p></Show>
    <For each={['displayZone','comparisonZone'] as const}>{kind=><ZoneField kind={kind} navigation={props.navigation}/>}</For>
  </section>;
}
