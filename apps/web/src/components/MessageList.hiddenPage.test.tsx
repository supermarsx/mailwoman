import {afterEach,expect,it,vi} from 'vitest';
import {cleanup,render,screen,waitFor} from '@solidjs/testing-library';
import {MessageList} from './MessageList.tsx';
import {makeClient,mkEmail} from './appHarness.tsx';
import {createAppState} from '../state/store.ts';
import {AppContext} from '../state/context.ts';
import {clearSettings,loadSettings,type Fetcher} from '../api/settings.ts';

vi.mock('./initialSync.ts',()=>({createInitialSync:()=>({pending:()=>false,running:()=>false,failed:()=>false,status:()=> 'ready',retry:()=>undefined})}));
afterEach(()=>{cleanup();clearSettings();});
const json=(body:unknown):Response=>new Response(JSON.stringify(body),{status:200,headers:{'content-type':'application/json'}});

// A page whose every message is expired is hidden in `hide` mode. With more pages
// behind it the list is still loading, not an empty folder: it used to say
// "No messages" while the next pages were being fetched.
it('hide-expired with a fully hidden first page shows the loading skeleton while more pages exist, not "No messages"',async()=>{
 await loadSettings({refresh:true,fetcher:(async()=>json({settings:{'mail.expiry.hideExpired':{value:'hide',source:'user',locked:false}}}))as Fetcher});
 const old='2001-01-01T00:00:00Z';
 const app=createAppState(makeClient({emails:[mkEmail('a',{subject:'Gone A','header:Expires:asText':old}),mkEmail('b',{subject:'Gone B','header:Expires:asText':old})]}));
 await app.login({username:'me@example.org',password:'p'});
 await waitFor(()=>expect(app.messages().length).toBe(2));
 const state={...app,hasMore:()=>true,loadMoreFailed:()=>false};
 render(()=><AppContext.Provider value={state}><MessageList/></AppContext.Provider>);
 expect(screen.queryByText('Gone A')).toBeNull();
 expect(screen.queryByText(/no messages/i)).toBeNull();
 expect(document.querySelector('[data-skeleton]')??document.querySelector('[aria-busy="true"]')).not.toBeNull();
});

it('hide-expired with nothing more to load is the empty folder',async()=>{
 await loadSettings({refresh:true,fetcher:(async()=>json({settings:{'mail.expiry.hideExpired':{value:'hide',source:'user',locked:false}}}))as Fetcher});
 const app=createAppState(makeClient({emails:[mkEmail('a',{subject:'Gone A','header:Expires:asText':'2001-01-01T00:00:00Z'})]}));
 await app.login({username:'me@example.org',password:'p'});
 await waitFor(()=>expect(app.messages().length).toBe(1));
 render(()=><AppContext.Provider value={{...app,hasMore:()=>false}}><MessageList/></AppContext.Provider>);
 expect(screen.queryByText('Gone A')).toBeNull();
 await waitFor(()=>expect(screen.getByText(/^No messages/)).toBeInTheDocument());
});
