import {afterEach,beforeAll,expect,it,vi} from 'vitest';
import {fireEvent,screen,waitFor} from '@solidjs/testing-library';
import {Compose} from './Compose.tsx';
import {renderWithApp} from './appHarness.tsx';
import {clearSettings,loadSettings} from '../api/settings.ts';
import {toast} from './toast/toast.ts';
const json=(value:unknown)=>new Response(JSON.stringify(value),{headers:{'content-type':'application/json'}});
beforeAll(async()=>{await import('./compose/RichTextEditor.tsx');},60000);
afterEach(()=>{vi.unstubAllGlobals();clearSettings();toast.clear();});
it('saves partial recipients to account Drafts quietly with the captured authority',async()=>{
 const writes:Record<string,unknown>[]=[];let state:Record<string,unknown>|undefined;
 const fetcher=vi.fn(async(input:RequestInfo|URL,init?:RequestInit)=>{
  const path=String(input);
  if(path.endsWith('/api/session'))return json({csrfToken:'csrf'});
  if(path.endsWith('/api/account/drafts'))return json({accountId:'acct1',storageAuthority:'test-storage-authority-acct1',supported:true,items:[]});
  if(path.includes('/api/account/drafts/')&&init?.method==='PUT'){
   const request=JSON.parse(String(init.body)) as {state:Record<string,unknown>};writes.push(request);state=request.state;
   return json({accountId:'acct1',storageAuthority:'test-storage-authority-acct1',id:path.split('/').at(-1),revision:'revision-one',emailId:'upstream-message',savedAt:1,status:'saved',cleanupPending:false,summary:{subject:state.subject},state});
  }
  return json({});
 });
 vi.stubGlobal('fetch',fetcher);
 await loadSettings({refresh:true,fetcher:async()=>json({settings:{'mail.draftSaveMode':{value:'manual',source:'user',locked:false}}})});
 const success=vi.spyOn(toast,'success');const {app}=renderWithApp(()=><Compose onClose={()=>undefined}/>);
 await app.login({username:'me@example.org',password:'p'});
 fireEvent.input(await screen.findByLabelText('To'),{target:{value:'unfinished'}});
 fireEvent.input(screen.getByLabelText('Subject'),{target:{value:'Remote editable'}});
 await waitFor(()=>expect(screen.getByRole('button',{name:'Save to Drafts'})).toBeEnabled());
 fireEvent.click(screen.getByRole('button',{name:'Save to Drafts'}));
 await waitFor(()=>expect(screen.getByTestId('account-draft-status')).toHaveTextContent('Saved to account Drafts'));
 expect(writes).toHaveLength(1);expect(state?.to).toBe('unfinished');expect(state?.subject).toBe('Remote editable');expect(success).not.toHaveBeenCalled();
 const saveCall=fetcher.mock.calls.find(call=>String(call[0]).includes('/api/account/drafts/')&&call[1]?.method==='PUT');
 expect(new Headers(saveCall?.[1]?.headers).get('x-mailwoman-storage-authority')).toBe('test-storage-authority-acct1');success.mockRestore();
});
