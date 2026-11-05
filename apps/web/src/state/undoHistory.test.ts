import {afterEach,beforeEach,describe,expect,it,vi} from 'vitest';
import {clearSettings,loadSettings,type Fetcher} from '../api/settings.ts';
import {toast as toastStore,toasts} from '../components/toast/toast.ts';
import {loadCatalog} from '../i18n/index.ts';
import {clearUndoHistory,handleUndoKey,recordUndoable,redoEntries,redoLast,redoThrough,undoCapacity,undoEntries,undoLast,undoThrough,UndoUnavailable,type UndoableAction} from './undoHistory.ts';

const json=(body:unknown):Response=>new Response(JSON.stringify(body),{status:200,headers:{'content-type':'application/json'}});
const configure=(values:{size?:number;max?:number})=>loadSettings({refresh:true,fetcher:(async()=>json({settings:{
 ...(values.size!==undefined?{'ui.undoHistorySize':{value:values.size,source:'user',locked:false}}:{}),
 ...(values.max!==undefined?{'ui.undoHistoryMax':{value:values.max,source:'global',locked:false}}:{}),
}})) as Fetcher});
function action(label:string,over:Partial<UndoableAction>={}):UndoableAction&{log:string[]} {
 const log:string[]=[];
 return {label,accountId:'acct1',log,undo:async()=>{log.push(`undo ${label}`);},redo:async()=>{log.push(`redo ${label}`);},...over};
}
beforeEach(async()=>{await loadCatalog('undo');clearUndoHistory();toastStore.clear();});
afterEach(()=>{clearSettings();clearUndoHistory();toastStore.clear();document.body.innerHTML='';});

describe('history stack',()=>{
 it('keeps the newest 50 by default and never more',()=>{
  for(let i=0;i<60;i++)recordUndoable(action(`a${i}`));
  const entries=undoEntries();
  expect(entries).toHaveLength(50);
  expect(entries[0]!.label).toBe('a59');
  expect(entries.at(-1)!.label).toBe('a10');
 });
 it('follows the user size and the administrator cap',async()=>{
  await configure({size:10});
  expect(undoCapacity()).toBe(10);
  for(let i=0;i<15;i++)recordUndoable(action(`a${i}`));
  expect(undoEntries()).toHaveLength(10);
  await configure({size:40,max:25});
  expect(undoCapacity()).toBe(25);
  await configure({size:99});
  expect(undoCapacity()).toBe(50);
 });
 it('records nothing when off',async()=>{
  await configure({size:0});
  expect(recordUndoable(action('x'))).toBeNull();
  expect(undoEntries()).toHaveLength(0);
  clearSettings();
  await configure({size:30,max:0});
  expect(recordUndoable(action('x'))).toBeNull();
 });
 it('undo moves an entry to redo and redo back; recording clears redo',async()=>{
  const a=action('a'),b=action('b');
  recordUndoable(a);recordUndoable(b);
  expect(await undoLast()).toBe(1);
  expect(b.log).toEqual(['undo b']);
  expect(redoEntries().map(e=>e.label)).toEqual(['b']);
  expect(await redoLast()).toBe(1);
  expect(b.log).toEqual(['undo b','redo b']);
  expect(undoEntries().map(e=>e.label)).toEqual(['b','a']);
  await undoLast();
  recordUndoable(action('c'));
  expect(redoEntries()).toHaveLength(0);
 });
 it('remembers the account of each entry',()=>{
  recordUndoable(action('a',{accountId:'one'}));recordUndoable(action('b',{accountId:'two'}));
  expect(undoEntries().map(e=>e.accountId)).toEqual(['two','one']);
 });
 it('clears on sign-out or account switch',()=>{
  recordUndoable(action('a'));
  clearUndoHistory();
  expect(undoEntries()).toHaveLength(0);
  expect(redoEntries()).toHaveLength(0);
 });
 it('does not record while an entry replays (a redo that repeats an action)',async()=>{
  recordUndoable(action('a',{redo:async()=>{expect(recordUndoable(action('inner'))).toBeNull();}}));
  await undoLast();await redoLast();
  expect(undoEntries().map(e=>e.label)).toEqual(['a']);
 });
});

describe('failure handling',()=>{
 it('drops an entry whose target vanished, with a toast naming the reason, and leaves the rest',async()=>{
  const a=action('a'),b=action('b',{undo:async()=>{throw new UndoUnavailable('the message was moved');}});
  recordUndoable(a);recordUndoable(b);
  expect(await undoLast()).toBe(0);
  expect(undoEntries().map(e=>e.label)).toEqual(['a']);
  expect(redoEntries()).toHaveLength(0);
  expect(toasts().at(-1)?.message).toBe("Can't undo: the message was moved");
 });
 it('a generic failure names the action',async()=>{
  recordUndoable(action('Archived',{undo:async()=>{throw new Error('boom');}}));
  await undoLast();
  expect(toasts().at(-1)?.message).toBe("Can't undo: Archived");
 });
 it('multi-step undo stops at the first failure',async()=>{
  const a=action('a'),b=action('b',{undo:async()=>{throw new UndoUnavailable();}}),c=action('c');
  recordUndoable(a);const eb=recordUndoable(b)!;recordUndoable(c);
  const first=undoEntries().at(-1)!;
  expect(await undoThrough(first.id)).toBe(1);
  expect(c.log).toEqual(['undo c']);
  expect(b.log).toEqual([]);
  expect(a.log).toEqual([]);
  expect(undoEntries().map(e=>e.label)).toEqual(['a']);
  expect(eb.id).toBeGreaterThan(0);
 });
});

describe('multi-step',()=>{
 it('undoes the chosen action and every newer one, newest first, and redoes back through one',async()=>{
  const items=['a','b','c','d'].map(label=>action(label));
  const order:string[]=[];
  for(const item of items){const original=item.undo;item.undo=async()=>{order.push(item.label);await original();};recordUndoable(item);}
  const target=undoEntries().find(e=>e.label==='b')!;
  expect(await undoThrough(target.id)).toBe(3);
  expect(order).toEqual(['d','c','b']);
  expect(undoEntries().map(e=>e.label)).toEqual(['a']);
  expect(redoEntries().map(e=>e.label)).toEqual(['b','c','d']);
  const redoTarget=redoEntries().find(e=>e.label==='c')!;
  expect(await redoThrough(redoTarget.id)).toBe(2);
  expect(undoEntries().map(e=>e.label)).toEqual(['c','b','a']);
  expect(await undoThrough(9999)).toBeNull();
 });
});

describe('keyboard shortcut',()=>{
 const key=(init:KeyboardEventInit,target:Element=document.body)=>{
  const event=new KeyboardEvent('keydown',{bubbles:true,cancelable:true,...init});
  Object.defineProperty(event,'target',{value:target});
  return {event,handled:handleUndoKey(event)};
 };
 it('Ctrl/Cmd+Z undoes; Ctrl/Cmd+Shift+Z and Ctrl+Y redo',async()=>{
  const a=action('a');recordUndoable(a);
  const first=key({key:'z',ctrlKey:true});
  expect(first.handled).toBe(true);expect(first.event.defaultPrevented).toBe(true);
  await vi.waitFor(()=>expect(redoEntries()).toHaveLength(1));
  expect(key({key:'Z',ctrlKey:true,shiftKey:true}).handled).toBe(true);
  await vi.waitFor(()=>expect(undoEntries()).toHaveLength(1));
  await undoLast();
  expect(key({key:'y',ctrlKey:true}).handled).toBe(true);
  await vi.waitFor(()=>expect(undoEntries()).toHaveLength(1));
  await undoLast();
  expect(key({key:'z',metaKey:true,shiftKey:true}).handled).toBe(true);
 });
 it('leaves text fields, editors and dialogs to the browser',()=>{
  recordUndoable(action('a'));
  const input=document.createElement('input');const area=document.createElement('textarea');
  const editable=document.createElement('div');editable.setAttribute('contenteditable','true');
  const inner=document.createElement('span');editable.append(inner);
  const dialog=document.createElement('div');dialog.setAttribute('aria-modal','true');const inDialog=document.createElement('button');dialog.append(inDialog);
  document.body.append(input,area,editable,dialog);
  for(const target of [input,area,editable,inner,inDialog]){
   const result=key({key:'z',ctrlKey:true},target);
   expect(result.handled).toBe(false);expect(result.event.defaultPrevented).toBe(false);
  }
  expect(undoEntries()).toHaveLength(1);
  const checkbox=document.createElement('input');checkbox.type='checkbox';document.body.append(checkbox);
  expect(key({key:'z',ctrlKey:true},checkbox).handled).toBe(true);
 });
 it('ignores other keys, plain Z, empty history and an off history',async()=>{
  expect(key({key:'z',ctrlKey:true}).handled).toBe(false);
  recordUndoable(action('a'));
  expect(key({key:'z'}).handled).toBe(false);
  expect(key({key:'x',ctrlKey:true}).handled).toBe(false);
  expect(key({key:'z',ctrlKey:true,altKey:true}).handled).toBe(false);
  await configure({size:0});
  expect(key({key:'z',ctrlKey:true}).handled).toBe(false);
 });
});
