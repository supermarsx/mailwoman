import {describe,expect,it} from 'vitest';
import {reorderPins} from './pinDnd.ts';
describe('reorderPins',()=>{
 const ids=['forced','a','b','c'],forced=new Set(['forced']);
 it('moves a pin before or after another and returns only the user\'s own order',()=>{
  expect(reorderPins(ids,forced,'c','a','before')).toEqual(['c','a','b']);
  expect(reorderPins(ids,forced,'a','c','after')).toEqual(['b','c','a']);
 });
 it('treats a drop next to a forced pin as the start of the user\'s pins',()=>{
  expect(reorderPins(ids,forced,'c','forced','after')).toEqual(['c','a','b']);
 });
 it('refuses to move a forced pin, ignores a no-op and unknown ids',()=>{
  expect(reorderPins(ids,forced,'forced','a','after')).toBeNull();
  expect(reorderPins(ids,forced,'a','b','before')).toBeNull();
  expect(reorderPins(ids,forced,'a','a','after')).toBeNull();
  expect(reorderPins(ids,forced,'x','a','after')).toBeNull();
 });
});
