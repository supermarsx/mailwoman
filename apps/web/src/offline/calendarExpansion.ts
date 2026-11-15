import {RRule} from 'rrule';
type Event=Record<string,unknown>&{id:string};
const DAY=86400000;
const wall=(value:string)=>Date.parse(value.endsWith('Z')?value:value.length===10?value+'T00:00:00Z':value+'Z');
const duration=(value:unknown)=>{const match=/^P(?:(\d+)W)?(?:(\d+)D)?(?:T(?:(\d+)H)?(?:(\d+)M)?(?:(\d+(?:\.\d+)?)S)?)?$/.exec(String(value));return match?((Number(match[1]??0)*7+Number(match[2]??0))*86400+Number(match[3]??0)*3600+Number(match[4]??0)*60+Number(match[5]??0))*1000:0;};
/** Resolve a wall clock independently of the device zone. Ambiguous clocks use
 * the earliest instant, matching the engine; nonexistent clocks are skipped. */
export function calendarInstant(value:string,zone:unknown):number{
 const seed=wall(value);if(!zone||zone==='UTC'||zone==='Etc/UTC'||value.length===10)return seed;
 const formatter=new Intl.DateTimeFormat('en-US-u-ca-gregory-nu-latn',{timeZone:String(zone),year:'numeric',month:'2-digit',day:'2-digit',hour:'2-digit',minute:'2-digit',second:'2-digit',hourCycle:'h23'});
 const stamp=(instant:number)=>{const p=Object.fromEntries(formatter.formatToParts(instant).map(p=>[p.type,p.value]));return Date.UTC(Number(p.year),Number(p.month)-1,Number(p.day),Number(p.hour),Number(p.minute),Number(p.second));};
 const offsets=[seed-DAY*2,seed,seed+DAY*2].map(at=>stamp(at)-at);
 const matches=[...new Set(offsets)].map(offset=>seed-offset).filter(at=>stamp(at)===seed);
 return matches.length?Math.min(...matches):NaN;
}
/** RRULE evaluation runs in a terminable worker. Its output uses the same UTC
 * instance bounds and floating-clock encoding as CalendarEvent/expand. */
export function expandCachedCalendar(events:Event[],start:string,end:string):Event[]{
 const low=Date.parse(start),high=Date.parse(end);if(!Number.isFinite(low)||!Number.isFinite(high)||high<=low||high-low>DAY*3660)throw new Error('Invalid offline calendar window');
 const result:Event[]=[];const deadline=Date.now()+1500;
 for(const event of events){
  if(Date.now()>deadline)throw new Error('Offline calendar expansion timed out');
  const seed=wall(String(event['start']));if(!Number.isFinite(seed))continue;
  const rules=Array.isArray(event['recurrenceRules'])?event['recurrenceRules'] as Record<string,unknown>[]:[];
  const overrides=(event['recurrenceOverrides']??{}) as Record<string,Record<string,unknown>>;
  const dates=new Set<number>();if(!rules.length)dates.add(seed);
  for(const rule of rules){
   if(typeof rule['rrule']!=='string')throw new Error('Offline recurrence requires the stored RRULE');
   const options=RRule.parseString(rule['rrule']);options.dtstart=new Date(seed);delete options.tzid;
   const until=options.until?.getTime();delete options.until;
   new RRule(options,true).between(new Date(low-DAY*2-duration(event['duration'])),new Date(high+DAY*2),true,(date,index)=>{
    if(index>=10000||Date.now()>deadline)throw new Error('Offline recurrence exceeds the display limit');
    const instant=calendarInstant(date.toISOString().slice(0,19),event['timeZone']);
    if(until===undefined||instant<=until)dates.add(date.getTime());return true;
   });
  }
  for(const key of Object.keys(overrides)){const stamp=wall(key);if(Number.isFinite(stamp))dates.add(stamp);}
  const excluded=new Set((Array.isArray(event['excludedRecurrenceDates'])?event['excludedRecurrenceDates']:[]) as string[]);
  for(const stamp of [...dates].sort((a,b)=>a-b)){
   const local=new Date(stamp).toISOString().slice(0,19),date=local.slice(0,10);
   if(excluded.has(local)||excluded.has(local+'Z')||excluded.has(date))continue;
   const patch=overrides[local]??overrides[local+'Z']??overrides[date]??{};if(patch['excluded']===true)continue;
   const instance={...event,...patch};const begin=calendarInstant(String(patch['start']??local),event['timeZone']),finish=begin+duration(instance['duration']);
   if(!Number.isFinite(begin)||begin<low||begin>=high)continue;
   result.push({...instance,eventId:event.id,start:new Date(begin).toISOString(),timeZone:'UTC',instanceStart:new Date(begin).toISOString(),instanceEnd:new Date(finish).toISOString()});
   if(result.length>10000)throw new Error('Offline calendar display limit exceeded');
  }
 }
 return result;
}
