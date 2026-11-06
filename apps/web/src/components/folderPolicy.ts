import {settings} from '../api/settings.ts';
import type {AccountUiPreferences} from '../api/accountPreferences.ts';
import {DEFAULT_POLICY,type DndPolicy} from './folderDnd.ts';
import type {TreeFolder} from './folderTree.ts';
/** The administrator's folder-layout settings, read from the settings map (model C). Reactive. */
export const FOLDER_KEYS={
 enabled:'folders.reorder.enabled',nesting:'folders.reorder.allowNesting',server:'folders.reorder.allowServerMove',system:'folders.reorder.allowSystemFolders',
 depth:'folders.reorder.maxDepth',locked:'folders.order.locked',pinned:'folders.pinned.admin',order:'folders.order.default',placement:'folders.newFolderPlacement',
} as const;
const bool=(key:string,fallback:boolean):boolean=>{const v=settings()[key]?.value;return typeof v==='boolean'?v:fallback;};
const list=(key:string):string[]=>{const v=settings()[key]?.value;return Array.isArray(v)?v.filter((x):x is string=>typeof x==='string'&&x.length>0):[];};
export interface FolderLayoutPolicy {
 /** The admin switches that apply to drag and drop (before the user's own switch). */
 dnd:Omit<DndPolicy,'forcePinned'>;
 allowServerMove:boolean;
 /** Entries of `folders.pinned.admin` and `folders.order.default`. */
 pinnedEntries:string[];orderEntries:string[];
 placement:'end'|'alphabetical';
 /** True when the order is fixed by the administrator (shows the lock hint). */
 locked:boolean;
}
export function folderLayoutPolicy():FolderLayoutPolicy {
 const depth=settings()[FOLDER_KEYS.depth]?.value,locked=bool(FOLDER_KEYS.locked,false);
 const placement=settings()[FOLDER_KEYS.placement]?.value;
 return {
  dnd:{enabled:bool(FOLDER_KEYS.enabled,true),allowNesting:bool(FOLDER_KEYS.nesting,true),allowSystemFolders:bool(FOLDER_KEYS.system,false),maxDepth:typeof depth==='number'&&Number.isInteger(depth)?Math.max(1,Math.min(16,depth)):8,locked},
  allowServerMove:bool(FOLDER_KEYS.server,true),pinnedEntries:list(FOLDER_KEYS.pinned),orderEntries:list(FOLDER_KEYS.order),
  placement:placement==='alphabetical'?'alphabetical':'end',locked,
 };
}
type Matchable=TreeFolder&{mailwomanPath?:string};
/** Folders an entry names: `role:<role>`, or a folder path / name (case-insensitive). */
export function entryMatches(entry:string,folder:Matchable):boolean {
 if(entry.startsWith('role:'))return folder.role===entry.slice(5);
 const path=(folder.mailwomanPath??folder.name).toLocaleLowerCase();
 return path===entry.toLocaleLowerCase()||(!entry.includes('/')&&folder.name.toLocaleLowerCase()===entry.toLocaleLowerCase());
}
/** The ids the administrator force-pinned, in the order of the entries; entries that match no folder are skipped. */
export function forcePinnedIds(folders:readonly Matchable[],entries:readonly string[]):string[] {
 const out:string[]=[];
 for(const entry of entries)for(const folder of folders)if(entryMatches(entry,folder)&&!out.includes(folder.id)&&folder.mailwomanSearchQuery===undefined)out.push(folder.id);
 return out;
}
/**
 * The folders merged with the stored arrangement, as ids in display order: the user's own order first, then
 * (for folders the user has not placed) the administrator's template, then the rest per `placement`.
 * Unknown ids in the stored order are ignored; folders missing from it are slotted in, never lost.
 */
export function mergeOrder(folders:readonly Matchable[],stored:readonly string[]|undefined,template:readonly string[],placement:'end'|'alphabetical'):string[] {
 const ids=new Set(folders.map(f=>f.id)),known=(stored??[]).filter((id,i,all)=>ids.has(id)&&all.indexOf(id)===i);
 if(known.length)return slotNew(folders,known,placement);
 const templated=forcePinnedIds(folders,template);
 return slotNew(folders,templated,placement);
}
function slotNew(folders:readonly Matchable[],known:readonly string[],placement:'end'|'alphabetical'):string[] {
 const placed=new Set(known),fresh=folders.filter(f=>!placed.has(f.id));
 if(placement==='end'||!known.length)return [...known,...fresh.map(f=>f.id)];
 const collator=new Intl.Collator(undefined,{numeric:true,sensitivity:'base'}),out=[...known];
 // Alphabetical: a new folder goes before the first placed sibling that sorts after it, else after the last sibling.
 for(const folder of [...fresh].sort((a,b)=>collator.compare(a.name,b.name))){
  const siblings=out.map(id=>folders.find(f=>f.id===id)!).filter(f=>(f.parentId??null)===(folder.parentId??null));
  const after=siblings.find(s=>collator.compare(s.name,folder.name)>0);
  if(after)out.splice(out.indexOf(after.id),0,folder.id);else{const last=siblings.at(-1);out.splice(last?out.indexOf(last.id)+1:out.length,0,folder.id);}
 }
 return out;
}
/** The effective starting preferences for the arrangement: stored order, or the admin template, merged. */
export function effectiveOrder(folders:readonly Matchable[],prefs:AccountUiPreferences,policy:FolderLayoutPolicy):string[] {
 return mergeOrder(folders,prefs.folderOrder,policy.orderEntries,policy.placement);
}
export {DEFAULT_POLICY};
