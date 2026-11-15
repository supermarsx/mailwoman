import {webcrypto} from 'node:crypto';
import {beforeEach,expect,it,vi} from 'vitest';
import {ApiError,NetworkError,type Client,type Me} from '../api/client.ts';
import {CAP_MAIL,type JmapRequest,type JmapResponse,type JmapSession} from '../api/jmap-types.ts';
import {withOfflineClient} from './client.ts';
import {configureOfflinePolicy,DEFAULT_OFFLINE_POLICY,isOffline,reportOffline} from './runtime.ts';
import {EncryptedCache,memoryBackend} from './opfs.ts';
import {createOfflineVault,type VaultEntry} from './vault.ts';
import {generateProfileKey} from './crypto.ts';

const me={username:'alice@example.test',accountId:'a',storageAuthority:'verified_owner_alice_123456'} as Me;
const session={accounts:{a:{name:'Alice'}},primaryAccounts:{[CAP_MAIL]:'a'},state:'s',downloadUrl:'/download',capabilities:{}} as unknown as JmapSession;
const request={using:[CAP_MAIL],methodCalls:[['Email/query',{accountId:'a',filter:{inMailbox:'inbox'},limit:50},'q'],['Email/get',{accountId:'a','#ids':{resultOf:'q',name:'Email/query',path:'/ids'},properties:['id','subject']},'g']]} as JmapRequest;
const response={sessionState:'s',methodResponses:[['Email/query',{accountId:'a',queryState:'q1',ids:['m1'],position:0,total:1},'q'],['Email/get',{accountId:'a',state:'e1',list:[{id:'m1',subject:'Saved offline',receivedAt:'2026-11-15T00:00:00Z',mailboxIds:{inbox:true},keywords:{}}],notFound:[]},'g']]} as JmapResponse;
async function storage(){
 const rows=new Map<string,VaultEntry>();
 return createOfflineVault(new EncryptedCache(memoryBackend(),await generateProfileKey()),{
  async all(namespace){return [...rows.values()].filter(row=>row.namespace===namespace);},
  async group(group){return [...rows.values()].filter(row=>row.group===group);},
  async put(row){rows.set(row.id,row);},async remove(id){rows.delete(id);},
 });
}
function raw(){
 return {me:vi.fn(async()=>me),session:vi.fn(async()=>session),jmap:vi.fn(async()=>response),onNetwork:()=>()=>undefined,login:vi.fn(async()=>me),logout:vi.fn(async()=>undefined)} as unknown as Client;
}
beforeEach(()=>{vi.stubGlobal('crypto',webcrypto);configureOfflinePolicy({...DEFAULT_OFFLINE_POLICY,enabled:true});reportOffline(false);});
it('cold starts from encrypted identity/session/mail snapshots without authenticating offline writes',async()=>{
 const vault=await storage(),online=raw(),first=withOfflineClient(online,()=>'/test',async()=>vault);
 await first.me();await first.session();await first.jmap(request);
 const down=raw();vi.mocked(down.me).mockRejectedValue(new NetworkError('offline'));
 const reloaded=withOfflineClient(down,()=>'/test',async()=>vault);
 expect(await reloaded.me()).toEqual(me);expect(isOffline()).toBe(true);
 expect(await reloaded.session()).toEqual(session);
 const cached=await reloaded.jmap(request);
 expect(cached.methodResponses[1]?.[1]['list']).toEqual([expect.objectContaining({subject:'Saved offline'})]);
 await expect(reloaded.jmap({using:[CAP_MAIL],methodCalls:[['Email/set',{accountId:'a',destroy:['m1']},'set']]})).rejects.toBeInstanceOf(NetworkError);
 expect(down.jmap).not.toHaveBeenCalled();
});
it('revalidates the server principal before allowing queued writes on recovery',async()=>{
 const vault=await storage(),first=withOfflineClient(raw(),()=>'/changed',async()=>vault);
 await first.me();const down=raw();vi.mocked(down.me).mockRejectedValueOnce(new NetworkError('offline'));
 const client=withOfflineClient(down,()=>'/changed',async()=>vault);await client.me();
 vi.mocked(down.me).mockResolvedValue({...me,storageAuthority:'different_principal_123456'});
 await expect(client.jmap({using:[CAP_MAIL],methodCalls:[['Email/set',{accountId:'a',destroy:['m1']},'set']]})).rejects.toMatchObject({status:401});
 expect(down.jmap).not.toHaveBeenCalled();
});
it('never turns an explicit session refusal into offline access',async()=>{
 const vault=await storage(),first=withOfflineClient(raw(),()=>'/revoked',async()=>vault);await first.me();
 const denied=raw();vi.mocked(denied.me).mockRejectedValue(new ApiError(401,'revoked'));
 const client=withOfflineClient(denied,()=>'/revoked',async()=>vault);
 await expect(client.me()).rejects.toMatchObject({status:401});
 vi.mocked(denied.me).mockRejectedValue(new NetworkError('offline'));
 await expect(client.me()).rejects.toBeInstanceOf(NetworkError);
});
it('keeps separate JMAP accounts with colliding message ids isolated',async()=>{
 const vault=await storage(),api=raw(),client=withOfflineClient(api,()=>'/shared',async()=>vault);await client.me();
 await client.jmap(request);
 vi.mocked(api.jmap).mockResolvedValue({sessionState:'s',methodResponses:[['Email/get',{accountId:'other',state:'e2',list:[{id:'m1',subject:'Other mailbox'}]},'g']]});
 await client.jmap({using:[CAP_MAIL],methodCalls:[['Email/get',{accountId:'other',ids:['m1']},'g']]});
 vi.mocked(api.me).mockRejectedValue(new NetworkError('offline'));await client.me();
 const cached=await client.jmap(request);
 expect(cached.methodResponses[1]?.[1]['list']).toEqual([expect.objectContaining({subject:'Saved offline'})]);
});
