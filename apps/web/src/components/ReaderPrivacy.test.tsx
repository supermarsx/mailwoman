import { beforeAll, beforeEach, afterEach, describe, expect, it, vi } from 'vitest';
import { render, screen, waitFor, fireEvent } from '@solidjs/testing-library';
import { readFileSync } from 'node:fs';
import { resolve } from 'node:path';
import { Reader } from './Reader.tsx';
import { makeClient, mkEmail } from './appHarness.tsx';
import { createAppState } from '../state/store.ts';
import { AppContext } from '../state/context.ts';
import { getCryptoWorker } from '../crypto/index.ts';
import { initSync, sanitizeEmailHtmlReader } from '../wasm/mw-sanitize/mw_sanitize.js';
import type { RemotePolicy, SignedContent } from '../api/remote-content.ts';
import type { Email } from '../api/jmap-types.ts';
// Unit tests inspect the prepared document; real browser tests exercise the
// opaque outer bridge and its script-free nested frame with actual CSP headers.
vi.mock('./DirectReaderFrame.tsx', () => ({ DirectReaderFrame: (props: {html:string;title:string}) => <iframe class="reader__frame" title={props.title} sandbox="" srcdoc={props.html} /> }));
beforeAll(() => initSync({ module: readFileSync(resolve('src/wasm/mw-sanitize/mw_sanitize_bg.wasm')) }));
const message = (id: string, html: string): Email => mkEmail(id, { from: [{ name: 'Sender', email:'sender@example.test' }], htmlBody: [{ partId:'1',blobId:null,size:html.length,type:'text/html' }], bodyValues: { '1':{value:html} } });
let policy: RemotePolicy; let granted: boolean; let signFailed: boolean; let requests: { emailId:string; urls?:string[]; parts?:string[]; files?:string[] }[];
let signOverride: ((body: typeof requests[number]) => Promise<Response>) | undefined;
const json = (body:unknown,status=200) => new Response(JSON.stringify(body), {status,headers:{'content-type':'application/json'}});
beforeEach(() => {
  policy = { mode:'proxy',trackers:true,cleanLinks:true,allowDirect:false,inlineParts:true,locked:{mode:false,trackers:false,cleanLinks:false} }; granted=false; signFailed=false;requests=[];signOverride=undefined;
  vi.spyOn(getCryptoWorker(),'sanitizeReader').mockImplementation(async r => sanitizeEmailHtmlReader(r.html,r.blockTrackers,r.cleanLinks));
  vi.stubGlobal('fetch',vi.fn(async(input: RequestInfo | URL,init?:RequestInit) => {
    const url=String(input);
    if(url.includes('/api/rc/stats?')) return json({decodedTrackersReplaced:2,limited:false,windowSecs:3600});
    if(url.endsWith('/api/rc/policy')) return json(policy);
    if(url.endsWith('/api/remote-images/grants')) return json({list:granted?[{scopeKind:'single',scopeValue:'m1',grantedAt:'2026-10-25T00:00:00Z'}]:[]});
    if(url.endsWith('/api/rc/sign')) {
      const body=JSON.parse(String(init?.body)) as typeof requests[number]; requests.push(body);
      if(signOverride)return signOverride(body);
      if(signFailed)return json({error:'refused'},403);
      return json({images:(body.urls??[]).map(url=>({url,src:`/api/rc/i/${body.emailId}.signature`})),parts:(body.parts??[]).map(blobId=>({blobId,src:'/api/rc/p/part.signature'})),files:[],refused:[]} satisfies SignedContent);
    }
    if(init?.body && String(init.body).includes('methodCalls')) {
      const body=JSON.parse(String(init.body)) as {methodCalls:[string,Record<string,unknown>,string][]};
      return json({sessionState:'s',methodResponses:body.methodCalls.map(([method,,tag])=>[method,{accountId:'acct1',state:'s',list:method==='Email/get'?[{id:'m1',mailwomanInlineParts:[{cid:'logo@test',blobId:'m1.2'}]}]:[],notFound:[]},tag])});
    }
    return json({},404);
  }));
});
afterEach(()=>{vi.restoreAllMocks();vi.unstubAllGlobals();});
async function open(emails:Email[]) {
  const client = makeClient({emails}); const original = client.jmap;
  client.jmap = async (request, options) => { const response = await original(request, options); const get = request.methodCalls.find(call => call[0] === 'Email/get'); const ids = get?.[1]['ids']; if (Array.isArray(ids)) for (const invocation of response.methodResponses) if (invocation[0] === 'Email/get') { const body = invocation[1] as {list:Email[]}; body.list = body.list.filter(email => ids.includes(email.id)); } return response; };
  const app=createAppState(client);render(()=> <AppContext.Provider value={app}><Reader /></AppContext.Provider>);
  await app.login({username:'me@example.org',password:'p'});await app.openMessage(emails[0]!.id);await screen.findByRole('toolbar',{name:'Message actions'});return app;
}
const frame=()=>document.querySelector<HTMLIFrameElement>('iframe.reader__frame');
describe('Reader signed privacy pipeline',()=>{
  it('loads proxy images through signed URLs and maps CID references to the message part',async()=>{
    await open([message('m1','<p>Private body</p><img src="https://cdn.example.test/image.png"><img src="cid:logo@test">')]);
    await waitFor(()=>expect(frame()?.srcdoc).toContain('/api/rc/i/m1.signature'));
    expect(frame()?.srcdoc).toContain('/api/rc/p/part.signature');expect(frame()?.getAttribute('sandbox')).toBe('');
    expect(requests).toContainEqual({emailId:'m1',urls:['https://cdn.example.test/image.png'],parts:['m1.2']});
    expect(JSON.stringify(requests)).not.toContain('Private body');expect(frame()?.srcdoc).not.toContain('/api/image-proxy');
  });
  it.each(['block','ask'] as const)('does not sign ungranted remote URLs in %s mode',async mode=>{
    policy.mode=mode;await open([message('m1','<img src="https://cdn.example.test/image.png">')]);
    await waitFor(()=>expect(frame()?.srcdoc).toContain('data:image/gif'));
    expect(requests).toEqual([]);
  });
  it('honors a message grant in ask mode',async()=>{
    policy.mode='ask';granted=true;await open([message('m1','<img src="https://cdn.example.test/image.png">')]);
    await waitFor(()=>expect(frame()?.srcdoc).toContain('/api/rc/i/m1.signature'));
  });
  it('keeps refused images blocked while retaining readable text',async()=>{
    signFailed=true;await open([message('m1','<p>Still readable</p><img src="https://cdn.example.test/image.png">')]);
    await waitFor(()=>expect(frame()?.srcdoc).toContain('Still readable'));
    expect(frame()?.srcdoc).not.toContain('/api/rc/i/');expect(await screen.findByRole('alert')).toHaveTextContent('blocked');
  });
  it('does not sign image URLs when no-media is selected during sanitization', async () => {
    let finish: ((html:string)=>void)|undefined;
    vi.mocked(getCryptoWorker().sanitizeReader).mockImplementationOnce(() => new Promise(resolve => { finish=resolve; }));
    const html='<p>Visible text</p><img src="https://cdn.example.test/image.png">';
    await open([message('m1',html)]); await waitFor(()=>expect(finish).toBeDefined());
    fireEvent.click(screen.getByTestId('max-security-opt-sanitized-no-media'));
    finish!(sanitizeEmailHtmlReader(html,false,true));
    await waitFor(()=>expect(frame()?.srcdoc).toContain('Visible text'));
    await new Promise(resolve=>setTimeout(resolve,0)); expect(requests).toEqual([]);
    expect(frame()?.srcdoc).not.toContain("img-src");
    fireEvent.click(screen.getByTestId('max-security-opt-full-sanitized'));
    await waitFor(()=>expect(frame()?.srcdoc).toContain('/api/rc/i/m1.signature'));
  });
  it('discards a delayed signing result after a different message opens',async()=>{
    let resolveOld: ((value:Response)=>void)|undefined;
    signOverride=body=>body.emailId==='m1'?new Promise(resolve=>{resolveOld=resolve;}):Promise.resolve(json({images:[],parts:[],files:[],refused:[]}));
    const app=await open([message('m1','<p>Old secret</p><img src="https://cdn.example.test/image.png">'),message('m2','<p>Current message</p>')]);
    await waitFor(()=>expect(resolveOld).toBeDefined());await app.openMessage('m2');
    await waitFor(()=>expect(frame()?.srcdoc).toContain('Current message'));
    resolveOld!(json({images:[{url:'https://cdn.example.test/image.png',src:'/api/rc/i/old.signature'}],parts:[],files:[],refused:[]}));
    await new Promise(resolve => setTimeout(resolve, 0));expect(frame()?.srcdoc).not.toContain('Old secret');expect(frame()?.srcdoc).not.toContain('old.signature');
  });
});

it('downloads an arbitrary message link only after explicit selection and action', async () => {
  await open([message('m1','<a href="https://files.test/download?id=7">a file</a>')]);
  const select = await screen.findByLabelText('Download another message link as a file');
  expect(requests).toEqual([]);
  fireEvent.change(select,{target:{value:'https://files.test/download?id=7'}});
  fireEvent.click(screen.getByRole('button',{name:'Download through the server'}));
  await waitFor(()=>expect(requests).toContainEqual({emailId:'m1',files:['https://files.test/download?id=7']}));
});
it('displays server decoded counts separately from HTML tracker markers',async()=>{
  await open([message('m1','<img src="https://cdn.test/image.png">')]);
  expect(await screen.findByText(/2 recently decoded tracking images blocked/)).toBeVisible();
});
