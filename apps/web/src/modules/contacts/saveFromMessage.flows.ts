// The save flows behind the menu items: sender in one click, new contacts
// (one saved at once, several through the dialog), all recipients. No UI; the
// dialog opens through `deps.openDialog` so tests can stand in for it.
import type {AddressBook,ContactCard} from '../../api/pim-types.ts';
import type {Email} from '../../api/jmap-types.ts';
import {CAP_CORE,CAP_MAIL} from '../../api/jmap-types.ts';
import {responseFor} from '../../api/jmap.ts';
import {createConfiguredClient} from '../../api/transport.ts';
import type {AppState} from '../../state/store.ts';
import {t} from '../../i18n';
import {linkedMailboxEpoch} from '../../api/linkedMailboxSession.ts';
import {preferenceAccount,preferenceAccountIsCurrent} from '../../api/accountPreferenceLifecycle.ts';
import {formatDateWith} from '../../i18n/format.ts';
import {routeHash} from '../../shell/router.ts';
import {candidates,collectParticipants,draftFor,pickBook,saveSettings,senderParticipants,splitSaved,type Participant} from './saveFromMessage.ts';

export type SaveApp=Pick<AppState,'accountId'|'identities'|'contacts'|'addressBooks'|'contactGroups'|'lookupContactAddress'|'createContact'|'deleteContact'|'setGroupMembership'|'showToast'|'selectContact'|'loadContacts'>;
/** Keep an asynchronous menu action bound to the mailbox that opened it. */
export function contactOwner(app:Pick<SaveApp,'accountId'>):()=>boolean {
 const account=app.accountId(),authority=preferenceAccount(),epoch=linkedMailboxEpoch();
 return ()=>linkedMailboxEpoch()===epoch&&app.accountId()===account&&(authority?preferenceAccountIsCurrent(authority):preferenceAccount()===null);
}
const requireOwner=(current:()=>boolean):void=>{if(!current())throw new DOMException('Mailbox changed','AbortError');};

export interface DialogRequest {app:SaveApp;messages:readonly Email[];deps:Deps}
export interface Deps {
 /** Reads Reply-To / Bcc / Sender of the messages that do not carry them yet. */
 fetchFields?:(app:SaveApp,messages:readonly Email[])=>Promise<Email[]>;
 openDialog?:(request:DialogRequest)=>void;
 openContact?:(app:SaveApp,id:string)=>void;
}

export const ownAddresses=(app:Pick<SaveApp,'identities'>):string[]=>(app.identities?.()??[]).map(identity=>identity.email);

/** Open a saved contact in the Contacts screen. */
export function openContactById(app:Pick<SaveApp,'selectContact'>,id:string):void {
 app.selectContact(id);
 if(typeof window!=='undefined')window.location.hash=routeHash('contacts',id);
}

/** Adds the header fields a list row does not carry (Reply-To, Bcc, Sender) with one Email/get. */
export async function fetchMissingFields(app:Pick<SaveApp,'accountId'>,messages:readonly Email[]):Promise<Email[]> {
 const current=contactOwner(app);
 const missing=messages.filter(message=>message.replyTo===undefined);
 const account=app.accountId();
 if(!missing.length||!account)return [...messages];
 const response=await createConfiguredClient().jmap({using:[CAP_CORE,CAP_MAIL],methodCalls:[['Email/get',{accountId:account,ids:missing.map(message=>message.id),properties:['id','replyTo','bcc','sender']},'g']]});
 requireOwner(current);
 const list=responseFor<{list:(Pick<Email,'id'|'replyTo'|'bcc'>&{sender?:Email['from']})[]}>(response,'g').list;
 const byId=new Map(list.map(row=>[row.id,row]));
 return messages.map(message=>{const extra=byId.get(message.id);return extra?{...message,replyTo:extra.replyTo??null,bcc:extra.bcc??null,sender:extra.sender??null} as Email:message;});
}

export interface Resolved {fresh:Participant[];saved:{participant:Participant;card:ContactCard}[];books:AddressBook[]}
/** Looks each participant up across every address book (by e-mail), splitting new from already saved. */
export async function resolveParticipants(app:SaveApp,items:readonly Participant[]):Promise<Resolved> {
 const current=contactOwner(app);
 const results=await Promise.all(items.map(item=>app.lookupContactAddress(item.email).catch(()=>null)));
 requireOwner(current);
 const books=results.find(result=>result&&result.addressBooks.length>0)?.addressBooks??app.addressBooks();
 const found=results.flatMap(result=>result?.contact?[result.contact]:[]);
 const {fresh,saved}=splitSaved(items,[...found,...app.contacts()]);
 return {fresh,saved,books};
}

export interface SaveItem {email:string;name:string;bookId:string;groupId?:string}
/** Creates the cards through the normal create path; returns the new ids (failures are left out). */
export async function saveItems(app:SaveApp,items:readonly SaveItem[]):Promise<{ids:string[];names:string[]}> {
 const current=contactOwner(app);
 const note=saveSettings.addNote()?t('contacts-save-note',{date:formatDateWith(new Date(),{dateStyle:'medium'})}):null;
 const ids:string[]=[],names:string[]=[];
 for(const item of items){
  requireOwner(current);
  const draft=draftFor(item,note);
  const id=await app.createContact(draft,item.bookId).catch(()=>null);
  requireOwner(current);
  if(!id)continue;
  ids.push(id);names.push(draft.name!.full);
  if(item.groupId)await app.setGroupMembership(id,item.groupId,true).catch(()=>undefined);
 }
 return {ids,names};
}


/** The toast after a save: one name or a count, with Undo. */
export function announceSaved(app:SaveApp,done:{ids:string[];names:string[]},total:number,book:string):void {
 if(!done.ids.length){app.showToast('error',t('contacts-save-failed'));return;}
 const message=done.ids.length===1?t('contacts-save-saved-one',{name:done.names[0]!,book}):t('contacts-save-saved-many',{count:done.ids.length,book});
 const ids=[...done.ids],current=contactOwner(app);
 app.showToast(done.ids.length<total?'info':'success',done.ids.length<total?t('contacts-save-partial',{saved:done.ids.length,total}):message,8000,{action:{label:t('contacts-save-undo'),run:async()=>{if(!current())return;await Promise.all(ids.map(id=>app.deleteContact(id)));if(!current())return;app.showToast('success',t('contacts-save-removed'));}}});
}

function announceAlready(app:SaveApp,resolved:Resolved,deps:Deps):void {
 const first=resolved.saved[0];
 if(!first){app.showToast('info',t('contacts-save-nothing'));return;}
 const open=deps.openContact??openContactById,current=contactOwner(app);
 app.showToast('info',resolved.saved.length===1?t('contacts-save-already',{name:first.card.name.full||first.participant.email}):t('contacts-save-already-many',{count:resolved.saved.length}),6000,{action:{label:t('contacts-save-open'),run:()=>{if(current())open(app,first.card.id);}}});
}

async function saveFresh(app:SaveApp,resolved:Resolved,deps:Deps):Promise<void> {
 const current=contactOwner(app);
 const book=pickBook(resolved.books,saveSettings.defaultBook());
 if(!resolved.fresh.length){announceAlready(app,resolved,deps);return;}
 if(!book){app.showToast('error',t('contacts-save-no-book'));return;}
 const done=await saveItems(app,resolved.fresh.map(item=>({email:item.email,name:item.name,bookId:book.id})));
 if(current())announceSaved(app,done,resolved.fresh.length,book.name);
}

/** One click: the sender of each message (distinct), into the default book. */
export async function saveSender(app:SaveApp,messages:readonly Email[],deps:Deps={}):Promise<void> {
 const current=contactOwner(app);
 try{
  const senders=senderParticipants(messages,ownAddresses(app),saveSettings.useReplyTo()).filter(item=>!item.own);
  await saveFresh(app,await resolveParticipants(app,candidates(senders,saveSettings.skipAutomated())),deps);
 }catch{if(current())app.showToast('error',t('contacts-save-failed'));}
}

/** Everyone the message went to (To and Cc), minus the user and, by setting, automated addresses. */
export async function saveRecipients(app:SaveApp,messages:readonly Email[],deps:Deps={}):Promise<void> {
 const current=contactOwner(app);
 try{
  const all=collectParticipants(messages,ownAddresses(app),['to','cc']);
  await saveFresh(app,await resolveParticipants(app,candidates(all,saveSettings.skipAutomated())),deps);
 }catch{if(current())app.showToast('error',t('contacts-save-failed'));}
}

/** Exactly one new contact is saved at once; several open the dialog. */
export async function saveNew(app:SaveApp,messages:readonly Email[],deps:Deps={}):Promise<void> {
 const current=contactOwner(app);
 try{
  const all=collectParticipants(messages,ownAddresses(app));
  const resolved=await resolveParticipants(app,candidates(all,saveSettings.skipAutomated()));
  if(resolved.fresh.length>1)(deps.openDialog??(()=>undefined))({app,messages,deps});
  else await saveFresh(app,resolved,deps);
 }catch{if(current())app.showToast('error',t('contacts-save-failed'));}
}

export function chooseContacts(app:SaveApp,messages:readonly Email[],deps:Deps={}):void {
 (deps.openDialog??(()=>undefined))({app,messages,deps});
}
