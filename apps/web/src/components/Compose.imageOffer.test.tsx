import {structuredComposeEditor} from './compose/nativeHtmlTestHarness.ts';
import {describe,it,expect,beforeAll,beforeEach,vi} from 'vitest';
import {cleanup,render,screen,fireEvent,waitFor} from '@solidjs/testing-library';
import {Compose} from './Compose.tsx';
import {makeClient} from './appHarness.tsx';
import {createAppState} from '../state/store.ts';
import {AppContext} from '../state/context.ts';
import type {ComposeInitial,ComposeImageOffer} from './compose/reply.ts';

const base:ComposeInitial={mode:'reply',to:'a@example.org',cc:'',subject:'Re: x',bodyHtml:'<p></p><p>Alice wrote:</p><blockquote><p>original text</p></blockquote>',bodyText:'\n\nAlice wrote:\n> original text',inReplyTo:[],references:[],attachments:[],quotesDecrypted:false};
async function open(imageOffer:ComposeImageOffer){
 const app=createAppState(makeClient());await app.login({username:'me@example.org',password:'p'});
 render(()=><AppContext.Provider value={app}><Compose onClose={vi.fn()} initial={{...base,imageOffer}}/></AppContext.Provider>);
 await structuredComposeEditor();
}
function offer(over:Partial<ComposeImageOffer>={}):ComposeImageOffer{
 return {domain:'example.org',canTrust:true,include:vi.fn().mockResolvedValue({bodyHtml:base.bodyHtml,bodyText:base.bodyText,inline:[{cid:'r1@mailwoman',blobId:'b1',name:'a.png',type:'image/png',size:3}]}),trust:vi.fn().mockResolvedValue(undefined),...over};
}
describe('reply images prompt',()=>{
 beforeAll(async()=>{await import('./compose/RichTextEditor.tsx');},60_000);
 beforeEach(()=>{cleanup();localStorage.clear();});
 it('shows the three choices and fetches nothing until one is picked',async()=>{
  const o=offer();await open(o);
  expect(screen.getByTestId('compose-image-offer')).toBeInTheDocument();
  expect(screen.getByRole('button',{name:'Include images'})).toBeInTheDocument();
  expect(screen.getByRole('button',{name:'Not now'})).toBeInTheDocument();
  expect(screen.getByRole('button',{name:'Always trust example.org'})).toBeInTheDocument();
  expect(o.include).not.toHaveBeenCalled();
  fireEvent.click(screen.getByRole('button',{name:'Not now'}));
  expect(screen.queryByTestId('compose-image-offer')).toBeNull();expect(o.include).not.toHaveBeenCalled();expect(o.trust).not.toHaveBeenCalled();
 });
 it('Always trust remembers the domain and then includes the images',async()=>{
  const o=offer();await open(o);
  fireEvent.click(screen.getByRole('button',{name:'Always trust example.org'}));
  await waitFor(()=>expect(o.include).toHaveBeenCalledOnce());expect(o.trust).toHaveBeenCalledOnce();
  await waitFor(()=>expect(screen.queryByTestId('compose-image-offer')).toBeNull());
 });
 it('hides the trust choice when an administrator locks the setting',async()=>{
  await open(offer({canTrust:false}));
  expect(screen.queryByTestId('image-offer-trust')).toBeNull();
 });
 it('loading is cancelable and leaves a load-again placeholder',async()=>{
  let signal!:AbortSignal;
  const o=offer({include:vi.fn().mockImplementation((s:AbortSignal)=>{signal=s;return new Promise(()=>{});})});
  await open(o);
  fireEvent.click(screen.getByTestId('image-offer-include'));
  fireEvent.click(await screen.findByRole('button',{name:'Cancel'}));
  expect(signal.aborted).toBe(true);
  expect(screen.getByText(/Loading cancelled/)).toBeInTheDocument();
  expect(screen.getByRole('button',{name:'Load again'})).toBeInTheDocument();
 });
});
