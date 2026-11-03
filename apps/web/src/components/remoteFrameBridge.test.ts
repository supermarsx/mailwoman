// t68: the /api/rc/frame bridge (crates/mw-server/src/remote-frame.js) takes a
// body only from the app origin, with its token, and only from the window that
// holds it or that window's opener (a detached workspace window is rendered by
// the main window). Another same-origin window or a cross-origin sender holding
// the token is refused.
import { readFileSync } from 'node:fs';
import { resolve } from 'node:path';
import { expect, it, vi } from 'vitest';

const handler = readFileSync(resolve(__dirname, '../../../../crates/mw-server/src/reader-address-handler.js'), 'utf8').replace(/\r\n?/g,'\n');
const source = readFileSync(resolve(__dirname, '../../../../crates/mw-server/src/remote-frame.js'), 'utf8').replace('__MW_PARENT_ORIGIN__','null').replace('__MW_ADDRESS_HANDLER_SOURCE__',JSON.stringify(handler)).replace('__MW_ADDRESS_HANDLER_HASH__',JSON.stringify('fixed-handler-hash'));
const ORIGIN = 'https://mail.example.org';
const TOKEN = 'abcdefghijklmnop1234';

interface Posted { data: { type: string; token: string; revision?: number }; origin: string }
function bridge(options: { opener?: object | null; openerThrows?: boolean } = {}) {
  const posted: Posted[] = [];
  const parent: Record<string, unknown> = { postMessage: (data: Posted['data'], origin: string) => posted.push({ data, origin }) };
  if (options.openerThrows) Object.defineProperty(parent, 'opener', { get() { throw new Error('SecurityError'); } });
  else parent['opener'] = options.opener ?? null;
  let listener: ((event: { origin: string; source: unknown; data: unknown }) => void) | undefined;
  const replaced: string[] = [];
  const sandboxes:string[]=[];
  const frames:Record<string,unknown>[]=[];
  const channels:{port1:{onmessage:((event:{data:unknown})=>void)|null;start:ReturnType<typeof vi.fn>;close:ReturnType<typeof vi.fn>};port2:object}[]=[];
  class Channel {port1={onmessage:null as ((event:{data:unknown})=>void)|null,start:vi.fn(),close:vi.fn()};port2={};constructor(){channels.push(this);}}
  const frame = (): Record<string, unknown> => {
    const element: Record<string, unknown> = { srcdoc: '', style: {},contentWindow:{postMessage:vi.fn()},clientWidth:500,clientHeight:400,getBoundingClientRect:()=>({left:0,top:0}), setAttribute(name:string,value:string) {if(name==='sandbox')sandboxes.push(value);}, addEventListener() {},
      cloneNode: () => frame(), replaceWith: (next: { srcdoc: string }) => { replaced.push(next.srcdoc); } };
    frames.push(element);
    return element;
  };
  const document = { referrer: '', createElement: frame, body: { append() {} } };
  const location = { hash: `#token=${TOKEN}&origin=${encodeURIComponent(ORIGIN)}`, origin: ORIGIN, search: '' };
  new Function('location', 'parent', 'document', 'addEventListener','MessageChannel', source)(location, parent, document,
    (type: string, handler: typeof listener) => { if (type === 'message') listener = handler; },Channel);
  const send = (from: unknown, data: unknown, origin = ORIGIN) => listener!({ origin, source: from, data });
  return { parent, posted, replaced, send, sandboxes,frames,channels };
}
const body = (html = '<p>hi</p>') => ({ type: 'mw-reader-frame', token: TOKEN, html, revision: 1 });

it('forwards image progress only from the authenticated current child with consistent bounded counts',()=>{
 const frame=bridge();frame.send(frame.parent,body());
 const child=frame.frames.at(-1)!.contentWindow as {postMessage:ReturnType<typeof vi.fn>};frame.send(child,{type:'mw-reader-address-ready'},'null');
 const init=child.postMessage.mock.calls[0]![0] as {nonce:string;revision:number};const port=frame.channels[0]!.port1;
 const value={...init,type:'image-progress',loaded:2,failed:1,pending:3,total:6};const before=frame.posted.length;
 for(const data of [{...value,nonce:'wrong'},{...value,revision:2},{...value,total:7},{...value,loaded:-1},{...value,pending:NaN},{...value,failed:0.5},{...value,total:1_000_001}])port.onmessage!({data});
 expect(frame.posted).toHaveLength(before);port.onmessage!({data:value});expect(frame.posted.at(-1)).toEqual({data:{type:'mw-reader-image-progress',token:TOKEN,revision:1,loaded:2,failed:1,pending:3,total:6},origin:ORIGIN});
 frame.send(frame.parent,{...body(),revision:2});const count=frame.posted.length;port.onmessage!({data:value});expect(frame.posted).toHaveLength(count);
});

it('announces itself to its parent and takes a body from the parent', () => {
  const frame = bridge();
  expect(frame.posted).toEqual([{ data: { type: 'mw-reader-ready', token: TOKEN }, origin: ORIGIN }]);
  frame.send(frame.parent, body());
  expect(frame.replaced).toHaveLength(1);expect(frame.replaced[0]).toContain('<p>hi</p>');
});

it('takes a body from the opener of a detached window, and answers its ping', () => {
  const main = {};
  const frame = bridge({ opener: main });
  frame.send(main, { type: 'mw-reader-ping', token: TOKEN });
  expect(frame.posted).toHaveLength(2);
  frame.send(main, body('<p>detached</p>'));
  expect(frame.replaced).toHaveLength(1);expect(frame.replaced[0]).toContain('<p>detached</p>');
});

it('refuses another same-origin window, a wrong token and a cross-origin sender', () => {
  const main = {}, stranger = {};
  const frame = bridge({ opener: main });
  frame.send(stranger, body('<p>stranger</p>'));
  frame.send(stranger, { type: 'mw-reader-ping', token: TOKEN });
  frame.send(null, body('<p>no source</p>'));
  frame.send(main, { ...body('<p>bad token</p>'), token: 'zzzzzzzzzzzzzzzzzzzz' });
  frame.send(main, body('<p>cross origin</p>'), 'https://evil.example');
  frame.send(frame.parent, body('<p>cross origin parent</p>'), 'null');
  expect(frame.replaced).toEqual([]);
  expect(frame.posted).toHaveLength(1);
});

it('refuses everything but its parent when it has no readable opener', () => {
  const stranger = {};
  for (const frame of [bridge({ opener: null }), bridge({ openerThrows: true })]) {
    frame.send(stranger, body('<p>stranger</p>'));
    frame.send(frame.parent, body());
    expect(frame.replaced).toHaveLength(1);expect(frame.replaced[0]).toContain('<p>hi</p>');
  }
});

it('permits only popups after authenticated explicit consent and revokes them on the next document',()=>{
 const frame=bridge();expect(frame.sandboxes).toEqual(['']);
 frame.send({}, {...body(),linksEnabled:true});expect(frame.sandboxes).toEqual(['']);
 frame.send(frame.parent,{...body(),linksEnabled:true});expect(frame.sandboxes.at(-1)).toBe('allow-scripts allow-popups allow-popups-to-escape-sandbox');
 frame.send(frame.parent,{...body(),revision:2,linksEnabled:'true'});expect(frame.sandboxes.at(-1)).toBe('allow-scripts');
 frame.send(frame.parent,{...body(),revision:3});expect(frame.sandboxes.at(-1)).toBe('allow-scripts');
 expect(frame.sandboxes.every(tokens=>!tokens.includes('allow-same-origin')&&!tokens.includes('allow-top-navigation'))).toBe(true);
});

it('preserves layout but removes sender scripts/handlers and replaces permissive script policies with only the fixed hash',()=>{
 const frame=bridge();
 frame.send(frame.parent,body('<!doctype html><meta http-equiv="Content-Security-Policy" content="default-src \'none\'; img-src data:; script-src * \'unsafe-inline\'; script-src-elem *"><style>.logo{padding:9px}</style><table class="logo" onclick="evil()"><tr><td>A</td></tr></table><script>evil()</script>'));
 const doc=new DOMParser().parseFromString(frame.replaced[0]!, 'text/html');
 expect(doc.querySelector('table')?.className).toBe('logo');expect(doc.querySelector('style')?.textContent).toBe('.logo{padding:9px}');
 expect(doc.querySelector('[onclick]')).toBeNull();expect(doc.querySelectorAll('script')).toHaveLength(1);expect(doc.querySelector('script')?.textContent).toBe(handler);
 expect(doc.querySelector('meta')?.getAttribute('content')).toBe("default-src 'none'; img-src data:; script-src 'sha256-fixed-handler-hash'; script-src-attr 'none'");
});

it('binds address events to the exact opaque child, fresh document channel, nonce and revision, and closes replaced ports',()=>{
 const bridgeFrame=bridge();bridgeFrame.send(bridgeFrame.parent,body());
 const child=bridgeFrame.frames.at(-1)!.contentWindow as {postMessage:ReturnType<typeof vi.fn>};
 bridgeFrame.send({}, {type:'mw-reader-address-ready'},'null');expect(bridgeFrame.channels).toHaveLength(0);
 bridgeFrame.send(child,{type:'mw-reader-address-ready'},ORIGIN);expect(bridgeFrame.channels).toHaveLength(0);
 bridgeFrame.send(child,{type:'mw-reader-address-ready'},'null');expect(bridgeFrame.channels).toHaveLength(1);
 const init=child.postMessage.mock.calls[0]![0] as {nonce:string;revision:number};
 const port=bridgeFrame.channels[0]!.port1;
 const value={type:'address-context',...init,address:'A@example.org',x:20,y:30};value.type='address-context';
 const before=bridgeFrame.posted.length;
 for(const data of [{...value,nonce:'wrong'},{...value,revision:2},{...value,x:501},{...value,y:NaN}])port.onmessage!({data});
 expect(bridgeFrame.posted).toHaveLength(before);
 port.onmessage!({data:value});expect(bridgeFrame.posted.at(-1)?.data).toMatchObject({type:'mw-reader-address-context',address:'A@example.org',revision:1,x:20,y:30});
 bridgeFrame.send(bridgeFrame.parent,{...body(),revision:2});expect(port.close).toHaveBeenCalledOnce();
 const count=bridgeFrame.posted.length;port.onmessage!({data:value});expect(bridgeFrame.posted).toHaveLength(count);
});

it('accepts parsed-body readiness only through the current authenticated channel and acknowledges each document once',()=>{
 const frame=bridge();frame.send(frame.parent,body());
 const child=frame.frames.at(-1)!.contentWindow as {postMessage:ReturnType<typeof vi.fn>};
 frame.send(child,{type:'mw-reader-address-ready'},'null');
 const init=child.postMessage.mock.calls[0]![0] as {nonce:string;revision:number};const port=frame.channels[0]!.port1;
 const parsed={...init,type:'displayed'};
 port.onmessage!({data:{...parsed,nonce:'wrong'}});port.onmessage!({data:{...parsed,revision:2}});
 expect(frame.posted.filter(value=>value.data.type==='mw-reader-displayed')).toHaveLength(0);
 port.onmessage!({data:parsed});port.onmessage!({data:parsed});
 expect(frame.posted.filter(value=>value.data.type==='mw-reader-displayed')).toEqual([{data:{type:'mw-reader-displayed',token:TOKEN,revision:1},origin:ORIGIN}]);
 frame.send(frame.parent,{...body(),revision:2});port.onmessage!({data:parsed});
 expect(frame.posted.filter(value=>value.data.type==='mw-reader-displayed')).toHaveLength(1);
});
