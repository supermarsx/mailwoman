import {Show,type JSX} from 'solid-js';
import {useSetting} from './useSetting.ts';
import {LockNote} from './LockNote.tsx';
import {t} from '../../i18n';
import * as css from './styles.css.ts';
// The Focused inbox (t73): off by default, turned on here. The Inbox header
// then shows the Focused / Other tabs; it has no switch of its own. An
// administrator can set the default or lock it (the checkbox is then disabled).
export function FocusedInboxPreference():JSX.Element {
 const focused=useSetting<boolean>('mail.focusedInbox',false);
 const disabled=()=>!focused.entry()||focused.locked()||focused.saving();
 return <div class={css.field} data-testid="focused-inbox-preference"><label class={css.check}><input class={css.checkbox} type="checkbox" checked={focused.value()} disabled={disabled()} onChange={event=>void focused.set(event.currentTarget.checked)} aria-describedby="mail-focused-inbox-help"/>{t('settings-focused-inbox')}</label>
  <p id="mail-focused-inbox-help" class={css.meta}>{t('settings-focused-inbox-help')}</p><LockNote locked={focused.locked()}/><Show when={focused.error()}><p role="alert">{focused.error()}</p></Show></div>;
}
