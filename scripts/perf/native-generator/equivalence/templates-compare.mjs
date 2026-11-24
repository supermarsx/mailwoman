// Byte-for-byte comparison of the HTTP/1.1 requests the Node generator sends
// (captured raw from fetch on a loopback socket) with the requests mw-loadgen
// builds, for every shape of every group and the first N principals.
// Usage: node templates-compare.mjs [--handoff=<private file>] [--principals=N] [--compare=K]
import net from 'node:net';
import {mkdtempSync,rmSync,readFileSync} from 'node:fs';
import {tmpdir} from 'node:os';
import {join} from 'node:path';
import {syntheticPrincipals,syntheticSimulation,prepare,writePlan,nodeSpecAt,runNative} from './common.mjs';
import {goalMixDraftSpecs} from '../../representative-workload.mjs';

const args=Object.fromEntries(process.argv.slice(2).map(a=>{const i=a.indexOf('=');return [a.slice(2,i),a.slice(i+1)];}));
let principals,clientSimulation;
if(args.handoff){const h=JSON.parse(readFileSync(args.handoff,'utf8'));principals=h.principals;clientSimulation=h.clientSimulation;}
else{const n=Number(args.principals??7);principals=syntheticPrincipals(n);clientSimulation=syntheticSimulation(n);}
const compare=Math.min(principals.length,Number(args.compare??3));
const prepared=prepare({principals,clientSimulation});
const dir=mkdtempSync(join(tmpdir(),'mw-native-templates-'));
const captured=[];
const server=net.createServer(socket=>{let buf=Buffer.alloc(0);socket.on('data',d=>{buf=Buffer.concat([buf,d]);for(;;){const end=buf.indexOf('\r\n\r\n');if(end<0)return;const head=buf.subarray(0,end).toString('latin1');const m=/\r\ncontent-length: (\d+)/i.exec(head);const total=end+4+(m?Number(m[1]):0);if(buf.length<total)return;captured.push(buf.subarray(0,total));buf=buf.subarray(total);socket.write('HTTP/1.1 200 OK\r\ncontent-length: 2\r\n\r\n{}');}});});
await new Promise(r=>server.listen(0,'127.0.0.1',r));
const host=`127.0.0.1:${server.address().port}`,base=`http://${host}`;
let compared=0,mismatched=0;const failures=[];
try{
 const plan=join(dir,'plan.private.json');
 writePlan(plan,{principals,clientSimulation,groups:prepared.groups});
 for(const group of Object.keys(prepared.groups)){
  const native=JSON.parse(runNative(['templates',`--plan=${plan}`,`--endpoint=${group}`,`--count=${compare}`,`--host=${host}`]));
  const list=prepared.groups[group];
  for(let i=0;i<compare;i++){
   const seen=new Set();
   const jobs=[];
   list.forEach((template,slot)=>{if(!seen.has(template.name)){seen.add(template.name);jobs.push({name:template.name,spec:nodeSpecAt(prepared.groups,group,slot,i,principals.length)});}});
   if(seen.has('draft-discard'))jobs.push({name:'draft-discard(id)',spec:{...goalMixDraftSpecs(principals[i],{}).discard('DRAFT-ID'),headers:{...prepared.clientModel.clients[i].headers,...goalMixDraftSpecs(principals[i],{}).discard('DRAFT-ID').headers}}});
   for(const {name,spec} of jobs){
    const before=captured.length;
    // exactly measure()'s fetch call
    const response=await fetch(new URL(spec.path,base),{method:spec.method??'GET',headers:{...prepared.headers,...spec.headers},...(spec.body!==undefined?{body:spec.body}:{}),redirect:'error'});
    await response.arrayBuffer();
    const node=captured[before];
    const rust=native.find(t=>t.principalIndex===i&&t.name===name);
    compared++;
    const flags=rust&&(name==='draft-discard(id)'||(rust.serialize===(spec.serializePrincipal===true)&&rust.stateful===(spec.statefulMutation===true)));
    if(!rust||Buffer.from(rust.http1,'utf8').compare(node)!==0||!flags){mismatched++;if(failures.length<5)failures.push({group,principal:i,name,flags,node:node?.toString('utf8').slice(0,600),native:rust?.http1.slice(0,600)});}
   }
  }
 }
}finally{server.close();rmSync(dir,{recursive:true,force:true});}
console.log(JSON.stringify({groups:Object.keys(prepared.groups).length,principals:principals.length,comparedPrincipals:compare,requestsCompared:compared,mismatched,failures},null,1));
process.exitCode=mismatched?1:0;
