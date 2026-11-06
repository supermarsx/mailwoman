import type {Client} from './client.ts';
import type {Mailbox} from './jmap-types.ts';
import {CAP_CORE,CAP_MAIL,type EmailQueryResponse,type EmailSetResponse} from './jmap-types.ts';
import {responseFor} from './jmap.ts';
import {canEmpty,prepareEmpty,runEmpty,type EmptyAbility,type EmptyAuthority,type EmptyPreview,type EmptyResult} from './emptyFolder.ts';
/**
 * Empty several folders in one go: every folder is counted first (so the confirmation can name each count and
 * whether its mail goes to Trash or is deleted for good), then emptied one after another in the engine's bounded
 * batches (`Mailbox/empty`, or Email/query plus Email/set against an upstream JMAP server). A folder that fails
 * does not stop the others; the report lists each failure with the reason.
 */
type Transport=Pick<Client,'jmap'>;
export interface EmptyPlanRow {box:Mailbox;preview:EmptyPreview|null;/** Why this folder is skipped, when it is. */skip:'search'|'unsupported'|'empty'|'count-failed'|'permanent-denied'|null;error?:string}
export interface EmptyPlan {rows:EmptyPlanRow[];/** Messages that will be touched. */total:number;/** Messages deleted for good (Trash, Junk, accounts without Trash). */permanent:number}

/**
 * Count every selected folder. Folders that cannot be emptied, are empty already or could not be counted are skipped with a reason.
 * `allowPermanent` is the administrator's permanent-delete policy (`properties.message.permanentDelete` is not `deny`): when it is
 * off, a folder whose mail would be deleted for good (Trash, Junk, accounts without Trash) is skipped instead of offered.
 */
export async function planEmpty(client:Transport,accountId:string,boxes:readonly Mailbox[],all:readonly Mailbox[],ability:EmptyAbility,authority:EmptyAuthority,allowPermanent=true):Promise<EmptyPlan> {
 const rows:EmptyPlanRow[]=[];
 for(const box of boxes){
  if(box.mailwomanSearchQuery!==undefined){rows.push({box,preview:null,skip:'search'});continue;}
  if(!canEmpty(box,all,ability)){rows.push({box,preview:null,skip:'unsupported'});continue;}
  try{
   const preview=await prepareEmpty(client,accountId,box,all,ability,authority);
   rows.push({box,preview,skip:preview.total===0?'empty':preview.outcome==='destroy'&&!allowPermanent?'permanent-denied':null});
  }catch(e){rows.push({box,preview:null,skip:'count-failed',error:e instanceof Error?e.message:String(e)});}
 }
 const live=rows.filter(row=>row.preview&&row.skip===null);
 return {rows,total:live.reduce((n,row)=>n+row.preview!.total,0),permanent:live.filter(row=>row.preview!.outcome==='destroy').reduce((n,row)=>n+row.preview!.total,0)};
}
export interface BulkEmptyProgress {done:number;of:number;current:string|null;processed:number}
/** Messages moved to Trash by a bulk empty, kept so the whole action can be undone. */
export interface EmptyUndo {folders:{mailboxId:string;trashId:string;ids:string[]}[]}
export interface BulkEmptyReport {undo:EmptyUndo|null;emptied:{id:string;name:string;processed:number}[];failed:{id:string;name:string;reason:'refused'|'stopped'|'uncertain';remaining:number}[];processed:number}
export async function runBulkEmpty(client:Transport,plan:EmptyPlan,authority:EmptyAuthority,progress:(p:BulkEmptyProgress)=>void=()=>{}):Promise<BulkEmptyReport> {
 const todo=plan.rows.filter(row=>row.preview&&row.skip===null);
 const report:BulkEmptyReport={undo:null,emptied:[],failed:[],processed:0};
 const kept:EmptyUndo['folders']=[];
 let base=0;
 for(const [index,row] of todo.entries()){
  const preview=row.preview!;
  progress({done:index,of:todo.length,current:preview.name,processed:report.processed});
  if(!authority.current()||authority.signal.aborted||authority.stopRequested?.()){report.failed.push({id:preview.mailboxId,name:preview.name,reason:'stopped',remaining:preview.total});continue;}
  const ids=preview.outcome==='trash'&&preview.trashId?await collectIds(client,preview,authority):null;
  const result:EmptyResult=await runEmpty(client,preview,authority,partial=>progress({done:index,of:todo.length,current:preview.name,processed:base+partial.processed}));
  report.processed+=result.processed;base=report.processed;
  if(ids&&ids.length>0&&result.processed>0&&preview.trashId)kept.push({mailboxId:preview.mailboxId,trashId:preview.trashId,ids});
  if(result.remaining===0&&!result.uncertain)report.emptied.push({id:preview.mailboxId,name:preview.name,processed:result.processed});
  else report.failed.push({id:preview.mailboxId,name:preview.name,reason:result.uncertain?'uncertain':result.refused>0?'refused':'stopped',remaining:result.remaining});
 }
 progress({done:todo.length,of:todo.length,current:null,processed:report.processed});
 if(kept.length>0)report.undo={folders:kept};
 return report;
}

/** A folder with more messages than this is emptied without an Undo: remembering its ids would not be reasonable. */
export const UNDO_MESSAGE_CAP=2000;
const PAGE=500;
/** The ids of the messages a to-Trash empty is about to move, or null when there are too many (or the list cannot be read) to offer Undo. */
async function collectIds(client:Transport,preview:EmptyPreview,authority:EmptyAuthority):Promise<string[]|null> {
 if(preview.total>UNDO_MESSAGE_CAP)return null;
 const ids:string[]=[],seen=new Set<string>();
 try{
  for(let position=0;ids.length<preview.total;position+=PAGE){
   const response=await client.jmap({using:[CAP_CORE,CAP_MAIL],methodCalls:[['Email/query',{accountId:preview.accountId,filter:{inMailbox:preview.mailboxId},position,limit:PAGE},'ids']]},{signal:authority.signal});
   const page=responseFor<EmailQueryResponse>(response,'ids');
   if(page.accountId!==preview.accountId||!Array.isArray(page.ids))return null;
   const fresh=page.ids.filter(id=>!seen.has(id));
   if(fresh.length===0)break;
   for(const id of fresh){seen.add(id);ids.push(id);}
  }
 }catch{return null;}
 return ids;
}
/**
 * Move the messages a bulk empty sent to Trash back (`reverse`) or to Trash again (`forward`, the redo). Messages that are no longer
 * where they were (deleted or moved elsewhere in the meantime) are left alone and counted; the number is returned.
 */
export async function replayEmpty(client:Transport,accountId:string,undo:EmptyUndo,direction:'reverse'|'forward',authority:EmptyAuthority):Promise<{moved:number;skipped:number}> {
 let moved=0,skipped=0;
 for(const folder of undo.folders){
  const from=direction==='reverse'?folder.trashId:folder.mailboxId,to=direction==='reverse'?folder.mailboxId:folder.trashId;
  for(let i=0;i<folder.ids.length;i+=EMPTY_BATCH_IDS){
   if(!authority.current()||authority.signal.aborted)throw new Error('Folder operation is no longer active');
   const chunk=folder.ids.slice(i,i+EMPTY_BATCH_IDS);
   const update=Object.fromEntries(chunk.map(id=>[id,{[`mailboxIds/${from}`]:null,[`mailboxIds/${to}`]:true}]));
   const response=await client.jmap({using:[CAP_CORE,CAP_MAIL],methodCalls:[['Email/set',{accountId,update},'replay']]},{signal:authority.signal});
   const set=responseFor<EmailSetResponse>(response,'replay');
   if(set.accountId!==accountId)throw new Error('Invalid response');
   const done=Object.keys(set.updated??{}).filter(id=>chunk.includes(id)).length;
   moved+=done;skipped+=chunk.length-done;
  }
 }
 return {moved,skipped};
}
const EMPTY_BATCH_IDS=100;
