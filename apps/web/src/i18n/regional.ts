import {createSignal} from 'solid-js';

export type RegionalKey='region'|'dateFormat'|'timeFormat'|'hourCycle'|'firstDayOfWeek';
type Source=()=>Readonly<Record<string,{value:unknown}>>;
const [source,setSource]=createSignal<Source>(()=>({}));
/** Bind to the effective account settings without importing their transport into i18n. */
export function configureRegionalSettings(read:Source):void {setSource(()=>read);}
export function regionalValue(key:RegionalKey):string {const value=source()()[`time.${key}`]?.value;return typeof value==='string'?value:'auto';}
export const DATE_FORMATS=['auto','short','medium','long','full','yyyy-MM-dd','dd/MM/yyyy','MM/dd/yyyy','dd.MM.yyyy','dd-MM-yyyy'] as const;
export const TIME_FORMATS=['auto','short','medium','long','HH:mm','HH:mm:ss','h:mm a','h:mm:ss a'] as const;
const TOKENS={date:['yyyy','yy','MMMM','MMM','MM','M','dd','d','EEEE','EEE'],time:['HH','H','hh','h','mm','m','ss','s','aaaa','a','zzzz','z']} as const;
export interface PatternPart {token?:string;literal?:string}
export function parseRegionalPattern(pattern:string,kind:'date'|'time'):PatternPart[]|null {
 if(!pattern||new TextEncoder().encode(pattern).length>120||/\p{Cc}/u.test(pattern))return null;
 const parts:PatternPart[]=[];let index=0,hasToken=false;
 while(index<pattern.length){
  if(pattern[index]==="'"){
   if(pattern[index+1]==="'"){parts.push({literal:"'"});index+=2;continue;}
   let literal='';index++;let closed=false;
   while(index<pattern.length){if(pattern[index]==="'"){if(pattern[index+1]==="'"){literal+="'";index+=2;}else{index++;closed=true;break;}}else literal+=pattern[index++];}
   if(!closed)return null;parts.push({literal});continue;
  }
  const token=TOKENS[kind].find(value=>pattern.startsWith(value,index));
  if(token){parts.push({token});index+=token.length;hasToken=true;continue;}
  if(/[A-Za-z]/.test(pattern[index]!))return null;
  parts.push({literal:pattern[index++]!});
 }
 return hasToken?parts:null;
}
export function validRegionalFormat(value:string,kind:'date'|'time'):boolean {return ['auto','short','medium','long','full'].includes(value)||parseRegionalPattern(value,kind)!==null;}
export function regionalLocale(app:string,region=regionalValue('region'),languages:readonly string[]=typeof navigator==='undefined'?[]:navigator.languages??[navigator.language]):string {
 try{
  const base=new Intl.Locale(app);
  if(/^[A-Z]{2}$/.test(region))return new Intl.Locale(base,{region}).toString();
  if(base.region)return base.toString();
  for(const tag of languages){try{const browser=new Intl.Locale(tag);if(browser.language===base.language&&browser.region)return new Intl.Locale(base,{region:browser.region,...(browser.script?{script:browser.script}:{})}).toString();}catch{/* Ignore malformed platform tags. */}}
  return base.toString();
 }catch{return app;}
}
export function regionalWeekStart():number|undefined {const value=regionalValue('firstDayOfWeek');return /^[0-6]$/.test(value)?Number(value):undefined;}

export function regionalOptions(options:Intl.DateTimeFormatOptions,hour=regionalValue('hourCycle')):Intl.DateTimeFormatOptions {
 const result={...options};
 if(hour==='12'||hour==='24'){delete result.hour12;result.hourCycle=hour==='12'?'h12':'h23';}
 return result;
}

/** Only complete date labels change shape; month/weekday headings stay meaningful. */
export function configuredDateParts(date:Date,options:Intl.DateTimeFormatOptions,format:(options:Intl.DateTimeFormatOptions)=>Intl.DateTimeFormat,overrides:Partial<Record<RegionalKey,string>>={}):Intl.DateTimeFormatPart[]|undefined {
 const hasDate=!!(options.dateStyle||options.month&&options.day),hasTime=!!(options.timeStyle||options.hour||options.minute||options.second);
 const dateFormat=overrides.dateFormat??regionalValue('dateFormat'),timeFormat=overrides.timeFormat??regionalValue('timeFormat'),hour=overrides.hourCycle??regionalValue('hourCycle');
 const customDate=hasDate&&dateFormat!=='auto'&&validRegionalFormat(dateFormat,'date'),customTime=hasTime&&timeFormat!=='auto'&&validRegionalFormat(timeFormat,'time');
 if(!customDate&&!customTime)return;
 const dateOptions={...options},timeOptions={...options};
 for(const key of ['timeStyle','hour','minute','second','fractionalSecondDigits','dayPeriod','timeZoneName'] as const)delete dateOptions[key];
 for(const key of ['dateStyle','year','month','day','weekday','era'] as const)delete timeOptions[key];
 const render=(value:string,kind:'date'|'time',original:Intl.DateTimeFormatOptions):Intl.DateTimeFormatPart[]=>{
  if(value==='auto')return format(regionalOptions(original,hour)).formatToParts(date);
  if(['short','medium','long','full'].includes(value))return format(regionalOptions({...(options.timeZone?{timeZone:options.timeZone}:{}),[kind==='date'?'dateStyle':'timeStyle']:value},hour)).formatToParts(date);
  const parts=parseRegionalPattern(value,kind)!;
  const common:Intl.DateTimeFormatOptions={...(options.timeZone?{timeZone:options.timeZone}:{}),...(options.calendar?{calendar:options.calendar}:{}),...(options.numberingSystem?{numberingSystem:options.numberingSystem}:{})};
  return parts.map<Intl.DateTimeFormatPart>(part=>{
   if(part.literal!==undefined)return {type:'literal',value:part.literal};
   const token=part.token!;let wanted:Intl.DateTimeFormatOptions={},type:Intl.DateTimeFormatPartTypes;
   if(token[0]==='y'){type='year';wanted.year=token==='yy'?'2-digit':'numeric';}
   else if(token[0]==='M'){type='month';wanted.month=token==='MMMM'?'long':token==='MMM'?'short':token==='MM'?'2-digit':'numeric';}
   else if(token[0]==='d'){type='day';wanted.day=token==='dd'?'2-digit':'numeric';}
   else if(token[0]==='E'){type='weekday';wanted.weekday=token==='EEEE'?'long':'short';}
   else if(token[0]==='H'||token[0]==='h'){type='hour';wanted={hour:token.length===2?'2-digit':'numeric',hourCycle:token[0]==='h'?'h12':'h23'};}
   else if(token[0]==='m'){type='minute';wanted.minute=token==='mm'?'2-digit':'numeric';}
   else if(token[0]==='s'){type='second';wanted.second=token==='ss'?'2-digit':'numeric';}
   else if(token[0]==='a'){type='dayPeriod';wanted={hour:'numeric',hour12:true};}
   else{type='timeZoneName';wanted={timeZoneName:token==='zzzz'?'long':'short'};}
   // Explicit H/h pattern tokens outrank the general hour-cycle preference.
   if(type==='month'&&['MMM','MMMM'].includes(token)&&parts.some(part=>part.token==='d'||part.token==='dd'))wanted.day='numeric';
   const raw=format({...common,...wanted});const values=raw.formatToParts(date);
   let text=values.find(piece=>piece.type===type)?.value??'';
   if(['MM','dd','HH','hh','mm','ss'].includes(token)&&text.length===1)text=new Intl.NumberFormat(raw.resolvedOptions().locale,{useGrouping:false}).format(0)+text;
   return {type,value:text};
  });
 };
 const result:Intl.DateTimeFormatPart[]=[];
 if(hasDate)result.push(...render(customDate?dateFormat:'auto','date',dateOptions));
 if(hasDate&&hasTime)result.push({type:'literal',value:', '});
 if(hasTime)result.push(...render(customTime?timeFormat:'auto','time',timeOptions));
 return result;
}
