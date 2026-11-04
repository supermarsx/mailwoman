import {describe,expect,it} from 'vitest';
import {placePanel} from './ContextMenu.tsx';

const viewport={width:800,height:600};
describe('placePanel',()=>{
 it('opens at the pointer when there is room',()=>{
  expect(placePanel({point:{x:100,y:120},size:{width:180,height:200},viewport})).toEqual({left:100,top:120,maxHeight:584});
 });
 it('shifts a menu back on screen at the right and bottom edges',()=>{
  expect(placePanel({point:{x:790,y:590},size:{width:180,height:200},viewport})).toEqual({left:612,top:392,maxHeight:584});
 });
 it('limits a tall menu to the viewport so it scrolls instead of overflowing',()=>{
  const placed=placePanel({point:{x:10,y:300},size:{width:180,height:900},viewport});
  expect(placed.maxHeight).toBe(584);
  expect(placed.top).toBe(8);
 });
 it('opens upward from the point when asked (menu above its trigger)',()=>{
  expect(placePanel({point:{x:50,y:500,above:true},size:{width:150,height:100},viewport}).top).toBe(400);
 });
 it('opens a submenu beside its row, and flips to the left when there is no room on the right',()=>{
  const row={left:300,right:490,top:200};
  expect(placePanel({point:{x:0,y:0},anchor:row,size:{width:150,height:100},viewport}).left).toBe(490);
  expect(placePanel({point:{x:0,y:0},anchor:{left:640,right:790,top:200},size:{width:150,height:100},viewport}).left).toBe(490);
 });
 it('mirrors the submenu side in right-to-left layouts',()=>{
  expect(placePanel({point:{x:0,y:0},anchor:{left:400,right:590,top:200},size:{width:150,height:100},viewport,rtl:true}).left).toBe(250);
  expect(placePanel({point:{x:0,y:0},anchor:{left:100,right:290,top:200},size:{width:150,height:100},viewport,rtl:true}).left).toBe(290);
 });
 it('keeps a submenu inside the viewport vertically',()=>{
  expect(placePanel({point:{x:0,y:0},anchor:{left:100,right:290,top:560},size:{width:150,height:200},viewport}).top).toBe(392);
 });
});
