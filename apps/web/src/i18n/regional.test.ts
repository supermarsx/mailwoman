import {afterEach,beforeEach,expect,it,vi} from 'vitest';
import {clearSettings,loadSettings} from '../api/settings.ts';
import {setActiveLocale} from './registry.ts';
import {formatDate,formatDateTime,formatDateToParts,formatDateWith,formatLocale,formatTime} from './format.ts';
import {parseRegionalPattern,regionalLocale,validRegionalFormat} from './regional.ts';
import {localeWeekStart} from '../modules/calendar/datetime.ts';
const date=new Date('2026-11-23T21:05:09Z'),options={dateStyle:'medium',timeStyle:'short',timeZone:'UTC'} as const;
const apply=(values:Record<string,string>)=>loadSettings({refresh:true,fetcher:async()=>new Response(JSON.stringify({settings:Object.fromEntries(Object.entries(values).map(([key,value])=>[`time.${key}`,{value,source:'user',locked:false}]))}),{headers:{'content-type':'application/json'}})});
beforeEach(()=>{clearSettings();setActiveLocale('en');vi.spyOn(navigator,'languages','get').mockReturnValue(['en-US']);});
afterEach(()=>{clearSettings();setActiveLocale('en');vi.restoreAllMocks();});
it('automatically preserves the user language and matching browser region',()=>{
 expect(regionalLocale('en','auto',['en-GB'])).toBe('en-GB');expect(regionalLocale('en','auto',['en-US'])).toBe('en-US');
 expect(regionalLocale('pt-PT','auto',['pt-BR'])).toBe('pt-PT');expect(regionalLocale('de','auto',['en-US'])).toBe('de');
 expect(regionalLocale('en','PT',['en-US'])).toBe('en-PT');
});
it('accepts bounded display patterns with quoted text and rejects typos and controls',()=>{
 for(const pattern of ['yyyy-MM-dd','dd/MM/yyyy',"EEEE, d 'de' MMMM 'de' yyyy",'yyyy年MM月dd日'])expect(validRegionalFormat(pattern,'date')).toBe(true);
 for(const pattern of ['YYYY-MM-DD',"yyyy 'unfinished",'HH:mm','yyyy\nMM',"'literal only'"])expect(validRegionalFormat(pattern,'date')).toBe(false);
 expect(parseRegionalPattern('H:mm:ss a z','time')).not.toBeNull();expect(validRegionalFormat('HH:MM','time')).toBe(false);
 expect(validRegionalFormat('dd/'.repeat(50),'date')).toBe(false);
});
it('applies custom formats to display labels and parts, with explicit zones preserved',async()=>{
 await apply({region:'GB',dateFormat:'yyyy-MM-dd',timeFormat:'HH:mm:ss'});
 expect(formatLocale()).toBe('en-GB');expect(formatDateTime(date,options)).toBe('2026-11-23, 21:05:09');
 expect(formatDateToParts(date,options).map(part=>part.value).join('')).toBe(formatDateTime(date,options));
 expect(formatDateTime(date,{...options,timeZone:'America/New_York'})).toBe('2026-11-23, 16:05:09');
 expect(formatDateWith(date,{month:'long',year:'numeric',timeZone:'UTC'})).toBe('November 2026');
});
it('uses a true 00-based 24-hour custom clock and explicit 12-hour pattern tokens',async()=>{
 await apply({timeFormat:'HH:mm',hourCycle:'12'});expect(formatTime(new Date('2026-11-23T00:05:09Z'),{timeStyle:'short',timeZone:'UTC'})).toBe('00:05');
 await apply({timeFormat:'h:mm a',hourCycle:'24'});expect(formatTime(date,{timeStyle:'short',timeZone:'UTC'})).toMatch(/^9:05\sPM$/);
});
it('supports automatic regional styles, explicit hour cycles, calendar week overrides and resetting',async()=>{
 await apply({region:'GB'});expect(formatDate(date,{dateStyle:'short',timeZone:'UTC'})).toBe(new Intl.DateTimeFormat('en-GB',{dateStyle:'short',timeZone:'UTC'}).format(date));expect(localeWeekStart()).toBe(1);
 await apply({region:'US',hourCycle:'24',firstDayOfWeek:'6'});expect(formatTime(date,{timeStyle:'short',timeZone:'UTC'})).toBe('21:05');expect(localeWeekStart()).toBe(6);
 clearSettings();expect(localeWeekStart()).toBe(0);expect(formatTime(date,{timeStyle:'short',timeZone:'UTC'})).toMatch(/^9:05\sPM$/);
});
