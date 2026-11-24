import test from 'node:test';
import assert from 'node:assert/strict';
import {vmWitness,hostWitness,stageQuiet,DEFAULT_THRESHOLDS} from './rig-quiet.mjs';
import {allocate} from './rig-cpuset.mjs';

// 8 vCPUs: run on 1-5 (app 1-2, generator 3-4, rig 5); outside: 0, 6, 7. One sample per second over 10 s.
const plan=allocate({app:2,generator:2,rig:1},{vcpus:8});
function samples({outsideBusyPerSecond=0,other={},runBusyPerSecond=100}={}){
 return Array.from({length:12},(_,i)=>{
  const cpus={};
  for(let cpu=0;cpu<8;cpu++){const busy=plan.usedCpus.includes(cpu)?runBusyPerSecond*i:outsideBusyPerSecond*i;cpus[cpu]=[busy,100*i];}
  const others={};for(const [id,cps] of Object.entries(other))others[id]={usage_usec:cps*1e6*i,...(id==='bbbbbbbbbbbb'?{name:'mw-shots-greenmail'}:{})};
  return {schema:1,tMs:i*1000,containers:{},others,procStat:{cpus}};
 });
}
const window={startMs:1000,endMs:11000};

test('a quiet VM: idle outside CPUs and near-idle other containers',()=>{
 const vm=vmWitness(samples({outsideBusyPerSecond:2,other:{aaaaaaaaaaaa:0.01}}),window,{plan});
 assert.deepEqual(vm.reasons,[]);
 assert.equal(vm.outsideCpuset.busyShare,0.02);assert.equal(vm.otherContainers.count,1);
 assert.equal(vm.runCpuset.busyCpus,5);
});

test('another container above 5% of one CPU, or busy outside CPUs, contaminates the stage',()=>{
 const loud=vmWitness(samples({other:{bbbbbbbbbbbb:0.2}}),window,{plan});
 assert.equal(loud.reasons.length,1);assert.match(loud.reasons[0],/other container mw-shots-greenmail used 0.2 CPU/);
 const busy=vmWitness(samples({outsideBusyPerSecond:15}),window,{plan});
 assert.match(busy.reasons[0],/outside the run's cpusets were 15.0% busy/);
 // This run's own containers are never "other".
 assert.deepEqual(vmWitness(samples({other:{cccccccccccc:3}}),window,{plan,runContainerIds:['cccccccccccc0000']}).reasons,[]);
 // A container that starts inside the window counts from zero.
 const late=samples();for(const s of late.slice(5))s.others.dddddddddddd={usage_usec:1e6*(s.tMs/1000-5)};
 const lateVerdict=vmWitness(late,window,{plan});
 assert.equal(lateVerdict.otherContainers.top[0].appeared,true);assert.equal(lateVerdict.otherContainers.top[0].cpus,0.6);
 assert.equal(lateVerdict.reasons.length,1);
 // No /proc/stat: the outside cannot be shown quiet.
 assert.match(vmWitness(samples().map(s=>({...s,procStat:null})),window,{plan}).reasons[0],/cannot be shown quiet/);
});

test('Windows host witness: builds not paused and busy host processes contaminate; VM processes do not count',()=>{
 const host=(rows)=>Array.from({length:7},(_,i)=>({tMs:i*2000,processes:Object.fromEntries(rows.map(([pid,name,cps])=>[pid,{name,cpuSeconds:cps*i*2}]))}));
 const quiet=hostWitness(host([['1','vmmemWSL',20],['2','explorer',0.01],['3','powershell',0.5]]),window,{hostLogicalCpus:80,excludePids:[3]});
 assert.deepEqual(quiet.reasons,[]);assert.equal(quiet.otherCpus,0.01);
 const build=hostWitness(host([['4','cargo',0.3],['5','rustc',4]]),window,{hostLogicalCpus:80});
 assert.equal(build.reasons.length,1);assert.match(build.reasons[0],/builds were not paused/);
 const busy=hostWitness(host([['6','someapp',9]]),window,{hostLogicalCpus:80});
 assert.match(busy.reasons[0],/11.3% of 80 logical CPUs/);
 assert.equal(hostWitness([],window,{hostLogicalCpus:80}).sampled,false);
});

test('stage verdict combines both sides and names the rule; no window means contaminated',()=>{
 const verdict=stageQuiet({},{window,samples:samples({other:{bbbbbbbbbbbb:1}}),hostSamples:[],plan});
 assert.equal(verdict.contaminated,true);assert.match(verdict.rule,/section 8 rule 1/);assert.equal(verdict.thresholds,DEFAULT_THRESHOLDS);
 assert.equal(stageQuiet({},{window:null,samples:[],plan}).contaminated,true);
 assert.equal(stageQuiet({},{window,samples:samples(),hostSamples:[],plan}).contaminated,false);
});
