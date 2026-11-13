import {apiFetch} from '../../api/http.ts';
import {withBase} from '../../api/basePath.ts';
import {parseEntry,type SettingEntry} from '../../api/settings.ts';
import type {AccountUiPreferences} from '../../api/accountPreferences.ts';
import type {WorkspaceMailbox} from '../../api/workspaceMail.ts';
import {parseAppearancePrefs} from '../../theme/appearance.ts';
import {buildBackup,TRANSFER_GROUPS,type SettingsBackup} from './settingsTransferModel.ts';
/** Read a verified source without changing the active mailbox or its settings cache. */
export async function readSettingsSource(target:WorkspaceMailbox,signal:AbortSignal):Promise<SettingsBackup>{
 const bodies=await Promise.all(['/api/settings','/api/account/preferences','/api/account/appearance'].map(async path=>{
  const response=await apiFetch(withBase(path),{mailboxTarget:target,signal});if(!response.ok)throw new Error('Source unavailable');return await response.json() as Record<string,unknown>;
 }));
 const [settings,preferences,appearance]=bodies as [Record<string,unknown>,Record<string,unknown>,Record<string,unknown>];
 if(!settings['settings']||typeof settings['settings']!=='object'||!preferences['preferences']||typeof preferences['preferences']!=='object')throw new Error('Invalid source');
 const map:Record<string,SettingEntry>={};for(const [key,raw] of Object.entries(settings['settings'])){const entry=parseEntry(raw);if(entry)map[key]={...entry,value:entry.value,userValue:entry.value,source:'user',inherited:false};}
 const backup=buildBackup(map,preferences['preferences'] as AccountUiPreferences,parseAppearancePrefs(appearance['appearance']),{accountId:target.accountId,storageAuthority:target.storageAuthority,origin:location.origin},TRANSFER_GROUPS,false);
 // No stored appearance is a source default, not an explicit theme to impose elsewhere.
 if(!appearance['appearance'])backup.items=backup.items.filter(item=>item.kind!=='appearance');
 return backup;
}
