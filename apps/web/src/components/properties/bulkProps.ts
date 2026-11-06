// What a selection of messages adds up to, for the bulk Properties dialog. Pure:
// it reads only what the list already holds (the loaded rows), so a selection
// never costs a request and the numbers are honest about being the loaded rows.
import type {Email} from '../../api/jmap-types.ts';

export interface BulkSender {email:string;name:string;count:number}
export interface BulkFolder {id:string;count:number}
export interface BulkSummary {
 count:number;
 totalBytes:number;
 /** Oldest and newest received time (ISO), null when no message has one. */
 oldest:string|null;newest:string|null;
 /** Distinct senders, most messages first. */
 senders:BulkSender[];
 folders:BulkFolder[];
 withAttachments:number;
 unread:number;
}

const received=(email:Pick<Email,'receivedAt'>):number|null=>{const time=Date.parse(email.receivedAt??'');return Number.isFinite(time)?time:null;};

export function summarizeSelection(emails:readonly Pick<Email,'size'|'receivedAt'|'from'|'mailboxIds'|'hasAttachment'|'keywords'>[]):BulkSummary {
 let oldest:number|null=null,newest:number|null=null,totalBytes=0,withAttachments=0,unread=0;
 const senders=new Map<string,BulkSender>(),folders=new Map<string,number>();
 for(const email of emails){
  const size=email.size??0;totalBytes+=Number.isFinite(size)&&size>0?size:0;
  const time=received(email);
  if(time!==null){oldest=oldest===null?time:Math.min(oldest,time);newest=newest===null?time:Math.max(newest,time);}
  const from=email.from?.[0];
  if(from){const key=from.email.toLowerCase(),known=senders.get(key);if(known)known.count+=1;else senders.set(key,{email:from.email,name:from.name??'',count:1});}
  for(const [id,on] of Object.entries(email.mailboxIds??{}))if(on)folders.set(id,(folders.get(id)??0)+1);
  if(email.hasAttachment)withAttachments+=1;
  if(email.keywords?.['$seen']!==true)unread+=1;
 }
 return {
  count:emails.length,totalBytes,
  oldest:oldest===null?null:new Date(oldest).toISOString(),newest:newest===null?null:new Date(newest).toISOString(),
  senders:[...senders.values()].sort((a,b)=>b.count-a.count||a.email.localeCompare(b.email)),
  folders:[...folders].map(([id,count])=>({id,count})).sort((a,b)=>b.count-a.count||a.id.localeCompare(b.id)),
  withAttachments,unread,
 };
}

/** Most messages a ZIP download of a selection will fetch. */
export const BULK_ZIP_LIMIT=200;
const FORBIDDEN='<>:"/'+String.fromCharCode(92)+'|?*';
/** A file name for one message in the ZIP: `0001 - subject.eml`, safe on every platform. */
export function zipEntryName(index:number,subject:string|null|undefined):string {
 const clean=Array.from(subject??'').map(char=>{const code=char.charCodeAt(0);return code<32||code===127||FORBIDDEN.includes(char)?' ':char;}).join('');
 const base=clean.replace(/\s+/g,' ').trim().slice(0,60)||'message';
 return `${String(index+1).padStart(4,'0')} - ${base}.eml`;
}
