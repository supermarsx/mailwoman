// The two menu entries, shared by the message list menu, the thread-row menu
// and the reader More menu: Save sender as contact (one click) and Save
// contacts from this message (a submenu). Hidden when the feature is off.
import type {Email} from '../../api/jmap-types.ts';
import type {ContextAction} from '../../components/ContextMenu.tsx';
import {ACTION_ICONS} from '../../actionIcons.ts';
import {loadCatalog,t} from '../../i18n';
import {addressKey,saveSettings,senderParticipants,collectParticipants,candidates} from './saveFromMessage.ts';
import {chooseContacts,openContactById,ownAddresses,saveNew,saveRecipients,saveSender,type Deps,type SaveApp} from './saveFromMessage.flows.ts';

void loadCatalog('contacts');

/** Why Save sender is unavailable for these messages, or undefined when it works. */
function senderBlock(app:SaveApp,messages:readonly Email[]):string|undefined {
 const senders=senderParticipants(messages,ownAddresses(app),saveSettings.useReplyTo());
 if(!senders.length)return t('contacts-save-no-sender');
 if(senders.every(item=>item.own))return t('contacts-save-own');
 const usable=senders.filter(item=>!item.own);
 if(saveSettings.skipAutomated()&&usable.every(item=>item.automated))return t('contacts-save-automated');
 return undefined;
}

/**
 * Menu rows for one message, or for every selected message. `deps.openDialog` draws the chooser;
 * the menus pass the real one, tests a stand-in.
 */
export function saveContactRows(app:SaveApp,messages:readonly Email[],deps:Deps):ContextAction[] {
 const rows:ContextAction[]=[];
 if(!messages.length)return rows;
 if(saveSettings.senderEnabled()){
  const why=senderBlock(app,messages);
  const senders=senderParticipants(messages,ownAddresses(app),saveSettings.useReplyTo()).filter(item=>!item.own);
  // Already a contact (in the cards loaded so far): the item opens it instead.
  const known=senders.length===1?(app.contacts?.()??[]).find(card=>card.emails.some(entry=>addressKey(entry.value)===senders[0]!.email)):undefined;
  if(known)rows.push({id:'save-sender',label:t('contacts-save-open-contact'),icon:ACTION_ICONS.openContact,run:()=>(deps.openContact??openContactById)(app,known.id)});
  else rows.push({id:'save-sender',label:t('contacts-save-sender'),icon:ACTION_ICONS.saveSender,...(why?{disabled:true,title:why}:{}),run:()=>saveSender(app,messages,deps)});
 }
 if(saveSettings.fromMessageEnabled()){
  const any=candidates(collectParticipants(messages,ownAddresses(app)),saveSettings.skipAutomated()).length>0;
  rows.push({id:'save-contacts',label:t('contacts-save-contacts'),icon:ACTION_ICONS.saveContacts,...(any?{}:{disabled:true,title:t('contacts-save-nothing')}),run:()=>{},children:[
   {id:'save-contacts:new',label:t('contacts-save-new'),icon:ACTION_ICONS.saveContacts,run:()=>saveNew(app,messages,deps)},
   {id:'save-contacts:sender',label:t('contacts-save-sender'),icon:ACTION_ICONS.saveSender,...(senderBlock(app,messages)?{disabled:true,title:senderBlock(app,messages)!}:{}),run:()=>saveSender(app,messages,deps)},
   {id:'save-contacts:recipients',label:t('contacts-save-recipients'),icon:ACTION_ICONS.saveContacts,run:()=>saveRecipients(app,messages,deps)},
   {id:'save-contacts:choose',label:t('contacts-save-choose'),icon:ACTION_ICONS.saveContacts,run:()=>chooseContacts(app,messages,deps)}]});
 }
 return rows;
}

/** The real dependencies: the dialog is loaded when first opened. */
export const liveDeps:Deps={openDialog:request=>void import('./SaveContactsDialog.tsx').then(module=>module.openSaveContactsDialog(request))};

/** What the menus call: rows with the real dialog. */
export const saveRowsFor=(app:SaveApp,messages:readonly Email[]):ContextAction[]=>saveContactRows(app,messages,liveDeps);
