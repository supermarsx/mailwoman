import type {JmapRequest} from './jmap-types.ts';
const EXTRA_READS=new Set(['Core/echo','CalendarEvent/expand','Calendar/detectConflicts','Calendar/freeBusy','CalendarEvent/parse','CalendarEvent/export','Note/export','ContactCard/parse','ContactCard/export']);
/** Only protocol reads may be coalesced or resubmitted after owner failover. */
export function jmapReadsOnly(body:JmapRequest):boolean{return Array.isArray(body.methodCalls)&&body.methodCalls.length>0&&body.methodCalls.every(([method,args])=>/\/(get|query|changes|queryChanges)$/.test(method)||EXTRA_READS.has(method)||method==='CalendarEvent/itip'&&args['apply']!==true);}
