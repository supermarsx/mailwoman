// t68: the /api/rc/frame bridge (crates/mw-server/src/remote-frame.js) takes a
// body only from the app origin, with its token, and only from the window that
// holds it or that window's opener (a detached workspace window is rendered by
// the main window). Another same-origin window or a cross-origin sender holding
// the token is refused.
import { readFileSync } from 'node:fs';
import { resolve } from 'node:path';
import { expect, it } from 'vitest';

const source = readFileSync(resolve(__dirname, '../../../../crates/mw-server/src/remote-frame.js'), 'utf8');
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
  const frame = (): Record<string, unknown> => {
    const element: Record<string, unknown> = { srcdoc: '', style: {}, setAttribute(name:string,value:string) {if(name==='sandbox')sandboxes.push(value);}, addEventListener() {},
      cloneNode: () => frame(), replaceWith: (next: { srcdoc: string }) => { replaced.push(next.srcdoc); } };
    return element;
  };
  const document = { referrer: '', createElement: frame, body: { append() {} } };
  const location = { hash: `#token=${TOKEN}&origin=${encodeURIComponent(ORIGIN)}`, origin: ORIGIN, search: '' };
  new Function('location', 'parent', 'document', 'addEventListener', source)(location, parent, document,
    (type: string, handler: typeof listener) => { if (type === 'message') listener = handler; });
  const send = (from: unknown, data: unknown, origin = ORIGIN) => listener!({ origin, source: from, data });
  return { parent, posted, replaced, send, sandboxes };
}
const body = (html = '<p>hi</p>') => ({ type: 'mw-reader-frame', token: TOKEN, html, revision: 1 });

it('announces itself to its parent and takes a body from the parent', () => {
  const frame = bridge();
  expect(frame.posted).toEqual([{ data: { type: 'mw-reader-ready', token: TOKEN }, origin: ORIGIN }]);
  frame.send(frame.parent, body());
  expect(frame.replaced).toEqual(['<p>hi</p>']);
});

it('takes a body from the opener of a detached window, and answers its ping', () => {
  const main = {};
  const frame = bridge({ opener: main });
  frame.send(main, { type: 'mw-reader-ping', token: TOKEN });
  expect(frame.posted).toHaveLength(2);
  frame.send(main, body('<p>detached</p>'));
  expect(frame.replaced).toEqual(['<p>detached</p>']);
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
    expect(frame.replaced).toEqual(['<p>hi</p>']);
  }
});

it('permits only popups after authenticated explicit consent and revokes them on the next document',()=>{
 const frame=bridge();expect(frame.sandboxes).toEqual(['']);
 frame.send({}, {...body(),linksEnabled:true});expect(frame.sandboxes).toEqual(['']);
 frame.send(frame.parent,{...body(),linksEnabled:true});expect(frame.sandboxes.at(-1)).toBe('allow-popups allow-popups-to-escape-sandbox');
 frame.send(frame.parent,{...body(),linksEnabled:'true'});expect(frame.sandboxes.at(-1)).toBe('');
 frame.send(frame.parent,body());expect(frame.sandboxes.at(-1)).toBe('');
 expect(frame.sandboxes.every(tokens=>!tokens.includes('allow-scripts')&&!tokens.includes('allow-same-origin')&&!tokens.includes('allow-top-navigation'))).toBe(true);
});
