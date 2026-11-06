import {beforeEach,describe,expect,it,vi} from 'vitest';
import {createSignal} from 'solid-js';
import type {AccountUiPreferences} from '../api/accountPreferences.ts';

// The account preference store needs a signed-in sync controller; this stands in
// for it with a plain signal so the folder-tree expansion can be observed.
const store=vi.hoisted(()=>({current:undefined as undefined|{get:()=>AccountUiPreferences;set:(patch:AccountUiPreferences)=>void}}));
vi.mock('../api/accountPreferences.ts',()=>{
 const [prefs,setPrefs]=createSignal<AccountUiPreferences>({});
 store.current={get:prefs,set:patch=>setPrefs(old=>({...old,...patch}))};
 return {accountUiPreferences:prefs,setAccountUiPreferences:(patch:AccountUiPreferences)=>setPrefs(old=>({...old,...patch}))};
});
import {accountUiPreferences,setAccountUiPreferences} from '../api/accountPreferences.ts';
import {expandFolderTree,folderAncestors,focusMessageInFolder,focusRequest,clearFocusRequest} from './messageFocus.ts';

const boxes=[{id:'root',parentId:null,name:'Root'},{id:'mid',parentId:'root',name:'Mid'},{id:'leaf',parentId:'mid',name:'Leaf'},{id:'other',parentId:null,name:'Other'}];
beforeEach(()=>{setAccountUiPreferences({folderCollapsed:[]});clearFocusRequest();});

describe('Go to folder expands the folder tree',()=>{
 it('lists the ancestors nearest first and survives a cycle',()=>{
  expect(folderAncestors(boxes,'leaf')).toEqual(['mid','root']);
  expect(folderAncestors(boxes,'other')).toEqual([]);
  expect(folderAncestors([{id:'a',parentId:'b'},{id:'b',parentId:'a'}],'a')).toEqual(['b','a']);
 });
 it('opens only the collapsed ancestors and leaves other folders collapsed',()=>{
  setAccountUiPreferences({folderCollapsed:['mid','other','root']});
  expandFolderTree(boxes,'leaf');
  expect(accountUiPreferences().folderCollapsed).toEqual(['other']);
 });
 it('changes nothing when no ancestor is collapsed',()=>{
  setAccountUiPreferences({folderCollapsed:['other']});
  const before=accountUiPreferences();
  expandFolderTree(boxes,'leaf');
  expect(accountUiPreferences()).toBe(before);
 });
 it('focusMessageInFolder expands the tree to the target folder, and records a same-thread jump',async()=>{
  setAccountUiPreferences({folderCollapsed:['root','mid']});
  const app={
   accountId:()=>'acct',selectedMailboxId:()=>'leaf',searchActive:()=>false,selectMailbox:async()=>undefined,
   messages:()=>[{id:'m1'} as never],hasMore:()=>false,loadMore:async()=>undefined,mailboxes:()=>boxes,showToast:vi.fn(),
  };
  expect(await focusMessageInFolder(app,'leaf','m1',{thread:true})).toBe(true);
  expect(accountUiPreferences().folderCollapsed).toEqual([]);
  expect(focusRequest()).toMatchObject({emailId:'m1',thread:true});
  clearFocusRequest();
  await focusMessageInFolder(app,'leaf','m1');
  expect(focusRequest()?.thread).toBeUndefined();
 });
});
