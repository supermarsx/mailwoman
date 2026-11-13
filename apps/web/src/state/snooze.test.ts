import {afterEach,expect,it} from 'vitest';
import {clearSettings,loadSettings} from '../api/settings.ts';
import {configuredSnoozePresets,DEFAULT_SNOOZE_PRESETS,resolveSnooze,SNOOZE_PRESETS_KEY,snoozeWallInput,validSnoozePresets,type SnoozePreset} from './snooze.ts';
afterEach(()=>clearSettings());
const rule=(kind:SnoozePreset['kind'],extra:Partial<SnoozePreset>={}):SnoozePreset=>({id:'test',kind,enabled:true,...extra});
it('uses elapsed hours but zoned calendar days across daylight saving',()=>{
 const now=new Date('2026-03-28T09:00:00Z');
 expect(resolveSnooze(rule('hours',{amount:24}),now,'Europe/London',1)).toBe('2026-03-29T09:00:00.000Z');
 expect(resolveSnooze(rule('days',{amount:1,time:'09:00'}),now,'Europe/London',1)).toBe('2026-03-29T08:00:00.000Z');
 expect(snoozeWallInput(now,'Asia/Kolkata')).toBe('2026-03-28T14:30');
});
it('resolves regional weeks, leap month ends and year rollover into the future',()=>{
 expect(resolveSnooze(rule('end-month',{time:'17:00'}),new Date('2028-02-28T18:00Z'),'UTC',1)).toBe('2028-02-29T17:00:00.000Z');
 expect(resolveSnooze(rule('end-month',{time:'17:00'}),new Date('2028-02-29T18:00Z'),'UTC',1)).toBe('2028-03-31T17:00:00.000Z');
 expect(resolveSnooze(rule('next-month',{time:'09:00'}),new Date('2026-12-31T22:00Z'),'UTC',1)).toBe('2027-01-01T09:00:00.000Z');
 expect(resolveSnooze(rule('end-week',{time:'17:00'}),new Date('2026-11-13T12:00Z'),'UTC',1)).toBe('2026-11-15T17:00:00.000Z');
 expect(resolveSnooze(rule('end-week',{time:'17:00'}),new Date('2026-11-13T12:00Z'),'UTC',0)).toBe('2026-11-14T17:00:00.000Z');
 expect(resolveSnooze(rule('next-week',{time:'09:00'}),new Date('2026-11-13T12:00Z'),'UTC',1)).toBe('2026-11-16T09:00:00.000Z');
});
it('omits nonexistent wall times and chooses a future occurrence of repeated wall times',()=>{
 expect(resolveSnooze(rule('tomorrow',{time:'01:30'}),new Date('2026-03-28T12:00Z'),'Europe/London',1)).toBeNull();
 expect(resolveSnooze(rule('tomorrow',{time:'01:30'}),new Date('2026-10-24T12:00Z'),'Europe/London',1)).toBe('2026-10-25T00:30:00.000Z');
});
it('validates bounded rules and preserves an explicitly empty menu',async()=>{
 expect(validSnoozePresets(DEFAULT_SNOOZE_PRESETS)).toBe(true);
 for(const value of [[rule('hours',{amount:0})],[rule('days',{amount:1,time:'24:00'})],[rule('hours',{amount:2,label:'\n'})],[rule('hours',{amount:2}),rule('hours',{amount:3})]])expect(validSnoozePresets(value)).toBe(false);
 await loadSettings({fetcher:async()=>new Response(JSON.stringify({settings:{[SNOOZE_PRESETS_KEY]:{value:[],source:'user',locked:false}}}))});
 expect(configuredSnoozePresets()).toEqual([]);
});
