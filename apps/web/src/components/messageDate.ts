import {settings} from '../api/settings.ts';
import {formatDateWith} from '../i18n/format.ts';
import {calendarDateInZone} from '../i18n/timezones.ts';

/** Compact message timestamps follow the effective user/admin regional policy. */
export function formatMessageDate(iso:string,timeZone:string,now:Date=new Date()):string {
 const date=new Date(iso);if(Number.isNaN(date.getTime()))return '';
 const year=settings()['time.messageYear']?.value??'auto';
 const includeYear=year==='always'||year!=='never'&&calendarDateInZone(date,timeZone).slice(0,4)!==calendarDateInZone(now,timeZone).slice(0,4);
 const showTime=settings()['time.messageShowTime']?.value!==false;
 return formatDateWith(date,{month:'short',day:'numeric',...(includeYear?{year:'numeric'}:{}),...(showTime?{hour:'2-digit',minute:'2-digit'}:{}),timeZone});
}
