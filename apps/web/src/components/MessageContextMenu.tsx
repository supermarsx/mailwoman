import {onCleanup,type Accessor} from 'solid-js';
import {useApp} from '../state/context.ts';
import {observePreferenceAccount,preferenceAccount} from '../api/accountPreferenceLifecycle.ts';
import {t} from '../i18n';
import {relatedSearchValue} from './messageSearch.ts';
import type {Email} from '../api/jmap-types.ts';
import {snoozePresets} from './MessageActions.tsx';
import {createContextMenu,type ContextAction} from './ContextMenu.tsx';
/** Always targets the indicated message, not an unrelated existing selection. */
export function createMessageContextMenu(email:Accessor<Email|undefined>) {
 const app=useApp();
 const menu=createContextMenu({label:()=>t('mail-more-actions'),scope:()=>`${app.accountId()}:${preferenceAccount()?.generation}:${app.selectedMailboxId()}:${email()?.id}`,actions:()=>{
  const owner=preferenceAccount();if(!owner||owner.id!==app.accountId())return [];
  const message=app.messages().find(value=>value.id===email()?.id);if(!message)return [];const id=message.id;const busy=app.listLoading();
  const rows:ContextAction[]=[
   {id:'open',label:t('mail-context-open-message'),icon:'mail',run:()=>app.openMessage(id)},
   {id:'read',label:t(message.keywords?.$seen?'mail-mark-unread':'mail-mark-read'),icon:'mailRead',run:()=>app.markRead(id,!message.keywords?.$seen)},
   {id:'flag',label:t(message.keywords?.$flagged?'mail-unflag-message':'mail-flag-message'),icon:'flag',run:()=>app.flagMessage(id,!message.keywords?.$flagged)},
   {id:'pin',label:t(message.pinned?'mail-unpin':'mail-pin'),icon:'pin',run:()=>app.pinMessage(id,!message.pinned)},
  ];
  for(const field of ['from','subject'] as const){const value=relatedSearchValue(message,field);if(value)rows.push({id:`search:${field}`,label:t(field==='from'?'mail-context-search-sender':'mail-context-search-subject'),icon:'search',run:()=>app.searchMessageField(field,value)});}
  for(const [role,label,icon,run] of [
   ['archive','mail-archive','archive',()=>app.archiveMessage(id)],
   ['junk','mail-mark-spam','spam',()=>app.markSpam(id)],
   ['trash','mail-delete','delete',()=>app.trashMessage(id)],
  ] as const)if(app.mailboxes().some(box=>box.role===role&&!message.mailboxIds[box.id]))rows.push({id:role,label:t(label),icon,run});
  for(const preset of snoozePresets())rows.push({id:`snooze:${preset.labelId}`,label:`${t('mail-snooze')}: ${t(preset.labelId)}`,icon:'clock',run:()=>app.snoozeMessage(id,snoozePresets().find(value=>value.labelId===preset.labelId)!.at)});
  if(message.snoozedUntil)rows.push({id:'unsnooze',label:t('mail-unsnooze'),icon:'clock',run:()=>app.unsnoozeMessage(id)});
  rows.push({id:'followup',label:t(message.followUpAt?'mail-clear-flag':'mail-flag'),icon:'flag',run:()=>app.setFollowUp(id,message.followUpAt?null:new Date(Date.now()+86400000).toISOString())});
  for(const tag of app.tags())rows.push({id:`tag:${tag.id}`,label:`${t('mail-label')}: ${tag.name}`,icon:message.keywords?.[tag.id]?'check':'tag',run:()=>message.keywords?.[tag.id]?app.removeTag(id,tag.id):app.applyTag(id,tag.id)});
  rows.push({id:'download' ,label:t('mail-context-download'),icon:'download',run:()=>app.exportMessage(id)});
  return rows.map(row=>({...row,disabled:busy}));
 },restoreFocus:()=>Array.from(document.querySelectorAll<HTMLElement>('.list__row')).find(row=>row.dataset['emailId']===email()?.id)?.focus(),onError:()=>app.showToast('error',t('mail-context-failed'))});
 onCleanup(observePreferenceAccount(()=>menu.close(false)));
 return menu;
}
