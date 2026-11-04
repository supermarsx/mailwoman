import {describe,expect,it} from 'vitest';
import {childrenOf,descendantsOf,DEFAULT_SORT,moveTargets,moveWithinSiblings,rolledTotal,rolledUnread,siblingsOf,sortFor,sortSiblings,treeRows,type TreeFolder} from './folderTree.ts';

const box=(id:string,name:string,parentId:string|null,extra:Partial<TreeFolder>={}):TreeFolder=>({id,name,parentId,role:null,totalEmails:0,unreadEmails:0,...extra});
const BOXES:TreeFolder[]=[
 box('inbox','Inbox',null,{role:'inbox',unreadEmails:3,totalEmails:10}),
 box('sent','Sent',null,{role:'sent'}),
 box('trash','Trash',null,{role:'trash'}),
 box('work','Work',null,{unreadEmails:1,totalEmails:5}),
 box('q3','Q3',null,{parentId:'work',unreadEmails:4,totalEmails:20}),
 box('q3a','Reports',null,{parentId:'q3',unreadEmails:2,totalEmails:7}),
 box('home','Home',null,{unreadEmails:9,totalEmails:2}),
 box('apple','Apple',null,{parentId:'inbox',totalEmails:1}),
 box('orphan','Orphan',null,{parentId:'gone'}),
];
const ids=(rows:{box:TreeFolder}[])=>rows.map(row=>row.box.id);

describe('folder tree nesting',()=>{
 it('lists system folders first (with their subfolders), then user folders, to any depth',()=>{
  const rows=treeRows(BOXES,()=>true);
  expect(ids(rows)).toEqual(['inbox','apple','sent','trash','work','q3','q3a','home','orphan']);
  expect(rows.map(row=>row.depth)).toEqual([0,1,0,0,0,1,2,0,0]);
  expect(rows.find(row=>row.box.id==='work')).toMatchObject({hasChildren:true,expanded:true});
 });
 it('lists no children of a collapsed folder, and keeps the folder itself',()=>{
  const rows=treeRows(BOXES,()=>true,{collapsed:new Set(['work'])});
  expect(ids(rows)).not.toContain('q3');
  expect(rows.find(row=>row.box.id==='work')).toMatchObject({hasChildren:true,expanded:false});
 });
 it('treats a folder whose parent is missing as a root and survives a cycle',()=>{
  expect(childrenOf(BOXES,null).map(value=>value.id)).toContain('orphan');
  const cyclic=[box('a','A',null,{parentId:'b'}),box('b','B',null,{parentId:'a'})];
  expect(descendantsOf(cyclic,'a').map(value=>value.id)).toEqual(['b']);
  expect(treeRows(cyclic,()=>true)).toEqual([]);
 });
 it('rolls up unread and totals through the subtree',()=>{
  expect(rolledUnread(BOXES,'work')).toBe(1+4+2);
  expect(rolledTotal(BOXES,'work')).toBe(5+20+7);
  expect(rolledUnread(BOXES,'home')).toBe(9);
 });
 it('offers every folder except itself, its subtree and search folders as a move target',()=>{
  const withSearch=[...BOXES,box('saved','Saved',null,{mailwomanSearchQuery:'{}'})];
  expect(moveTargets(withSearch,'work').map(value=>value.id)).toEqual(['inbox','sent','trash','home','apple','orphan']);
 });
});

describe('subfolder ordering',()=>{
 const kids=[box('b','b',null,{unreadEmails:1,totalEmails:30}),box('a','A',null,{unreadEmails:5,totalEmails:10}),box('c','C',null,{unreadEmails:5,totalEmails:20})];
 it('sorts by name (natural, case-insensitive) both ways',()=>{
  expect(sortSiblings(kids,{by:'name',dir:'asc'}).map(value=>value.id)).toEqual(['a','b','c']);
  expect(sortSiblings(kids,{by:'name',dir:'desc'}).map(value=>value.id)).toEqual(['c','b','a']);
 });
 it('sorts by unread then keeps the existing order for ties',()=>{
  expect(sortSiblings(kids,{by:'unread',dir:'desc'}).map(value=>value.id)).toEqual(['a','c','b']);
  expect(sortSiblings(kids,{by:'unread',dir:'asc'}).map(value=>value.id)).toEqual(['b','a','c']);
 });
 it('sorts by total and by last activity',()=>{
  expect(sortSiblings(kids,{by:'total',dir:'desc'}).map(value=>value.id)).toEqual(['b','c','a']);
  const activity=new Map([['a',300],['b',100],['c',200]]);
  expect(sortSiblings(kids,{by:'date',dir:'desc'},activity).map(value=>value.id)).toEqual(['a','c','b']);
  expect(sortSiblings(kids,{by:'date',dir:'asc'},activity).map(value=>value.id)).toEqual(['b','c','a']);
 });
 it('keeps the given (manual) order for custom, reversing it for descending',()=>{
  expect(sortSiblings(kids,DEFAULT_SORT).map(value=>value.id)).toEqual(['b','a','c']);
  expect(sortSiblings(kids,{by:'custom',dir:'desc'}).map(value=>value.id)).toEqual(['c','a','b']);
 });
 it('applies each parent\'s own choice in the tree and never reorders system folders',()=>{
  const rows=treeRows(BOXES,()=>true,{sorts:{_top:{by:'name',dir:'desc'},work:{by:'name',dir:'asc'}}});
  expect(ids(rows).filter(id=>['home','orphan','work'].includes(id))).toEqual(['work','orphan','home']);
  expect(ids(rows).slice(0,4)).toEqual(['inbox','apple','sent','trash']);
 });
 it('ignores a stored choice it does not recognise',()=>{
  expect(sortFor({_top:{by:'bogus',dir:'asc'} as never},null)).toEqual(DEFAULT_SORT);
  expect(sortFor(undefined,'work')).toEqual(DEFAULT_SORT);
 });
 it('finds siblings in display order',()=>{
  expect(siblingsOf(BOXES,BOXES.find(value=>value.id==='home')!).map(value=>value.id)).toEqual(['work','home','orphan']);
  expect(siblingsOf(BOXES,BOXES.find(value=>value.id==='sent')!).map(value=>value.id)).toEqual(['inbox','sent','trash']);
 });
});

describe('moving up and down among siblings',()=>{
 it('swaps neighbours and renumbers only the siblings\' slots',()=>{
  expect(moveWithinSiblings(['x','a','y','b','c'],['a','b','c'],'b','up')).toEqual(['x','b','y','a','c']);
  expect(moveWithinSiblings(['a','b','c'],['a','b','c'],'a','down')).toEqual(['b','a','c']);
 });
 it('starts a manual order when none is saved, and refuses at the ends',()=>{
  expect(moveWithinSiblings([],['a','b','c'],'c','up')).toEqual(['a','c','b']);
  expect(moveWithinSiblings([],['a','b'],'a','up')).toBeNull();
  expect(moveWithinSiblings([],['a','b'],'b','down')).toBeNull();
  expect(moveWithinSiblings([],['a','b'],'zzz','down')).toBeNull();
 });
});
