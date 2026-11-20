// IPC protocol test double ONLY: fabricates measurements and makes NO requests.
// Never use this entrypoint for performance or application-capacity evidence.
import {readFile} from 'node:fs/promises';
import {Measurements} from '../http-load.mjs';
const assigned=JSON.parse(await readFile(process.argv[2],'utf8'));
process.on('disconnect',()=>process.exit(0));
let phase;
process.on('message',message=>{
 if(message.type==='prepare'){phase=message;process.send({type:'stage-ready',workerIndex:assigned.workerIndex,stageId:phase.stageId});return;}
 if(message.type!=='start')return;
 const start=message.startAtMs,active=phase;
 setTimeout(()=>{
  const count=active.arrivalRate*active.seconds,aggregate=new Measurements(),scheduling=new Measurements();
  for(let i=0;i<count;i++){aggregate.add(1,'200',4,true);scheduling.add(0,'scheduled',0,true);}
  const users=Math.min(assigned.principals.length,count),completedAtMs=Math.max(Date.now(),start+active.seconds*1000);
  process.send({type:'result',schema:1,workerIndex:assigned.workerIndex,partitionCount:assigned.principals.length,stageId:active.stageId,measurement:{window:{requestedStartAtMs:start,scheduledStartAtMs:start,scheduledEndAtMs:start+active.seconds*1000,actualStartAtMs:start,completedAtMs,startLagMs:0},rawMeasurements:{schema:1,aggregate:aggregate.snapshot(),byRequest:{'Email/query':aggregate.snapshot()},scheduling:scheduling.snapshot(),arrivalLatency:aggregate.snapshot()},concurrency:active.concurrency,scheduledSeconds:active.seconds,arrivalRate:active.arrivalRate,offered:count,droppedCapacity:0,droppedScheduler:0,acceptedDuringScheduledWindow:count,uniquePrincipalsAttempted:users,uniquePrincipalsAccepted:users,uniquePrincipalsAcceptedDuringScheduledWindow:users,uniqueClientsAttempted:users,uniqueClientsAccepted:users,uniqueClientsAcceptedDuringScheduledWindow:users,peakOutstanding:1,mutationStateTainted:false,unknownMutationOutcomes:0,taintedPrincipals:0,httpDispatchAttempts:count,blockedAfterMutationTaint:0,abortedUnscheduled:0,generator:{cpuUserMs:0,cpuSystemMs:0,rssBeforeBytes:0,rssAfterBytes:0,heapAfterBytes:0}}});
 },Math.max(0,start+active.seconds*1000-Date.now()));
});
process.send({type:'ready',schema:1,workerIndex:assigned.workerIndex,partitionCount:assigned.principals.length});
