import {DraftSaveError,type AccountDraftDocument,type DraftEditorState} from '../../api/accountDrafts.ts';
export type DraftSaveMode='realtime'|'periodic'|'manual';
export type DraftSaveStatus='idle'|'pending'|'saving'|'saved'|'error'|'uncertain'|'conflict'|'unsupported';
export interface DraftSavePolicy {mode:DraftSaveMode;debounceMs:number;intervalSecs:number}
export interface DraftSaveTransport {save(id:string,operation:string,revision:string|null,state:DraftEditorState):Promise<AccountDraftDocument>}
/** One serial writer per composer. Ambiguous transport outcomes never retry. */
export function createDraftAutosave(options:{id:string;transport:DraftSaveTransport;current:()=>boolean;status:(value:DraftSaveStatus)=>void;failure:(code:string)=>void;saved:(before:DraftEditorState,document:AccountDraftDocument)=>void;operation?:()=>string}){
 let id=options.id,revision:string|null=null,latest:DraftEditorState|undefined,lastSaved='',busy=false,halted=false,disposed=false,suspended=false;
 let policy:DraftSavePolicy={mode:'realtime',debounceMs:1000,intervalSecs:30};
 let timer:ReturnType<typeof setTimeout>|undefined;
 const waiters:Array<()=>void>=[];
 const active=()=>!disposed&&options.current();
 const emit=(status:DraftSaveStatus)=>{if(active())options.status(status);};
 function schedule(){
  clearTimeout(timer);timer=undefined;
  if(!active()||suspended||busy||halted||!latest||stateKey(latest)===lastSaved||policy.mode==='manual')return;
  timer=setTimeout(()=>{timer=undefined;void flush();},policy.mode==='periodic'?policy.intervalSecs*1000:policy.debounceMs);
 }
 async function flush():Promise<boolean>{
  clearTimeout(timer);timer=undefined;
  if(!active()||suspended||busy||halted||!latest)return false;
  const snapshot=structuredClone(latest),fingerprint=stateKey(snapshot);
  if(fingerprint===lastSaved)return true;
  busy=true;let accepted=false;emit('saving');
  try{
   const document=await options.transport.save(id,(options.operation??(()=>crypto.randomUUID()))(),revision,snapshot);
   if(!active())return false;
   if(document.status!=='saved'||!document.revision||!document.state)throw new DraftSaveError('draftSaveUncertain');
   revision=document.revision;
   // Only translate confirmed blob references; never overwrite newer text edits.
   const replacements=new Map<string,{blobId:string;size:number}>();
   snapshot.attachments.forEach((file,index)=>{const saved=document.state?.attachments[index];if(saved)replacements.set(file.blobId,saved);});
   snapshot.inline.forEach(file=>{const saved=document.state?.inline.find(value=>value.cid===file.cid);if(saved)replacements.set(file.blobId,saved);});
   if(latest){latest={...latest,attachments:latest.attachments.map(file=>({...file,...pick(replacements.get(file.blobId))})),inline:latest.inline.map(file=>({...file,...pick(replacements.get(file.blobId))}))};}
   lastSaved=stateKey(document.state);options.saved(snapshot,document);
   if(document.cleanupPending){halted=true;emit('error');options.failure('draftCleanupPending');return false;}
   accepted=true;emit(latest&&stateKey(latest)!==lastSaved?'pending':'saved');return true;
  }catch(error){
   if(!active())return false;
   const code=error instanceof DraftSaveError?error.code:'draftSaveUncertain';
   const refused=['draftPreparationRefused','invalidDraft','invalidDraftState','protectedDraftUnsupported','draftLimit','upstreamDraftsUnavailable','draftSaveRefused','draftCapacityBusy','storageAuthorityRequired','storageAuthorityUnavailable'];
   halted=!refused.includes(code);
   emit(code==='draftConflict'?'conflict':halted?'uncertain':code==='upstreamDraftsUnavailable'?'unsupported':'error');options.failure(code);return false;
  }finally{busy=false;for(const done of waiters.splice(0))done();if(accepted)schedule();}
 }
 function update(state:DraftEditorState){
  if(!active())return;
  const changed=stateKey(state)!==stateKey(latest);latest=structuredClone(state);
  if(!changed)return;
  if(!busy&&!halted)emit(stateKey(latest)===lastSaved?'saved':'pending');
  if(policy.mode!=='periodic'||timer===undefined)schedule();
 }
 function configure(value:DraftSavePolicy){policy={mode:value.mode,debounceMs:Math.max(500,Math.min(10000,value.debounceMs)),intervalSecs:Math.max(5,Math.min(3600,value.intervalSecs))};schedule();}
 function resume(document:AccountDraftDocument){clearTimeout(timer);id=document.id;revision=document.revision;latest=document.state?structuredClone(document.state):undefined;lastSaved=document.status==='saved'&&document.state?stateKey(document.state):'';halted=document.status==='uncertain'||document.cleanupPending;emit(halted?'uncertain':'saved');}
 function dispose(){disposed=true;clearTimeout(timer);latest=undefined;}
 return {seedUncertain:()=>{halted=true;emit('uncertain');},settled:()=>busy?new Promise<void>(resolve=>waiters.push(resolve)):Promise.resolve(),suspend:(value:boolean)=>{suspended=value;if(value)clearTimeout(timer);else schedule();},seedRevision:(value:string|null)=>{revision=value;},update,configure,flush,resume,dispose,hasPending:()=>!!latest&&stateKey(latest)!==lastSaved,isBusy:()=>busy};
}
function pick(value:{blobId:string;size:number}|undefined):{blobId?:string;size?:number}{return value?{blobId:value.blobId,size:value.size}:{};}

function stateKey(value:DraftEditorState|undefined):string{
 if(!value)return '';
 const normalized={...value,attachments:value.attachments.map(file=>({...file,contentType:file.contentType??null}))};
 return JSON.stringify(normalized,(_key,entry:unknown)=>entry!==null&&typeof entry==='object'&&!Array.isArray(entry)?Object.fromEntries(Object.entries(entry).sort(([a],[b])=>a.localeCompare(b))):entry);
}
