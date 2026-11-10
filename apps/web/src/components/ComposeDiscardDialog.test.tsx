import './compose/nativeHtmlTestHarness.ts';
// t42: draft discard and unsaved close confirm in one themed modal (the shared
// ConfirmDialog), never an inline bar. The modal opens in the composer's own
// document, so a detached workspace window shows it where the user is working.
import {afterEach,beforeAll,expect,it,vi} from 'vitest';
import {render,fireEvent,screen,waitFor,within} from '@solidjs/testing-library';
import {delegateEvents} from 'solid-js/web';
import {Compose} from './Compose.tsx';
import {makeClient} from './appHarness.tsx';
import {createAppState} from '../state/store.ts';
import {AppContext} from '../state/context.ts';
import {clearSettings,loadSettings} from '../api/settings.ts';
import {clearDrafts,listDrafts,saveDraft} from './compose/drafts-store.ts';
import {toast} from './toast/toast.ts';
import type {Me} from '../api/client.ts';

const json=(value:unknown)=>new Response(JSON.stringify(value),{headers:{'content-type':'application/json'}});
beforeAll(async()=>{await import('./compose/RichTextEditor.tsx');},60000);
afterEach(()=>{vi.unstubAllGlobals();clearSettings();clearDrafts();toast.clear();document.body.innerHTML='';});

async function signedIn(storage:boolean){
  vi.stubGlobal('fetch',vi.fn(async(input:RequestInfo|URL)=>String(input).endsWith('/api/account/drafts')?json({accountId:'acct1',supported:false,items:[]}):json({})));
  const client=makeClient();
  if(!storage){const me:Me={username:'me@example.org',accountId:'acct1'};vi.mocked(client.login).mockResolvedValue(me);vi.mocked(client.me).mockResolvedValue(me);}
  const app=createAppState(client);await app.login({username:'me@example.org',password:'p'});
  return app;
}

it('asks in a modal before closing a message that no draft would keep',async()=>{
  const app=await signedIn(false);let guard:(()=>Promise<boolean>)|undefined;const closed=vi.fn();
  render(()=><AppContext.Provider value={app}><Compose embedded onClose={closed} registerBeforeClose={handler=>{guard=handler;}}/></AppContext.Provider>);
  fireEvent.input(await screen.findByLabelText('Subject'),{target:{value:'Not stored'}});
  const opener=screen.getByLabelText('Subject');opener.focus();
  let answer=guard!();
  const dialog=await screen.findByRole('alertdialog',{name:'Discard unsaved changes?'});
  expect(dialog).toHaveAttribute('aria-modal','true');
  expect(dialog).toHaveAccessibleDescription(/cannot be saved in this session/);
  await waitFor(()=>expect(within(dialog).getByRole('button',{name:'Keep editing'})).toHaveFocus());
  expect(within(dialog).getByRole('button',{name:'Discard'})).toHaveClass('btn--danger');
  fireEvent.keyDown(dialog,{key:'Escape'});
  await expect(answer).resolves.toBe(false);
  expect(screen.queryByRole('alertdialog')).toBeNull();
  await waitFor(()=>expect(opener).toHaveFocus());
  answer=guard!();
  fireEvent.click(within(await screen.findByRole('alertdialog')).getByRole('button',{name:'Discard'}));
  await expect(answer).resolves.toBe(true);
});

it('closes without asking when a draft keeps the message',async()=>{
  const app=await signedIn(true);let guard:(()=>Promise<boolean>)|undefined;
  render(()=><AppContext.Provider value={app}><Compose embedded onClose={()=>{}} registerBeforeClose={handler=>{guard=handler;}}/></AppContext.Provider>);
  fireEvent.input(await screen.findByLabelText('Subject'),{target:{value:'Kept locally'}});
  await expect(guard!()).resolves.toBe(true);
  expect(screen.queryByRole('alertdialog')).toBeNull();
});

it('opens the discard modal in the document that holds the composer (detached window)',async()=>{
  const app=await signedIn(true);
  const frame=document.createElement('iframe');document.body.append(frame);const popup=frame.contentDocument!;delegateEvents(['click','keydown','input'],popup);
  const container=popup.createElement('main');popup.body.append(container);
  render(()=><AppContext.Provider value={app}><Compose embedded onClose={()=>{}}/></AppContext.Provider>,{container});
  const discard=await within(container).findByRole('button',{name:'Discard'});
  // Cross-realm nodes: jest-dom matchers reject them, so read the DOM directly.
  expect(discard.hasAttribute('data-detached-local')).toBe(true);
  fireEvent.click(discard);
  const dialog=await within(popup.body).findByRole('alertdialog',{name:'Discard this draft?'});
  expect(dialog.ownerDocument).toBe(popup);
  await waitFor(()=>expect(popup.activeElement?.textContent).toBe('Keep editing'));
  expect(document.querySelector('[role="alertdialog"]')).toBeNull();
  fireEvent.click(within(dialog).getByRole('button',{name:'Keep editing'}));
  expect(popup.querySelector('[role="alertdialog"]')).toBeNull();
});

it.each(['saving','uncertain','conflict','refused'] as const)('closes a resumed draft during %s and retains the latest local edits',async(status)=>{
 const app=await signedIn(true);
 saveDraft({id:'existing-draft',accountId:'acct1',storageAuthority:'test-storage-authority-acct1',serverRevision:'old-revision',to:'',subject:'Saved before',bodyText:'',bodyHtml:'',bodyFormat:'plain',savedAt:1});
 let started=false;
 vi.stubGlobal('fetch',vi.fn(async(input:RequestInfo|URL,init?:RequestInit)=>{
  const path=String(input);
  if(path.endsWith('/api/account/drafts'))return json({accountId:'acct1',storageAuthority:'test-storage-authority-acct1',supported:true,items:[]});
  if(init?.method==='PUT'){
   started=true;
   if(status==='saving')return new Promise<Response>((_resolve,reject)=>{init.signal?.addEventListener('abort',()=>reject(new DOMException('Aborted','AbortError')),{once:true});});
   return new Response(JSON.stringify({code:status==='conflict'?'draftConflict':status==='refused'?'draftSaveRefused':'draftSaveUncertain'}),{status:409,headers:{'content-type':'application/json'}});
  }
  return json({csrfToken:'csrf'});
 }));
 await loadSettings({refresh:true,fetcher:async()=>json({settings:{'mail.draftSaveMode':{value:'manual',source:'user',locked:false}}})});
 let guard:(()=>Promise<boolean>)|undefined;
 const view=render(()=><AppContext.Provider value={app}><Compose embedded resumeDraftId="existing-draft" onClose={()=>{}} registerBeforeClose={handler=>{guard=handler;}}/></AppContext.Provider>);
 await waitFor(()=>expect(screen.getByLabelText('Subject')).toHaveValue('Saved before'));
 fireEvent.input(screen.getByLabelText('Subject'),{target:{value:'Latest edits'}});
 const actions=within(view.container.querySelector<HTMLElement>('.compose__header')!);
 await waitFor(()=>expect(actions.getByRole('button',{name:'Save to Drafts'})).toBeEnabled());
 fireEvent.click(actions.getByRole('button',{name:'Save to Drafts'}));
 await waitFor(()=>expect(started).toBe(true));
 if(status!=='saving')await waitFor(()=>expect(screen.getByTestId('account-draft-status')).not.toHaveTextContent('Saving to account Drafts'));
 await expect(guard!()).resolves.toBe(true);
 const recovered=listDrafts('acct1','test-storage-authority-acct1').find(draft=>draft.id==='existing-draft');
 expect(recovered?.subject).toBe('Latest edits');
 if(status!=='refused')expect(recovered?.serverStatus).toBe('uncertain');
 expect(screen.queryByRole('alertdialog')).toBeNull();
 view.unmount();
});
