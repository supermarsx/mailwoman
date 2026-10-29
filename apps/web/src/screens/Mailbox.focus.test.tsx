// t51: opening a message never moves focus off the list row that opened it
// while that list is still on screen. The reader shows the message, a polite
// status names it, and the arrow keys keep working. Only where the reader covers
// the list (reading pane Off) does focus move, to the reader's subject, because
// the focused row is hidden and focus would otherwise fall to <body>.
import {loadSettings} from '../api/settings.ts';
import {WORKSPACE_STORAGE_KEY} from '../realtime/workspaceStorage.ts';
import {suspendAccountPreferences} from '../api/accountPreferenceLifecycle.ts';
import { beforeEach, afterEach, describe, expect, it, vi } from 'vitest';
import type { EmailGetResponse, JmapRequest } from '../api/jmap-types.ts';
import { render, fireEvent, screen, waitFor, cleanup } from '@solidjs/testing-library';
import { AppContext } from '../state/context.ts';
import { createAppState } from '../state/store.ts';
import { makeClient, mkEmail } from '../components/appHarness.tsx';
import { MailboxScreen } from './Mailbox.tsx';
import { RealtimeContext } from '../realtime/context.ts';
import { createRealtimeController } from '../realtime/controller.ts';
import { setReadingPane, type ReadingPane } from '../components/readingPane.ts';

beforeEach(async()=>{suspendAccountPreferences();localStorage.removeItem(WORKSPACE_STORAGE_KEY);history.replaceState(null,'','#/mail');await new Promise(r=>setTimeout(r,0));});
afterEach(()=>{cleanup();setReadingPane('right');});

async function mount(){
  const emails=['a','b','c'].map(id=>mkEmail(id,{keywords:{}}));const client=makeClient({emails});
  // As Mailbox.workspace.test: Email/get answers only the ids it was asked for.
  const original=client.jmap;
  client.jmap=vi.fn(async(request:JmapRequest)=>{const response=await original(request);const ids=request.methodCalls.find(call=>call[0]==='Email/get')?.[1]['ids'];if(Array.isArray(ids)){for(const call of response.methodResponses)if(call[0]==='Email/get')(call[1] as unknown as EmailGetResponse).list=emails.filter(email=>ids.includes(email.id));}return response;});
  const app=createAppState(client);const realtime=createRealtimeController();
  const view=render(()=><AppContext.Provider value={app}><RealtimeContext.Provider value={realtime}><MailboxScreen/></RealtimeContext.Provider></AppContext.Provider>);
  await loadSettings({refresh:true,fetcher:async()=>new Response(JSON.stringify({settings:{}}),{headers:{'Content-Type':'application/json'}})});
  await app.login({username:'me@example.org',password:'p'});
  await waitFor(()=>expect(view.container.querySelectorAll('.list__row').length).toBe(3));
  return {app,realtime,...view};
}
const row=(subject:string):HTMLElement=>[...document.querySelectorAll<HTMLElement>('.list__row')].find(el=>el.textContent?.includes(subject))!;
const settle=():Promise<void>=>new Promise(r=>setTimeout(r,50));

describe('opening a message keeps focus in the list (t51)',()=>{
  for(const mode of ['right','bottom'] as ReadingPane[]){
    it(`reading pane ${mode}: click, Enter and arrows keep the row focused while the message tab shows the message`,async()=>{
      const {realtime}=await mount();setReadingPane(mode);
      const first=row('Subject a');first.focus();fireEvent.click(first);
      await waitFor(()=>expect(realtime.subTabs.activeId()).toBe('message:acct1:a'));
      await screen.findByRole('heading',{name:'Subject a'});await settle();
      expect(document.activeElement).toBe(first);
      await waitFor(()=>expect(screen.getByTestId('message-opened-status').textContent).toBe('Opened: Subject a'));
      fireEvent.keyDown(first,{key:'ArrowDown'});await settle();
      const second=row('Subject b');expect(document.activeElement).toBe(second);
      fireEvent.click(second);// Enter on a focused <button> is a click
      await waitFor(()=>expect(realtime.subTabs.activeId()).toBe('message:acct1:b'));
      await screen.findByRole('heading',{name:'Subject b'});await settle();
      expect(document.activeElement).toBe(second);
      // The `next` shortcut opens the following message and focus follows the cursor in the list.
      fireEvent.keyDown(document.body,{key:'j'});
      await waitFor(()=>expect(realtime.subTabs.activeId()).toBe('message:acct1:c'));await settle();
      expect(document.activeElement).toBe(row('Subject c'));
    });
  }
  it('a message tab that is already open: activating it again keeps the row focused',async()=>{
    const {realtime}=await mount();
    fireEvent.click(row('Subject a'));await screen.findByRole('heading',{name:'Subject a'});
    fireEvent.click(row('Subject b'));await screen.findByRole('heading',{name:'Subject b'});
    const first=row('Subject a');first.focus();fireEvent.click(first);
    await waitFor(()=>expect(realtime.subTabs.activeId()).toBe('message:acct1:a'));await settle();
    expect(document.activeElement).toBe(first);
  });
  it('reading pane off: the reader covers the list, so focus goes to its subject instead of <body>',async()=>{
    const {realtime}=await mount();setReadingPane('off');
    const first=row('Subject a');first.focus();fireEvent.click(first);
    await waitFor(()=>expect(realtime.subTabs.activeId()).toBe('message:acct1:a'));
    const heading=await screen.findByRole('heading',{name:'Subject a'});
    await waitFor(()=>expect(document.activeElement).toBe(heading));
    expect(screen.getByTestId('message-opened-status').textContent).toBe('');
  });
  // Folder tabs: the jsdom harness has no folder page (Email/query by mailbox),
  // so the companion pane is covered live in e2e/receipts/list-keyboard-live.spec.ts.
});
