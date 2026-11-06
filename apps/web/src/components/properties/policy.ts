// What the administrator allows in the Properties dialogs, read from the
// effective settings (settings model C: global, domain, user; an admin lock wins).
// A dialog, a section, an override or a button the policy does not allow is not
// drawn at all; nothing here trusts a value a user typed.
import {settings} from '../../api/settings.ts';

export type FolderSection='general'|'remote'|'size'|'offline'|'overrides'|'notes'|'actions';
export type MessageSection='summary'|'scoring'|'headers'|'attachments'|'actions';
export const FOLDER_SECTIONS:readonly FolderSection[]=['general','remote','size','offline','overrides','notes','actions'];
export const MESSAGE_SECTIONS:readonly MessageSection[]=['summary','scoring','headers','attachments','actions'];
export type OverrideKey='counters'|'badgeColour'|'compactBadge'|'notifications'|'sort'|'threading'|'previewLines'|'markReadDelay'|'replyIdentity'|'unifiedInbox';
export const OVERRIDE_KEYS:readonly OverrideKey[]=['counters','badgeColour','compactBadge','notifications','sort','threading','previewLines','markReadDelay','replyIdentity','unifiedInbox'];
export type RemoteVisibility='full'|'masked-host'|'path-only'|'hidden';
export type SizeCompute='off'|'on-open'|'background';
export type PermanentDelete='allow'|'confirm-twice'|'deny';
export type ExportFormat='eml'|'mbox';

type Read=(key:string)=>unknown;
const live:Read=key=>settings()[key]?.value;

function list<T extends string>(value:unknown,all:readonly T[]):readonly T[] {
 // Unset (the setting not loaded yet) means the shipped default: everything.
 if(!Array.isArray(value))return all;
 return all.filter(item=>value.includes(item));
}
const flag=(value:unknown,fallback=true)=>typeof value==='boolean'?value:fallback;

export function folderDialogEnabled(read:Read=live):boolean {return flag(read('properties.folder.enabled'));}
export function messageDialogEnabled(read:Read=live):boolean {return flag(read('properties.message.enabled'));}
export function folderSections(read:Read=live):readonly FolderSection[] {
 const wanted=list(read('properties.folder.sections'),FOLDER_SECTIONS);
 return wanted.filter(section=>{
  if(section==='notes')return notesEnabled(read);
  if(section==='overrides')return allowedOverrides(read).length>0;
  if(section==='offline')return true;
  return true;
 });
}
export function messageSections(read:Read=live):readonly MessageSection[] {
 return list(read('properties.message.sections'),MESSAGE_SECTIONS).filter(section=>{
  if(section==='scoring')return flag(read('properties.scoring.show'));
  if(section==='headers')return flag(read('properties.headers.raw'));
  return true;
 });
}
export function allowedOverrides(read:Read=live):readonly OverrideKey[] {
 return list(read('properties.folder.overrides.allowed'),OVERRIDE_KEYS);
}
export function notesEnabled(read:Read=live):boolean {return flag(read('properties.folder.notes.enabled'));}
export function notesMaxLength(read:Read=live):number {
 const value=read('properties.folder.notes.maxLength');
 return typeof value==='number'&&value>=1?Math.min(2000,value):2000;
}
export function sizeCompute(read:Read=live):SizeCompute {
 const value=read('properties.folder.sizeCompute');
 return value==='off'||value==='background'?value:'on-open';
}
export function remoteVisibility(read:Read=live):RemoteVisibility {
 const value=read('properties.remoteLocation.visibility');
 return value==='full'||value==='path-only'||value==='hidden'?value:'masked-host';
}
export function offlineClearEnabled(read:Read=live):boolean {return flag(read('properties.offlineClear.enabled'));}
export function folderExportEnabled(read:Read=live):boolean {return flag(read('export.folder.enabled'));}
export function exportFormats(read:Read=live):readonly ExportFormat[] {
 const value=read('export.folder.format');
 return value==='zip-eml'?['eml']:value==='zip-mbox'?['mbox']:['eml','mbox'];
}
export function exportSubfoldersDefault(read:Read=live):boolean {return flag(read('export.folder.includeSubfolders'));}
export function messageDownloadEnabled(read:Read=live):boolean {return flag(read('export.message.download.enabled'));}
export function permanentDeletePolicy(read:Read=live):PermanentDelete {
 const value=read('properties.message.permanentDelete');
 return value==='deny'||value==='confirm-twice'?value:'allow';
}
export function shortcutEnabled(read:Read=live):boolean {return flag(read('properties.shortcut.enabled'));}
export function shortcutKey(read:Read=live):string {
 const value=read('properties.shortcut.key');
 return typeof value==='string'&&value.trim()?value.trim():'Alt+Enter';
}

/** Whether a keyboard event is the Properties shortcut (default Alt+Enter). */
export function matchesShortcut(event:Pick<KeyboardEvent,'key'|'altKey'|'ctrlKey'|'metaKey'|'shiftKey'>,combo:string=shortcutKey()):boolean {
 const parts=combo.split('+').map(part=>part.trim().toLowerCase()).filter(Boolean);
 const key=parts.pop();if(!key)return false;
 const want={alt:parts.includes('alt'),ctrl:parts.includes('ctrl')||parts.includes('control'),meta:parts.includes('meta')||parts.includes('cmd'),shift:parts.includes('shift')};
 return event.key.toLowerCase()===key&&event.altKey===want.alt&&event.ctrlKey===want.ctrl&&event.metaKey===want.meta&&event.shiftKey===want.shift;
}

/** The server location, reduced to what policy lets this user see. `host` is never filled by the engine; a deployment that does supply one is masked here. */
export interface RemoteLocation {path:string;upstreamName?:string;delimiter?:string|null;uidValidity?:number;uidNext?:number;highestModSeq?:number;namespace?:string;host?:string|null}
export function maskHost(host:string):string {
 const parts=host.split('.');
 if(parts.length<=2)return `${host.slice(0,1)}***`;
 return `${parts[0]!.slice(0,1)}***.${parts.slice(-2).join('.')}`;
}
export function remoteForDisplay(remote:RemoteLocation,visibility:RemoteVisibility):RemoteLocation|null {
 if(visibility==='hidden')return null;
 const {host,...rest}=remote;
 if(visibility==='path-only')return rest;
 if(visibility==='masked-host')return host?{...rest,host:maskHost(host)}:rest;
 return remote;
}
