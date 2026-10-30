import {afterEach,beforeEach,expect,it,vi} from 'vitest';
import {cleanup,fireEvent,render,screen,waitFor,within} from '@solidjs/testing-library';
import {AppContext} from '../state/context.ts';
import {createAppState} from '../state/store.ts';
import {makeClient,mkEmail} from '../components/appHarness.tsx';
import {RealtimeContext} from '../realtime/context.ts';
import {createRealtimeController} from '../realtime/controller.ts';
import {MailboxScreen} from './Mailbox.tsx';
import type {ComposeInitial} from '../components/compose/reply.ts';
import {suspendAccountPreferences} from '../api/accountPreferenceLifecycle.ts';
import {WORKSPACE_STORAGE_KEY} from '../realtime/workspaceStorage.ts';

// Keep the workspace's actual folder/message/reply routing. Folder transport and
// the outgoing editor are independent contracts covered in their own suites.
vi.mock('../components/FolderPane.tsx',()=>({FolderPane:(props:{onOpen:(id:string)=>void})=><button onClick={()=>props.onOpen('shared')}>Open project message</button>}));
vi.mock('../components/Compose.tsx',()=>({Compose:(props:{initial?:ComposeInitial})=><output data-testid="reply-origin">{JSON.stringify(props.initial)}</output>}));
beforeEach(()=>{suspendAccountPreferences();localStorage.removeItem(WORKSPACE_STORAGE_KEY);history.replaceState(null,'','#/mail');});
afterEach(()=>{cleanup();suspendAccountPreferences();});

it('keeps the originating folder tab when its message also belongs to the main Inbox',async()=>{
 const message=mkEmail('shared',{mailboxIds:{inbox:true,projects:true},messageId:['shared@example.org']});
 const app=createAppState(makeClient({emails:[message]}));
 await app.login({username:'me@example.org',password:'p'});
 const boxes=[...app.mailboxes(),{id:'projects',name:'Projects',parentId:null,role:null,sortOrder:10,totalEmails:1,unreadEmails:1}];
 vi.spyOn(app,'mailboxes').mockReturnValue(boxes);
 const realtime=createRealtimeController();
 render(()=><AppContext.Provider value={app}><RealtimeContext.Provider value={realtime}><MailboxScreen/></RealtimeContext.Provider></AppContext.Provider>);
 fireEvent.contextMenu(await screen.findByRole('button',{name:'Projects'}));
 fireEvent.click(await screen.findByRole('menuitem',{name:'Open in new tab'}));
 fireEvent.click(await screen.findByRole('button',{name:'Open project message'}));
 const tab=await screen.findByRole('tabpanel',{name:'Subject shared'});
 await waitFor(()=>expect(within(tab).getByRole('button',{name:/^Reply$/})).toBeEnabled());
 fireEvent.click(within(tab).getByRole('button',{name:/^Reply$/}));
 const initial=JSON.parse((await screen.findByTestId('reply-origin')).textContent!) as ComposeInitial;
 expect(app.selectedMailboxId()).toBe('inbox');
 expect(initial.sourceMailboxIds).toEqual(['inbox','projects']);
 expect(initial.sourceMailboxId).toBe('projects');
});
