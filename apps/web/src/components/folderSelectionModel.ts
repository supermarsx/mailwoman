/**
 * Folder selection geometry, shared by the sidebar and the Settings folder list. Pure functions over the
 * visible rows in tree order (collapsed folders hide their subfolders, so those are never in `visible`).
 *
 * One model with the marquee: the selection is a set of ids plus an anchor, the row a range starts from.
 *  - plain click: selects nothing here (the caller opens the folder) and moves the anchor;
 *  - Ctrl/Cmd+click: toggles the row and moves the anchor to it;
 *  - Shift+click: the range from the anchor to the row replaces the part of the selection the previous range
 *    added, keeping what was picked before it (like a file manager);
 *  - Shift+Arrow: extends or shrinks the range one visible row at a time from the anchor;
 *  - Ctrl+A: every visible selectable row.
 */
export interface SelectionState {selected:ReadonlySet<string>;anchor:string|null;/** Ids the last Shift range added, so the next range replaces them. */rangeAdded:readonly string[]}
export const EMPTY_SELECTION:SelectionState={selected:new Set<string>(),anchor:null,rangeAdded:[]};

/** The ids from `from` to `to` inclusive in visible order, `selectable` rows only; empty when either is not visible. */
export function rangeIds(visible:readonly string[],from:string,to:string,selectable:(id:string)=>boolean=()=>true):string[] {
 const a=visible.indexOf(from),b=visible.indexOf(to);
 if(a<0||b<0)return [];
 const [lo,hi]=a<=b?[a,b]:[b,a];
 return visible.slice(lo,hi+1).filter(selectable);
}

export interface ClickModifiers {shift:boolean;toggle:boolean}
export function applyClick(state:SelectionState,visible:readonly string[],id:string,mod:ClickModifiers,selectable:(id:string)=>boolean=()=>true):SelectionState {
 if(!visible.includes(id))return state;
 if(mod.shift&&state.anchor&&visible.includes(state.anchor)){
  const base=new Set([...state.selected].filter(x=>!state.rangeAdded.includes(x)));
  const range=rangeIds(visible,state.anchor,id,selectable);
  const added=range.filter(x=>!base.has(x));
  return {selected:new Set([...base,...range]),anchor:state.anchor,rangeAdded:added};
 }
 if(mod.toggle||mod.shift){
  if(!selectable(id))return {...state,anchor:id,rangeAdded:[]};
  const next=new Set(state.selected);
  if(next.has(id))next.delete(id);else next.add(id);
  return {selected:next,anchor:id,rangeAdded:[]};
 }
 return {...state,anchor:id,rangeAdded:[]};
}

/** Shift+Arrow: move the range end one visible row from `focus`, the anchor staying put. Returns the new state and the row to focus. */
export function extendByKey(state:SelectionState,visible:readonly string[],focus:string,dir:1|-1,selectable:(id:string)=>boolean=()=>true):{state:SelectionState;focus:string}|null {
 const at=visible.indexOf(focus);
 if(at<0)return null;
 const target=visible[at+dir];
 if(target===undefined)return null;
 const anchored:SelectionState=state.anchor&&visible.includes(state.anchor)?state:{...state,anchor:focus,selected:new Set([...state.selected,...(selectable(focus)?[focus]:[])]),rangeAdded:selectable(focus)&&!state.selected.has(focus)?[focus]:[]};
 return {state:applyClick(anchored,visible,target,{shift:true,toggle:false},selectable),focus:target};
}

export function selectAll(visible:readonly string[],selectable:(id:string)=>boolean=()=>true):SelectionState {
 const all=visible.filter(selectable);
 return {selected:new Set(all),anchor:all[0]??null,rangeAdded:[]};
}

/** The selection with ids that are no longer visible dropped (a folder collapsed away, hidden or deleted). */
export function pruneSelection(state:SelectionState,visible:ReadonlySet<string>):SelectionState {
 if([...state.selected].every(id=>visible.has(id))&&(state.anchor===null||visible.has(state.anchor)))return state;
 return {selected:new Set([...state.selected].filter(id=>visible.has(id))),anchor:state.anchor&&visible.has(state.anchor)?state.anchor:null,rangeAdded:state.rangeAdded.filter(id=>visible.has(id))};
}
