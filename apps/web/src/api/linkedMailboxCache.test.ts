import {afterEach,expect,it,vi} from 'vitest';
import {clearLinkedMailboxCache,rememberLinkedMailbox,peekLinkedMailbox,linkedMailboxCachedPage} from './linkedMailboxCache.ts';
import {CAP_MAIL,type JmapSession,type JmapRequest} from './jmap-types.ts';
import {installLinkedMailbox} from './linkedMailboxSession.ts';
const request:JmapRequest={using:[],methodCalls:[['Email/query',{accountId:'child'},'q']]};
const snapshot=()=>({user:{accountId:'child',username:'child@example.org',storageAuthority:'child_authority'},discovery:{state:'s',primaryAccounts:{[CAP_MAIL]:'child'},accounts:{child:{}}} as unknown as JmapSession,context:{linkId:'link',accountId:'child',storageAuthority:'child_authority'},folders:{sessionState:'s',methodResponses:[]},page:{request,response:{sessionState:'s',methodResponses:[]}}});
afterEach(()=>{clearLinkedMailboxCache();vi.restoreAllMocks();});
it('isolates snapshots by primary authority, link, account, realm and query',()=>{
 rememberLinkedMailbox('owner','link',snapshot());
 expect(peekLinkedMailbox('owner','link','child','child_authority')).toBeDefined();
 for(const args of [['other','link','child','child_authority'],['owner','other','child','child_authority'],['owner','link','other','child_authority'],['owner','link','child','other']] as const)expect(peekLinkedMailbox(args[0],args[1],args[2],args[3])).toBeUndefined();
 expect(linkedMailboxCachedPage('owner','link','child','child_authority',request)).toBeDefined();
 expect(linkedMailboxCachedPage('owner','link','child','child_authority',{...request,methodCalls:[['Email/query',{accountId:'child',position:30},'q']]})).toBeUndefined();
 const held=peekLinkedMailbox('owner','link','child','child_authority')!;held.user.username='changed';expect(peekLinkedMailbox('owner','link','child','child_authority')!.user.username).toBe('child@example.org');
});
it('expires session snapshots and drops oversized entries',()=>{
 let now=100;vi.spyOn(Date,'now').mockImplementation(()=>now);rememberLinkedMailbox('owner','link',snapshot());now+=120001;expect(peekLinkedMailbox('owner','link','child','child_authority')).toBeUndefined();
 rememberLinkedMailbox('owner','huge',{...snapshot(),user:{...snapshot().user,username:'x'.repeat(2*1024*1024)}});expect(peekLinkedMailbox('owner','huge','child','child_authority')).toBeUndefined();
});
it('clears all snapshots when a primary sign-in boundary is installed',()=>{
 installLinkedMailbox(null);rememberLinkedMailbox('owner','link',snapshot());expect(peekLinkedMailbox('owner','link','child','child_authority')).toBeDefined();installLinkedMailbox(null);expect(peekLinkedMailbox('owner','link','child','child_authority')).toBeUndefined();
});

it('obeys the configured entry and byte limits',()=>{
 rememberLinkedMailbox('owner','one',snapshot(),{maxEntries:1,maxBytes:65536});rememberLinkedMailbox('owner','two',snapshot(),{maxEntries:1,maxBytes:65536});expect(peekLinkedMailbox('owner','one','child','child_authority')).toBeUndefined();expect(peekLinkedMailbox('owner','two','child','child_authority')).toBeDefined();
 rememberLinkedMailbox('owner','two',snapshot(),{maxEntries:1,maxBytes:1});expect(peekLinkedMailbox('owner','two','child','child_authority')).toBeUndefined();
});
