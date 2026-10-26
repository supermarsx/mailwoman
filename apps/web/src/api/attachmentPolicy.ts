import { apiFetch } from './http.ts';
import { transportBase } from './transport.ts';

export interface AttachmentPolicy {
  previewAllowed:boolean; downloadAllowed:boolean; maxBytes:number;
  allowedExtensions:string[]; blockedExtensions:string[]; warnExtensions:string[];
  allowedMimeTypes:string[]; blockedMimeTypes:string[]; warnMimeTypes:string[];
}
export async function loadAttachmentPolicy():Promise<AttachmentPolicy> {
  const response=await apiFetch(`${transportBase()}/api/attachments/policy`,{cache:'no-store'});
  if(!response.ok)throw new Error('attachmentPolicyUnavailable');
  const value:unknown=await response.json();
  if(!value || typeof value!=='object')throw new Error('attachmentPolicyUnavailable');
  const row=value as Record<string,unknown>;
  if(typeof row['previewAllowed']!=='boolean'||typeof row['downloadAllowed']!=='boolean'||typeof row['maxBytes']!=='number'||row['maxBytes']<0)throw new Error('attachmentPolicyUnavailable');
  for(const key of ['allowedExtensions','blockedExtensions','warnExtensions','allowedMimeTypes','blockedMimeTypes','warnMimeTypes'])if(!Array.isArray(row[key])||!(row[key] as unknown[]).every(item=>typeof item==='string'))throw new Error('attachmentPolicyUnavailable');
  return value as AttachmentPolicy;
}
export function attachmentDecision(policy:AttachmentPolicy,item:{name:string;mime:string;size:number}):{allowed:boolean;preview:boolean;warning:boolean} {
  const name=(item.name.split(/[\\/]/).pop()??'').replace(/[ .]+$/g,'').toLowerCase();
  const mime=item.mime.split(';')[0]!.trim().toLowerCase();
  const ext=(rules:string[]):boolean=>rules.some(rule=>name.endsWith(`.${rule}`));
  const type=(rules:string[]):boolean=>rules.some(rule=>rule===mime||(rule.endsWith('/*')&&mime.startsWith(rule.slice(0,-1))));
  const allowed=policy.downloadAllowed && (!policy.maxBytes||item.size<=policy.maxBytes) && !ext(policy.blockedExtensions)&&!type(policy.blockedMimeTypes)&&(!policy.allowedExtensions.length||ext(policy.allowedExtensions))&&(!policy.allowedMimeTypes.length||type(policy.allowedMimeTypes));
  return {allowed,preview:allowed&&policy.previewAllowed,warning:allowed&&(ext(policy.warnExtensions)||type(policy.warnMimeTypes))};
}
export function attachmentPolicyUrl(url:string,emailId?:string,preview=false,acknowledge=false):string {
  const result=new URL(url,globalThis.location?.href??'http://localhost');
  if(emailId)result.searchParams.set('emailId',emailId);
  if(preview)result.searchParams.set('preview','true');
  if(acknowledge)result.searchParams.set('acknowledgeWarning','true');
  return result.href;
}
