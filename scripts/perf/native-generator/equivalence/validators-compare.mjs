// Validator equivalence: the same response corpus is judged by the Node validators
// (spec.validate / spec.validateBytes, a throw rejects) and by `mw-loadgen validate`;
// every verdict must be identical. The corpus is a set of well-formed responses per
// shape plus systematic corruptions of each: every node of the JSON tree deleted,
// nulled, retyped (object, array, string, number, boolean) and, for arrays, an
// element dropped, duplicated or added; plus non-JSON, truncated and empty bodies
// and non-200 statuses. Recorded live responses can be added with --corpus=<file>
// (entries {principalIndex,name,status,body|bodyBase64,draftId?}, private file).
// Usage: node validators-compare.mjs [--handoff=<private>] [--corpus=<private>] [--principals=N]
import {mkdtempSync,rmSync,writeFileSync,readFileSync} from 'node:fs';
import {tmpdir} from 'node:os';
import {join} from 'node:path';
import {syntheticPrincipals,syntheticSimulation,prepare,writePlan,nodeSpecByName,nodeVerdict,runNative,shapeIndex} from './common.mjs';
import {good} from './synthetic-responses.mjs';

const args=Object.fromEntries(process.argv.slice(2).map(a=>{const i=a.indexOf('=');return [a.slice(2,i),a.slice(i+1)];}));
let principals,clientSimulation;
if(args.handoff){const h=JSON.parse(readFileSync(args.handoff,'utf8'));principals=h.principals;clientSimulation=h.clientSimulation;}
else{const n=Number(args.principals??4);principals=syntheticPrincipals(n);clientSimulation=syntheticSimulation(n);}
const prepared=prepare({principals,clientSimulation});
const names=[...shapeIndex(prepared.groups).keys()];


// JSON.parse edge cases around a good body: duplicate keys (last wins), escapes, lone surrogate escape, BOM,
// surrounding whitespace, grammar errors, raw control character, deep nesting.
function edgeCases(text){
 const inner=text.startsWith('{')?text.slice(1):null,R=String.raw;
 return [
  ...(inner===null?[]:[R`{"x":"\ud800","y":"\u00e9\ud83d\ude00",`+inner,text.slice(0,-1)+',"methodResponses":[]}',text.slice(0,-1)+',"accountId":"zz"}','{"accountId":"zz",'+inner,text.slice(0,-1)+',}','{"x":01,'+inner,'{"x":1.,'+inner,'{"x":"\u0001",'+inner,R`{"x":"\q",`+inner,'{"x":-0,"y":1e400,'+inner]),
  text.replace('"accountId"',R`"\u0061ccountId"`),'\ufeff'+text,' \n\t'+text+'\r\n ',text+'x','['.repeat(3000)+']'.repeat(3000),
 ];
}
// ---- corruptions ----
const replacements=[undefined,null,{},[],'','zz',0,1,-1,1.5,true,false];
function* mutants(value){
 function* walk(node,path){
  if(Array.isArray(node)){
   for(let i=0;i<node.length;i++){yield* replace(path.concat(i));yield* walk(node[i],path.concat(i));}
   yield {path,op:'drop'};yield {path,op:'dup'};yield {path,op:'add'};
  }else if(node&&typeof node==='object'){
   for(const key of Object.keys(node)){yield* replace(path.concat(key));yield* walk(node[key],path.concat(key));}
   yield {path,op:'extra'};
  }
 }
 function* replace(path){for(const r of replacements)yield {path,op:'set',value:r};}
 yield* walk(value,[]);
}
function apply(root,{path,op,value}){
 const copy=structuredClone(root);
 if(!path.length&&op==='set')return value;
 let parent=copy;for(const step of path.slice(0,-1))parent=parent[step];
 const last=path.at(-1),target=path.length?parent[last]:copy;
 if(op==='set'){if(value===undefined){if(Array.isArray(parent))parent.splice(last,1);else delete parent[last];}else parent[last]=structuredClone(value);}
 else if(op==='drop'&&target.length)target.pop();
 else if(op==='dup'&&target.length)target.push(structuredClone(target[0]));
 else if(op==='add')target.push('zz');
 else if(op==='extra')target.zzExtra=1;
 return copy;
}

let corpus=[],nodeVerdicts=[],batchBytes=0;
const dir=mkdtempSync(join(tmpdir(),'mw-native-validators-'));
const plan=join(dir,'plan.private.json'),file=join(dir,'corpus.private.json');
writePlan(plan,{principals,clientSimulation,groups:prepared.groups});
const perShape={};let differ=0,total=0;const examples=[];
function flush(){
 if(!corpus.length)return;
 writeFileSync(file,JSON.stringify(corpus),{mode:0o600});
 const native=JSON.parse(runNative(['validate',`--plan=${plan}`,`--corpus=${file}`]));
 corpus.forEach((entry,i)=>{
  const key=entry.draftId?'draft-discard(id)':entry.name,row=perShape[key]??={entries:0,accepted:0,rejected:0,differ:0};
  row.entries++;row[nodeVerdicts[i]?'accepted':'rejected']++;total++;
  if(native[i].ok!==nodeVerdicts[i]){row.differ++;differ++;if(examples.length<8)examples.push({name:key,node:nodeVerdicts[i],native:native[i].ok,body:(entry.body??entry.bodyBase64).slice(0,400)});}
 });
 corpus=[];nodeVerdicts=[];batchBytes=0;
}
const add=(entry,spec)=>{corpus.push(entry);batchBytes+=(entry.body??entry.bodyBase64??'').length;const body=entry.bodyBase64!==undefined?Buffer.from(entry.bodyBase64,'base64'):Buffer.from(entry.body??'','utf8');nodeVerdicts.push(nodeVerdict(spec,entry.status,body));if(batchBytes>64e6)flush();};
const synthetic=Math.min(principals.length,Number(args['mutate-principals']??4));
for(let i=0;i<synthetic;i++){
 const p=principals[i];
 for(const name of [...names,'draft-discard(id)']){
  const draftId=name==='draft-discard(id)'?'D-old-7':undefined,shape=draftId?'draft-discard':name;
  const spec=nodeSpecByName(prepared.groups,principals,shape,i,draftId);
  const base={principalIndex:i,name:shape,...(draftId?{draftId}:{})};
  if(name==='attachment-download'){
   const size=p.attachmentSampleBytes??4,ok=Buffer.alloc(size,'ABCD');
   const variants=[ok,ok.subarray(1),Buffer.concat([ok,Buffer.from('A')]),Buffer.alloc(size,'ABCE'),Buffer.alloc(size,'ABCD').fill(0x42,size-1),Buffer.alloc(0)];
   for(const v of variants)add({...base,status:200,bodyBase64:v.toString('base64')},spec);
   add({...base,status:206,bodyBase64:ok.toString('base64')},spec);
   continue;
  }
  const value=good(name,p,spec,draftId),text=JSON.stringify(value);
  add({...base,status:200,body:text},spec);
  for(const extra of [{...base,status:500,body:text},{...base,status:200,body:text.slice(0,-1)},{...base,status:200,body:''},{...base,status:200,body:'null'},{...base,status:200,body:'[]'},{...base,status:200,body:'"x"'},{...base,status:200,bodyBase64:Buffer.concat([Buffer.from(text.slice(0,-2)),Buffer.from([0xff,0xfe]),Buffer.from(text.slice(-2))]).toString('base64')},
   // parser edge cases: duplicate keys (last wins), escapes, lone surrogate, BOM, whitespace, grammar errors
   ...edgeCases(text).map(body=>({...base,status:200,body}))])add(extra,spec);
  for(const m of mutants(value)){const v=apply(value,m);add({...base,status:200,body:v===undefined?'':JSON.stringify(v)},spec);}
 }
}
let recorded=0,recordedMutants=0;
try{
 if(args.corpus){
  for(const entry of JSON.parse(readFileSync(args.corpus,'utf8'))){
   const spec=nodeSpecByName(prepared.groups,principals,entry.name,entry.principalIndex,entry.draftId);
   const base={principalIndex:entry.principalIndex,name:entry.name,...(entry.draftId?{draftId:entry.draftId}:{})};
   add({...base,status:entry.status,...(entry.bodyBase64!==undefined?{bodyBase64:entry.bodyBase64}:{body:entry.body})},spec);recorded++;
   // corruptions of the recorded (real) responses of the first --mutate-recorded principals
   if(entry.principalIndex<Number(args['mutate-recorded']??1)&&entry.status===200&&entry.name!=='attachment-download'){
    const text=entry.bodyBase64!==undefined?Buffer.from(entry.bodyBase64,'base64').toString('utf8'):entry.body;let value;try{value=JSON.parse(text);}catch{continue;}
    for(const m of mutants(value)){const v=apply(value,m);add({...base,status:200,body:v===undefined?'':JSON.stringify(v)},spec);recordedMutants++;}
   }
  }
 }
 flush();
 console.log(JSON.stringify({entries:total,recordedLiveEntries:recorded,recordedMutants,differ,perShape,examples},null,1));
 process.exitCode=differ?1:0;
}finally{rmSync(dir,{recursive:true,force:true});}
