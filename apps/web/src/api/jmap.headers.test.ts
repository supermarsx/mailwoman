import {afterEach,expect,it,vi} from 'vitest';
const policy=vi.hoisted(()=>({values:{} as Record<string,{value:unknown}>}));
vi.mock('./settings.ts',()=>({settings:()=>policy.values}));
import {listMailbox,searchEmails,listHeaderProperties} from './jmap.ts';
afterEach(()=>{policy.values={};});
it('fetches envelope and delivery headers with folder and search rows without fetching bodies',()=>{
 for(const request of [listMailbox('a','inbox'),searchEmails('a',{text:'hello'})]){
  const args=request.methodCalls.find(call=>call[0]==='Email/get')![1];
  expect(args.properties).toEqual(expect.arrayContaining(['cc','bcc','replyTo','sentAt','messageId','references','header:Delivered-To:asAddresses:all']));
  expect(args.properties).not.toEqual(expect.arrayContaining(['htmlBody']));
  expect(args.fetchHTMLBodyValues).toBeUndefined();expect(args.fetchTextBodyValues).toBeUndefined();
 }
});
it('honors the effective user/admin header-preload policy without dropping required list metadata',()=>{
 policy.values={'mail.cache.preloadHeaders':{value:false}};
 expect(listHeaderProperties()).not.toContain('cc');
 expect(listHeaderProperties()).toEqual(expect.arrayContaining(['id','from','to','subject','mailboxIds']));
 expect(listMailbox('a','inbox').methodCalls[1]![1].properties).not.toContain('messageId');
});
