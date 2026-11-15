import {createComputed,createRoot,createSignal} from 'solid-js';
import {afterEach,expect,it} from 'vitest';
import {createRangeSelection} from './rangeSelection.ts';
import {activateAccountPreferences,suspendAccountPreferences} from '../api/accountPreferenceLifecycle.ts';
let dispose=()=>undefined as void;afterEach(()=>{dispose();suspendAccountPreferences();});
const none={shiftKey:false,ctrlKey:false,metaKey:false},ctrl={...none,ctrlKey:true},shift={...none,shiftKey:true};
function model(){return createRoot(end=>{dispose=end;const [ids,setIds]=createSignal(['a','b','c','d','e']);const selection=createRangeSelection({ids,scope:()=> 'inbox'});return {...selection,setIds};});}
it('uses the last plain row as a range anchor and allows discontiguous Ctrl/Cmd toggles',()=>{
 const s=model();expect(s.pick('b',none)).toBe(false);expect(s.pick('d',shift)).toBe(true);expect([...s.selected()]).toEqual(['b','c','d']);
 s.pick('a',ctrl);s.pick('e',{...none,metaKey:true});s.pick('c',ctrl);expect([...s.selected()].sort()).toEqual(['a','b','d','e']);
});
it('shrinks a range while retaining selections made separately',()=>{
 const s=model();s.pick('e',ctrl);s.pick('a',ctrl);s.pick('d',shift);expect([...s.selected()].sort()).toEqual(['a','b','c','d','e']);s.pick('b',shift);expect([...s.selected()].sort()).toEqual(['a','b','e']);
});
it('does not select rows hidden by a filter or retain them after a list update',async()=>{
 const s=model();s.pick('a',none);s.setIds(['a','c','e']);s.pick('e',shift);expect([...s.selected()]).toEqual(['a','c','e']);await Promise.resolve();s.setIds(['c']);await Promise.resolve();expect([...s.selected()]).toEqual(['c']);
});
it('clears account-owned selections when the active identity changes',async()=>{
 activateAccountPreferences('a','selection_authority_123456');const s=model();s.pick('a',ctrl);await Promise.resolve();activateAccountPreferences('b','other_selection_authority_123456');await Promise.resolve();expect(s.selected().size).toBe(0);expect(s.mode()).toBe(false);
});
it('stops a bulk operation if account authority changes after its first item',async()=>{
 activateAccountPreferences('a','selection_authority_123456');const s=model();s.all();const applied:string[]=[];
 await s.run(async id=>{applied.push(id);suspendAccountPreferences();});expect(applied).toEqual(['a']);
});
it('does not subscribe its caller to the initial scope read',()=>{
 createRoot(end=>{dispose=end;const [scope,setScope]=createSignal('first');let constructions=0;
  createComputed(()=>{constructions++;createRangeSelection({ids:()=>['a'],scope});});
  setScope('second');expect(constructions).toBe(1);
 });
});
