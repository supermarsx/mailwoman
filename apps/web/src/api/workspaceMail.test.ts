import {afterEach,expect,it,vi} from 'vitest';
import {activateAccountPreferences,preferenceAccount,suspendAccountPreferences} from './accountPreferenceLifecycle.ts';
import {apiFetch} from './http.ts';
import {transferWorkspaceMessage,workspaceMessages,updateWorkspaceMessages,type WorkspaceMailbox} from './workspaceMail.ts';
import type {Email,Mailbox} from './jmap-types.ts';

vi.mock('./http.ts',()=>({apiFetch:vi.fn()}));
const mock=vi.mocked(apiFetch);
it('keeps query totals, unread totals and keyword writes bound to their source account',async()=>{
 activateAccountPreferences('account','primary_authority_1234');const owner=preferenceAccount()!;
 const source:WorkspaceMailbox={linkId:'linked-b',accountId:'b',storageAuthority:'linked_authority_12345',owner,label:'Other',current:false};
 const boxes=[{id:'inbox',role:'inbox',myRights:{maySetKeywords:true,maySetSeen:true,mayRemoveItems:true}},{id:'trash',role:'trash',myRights:{mayAddItems:true}}] as unknown as Mailbox[];
 mock.mockImplementation(async(_url,init)=>{expect(init?.mailboxTarget).toBe(source);const request=JSON.parse(String(init?.body));
  if(request.methodCalls[0][0]==='Email/set'){expect(request.methodCalls[0][1]).toMatchObject({accountId:'b',update:{same:{'keywords/$seen':true}}});return Response.json({methodResponses:[['Email/set',{updated:{same:null},notUpdated:{refused:{type:'forbidden'}}},'update']]});}
  expect(request.methodCalls[0][1]).toMatchObject({accountId:'b',calculateTotal:true,sort:[{property:'subject',isAscending:true},{property:'receivedAt',isAscending:false}]});
  return Response.json({methodResponses:[['Email/query',{ids:['same'],position:0,total:7},'query'],['Email/get',{list:[{id:'same',mailboxIds:{inbox:true},keywords:{}}]},'rows'],['Email/query',{total:4},'unread']]});
 });
 const result=await workspaceMessages(source,boxes,'inbox','',0,new AbortController().signal,'subject');expect(result).toMatchObject({total:7,unread:4,next:1});
 expect(await updateWorkspaceMessages(source,boxes,[{id:'same',mailboxIds:{inbox:true}} as unknown as Email],'read',new AbortController().signal)).toEqual(['same']);
});
afterEach(()=>{mock.mockReset();suspendAccountPreferences();});

function scenario(receipt:object,moveConfirmed=true){
 activateAccountPreferences('a','primary_authority_1234');
 const owner=preferenceAccount()!;
 const source:WorkspaceMailbox={linkId:null,accountId:'a',storageAuthority:'primary_authority_1234',owner,label:'Primary',current:true};
 const target:WorkspaceMailbox={linkId:'linked-b',accountId:'b',storageAuthority:'linked_authority_12345',owner,label:'Other',current:false};
 const mutations:string[]=[];
 mock.mockImplementation(async (url,init)=>{
  const mailbox=init?.mailboxTarget;
  if(url.includes('/api/import/formats'))return Response.json({engineMode:true,available:true,maxBytes:1024});
  if(url.includes('/api/export/')){
   expect(mailbox).toBe(source);
   return new Response('Subject: Test\r\n\r\nBody');
  }
  if(url.includes('/api/import/eml')){
   expect(mailbox).toBe(target);
   mutations.push('import');
   return Response.json(receipt);
  }
  const request=JSON.parse(String(init?.body)) as {methodCalls:[string,Record<string,unknown>,string][]};
  const [method,args,id]=request.methodCalls[0]!;
  expect(args['accountId']).toBe(mailbox?.accountId);
  if(method==='Mailbox/get')return Response.json({methodResponses:[[method,{list:mailbox===source?[
   {id:'inbox',role:'inbox',myRights:{mayRemoveItems:true}},
   {id:'trash',role:'trash',myRights:{mayAddItems:true}},
  ]:[{id:'work',myRights:{mayAddItems:true}}]},id]]});
  if(method==='Email/get')return Response.json({methodResponses:[[method,{state:'s1',list:[{id:'m',blobId:'raw',mailboxIds:{inbox:true}}]},id]]});
  expect(mailbox).toBe(source);
  expect(args).toMatchObject({ifInState:'s1',update:{m:{mailboxIds:{trash:true}}}});
  mutations.push('trash');
  return Response.json({methodResponses:[[method,moveConfirmed?{updated:{m:null}}:{notUpdated:{m:{type:'stateMismatch'}}},id]]});
 });
 const run=()=>transferWorkspaceMessage({source,target,emailId:'m',folderId:'work',move:true,signal:new AbortController().signal,allowed:()=>true});
 return {run,mutations};
}

it('moves the original only after the other mailbox confirms the durable import',async()=>{
 const {run,mutations}=scenario({imported:1,failed:0,total:1,mailboxId:'work'});
 await expect(run()).resolves.toBe('moved');
 expect(mutations).toEqual(['import','trash']);
});

it('leaves the source alone when the import acknowledgement is ambiguous',async()=>{
 const {run,mutations}=scenario({imported:1,failed:0,total:1,mailboxId:'unexpected'});
 await expect(run()).resolves.toBe('import-unconfirmed');
 expect(mutations).toEqual(['import']);
});

it('reports a confirmed copy separately from an unconfirmed source update',async()=>{
 const {run,mutations}=scenario({imported:1,failed:0,total:1,mailboxId:'work'},false);
 await expect(run()).resolves.toBe('source-unconfirmed');
 expect(mutations).toEqual(['import','trash']);
});
