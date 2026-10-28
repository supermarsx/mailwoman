import {render,screen,fireEvent,within} from '@solidjs/testing-library';
import {createSignal} from 'solid-js';
import {describe,it,expect,vi} from 'vitest';
import {ComposerHistory} from './ComposerHistory.tsx';
import type {RichTextApi} from './RichTextEditor.tsx';
describe('ComposerHistory',()=>{
 it('uses independent bounded Undo and Redo disclosures and keeps keyboard focus on escape',()=>{
  const [depth,setDepth]=createSignal({undo:3,redo:2});
  const undo=vi.fn(()=>setDepth(value=>({...value,undo:value.undo-1,redo:value.redo+1})));
  const redo=vi.fn(()=>setDepth(value=>({...value,redo:value.redo-1,undo:value.undo+1})));
  const api:RichTextApi={undo,redo,historyDepth:depth,focus:()=>undefined,appendHtml:()=>undefined,setHtml:()=>undefined,refreshImages:()=>undefined};
  render(()=><ComposerHistory api={api}/>);
  expect(screen.queryByRole('combobox')).toBeNull();
  const undoSummary=screen.getByLabelText('Undo history',{selector:'summary'});
  const details=undoSummary.closest('details')!;details.open=true;
  fireEvent.click(within(details).getByRole('button',{name:'Undo last 2 change groups'}));
  expect(undo).toHaveBeenCalledTimes(2);expect(details.open).toBe(false);
  const redoSummary=screen.getByLabelText('Redo history',{selector:'summary'});const redoDetails=redoSummary.closest('details')!;redoDetails.open=true;
  fireEvent.click(within(redoDetails).getByRole('button',{name:'Redo 2 changes'}));expect(redo).toHaveBeenCalledTimes(2);
  details.open=true;fireEvent.keyDown(undoSummary,{key:'Escape'});expect(details.open).toBe(false);expect(undoSummary).toHaveFocus();
 });
 it('leaves Esc on a closed disclosure to the ribbon around it',()=>{
  const api:RichTextApi={undo:()=>undefined,redo:()=>undefined,historyDepth:()=>({undo:1,redo:0}),focus:()=>undefined,appendHtml:()=>undefined,setHtml:()=>undefined,refreshImages:()=>undefined};
  const outer=vi.fn();
  render(()=><div onKeyDown={event=>outer(event.key)}><ComposerHistory api={api}/></div>);
  const undoSummary=screen.getByLabelText('Undo history',{selector:'summary'});
  fireEvent.keyDown(undoSummary,{key:'Escape'});
  expect(outer).toHaveBeenCalledWith('Escape');
  outer.mockClear();undoSummary.closest('details')!.open=true;
  fireEvent.keyDown(undoSummary,{key:'Escape'});
  expect(outer).not.toHaveBeenCalled();
 });
});
