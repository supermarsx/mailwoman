import {createSignal} from 'solid-js';
import {settings} from '../api/settings.ts';
import {validAvatar} from '../api/mailboxAvatar.ts';
import type {WorkspaceMailbox} from '../api/workspaceMail.ts';
import type {Mailbox} from '../api/jmap-types.ts';
import {activeLinkedMailbox,onLinkedMailboxChange} from '../api/linkedMailboxSession.ts';
export const unifiedAccountKey=(target:WorkspaceMailbox)=>target.linkId??'primary';
export const unifiedFolderKey=(target:WorkspaceMailbox,folder:Mailbox)=>JSON.stringify([unifiedAccountKey(target),folder.id]);
export const [unifiedConfig,writeUnifiedConfig]=createSignal<Record<string,unknown>>({});
export function setUnifiedConfig(value:Record<string,unknown>):void{
 writeUnifiedConfig(value);
 const selected=activeUnifiedGroup();if(selected&&!unifiedGroups().some(group=>group.id===selected))setActiveUnifiedGroup(null);
}
export const baseUnifiedPreference=(key:string)=>unifiedConfig()[key]??settings()[`mail.unifiedAccount.${key}`]?.value;
export const unifiedAccountEnabled=()=>activeLinkedMailbox()!==null&&typeof unifiedConfig()['enabled']==='boolean'?unifiedConfig()['enabled']===true:settings()['mail.unifiedAccount']?.value===true;
export interface UnifiedAccountGroup {id:string;name:string;accounts:string[];folders:string[];avatar?:string}
export function parseUnifiedGroups(value:unknown):UnifiedAccountGroup[]{
 try{const rows=typeof value==='string'?JSON.parse(value):value;if(!Array.isArray(rows)||rows.length>16)return [];const ids=new Set<string>();
  const text=(value:unknown,max:number)=>typeof value==='string'&&value.trim().length>0&&new TextEncoder().encode(value).length<=max&&![...value].some(char=>char.charCodeAt(0)<32||char.charCodeAt(0)>=127&&char.charCodeAt(0)<=159);
  if(!rows.every((row:UnifiedAccountGroup)=>row&&typeof row.id==='string'&&/^[A-Za-z0-9_-]{1,64}$/.test(row.id)&&!ids.has(row.id)&&!!ids.add(row.id)&&text(row.name,128)&&Array.isArray(row.accounts)&&row.accounts.length<=32&&row.accounts.every(value=>text(value,128))&&Array.isArray(row.folders)&&row.folders.length<=512&&row.folders.every(value=>text(value,512))&&(row.avatar===undefined||typeof row.avatar==='string'&&new TextEncoder().encode(row.avatar).length<=512)&&Object.keys(row).every(key=>['id','name','accounts','folders','avatar'].includes(key))))return [];
  return rows;
 }catch{return [];}
}
export const unifiedGroups=()=>parseUnifiedGroups(baseUnifiedPreference('groups'));
export const [activeUnifiedGroup,setActiveUnifiedGroup]=createSignal<string|null>(null);
onLinkedMailboxChange(()=>setActiveUnifiedGroup(null));
export const unifiedDefaultGroup=()=>{const id=baseUnifiedPreference('primaryGroup');return typeof id==='string'&&unifiedGroups().some(group=>group.id===id)?id:null;};
export const unifiedPreference=(key:string)=>{
 const group=unifiedGroups().find(group=>group.id===activeUnifiedGroup());
 return group&&['name','avatar','accounts','folders'].includes(key)?group[key as keyof UnifiedAccountGroup]:baseUnifiedPreference(key);
};
const list=(key:string):string[]=>{const value=unifiedPreference(key);return Array.isArray(value)?value.filter((item):item is string=>typeof item==='string'):['*'];};
const accepts=(values:string[],key:string,role?:string|null)=>values.includes('*')||values.includes(key)||!!role&&values.includes(`role:${role}`);
export const unifiedAccountAllowed=(target:WorkspaceMailbox,virtual=true)=>accepts(list('allowedAccounts'),unifiedAccountKey(target))&&(!virtual||accepts(list('accounts'),unifiedAccountKey(target)));
export const unifiedFolderAllowed=(target:WorkspaceMailbox,folder:Mailbox,virtual=true)=>accepts(list('allowedFolders'),unifiedFolderKey(target,folder),folder.role)&&(!virtual||accepts(list('folders'),unifiedFolderKey(target,folder),folder.role));
export function parseUnifiedAvatar(value:unknown){try{return validAvatar(JSON.parse(String(value??'')));}catch{return undefined;}}
export function unifiedAvatar(){return parseUnifiedAvatar(unifiedPreference('avatar'));}
export const unifiedName=()=>String(unifiedPreference('name')??'').trim();
