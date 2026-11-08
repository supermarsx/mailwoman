// Runs one export: fetch each message through `apiFetch` (so the step-up dialog,
// the linked-mailbox routing and the session handling all apply), apply the
// header privacy options, and package the result. A folder goes through the
// existing folder ZIP route. `signal` cancels between and during requests.
import {linkedMailboxEpoch} from '../../api/linkedMailboxSession.ts';
import {preferenceAccount,preferenceAccountIsCurrent} from '../../api/accountPreferenceLifecycle.ts';
import {apiFetch} from '../../api/http.ts';
import {withBase} from '../../api/basePath.ts';
import type {Email} from '../../api/jmap-types.ts';
import {buildZip} from '../../viewers/zipStore.ts';
import {downloadFolderZip,ExportRefused} from '../properties/folderExport.ts';
import {buildCsv,buildJson,EXT,fileNames,isServerFormat,MIME,sanitizeName,stripHeaders,type ExportFormatId,type ExportScope,type Packaging,type PrivacyOptions} from './exportPlan.ts';

export class ExportError extends Error {
  constructor(readonly code:string,readonly status:number){super(code);}
}
export interface RunInput {
  scope:ExportScope;format:ExportFormatId;packaging:Packaging;emails:readonly Email[];
  folder?:{id:string;name:string};subfolders:boolean;pattern:string;maxName:number;privacy:PrivacyOptions;
  /** Name for a single file that holds many messages. */
  label:string;
}
export interface RunResult {blob:Blob;name:string}
export interface Progress {done:number;total:number;bytes:number}

async function failure(response:Response):Promise<ExportError> {
  let code=`http-${response.status}`;
  try{const body=await response.json() as {error?:string};if(typeof body.error==='string')code=body.error;}catch{/* the status stays */}
  return new ExportError(code,response.status);
}

async function fetchOne(id:string,format:string,signal:AbortSignal):Promise<Uint8Array> {
  const response=await apiFetch(withBase(`/api/export/${encodeURIComponent(id)}?format=${format}`),{credentials:'same-origin',signal});
  if(!response.ok)throw await failure(response);
  return new Uint8Array(await response.arrayBuffer());
}

const encode=(text:string)=>new TextEncoder().encode(text);
const withNewline=(bytes:Uint8Array)=>bytes.length>0&&bytes[bytes.length-1]!==10?Uint8Array.from([...bytes,10]):bytes;

export async function runExport(input:RunInput,onProgress:(progress:Progress)=>void,signal:AbortSignal):Promise<RunResult> {
  const {format}=input;
  const epoch=linkedMailboxEpoch(),owner=preferenceAccount();
  const requireCurrent=()=>{if(signal.aborted||linkedMailboxEpoch()!==epoch||(owner?!preferenceAccountIsCurrent(owner):preferenceAccount()!==null))throw new DOMException('Export cancelled','AbortError');};
  requireCurrent();
  if(input.scope==='folder'){
    if(!input.folder||(format!=='eml'&&format!=='mbox'))throw new ExportError('exportFormatNotAllowed',400);
    try{
      const blob=await downloadFolderZip(input.folder.id,{format,subfolders:input.subfolders},bytes=>onProgress({done:0,total:0,bytes}),signal);
      requireCurrent();
      return {blob,name:`${sanitizeName(input.folder.name,input.maxName)}.zip`};
    }catch(error){
      if(error instanceof ExportRefused)throw new ExportError(error.code,error.status);
      throw error;
    }
  }
  const emails=input.emails,total=emails.length;
  const names=fileNames(emails,input.pattern,EXT[format],input.maxName);
  const files:{name:string;data:Uint8Array}[]=[];
  let bytes=0;
  const bodies:Uint8Array[]=[];
  if(isServerFormat(format)){
    for(const [index,email] of emails.entries()){
      requireCurrent();
      let data=await fetchOne(email.id,format,signal);
      requireCurrent();
      if(format==='eml'||format==='mbox')data=stripHeaders(data,input.privacy);
      bytes+=data.length;bodies.push(data);files.push({name:names[index]!,data});
      onProgress({done:index+1,total,bytes});
    }
  }else if(format==='csv'||format==='json'){
    if(input.packaging==='single'){bodies.push(encode(format==='csv'?buildCsv(emails):buildJson(emails,input.privacy)));}
    else for(const [index,email] of emails.entries())files.push({name:names[index]!,data:encode(format==='csv'?buildCsv([email]):buildJson([email],input.privacy))});
    onProgress({done:total,total,bytes:bodies.reduce((sum,body)=>sum+body.length,0)||files.reduce((sum,file)=>sum+file.data.length,0)});
  }else{
    throw new ExportError('exportFormatNotAllowed',400);
  }
  requireCurrent();
  if(input.packaging==='zip'){
    return {blob:buildZip(files),name:`${sanitizeName(input.label,input.maxName)}.zip`};
  }
  const single=total===1&&format!=='csv'&&format!=='json'?names[0]!:`${sanitizeName(input.label,input.maxName)}.${EXT[format]}`;
  const parts=format==='mbox'?bodies.map(withNewline):bodies;
  return {blob:new Blob(parts as BlobPart[],{type:MIME[format]}),name:single};
}
