import {apiFetch} from '../api/http.ts';
import {attachmentDecision,attachmentPolicyUrl,type AttachmentPolicy} from '../api/attachmentPolicy.ts';
import {buildDownloadUrl,type AttachmentPart} from '../viewers/attachments.ts';
import {buildZip} from '../viewers/zipStore.ts';
import {formatBytes} from '../i18n/format.ts';
import {t} from '../i18n';
export const ZIP_MAX_FILES=200;
export const ZIP_MAX_BYTES=200*1024*1024;
export interface Named {part:AttachmentPart;name:string;mime:string;size:number}
export const describe=(part:AttachmentPart):Named=>({part,name:part.name?part.name:t('mail-attachment-unnamed'),mime:part.type||'application/octet-stream',size:part.size??0});
/** Why the attachment policy refuses this file (a tooltip), or undefined when it may be saved. */
export function blockedReason(policy:AttachmentPolicy|undefined,item:Named):string|undefined {
 if(!policy)return undefined;
 return attachmentDecision(policy,item).allowed?undefined:t('mail-attachment-policy-blocked');
}
export const sizeLabel=(item:Named):string=>formatBytes(item.size);
interface DownloadApp {accountId:()=>string|null|undefined;downloadUrl:()=>string|null;showToast:(tone:'success'|'error'|'info',message:string)=>void}
// Control characters are what is being replaced here.
// eslint-disable-next-line no-control-regex
const safe=(name:string):string=>name.replace(/[/:*?"<>|\u0000-\u001f]+/g,'_').slice(0,120)||'attachment';
function save(blob:Blob,name:string):void {
 const url=URL.createObjectURL(blob),a=document.createElement('a');a.href=url;a.download=name;a.rel='noopener';document.body.appendChild(a);a.click();a.remove();setTimeout(()=>URL.revokeObjectURL(url),10_000);
}
async function bytes(app:DownloadApp,emailId:string,item:Named,signal?:AbortSignal):Promise<Blob> {
 const template=app.downloadUrl(),account=app.accountId();
 if(!template||!account||!item.part.blobId)throw new Error('no download');
 const url=attachmentPolicyUrl(buildDownloadUrl(template,{accountId:account,blobId:item.part.blobId,name:item.name,mime:item.mime}),emailId,false,false,true);
 const response=await apiFetch(url,{credentials:'same-origin',...(signal?{signal}:{})});
 if(!response.ok)throw new Error(`download failed: ${response.status}`);
 return response.blob();
}
/** Save one attachment through the policy-checked `/jmap/download` URL (`download=true`, the server refuses it when mail.allowDownload is off). */
export async function downloadOne(app:DownloadApp,emailId:string,item:Named):Promise<void> {
 try{save(await bytes(app,emailId,item),safe(item.name));app.showToast('success',t('mail-attachment-downloaded',{name:item.name}));}
 catch{app.showToast('error',t('mail-attachment-download-failed'));}
}
/** One folder name per message: unsafe characters replaced, and a repeated subject numbered so two messages never share a folder. */
export function folderNames(labels:string[]):string[] {
 const seen=new Map<string,number>();
 return labels.map(label=>{const base=safe(label.trim()||t('mail-no-subject'));const count=(seen.get(base)??0)+1;seen.set(base,count);return count===1?base:`${base} (${count})`;});
}
/** Stream the allowed attachments of several messages into one ZIP, one folder per message when there is more than one.
 * Files the attachment policy refuses are left out and counted in the closing toast; when nothing is allowed nothing is downloaded. */
export async function downloadZip(app:DownloadApp,groups:{emailId:string;label:string;items:Named[]}[],policy?:AttachmentPolicy):Promise<void> {
 const skipped=groups.reduce((sum,group)=>sum+group.items.filter(item=>blockedReason(policy,item)).length,0);
 const allowed=groups.map(group=>({...group,items:group.items.filter(item=>!blockedReason(policy,item))})).filter(group=>group.items.length);
 const folders=folderNames(allowed.map(group=>group.label));
 const picked=allowed.map((group,i)=>({...group,label:folders[i]!}));
 if(!picked.length){app.showToast('error',t(skipped?'mail-attachments-zip-all-blocked':'mail-attachment-download-failed',{skipped}));return;}
 let files=0,total=0,capped=false;const entries:{name:string;data:Uint8Array}[]=[];
 try{
  outer:for(const group of picked){
   const seen=new Set<string>();
   for(const item of group.items){
    if(files>=ZIP_MAX_FILES||total+item.size>ZIP_MAX_BYTES){capped=true;break outer;}
    const blob=await bytes(app,group.emailId,item);
    let name=safe(item.name);for(let n=2;seen.has(name);n++)name=safe(item.name).replace(/(\.[^.]*)?$/,` (${n})$1`);seen.add(name);
    total+=blob.size;files++;
    entries.push({name:picked.length>1?`${group.label}/${name}`:name,data:new Uint8Array(await blob.arrayBuffer())});
   }
  }
  if(!entries.length){app.showToast('error',t('mail-attachment-download-failed'));return;}
  save(buildZip(entries),'attachments.zip');
  const summary=t(capped?'mail-attachments-zip-capped':'mail-attachments-zip-done',{count:files,max:ZIP_MAX_FILES,size:formatBytes(ZIP_MAX_BYTES)});
  app.showToast(capped||skipped?'info':'success',skipped?`${summary} ${t('mail-attachments-zip-skipped',{skipped})}`:summary);
 }catch{app.showToast('error',t('mail-attachment-download-failed'));}
}
