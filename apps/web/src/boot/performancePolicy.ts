/** Public delivery hints only: never authentication or message-rendering authority. */
export interface WebPerformancePolicy {
 retainedLocales:number;
 concurrentLoads:number;
 prefetchEnabled:boolean;
 idleDelayMs:number;
 precacheMode:'all'|'core';
 precacheConcurrency:number;
 respectSaveData:boolean;
}
export const DEFAULT_WEB_PERFORMANCE:Readonly<WebPerformancePolicy>=Object.freeze({retainedLocales:3,concurrentLoads:4,prefetchEnabled:true,idleDelayMs:2500,precacheMode:'all',precacheConcurrency:4,respectSaveData:true});
let current:Readonly<WebPerformancePolicy>=DEFAULT_WEB_PERFORMANCE;
let ready=false;
const listeners=new Set<()=>void>();
export function webPerformancePolicy():Readonly<WebPerformancePolicy>{return current;}
export function webPerformanceReady():boolean{return ready;}
/** Demand limits remain usable, but new speculation needs a fresh response. */
export function invalidateWebPerformance():void{ready=false;}
export function observeWebPerformance(listener:()=>void):()=>void{listeners.add(listener);return()=>listeners.delete(listener);}
export function parseWebPerformance(raw:unknown):WebPerformancePolicy {
 const value=raw&&typeof raw==='object'?raw as Record<string,unknown>:{};
 const integer=(key:keyof WebPerformancePolicy,min:number,max:number):number=>typeof value[key]==='number'&&Number.isInteger(value[key])&&Number(value[key])>=min&&Number(value[key])<=max?Number(value[key]):Number(DEFAULT_WEB_PERFORMANCE[key]);
 const boolean=(key:keyof WebPerformancePolicy):boolean=>typeof value[key]==='boolean'?value[key] as boolean:DEFAULT_WEB_PERFORMANCE[key] as boolean;
 return {retainedLocales:integer('retainedLocales',2,8),concurrentLoads:integer('concurrentLoads',1,8),prefetchEnabled:boolean('prefetchEnabled'),idleDelayMs:integer('idleDelayMs',0,30000),precacheMode:value.precacheMode==='core'?'core':'all',precacheConcurrency:integer('precacheConcurrency',1,4),respectSaveData:boolean('respectSaveData')};
}
export function applyWebPerformance(raw:unknown):void{current=Object.freeze(parseWebPerformance(raw));ready=true;for(const listener of [...listeners])listener();}
/** Test isolation; demand loads always retain safe defaults before bootstrap. */
export function resetWebPerformance():void{current=DEFAULT_WEB_PERFORMANCE;ready=false;for(const listener of [...listeners])listener();}
