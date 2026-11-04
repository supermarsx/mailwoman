import {afterEach,beforeEach,describe,expect,it,vi} from 'vitest';
import {cleanup,fireEvent,render,screen,waitFor,within} from '@solidjs/testing-library';
import {AppContext} from '../state/context.ts';
import {createAppState} from '../state/store.ts';
import {makeClient,mkEmail,MAILBOXES} from '../components/appHarness.tsx';
import {MailboxScreen} from './Mailbox.tsx';
import {RealtimeContext} from '../realtime/context.ts';
import {createRealtimeController} from '../realtime/controller.ts';
import {clearSettings,loadSettings} from '../api/settings.ts';
import {suspendAccountPreferences} from '../api/accountPreferenceLifecycle.ts';
import {WORKSPACE_STORAGE_KEY} from '../realtime/workspaceStorage.ts';
import type {JmapRequest,Mailbox} from '../api/jmap-types.ts';

// The mail.showSpam and mail.closeOnDelete settings, through the whole mailbox
// screen: the Junk folder leaves the folder list, and a message open in a tab
// closes (or stays) when it is deleted.

const MAILBOXES_WITH_JUNK:Mailbox[]=[...MAILBOXES,{id:'junk',name:'Junk',parentId:null,role:'junk',sortOrder:3,totalEmails:0,unreadEmails:0}];
const apply=(values:Record<string,boolean>)=>loadSettings({refresh:true,fetcher:async()=>new Response(JSON.stringify({settings:Object.fromEntries(Object.entries(values).map(([key,value])=>[key,{value,source:'global',locked:true}]))}),{headers:{'content-type':'application/json'}})});

beforeEach(async()=>{suspendAccountPreferences();clearSettings();localStorage.removeItem(WORKSPACE_STORAGE_KEY);history.replaceState(null,'','#/mail');await new Promise(resolve=>setTimeout(resolve,0));});
afterEach(()=>{cleanup();clearSettings();});

async function mount(values:Record<string,boolean>){
  const client=makeClient({emails:[mkEmail('a'),mkEmail('b')],mailboxes:MAILBOXES_WITH_JUNK});
  const app=createAppState(client);const realtime=createRealtimeController();
  await apply(values);
  await app.login({username:'me@example.org',password:'p'});
  render(()=><AppContext.Provider value={app}><RealtimeContext.Provider value={realtime}><MailboxScreen/></RealtimeContext.Provider></AppContext.Provider>);
  await screen.findByText('Subject a',{selector:'.list__subject'});
  return {app,realtime,client};
}
const messageTab=(realtime:ReturnType<typeof createRealtimeController>)=>realtime.subTabs.tabs().find(tab=>tab.id==='message:acct1:a');
async function openTab(realtime:ReturnType<typeof createRealtimeController>){
  fireEvent.dblClick(screen.getByText('Subject a',{selector:'.list__subject'}));
  await waitFor(()=>expect(messageTab(realtime)).toBeDefined());
  await screen.findByRole('heading',{name:'Subject a'});
}

describe('Mailbox and mail.showSpam',()=>{
  it('lists the Junk folder by default',async()=>{
    await mount({});
    expect(screen.queryAllByText('Junk').length).toBeGreaterThan(0);
  });
  it('hides the Junk folder when the setting is off, keeping the other folders',async()=>{
    await mount({'mail.showSpam':false});
    await waitFor(()=>expect(screen.queryAllByText('Junk')).toHaveLength(0));
    expect(screen.queryAllByText('Archive').length).toBeGreaterThan(0);
  });
  it('asks the server for the folder without $junk messages only when the setting is off',async()=>{
    const queryFilters=(client:ReturnType<typeof makeClient>)=>vi.mocked(client.jmap).mock.calls.flatMap(call=>(call[0] as JmapRequest).methodCalls.filter(method=>method[0]==='Email/query').map(method=>method[1]['filter'] as Record<string,unknown>)).filter(filter=>filter['inMailbox']==='inbox');
    const on=await mount({});
    expect(queryFilters(on.client).length).toBeGreaterThan(0);
    for(const filter of queryFilters(on.client))expect(filter['notKeyword']).toBeUndefined();
    cleanup();
    const off=await mount({'mail.showSpam':false});
    expect(queryFilters(off.client).length).toBeGreaterThan(0);
    for(const filter of queryFilters(off.client))expect(filter['notKeyword']).toBe('$junk');
  });
});

describe('Mailbox move targets and mail.showSpam',()=>{
  let values:Record<string,boolean>={};
  const selectOne=()=>mount(values);
  const moveTargets=async()=>{
    fireEvent.click(await screen.findByRole('button',{name:'Select messages'}));
    fireEvent.click(await screen.findByText('Subject a',{selector:'.list__subject'}));
    fireEvent.click(await screen.findByTestId('bulk-move'));
    return within(await screen.findByRole('menu')).getAllByRole('menuitem').map(item=>item.textContent?.trim());
  };
  const rowMenu=async()=>{
    fireEvent.contextMenu(await screen.findByText('Subject a',{selector:'.list__subject'}));
    return within(await screen.findByRole('menu')).getAllByRole('menuitem').map(item=>item.textContent?.trim());
  };
  it('offers Junk as a bulk move target and a Mark spam row by default',async()=>{
    values={};await selectOne();
    expect(await moveTargets()).toContain('Junk');
    cleanup();
    await selectOne();
    expect((await rowMenu()).some(label=>/spam/i.test(label??''))).toBe(true);
  });
  it('leaves Junk out of the bulk move menu and the row menu when the setting is off',async()=>{
    values={'mail.showSpam':false};await selectOne();
    const targets=await moveTargets();
    expect(targets).toContain('Archive');
    expect(targets).not.toContain('Junk');
    cleanup();
    await selectOne();
    const rows=await rowMenu();
    expect(rows.length).toBeGreaterThan(0);
    expect(rows.some(label=>/spam/i.test(label??''))).toBe(false);
  });
});

describe('Mailbox and mail.closeOnDelete',()=>{
  it('closes the message tab of a deleted message by default',async()=>{
    const {app,realtime}=await mount({});
    await openTab(realtime);
    await app.trashMessage('a');
    await waitFor(()=>expect(messageTab(realtime)).toBeUndefined());
  });
  it('keeps the message tab open when the setting is off',async()=>{
    const {app,realtime}=await mount({'mail.closeOnDelete':false});
    await openTab(realtime);
    await app.trashMessage('a');
    await waitFor(()=>expect(screen.queryByText('Subject a',{selector:'.list__subject'})).toBeNull());
    await new Promise(resolve=>setTimeout(resolve,100));
    expect(messageTab(realtime)).toBeDefined();
  });
});
