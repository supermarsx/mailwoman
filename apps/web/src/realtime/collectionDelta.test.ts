import {expect,it,vi} from 'vitest';
import type {Client} from '../api/client.ts';
import {collectionDelta} from './collectionDelta.ts';
const delta=(oldState:string,newState:string,created:string[],updated:string[],destroyed:string[],hasMoreChanges=false)=>({sessionState:'s',methodResponses:[['Mailbox/changes',{accountId:'a',oldState,newState,created,updated,destroyed,hasMoreChanges},'changes']]});
it('pages through cursors and reconciles create/update/destroy transitions',async()=>{
 const call=vi.fn().mockResolvedValueOnce(delta('1','2',['new'],['rename'],['gone'],true)).mockResolvedValueOnce(delta('2','3',[],['gone'],['new']));
 expect(await collectionDelta('Mailbox','a','1',[],call as Client['jmap'])).toEqual({state:'3',changed:['rename','gone'],destroyed:['new']});
 expect(call.mock.calls[1]?.[0].methodCalls[0][1].sinceState).toBe('2');
});
it('rejects an expired cursor, a different account and a nonadvancing cursor',async()=>{
 for(const response of [
  {sessionState:'s',methodResponses:[['error',{type:'cannotCalculateChanges'},'changes']]},
  delta('different','2',[],[],[]),delta('1','1',[],[],[],true),
 ])expect(await collectionDelta('Mailbox','a','1',[],vi.fn().mockResolvedValue(response) as Client['jmap'])).toBeUndefined();
});
