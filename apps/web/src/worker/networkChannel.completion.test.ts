import {afterEach,expect,it,vi} from 'vitest';
import {createChannelNetworkConnection} from './networkChannel.ts';
class Channel {
 static latest:Channel;onmessage:((event:{data:unknown})=>void)|null=null;sent:unknown[]=[];
 constructor(_name:string){Channel.latest=this;}postMessage(value:unknown){this.sent.push(value);}close(){}
 receive(data:unknown){this.onmessage?.({data});}
}
afterEach(()=>{window.dispatchEvent(new Event('pagehide'));vi.useRealTimers();vi.unstubAllGlobals();});
it('does not replay an acknowledged-by-neither-coordinator keyed JMAP mutation on leader replacement',async()=>{
 vi.useFakeTimers();vi.stubGlobal('BroadcastChannel',Channel);vi.stubGlobal('navigator',{locks:{request:vi.fn(async()=>{})}});
 const context={base:location.origin,linkId:null};const connection=createChannelNetworkConnection(context)!;
 Channel.latest.receive({kind:'leader',sender:'first'});expect(await connection.ready).toBe(true);
 const request=connection.call('jmap',context,{idempotencyKey:'same-user-action',body:{using:[],methodCalls:[['Email/set',{accountId:'a',update:{m:{'keywords/$seen':true}}},'set']]}});
 const rejected=expect(request).rejects.toMatchObject({code:'outcomeUnknown'});
 const asks=()=>Channel.latest.sent.filter(row=>(row as {kind?:string}).kind==='ask');expect(asks()).toHaveLength(1);
 Channel.latest.receive({kind:'leader',sender:'second'});await rejected;expect(asks()).toHaveLength(1);
});
