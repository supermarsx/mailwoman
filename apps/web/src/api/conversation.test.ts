import {describe,it,expect,vi} from 'vitest';
import type {Client} from './client.ts';
import type {JmapRequest,JmapResponse} from './jmap-types.ts';
import {conversationSnapshot,conversationPage,CONVERSATION_LIMIT} from './conversation.ts';
const signal=()=>new AbortController().signal;
function transport(result:Record<string,unknown>):Pick<Client,'jmap'>{return {jmap:vi.fn(async (request:JmapRequest):Promise<JmapResponse>=>({methodResponses:[[request.methodCalls[0]![0],result,request.methodCalls[0]![2]]],sessionState:'s'}))};}
describe('complete conversation retrieval',()=>{
 it('uses bounded pages in server thread order without requiring current-folder membership',async()=>{
  const ids=Array.from({length:71},(_,index)=>`e${index}`);const snapshot=await conversationSnapshot(transport({accountId:'a',list:[{id:'t',emailIds:ids}]}),'a','t',signal());
  const client=transport({accountId:'a',list:[{id:'e1',threadId:'t',mailboxIds:{archive:true}},{id:'e0',threadId:'t',mailboxIds:{inbox:true}}]});const result=await conversationPage(client,snapshot,0,signal());
  expect(result.messages.map(message=>message.id)).toEqual(['e0','e1']);expect(result.next).toBe(50);expect(result.missing).toBe(48);expect(client.jmap).toHaveBeenCalledWith(expect.objectContaining({methodCalls:[['Email/get',expect.objectContaining({ids:ids.slice(0,50)}),'members']]}),expect.anything());
 });
 it('rejects wrong-account, foreign-thread and duplicate membership responses',async()=>{
  await expect(conversationSnapshot(transport({accountId:'b',list:[{id:'t',emailIds:['x']}]}),'a','t',signal())).rejects.toThrow();
  await expect(conversationSnapshot(transport({accountId:'a',list:[{id:'t',emailIds:['x','x']}]}),'a','t',signal())).rejects.toThrow();
  const snapshot={accountId:'a',threadId:'t',ids:['x'],total:1};await expect(conversationPage(transport({accountId:'a',list:[{id:'x',threadId:'other'}]}),snapshot,0,signal())).rejects.toThrow();
 });
 it('reports the full count while bounding retained membership',async()=>{
  const snapshot=await conversationSnapshot(transport({accountId:'a',list:[{id:'t',emailIds:Array.from({length:CONVERSATION_LIMIT+1},(_,index)=>String(index))}]}),'a','t',signal());expect(snapshot.ids).toHaveLength(CONVERSATION_LIMIT);expect(snapshot.total).toBe(CONVERSATION_LIMIT+1);
 });
});
