import {apiFetch} from './http.ts';
import {withBase} from './basePath.ts';
import type {Email} from './jmap-types.ts';
import type {ComposeInitial} from '../components/compose/reply.ts';

export const BULK_MESSAGE_LIMIT=100;
export const BULK_BYTE_LIMIT=32*1024*1024;
export class BulkMailError extends Error {}

/** Original messages remain MIME attachments: no HTML interpolation/decryption. */
export function forwardMessages(messages:readonly Email[],subject:string):ComposeInitial {
  if(!messages.length||messages.length>BULK_MESSAGE_LIMIT||messages.some(message=>!message.blobId))throw new BulkMailError('selection');
  if(messages.reduce((total,message)=>total+(message.size??0),0)>BULK_BYTE_LIMIT)throw new BulkMailError('size');
  return {mode:'forward',to:'',cc:'',subject,bodyHtml:'',bodyText:'',inReplyTo:[],references:[],quotesDecrypted:false,
    attachments:messages.map((message,index)=>({blobId:message.blobId!,name:`${index+1}-${(message.subject??'message').replace(/[^\p{L}\p{N}._-]+/gu,'_').slice(0,80)||'message'}.eml`,size:message.size??0,contentType:'message/rfc822'})),
    sources:messages.map(message=>({emailId:message.id,keyword:'$forwarded'}))};
}

/** One bounded mbox download; server export policy/step-up applies to every row. */
export async function downloadMessages(ids:readonly string[],current:()=>boolean,signal:AbortSignal):Promise<Blob> {
  const unique=[...new Set(ids)];
  if(!unique.length||unique.length>BULK_MESSAGE_LIMIT)throw new BulkMailError('selection');
  const parts:Uint8Array<ArrayBuffer>[]=[];let bytes=0;
  for(const id of unique){
    if(!current())throw new BulkMailError('account');
    const response=await apiFetch(withBase(`/api/export/${encodeURIComponent(id)}?format=mbox`),{credentials:'same-origin',signal});
    if(!response.ok||!response.body)throw new BulkMailError('export');
    const reader=response.body.getReader();
    try {
      for(;;){const {done,value}=await reader.read();if(done)break;if(!current())throw new BulkMailError('account');bytes+=value.length;if(bytes>BULK_BYTE_LIMIT)throw new BulkMailError('size');parts.push(new Uint8Array(value));}
    }finally{await reader.cancel().catch(()=>undefined);reader.releaseLock();}
  }
  if(!current())throw new BulkMailError('account');
  return new Blob(parts,{type:'application/mbox'});
}
