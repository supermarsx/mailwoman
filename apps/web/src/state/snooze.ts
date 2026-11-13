import {settings} from '../api/settings.ts';
import {loadCatalog,t,formatLocale} from '../i18n';
import {calendarDateInZone,displayTimeZone,wallTimeCandidates} from '../i18n/timezones.ts';
import {regionalWeekStart} from '../i18n/regional.ts';
void loadCatalog('snooze');
export const SNOOZE_PRESETS_KEY='mail.snooze.presets';
export const SNOOZE_KINDS=['hours','days','tomorrow','next-week','end-week','end-month','next-month'] as const;
export type SnoozeKind=typeof SNOOZE_KINDS[number];
export interface SnoozePreset {id:string;kind:SnoozeKind;amount?:number;time?:string;label?:string;enabled:boolean}
export const DEFAULT_SNOOZE_PRESETS:SnoozePreset[]=[
 {id:'later',kind:'hours',amount:3,enabled:true},
 {id:'tomorrow',kind:'tomorrow',time:'09:00',enabled:true},
 {id:'next-week',kind:'next-week',time:'09:00',enabled:true},
 {id:'2-hours',kind:'hours',amount:2,enabled:true},
 {id:'6-hours',kind:'hours',amount:6,enabled:true},
 {id:'24-hours',kind:'hours',amount:24,enabled:true},
 {id:'3-days',kind:'days',amount:3,time:'09:00',enabled:true},
 {id:'7-days',kind:'days',amount:7,time:'09:00',enabled:true},
 {id:'end-week',kind:'end-week',time:'17:00',enabled:true},
 {id:'end-month',kind:'end-month',time:'17:00',enabled:true},
 {id:'next-month',kind:'next-month',time:'09:00',enabled:true},
];
export function validSnoozePresets(value:unknown):value is SnoozePreset[]{
 if(!Array.isArray(value)||value.length>32)return false;
 const ids=new Set<string>();
 return value.every(item=>{
  if(!item||typeof item!=='object'||Array.isArray(item)||Object.keys(item).some(key=>!['id','kind','amount','time','label','enabled'].includes(key)))return false;
  const p=item as SnoozePreset;
  if(typeof p.id!=='string'||!/^[a-zA-Z0-9_-]{1,64}$/.test(p.id)||ids.has(p.id)||!SNOOZE_KINDS.includes(p.kind)||typeof p.enabled!=='boolean')return false;
  ids.add(p.id);
  if(p.label!==undefined&&(typeof p.label!=='string'||![...p.label.trim()].length||[...p.label].length>80||/[\p{Cc}\p{Cf}]/u.test(p.label)))return false;
  if(p.kind==='hours'||p.kind==='days'){
   if(!Number.isInteger(p.amount)||p.amount!<1||p.amount!>(p.kind==='hours'?87600:3650))return false;
  }else if(p.amount!==undefined)return false;
  return p.kind==='hours'?p.time===undefined:typeof p.time==='string'&&/^([01]\d|2[0-3]):[0-5]\d$/.test(p.time);
 });
}
export function snoozeZone():string{return displayTimeZone(settings()['time.displayZone']?.value);}
export function snoozeWeekStart():number{
 const configured=regionalWeekStart();if(configured!==undefined)return configured;
 try{const info=new Intl.Locale(formatLocale()) as Intl.Locale&{weekInfo?:{firstDay:number};getWeekInfo?:()=>{firstDay:number}};const first=(info.getWeekInfo?.()??info.weekInfo)?.firstDay;return first===undefined?1:first%7;}catch{return 1;}
}
export function configuredSnoozePresets():SnoozePreset[]{const value=settings()[SNOOZE_PRESETS_KEY]?.value;return validSnoozePresets(value)?value:DEFAULT_SNOOZE_PRESETS;}
export function snoozePresetLabel(p:SnoozePreset):string{
 if(p.label)return p.label;
 if(p.id==='later'&&p.kind==='hours'&&p.amount===3)return t('mail-snooze-later');
 return p.kind==='hours'||p.kind==='days'?t(`snooze-kind-${p.kind}`,{count:p.amount!}):t(`snooze-kind-${p.kind}`);
}
/** Calendar presets use wall time in the display zone; hours mean elapsed hours. */
export function resolveSnooze(p:SnoozePreset,now=new Date(),zone=snoozeZone(),weekStart=snoozeWeekStart()):string|null{
 const rule={id:p.id,kind:p.kind,enabled:p.enabled,...(p.amount!==undefined?{amount:p.amount}:{}),...(p.time!==undefined?{time:p.time}:{}),...(p.label!==undefined?{label:p.label}:{})};
 if(!validSnoozePresets([rule])||!Number.isFinite(now.getTime()))return null;
 if(p.kind==='hours')return new Date(now.getTime()+p.amount!*3600000).toISOString();
 const date=new Date(`${calendarDateInZone(now,zone)}T12:00:00Z`);
 if(p.kind==='days'||p.kind==='tomorrow')date.setUTCDate(date.getUTCDate()+(p.kind==='days'?p.amount!:1));
 if(p.kind==='next-week')date.setUTCDate(date.getUTCDate()+((weekStart-date.getUTCDay()+7)%7||7));
 if(p.kind==='end-week')date.setUTCDate(date.getUTCDate()+((weekStart+6-date.getUTCDay()+7)%7));
 if(p.kind==='end-month')date.setUTCMonth(date.getUTCMonth()+1,0);
 if(p.kind==='next-month')date.setUTCMonth(date.getUTCMonth()+1,1);
 for(let attempt=0;attempt<2;attempt++){
  // A preset in a repeated hour takes the first future occurrence. A missing hour is omitted.
  const result=wallTimeCandidates(`${date.toISOString().slice(0,10)}T${p.time}`,zone).find(candidate=>candidate>now);
  if(result)return result.toISOString();
  if(p.kind==='end-week')date.setUTCDate(date.getUTCDate()+7);
  else if(p.kind==='end-month')date.setUTCMonth(date.getUTCMonth()+2,0);
  else return null;
 }
 return null;
}
export function snoozeWallInput(at:Date,zone=snoozeZone()):string{
 // Native inputs require invariant numeric components, regardless of display preferences.
 const parts=Object.fromEntries(new Intl.DateTimeFormat('en-CA-u-ca-gregory-nu-latn',{timeZone:zone,hour:'2-digit',minute:'2-digit',hourCycle:'h23'}).formatToParts(at).map(p=>[p.type,p.value]));
 return `${calendarDateInZone(at,zone)}T${parts.hour}:${parts.minute}`;
}
