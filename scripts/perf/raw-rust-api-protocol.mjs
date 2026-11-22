// Synthetic Rust/Axum transport only. Never an application/security benchmark.
import {isAbsolute} from 'node:path';
import {loopbackOrigin,aggregateCalibrationWorkers,calibrationStageBudget} from './http-cluster-calibration-protocol.mjs';
export const RAW_JSON=Buffer.from('{"ok":true}');
export function rawAuthority(value,now=Date.now()){
 if(value?.schema!==1||value.kind!=='raw-rust-axum-api'||typeof value.token!=='string'||!/^[a-f0-9]{64}$/.test(value.token)||!Number.isSafeInteger(value.expiresAtMs)||value.expiresAtMs<=now||value.expiresAtMs>now+901000||!Number.isInteger(value.runtimeWorkers)||value.runtimeWorkers<1||value.runtimeWorkers>32)throw Error('Invalid private raw Rust authority');
 const dataBase=loopbackOrigin(value.dataBase),controlBase=loopbackOrigin(value.controlBase);
 if(dataBase===controlBase)throw Error('Raw Rust listeners must be distinct');
 return {schema:1,kind:'raw-rust-axum-api',dataBase,controlBase,token:value.token,expiresAtMs:value.expiresAtMs,runtimeWorkers:value.runtimeWorkers};
}
export function rawOptions(args){
 if(Object.keys(args).some(key=>!['server','output','seconds','rates','concurrency','workers','server-workers','lifetime'].includes(key))||typeof args.server!=='string'||!isAbsolute(args.server)||typeof args.output!=='string'||!args.output)throw Error('Explicit Rust example binary and report path required');
 const seconds=Number(args.seconds??3),workers=Number(args.workers??4),serverWorkers=Number(args['server-workers']??4),concurrency=Number(args.concurrency??256),lifetime=Number(args.lifetime??120),rates=String(args.rates??'10000,20000,40000,80000,100000').split(',').map(Number);
 if(!Number.isInteger(seconds)||seconds<1||seconds>30||!Number.isInteger(workers)||workers<1||workers>16||!Number.isInteger(serverWorkers)||serverWorkers<1||serverWorkers>32||!Number.isInteger(concurrency)||concurrency<workers||concurrency>4096||!Number.isInteger(lifetime)||lifetime<10||lifetime>900||!rates.length||rates.length>5||new Set(rates).size!==rates.length||rates.some(rate=>!Number.isInteger(rate)||rate<100||rate<workers||rate>100000)||2*rates.length*(seconds+7)+15>lifetime)throw Error('Raw Rust calibration bounds exceeded');
 return {seconds,workers,serverWorkers,concurrency,lifetime,rates};
}
export function rawRequests(){return [
 {name:'raw-rust-json-11b',path:'/json',statuses:[200],validateBytes:bytes=>bytes.equals(RAW_JSON)},
 {name:'raw-rust-empty-204',path:'/empty',statuses:[204],validateBytes:bytes=>bytes.length===0},
];}
export function rawStage(value){
 if(!rawRequests().some(spec=>spec.name===value?.endpoint)||!Number.isInteger(value.id)||value.id<0||!Number.isInteger(value.seconds)||value.seconds<1||value.seconds>30||!Number.isInteger(value.concurrency)||value.concurrency<1||value.concurrency>4096||!Number.isInteger(value.rate)||value.rate<1||value.rate>100000)throw Error('Invalid raw Rust worker phase');
 return value;
}
export function rawMetrics(value,workers){
 if(value?.schema!==1||value.kind!=='raw-rust-axum-api'||value.runtimeWorkers!==workers||!Number.isSafeInteger(value.handlerInvocations)||value.handlerInvocations<0||!Number.isSafeInteger(value.handlerBytesOffered)||value.handlerBytesOffered<0||!Number.isFinite(value.elapsedSeconds)||value.elapsedSeconds<0||['cpuUserMs','cpuSystemMs','rssBytes'].some(key=>value[key]!==null))throw Error('Invalid raw Rust handler counters');
 return {handlerInvocations:value.handlerInvocations,handlerBytesOffered:value.handlerBytesOffered,elapsedSeconds:value.elapsedSeconds,runtimeWorkers:workers,cpuUserMs:null,cpuSystemMs:null,rssBytes:null,meaning:'reset through post-drain, including start lead; handler invocations and body bytes offered, not completed network responses or wire bytes; server CPU/RSS unmeasured'};
}
export function aggregateRaw(results,options){
 if(!rawRequests().some(spec=>spec.name===options.endpoint))throw Error('Unknown raw Rust payload');
 const stage=aggregateCalibrationWorkers(results,options),expected=options.endpoint==='raw-rust-empty-204'?'204':'200';
 if(Object.keys(stage.statuses).some(status=>Number(status)>=200&&Number(status)<300&&status!==expected))throw Error('Raw Rust response status differs from payload contract');
 return stage;
}
export {calibrationStageBudget};
export function rawReport(options,stages){
 const generatorLimited=stages.some(stage=>stage.droppedCapacity>0||stage.droppedScheduler>0||stage.coordination?.valid!==true);
 return {schema:1,kind:'raw-rust-axum-api-calibration',applicationCapacityMeasured:false,securityProof:false,mailwomanCapacityClaim:false,guardedApplicationMeasured:false,distinctPhysicalGeneratorHostsMeasured:false,interpretation:'Separate feature-only Axum/Hyper static response plus private calibration token; no Mailwoman guards, DB, accounts or index. Generator/transport ceiling only; not authenticated application throughput.',options,payloadBytes:{json:11,empty:0},wireResponseBytes:null,serverResourceMeasurement:'CPU/RSS unavailable here; use independent external process/cgroup sampling',generatorLimited,ceilingEstablished:false,ceilingObservation:generatorLimited?'generator-or-coordination-limited':stages.some(stage=>stage.errors>0||stage.timeouts>0)?'transport-or-server-limit-unattributed':'offered-rate-cap-or-observed-point; no maximum server saturation established',stages};
}
