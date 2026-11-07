// Download a folder as a ZIP (`GET /api/export/folder/<id>.zip`, crates/mw-server/src/folder_export.rs).
// The estimate comes first: the server says how many messages and bytes the
// archive holds and whether the administrator's caps allow it. A small archive
// is read here, with progress and a Cancel; a big one is handed to the browser
// as a plain download so it streams to disk instead of into memory.
import {apiFetch} from '../../api/http.ts';
import {withBase} from '../../api/basePath.ts';
import {activeLinkedMailbox} from '../../api/linkedMailboxSession.ts';
import {linkedResourceUrl} from '../../api/linkedAccountRouting.ts';
import {transportBase} from '../../api/transport.ts';
import type {ExportFormat} from './policy.ts';

export interface ExportEstimate {folders:number;messages:number;bytes:number;maxBytes:number;maxMessages:number;format:ExportFormat;formats:ExportFormat[];subfolders:boolean;allowed:boolean;reason?:string}
export interface ExportOptions {format:ExportFormat;subfolders:boolean}
/** Archives above this are downloaded by the browser itself (no progress, no memory use). */
export const IN_PAGE_LIMIT_BYTES=200*1024*1024;

export class ExportRefused extends Error {constructor(readonly code:string,readonly status:number,readonly detail?:Partial<ExportEstimate>){super(code);}}

export function exportUrl(mailboxId:string,options:ExportOptions,estimate=false):string {
 const params=new URLSearchParams({format:options.format,subfolders:options.subfolders?'1':'0'});
 if(estimate)params.set('estimate','1');
 const url=withBase(`/api/export/folder/${encodeURIComponent(mailboxId)}.zip?${params}`);
 // A browser-level download (no apiFetch) carries the link selector in the URL, never the primary's.
 const linked=activeLinkedMailbox();
 return linked?linkedResourceUrl(url,linked.linkId,new URL(`${transportBase()}/`,globalThis.location.href).href):url;
}

async function refusal(response:Response):Promise<ExportRefused> {
 let body:Partial<ExportEstimate>&{error?:string}={};
 try{body=await response.json() as typeof body;}catch{/* an empty body keeps the status */}
 return new ExportRefused(body.error??`http-${response.status}`,response.status,body);
}

export async function fetchExportEstimate(mailboxId:string,options:ExportOptions,signal?:AbortSignal):Promise<ExportEstimate> {
 const response=await apiFetch(exportUrl(mailboxId,options,true),signal?{signal}:{});
 if(!response.ok)throw await refusal(response);
 return await response.json() as ExportEstimate;
}

/** Read the archive in the page, reporting bytes received; `signal` cancels it. */
export async function downloadFolderZip(mailboxId:string,options:ExportOptions,onProgress:(received:number)=>void,signal:AbortSignal):Promise<Blob> {
 const response=await apiFetch(exportUrl(mailboxId,options),{signal});
 if(!response.ok||!response.body)throw await refusal(response);
 const reader=response.body.getReader(),chunks:Uint8Array<ArrayBuffer>[]=[];let received=0;
 for(;;){
  const {done,value}=await reader.read();
  if(value){chunks.push(value as Uint8Array<ArrayBuffer>);received+=value.length;onProgress(received);}
  if(done)break;
 }
 return new Blob(chunks,{type:'application/zip'});
}

/** Hand a blob to the browser as a file. */
export function saveBlob(blob:Blob,name:string):void {
 const url=URL.createObjectURL(blob),link=document.createElement('a');
 link.href=url;link.download=name;document.body.appendChild(link);link.click();link.remove();
 setTimeout(()=>URL.revokeObjectURL(url),10_000);
}
