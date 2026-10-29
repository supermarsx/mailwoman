import {createEffect,createSignal,untrack,type Accessor} from 'solid-js';

export const WORKSPACE_LAYOUTS=['tabs','side-by-side','stack','masonry-side-by-side','masonry-stack'] as const;
export type WorkspaceLayout=typeof WORKSPACE_LAYOUTS[number];
export type WorkspaceViewSlot=0|1;
export type WorkspaceViewAssignments=[string|null,string|null];
export function workspaceLayout(value:unknown):WorkspaceLayout {
  return typeof value==='string'&&WORKSPACE_LAYOUTS.includes(value as WorkspaceLayout)?value as WorkspaceLayout:'tabs';
}
export function isSplitLayout(value:WorkspaceLayout):boolean {return value==='side-by-side'||value==='stack';}
export function isStackedLayout(value:WorkspaceLayout):boolean {return value==='stack'||value==='masonry-stack';}

/** Exactly two owned views. Hidden tabs remain in their retained owner. */
export function reconcileWorkspaceViews(previous:WorkspaceViewAssignments,ids:readonly string[],active:string|null,focused:WorkspaceViewSlot):WorkspaceViewAssignments {
  const next:WorkspaceViewAssignments=[previous[0]&&ids.includes(previous[0])?previous[0]:null,previous[1]&&ids.includes(previous[1])?previous[1]:null];
  if(next[0]===next[1])next[1]=null;
  for(const slot of [0,1] as const)if(!next[slot])next[slot]=ids.find(id=>!next.includes(id))??null;
  if(active&&ids.includes(active)&&!next.includes(active))next[focused]=active;
  return next;
}
export function createWorkspaceViews(options:{ids:Accessor<readonly string[]>;activeId:Accessor<string|null>;owner:Accessor<string|null>;activate:(id:string)=>void}) {
  const [assignments,setAssignments]=createSignal<WorkspaceViewAssignments>([null,null]);
  let previousOwner:string|null=null;
  let focused:WorkspaceViewSlot=1;
  createEffect(()=>{
    const owner=options.owner(),ids=options.ids(),active=options.activeId();
    if(owner!==previousOwner){previousOwner=owner;focused=1;setAssignments([null,null]);}
    if(!owner){setAssignments([null,null]);return;}
    const previous=untrack(assignments);
    if(active===previous[0])focused=0;else if(active===previous[1])focused=1;
    const next=reconcileWorkspaceViews(previous,ids,active,focused);
    if(next[0]!==previous[0]||next[1]!==previous[1])setAssignments(next);
  });
  function assignView(tabId:string,slot:WorkspaceViewSlot):boolean {
    if(!options.owner()||options.owner()!==previousOwner||!options.ids().includes(tabId))return false;
    const next:WorkspaceViewAssignments=[...assignments()];
    const other:WorkspaceViewSlot=slot===0?1:0;
    if(next[other]===tabId)next[other]=next[slot];
    next[slot]=tabId;focused=slot;setAssignments(next);options.activate(tabId);return true;
  }
  function restore(value:WorkspaceViewAssignments|undefined):void {
    if(!value||!options.owner()||options.owner()!==previousOwner)return;
    const ids=options.ids();
    if(value.some(id=>id!==null&&!ids.includes(id))||(value[0]!==null&&value[0]===value[1]))return;
    setAssignments(reconcileWorkspaceViews(value,ids,options.activeId(),focused));
  }
  /** The next tab to become active goes into the view slot that `openerId` does
   *  NOT occupy, so the list a message was opened from stays on screen (t51). */
  function openBeside(openerId:string):void {
    const at=assignments()[0]===openerId?0:assignments()[1]===openerId?1:null;
    if(at!==null)focused=at===0?1:0;
  }
  return {assignments,assignView,openBeside,restore,slot:(id:string):WorkspaceViewSlot|null=>assignments()[0]===id?0:assignments()[1]===id?1:null};
}
