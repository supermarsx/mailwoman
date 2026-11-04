import type {Accessor} from 'solid-js';
import {accountUiPreferences,setAccountUiPreferences} from '../api/accountPreferences.ts';
import {t} from '../i18n';
import type {ContextAction} from './ContextMenu.tsx';
import {FOLDER_SORT_BYS,sortFor,sortKey,type FolderSortBy,type FolderSortDir} from './folderTree.ts';
/** "Sort subfolders by" for one parent (null: the top level): a by-choice group and a direction group. Each choice is saved per parent in the account preferences. */
export function folderSortAction(parentId:string|null,label:string,onDate:Accessor<void>|(()=>void)):ContextAction {
 const current=()=>sortFor(accountUiPreferences().folderSort,parentId);
 function save(next:{by?:FolderSortBy;dir?:FolderSortDir}):void {
  const now=current(),merged={by:next.by??now.by,dir:next.dir??now.dir};
  setAccountUiPreferences({folderSort:{...accountUiPreferences().folderSort,[sortKey(parentId)]:merged}});
  if(merged.by==='date')onDate();
 }
 const by:ContextAction[]=FOLDER_SORT_BYS.map(value=>({id:`sort:by:${value}`,label:t(`folder-ops-sort-${value}`),icon:'sort',radio:true,checked:current().by===value,current:current().by===value,run:()=>save({by:value})}));
 const dir:ContextAction[]=(['asc','desc'] as const).map((value,index)=>({id:`sort:dir:${value}`,label:t(`folder-ops-sort-${value}`),icon:value==='asc'?'chevronUp':'chevronDown',radio:true,checked:current().dir===value,current:current().dir===value,...(index===0?{separatorBefore:true}:{}),run:()=>save({dir:value})}));
 return {id:'sort',label,icon:'sort',children:[...by,...dir],run:()=>{}};
}
