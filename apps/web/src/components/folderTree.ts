import type {Mailbox} from '../api/jmap-types.ts';
/** The folder tree: nesting, per-parent ordering, roll-ups and the moves a folder may make. Pure, so the sidebar, the menus and the tests share one answer. */
export type FolderSortBy='name'|'unread'|'total'|'date'|'custom';
export type FolderSortDir='asc'|'desc';
export interface FolderSort {by:FolderSortBy;dir:FolderSortDir}
export const FOLDER_SORT_BYS:readonly FolderSortBy[]=['name','unread','total','date','custom'];
/** The preference key of the top level (a folder id can never be this). */
export const TOP_LEVEL='_top';
export const DEFAULT_SORT:FolderSort={by:'custom',dir:'asc'};
export type TreeFolder=Pick<Mailbox,'id'|'name'|'parentId'|'role'|'totalEmails'|'unreadEmails'>&{mailwomanSearchQuery?:string};
export interface TreeRow<T extends TreeFolder=TreeFolder> {box:T;depth:number;hasChildren:boolean;expanded:boolean}
export function sortKey(parentId:string|null|undefined):string {return parentId?parentId:TOP_LEVEL;}
/** A stored choice, validated; anything else is the default (custom order). */
export function sortFor(sorts:Record<string,FolderSort>|undefined,parentId:string|null|undefined):FolderSort {
 const value=sorts?.[sortKey(parentId)];
 return value&&FOLDER_SORT_BYS.includes(value.by)&&(value.dir==='asc'||value.dir==='desc')?value:DEFAULT_SORT;
}
/** A missing parent makes the folder a root. */
function parentOf<T extends TreeFolder>(box:T,byId:ReadonlyMap<string,T>):string|null {return box.parentId&&byId.has(box.parentId)&&box.parentId!==box.id?box.parentId:null;}
export function childrenOf<T extends TreeFolder>(boxes:readonly T[],parentId:string|null):T[] {
 const byId=new Map(boxes.map(box=>[box.id,box]));
 return boxes.filter(box=>parentOf(box,byId)===parentId);
}
/** Every folder below `id`, nearest first; cycles in bad data end the walk. */
export function descendantsOf<T extends TreeFolder>(boxes:readonly T[],id:string):T[] {
 const out:T[]=[],seen=new Set<string>([id]);let frontier=[id];
 while(frontier.length){const next:string[]=[];for(const box of boxes)if(box.parentId&&frontier.includes(box.parentId)&&!seen.has(box.id)){seen.add(box.id);out.push(box);next.push(box.id);}frontier=next;}
 return out;
}
/** The folders `id` may move under: not itself, not a descendant, not a search folder, and a place the account lets you create in. The top level is always offered by the picker. */
export function moveTargets<T extends TreeFolder&{myRights?:Mailbox['myRights']}>(boxes:readonly T[],id:string):T[] {
 const banned=new Set([id,...descendantsOf(boxes,id).map(box=>box.id)]);
 return boxes.filter(box=>!banned.has(box.id)&&box.mailwomanSearchQuery===undefined&&box.myRights?.mayCreateChild!==false);
}
/** Unread of a folder plus everything below it. */
export function rolledUnread(boxes:readonly TreeFolder[],id:string):number {
 const own=boxes.find(box=>box.id===id)?.unreadEmails??0;
 return own+descendantsOf(boxes,id).reduce((sum,box)=>sum+(box.unreadEmails||0),0);
}
export function rolledTotal(boxes:readonly TreeFolder[],id:string):number {
 const own=boxes.find(box=>box.id===id)?.totalEmails??0;
 return own+descendantsOf(boxes,id).reduce((sum,box)=>sum+(box.totalEmails||0),0);
}
/** Order siblings. `siblings` arrive in the custom order (pinned, then the saved manual order, then the server's); the other choices re-sort a copy, keeping that order for ties. `activity` maps a folder id to its newest message time. */
export function sortSiblings<T extends TreeFolder>(siblings:readonly T[],choice:FolderSort,activity?:ReadonlyMap<string,number>):T[] {
 if(choice.by==='custom')return choice.dir==='desc'?[...siblings].reverse():[...siblings];
 const sign=choice.dir==='desc'?-1:1,collator=new Intl.Collator(undefined,{numeric:true,sensitivity:'base'});
 const value=(box:T):number=>choice.by==='unread'?box.unreadEmails:choice.by==='total'?box.totalEmails:activity?.get(box.id)??0;
 return siblings.map((box,index)=>({box,index})).sort((a,b)=>{
  const order=choice.by==='name'?collator.compare(a.box.name,b.box.name):value(a.box)-value(b.box);
  return (order*sign)||a.index-b.index;
 }).map(row=>row.box);
}
/** The rows of one section of the sidebar, depth-first. `isRoot` picks the roots among the system folders and the custom folders with no visible parent; a collapsed folder lists no children. System folders keep their fixed order: only non-system folders are sorted. */
export function treeRows<T extends TreeFolder>(boxes:readonly T[],isRoot:(box:T)=>boolean,options:{sorts?:Record<string,FolderSort>|undefined;collapsed?:ReadonlySet<string>;activity?:ReadonlyMap<string,number>|undefined}={}):TreeRow<T>[] {
 const byId=new Map(boxes.map(box=>[box.id,box])),rows:TreeRow<T>[]=[],seen=new Set<string>();
 const kids=(parent:string):T[]=>sortSiblings(boxes.filter(box=>box.parentId===parent&&box.id!==parent&&!box.role),sortFor(options.sorts,parent),options.activity);
 const walk=(box:T,depth:number):void=>{
  if(seen.has(box.id))return;seen.add(box.id);
  const children=kids(box.id).filter(child=>!seen.has(child.id)),expanded=!options.collapsed?.has(box.id);
  rows.push({box,depth,hasChildren:children.length>0,expanded:children.length>0&&expanded});
  if(expanded)for(const child of children)walk(child,depth+1);
 };
 const roots=boxes.filter(box=>isRoot(box)&&(box.role||!box.parentId||!byId.has(box.parentId)));
 const system=roots.filter(box=>box.role),custom=sortSiblings(roots.filter(box=>!box.role),sortFor(options.sorts,null),options.activity);
 for(const box of [...system,...custom])walk(box,0);
 return rows;
}
/** The saved manual order after moving `id` one place up or down among its siblings. `order` is the saved id list (may be empty or stale); `siblings` are the folder's siblings in display order. Siblings take, in their new order, the slots they held in the list; every other id keeps its place. Null when there is nowhere to go. */
export function moveWithinSiblings(order:readonly string[],siblings:readonly string[],id:string,direction:'up'|'down'):string[]|null {
 const at=siblings.indexOf(id),to=direction==='up'?at-1:at+1;
 if(at<0||to<0||to>=siblings.length)return null;
 const next=[...siblings];[next[at],next[to]]=[next[to]!,next[at]!];
 const slots=order.map((value,index)=>siblings.includes(value)?index:-1).filter(index=>index>=0);
 if(slots.length===siblings.length){const out=[...order];slots.forEach((slot,index)=>{out[slot]=next[index]!;});return out;}
 return [...order.filter(value=>!siblings.includes(value)),...next];
}
/** Whether a folder is one of the fixed system folders (no rename, delete or move). */
export function isSystemFolder(box:Pick<Mailbox,'role'>):boolean {return box.role!==null&&box.role!==undefined;}
/** `box`'s siblings in the order the sidebar shows them (system folders never move; their siblings are the other system folders). `ordered` is the folder list in custom order. */
export function siblingsOf<T extends TreeFolder>(ordered:readonly T[],box:T,options:{sorts?:Record<string,FolderSort>|undefined;activity?:ReadonlyMap<string,number>|undefined}={}):T[] {
 if(box.role)return ordered.filter(other=>!!other.role);
 const byId=new Map(ordered.map(other=>[other.id,other])),parent=parentOf(box,byId);
 const list=ordered.filter(other=>!other.role&&parentOf(other,byId)===parent);
 return sortSiblings(list,sortFor(options.sorts,parent),options.activity);
}
