import {SettingsCard} from './layout/SettingsCard.tsx';
import {type JSX} from 'solid-js';
import {useSetting} from './useSetting.ts';
import {ToggleRow} from './layout/rows.tsx';
import {t} from '../../i18n';
// The Focused inbox (t73): off by default, turned on here. The Inbox header
// then shows the Focused / Other tabs; it has no switch of its own. An
// administrator can set the default or lock it (the checkbox is then disabled).
export function FocusedInboxPreference(props:{children?:JSX.Element}={}):JSX.Element {
 const focused=useSetting<boolean>('mail.focusedInbox',false);
 return <SettingsCard id="folders-card-focused" title={t('settings-focused-inbox')} testId="focused-inbox-preference"><ToggleRow id="mail-focused-inbox" label={t('settings-focused-inbox')} help={t('settings-focused-inbox-help')} handle={focused}/>{props.children}</SettingsCard>;
}
