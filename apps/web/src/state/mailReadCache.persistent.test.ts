import {expect,it,vi} from 'vitest';
import {createMailReadCache} from './mailReadCache.ts';
import {DEFAULT_MAIL_CACHE_POLICY} from './mailCachePolicy.ts';
import type {PersistentMailStore} from '../offline/persistentMail.ts';
import type {PreferenceAccount} from '../api/accountPreferenceLifecycle.ts';

function setup(enabled=true){
 let owner:PreferenceAccount={id:'a',generation:1,storageAuthority:'realm-a'};
 let policy={...DEFAULT_MAIL_CACHE_POLICY,keepIndefinitely:enabled};
 const rows=new Map<string,string>();
 const id=(o:PreferenceAccount,key:string)=>JSON.stringify([o.storageAuthority,o.id,key]);
 const store:PersistentMailStore<string>={get:vi.fn(async(o,key)=>rows.get(id(o,key))),put:vi.fn(async(o,key,_kind,value)=>{rows.set(id(o,key),value);}),clear:vi.fn(async(o,predicate)=>{for(const key of rows.keys()){const [realm,account,k]=JSON.parse(key) as string[];if(realm===o.storageAuthority&&account===o.id&&(!predicate||predicate(k!,'message')))rows.delete(key);}})};
 const make=()=>createMailReadCache<string>({policy:()=>policy,currentOwner:()=>owner,persistent:store});
 return {make,owner:()=>owner,store,rows,policy:(on:boolean)=>{policy={...policy,keepIndefinitely:on};},switch:()=>{owner={id:'a',generation:2,storageAuthority:'realm-b'};}};
}
it('never touches durable storage unless explicitly enabled',async()=>{
 const s=setup(false),cache=s.make();await cache.load(s.owner(),'mail','message',async()=> 'secret');
 expect(await cache.restore(s.owner(),'mail','message')).toBeUndefined();expect(s.store.put).not.toHaveBeenCalled();expect(s.store.get).not.toHaveBeenCalled();
});
it('survives disposal and a new session without age expiry, but still loads fresh online data',async()=>{
 const s=setup(),first=s.make();await first.load(s.owner(),'mail','message',async()=> 'saved');first.dispose();
 const second=s.make();expect(await second.restore(s.owner(),'mail','message')).toBe('saved');
 expect(await second.load(s.owner(),'mail','message',async()=> 'fresh')).toBe('fresh');expect(s.store.clear).not.toHaveBeenCalled();
});
it('does not restore a different storage authority or late results from a switched account',async()=>{
 const s=setup(),cache=s.make();await cache.load(s.owner(),'mail','message',async()=> 'secret');
 let resolve!:(v:string)=>void;vi.mocked(s.store.get).mockImplementationOnce(()=>new Promise(r=>{resolve=r;}));
 const result=cache.restore(s.owner(),'mail','message');s.switch();resolve('secret');expect(await result).toBeUndefined();
 expect(await cache.restore(s.owner(),'mail','message')).toBeUndefined();
});
it('removes saved copies when disabled and fences a restore racing manual purge',async()=>{
 const s=setup(),cache=s.make();await cache.load(s.owner(),'mail','message',async()=> 'secret');
 let resolve!:(v:string)=>void;vi.mocked(s.store.get).mockImplementationOnce(()=>new Promise(r=>{resolve=r;}));
 const result=cache.restore(s.owner(),'mail','message');await cache.purge();resolve('secret');expect(await result).toBeUndefined();expect(s.rows.size).toBe(0);
 await cache.load(s.owner(),'mail','message',async()=> 'new');s.policy(false);cache.syncPolicy();await Promise.resolve();expect(s.rows.size).toBe(0);
});
it('handles blocked storage as a cache miss without breaking an online read',async()=>{
 const s=setup(),cache=s.make();vi.mocked(s.store.put).mockRejectedValue(new Error('quota'));vi.mocked(s.store.get).mockRejectedValue(new Error('blocked'));
 await expect(cache.load(s.owner(),'mail','message',async()=> 'online')).resolves.toBe('online');expect(await cache.restore(s.owner(),'mail','message')).toBeUndefined();
});
