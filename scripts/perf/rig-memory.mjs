// Bench rig (t62-n12): memory per stage, from the t63 memory audit (section 6). Two sources:
//  - cgroup v2 per container (sampler): memory.current at the stage's start and end edges and its largest sampled
//    value inside the window, memory.peak (lifetime), memory.events increase (high, max, oom, oom_kill),
//    memory.stat anon/file/kernel/sock, pids.current (tasks = threads; for PostgreSQL: its processes, backends
//    included);
//  - the fixture diagnostics before/after the stage: process RSS split (anonymous / file-backed / shared), threads,
//    pool open/idle, cached statements per idle connection, SQLite page-cache plan and idle sample, prepared-mail
//    charged bytes, write-behind pending, session touch buffer held/pending, search writer memory budget.
// Derived: app bytes per connected user (users accepted inside the window; the fixture has one session per user).
// Not exposed by the fixture today (so not reported): the mail cache layer's weighted size, the public asset cache.
const num=v=>Number.isFinite(v)?v:null;

// Last sample at or before t, first at or after t.
function around(samples,read,t){
 let before=null,after=null;
 for(const s of samples){const v=read(s);if(!Number.isFinite(v))continue;if(s.tMs<=t)before={t:s.tMs,v};if(s.tMs>=t){after={t:s.tMs,v};break;}}
 return {before,after};
}
// Memory of one container at startMs + offsetMs (last sample at or before it): current and anon.
export function memoryAt(samples,label,tMs){
 const current=around(samples,s=>s.containers?.[label]?.mem?.current,tMs).before?.v??null;
 const anon=around(samples,s=>s.containers?.[label]?.mem?.stat?.anon,tMs).before?.v??null;
 return {tMs,currentBytes:current,anonBytes:anon};
}
export function containerMemory(samples,label,{startMs,endMs}){
 const read=key=>s=>s.containers?.[label]?.mem?.[key];
 const readEvent=key=>s=>s.containers?.[label]?.mem?.events?.[key];
 const readStat=key=>s=>s.containers?.[label]?.mem?.stat?.[key];
 const atStart=around(samples,read('current'),startMs).before?.v??null,atEnd=around(samples,read('current'),endMs).before?.v??null;
 if(atStart===null&&atEnd===null)return {available:false,reason:'no cgroup memory samples (memory controller not readable)'};
 const inside=samples.filter(s=>s.tMs>=startMs&&s.tMs<=endMs).map(read('current')).filter(Number.isFinite);
 const events={};
 for(const key of ['high','max','oom','oom_kill']){
  const a=around(samples,readEvent(key),startMs).before?.v,b=around(samples,readEvent(key),endMs).before?.v;
  events[key]=Number.isFinite(a)&&Number.isFinite(b)?b-a:null;
 }
 const end=key=>num(around(samples,readStat(key),endMs).before?.v);
 const anonInside=samples.filter(s=>s.tMs>=startMs&&s.tMs<=endMs).map(readStat('anon')).filter(Number.isFinite);
 return {available:true,currentStartBytes:atStart,currentEndBytes:atEnd,maxSampledBytes:inside.length?Math.max(...inside):null,
  anonStartBytes:num(around(samples,readStat('anon'),startMs).before?.v),anonEndBytes:end('anon'),anonMaxSampledBytes:anonInside.length?Math.max(...anonInside):null,
  peakBytes:num(around(samples,read('peak'),endMs).before?.v),events,
  statEnd:{anon:end('anon'),file:end('file'),kernel:end('kernel'),sock:end('sock'),shmem:end('shmem')},
  pidsEnd:num(around(samples,read('pids'),endMs).before?.v)};
}

// Readable server sizes from the fixture diagnostics (after the stage, with the start value where it is a level).
export function serverSizes(stage){
 const b=stage?.diagnostics?.before,a=stage?.diagnostics?.after;
 if(!a)return {available:false};
 const pm=v=>v&&typeof v==='object'?{residentBytes:num(v.residentBytes),anonymousBytes:num(v.anonymousBytes),fileBackedBytes:num(v.fileBackedBytes),sharedMemoryBytes:num(v.sharedMemoryBytes),peakResidentBytes:num(v.peakResidentBytes),threads:num(v.threads)}:null;
 const cached=a.pool?.statementCache?.cachedPerIdleConnection;
 const touch=a.sessionTouchBuffer;
 return {available:true,
  process:{start:pm(b?.processMemory),end:pm(a.processMemory),fallbackResidentBytes:num(a.resources?.processResidentBytes)},
  pool:{open:num(a.pool?.open),idle:num(a.pool?.idle),maximum:num(a.pool?.maximum),cachedStatementsPerIdleConnection:Array.isArray(cached)?cached:null},
  sqlitePageCache:a.pool?.sqlitePageCache?{plannedForegroundBytes:num(a.pool.sqlitePageCache.plannedForegroundBytes),perConnectionKiB:num(a.pool.sqlitePageCache.perConnectionKiB),
   idleSampledConnections:num(a.pool.sqlitePageCache.idleSample?.sampledConnections)}:null,
  preparedMailChargedBytes:num(a.preparedMail?.chargedBytes),
  writeBehindPending:num(a.t34Modes?.requestProtection?.pending),
  sessionTouchBuffer:touch&&typeof touch==='object'?{held:num(touch.held),pending:num(touch.pending)}:null,
  searchWriterMemoryBudgetBytes:num(a.searchIndex?.writerConfiguration?.memoryBudgetBytes),
  notExposed:['mail cache layer weighted size','public asset cache occupancy']};
}

export function stageMemory(stage,samples,{labels,window}){
 const containers={};
 if(window)for(const [label,role] of Object.entries(labels))containers[label]={role,...containerMemory(samples,label,window)};
 const app=Object.entries(containers).find(([,c])=>c.role==='app')?.[1];
 const users=Number.isSafeInteger(stage?.uniquePrincipalsAcceptedDuringScheduledWindow)&&stage.uniquePrincipalsAcceptedDuringScheduledWindow>0?stage.uniquePrincipalsAcceptedDuringScheduledWindow:null;
 const db=Object.values(containers).find(c=>c.role==='database');
 return {containers,server:serverSizes(stage),
  derived:{connectedUsers:users,appBytesPerConnectedUser:app?.available&&users?Math.round(app.currentEndBytes/users):null,
   note:'app cgroup memory at the stage end divided by users accepted inside the window (the fixture holds one session per user)'},
  ...(db?{postgres:{processes:db.pidsEnd??null,memoryCurrentEndBytes:db.currentEndBytes??null,note:'processes = pids.current of the database container: backends plus auxiliary processes'}}:{})};
}
