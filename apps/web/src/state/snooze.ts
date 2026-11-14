import {settings} from '../api/settings.ts';
import {loadCatalog,t,formatLocale} from '../i18n';
import {calendarDateInZone,displayTimeZone,wallTimeCandidates} from '../i18n/timezones.ts';
import {regionalWeekStart} from '../i18n/regional.ts';
void loadCatalog('snooze');
export const SNOOZE_PRESETS_KEY='mail.snooze.presets';
export const FOLLOW_UP_PRESETS_KEY='mail.followUp.presets';
export const SNOOZE_KINDS=['hours','days','tomorrow','next-week','end-week','end-month','next-month','weekday','nth-weekday','day-of-month','next-year'] as const;
export type SnoozeKind=typeof SNOOZE_KINDS[number];
export interface SnoozePreset {id:string;kind:SnoozeKind;amount?:number;time?:string;label?:string;weekday?:number;ordinal?:number;day?:number;enabled:boolean}
export const DEFAULT_SNOOZE_PRESETS:SnoozePreset[]=[
 {id:'later',kind:'hours',amount:3,enabled:true},
 {id:'tomorrow',kind:'tomorrow',time:'09:00',enabled:true},
 {id:'next-week',kind:'next-week',time:'09:00',enabled:true},
 {id:'2-hours',kind:'hours',amount:2,enabled:false},
 {id:'6-hours',kind:'hours',amount:6,enabled:false},
 {id:'24-hours',kind:'hours',amount:24,enabled:false},
 {id:'3-days',kind:'days',amount:3,time:'09:00',enabled:false},
 {id:'7-days',kind:'days',amount:7,time:'09:00',enabled:false},
 {id:'end-week',kind:'end-week',time:'17:00',enabled:false},
 {id:'end-month',kind:'end-month',time:'17:00',enabled:false},
 {id:'next-month',kind:'next-month',time:'09:00',enabled:false},
 {id:'next-monday',kind:'weekday',weekday:1,time:'09:00',enabled:false},
 {id:'second-tuesday',kind:'nth-weekday',weekday:2,ordinal:2,time:'09:00',enabled:false},
 {id:'next-first',kind:'day-of-month',day:1,time:'09:00',enabled:false},
 {id:'next-year',kind:'next-year',time:'09:00',enabled:false},
];
export function validSnoozePresets(value:unknown):value is SnoozePreset[]{
 if(!Array.isArray(value)||value.length>32)return false;
 const ids=new Set<string>();
 return value.every(item=>{
  if(!item||typeof item!=='object'||Array.isArray(item)||Object.keys(item).some(key=>!['id','kind','amount','time','label','enabled','weekday','ordinal','day'].includes(key)))return false;
  const p=item as SnoozePreset;
  if(typeof p.id!=='string'||!/^[a-zA-Z0-9_-]{1,64}$/.test(p.id)||ids.has(p.id)||!SNOOZE_KINDS.includes(p.kind)||typeof p.enabled!=='boolean')return false;
  ids.add(p.id);
  if(p.label!==undefined&&(typeof p.label!=='string'||![...p.label.trim()].length||[...p.label].length>80||/[\p{Cc}\p{Cf}]/u.test(p.label)))return false;
  if(p.kind==='hours'||p.kind==='days'){
   if(!Number.isInteger(p.amount)||p.amount!<1||p.amount!>(p.kind==='hours'?87600:3650))return false;
  }else if(p.amount!==undefined)return false;
  if(p.kind==='weekday'||p.kind==='nth-weekday'){if(!Number.isInteger(p.weekday)||p.weekday!<0||p.weekday!>6)return false;}else if(p.weekday!==undefined)return false;
  if(p.kind==='nth-weekday'){if(!Number.isInteger(p.ordinal)||![1,2,3,4,5,-1].includes(p.ordinal!))return false;}else if(p.ordinal!==undefined)return false;
  if(p.kind==='day-of-month'){if(!Number.isInteger(p.day)||p.day!<1||p.day!>31)return false;}else if(p.day!==undefined)return false;
  return p.kind==='hours'?p.time===undefined:typeof p.time==='string'&&/^([01]\d|2[0-3]):[0-5]\d$/.test(p.time);
 });
}
export function snoozeZone():string{return displayTimeZone(settings()['time.displayZone']?.value);}
export function snoozeWeekStart():number{
 const configured=regionalWeekStart();if(configured!==undefined)return configured;
 try{const info=new Intl.Locale(formatLocale()) as Intl.Locale&{weekInfo?:{firstDay:number};getWeekInfo?:()=>{firstDay:number}};const first=(info.getWeekInfo?.()??info.weekInfo)?.firstDay;return first===undefined?1:first%7;}catch{return 1;}
}
export function configuredSnoozePresets(key=SNOOZE_PRESETS_KEY):SnoozePreset[]{const value=settings()[key]?.value;return validSnoozePresets(value)?value:DEFAULT_SNOOZE_PRESETS;}
export function scheduleWeekday(day:number):string{return new Intl.DateTimeFormat(formatLocale(),{weekday:'long',timeZone:'UTC'}).format(new Date(Date.UTC(2023,0,1+day)));}
export function snoozePresetLabel(p:SnoozePreset):string{
 if(p.label)return p.label;
 if(p.kind==='weekday')return t('snooze-kind-weekday',{weekday:scheduleWeekday(p.weekday!)});
 if(p.kind==='nth-weekday')return t(p.ordinal===-1?'snooze-kind-last-weekday':'snooze-kind-nth-weekday',{weekday:scheduleWeekday(p.weekday!),ordinal:p.ordinal!});
 if(p.kind==='day-of-month')return t('snooze-kind-day-of-month',{day:p.day!});
 if(p.id==='later'&&p.kind==='hours'&&p.amount===3)return t('mail-snooze-later');
 return p.kind==='hours'||p.kind==='days'?t(`snooze-kind-${p.kind}`,{count:p.amount!}):t(`snooze-kind-${p.kind}`);
}
/** Calendar presets use wall time in the display zone; hours mean elapsed hours. */
export function resolveSnooze(p:SnoozePreset,now=new Date(),zone=snoozeZone(),weekStart=snoozeWeekStart()):string|null{
 const rule={id:p.id,kind:p.kind,enabled:p.enabled,...(p.weekday!==undefined?{weekday:p.weekday}:{}),...(p.ordinal!==undefined?{ordinal:p.ordinal}:{}),...(p.day!==undefined?{day:p.day}:{}),...(p.amount!==undefined?{amount:p.amount}:{}),...(p.time!==undefined?{time:p.time}:{}),...(p.label!==undefined?{label:p.label}:{})};
 if(!validSnoozePresets([rule])||!Number.isFinite(now.getTime()))return null;
 if(p.kind==='hours')return new Date(now.getTime()+p.amount!*3600000).toISOString();
 const date=new Date(`${calendarDateInZone(now,zone)}T12:00:00Z`);
 if(p.kind==='days'||p.kind==='tomorrow')date.setUTCDate(date.getUTCDate()+(p.kind==='days'?p.amount!:1));
 if(p.kind==='next-week')date.setUTCDate(date.getUTCDate()+((weekStart-date.getUTCDay()+7)%7||7));
 if(p.kind==='end-week')date.setUTCDate(date.getUTCDate()+((weekStart+6-date.getUTCDay()+7)%7));
 if(p.kind==='end-month')date.setUTCMonth(date.getUTCMonth()+1,0);
 if(p.kind==='next-month')date.setUTCMonth(date.getUTCMonth()+1,1);
 if(p.kind==='weekday')date.setUTCDate(date.getUTCDate()+((p.weekday!-date.getUTCDay()+7)%7||7));
 if(p.kind==='next-year')date.setUTCFullYear(date.getUTCFullYear()+1,0,1);
 if(p.kind==='nth-weekday'){
  date.setUTCMonth(date.getUTCMonth()+1,1);const month=date.getUTCMonth();
  if(p.ordinal===-1){date.setUTCMonth(month+1,0);date.setUTCDate(date.getUTCDate()-(date.getUTCDay()-p.weekday!+7)%7);}
  else date.setUTCDate(1+(p.weekday!-date.getUTCDay()+7)%7+7*(p.ordinal!-1));
  if(date.getUTCMonth()!==month)return null;
 }
 if(p.kind==='day-of-month'){
  const year=date.getUTCFullYear(),month=date.getUTCMonth();
  for(let offset=0;offset<14;offset++){
   const candidate=new Date(Date.UTC(year,month+offset,p.day!,12));
   if(candidate.getUTCMonth()!==(month+offset)%12)continue;
   const at=wallTimeCandidates(`${candidate.toISOString().slice(0,10)}T${p.time}`,zone).find(at=>at>now);if(at)return at.toISOString();
  }return null;
 }
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
