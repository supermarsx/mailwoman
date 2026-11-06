// Downloads from the reader's attachments bar. Everything goes through the same
// policy-checked `/jmap/download` helpers as the message context menu
// (messageAttachmentDownload.ts): this file only decides WHICH files a bar
// offers, never how they are fetched.

import {attachmentDecision,type AttachmentPolicy} from '../api/attachmentPolicy.ts';
import {t} from '../i18n';
import {blockedReason,downloadOne,downloadZip,type Named} from './messageAttachmentDownload.ts';
import type {StripItem} from '../viewers/ThumbnailStrip.tsx';

export type DownloadButtonMode='always'|'hover'|'never';
export const DOWNLOAD_BUTTON_MODES:readonly DownloadButtonMode[]=['always','hover','never'];
export const isDownloadButtonMode=(value:unknown):value is DownloadButtonMode=>typeof value==='string'&&(DOWNLOAD_BUTTON_MODES as readonly string[]).includes(value);

export interface BarApp {accountId:()=>string|null|undefined;downloadUrl:()=>string|null;showToast:(tone:'success'|'error'|'info',message:string)=>void}
export interface BarItem extends StripItem {inline?:boolean}

export const toNamed=(item:StripItem):Named=>({part:{blobId:item.blobId,type:item.mime,name:item.name,size:item.size} as Named['part'],name:item.name,mime:item.mime,size:item.size});

/** Why this one attachment cannot be downloaded (tooltip text), or undefined when it can. */
export function itemDownloadBlock(policy:AttachmentPolicy|undefined,item:StripItem,allowDownload:boolean):string|undefined {
 if(!allowDownload)return t('reader-attachments-download-off');
 return blockedReason(policy,toNamed(item));
}

/** What "Download all" would do: the files it saves, and how many it leaves out and why. */
export interface BarPlan {files:BarItem[];blocked:number;warned:number;inline:number}
export function planBarDownload(items:BarItem[],policy:AttachmentPolicy|undefined,includeInline:boolean):BarPlan {
 const files:BarItem[]=[];let blocked=0,warned=0,inline=0;
 for(const item of items){
  if(item.inline&&!includeInline){inline++;continue;}
  const decision=policy?attachmentDecision(policy,item):undefined;
  if(decision&&!decision.allowed){blocked++;continue;}
  if(decision?.warning){warned++;continue;}
  files.push(item);
 }
 return {files,blocked,warned,inline};
}

/** Save one attachment. A policy warning (executables and the like) asks first and then acknowledges. */
export async function downloadBarItem(app:BarApp,emailId:string,item:StripItem,policy:AttachmentPolicy|undefined,confirm:(message:string)=>boolean=message=>globalThis.confirm(message)):Promise<void> {
 const decision=policy?attachmentDecision(policy,item):undefined;
 if(decision&&!decision.allowed){app.showToast('error',t('reader-attachments-download-blocked',{name:item.name,reason:t('mail-attachment-policy-blocked')}));return;}
 if(decision?.warning&&!confirm(t('mail-attachment-policy-warning')))return;
 await downloadOne(app,emailId,toNamed(item),!!decision?.warning);
}

/** Save the bar's allowed attachments: one file directly, several as one ZIP (store-only writer, existing 200 file / 200 MB caps). Blocked files are counted in the summary toast; warned ones and inline images are left out. */
export async function downloadBarAll(app:BarApp,emailId:string,items:BarItem[],policy:AttachmentPolicy|undefined,includeInline:boolean):Promise<void> {
 const plan=planBarDownload(items,policy,includeInline);
 if(!plan.files.length){app.showToast('error',t(plan.blocked?'reader-attachments-download-all-blocked':'reader-attachments-download-all-none'));return;}
 if(plan.files.length===1&&!plan.blocked){await downloadOne(app,emailId,toNamed(plan.files[0]!));}
 else{
  // Blocked files are passed on so downloadZip's own summary counts them as skipped.
  const blocked=policy?items.filter(item=>!(item.inline&&!includeInline)&&!attachmentDecision(policy,item).allowed):[];
  await downloadZip(app,[{emailId,label:'',items:[...plan.files,...blocked].map(toNamed)}],policy);
 }
 if(plan.warned)app.showToast('info',t('reader-attachments-download-all-warned',{count:plan.warned}));
}
