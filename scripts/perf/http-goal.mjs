// The target a run is judged against, and the network-budget rule. Defaults are
// the standing goal; anything else is recorded in the report (first key and
// first Markdown line) so a report cannot be mistaken for the standing claim.
//
// Standing goal, revised again on 2026-11-24 by the user: 100,000 successful requests per
// second on a balanced mix of about 65% reads and 35% writes (by request count),
// this machine being the base test hardware. Earlier text said 30,000, and before that 40,000.
import os from 'node:os';

export const DEFAULT_TARGET_RPS=100000;
export const DEFAULT_TARGET_USERS=10000;
export const GOAL_WORKLOAD='about 65% reads and 35% writes by HTTP request count (goal-65-35)';

function bounded(env,key,fallback,min,max,integer){
 if(env[key]===undefined||env[key]==='')return fallback;
 const value=Number(env[key]);
 if(!Number.isFinite(value)||value<min||value>max||(integer&&!Number.isInteger(value)))throw new Error(`Invalid ${key}`);
 return value;
}
export function goalFromEnv(env){
 const successfulRps=bounded(env,'MW_LOAD_TARGET_RPS',DEFAULT_TARGET_RPS,1,1000000,true);
 const simultaneousAuthenticatedUsers=bounded(env,'MW_LOAD_TARGET_USERS',DEFAULT_TARGET_USERS,1,10000,true);
 const networkGbit=bounded(env,'MW_LOAD_NETWORK_GBIT',undefined,0.001,1000,false);
 const qualificationRun=env.MW_LOAD_QUALIFICATION_RUN==='1';
 const host=env.MW_LOAD_QUALIFICATION_HOST;
 if(host!==undefined&&host!==''&&host!=='local')throw new Error('Invalid MW_LOAD_QUALIFICATION_HOST (only local is defined)');
 const qualificationHost=host==='local'?'local':undefined;
 // On the designated base hardware the run is loopback by construction; the network budget is then optional.
 if(qualificationRun&&networkGbit===undefined&&qualificationHost!=='local')throw new Error('A qualification run needs MW_LOAD_NETWORK_GBIT');
 return {successfulRps,simultaneousAuthenticatedUsers,...(networkGbit===undefined?{}:{networkGbit}),qualificationRun,...(qualificationHost?{qualificationHost}:{}),
  isDefault:successfulRps===DEFAULT_TARGET_RPS&&simultaneousAuthenticatedUsers===DEFAULT_TARGET_USERS};
}
// Options carry a goal only when something was set, so default option objects are unchanged.
export function goalOption(env){
 const goal=goalFromEnv(env);
 return goal.isDefault&&goal.networkGbit===undefined&&!goal.qualificationRun&&!goal.qualificationHost?{}:{goal};
}
export function goalOf(options){
 const goal=options?.goal;
 return goal??{successfulRps:DEFAULT_TARGET_RPS,simultaneousAuthenticatedUsers:DEFAULT_TARGET_USERS,qualificationRun:false,isDefault:true};
}
// Body bytes both ways, converted to Gbit/s. Headers, TLS and framing are not counted, so this understates the wire.
export function stageNetworkGbit(stage){
 const transfer=stage?.transfer;
 return transfer&&Number.isFinite(transfer.networkGbitBodyBytes)?transfer.networkGbitBodyBytes:null;
}
export function withinNetworkBudget(stage,goal){
 if(goal.networkGbit===undefined)return true;
 const used=stageNetworkGbit(stage);
 return used!==null&&used<=goal.networkGbit;
}
// The mix stage of the goal: accepted writes must be about 35% of accepted requests (30-40%).
export function goalMixShareOk(stage){
 const share=stage?.mix?.acceptedInWindow?.writeShare;
 return Number.isFinite(share)&&share>=0.30&&share<=0.40;
}
// Loopback or a shared network namespace carries no network claim, whatever the host option says.
export function networkClaim(goal,{loopback}){
 if(loopback)return {declared:goal.networkGbit!==undefined,budgetGbit:goal.networkGbit,qualifies:false,reason:'loopback or shared network namespace run: it cannot qualify a network claim'+(goal.qualificationHost==='local'?' (throughput on the designated base hardware may still qualify)':'')};
 if(goal.networkGbit===undefined)return {declared:false,qualifies:false,reason:'no network budget declared (MW_LOAD_NETWORK_GBIT)'};
 return {declared:true,budgetGbit:goal.networkGbit,qualifies:true,reason:'stage body bytes per second must stay within the declared budget'};
}
const cpuset=value=>typeof value==='string'&&/^[0-9][0-9,-]{0,63}$/.test(value)?value:null;
// Facts about the host the run was judged on. Cores come from the OS; the Docker VM size and the cpusets are
// declared by the operator (Node cannot see them) and are recorded as declared, or null when not declared.
export function hostFacts(env=process.env,system=os){
 const vcpus=Number(env.MW_LOAD_HOST_DOCKER_VCPUS);
 return {logicalCores:system.availableParallelism?.()??system.cpus().length,platform:system.platform(),totalMemoryBytes:system.totalmem(),
  dockerVmVcpus:Number.isInteger(vcpus)&&vcpus>=1&&vcpus<=1024?vcpus:null,
  cpusets:{app:cpuset(env.MW_LOAD_HOST_CPUSET_APP),database:cpuset(env.MW_LOAD_HOST_CPUSET_DB),generator:cpuset(env.MW_LOAD_HOST_CPUSET_GENERATOR)},
  declaredBy:'logical cores and memory from the operating system; Docker VM vCPUs and cpusets as declared in MW_LOAD_HOST_* (null = not declared)'};
}
export function judgedAgainst(goal,{loopback,env=process.env,system=os}){
 return {successfulRps:goal.successfulRps,simultaneousAuthenticatedUsers:goal.simultaneousAuthenticatedUsers,workload:GOAL_WORKLOAD,default:goal.isDefault,
  ...(goal.isDefault?{}:{note:`NON-DEFAULT TARGET: judged against ${goal.successfulRps} requests/s and ${goal.simultaneousAuthenticatedUsers} users, not the standing ${DEFAULT_TARGET_RPS} / ${DEFAULT_TARGET_USERS}`}),
  qualificationHost:goal.qualificationHost??null,host:hostFacts(env,system),
  network:networkClaim(goal,{loopback})};
}
// A qualification run on loopback cannot qualify unless the operator designated this machine as the base hardware.
export function qualificationBlocked(goal,{loopback}){return goal.qualificationRun&&loopback&&goal.qualificationHost!=='local';}
