export interface SessionBatchResult { revoked: string[]; unchanged: string[]; failed: string[]; skipped: string[]; currentRevoked: boolean }
/** One reviewed snapshot; never retries mutations. Keep the current session until
 * every other selected session has been successfully handled. */
export async function revokeSessionBatch<T>(rows:readonly T[],key:(row:T)=>string,current:(row:T)=>boolean,revoke:(row:T)=>Promise<boolean|void>,authorityCurrent:()=>boolean=()=>true):Promise<SessionBatchResult>{
 const result:SessionBatchResult={revoked:[],unchanged:[],failed:[],skipped:[],currentRevoked:false};
 const unique=[...new Map(rows.map(row=>[key(row),row])).values()].sort((a,b)=>Number(current(a))-Number(current(b)));
 let stopped=false;
 for(const row of unique){
  const id=key(row);
  if(stopped||!authorityCurrent()||(current(row)&&result.failed.length>0)){result.skipped.push(id);continue;}
  try{
   if(await revoke(row)===false)result.unchanged.push(id);
   else{result.revoked.push(id);if(current(row))result.currentRevoked=true;}
  }catch(error){
   result.failed.push(id);
   const status=typeof error==='object'&&error!==null&&'status' in error?Number(error.status):0;
   // Lost authority, refusal of fresh proof, pacing and uncertain transport
   // failures stop the batch; they never trigger repeated confirmation prompts.
   if(status===0||status===401||status===403||status===429||status>=500)stopped=true;
  }
 }
 return result;
}
