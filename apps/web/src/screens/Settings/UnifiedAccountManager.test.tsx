import {createSignal} from 'solid-js';
import {cleanup,fireEvent,render,screen,within,waitFor} from '@solidjs/testing-library';
import {afterEach,expect,it,vi} from 'vitest';
import {UnifiedAccountManager} from './UnifiedAccountManager.tsx';
import type {SettingHandle} from './useSetting.ts';
import type {LinkedMailboxList} from '../../api/linkedAccounts.ts';
const handles=new Map<string,SettingHandle<unknown>>();const locked=new Set<string>();
vi.mock('./useSetting.ts',()=>({useSetting:(key:string,fallback:unknown)=>{
 if(!handles.has(key)){const [value,setValue]=createSignal(fallback);handles.set(key,{value,entry:()=>({value:value(),source:'user',locked:locked.has(key)}),locked:()=>locked.has(key),saving:()=>false,error:()=>'',set:vi.fn(async next=>{if(locked.has(key))return false;setValue(()=>next);return true;}),reset:async()=>true});}return handles.get(key);
}}));
vi.mock('../../i18n',()=>({loadCatalog:async()=>{},t:(key:string)=>key,isolate:(value:string)=>value}));
vi.mock('../../api/workspaceMail.ts',()=>({workspaceMailboxes:async()=>[{linkId:null,label:'Primary'},{linkId:'work-link',label:'Work'}],workspaceFolders:async()=>[{id:'inbox',role:'inbox',name:'Inbox'},{id:'sent',role:'sent',name:'Sent'}]}));
const value:LinkedMailboxList={primary:{accountId:'a',address:'a@example.org',storageAuthority:'primary_authority'},accounts:[{id:'work-link',accountId:'b',displayName:'Work',status:'ready',revision:'1'}],policy:{enabled:true,maxAccounts:4,supportedAuthentication:['password']}};
const get=(key:string)=>handles.get('mail.unifiedAccount.'+key)!;
afterEach(()=>{cleanup();handles.clear();locked.clear();});
it('creates and edits independent groups with included accounts and folders',async()=>{
 render(()=><UnifiedAccountManager list={value} onChanged={()=>{}}/>);
 fireEvent.click(screen.getByRole('button',{name:'unified-config-add-group'}));await waitFor(()=>expect(document.querySelector('[data-unified-group]')).not.toBeNull());
 let row=document.querySelector<HTMLElement>('[data-unified-group]')!;
 fireEvent.change(within(row).getByLabelText('unified-config-name'),{target:{value:'Work only'}});await waitFor(()=>expect(JSON.parse(String(get('groups').value()))[0].name).toBe('Work only'));
 row=document.querySelector<HTMLElement>('[data-unified-group]')!;
 await waitFor(()=>expect(within(row).getByLabelText('Work')).toBeInTheDocument());fireEvent.click(within(row).getByLabelText('Work'));await waitFor(()=>expect(JSON.parse(String(get('groups').value()))[0].accounts).toEqual(['work-link']));
 row=document.querySelector<HTMLElement>('[data-unified-group]')!;
 const inbox=within(row).getByLabelText('Inbox');expect(inbox).toBeChecked();fireEvent.click(inbox);await waitFor(()=>expect(JSON.parse(String(get('groups').value()))[0].folders).not.toContain(JSON.stringify(['work-link','inbox'])));
 row=document.querySelector<HTMLElement>('[data-unified-group]')!;
 fireEvent.click(within(row).getByRole('button',{name:'common-delete'}));await waitFor(()=>expect(get('groups').value()).toBe('[]'));
});
it('honors administrator locks and preserves enabled state when switching the default group',async()=>{
 render(()=><UnifiedAccountManager list={value} onChanged={()=>{}}/>);await handles.get('mail.unifiedAccount')!.set(true);await get('openFirst').set(true);await get('primaryGroup').set('work');
 fireEvent.click(screen.getByRole('button',{name:'unified-config-open-first'}));await waitFor(()=>expect(get('primaryGroup').value()).toBe(''));expect(get('openFirst').value()).toBe(true);
 locked.add('mail.unifiedAccount.groups');cleanup();render(()=><UnifiedAccountManager list={value} onChanged={()=>{}}/>);expect(screen.getByRole('button',{name:'unified-config-add-group'})).toBeDisabled();
});
