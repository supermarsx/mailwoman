import {describe,expect,it} from 'vitest';
import {foldActions} from './readerToolbarFit.ts';
const items=(widths:number[])=>widths.map((width,index)=>({id:`a${index}`,priority:index,width}));
describe('reader action row fitting',()=>{
 it('folds nothing when the whole row fits, More included or not',()=>{
  expect(foldActions(items([40,40,40]),40*3+2*4,4,40).size).toBe(0);
 });
 it('keeps actions in priority order and reserves room for More',()=>{
  // 200px: More (40) + gap + 3x(40+4) = 172; a fourth would need 216.
  expect([...foldActions(items([40,40,40,40,40,40]),200,4,40)]).toEqual(['a3','a4','a5']);
 });
 it('stops at the first action that does not fit instead of skipping to a smaller one',()=>{
  const row=[{id:'reply',priority:0,width:40},{id:'wide',priority:1,width:150},{id:'tiny',priority:2,width:10}];
  expect([...foldActions(row,120,4,40)].sort()).toEqual(['tiny','wide']);
 });
 it('folds by priority, not by row position',()=>{
  const row=[{id:'print',priority:9,width:40},{id:'reply',priority:0,width:40},{id:'delete',priority:3,width:40}];
  expect([...foldActions(row,125,4,30)]).toEqual(['print']);
 });
});
