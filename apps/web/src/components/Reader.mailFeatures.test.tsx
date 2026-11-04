import {afterEach,beforeAll,beforeEach,describe,expect,it,vi} from 'vitest';
import {cleanup,fireEvent,render,screen,waitFor,within} from '@solidjs/testing-library';
import {readFileSync} from 'node:fs';
import {resolve} from 'node:path';
import {Reader} from './Reader.tsx';
import {makeClient,mkEmail} from './appHarness.tsx';
import {createAppState} from '../state/store.ts';
import {AppContext} from '../state/context.ts';
import {clearSettings,loadSettings} from '../api/settings.ts';
import {initSync} from '../wasm/mw-sanitize/mw_sanitize.js';
import type {ComposeInitial} from './compose/reply.ts';

beforeAll(()=>initSync({module:readFileSync(resolve('src/wasm/mw-sanitize/mw_sanitize_bg.wasm'))}));
vi.mock('./security/SpamScore.tsx',()=>({SpamScore:()=>null}));
vi.mock('./security/DeliveryStatus.tsx',()=>({DeliveryStatus:()=>null}));

// The reader's Reply / Reply all / Forward / Download / Print actions follow the
// mail.allow* settings (the toolbar's own item list is covered by
// ReaderToolbar.mailFeatures.test.tsx; this runs the whole Reader).

const EMAIL=mkEmail('m1',{blobId:'m1',subject:'Plans',htmlBody:[{partId:'1',blobId:null,size:0,type:'text/html'}],bodyValues:{'1':{value:'<p>Body</p>'}}});
const apply=(values:Record<string,boolean>)=>loadSettings({refresh:true,fetcher:async()=>new Response(JSON.stringify({settings:Object.fromEntries(Object.entries(values).map(([key,value])=>[key,{value,source:'global',locked:true}]))}),{headers:{'content-type':'application/json'}})});

async function open(values:Record<string,boolean>){
  const client=makeClient({emails:[EMAIL]});
  const app=createAppState(client);const onCompose=vi.fn<(initial:ComposeInitial)=>void>();
  render(()=><AppContext.Provider value={app}><Reader onCompose={onCompose}/></AppContext.Provider>);
  await app.login({username:'me@example.org',password:'p'});
  await apply(values);
  await app.openMessage('m1');
  const toolbar=await screen.findByRole('toolbar',{name:'Message actions'});
  return {app,onCompose,toolbar};
}

describe('Reader and the mail feature settings',()=>{
  beforeEach(()=>{clearSettings();vi.stubGlobal('fetch',vi.fn(async()=>new Response('{}',{status:404})));});
  afterEach(()=>{cleanup();clearSettings();vi.unstubAllGlobals();});

  it('offers Reply, Reply all and Forward by default and hands the composer a reply',async()=>{
    const {toolbar,onCompose}=await open({});
    for(const name of ['Reply','Reply all','Forward'])expect(within(toolbar).getByRole('button',{name})).toBeInTheDocument();
    fireEvent.click(within(toolbar).getByRole('button',{name:'Reply'}));
    await waitFor(()=>expect(onCompose).toHaveBeenCalledTimes(1));
  });

  it('removes Reply and Reply all when mail.allowReply is off, keeping Forward usable',async()=>{
    const {toolbar,onCompose}=await open({'mail.allowReply':false});
    expect(within(toolbar).queryByRole('button',{name:'Reply'})).toBeNull();
    expect(within(toolbar).queryByRole('button',{name:'Reply all'})).toBeNull();
    fireEvent.click(within(toolbar).getByRole('button',{name:'Forward'}));
    await waitFor(()=>expect(onCompose).toHaveBeenCalledTimes(1));
    expect((onCompose.mock.calls[0]![0] as ComposeInitial).mode).toBe('forward');
  });

  it('removes Forward when mail.allowForward is off, keeping Reply',async()=>{
    const {toolbar}=await open({'mail.allowForward':false});
    expect(within(toolbar).queryByRole('button',{name:'Forward'})).toBeNull();
    expect(within(toolbar).getByRole('button',{name:'Reply'})).toBeInTheDocument();
  });

  it('removes Print and message export when mail.allowPrint and mail.allowDownload are off',async()=>{
    const {toolbar}=await open({});
    const present=(id:string)=>toolbar.querySelector(`[data-reader-item="${id}"]`)!==null;
    expect(present('print')).toBe(true);expect(present('export')).toBe(true);
    cleanup();
    const off=await open({'mail.allowPrint':false,'mail.allowDownload':false});
    const gone=(id:string)=>off.toolbar.querySelector(`[data-reader-item="${id}"]`)===null;
    expect(gone('print')).toBe(true);expect(gone('export')).toBe(true);
  });
});
