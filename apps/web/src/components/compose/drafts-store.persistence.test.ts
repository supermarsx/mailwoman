import {beforeEach,expect,it,vi} from 'vitest';
const disk=vi.hoisted(()=>({rows:new Map<string,unknown>(),allowed:true,block:false,gates:[] as Array<()=>void>,lists:0}));
vi.mock('../../offline/vault.ts',()=>({offlineVault:async()=>({
 async put(_namespace:string,key:string,value:unknown){if(disk.block)await new Promise<void>(resolve=>disk.gates.push(resolve));if(!disk.allowed)return false;disk.rows.set(key,structuredClone(value));return true;},
 async list(){disk.lists++;return [...disk.rows].map(([key,value])=>({key,value:structuredClone(value)}));},
 async get(_namespace:string,key:string){return disk.rows.get(key);},
 async remove(_namespace:string,key:string){disk.rows.delete(key);},async clear(){disk.rows.clear();},
})}));
beforeEach(()=>{vi.resetModules();disk.rows.clear();disk.allowed=true;disk.block=false;disk.gates=[];disk.lists=0;localStorage.clear();});
const draft=(bodyText:string)=>({id:'recover',accountId:'account',storageAuthority:'verified-test-authority',to:'unfinished@',subject:'Exact recovery',bodyText,bodyHtml:'<p>'+bodyText+'</p>',savedAt:1,serverStatus:'uncertain' as const});
it('persists the latest exact edit and recovers it after a fresh module load',async()=>{
 const first=await import('./drafts-store.ts');
 for(const value of ['A','A🙂','A🙂  \n','A🙂  \nNext line']){first.saveDraft(draft(value));expect(first.listDrafts('account','verified-test-authority')[0]?.bodyText).toBe(value);}
 await first.flushDrafts();expect(first.draftPersistenceState()).toBe('saved');expect(disk.lists).toBe(0);
 vi.resetModules();const reopened=await import('./drafts-store.ts');await reopened.hydrateDrafts();
 expect(reopened.listDrafts('account','verified-test-authority')[0]).toEqual(draft('A🙂  \nNext line'));
});
it('does not report recovery as saved when the browser refuses the write',async()=>{
 const store=await import('./drafts-store.ts');disk.allowed=false;store.saveDraft(draft('keep me in memory'));await store.flushDrafts();
 expect(store.draftPersistenceState()).toBe('unavailable');expect(disk.rows.size).toBe(0);expect(store.listDrafts('account','verified-test-authority')[0]?.bodyText).toBe('keep me in memory');
});

it('waits for edits queued while a recovery flush is already in progress',async()=>{
 const store=await import('./drafts-store.ts');disk.block=true;store.saveDraft(draft('first'));
 await vi.waitFor(()=>expect(disk.gates).toHaveLength(1));
 let finished=false;const flushing=store.flushDrafts().then(()=>{finished=true;});store.saveDraft(draft('last character'));
 disk.gates.shift()!();await vi.waitFor(()=>expect(disk.gates).toHaveLength(1));expect(finished).toBe(false);expect(store.draftPersistenceState()).toBe('saving');
 disk.gates.shift()!();await flushing;expect(store.draftPersistenceState()).toBe('saved');expect([...disk.rows.values()][0]).toEqual(draft('last character'));
});
