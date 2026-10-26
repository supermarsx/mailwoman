import {beforeEach,expect,it,vi} from 'vitest';
import {accountInsertService} from './accountInsert.ts';
import {activateAccountPreferences,preferenceAccount,suspendAccountPreferences} from '../../api/accountPreferenceLifecycle.ts';
import {CAP_CORE,CAP_MAIL} from '../../api/jmap-types.ts';
import {CAP_CONTACTS} from '../../api/pim-types.ts';
import {draftToCard} from '../../state/slices/contacts.ts';
const signal=()=>new AbortController().signal;
const response=(method:string,value:unknown,id='item')=>new Response(JSON.stringify({methodResponses:[[method,value,id]]}));
beforeEach(()=>{suspendAccountPreferences();activateAccountPreferences('a','verified-authority-a');});
it('offers only capabilities advertised for this exact account',async()=>{
 const fetcher=vi.fn(async(_url:string,_init?:RequestInit)=>new Response(JSON.stringify({capabilities:{[CAP_CORE]:{},[CAP_MAIL]:{},[CAP_CONTACTS]:{}},accounts:{a:{accountCapabilities:{[CAP_MAIL]:{}}},b:{accountCapabilities:{[CAP_CONTACTS]:{}}}}})));
 expect(await accountInsertService(preferenceAccount()!,fetcher).capabilities(signal())).toEqual(['message']);
 expect(new Headers(fetcher.mock.calls[0]?.[1]?.headers).get('x-mailwoman-account')).toBe('a');
});
it('fetches actual event ICS and refuses a mismatched account response',async()=>{
 const fetcher=vi.fn(async(_url:string,init?:RequestInit)=>{const request=JSON.parse(String(init?.body));expect(request.methodCalls[0]).toEqual(['CalendarEvent/export',{accountId:'a',ids:['e1']},'export']);return response('CalendarEvent/export',{accountId:'a',blob:'BEGIN:VCALENDAR\r\nEND:VCALENDAR'},'export');});
 const file=await accountInsertService(preferenceAccount()!,fetcher).file('event',{id:'e1',label:'Meeting'},signal());expect(file.name).toBe('Meeting.ics');expect(file.type).toBe('text/calendar');
 const foreign=accountInsertService(preferenceAccount()!,async()=>response('CalendarEvent/export',{accountId:'b',blob:'BEGIN:VCALENDAR'},'export'));
 await expect(foreign.file('event',{id:'e1',label:'Meeting'},signal())).rejects.toThrow('invalidExport');
});
it('exports one fresh owned contact as a real vCard',async()=>{
 const card=draftToCard('c1','book',{name:{full:'Alice',given:'Alice',surname:'',prefix:'',suffix:''}});
 const fetcher=vi.fn(async()=>response('ContactCard/get',{accountId:'a',list:[card]}));
 const file=await accountInsertService(preferenceAccount()!,fetcher).file('contact',{id:'c1',label:'Alice'},signal());expect(file.name).toBe('Alice.vcf');expect(file.type).toBe('text/vcard');expect(file.size).toBeGreaterThan(30);
});
it('uses policy-checked original EML export and preserves refusal',async()=>{
 const fetcher=vi.fn(async(_url:string)=>new Response('From: sender@example.test\r\n\r\nOriginal'));
 const file=await accountInsertService(preferenceAccount()!,fetcher).file('message',{id:'m/1',label:'Report'},signal());expect(file.type).toBe('message/rfc822');expect(fetcher.mock.calls[0]?.[0]).toBe('/api/export/m%2F1?format=eml');
 await expect(accountInsertService(preferenceAccount()!,async()=>new Response('',{status:403})).file('message',{id:'m',label:'x'},signal())).rejects.toThrow('requestRefused');
});
it('discards an export returned after authority replacement',async()=>{
 const service=accountInsertService(preferenceAccount()!,async()=>{suspendAccountPreferences();activateAccountPreferences('a','other-authority-a');return new Response('private');});
 await expect(service.file('message',{id:'m',label:'x'},signal())).rejects.toThrow('accountChanged');
});
it('refuses partial whole-calendar exports before export request',async()=>{
 const fetcher=vi.fn(async()=>response('CalendarEvent/query',{accountId:'a',ids:['e'],total:1001}));
 await expect(accountInsertService(preferenceAccount()!,fetcher).file('calendar',{id:'cal',label:'Calendar'},signal())).rejects.toThrow('tooManyEvents');expect(fetcher).toHaveBeenCalledTimes(1);
});
it('bounds the picker list and reports truncation instead of claiming complete search',async()=>{
 const ids=Array.from({length:101},(_,i)=>String(i));const fetcher=vi.fn(async(_url:string,init?:RequestInit)=>{const [name,args]=JSON.parse(String(init?.body)).methodCalls[0];return name==='Email/query'?response(name,{accountId:'a',ids,total:500}):response(name,{accountId:'a',list:args.ids.map((id:string)=>({id,subject:'Message '+id}))});});
 const page=await accountInsertService(preferenceAccount()!,fetcher).list('message',signal());expect(page.items).toHaveLength(100);expect(page.limited).toBe(true);
});
