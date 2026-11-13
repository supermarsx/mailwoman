import {afterEach,expect,it,vi} from 'vitest';
import {cleanup,fireEvent,render,screen,waitFor} from '@solidjs/testing-library';
import {AppContext} from '../state/context.ts';
import {activateAccountPreferences,preferenceAccount,suspendAccountPreferences} from '../api/accountPreferenceLifecycle.ts';
import {clearSettings,loadSettings} from '../api/settings.ts';
import type {AppState} from '../state/store.ts';
import type {Email,Mailbox} from '../api/jmap-types.ts';
import type {WorkspaceMailbox} from '../api/workspaceMail.ts';
import UnifiedMail from './UnifiedMail.tsx';
const fixture=vi.hoisted(()=>({targets:[] as WorkspaceMailbox[],update:vi.fn(async(..._args:unknown[])=>['same'])}));
vi.mock('../api/workspaceMail.ts',async original=>({...await original<typeof import('../api/workspaceMail.ts')>(),workspaceMailboxes:async()=>fixture.targets,workspaceFolders:async()=>[{id:'inbox',name:'Inbox',role:'inbox',myRights:{maySetKeywords:true,mayRemoveItems:true}},{id:'archive',name:'Archive',role:'archive',myRights:{mayAddItems:true}}] as unknown as Mailbox[],workspaceMessages:async(target:WorkspaceMailbox)=>({rows:[{target,email:{id:'same',subject:`Mail from ${target.label}`,from:[{name:'Sender',email:'sender@example.test'}],to:[],receivedAt:'2026-11-13T00:00:00Z',preview:'A readable preview',keywords:{},mailboxIds:{inbox:true},hasAttachment:true} as Email,folderId:'inbox'}],next:null,total:5,unread:3}),updateWorkspaceMessages:(...args:unknown[])=>fixture.update(...args)}));
afterEach(()=>{cleanup();clearSettings();suspendAccountPreferences();fixture.update.mockReset().mockResolvedValue(['same']);});
it('uses normal message rows with aggregate counts and routes bulk updates to each source mailbox',async()=>{
 activateAccountPreferences('owner','primary_authority_1234');const owner=preferenceAccount()!;
 fixture.targets=[{linkId:null,accountId:'a',storageAuthority:'primary_authority_1234',owner,label:'Primary',current:false},{linkId:'other',accountId:'b',storageAuthority:'linked_authority_12345',owner,label:'Work',current:false}];
 const app={density:()=> 'cozy',tags:()=>[],identities:()=>[],me:()=>({username:'me@example.test'})} as unknown as AppState;
 const view=render(()=><AppContext.Provider value={app}><UnifiedMail activeLink={null}/></AppContext.Provider>);
 await screen.findByText('Mail from Work');expect(view.container.querySelectorAll('.list__row')).toHaveLength(2);expect(screen.getByText('Total: 10')).toBeInTheDocument();expect(screen.getByText('Unread: 6')).toBeInTheDocument();expect(view.container.querySelectorAll('.list__preview')).toHaveLength(2);
 fireEvent.click(screen.getByRole('button',{name:'Select messages'}));fireEvent.click(screen.getByRole('checkbox',{name:'Selected: Mail from Primary'}));fireEvent.click(screen.getByRole('checkbox',{name:'Selected: Mail from Work'}));
 const archive=screen.getAllByRole('button',{name:'Archive'}).at(-1)!;fireEvent.click(archive);await waitFor(()=>expect(fixture.update).toHaveBeenCalledTimes(2));
 expect(fixture.update.mock.calls.map(call=>(call[0] as unknown as WorkspaceMailbox).accountId).sort()).toEqual(['a','b']);
 await waitFor(()=>expect(view.container.querySelectorAll('.list__row')).toHaveLength(0));expect(screen.getByText('Total: 8')).toBeInTheDocument();expect(screen.getByText('Unread: 4')).toBeInTheDocument();
});
it('retains failed selections and only removes confirmed messages when a bulk action partially succeeds',async()=>{
 activateAccountPreferences('owner','primary_authority_1234');const owner=preferenceAccount()!;
 fixture.targets=[{linkId:null,accountId:'a',storageAuthority:'primary_authority_1234',owner,label:'Primary',current:false},{linkId:'other',accountId:'b',storageAuthority:'linked_authority_12345',owner,label:'Work',current:false}];
 fixture.update.mockResolvedValueOnce([]);
 const app={density:()=> 'cozy',tags:()=>[],identities:()=>[],me:()=>({username:'me@example.test'})} as unknown as AppState;
 const view=render(()=><AppContext.Provider value={app}><UnifiedMail activeLink={null}/></AppContext.Provider>);await screen.findByText('Mail from Work');
 fireEvent.click(screen.getByRole('button',{name:'Select messages'}));fireEvent.click(screen.getByRole('checkbox',{name:'Selected: Mail from Primary'}));fireEvent.click(screen.getByRole('checkbox',{name:'Selected: Mail from Work'}));fireEvent.click(screen.getAllByRole('button',{name:'Archive'}).at(-1)!);
 await waitFor(()=>expect(view.container.querySelectorAll('.list__row')).toHaveLength(1));expect(screen.getByRole('checkbox',{name:'Selected: Mail from Primary'})).toBeChecked();expect(screen.getByText('Total: 9')).toBeInTheDocument();expect(screen.getByText('Unread: 5')).toBeInTheDocument();
});

it('uses standard row height, bulk-mode gestures and an in-app reader callback',async()=>{
 activateAccountPreferences('owner','primary_authority_1234');const owner=preferenceAccount()!;fixture.targets=[{linkId:null,accountId:'a',storageAuthority:'primary_authority_1234',owner,label:'Primary',current:false}];
 const app={density:()=> 'cozy',tags:()=>[],identities:()=>[],me:()=>({username:'me@example.test'})} as unknown as AppState,open=vi.fn();
 const view=render(()=><AppContext.Provider value={app}><UnifiedMail activeLink={null} onOpenMessage={open}/></AppContext.Provider>);await screen.findByText('Mail from Primary');
 const row=view.container.querySelector<HTMLButtonElement>('.list__row')!;expect(row.closest('li')?.style.height).toBe('72px');expect(screen.queryByRole('checkbox',{name:'Selected: Mail from Primary'})).toBeNull();
 fireEvent.click(row);expect(open).toHaveBeenCalledWith(expect.objectContaining({target:fixture.targets[0],email:expect.objectContaining({id:'same'})}));
 fireEvent.click(screen.getByRole('button',{name:'Select messages'}));fireEvent.click(row);expect(open).toHaveBeenCalledTimes(1);expect(screen.getByRole('checkbox',{name:'Selected: Mail from Primary'})).toBeChecked();
 fireEvent.keyDown(row,{key:'Escape'});expect(screen.getByRole('button',{name:'Select messages'})).toHaveAttribute('aria-pressed','true');fireEvent.keyDown(row,{key:'Escape'});expect(screen.getByRole('button',{name:'Select messages'})).toHaveAttribute('aria-pressed','false');
 await loadSettings({refresh:true,fetcher:async()=>Response.json({settings:{'mail.messageListStyle':{value:'table',source:'user',locked:false}}})});
 await waitFor(()=>expect(view.container.querySelector('.list__slot')?.getAttribute('style')).toContain('44px'));expect(screen.getByTestId('unified-mail')).toHaveAttribute('data-list-style','table');
});
