// Bench rig (t62-n12): cgroup v2 cpu.stat and /proc/stat parsing, the sample series the sampler writes, and the
// per-stage CPU cost of each container: µs of CPU per request and requests/s per logical CPU. Pure functions.
//
// Source of truth is the kernel's per-cgroup counter (cpu.stat usage_usec), not docker stats. A stage's cost is
// the counter's increase over the stage's scheduled window, read by linear interpolation between the two samples
// that bracket each window edge.
export const CPU_STAT_KEYS=Object.freeze(['usage_usec','user_usec','system_usec','nr_periods','nr_throttled','throttled_usec']);
export const SAMPLE_SCHEMA=1;

// cgroup v2 cpu.stat text -> {usage_usec,...}. Unknown keys are kept; the three usage keys are required.
export function parseCpuStat(text){
 if(typeof text!=='string'||text.length>4096)throw new Error('Invalid cpu.stat');
 const out={};
 for(const line of text.split('\n')){
  const trimmed=line.trim();if(!trimmed)continue;
  const match=/^([a-z_]+) ([0-9]+)$/.exec(trimmed);
  if(!match)throw new Error('Invalid cpu.stat line');
  const value=Number(match[2]);if(!Number.isSafeInteger(value))throw new Error('Invalid cpu.stat value');
  out[match[1]]=value;
 }
 for(const key of ['usage_usec','user_usec','system_usec'])if(!Number.isSafeInteger(out[key]))throw new Error('cpu.stat lacks '+key);
 for(const key of ['nr_periods','nr_throttled','throttled_usec'])out[key]??=0;
 return out;
}

// /proc/stat text -> {cpus:{index:[busyJiffies,totalJiffies]}}. busy = user+nice+system+irq+softirq+steal
// (guest time is already inside user); total = busy + idle + iowait.
export function parseProcStat(text){
 if(typeof text!=='string'||text.length>1<<20)throw new Error('Invalid /proc/stat');
 const cpus={};
 for(const line of text.split('\n')){
  const match=/^cpu([0-9]+) +([0-9 ]+)$/.exec(line.trim());
  if(!match)continue;
  const fields=match[2].trim().split(/ +/).map(Number);
  if(fields.length<8||fields.some(n=>!Number.isSafeInteger(n)))throw new Error('Invalid /proc/stat cpu line');
  const [user,nice,system,idle,iowait,irq,softirq,steal]=fields;
  const busy=user+nice+system+irq+softirq+steal;
  cpus[match[1]]=[busy,busy+idle+iowait];
 }
 if(!Object.keys(cpus).length)throw new Error('/proc/stat has no per-CPU lines');
 return {cpus};
}

function validSample(sample){
 return sample&&sample.schema===SAMPLE_SCHEMA&&Number.isFinite(sample.tMs)&&sample.containers&&typeof sample.containers==='object';
}
// JSONL text -> samples sorted by time. Malformed lines are counted, not silently dropped.
export function parseSamples(text){
 const samples=[];let malformed=0;
 for(const line of String(text).split('\n')){
  if(!line.trim())continue;
  try{const value=JSON.parse(line);if(validSample(value))samples.push(value);else malformed++;}catch{malformed++;}
 }
 samples.sort((a,b)=>a.tMs-b.tMs);
 return {samples,malformed};
}

// Value of a cumulative counter at time t by linear interpolation between the samples that bracket t. `read`
// returns the counter from one sample (or undefined). Null when t is not bracketed by readable samples no more than
// `maxGapMs` apart, or when the counter went backwards (a restarted container).
export function valueAt(samples,read,tMs,{maxGapMs}){
 let before=null,after=null;
 for(const sample of samples){
  const value=read(sample);if(!Number.isFinite(value))continue;
  if(sample.tMs<=tMs)before={t:sample.tMs,value};
  if(sample.tMs>=tMs){after={t:sample.tMs,value};break;}
 }
 if(!before||!after||after.t-before.t>maxGapMs||after.value<before.value)return null;
 if(after.t===before.t)return before.value;
 return before.value+(after.value-before.value)*(tMs-before.t)/(after.t-before.t);
}
// Increase of a counter over [startMs,endMs]; null when either edge cannot be read.
export function deltaOver(samples,read,startMs,endMs,options){
 const a=valueAt(samples,read,startMs,options),b=valueAt(samples,read,endMs,options);
 return a===null||b===null||b<a?null:b-a;
}

// The scheduled measurement window of a stage report (coordinated controller or single-process measure()).
export function stageWindow(stage){
 const c=stage?.coordination;
 if(Number.isFinite(c?.scheduledStartAtMs)&&Number.isFinite(c?.scheduledEndAtMs))return {startMs:c.scheduledStartAtMs,endMs:c.scheduledEndAtMs};
 const w=stage?.window;
 if(Number.isFinite(w?.scheduledStartAtMs)&&Number.isFinite(w?.scheduledEndAtMs))return {startMs:w.scheduledStartAtMs,endMs:w.scheduledEndAtMs};
 return null;
}

const round=(value,digits=2)=>value===null||!Number.isFinite(value)?null:Number(value.toFixed(digits));

// CPU cost of every labelled run container over one stage.
// plan: rig-cpuset validatePlan() result (roles with cpu counts); labels map container label -> role.
// Returns {available, window, acceptedInWindow, acceptedRps, containers:{label:{...}}, server:{...}, rpsPerLogicalCpu}.
export function stageCpuCost(stage,samples,{plan,labels,serverRoles=['app','database','upstream'],intervalMs=250}){
 const window=stageWindow(stage);
 const accepted=Number.isSafeInteger(stage?.acceptedDuringScheduledWindow)?stage.acceptedDuringScheduledWindow:null;
 const seconds=Number.isFinite(stage?.scheduledSeconds)&&stage.scheduledSeconds>0?stage.scheduledSeconds:null;
 if(!window||accepted===null||!seconds)return {available:false,reason:'stage has no scheduled window or accepted count'};
 const maxGapMs=Math.max(intervalMs*4,1000),spanUs=(window.endMs-window.startMs)*1000;
 const containers={};
 for(const [label,role] of Object.entries(labels)){
  const read=key=>sample=>sample.containers?.[label]?.cpu?.[key];
  const d=key=>deltaOver(samples,read(key),window.startMs,window.endMs,{maxGapMs});
  const usage=d('usage_usec');
  if(usage===null){containers[label]={role,available:false,reason:'no bracketing cgroup samples for the stage window'};continue;}
  const user=d('user_usec'),system=d('system_usec'),periods=d('nr_periods'),throttled=d('nr_throttled'),throttledUs=d('throttled_usec');
  const allotted=plan.roles[role]?.cpus??null;
  containers[label]={role,available:true,cpuUsec:Math.round(usage),userUsec:user===null?null:Math.round(user),systemUsec:system===null?null:Math.round(system),
   coresUsed:round(usage/spanUs,3),allottedCpus:allotted,utilisationOfAllotted:allotted?round(usage/spanUs/allotted,3):null,
   usecCpuPerRequest:accepted?round(usage/accepted,1):null,
   throttle:{periods:periods===null?null:Math.round(periods),throttledPeriods:throttled===null?null:Math.round(throttled),throttledUsec:throttledUs===null?null:Math.round(throttledUs)}};
 }
 const rows=Object.values(containers);
 const server=rows.filter(row=>serverRoles.includes(row.role));
 const serverAvailable=server.length>0&&server.every(row=>row.available);
 const serverUsec=serverAvailable?server.reduce((n,row)=>n+row.cpuUsec,0):null;
 // Only roles that ran a sampled container count towards the server's logical CPUs.
 const measuredRoles=[...new Set(server.map(row=>row.role))];
 const serverAllotted=measuredRoles.reduce((n,role)=>n+(plan.roles[role]?.cpus??0),0);
 const rps=accepted/seconds;
 return {available:rows.length>0&&rows.every(row=>row.available),window,acceptedInWindow:accepted,acceptedRps:round(rps,1),
  basis:'cgroup v2 cpu.stat usage_usec increase over the scheduled window (linear interpolation between samples), divided by requests accepted inside the window',
  containers,
  server:{roles:measuredRoles,allottedCpus:serverAllotted,cpuUsec:serverUsec,coresUsed:serverUsec===null?null:round(serverUsec/spanUs,3),
   usecCpuPerRequest:serverUsec!==null&&accepted?round(serverUsec/accepted,1):null},
  rpsPerLogicalCpu:{
   allottedServer:serverAllotted?round(rps/serverAllotted,1):null,
   allottedApp:plan.roles.app?.cpus?round(rps/plan.roles.app.cpus,1):null,
   consumedServer:serverUsec?round(rps/(serverUsec/spanUs),1):null,
   note:'allotted = cpuset size (logical CPUs given to the server containers); consumed = CPU time they actually used in the window'}};
}
