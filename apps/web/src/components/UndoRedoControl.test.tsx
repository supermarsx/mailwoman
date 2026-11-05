import {afterEach,beforeEach,describe,expect,it} from 'vitest';
import {cleanup,fireEvent,render,screen,waitFor} from '@solidjs/testing-library';
import {clearSettings,loadSettings,type Fetcher} from '../api/settings.ts';
import {suspendAccountPreferences} from '../api/accountPreferenceLifecycle.ts';
import {toast as toastStore} from './toast/toast.ts';
import {loadCatalog} from '../i18n/index.ts';
import {clearUndoHistory,recordUndoable,redoEntries,undoEntries} from '../state/undoHistory.ts';
import {UndoRedoControl} from './UndoRedoControl.tsx';

const json=(body:unknown):Response=>new Response(JSON.stringify(body),{status:200,headers:{'content-type':'application/json'}});
function add(label:string,log:string[]){recordUndoable({label,accountId:'a',undo:async()=>{log.push(`undo ${label}`);},redo:async()=>{log.push(`redo ${label}`);}});}
beforeEach(async()=>{suspendAccountPreferences();await loadCatalog('undo');clearUndoHistory();toastStore.clear();});
afterEach(()=>{cleanup();clearSettings();clearUndoHistory();toastStore.clear();});

describe('UndoRedoControl',()=>{
 it('is disabled when there is nothing to undo or redo',()=>{
  render(()=><UndoRedoControl/>);
  const undo=screen.getByTestId('undo-button'),redo=screen.getByTestId('redo-button');
  expect(undo).toBeDisabled();expect(redo).toBeDisabled();
  expect(undo).toHaveAttribute('aria-label','Nothing to undo');
  expect(screen.getByTestId('undo-menu-button')).toBeDisabled();
 });

 it('names the next action in the tooltip and undoes it on click, then redoes it',async()=>{
  const log:string[]=[];add('Archived 3 messages',log);
  render(()=><UndoRedoControl/>);
  const undo=screen.getByTestId('undo-button');
  expect(undo).toHaveAttribute('title','Undo: Archived 3 messages');
  fireEvent.click(undo);
  await waitFor(()=>expect(log).toEqual(['undo Archived 3 messages']));
  await waitFor(()=>expect(screen.getByTestId('redo-button')).toHaveAttribute('title','Redo: Archived 3 messages'));
  expect(screen.getByTestId('undo-status').textContent).toBe('Undid: Archived 3 messages');
  expect(screen.getByTestId('undo-status')).toHaveAttribute('aria-live','polite');
  fireEvent.click(screen.getByTestId('redo-button'));
  await waitFor(()=>expect(log).toEqual(['undo Archived 3 messages','redo Archived 3 messages']));
 });

 it('lists the recent actions newest first and undoes back through the chosen one',async()=>{
  const log:string[]=[];
  for(const label of ['one','two','three','four'])add(label,log);
  render(()=><UndoRedoControl/>);
  const caret=screen.getByTestId('undo-menu-button');
  expect(caret).toHaveAttribute('aria-haspopup','menu');expect(caret).toHaveAttribute('aria-expanded','false');
  fireEvent.click(caret);
  expect(caret).toHaveAttribute('aria-expanded','true');
  const items=screen.getAllByRole('menuitem');
  expect(items.map(item=>item.textContent)).toEqual(['four','three','two','one']);
  fireEvent.click(items[2]!);
  await waitFor(()=>expect(log).toEqual(['undo four','undo three','undo two']));
  await waitFor(()=>expect(undoEntries().map(e=>e.label)).toEqual(['one']));
  expect(redoEntries().map(e=>e.label)).toEqual(['two','three','four']);
  fireEvent.click(screen.getByTestId('redo-menu-button'));
  expect(screen.getAllByRole('menuitem').map(item=>item.textContent)).toEqual(['two','three','four']);
 });

 it('offers up to 50 actions',()=>{
  for(let i=0;i<60;i++)add(`a${i}`,[]);
  render(()=><UndoRedoControl/>);
  fireEvent.click(screen.getByTestId('undo-menu-button'));
  expect(screen.getAllByRole('menuitem')).toHaveLength(50);
 });

 it('is not shown when the history size is 0',async()=>{
  await loadSettings({refresh:true,fetcher:(async()=>json({settings:{'ui.undoHistorySize':{value:0,source:'default',locked:false}}})) as Fetcher});
  render(()=><UndoRedoControl/>);
  expect(screen.queryByTestId('undo-redo')).toBeNull();
 });

 it('answers Ctrl+Z on the page but not inside a text field',async()=>{
  const log:string[]=[];add('moved',log);
  render(()=><><UndoRedoControl/><input data-testid="field"/></>);
  fireEvent.keyDown(screen.getByTestId('field'),{key:'z',ctrlKey:true});
  expect(log).toEqual([]);
  fireEvent.keyDown(document.body,{key:'z',ctrlKey:true});
  await waitFor(()=>expect(log).toEqual(['undo moved']));
 });
});
