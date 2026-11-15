import {expect,it} from 'vitest';
import {calendarInstant,expandCachedCalendar} from './calendarExpansion.ts';
it('resolves DST overlaps to the earliest instant and rejects skipped local clocks',()=>{
 expect(new Date(calendarInstant('2026-11-01T01:30:00','America/New_York')).toISOString()).toBe('2026-11-01T05:30:00.000Z');
 expect(calendarInstant('2026-03-08T02:30:00','America/New_York')).toBeNaN();
});
it('keeps the event wall clock across daylight savings and applies exclusions and rescheduling',()=>{
 const event={id:'e',start:'2026-03-01T09:00:00',timeZone:'America/New_York',duration:'PT1H',recurrenceRules:[{rrule:'FREQ=WEEKLY;COUNT=4;BYDAY=SU'}],excludedRecurrenceDates:['2026-03-15'],recurrenceOverrides:{'2026-03-22T09:00:00':{start:'2026-03-22T10:00:00'}}};
 const rows=expandCachedCalendar([event],'2026-03-01T00:00:00Z','2026-04-01T00:00:00Z');
 expect(rows.map(row=>row['instanceStart'])).toEqual(['2026-03-01T14:00:00.000Z','2026-03-08T13:00:00.000Z','2026-03-22T14:00:00.000Z']);
});
it('supports ordinal BYDAY/BYSETPOS rules and preserves floating clock encoding',()=>{
 const rows=expandCachedCalendar([{id:'e',start:'2026-01-01T09:00:00',timeZone:null,duration:'PT30M',recurrenceRules:[{rrule:'FREQ=MONTHLY;BYDAY=MO,TU,WE,TH,FR;BYSETPOS=-1;COUNT=3'}]}],'2026-01-01T00:00:00Z','2026-04-01T00:00:00Z');
 expect(rows.map(row=>row['instanceStart'])).toEqual(['2026-01-30T09:00:00.000Z','2026-02-27T09:00:00.000Z','2026-03-31T09:00:00.000Z']);
});
