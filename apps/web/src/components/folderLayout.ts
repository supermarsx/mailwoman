import {createMemo} from 'solid-js';
import {canMoveFolder,moveFolderTo} from '../api/folderStructure.ts';
import {preferenceAccount,preferenceAccountIsCurrent} from '../api/accountPreferenceLifecycle.ts';
import type {Mailbox} from '../api/jmap-types.ts';
import type {RenameAbility} from '../api/renameFolder.ts';
import type {AccountUiPreferences} from '../api/accountPreferences.ts';
import {recordUndoable,undoOnly} from '../state/undoHistory.ts';
import {isolate,loadCatalog,t} from '../i18n';
import {toast} from './toast/toast.ts';
import {createFolderArranger} from './folderArrange.ts';
import {type DndFolder,type DndPolicy,type DropProblem} from './folderDnd.ts';
import {effectiveOrder,folderLayoutPolicy,forcePinnedIds} from './folderPolicy.ts';
import {createFolderDnd,type FolderDndOptions} from './useFolderDnd.ts';
/**
 * Everything the sidebar and the Settings folder list share about arranging folders: the effective policy (admin
 * keys + the user's own switch), the arranger (preferences, server move, undo, toast) and the drag-and-drop binding.
 * Both surfaces call this, so their behaviour cannot diverge.
 */
export interface FolderLayoutOptions {
 folders:()=>readonly DndFolder[];
 prefs:()=>AccountUiPreferences;
 update:(patch:AccountUiPreferences)=>void;
 /** Move one folder on the server; absent when the account cannot (then every nest is an arrangement only). */
 moveOnServer?:(id:string,parentId:string|null)=>Promise<boolean>;
 /** Whether the account may change the folder hierarchy right now. */
 serverAvailable?:()=>boolean;
 accountId?:()=>string|null;
}
export function createFolderLayout(o:FolderLayoutOptions){
 const layout=()=>folderLayoutPolicy();
 const forced=createMemo(()=>new Set(forcePinnedIds(o.folders(),layout().pinnedEntries)));
 /** The stored preferences with the admin's default order and force-pins applied. */
 const effective=():AccountUiPreferences=>{
  const prefs=o.prefs(),pinned=[...forced(),...(prefs.folderPinned??[]).filter(id=>!forced().has(id))];
  return {...prefs,folderOrder:effectiveOrder(o.folders(),prefs,layout()),folderPinned:pinned};
 };
 const policy=():DndPolicy=>({...layout().dnd,enabled:layout().dnd.enabled,forcePinned:forced(),pinned:new Set(effective().folderPinned??[])});
 const problemText=(problem:'server'|'collision'|DropProblem,name?:string):string=>t(`folder-dnd-problem-${problem}`,{name:isolate(name??'')});
 const arranger=createFolderArranger({
  folders:o.folders,prefs:effective,update:o.update,policy,
  serverMoveAllowed:()=>!!o.moveOnServer&&(o.serverAvailable?.()??true)&&layout().allowServerMove&&layout().dnd.allowNesting,
  moveOnServer:(id,parent)=>o.moveOnServer?.(id,parent)??Promise.resolve(false),
  accountId:()=>o.accountId?.()??null,
  label:(count,name)=>count===1?t('folder-dnd-moved',{name:isolate(name)}):t('folder-dnd-moved-many',{count}),
  failed:(problem,name)=>{toast.error(problemText(problem,name));},
  record:action=>{
   const entry=recordUndoable(action);
   toast.success(action.label,{action:{label:t('folder-dnd-undo'),run:()=>{if(entry)void undoOnly(entry.id);else void action.undo();}}});
  },
 });
 /** Settings and sidebar both start the catalog once. */
 void loadCatalog('folder-dnd');
 /** Why a folder cannot be dragged, or null when it can. */
 function lockReason(id:string):'locked'|'disabled'|'forced'|'system'|null {
  const p=policy(),f=o.folders().find(v=>v.id===id);
  if(p.locked)return 'locked';
  if(!p.enabled)return 'disabled';
  if(forced().has(id))return 'forced';
  if(f?.role&&!p.allowSystemFolders)return 'system';
  return null;
 }
 const dnd=(rest:Omit<FolderDndOptions,'folders'|'policy'|'canDrag'|'onMove'|'onRefused'|'describe'>&{canDrag?:(id:string)=>boolean})=>createFolderDnd({
  label:id=>arranger.visual().find(f=>f.id===id)?.name??'',
  ...rest,folders:arranger.visual,policy,canDrag:id=>lockReason(id)===null&&(rest.canDrag?.(id)??true),
  onMove:(moving,placement)=>{arranger.place(moving,placement);},
  onRefused:problem=>{toast.error(problemText(problem));},
  describe:(move,name)=>t(`folder-dnd-announce-${move}`,{name:isolate(name)}),
 });
 return {layout,forced,effective,policy,arranger,dnd,lockReason,problemText};
}
export type FolderLayout=ReturnType<typeof createFolderLayout>;
/** The server half of a folder move (JMAP Mailbox/set parentId; IMAP-backed accounts RENAME on the engine side). */
export function serverFolderMover(app:{mailboxes:()=>Mailbox[];refreshFolders:()=>Promise<unknown>},ability:()=>RenameAbility|null,client:Parameters<typeof moveFolderTo>[0]){
 return async(id:string,parent:string|null):Promise<boolean>=>{
  const owner=preferenceAccount();
  if(!owner||!preferenceAccountIsCurrent(owner))return false;
  const box=app.mailboxes().find(value=>value.id===id);
  if(!box||!canMoveFolder(box,app.mailboxes(),ability(),parent))return false;
  const result=await moveFolderTo(client,owner.id,id,parent,{current:()=>preferenceAccountIsCurrent(owner)});
  await app.refreshFolders().catch(()=>undefined);
  return result.ok;
 };
}
