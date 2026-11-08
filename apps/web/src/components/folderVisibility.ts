import {settings,type SettingEntry} from '../api/settings.ts';
import type {AccountUiPreferences} from '../api/accountPreferences.ts';
import type {PreferenceFolder} from '../screens/Settings/folderPreferenceModel.ts';
import {effectiveOverride,OVERRIDE_DEFS} from './properties/folderOverrides.ts';
import {allowedOverrides,type OverrideKey} from './properties/policy.ts';

type Folder=PreferenceFolder&{totalEmails?:number;unreadEmails?:number};
type Read=(key:string)=>SettingEntry|undefined;

/** Apply only to navigation: management and move targets retain every folder.
 * Unknown counts and virtual searches stay reachable. A visible descendant
 * keeps its ancestors, including when its own override forces it to show. */
export function visibleSidebarFolders<T extends Folder>(folders:readonly T[],prefs:AccountUiPreferences,read:Read=key=>settings()[key],allowed:readonly OverrideKey[]=allowedOverrides()):T[]{
 const def=OVERRIDE_DEFS.find(item=>item.key==='hideWhenEmpty')!;
 const byId=new Map(folders.map(folder=>[folder.id,folder]));
 const shown=new Set<string>();
 for(const folder of folders){
  const hide=effectiveOverride(def,prefs.folderProps?.[folder.id],read,allowed).value===true;
  const empty=folder.totalEmails===0&&!(typeof folder.unreadEmails==='number'&&folder.unreadEmails>0);
  if(hide&&empty&&folder.role!=='inbox'&&folder.mailwomanSearchQuery===undefined)continue;
  let current:T|undefined=folder;
  while(current&&!shown.has(current.id)){
   shown.add(current.id);current=current.parentId?byId.get(current.parentId):undefined;
  }
 }
 return folders.filter(folder=>shown.has(folder.id));
}
