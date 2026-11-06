import {createSignal,onCleanup} from 'solid-js';
import {createTouchDrag} from './touchDrag.ts';
/**
 * Drag behaviour of the sidebar's Pinned group: reorder the pins among themselves, or drop one outside the group to
 * unpin it. Pins have their own order (the `folderPinned` list); this never changes where a folder sits in the tree.
 * The administrator's force-pinned folders come first and cannot be moved or unpinned.
 */
export const PIN_DRAG_TYPE='application/x-mailwoman-pin';
export interface PinIndicator {id:string;zone:'before'|'after'}
export interface PinDndOptions {
 /** Every pinned id in display order: forced first, then the user's own. */
 ids:()=>readonly string[];
 forced:()=>ReadonlySet<string>;
 /** False while folder arranging is off or locked. */
 enabled:()=>boolean;
 /** The user's own pins in their new order (forced ones excluded). */
 onReorder:(userOrder:string[])=>void;
 onUnpin:(id:string)=>void;
 label?:(id:string)=>string;
 /** Whether an element lies inside the pinned group / inside the sidebar navigation. */
 inGroup:(el:Element|null)=>boolean;
 inNav:(el:Element|null)=>boolean;
}
/** The new order of the user's pins after moving `id` next to `targetId`. Null when nothing changes or the move is not allowed. */
export function reorderPins(ids:readonly string[],forced:ReadonlySet<string>,id:string,targetId:string,zone:'before'|'after'):string[]|null {
 if(forced.has(id)||id===targetId||!ids.includes(id)||!ids.includes(targetId))return null;
 const own=ids.filter(value=>!forced.has(value)&&value!==id);
 // A forced row is not a slot: dropping next to one means the start of the user's own pins.
 let at=forced.has(targetId)?0:own.indexOf(targetId)+(zone==='after'?1:0);
 if(at<0)at=own.length;
 own.splice(at,0,id);
 const before=ids.filter(value=>!forced.has(value));
 return own.every((value,index)=>value===before[index])?null:own;
}
export function createPinDnd(options:PinDndOptions){
 const [dragging,setDragging]=createSignal<string|null>(null);
 const [indicator,setIndicator]=createSignal<PinIndicator|null>(null);
 const [lifted,setLifted]=createSignal<string|null>(null);
 const canDrag=(id:string):boolean=>options.enabled()&&!options.forced().has(id);
 const zoneAt=(el:HTMLElement,y:number):'before'|'after'=>{const rect=el.getBoundingClientRect();return y<rect.top+rect.height/2?'before':'after';};
 const end=():void=>{setDragging(null);setIndicator(null);setLifted(null);};
 function show(el:HTMLElement,id:string,y:number):void {
  const dragged=dragging();
  if(!dragged||dragged===id){setIndicator(null);return;}
  setIndicator({id,zone:zoneAt(el,y)});
 }
 function commitOn(id:string|null):void {
  const dragged=dragging(),shown=indicator();
  end();
  if(!dragged||!id||!shown||shown.id!==id)return;
  const next=reorderPins(options.ids(),options.forced(),dragged,id,shown.zone);
  if(next)options.onReorder(next);
 }
 const touch=createTouchDrag({
  begin:id=>{if(!canDrag(id))return false;setDragging(id);setLifted(id);return true;},
  move:(_x,y,under)=>{
   const row=under?.closest<HTMLElement>('[data-pin-id]');
   if(row&&row.getAttribute('data-pin-id'))show(row,row.getAttribute('data-pin-id')!,y);else setIndicator(null);
  },
  finish:(_x,_y,under)=>{
   const row=under?.closest<HTMLElement>('[data-pin-id]'),id=row?.getAttribute('data-pin-id')??null,dragged=dragging();
   if(row&&id)commitOn(id);
   else if(dragged&&options.inNav(under)&&!options.inGroup(under)){end();options.onUnpin(dragged);}
   else end();
  },
  cancel:end,
  ...(options.label?{label:options.label}:{}),
 });
 onCleanup(()=>touch.dispose());
 /** Row binding; the same shape the folder tree's binding has, so FolderContextMenu takes either. */
 function rowProps(id:string){
  return {
   ...touch.props(id),
   'data-pin-id':id,
   get draggable(){return canDrag(id);},
   onDragStart:(event:DragEvent)=>{
    if(!canDrag(id)){event.preventDefault();return;}
    event.dataTransfer?.setData(PIN_DRAG_TYPE,id);event.dataTransfer?.setData('text/plain',id);
    if(event.dataTransfer)event.dataTransfer.effectAllowed='move';
    // Changing the DOM inside dragstart can cancel the drag in some browsers.
    setTimeout(()=>setDragging(id),0);
   },
   onDragEnd:()=>end(),
   onDragOver:(event:DragEvent)=>{
    if(!event.dataTransfer?.types?.includes(PIN_DRAG_TYPE)&&!dragging())return;
    event.preventDefault();event.stopPropagation();
    if(event.dataTransfer)event.dataTransfer.dropEffect='move';
    show(event.currentTarget as HTMLElement,id,event.clientY);
   },
   onDragLeave:(event:DragEvent)=>{const next=event.relatedTarget as Node|null;if(next&&(event.currentTarget as HTMLElement).contains(next))return;if(indicator()?.id===id)setIndicator(null);},
   onDrop:(event:DragEvent)=>{if(!dragging())return;event.preventDefault();event.stopPropagation();commitOn(id);},
   get 'data-drop'(){const i=indicator();return i&&i.id===id?i.zone:undefined;},
   get 'data-dragging'(){return dragging()===id?'true':undefined;},
   get 'data-lifted'(){return lifted()===id?'true':undefined;},
  };
 }
 const dropVars=(_id:string):Record<string,string>=>({});
 /** The unpin target: the strip shown while a pin is dragged, and the rest of the navigation. */
 const unpinProps={
  onDragOver:(event:DragEvent)=>{if(!dragging())return;event.preventDefault();if(event.dataTransfer)event.dataTransfer.dropEffect='move';},
  onDrop:(event:DragEvent)=>{const id=dragging();if(!id)return;event.preventDefault();end();options.onUnpin(id);},
 };
 /** The group's own area: a drop on its label or gaps is swallowed so it does not count as dropping outside. */
 const groupProps={
  onDragOver:(event:DragEvent)=>{if(!dragging())return;event.preventDefault();event.stopPropagation();if(event.dataTransfer)event.dataTransfer.dropEffect='none';},
  onDrop:(event:DragEvent)=>{if(!dragging())return;event.preventDefault();event.stopPropagation();end();},
 };
 return {dragging,indicator,rowProps,dropVars,unpinProps,groupProps,end};
}
export type PinDnd=ReturnType<typeof createPinDnd>;
