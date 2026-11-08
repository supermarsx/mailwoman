import {describe,it,expect} from 'vitest';
import {createRoot,createMemo,createSignal} from 'solid-js';
import type {SettingEntry} from '../api/settings.ts';
import type {AccountUiPreferences} from '../api/accountPreferences.ts';
import {visibleSidebarFolders} from './folderVisibility.ts';
const box=(id:string,totalEmails?:number,parentId:string|null=null,role:string|null=null)=>({id,name:id,parentId,role,...(totalEmails===undefined?{}:{totalEmails})});
const read=(value:boolean,locked=false)=>(key:string):SettingEntry|undefined=>key==='folders.hideEmpty'?{value,locked,source:locked?'global':'user'}:undefined;
type TestFolder=ReturnType<typeof box>&{unreadEmails?:number;mailwomanSearchQuery?:string};
const ids=(boxes:TestFolder[],prefs:AccountUiPreferences={},value=true,locked=false)=>visibleSidebarFolders(boxes,prefs,read(value,locked)).map(x=>x.id);

describe('empty folder navigation',()=>{
 it('is opt-in and keeps the Inbox even when empty',()=>{
  const list=[box('inbox',0,null,'inbox'),box('spam',0,null,'junk'),box('drafts',0,null,'drafts')];
  expect(ids(list,{},false)).toEqual(['inbox','spam','drafts']);expect(ids(list)).toEqual(['inbox']);
 });
 it('uses total messages rather than unread and preserves unknown or inconsistent counts',()=>{
  expect(ids([box('read',4),box('unknown'),{...box('stale',0),unreadEmails:1},box('empty',0)])).toEqual(['read','unknown','stale']);
 });
 it('allows folder-specific always-show and hide-when-empty independently of the default',()=>{
  const list=[box('spam',0),box('drafts',0)];
  expect(ids(list,{folderProps:{spam:{hideWhenEmpty:false}}})).toEqual(['spam']);
  expect(ids(list,{folderProps:{spam:{hideWhenEmpty:true}}},false)).toEqual(['drafts']);
 });
 it('respects administrator locks in both directions',()=>{
  expect(ids([box('spam',0)],{folderProps:{spam:{hideWhenEmpty:false}}},true,true)).toEqual([]);
  expect(ids([box('spam',0)],{folderProps:{spam:{hideWhenEmpty:true}}},false,true)).toEqual(['spam']);
 });
 it('ignores per-folder overrides when administrator disables that override',()=>{
  const list=[box('spam',0)];
  expect(visibleSidebarFolders(list,{folderProps:{spam:{hideWhenEmpty:false}}},read(true),[])).toEqual([]);
 });
 it('keeps empty ancestors of nonempty or explicitly visible descendants',()=>{
  const list=[box('parent',0),box('child',2,'parent'),box('other',0)];
  expect(ids(list)).toEqual(['parent','child']);
  expect(ids([box('parent',0),box('child',0,'parent')],{folderProps:{child:{hideWhenEmpty:false}}})).toEqual(['parent','child']);
 });
 it('hides entirely empty branches and terminates malformed cycles',()=>{
  expect(ids([box('parent',0),box('child',0,'parent')])).toEqual([]);
  expect(ids([box('a',0,'b'),box('b',1,'a')])).toEqual(['a','b']);
 });
 it('retains virtual searches whose placeholder count is not authoritative',()=>{
  expect(ids([{...box('search',0),mailwomanSearchQuery:'from:me'}])).toEqual(['search']);
 });
 it('reacts when counts change in either direction, retaining original objects/order',()=>{
  createRoot(dispose=>{
   const a=box('spam',0),b=box('drafts',1);const [folders,setFolders]=createSignal([a,b]);
   const visible=createMemo(()=>visibleSidebarFolders(folders(),{},read(true)));
   expect(visible()).toEqual([b]);
   const filled={...a,totalEmails:1};setFolders([filled,b]);expect(visible()).toEqual([filled,b]);
   setFolders([a,{...b,totalEmails:0}]);expect(visible()).toEqual([]);dispose();
  });
 });
});
