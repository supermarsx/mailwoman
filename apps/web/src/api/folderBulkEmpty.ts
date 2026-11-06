import type {Client} from './client.ts';
import type {Mailbox} from './jmap-types.ts';
import {canEmpty,prepareEmpty,runEmpty,type EmptyAbility,type EmptyAuthority,type EmptyPreview,type EmptyResult} from './emptyFolder.ts';
/**
 * Empty several folders in one go: every folder is counted first (so the confirmation can name each count and
 * whether its mail goes to Trash or is deleted for good), then emptied one after another in the engine's bounded
 * batches (`Mailbox/empty`, or Email/query plus Email/set against an upstream JMAP server). A folder that fails
 * does not stop the others; the report lists each failure with the reason.
 */
type Transport=Pick<Client,'jmap'>;
export interface EmptyPlanRow {box:Mailbox;preview:EmptyPreview|null;/** Why this folder is skipped, when it is. */skip:'search'|'unsupported'|'empty'|'count-failed'|null;error?:string}
export interface EmptyPlan {rows:EmptyPlanRow[];/** Messages that will be touched. */total:number;/** Messages deleted for good (Trash, Junk, accounts without Trash). */permanent:number}

/** Count every selected folder. Folders that cannot be emptied, are empty already or could not be counted are skipped with a reason. */
export async function planEmpty(client:Transport,accountId:string,boxes:readonly Mailbox[],all:readonly Mailbox[],ability:EmptyAbility,authority:EmptyAuthority):Promise<EmptyPlan> {
 const rows:EmptyPlanRow[]=[];
 for(const box of boxes){
  if(box.mailwomanSearchQuery!==undefined){rows.push({box,preview:null,skip:'search'});continue;}
  if(!canEmpty(box,all,ability)){rows.push({box,preview:null,skip:'unsupported'});continue;}
  try{
   const preview=await prepareEmpty(client,accountId,box,all,ability,authority);
   rows.push({box,preview,skip:preview.total===0?'empty':null});
  }catch(e){rows.push({box,preview:null,skip:'count-failed',error:e instanceof Error?e.message:String(e)});}
 }
 const live=rows.filter(row=>row.preview&&row.skip===null);
 return {rows,total:live.reduce((n,row)=>n+row.preview!.total,0),permanent:live.filter(row=>row.preview!.outcome==='destroy').reduce((n,row)=>n+row.preview!.total,0)};
}
export interface BulkEmptyProgress {done:number;of:number;current:string|null;processed:number}
export interface BulkEmptyReport {emptied:{id:string;name:string;processed:number}[];failed:{id:string;name:string;reason:'refused'|'stopped'|'uncertain';remaining:number}[];processed:number}
export async function runBulkEmpty(client:Transport,plan:EmptyPlan,authority:EmptyAuthority,progress:(p:BulkEmptyProgress)=>void=()=>{}):Promise<BulkEmptyReport> {
 const todo=plan.rows.filter(row=>row.preview&&row.skip===null);
 const report:BulkEmptyReport={emptied:[],failed:[],processed:0};
 let base=0;
 for(const [index,row] of todo.entries()){
  const preview=row.preview!;
  progress({done:index,of:todo.length,current:preview.name,processed:report.processed});
  if(!authority.current()||authority.signal.aborted||authority.stopRequested?.()){report.failed.push({id:preview.mailboxId,name:preview.name,reason:'stopped',remaining:preview.total});continue;}
  const result:EmptyResult=await runEmpty(client,preview,authority,partial=>progress({done:index,of:todo.length,current:preview.name,processed:base+partial.processed}));
  report.processed+=result.processed;base=report.processed;
  if(result.remaining===0&&!result.uncertain)report.emptied.push({id:preview.mailboxId,name:preview.name,processed:result.processed});
  else report.failed.push({id:preview.mailboxId,name:preview.name,reason:result.uncertain?'uncertain':result.refused>0?'refused':'stopped',remaining:result.remaining});
 }
 progress({done:todo.length,of:todo.length,current:null,processed:report.processed});
 return report;
}
