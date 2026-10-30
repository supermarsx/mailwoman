import {describe,expect,it,vi} from 'vitest';
import {focusActions} from './MessageContextMenu.tsx';
import type {Email} from '../api/jmap-types.ts';
// t94: "Always move to Focused / Other" in the message menus.
const message=(email:string):Email=>({id:'m',mailboxIds:{inbox:true},from:[{name:null,email}],to:[],subject:'s',receivedAt:'2026-01-01T00:00:00Z',preview:''});
describe('focusActions',()=>{
 it('offers both tabs for the sender while the Focused inbox is on, the current choice checked',()=>{
  const alwaysMoveSender=vi.fn(async()=>undefined);
  const rows=focusActions({focusedInbox:()=>true,senderFocus:()=>'other',alwaysMoveSender},message('news@shop.example'));
  expect(rows.map(row=>[row.id,row.label,row.checked])).toEqual([['focus:focused','Always move to Focused',false],['focus:other','Always move to Other',true]]);
  void rows[0]!.run();
  expect(alwaysMoveSender).toHaveBeenCalledWith('news@shop.example','focused');
 });
 it('offers nothing while the Focused inbox is off or the message has no sender',()=>{
  const app={senderFocus:()=>undefined,alwaysMoveSender:async()=>undefined};
  expect(focusActions({...app,focusedInbox:()=>false},message('a@x.example'))).toEqual([]);
  expect(focusActions({...app,focusedInbox:()=>true},message(''))).toEqual([]);
 });
});
