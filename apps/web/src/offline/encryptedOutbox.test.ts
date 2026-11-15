import {webcrypto} from 'node:crypto';
import {beforeAll,expect,it,vi} from 'vitest';
import {EncryptedCache,memoryBackend} from './opfs.ts';
import {generateProfileKey} from './crypto.ts';
import {createOfflineVault,type VaultEntry} from './vault.ts';
import {memoryOutboxStore} from './outbox.ts';
import {encryptedOutboxStore} from './encryptedOutbox.ts';
import type {OutboundItem} from '../contracts/offline.ts';
beforeAll(()=>vi.stubGlobal('crypto',webcrypto));
async function fixture(){
 const rows=new Map<string,VaultEntry>(),backend=memoryBackend();
 const vault=createOfflineVault(new EncryptedCache(backend,await generateProfileKey()),{
  async all(namespace){return [...rows.values()].filter(row=>row.namespace===namespace);},
  async put(row){rows.set(row.id,row);},async remove(id){rows.delete(id);},
 });return {vault,rows,backend};
}
const item={id:'queued-1',type:'send',createdAt:1,state:'queued',storageAuthority:'test_verified_owner_123456',payload:{accountId:'a',draft:{subject:'Sensitive queued subject',textBody:'Private body'}}} as OutboundItem;
it('encrypts legacy queue data before deleting the plaintext copy',async()=>{
 const {vault,rows,backend}=await fixture(),legacy=memoryOutboxStore();await legacy.add(item);
 const queue=encryptedOutboxStore(legacy,async()=>vault);
 expect(await queue.all()).toEqual([item]);expect(await legacy.all()).toEqual([]);
 const id=[...rows.keys()][0]!;
 expect(new TextDecoder().decode((await backend.read(`vault/${id}.enc`))!)).not.toContain('Sensitive queued subject');
 const reopened=encryptedOutboxStore(legacy,async()=>vault);
 expect(await reopened.all()).toEqual([item]);
 await reopened.put({...item,state:'failed'});expect((await reopened.all())[0]?.state).toBe('failed');
 await reopened.delete(item.id);expect(await reopened.all()).toEqual([]);
});
it('preserves the original queue when encryption or storage fails',async()=>{
 const legacy=memoryOutboxStore();await legacy.add(item);
 const queue=encryptedOutboxStore(legacy,async()=>{throw new Error('storage unavailable');});
 await expect(queue.all()).rejects.toThrow('storage unavailable');
 expect(await legacy.all()).toEqual([item]);
});
