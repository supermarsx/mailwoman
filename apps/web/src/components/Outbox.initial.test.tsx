import {Show} from 'solid-js';
import {afterEach,expect,it,vi} from 'vitest';
import {fireEvent,render,screen,waitFor} from '@solidjs/testing-library';
import {Outbox} from './Outbox.tsx';
import {AppContext} from '../state/context.ts';
import {createAppState,type AppState} from '../state/store.ts';
import {makeClient} from './appHarness.tsx';
import {suspendAccountPreferences} from '../api/accountPreferenceLifecycle.ts';

afterEach(()=>{suspendAccountPreferences();vi.unstubAllGlobals();localStorage.clear();});

it.each(['login','init'] as const)('loads the first Outbox check when mounted by cold %s',async action=>{
  suspendAccountPreferences();
  vi.stubGlobal('fetch',vi.fn(async()=>new Response('{}',{status:404,headers:{'content-type':'application/json'}})));
  const client=makeClient();let app!:AppState;
  render(()=>{
    app=createAppState(client);
    return <AppContext.Provider value={app}><Show when={app.me()}><Outbox/></Show></AppContext.Provider>;
  });
  if(action==='login')await app.login({username:'me@example.org',password:'p'});else await app.init();
  await waitFor(()=>expect(vi.mocked(client.jmap).mock.calls.flatMap(([request])=>request.methodCalls).filter(call=>call[0]==='EmailSubmission/query')).toHaveLength(1));
  expect(app.outboxLoading()).toBe(false);
});

it('reports a failed first check and lets Refresh recover without claiming the Outbox is empty',async()=>{
  suspendAccountPreferences();
  vi.stubGlobal('fetch',vi.fn(async()=>new Response('{}',{status:404,headers:{'content-type':'application/json'}})));
  const client=makeClient();const jmap=vi.mocked(client.jmap).getMockImplementation()!;let fail=true;let app!:AppState;
  vi.mocked(client.jmap).mockImplementation(async(request,options)=>{
    if(fail&&request.methodCalls.some(call=>call[0]==='EmailSubmission/query'))throw new TypeError('Failed to fetch');
    return jmap(request,options);
  });
  render(()=>{
    app=createAppState(client);
    return <AppContext.Provider value={app}><Show when={app.me()}><Outbox/></Show></AppContext.Provider>;
  });
  await app.login({username:'me@example.org',password:'p'});
  expect(await screen.findByText('Outbox unavailable. Use Refresh to try again.')).toBeVisible();
  expect(screen.queryByText('Nothing waiting to send')).not.toBeInTheDocument();
  expect(app.toast()?.message).toContain('The Outbox could not be checked');
  fail=false;fireEvent.click(screen.getByRole('button',{name:'Refresh'}));
  expect(await screen.findByText('Nothing waiting to send')).toBeVisible();
});
