// Per-folder overrides of the user's own settings, stored with the folder's
// preferences (`folderProps` in the account UI preferences, synced like the rest).
// Each override shows the value that applies and where it comes from:
// the folder's own override, the user's setting, the administrator's default,
// or an administrator lock, which the folder cannot override.
import {settings,type SettingEntry} from '../../api/settings.ts';
import {allowedOverrides,type OverrideKey} from './policy.ts';

export interface FolderProps {
 notes?:string;hideWhenEmpty?:boolean;counters?:'none'|'unread'|'total'|'both';badgeColour?:string;compactBadge?:boolean;
 notifications?:'on'|'off'|'vips';sort?:'date'|'from'|'subject'|'size';threading?:boolean;
 previewLines?:number;markReadDelayMs?:number;replyIdentity?:string;unifiedInbox?:boolean;
}
export type OverrideSource='folder'|'user'|'admin-default'|'admin-locked';
export interface Effective<T=unknown> {value:T|undefined;source:OverrideSource;locked:boolean;overridden:boolean}
export type OverrideKind={kind:'enum';options:readonly string[]}|{kind:'bool'}|{kind:'colour'}|{kind:'lines'}|{kind:'delay'}|{kind:'identity'};
export interface OverrideDef {key:OverrideKey;prop:keyof FolderProps;/** The user setting whose value applies when the folder does not override. */setting:string|null;type:OverrideKind;/** Shipped value while the settings snapshot is not loaded. */defaultValue?:unknown;/** Maps the setting's value to the override's value space. */fromSetting?:(value:unknown)=>unknown}

export const OVERRIDE_DEFS:readonly OverrideDef[]=[
 {key:'hideWhenEmpty',prop:'hideWhenEmpty',setting:'folders.hideEmpty',type:{kind:'bool'},defaultValue:false},
 {key:'counters',prop:'counters',setting:'mail.folderCounts',type:{kind:'enum',options:['none','unread','total','both']},fromSetting:v=>v==='unreadIfAny'?'unread':v},
 {key:'badgeColour',prop:'badgeColour',setting:null,type:{kind:'colour'}},
 {key:'compactBadge',prop:'compactBadge',setting:'mail.compactFolderBadges',type:{kind:'bool'}},
 {key:'notifications',prop:'notifications',setting:null,type:{kind:'enum',options:['on','off','vips']}},
 {key:'sort',prop:'sort',setting:'mail.messageListOrder',type:{kind:'enum',options:['date','from','subject','size']},fromSetting:v=>v==='newest'||v==='oldest'?'date':v==='sender'?'from':v},
 {key:'threading',prop:'threading',setting:'mail.threading',type:{kind:'bool'}},
 {key:'previewLines',prop:'previewLines',setting:null,type:{kind:'lines'}},
 {key:'markReadDelay',prop:'markReadDelayMs',setting:null,type:{kind:'delay'}},
 {key:'replyIdentity',prop:'replyIdentity',setting:null,type:{kind:'identity'}},
 {key:'unifiedInbox',prop:'unifiedInbox',setting:null,type:{kind:'bool'}},
];

const ADMIN_SOURCES=new Set(['global','domain','profile','group','admin_user','envPin']);
type ReadEntry=(key:string)=>SettingEntry|undefined;
const liveEntry:ReadEntry=key=>settings()[key];

/** The value that applies to a folder for one override, and why. A locked setting wins over the folder. */
export function effectiveOverride(def:OverrideDef,props:FolderProps|undefined,read:ReadEntry=liveEntry,allowed:readonly OverrideKey[]=allowedOverrides()):Effective {
 const entry=def.setting?read(def.setting):undefined;
 const base=entry?(def.fromSetting?def.fromSetting(entry.value):entry.value):def.defaultValue;
 if(entry?.locked)return {value:base,source:'admin-locked',locked:true,overridden:false};
 const own=props?.[def.prop];
 if(own!==undefined&&allowed.includes(def.key))return {value:own,source:'folder',locked:false,overridden:true};
 const source:OverrideSource=entry&&ADMIN_SOURCES.has(entry.source)?'admin-default':'user';
 return {value:base,source,locked:false,overridden:false};
}

/** Folder props with one override set (or cleared with `undefined`); an empty entry is removed. */
export function withOverride(all:Readonly<Record<string,FolderProps>>|undefined,folderId:string,patch:{[K in keyof FolderProps]?:FolderProps[K]|undefined}):Record<string,FolderProps> {
 const next={...(all??{})};
 const merged:Record<string,unknown>={...(next[folderId]??{}),...patch};
 for(const key of Object.keys(merged))if(merged[key]===undefined||merged[key]==='')delete merged[key];
 if(Object.keys(merged).length===0)delete next[folderId];else next[folderId]=merged as FolderProps;
 return next;
}

/** The folder's own override for `key` when one applies (allowed, and not beaten by an administrator lock); otherwise undefined, so the caller uses the setting. */
export function appliedOverride(key:OverrideKey,folderId:string|undefined,all:Readonly<Record<string,FolderProps>>|undefined,read:ReadEntry=liveEntry,allowed:readonly OverrideKey[]=allowedOverrides()):unknown {
 if(!folderId)return undefined;
 const def=OVERRIDE_DEFS.find(item=>item.key===key);if(!def)return undefined;
 const eff=effectiveOverride(def,all?.[folderId],read,allowed);
 return eff.source==='folder'?eff.value:undefined;
}
