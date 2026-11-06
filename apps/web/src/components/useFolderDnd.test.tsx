import {createRoot} from 'solid-js';
import {afterEach,describe,expect,it,vi} from 'vitest';
import {DEFAULT_POLICY,type DndFolder,type DndPolicy,type DndRow} from './folderDnd.ts';
import {createFolderDnd,EXPAND_DELAY_MS,FOLDER_DRAG_TYPE} from './useFolderDnd.ts';
const f=(id:string,parentId:string|null=null,extra:Partial<DndFolder>={}):DndFolder=>({id,name:id.toUpperCase(),parentId,role:null,totalEmails:0,unreadEmails:0,...extra});
const folders=[f('inbox',null,{role:'inbox'}),f('a'),f('b','a'),f('e')];
const rows:DndRow[]=[{id:'inbox',depth:0,hasChildren:false,expanded:false},{id:'a',depth:0,hasChildren:true,expanded:false},{id:'e',depth:0,hasChildren:false,expanded:false}];
function rowEl(top=0,height=40):HTMLElement{const el=document.createElement('div');el.getBoundingClientRect=()=>({top,height,left:0,width:300,bottom:top+height,right:300,x:0,y:top,toJSON:()=>({})});document.body.append(el);return el;}
function dragEvent(el:HTMLElement,clientY:number,types:string[]=[FOLDER_DRAG_TYPE]):DragEvent{
 const event={currentTarget:el,clientY,clientX:10,relatedTarget:null,dataTransfer:{types,setData:vi.fn(),effectAllowed:'',dropEffect:''},preventDefault:vi.fn()};
 return event as unknown as DragEvent;
}
function setup(policy:Partial<DndPolicy>={},selected:string[]=[]){
 const moves:{moving:string[];placement:unknown}[]=[],refused:string[]=[],expanded:string[]=[];
 let dnd!:ReturnType<typeof createFolderDnd>;
 const dispose=createRoot(d=>{dnd=createFolderDnd({rows:()=>rows,folders:()=>folders,policy:()=>({...DEFAULT_POLICY,...policy}),selected:()=>new Set(selected),scroller:()=>undefined,indent:()=>16,canDrag:id=>!(policy.locked)&&!folders.find(x=>x.id===id)?.role,onExpand:id=>expanded.push(id),onMove:(moving,placement)=>moves.push({moving,placement}),onRefused:p=>refused.push(p),describe:(m,name)=>`${name} ${m}`});return d;});
 return {dnd,moves,refused,expanded,dispose};
}
afterEach(()=>{vi.useRealTimers();document.body.innerHTML='';});
describe('folder drag binding',()=>{
 it('drops before a row with an insertion indicator, then moves',()=>{
  const s=setup(),el=rowEl();
  s.dnd.start(dragEvent(el,0),'e');
  const over=dragEvent(el,2);s.dnd.over(over,'a');
  expect(s.dnd.indicator()).toMatchObject({id:'a',zone:'before',ok:true});expect(s.dnd.rowProps('a')['data-drop']).toBe('before');
  expect(over.preventDefault).toHaveBeenCalled();
  s.dnd.drop(dragEvent(el,2),'a');
  expect(s.moves).toEqual([{moving:['e'],placement:{parentId:null,beforeId:'a'}}]);expect(s.dnd.dragging()).toEqual([]);expect(s.dnd.indicator()).toBeNull();
  s.dispose();
 });
 it('leaves message drags alone',()=>{
  const s=setup(),el=rowEl();
  const message=dragEvent(el,20,['application/x-mailwoman-message']);
  expect(s.dnd.over(message,'a')).toBe(false);expect(message.preventDefault).not.toHaveBeenCalled();
  expect(s.dnd.drop(message,'a')).toBe(false);expect(s.moves).toEqual([]);
  s.dispose();
 });
 it('marks an invalid drop as refused without allowing it',()=>{
  const s=setup(),el=rowEl();
  s.dnd.start(dragEvent(el,0),'a');
  const over=dragEvent(el,20);s.dnd.over(over,'a');
  expect(s.dnd.indicator()).toMatchObject({ok:false,problem:'self'});expect(over.preventDefault).not.toHaveBeenCalled();
  s.dnd.drop(dragEvent(el,20),'a');expect(s.moves).toEqual([]);expect(s.refused).toEqual(['self']);
  s.dispose();
 });
 it('expands a collapsed folder after hovering its middle for 600 ms, and not when the pointer leaves',()=>{
  vi.useFakeTimers();
  const s=setup(),el=rowEl();
  s.dnd.start(dragEvent(el,0),'e');s.dnd.over(dragEvent(el,20),'a');
  vi.advanceTimersByTime(EXPAND_DELAY_MS-1);expect(s.expanded).toEqual([]);
  vi.advanceTimersByTime(2);expect(s.expanded).toEqual(['a']);
  s.expanded.length=0;s.dnd.over(dragEvent(el,20),'a');s.dnd.leave(dragEvent(el,20),'a');vi.advanceTimersByTime(EXPAND_DELAY_MS*2);expect(s.expanded).toEqual([]);
  s.dispose();
 });
 it('cancels with Escape',()=>{
  const s=setup(),el=rowEl();
  s.dnd.start(dragEvent(el,0),'e');expect(s.dnd.dragging()).toEqual(['e']);
  document.dispatchEvent(new KeyboardEvent('keydown',{key:'Escape'}));
  expect(s.dnd.dragging()).toEqual([]);
  s.dispose();
 });
 it('drags the whole selection when the dragged folder is part of it',()=>{
  const s=setup({},['a','e']),el=rowEl();
  s.dnd.start(dragEvent(el,0),'e');expect(s.dnd.dragging()).toEqual(['a','e']);
  s.dispose();
 });
 it('does not start dragging when the order is locked or the folder is a system folder',()=>{
  const locked=setup({locked:true}),el=rowEl(),event=dragEvent(el,0);
  locked.dnd.start(event,'e');expect(locked.dnd.dragging()).toEqual([]);expect(event.preventDefault).toHaveBeenCalled();
  locked.dispose();
  const system=setup();system.dnd.start(dragEvent(el,0),'inbox');expect(system.dnd.dragging()).toEqual([]);system.dispose();
 });
 it('moves with Alt+arrows and announces it; refuses when locked',()=>{
  const s=setup(),el=document.createElement('div'),key=(k:string)=>({key:k,altKey:true,currentTarget:el,preventDefault:vi.fn()}) as unknown as KeyboardEvent;
  expect(s.dnd.key(key('ArrowUp'),'e')).toBe(true);
  expect(s.moves).toEqual([{moving:['e'],placement:{parentId:null,beforeId:'a'}}]);expect(s.dnd.announcement()).toBe('E up');
  expect(s.dnd.key({key:'ArrowUp',altKey:false,currentTarget:el,preventDefault:vi.fn()} as unknown as KeyboardEvent,'e')).toBe(false);
  s.dispose();
  const locked=setup({locked:true});locked.dnd.key(key('ArrowUp'),'e');expect(locked.moves).toEqual([]);expect(locked.refused).toEqual(['locked']);locked.dispose();
 });

 describe('touch (long press) drag',()=>{
  const pointer=(el:HTMLElement,x:number,y:number,type='touch')=>({pointerType:type,isPrimary:true,button:0,pointerId:1,clientX:x,clientY:y,currentTarget:el}) as unknown as PointerEvent;
  function touchSetup(policy:Partial<DndPolicy>={}){
   const s=setup(policy),el=rowEl(),a=rowEl(100);
   el.setAttribute('data-dnd-id','e');a.setAttribute('data-dnd-id','a');
   el.setPointerCapture=vi.fn();el.releasePointerCapture=vi.fn();
   let under:Element|null=a;
   document.elementFromPoint=vi.fn(()=>under);
   return {...s,el,a,at:(next:Element|null)=>{under=next;}};
  }
  it('lifts after 350 ms, shows the drop target while moving and moves on release',()=>{
   vi.useFakeTimers();
   const s=touchSetup(),p=s.dnd.rowProps('e') as unknown as Record<string,(e:PointerEvent)=>void>;
   p.onPointerDown!(pointer(s.el,10,10));
   vi.advanceTimersByTime(349);expect(s.dnd.dragging()).toEqual([]);
   vi.advanceTimersByTime(2);expect(s.dnd.dragging()).toEqual(['e']);expect(s.dnd.lifted()).toBe('e');expect(s.dnd.rowProps('e')['data-lifted']).toBe('true');
   expect(s.el.setPointerCapture).toHaveBeenCalled();
   p.onPointerMove!(pointer(s.el,10,102));
   expect(s.dnd.indicator()).toMatchObject({id:'a',zone:'before',ok:true});
   p.onPointerUp!(pointer(s.el,10,102));
   expect(s.moves).toEqual([{moving:['e'],placement:{parentId:null,beforeId:'a'}}]);
   expect(s.dnd.dragging()).toEqual([]);expect(s.dnd.lifted()).toBeNull();
   s.dispose();
  });
  it('treats movement before the hold elapses as a scroll',()=>{
   vi.useFakeTimers();
   const s=touchSetup(),p=s.dnd.rowProps('e') as unknown as Record<string,(e:PointerEvent)=>void>;
   p.onPointerDown!(pointer(s.el,10,10));p.onPointerMove!(pointer(s.el,10,40));
   vi.advanceTimersByTime(1000);expect(s.dnd.dragging()).toEqual([]);
   s.dispose();
  });
  it('ignores the mouse, a locked folder, and cancels on pointercancel or Escape without moving',()=>{
   vi.useFakeTimers();
   const s=touchSetup(),p=s.dnd.rowProps('e') as unknown as Record<string,(e:PointerEvent)=>void>;
   p.onPointerDown!(pointer(s.el,10,10,'mouse'));vi.advanceTimersByTime(1000);expect(s.dnd.dragging()).toEqual([]);
   p.onPointerDown!(pointer(s.el,10,10));vi.advanceTimersByTime(400);expect(s.dnd.dragging()).toEqual(['e']);
   p.onPointerCancel!(pointer(s.el,10,10));expect(s.dnd.dragging()).toEqual([]);
   p.onPointerDown!(pointer(s.el,10,10));vi.advanceTimersByTime(400);expect(s.dnd.dragging()).toEqual(['e']);
   document.dispatchEvent(new KeyboardEvent('keydown',{key:'Escape'}));expect(s.dnd.dragging()).toEqual([]);
   p.onPointerUp!(pointer(s.el,10,102));expect(s.moves).toEqual([]);
   s.dispose();
   const locked=touchSetup({locked:true}),lp=locked.dnd.rowProps('e') as unknown as Record<string,(e:PointerEvent)=>void>;
   lp.onPointerDown!(pointer(locked.el,10,10));vi.advanceTimersByTime(1000);expect(locked.dnd.dragging()).toEqual([]);
   locked.dispose();
  });
  it('refuses a drop onto itself and reports why',()=>{
   vi.useFakeTimers();
   const s=touchSetup(),p=s.dnd.rowProps('e') as unknown as Record<string,(e:PointerEvent)=>void>;
   s.at(s.el);
   p.onPointerDown!(pointer(s.el,10,10));vi.advanceTimersByTime(400);
   p.onPointerMove!(pointer(s.el,10,20));expect(s.dnd.indicator()).toMatchObject({ok:false,problem:'self'});
   p.onPointerUp!(pointer(s.el,10,20));expect(s.moves).toEqual([]);expect(s.refused).toEqual(['self']);
   s.dispose();
  });
 });
});
