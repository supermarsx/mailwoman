import {workspaceLayout,type WorkspaceLayout,type WorkspaceViewAssignments} from './workspaceLayout.ts';
/** Account-bound tab references only. Bodies, recipients, titles, URLs and keys
 * belong to their existing authorized stores and never enter this record. */
export const WORKSPACE_STORAGE_KEY='mw.workspace.v2';
export const MAX_WORKSPACE_TABS=32;
export type WorkspaceKind='messages'|'composer'|'settings'|'attachment'|'module'|'attachments'|'outbox'|'sharing';
export interface WorkspaceReference {id:string;kind:WorkspaceKind;pinned:boolean;reference?:string;messageId?:string}
export interface WorkspaceSnapshot {version:2;accountId:string;storageAuthority:string;tabs:WorkspaceReference[];activeId:string;layout:WorkspaceLayout;viewAssignments?:WorkspaceViewAssignments}
const kinds=new Set<WorkspaceKind>(['messages','composer','settings','attachment','module','attachments','outbox','sharing']);
const modules=new Set(['calendar','tasks','notes','contacts','keys']);
const identifier=(value:unknown):value is string=>typeof value==='string'&&value.length>0&&value.length<=512&&![...value].some(character=>character.charCodeAt(0)<32||character.charCodeAt(0)===127);
export function parseWorkspace(raw:string,accountId:string,storageAuthority:string):WorkspaceSnapshot|null {
 if(raw.length>65536||!identifier(accountId)||!/^[A-Za-z0-9_-]{16,256}$/.test(storageAuthority))return null;
 try {
  const value:unknown=JSON.parse(raw);if(!value||typeof value!=='object')return null;
  const record=value as Record<string,unknown>;
  if(record.version!==2||record.accountId!==accountId||record.storageAuthority!==storageAuthority||!Array.isArray(record.tabs)||record.tabs.length>MAX_WORKSPACE_TABS)return null;
  const tabs:WorkspaceReference[]=[];const seen=new Set<string>();
  for(const item of record.tabs){
   if(!item||typeof item!=='object')return null;
   const tab=item as Record<string,unknown>;
   if(!identifier(tab.id)||!kinds.has(tab.kind as WorkspaceKind)||typeof tab.pinned!=='boolean'||seen.has(tab.id))return null;
   if(tab.reference!==undefined&&!identifier(tab.reference)||tab.messageId!==undefined&&!identifier(tab.messageId))return null;
   if(tab.kind==='module'&&!modules.has(String(tab.reference)))return null;
   if(['attachment','sharing'].includes(String(tab.kind))&&!tab.reference)return null;
   if(tab.kind==='messages'&&tab.id!=='mail'&&!tab.reference)return null;
   seen.add(tab.id);tabs.push({id:tab.id,kind:tab.kind as WorkspaceKind,pinned:tab.pinned,...(typeof tab.reference==='string'?{reference:tab.reference}:{}),...(typeof tab.messageId==='string'?{messageId:tab.messageId}:{})});
  }
  const layout=workspaceLayout(record.layout);
  const rawViews=record.viewAssignments;
  const viewAssignments:WorkspaceViewAssignments|undefined=Array.isArray(rawViews)&&rawViews.length===2&&rawViews.every(id=>id===null||(typeof id==='string'&&id!=='mail'&&seen.has(id)))&&(rawViews[0]===null||rawViews[0]!==rawViews[1])?[rawViews[0] as string|null,rawViews[1] as string|null]:undefined;
  return {version:2,accountId,storageAuthority,tabs,activeId:typeof record.activeId==='string'&&seen.has(record.activeId)?record.activeId:'mail',layout,...(viewAssignments?{viewAssignments}:{})};
 }catch{return null;}
}
export function readWorkspace(accountId:string,storageAuthority:string):WorkspaceSnapshot|null {
 try {const raw=localStorage.getItem(WORKSPACE_STORAGE_KEY);if(raw===null)return null;const value=parseWorkspace(raw,accountId,storageAuthority);if(!value)localStorage.removeItem(WORKSPACE_STORAGE_KEY);return value;}catch{return null;}
}
export function saveWorkspace(snapshot:WorkspaceSnapshot):void {
 try {const safe=parseWorkspace(JSON.stringify(snapshot),snapshot.accountId,snapshot.storageAuthority);if(safe)localStorage.setItem(WORKSPACE_STORAGE_KEY,JSON.stringify(safe));}catch{/* Storage denial never prevents navigation. */}
}
export function clearWorkspace():void {try{localStorage.removeItem(WORKSPACE_STORAGE_KEY);}catch{/* Storage unavailable. */}}
