import type {expandCachedCalendar} from './calendarExpansion.ts';
export async function expandOfflineCalendar(events:Parameters<typeof expandCachedCalendar>[0],start:string,end:string):Promise<ReturnType<typeof expandCachedCalendar>>{
 if(typeof Worker==='undefined')return (await import('./calendarExpansion.ts')).expandCachedCalendar(events,start,end);
 const worker=new Worker(new URL('./calendarWorker.ts',import.meta.url),{type:'module'});
 return new Promise((resolve,reject)=>{
  const timer=setTimeout(()=>{worker.terminate();reject(new Error('Offline calendar expansion timed out'));},2500);
  worker.onerror=()=>{clearTimeout(timer);worker.terminate();reject(new Error('Offline calendar unavailable'));};
  worker.onmessage=(event:MessageEvent<{list?:ReturnType<typeof expandCachedCalendar>;error?:string}>)=>{clearTimeout(timer);worker.terminate();if(event.data.error)reject(new Error(event.data.error));else resolve(event.data.list??[]);};
  worker.postMessage({events,start,end});
 });
}
