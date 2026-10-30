import type { IconName } from '../../components/Icon.tsx';
import { FOLDER_ICONS, type AccountUiPreferences } from '../../api/accountPreferences.ts';
export interface PreferenceFolder { parentId?: string | null; mailwomanUpstreamName?: string; mailwomanPath?: string; id: string; name: string; role?: string | null; mailwomanSearchQuery?: string }
/** Hiding a parent hides its descendants from navigation, never from management.
 * Inbox remains reachable even if stale preferences mention it. Cycles terminate. */
export function folderIsHidden(folder: PreferenceFolder, folders: readonly PreferenceFolder[], preferences: AccountUiPreferences): boolean {
  if(folder.role==='inbox')return false;
  const hidden=new Set(preferences.folderHidden??[]),visited=new Set<string>();
  let current:PreferenceFolder|undefined=folder;
  while(current&&!visited.has(current.id)){
    if(current.role!=='inbox'&&hidden.has(current.id))return true;
    visited.add(current.id);current=folders.find(value=>value.id===current?.parentId);
  }
  return false;
}
/** Restoring a descendant also restores hidden ancestors, so it becomes reachable. */
export function shownFolderIds(folder:PreferenceFolder,folders:readonly PreferenceFolder[],hidden:readonly string[]):string[]{
  const ancestors=new Set<string>();let current:PreferenceFolder|undefined=folder;
  while(current&&!ancestors.has(current.id)){ancestors.add(current.id);current=folders.find(value=>value.id===current?.parentId);}
  return hidden.filter(id=>!ancestors.has(id));
}
/** A folder's full path as people read it: the server's decoded path (t92), never the raw modified UTF-7 spelling. */
export function folderPathLabel(folder: PreferenceFolder): string {
  return folder.mailwomanPath ?? folder.name;
}
const ROLE_ICONS: Readonly<Record<string, IconName>> = {inbox:'inbox',sent:'sent',drafts:'edit',trash:'delete',junk:'alert',archive:'archive',outbox:'clock',important:'flag',all:'mail'};
export function folderIcon(folder: PreferenceFolder, preferences: AccountUiPreferences): IconName {
  const custom = preferences.folderIcons?.[folder.id];
  return custom && (FOLDER_ICONS as readonly string[]).includes(custom) ? custom : (folder.mailwomanSearchQuery !== undefined ? 'search' : ROLE_ICONS[folder.role ?? ''] ?? 'folder');
}
/** Preserve server order for new/unlisted folders; never rename or move mail. */
export function orderedFolders<T extends PreferenceFolder>(folders: readonly T[], preferences: AccountUiPreferences): T[] {
  const rank = new Map((preferences.folderOrder ?? []).map((id, i) => [id, i]));
  const pinned = new Set(preferences.folderPinned ?? []);
  return folders.map((folder, index) => ({folder,index})).sort((a,b) =>
    Number(pinned.has(b.folder.id)) - Number(pinned.has(a.folder.id)) ||
    (rank.get(a.folder.id) ?? Number.MAX_SAFE_INTEGER) - (rank.get(b.folder.id) ?? Number.MAX_SAFE_INTEGER) || a.index-b.index
  ).map(value => value.folder);
}
export function moveFolder(ids: readonly string[], id: string, target: string): string[] {
  if (id === target || !ids.includes(id) || !ids.includes(target)) return [...ids];
  const next = ids.filter(value => value !== id);
  next.splice(ids.indexOf(target), 0, id);
  return next;
}
