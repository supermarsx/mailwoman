import {afterEach,beforeEach,describe,expect,it,vi} from 'vitest';
import {fireEvent,render,screen,waitFor} from '@solidjs/testing-library';
import {Compose} from './Compose.tsx';
import {makeClient} from './appHarness.tsx';
import {createAppState} from '../state/store.ts';
import {AppContext} from '../state/context.ts';
import {clearSettings,loadSettings} from '../api/settings.ts';
import {chooseComposeFormat} from './compose/nativeHtmlTestHarness.ts';

// mail.allowSend off: the composer cannot send. The Send button is disabled with
// the feature-off tooltip, and a submit that bypasses the disabled button sends nothing.

const featureSettings=(values:Record<string,boolean>)=>loadSettings({refresh:true,fetcher:async()=>new Response(JSON.stringify({settings:Object.fromEntries(Object.entries(values).map(([key,value])=>[key,{value,source:'global',locked:true}]))}),{headers:{'content-type':'application/json'}})});

async function mount(){
  const client=makeClient();const app=createAppState(client);
  await app.login({username:'me@example.org',password:'p'});
  const onClose=vi.fn();
  const view=render(()=><AppContext.Provider value={app}><Compose onClose={onClose}/></AppContext.Provider>);
  await chooseComposeFormat('plain');
  fireEvent.input(screen.getByLabelText('To'),{target:{value:'you@example.org'}});
  fireEvent.input(screen.getByLabelText('Subject'),{target:{value:'Hi'}});
  fireEvent.input(screen.getByLabelText('Body'),{target:{value:'typed body'}});
  return {client,app,onClose,view};
}
const sends=(client:ReturnType<typeof makeClient>)=>vi.mocked(client.jmap).mock.calls.filter(call=>call[0].methodCalls.some(method=>method[0]==='EmailSubmission/set'&&'create' in method[1])).length;

describe('Compose and mail.allowSend',()=>{
  beforeEach(()=>{clearSettings();localStorage.clear();});
  afterEach(()=>clearSettings());

  it('sends normally by default',async()=>{
    const {client,onClose}=await mount();
    expect(screen.getByRole('button',{name:'Send'})).toBeEnabled();
    fireEvent.click(screen.getByRole('button',{name:'Send'}));
    await waitFor(()=>expect(onClose).toHaveBeenCalledTimes(1));
    expect(sends(client)).toBe(1);
  });

  it('disables Send, explains why, and sends nothing when the setting is off',async()=>{
    const {client,onClose}=await mount();
    await featureSettings({'mail.allowSend':false});
    const send=await screen.findByRole('button',{name:'Send'});
    await waitFor(()=>expect(send).toBeDisabled());
    expect(send).toHaveAttribute('title');
    fireEvent.submit(send.closest('form')!);
    await new Promise(resolve=>setTimeout(resolve,50));
    expect(sends(client)).toBe(0);
    expect(onClose).not.toHaveBeenCalled();
  });
});
