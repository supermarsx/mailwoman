import {afterEach, expect, it, vi} from 'vitest';
import {createRoot} from 'solid-js';
import {cleanup, fireEvent, render, screen, waitFor} from '@solidjs/testing-library';
import {CreateFolderDialog, CleanupFolderDialog, FolderCreationControl} from './FolderMaintenance.tsx';
import {makeClient, MAILBOXES, mkEmail} from './appHarness.tsx';
import {AppContext} from '../state/context.ts';
import {createAppState} from '../state/store.ts';
import {suspendAccountPreferences} from '../api/accountPreferenceLifecycle.ts';
import type {Client} from '../api/client.ts';
import type {JmapRequest, JmapResponse} from '../api/jmap-types.ts';
const state=vi.hoisted(()=>({client:null as Client|null,abilityReads:vi.fn()}));
vi.mock('../api/transport.ts',()=>({createConfiguredClient:()=>state.client!}));
vi.mock('./RenameFolder.tsx',()=>({useRenameContext:()=>()=>{state.abilityReads();return {manages:true,delimiter:'/',maxLength:255};}}));
const disposers:(()=>void)[]=[];
afterEach(()=>{cleanup();disposers.splice(0).forEach(dispose=>dispose());suspendAccountPreferences();localStorage.clear();});
async function setup(){
 const base=makeClient();
 const jmap=vi.fn(async(request:JmapRequest):Promise<JmapResponse>=>{
 const [name,args,id]=request.methodCalls[0]!;
 const reply=(data:Record<string,unknown>):JmapResponse=>({methodResponses:[[name,{accountId:'acct1',...data},id]],sessionState:'s'});
 if(name==='Mailbox/set')return reply({created:{folder:{id:'new-folder'}}});
 if(id==='old')return reply({ids:['old-message'],queryState:'q'});
 if(id==='verify')return reply({state:'mail-state',list:[mkEmail('old-message',{receivedAt:'2020-01-01T00:00:00Z'})],notFound:[]});
 if(id==='clean')return reply({updated:{'old-message':null}});
 void args;return base.jmap(request);
 });
 const client={...base,jmap};state.client=client;
 const app=createRoot(dispose=>{disposers.push(dispose);return createAppState(client);});await app.login({username:'me@example.org',password:'p'});
 return {app,jmap};
}
it('creates a folder from the dialog and refreshes the folder list',async()=>{
 const {app,jmap}=await setup();const close=vi.fn(),refresh=vi.spyOn(app,'refreshFolders');
 render(()=><AppContext.Provider value={app}><CreateFolderDialog onClose={close}/></AppContext.Provider>);
 fireEvent.input(screen.getByLabelText('Folder name'),{target:{value:'Reports'}});
 fireEvent.click(screen.getByRole('button',{name:'Create folder'}));
 await waitFor(()=>expect(close).toHaveBeenCalledOnce());
 expect(jmap.mock.calls.some(([request])=>request.methodCalls.some(([method,args])=>method==='Mailbox/set'&&JSON.stringify(args).includes('Reports')))).toBe(true);
 expect(refresh).toHaveBeenCalled();
});
it('previews old messages before a separate confirmation moves them to Trash',async()=>{
 const {app,jmap}=await setup();const close=vi.fn();
 render(()=><AppContext.Provider value={app}><CleanupFolderDialog box={MAILBOXES[0]!} onClose={close}/></AppContext.Provider>);
 fireEvent.input(screen.getByLabelText('Older than'),{target:{value:'4'}});
 fireEvent.click(screen.getByRole('button',{name:'Preview cleanup'}));
 await screen.findByText('Move 1 matching messages to Trash.');
 expect(jmap.mock.calls.some(([request])=>request.methodCalls[0]?.[2]==='clean')).toBe(false);
 fireEvent.click(screen.getByRole('button',{name:'Delete matching messages'}));
 await waitFor(()=>expect(close).toHaveBeenCalledOnce());
 const mutation=jmap.mock.calls.find(([request])=>request.methodCalls[0]?.[2]==='clean')![0].methodCalls[0]![1];
 expect(mutation).toMatchObject({accountId:'acct1',ifInState:'mail-state',update:{'old-message':{'mailboxIds/inbox':null,'mailboxIds/trash':true}}});
});
it('closes folder dialogs when account authority changes',async()=>{
 const {app,jmap}=await setup();const createClose=vi.fn(),cleanupClose=vi.fn();
 render(()=><AppContext.Provider value={app}><CreateFolderDialog onClose={createClose}/><CleanupFolderDialog box={MAILBOXES[0]!} onClose={cleanupClose}/></AppContext.Provider>);
 suspendAccountPreferences();
 expect(createClose).toHaveBeenCalledOnce();expect(cleanupClose).toHaveBeenCalledOnce();
 expect(jmap.mock.calls.some(([request])=>request.methodCalls[0]?.[0]==='Mailbox/set'||request.methodCalls[0]?.[2]==='clean')).toBe(false);
});

it('keeps folder creation in the background menu without a standalone button',async()=>{
 const {app}=await setup();const reads=state.abilityReads.mock.calls.length;
 render(()=><AppContext.Provider value={app}><FolderCreationControl><div data-testid="folder-background">Folders</div></FolderCreationControl></AppContext.Provider>);
 expect(state.abilityReads.mock.calls.length).toBeGreaterThan(reads);
 expect(screen.queryByRole('button',{name:'Create folder'})).toBeNull();
 fireEvent.contextMenu(screen.getByTestId('folder-background'));
 fireEvent.click(screen.getByRole('menuitem',{name:'Create folder'}));
 expect(await screen.findByRole('alertdialog',{name:'Create folder'})).toBeInTheDocument();
});
