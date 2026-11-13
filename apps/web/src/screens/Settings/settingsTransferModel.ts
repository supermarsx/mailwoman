import type {SettingsMap} from '../../api/settings.ts';
import type {AccountUiPreferences} from '../../api/accountPreferences.ts';
import {parseAppearancePrefs, type AppearancePrefs} from '../../theme/appearance.ts';

export const TRANSFER_LIMIT=1024*1024;
export const TRANSFER_GROUPS=['appearance','reader','compose','mail','folders','notifications','other'] as const;
export type TransferGroup=typeof TRANSFER_GROUPS[number];
export type TransferKind='setting'|'interface'|'appearance';
export interface TransferItem {kind:TransferKind;key:string;value:unknown}
export interface SettingsBackup {format:'mailwoman-user-settings';version:1;createdAt:string;source:{accountId:string;origin:string;storageAuthority?:string};items:TransferItem[]}
export type ReviewStatus='ready'|'unchanged'|'locked'|'unknown'|'invalid'|'account';
export interface TransferReview extends TransferItem {id:string;group:TransferGroup;current:unknown;status:ReviewStatus}
const portable=['keyboardPreset','direction','markRead','markReadDelayMs','offlineBudgetMb','offlineRetentionDays','eviction','folderSidebarMode','folderSidebarWidth','messageListWidth','messageListHeight','readingPane','messageListCollapsed','readerCollapsed','workspaceSplitRatio','userFoldersCollapsed','folderNestMode','folderGrabHandles'] as const;
const accountSpecific=['workspaceLayout','folderPinned','folderOrder','folderHidden','folderIcons','tags','folderLabels','folderSort','folderCollapsed','folderParent','folderStarred','folderProps'] as const;
const uiKeys=new Set<string>([...portable,...accountSpecific]);
const dedicated=new Set(['appearance.theme','appearance.accent','appearance.badgeColor','appearance.density','appearance.font','mail.readReceipts','notifications.enabled']);
const blocked=/^(?:__proto__|prototype|constructor)$/;
export const equal=(a:unknown,b:unknown):boolean=>JSON.stringify(canonical(a))===JSON.stringify(canonical(b));
function canonical(value:unknown):unknown {return Array.isArray(value)?value.map(canonical):value&&typeof value==='object'?Object.fromEntries(Object.entries(value).sort(([a],[b])=>a.localeCompare(b)).map(([k,v])=>[k,canonical(v)])):value;}
function safeJson(value:unknown,depth=0,budget={left:50000}):boolean {
 if(--budget.left<0||depth>12)return false;
 if(value===null||typeof value==='boolean')return true;
 if(typeof value==='number')return Number.isFinite(value);
 if(typeof value==='string')return value.length<=65536;
 if(Array.isArray(value))return value.length<=5000&&value.every(x=>safeJson(x,depth+1,budget));
 if(typeof value==='object')return Object.keys(value).length<=2000&&Object.entries(value).every(([k,v])=>!blocked.test(k)&&safeJson(v,depth+1,budget));
 return false;
}
export function transferGroup(item:TransferItem):TransferGroup {
 if(item.kind==='appearance'||item.kind==='interface'&&!accountSpecific.includes(item.key as typeof accountSpecific[number]))return 'appearance';
 if(item.kind==='interface')return 'folders';
 const prefix=item.key.split('.')[0];return TRANSFER_GROUPS.some(x=>x===prefix)?prefix as TransferGroup:'other';
}
export function buildBackup(map:SettingsMap,ui:AccountUiPreferences,appearance:AppearancePrefs,source:SettingsBackup['source'],groups:readonly TransferGroup[],includeFolders=false):SettingsBackup {
 const items:TransferItem[]=[];
 for(const [key,entry] of Object.entries(map))if(!entry.inherited&&!dedicated.has(key)&&(entry.userValue!==undefined||entry.source==='user'))items.push({kind:'setting',key,value:entry.userValue!==undefined?entry.userValue:entry.value});
 items.push({kind:'appearance',key:'appearance',value:appearance});
 for(const [key,value] of Object.entries(ui))if(uiKeys.has(key)&&(includeFolders||!accountSpecific.includes(key as typeof accountSpecific[number])))items.push({kind:'interface',key,value});
 return {format:'mailwoman-user-settings',version:1,createdAt:new Date().toISOString(),source,items:items.filter(item=>groups.includes(transferGroup(item))).sort((a,b)=>(a.kind+':'+a.key).localeCompare(b.kind+':'+b.key))};
}
export function parseBackup(text:string):SettingsBackup {
 if(new TextEncoder().encode(text).length>TRANSFER_LIMIT)throw new Error('st-file-large');
 let raw:unknown;try{raw=JSON.parse(text);}catch{throw new Error('st-file-invalid');}
 if(!raw||typeof raw!=='object'||!safeJson(raw))throw new Error('st-file-invalid');
 const value=raw as SettingsBackup;
 if(value.format!=='mailwoman-user-settings'||value.version!==1)throw new Error('st-file-version');
 if(!value.source||typeof value.source.accountId!=='string'||typeof value.source.origin!=='string'||!Array.isArray(value.items)||value.items.length>3000||typeof value.createdAt!=='string')throw new Error('st-file-invalid');
 const ids=new Set<string>();
 for(const item of value.items) {
  if(!item||!['setting','interface','appearance'].includes(item.kind)||typeof item.key!=='string'||!item.key||item.key.length>160||!Object.hasOwn(item,'value'))throw new Error('st-file-invalid');
  const id=`${item.kind}:${item.key}`;if(ids.has(id)||blocked.test(item.key))throw new Error('st-file-invalid');ids.add(id);
 }
 return value;
}
function validUi(key:string,value:unknown):boolean {
 const enums:Record<string,readonly string[]>={keyboardPreset:['default','gmail','outlook','vim','off'],direction:['auto','ltr','rtl'],markRead:['on-open','never'],eviction:['lru','oldest','manual'],folderSidebarMode:['expanded','compact','hidden'],readingPane:['right','bottom','off'],folderNestMode:['ask','server','local'],folderGrabHandles:['hover','always']};
 if(enums[key])return typeof value==='string'&&enums[key]!.includes(value);
 if(['messageListCollapsed','readerCollapsed','userFoldersCollapsed'].includes(key))return typeof value==='boolean';
 if(['folderSidebarWidth','messageListWidth','messageListHeight','offlineBudgetMb','offlineRetentionDays','markReadDelayMs','workspaceSplitRatio'].includes(key))return typeof value==='number'&&Number.isFinite(value)&&value>=0&&value<=1000000;
 if(['folderPinned','folderOrder','folderHidden','folderCollapsed','folderStarred'].includes(key))return Array.isArray(value)&&value.length<=1000&&value.every(x=>typeof x==='string'&&x.length<=512);
 return accountSpecific.includes(key as typeof accountSpecific[number])&&!!value&&typeof value==='object'&&!Array.isArray(value)&&JSON.stringify(value).length<=60000;
}
export function reviewBackup(backup:SettingsBackup,map:SettingsMap,ui:AccountUiPreferences,appearance:AppearancePrefs,source:SettingsBackup['source']):TransferReview[] {
 return backup.items.map(item=>{
  let status:ReviewStatus='ready',current:unknown;
  if(item.kind==='setting') {
   const entry=Object.hasOwn(map,item.key)?map[item.key]:undefined;current=entry?.value;
   if(!entry||dedicated.has(item.key))status='unknown';
   else if(entry.locked)status='locked';
   else if(typeof item.value!==typeof entry.value||Array.isArray(item.value)!==Array.isArray(entry.value))status='invalid';
  }else if(item.kind==='appearance') {
   current=appearance;
   if(item.key!=='appearance'||!item.value||typeof item.value!=='object'||!equal(parseAppearancePrefs(item.value),item.value))status='invalid';
   else {
    const locks:Record<string,readonly (keyof AppearancePrefs)[]>={'appearance.theme':['mode','theme','lightTheme','darkTheme','schedule','chosen'],'appearance.accent':['accentId','customAccent'],'appearance.badgeColor':['badgeColor'],'appearance.density':['density'],'appearance.font':['font']};
    if(Object.entries(locks).some(([key,fields])=>map[key]?.locked&&fields.some(field=>!equal(appearance[field],(item.value as AppearancePrefs)[field]))))status='locked';
   }
  }else {
   current=ui[item.key as keyof AccountUiPreferences];
   if(!uiKeys.has(item.key))status='unknown';
   else if(accountSpecific.includes(item.key as typeof accountSpecific[number])&&(backup.source.accountId!==source.accountId||backup.source.origin!==source.origin||backup.source.storageAuthority!==source.storageAuthority))status='account';
   else if(!validUi(item.key,item.value))status='invalid';
  }
  if(status==='ready'&&equal(current,item.value))status='unchanged';
  return {...item,id:`${item.kind}:${item.key}`,group:transferGroup(item),current,status};
 });
}
export async function applyReviewed(rows:readonly TransferReview[],selected:ReadonlySet<string>,isCurrent:()=>boolean,write:(row:TransferReview)=>Promise<void>,result:(id:string,error?:unknown)=>void):Promise<void> {
 for(const row of rows) {
  if(!isCurrent())break;
  if(row.status!=='ready'||!selected.has(row.id))continue;
  try{await write(row);if(!isCurrent())break;result(row.id);}catch(error){if(!isCurrent())break;result(row.id,error);}
 }
}
