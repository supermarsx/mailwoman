import {webcrypto} from 'node:crypto';
import {beforeAll,expect,it,vi} from 'vitest';
import {EncryptedCache,memoryBackend} from './opfs.ts';
import {generateProfileKey} from './crypto.ts';
import {createOfflineVault,type VaultEntry} from './vault.ts';
beforeAll(()=>vi.stubGlobal('crypto',webcrypto));
async function fixture(){
 const records=new Map<string,VaultEntry>(),backend=memoryBackend();let now=1;
 const vault=createOfflineVault(new EncryptedCache(backend,await generateProfileKey()),{
  async all(namespace){return [...records.values()].filter(row=>row.namespace===namespace);},
  async group(group){return [...records.values()].filter(row=>row.group===group);},
  async pool(pool){return [...records.values()].filter(row=>row.pool===pool);},
  async put(row){records.set(row.id,row);},async remove(id){records.delete(id);},
 },()=>now);
 return {vault,records,backend,time(value:number){now=value;}};
}
it('stores authenticated encrypted records and isolates account namespaces',async()=>{
 const {vault,records,backend}=await fixture();
 await vault.put('account-a','draft-private',{subject:'secret subject'});
 expect(await vault.get('account-a','draft-private')).toEqual({subject:'secret subject'});
 expect(await vault.get('account-b','draft-private')).toBeUndefined();
 const id=[...records.keys()][0]!;
 expect(id).toMatch(/^[a-f0-9]{64}$/);
 expect(new TextDecoder().decode((await backend.read(`vault/${id}.enc`))!)).not.toContain('secret subject');
});
it('uses the configured byte budget and LRU eviction without evicting pins',async()=>{
 const {vault,time}=await fixture();
 const limits={maxBytes:10000,maxEntries:2};
 await vault.put('a','pin',{x:1},{...limits,pinned:true});time(2);
 await vault.put('a','old',{x:2},limits);time(3);
 await vault.put('a','new',{x:3},limits);
 expect(await vault.get('a','pin')).toEqual({x:1});
 expect(await vault.get('a','old')).toBeUndefined();
 expect(await vault.get('a','new')).toEqual({x:3});
 expect(await vault.put('a','too-big','x'.repeat(20000),limits)).toBe(false);
 expect((await vault.usage('a')).protectedBytes).toBeGreaterThan(0);
});
it('expires ordinary records while keeping flagged records and supports explicit purge',async()=>{
 const {vault,time}=await fixture(),limits={maxBytes:10000,retentionDays:1};
 await vault.put('a','ordinary','old',limits);
 await vault.put('a','flag','important',{...limits,flagged:true});time(2*86400000);
 expect(await vault.get('a','ordinary')).toBeUndefined();
 expect(await vault.get('a','flag')).toBe('important');
 await vault.clear('a');expect((await vault.usage('a')).entries).toBe(0);
});
it('manual eviction refuses overflow rather than discarding existing records',async()=>{
 const {vault}=await fixture(),limits={maxBytes:10000,maxEntries:1,eviction:'manual' as const};
 await vault.put('a','saved','keep',limits);
 expect(await vault.put('a','other','refuse',limits)).toBe(false);
 expect(await vault.get('a','saved')).toBe('keep');
});

it('enforces one device pool across linked accounts while keeping account purges separate',async()=>{
 const {vault}=await fixture();const a=JSON.stringify(['offline-account','https://mail.test','alice','a']),b=JSON.stringify(['offline-account','https://mail.test','bob','b']);
 const limits={maxBytes:20000,maxEntries:2};
 await vault.put(a+':headers','old','first',{...limits,budgetGroup:a});
 await vault.put(b+':headers','pin','second',{...limits,budgetGroup:b,pinned:true});
 await vault.put(a+':bodies','new','third',{...limits,budgetGroup:a});
 expect(await vault.get(a+':headers','old')).toBeUndefined();
 expect((await vault.usage(JSON.stringify(['offline-device','https://mail.test']))).entries).toBe(2);
 await vault.clear(a);expect(await vault.get(b+':headers','pin')).toBe('second');
});
