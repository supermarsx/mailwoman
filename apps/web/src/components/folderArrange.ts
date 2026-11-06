import {createSignal} from 'solid-js';
import type {AccountUiPreferences} from '../api/accountPreferences.ts';
import {orderedFolders} from '../screens/Settings/folderPreferenceModel.ts';
import {applyPlacement,applyVisualParents,nameCollision,validatePlacement,type DndFolder,type DndPolicy,type DropProblem,type Placement} from './folderDnd.ts';
import type {UndoableAction} from '../state/undoHistory.ts';
/**
 * The one controller behind every folder move (sidebar drag, Settings list drag, keyboard, the "Move..." dialog).
 *
 * Reordering and visual nesting are saved in the user's UI preferences (the VISUAL layer, never touching the server).
 * A nest or un-nest may also change the REAL hierarchy: JMAP `Mailbox/set parentId` / IMAP RENAME. That needs the admin
 * to allow server moves, the account to allow them, and the user's `folderNestMode` (`ask` shows a choice dialog first).
 * The preferences are changed first (optimistic); a refused or failed server move puts them back and tells the user.
 */
export interface ArrangeDeps {
 /** The server's folders, unmodified. */
 folders:()=>readonly DndFolder[];
 prefs:()=>AccountUiPreferences;
 update:(patch:AccountUiPreferences)=>void;
 policy:()=>DndPolicy;
 /** Whether this account and the administrator allow a hierarchy change on the server. */
 serverMoveAllowed:()=>boolean;
 /** Move one folder on the server (parent null = top level). True when it worked. */
 moveOnServer:(id:string,parentId:string|null)=>Promise<boolean>;
 /** Called once per finished gesture: an undo entry and the "Folder moved" toast. */
 record:(action:UndoableAction&{moved:number})=>void;
 failed:(problem:'server'|'collision'|DropProblem,name?:string)=>void;
 label:(count:number,name:string)=>string;
 accountId?:()=>string|null;
}
export interface PendingNest {moving:string[];placement:Placement;outcome:ReturnType<typeof applyPlacement>}
type Snapshot=Pick<AccountUiPreferences,'folderOrder'|'folderParent'>;
export function createFolderArranger(deps:ArrangeDeps){
 const [pending,setPending]=createSignal<PendingNest|null>(null);
 const visual=():DndFolder[]=>applyVisualParents(orderedFolders([...deps.folders()],deps.prefs()),deps.prefs().folderParent);
 const orderIds=()=>visual().map(f=>f.id);
 const snapshot=():Snapshot=>({folderOrder:deps.prefs().folderOrder??[],folderParent:deps.prefs().folderParent??{}});
 const save=(value:Snapshot):void=>deps.update({folderOrder:value.folderOrder??[],folderParent:value.folderParent??{}});
 const nameOf=(ids:readonly string[]):string=>deps.folders().find(f=>f.id===ids[0])?.name??'';
 /** Move the folders; returns what happened so callers can announce it. */
 function place(moving:readonly string[],placement:Placement):'applied'|'asked'|'invalid' {
  const checked=validatePlacement(visual(),moving,placement,deps.policy());
  if(!checked.ok){deps.failed(checked.problem);return 'invalid';}
  const outcome=applyPlacement(orderIds(),deps.folders(),deps.prefs().folderParent,moving,placement);
  if(!outcome.reparented.length){commit([...moving],outcome,'local');return 'applied';}
  const mode=deps.prefs().folderNestMode??'ask',canServer=deps.serverMoveAllowed();
  if(!canServer||mode==='local'){commit([...moving],outcome,'local');return 'applied';}
  const clash=nameCollision(visual(),moving,placement);
  if(mode==='server'){
   if(clash){deps.failed('collision',clash);return 'invalid';}
   void commit([...moving],outcome,'server');return 'applied';
  }
  setPending({moving:[...moving],placement,outcome});return 'asked';
 }
 async function commit(moving:string[],outcome:PendingNest['outcome'],layer:'local'|'server'):Promise<void> {
  const before=snapshot(),after:Snapshot={folderOrder:outcome.order,folderParent:outcome.parents};
  const serverParents=new Map(outcome.reparented.map(id=>[id,outcome.parents[id]===undefined?(deps.folders().find(f=>f.id===id)?.parentId??null):outcome.parents[id]==='_top'?null:outcome.parents[id]!]));
  const previousServer=new Map(outcome.reparented.map(id=>[id,deps.folders().find(f=>f.id===id)?.parentId??null]));
  save(after);
  if(layer==='server'){
   const moved:string[]=[];
   for(const id of outcome.reparented){
    // The visual override is dropped once the server holds the new parent.
    if(await deps.moveOnServer(id,serverParents.get(id)??null))moved.push(id);
    else{for(const done of moved)await deps.moveOnServer(done,previousServer.get(done)??null);save(before);deps.failed('server',nameOf([id]));return;}
   }
   const cleared={...outcome.parents};for(const id of outcome.reparented)delete cleared[id];
   save({folderOrder:outcome.order,folderParent:cleared});
  }
  const name=nameOf(moving);
  deps.record({moved:moving.length,label:deps.label(moving.length,name),accountId:deps.accountId?.()??null,
   undo:async()=>{
    if(layer==='server')for(const id of outcome.reparented)if(!await deps.moveOnServer(id,previousServer.get(id)??null))throw new Error('server');
    save(before);
   },
   redo:async()=>{
    if(layer==='server')for(const id of outcome.reparented)if(!await deps.moveOnServer(id,serverParents.get(id)??null))throw new Error('server');
    save(layer==='server'?{folderOrder:outcome.order,folderParent:Object.fromEntries(Object.entries(outcome.parents).filter(([id])=>!outcome.reparented.includes(id)))}:after);
   }});
 }
 /** The answer of the nest dialog. */
 function choose(mode:'server'|'local',remember:boolean):void {
  const value=pending();if(!value)return;
  setPending(null);
  if(remember)deps.update({folderNestMode:mode});
  if(mode==='server'){
   const clash=nameCollision(visual(),value.moving,value.placement);
   if(clash){deps.failed('collision',clash);return;}
  }
  void commit(value.moving,value.outcome,mode);
 }
 const cancel=():void=>{setPending(null);};
 /** Back to the server's own order: no manual order, no visual parents. */
 function reset():void {save({folderOrder:[],folderParent:{}});}
 return {visual,pending,place,choose,cancel,reset};
}
export type FolderArranger=ReturnType<typeof createFolderArranger>;
