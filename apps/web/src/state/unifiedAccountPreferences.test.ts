import {afterEach,expect,it} from 'vitest';
import {parseUnifiedGroups,setUnifiedConfig,setActiveUnifiedGroup,unifiedGroups,unifiedName,unifiedAccountAllowed,unifiedFolderAllowed,unifiedDefaultGroup} from './unifiedAccountPreferences.ts';
import {installLinkedMailbox} from '../api/linkedMailboxSession.ts';
import type {WorkspaceMailbox} from '../api/workspaceMail.ts';
import type {Mailbox} from '../api/jmap-types.ts';
const group={id:'work',name:'Work',accounts:['link'],folders:['role:inbox']};
afterEach(()=>{setUnifiedConfig({});setActiveUnifiedGroup(null);});
it('validates independent groups and refuses malformed, duplicate or unbounded definitions',()=>{
 expect(parseUnifiedGroups(JSON.stringify([group]))).toEqual([group]);
 for(const value of [[group,group],[{...group,id:'../bad'}],[{...group,name:' '}],[{...group,accounts:['']}],[{...group,folders:['bad\nfolder']}],[{...group,unknown:true}],Array.from({length:17},(_,i)=>({...group,id:String(i)}))])expect(parseUnifiedGroups(value)).toEqual([]);
});
it('switches group account and folder choices while retaining administrator bounds',()=>{
 setUnifiedConfig({name:'Default',accounts:['*'],folders:['*'],groups:JSON.stringify([group,{...group,id:'personal',name:'Personal',accounts:['primary']}]),primaryGroup:'work',allowedAccounts:['link'],allowedFolders:['role:inbox']});
 const target={linkId:'link'} as WorkspaceMailbox,primary={linkId:null} as WorkspaceMailbox;
 expect(unifiedGroups()).toHaveLength(2);expect(unifiedDefaultGroup()).toBe('work');setActiveUnifiedGroup('work');expect(unifiedName()).toBe('Work');expect(unifiedAccountAllowed(target)).toBe(true);expect(unifiedAccountAllowed(primary)).toBe(false);
 expect(unifiedFolderAllowed(target,{id:'in',role:'inbox'} as Mailbox)).toBe(true);expect(unifiedFolderAllowed(target,{id:'sent',role:'sent'} as Mailbox)).toBe(false);
 setActiveUnifiedGroup('personal');expect(unifiedName()).toBe('Personal');expect(unifiedAccountAllowed(primary)).toBe(false);
 setUnifiedConfig({name:'Default',groups:'[]'});expect(unifiedName()).toBe('Default');
 installLinkedMailbox(null);expect(unifiedName()).toBe('Default');
});
