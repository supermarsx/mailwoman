import {afterEach,beforeAll,expect,it,vi} from 'vitest';
import {render,fireEvent,screen,waitFor,within} from '@solidjs/testing-library';
import {Compose} from './Compose.tsx';
import {makeClient} from './appHarness.tsx';
import {createAppState} from '../state/store.ts';
import {AppContext} from '../state/context.ts';
import {clearSettings,loadSettings} from '../api/settings.ts';
import {toast} from './toast/toast.ts';
const json=(value:unknown)=>new Response(JSON.stringify(value),{headers:{'content-type':'application/json'}});
beforeAll(async()=>{await import('./compose/RichTextEditor.tsx');},60000);
afterEach(()=>{vi.unstubAllGlobals();clearSettings();toast.clear();});
it.each(['revision-one','revision-two','switch-draft'])('saves with captured authority and refuses discard of a changed remote revision (%s)',async(loadedRevision)=>{
 const closed=vi.fn();const removals:unknown[]=[];let releaseLoad:(()=>void)|undefined;
 const writes:Record<string,unknown>[]=[];let state:Record<string,unknown>|undefined;
 const fetcher=vi.fn(async(input:RequestInfo|URL,init?:RequestInit)=>{
  const path=String(input);
  if(path.endsWith('/api/session'))return json({csrfToken:'csrf'});
  if(path.endsWith('/api/account/drafts'))return json({accountId:'acct1',storageAuthority:'test-storage-authority-acct1',supported:true,items:loadedRevision==='switch-draft'?[{id:'other-draft',revision:'other-revision',status:'saved',summary:{subject:'Other draft'}}]:[]});
  if(path.includes('/api/account/drafts/')&&init?.method==='DELETE'){removals.push(JSON.parse(String(init.body)));return json({accountId:'acct1',storageAuthority:'test-storage-authority-acct1',deleted:true});}
  if(path.includes('/api/account/drafts/')&&init?.method==='GET'){
   const other=path.endsWith('/other-draft');
   if(loadedRevision==='switch-draft'&&!other)await new Promise<void>(resolve=>{releaseLoad=resolve;});
   return json({accountId:'acct1',storageAuthority:'test-storage-authority-acct1',id:path.split('/').at(-1),revision:other?'other-revision':loadedRevision==='switch-draft'?'revision-one':loadedRevision,emailId:'upstream-message',savedAt:1,status:'saved',cleanupPending:false,summary:{subject:other?'Other draft':state?.subject},state:other?{...state,subject:'Other draft'}:state});
  }
  if(path.includes('/api/account/drafts/')&&init?.method==='PUT'){
   const request=JSON.parse(String(init.body)) as {state:Record<string,unknown>};writes.push(request);state=request.state;
   return json({accountId:'acct1',storageAuthority:'test-storage-authority-acct1',id:path.split('/').at(-1),revision:'revision-one',emailId:'upstream-message',savedAt:1,status:'saved',cleanupPending:false,summary:{subject:state.subject},state});
  }
  return json({});
 });
 vi.stubGlobal('fetch',fetcher);
 const app=createAppState(makeClient());await app.login({username:'me@example.org',password:'p'});
 await loadSettings({refresh:true,fetcher:async()=>json({settings:{'mail.draftSaveMode':{value:'manual',source:'user',locked:false}}})});
 const success=vi.spyOn(toast,'success');render(()=><AppContext.Provider value={app}><Compose embedded onClose={closed}/></AppContext.Provider>);
 fireEvent.input(await screen.findByLabelText('To'),{target:{value:'unfinished'}});
 fireEvent.input(screen.getByLabelText('Subject'),{target:{value:'Remote editable'}});
 await waitFor(()=>expect(screen.getByRole('button',{name:'Save to Drafts'})).toBeEnabled());
 fireEvent.click(screen.getByRole('button',{name:'Save to Drafts'}));
 await waitFor(()=>expect(screen.getByTestId('account-draft-status')).toHaveTextContent('Saved to account Drafts'));
 expect(writes).toHaveLength(1);expect(state?.to).toBe('unfinished');expect(state?.subject).toBe('Remote editable');expect(success).not.toHaveBeenCalled();
 const saveCall=fetcher.mock.calls.find(call=>String(call[0]).includes('/api/account/drafts/')&&call[1]?.method==='PUT');
 expect(new Headers(saveCall?.[1]?.headers).get('x-mailwoman-storage-authority')).toBe('test-storage-authority-acct1');success.mockRestore();
 fireEvent.click(screen.getByRole('button',{name:'Discard account draft'}));
 expect(removals).toHaveLength(0);expect(closed).not.toHaveBeenCalled();expect(screen.getByRole('button',{name:'Send'})).toBeDisabled();
 // A themed modal (t42): labelled alertdialog, the safe choice focused, Esc cancels.
 const dialog=await screen.findByRole('alertdialog',{name:'Discard this draft?'});
 expect(dialog).toHaveAttribute('aria-modal','true');expect(dialog).toHaveAccessibleDescription(/Remote editable/);
 await waitFor(()=>expect(screen.getByRole('button',{name:'Keep editing'})).toHaveFocus());
 fireEvent.keyDown(dialog,{key:'Escape'});expect(screen.queryByRole('alertdialog')).toBeNull();expect(closed).not.toHaveBeenCalled();
 fireEvent.click(screen.getByRole('button',{name:'Discard account draft'}));
 fireEvent.click(screen.getByRole('button',{name:'Keep editing'}));expect(closed).not.toHaveBeenCalled();
 fireEvent.click(screen.getByRole('button',{name:'Discard account draft'}));
 const failure=vi.spyOn(toast,'error');
 fireEvent.click(within(screen.getByRole('alertdialog')).getByRole('button',{name:'Discard'}));
 if(loadedRevision==='switch-draft'){
  await waitFor(()=>expect(releaseLoad).toBeDefined());
  fireEvent.click(screen.getByRole('button',{name:/^Drafts$/}));
  await waitFor(()=>expect(screen.getByRole('button',{name:'Other draft'})).toBeEnabled());fireEvent.click(screen.getByRole('button',{name:'Other draft'}));
  await waitFor(()=>expect(screen.getByLabelText('Subject')).toHaveValue('Other draft'));
  releaseLoad!();
  await waitFor(()=>expect(screen.getByRole('button',{name:'Send'})).toBeEnabled());
  expect(removals).toHaveLength(0);expect(closed).not.toHaveBeenCalled();
  expect(screen.getByLabelText('Subject')).toHaveValue('Other draft');failure.mockRestore();return;
 }
 if(loadedRevision!=='revision-one'){
  await waitFor(()=>expect(failure).toHaveBeenCalled());
  expect(removals).toHaveLength(0);expect(closed).not.toHaveBeenCalled();
  expect(screen.getByLabelText('Subject')).toHaveValue('Remote editable');
  expect(screen.getByRole('button',{name:'Send'})).toBeEnabled();
  failure.mockRestore();return;
 }
 failure.mockRestore();
 await waitFor(()=>expect(closed).toHaveBeenCalledTimes(1));
 expect(removals).toEqual([{expectedRevision:'revision-one'}]);
 const removal=fetcher.mock.calls.find(call=>call[1]?.method==='DELETE');
 expect(new Headers(removal?.[1]?.headers).get('x-mailwoman-storage-authority')).toBe('test-storage-authority-acct1');

});
