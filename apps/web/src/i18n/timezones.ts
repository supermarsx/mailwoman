/** Time-zone calculations never infer a person's location from an email address. */
import {formatDateWith} from './format.ts';
export function validTimeZone(value: string): boolean {
  if (!value || value.length > 64 || !/^[A-Za-z_+-]+(?:\/[A-Za-z0-9_+-]+)*$/.test(value)) return false;
  try { new Intl.DateTimeFormat('en', { timeZone: value }).format(0); return true; } catch { return false; }
}
export function localTimeZone(): string { return Intl.DateTimeFormat().resolvedOptions().timeZone || 'UTC'; }
const dateKeyFormatters=new Map<string,Intl.DateTimeFormat>();
/** An invariant Gregorian date key for calculations, independent of display preferences. */
export function calendarDateInZone(date:Date,zone:string):string {
 let formatter=dateKeyFormatters.get(zone);
 if(!formatter){formatter=new Intl.DateTimeFormat('en-CA-u-ca-gregory-nu-latn',{timeZone:zone,year:'numeric',month:'2-digit',day:'2-digit'});if(dateKeyFormatters.size>=128)dateKeyFormatters.clear();dateKeyFormatters.set(zone,formatter);}
 const parts=Object.fromEntries(formatter.formatToParts(date).map(part=>[part.type,part.value]));
 return `${parts['year']}-${parts['month']}-${parts['day']}`;
}
/** Canonical IANA zones from the browser's bundled time-zone database, no network. */
export function supportedTimeZones(): string[] {
  const api=Intl as typeof Intl & {supportedValuesOf?:(key:string)=>string[]};
  return [...new Set(['UTC',localTimeZone(),...(api.supportedValuesOf?.('timeZone')??[])])].sort();
}
export function displayTimeZone(value: unknown): string {
  return typeof value === 'string' && value !== 'local' && validTimeZone(value) ? value : localTimeZone();
}

/** Resolve a wall-clock value without guessing through a DST gap or overlap.
 * Zero results means nonexistent/invalid; two means the repeated fall-back hour. */
export function wallTimeCandidates(value: string, zone: string): Date[] {
  if (!validTimeZone(zone)) return [];
  const match=/^(\d{4})-(\d{2})-(\d{2})T(\d{2}):(\d{2})(?::(\d{2}))?$/.exec(value);
  if (!match) return [];
  const wanted=match.slice(1).map(part=>Number(part??0));
  const [year,month,day,hour,minute,second]=wanted as [number,number,number,number,number,number];
  const seed=new Date(0);seed.setUTCFullYear(year,month-1,day);seed.setUTCHours(hour,minute,second,0);
  if (seed.getUTCFullYear()!==year||seed.getUTCMonth()!==month-1||seed.getUTCDate()!==day||seed.getUTCHours()!==hour||seed.getUTCMinutes()!==minute||seed.getUTCSeconds()!==second) return [];
  const formatter=new Intl.DateTimeFormat('en-US-u-ca-gregory-nu-latn',{timeZone:zone,year:'numeric',month:'2-digit',day:'2-digit',hour:'2-digit',minute:'2-digit',second:'2-digit',hourCycle:'h23'});
  function parts(instant: number): number[] {
    const found=Object.fromEntries(formatter.formatToParts(instant).map(part=>[part.type,part.value]));
    return ['year','month','day','hour','minute','second'].map(key=>Number(found[key]));
  }
  const offsets=new Set<number>();
  for (const hours of [-48,-24,0,24,48]) {
    const instant=seed.getTime()+hours*3600000;
    const [y,m,d,h,mi,s]=parts(instant) as [number,number,number,number,number,number];
    const local=new Date(0);local.setUTCFullYear(y,m-1,d);local.setUTCHours(h,mi,s,0);
    offsets.add(local.getTime()-instant);
  }
  return [...offsets].map(offset=>seed.getTime()-offset)
    .filter(instant=>parts(instant).every((part,index)=>part===wanted[index]))
    .sort((a,b)=>a-b).map(instant=>new Date(instant));
}

/** Zones offered when the runtime has no `Intl.supportedValuesOf` (older engines). */
export const FALLBACK_TIME_ZONES: readonly string[] = [
  'UTC', 'Africa/Cairo', 'Africa/Johannesburg', 'Africa/Lagos', 'Africa/Nairobi', 'America/Anchorage', 'America/Argentina/Buenos_Aires',
  'America/Bogota', 'America/Chicago', 'America/Denver', 'America/Halifax', 'America/Los_Angeles', 'America/Mexico_City', 'America/New_York',
  'America/Sao_Paulo', 'America/Toronto', 'Asia/Bangkok', 'Asia/Dhaka', 'Asia/Dubai', 'Asia/Hong_Kong', 'Asia/Jakarta', 'Asia/Karachi',
  'Asia/Kolkata', 'Asia/Seoul', 'Asia/Shanghai', 'Asia/Singapore', 'Asia/Tokyo', 'Atlantic/Azores', 'Atlantic/Reykjavik', 'Australia/Perth',
  'Australia/Sydney', 'Europe/Amsterdam', 'Europe/Athens', 'Europe/Berlin', 'Europe/Dublin', 'Europe/Istanbul', 'Europe/Lisbon', 'Europe/London',
  'Europe/Madrid', 'Europe/Moscow', 'Europe/Paris', 'Europe/Rome', 'Pacific/Auckland', 'Pacific/Honolulu',
];
/** The zone list for a picker: the runtime database, or the short fallback list. */
export function pickerTimeZones(): string[] {
  const api=Intl as typeof Intl & {supportedValuesOf?:(key:string)=>string[]};
  let known:string[]=[];
  try { known=api.supportedValuesOf?.('timeZone')??[]; } catch { known=[]; }
  return [...new Set(['UTC',localTimeZone(),...(known.length?known:FALLBACK_TIME_ZONES)])].filter(validTimeZone).sort();
}
/** Offset from UTC in minutes for `zone` at `at` (DST aware), or null for an unknown zone. */
export function zoneOffsetMinutes(zone: string, at: Date = new Date()): number | null {
  try {
    const parts=Object.fromEntries(new Intl.DateTimeFormat('en-US-u-ca-gregory-nu-latn',{timeZone:zone,year:'numeric',month:'2-digit',day:'2-digit',hour:'2-digit',minute:'2-digit',second:'2-digit',hourCycle:'h23'}).formatToParts(at).map(part=>[part.type,part.value]));
    const asUtc=Date.UTC(Number(parts.year),Number(parts.month)-1,Number(parts.day),Number(parts.hour),Number(parts.minute),Number(parts.second));
    return Math.round((asUtc-Math.floor(at.getTime()/1000)*1000)/60000);
  } catch { return null; }
}
/** `UTC+01:00`, `UTC-03:30`, `UTC` for zero. */
export function formatUtcOffset(minutes: number): string {
  if (minutes===0) return 'UTC';
  const abs=Math.abs(minutes);
  return `UTC${minutes<0?'-':'+'}${String(Math.floor(abs/60)).padStart(2,'0')}:${String(abs%60).padStart(2,'0')}`;
}
/** The wall-clock time in `zone` at `at`, 24-hour `HH:MM`. */
export function zoneClock(zone: string, at: Date = new Date()): string {
  try { return formatDateWith(at,{timeZone:zone,hour:'2-digit',minute:'2-digit'}); } catch { return ''; }
}
