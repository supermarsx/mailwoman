// Bench rig (t62-n12): disjoint cpusets for the containers of one run, and the docker resource arguments and
// MW_LOAD_HOST_* declarations that record them. Pure functions; no Docker calls.
//
// Roles: app (the fixture process: HTTP server + engine + in-process upstream fake), database (PostgreSQL, when the
// backend is PostgreSQL), generator (load generator), upstream (a separate fake upstream container, when one is run),
// rig (the cgroup sampler sidecar). Every role gets its own CPUs; no CPU is in two roles.
export const ROLES=Object.freeze(['app','database','generator','upstream','rig']);
export const MAX_CPU_INDEX=1023;

// "0-3,8,10-11" -> [0,1,2,3,8,10,11]. Strict: ascending, no duplicates, no empty parts, bounded.
export function parseCpuset(text){
 if(typeof text!=='string'||!/^[0-9][0-9,-]{0,255}$/.test(text))throw new Error('Invalid cpuset');
 const cpus=[];
 for(const part of text.split(',')){
  const match=/^([0-9]+)(?:-([0-9]+))?$/.exec(part);
  if(!match)throw new Error('Invalid cpuset');
  const from=Number(match[1]),to=match[2]===undefined?from:Number(match[2]);
  if(!Number.isSafeInteger(from)||!Number.isSafeInteger(to)||from>to||to>MAX_CPU_INDEX)throw new Error('Invalid cpuset');
  for(let cpu=from;cpu<=to;cpu++){if(cpus.length&&cpu<=cpus.at(-1))throw new Error('Invalid cpuset (overlapping or unordered)');cpus.push(cpu);}
 }
 return cpus;
}
// [0,1,2,3,8,10,11] -> "0-3,8,10-11".
export function formatCpuset(cpus){
 if(!Array.isArray(cpus)||!cpus.length)throw new Error('Empty cpuset');
 const sorted=[...cpus].sort((a,b)=>a-b);
 if(sorted.some((cpu,index)=>!Number.isInteger(cpu)||cpu<0||cpu>MAX_CPU_INDEX||(index>0&&cpu===sorted[index-1])))throw new Error('Invalid cpuset');
 const parts=[];let start=sorted[0],previous=sorted[0];
 for(const cpu of sorted.slice(1).concat(Infinity)){
  if(cpu===previous+1){previous=cpu;continue;}
  parts.push(start===previous?String(start):`${start}-${previous}`);start=previous=cpu;
 }
 return parts.join(',');
}

// Explicit plan: {app:'0-3',generator:'4-7',...}. Checks every set is valid, disjoint, and inside the VM's vCPUs.
export function validatePlan(sets,{vcpus}){
 if(!Number.isInteger(vcpus)||vcpus<1||vcpus>MAX_CPU_INDEX+1)throw new Error('Invalid VM vCPU count');
 if(!sets||typeof sets!=='object')throw new Error('Invalid cpuset plan');
 const owner=new Map(),plan={};
 for(const [role,text] of Object.entries(sets)){
  if(!ROLES.includes(role))throw new Error(`Unknown cpuset role ${role}`);
  if(text===undefined||text===null)continue;
  const cpus=parseCpuset(text);
  for(const cpu of cpus){
   if(cpu>=vcpus)throw new Error(`cpuset ${role} uses CPU ${cpu}, outside the ${vcpus} VM vCPUs`);
   if(owner.has(cpu))throw new Error(`CPU ${cpu} is in both ${owner.get(cpu)} and ${role}`);
   owner.set(cpu,role);
  }
  plan[role]={cpuset:formatCpuset(cpus),cpus:cpus.length,list:cpus};
 }
 if(!plan.app||!plan.generator)throw new Error('A run needs app and generator cpusets');
 const used=[...owner.keys()].sort((a,b)=>a-b);
 const outside=Array.from({length:vcpus},(_,cpu)=>cpu).filter(cpu=>!owner.has(cpu));
 return {vcpus,roles:plan,usedCpus:used,outsideCpus:outside,outsideCpuset:outside.length?formatCpuset(outside):null};
}

// Sizes to sets: consecutive CPUs from `first` in ROLES order (app, database, generator, upstream, rig); 0 or
// missing = role not used. CPU 0 is left out by default: the VM's own kernel and Docker daemon work lands there.
export function allocate(sizes,{vcpus,first=1}){
 if(!Number.isInteger(first)||first<0)throw new Error('Invalid first CPU');
 let next=first;const sets={};
 for(const role of ROLES){
  const size=sizes?.[role]??0;
  if(!Number.isInteger(size)||size<0||size>MAX_CPU_INDEX+1)throw new Error(`Invalid ${role} CPU count`);
  if(!size)continue;
  if(next+size>vcpus)throw new Error(`Not enough VM vCPUs: ${role} needs ${next+size-vcpus} more (VM has ${vcpus})`);
  sets[role]=formatCpuset(Array.from({length:size},(_,i)=>next+i));next+=size;
 }
 return validatePlan(sets,{vcpus});
}

// Docker resource flags for one role: the cpuset, and a CPU quota equal to its size so cpu.stat throttling shows
// when the container tries to use more (it cannot: the cpuset already bounds it; the quota is recorded for clarity).
export function dockerCpuArgs(plan,role){
 const entry=plan.roles[role];
 if(!entry)throw new Error(`No cpuset for ${role}`);
 return ['--cpuset-cpus',entry.cpuset,'--cpus',String(entry.cpus)];
}

// The declarations http-goal.mjs hostFacts() records in every report (MW_LOAD_HOST_*).
export function hostEnv(plan){
 const env={MW_LOAD_HOST_DOCKER_VCPUS:String(plan.vcpus),MW_LOAD_HOST_CPUSET_APP:plan.roles.app.cpuset,MW_LOAD_HOST_CPUSET_GENERATOR:plan.roles.generator.cpuset};
 if(plan.roles.database)env.MW_LOAD_HOST_CPUSET_DB=plan.roles.database.cpuset;
 if(plan.roles.upstream)env.MW_LOAD_HOST_CPUSET_UPSTREAM=plan.roles.upstream.cpuset;
 if(plan.roles.rig)env.MW_LOAD_HOST_CPUSET_RIG=plan.roles.rig.cpuset;
 return env;
}
