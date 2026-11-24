// Shared helpers of the native/Node equivalence checks. Synthetic principals carry
// fake identifiers only; a live handoff is read only from a private file path.
import {spawnSync} from 'node:child_process';
import {writeFileSync} from 'node:fs';
import {prepareFixtureRequests} from '../../fixture-requests.mjs';
import {goalMixDraftSpecs} from '../../representative-workload.mjs';
import {nativePlan,nativeBinary} from '../native-measure.mjs';

export function syntheticPrincipals(count,{messages=50}={}){
 return Array.from({length:count},(_,i)=>({cookie:`mw_session=sess${i}x; mw_csrf=csrf${i}%2Bv`,csrfToken:`csrf${i}%2Bv`,accountId:`acct-${i}`,ownerAccountId:`acct-${i}`,
  messageIds:Array.from({length:messages},(_,j)=>`M${i}m${j}`),inboxId:`inbox-${i}`,archiveId:`archive-${i}`,trashId:`trash-${i}`,draftsId:`drafts-${i}`,attachmentBlobId:`blob/${i} é`,
  attachmentSampleBytes:64,bodySampleBytes:40,plantedQuarterly:i%3===0?60:9,plantedZephyrine:0,clientAddress:`198.18.0.${1+i}`}));
}
export const syntheticSimulation=count=>({mode:'fixture-loopback-trusted-proxy',clientCount:count});

export function prepare({principals,clientSimulation,engineMode='native',representative=true}){
 return prepareFixtureRequests({principals,clientSimulation,engineMode,representative});
}
export function writePlan(path,{principals,clientSimulation,engineMode='native',representative=true,groups,env={}}){
 writeFileSync(path,JSON.stringify(nativePlan({principals,clientSimulation,engineMode,representative,groups,enforcedPolicy:null,env})),{mode:0o600});
}
// The per-principal spec Node sends for slot `slot` of `group`, found through the group's own rotation.
export function nodeSpecAt(groups,groupName,slot,principalIndex,principalCount){
 const list=groups[groupName],length=list.length,step=Math.ceil(principalCount/length);
 const j=(((principalIndex-slot*step)%principalCount)+principalCount)%principalCount;
 const sequence=slot+length*j,template=list[slot];
 const spec={...template,...template.materialize(sequence,{requestCount:length}),name:template.name};
 if(spec.principalIndex!==principalIndex)throw new Error('rotation inverse failed');
 return spec;
}
// name -> first {group, slot} that carries it
export function shapeIndex(groups){
 const index=new Map();
 for(const [group,list] of Object.entries(groups))list.forEach((spec,slot)=>{if(!index.has(spec.name))index.set(spec.name,{group,slot});});
 return index;
}
export function nodeSpecByName(groups,principals,name,principalIndex,draftId){
 if(draftId!==undefined)return goalMixDraftSpecs(principals[principalIndex],{}).discard(draftId);
 // the discard slot with an empty pool: the save template under the discard label (the pool of `groups` may have filled)
 if(name==='draft-discard')return {...goalMixDraftSpecs(principals[principalIndex],{}).save,name};
 const at=shapeIndex(groups).get(name);if(!at)throw new Error('unknown shape '+name);
 return nodeSpecAt(groups,at.group,at.slot,principalIndex,principals.length);
}
export function nodeVerdict(spec,status,body){
 if(status!==200)return false;
 try{
  if(spec.validateBytes)return Boolean(spec.validateBytes(body));
  return Boolean(spec.validate(JSON.parse(body.toString('utf8'))));
 }catch{return false;}
}
export function runNative(args){
 const result=spawnSync(nativeBinary(),args,{encoding:'utf8',maxBuffer:1<<30});
 if(result.status!==0)throw new Error('mw-loadgen failed: '+(result.stderr||'').slice(0,500));
 return result.stdout;
}
