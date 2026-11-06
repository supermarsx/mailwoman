import {describe,expect,it} from 'vitest';
import {entryMatches,forcePinnedIds,mergeOrder} from './folderPolicy.ts';
const f=(id:string,name:string,extra:Record<string,unknown>={})=>({id,name,parentId:null as string|null,role:null as string|null,totalEmails:0,unreadEmails:0,...extra});
const folders=[f('1','Inbox',{role:'inbox'}),f('2','Support',{mailwomanPath:'Shared/Support'}),f('3','Zed'),f('4','Alpha'),f('5','Sent',{role:'sent'})];
describe('folder policy entries',()=>{
 it('matches roles, paths and plain names, ignoring case',()=>{
  expect(entryMatches('role:inbox',folders[0]!)).toBe(true);
  expect(entryMatches('shared/support',folders[1]!)).toBe(true);
  expect(entryMatches('support',folders[1]!)).toBe(true);
  expect(entryMatches('Other/Support',folders[1]!)).toBe(false);
 });
 it('lists force-pinned ids in entry order and skips entries without a folder',()=>{
  expect(forcePinnedIds(folders,['role:sent','missing','Shared/Support'])).toEqual(['5','2']);
 });
});
describe('merging the stored order with the server folders',()=>{
 it('keeps the stored order, ignores unknown ids and appends new folders at the end',()=>{
  expect(mergeOrder(folders,['4','gone','1'],[],'end')).toEqual(['4','1','2','3','5']);
 });
 it('slots new folders alphabetically among their siblings when asked',()=>{
  const out=mergeOrder([f('a','Beta'),f('b','Delta'),f('c','Alpha'),f('d','Charlie')],['a','b'],[],'alphabetical');
  expect(out).toEqual(['c','a','d','b']);
 });
 it('starts from the admin template when the user has no order of their own',()=>{
  expect(mergeOrder(folders,undefined,['role:sent','Alpha'],'end')).toEqual(['5','4','1','2','3']);
  expect(mergeOrder(folders,['3'],['role:sent'],'end')).toEqual(['3','1','2','4','5']);
 });
});
