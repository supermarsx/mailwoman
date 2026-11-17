import {settings,type SettingsMap} from './settings.ts';
export const ALERT_CONTROLS = [
 ['notifications.incoming.toast',true,'New-mail toast','Show an in-app alert when matching mail arrives.'],
 ['notifications.incoming.native',true,'Native new-mail notification','Use browser or operating-system notifications. Browser permission is required.'],
 ['notifications.incoming.tab',false,'Flash the browser tab','Alternate the tab title until focused or the alert timeout expires. Reduced motion uses a steady indicator.'],
 ['notifications.incoming.highlight',true,'Highlight arriving messages','Briefly highlight newly received rows without changing selection.'],
 ['notifications.incoming.foreground',false,'Native notifications while focused','Also show native alerts while this window is being used.'],
 ['notifications.incoming.onlyMatchingRules',false,'Only notify matching rules','Notify only mail matching a Notify rule. Matching mute rules, quiet hours and muted folders still take priority.'],
 ['notifications.incoming.includeJunk',false,'Allow spam-folder alerts','Spam is muted unless this is enabled. Folder scope and mute rules still apply.'],
 ['notifications.incoming.durationSeconds',15,'New-mail attention duration','Seconds before tab attention and row highlights stop (2–120).'],
 ['notifications.incoming.scope','received','New-mail folder scope','Choose Inbox only, all received folders, or selected folders. Drafts, Sent and Trash never trigger new-mail alerts.'],
 ['notifications.events.queued','toast','Message queued or scheduled','Choose alerts after a message is successfully queued or scheduled.'],
 ['notifications.events.sent','toast','Message accepted by the mail server','Choose alerts after successful submission. This does not prove recipient delivery.'],
 ['notifications.events.failed','toast','Sending failed or uncertain','Choose alerts for send failures and uncertain outcomes; the Outbox retains the status.'],
 ['notifications.events.security','toast','New security notices','Choose alerts for new security-centre records. Records remain available when alerts are muted.'],
 ['notifications.events.update','toast','New application version','Choose alerts when a new application version becomes available.'],
 ['notifications.toasts.enabled',true,'Show toasts','Enable transient in-app alerts. Persistent progress and undo controls stay available.'],
 ['notifications.toasts.success',true,'Success toasts','Show success confirmations.'],
 ['notifications.toasts.info',true,'Information toasts','Show informational alerts.'],
 ['notifications.toasts.warning',true,'Warning toasts','Show transient warnings.'],
 ['notifications.toasts.error',true,'Error toasts','Show transient error alerts. Errors still remain in their owning views.'],
 ['notifications.toasts.durationSeconds',5,'Toast duration','Default duration in seconds for ordinary alerts. Timed actions keep their own deadlines.'],
 ['notifications.toasts.limit',3,'Visible toast limit','Maximum number of ordinary toasts visible together (1–8).'],
 ['notifications.toasts.swipe',true,'Swipe to dismiss','Allow dismissing toasts with a horizontal touch or pen gesture.'],
 ['offline.requestPersistence',true,'Offer persistent browser storage','When encrypted offline mail is enabled, offer a browser persistence request to protect the cache from automatic eviction.'],
] as const;
export const INCOMING_KEYS=[...ALERT_CONTROLS.filter(row=>row[0].startsWith('notifications.incoming.')).map(row=>row[0]),'notifications.incoming.folders','notifications.incoming.mutedFolders'] as const;
export type AlertValues=Readonly<Record<string,unknown>>;
export const alertSlug=(key:string):string=>key.replace(/([a-z0-9])([A-Z])/g,'$1-$2').replaceAll('.','-').toLowerCase();
export function alertValue<T>(key:string,fallback:T,values?:AlertValues):T{const value=values?values[key]:settings()[key]?.value;return typeof value===typeof fallback?value as T:fallback;}
export function incomingValues(map:SettingsMap):AlertValues{return Object.fromEntries(INCOMING_KEYS.map(key=>[key,map[key]?.value]));}
export function folderAlertAllowed(values:AlertValues|undefined,boxes:readonly {id:string;role?:string|null}[],ids:Readonly<Record<string,boolean>>|null|undefined):boolean{
 const matching=boxes.filter(box=>ids?.[box.id]);if(!matching.length)return false;
 const muted=Array.isArray(values?.['notifications.incoming.mutedFolders'])?values['notifications.incoming.mutedFolders'] as unknown[]:[];
 if(matching.some(box=>muted.includes(box.id)))return false;
 const received=matching.filter(box=>!['drafts','sent','trash'].includes(box.role??'')&&(box.role!=='junk'||alertValue('notifications.incoming.includeJunk',false,values)));
 const scope=alertValue<string>('notifications.incoming.scope','received',values);
 const selected=Array.isArray(values?.['notifications.incoming.folders'])?values['notifications.incoming.folders'] as unknown[]:[];
 return received.some(box=>scope==='inbox'?box.role==='inbox':scope==='selected'?selected.includes(box.id):true);
}
