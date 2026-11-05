import {For,Show,type JSX} from 'solid-js';
import {useSetting} from './useSetting.ts';
import {LockNote} from './LockNote.tsx';
import {isRowActionsMode,MAIL_FEATURE_DEFAULTS,mailFeatureKey,ROW_ACTIONS_MODE_DEFAULT,ROW_ACTIONS_MODE_KEY,ROW_ACTIONS_MODES,type MailFeature,type RowActionsMode} from '../../state/mailFeatures.ts';
import {t} from '../../i18n';
import * as css from './styles.css.ts';
// The mail feature switches (`mail.allowSend` ... `mail.closeOnDelete`): each is
// a global/domain/user boolean, so an administrator can set the default or lock it.
const FEATURES=Object.keys(MAIL_FEATURE_DEFAULTS) as MailFeature[];
export function MailFeaturePreferences():JSX.Element {
 const choices=FEATURES.map(feature=>({feature,key:mailFeatureKey(feature),handle:useSetting<boolean>(mailFeatureKey(feature),MAIL_FEATURE_DEFAULTS[feature])}));
 const rowActions=useSetting<RowActionsMode>(ROW_ACTIONS_MODE_KEY,ROW_ACTIONS_MODE_DEFAULT,isRowActionsMode);
 const rowActionsDisabled=()=>!rowActions.entry()||rowActions.locked()||rowActions.saving();
 return <section class={css.section} aria-label={t('settings-mail-features')}><h3 class={css.heading}>{t('settings-mail-features')}</h3><div class={css.field}><label for="row-actions-mode">{t('settings-row-actions-mode')}</label><select id="row-actions-mode" class={css.select} value={rowActions.value()} disabled={rowActionsDisabled()} aria-describedby="row-actions-mode-help row-actions-mode-lock" onChange={event=>void rowActions.set(event.currentTarget.value as RowActionsMode)}><For each={ROW_ACTIONS_MODES}>{mode=><option value={mode}>{t(`settings-row-actions-mode-${mode}`)}</option>}</For></select><p id="row-actions-mode-help" class={css.meta}>{t('settings-row-actions-mode-help')}</p><LockNote id="row-actions-mode-lock" locked={rowActions.locked()}/><Show when={rowActions.error()}><p role="alert">{rowActions.error()}</p></Show><button type="button" class={css.ghost} disabled={rowActionsDisabled()} onClick={()=>void rowActions.reset()}>{t('reader-inherit')}</button></div><For each={choices}>{choice=>{
  const disabled=()=>!choice.handle.entry()||choice.handle.locked()||choice.handle.saving();
  return <div class={css.field}><label class={css.check}><input class={css.checkbox} type="checkbox" checked={choice.handle.value()} disabled={disabled()} onChange={event=>void choice.handle.set(event.currentTarget.checked)} aria-describedby={`${choice.key}-help`}/>{t(`settings-mail-feature-${choice.feature}`)}</label>
   <p id={`${choice.key}-help`} class={css.meta}>{t(`settings-mail-feature-${choice.feature}-help`)}</p><LockNote locked={choice.handle.locked()}/><Show when={choice.handle.error()}><p role="alert">{choice.handle.error()}</p></Show>
   <button type="button" class={css.ghost} disabled={disabled()} onClick={()=>void choice.handle.reset()}>{t('reader-inherit')}</button></div>;
 }}</For></section>;
}
