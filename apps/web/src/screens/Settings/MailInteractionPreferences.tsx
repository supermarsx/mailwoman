import {For,Show,type JSX} from 'solid-js';
import {useSetting} from './useSetting.ts';
import {LockNote} from './LockNote.tsx';
import {t} from '../../i18n';
import * as css from './styles.css.ts';
export function MailInteractionPreferences():JSX.Element {
 const choices=[
  {key:'mail.resumeTabs',label:'settings-mail-resume-tabs',help:'settings-mail-resume-tabs-help',handle:useSetting<boolean>('mail.resumeTabs',true)},
  {key:'mail.modifierSelection',label:'settings-mail-modifier-selection',help:'settings-mail-modifier-selection-help',handle:useSetting<boolean>('mail.modifierSelection',true)},
  {key:'mail.dragToFolder',label:'settings-mail-drag-folder',help:'settings-mail-drag-folder-help',handle:useSetting<boolean>('mail.dragToFolder',true)},
 ];
 return <section class={css.section} aria-label={t('settings-mail-interactions')}><h3 class={css.heading}>{t('settings-mail-interactions')}</h3><For each={choices}>{choice=>{
  const disabled=()=>!choice.handle.entry()||choice.handle.locked()||choice.handle.saving();
  return <div class={css.field}><label class={css.check}><input class={css.checkbox} type="checkbox" checked={choice.handle.value()} disabled={disabled()} onChange={event=>void choice.handle.set(event.currentTarget.checked)} aria-describedby={`${choice.key}-help`}/>{t(choice.label)}</label>
   <p id={`${choice.key}-help`} class={css.meta}>{t(choice.help)}</p><LockNote locked={choice.handle.locked()}/><Show when={choice.handle.error()}><p role="alert">{choice.handle.error()}</p></Show>
   <button type="button" class={css.ghost} disabled={disabled()} onClick={()=>void choice.handle.reset()}>{t('reader-inherit')}</button></div>;
 }}</For></section>;
}
