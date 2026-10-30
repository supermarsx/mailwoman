import {settings,settingsLoadStatus} from '../api/settings.ts';

export const MAIL_CACHE_FIELDS = [
 ['enabled',true],['refreshOnOpen',true],['mailboxes',true],['messages',true],
 ['freshSeconds',30,0,3600],['maxAgeSeconds',300,1,86400],
 ['maxEntries',32,1,512],['maxBytes',16777216,65536,268435456],
 ['prefetchEnabled',false],['hoverEnabled',true],['focusEnabled',true],
 ['dwellMs',200,0,10000],['idleEnabled',false],['idleDelayMs',1500,100,60000],
 ['maxConcurrency',1,1,4],['maxQueued',8,1,64],['preloadMessages',false],
] as const;
export type MailCacheField = typeof MAIL_CACHE_FIELDS[number][0];
export interface MailCachePolicy {
 enabled:boolean;refreshOnOpen:boolean;mailboxes:boolean;messages:boolean;freshSeconds:number;maxAgeSeconds:number;
 maxEntries:number;maxBytes:number;prefetchEnabled:boolean;hoverEnabled:boolean;focusEnabled:boolean;
 dwellMs:number;idleEnabled:boolean;idleDelayMs:number;maxConcurrency:number;maxQueued:number;
 preloadMessages:boolean;excludedFolders:readonly string[];
}
export const DEFAULT_MAIL_CACHE_POLICY:MailCachePolicy={
 enabled:true,refreshOnOpen:true,mailboxes:true,messages:true,freshSeconds:30,maxAgeSeconds:300,maxEntries:32,maxBytes:16777216,
 prefetchEnabled:false,hoverEnabled:true,focusEnabled:true,dwellMs:200,idleEnabled:false,idleDelayMs:1500,
 maxConcurrency:1,maxQueued:8,preloadMessages:false,excludedFolders:['role:trash','role:junk'],
};
/** Unknown/unloaded effective policy never authorizes speculative work. */
export function readMailCachePolicy():MailCachePolicy {
 const result={...DEFAULT_MAIL_CACHE_POLICY};const map=settings();
 for(const field of MAIL_CACHE_FIELDS){
  const value=map[`mail.cache.${field[0]}`]?.value;
  if(typeof field[1]==='boolean'){
   if(typeof value==='boolean')Object.assign(result,{[field[0]]:value});
  }else if(typeof value==='number'&&Number.isInteger(value)&&value>=field[2]&&value<=field[3])Object.assign(result,{[field[0]]:value});
 }
 const excluded=map['mail.cache.excludedFolders']?.value;
 if(Array.isArray(excluded)&&excluded.length<=100&&excluded.every(v=>typeof v==='string'&&v.length>0&&v.length<=256&&![...v].some(c=>c.charCodeAt(0)<32||c.charCodeAt(0)===127)))result.excludedFolders=excluded as string[];
 if(settingsLoadStatus()!=='ready')result.prefetchEnabled=false;
 result.freshSeconds=Math.min(result.freshSeconds,result.maxAgeSeconds);
 return result;
}
