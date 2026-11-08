// Save contacts from a message: the pure part (who is on a message, who is
// worth saving, what the new card looks like) and the settings that steer it.
// No UI here; saveFromMessageActions.tsx draws the menu rows and the dialog.
import type {AddressBook,ContactCard} from '../../api/pim-types.ts';
import type {Email,EmailAddress} from '../../api/jmap-types.ts';
import {settings} from '../../api/settings.ts';
import {featureOn} from '../../state/features.ts';
import {normalizeContactAddress} from './contactAddress.ts';
import type {ContactDraft} from '../../state/slices/contacts.ts';

export type Role='from'|'replyTo'|'to'|'cc'|'bcc'|'sender';
export type Scope='sender'|'sender+replyto'|'all';
export const ROLE_ORDER:readonly Role[]=['from','replyTo','to','cc','bcc','sender'];

export interface Participant {email:string;name:string;roles:Role[];own:boolean;automated:boolean}

const readSetting=<T>(key:string,fallback:T,ok:(value:unknown)=>value is T):T=>{const value=settings()[key]?.value;return ok(value)?value:fallback;};
const isBool=(value:unknown):value is boolean=>typeof value==='boolean';
const isScope=(value:unknown):value is Scope=>value==='sender'||value==='sender+replyto'||value==='all';
const isString=(value:unknown):value is string=>typeof value==='string';

/** Setting readers. Admin locks and defaults are already folded into `settings()`. */
export const saveSettings={
 /** Both the Contacts feature switch and the key must allow it. */
 fromMessageEnabled:():boolean=>featureOn('contacts')&&readSetting('contacts.saveFromMessage.enabled',true,isBool),
 senderEnabled:():boolean=>featureOn('contacts')&&readSetting('contacts.saveSender.enabled',true,isBool),
 defaultScope:():Scope=>readSetting<Scope>('contacts.saveFromMessage.defaultScope','sender',isScope),
 skipAutomated:():boolean=>readSetting('contacts.saveFromMessage.skipAutomated',true,isBool),
 addNote:():boolean=>readSetting('contacts.saveFromMessage.addNote',true,isBool),
 defaultBook:():string=>readSetting('contacts.saveFromMessage.defaultBook','default',isString),
 useReplyTo:():boolean=>readSetting('contacts.saveSender.useReplyTo',false,isBool),
};

const AUTOMATED=/^(no[-_.]?reply|do[-_.]?not[-_.]?reply|donotreply|mailer[-_.]?daemon|postmaster|notifications?|notify|bounces?|bounce[-+_.=].*|no[-_.]?reply[-+_.].*|alerts?|automated?)$/i;
/** Obvious no-reply / automated sender addresses (by local part). */
export function isAutomatedAddress(email:string):boolean {
 const local=email.split('@')[0]??'';
 return AUTOMATED.test(local);
}

/** The dedupe key and stored form: the valid address, lower-cased. */
export const addressKey=(value:string):string|null=>normalizeContactAddress(value)?.toLowerCase()??null;
const clean=(value:string|null|undefined):string=>(value??'').replace(/^["'\s]+|["'\s]+$/g,'').replace(/\s+/g,' ');

/** The name to store: the display-name part when there is one, else the local part with capital letters ("jane.doe" becomes "Jane Doe"). */
export function displayNameFor(name:string|null|undefined,email:string):string {
 const given=clean(name);
 if(given&&given.toLowerCase()!==email.toLowerCase()&&!given.includes('@'))return given;
 const words=(email.split('@')[0]??'').split('+')[0]!.split(/[._-]+/).filter(word=>word&&!/^\d+$/.test(word));
 if(!words.length)return email;
 return words.map(word=>word.charAt(0).toLocaleUpperCase()+word.slice(1).toLocaleLowerCase()).join(' ');
}

type Fields={from?:EmailAddress[]|null;replyTo?:EmailAddress[]|null;to?:EmailAddress[]|null;cc?:EmailAddress[]|null;bcc?:EmailAddress[]|null;sender?:EmailAddress[]|null};

/**
 * Every distinct participant (by lowercase address) of the given messages, in role order.
 * `own` marks the user's identities; `automated` marks no-reply style senders.
 */
export function collectParticipants(messages:readonly (Pick<Email,'id'>&Fields)[],ownAddresses:Iterable<string>,roles:readonly Role[]=ROLE_ORDER.filter(role=>role!=='sender'&&role!=='bcc')):Participant[] {
 const own=new Set([...ownAddresses].map(value=>value.trim().toLowerCase()));
 const found=new Map<string,Participant>();
 for(const role of ROLE_ORDER){
  if(!roles.includes(role))continue;
  for(const message of messages)for(const address of message[role]??[]){
   const email=addressKey(address.email);
   if(!email)continue;
   const existing=found.get(email);
   if(existing){
    if(!existing.roles.includes(role))existing.roles.push(role);
    if(!existing.name&&clean(address.name))existing.name=clean(address.name);
   }else found.set(email,{email,name:clean(address.name),roles:[role],own:own.has(email),automated:isAutomatedAddress(email)});
  }
 }
 return [...found.values()];
}

/** Participants worth offering: never the user's own, automated ones only when `skipAutomated` is off. */
export function candidates(all:readonly Participant[],skipAutomated:boolean):Participant[] {
 return all.filter(item=>!item.own&&!(skipAutomated&&item.automated));
}

/** The sender to save with one click: From, or Reply-To for a list/automated sender when the setting asks. */
export function senderParticipants(messages:readonly (Pick<Email,'id'>&Fields)[],ownAddresses:Iterable<string>,useReplyTo:boolean):Participant[] {
 const picked:Participant[]=[],seen=new Set<string>();
 for(const message of messages){
  const [from]=collectParticipants([message],ownAddresses,['from']);
  if(!from)continue;
  let chosen=from;
  if(useReplyTo&&from.automated){const [reply]=collectParticipants([message],ownAddresses,['replyTo']);if(reply&&!reply.own)chosen={...reply,roles:['from']};}
  if(!seen.has(chosen.email)){seen.add(chosen.email);picked.push(chosen);}
 }
 return picked;
}

/** Whether a participant starts ticked, for the chosen scope. */
export function defaultChecked(item:Participant,scope:Scope):boolean {
 if(scope==='all')return true;
 if(scope==='sender+replyto')return item.roles.includes('from')||item.roles.includes('replyTo');
 return item.roles.includes('from');
}

export interface Existing {card:ContactCard}
/** Splits participants into new ones and those already in a contact (matched by e-mail over all books). */
export function splitSaved(items:readonly Participant[],cards:readonly ContactCard[]):{fresh:Participant[];saved:{participant:Participant;card:ContactCard}[]} {
 const byEmail=new Map<string,ContactCard>();
 for(const card of cards)for(const entry of card.emails){const key=addressKey(entry.value);if(key&&!byEmail.has(key))byEmail.set(key,card);}
 const fresh:Participant[]=[],saved:{participant:Participant;card:ContactCard}[]=[];
 for(const item of items){const card=byEmail.get(item.email);if(card)saved.push({participant:item,card});else fresh.push(item);}
 return {fresh,saved};
}

/** A book can take a new card unless it says otherwise (`readOnly` / `myRights.mayWrite`, when the engine reports them). */
export function bookWritable(book:AddressBook):boolean {
 const extra=book as AddressBook&{readOnly?:boolean;isReadOnly?:boolean;myRights?:{mayWrite?:boolean}};
 return extra.readOnly!==true&&extra.isReadOnly!==true&&extra.myRights?.mayWrite!==false;
}

/** The book the settings name, else the default book, else the first writable one. */
export function pickBook(books:readonly AddressBook[],preferred:string):AddressBook|undefined {
 const writable=books.filter(bookWritable);
 return writable.find(book=>book.id===preferred)??writable.find(book=>book.isDefault)??writable[0];
}


export function draftFor(item:{email:string;name:string},note:string|null):ContactDraft {
 return {name:{full:displayNameFor(item.name,item.email),given:'',surname:'',prefix:'',suffix:''},emails:[{value:item.email,context:'',pref:1}],...(note?{notes:note}:{})};
}
