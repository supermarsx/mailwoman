import {afterEach,expect,it} from 'vitest';
import {MAX_WORKSPACE_TABS,parseWorkspace,readWorkspace,saveWorkspace,WORKSPACE_STORAGE_KEY} from './workspaceStorage.ts';
import {WORKSPACE_LAYOUTS} from './workspaceLayout.ts';
afterEach(()=>localStorage.clear());
const snapshot={version:2 as const,storageAuthority:'test-storage-authority-a',accountId:'a',tabs:[{id:'mail',kind:'messages' as const,pinned:true},{id:'module:contacts',kind:'module' as const,pinned:false,reference:'contacts'}],activeId:'module:contacts',layout:'tabs' as const};
it('preserves all five modes and only owned view assignments',()=>{
 for(const layout of WORKSPACE_LAYOUTS){saveWorkspace({...snapshot,layout,viewAssignments:['module:contacts',null]});expect(readWorkspace('a','test-storage-authority-a')).toEqual({...snapshot,layout,viewAssignments:['module:contacts',null]});}
 expect(parseWorkspace(JSON.stringify({...snapshot,viewAssignments:['foreign',null]}),'a','test-storage-authority-a')?.viewAssignments).toBeUndefined();
 expect(parseWorkspace(JSON.stringify({...snapshot,viewAssignments:['module:contacts','module:contacts']}),'a','test-storage-authority-a')?.viewAssignments).toBeUndefined();
});
it('stores only bounded account-owned metadata and drops unknown content fields',()=>{
 saveWorkspace({...snapshot,tabs:snapshot.tabs.map(tab=>({...tab,body:'secret',title:'private subject',url:'https://external.invalid'}))});
 expect(readWorkspace('a','test-storage-authority-a')).toEqual(snapshot);const raw=localStorage.getItem(WORKSPACE_STORAGE_KEY)!;expect(raw).not.toContain('secret');expect(raw).not.toContain('private');expect(raw).not.toContain('external');
 expect(readWorkspace('other','test-storage-authority-a')).toBeNull();expect(localStorage.getItem(WORKSPACE_STORAGE_KEY)).toBeNull();
});
it('rejects malformed, duplicate, unsupported and oversized references',()=>{
 for(const value of [{...snapshot,version:1},{...snapshot,tabs:[snapshot.tabs[0],snapshot.tabs[0]]},{...snapshot,tabs:[{id:'x',kind:'module',pinned:false,reference:'admin'}]},{...snapshot,tabs:Array.from({length:MAX_WORKSPACE_TABS+1},(_,i)=>({id:String(i),kind:'settings',pinned:false}))}])expect(parseWorkspace(JSON.stringify(value),'a','test-storage-authority-a')).toBeNull();
 expect(parseWorkspace('x'.repeat(65537),'a','test-storage-authority-a')).toBeNull();expect(parseWorkspace('{','a','test-storage-authority-a')).toBeNull();
});

it('refuses another realm even when the upstream account identifier is identical',()=>{
 saveWorkspace({...snapshot,accountId:'1'});expect(readWorkspace('1','different-storage-authority')).toBeNull();
 expect(parseWorkspace(JSON.stringify({...snapshot,version:1}),'a','test-storage-authority-a')).toBeNull();
});
