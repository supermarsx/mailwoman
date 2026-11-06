import {describe,expect,it} from 'vitest';
import {EMPTY_SELECTION,applyClick,extendByKey,pruneSelection,rangeIds,selectAll,type SelectionState} from './folderSelectionModel.ts';
// Tree order with Work collapsed: its children q1/q2 are not visible.
const visible=['inbox','sent','work','home','misc','tail'];
const user=(id:string)=>!['inbox','sent'].includes(id);
const ids=(s:SelectionState)=>[...s.selected].sort();
describe('folder selection geometry',()=>{
 it('ranges run in visible tree order, either direction, skipping unselectable rows and hidden ones',()=>{
  expect(rangeIds(visible,'work','misc')).toEqual(['work','home','misc']);
  expect(rangeIds(visible,'misc','work')).toEqual(['work','home','misc']);
  expect(rangeIds(visible,'sent','home',user)).toEqual(['work','home']);
  expect(rangeIds(visible,'work','q1')).toEqual([]);
 });
 it('ctrl toggles and moves the anchor; shift extends from it and a later shift replaces its own range',()=>{
  let s=applyClick(EMPTY_SELECTION,visible,'work',{shift:false,toggle:true},user);
  expect(ids(s)).toEqual(['work']);
  s=applyClick(s,visible,'tail',{shift:true,toggle:false},user);
  expect(ids(s)).toEqual(['home','misc','tail','work']);
  s=applyClick(s,visible,'home',{shift:true,toggle:false},user);
  expect(ids(s)).toEqual(['home','work']);
  s=applyClick(s,visible,'home',{shift:false,toggle:true},user);
  expect(ids(s)).toEqual(['work']);
  expect(s.anchor).toBe('home');
 });
 it('keeps earlier picks when a shift range is replaced',()=>{
  let s=applyClick(EMPTY_SELECTION,visible,'tail',{shift:false,toggle:true},user);
  s=applyClick(s,visible,'work',{shift:false,toggle:true},user);
  s=applyClick(s,visible,'misc',{shift:true,toggle:false},user);
  expect(ids(s)).toEqual(['home','misc','tail','work']);
  s=applyClick(s,visible,'home',{shift:true,toggle:false},user);
  expect(ids(s)).toEqual(['home','tail','work']);
 });
 it('a plain click only moves the anchor; shift without an anchor toggles',()=>{
  const s=applyClick(EMPTY_SELECTION,visible,'home',{shift:false,toggle:false},user);
  expect(ids(s)).toEqual([]);expect(s.anchor).toBe('home');
  expect(ids(applyClick(EMPTY_SELECTION,visible,'misc',{shift:true,toggle:false},user))).toEqual(['misc']);
 });
 it('shift+arrow grows and shrinks the range one visible row at a time',()=>{
  let s=EMPTY_SELECTION;let focus='work';
  let r=extendByKey(s,visible,focus,1,user)!;s=r.state;focus=r.focus;
  expect(ids(s)).toEqual(['home','work']);
  r=extendByKey(s,visible,focus,1,user)!;s=r.state;focus=r.focus;
  expect(ids(s)).toEqual(['home','misc','work']);
  r=extendByKey(s,visible,focus,-1,user)!;s=r.state;focus=r.focus;
  expect(ids(s)).toEqual(['home','work']);
  expect(extendByKey(s,visible,'tail',1,user)).toBeNull();
 });
 it('select all takes user folders only; prune drops rows that collapsed away',()=>{
  const all=selectAll(visible,user);
  expect(ids(all)).toEqual(['home','misc','tail','work']);
  const pruned=pruneSelection(all,new Set(['home','misc']));
  expect(ids(pruned)).toEqual(['home','misc']);
 });
});
