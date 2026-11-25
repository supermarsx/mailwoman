import {beforeEach,expect,it,vi} from 'vitest';
import {configureRegionalSettings} from '../i18n/regional.ts';
import {formatMessageDate} from './messageDate.ts';
const policy=vi.hoisted(()=>({values:{} as Record<string,{value:unknown}>}));
vi.mock('../api/settings.ts',()=>({settings:()=>policy.values}));
const now=new Date('2026-06-15T12:00:00Z');
beforeEach(()=>{policy.values={};configureRegionalSettings(()=>policy.values);});
it('includes different years by default and keeps this year compact',()=>{
 expect(formatMessageDate('2025-06-01T12:00:00Z','UTC',now)).toContain('2025');
 expect(formatMessageDate('2027-06-01T12:00:00Z','UTC',now)).toContain('2027');
 expect(formatMessageDate('2026-06-01T12:00:00Z','UTC',now)).not.toContain('2026');
});
it('compares years in the configured display zone at the New Year boundary',()=>{
 const at=new Date('2026-01-01T00:30:00Z'),message='2025-12-31T23:30:00Z';
 expect(formatMessageDate(message,'UTC',at)).toContain('2025');
 expect(formatMessageDate(message,'America/New_York',at)).not.toContain('2025');
});
it('applies effective always/never and time visibility settings',()=>{
 policy.values={'time.messageYear':{value:'always'},'time.messageShowTime':{value:false}};
 expect(formatMessageDate('2026-06-01T12:34:00Z','UTC',now)).toContain('2026');
 expect(formatMessageDate('2026-06-01T12:34:00Z','UTC',now)).not.toContain(':');
 policy.values['time.messageYear']={value:'never'};
 expect(formatMessageDate('2025-06-01T12:34:00Z','UTC',now)).not.toContain('2025');
});
it('respects explicit regional patterns and rejects invalid timestamps',()=>{
 policy.values={'time.dateFormat':{value:'dd/MM/yyyy'},'time.messageYear':{value:'never'},'time.messageShowTime':{value:false}};
 expect(formatMessageDate('2025-06-01T12:34:00Z','UTC',now)).toBe('01/06/2025');
 expect(formatMessageDate('invalid','UTC',now)).toBe('');
});
