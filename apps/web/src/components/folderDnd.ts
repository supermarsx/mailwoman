import {TOP_LEVEL,descendantsOf,type TreeFolder} from './folderTree.ts';
/**
 * Folder drag-and-drop, as pure functions (the sidebar and the Settings folder list share them).
 *
 * Two layers, kept apart on purpose:
 *  - VISUAL ARRANGEMENT: the order (`folderOrder`), the pins and the "visual parent" overrides (`folderParent`)
 *    live in the user's UI preferences. They never touch the server, so they work for IMAP, POP and read-only accounts.
 *  - REAL HIERARCHY: the folder's server parent (JMAP `Mailbox/set parentId`, IMAP RENAME). Only a nest or un-nest
 *    changes it, and only when the account allows it and the user chose "move on the server".
 * Reordering among siblings is always the visual layer.
 */
export type DropZone='before'|'inside'|'after';
export interface DndFolder extends TreeFolder {myRights?:{mayCreateChild?:boolean}|undefined}
/** The visible rows, depth-first, as the list renders them. */
export interface DndRow {id:string;depth:number;hasChildren:boolean;expanded:boolean}
export interface DropIntent {targetId:string;zone:DropZone;/** The depth the pointer asks for on an 'after' drop (outdent). */depth?:number|undefined}
/** Where the moved folders land: under `parentId` (null = top level), before `beforeId` (null = last). */
export interface Placement {parentId:string|null;beforeId:string|null}
export interface DndPolicy {
 enabled:boolean;allowNesting:boolean;allowSystemFolders:boolean;maxDepth:number;locked:boolean;
 /** Folders the administrator pinned: never moved by users. */
 forcePinned?:ReadonlySet<string>;
}
export const DEFAULT_POLICY:DndPolicy={enabled:true,allowNesting:true,allowSystemFolders:false,maxDepth:10,locked:false};
export type DropProblem='disabled'|'locked'|'self'|'descendant'|'system'|'no-children'|'nesting-off'|'depth'|'forced'|'unknown'|'noop';
export type DropResult={ok:true;placement:Placement}|{ok:false;problem:DropProblem};
/** The row zones: the top quarter is "before", the bottom quarter "after", the middle "inside". */
export function zoneOf(y:number,top:number,height:number):DropZone {
 if(height<=0)return 'inside';
 const at=(y-top)/height;
 return at<0.25?'before':at>0.75?'after':'inside';
}
/** The depth the pointer asks for: how many indents right of the row's left edge, clamped. */
export function depthFromX(x:number,left:number,indent:number,min:number,max:number):number {
 const raw=indent>0?Math.round((x-left)/indent):max;
 return Math.max(min,Math.min(max,raw));
}
/** The visual parent overrides applied to the server's folders. An override to a missing folder, to itself or one that would close a cycle is ignored. */
export function applyVisualParents<T extends TreeFolder>(folders:readonly T[],overrides:Readonly<Record<string,string>>|undefined):T[] {
 if(!overrides)return [...folders];
 const ids=new Set(folders.map(f=>f.id)),parents=new Map<string,string|null>();
 for(const f of folders)parents.set(f.id,f.parentId&&ids.has(f.parentId)&&f.parentId!==f.id?f.parentId:null);
 for(const f of folders){
  const want=overrides[f.id];
  if(want===undefined||f.role||f.mailwomanSearchQuery!==undefined)continue;
  const next=want===TOP_LEVEL?null:ids.has(want)&&want!==f.id?want:undefined;
  if(next===undefined)continue;
  const previous=parents.get(f.id)??null;
  parents.set(f.id,next);
  let cursor=next,steps=0;
  while(cursor&&cursor!==f.id&&steps++<ids.size)cursor=parents.get(cursor)??null;
  if(cursor===f.id)parents.set(f.id,previous);
 }
 return folders.map(f=>{const parent=parents.get(f.id)??null;return (f.parentId??null)===parent?f:{...f,parentId:parent};});
}
function depthOf(by:ReadonlyMap<string,DndFolder>,id:string):number {
 let depth=0,cursor=by.get(id)?.parentId,guard=0;
 while(cursor&&by.has(cursor)&&guard++<64){depth++;cursor=by.get(cursor)?.parentId;}
 return depth;
}
function heightOf(folders:readonly DndFolder[],id:string):number {
 let height=0,frontier=[id];
 const seen=new Set([id]);
 while(frontier.length){
  const next=folders.filter(f=>f.parentId&&frontier.includes(f.parentId)&&!seen.has(f.id));
  if(!next.length)break;
  height++;next.forEach(f=>seen.add(f.id));frontier=next.map(f=>f.id);
 }
 return height;
}
/** Whether the folders may be placed there. `folders` carry their visual parents already. */
export function validatePlacement(folders:readonly DndFolder[],moving:readonly string[],placement:Placement,policy:DndPolicy):DropResult {
 if(!policy.enabled)return {ok:false,problem:'disabled'};
 if(policy.locked)return {ok:false,problem:'locked'};
 if(!moving.length)return {ok:false,problem:'noop'};
 const by=new Map(folders.map(f=>[f.id,f]));
 for(const id of moving){
  const f=by.get(id);
  if(!f||f.mailwomanSearchQuery!==undefined)return {ok:false,problem:'unknown'};
  if(f.role&&!policy.allowSystemFolders)return {ok:false,problem:'system'};
  if(policy.forcePinned?.has(id))return {ok:false,problem:'forced'};
 }
 const parent=placement.parentId?by.get(placement.parentId):null;
 if(placement.parentId&&!parent)return {ok:false,problem:'unknown'};
 if(placement.beforeId&&!by.has(placement.beforeId))return {ok:false,problem:'unknown'};
 for(const id of moving){
  if(placement.parentId===id||(placement.parentId&&moving.includes(placement.parentId)))return {ok:false,problem:'self'};
  if(placement.parentId&&descendantsOf(folders,id).some(d=>d.id===placement.parentId))return {ok:false,problem:'descendant'};
 }
 const changesParent=moving.some(id=>(by.get(id)!.parentId??null)!==placement.parentId);
 if(changesParent){
  if(!policy.allowNesting)return {ok:false,problem:'nesting-off'};
  if(parent&&parent.myRights?.mayCreateChild===false)return {ok:false,problem:'no-children'};
  const base=parent?depthOf(by,parent.id)+1:0;
  if(moving.some(id=>base+heightOf(folders,id)+1>policy.maxDepth))return {ok:false,problem:'depth'};
 }
 return {ok:true,placement};
}
/** The index of the ancestor of row `index` that sits at `depth` (the row itself when depths match). */
function ancestorAt(rows:readonly DndRow[],index:number,depth:number):number {
 let at=index;
 while(at>0&&rows[at]!.depth>depth)at--;
 return at;
}
/** Turn a pointer drop (target row, zone, requested depth) into a placement, before validation. */
export function placementFor(rows:readonly DndRow[],folders:readonly DndFolder[],intent:DropIntent):Placement|null {
 const index=rows.findIndex(r=>r.id===intent.targetId);
 if(index<0)return null;
 const row=rows[index]!,by=new Map(folders.map(f=>[f.id,f])),parentOf=(id:string)=>by.get(id)?.parentId??null;
 if(intent.zone==='inside')return {parentId:row.id,beforeId:null};
 if(intent.zone==='before')return {parentId:parentOf(row.id),beforeId:row.id};
 const next=rows[index+1];
 // After an expanded folder: the first child.
 if(row.hasChildren&&row.expanded&&next&&next.depth>row.depth)return {parentId:row.id,beforeId:next.id};
 const min=next?Math.min(next.depth,row.depth):0,depth=Math.max(min,Math.min(row.depth,intent.depth??row.depth));
 const anchor=rows[ancestorAt(rows,index,depth)]!;
 // The row after the anchor's subtree is its next sibling only when it sits at the same depth.
 return {parentId:parentOf(anchor.id),beforeId:next&&next.depth===depth?next.id:null};
}
export function resolveDrop(rows:readonly DndRow[],folders:readonly DndFolder[],moving:readonly string[],intent:DropIntent,policy:DndPolicy):DropResult {
 if(moving.includes(intent.targetId))return {ok:false,problem:'self'};
 const target=folders.find(f=>f.id===intent.targetId);
 if(!target)return {ok:false,problem:'unknown'};
 if(target.role&&intent.zone!=='inside')return {ok:false,problem:'system'};
 const placement=placementFor(rows,folders,intent);
 if(!placement)return {ok:false,problem:'unknown'};
 // Dropping a folder right where it already is changes nothing.
 if(moving.length===1&&(folders.find(f=>f.id===moving[0])?.parentId??null)===placement.parentId){
  const siblings=folders.filter(f=>(f.parentId??null)===placement.parentId&&!f.role).map(f=>f.id),at=siblings.indexOf(moving[0]!);
  if(at>=0&&(siblings[at+1]??null)===placement.beforeId&&placement.beforeId!==null)return {ok:false,problem:'noop'};
  if(at>=0&&at===siblings.length-1&&placement.beforeId===null)return {ok:false,problem:'noop'};
 }
 return validatePlacement(folders,moving,placement,policy);
}
/** Keyboard moves: Alt+Up/Down reorder among siblings, Alt+Left outdents, Alt+Right indents under the previous sibling. */
export type KeyMove='up'|'down'|'outdent'|'indent';
export function keyMoveFor(event:{key:string;altKey:boolean},rtl=false):KeyMove|null {
 if(!event.altKey)return null;
 if(event.key==='ArrowUp')return 'up';
 if(event.key==='ArrowDown')return 'down';
 if(event.key===(rtl?'ArrowRight':'ArrowLeft'))return 'outdent';
 if(event.key===(rtl?'ArrowLeft':'ArrowRight'))return 'indent';
 return null;
}
/** `folders` are in display order. */
export function placementForKey(folders:readonly DndFolder[],id:string,move:KeyMove,policy:DndPolicy):DropResult {
 const by=new Map(folders.map(f=>[f.id,f])),me=by.get(id);
 if(!me)return {ok:false,problem:'unknown'};
 const parent=me.parentId??null,siblings=folders.filter(f=>(f.parentId??null)===parent&&!f.role),at=siblings.findIndex(f=>f.id===id);
 let placement:Placement|null=null;
 if(move==='up'&&at>0)placement={parentId:parent,beforeId:siblings[at-1]!.id};
 else if(move==='down'&&at>=0&&at<siblings.length-1)placement={parentId:parent,beforeId:siblings[at+2]?.id??null};
 else if(move==='indent'&&at>0)placement={parentId:siblings[at-1]!.id,beforeId:null};
 else if(move==='outdent'&&parent){
  const up=by.get(parent)!,grand=up.parentId??null,uncles=folders.filter(f=>(f.parentId??null)===grand&&!f.role),u=uncles.findIndex(f=>f.id===parent);
  placement={parentId:grand,beforeId:uncles[u+1]?.id??null};
 }
 if(!placement)return {ok:false,problem:'noop'};
 return validatePlacement(folders,[id],placement,policy);
}
export interface MoveOutcome {order:string[];parents:Record<string,string>;/** Folders whose effective parent changes. */reparented:string[]}
/**
 * The new arrangement after placing `moving`. `ordered` lists every folder id in display order,
 * `server` the unmodified folders, `parents` the stored overrides. A folder returning to its server parent loses its override.
 */
export function applyPlacement(ordered:readonly string[],server:readonly TreeFolder[],parents:Readonly<Record<string,string>>|undefined,moving:readonly string[],placement:Placement):MoveOutcome {
 const move=ordered.filter(id=>moving.includes(id)),rest=ordered.filter(id=>!moving.includes(id));
 const at=placement.beforeId&&rest.includes(placement.beforeId)?rest.indexOf(placement.beforeId):rest.length;
 rest.splice(at,0,...move);
 const next={...parents},ids=new Set(server.map(f=>f.id)),reparented:string[]=[];
 for(const id of move){
  const f=server.find(value=>value.id===id),serverParent=f?.parentId&&ids.has(f.parentId)&&f.parentId!==id?f.parentId:null;
  const current=parents?.[id]===undefined?serverParent:parents[id]===TOP_LEVEL?null:parents[id]!;
  if((placement.parentId??null)===serverParent)delete next[id];
  else next[id]=placement.parentId??TOP_LEVEL;
  if((current??null)!==(placement.parentId??null))reparented.push(id);
 }
 return {order:rest,parents:next,reparented};
}
/** Folders arranged only in this app: their visual parent differs from the server's. */
export function arrangedOnly<T extends TreeFolder>(server:readonly T[],parents:Readonly<Record<string,string>>|undefined):string[] {
 const visual=applyVisualParents(server,parents),by=new Map(server.map(f=>[f.id,f]));
 return visual.filter(f=>(by.get(f.id)!.parentId??null)!==(f.parentId??null)).map(f=>f.id);
}
/** The folders that move together: the selection when the dragged one is part of it, else just that one; descendants of a moving folder ride along. */
export function movingSet(ordered:readonly DndFolder[],selected:ReadonlySet<string>,dragged:string):string[] {
 const base=selected.has(dragged)?ordered.filter(f=>selected.has(f.id)).map(f=>f.id):[dragged];
 return base.filter(id=>!base.some(other=>other!==id&&descendantsOf(ordered,other).some(d=>d.id===id)));
}
/** The name of a moving folder that a sibling under the new parent already uses, or null. */
export function nameCollision(folders:readonly DndFolder[],moving:readonly string[],placement:Placement):string|null {
 const taken=new Set(folders.filter(f=>(f.parentId??null)===placement.parentId&&!moving.includes(f.id)).map(f=>f.name.toLocaleLowerCase()));
 for(const id of moving){const f=folders.find(v=>v.id===id);if(f&&taken.has(f.name.toLocaleLowerCase()))return f.name;}
 return null;
}
