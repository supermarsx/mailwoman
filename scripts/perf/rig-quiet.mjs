// Bench rig (t62-n12): the "quiet host" witness. For each stage it measures what else used CPU while the stage ran
// and marks the stage `contaminated` instead of letting it be averaged (t62-100k plan, section 8 rule 1):
//  - another container (any container that is not one of this run's) above 5% of one CPU;
//  - VM vCPUs outside the run's cpusets busier than 10% of their capacity;
//  - on the Windows host (when the host sampler ran): processes other than the VM's own above 10% of the host's
//    logical CPUs, or any build/test process (cargo, rustc, linker, compiler, docker build, node) above 0.25 CPU
//    (the "builds paused" witness). The two host thresholds are this rig's reading of rule 1, which names only the
//    first two; they are options, and every report states the values used.
// Not a criterion, reported only: CPU on the run's own vCPUs that the run's containers did not account for (kernel
// softirq work for the run's own traffic is not always charged to its cgroup, so this is above zero even when quiet).
import {valueAt} from './rig-cgroup.mjs';

export const DEFAULT_THRESHOLDS=Object.freeze({otherContainerCpus:0.05,outsideCpusetShare:0.10,hostOtherShare:0.10,hostBuildCpus:0.25});
export const VM_PROCESS_NAMES=Object.freeze(['vmmem','vmmemwsl','vmwp','vmcompute','idle','system','secure system','registry','memory compression']);
export const BUILD_PROCESS=/^(cargo|rustc|rustdoc|link|lld-link|rust-lld|ld|cl|clang|clang\+\+|gcc|g\+\+|cc1|cc1plus|msbuild|docker-buildx|buildkitd|node|pnpm|npm|vitest|playwright)(\.exe)?$/i;
const USER_HZ_US=10000;// /proc/stat counts in USER_HZ (100 per second on Linux): 10,000 µs each.

const round=(value,digits=3)=>value===null||!Number.isFinite(value)?null:Number(value.toFixed(digits));

// Increase of a counter of an entity that may appear or disappear during the window: missing at the start edge
// after appearing inside the window counts from 0; missing at the end edge uses its last reading (an underestimate).
function entityDelta(samples,read,startMs,endMs,maxGapMs){
 const seen=samples.filter(sample=>Number.isFinite(read(sample)));
 if(!seen.length)return null;
 const first=seen[0],last=seen.at(-1);
 if(last.tMs<startMs||first.tMs>endMs)return null;
 let a=valueAt(samples,read,startMs,{maxGapMs});
 if(a===null)a=first.tMs>startMs?0:read(seen.filter(s=>s.tMs<=startMs).at(-1));
 let b=valueAt(samples,read,endMs,{maxGapMs});
 if(b===null)b=read(seen.filter(s=>s.tMs<=endMs).at(-1));
 return b>=a?{delta:b-a,appeared:first.tMs>startMs,disappeared:last.tMs<endMs}:null;
}

// VM side: other containers and the vCPUs outside the run. `samples` from the sampler; `plan` from rig-cpuset.
export function vmWitness(samples,{startMs,endMs},{plan,runContainerIds=[],thresholds=DEFAULT_THRESHOLDS,intervalMs=250}){
 const maxGapMs=Math.max(intervalMs*4,1000),seconds=(endMs-startMs)/1000;
 const own=new Set(runContainerIds.map(id=>String(id).slice(0,12)));
 const ids=new Set();for(const sample of samples)for(const id of Object.keys(sample.others??{}))if(!own.has(id.slice(0,12)))ids.add(id);
 const others=[];
 for(const id of ids){
  const result=entityDelta(samples,sample=>sample.others?.[id]?.usage_usec,startMs,endMs,maxGapMs);
  if(!result)continue;
  const name=samples.findLast(sample=>sample.others?.[id]?.name)?.others[id].name??null;
  others.push({id:id.slice(0,12),name,cpus:round(result.delta/1e6/seconds),appeared:result.appeared,disappeared:result.disappeared});
 }
 others.sort((a,b)=>b.cpus-a.cpus);
 const cpuDelta=(cpu,index)=>{
  const read=sample=>sample.procStat?.cpus?.[cpu]?.[index];
  const a=valueAt(samples,read,startMs,{maxGapMs}),b=valueAt(samples,read,endMs,{maxGapMs});
  return a===null||b===null||b<a?null:b-a;
 };
 const sumCpus=list=>{let busy=0,total=0;for(const cpu of list){const b=cpuDelta(String(cpu),0),t=cpuDelta(String(cpu),1);if(b===null||t===null)return null;busy+=b;total+=t;}return {busy,total};};
 const outside=plan.outsideCpus.length?sumCpus(plan.outsideCpus):{busy:0,total:0};
 const run=sumCpus(plan.usedCpus);
 const reasons=[];
 const loud=others.filter(row=>row.cpus>thresholds.otherContainerCpus);
 for(const row of loud)reasons.push(`other container ${row.name??row.id} used ${row.cpus} CPU (limit ${thresholds.otherContainerCpus})`);
 const outsideShare=outside&&outside.total?outside.busy/outside.total:outside?0:null;
 if(outsideShare===null)reasons.push('no /proc/stat samples bracketing the stage: the host outside the run cannot be shown quiet');
 else if(outsideShare>thresholds.outsideCpusetShare)reasons.push(`VM vCPUs outside the run's cpusets were ${(outsideShare*100).toFixed(1)}% busy (limit ${thresholds.outsideCpusetShare*100}%)`);
 return {otherContainers:{count:others.length,totalCpus:round(others.reduce((n,row)=>n+row.cpus,0)),top:others.slice(0,10)},
  outsideCpuset:{cpus:plan.outsideCpus.length,busyShare:round(outsideShare,4),busyCpus:outside&&outside.total?round(outside.busy*USER_HZ_US/1e6/seconds):outside?0:null},
  runCpuset:{cpus:plan.usedCpus.length,busyCpus:run?round(run.busy*USER_HZ_US/1e6/seconds):null},
  reasons};
}

// Windows host side. hostSamples: [{tMs, processes:{pid:{name,cpuSeconds}}}] already shifted to the VM clock.
export function hostWitness(hostSamples,{startMs,endMs},{hostLogicalCpus,excludePids=[],thresholds=DEFAULT_THRESHOLDS,intervalMs=2000}){
 if(!Array.isArray(hostSamples)||!hostSamples.length)return {sampled:false,reasons:[],note:'Windows host processes were not sampled: work outside the Docker VM is invisible to this stage'};
 const maxGapMs=Math.max(intervalMs*3,3000),seconds=(endMs-startMs)/1000,skip=new Set(excludePids.map(String));
 const pids=new Set();for(const sample of hostSamples)for(const pid of Object.keys(sample.processes??{}))if(!skip.has(pid))pids.add(pid);
 const rows=[];
 for(const pid of pids){
  const name=hostSamples.findLast(sample=>sample.processes?.[pid])?.processes[pid].name??'';
  if(VM_PROCESS_NAMES.includes(name.toLowerCase().replace(/\.exe$/,'')))continue;
  const result=entityDelta(hostSamples,sample=>sample.processes?.[pid]?.cpuSeconds,startMs,endMs,maxGapMs);
  if(!result||result.delta<=0)continue;
  rows.push({pid:Number(pid),name,cpus:round(result.delta/seconds),build:BUILD_PROCESS.test(name)});
 }
 rows.sort((a,b)=>b.cpus-a.cpus);
 const total=rows.reduce((n,row)=>n+row.cpus,0),build=rows.filter(row=>row.build).reduce((n,row)=>n+row.cpus,0);
 const reasons=[];
 if(Number.isInteger(hostLogicalCpus)&&hostLogicalCpus>0&&total/hostLogicalCpus>thresholds.hostOtherShare)reasons.push(`Windows processes outside the VM used ${round(total)} CPU, ${(total/hostLogicalCpus*100).toFixed(1)}% of ${hostLogicalCpus} logical CPUs (limit ${thresholds.hostOtherShare*100}%)`);
 if(build>thresholds.hostBuildCpus)reasons.push(`build or test processes on the Windows host used ${round(build)} CPU (limit ${thresholds.hostBuildCpus}): builds were not paused`);
 return {sampled:true,otherCpus:round(total),buildCpus:round(build),top:rows.slice(0,10),reasons,
  note:'per-process CPU time from Get-Process; a process that exits inside the window is counted up to its last sample'};
}

export function stageQuiet(stage,{window,samples,hostSamples,plan,runContainerIds,hostLogicalCpus,excludePids,thresholds=DEFAULT_THRESHOLDS,intervalMs,hostIntervalMs}){
 if(!window)return {contaminated:true,reasons:['stage has no scheduled window: quiet host cannot be shown'],thresholds};
 const vm=vmWitness(samples,window,{plan,runContainerIds,thresholds,intervalMs});
 const host=hostWitness(hostSamples,window,{hostLogicalCpus,excludePids,thresholds,intervalMs:hostIntervalMs});
 const reasons=[...vm.reasons,...host.reasons];
 return {contaminated:reasons.length>0,reasons,thresholds,vm,host,
  rule:'t62-100k plan section 8 rule 1: a contaminated stage is discarded, not averaged'};
}
