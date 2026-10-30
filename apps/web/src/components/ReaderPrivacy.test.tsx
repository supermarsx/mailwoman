import { beforeAll, beforeEach, afterEach, describe, expect, it, vi } from 'vitest';
import { render, screen, waitFor, fireEvent } from '@solidjs/testing-library';
import { readFileSync } from 'node:fs';
import { resolve } from 'node:path';
import { Suspense } from 'solid-js';
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
vi.mock('./DirectReaderFrame.tsx', () => ({ DirectReaderFrame: (props: {html:string;title:string;documentKey?:string;onDisplayed?:(html:string,key:string|undefined)=>void}) => { setTimeout(() => props.onDisplayed?.(props.html, props.documentKey), 0); return <iframe class="reader__frame" title={props.title} sandbox="" srcdoc={props.html} />; } }));
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
    fireEvent.click(screen.getByTestId('max-security-switch'));fireEvent.click(screen.getByRole('menuitemcheckbox',{name:'No media'}));
    finish!(sanitizeEmailHtmlReader(html,false,true));
    await waitFor(()=>expect(frame()?.srcdoc).toContain('Visible text'));
    await new Promise(resolve=>setTimeout(resolve,0)); expect(requests).toEqual([]);
    expect(frame()?.srcdoc).not.toContain("img-src");
    fireEvent.click(screen.getByTestId('max-security-switch'));fireEvent.click(screen.getByRole('menuitemcheckbox',{name:'Full'}));
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
  // t110: the linked-files bar; its menu lists the link, nothing is fetched before a choice.
  const show = await screen.findByRole('button',{name:'Download a link'});
  expect(screen.getByTestId('linked-files-bar')).toHaveTextContent('Links in this message');
  expect(requests).toEqual([]);
  fireEvent.click(show);
  expect(requests).toEqual([]);
  fireEvent.click(await screen.findByRole('menuitem',{name:/Download .*files\.test\/download/}));
  await waitFor(()=>expect(requests).toContainEqual({emailId:'m1',files:['https://files.test/download?id=7']}));
});
it('folds the server decoded count into the banner, read once, never as a separate line',async()=>{
  await open([message('m1','<img src="https://cdn.test/image.png">')]);
  await waitFor(()=>expect(screen.getByTestId('remote-content-bar')).toHaveTextContent('Images loaded through the server · 2 trackers removed'),{timeout:5000});
  const stats=vi.mocked(fetch).mock.calls.filter(([u])=>String(u).includes('/api/rc/stats?'));
  expect(stats).toHaveLength(1);
  expect(screen.queryByText(/recently decoded/)).toBeNull();
});

// t110: every notice above the body is the same bar, in one stack and a fixed
// order (list, images, files, attachments); attachments collapse in place.
describe('Reader notice bars',()=>{
  it('stacks the remote-images, linked-files and attachments bars on one frame, in order',async()=>{
    policy.mode='ask';
    const html='<p>Report</p><img src="https://cdn.example.test/image.png"><a href="https://files.example.test/q3/report%20final.pdf">report</a>';
    const email={...message('m1',html),hasAttachment:true,attachments:[{partId:'2',blobId:'a1',name:'notes.txt',type:'text/plain',size:2048},{partId:'3',blobId:'a2',name:'plan.pdf',type:'application/pdf',size:1024*1024}]} as Email;
    await open([email]);
    const stack=await screen.findByTestId('reader-notice-stack');
    await waitFor(()=>expect([...stack.querySelectorAll('[data-notice-bar]')].map(bar=>bar.getAttribute('data-testid'))).toEqual(['remote-content-bar','linked-files-bar','reader-attachments-bar']));
    const bars=[...stack.querySelectorAll<HTMLElement>('[data-notice-bar]')];
    expect(new Set(bars.map(bar=>bar.className)).size).toBe(1);
    expect(screen.getByTestId('linked-files-bar')).toHaveTextContent(/One file linked in this message · \u2068files\.example\.test\u2069/);
    fireEvent.click(screen.getByRole('button',{name:'Show files'}));
    expect(screen.getByRole('menuitem',{name:/Download \u2068report final\.pdf\u2069 from \u2068files\.example\.test\u2069/})).toBeInTheDocument();
    expect(requests.some(r=>r.files)).toBe(false);
    const attachments=screen.getByTestId('reader-attachments-bar');
    expect(attachments).toHaveTextContent(/^2 attachments · 1 MB/);
    const toggle=screen.getByTestId('reader-attachments-toggle');
    expect(toggle).toHaveAttribute('aria-expanded','true');expect(screen.getByTestId('reader-attachments').parentElement).not.toHaveAttribute('hidden');
    fireEvent.click(toggle);
    expect(toggle).toHaveAttribute('aria-expanded','false');expect(toggle).toHaveAccessibleName('Show attachments');
    expect(screen.getByTestId('reader-attachments').parentElement).toHaveAttribute('hidden');
    expect(document.getElementById(toggle.getAttribute('aria-controls')!)).toBe(screen.getByTestId('reader-attachments').parentElement);
  });
});

// t60: the mailbox route's <Suspense> wraps the whole shell. A reader that reads
// a loading resource directly (here the remote-content policy, for the banner)
// suspends it: the shell is swapped for its fallback and back, and every open
// message frame reloads (blank bodies, remote images fetched twice).
describe('Reader under the mailbox Suspense',()=>{
  it('never suspends the shell while the remote-content policy loads',async()=>{
    // A standing grant shows the remote-content bar before the policy arrives.
    policy.mode='ask';granted=true;
    let release:()=>void=()=>{};const gate=new Promise<void>(resolve=>{release=resolve;});
    const base=vi.mocked(fetch).getMockImplementation()!;
    vi.mocked(fetch).mockImplementation(async(input,init)=>{if(String(input).endsWith('/api/rc/policy'))await gate;return base(input,init);});
    const emails=[message('m1','<p>Body text</p><img src="https://cdn.example.test/image.png">')];
    const app=createAppState(makeClient({emails}));
    let fallbacks=0;const Fallback=()=>{fallbacks+=1;return <p>Shell loading</p>;};
    render(()=><Suspense fallback={<Fallback/>}><AppContext.Provider value={app}><p>Shell</p><Reader /></AppContext.Provider></Suspense>);
    await app.login({username:'me@example.org',password:'p'});await app.openMessage('m1');
    await screen.findByRole('toolbar',{name:'Message actions'});
    await new Promise(resolve=>setTimeout(resolve,50));
    expect(screen.getByText('Shell')).toBeInTheDocument();
    release();
    await waitFor(()=>expect(frame()?.srcdoc).toContain('Body text'));
    expect(fallbacks).toBe(0);
  });
});

// t79: a zoom step stores reader.zoom, which replaces the settings map. Neither
// the level nor that write may rebuild the body: the policy is not asked for
// again, nothing is re-signed, the frame keeps its element and its document.
describe('Reader body across zoom steps',()=>{
  it('keeps the same frame and document, and asks for nothing again',async()=>{
    const { setReaderZoom, resetZoom, readerZoom } = await import('./readerZoom.ts');
    const { settings } = await import('../api/settings.ts');
    let policyReads=0;const puts:string[]=[];
    const base=vi.mocked(fetch).getMockImplementation()!;
    vi.mocked(fetch).mockImplementation(async(input,init)=>{
      const url=String(input);
      if(url.endsWith('/api/rc/policy'))policyReads++;
      if(url.includes('/api/settings/')&&init?.method==='PUT'){puts.push(url);return json({value:JSON.parse(String(init.body)).value,source:'user',locked:false});}
      return base(input,init);
    });
    await open([message('m1','<p>Zoom body</p><img src="https://cdn.example.test/image.png">')]);
    await waitFor(()=>expect(frame()?.srcdoc).toContain('/api/rc/i/m1.signature'));
    await new Promise(resolve=>setTimeout(resolve,20));
    const element=frame()!;const doc=element.srcdoc;const reads=policyReads;const signs=requests.length;
    expect(doc).not.toContain('zoom');
    for(const level of [125,150,200,90]){
      setReaderZoom(level);expect(readerZoom()).toBe(level);
      await new Promise(resolve=>setTimeout(resolve,20));
      expect(frame()).toBe(element);expect(frame()!.srcdoc).toBe(doc);
    }
    await waitFor(()=>expect(settings()['reader.zoom']?.value).toBe(90));
    await new Promise(resolve=>setTimeout(resolve,20));
    expect(puts.length).toBeGreaterThan(0);
    expect(frame()).toBe(element);expect(frame()!.srcdoc).toBe(doc);
    expect(policyReads-reads,'policy fetched again').toBe(0);
    expect(requests.length-signs,'content signed again').toBe(0);
    resetZoom();
  });
});
