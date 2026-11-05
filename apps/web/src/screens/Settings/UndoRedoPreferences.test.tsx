import {afterEach,describe,expect,it} from 'vitest';
import {cleanup,render,screen,waitFor} from '@solidjs/testing-library';
import {clearSettings,loadSettings,type Fetcher} from '../../api/settings.ts';
import {UndoRedoPreferences} from './UndoRedoPreferences.tsx';

const json=(body:unknown):Response=>new Response(JSON.stringify(body),{status:200,headers:{'content-type':'application/json'}});
async function load(size:number,opts:{locked?:boolean;max?:number}={}):Promise<void>{
 await loadSettings({refresh:true,fetcher:(async()=>json({settings:{
  'ui.undoHistorySize':{value:size,source:opts.locked?'global':'default',locked:opts.locked===true},
  ...(opts.max!==undefined?{'ui.undoHistoryMax':{value:opts.max,source:'global',locked:false}}:{}),
 }})) as Fetcher});
}
afterEach(()=>{cleanup();clearSettings();});

describe('undo history setting',()=>{
 it('offers Off, 10, 25 and 50 and reflects the stored size',async()=>{
  await load(25);
  render(()=><UndoRedoPreferences/>);
  const select=await screen.findByLabelText('Actions remembered') as HTMLSelectElement;
  await waitFor(()=>expect(select.disabled).toBe(false));
  expect([...select.options].map(o=>o.text)).toEqual(['Off','10 actions','25 actions','50 actions']);
  expect(select.value).toBe('25');
  expect((screen.getByTestId('undo-enabled') as HTMLInputElement).checked).toBe(true);
  expect(screen.queryByTestId('setting-lock-note')).toBeNull();
 });
 it('shows the toggle off for size 0',async()=>{
  await load(0);
  render(()=><UndoRedoPreferences/>);
  const box=await screen.findByTestId('undo-enabled') as HTMLInputElement;
  expect(box.checked).toBe(false);
  expect((screen.getByTestId('undo-size') as HTMLSelectElement).value).toBe('0');
 });
 it('is disabled with the administrator note when locked',async()=>{
  await load(10,{locked:true});
  render(()=><UndoRedoPreferences/>);
  await waitFor(()=>expect(screen.getByTestId('setting-lock-note')).toBeInTheDocument());
  expect((screen.getByTestId('undo-size') as HTMLSelectElement).disabled).toBe(true);
  expect((screen.getByTestId('undo-enabled') as HTMLInputElement).disabled).toBe(true);
 });
 it('limits the choices to the administrator maximum and says so',async()=>{
  await load(50,{max:25});
  render(()=><UndoRedoPreferences/>);
  const select=await screen.findByTestId('undo-size') as HTMLSelectElement;
  await waitFor(()=>expect([...select.options].map(o=>o.value)).toEqual(['0','10','25']));
  expect(select.value).toBe('25');
  expect(screen.getByTestId('undo-capped').textContent).toContain('25');
 });
 it('is unavailable when the administrator maximum is 0',async()=>{
  await load(50,{max:0});
  render(()=><UndoRedoPreferences/>);
  const box=await screen.findByTestId('undo-enabled') as HTMLInputElement;
  expect(box.checked).toBe(false);expect(box.disabled).toBe(true);
 });
});
