// Fixture-only, bounded, credential-free cache experiment evidence. No I/O.
const MAX=32,MAX_COUNTER=2147483647;
const integer=(value,min,max)=>Number.isSafeInteger(value)&&value>=min&&value<=max;
const fail=()=>{throw new Error('Invalid SQLite page-cache witness');};
const numeric=(value,min,max)=>{if(!integer(value,min,max))fail();return value;};
const optional=(value,min,max)=>value===null?null:numeric(value,min,max);
const flag=value=>{if(typeof value!=='boolean')fail();return value;};
const fields=['cacheUsedBytes','cacheHits','cacheMisses','cacheWrites','cacheSpills'];
function connection(value,{audit=false}={}){
 if(!value||typeof value!=='object')fail();
 const out={connectionEpoch:optional(value.connectionEpoch,1,Number.MAX_SAFE_INTEGER)};
 for(const key of fields)out[key]=optional(value[key],0,MAX_COUNTER);
 if(audit)out.mainCacheSize=numeric(value.mainCacheSize,-2147483648,2147483647);
 return out;
}
function sample(value,{audit=false,maximum=MAX}={}){
 if(!value||!Array.isArray(value.connections)||value.connections.length>MAX)fail();
 const openBefore=numeric(value.openBefore,0,maximum),openAfter=numeric(value.openAfter,0,maximum);
 const sampledConnections=numeric(value.sampledConnections,0,MAX);
 if(sampledConnections!==value.connections.length||sampledConnections>openBefore)fail();
 const connections=value.connections.map(row=>connection(row,{audit}));
 const known=connections.map(row=>row.connectionEpoch).filter(n=>n!==null);
 if(new Set(known).size!==known.length)fail();
 const completeIdleCoverage=flag(value.completeIdleCoverage),deadlineReached=flag(value.deadlineReached);
 const failedConnections=audit?numeric(value.failedConnections,0,MAX):0;
 if(sampledConnections+failedConnections>openBefore||completeIdleCoverage&&(deadlineReached||failedConnections!==0||sampledConnections!==openBefore||openAfter!==openBefore))fail();
 return {openBefore,openAfter,sampledConnections,completeIdleCoverage,deadlineReached,...(audit?{failedConnections}:{}),connections};
}
function targets(value){
 if(!value||typeof value!=='object')fail();
 const aggregateMiB=optional(value.aggregateMiB,16,1024);
 const plannedMaximumConnections=numeric(value.plannedMaximumConnections,1,MAX);
 const perConnectionKiB=optional(value.perConnectionKiB,1,1024*1024);
 const auxiliaryConnectionKiB=optional(value.auxiliaryConnectionKiB,1,2000);
 const driverDefaultUnchanged=flag(value.driverDefaultUnchanged);
 if(value.hardMemoryLimit!==false||driverDefaultUnchanged!==(aggregateMiB===null))fail();
 // Metadata can describe a failed startup/restored old pool. Keep unknown
 // shares reportable; experiment validation below must reject that evidence.
 if(aggregateMiB===null&&(perConnectionKiB!==null||auxiliaryConnectionKiB!==null))fail();
 return {aggregateMiB,plannedMaximumConnections,perConnectionKiB,auxiliaryConnectionKiB,driverDefaultUnchanged,hardMemoryLimit:false};
}
export function sqlitePageCacheReport(value){
 if(value==null)return null;
 const out=targets(value);
 const plannedForegroundBytes=optional(value.plannedForegroundBytes,0,1024*1024*1024);
 if(plannedForegroundBytes!==(out.perConnectionKiB===null?null:out.perConnectionKiB*1024*out.plannedMaximumConnections))fail();
 return {...out,plannedForegroundBytes,restartRequired:flag(value.restartRequired),idleSample:sample(value.idleSample,{maximum:out.plannedMaximumConnections})};
}
export function sqlitePageCacheAuditReport(value){
 if(value==null)return null;
 if(value.schema!==1||value.scope!=='foreground-pool-main-target'||value.generationPinned!==true||value.deadlineMs!==25||value.deadlineIsCooperative!==true||value.counterScope!=='all-attached-pagers-per-handle')fail();
 const out=targets(value);
 return {schema:1,scope:'foreground-pool-main-target',generationPinned:true,...out,deadlineMs:25,deadlineIsCooperative:true,counterScope:'all-attached-pagers-per-handle',sample:sample(value.sample,{audit:true,maximum:out.plannedMaximumConnections})};
}
export function cacheExperimentReport(value){
 if(value==null)return null;
 if(value.kind!=='sqlite-page-cache'||!(value.requestedAggregateMiB===null||integer(value.requestedAggregateMiB,16,1024)))fail();
 return {kind:'sqlite-page-cache',requestedAggregateMiB:value.requestedAggregateMiB};
}
export function cacheExperimentFromEnv(value){
 if(value===undefined)return null;
 if(value==='null')return {kind:'sqlite-page-cache',requestedAggregateMiB:null};
 if(typeof value!=='string'||!/^([1-9][0-9]*)$/.test(value))fail();
 return cacheExperimentReport({kind:'sqlite-page-cache',requestedAggregateMiB:Number(value)});
}
export function campaignCacheExperiment(handoff,options){
 const offered=cacheExperimentReport(handoff.sqliteCacheExperiment),requested=cacheExperimentReport(options.sqliteCacheExperiment);
 if(!offered&&!requested)return null;
 if(handoff.engineMode!=='native'||handoff.backend!=='sqlite'||!offered||!requested||offered.requestedAggregateMiB!==requested.requestedAggregateMiB)fail();
 return requested;
}
export function assertSqliteCacheWitness(snapshot,experiment){
 const expected=cacheExperimentReport(experiment);if(!expected)fail();
 const audit=sqlitePageCacheAuditReport(snapshot?.sqlitePageCacheAudit);
 const report=sqlitePageCacheReport(snapshot?.pool?.sqlitePageCache);
 if(!audit||!report||snapshot.pool.maximum!==audit.plannedMaximumConnections)fail();
 for(const key of ['aggregateMiB','plannedMaximumConnections','perConnectionKiB','auxiliaryConnectionKiB','driverDefaultUnchanged'])if(audit[key]!==report[key])fail();
 if(audit.aggregateMiB!==expected.requestedAggregateMiB)fail();
 const observed=audit.sample;
 if(!observed.completeIdleCoverage||observed.sampledConnections===0||observed.failedConnections||observed.deadlineReached||observed.connections.some(row=>row.connectionEpoch===null||fields.some(key=>row[key]===null)))fail();
 if(expected.requestedAggregateMiB!==null){
  const share=Math.floor(expected.requestedAggregateMiB*1024/audit.plannedMaximumConnections);
  if(audit.perConnectionKiB!==share||audit.auxiliaryConnectionKiB!==2000||observed.connections.some(row=>row.mainCacheSize!==-share))fail();
 }
 return audit;
}
export function cacheExperimentEvidence(experiment,stages){
 if(!experiment)return null;
 if(!Array.isArray(stages)||stages.length<1||stages.length>256)fail();
 for(const stage of stages)for(const phase of ['before','after'])assertSqliteCacheWitness(stage.diagnostics?.[phase],experiment);
 return {...cacheExperimentReport(experiment),verified:true,scope:'foreground-pool-main-target',coverage:'all open serving handles at each boundary; not necessarily the maximum pool size',deadlineIsCooperative:true,counterScope:'all-attached-pagers-per-handle',auxiliaryCountersMeasured:false,hardMemoryLimit:false};
}
