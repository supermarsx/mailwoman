import {For,onMount,type JSX} from 'solid-js';
import {loadCatalog,t} from '../../i18n';
import {useApp} from '../../state/context.ts';
import {useSetting} from './useSetting.ts';
import {SettingRow} from './layout/SettingRow.tsx';
import {SettingsCard} from './layout/SettingsCard.tsx';
import {SelectRow,ToggleRow,type CardSpec} from './layout/rows.tsx';
import * as css from './styles.css.ts';

export const SAVE_CONTACTS_CARD:CardSpec={id:'save-contacts-card',section:'reader',title:'contacts-save-settings-title',rows:[
 {id:'save-contacts-enabled',label:'contacts-save-settings-enabled',help:'contacts-save-settings-enabled-help'},
 {id:'save-contacts-sender',label:'contacts-save-settings-sender',help:'contacts-save-settings-sender-help'},
 {id:'save-contacts-scope',label:'contacts-save-settings-scope'},
 {id:'save-contacts-skip',label:'contacts-save-settings-skip',help:'contacts-save-settings-skip-help'},
 {id:'save-contacts-note',label:'contacts-save-settings-note',help:'contacts-save-settings-note-help'},
 {id:'save-contacts-reply-to',label:'contacts-save-settings-reply-to',help:'contacts-save-settings-reply-to-help'},
 {id:'save-contacts-book',label:'contacts-save-settings-book'}]};
const SCOPES=['sender','sender+replyto','all'] as const;
type Scope=typeof SCOPES[number];
const bool=(value:unknown):value is boolean=>typeof value==='boolean';
export function SaveContactsSettings():JSX.Element {
 onMount(()=>void loadCatalog('contacts'));
 const app=useApp();
 const enabled=useSetting<boolean>('contacts.saveFromMessage.enabled',true,bool);
 const sender=useSetting<boolean>('contacts.saveSender.enabled',true,bool);
 const scope=useSetting<Scope>('contacts.saveFromMessage.defaultScope','sender',(value):value is Scope=>SCOPES.includes(value as Scope));
 const skip=useSetting<boolean>('contacts.saveFromMessage.skipAutomated',true,bool);
 const note=useSetting<boolean>('contacts.saveFromMessage.addNote',true,bool);
 const replyTo=useSetting<boolean>('contacts.saveSender.useReplyTo',false,bool);
 const book=useSetting<string>('contacts.saveFromMessage.defaultBook','default',(value):value is string=>typeof value==='string');
 return <SettingsCard id="save-contacts-card" title={t('contacts-save-settings-title')}>
  <ToggleRow id="save-contacts-enabled" label={t('contacts-save-settings-enabled')} help={t('contacts-save-settings-enabled-help')} handle={enabled}/>
  <ToggleRow id="save-contacts-sender" label={t('contacts-save-settings-sender')} help={t('contacts-save-settings-sender-help')} handle={sender}/>
  <SelectRow id="save-contacts-scope" label={t('contacts-save-settings-scope')} handle={scope} accept={(value):value is Scope=>SCOPES.includes(value as Scope)} options={SCOPES.map(value=>({value,label:t(`contacts-save-settings-scope-${value}`)}))}/>
  <ToggleRow id="save-contacts-skip" label={t('contacts-save-settings-skip')} help={t('contacts-save-settings-skip-help')} handle={skip}/>
  <ToggleRow id="save-contacts-note" label={t('contacts-save-settings-note')} help={t('contacts-save-settings-note-help')} handle={note}/>
  <ToggleRow id="save-contacts-reply-to" label={t('contacts-save-settings-reply-to')} help={t('contacts-save-settings-reply-to-help')} handle={replyTo}/>
  <SettingRow controlId="save-contacts-book" label={t('contacts-save-settings-book')} handle={book}>{d=>
   <select id="save-contacts-book" class={css.select} aria-describedby={d()} disabled={!book.entry()||book.locked()||book.saving()} value={book.value()} onChange={event=>void book.set(event.currentTarget.value)}>
    <option value="default">{t('contacts-save-settings-book-default')}</option>
    <For each={app.addressBooks?.()??[]}>{item=><option value={item.id}>{item.name}</option>}</For>
   </select>}</SettingRow>
 </SettingsCard>;
}
