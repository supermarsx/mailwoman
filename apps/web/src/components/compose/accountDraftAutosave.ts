import {DraftSaveError,type AccountDraftDocument,type DraftEditorState} from '../../api/accountDrafts.ts';
export type DraftSaveMode='realtime'|'periodic'|'manual';
export type DraftSaveStatus='idle'|'pending'|'saving'|'saved'|'error'|'uncertain'|'conflict'|'unsupported';
export interface DraftSavePolicy {mode:DraftSaveMode;debounceMs:number;intervalSecs:number}
export interface DraftSaveTransport {load?(id:string):Promise<AccountDraftDocument>;cleanup?(id:string):Promise<AccountDraftDocument>;save(id:string,operation:string,revision:string|null,state:DraftEditorState):Promise<AccountDraftDocument>}
/** One serial writer per composer. Reconcile ambiguous saves before another write. */
export function createDraftAutosave(options:{id:string;transport:DraftSaveTransport;current:()=>boolean;status:(value:DraftSaveStatus)=>void;failure:(code:string)=>void;saved:(before:DraftEditorState,document:AccountDraftDocument)=>void;operation?:()=>string}){
 let id=options.id,revision:string|null=null,latest:DraftEditorState|undefined,lastSaved='',busy=false,halted=false,disposed=false,suspended=false;
 let policy:DraftSavePolicy={mode:'realtime',debounceMs:1000,intervalSecs:30};
 let timer:ReturnType<typeof setTimeout>|undefined,retryTimer:ReturnType<typeof setTimeout>|undefined,retryCount=0;
 let attempted:DraftEditorState|undefined,lastFailure='';
 const waiters:Array<()=>void>=[];
 const active=()=>!disposed&&options.current();
 const emit=(status:DraftSaveStatus)=>{if(active())options.status(status);};
 function retry(delay=30_000){
  if(retryTimer!==undefined||!active()||suspended||busy||!halted||!options.transport.load)return;
  const wait=Math.max(delay,Math.min(300_000,30_000*2**Math.min(retryCount++,4)));
  retryTimer=setTimeout(()=>{retryTimer=undefined;void flush();},wait);
 }
 function schedule(){
  clearTimeout(timer);timer=undefined;
  if(halted){retry();return;}
  if(!active()||suspended||busy||halted||!latest||stateKey(latest)===lastSaved||policy.mode==='manual')return;
  timer=setTimeout(()=>{timer=undefined;void flush();},policy.mode==='periodic'?policy.intervalSecs*1000:policy.debounceMs);
 }
 async function flush():Promise<boolean>{
  clearTimeout(timer);timer=undefined;
  if(!active()||suspended||busy||!latest||halted&&!options.transport.load)return false;
  clearTimeout(retryTimer);retryTimer=undefined;
  busy=true;let accepted=false,retryDelay=30_000;
  try{
   if(halted){
    try{
     let document=await options.transport.load!(id);
     if(document.cleanupPending&&document.status==='saved'&&options.transport.cleanup)document=await options.transport.cleanup(id);
     if(!active())return false;
     if(document.id!==id||document.cleanupPending||document.status!=='saved'&&document.status!=='refused')throw new DraftSaveError('draftSaveUncertain');
     if(document.status==='saved'&&(!document.state||!document.revision))throw new DraftSaveError('draftSaveUncertain');
     revision=document.revision;
     if(document.status==='saved'&&document.state&&document.revision){
      // A confirmed older save may have rebound upload IDs. Keep every newer
      // editor field; translate only the matching immutable attachment references.
      let acknowledged=false;
      if(attempted&&sameContent(attempted,document.state)){confirm(attempted,document);acknowledged=true;}
      else lastSaved=stateKey(document.state);
      if(latest&&stateKey(latest)===lastSaved){if(!acknowledged)options.saved(document.state,document);halted=false;retryCount=0;lastFailure='';emit('saved');return true;}
     }
     halted=false;attempted=undefined;
    }catch(error){
     // An absent initial claim is safe: the server's atomic revision/operation
     // fence still serializes it against an earlier request arriving late.
     if(error instanceof DraftSaveError&&error.code==='draftNotFound'&&revision===null){halted=false;attempted=undefined;}else throw error;
    }
   }
   const snapshot=structuredClone(latest!),fingerprint=stateKey(snapshot);
   if(fingerprint===lastSaved){accepted=true;emit('saved');return true;}
   attempted=snapshot;emit('saving');
   let document:AccountDraftDocument;
   try{document=await options.transport.save(id,(options.operation??(()=>crypto.randomUUID()))(),revision,snapshot);}
   catch(error){
    if(!(error instanceof DraftSaveError)||error.code!=='draftConflict'||!options.transport.load)throw error;
    // An explicit revision conflict is safe to rebase once. The editor's
    // snapshot wins; uncertain outcomes use the confirmation loop above.
    const current=await options.transport.load(id);
    if(!active()||current.id!==id||current.status!=='saved'||!current.revision||current.cleanupPending)throw error;
    revision=current.revision;
    document=await options.transport.save(id,(options.operation??(()=>crypto.randomUUID()))(),revision,snapshot);
   }
   if(!active())return false;
   if(document.status!=='saved'||!document.revision||!document.state)throw new DraftSaveError('draftSaveUncertain');
   confirm(snapshot,document);
   if(document.cleanupPending){halted=true;emit('error');options.failure('draftCleanupPending');return false;}
   accepted=true;attempted=undefined;retryCount=0;lastFailure='';emit(latest&&stateKey(latest)!==lastSaved?'pending':'saved');return true;
  }catch(error){
   if(!active())return false;
   const code=error instanceof DraftSaveError?error.code:'draftSaveUncertain';
   retryDelay=error instanceof DraftSaveError?error.retryAfterMs:30_000;
   const refused=['draftPreparationRefused','invalidDraft','invalidDraftState','protectedDraftUnsupported','draftLimit','upstreamDraftsUnavailable','draftSaveRefused','draftCapacityBusy','storageAuthorityRequired','storageAuthorityUnavailable'];
   halted=!refused.includes(code);
   emit(code==='draftConflict'?'conflict':halted?'uncertain':code==='upstreamDraftsUnavailable'?'unsupported':'error');if(lastFailure!==code){lastFailure=code;options.failure(code);}return false;
  }finally{busy=false;for(const done of waiters.splice(0))done();if(halted)retry(retryDelay);else if(accepted)schedule();}
 }
 function confirm(snapshot:DraftEditorState,document:AccountDraftDocument){
  revision=document.revision;
  const replacements=new Map<string,{blobId:string;size:number}>();
  snapshot.attachments.forEach((file,index)=>{const saved=document.state?.attachments[index];if(saved&&saved.name===file.name)replacements.set(file.blobId,saved);});
  snapshot.inline.forEach(file=>{const saved=document.state?.inline.find(value=>value.cid===file.cid);if(saved)replacements.set(file.blobId,saved);});
  if(latest)latest={...latest,attachments:latest.attachments.map(file=>({...file,...pick(replacements.get(file.blobId))})),inline:latest.inline.map(file=>({...file,...pick(replacements.get(file.blobId))}))};
  lastSaved=stateKey(document.state);options.saved(snapshot,document);
 }
 function update(state:DraftEditorState){
  if(!active())return;
  const changed=stateKey(state)!==stateKey(latest);latest=structuredClone(state);
  if(!changed)return;
  if(!busy&&!halted)emit(stateKey(latest)===lastSaved?'saved':'pending');
  if(halted){retry();return;}
  if(policy.mode!=='periodic'||timer===undefined)schedule();
 }
 function configure(value:DraftSavePolicy){policy={mode:value.mode,debounceMs:Math.max(500,Math.min(10000,value.debounceMs)),intervalSecs:Math.max(5,Math.min(3600,value.intervalSecs))};schedule();}
 function resume(document:AccountDraftDocument){clearTimeout(timer);clearTimeout(retryTimer);retryTimer=undefined;id=document.id;revision=document.revision;latest=document.state?structuredClone(document.state):undefined;lastSaved=document.status==='saved'&&document.state?stateKey(document.state):'';halted=document.status==='uncertain'||document.cleanupPending;emit(halted?'uncertain':'saved');if(halted)retry();}
 function dispose(){disposed=true;clearTimeout(timer);clearTimeout(retryTimer);latest=undefined;}
 return {seedUncertain:()=>{halted=true;emit('uncertain');retry();},settled:()=>busy?new Promise<void>(resolve=>waiters.push(resolve)):Promise.resolve(),suspend:(value:boolean)=>{suspended=value;if(value){clearTimeout(timer);clearTimeout(retryTimer);retryTimer=undefined;}else schedule();},seedRevision:(value:string|null)=>{revision=value;},update,configure,flush,resume,dispose,hasPending:()=>!!latest&&stateKey(latest)!==lastSaved,isBusy:()=>busy};
}
function pick(value:{blobId:string;size:number}|undefined):{blobId?:string;size?:number}{return value?{blobId:value.blobId,size:value.size}:{};}

function stateKey(value:DraftEditorState|undefined):string{
 if(!value)return '';
 const normalized={...value,attachments:value.attachments.map(file=>({...file,contentType:file.contentType??null}))};
 return JSON.stringify(normalized,(_key,entry:unknown)=>entry!==null&&typeof entry==='object'&&!Array.isArray(entry)?Object.fromEntries(Object.entries(entry).sort(([a],[b])=>a.localeCompare(b))):entry);
}

function sameContent(a:DraftEditorState,b:DraftEditorState):boolean {
 const withoutBlobs=(value:DraftEditorState)=>({...value,attachments:value.attachments.map(({blobId:_blob,...file})=>file),inline:value.inline.map(({blobId:_blob,...file})=>file)});
 return stateKey(withoutBlobs(a) as DraftEditorState)===stateKey(withoutBlobs(b) as DraftEditorState);
}
