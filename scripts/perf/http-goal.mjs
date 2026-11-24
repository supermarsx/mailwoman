// The target a run is judged against, and the network-budget rule. Defaults are
// the standing goal; anything else is recorded in the report (first key and
// first Markdown line) so a report cannot be mistaken for the standing claim.
export const DEFAULT_TARGET_RPS=40000;
export const DEFAULT_TARGET_USERS=10000;

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
 if(qualificationRun&&networkGbit===undefined)throw new Error('A qualification run needs MW_LOAD_NETWORK_GBIT');
 return {successfulRps,simultaneousAuthenticatedUsers,...(networkGbit===undefined?{}:{networkGbit}),qualificationRun,
  isDefault:successfulRps===DEFAULT_TARGET_RPS&&simultaneousAuthenticatedUsers===DEFAULT_TARGET_USERS};
}
// Options carry a goal only when something was set, so default option objects are unchanged.
export function goalOption(env){
 const goal=goalFromEnv(env);
 return goal.isDefault&&goal.networkGbit===undefined&&!goal.qualificationRun?{}:{goal};
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
// Loopback or a shared network namespace carries no network claim.
export function networkClaim(goal,{loopback}){
 if(goal.networkGbit===undefined&&!goal.qualificationRun&&!loopback)return {declared:false,qualifies:false,reason:'no network budget declared (MW_LOAD_NETWORK_GBIT)'};
 if(loopback)return {declared:goal.networkGbit!==undefined,budgetGbit:goal.networkGbit,qualifies:false,reason:'loopback or shared network namespace run: it cannot qualify a network claim'};
 return {declared:true,budgetGbit:goal.networkGbit,qualifies:goal.networkGbit!==undefined,reason:goal.networkGbit!==undefined?'stage body bytes per second must stay within the declared budget':'no network budget declared (MW_LOAD_NETWORK_GBIT)'};
}
export function judgedAgainst(goal,{loopback}){
 return {successfulRps:goal.successfulRps,simultaneousAuthenticatedUsers:goal.simultaneousAuthenticatedUsers,default:goal.isDefault,
  ...(goal.isDefault?{}:{note:`NON-DEFAULT TARGET: judged against ${goal.successfulRps} requests/s and ${goal.simultaneousAuthenticatedUsers} users, not the standing ${DEFAULT_TARGET_RPS} / ${DEFAULT_TARGET_USERS}`}),
  network:networkClaim(goal,{loopback})};
}
// A qualification run on loopback can never qualify.
export function qualificationBlocked(goal,{loopback}){return goal.qualificationRun&&loopback;}
