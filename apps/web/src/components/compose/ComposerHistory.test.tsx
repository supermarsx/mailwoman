import {render,screen,fireEvent,within} from '@solidjs/testing-library';
import {createSignal} from 'solid-js';
import {describe,it,expect,vi} from 'vitest';
import {ComposerHistory,historyShortcut,isApplePlatform} from './ComposerHistory.tsx';
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
 it('disables Undo and Redo when there is nothing to undo or redo, and names them', ()=>{
  const [depth,setDepth]=createSignal({undo:0,redo:0});
  const api:RichTextApi={undo:()=>undefined,redo:()=>undefined,historyDepth:depth,focus:()=>undefined,appendHtml:()=>undefined,setHtml:()=>undefined,refreshImages:()=>undefined};
  render(()=><ComposerHistory api={api}/>);
  const undo=screen.getByRole('button',{name:'Undo'});const redo=screen.getByRole('button',{name:'Redo'});
  expect(undo).toBeDisabled();expect(redo).toBeDisabled();
  setDepth({undo:2,redo:0});expect(undo).toBeEnabled();expect(redo).toBeDisabled();
  setDepth({undo:0,redo:1});expect(undo).toBeDisabled();expect(redo).toBeEnabled();
  // The disclosures stay usable (they say the history is empty) and keep their own names.
  expect(screen.getByLabelText('Undo history',{selector:'summary'})).toBeInTheDocument();
  expect(screen.getByLabelText('Redo history',{selector:'summary'})).toBeInTheDocument();
 });
 it('shows the icon and the label, with the shortcut in the tooltip', ()=>{
  render(()=><ComposerHistory api={null}/>);
  const undo=screen.getByRole('button',{name:'Undo'});const redo=screen.getByRole('button',{name:'Redo'});
  expect(undo.querySelector('svg[data-icon="undo"]')).not.toBeNull();expect(redo.querySelector('svg[data-icon="redo"]')).not.toBeNull();
  expect(undo.querySelector('svg')!.classList.contains('icon--directional')).toBe(true);
  expect(undo.textContent).toBe('Undo');expect(redo.textContent).toBe('Redo');
  const apple=isApplePlatform();
  expect(undo.getAttribute('data-tooltip')).toBe(`Undo (${apple?'⌘Z':'Ctrl+Z'})`);
  expect(redo.getAttribute('data-tooltip')).toBe(`Redo (${apple?'⇧⌘Z':'Ctrl+Y'})`);
  expect(undo.getAttribute('aria-keyshortcuts')).toContain('Control+Z');expect(redo.getAttribute('aria-keyshortcuts')).toContain('Control+Y');
  // No inline style attributes (CSP).
  expect(document.querySelector('.compose-rt__history [style]')).toBeNull();
 });
 it('names the Command chords on Apple platforms and Ctrl elsewhere', ()=>{
  expect(isApplePlatform({platform:'MacIntel',userAgent:''})).toBe(true);
  expect(isApplePlatform({platform:'',userAgent:'',userAgentData:{platform:'macOS'}})).toBe(true);
  expect(isApplePlatform({platform:'Win32',userAgent:'Windows NT 10.0'})).toBe(false);
  expect(historyShortcut('undo',true)).toBe('⌘Z');expect(historyShortcut('redo',true)).toBe('⇧⌘Z');
  expect(historyShortcut('undo',false)).toBe('Ctrl+Z');expect(historyShortcut('redo',false)).toBe('Ctrl+Y');
 });
});
