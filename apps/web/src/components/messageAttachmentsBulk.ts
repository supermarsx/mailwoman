import {hasFileAttachments} from './attachmentMetadata.ts';
// Attachments of several messages at once: a multi-selection, or the messages of
// one conversation. The parts are not in the list rows, so they are fetched
// (a few at a time) and then handed to `downloadZip`, which makes one folder per
// message, leaves out what the attachment policy refuses and stops at its caps.

import type {Email} from '../api/jmap-types.ts';
import type {Client} from '../api/client.ts';
import {loadAttachmentPolicy,type AttachmentPolicy} from '../api/attachmentPolicy.ts';
import {t} from '../i18n';
import {fetchMessageExtras,type MessageExtras} from './messageRelations.ts';
import {describe,downloadZip,type Named} from './messageAttachmentDownload.ts';

export interface AttachmentGroup {emailId:string;label:string;items:Named[]}
type Extras=typeof fetchMessageExtras;
const POOL=4;

/** The attachments of each message that has any; `failed` counts messages whose parts could not be read. */
export async function collectAttachmentGroups(client:Pick<Client,'jmap'>,accountId:string,messages:Email[],sentIds:ReadonlySet<string>,fetchExtras:Extras=fetchMessageExtras):Promise<{groups:AttachmentGroup[];failed:number}> {
 const withFiles=messages.filter(hasFileAttachments);
 const found:(MessageExtras|'failed')[]=new Array(withFiles.length);
 let next=0;
 const worker=async():Promise<void>=>{
  for(;;){
   const i=next++;if(i>=withFiles.length)return;
   try{found[i]=await fetchExtras(client,accountId,withFiles[i]!,sentIds);}catch{found[i]='failed';}
  }
 };
 await Promise.all(Array.from({length:Math.min(POOL,withFiles.length)},worker));
 const groups:AttachmentGroup[]=[];let failed=0;
 withFiles.forEach((message,i)=>{
  const value=found[i]!;
  if(value==='failed'){failed++;return;}
  if(value.attachments.length)groups.push({emailId:message.id,label:message.subject??'',items:value.attachments.map(describe)});
 });
 return {groups,failed};
}

interface BulkApp {
 accountId:()=>string|null|undefined;downloadUrl:()=>string|null;
 showToast:(tone:'success'|'error'|'info',message:string)=>void;
 mailboxes:()=>{id:string;role?:string|null}[];
}

/** Download the attachments of `messages` as one ZIP with a folder per message. */
export async function downloadAttachmentsOf(app:BulkApp,client:Pick<Client,'jmap'>,messages:Email[],policy?:AttachmentPolicy,fetchExtras:Extras=fetchMessageExtras):Promise<void> {
 const account=app.accountId();
 if(!account)return;
 const sent=new Set(app.mailboxes().filter(box=>box.role==='sent').map(box=>box.id));
 app.showToast('info',t('mail-attachments-zip-gathering',{count:messages.filter(hasFileAttachments).length}));
 const {groups,failed}=await collectAttachmentGroups(client,account,messages,sent,fetchExtras);
 if(!groups.length){app.showToast('error',t(failed?'mail-attachment-download-failed':'mail-attachments-none'));return;}
 let rules=policy;
 if(!rules){try{rules=await loadAttachmentPolicy();}catch{rules=undefined;}}
 if(failed)app.showToast('info',t('mail-attachments-zip-unreadable',{count:failed}));
 await downloadZip(app,groups,rules);
}
