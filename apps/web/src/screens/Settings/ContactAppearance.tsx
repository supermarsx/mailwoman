import {For,onMount,type JSX} from 'solid-js';
import {loadCatalog,t} from '../../i18n';
import {useSetting} from './useSetting.ts';
import {SettingRow} from './layout/SettingRow.tsx';
import {SettingsCard} from './layout/SettingsCard.tsx';
import {ToggleRow,type CardSpec} from './layout/rows.tsx';
import * as css from './styles.css.ts';
export const AVATAR_CARD:CardSpec={id:'avatars-card',section:'reader',title:'avatars-title',rows:[
 {id:'avatars-mode',label:'avatars-mode',help:'avatars-help'},{id:'avatars-sender-icons',label:'avatars-sender-icons',help:'avatars-sender-icons-help'},
 {id:'avatars-brand-avatars',label:'avatars-brand-avatars',help:'avatars-brand-avatars-help'},{id:'avatars-favorites-first',label:'avatars-favorites-first',help:'avatars-favorites-first-help'}]};
export function ContactAppearance():JSX.Element {
 onMount(()=>void loadCatalog('avatars'));
 const setting=useSetting<string>('contacts.avatarMode','initials',(value):value is string=>typeof value==='string'&&['off','initials','contacts'].includes(value));
 const icons=useSetting<boolean>('contacts.senderIcons',true,(value):value is boolean=>typeof value==='boolean');
 const brands=useSetting<boolean>('mail.brandAvatars',true,(value):value is boolean=>typeof value==='boolean');
 const favorites=useSetting<boolean>('contacts.favoritesFirst',true,(value):value is boolean=>typeof value==='boolean');
 const disabled=()=>!setting.entry()||setting.locked()||setting.saving();
 return <SettingsCard id="avatars-card" title={t('avatars-title')}>
  <SettingRow controlId="avatars-mode" label={t('avatars-mode')} help={t('avatars-help')} handle={setting}>{d=>
   <select id="avatars-mode" class={css.select} aria-describedby={d()} disabled={disabled()} value={setting.value()} onChange={event=>void setting.set(event.currentTarget.value)}><For each={['off','initials','contacts']}>{value=><option value={value}>{t(`avatars-${value}`)}</option>}</For></select>}</SettingRow>
  <ToggleRow id="avatars-sender-icons" label={t('avatars-sender-icons')} help={t('avatars-sender-icons-help')} handle={icons}/>
  <ToggleRow id="avatars-brand-avatars" label={t('avatars-brand-avatars')} help={t('avatars-brand-avatars-help')} handle={brands}/>
  <ToggleRow id="avatars-favorites-first" label={t('avatars-favorites-first')} help={t('avatars-favorites-first-help')} handle={favorites}/>
 </SettingsCard>;
}
