// Bench rig (t62-n12): the run definitions for plan runs R0a and R1 (t62-100k section 8), the per-stage database
// and upstream counters read from the fixture diagnostics, and the summary table: per shape statements
// (reads/writes), commits, upstream commands, µs CPU and p50/p99 — the "today" columns of plan sections 3.3 and 4.2.
import {GOAL_MIX_TABLE,GOAL_MIX_NAME,upstreamReport} from './representative-workload.mjs';
import {parseT34Selection} from './t34-modes.mjs';
import {stageCpuCost,stageWindow} from './rig-cgroup.mjs';
import {stageQuiet} from './rig-quiet.mjs';
import {stageMemory} from './rig-memory.mjs';

export const TRANSPORT_ENDPOINT='transport-healthz';
export const SHAPE_PREFIX='goal-shape-';
export const TRANSPORT_LABEL='transport control only: GET /healthz bypasses the application guards';

// Server modes of R1 (plan section 8): guard snapshot bounded, write-behind accounting, SQLite write gate.
export const R1_MODES=Object.freeze({MW_GUARD_SNAPSHOT:'bounded',MW_INDEX_PUBLICATION_ACK:'commit',MW_REQUEST_PROTECTION_ACCOUNTING:'write-behind',
 MW_NATIVE_LOAD_REQUEST_INPUTS_MODE:'published',MW_LOAD_FIXTURE_REQUEST_INPUTS_EXPECT:'published'});

// Fixture environment shared by every leg (no secrets: the database key is added by the runner, privately).
function fixtureEnv({principals,messages,workload,workers,seconds,warmupSeconds,concurrency,endpoints,deadlineSeconds,requestProfile,backend}){
 return {
  MW_DB_ENCRYPTION:'required',MW_NATIVE_LOAD_PRINCIPALS:String(principals),MW_NATIVE_LOAD_WORKLOAD:workload,MW_NATIVE_LOAD_MESSAGES:String(messages),
  MW_LOAD_FIXTURE_EXTERNAL_DRIVER:'1',MW_LOAD_FIXTURE_EXTERNAL_ROOT:'/scratch/control',MW_LOAD_REPORT_DIR:'/scratch/reports',
  MW_LOAD_FIXTURE_MODE:'ramp',MW_LOAD_FIXTURE_LONG:'1',MW_LOAD_FIXTURE_SECONDS:String(seconds),MW_LOAD_FIXTURE_WARMUP_SECONDS:String(warmupSeconds),
  MW_LOAD_FIXTURE_DEADLINE_SECONDS:String(deadlineSeconds),MW_LOAD_FIXTURE_CONCURRENCY:concurrency,MW_LOAD_FIXTURE_ENDPOINTS:endpoints,
  // One closed-loop stage per endpoint and concurrency: no plateau stop between the listed concurrencies.
  MW_LOAD_FIXTURE_ADAPTIVE:'0',
  MW_LOAD_FIXTURE_WORKERS:String(workers),MW_NATIVE_LOAD_POLICY:'loaded',MW_FIXTURE_REQUEST_PROFILE:requestProfile?'1':'0',
  MW_NATIVE_LOAD_INDEX_TIMING:'0',MW_NATIVE_LOAD_INDEX_WORKERS:'3',MW_NATIVE_LOAD_INDEX_MODE:'sealed',MW_NATIVE_LOAD_SECURITY_PROFILE:'enforced',
  ...(backend==='postgres'?{MW_LOAD_FIXTURE_BACKEND:'postgres',MW_LOAD_FIXTURE_PG_ACK_SCHEMA:'1'}:{MW_SQLITE_WRITER:'gate'}),
 };
}

// A run is a list of legs; each leg is one fixture process (seeded once) plus one generator campaign.
// image: 'release' (ordinary fixture image) or 'diagnostics' (built with MW_FIXTURE_DIAGNOSTICS=1, i.e. the
// sql-shape-diagnostics feature; its numbers are counts only, never throughput).
export function runDefinition(name,{backend='sqlite',appCpus=4,transportConcurrency='1,4,8',principals,smoke=false}={}){
 if(!['sqlite','postgres'].includes(backend))throw new Error('Invalid backend');
 if(!Number.isInteger(appCpus)||appCpus<1||appCpus>128)throw new Error('Invalid app CPU count');
 const workers=appCpus;
 const scale=smoke?{seconds:5,warmup:1,deadline:300}:null;
 if(name==='r0a'){
  return {name,description:'R0a transport floor: GET /healthz only (labelled: bypasses the application guards; transport control only)',legs:[{
   id:'transport',image:'release',instrumented:false,
   env:fixtureEnv({principals:principals??100,messages:2,workload:'baseline',workers,seconds:scale?.seconds??30,warmupSeconds:scale?.warmup??5,
    concurrency:transportConcurrency,endpoints:TRANSPORT_ENDPOINT,deadlineSeconds:scale?.deadline??900,requestProfile:false,backend}),
   note:`${TRANSPORT_LABEL}. /healthz has its own admission pool of 8 permits with no wait (t57-e1), so concurrency above 8 measures refusals; the default list stays at or below 8.`}]};
 }
 if(name==='r1'){
  // Write-behind accounting is SQLite-only (PostgreSQL stays synchronous, t50hp-e8).
  const modes={...R1_MODES,...(backend==='postgres'?{MW_REQUEST_PROTECTION_ACCOUNTING:'synchronous'}:{})};
  parseT34Selection(modes);
  const base={principals:principals??1000,messages:50,workload:GOAL_MIX_NAME,workers,backend};
  return {name,description:'R1 per-shape cost of the goal mix: each of the 17 shapes alone, then the mix; plain release leg and sql-shape-diagnostics leg',legs:[
   {id:'release-shapes',image:'release',instrumented:false,env:{...fixtureEnv({...base,seconds:scale?.seconds??30,warmupSeconds:scale?.warmup??10,concurrency:'32',endpoints:'goal-shapes',deadlineSeconds:scale?.deadline??3600,requestProfile:false}),...modes}},
   {id:'release-mix',image:'release',instrumented:false,env:{...fixtureEnv({...base,seconds:scale?.seconds??60,warmupSeconds:scale?.warmup??10,concurrency:'64',endpoints:GOAL_MIX_NAME,deadlineSeconds:scale?.deadline??900,requestProfile:false}),...modes}},
   {id:'diagnostics-shapes',image:'diagnostics',instrumented:true,env:{...fixtureEnv({...base,seconds:scale?.seconds??10,warmupSeconds:scale?.warmup??5,concurrency:'32',endpoints:'goal-shapes',deadlineSeconds:scale?.deadline??3600,requestProfile:true}),...modes},
    note:'instrumented: sql-shape-diagnostics build with the request profile on; statement and commit counts are request-scoped; its CPU and latency are not throughput numbers'},
  ]};
 }
 throw new Error('Unknown run; use r0a or r1');
}

const delta=(after,before)=>Number.isFinite(after)&&Number.isFinite(before)&&after>=before?after-before:null;
// Database counters of one stage. Request-scoped when the request profile was on (requestProfile.queries, reset
// before each stage, covering exactly the stage's HTTP requests); otherwise process-wide (pool.operations), which
// includes background work and the drain after the window.
export function stageCounters(stage){
 const before=stage?.diagnostics?.before,after=stage?.diagnostics?.after;
 const profiled=after?.requestProfile?.enabled===true&&before?.requestProfile?.enabled===true&&after.requestProfile.queries;
 const source=profiled?'request-profile':'process';
 // The request profile is reset before each stage, so its "before" is zero.
 const a=profiled?after.requestProfile.queries:after?.pool?.operations,b=profiled?null:before?.pool?.operations;
 const requests=Number.isSafeInteger(stage?.requests)?stage.requests:null;
 if(!a||(!profiled&&!b)||!requests)return {available:false,reason:'stage diagnostics lack database counters'};
 const d=(path)=>{const read=o=>path.reduce((v,k)=>v?.[k],o);return profiled?(Number.isFinite(read(a))?read(a):null):delta(read(a),read(b));};
 const reads=d(['read','calls']),writes=d(['write','calls']),other=d(['other','calls']);
 const txSqlite=d(['commits','sqlite','calls']),txPostgres=d(['commits','postgres','calls']),autocommit=d(['autocommitWrites']);
 const commitsKnown=txSqlite!==null&&txPostgres!==null&&autocommit!==null;
 const per=value=>value===null?null:Number((value/requests).toFixed(3));
 const upstream=upstreamReport(before?.upstreamCommands,after?.upstreamCommands,{seconds:stage.scheduledSeconds??1,writeRequests:requests});
 return {available:true,source,requests,
  basis:profiled?'request-scoped q() counters of the stage\'s HTTP requests (request profile on)':'process-wide q() counters between the stage\'s before/after diagnostics; include background work and the drain after the window',
  totals:{reads,writes,other,transactionCommits:commitsKnown?txSqlite+txPostgres:null,autocommitWrites:autocommit,databaseCommits:commitsKnown?txSqlite+txPostgres+autocommit:null,upstreamCommands:upstream.available?upstream.commands:null},
  perRequest:{reads:per(reads),writes:per(writes),other:per(other),statements:reads!==null&&writes!==null&&other!==null?per(reads+writes+other):null,
   transactionCommits:commitsKnown?per(txSqlite+txPostgres):null,autocommitWrites:per(autocommit),databaseCommits:commitsKnown?per(txSqlite+txPostgres+autocommit):null,
   upstreamCommands:upstream.available?per(upstream.commands):null,upstreamWriteCommands:upstream.available?per(upstream.writeCommands):null},
  commitsAvailable:commitsKnown,commitsNote:commitsKnown?undefined:'the fixture build predates the commit counter (t62-n12)'};
}

export function shapeOf(endpoint){
 if(endpoint===TRANSPORT_ENDPOINT)return {name:TRANSPORT_ENDPOINT,kind:'transport',weight:null};
 if(endpoint===GOAL_MIX_NAME)return {name:GOAL_MIX_NAME,kind:'mix',weight:100};
 if(typeof endpoint==='string'&&endpoint.startsWith(SHAPE_PREFIX)){const row=GOAL_MIX_TABLE.find(r=>r.name===endpoint.slice(SHAPE_PREFIX.length));if(row)return {name:row.name,kind:row.kind,weight:row.weight};}
 return {name:String(endpoint),kind:'other',weight:null};
}

// One row per measured stage of one leg.
export function legRows(report,{leg,samples,hostSamples,plan,labels,runContainerIds,hostLogicalCpus,excludePids,thresholds,intervalMs,hostIntervalMs}){
 return (report?.stages??[]).map(stage=>{
  const shape=shapeOf(stage.endpoint),window=stageWindow(stage);
  const cpu=stageCpuCost(stage,samples,{plan,labels,intervalMs});
  const quiet=stageQuiet(stage,{window,samples,hostSamples,plan,runContainerIds,hostLogicalCpus,excludePids,thresholds,intervalMs,hostIntervalMs});
  const generatorLabel=Object.entries(labels).find(([,role])=>role==='generator')?.[0];
  return {leg:leg.id,instrumented:leg.instrumented,shape:shape.name,kind:shape.kind,weight:shape.weight,endpoint:stage.endpoint,concurrency:stage.concurrency??null,
   phase:stage.phase??null,requests:stage.requests??null,acceptedInWindow:stage.acceptedDuringScheduledWindow??null,acceptedRps:Number.isFinite(stage.acceptedDuringScheduledWindowRps)?Number(stage.acceptedDuringScheduledWindowRps.toFixed(1)):null,
   errors:stage.errors??null,throttled:stage.throttled??null,timeouts:stage.timeouts??null,http503:Number.isSafeInteger(stage.statuses?.['503'])?stage.statuses['503']:(stage.statuses?0:null),
   latencyMs:{p50:stage.latencyMs?.p50??null,p99:stage.latencyMs?.p99??null},
   database:stageCounters(stage),cpu,
   usecCpuPerRequest:{server:cpu.server?.usecCpuPerRequest??null,generator:generatorLabel?cpu.containers?.[generatorLabel]?.usecCpuPerRequest??null:null},
   rpsPerLogicalCpu:cpu.rpsPerLogicalCpu??null,
   memory:stageMemory(stage,samples,{labels,window}),
   quiet,contaminated:quiet.contaminated,
   ...(shape.kind==='transport'?{label:TRANSPORT_LABEL}:{}),
   ...(leg.instrumented?{label:'instrumented (sql-shape-diagnostics, request profile on): counts only; CPU and latency are not throughput numbers'}:{})};
 });
}

// Weighted µs CPU per request of the mix from the per-shape rows (sum of weight × µs / 100), next to the measured mix.
export function weightedMix(rows){
 const shapes=rows.filter(row=>!row.instrumented&&(row.kind==='read'||row.kind==='write')&&!row.contaminated);
 const pick=name=>shapes.find(row=>row.shape===name);
 const missing=GOAL_MIX_TABLE.filter(row=>!Number.isFinite(pick(row.name)?.usecCpuPerRequest?.server));
 const weighted=missing.length?null:GOAL_MIX_TABLE.reduce((n,row)=>n+row.weight*pick(row.name).usecCpuPerRequest.server,0)/100;
 const mix=rows.find(row=>row.kind==='mix'&&!row.instrumented);
 return {weightedServerUsecPerRequest:weighted===null?null:Number(weighted.toFixed(1)),missingShapes:missing.map(row=>row.name),
  measuredMix:mix?{usecCpuPerRequest:mix.usecCpuPerRequest.server,acceptedRps:mix.acceptedRps,contaminated:mix.contaminated}:null,
  basis:'weights from GOAL_MIX_TABLE (per 100 requests); uncontaminated plain-release stages only'};
}

const mib=v=>Number.isFinite(v)?(v/1048576).toFixed(1):null;
const appMem=row=>Object.values(row.memory?.containers??{}).find(c=>c.role==='app');
const memCell=row=>{const m=appMem(row);return m?.available?`${mib(m.currentEndBytes)} (${mib(m.maxSampledBytes)??'–'})`:'–';};
const eventsCell=row=>{const m=appMem(row);return m?.available?`${m.events.oom??'–'}/${m.events.max??'–'}`:'–';};
const cell=value=>value===null||value===undefined?'–':String(value);
export function markdownSummary(summary){
 const lines=[`# ${summary.run.toUpperCase()} summary (${summary.backend})`,'',summary.description,'',
  `Host: ${summary.host.vmVcpus} VM vCPUs; cpusets ${Object.entries(summary.cpusets).map(([role,set])=>`${role} ${set}`).join(', ')}. VM vCPUs are not pinned to physical cores.`,
  `Fixture image runtime: ${summary.imageRuntime?`${summary.imageRuntime.libc}; allocator ${summary.imageRuntime.allocator}`:'not recorded'}.`,''];
 lines.push('| Leg | Shape | Kind | W | C | Requests | Reads/req | Writes/req | Other/req | Commits/req (tx + autocommit) | Upstream cmds/req | Server µs CPU/req | Generator µs CPU/req | rps | rps/allotted server CPU | p50 ms | p99 ms | Errors | App MiB end (max) | App bytes/user | OOM/max events | Quiet |');
 lines.push('| --- | --- | --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | --- |');
 for(const row of summary.rows){
  const p=row.database.available?row.database.perRequest:{};
  const commits=row.database.available&&row.database.commitsAvailable?`${p.databaseCommits} (${p.transactionCommits} + ${p.autocommitWrites})`:null;
  const fixed=v=>Number.isFinite(v)?v.toFixed(2):null;
  lines.push(`| ${row.leg} | ${row.shape} | ${row.kind} | ${cell(row.weight)} | ${cell(row.concurrency)} | ${cell(row.requests)} | ${cell(p.reads)} | ${cell(p.writes)} | ${cell(p.other)} | ${cell(commits)} | ${cell(p.upstreamCommands)} | ${cell(row.usecCpuPerRequest.server)} | ${cell(row.usecCpuPerRequest.generator)} | ${cell(row.acceptedRps)} | ${cell(row.rpsPerLogicalCpu?.allottedServer)} | ${cell(fixed(row.latencyMs.p50))} | ${cell(fixed(row.latencyMs.p99))} | ${cell((row.errors??0)+(row.throttled??0)+(row.timeouts??0))} | ${memCell(row)} | ${cell(row.memory?.derived?.appBytesPerConnectedUser)} | ${eventsCell(row)} | ${row.contaminated?'CONTAMINATED: '+row.quiet.reasons.join('; '):'quiet'} |`);
 }
 lines.push('');
 if(summary.mix)lines.push(`Weighted server µs CPU per request from the shapes: ${cell(summary.mix.weightedServerUsecPerRequest)}${summary.mix.missingShapes.length?` (missing: ${summary.mix.missingShapes.join(', ')})`:''}; measured mix: ${cell(summary.mix.measuredMix?.usecCpuPerRequest)} µs at ${cell(summary.mix.measuredMix?.acceptedRps)} rps.`,'');
 lines.push('Columns: statements per HTTP request by class (reads = SELECT; writes = INSERT/UPDATE/DELETE/REPLACE; other = BEGIN, PRAGMA and the rest); commits = transaction commits plus autocommit write statements; upstream = commands the in-process upstream fake received; µs CPU from cgroup cpu.stat over the scheduled window divided by requests accepted in it; rps per allotted server CPU = accepted rps / logical CPUs in the server cpusets; p50/p99 from the generator.');
 lines.push('Counts in plain-release legs are process-wide (background work included); counts in the instrumented leg are request-scoped. Instrumented rows are counts only.');
 for(const note of summary.notes??[])lines.push('',note);
 return lines.join('\n')+'\n';
}
