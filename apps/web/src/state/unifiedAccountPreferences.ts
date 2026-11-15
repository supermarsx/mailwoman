import {createSignal} from 'solid-js';
import {settings} from '../api/settings.ts';
import {validAvatar} from '../api/mailboxAvatar.ts';
import type {WorkspaceMailbox} from '../api/workspaceMail.ts';
import type {Mailbox} from '../api/jmap-types.ts';
export const unifiedAccountKey=(target:WorkspaceMailbox)=>target.linkId??'primary';
export const unifiedFolderKey=(target:WorkspaceMailbox,folder:Mailbox)=>JSON.stringify([unifiedAccountKey(target),folder.id]);
export const [unifiedConfig,setUnifiedConfig]=createSignal<Record<string,unknown>>({});
export const unifiedPreference=(key:string)=>unifiedConfig()[key]??settings()[`mail.unifiedAccount.${key}`]?.value;
const list=(key:string):string[]=>{const value=unifiedPreference(key);return Array.isArray(value)?value.filter((item):item is string=>typeof item==='string'):['*'];};
const accepts=(values:string[],key:string,role?:string|null)=>values.includes('*')||values.includes(key)||!!role&&values.includes(`role:${role}`);
export const unifiedAccountAllowed=(target:WorkspaceMailbox,virtual=true)=>accepts(list('allowedAccounts'),unifiedAccountKey(target))&&(!virtual||accepts(list('accounts'),unifiedAccountKey(target)));
export const unifiedFolderAllowed=(target:WorkspaceMailbox,folder:Mailbox,virtual=true)=>accepts(list('allowedFolders'),unifiedFolderKey(target,folder),folder.role)&&(!virtual||accepts(list('folders'),unifiedFolderKey(target,folder),folder.role));
export function unifiedAvatar(){try{return validAvatar(JSON.parse(String(unifiedPreference('avatar')??'')));}catch{return undefined;}}
export const unifiedName=()=>String(unifiedPreference('name')??'').trim();
