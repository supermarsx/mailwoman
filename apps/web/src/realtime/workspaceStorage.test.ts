import {afterEach,expect,it} from 'vitest';
import {MAX_WORKSPACE_TABS,parseWorkspace,readWorkspace,saveWorkspace,WORKSPACE_STORAGE_KEY} from './workspaceStorage.ts';
import {WORKSPACE_LAYOUTS} from './workspaceLayout.ts';
afterEach(()=>{localStorage.clear();sessionStorage.clear();});
const snapshot={version:2 as const,storageAuthority:'test-storage-authority-a',accountId:'a',tabs:[{id:'mail',kind:'messages' as const,pinned:true},{id:'module:contacts',kind:'module' as const,pinned:false,reference:'contacts'}],activeId:'module:contacts',layout:'tabs' as const};
it('stores folder IDs without row content and refuses another authority',()=>{
 const folder={id:'folder:a:inbox',kind:'folder' as const,pinned:false,reference:'inbox'};
 saveWorkspace({...snapshot,tabs:[...snapshot.tabs,folder]});
 expect(readWorkspace('a','test-storage-authority-a')?.tabs.at(-1)).toEqual(folder);
 expect(parseWorkspace(JSON.stringify({...snapshot,tabs:[{...folder,reference:undefined}]}),'a','test-storage-authority-a')).toBeNull();
 expect(readWorkspace('a','different-verified-authority')).toBeNull();
});
it('preserves all five modes and only owned view assignments',()=>{
 for(const layout of WORKSPACE_LAYOUTS){saveWorkspace({...snapshot,layout,viewAssignments:['module:contacts',null]});expect(readWorkspace('a','test-storage-authority-a')).toEqual({...snapshot,layout,viewAssignments:['module:contacts',null]});}
 expect(parseWorkspace(JSON.stringify({...snapshot,viewAssignments:['foreign',null]}),'a','test-storage-authority-a')?.viewAssignments).toBeUndefined();
 expect(parseWorkspace(JSON.stringify({...snapshot,viewAssignments:['module:contacts','module:contacts']}),'a','test-storage-authority-a')?.viewAssignments).toBeUndefined();
});
it('stores only bounded account-owned metadata and drops unknown content fields',()=>{
 saveWorkspace({...snapshot,tabs:snapshot.tabs.map(tab=>({...tab,body:'secret',title:'private subject',url:'https://external.invalid'}))});
 expect(readWorkspace('a','test-storage-authority-a')).toEqual(snapshot);const raw=localStorage.getItem(WORKSPACE_STORAGE_KEY)!;expect(raw).not.toContain('secret');expect(raw).not.toContain('private');expect(raw).not.toContain('external');
 expect(readWorkspace('other','test-storage-authority-a')).toBeNull();
 // t109: another mailbox's record is that mailbox's, not garbage: it stays.
 expect(localStorage.getItem(WORKSPACE_STORAGE_KEY)).not.toBeNull();
});
it('rejects malformed, duplicate, unsupported and oversized references',()=>{
 for(const value of [{...snapshot,version:1},{...snapshot,tabs:[snapshot.tabs[0],snapshot.tabs[0]]},{...snapshot,tabs:[{id:'x',kind:'module',pinned:false,reference:'admin'}]},{...snapshot,tabs:Array.from({length:MAX_WORKSPACE_TABS+1},(_,i)=>({id:String(i),kind:'settings',pinned:false}))}])expect(parseWorkspace(JSON.stringify(value),'a','test-storage-authority-a')).toBeNull();
 expect(parseWorkspace('x'.repeat(65537),'a','test-storage-authority-a')).toBeNull();expect(parseWorkspace('{','a','test-storage-authority-a')).toBeNull();
});

it('refuses another realm even when the upstream account identifier is identical',()=>{
 saveWorkspace({...snapshot,accountId:'1'});expect(readWorkspace('1','different-storage-authority')).toBeNull();
 expect(parseWorkspace(JSON.stringify({...snapshot,version:1}),'a','test-storage-authority-a')).toBeNull();
});

it('t109: keeps the tabs of each mailbox per browser tab, so a switch or reload in a linked mailbox finds its own',()=>{
 const linked={...snapshot,accountId:'linked',storageAuthority:'test-storage-authority-linked',tabs:[snapshot.tabs[0]!,{id:'message:linked:m1',kind:'messages' as const,pinned:false,reference:'m1'}],activeId:'message:linked:m1'};
 saveWorkspace(snapshot);saveWorkspace(linked);
 // The shared record is the last mailbox's; the primary's own tabs are still in this tab's session.
 expect(JSON.parse(localStorage.getItem(WORKSPACE_STORAGE_KEY)!).accountId).toBe('linked');
 expect(readWorkspace('a','test-storage-authority-a')).toEqual(snapshot);
 expect(readWorkspace('linked','test-storage-authority-linked')).toEqual(linked);
 // Never under another authority, even with the same account id.
 expect(readWorkspace('linked','test-storage-authority-a')).toBeNull();
 // A new browser tab (empty session) falls back to the shared record when it is this mailbox's.
 sessionStorage.clear();
 expect(readWorkspace('linked','test-storage-authority-linked')).toEqual(linked);
 expect(readWorkspace('a','test-storage-authority-a')).toBeNull();
});
it('t109: drops a malformed per-mailbox record and clears every mailbox on clearWorkspace',async()=>{
 const {clearWorkspace}=await import('./workspaceStorage.ts');
 sessionStorage.setItem(`${WORKSPACE_STORAGE_KEY}:a`,'{');
 expect(readWorkspace('a','test-storage-authority-a')).toBeNull();
 expect(sessionStorage.getItem(`${WORKSPACE_STORAGE_KEY}:a`)).toBeNull();
 saveWorkspace(snapshot);saveWorkspace({...snapshot,accountId:'b'});
 clearWorkspace();
 expect(sessionStorage.length).toBe(0);expect(localStorage.getItem(WORKSPACE_STORAGE_KEY)).toBeNull();
});
