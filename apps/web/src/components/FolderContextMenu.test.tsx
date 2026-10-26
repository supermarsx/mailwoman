import {afterEach,expect,it,vi} from 'vitest';
import {cleanup,fireEvent,render,screen,waitFor} from '@solidjs/testing-library';
import {FolderContextMenu} from './FolderContextMenu.tsx';
import {makeClient,MAILBOXES} from './appHarness.tsx';
import {createAppState} from '../state/store.ts';
import {AppContext} from '../state/context.ts';
import {accountUiPreferences} from '../api/accountPreferences.ts';
import {suspendAccountPreferences} from '../api/accountPreferenceLifecycle.ts';
afterEach(()=>{cleanup();localStorage.clear();});
it('offers pin and real folder actions without navigating on open',async()=>{
 const app=createAppState(makeClient()),open=vi.fn(),settings=vi.fn(),share=vi.fn();
 render(()=><AppContext.Provider value={app}><FolderContextMenu box={MAILBOXES[0]!} onOpen={open} onSettings={settings} onShare={share} canShare={()=>false} onProbe={()=>{}}><button>Inbox target</button></FolderContextMenu></AppContext.Provider>);await app.login({username:'me@example.org',password:'p'});await waitFor(()=>expect(app.mailboxes()).toHaveLength(3));
 fireEvent.contextMenu(screen.getByText('Inbox target'));expect(open).not.toHaveBeenCalled();expect(screen.queryByRole('menuitem',{name:'Share folder'})).toBeNull();expect(screen.queryByRole('menuitem',{name:'Empty Trash'})).toBeNull();expect(screen.queryByRole('menuitem',{name:'Delete folder'})).toBeNull();fireEvent.click(screen.getByRole('menuitem',{name:'Pin'}));expect(accountUiPreferences().folderPinned).toContain('inbox');
 expect(screen.queryByRole('button',{name:'More actions: Inbox'})).toBeNull();fireEvent.keyDown(screen.getByText('Inbox target'),{key:'F10',shiftKey:true});expect(screen.getByRole('menuitem',{name:'Unpin'})).toBeInTheDocument();fireEvent.click(screen.getByRole('menuitem',{name:'Folder settings'}));expect(settings).toHaveBeenCalledOnce();expect(open).not.toHaveBeenCalled();
 fireEvent.keyDown(screen.getByText('Inbox target'),{key:'F10',shiftKey:true});suspendAccountPreferences();expect(screen.queryByRole('menu')).toBeNull();
});
it('does not open sharing after a pending selection crosses account authority',async()=>{
 const app=createAppState(makeClient()),share=vi.fn();render(()=><AppContext.Provider value={app}><FolderContextMenu box={MAILBOXES[0]!} onOpen={()=>{}} onSettings={()=>{}} onShare={share} canShare={()=>true} onProbe={()=>{}}><button>Inbox target</button></FolderContextMenu></AppContext.Provider>);await app.login({username:'me@example.org',password:'p'});
 let finish!:()=>void;vi.spyOn(app,'selectMailbox').mockImplementation(async()=>await new Promise<void>(resolve=>{finish=resolve;}));fireEvent.contextMenu(screen.getByText('Inbox target'));fireEvent.click(screen.getByRole('menuitem',{name:'Share folder'}));suspendAccountPreferences();finish();await Promise.resolve();expect(share).not.toHaveBeenCalled();
});
