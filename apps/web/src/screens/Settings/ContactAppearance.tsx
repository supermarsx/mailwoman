import {For,onMount,Show,type JSX} from 'solid-js';
import {loadCatalog,t} from '../../i18n';
import {useSetting} from './useSetting.ts';
import {LockNote} from './LockNote.tsx';
import * as css from './styles.css.ts';
export function ContactAppearance():JSX.Element {
 onMount(()=>void loadCatalog('avatars'));
 const setting=useSetting<string>('contacts.avatarMode','initials',(value):value is string=>typeof value==='string'&&['off','initials','contacts'].includes(value));
 const disabled=()=>!setting.entry()||setting.locked()||setting.saving();
 return <section class={css.section}><h3>{t('avatars-title')}</h3><label class={css.field}><span>{t('avatars-mode')}</span><select class={css.select} aria-label={t('avatars-mode')} aria-describedby="avatars-help" disabled={disabled()} value={setting.value()} onChange={event=>void setting.set(event.currentTarget.value)}><For each={['off','initials','contacts']}>{value=><option value={value}>{t(`avatars-${value}`)}</option>}</For></select></label><p id="avatars-help" class={css.meta}>{t('avatars-help')}</p><LockNote locked={setting.locked()}/><Show when={setting.error()}><p role="alert">{setting.error()}</p></Show><button class={css.ghost} disabled={disabled()} onClick={()=>void setting.reset()}>{t('avatars-default')}</button></section>;
}
