import {createSignal,onCleanup} from 'solid-js';
import {afterDepthRange,depthFromX,keyMoveFor,movingSet,placementForKey,resolveDrop,zoneOf,type DndFolder,type DndPolicy,type DndRow,type DropProblem,type DropZone,type Placement} from './folderDnd.ts';
/** The drag type of a folder; message drags use their own, so the two never mix. */
export const FOLDER_DRAG_TYPE='application/x-mailwoman-folder';
export const EXPAND_DELAY_MS=600;
const EDGE_PX=36,SCROLL_STEP=14;
export interface DropIndicator {id:string;zone:DropZone;depth:number;ok:boolean;problem?:DropProblem}
export interface FolderDndOptions {
 /** The visible rows, depth-first. */
 rows:()=>readonly DndRow[];
 /** The folders with their visual parents, in display order. */
 folders:()=>readonly DndFolder[];
 policy:()=>DndPolicy;
 selected:()=>ReadonlySet<string>;
 /** The scrolling element holding the rows (auto-scroll near its edges, depth from the pointer x). */
 scroller:()=>HTMLElement|undefined;
 /** Pixels of indent per depth level. */
 indent:()=>number;
 canDrag:(id:string)=>boolean;
 onExpand:(id:string)=>void;
 /** A valid drop or keyboard move; the caller applies it. */
 onMove:(moving:string[],placement:Placement)=>void;
 /** An invalid drop or key. */
 onRefused?:(problem:DropProblem)=>void;
 /** Text for the live region after a keyboard move. */
 describe?:(move:'up'|'down'|'indent'|'outdent',name:string)=>string;
 ghost?:(id:string)=>HTMLElement|undefined;
}
/**
 * The folder drag-and-drop behaviour, headless: the sidebar and the Settings folder list both bind their rows to it.
 * A row spreads `rowProps(id)` (draggable, drag handlers, `data-drop`, `--drop-depth`) and renders the indicator line from CSS
 * on `[data-drop]`. Folder drags carry FOLDER_DRAG_TYPE only, so message drops onto a folder stay unambiguous.
 */
export function createFolderDnd(options:FolderDndOptions){
 const [dragging,setDragging]=createSignal<string[]>([]);
 const [indicator,setIndicator]=createSignal<DropIndicator|null>(null);
 const [announcement,setAnnouncement]=createSignal('');
 let expandTimer:ReturnType<typeof setTimeout>|undefined,expandId:string|null=null,scrollTimer:ReturnType<typeof setInterval>|undefined,velocity=0,ownRoot:Document|undefined;
 const clearExpand=():void=>{if(expandTimer)clearTimeout(expandTimer);expandTimer=undefined;expandId=null;};
 const stopScroll=():void=>{if(scrollTimer)clearInterval(scrollTimer);scrollTimer=undefined;velocity=0;};
 function end():void {setDragging([]);setIndicator(null);clearExpand();stopScroll();ownRoot?.removeEventListener('keydown',escape,true);ownRoot=undefined;}
 function escape(event:KeyboardEvent):void {if(event.key==='Escape')end();}
 onCleanup(end);
 const isFolderDrag=(event:DragEvent):boolean=>dragging().length>0||(event.dataTransfer?.types?.includes(FOLDER_DRAG_TYPE)??false);
 function autoScroll(event:DragEvent):void {
  const el=options.scroller();if(!el)return;
  const rect=el.getBoundingClientRect();
  velocity=event.clientY<rect.top+EDGE_PX?-SCROLL_STEP:event.clientY>rect.bottom-EDGE_PX?SCROLL_STEP:0;
  if(velocity&&!scrollTimer)scrollTimer=setInterval(()=>{if(velocity)el.scrollTop+=velocity;else stopScroll();},30);
  else if(!velocity)stopScroll();
 }
 function start(event:DragEvent,id:string):void {
  if(!options.canDrag(id)){event.preventDefault();return;}
  const ids=movingSet(options.folders(),options.selected(),id);
  setDragging(ids);
  const transfer=event.dataTransfer;
  if(transfer){
   transfer.effectAllowed='move';transfer.setData(FOLDER_DRAG_TYPE,ids.join('\n'));transfer.setData('text/plain',ids.join('\n'));
   const ghost=options.ghost?.(id);if(ghost&&transfer.setDragImage)transfer.setDragImage(ghost,12,12);
  }
  const doc=(event.currentTarget as HTMLElement|null)?.ownerDocument;
  if(doc){ownRoot=doc;doc.addEventListener('keydown',escape,true);}
 }
 /** Returns true when the event was a folder drag (and handled), so the caller skips its message-drop handling. */
 function over(event:DragEvent,id:string):boolean {
  if(!isFolderDrag(event))return false;
  const el=event.currentTarget as HTMLElement,rect=el.getBoundingClientRect(),zone=zoneOf(event.clientY,rect.top,rect.height);
  const rows=options.rows(),scroller=options.scroller(),base=scroller?.getBoundingClientRect().left??rect.left;
  const range=zone==='after'?afterDepthRange(rows,id):{min:0,max:0};
  const rtl=getComputedStyle(el).direction==='rtl',x=rtl?2*base+(scroller?.clientWidth??0)-event.clientX:event.clientX;
  const asked=zone==='after'?depthFromX(x,base+8,options.indent(),range.min,range.max):undefined;
  const result=resolveDrop(rows,options.folders(),dragging(),{targetId:id,zone,depth:asked},options.policy());
  const row=rows.find(r=>r.id===id);
  const shown=zone==='inside'?(row?.depth??0)+1:zone==='before'?(row?.depth??0):(asked??row?.depth??0);
  autoScroll(event);
  if(result.ok){
   event.preventDefault();if(event.dataTransfer)event.dataTransfer.dropEffect='move';
   setIndicator({id,zone,depth:shown,ok:true});
   if(zone==='inside'&&row?.hasChildren&&!row.expanded){
    if(expandId!==id){clearExpand();expandId=id;expandTimer=setTimeout(()=>{options.onExpand(id);clearExpand();},EXPAND_DELAY_MS);}
   }else clearExpand();
  }else{
   // The drop is refused: no preventDefault, so the cursor shows "not allowed".
   if(event.dataTransfer)event.dataTransfer.dropEffect='none';
   setIndicator({id,zone,depth:shown,ok:false,problem:result.problem});clearExpand();
  }
  return true;
 }
 function drop(event:DragEvent,id:string):boolean {
  if(!isFolderDrag(event))return false;
  event.preventDefault();
  const shown=indicator(),moving=dragging();
  end();
  if(!shown||shown.id!==id)return true;
  if(!shown.ok){if(shown.problem)options.onRefused?.(shown.problem);return true;}
  const result=resolveDrop(options.rows(),options.folders(),moving,{targetId:id,zone:shown.zone,depth:shown.zone==='after'?shown.depth:undefined},options.policy());
  if(result.ok)options.onMove(moving,result.placement);else options.onRefused?.(result.problem);
  return true;
 }
 function leave(event:DragEvent,id:string):void {
  const next=event.relatedTarget as Node|null;
  if(next&&(event.currentTarget as HTMLElement).contains(next))return;
  if(indicator()?.id===id)setIndicator(null);
  if(expandId===id)clearExpand();
 }
 /** Alt+arrows on a focused row. Returns true when the key was a move key (handled). */
 function key(event:KeyboardEvent,id:string):boolean {
  const el=event.currentTarget as HTMLElement|null,rtl=el?getComputedStyle(el).direction==='rtl':false;
  const move=keyMoveFor(event,rtl);if(!move)return false;
  event.preventDefault();
  if(!options.canDrag(id)){options.onRefused?.(options.policy().locked?'locked':'disabled');return true;}
  const result=placementForKey(options.folders(),id,move,options.policy());
  if(!result.ok){if(result.problem!=='noop')options.onRefused?.(result.problem);return true;}
  options.onMove([id],result.placement);
  setAnnouncement(options.describe?.(move,options.folders().find(f=>f.id===id)?.name??'')??'');
  return true;
 }
 function rowProps(id:string){
  return {
   get draggable(){return options.canDrag(id);},
   onDragStart:(event:DragEvent)=>start(event,id),
   onDragEnd:()=>end(),
   onDragOver:(event:DragEvent)=>{over(event,id);},
   onDragLeave:(event:DragEvent)=>leave(event,id),
   onDrop:(event:DragEvent)=>{drop(event,id);},
   get 'data-drop'(){const i=indicator();return i&&i.id===id?(i.ok?i.zone:`${i.zone}-refused`):undefined;},
   get 'data-dragging'(){return dragging().includes(id)?'true':undefined;},
  };
 }
 /** The CSS variable the insertion line is indented by; merge it into the row's style. */
 function dropVars(id:string):Record<string,string>{const i=indicator();return i&&i.id===id?{'--drop-depth':String(i.depth)}:{};}
 return {dragging,indicator,announcement,setAnnouncement,rowProps,dropVars,over,drop,leave,start,end,key};
}
export type FolderDnd=ReturnType<typeof createFolderDnd>;
