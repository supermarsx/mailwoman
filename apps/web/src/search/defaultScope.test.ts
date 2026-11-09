import { afterEach, expect, it } from 'vitest';
import { applyDefaultSearchScope,normalSearchRequest } from './defaultScope.ts';
import { serverQueryText } from './searchQuery.ts';
import { clearSettings, loadSettings } from '../api/settings.ts';
const boxes=[{id:'inbox-id',name:'Inbox'},{id:'archive-id',name:'Archive'}];
afterEach(clearSettings);
it('strips only a known automatic scope from transport text, retaining typed operator requirements',()=>{
 clearSettings();expect(normalSearchRequest('hello','inbox-id',boxes)).toEqual({query:'hello in:Inbox',scopeMailboxId:'inbox-id',baseQuery:'hello'});
 expect(normalSearchRequest('hello in:Inbox','inbox-id',boxes)).toEqual({query:'hello in:Inbox'});
 expect(normalSearchRequest('from:alice in:Inbox','inbox-id',boxes,false,'inbox-id')).toEqual({query:'from:alice in:Inbox',scopeMailboxId:'inbox-id',baseQuery:'from:alice'});
 expect(normalSearchRequest('hello in:Archive','inbox-id',boxes,false,'inbox-id')).toEqual({query:'hello in:Archive'});
 expect(normalSearchRequest('hello','inbox-id',boxes,true,'inbox-id')).toEqual({query:'hello'});
});
it('defaults to the current folder and ANDs the entire Boolean expression',()=>{
  clearSettings();expect(applyDefaultSearchScope('hello','inbox-id',boxes)).toBe('hello in:Inbox');
  const query=applyDefaultSearchScope('from:a OR from:b','inbox-id',boxes);
  expect(query).toBe('(from:a OR from:b) in:Inbox');expect(serverQueryText(query,boxes)).toBe('(from:a OR from:b) in:inbox-id');
});
it('preserves explicit scopes including nested and negative operators, All folders, Trash and quoted literals',()=>{
  for(const query of ['hello in:Archive','from:a OR in:Archive','-in:Archive hello','hello include:trash'])expect(applyDefaultSearchScope(query,'inbox-id',boxes)).toBe(query);
  expect(applyDefaultSearchScope('hello','inbox-id',boxes,true)).toBe('hello');
  expect(applyDefaultSearchScope('"in:Archive"','inbox-id',boxes)).toBe('"in:Archive" in:Inbox');
  expect(applyDefaultSearchScope('hello','missing',boxes)).toBe('hello');expect(applyDefaultSearchScope('','inbox-id',boxes)).toBe('');
});
it('uses the reactive effective user/admin setting and rejects invalid values through the safe current default',async()=>{
  const set=async(value:unknown)=>loadSettings({refresh:true,fetcher:async()=>new Response(JSON.stringify({settings:{'mail.search.defaultScope':{value,locked:true,source:'global'}}}))});
  await set('all');expect(applyDefaultSearchScope('hello','inbox-id',boxes)).toBe('hello');
  await set('current');expect(applyDefaultSearchScope('hello','inbox-id',boxes)).toBe('hello in:Inbox');
  await set('invalid');expect(applyDefaultSearchScope('hello','inbox-id',boxes)).toBe('hello in:Inbox');
});
