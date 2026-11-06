import {describe,expect,it} from 'vitest';
import {DEFAULT_POLICY,applyPlacement,applyVisualParents,arrangedOnly,depthFromX,keyMoveFor,movingSet,nameCollision,placementForKey,placementFor,resolveDrop,validatePlacement,zoneOf,type DndFolder,type DndRow} from './folderDnd.ts';
const f=(id:string,parentId:string|null=null,extra:Partial<DndFolder>={}):DndFolder=>({id,name:id.toUpperCase(),parentId,role:null,totalEmails:0,unreadEmails:0,...extra});
// inbox | a (b (c), d) | e
const folders=[f('inbox',null,{role:'inbox'}),f('a'),f('b','a'),f('c','b'),f('d','a'),f('e')];
const rows:DndRow[]=[{id:'inbox',depth:0,hasChildren:false,expanded:false},{id:'a',depth:0,hasChildren:true,expanded:true},{id:'b',depth:1,hasChildren:true,expanded:true},{id:'c',depth:2,hasChildren:false,expanded:false},{id:'d',depth:1,hasChildren:false,expanded:false},{id:'e',depth:0,hasChildren:false,expanded:false}];
describe('drop geometry',()=>{
 it('splits a row into before, inside and after',()=>{
  expect(zoneOf(2,0,40)).toBe('before');expect(zoneOf(20,0,40)).toBe('inside');expect(zoneOf(38,0,40)).toBe('after');
 });
 it('reads the depth from the pointer x and clamps it',()=>{
  expect(depthFromX(100,100,16,0,3)).toBe(0);expect(depthFromX(132,100,16,0,3)).toBe(2);expect(depthFromX(900,100,16,1,3)).toBe(3);expect(depthFromX(0,100,16,1,3)).toBe(1);
 });
});
describe('placement',()=>{
 it('before a row is its parent and that row',()=>expect(placementFor(rows,folders,{targetId:'d',zone:'before'})).toEqual({parentId:'a',beforeId:'d'}));
 it('inside a row appends under it',()=>expect(placementFor(rows,folders,{targetId:'e',zone:'inside'})).toEqual({parentId:'e',beforeId:null}));
 it('after an expanded parent becomes its first child',()=>expect(placementFor(rows,folders,{targetId:'a',zone:'after'})).toEqual({parentId:'a',beforeId:'b'}));
 it('after the last row of a subtree can outdent by pointer depth',()=>{
  expect(placementFor(rows,folders,{targetId:'c',zone:'after',depth:2})).toEqual({parentId:'b',beforeId:null});
  expect(placementFor(rows,folders,{targetId:'c',zone:'after',depth:1})).toEqual({parentId:'a',beforeId:'d'});
  expect(placementFor(rows,folders,{targetId:'d',zone:'after',depth:0})).toEqual({parentId:null,beforeId:'e'});
  expect(placementFor(rows,folders,{targetId:'d',zone:'after',depth:9})).toEqual({parentId:'a',beforeId:null});
 });
});
describe('invalid drops',()=>{
 const drop=(moving:string[],targetId:string,zone:'before'|'inside'|'after',policy={})=>resolveDrop(rows,folders,moving,{targetId,zone},{...DEFAULT_POLICY,...policy});
 it('refuses into itself and its descendants',()=>{
  expect(drop(['a'],'a','inside')).toEqual({ok:false,problem:'self'});
  expect(drop(['a'],'c','inside')).toEqual({ok:false,problem:'descendant'});
 });
 it('refuses system folders as movers and as siblings',()=>{
  expect(drop(['inbox'],'e','before')).toEqual({ok:false,problem:'system'});
  expect(drop(['e'],'inbox','before')).toEqual({ok:false,problem:'system'});
  expect(drop(['inbox'],'e','before',{allowSystemFolders:true}).ok).toBe(true);
 });
 it('refuses nesting under a folder that cannot have children',()=>{
  const locked=folders.map(x=>x.id==='e'?{...x,myRights:{mayCreateChild:false}}:x);
  expect(validatePlacement(locked,['d'],{parentId:'e',beforeId:null},DEFAULT_POLICY)).toEqual({ok:false,problem:'no-children'});
 });
 it('enforces the depth limit counting the moving subtree',()=>{
  expect(drop(['a'],'e','inside',{maxDepth:4}).ok).toBe(true);
  expect(drop(['a'],'e','inside',{maxDepth:3})).toEqual({ok:false,problem:'depth'});
 });
 it('honours the admin switches',()=>{
  expect(drop(['e'],'a','before',{locked:true})).toEqual({ok:false,problem:'locked'});
  expect(drop(['e'],'a','before',{enabled:false})).toEqual({ok:false,problem:'disabled'});
  expect(drop(['e'],'a','inside',{allowNesting:false})).toEqual({ok:false,problem:'nesting-off'});
  expect(drop(['e'],'a','before',{forcePinned:new Set(['e'])})).toEqual({ok:false,problem:'forced'});
 });
 it('treats dropping where it already is as nothing',()=>expect(drop(['b'],'d','before')).toEqual({ok:false,problem:'noop'}));
 it('finds a name collision under the new parent',()=>{
  const dup=[...folders,f('x','e',{name:'B'})];
  expect(nameCollision(dup,['b'],{parentId:'e',beforeId:null})).toBe('B');
  expect(nameCollision(dup,['b'],{parentId:'d',beforeId:null})).toBeNull();
 });
});
describe('applying a move',()=>{
 const ids=folders.map(x=>x.id);
 it('reorders among siblings without a parent override',()=>{
  const out=applyPlacement(ids,folders,undefined,['e'],{parentId:null,beforeId:'a'});
  expect(out.order).toEqual(['inbox','e','a','b','c','d']);expect(out.parents).toEqual({});expect(out.reparented).toEqual([]);
 });
 it('nests with a visual parent, un-nests to the top level, and drops the override at the server parent',()=>{
  const nested=applyPlacement(ids,folders,undefined,['e'],{parentId:'d',beforeId:null});
  expect(nested.parents).toEqual({e:'d'});expect(nested.reparented).toEqual(['e']);
  const top=applyPlacement(ids,folders,undefined,['d'],{parentId:null,beforeId:null});
  expect(top.parents).toEqual({d:'_top'});
  const back=applyPlacement(ids,folders,{d:'_top'},['d'],{parentId:'a',beforeId:null});
  expect(back.parents).toEqual({});
 });
 it('moves a multi-selection together, keeping its order',()=>{
  const out=applyPlacement(ids,folders,undefined,['e','b'],{parentId:null,beforeId:'a'});
  expect(out.order).toEqual(['inbox','b','e','a','c','d']);
  expect(movingSet(folders,new Set(['e','b','c']),'b')).toEqual(['b','e']);
  expect(movingSet(folders,new Set(['e']),'d')).toEqual(['d']);
 });
 it('applies visual parents, ignoring missing targets, cycles and system folders',()=>{
  const out=applyVisualParents(folders,{e:'d',a:'c',inbox:'a',d:'gone',b:'_top'});
  const parent=(id:string)=>out.find(x=>x.id===id)?.parentId??null;
  expect(parent('e')).toBe('d');expect(parent('a')).toBeNull();expect(parent('inbox')).toBeNull();expect(parent('d')).toBe('a');expect(parent('b')).toBeNull();
  expect(arrangedOnly(folders,{e:'d',b:'_top'}).sort()).toEqual(['b','e']);
 });
});
describe('keyboard moves',()=>{
 it('maps Alt+arrows, flipping in RTL',()=>{
  expect(keyMoveFor({key:'ArrowUp',altKey:true})).toBe('up');expect(keyMoveFor({key:'ArrowUp',altKey:false})).toBeNull();
  expect(keyMoveFor({key:'ArrowLeft',altKey:true})).toBe('outdent');expect(keyMoveFor({key:'ArrowLeft',altKey:true},true)).toBe('indent');
 });
 it('reorders, indents and outdents',()=>{
  const go=(id:string,move:'up'|'down'|'indent'|'outdent')=>placementForKey(folders,id,move,DEFAULT_POLICY);
  expect(go('d','up')).toEqual({ok:true,placement:{parentId:'a',beforeId:'b'}});
  expect(go('b','down')).toEqual({ok:true,placement:{parentId:'a',beforeId:null}});
  expect(go('d','indent')).toEqual({ok:true,placement:{parentId:'b',beforeId:null}});
  expect(go('c','outdent')).toEqual({ok:true,placement:{parentId:'a',beforeId:'d'}});
  expect(go('a','up')).toEqual({ok:false,problem:'noop'});
  expect(placementForKey(folders,'d','up',{...DEFAULT_POLICY,locked:true})).toEqual({ok:false,problem:'locked'});
 });
});
