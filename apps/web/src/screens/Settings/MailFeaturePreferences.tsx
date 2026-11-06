import {For,type JSX} from 'solid-js';
import {useSetting} from './useSetting.ts';
import {SettingRow} from './layout/SettingRow.tsx';
import {SettingsCard} from './layout/SettingsCard.tsx';
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
 return <SettingsCard id="settings-card-mail-features" title={t('settings-mail-features')} description={t('settings-mail-card-list-help')}>
  <SettingRow controlId="row-actions-mode" label={t('settings-row-actions-mode')} help={t('settings-row-actions-mode-help')} handle={rowActions}>{describedBy=>
   <select id="row-actions-mode" class={css.select} value={rowActions.value()} disabled={rowActionsDisabled()} aria-describedby={describedBy()} onChange={event=>void rowActions.set(event.currentTarget.value as RowActionsMode)}><For each={ROW_ACTIONS_MODES}>{mode=><option value={mode}>{t(`settings-row-actions-mode-${mode}`)}</option>}</For></select>}</SettingRow>
  <For each={choices}>{choice=>{
   const disabled=()=>!choice.handle.entry()||choice.handle.locked()||choice.handle.saving();
   return <SettingRow controlId={choice.key} label={t(`settings-mail-feature-${choice.feature}`)} help={t(`settings-mail-feature-${choice.feature}-help`)} handle={choice.handle}>{describedBy=>
    <input id={choice.key} type="checkbox" checked={choice.handle.value()} disabled={disabled()} onChange={event=>void choice.handle.set(event.currentTarget.checked)} aria-describedby={describedBy()}/>}</SettingRow>;
  }}</For></SettingsCard>;
}
